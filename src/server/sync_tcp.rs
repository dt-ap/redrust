use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
};

use crate::{
    config::Config,
    core::{cmd::Command, eval, resp::decode_array_string},
    error::EOFError,
};

fn read_command(mut stream: &TcpStream) -> anyhow::Result<Command> {
    let mut buf = [0u8; 512];

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

fn respond_error(err: anyhow::Error, mut stream: &TcpStream) -> io::Result<()> {
    return stream.write(format!("-{}\r\n", err).as_bytes()).and(Ok(()));
}

fn respond(cmd: Command, stream: &TcpStream) -> io::Result<()> {
    return match eval::respond(cmd, stream) {
        Ok(_) => Ok(()),
        Err(err) => respond_error(err, stream),
    };
}

pub fn run(conf: Config) -> io::Result<()> {
    println!(
        "Starting a synchronous TCP Server on {0}:{1}",
        conf.host, conf.port
    );

    let mut con_clients = 0u8;

    let listener = TcpListener::bind(format!("{0}:{1}", conf.host, conf.port))?;

    loop {
        let (stream, _) = match listener.accept() {
            Ok(s) => s,
            Err(err) => panic!("{:?}", err),
        };

        let address = stream.peer_addr()?;

        con_clients += 1;
        println!(
            "Client connected with address: {0} concurrent clients {1}",
            address, con_clients
        );

        loop {
            let cmd = match read_command(&stream) {
                Ok(res) => res,
                Err(err) => {
                    stream.shutdown(Shutdown::Both)?;

                    con_clients -= 1;
                    println!(
                        "Client disconnected {0} concurrent clients {1}",
                        address, con_clients
                    );

                    if err.is::<EOFError>() {
                        break;
                    }

                    println!("Err {}", err);
                    return Ok(());
                }
            };

            respond(cmd, &stream)?;
        }
    }
}
