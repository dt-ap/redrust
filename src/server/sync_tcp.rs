use anyhow::anyhow;

use std::io;
use std::net::{Shutdown, TcpListener};

use crate::common::Value;
use crate::core::cmd::Commands;
use crate::data::store::Store;
use crate::{
    config::Config,
    core::{cmd::Command, eval, resp::decode},
    error::EOFError,
};

pub trait Stream: io::Write + io::Read {}
impl<T> Stream for T where T: io::Write + io::Read {}

fn to_array_string(values: Vec<Value>) -> anyhow::Result<Vec<String>> {
    let mut arrs = Vec::<String>::with_capacity(values.len());
    for v in values {
        arrs.push(v.to_string());
    }

    return Ok(arrs);
}

pub fn read_command(stream: &mut impl Stream) -> anyhow::Result<Commands> {
    let mut buf: [u8; 512] = [0u8; 512];

    let bytes = stream.read(&mut buf)?;
    if bytes == 0 {
        return Err(EOFError.into());
    }

    let values = decode(&buf[..bytes])?;
    let mut cmds = Commands::with_capacity(values.len());

    for val in values {
        if let Value::Vector(v) = val {
            let tokens = to_array_string(v)?;

            cmds.push(Command {
                cmd: tokens[0].to_uppercase(),
                args: tokens[1..].to_vec(),
            })
        } else {
            return Err(anyhow!("Value is not a Vec type"));
        }
    }

    return Ok(cmds);
}

fn respond_error(err: anyhow::Error, stream: &mut impl Stream) -> io::Result<()> {
    return stream.write(format!("-{}\r\n", err).as_bytes()).and(Ok(()));
}

pub fn respond(cmds: Commands, store: &mut Store, stream: &mut impl Stream) -> io::Result<()> {
    return eval::respond(cmds, store, stream);
}

pub fn run(conf: Config) -> io::Result<()> {
    println!(
        "Starting a synchronous TCP Server on {0}:{1}",
        conf.host, conf.port
    );

    let mut store = Store::new(conf.clone());

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
            let cmds = match read_command(&mut stream) {
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

            respond(cmds, &mut store, &mut stream)?;
        }
    }
}
