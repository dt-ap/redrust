use std::{io::Write, string::String};

use anyhow::anyhow;

use super::{
    cmd::Command,
    resp::{encode, Value},
};

pub fn ping(args: Vec<String>, stream: &mut impl Write) -> anyhow::Result<()> {
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

pub fn respond(cmd: Command, stream: &mut impl Write) -> anyhow::Result<()> {
    return match cmd.cmd.as_str() {
        "PING" => ping(cmd.args.clone(), stream),
        _ => ping(cmd.args.clone(), stream),
    };
}
