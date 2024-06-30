use std::{
    error::Error,
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    result::Result,
};

use crate::{config::Config, error::EOFError};

fn read_command(mut stream: &TcpStream) -> Result<String, Box<dyn Error>> {
    let mut buf = [0u8; 512];

    let bytes = stream.read(&mut buf)?;
    if bytes == 0 {
        return Err(Box::new(EOFError));
    }
    let res = String::from_utf8(buf[..bytes].to_vec())?;

    return Ok(res);
}

fn respond(cmd: String, mut stream: &TcpStream) -> io::Result<()> {
    return stream.write(cmd.as_bytes()).and(Ok(()));
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

            println!("Command {}", cmd);
            respond(cmd, &stream).unwrap_or_else(|err| {
                println!("Err write: {}", err);
            });
        }
    }
}
