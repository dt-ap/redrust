use std::{io::Write, net::TcpStream, string::String};

use anyhow::anyhow;

use super::{
    cmd::Command,
    resp::{encode, Value},
};

pub fn ping(args: Vec<String>, mut stream: &TcpStream) -> anyhow::Result<()> {
    if args.len() >= 2 {
        return Err(anyhow!("ERR wrong number of arguments for 'ping' commands"));
    }

    let b: Vec<u8>;
    if args.len() == 0 {
        b = encode(Value::String("PONG".to_owned()), true);
    } else {
        b = encode(Value::String(args[0].clone()), false);
    }

    stream.write(&b)?;
    return Ok(());
}

pub fn respond(cmd: Command, stream: &TcpStream) -> anyhow::Result<()> {
    println!("Command: {}", cmd.cmd);

    return match cmd.cmd.as_str() {
        "PING" => ping(cmd.args.clone(), stream),
        _ => ping(cmd.args.clone(), stream),
    };
}
