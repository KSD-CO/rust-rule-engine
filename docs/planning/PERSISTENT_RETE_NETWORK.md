# Persistent RETE Network

**Status**: Planning Phase
**Priority**: Medium
**Estimated Impact**: 10-100x faster startup, crash recovery
**Complexity**: Medium
**Dependencies**: None

---

## 📋 Executive Summary

Enable serialization and deserialization of RETE network state to avoid rebuilding the entire network on every startup. This provides fast startup times and crash recovery capabilities.

**Current Problem:**
- RETE network rebuilt from scratch on every engine startup
- Loading 10,000 rules + building network takes 5-10 seconds
- No state recovery after crashes
- All working memory lost on shutdown

**Proposed Solution:**
- Serialize RETE network to disk (bincode/msgpack)
- Deserialize on startup (50-100ms instead of 5-10s)
- Snapshot working memory for crash recovery
- Incremental updates to persisted state

---

## 🎯 Goals

### Primary Goals
1. **10-100x Faster Startup** - Load pre-built network instead of rebuilding
2. **Crash Recovery** - Restore state from last snapshot
3. **Hot Reload** - Update rules without full rebuild
4. **Compact Format** - Efficient binary serialization

### Non-Goals (Phase 1)
- ❌ Distributed state replication (separate feature)
- ❌ Time-travel debugging (future enhancement)
- ❌ Automatic failover (requires distributed setup)

---

## 🏗️ Architecture

### Current Startup Flow (No Persistence)

```
┌──────────────┐
│ Load GRL     │  2-3 seconds (10K rules)
└──────────────┘
       │
       ▼
┌──────────────┐
│ Parse Rules  │  1-2 seconds
└──────────────┘
       │
       ▼
┌──────────────┐
│ Build RETE   │  2-3 seconds (create nodes, wire network)
└──────────────┘
       │
       ▼
┌──────────────┐
│ Ready        │  Total: 5-8 seconds
└──────────────┘
```

**Problem:** Every startup pays full cost, even if rules unchanged

---

### Proposed Startup Flow (With Persistence)

```
┌──────────────┐
│ Check Cache  │  <1 ms (file stat)
└──────────────┘
       │
       ├─ Cache Hit ──────────┐
       │                      ▼
       │               ┌──────────────┐
       │               │ Deserialize  │  50-100 ms
       │               └──────────────┘
       │                      │
       ├─ Cache Miss ─────────┤
       │                      │
       ▼                      ▼
┌──────────────┐       ┌──────────────┐
│ Build Fresh  │       │ Ready        │  Total: 50-100 ms (20-100x faster!)
└──────────────┘       └──────────────┘
       │
       ▼
┌──────────────┐
│ Serialize &  │
│ Cache        │
└──────────────┘
```

---

## 🔧 Technical Design

### Data Structures to Serialize

```rust
use serde::{Serialize, Deserialize};

/// Complete RETE network snapshot
#[derive(Serialize, Deserialize)]
pub struct ReteSnapshot {
    /// Schema version for migration
    pub version: u32,

    /// Timestamp of snapshot
    pub created_at: SystemTime,

    /// Hash of rules (for cache invalidation)
    pub rules_hash: u64,

    /// All alpha nodes
    pub alpha_nodes: Vec<AlphaNodeSnapshot>,

    /// All beta nodes
    pub beta_nodes: Vec<BetaNodeSnapshot>,

    /// All rules
    pub rules: Vec<TypedReteUlRuleSnapshot>,

    /// Working memory (optional, for crash recovery)
    pub working_memory: Option<WorkingMemorySnapshot>,

    /// Optimization indexes
    pub indexes: OptimizationSnapshot,
}

#[derive(Serialize, Deserialize)]
pub struct AlphaNodeSnapshot {
    pub id: usize,
    pub fact_type: String,
    pub conditions: Vec<ConditionSnapshot>,
    pub memory: Vec<TypedFacts>,
}

#[derive(Serialize, Deserialize)]
pub struct BetaNodeSnapshot {
    pub id: usize,
    pub left_parent: usize,
    pub right_parent: usize,
    pub join_conditions: Vec<JoinConditionSnapshot>,
    pub memory: Vec<TokenSnapshot>,
}

#[derive(Serialize, Deserialize)]
pub struct WorkingMemorySnapshot {
    pub facts: HashMap<FactHandle, TypedFacts>,
    pub next_handle: u64,
    pub retracted: HashSet<FactHandle>,
}

#[derive(Serialize, Deserialize)]
pub struct OptimizationSnapshot {
    /// Beta memory indexes
    pub beta_indexes: HashMap<String, BetaMemoryIndexSnapshot>,

    /// Alpha memory indexes (from future feature)
    pub alpha_indexes: HashMap<String, AlphaMemoryIndexSnapshot>,

    /// Node sharing registry
    pub node_sharing: NodeSharingSnapshot,
}
```

---

### Serialization Format Options

| Format | Pros | Cons | Verdict |
|--------|------|------|---------|
| **bincode** | Fast, compact | Not human-readable | ✅ **Primary** |
| **msgpack** | Compact, cross-language | Slower than bincode | 🔄 Alternative |
| **JSON** | Human-readable, debuggable | Large size, slow | 🛠️ Debug mode |
| **Protobuf** | Cross-language, compact | Complex setup | ❌ Overkill |

**Decision:** Use **bincode** for production, **JSON** for debugging

---

### Cache Invalidation Strategy

```rust
impl IncrementalEngine {
    /// Generate hash of current rules
    fn compute_rules_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        for rule in &self.rules {
            rule.name.hash(&mut hasher);
            // Hash rule content
        }
        hasher.finish()
    }

    /// Check if cached snapshot is valid
    fn is_cache_valid(&self, snapshot: &ReteSnapshot) -> bool {
        // Version check
        if snapshot.version != SNAPSHOT_VERSION {
            return false;
        }

        // Rules haven't changed
        if snapshot.rules_hash != self.compute_rules_hash() {
            return false;
        }

        // Snapshot not too old (configurable TTL)
        let age = SystemTime::now()
            .duration_since(snapshot.created_at)
            .unwrap_or(Duration::MAX);

        if age > Duration::from_secs(self.config.snapshot_ttl_seconds) {
            return false;
        }

        true
    }
}
```

---

### Incremental Updates

Instead of full snapshots, support incremental updates:

```rust
#[derive(Serialize, Deserialize)]
pub enum ReteUpdate {
    /// Add a new rule
    AddRule { rule: TypedReteUlRuleSnapshot },

    /// Remove a rule
    RemoveRule { rule_name: String },

    /// Insert fact
    InsertFact { handle: FactHandle, fact: TypedFacts },

    /// Update fact
    UpdateFact { handle: FactHandle, fact: TypedFacts },

    /// Retract fact
    RetractFact { handle: FactHandle },
}

pub struct ReteChangeLog {
    pub base_snapshot: ReteSnapshot,
    pub updates: Vec<ReteUpdate>,
}

impl IncrementalEngine {
    /// Apply change log on top of base snapshot
    fn load_with_changelog(&mut self, log: ReteChangeLog) -> Result<()> {
        // Deserialize base
        self.load_snapshot(log.base_snapshot)?;

        // Apply updates sequentially
        for update in log.updates {
            self.apply_update(update)?;
        }

        Ok(())
    }
}
```

**Benefits:**
- Smaller files (only changes, not full state)
- Faster saves (append-only)
- Audit trail (replay updates)

---

## 📐 Implementation Plan

### Stage 1: Basic Serialization (Week 1-2)

**Tasks:**
1. ✅ Add `serde` derive to all RETE structs
2. ✅ Implement `ReteSnapshot` struct
3. ✅ Implement `serialize()` and `deserialize()`
4. ✅ Unit tests for roundtrip (serialize → deserialize → compare)
5. ✅ Benchmark serialization speed

**Files to Modify:**
- `src/rete/alpha_node.rs` - Add `#[derive(Serialize, Deserialize)]`
- `src/rete/beta_node.rs` - Add `#[derive(Serialize, Deserialize)]`
- `src/rete/typed_facts.rs` - Add `#[derive(Serialize, Deserialize)]`
- Create `src/rete/persistence/mod.rs`

**Success Criteria:**
- Can serialize 10,000 rule RETE network
- Deserialized network produces same results
- Serialization time: <1 second

---

### Stage 2: File I/O & Caching (Week 3)

**Tasks:**
1. ✅ Implement `save_to_file()` and `load_from_file()`
2. ✅ Add cache path configuration
3. ✅ Implement cache invalidation (hash check)
4. ✅ Atomic file writes (tmp file + rename)
5. ✅ Handle file corruption gracefully

**Files to Create:**
- `src/rete/persistence/cache.rs`
- `src/rete/persistence/file_io.rs`

**Success Criteria:**
- Cache hit: 50-100ms load time
- Cache miss: Rebuild + cache for next time
- Corrupted cache: Fallback to rebuild

---

### Stage 3: Working Memory Persistence (Week 4)

**Tasks:**
1. ✅ Serialize `WorkingMemory`
2. ✅ Handle `FactHandle` correctly
3. ✅ Preserve TMS justifications
4. ✅ Test crash recovery scenario

**Files to Modify:**
- `src/rete/working_memory.rs` - Add serialization
- `src/rete/tms.rs` - Add serialization

**Success Criteria:**
- Can snapshot + restore 100K facts
- TMS state preserved correctly
- Recovery time: <500ms

---

### Stage 4: Incremental Updates (Week 5-6)

**Tasks:**
1. ✅ Implement `ReteUpdate` enum
2. ✅ Implement `ReteChangeLog`
3. ✅ Track updates since last snapshot
4. ✅ Compact changelog periodically
5. ✅ Benchmark: full snapshot vs incremental

**Success Criteria:**
- Incremental updates 10x faster than full snapshot
- Automatic compaction when log grows large

---

### Stage 5: Integration & Polish (Week 7-8)

**Tasks:**
1. ✅ Add `enable_persistence()` to `IncrementalEngine`
2. ✅ CLI flag: `--cache-dir <path>`
3. ✅ Metrics: cache hit rate, load time
4. ✅ Documentation and examples
5. ✅ Comprehensive benchmarks

**Files to Modify:**
- `src/rete/incremental_engine.rs` - Add persistence API

**Success Criteria:**
- Zero-config: works with sensible defaults
- Clear error messages
- Production-ready

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[test]
fn test_roundtrip_serialization() {
    let mut engine = IncrementalEngine::new();
    load_rules(&mut engine, "examples/rules/*.grl");

    // Serialize
    let snapshot = engine.snapshot().unwrap();
    let bytes = bincode::serialize(&snapshot).unwrap();

    // Deserialize
    let restored: ReteSnapshot = bincode::deserialize(&bytes).unwrap();

    // Compare
    assert_eq!(snapshot.rules_hash, restored.rules_hash);
    assert_eq!(snapshot.alpha_nodes.len(), restored.alpha_nodes.len());
}

#[test]
fn test_cache_invalidation() {
    let mut engine = IncrementalEngine::new();
    engine.enable_persistence("cache/");

    // First load: cache miss
    load_rules(&mut engine, "rules.grl");
    assert_eq!(engine.cache_stats().hit, 0);

    // Second load: cache hit
    let mut engine2 = IncrementalEngine::new();
    engine2.enable_persistence("cache/");
    load_rules(&mut engine2, "rules.grl");
    assert_eq!(engine2.cache_stats().hit, 1);

    // Modify rules: cache miss again
    modify_rules("rules.grl");
    let mut engine3 = IncrementalEngine::new();
    engine3.enable_persistence("cache/");
    load_rules(&mut engine3, "rules.grl");
    assert_eq!(engine3.cache_stats().hit, 1); // Still 1 (cache invalidated)
}
```

### Integration Tests

```rust
#[test]
fn test_crash_recovery() {
    let cache_dir = tempdir().unwrap();

    // Create engine, insert facts
    {
        let mut engine = IncrementalEngine::new();
        engine.enable_persistence(cache_dir.path());

        for i in 0..1000 {
            engine.insert("Order".to_string(), order(i));
        }

        engine.snapshot_to_disk().unwrap();
    } // Engine dropped (simulated crash)

    // Recover from snapshot
    {
        let mut engine2 = IncrementalEngine::new();
        engine2.enable_persistence(cache_dir.path());
        engine2.load_from_snapshot().unwrap();

        // All facts restored
        assert_eq!(engine2.working_memory().facts_count(), 1000);
    }
}
```

### Performance Benchmarks

```rust
#[bench]
fn bench_cold_start_10k_rules(b: &mut Bencher) {
    b.iter(|| {
        let mut engine = IncrementalEngine::new();
        load_rules(&mut engine, "10k_rules.grl");
    });
    // Expected: 5-8 seconds
}

#[bench]
fn bench_warm_start_10k_rules(b: &mut Bencher) {
    // Pre-populate cache
    let cache_dir = setup_cache("10k_rules.grl");

    b.iter(|| {
        let mut engine = IncrementalEngine::new();
        engine.enable_persistence(&cache_dir);
        engine.load_from_cache().unwrap();
    });
    // Expected: 50-100 ms (50-100x faster!)
}

#[bench]
fn bench_serialize_10k_rules(b: &mut Bencher) {
    let engine = create_engine_with_10k_rules();

    b.iter(|| {
        let snapshot = engine.snapshot().unwrap();
        bincode::serialize(&snapshot).unwrap()
    });
    // Expected: <1 second
}
```

---

## 🎯 Performance Targets

| Scenario | Cold Start | Warm Start (Cached) | Target Speedup |
|----------|------------|---------------------|----------------|
| 100 rules | 50 ms | 5 ms | **10x** |
| 1,000 rules | 500 ms | 20 ms | **25x** |
| 10,000 rules | 5 seconds | 50 ms | **100x** |
| 100,000 rules | 60 seconds | 500 ms | **120x** |
| Serialize 10K rules | - | <1 second | - |
| Deserialize 10K rules | - | <100 ms | - |

---

## 🚧 Challenges & Solutions

### Challenge 1: Closures Not Serializable
**Problem:** Rule actions are `Arc<dyn Fn>` closures - can't serialize
**Solution:**
- Don't serialize closures directly
- Store rule source (GRL) in snapshot
- Re-parse and recompile actions on load
- Alternative: Use rule compilation (separate feature)

### Challenge 2: File Format Versioning
**Problem:** RETE structure changes between versions
**Solution:**
- Include schema version in snapshot
- Write migration code for old versions
- Support multiple versions concurrently

### Challenge 3: Memory-Mapped Files
**Problem:** Large snapshots (100K rules) slow to load into memory
**Solution:**
- Use memory-mapped files (`memmap2` crate)
- Lazy deserialization (deserialize on demand)
- Zero-copy where possible

### Challenge 4: Concurrent Access
**Problem:** Multiple processes accessing same cache file
**Solution:**
- File locking (`fs2` crate)
- Process-local cache (in `/tmp`)
- Redis backend for distributed caching (future)

---

## 📊 Success Metrics

### Performance
- ✅ 10x minimum speedup for cached loads
- ✅ 100x maximum speedup for large rule sets
- ✅ Serialization time <1% of cold start time

### Reliability
- ✅ No data loss on crashes
- ✅ Graceful handling of corrupted cache
- ✅ 100% test coverage for serialization

### Usability
- ✅ Zero-config default behavior
- ✅ Clear cache hit/miss metrics
- ✅ CLI support for cache management

---

## 🔄 Future Enhancements (Phase 2)

### 1. Distributed Cache (Redis)
```rust
use redis::Commands;

impl RedisCache {
    fn save_snapshot(&self, snapshot: &ReteSnapshot) -> Result<()> {
        let mut conn = self.pool.get()?;
        let key = format!("rete:snapshot:{}", snapshot.rules_hash);
        let bytes = bincode::serialize(snapshot)?;
        conn.set_ex(key, bytes, self.ttl_seconds)?;
        Ok(())
    }
}
```

### 2. Time-Travel Debugging
```rust
// Snapshot at every rule firing
engine.enable_time_travel();

// Replay to specific point
engine.rewind_to(timestamp);
engine.rewind_to_rule_firing(42);
```

### 3. Snapshot Compression
```rust
use flate2::Compression;

// Compress snapshots (5-10x smaller)
let compressed = compress_snapshot(&snapshot, Compression::default());
```

### 4. Automatic Snapshots
```rust
// Snapshot every N updates
engine.auto_snapshot_every(1000);

// Snapshot every T seconds
engine.auto_snapshot_interval(Duration::from_secs(60));
```

---

## 🗓️ Timeline

**Total Duration:** 8 weeks

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1-2 | Serialization | Can serialize/deserialize RETE |
| 3 | File I/O | Cache to disk, load from cache |
| 4 | Working Memory | Crash recovery support |
| 5-6 | Incremental | Changelog-based updates |
| 7-8 | Polish | Integration, docs, benchmarks |

**Release:** v1.16.0 (Persistent RETE)

---

## ✅ Next Steps

1. **Start Stage 1** - Add `serde` to RETE structs
2. **Benchmark POC** - Validate 10x+ speedup
3. **Design cache invalidation** - Hash-based strategy

---

**Last Updated:** 2025-12-25
**Author:** Ton That Vu
**Status:** Ready for Implementation
