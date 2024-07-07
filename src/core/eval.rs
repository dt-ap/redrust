use std::{io::Write, string::String};

use anyhow::anyhow;
use chrono::Utc;

use crate::core::{cmd::Command, resp::encode};

use crate::common::Value;

use crate::data::store::{Store, StoreValue};

use super::resp::{RESP_NIL, RESP_OK};

fn ping(args: Vec<String>, stream: &mut impl Write) -> anyhow::Result<()> {
    if args.len() >= 2 {
        return Err(anyhow!("ERR wrong number of arguments for 'ping' commands"));
    }

    let b = if args.len() == 0 {
        encode(Value::String("PONG".to_owned()), true)
    } else {
        encode(Value::String(args[0].clone()), false)
    };

    stream.write(&b)?;
    return Ok(());
}

pub fn get(args: Vec<String>, store: &mut Store, stream: &mut impl Write) -> anyhow::Result<()> {
    if args.len() != 1 {
        return Err(anyhow!("ERR wrong number of arguments for 'get' commands"));
    }

    let key = &args[0];

    let resp: &[u8] = match store.get(key.to_string()) {
        Some(s) => {
            if s.expires_at != -1 && s.expires_at <= Utc::now().timestamp_millis() {
                RESP_NIL
            } else {
                &encode(s.value.clone(), false)
            }
        }
        None => RESP_NIL,
    };

    stream.write(resp)?;
    return Ok(());
}

pub fn set(args: Vec<String>, store: &mut Store, stream: &mut impl Write) -> anyhow::Result<()> {
    if args.len() <= 1 {
        return Err(anyhow!("ERR wrong number of arguments for 'set' commands"));
    }

    let key = &args[0];
    let value = Value::String(args[1].clone());
    let mut exp_duration_ms = -1_i64;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "EX" | "ex" => {
                i += 1;
                if i == args.len() {
                    return Err(anyhow!("ERR syntax error"));
                }

                let exp_duration_s: i64 = args[3]
                    .parse()
                    .map_err(|_| anyhow!("ERR value is not an integer or out of range"))?;

                exp_duration_ms = exp_duration_s * 1_000;
            }
            _ => return Err(anyhow!("ERR syntax error")),
        }
        i += 1;
    }

    store.put(key.to_owned(), StoreValue::new(value, exp_duration_ms));
    stream.write(RESP_OK)?;

    return Ok(());
}

pub fn ttl(args: Vec<String>, store: &mut Store, stream: &mut impl Write) -> anyhow::Result<()> {
    if args.len() != 1 {
        return Err(anyhow!("ERR wrong number of arguments for 'ttl' commands"));
    }

    let key = &args[0];

    let val = store.get(key.to_string());

    if let Some(s) = val {
        if s.expires_at == -1 {
            stream.write(":-1\r\n".as_bytes())?; // Exist, but no expiration is set
            return Ok(());
        }

        let duration_ms = s.expires_at - Utc::now().timestamp_millis();

        let resp = if duration_ms < 0 {
            ":-2\r\n".as_bytes() // Expired
        } else {
            &encode(Value::Int64(duration_ms / 1_000), false)
        };

        stream.write(resp)?;
    } else {
        stream.write(":-2\r\n".as_bytes())?; // Key does not exist
    }

    return Ok(());
}

pub fn respond(cmd: Command, store: &mut Store, stream: &mut impl Write) -> anyhow::Result<()> {
    return match cmd.cmd.as_str() {
        "PING" => ping(cmd.args, stream),
        "SET" => set(cmd.args, store, stream),
        "GET" => get(cmd.args, store, stream),
        "TTL" => ttl(cmd.args, store, stream),
        _ => ping(cmd.args, stream),
    };
}
