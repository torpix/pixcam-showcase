# pixCAM v0.3 verification

Verified locally on Linux, 2026-10-21. No push was performed.

## Rust

- Public: 29 tests pass with `cargo test --locked --offline`; one installed-CAD test is ignored by default and passes when explicitly run.
- Real headless test: two independent MCP initialize/capture/shutdown cycles against the native 0.17.0 CLI and synthetic document.
- Private enclosing engine: all 37 regression tests pass, with its original inputs kept private.
- Both crates pass format and Clippy with warnings denied.

```sh
cargo test --locked --offline
cargo test --locked --offline actual_headless_mcp_roundtrip_and_restart -- --ignored
cargo fmt --check
cargo clippy --locked --offline --all-targets -- -D warnings
```

Fault injection covers timeout, process EOF, malformed JSON, wrong response ID, mixed result/error, incompatible version, JSON-RPC errors, tool errors and mutation denial. Endpoints reject non-localhost addresses and mutually exclusive input/connect configuration. Raw CAD diagnostics are suppressed.

Recognition checks document/body/bbox, ten patches → five segments, axes, origins, diameters, measured boundary spans, coaxial steps, right-handed frames, deterministic repeatability, invalid geometry, unknown external surfaces and absent-edge null spans.

## Chromium

Build the debug binary and dist first. Supply an installed Playwright module path as the final argument if needed.

```sh
cargo build --locked --offline
./target/debug/pixcam-showcase build dist
node scripts/browser-test.mjs /absolute/path/to/playwright/index.mjs
node scripts/bridge-browser-test.mjs /absolute/path/to/playwright/index.mjs
python3 scripts/live-gui-test.py
```

The legacy browser suite passes four setups, five frames, matrices, nine annotations, 27 clickable motions, interpolated arcs, all original downloads, PNG, mobile layout, SVG and zero network/JS errors.

The bridge suite passes offline DEMO/IMPORT with zero HTTP requests; exact Rust/JS feature/group/operation parity for the captured specimen; current-document report/CSV/snapshot/SVG/PNG exports; side-axis alignment after Y90°; diameter/coaxial groups; invalid and incomplete snapshots; no-CAD adapter guidance; mobile layout; real CONNECTED refresh; hostile Origin, Host, missing session and generic-tool-route denial; repeated refresh; disconnect and explicitly stale retained data.

The live harness passed against an actual isolated SolidCraft 0.17.0 GUI via --connect. It creates a temporary profile/control directory, uses a dynamically selected control port and checks the synthetic file hash. Actual app exit returns HTTP 502/stale; restarting that app recovers a fresh MCP capture through the same adapter. An unused app port fails closed. The harness closes only its own GUI and never records a desktop, active user window or credentials.

## Schemas, media and privacy

```sh
python3 scripts/schema-test.py
python3 scripts/geometry-schema-test.py
node scripts/privacy-check.mjs --history
```

Six original MCP schema/result examples pass. New draft 2020-12 snapshot/report schemas validate actual capture and browser export, with negative-radius and invalid-evidence rejection. Rust supplies numerical/cross-record constraints that JSON Schema cannot express.

Three v0.3 screenshots were captured from the running browser: DEMO, IMPORT surface map and actual GUI CONNECTED. The new film contains 360 actual browser frames, H.264, 1600 × 1200, 10 fps, exactly 36 seconds; its connection uses the isolated GUI, not a simulated status. Regression tests also refresh the three legacy-laboratory screenshots within the current page; the original 72-second v0.2 film remains historical.

Privacy checks cover the explicit release allowlist, public synthetic geometry, native CAD JSON members, known native-cache hash, secret/path markers, no remote/alternates and all local Git objects. Image/video signatures are complemented by visual review and isolated capture provenance.

The live GUI and headless results apply to this small synthetic specimen. No real machine, certified tool access, arbitrary-assembly performance, NC generation or CYCLE800 acceptance is claimed.
