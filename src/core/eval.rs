use std::io::{self, BufWriter};
use std::{io::Write, string::String};

use anyhow::anyhow;
use chrono::Utc;

use crate::core::{cmd::Command, cmd::Commands, resp::encode};

use crate::common::Value;

use crate::data::store::{deduce_type_encoding, Store, StoreObject, ENCODING_INT, TYPE_STRING};

use super::resp::{
    encode_error, RESP_MINUS_ONE, RESP_MINUS_TWO, RESP_NIL, RESP_OK, RESP_ONE, RESP_ZERO,
};

fn ping(args: Vec<String>) -> Vec<u8> {
    if args.len() >= 2 {
        return encode_error(anyhow!("ERR wrong number of arguments for 'ping' commands"));
    }

    return if args.len() == 0 {
        encode(Value::String("PONG".to_owned()), true)
    } else {
        encode(Value::String(args[0].clone()), false)
    };
}

pub fn get(args: Vec<String>, store: &mut Store) -> Vec<u8> {
    if args.len() != 1 {
        return encode_error(anyhow!("ERR wrong number of arguments for 'get' commands"));
    }

    let key = &args[0];

    return match store.get(key) {
        Some(s) => {
            if s.expires_at != -1 && s.expires_at <= Utc::now().timestamp_millis() {
                RESP_NIL.to_vec()
            } else {
                encode(s.value.clone(), false)
            }
        }
        None => RESP_NIL.to_vec(),
    };
}

pub fn set(args: Vec<String>, store: &mut Store) -> Vec<u8> {
    if args.len() <= 1 {
        return encode_error(anyhow!("ERR wrong number of arguments for 'set' commands"));
    }

    let key = &args[0];
    let (obj_type, obj_encoding) = deduce_type_encoding(&args[1]);
    let value = Value::String(args[1].clone());
    let mut exp_duration_ms = -1_i64;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "EX" | "ex" => {
                i += 1;
                if i == args.len() {
                    return encode_error(anyhow!("ERR syntax error"));
                }

                let exp_duration_s: i64 = match args[3].parse() {
                    Ok(res) => res,
                    Err(_) => {
                        return encode_error(anyhow!("ERR value is not an integer or out of range"))
                    }
                };

                exp_duration_ms = exp_duration_s * 1_000;
            }
            _ => return encode_error(anyhow!("ERR syntax error")),
        }
        i += 1;
    }

    store.put(
        key.to_owned(),
        StoreObject::new(value, exp_duration_ms, obj_type, obj_encoding),
    );
    return RESP_OK.to_vec();
}

pub fn ttl(args: Vec<String>, store: &mut Store) -> Vec<u8> {
    if args.len() != 1 {
        return encode_error(anyhow!("ERR wrong number of arguments for 'ttl' commands"));
    }

    let key = &args[0];

    let Some(obj) = store.get(key) else {
        return RESP_MINUS_TWO.to_vec(); // Key does not exist
    };

    if obj.expires_at == -1 {
        // Exist, but no expiration is set
        return RESP_MINUS_ONE.to_vec();
    }

    let duration_ms = obj.expires_at - Utc::now().timestamp_millis();

    return if duration_ms < 0 {
        RESP_MINUS_TWO.to_vec() // Expired
    } else {
        encode(Value::Int64(duration_ms / 1_000), false)
    };
}

pub fn del(args: Vec<String>, store: &mut Store) -> Vec<u8> {
    let mut count_deleted = 0;

    for key in args {
        if store.del(key) {
            count_deleted += 1;
        }
    }

    return encode(Value::Int32(count_deleted), false);
}

pub fn expire(args: Vec<String>, store: &mut Store) -> Vec<u8> {
    if args.len() <= 1 {
        return encode_error(anyhow!(
            "ERR wrong number of arguments for 'expire' commands"
        ));
    }

    let key = &args[0];
    let ex_duration_sec: i64 = match args[1].parse() {
        Ok(res) => res,
        Err(_) => return encode_error(anyhow!("ERR value is not an integer or out of range")),
    };

    match store.get_mut(key) {
        Some(s) => {
            s.expires_at = Utc::now().timestamp_millis() + ex_duration_sec * 1000;
        }
        None => {
            return RESP_ZERO.to_vec();
        }
    };

    // 1 if timeout is set
    return RESP_ONE.to_vec();
}

fn bg_rewrite_aof(_args: Vec<String>, store: &mut Store) -> Vec<u8> {
    store.dump_all_aof();
    return RESP_OK.to_vec();
}

fn incr(args: Vec<String>, store: &mut Store) -> Vec<u8> {
    if args.len() != 1 {
        return encode_error(anyhow!("ERR wrong number of arguments for 'incr' commands"));
    }

    let key = &args[0];
    let obj = store.get_or_insert(
        key,
        StoreObject::new(Value::String("0".to_owned()), -1, TYPE_STRING, ENCODING_INT),
    );

    if let Err(err) = obj.assert_type(TYPE_STRING) {
        return encode_error(err);
    }
    if let Err(err) = obj.assert_encoding(ENCODING_INT) {
        return encode_error(err);
    }

    return match &obj.value {
        Value::String(s) => {
            let Ok(mut i) = s.parse::<i64>() else {
                return encode_error(anyhow!("wrong data type for 'incr' command"))
            };

            i += 1;
            obj.value = Value::String(i.to_string());

            encode(Value::Int64(i), false)
        }
        _ => {
            encode_error(anyhow!("wrong data type for 'incr' command"))
        }
    }
}

pub fn respond(cmds: Commands, store: &mut Store, stream: &mut impl Write) -> io::Result<()> {
    let mut stream = BufWriter::new(stream);

    for cmd in cmds {
        let buf = match cmd.cmd.as_str() {
            "PING" => ping(cmd.args),
            "SET" => set(cmd.args, store),
            "GET" => get(cmd.args, store),
            "TTL" => ttl(cmd.args, store),
            "DEL" => del(cmd.args, store),
            "EXPIRE" => expire(cmd.args, store),
            "BGREWRITEAOF" => bg_rewrite_aof(cmd.args, store),
            "INCR" => incr(cmd.args, store),
            _ => ping(cmd.args),
        };
        stream.write(&buf)?;
    }

    return stream.flush();
}
