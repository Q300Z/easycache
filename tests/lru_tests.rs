#[cfg(test)]
mod lru_tests {
    use std::time::Duration;
    use tokio::time::sleep;
    use easycache::easycache::Easycache;

    fn create_cache(capacity: usize, ttl_secs: u64, interval_secs: u64) -> Easycache<String, String> {
        Easycache::new(capacity, Duration::from_secs(ttl_secs), Duration::from_secs(interval_secs), easycache::strategy::StrategyType::LRU)
    }

    #[tokio::test]
    async fn test_insert_and_get() {
        let mut cache = create_cache(2, 5, 60);
        cache.put("a".to_string(), "value1".to_string());

        assert_eq!(cache.get(&"a".to_string()), Some("value1".to_string()));
        assert!(cache.contains(&"a".to_string()));
        assert_eq!(cache.len(), 1);
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let mut cache = create_cache(2, 5, 60);
        cache.put("a".to_string(), "A".to_string());
        cache.put("b".to_string(), "B".to_string());
        cache.get(&"a".to_string()); // 'a' becomes recently used
        cache.put("c".to_string(), "C".to_string()); // Should evict 'b'

        assert!(cache.get(&"a".to_string()).is_some());
        assert!(cache.get(&"b".to_string()).is_none()); // evicted
        assert!(cache.get(&"c".to_string()).is_some());
    }

    #[tokio::test]
    async fn test_expiration_behavior() {
        let mut cache = create_cache(2, 1, 60);
        cache.put("x".to_string(), "expire_me".to_string());

        assert_eq!(cache.get(&"x".to_string()), Some("expire_me".to_string()));
        sleep(Duration::from_secs(2)).await;

        assert_eq!(cache.get(&"x".to_string()), None);
    }

    #[tokio::test]
    async fn test_clear_and_remove() {
        let mut cache = create_cache(3, 5, 60);
        cache.put("a".to_string(), "1".to_string());
        cache.put("b".to_string(), "2".to_string());
        cache.put("c".to_string(), "3".to_string());

        cache.remove(&"b".to_string());
        assert_eq!(cache.get(&"b".to_string()), None);
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_cleaner_removes_expired() {
        let mut cache = create_cache(2, 1, 1); // TTL = 1s, cleaner every 1s
        cache.put("k1".to_string(), "v1".to_string());

        sleep(Duration::from_secs(2)).await; // Let it expire

        assert_eq!(cache.get(&"k1".to_string()), None);
        assert_eq!(cache.len(), 0);
    }

    #[tokio::test]
    async fn test_eviction_order_preserved() {
        let mut cache = create_cache(3, 5, 60);
        cache.put("1".to_string(), "v1".to_string());
        cache.put("2".to_string(), "v2".to_string());
        cache.put("3".to_string(), "v3".to_string());

        // Access "1" to make it recently used
        cache.get(&"1".to_string());

        // Add another entry to trigger eviction
        cache.put("4".to_string(), "v4".to_string());

        assert!(cache.get(&"1".to_string()).is_some()); // Not evicted
        assert!(cache.get(&"2".to_string()).is_none()); // Least recently used
    }

    #[tokio::test]
    async fn test_stop_cleaner_does_not_panic() {
        let cache = create_cache(2, 1, 1);
        tokio::time::sleep(Duration::from_millis(100)).await;
        cache.stop_cleaner(); // Just test stop logic without panic
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_start_cleaner_does_not_panic() {
        let cache = create_cache(2, 1, 1);
        tokio::time::sleep(Duration::from_millis(100)).await;
        cache.start_cleaner(Duration::from_secs(1)); // just ensure no panic
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
