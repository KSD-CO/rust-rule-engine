# Streaming Architecture

## Overview

The streaming module provides production-ready real-time event processing with distributed state management, watermarking, and fault tolerance.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           STREAMING ARCHITECTURE                            │
└─────────────────────────────────────────────────────────────────────────────┘

                                INPUT LAYER
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   Event Sources:  IoT Sensors │ User Actions │ API Calls │ Message Queues   │
│                         │              │            │              │        │
│                         └──────────────┴────────────┴──────────────┘        │
│                                        │                                    │
│                              ┌─────────▼─────────┐                          │
│                              │   StreamEvent     │                          │
│                              │  - event_type     │                          │
│                              │  - data (JSON)    │                          │
│                              │  - timestamp      │                          │
│                              │  - source         │                          │
│                              └─────────┬─────────┘                          │
└────────────────────────────────────────┼────────────────────────────────────┘
                                         │
                    ┌────────────────────┼────────────────────┐
                    │                    │                    │
                    ▼                    ▼                    ▼

┌──────────────────────────────────────────────────────────────────────────────┐
│                            PROCESSING LAYER                                  │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐     │
│  │                      STREAM OPERATORS (Fluent API)                  │     │
│  ├─────────────────────────────────────────────────────────────────────┤     │
│  │                                                                     │     │
│  │  DataStream                                                         │     │
│  │    │                                                                │     │
│  │    ├─► filter()      ──► Predicate filtering                        │     │
│  │    ├─► map()         ──► Transform events                           │     │
│  │    ├─► flat_map()    ──► One-to-many transformation                 │     │
│  │    ├─► key_by()      ──► Partition by key ───► KeyedStream          │     │
│  │    ├─► window()      ──► Time-based windows ──► WindowedStream      │     │
│  │    └─► group_by()    ──► Group by field ────► GroupedStream         │     │
│  │                                                                     │     │
│  │  KeyedStream                                                        │     │
│  │    ├─► aggregate()   ──► Count, Sum, Average, Min, Max              │     │
│  │    └─► reduce()      ──► Custom aggregation                         │     │
│  │                                                                     │     │
│  │  WindowedStream                                                     │     │
│  │    ├─► Sliding Window   (overlapping)                               │     │
│  │    ├─► Tumbling Window  (non-overlapping)                           │     │
│  │    └─► Session Window   (gap-based)                                 │     │
│  │                                                                     │     │
│  └─────────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐     │
│  │                    WATERMARK & LATE DATA HANDLING                   │     │
│  ├─────────────────────────────────────────────────────────────────────┤     │
│  │                                                                     │     │
│  │  Watermark Strategies:                                              │     │
│  │    ├─► BoundedOutOfOrder    (max_delay tolerance)                   │     │
│  │    ├─► MonotonicAscending   (no tolerance)                          │     │
│  │    └─► Periodic             (interval-based)                        │     │
│  │                                                                     │     │
│  │  Late Data Strategies:                                              │     │
│  │    ├─► Drop                 (ignore late events)                    │     │
│  │    ├─► AllowedLateness      (accept within threshold)               │     │
│  │    ├─► SideOutput           (route to special stream)               │     │
│  │    └─► RecomputeWindows     (recalculate affected windows)          │     │
│  │                                                                     │     │
│  │  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐         │     │
│  │  │  On-time     │────►│  Watermark   │────►│  Process     │         │     │
│  │  │  Events      │     │  Generator   │     │  Stream      │         │     │
│  │  └──────────────┘     └──────────────┘     └──────────────┘         │     │
│  │         │                                                           │     │
│  │         ▼                                                           │     │
│  │  ┌──────────────┐     ┌──────────────┐                              │     │
│  │  │  Late        │────►│  Late Data   │────► [Drop|Allow|Side]       │     │
│  │  │  Events      │     │  Handler     │                              │     │
│  │  └──────────────┘     └──────────────┘                              │     │
│  │                                                                     │     │
│  └─────────────────────────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────────────────────────┘
                                         │
                                         ▼

┌──────────────────────────────────────────────────────────────────────────────┐
│                              STATE LAYER                                     │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐     │
│  │                        STATE MANAGEMENT                             │     │
│  ├─────────────────────────────────────────────────────────────────────┤     │
│  │                                                                     │     │
│  │   StatefulOperator ──► Maintains state across events                │     │
│  │         │                                                           │     │
│  │         ├─► put(key, value)                                         │     │
│  │         ├─► get(key) ──► Option<Value>                              │     │
│  │         ├─► update(key, value)                                      │     │
│  │         ├─► delete(key)                                             │     │
│  │         │                                                           │     │
│  │         └─► Checkpoint/Recovery                                     │     │
│  │                 ├─► checkpoint(name) ──► checkpoint_id              │     │
│  │                 └─► restore(checkpoint_id)                          │     │
│  │                                                                     │     │
│  └─────────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐     │
│  │                      STATE BACKENDS                                 │     │
│  ├─────────────────────────────────────────────────────────────────────┤     │
│  │                                                                     │     │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐      │     │
│  │  │   Memory    │    │    File     │    │       Redis         │      │     │
│  │  │   Backend   │    │   Backend   │    │      Backend        │      │     │
│  │  ├─────────────┤    ├─────────────┤    ├─────────────────────┤      │     │
│  │  │             │    │             │    │                     │      │     │
│  │  │  HashMap    │    │   JSON      │    │  Distributed State  │      │     │
│  │  │  in-memory  │    │   Files     │    │                     │      │     │
│  │  │             │    │             │    │  ┌─────────────┐    │      │     │
│  │  │  Fast       │    │  Persistent │    │  │  Instance 1 │    │      │     │
│  │  │  Ephemeral  │    │  Single     │    │  │  Instance 2 │────┼──────┼─────┼──► Redis Server
│  │  │             │    │  Machine    │    │  │  Instance 3 │    │      │     │   
│  │  │             │    │             │    │  └─────────────┘    │      │     │   
│  │  │  Dev/Test   │    │  Local      │    │                     │      │     │
│  │  │             │    │  Storage    │    │  • Connection Pool  │      │     │
│  │  │             │    │             │    │  • TTL Support      │      │     │
│  │  │             │    │             │    │  • Persistence      │      │     │
│  │  │             │    │             │    │  • Replication      │      │     │
│  │  │             │    │             │    │  • Clustering       │      │     │
│  │  │             │    │             │    │                     │      │     │
│  │  │             │    │             │    │  Production Ready   │      │     │
│  │  │             │    │             │    │                     │      │     │
│  │  └─────────────┘    └─────────────┘    └─────────────────────┘      │     │
│  │                                                                     │     │
│  └─────────────────────────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────────────────────────┘
                                         │
                                         ▼

┌──────────────────────────────────────────────────────────────────────────────┐
│                             OUTPUT LAYER                                     │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Results:  Aggregated Metrics │ Alerts │ Dashboards │ Database Writes       │
│             Actions │ Notifications │ Downstream Systems                     │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘


═══════════════════════════════════════════════════════════════════════════════
                              KEY FEATURES
═══════════════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────────────────┐
│  1. STREAM OPERATORS                                                        │
│     • 20+ fluent operators (filter, map, reduce, aggregate)                 │
│     • Built-in aggregators (Count, Sum, Average, Min, Max)                  │
│     • Custom operator support                                               │
│     • Type-safe transformations                                             │
│                                                                             │
│  2. WATERMARKING                                                            │
│     • Event-time processing                                                 │
│     • Out-of-order event handling                                           │
│     • Late data strategies                                                  │
│     • Side outputs for debugging                                            │
│                                                                             │
│  3. STATE MANAGEMENT                                                        │
│     • Multiple backends (Memory, File, Redis)                               │
│     • Distributed state with Redis                                          │
│     • Checkpointing for fault tolerance                                     │
│     • TTL-based state expiration                                            │
│                                                                             │
│  4. WINDOWING                                                               │
│     • Sliding windows (overlapping)                                         │
│     • Tumbling windows (non-overlapping)                                    │
│     • Session windows (gap-based)                                           │
│     • Custom window logic                                                   │
└─────────────────────────────────────────────────────────────────────────────┘


═══════════════════════════════════════════════════════════════════════════════
                            DEPLOYMENT SCENARIOS
═══════════════════════════════════════════════════════════════════════════════

┌──────────────────────────────────────┐
│   SINGLE INSTANCE (Development)      │
├──────────────────────────────────────┤
│                                      │
│  ┌──────────────────────┐            │
│  │   Stream Processor   │            │
│  │                      │            │
│  │   Memory Backend     │            │
│  │   (Fast, Ephemeral)  │            │
│  └──────────────────────┘            │
│                                      │
│  Use Case: Development, Testing      │
└──────────────────────────────────────┘


┌──────────────────────────────────────┐
│   SINGLE MACHINE (Small Production)  │
├──────────────────────────────────────┤
│                                      │
│  ┌──────────────────────┐            │
│  │   Stream Processor   │            │
│  │          │           │            │
│  │          ▼           │            │
│  │   File Backend       │            │
│  │   (Persistent)       │            │
│  └──────────────────────┘            │
│                                      │
│  Use Case: Small-scale production    │
└──────────────────────────────────────┘


┌─────────────────────────────────────────────────────────────┐
│   DISTRIBUTED (Large Scale Production)                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐     │
│  │  Instance 1  │   │  Instance 2  │   │  Instance N  │     │
│  │              │   │              │   │              │     │
│  │  Processor   │   │  Processor   │   │  Processor   │     │
│  └──────┬───────┘   └──────┬───────┘   └──────┬───────┘     │
│         │                  │                  │             │
│         └──────────────────┼──────────────────┘             │
│                            │                                │
│                            ▼                                │
│                   ┌────────────────┐                        │
│                   │  Redis Cluster │                        │
│                   │                │                        │
│                   │  • Shared State│                        │
│                   │  • Replication │                        │
│                   │  • Persistence │                        │
│                   │  • Sharding    │                        │
│                   └────────────────┘                        │
│                                                             │
│  Benefits:                                                  │
│    ✓ Horizontal scaling                                     │
│    ✓ High availability                                      │
│    ✓ Fault tolerance                                        │
│    ✓ 100k+ ops/sec                                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘


═══════════════════════════════════════════════════════════════════════════════
                            PERFORMANCE METRICS
═══════════════════════════════════════════════════════════════════════════════

Backend       Throughput      Latency      Scalability    Persistence
─────────────────────────────────────────────────────────────────────────
Memory        1M+ ops/sec     < 1μs        Single         ❌
File          10k ops/sec     1-10ms       Single         ✅
Redis         100k+ ops/sec   < 1ms        Distributed    ✅
Redis Cluster 1M+ ops/sec     < 1ms        Horizontal     ✅


═══════════════════════════════════════════════════════════════════════════════
                              USE CASES
═══════════════════════════════════════════════════════════════════════════════

1. IoT Monitoring
   ├─► Sensor data ingestion
   ├─► Anomaly detection
   ├─► Real-time alerts
   └─► Windowed aggregations

2. Financial Trading
   ├─► Price tick processing
   ├─► Risk calculations
   ├─► Order matching
   └─► Market analytics

3. User Behavior Analytics
   ├─► Session tracking
   ├─► Clickstream analysis
   ├─► Conversion funnels
   └─► A/B testing

4. System Monitoring
   ├─► Log aggregation
   ├─► Metric collection
   ├─► Performance tracking
   └─► SLA monitoring
