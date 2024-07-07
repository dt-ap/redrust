use super::Store;

impl Store {
    fn evict_first(&mut self) {
        for (k, _) in self.inner.iter() {
            self.inner.remove(&k.to_string());
            return;
        }
    }

    pub(super) fn evict(&mut self) {
        match self.config.eviction_strategy.as_str() {
            "simple-first" => self.evict_first(),
            _ => (),
        }
    }
}
