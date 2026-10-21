# pixCAM v0.4.1 · Parametric Cycles & SolidCraft Live Bridge

pixCAM is an MIT-licensed, Linux-focused showcase for parametric position
patterns, coordinate inspection, ISO-NC visualization, and read-only CAD
geometry capture from SolidCraft.

**[Launch the interactive demo](https://torpix.github.io/pixcam-showcase/)** ·
**[Releases](https://github.com/torpix/pixcam-showcase/releases)** ·
**[Changelog](CHANGELOG.md)**

## Quick start: offline DEMO

The standalone demo needs no Rust, SolidCraft, GPU, server, or network request:

```sh
git clone https://github.com/torpix/pixcam-showcase.git
cd pixcam-showcase
xdg-open dist/index.html
```

Opening the file manually in a browser is equivalent. DEMO uses only the
included synthetic Aurora and BridgePlate data. IMPORT analyzes a local pixCAM
snapshot/report in the browser; it uploads nothing.

To build and test the Rust CLI, install Rust 1.95 or newer, Python 3.11+ and
Node.js 20+:

```sh
cargo build --release --locked
cargo test --locked
./target/release/pixcam-showcase cycle fixtures/cycle-circular.json cycle-output
./target/release/pixcam-showcase cycle fixtures/cycle-rectangular.json grid-output
```

The cycle commands create JSON, CSV, SVG, and DXF exports from synthetic
fixtures. Full circles omit a duplicate closing point; open arcs include both
ends. See [cycle conventions](docs/CYCLES.md).

## CONNECTED: optional SolidCraft bridge

CONNECTED mode is optional and requires the separately installed SolidCraft
CLI **0.17.0**. pixCAM selects the executable in one unambiguous order:

1. `--solidcraft-cli PATH`;
2. `PIXCAM_SOLIDCRAFT_CLI`;
3. the first executable named `solidcraft-cli` on `PATH`.

Paths containing spaces are passed directly to the operating system; do not add
shell quotes inside an environment-variable value.

```sh
# Explicit path (highest priority)
./target/release/pixcam-showcase bridge \
  --solidcraft-cli "/opt/SolidCraft 0.17.0/solidcraft-cli" \
  --in fixtures/bridge-part.solidcraft

# Or configure once for the current shell
export PIXCAM_SOLIDCRAFT_CLI="/opt/SolidCraft 0.17.0/solidcraft-cli"
./target/release/pixcam-showcase bridge --in fixtures/bridge-part.solidcraft
```

Open the printed `http://127.0.0.1:.../` URL, choose CONNECTED, and refresh.
For an already running SolidCraft app with an explicitly configured control
port:

```sh
./target/release/pixcam-showcase bridge --connect 127.0.0.1:PORT
```

The bridge requires MCP server `solidcraft` 0.17.0 and protocol 2025-03-26.
It rejects a missing/bad executable, SolidCraft 0.16.x or 0.18.x, a different
protocol, timeouts, disconnects, malformed JSON, and non-loopback control
addresses with explicit errors. A real bridge smoke test is deliberately
opt-in:

```sh
PIXCAM_SOLIDCRAFT_CLI="/opt/SolidCraft 0.17.0/solidcraft-cli" \
  cargo test --locked actual_headless_mcp_roundtrip_and_restart -- --ignored
```

See the complete [SolidCraft setup and troubleshooting guide](docs/SOLIDCRAFT.md).

## Architecture and safety boundary

```text
SolidCraft 0.17.0
  └─ solidcraft-cli mcp (stdio JSON-RPC 2.0)
       └─ pixCAM Rust read-only allowlist
            └─ sanitized geometry snapshot / recognition
                 └─ 127.0.0.1 HTTP adapter
                      └─ browser CONNECTED view and exports
```

The MCP allowlist is exactly `list_commands`, `inspect_design`, `measure`, and
`body_topology`. There is no execute, batch, open, save, undo/redo, generic tool
proxy, CORS endpoint, or non-loopback bind. SolidCraft credentials remain in its
CLI and are never sent to the browser. Captured snapshots omit document paths,
arbitrary metadata, and raw server diagnostics.

Generated cycle positions are `GENERATED_GEOMETRY`; measured CAD surfaces are
`VERIFIED_GEOMETRY`; manufacturing interpretations remain `INFERRED_FEATURE` or
`UNKNOWN`. pixCAM does not claim production cutting simulation, collision or
access proof, stock removal, machine kinematics, postprocessing, certified
drilling depth, or production NC generation.

## Features

- Circular and rectangular position patterns with arbitrary rigid 4×4 frames.
- JSON/CSV/SVG/DXF cycle exports and a combined CAD/generated registry.
- Read-only SolidCraft geometry capture and cylindrical-surface analysis.
- Coordinate Inspector with five frames and geometric 3+2 orientations.
- ISO-NC viewer for the included illustrative synthetic program.
- Current-document JSON/CSV/SVG/PNG exports in the bridge view.

The existing `v0.4.0-rc.1` release remains available and unchanged. v0.4.1 is
a portability, CI, documentation, and security-hardening update, not a new CAM
module.

## Verify locally

```sh
cargo build --locked
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
python3 -m pip install -r requirements-ci.txt
python3 scripts/schema-test.py
python3 scripts/geometry-schema-test.py
python3 scripts/cycle-schema-test.py
node scripts/privacy-check.mjs
```

GitHub CI runs these checks without SolidCraft, a GPU, private CNC data, or the
private engine. GitHub Pages deployment remains a separate manual workflow.
Browser and real-CAD regression commands are documented in
[verification](docs/VERIFICATION.md).

## FAQ

**Can I use SolidCraft 0.18.x?** No compatibility claim is made. Only 0.17.0
has passed the MCP integration suite.

**Why does CONNECTED say the CLI is missing?** Pass `--solidcraft-cli`, set
`PIXCAM_SOLIDCRAFT_CLI`, or add the executable to `PATH`. The error states which
configured source was invalid.

**Why does a connection fail while DEMO works?** DEMO is entirely offline.
CONNECTED additionally needs the compatible CLI and, for GUI mode, a live
`127.0.0.1:PORT` control endpoint. Check the version/protocol message, timeout,
and app control port.

**Does pixCAM modify CAD?** No. Its MCP surface is read-only and tested to deny
mutation commands before transport.

**Is this safe for production machining?** No. Treat all manufacturing
hypotheses as review material, not certified toolpaths or simulation results.

## Contributing

Open an issue before a large feature. Keep fixtures synthetic, avoid machine or
customer data, add tests for behavior and failure modes, run the verification
commands above, and keep the SolidCraft bridge read-only/loopback-only. Pull
requests should explain compatibility and security effects. Do not commit
tokens, local paths, generated caches, or captures from real user documents.

See [security](docs/SECURITY.md), [privacy](docs/PRIVACY.md),
[future manufacture adapter](docs/MANUFACTURE_ADAPTER.md), [MCP](docs/MCP.md),
[verification](docs/VERIFICATION.md), and [brand assets](docs/BRAND.md).
