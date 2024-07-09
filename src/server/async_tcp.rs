use std::{
    io,
    mem::{size_of, MaybeUninit},
    net::SocketAddrV4,
    os::fd::RawFd,
};

use chrono::{TimeDelta, Utc};
use libc;

use crate::{
    config::Config,
    core::comm::FdComm,
    data::store::Store,
    server::sync_tcp::{read_command, respond},
    syscall,
};

fn set_nonblocking(fd: RawFd, nonblocking: bool) -> io::Result<()> {
    let flag = syscall!(fcntl(fd, libc::F_GETFL))?;

    let new_flag = if nonblocking {
        flag | libc::O_NONBLOCK
    } else {
        flag | &!libc::O_NONBLOCK
    };

    if flag != new_flag {
        syscall!(fcntl(fd, libc::F_SETFL, new_flag))?;
    }

    return Ok(());
}

fn accept(fd: RawFd) -> io::Result<i32> {
    let mut addr: MaybeUninit<libc::sockaddr_storage> = MaybeUninit::uninit();
    let mut length = size_of::<libc::sockaddr_storage>() as libc::socklen_t;

    return syscall!(accept4(
        fd,
        addr.as_mut_ptr() as *mut _,
        &mut length,
        libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
    ));
}

pub fn run(conf: Config) -> anyhow::Result<()> {
    println!(
        "Starting an asynchronous TCP Server on {0}:{1}",
        conf.host, conf.port
    );
    let mut store = Store::new(conf.clone());
    let mut con_clients = 0;

    let max_clients = 20000;
    let mut events = Vec::<libc::epoll_event>::with_capacity(max_clients);

    let cron_frequency = TimeDelta::seconds(1);
    let mut last_cron_exec_time = Utc::now();

    let server_fd = syscall!(socket(
        libc::AF_INET,
        libc::O_NONBLOCK | libc::SOCK_STREAM | libc::SOCK_CLOEXEC,
        0
    ))?;
    set_nonblocking(server_fd, true)?;

    let ip4: SocketAddrV4 = format!("{}:{}", conf.host, conf.port).parse()?;
    let sockaddr_in = libc::sockaddr_in {
        sin_family: libc::AF_INET as libc::sa_family_t,
        sin_addr: libc::in_addr {
            s_addr: u32::from_ne_bytes(ip4.ip().octets()),
        },
        sin_port: ip4.port().to_be(),
        sin_zero: [0; 8],
    };

    syscall!(bind(
        server_fd,
        &sockaddr_in as *const _ as *const libc::sockaddr,
        size_of::<libc::sockaddr_in>() as libc::socklen_t
    ))?;

    syscall!(listen(server_fd, max_clients as i32))?;

    // let listener = TcpListener::bind(format!("{0}:{1}", conf.host, conf.port))?;
    // listener.set_nonblocking(true)?;
    // let server_fd = listener.as_raw_fd();

    // AsyncIO starts here!!

    let epoll_fd = syscall!(epoll_create1(libc::EPOLL_CLOEXEC))?;
    let mut socket_server_event = libc::epoll_event {
        events: libc::EPOLLIN as u32,
        u64: server_fd as u64,
    };

    // Listen to read events on server itself
    syscall!(epoll_ctl(
        epoll_fd,
        libc::EPOLL_CTL_ADD,
        server_fd,
        &mut socket_server_event
    ))?;

    loop {
        if Utc::now() > last_cron_exec_time + cron_frequency {
            store.delete_expired_keys();
            last_cron_exec_time = Utc::now();
        }

        events.clear();
        let n_events = match syscall!(epoll_wait(
            epoll_fd,
            events.as_mut_ptr(),
            max_clients as i32,
            -1
        )) {
            Ok(res) => res,
            Err(_) => continue,
        };
        unsafe { events.set_len(n_events as usize) };

        for ev in events.iter() {
            // If socket server itself is ready for an IO

            if ev.u64 == server_fd as u64 {
                let fd = match accept(server_fd) {
                    Ok(res) => res,
                    Err(err) => {
                        println!("Accept err: {:?}", err);
                        return Err(err.into());
                    }
                };
                set_nonblocking(fd, true)?;

                con_clients += 1;

                // Add this new TCP connection to be monitored
                let mut socket_client_event = libc::epoll_event {
                    events: libc::EPOLLIN as u32,
                    u64: fd as u64,
                };

                match syscall!(epoll_ctl(
                    epoll_fd,
                    libc::EPOLL_CTL_ADD,
                    fd,
                    &mut socket_client_event
                )) {
                    Ok(_) => (),
                    Err(err) => {
                        println!("{:?}", err);
                    }
                };
            } else {
                let mut comm = FdComm { fd: ev.u64 as i32 };
                let cmds = match read_command(&mut comm) {
                    Ok(res) => res,
                    Err(_) => {
                        syscall!(close(ev.u64 as i32))?;
                        con_clients -= 1;
                        continue;
                    }
                };
                respond(cmds, &mut store, &mut comm)?;
            }
        }
    }
}
