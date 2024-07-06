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

    pub fn get(&mut self, k: String) -> Option<&StoreValue> {
        return self.may_remove(&k).and_then(|_| self.inner.get(&k));
    }

    pub fn get_mut(&mut self, k: String) -> Option<&mut StoreValue> {
        return self.may_remove(&k).and_then(|_| self.inner.get_mut(&k));
    }

    pub fn put(&mut self, k: String, v: StoreValue) -> Option<StoreValue> {
        return self.inner.insert(k, v);
    }

    pub fn del(&mut self, k: String) -> bool {
        return self.inner.remove(&k).map_or(false, |_| true);
    }
}

mod expire;

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
