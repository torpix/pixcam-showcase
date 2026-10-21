# Read-only AI tools

Build with `cargo build --locked --release`, then configure an MCP client to launch:

```json
{
  "mcpServers": {
    "pixcam": {
      "command": "/absolute/path/to/pixcam-public/target/release/pixcam-showcase",
      "args": ["mcp"]
    }
  }
}
```

The wrapper configuration is client-dependent; replace the example path. No server or client installation/configuration is performed by this project. The executable has no runtime file dependencies in MCP mode.

The small server implements initialization, initialized notification, ping, `tools/list` and `tools/call`, using protocol version `2025-06-18`. Requests are newline-delimited JSON-RPC over stdio; stdout contains only protocol messages. Tool results include `structuredContent` and a text JSON fallback, with input/output schemas and read-only annotations. See the official [stdio transport](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports), [lifecycle](https://modelcontextprotocol.io/specification/2025-06-18/basic/lifecycle) and [tool contract](https://modelcontextprotocol.io/specification/2025-06-18/server/tools).

| Tool | Arguments | Result |
|---|---|---|
| `inspect_features` | Optional `face` enum and positive `diameter_mm` | `count`, typed `features`, `units: mm`, `synthetic_only: true` |
| `explain_nc_cycle` | `nc`: at most 4096 UTF-8 bytes / 64 lines | `supported`, `motion_count`, `summary`, nullable `diagnostic`, `machining_verified: false` |
| `validate_coordinate_transform` | `transform: {rotation: 3×3, translation_mm: [x,y,z]}`, `point_mm: [x,y,z]` | `valid`, nullable transformed point, round-trip error and diagnostic |

Example tool calls after initialization:

```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"inspect_features","arguments":{"face":"Y_MIN","diameter_mm":6}}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"explain_nc_cycle","arguments":{"nc":"G0 X2\nG1 Y3 F100"}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"validate_coordinate_transform","arguments":{"transform":{"rotation":[[0,-1,0],[1,0,0],[0,0,1]],"translation_mm":[11,23,7]},"point_mm":[2,3,4]}}}
```

The last example yields `[8,25,11]` mm. Transforms use column vectors, active right-handed rotation followed by translation; vectors receive rotation only. Validation checks finite values, orthonormality and determinant +1, with tolerance 1e-8. It rejects reflection and scale. Round-trip error is diagnostic, not a physical accuracy certificate or machine calibration.

The showcase's Coordinate Inspector additionally uses the full homogeneous 4×4 library in `src/coordinates.rs`. The MCP transform tool intentionally retains its compatible 3×3-plus-translation input schema; both paths use the same right-handed active-transform convention.

`inspect_features` deliberately reads only the built-in synthetic part. None of the tools accepts a file path, follows a source reference, opens a document, writes a file, calls a network service or controls hardware. NC is parsed in memory; unsupported cycles return `supported: false`, never a fabricated expansion. Core parser diagnostics retain the original Polish wording, alongside English support summaries.

Transport messages are limited to 64 KiB. This is a minimal synchronous local implementation: no HTTP, subscriptions, prompts, resources or host integration. Rust subprocess tests cover real stdio; compatibility with an external GUI MCP client has not been tested. Full schemas can be printed using `pixcam-showcase tool-schemas`.
