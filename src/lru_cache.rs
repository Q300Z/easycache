use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task;
use tokio::time::sleep;

struct CacheEntry<V> {
    value: V,
    expires_at: DateTime<Utc>,
}

/// A generic, thread-safe LRU cache with TTL and background cleanup.
pub struct LRUCache<K, V>
where
    K: Eq + Hash + Clone + Send + 'static,
    V: Clone + Send + 'static,
{
    capacity: usize,
    ttl: Duration,
    map: Arc<Mutex<HashMap<K, CacheEntry<V>>>>,
    order: Arc<Mutex<VecDeque<K>>>,
    notify_stop: Arc<Notify>,
}

impl<K, V> LRUCache<K, V>
where
    K: Eq + Hash + Clone + Send + 'static,
    V: Clone + Send + 'static,
{
    /// Creates a new LRU cache with the given capacity, TTL, and cleaning interval.
    pub fn new(capacity: usize, ttl: Duration, clean_interval: Duration) -> Self {
        let cache = LRUCache {
            capacity,
            ttl,
            map: Arc::new(Mutex::new(HashMap::new())),
            order: Arc::new(Mutex::new(VecDeque::new())),
            notify_stop: Arc::new(Notify::new()),
        };

        let map_clone = cache.map.clone();
        let order_clone = cache.order.clone();
        let notify_clone = cache.notify_stop.clone();
        
        task::spawn(async move {
            loop {
                tokio::select! {
                    _ = sleep(clean_interval) => {
                        let now = Utc::now();
                        let mut map = map_clone.lock().unwrap();
                        let mut order = order_clone.lock().unwrap();

                        order.retain(|key| {
                            if let Some(entry) = map.get(key) {
                                if entry.expires_at > now {
                                    true
                                } else {
                                    map.remove(key);
                                    false
                                }
                            } else {
                                false
                            }
                        });
                    }
                    _ = notify_clone.notified() => {
                        break;
                    }
                }
            }
        });

        cache
    }

    /// Inserts a key-value pair into the cache.
    pub fn put(&mut self, key: K, value: V) {
        let mut map = self.map.lock().unwrap();
        let mut order = self.order.lock().unwrap();

        if map.contains_key(&key) {
            order.retain(|k| k != &key);
        }

        if order.len() >= self.capacity {
            if let Some(oldest) = order.pop_back() {
                map.remove(&oldest);
            }
        }

        order.push_front(key.clone());
        map.insert(
            key,
            CacheEntry {
                value,
                expires_at: Utc::now() + chrono::Duration::from_std(self.ttl).unwrap(),
            },
        );
    }

    /// Retrieves a value by key, or `None` if expired or not found.
    pub fn get(&self, key: &K) -> Option<V> {
        let mut map = self.map.lock().unwrap();
        let mut order = self.order.lock().unwrap();

        if let Some(entry) = map.get_mut(key) {
            if entry.expires_at > Utc::now() {
                entry.expires_at = Utc::now() + chrono::Duration::from_std(self.ttl).unwrap();
                order.retain(|k| k != key);
                order.push_front(key.clone());
                return Some(entry.value.clone());
            } else {
                map.remove(key);
                order.retain(|k| k != key);
            }
        }

        None
    }

    /// Check if the cache contains a key
    pub fn contains(&self, key: &K) -> bool {
        let map = self.map.lock().unwrap();
        map.contains_key(key)
    }

    /// Remove a key-value from the cache
    pub fn remove(&mut self, key: &K) {
        let mut map = self.map.lock().unwrap();
        let mut order = self.order.lock().unwrap();
        map.remove(key);
        order.retain(|k| k != key);
    }

    /// Stops the background cleaner task.
    pub async fn stop_cleaner(&self) {
        self.notify_stop.notify_waiters();
    }
}
