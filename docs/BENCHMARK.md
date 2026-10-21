# pixCAM v0.3 benchmark

Measured on the target Linux x86_64 workstation against the original synthetic SolidCraft specimen. Raw summarized measurements: [bridge-benchmark.json](bridge-benchmark.json).

| Scope | Samples | Median | p95 |
| --- | ---: | ---: | ---: |
| JSON parse, validation, recognition and report construction | 1,000 + 20 warmup | 0.211 ms | 0.326 ms |
| Fresh headless CLI/MCP capture including document load and consistency checks | 10 | 975.993 ms | 1,051.226 ms |

Input: 42,041 bytes, one 100 × 70 × 30 mm body, 24 topological faces, ten cylindrical patches and five merged cylindrical segments. Timing includes report construction but excludes serialization in the in-process recognition row. The capture row includes process startup, authentication-free isolated loading, four read-only tool types and local analysis; it is not the persistent connected-refresh latency.

```sh
cargo build --release --locked --offline
python3 scripts/bridge-benchmark.py
```

Run on an otherwise representative workstation; these are fixture measurements, not a throughput promise for large assemblies. Pairwise grouping can grow quadratically with cylinder count. GUI rendering, access proof, Booleans and stock removal are outside the recognition benchmark.

The retained `scripts/benchmark.mjs` measures full standalone showcase generation and remains available for build-focused comparisons.
