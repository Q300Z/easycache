use std::time::Duration;
use crate::strategy::{CacheStrategy, StrategyType};
use crate::strategy::fifo::FIFOCache;
use crate::strategy::lfu::LFUCache;
use crate::strategy::lru::LRUCache;

pub struct Rustycache<K, V> {
    inner: Box<dyn CacheStrategy<K, V>>,
}

impl<K, V> Rustycache<K, V>
where
    K: 'static + Send + Sync + Clone + Eq + std::hash::Hash,
    V: 'static + Send + Sync + Clone,
{
    pub fn new(cap: usize, ttl: Duration, clean_interval: Duration, strat: StrategyType) -> Self {
        let inner: Box<dyn CacheStrategy<K, V>> = match strat {
            StrategyType::LRU => Box::new(LRUCache::new(cap, ttl, clean_interval)),
            StrategyType::FIFO => Box::new(FIFOCache::new(cap, ttl, clean_interval)),
            StrategyType::LFU => Box::new(LFUCache::new(cap, ttl, clean_interval)),
        };

        inner.start_cleaner(clean_interval);

        Rustycache { inner }
    }

    pub fn put(&mut self, key: K, value: V) {
        self.inner.put(key, value)
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        self.inner.get(key)
    }

    pub fn remove(&mut self, key: &K) {
        self.inner.remove(key)
    }

    pub fn contains(&self, key: &K) -> bool {
        self.inner.contains(key)
    }

    pub fn stop_cleaner(&self) {
        self.inner.stop_cleaner()
    }

    pub fn start_cleaner(&self, interval: Duration) {
        self.inner.start_cleaner(interval)
    }
    
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.inner.clear()
    }
}
