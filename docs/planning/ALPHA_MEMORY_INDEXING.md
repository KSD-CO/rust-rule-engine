# Alpha Memory Indexing

**Status**: Planning Phase
**Priority**: High
**Estimated Impact**: 10-100x speedup for fact filtering
**Complexity**: Medium
**Dependencies**: None (complements existing Beta Memory Indexing)

---

## 📋 Executive Summary

Add hash-based indexing to Alpha Memory for O(1) fact lookups instead of O(n) linear scans. This complements Beta Memory Indexing and completes the RETE optimization stack.

**Current Problem:**
- Alpha nodes scan all facts linearly to find matches: O(n)
- With 10,000 facts, filtering by `status == "active"` requires 10,000 comparisons
- CPU cache misses, wasted cycles on irrelevant facts

**Proposed Solution:**
- Build hash indexes on frequently-filtered fields
- O(1) lookup: `index.get("status", "active")` → instant results
- Automatic index maintenance on insert/update/retract

---

## 🎯 Goals

### Primary Goals
1. **10-100x Speedup** - For alpha node filtering operations
2. **Automatic Indexing** - Auto-detect high-selectivity fields
3. **Zero API Changes** - Transparent to users
4. **Memory Efficient** - Only index high-value fields

### Non-Goals (Phase 1)
- ❌ Multi-field composite indexes (future enhancement)
- ❌ Range indexes (B-tree, for `>`, `<` operations)
- ❌ Full-text search indexes

---

## 🏗️ Architecture

### Current Alpha Memory (No Index)

```rust
pub struct AlphaMemory {
    facts: Vec<TypedFacts>,  // Linear scan: O(n)
}

impl AlphaMemory {
    fn filter(&self, field: &str, value: &FactValue) -> Vec<&TypedFacts> {
        self.facts.iter()
            .filter(|f| f.get(field) == Some(value))  // O(n) scan
            .collect()
    }
}
```

**Performance:** With 10,000 facts → 10,000 comparisons

---

### Proposed Alpha Memory (With Index)

```rust
pub struct AlphaMemoryIndex {
    // Primary storage
    facts: Vec<TypedFacts>,

    // Hash indexes: field_name → (value → Vec<fact_idx>)
    indexes: HashMap<String, HashMap<String, Vec<usize>>>,

    // Index statistics for auto-tuning
    stats: IndexStats,
}

impl AlphaMemoryIndex {
    fn filter(&self, field: &str, value: &FactValue) -> Vec<&TypedFacts> {
        // O(1) index lookup if index exists
        if let Some(index) = self.indexes.get(field) {
            let key = format!("{:?}", value);
            if let Some(indices) = index.get(&key) {
                return indices.iter()
                    .map(|&i| &self.facts[i])
                    .collect();  // O(k) where k = result size
            }
        }

        // Fallback to linear scan if no index
        self.facts.iter()
            .filter(|f| f.get(field) == Some(value))
            .collect()
    }
}
```

**Performance:** With index → 1 hash lookup → O(1)

---

## 🔧 Technical Design

### Data Structures

```rust
/// Alpha Memory with automatic indexing
pub struct AlphaMemoryIndex {
    /// All facts stored sequentially
    facts: Vec<TypedFacts>,

    /// Indexes: field → (value → [fact indices])
    /// Example: "status" → { "active" → [0, 5, 12], "pending" → [1, 3] }
    indexes: HashMap<String, HashMap<String, Vec<usize>>>,

    /// Track which fields are indexed
    indexed_fields: HashSet<String>,

    /// Statistics for index effectiveness
    stats: IndexStats,
}

#[derive(Default)]
pub struct IndexStats {
    /// How many times each field was queried
    query_counts: HashMap<String, usize>,

    /// Selectivity: % of facts matching (lower = better for indexing)
    selectivity: HashMap<String, f64>,

    /// Memory used by indexes
    index_memory_bytes: usize,
}

impl AlphaMemoryIndex {
    /// Insert fact and update indexes
    pub fn insert(&mut self, fact: TypedFacts) -> usize {
        let idx = self.facts.len();

        // Update all indexes
        for field_name in &self.indexed_fields {
            if let Some(value) = fact.get(field_name) {
                let key = format!("{:?}", value);
                self.indexes
                    .entry(field_name.clone())
                    .or_insert_with(HashMap::new)
                    .entry(key)
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }

        self.facts.push(fact);
        idx
    }

    /// Create index on a field
    pub fn create_index(&mut self, field: String) {
        if self.indexed_fields.contains(&field) {
            return; // Already indexed
        }

        let mut index = HashMap::new();

        // Build index from existing facts
        for (idx, fact) in self.facts.iter().enumerate() {
            if let Some(value) = fact.get(&field) {
                let key = format!("{:?}", value);
                index.entry(key)
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }

        self.indexes.insert(field.clone(), index);
        self.indexed_fields.insert(field);
    }

    /// Auto-detect which fields to index
    pub fn auto_tune(&mut self) {
        // Index fields that are:
        // 1. Frequently queried (>100 queries)
        // 2. High selectivity (<10% match rate)

        for (field, count) in &self.stats.query_counts {
            if *count > 100 {
                if let Some(&selectivity) = self.stats.selectivity.get(field) {
                    if selectivity < 0.1 {  // <10% of facts match
                        self.create_index(field.clone());
                    }
                }
            }
        }
    }
}
```

---

## 📐 Implementation Plan

### Stage 1: Basic Index Implementation (Week 1)

**Tasks:**
1. ✅ Create `src/rete/alpha_memory_index.rs`
2. ✅ Implement `AlphaMemoryIndex` struct
3. ✅ Implement `insert()` with index maintenance
4. ✅ Implement `filter()` with index lookup
5. ✅ Implement `create_index()` manual indexing
6. ✅ Unit tests

**Files:**
- `src/rete/alpha_memory_index.rs` (new)
- `tests/alpha_memory_index_test.rs` (new)

**Success Criteria:**
- Can create index on any field
- Filter uses index when available
- 10x+ speedup for indexed fields

---

### Stage 2: Index Maintenance (Week 2)

**Tasks:**
1. ✅ Implement `update()` - update indexes when fact changes
2. ✅ Implement `retract()` - remove from indexes
3. ✅ Handle multi-valued fields correctly
4. ✅ Track index memory usage
5. ✅ Integration tests

**Success Criteria:**
- Indexes stay consistent after updates/retracts
- No memory leaks

---

### Stage 3: Auto-Tuning (Week 3)

**Tasks:**
1. ✅ Implement `IndexStats` tracking
2. ✅ Track query frequency per field
3. ✅ Calculate selectivity (% of facts matching)
4. ✅ Implement `auto_tune()` - automatically create useful indexes
5. ✅ Drop unused indexes to save memory
6. ✅ Benchmark: auto vs manual indexing

**Success Criteria:**
- Automatically indexes frequently-queried, high-selectivity fields
- Minimal memory overhead (<10% of fact storage)

---

### Stage 4: RETE Integration (Week 4)

**Tasks:**
1. ✅ Modify `AlphaNode` to use `AlphaMemoryIndex`
2. ✅ Replace linear scans with indexed lookups
3. ✅ Benchmark RETE with alpha indexing
4. ✅ Compare: RETE + Alpha Index vs RETE + Beta Index vs Both

**Files to Modify:**
- `src/rete/alpha_node.rs`
- `src/rete/incremental_engine.rs`

**Success Criteria:**
- RETE uses alpha indexes transparently
- Combined with Beta indexing: 100-1000x total speedup

---

### Stage 5: Advanced Features (Week 5)

**Tasks:**
1. ✅ Support for `!=` (not-equal) queries
2. ✅ Support for `contains` (substring/multifield)
3. ✅ Composite indexes (index on 2+ fields together)
4. ✅ Range indexes (B-tree for `>`, `<`, `>=`, `<=`)
5. ✅ Null-aware indexing

**Success Criteria:**
- Handles all GRL condition types
- Smart index selection for complex queries

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[test]
fn test_basic_index() {
    let mut mem = AlphaMemoryIndex::new();
    mem.create_index("status".to_string());

    // Insert 1000 facts
    for i in 0..1000 {
        let mut fact = TypedFacts::new();
        fact.set("id", i);
        fact.set("status", if i % 10 == 0 { "active" } else { "pending" });
        mem.insert(fact);
    }

    // Query: should use index
    let active = mem.filter("status", &FactValue::String("active".to_string()));
    assert_eq!(active.len(), 100);  // 10% are active
}

#[test]
fn test_index_maintenance() {
    let mut mem = AlphaMemoryIndex::new();
    mem.create_index("status".to_string());

    let idx = mem.insert(fact("active"));
    assert_eq!(mem.filter("status", "active").len(), 1);

    // Update
    mem.update(idx, fact("pending"));
    assert_eq!(mem.filter("status", "active").len(), 0);
    assert_eq!(mem.filter("status", "pending").len(), 1);

    // Retract
    mem.retract(idx);
    assert_eq!(mem.filter("status", "pending").len(), 0);
}
```

### Performance Benchmarks

```rust
#[bench]
fn bench_linear_scan_10k(b: &mut Bencher) {
    let mem = create_facts(10_000);
    b.iter(|| {
        mem.facts.iter()
            .filter(|f| f.get("status") == Some("active"))
            .count()
    });
    // Expected: ~500 µs
}

#[bench]
fn bench_indexed_lookup_10k(b: &mut Bencher) {
    let mut mem = AlphaMemoryIndex::new();
    mem.create_index("status".to_string());
    populate(&mut mem, 10_000);

    b.iter(|| {
        mem.filter("status", &FactValue::String("active".to_string())).len()
    });
    // Expected: ~5 µs (100x faster)
}
```

---

## 🎯 Performance Targets

| Scenario | Linear Scan | Indexed | Target Speedup |
|----------|-------------|---------|----------------|
| 1,000 facts, 10% selectivity | 50 µs | 2 µs | **25x** |
| 10,000 facts, 10% selectivity | 500 µs | 5 µs | **100x** |
| 100,000 facts, 1% selectivity | 5 ms | 10 µs | **500x** |
| Update fact (indexed) | N/A | 2 µs | - |
| Auto-tune decision | N/A | 100 µs | - |

---

## 🚧 Challenges & Solutions

### Challenge 1: Index Memory Overhead
**Problem:** Indexes consume memory (field → value → [indices])
**Solution:**
- Only index high-selectivity fields (<10% match rate)
- Drop unused indexes after 1000 queries without use
- Estimate: ~20% memory overhead for typical workloads

### Challenge 2: Update Performance
**Problem:** Updates require updating indexes (slower inserts)
**Solution:**
- Batch updates when possible
- Lazy index updates (defer until next query)
- Expected: <5% insert overhead

### Challenge 3: Choosing What to Index
**Problem:** User doesn't know which fields to index
**Solution:**
- Auto-tuning based on query patterns
- Provide `engine.index_stats()` for visibility
- Default: index all equality-tested fields

### Challenge 4: Multi-Valued Fields
**Problem:** Multifield variables can have multiple values
**Solution:**
- Index each value separately
- Example: `tags: ["a", "b"]` → index["tags"]["a"] + index["tags"]["b"]

---

## 📊 Success Metrics

### Performance
- ✅ 10x minimum speedup for indexed queries
- ✅ 100x maximum speedup for high-selectivity queries
- ✅ <10% insert overhead
- ✅ <20% memory overhead

### Usability
- ✅ Zero-config auto-tuning works for 80% of cases
- ✅ Manual indexing available for advanced users
- ✅ `engine.index_stats()` shows index effectiveness

### Compatibility
- ✅ 100% backward compatible
- ✅ Works with existing RETE features
- ✅ Combines with Beta Memory Indexing

---

## 🔄 Integration with Existing Features

### Combine with Beta Memory Indexing

**Before (Beta Indexing Only):**
```
1. Alpha: Scan 10,000 facts → 1,000 match (500 µs)
2. Beta: Join 1,000 x 1,000 with index → 10,000 results (1 ms)
Total: 1.5 ms
```

**After (Alpha + Beta Indexing):**
```
1. Alpha: Index lookup → 1,000 match (5 µs)
2. Beta: Join 1,000 x 1,000 with index → 10,000 results (1 ms)
Total: 1.005 ms (1.5x faster overall)
```

**With High Selectivity (1% match rate):**
```
Before:
1. Alpha: Scan 10,000 → 100 match (500 µs)
2. Beta: Join 100 x 100 → 1,000 results (100 µs)
Total: 600 µs

After:
1. Alpha: Index → 100 match (5 µs)
2. Beta: Join 100 x 100 → 1,000 results (100 µs)
Total: 105 µs (5.7x faster)
```

---

## 🔄 Future Enhancements (Phase 2)

### 1. Composite Indexes
```rust
// Index on multiple fields together
mem.create_composite_index(&["customer_id", "status"]);

// Single lookup instead of two sequential lookups
let results = mem.filter_composite(&[
    ("customer_id", "C123"),
    ("status", "active"),
]);
```

### 2. Range Indexes (B-Tree)
```rust
// For >, <, >=, <= queries
mem.create_range_index("amount");

// Efficient range query
let high_value = mem.filter_range("amount", 1000.0, f64::MAX);
```

### 3. Full-Text Search
```rust
// For contains, startsWith, endsWith
mem.create_text_index("description");

let matches = mem.search("description", "premium customer");
```

### 4. Probabilistic Indexes (Bloom Filters)
```rust
// Space-efficient negative lookups
mem.create_bloom_filter("rare_field", 0.01);  // 1% false positive rate

if !mem.might_contain("rare_field", "value") {
    return vec![];  // Definitely not present
}
```

---

## 🗓️ Timeline

**Total Duration:** 5 weeks

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1 | Basic Index | Create/query indexes work |
| 2 | Maintenance | Update/retract maintain indexes |
| 3 | Auto-Tuning | Automatic index creation |
| 4 | RETE Integration | Works with IncrementalEngine |
| 5 | Polish | Benchmarks, docs, examples |

**Release:** v1.14.0 (alongside Rule Compilation)

---

## ✅ Next Steps

1. **Start Stage 1** - Implement basic `AlphaMemoryIndex`
2. **Benchmark POC** - Validate 10x+ speedup
3. **Design API** - How users enable/configure indexing

---

**Last Updated:** 2025-12-25
**Author:** Ton That Vu
**Status:** Ready for Implementation
