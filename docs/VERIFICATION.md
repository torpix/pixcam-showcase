# pixCAM v0.4.1 verification

Verified locally on Linux on 2026-10-21. The complete committed tree was cloned
to a fresh temporary directory before the final build, tests, exports, browser
regressions, real SolidCraft checks, and history audit. No push, release, tag
change, or Pages deployment was performed.

## Portable Rust and CI gate

- `cargo build --release --locked --offline`: pass in the fresh clone.
- `cargo test --locked --offline`: 40 pass; the one real-CAD test is ignored by
  default.
- `cargo fmt --all -- --check`: pass.
- `cargo clippy --locked --offline --all-targets -- -D warnings`: pass.
- Explicit real headless 0.17.0 test: pass, including two complete independent
  initialize/capture/shutdown sessions.
- README circular and rectangular release commands: pass; JSON/CSV/SVG/DXF
  files were produced outside the repository.
- Rebuilt `dist` matched the committed standalone demo byte-for-byte.

The GitHub workflow repeats build, default tests, format, Clippy, schemas,
synthetic fixture/export checks, privacy, and deterministic demo generation on
push and pull requests to `main`. It does not install SolidCraft, open a GUI,
require a GPU, read private data, or deploy Pages. The existing Pages workflow
remains `workflow_dispatch` only.

## Bridge failure and security cases

Automated fault injection passed for:

- absent CLI and invalid explicit path, with no snapshot on stdout;
- explicit option → environment → `PATH` precedence and a path containing
  spaces;
- incompatible SolidCraft version and MCP protocol;
- timeout, process disconnect, malformed JSON, wrong response ID, and mixed
  result/error envelopes;
- suppression of secret-like JSON-RPC and tool errors;
- denial of every tested CAD mutation command before transport;
- external/non-loopback control addresses and ambiguous input/connect mode;
- hostile Host/Origin, absent HTTP session, and generic tool route denial;
- actual app exit → HTTP 502/stale → app restart → fresh capture;
- unchanged hash of the synthetic native CAD file.

The security and residual-risk analysis is in [SECURITY.md](SECURITY.md).

## Chromium regressions

All three suites passed using headless Chromium without a GPU:

- Coordinate/ISO-NC: four animated setups, five frames, matrices, nine
  features, 27 motions, interpolated arcs, JSON/CSV/SVG/DXF and PNG downloads,
  mobile layout, and no network/JavaScript errors.
- Parametric Cycles: Rust/browser parity within 1e-8 mm, circular/open/CW and
  4×3 rotated-grid cases, 3+2/custom frames, numeric boundaries, export
  blocking, JSON/CSV/SVG/DXF and combined-registry exports, mobile layout, and
  no network/JavaScript errors.
- Live Bridge: offline DEMO/IMPORT parity, invalid/partial JSON, all bridge
  exports, real read-only MCP CONNECTED refresh, origin/Host/session/route
  rejection, disconnect/stale handling, and reconnect.

The full GUI harness passed against a real, isolated SolidCraft 0.17.0 process
using only the included BridgePlate, a temporary profile/control directory and
a dynamic loopback port. It terminated only its own process. This validates the
small synthetic specimen, not arbitrary assemblies or machine access.

## Schemas, exports, and privacy

All independent validators passed:

```sh
python3 scripts/schema-test.py
python3 scripts/geometry-schema-test.py
python3 scripts/cycle-schema-test.py
python3 scripts/manufacture-schema-test.py
```

They cover six real MCP schema results, snapshot/report positive and negative
examples, exact cycle coordinates/provenance and real CLI exports, plus the
neutral `pixcam.manufacture-adapter/1` example and units/handedness/evidence/
empty-pattern rejection.

`node scripts/privacy-check.mjs` passed 100 explicitly allowlisted public files.
`node scripts/privacy-check.mjs --history` passed the same tree and every
object reachable from the public refs. The checks cover secret and private-path markers,
synthetic contracts, the native fixture hash, unexpected binaries/symlinks, a
single-root/no-merge public history, and reviewed ancestry. Media signatures
and documented synthetic capture provenance complement the automated gate.

## Limits

No real machine, proprietary controller data, private engine, production NC,
certified access/collision proof, material-removal simulation, solved machine
kinematics, postprocessor validation, or SolidCraft 0.18.x compatibility is
claimed. The existing `v0.4.0-rc.1` release and tag were not changed.
