# SolidCraft 0.17.0 integration

Verified on Linux on 2026-10-21 with the installed native GUI and CLI 0.17.0. The previous v0.2 missing-CLI blocker is resolved.

## Contract

The pinned [SolidCraft MCP documentation](https://github.com/tangency-labs/solidcraft/blob/v0.17.0/docs/mcp.md) defines newline-delimited JSON-RPC 2.0 over stdio. pixCAM negotiates protocol 2025-03-26, validates server identity/version, sends initialized and calls only four read-only tools. [Control authentication](https://github.com/tangency-labs/solidcraft/blob/v0.17.0/docs/control-protocol.md) remains inside the CLI: it locates the app's token itself.

The production executable path is `/home/torpix/Aplikacje/solidcraft-cli-0.17.0`. No PATH fallback or invocation of the older CLI is used. Different installations may change the single CLI constant and rebuild.

## Offline and isolated headless

Run from this repository:

```sh
cargo build --release --locked --offline
/home/torpix/Aplikacje/solidcraft-cli-0.17.0 --version
/home/torpix/Aplikacje/solidcraft-cli-0.17.0 run fixtures/bridge-part.json --save fixtures/bridge-part.solidcraft --results
./target/release/pixcam-showcase bridge --in fixtures/bridge-part.solidcraft
```

Document construction is a separate authorized synthetic workflow. The pixCAM MCP client itself has no execute/batch/save/open tool path. Headless bridge loading is configured once with --in; repeated browser refreshes reuse that read-only session. Restart reloads that same isolated input.

To capture/analyze explicitly:

```sh
./target/release/pixcam-showcase capture-solidcraft --in fixtures/bridge-part.solidcraft
./target/release/pixcam-showcase analyze-solidcraft fixtures/bridge-snapshot.json
./target/release/pixcam-showcase benchmark-solidcraft fixtures/bridge-snapshot.json
```

Capture prints sanitized snapshot JSON on stdout and generic errors on stderr. To regenerate the included synthetic capture and report after a debug build, run `node scripts/capture.mjs`. Do not save real user-document captures into the public repository.

## Active GUI

Start the app with an explicitly chosen unused localhost control port and our synthetic document:

```sh
/home/torpix/Aplikacje/solidcraft-0.17.0.AppImage --control PORT fixtures/bridge-part.solidcraft
./target/release/pixcam-showcase bridge --connect 127.0.0.1:PORT
```

PORT is a placeholder for your selected port, not a fixed default. The adapter's independent HTTP port is allocated dynamically. Open exactly the localhost URL it prints. Choose CONNECTED, then Refresh analysis. Browsers never access the SolidCraft control port.

For an already active user document, launch only the adapter with its known control port. No CAD mutation, document opening, screenshot tool, undo/redo or save is exposed. Reading is explicit; nothing is automatically exported to disk.

`--timeout-ms N` configures each MCP request deadline (default 15000). A failed capture is discarded; the bridge closes the child and performs at most one fresh initialized retry. A changed revision/body/timeline during capture rejects the result. The browser retains prior data with an explicit stale/disconnected status on failure.

## Acceptance evidence

`python3 scripts/live-gui-test.py` opens our specimen in a separate temporary app profile and discovers a free port dynamically. Set PIXCAM_PLAYWRIGHT to an installed Playwright module path when it is not resolvable as a package. `--record` also records the browser-only film.

The actual GUI test passed: document bridge-part, body BridgePlate, bbox 100/70/30 mm, ten cylinder patches merged to five segments, expected axes and diameters, named-boundary spans 30/12/5/25/15 mm, coaxial Ø12/Ø6, local-frame transform checks and Chromium CONNECTED refresh. Actual app exit produces HTTP 502/stale; restart recovers through the same adapter. An unused/wrong app port fails closed. The specimen's SHA-256 stays unchanged. Temporary control credentials are neither read by the test nor recorded.

The headless integration test is deliberately opt-in on machines without SolidCraft:

```sh
cargo test --locked --offline actual_headless_mcp_roundtrip_and_restart -- --ignored
```

## Schema and interpretation

Snapshots contain schema_version 1, solidcraft_version 0.17.0, inspect_design and body_topology, with optional measurements. The exact contract is in [snapshot schema](../schemas/solidcraft-snapshot.schema.json); the report contract is in [report schema](../schemas/feature-report.schema.json). Missing topology produces a partial-data warning; missing edges leave spans null.

Persistent edge tokens identify face adjacency. Axial range is the projection of those edges' endpoints/midpoints onto the cylinder axis. This measures available boundary extent; it does not certify a closed circumference or full drilling depth. Merge tolerance is 0.00001 mm for line/radius tests and 0.00001 for angular cross-product tests. Cylinder axes must already be unit length to 1e-8; invalid geometry fails closed.

SolidCraft's persistent bore/cbore naming supplies a hole hypothesis, never verified manufacturing intent. Coaxial segments with different diameters and touching axial intervals supply a possible counterbore hypothesis. Tool-direction setup candidates are sign-ambiguous geometric alignments; stock side and physical access remain unknown.

The separate legacy `import-solidcraft-snapshot` command retains its v0.2 timeline/cylinder import format. New recognition uses `analyze-solidcraft`. Neither path invents a timeline-feature-to-evaluated-face association. No plugin ABI, CAM operation tree, postprocessor or production NC is implemented.
