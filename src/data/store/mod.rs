use chrono::Utc;

use crate::common::Value;
use std::collections::HashMap;

pub struct Store {
    inner: HashMap<String, StoreValue>,
}

impl Store {
    pub fn new() -> Store {
        return Store {
            inner: HashMap::new(),
        };
    }

    pub fn get(&self, k: String) -> Option<&StoreValue> {
        return self.inner.get(&k);
    }

    pub fn put(&mut self, k: String, v: StoreValue) -> Option<StoreValue> {
        return self.inner.insert(k, v);
    }
}

pub struct StoreValue {
    pub value: Value,
    pub expires_at: i64,
}

impl StoreValue {
    pub fn new(value: Value, duration_ms: i64) -> StoreValue {
        let mut expires_at = -1_i64;

        if duration_ms > 0 {
            expires_at = Utc::now().timestamp_millis() + duration_ms;
        }

        return StoreValue { value, expires_at };
    }
}
