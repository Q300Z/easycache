#[cfg(test)]
mod fifo_tests {
    use std::time::Duration;
    use tokio::time::sleep;
    use easycache::easycache::Easycache;

    fn create_cache(capacity: usize, ttl_secs: u64, clean_interval_secs: u64) -> Easycache<String, String> {
        Easycache::new(capacity, Duration::from_secs(ttl_secs), Duration::from_secs(clean_interval_secs), easycache::strategy::StrategyType::FIFO)
    }

    #[tokio::test]
    async fn test_put_and_get_basic() {
        let mut cache = create_cache(2, 5, 60);
        cache.put("key1".to_string(), "value1".to_string());

        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert!(cache.contains(&"key1".to_string()));
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());
    }

    #[tokio::test]
    async fn test_put_does_not_update_existing_value() {
        let mut cache = create_cache(2, 5, 60);
        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key1".to_string(), "value2".to_string()); // should be ignored

        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_fifo_eviction_order() {
        let mut cache = create_cache(2, 5, 60);
        cache.put("a".to_string(), "A".to_string());
        cache.put("b".to_string(), "B".to_string());
        cache.put("c".to_string(), "C".to_string()); // should evict "a"

        assert!(!cache.contains(&"a".to_string()));
        assert!(cache.contains(&"b".to_string()));
        assert!(cache.contains(&"c".to_string()));
        assert_eq!(cache.len(), 2);
    }

    #[tokio::test]
    async fn test_expiration_removes_entry() {
        let mut cache = create_cache(2, 1, 60);
        cache.put("x".to_string(), "expire_me".to_string());

        assert_eq!(cache.get(&"x".to_string()), Some("expire_me".to_string()));

        sleep(Duration::from_secs(2)).await;

        // Now expired, get returns None and removes it internally
        assert_eq!(cache.get(&"x".to_string()), None);
        assert!(!cache.contains(&"x".to_string()));
    }

    #[tokio::test]
    async fn test_remove_and_clear() {
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
        let mut cache = create_cache(2, 1, 1); // TTL=1s, cleaner every 1s
        cache.put("k1".to_string(), "v1".to_string());

        sleep(Duration::from_secs(2)).await; // let entry expire and cleaner run

        assert_eq!(cache.get(&"k1".to_string()), None);
        assert_eq!(cache.len(), 0);
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
