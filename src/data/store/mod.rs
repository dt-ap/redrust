use anyhow::anyhow;
use chrono::Utc;

use crate::{common::Value, config::Config};
use std::collections::HashMap;

pub const TYPE_STRING: u8 = 0 << 4;

pub const ENCODING_RAW: u8 = 0;
pub const ENCODING_INT: u8 = 1;
pub const ENCODING_EMBSTR: u8 = 8;

pub const EMBED_STRING_MAX_LENGTH: usize = 44;

pub struct Store {
    inner: HashMap<String, StoreObject>,
    config: Config,
}

impl Store {
    pub fn new(config: Config) -> Store {
        return Store {
            inner: HashMap::new(),
            config: config,
        };
    }

    fn may_remove(&mut self, k: &String) -> Option<()> {
        if let Some(i) = self.inner.get(k) {
            if i.expires_at != -1 && i.expires_at <= Utc::now().timestamp_millis() {
                self.inner.remove(k);
                return None;
            }

            return Some(());
        };

        return None;
    }

    pub fn get(&mut self, k: &String) -> Option<&StoreObject> {
        return self.may_remove(k).and_then(|_| self.inner.get(k));
    }

    pub fn get_mut(&mut self, k: &String) -> Option<&mut StoreObject> {
        return self.may_remove(k).and_then(|_| self.inner.get_mut(k));
    }

    pub fn get_or_insert(&mut self, k: &String, default: StoreObject) -> &mut StoreObject {
        return match self.may_remove(k) {
            _ => self.inner.entry(k.to_string()).or_insert(default),
        };
    }

    pub fn put(&mut self, k: String, obj: StoreObject) -> Option<StoreObject> {
        if self.inner.len() >= self.config.keys_limit as usize {
            self.evict();
        }
        return self.inner.insert(k, obj);
    }

    pub fn del(&mut self, k: String) -> bool {
        return self.inner.remove(&k).map_or(false, |_| true);
    }
}

mod aof;
mod eviction;
mod expire;

#[derive(Clone)]
pub struct StoreObject {
    pub type_encoding: u8,
    pub value: Value,
    pub expires_at: i64,
}

impl StoreObject {
    pub fn new(value: Value, duration_ms: i64, obj_type: u8, obj_encoding: u8) -> StoreObject {
        let mut expires_at = -1_i64;

        if duration_ms > 0 {
            expires_at = Utc::now().timestamp_millis() + duration_ms;
        }

        return StoreObject {
            type_encoding: obj_type | obj_encoding,
            value,
            expires_at,
        };
    }

    pub fn assert_type(&self, t: u8) -> anyhow::Result<()> {
        if self.get_type() != t {
            return Err(anyhow!("the operation is not permitted on this type"));
        }

        return Ok(());
    }

    pub fn assert_encoding(&self, t: u8) -> anyhow::Result<()> {
        if self.get_encoding() != t {
            return Err(anyhow!("ERR value is not an integer or out of range"));
        }

        return Ok(());
    }

    fn get_type(&self) -> u8 {
        return self.type_encoding & 0b11110000;
    }

    fn get_encoding(&self) -> u8 {
        return self.type_encoding & 0b00001111;
    }
}

pub fn deduce_type_encoding(value: &String) -> (u8, u8) {
    let obj_type = TYPE_STRING;
    let Err(_) = value.parse::<i64>() else {
        return (obj_type, ENCODING_INT);
    };

    if value.len() <= EMBED_STRING_MAX_LENGTH {
        return (obj_type, ENCODING_EMBSTR);
    }
    return (obj_type, ENCODING_RAW);
}
