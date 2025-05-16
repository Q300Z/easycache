use easycache::lru_cache::LRUCache;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let mut cache = LRUCache::new(3, Duration::from_secs(5), Duration::from_secs(2));
    cache.put("x", "hello");
    println!("{:?}", cache.get(&"x")); // Some("hello")
    cache.put("x2", "hello2");
    cache.put("x3", "hello3");
    cache.put("x4", "hello4");

    
    println!("{:?}", cache.get(&"x2")); // Some("hello2")
    println!("{:?}", cache.get(&"x3")); // Some("hello3")
    println!("{:?}", cache.get(&"x4")); // Some("hello4")
    println!("{:?}", cache.get(&"x5")); // None
    println!("{:?}", cache.get(&"x"));
    sleep(Duration::from_secs(6)).await;
    println!("{:?}", cache.get(&"x")); // None (expiré)

    cache.stop_cleaner().await;
}
