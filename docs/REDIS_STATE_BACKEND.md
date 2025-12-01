# Redis State Backend for Stream Processing

## Overview

Redis backend provides **distributed, scalable state management** for production streaming applications. Perfect for:

- **Multi-instance deployments** - Share state across multiple stream processors
- **High-throughput scenarios** - 100k+ operations/second
- **Automatic persistence** - RDB snapshots + AOF for durability
- **Built-in TTL** - Automatic key expiration
- **Redis Cluster** - Horizontal scaling for massive workloads

## Features

### ✅ Implemented
- Basic CRUD operations (get, put, update, delete)
- TTL support for automatic expiration
- Key listing with namespace prefixes
- Connection pooling via redis-rs
- Graceful fallback when Redis unavailable

### 🚧 Future Enhancements
- Atomic operations (INCR, HINCRBY)
- Transactions (MULTI/EXEC)
- Pub/Sub for cross-instance coordination
- Redis Streams integration
- Redis Cluster support
- Checkpointing to Redis keys

## Installation

Add Redis support to your `Cargo.toml`:

```toml
[dependencies]
rust-rule-engine = { version = "1.3", features = ["streaming-redis"] }
```

Or build with feature flag:

```bash
cargo build --features streaming-redis
```

## Quick Start

### 1. Start Redis Server

```bash
# Using Docker (recommended)
docker run -d -p 6379:6379 redis:latest

# Or install locally
# Ubuntu/Debian
sudo apt-get install redis-server

# macOS
brew install redis
```

### 2. Use Redis Backend

```rust
use rust_rule_engine::streaming::*;
use rust_rule_engine::types::Value;

// Create Redis-backed state store
let backend = StateBackend::Redis {
    url: "redis://127.0.0.1:6379".to_string(),
    key_prefix: "myapp".to_string(),  // Namespace your keys
};

let mut store = StateStore::new(backend);

// Operations are automatically persisted to Redis
store.put("counter", Value::Integer(42))?;
let value = store.get("counter")?;

// TTL support
store.put_with_ttl("session_key", 
    Value::String("token".to_string()), 
    Duration::from_secs(3600))?;
```

### 3. Distributed State Example

```rust
// Multiple instances can share state via Redis
let backend = StateBackend::Redis {
    url: "redis://127.0.0.1:6379".to_string(),
    key_prefix: "distributed_counter".to_string(),
};

// Instance 1
let store1 = StateStore::new(backend.clone());
store1.put("total", Value::Integer(10))?;

// Instance 2 (different process/machine)
let store2 = StateStore::new(backend.clone());
if let Some(Value::Integer(n)) = store2.get("total")? {
    println!("Shared counter: {}", n);  // Prints: 10
}
```

## Configuration

### Redis URL Formats

```rust
// Local Redis
"redis://127.0.0.1:6379"

// With authentication
"redis://:password@127.0.0.1:6379"

// Specific database
"redis://127.0.0.1:6379/2"

// TLS/SSL
"rediss://127.0.0.1:6380"

// Redis Sentinel (future)
"redis-sentinel://host1:26379,host2:26379/mymaster"
```

### Key Prefixing

Use `key_prefix` to namespace your keys:

```rust
StateBackend::Redis {
    url: "redis://127.0.0.1:6379".to_string(),
    key_prefix: "prod_stream_v2".to_string(),
}

// Key "counter" becomes "prod_stream_v2:counter" in Redis
```

## Examples

Run the comprehensive demo:

```bash
# Start Redis first
docker run -d -p 6379:6379 redis:latest

# Run demo
cargo run --example redis_state_demo --features streaming-redis
```

The demo includes:
1. **Basic Operations** - CRUD with Redis
2. **Distributed Counter** - State sharing across instances
3. **TTL Management** - Automatic key expiration
4. **Multi-Instance Coordination** - Simulated distributed processing

## Architecture

### Memory Backend
```
Application ──> In-Memory HashMap
                (Lost on restart)
```

### File Backend
```
Application ──> Local File System
                (Single machine only)
```

### Redis Backend ✨
```
Instance 1 ─┐
Instance 2 ─┼──> Redis Server ──> RDB/AOF Persistence
Instance 3 ─┘         │
            Redis Cluster (Sharding)
```

## Performance

Typical Redis performance on modern hardware:

| Operation | Throughput |
|-----------|------------|
| GET       | 100k+ ops/sec |
| SET       | 80k+ ops/sec |
| INCR      | 100k+ ops/sec |
| Pipeline  | 1M+ ops/sec |

## Best Practices

### 1. Use Key Prefixes
Always use descriptive key prefixes to avoid collisions:

```rust
StateBackend::Redis {
    key_prefix: format!("{}:{}:{}", app_name, environment, version),
    // e.g., "orderproc:prod:v2"
    ..
}
```

### 2. Set Appropriate TTLs
Don't let state accumulate indefinitely:

```rust
let config = StateConfig {
    enable_ttl: true,
    default_ttl: Duration::from_hours(24),
    ..
};
```

### 3. Monitor Redis Memory
```bash
redis-cli INFO memory
```

### 4. Use Redis Persistence
Enable both RDB and AOF in redis.conf:

```conf
save 900 1
save 300 10
save 60 10000

appendonly yes
appendfsync everysec
```

### 5. Connection Pooling
Redis-rs automatically handles connection pooling. For high concurrency, tune:

```conf
# redis.conf
maxclients 10000
tcp-backlog 511
```

## Troubleshooting

### Connection Refused
```
Error: Redis connection error: Connection refused
```

**Solution**: Ensure Redis is running:
```bash
docker ps | grep redis
# or
redis-cli ping
```

### Memory Issues
```
Error: OOM command not allowed when used memory > 'maxmemory'
```

**Solution**: Configure Redis eviction policy:
```conf
maxmemory 2gb
maxmemory-policy allkeys-lru
```

### Slow Operations
Enable slow log monitoring:
```bash
redis-cli CONFIG SET slowlog-log-slower-than 10000
redis-cli SLOWLOG GET 10
```

## Comparison with Other Backends

| Feature | Memory | File | Redis | RocksDB |
|---------|--------|------|-------|---------|
| Distributed | ❌ | ❌ | ✅ | ❌ |
| Persistent | ❌ | ✅ | ✅ | ✅ |
| TTL Support | ✅ | ❌ | ✅ | ✅ |
| Throughput | 🚀🚀🚀 | 🚀 | 🚀🚀 | 🚀🚀 |
| Scalability | ❌ | ❌ | ✅ Cluster | ✅ |
| Operations Cost | Low | Low | Medium | Low |

## Production Checklist

- [ ] Enable Redis persistence (RDB + AOF)
- [ ] Configure maxmemory and eviction policy
- [ ] Set up Redis replication (master-slave)
- [ ] Monitor memory usage and key count
- [ ] Use key prefixes for namespacing
- [ ] Set appropriate TTLs
- [ ] Enable Redis Cluster for horizontal scaling
- [ ] Configure connection timeouts
- [ ] Set up monitoring (Redis INFO, slowlog)
- [ ] Plan backup strategy

## Future Roadmap

### Short Term
- [ ] Atomic INCR/DECR operations
- [ ] Hash operations (HSET, HGET, HINCRBY)
- [ ] List operations for queues
- [ ] Transactions support (MULTI/EXEC)

### Medium Term
- [ ] Redis Streams integration
- [ ] Pub/Sub for event coordination
- [ ] Redis Cluster client support
- [ ] Lua scripting support

### Long Term
- [ ] Redis modules integration
- [ ] TimeSeries support
- [ ] RedisJSON support
- [ ] RedisGraph for complex state

## Resources

- [Redis Documentation](https://redis.io/documentation)
- [Redis Best Practices](https://redis.io/topics/best-practices)
- [redis-rs Crate](https://docs.rs/redis/)
- [Redis Cluster Tutorial](https://redis.io/topics/cluster-tutorial)

## License

Same as rust-rule-engine: MIT
