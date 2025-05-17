# RustyCache Rust Library
![Rust](https://img.shields.io/badge/Rust-lang-000000.svg?style=flat&logo=rust)
[![Rust](https://github.com/Q300Z/easycache/actions/workflows/ci.yml/badge.svg)](https://github.com/Q300Z/easycache/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/easycache.svg)](https://crates.io/crates/easycache)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A generic, thread-safe, asynchronous cache library in Rust implementing multiple cache eviction strategies with TTL and background cleaning.

## Features

- Supports multiple cache eviction strategies:
    - **LFU** (Least Frequently Used)
    - **FIFO** (First In First Out)
- Thread-safe with `Arc<Mutex<...>>`
- Time-to-live (TTL) expiration on entries
- Background cleaner task using Tokio async runtime
- Generic over keys and values (with necessary trait bounds)
- Simple trait-based `CacheStrategy` interface for easy extension

## Usage

### Add dependency

Add this crate to your `Cargo.toml`:

```toml
rustycache = { path = "path/to/rustycache" }
```
Or if published on crates.io, replace with version:
```toml
rustycache = "1.0"
```
### Example: Using LFU Cache
```rust
use easycache::LFUCache;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Create LFU cache with capacity 100, TTL 60 seconds, cleaner interval 10 seconds
    let mut cache = Easycache::new(100, Duration::from_secs(60), Duration::from_secs(10), easycache::strategy::StrategyType::LFU);

    // Put some values
    cache.put("key1".to_string(), "value1".to_string());
    cache.put("key2".to_string(), "value2".to_string());

    // Get a value
    if let Some(val) = cache.get(&"key1".to_string()) {
        println!("Got: {}", val);
    } else {
        println!("Key expired or not found");
    }
}
```
### Example: Using FIFO Cache
```rust
use easycache::FIFOCache;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Create FIFO cache with capacity 50, TTL 120 seconds, cleaner interval 15 seconds
    let mut cache = Easycache::new(50, Duration::from_secs(120), Duration::from_secs(15), easycache::strategy::StrategyType::FIFO);

    // Put some values
    cache.put("foo".to_string(), 123);
    cache.put("bar".to_string(), 456);

    // Get a value
    if let Some(val) = cache.get(&"foo".to_string()) {
        println!("Got: {}", val);
    } else {
        println!("Key expired or not found");
    }
}
```
## Testing
To run the tests, use the following command:

```bash
cargo test
```
Tests cover cache insertion, eviction, TTL expiration, concurrency safety, and cleanup logic.
## License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.