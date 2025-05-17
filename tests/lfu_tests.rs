#[cfg(test)]
mod lfu_tests {
    use std::time::Duration;
    use tokio::time::sleep;
    use easycache::easycache::Easycache;

    fn create_cache(capacity: usize, ttl_secs: u64, interval_secs: u64) -> Easycache<String, String> {
        Easycache::new(capacity, Duration::from_secs(ttl_secs), Duration::from_secs(interval_secs), easycache::strategy::StrategyType::LFU)
    }

    #[tokio::test]
    async fn test_put_and_get_basic() {
        let mut cache = create_cache(2, 5, 60);
        cache.put("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert!(cache.contains(&"key1".to_string()));
        assert_eq!(cache.len(), 1);
    }

    #[tokio::test]
    async fn test_update_value_and_frequency() {
        let mut cache = create_cache(2, 5, 60);
        cache.put("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        cache.put("key1".to_string(), "value2".to_string()); // update value resets frequency?
        assert_eq!(cache.get(&"key1".to_string()), Some("value2".to_string()));
    }

    #[tokio::test]
    async fn test_lfu_eviction() {
        let mut cache = create_cache(2, 5, 60);
        cache.put("a".to_string(), "A".to_string());
        cache.put("b".to_string(), "B".to_string());
        cache.get(&"a".to_string()); // freq a = 2
        // b freq = 1, a freq = 2

        cache.put("c".to_string(), "C".to_string()); // should evict 'b' (lowest freq)

        assert!(cache.get(&"a".to_string()).is_some());
        assert!(cache.get(&"b".to_string()).is_none());
        assert!(cache.get(&"c".to_string()).is_some());
    }

    #[tokio::test]
    async fn test_expiration_behavior() {
        let mut cache = create_cache(2, 1, 60);
        cache.put("x".to_string(), "expire_me".to_string());

        assert_eq!(cache.get(&"x".to_string()), Some("expire_me".to_string()));
        sleep(Duration::from_secs(2)).await;

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
        let mut cache = create_cache(2, 1, 1); // TTL = 1s, cleaner every 1s
        cache.put("k1".to_string(), "v1".to_string());

        sleep(Duration::from_secs(2)).await; // wait expiration + cleaner run

        assert_eq!(cache.get(&"k1".to_string()), None);
        assert_eq!(cache.len(), 0);
    }

    #[tokio::test]
    async fn test_stop_cleaner_no_panic() {
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
