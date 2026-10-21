# pixCAM v0.2 benchmark

Measured on 2026-10-21 on the target development workstation using the release binary and 100 consecutive deterministic rebuilds into a temporary directory:

```json
{
  "runs": 100,
  "median_ms": 6.427,
  "p95_ms": 8.327,
  "min_ms": 5.886,
  "max_ms": 10.232,
  "output_files": 6,
  "output_bytes": 439956
}
```

Reproduce after `cargo build --locked --release`:

```sh
node scripts/benchmark.mjs 100
```

The measurement includes constructing the synthetic scene, parsing 27 NC motions, interpolating arcs, serialising JSON/CSV/SVG/DXF and embedding the complete offline HTML. It excludes Rust compilation, browser startup and video recording. Results are a local implementation benchmark, not a machining-time or cross-machine performance claim.
