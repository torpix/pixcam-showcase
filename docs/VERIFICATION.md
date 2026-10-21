# Verification — 2026-10-21

## Acceptance gates and observed results

| Gate | Evidence |
|---|---|
| Original synthetic inputs only | New Aurora definition and NC; reviewed generic Rust reuse; source repository/history excluded |
| Rust behavior | 20 passing tests: 4×4 matrices/inverses/rotation order, four setups, geometry, exports, parser modes/arcs/rejections, SolidCraft snapshot adapter, MCP and actual stdio |
| Lint | `cargo clippy --locked --offline --all-targets -- -D warnings` passes |
| Formatting | `cargo fmt --check` passes |
| Independent schema check | Python `jsonschema` checks all three input/output schema pairs and six real MCP tool results, including unsupported NC and invalid transform |
| Browser | Chromium + Playwright: four animated setups; five frames; matrix and local/global coordinate changes; 9 features; 27 clickable motions; interpolated arc details; scrub/playback; keyboard orbit; four byte-equal downloads; PNG; 390 px layout; SVG parsing |
| Offline operation | No HTTP/HTTPS requests or JavaScript exceptions during browser verification; all exports work when opened as `file://` |
| Review media | Three original v0.2 full-page PNGs; viewport-only H.264 movie, 1440 × 1280, 10 fps, exactly 72.000 seconds by FFprobe |
| Benchmark | 100 release rebuilds: median 6.427 ms, p95 8.327 ms; six files, 439,956 bytes |
| SolidCraft boundary | v0.17.0 snapshot adapter contract tested with synthetic topology; live active-document bridge blocked because `solidcraft-cli` is not installed |
| Public boundary | Explicit release allowlist, text privacy scan, synthetic-data assertions and new root Git history; see PRIVACY.md |

Before public implementation, the original local project also passed its existing **36 tests** and Clippy. Those private-data tests and outputs were not copied into this repository. The public suite uses new analytic and synthetic cases, plus generic parser cases with no private inputs. The original workspace remains unchanged.

The first public test run found a floating-point seam in closed circle outlines; the generator now reuses the first point as the last point. Browser verification also caught `file://` link downloads navigating instead of downloading; exports are now embedded as strings and downloaded as local Blob objects. Assertions were retained and the failing cases were rerun successfully.

Visual review caught an incorrect elevation in the Top preset; it now projects XY with no Z displacement, covered by a browser assertion. Completed NC segments retain their motion-type colors rather than becoming white. Final screenshots and the movie were regenerated after these fixes.

## Reproduce

```sh
cargo fmt --check
cargo test --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo run --locked --offline -- build dist
python3 scripts/schema-test.py
node scripts/browser-test.mjs /absolute/path/to/playwright/index.mjs
node scripts/privacy-check.mjs --history
node scripts/benchmark.mjs 100
ffprobe -v error -show_entries format=duration:stream=codec_name,width,height,r_frame_rate -of json media/pixcam-coordinate-inspector-72s.mp4
```

Rust dependencies must already be cached for `--offline`; omit it on a fresh machine. Schema verification needs the Python `jsonschema` package; browser checks need Playwright and Chromium. Set `CHROMIUM` if it is not `/usr/bin/chromium`. These are development checks, not runtime dependencies of the delivered HTML. Browser tests overwrite their three named screenshots and test scratch downloads. Video recreation is documented in WALKTHROUGH.md.

`privacy-check.mjs --history` is deliberately strict for this initial release: exactly one root commit, no remote or alternates, a clean worktree, all release paths allowlisted. For future releases, review and deliberately update that policy rather than weakening it to pass.

## Verification limits

No real-machine run, stock removal, cutting-data validation, live SolidCraft active-document connection, SolidCraft plugin compilation, external MCP-client UI test or machining-accuracy certification was performed. Browser checks cover Chromium, not every browser engine. The recorder samples browser frames; it is not evidence of machining time. Raster/video privacy relies on controlled capture provenance and visual review as well as file checks.
