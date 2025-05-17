use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashMap, HashSet};
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
    frequency: usize,
}

pub struct LFUCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    capacity: usize,
    ttl: Duration,
    map: Arc<Mutex<HashMap<K, CacheEntry<V>>>>,
    freq_map: Arc<Mutex<BTreeMap<usize, HashSet<K>>>>,
    notify_stop: Arc<Notify>,
}

impl<K, V> LFUCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(capacity: usize, ttl: Duration, clean_interval: Duration) -> Self {
        let cache = LFUCache {
            capacity,
            ttl,
            map: Arc::new(Mutex::new(HashMap::<K, CacheEntry<V>>::new())),
            freq_map: Arc::new(Mutex::new(BTreeMap::new())),
            notify_stop: Arc::new(Notify::new()),
        };

        let map_clone = Arc::clone(&cache.map);
        let freq_map_clone = Arc::clone(&cache.freq_map);
        let notify_clone = Arc::clone(&cache.notify_stop);

        task::spawn(async move {
            loop {
                tokio::select! {
                    _ = sleep(clean_interval) => {
                        let now = Utc::now();
                        let mut map = map_clone.lock().unwrap();
                        let mut freq_map = freq_map_clone.lock().unwrap();
                        let keys_to_remove: Vec<K> = map.iter()
                            .filter_map(|(k, v)| {
                                if v.expires_at <= now {
                                    Some(k.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();

                        for key in keys_to_remove {
                            if let Some(entry) = map.remove(&key) {
                                if let Some(set) = freq_map.get_mut(&entry.frequency) {
                                    set.remove(&key);
                                    if set.is_empty() {
                                        freq_map.remove(&entry.frequency);
                                    }
                                }
                            }
                        }
                    },
                    _ = notify_clone.notified() => break,
                }
            }
        });

        cache
    }
}

impl<K, V> CacheStrategy<K, V> for LFUCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn put(&mut self, key: K, value: V) {
        let mut map = self.map.lock().unwrap();
        let mut freq_map = self.freq_map.lock().unwrap();

        if let Some(entry) = map.get_mut(&key) {
            entry.value = value;
            entry.expires_at = Utc::now() + chrono::Duration::from_std(self.ttl).unwrap();
            return;
        }

        if map.len() >= self.capacity {
            if let Some((&min_freq, keys)) = freq_map.iter_mut().next() {
                if let Some(k) = keys.iter().next().cloned() {
                    keys.remove(&k);
                    if keys.is_empty() {
                        freq_map.remove(&min_freq);
                    }
                    map.remove(&k);
                }
            }
        }

        map.insert(key.clone(), CacheEntry {
            value,
            expires_at: Utc::now() + chrono::Duration::from_std(self.ttl).unwrap(),
            frequency: 1,
        });

        freq_map.entry(1).or_insert_with(HashSet::new).insert(key);
    }

    fn get(&mut self, key: &K) -> Option<V> {
        let mut map = self.map.lock().unwrap();
        let mut freq_map = self.freq_map.lock().unwrap();

        if let Some(entry) = map.get_mut(key) {
            if entry.expires_at <= Utc::now() {
                let freq = entry.frequency;
                map.remove(key);
                if let Some(set) = freq_map.get_mut(&freq) {
                    set.remove(key);
                    if set.is_empty() {
                        freq_map.remove(&freq);
                    }
                }
                return None;
            }

            let old_freq = entry.frequency;
            entry.frequency += 1;

            if let Some(set) = freq_map.get_mut(&old_freq) {
                set.remove(key);
                if set.is_empty() {
                    freq_map.remove(&old_freq);
                }
            }

            freq_map
                .entry(entry.frequency)
                .or_insert_with(HashSet::new)
                .insert(key.clone());

            return Some(entry.value.clone());
        }

        None
    }

    fn remove(&mut self, key: &K) {
        let mut map = self.map.lock().unwrap();
        let mut freq_map = self.freq_map.lock().unwrap();

        if let Some(entry) = map.remove(key) {
            if let Some(set) = freq_map.get_mut(&entry.frequency) {
                set.remove(key);
                if set.is_empty() {
                    freq_map.remove(&entry.frequency);
                }
            }
        }
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
        let mut freq_map = self.freq_map.lock().unwrap();
        map.clear();
        freq_map.clear();
    }

    fn start_cleaner(&self, clean_interval: Duration) {
        let map = Arc::clone(&self.map);
        let notify = Arc::clone(&self.notify_stop);

        task::spawn(async move {
            loop {
                tokio::select! {
                    _ = sleep(clean_interval) => {
                        let now = Utc::now();
                        let mut map = map.lock().unwrap();

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
