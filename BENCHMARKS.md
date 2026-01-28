# Sandwich Rust - Performance Benchmarks

**Date:** 2026-01-28 15:27:48

## Results

| Test | Iterations | Total (ms) | Avg (ms) | Min (ms) | Max (ms) |
|------|------------|------------|----------|----------|----------|
| Basic (1 item) | 10 | 12796.95ms | 1279.69ms | 1262.76ms | 1296.74ms |
| Full outfit (3 items) | 10 | 12671.10ms | 1267.11ms | 1259.12ms | 1280.08ms |
| Complex outfit (5 items) | 10 | 12739.52ms | 1273.95ms | 1259.93ms | 1297.72ms |
| Back view (2 items) | 10 | 12596.22ms | 1259.62ms | 1254.57ms | 1267.86ms |
| Cache hit | 100 | 0.09ms | 0.00ms | 0.00ms | 0.00ms |

## System Information

- **OS:** macos
- **Architecture:** aarch64
- **Rust Version:** See Cargo.toml
- **Build:** Development (unoptimized)

## Notes

- All tests run with local filesystem storage
- Images loaded from local resources directory
- Cache tests include memory and filesystem caching
- Times include full pipeline: parsing, fetching, and compositing
- **IMPORTANT:** These are development build times (unoptimized)
- Release builds (`cargo build --release`) are typically 3-5x faster

## Key Findings

- **Average composition time:** ~1.28 seconds (dev build)
- **Cache retrieval:** <1 microsecond (extremely fast!)
- **Number of layers:** Minimal impact on performance (1 item vs 5 items is nearly the same)
- **Platform:** Apple Silicon (M-series chip), macOS

## Running Optimized Benchmarks

For production-level performance metrics, run with release optimizations:

```bash
cargo build --release --bin sandwich-cli
./target/release/sandwich-cli --local /path/to/resources bench --output BENCHMARKS-RELEASE.md
```

Expected improvements with release build:
- Composition: ~300-400ms (vs ~1280ms dev)
- 3-5x faster overall performance
- Better memory efficiency
