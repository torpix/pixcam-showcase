# SolidCraft 0.17.0 integration

## Supported contract

pixCAM has been tested with SolidCraft CLI 0.17.0 on Linux. The connected
client requires MCP protocol 2025-03-26 and server identity `solidcraft` version
`0.17.0`. SolidCraft 0.18.x is not supported unless a separate compatibility
suite is added and passes.

The bridge uses newline-delimited JSON-RPC 2.0 over stdio, sends the initialized
lifecycle notification, limits messages to 16 MiB, applies a per-request
deadline, and retries a failed capture once in a fresh session. It calls only:

- `list_commands`;
- `inspect_design`;
- `measure`;
- `body_topology`.

Control authentication stays inside the SolidCraft CLI. pixCAM does not read or
forward its token.

## Install and select the CLI

Install SolidCraft separately according to its own distribution instructions.
pixCAM never downloads, replaces, upgrades, or modifies that installation.

Selection precedence is fixed:

1. `--solidcraft-cli PATH` on the pixCAM command;
2. `PIXCAM_SOLIDCRAFT_CLI`;
3. `solidcraft-cli` discovered through `PATH`.

The resolved target must be an executable regular file. Both relative and
absolute configured paths are canonicalized; shell-safe process APIs preserve
spaces in paths.

```sh
cargo build --release --locked

./target/release/pixcam-showcase capture-solidcraft \
  --solidcraft-cli "/opt/SolidCraft 0.17.0/solidcraft-cli" \
  --in fixtures/bridge-part.solidcraft
```

## Isolated synthetic document

Start the adapter with the included synthetic native document:

```sh
export PIXCAM_SOLIDCRAFT_CLI="/opt/SolidCraft 0.17.0/solidcraft-cli"
./target/release/pixcam-showcase bridge --in fixtures/bridge-part.solidcraft
```

Open the printed localhost URL, select CONNECTED, and refresh. To capture and
then analyze JSON without the browser:

```sh
./target/release/pixcam-showcase capture-solidcraft \
  --in fixtures/bridge-part.solidcraft > /tmp/pixcam-snapshot.json
./target/release/pixcam-showcase analyze-solidcraft /tmp/pixcam-snapshot.json
```

Do not add captures of real user documents to this public repository.

## Active GUI

Start SolidCraft with a known unused loopback control port and then connect the
adapter to that same port:

```sh
# Start the GUI according to your SolidCraft package, with control port PORT.
./target/release/pixcam-showcase bridge --connect 127.0.0.1:PORT
```

Only `127.0.0.1` with a nonzero explicit port is accepted. The pixCAM browser
uses a different ephemeral loopback HTTP port and never talks directly to the
SolidCraft control endpoint.

## Troubleshooting

- **`SolidCraft CLI not found`**: configure one of the three sources above.
- **`from --solidcraft-cli ... unavailable`**: the explicit path wins; fix or
  remove it rather than expecting fallback to the environment or `PATH`.
- **`not executable`**: correct file permissions on your own installation.
- **`Incompatible SolidCraft MCP handshake`**: use CLI 0.17.0. The diagnostic
  includes the reported server name, version, and protocol. Do not assume 0.18.x
  compatibility.
- **`MCP timeout/disconnect`**: confirm the GUI is running, the control port is
  correct, and increase the default 15000 ms deadline only when justified with
  `--timeout-ms N`.
- **`Invalid MCP JSON` / envelope errors**: the peer did not return the expected
  newline JSON-RPC contract; verify the selected executable and version.
- **HTTP 502 with stale data**: the browser retains the last good report but
  marks it stale. Restore the app/CLI and refresh for a complete new capture.

Raw CAD/tool errors are suppressed so they cannot leak credentials or document
details. Failed sessions are killed and cannot be reused.

## Opt-in real integration test

The default tests use a fault-injection MCP peer and require no SolidCraft:

```sh
cargo test --locked
```

On a machine where 0.17.0 is installed:

```sh
PIXCAM_SOLIDCRAFT_CLI="/opt/SolidCraft 0.17.0/solidcraft-cli" \
  cargo test --locked actual_headless_mcp_roundtrip_and_restart -- --ignored
```

The full GUI harness is `python3 scripts/live-gui-test.py`. It is intentionally
outside CI because it needs the real GUI and local installation. It must be run
only against the included synthetic document and an isolated temporary profile.

## Interpretation limits

Snapshots carry schema version 1 and SolidCraft version 0.17.0. Cylindrical
surface parameters are geometry evidence; adjacency spans are not certified
drilling depth. Bore/counterbore names and coaxial groups remain hypotheses.
No plugin ABI, CAM operation tree, postprocessor, cutting simulation, or
production NC workflow is implemented.
