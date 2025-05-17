pub mod fifo;
pub mod lfu;
pub mod lru;

use std::time::Duration;

pub trait CacheStrategy<K, V>: Send + Sync {
    fn put(&mut self, key: K, value: V);
    fn get(&mut self, key: &K) -> Option<V>;
    fn remove(&mut self, key: &K);
    fn contains(&self, key: &K) -> bool;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn clear(&mut self);
    fn start_cleaner(&self, interval: Duration);
    fn stop_cleaner(&self);
}

pub enum StrategyType {
    LRU,
    FIFO,
    LFU,
}
