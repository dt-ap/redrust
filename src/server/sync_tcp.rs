use std::io;

use std::net::{Shutdown, TcpListener};

use crate::data::store::Store;
use crate::{
    config::Config,
    core::{cmd::Command, eval, resp::decode_array_string},
    error::EOFError,
};

pub trait Stream: io::Write + io::Read {}
impl<T> Stream for T where T: io::Write + io::Read {}

pub fn read_command(stream: &mut impl Stream) -> anyhow::Result<Command> {
    let mut buf: [u8; 512] = [0u8; 512];

    let bytes = stream.read(&mut buf)?;
    if bytes == 0 {
        return Err(EOFError.into());
    }

    let tokens = decode_array_string(&buf[..bytes])?;

    return Ok(Command {
        cmd: tokens[0].to_uppercase(),
        args: tokens[1..].to_vec(),
    });
}

fn respond_error(err: anyhow::Error, stream: &mut impl Stream) -> io::Result<()> {
    return stream.write(format!("-{}\r\n", err).as_bytes()).and(Ok(()));
}

pub fn respond(cmd: Command, store: &mut Store, stream: &mut impl Stream) -> io::Result<()> {
    return match eval::respond(cmd, store, stream) {
        Ok(_) => Ok(()),
        Err(err) => respond_error(err, stream),
    };
}

pub fn run(conf: Config) -> io::Result<()> {
    println!(
        "Starting a synchronous TCP Server on {0}:{1}",
        conf.host, conf.port
    );

    let mut store = Store::new();

    let mut con_clients = 0u8;

    let listener = TcpListener::bind(format!("{0}:{1}", conf.host, conf.port))?;

    loop {
        let (mut stream, _) = match listener.accept() {
            Ok(s) => s,
            Err(err) => {
                println!("Err: {:?}", err);
                continue;
            }
        };

        con_clients += 1;

        loop {
            let cmd = match read_command(&mut stream) {
                Ok(res) => res,
                Err(err) => {
                    stream.shutdown(Shutdown::Both)?;

                    con_clients -= 1;

                    if err.is::<EOFError>() {
                        break;
                    }

                    println!("Err {}", err);
                    return Ok(());
                }
            };

            respond(cmd, &mut store, &mut stream)?;
        }
    }
}
