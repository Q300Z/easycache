use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::Notify;
use tokio::task;
use tokio::time::sleep;
use crate::strategy::CacheStrategy;

struct CacheEntry<V> {
    value: V,
    expires_at: DateTime<Utc>,
}

pub struct FIFOCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    capacity: usize,
    ttl: Duration,
    map: Arc<Mutex<HashMap<K, CacheEntry<V>>>>,
    order: Arc<Mutex<VecDeque<K>>>,
    notify_stop: Arc<Notify>,
}

impl<K, V> FIFOCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(capacity: usize, ttl: Duration, clean_interval: Duration) -> Self {
        let cache = FIFOCache {
            capacity,
            ttl,
            map: Arc::new(Mutex::new(HashMap::new())),
            order: Arc::new(Mutex::new(VecDeque::new())),
            notify_stop: Arc::new(Notify::new()),
        };

        cache.start_cleaner(clean_interval);
        cache
    }
}

impl<K, V> CacheStrategy<K, V> for FIFOCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn put(&mut self, key: K, value: V) {
        let mut map = self.map.lock().unwrap();
        let mut order = self.order.lock().unwrap();

        if map.contains_key(&key) {
            return; // FIFO ne met pas à jour les valeurs existantes
        }

        if order.len() >= self.capacity {
            if let Some(oldest) = order.pop_front() {
                map.remove(&oldest);
            }
        }

        order.push_back(key.clone());
        map.insert(
            key,
            CacheEntry {
                value,
                expires_at: Utc::now() + chrono::Duration::from_std(self.ttl).unwrap(),
            },
        );
    }

    fn get(&mut self, key: &K) -> Option<V> {
        let map = self.map.lock().unwrap();
        if let Some(entry) = map.get(key) {
            if entry.expires_at > Utc::now() {
                return Some(entry.value.clone());
            } else {
                drop(map); // release before relocking
                let mut map = self.map.lock().unwrap();
                let mut order = self.order.lock().unwrap();
                map.remove(key);
                order.retain(|k| k != key);
            }
        }
        None
    }

    fn remove(&mut self, key: &K) {
        let mut map = self.map.lock().unwrap();
        let mut order = self.order.lock().unwrap();
        map.remove(key);
        order.retain(|k| k != key);
    }

    fn contains(&self, key: &K) -> bool {
        let map = self.map.lock().unwrap();
        map.contains_key(key)
    }

    fn len(&self) -> usize {
        let map = self.map.lock().unwrap();
        map.len()
    }
    fn is_empty(&self) -> bool {
        let map = self.map.lock().unwrap();
        map.is_empty()
    }
    fn clear(&mut self) {
        let mut map = self.map.lock().unwrap();
        let mut order = self.order.lock().unwrap();
        map.clear();
        order.clear();
    }

    fn start_cleaner(&self, clean_interval: Duration) {
        let map = Arc::clone(&self.map);
        let order = Arc::clone(&self.order);
        let notify = Arc::clone(&self.notify_stop);

        task::spawn(async move {
            loop {
                tokio::select! {
                    _ = sleep(clean_interval) => {
                        let now = Utc::now();
                        let mut map = map.lock().unwrap();
                        let mut order = order.lock().unwrap();

                        order.retain(|key| {
                            map.get(key).map_or(false, |entry| entry.expires_at > now)
                        });

                        map.retain(|_key, entry| entry.expires_at > now);
                    }
                    _ = notify.notified() => {
                        break;
                    }
                }
            }
        });
    }

    fn stop_cleaner(&self) {
        self.notify_stop.notify_waiters();
    }
}
