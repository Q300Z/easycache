
use std::time::Duration;
use tokio::time::sleep;
use easycache::lru_cache::LRUCache;

#[tokio::test]
async fn test_put_and_get() {
    let mut cache = LRUCache::new(2, Duration::from_secs(60), Duration::from_secs(10));
    cache.put("a", 1);
    cache.put("b", 2);

    assert_eq!(cache.get(&"a"), Some(1));
    assert_eq!(cache.get(&"b"), Some(2));
}

#[tokio::test]
async fn test_lru_eviction() {
    let mut cache = LRUCache::new(2, Duration::from_secs(60), Duration::from_secs(10));
    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3); // "a" devrait être éjecté

    assert_eq!(cache.get(&"a"), None);
    assert_eq!(cache.get(&"b"), Some(2));
    assert_eq!(cache.get(&"c"), Some(3));
}

#[tokio::test]
async fn test_ttl_expiration() {
    let mut cache = LRUCache::new(2, Duration::from_millis(100), Duration::from_millis(50));
    cache.put("a", 1);

    sleep(Duration::from_millis(150)).await;

    assert_eq!(cache.get(&"a"), None);
}

#[tokio::test]
async fn test_manual_remove() {
    let mut cache = LRUCache::new(2, Duration::from_secs(60), Duration::from_secs(10));
    cache.put("x", 99);
    assert!(cache.contains(&"x"));
    cache.remove(&"x");
    assert!(!cache.contains(&"x"));
}

#[tokio::test]
async fn test_ttl_refresh_on_access() {
    let mut cache = LRUCache::new(2, Duration::from_millis(200), Duration::from_millis(50));
    cache.put("x", 42);
    sleep(Duration::from_millis(100)).await;
    assert_eq!(cache.get(&"x"), Some(42));
    sleep(Duration::from_millis(100)).await;
    assert_eq!(cache.get(&"x"), Some(42));
}

#[tokio::test]
async fn test_stop_cleaner() {
    let mut cache = LRUCache::new(2, Duration::from_millis(100), Duration::from_millis(10));
    cache.put("a", 123);
    cache.stop_cleaner().await;
}
