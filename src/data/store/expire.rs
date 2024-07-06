use chrono::Utc;

use super::Store;

impl Store {
    // TODO: Optimize
    //   - Sampling
    //   - Unnecessary iteration
    fn expire_sample(&mut self) -> f32 {
        let mut limit = 20;
        let mut expired_count = 0;
        let mut keys_to_remove = Vec::<String>::new();

        // Iteration of Hashmap is not in order
        for (key, val) in self.inner.iter() {
            if val.expires_at != -1 {
                limit -= 1;

                if val.expires_at <= Utc::now().timestamp_millis() {
                    keys_to_remove.push(key.to_string());
                }
            }

            if limit == 0 {
                break;
            }
        }

        for k in keys_to_remove {
            self.inner.remove(&k);
            expired_count += 1;
        }

        return expired_count as f32 / 20.0;
    }

    // Delete expired keys active mode
    // Sampling approach: https://redis.io/commands/expire/
    pub fn delete_expired_keys(&mut self) {
        loop {
            let frac = self.expire_sample();

            if frac < 0.25 {
                break;
            }
        }
        println!(
            "deleted the expired but undeleted keys. total keys {}",
            self.inner.len()
        );
    }
}
