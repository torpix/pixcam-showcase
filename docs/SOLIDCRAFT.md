# SolidCraft v0.17 read-only integration spike

Evidence was checked against the public SolidCraft **v0.17.0** tag (`e0cc740`, released 2026-10-20) on 2026-10-21. This document separates shipped interfaces, the tested pixCAM adapter and future proposals.

## Shipped read surfaces

SolidCraft's [MCP documentation at v0.17.0](https://github.com/tangency-labs/solidcraft/blob/v0.17.0/docs/mcp.md) documents:

- `solidcraft-cli mcp --connect 127.0.0.1:7878` for a bridge to a running app started with `solidcraft --control 7878`;
- read-only MCP tools `inspect_design`, `measure` and `body_topology`;
- `inspect_design` output containing timeline definitions, bodies, sketches, selection and measurements;
- `body_topology` output containing edges and faces, including cylindrical origin, axis and radius;
- `timeline.inspect` for a feature's stored definition and inputs.

The control protocol also exposes document inspection in the active app session. These are supported read paths; pixCAM does not need a private process hook or CAD mutation to obtain a safe snapshot.

## Implemented adapter

`src/solidcraft.rs` is a version-pinned, read-only import boundary. It consumes an already captured v0.17.x `inspect_design` result plus optional `body_topology` results and emits:

- timeline feature definitions;
- evaluated cylindrical body faces with persistent face name, point, unit axis and diameter;
- explicit `read_only: true` and source provenance;
- explicit `per_feature_geometry_available: false`.

The CLI command is:

```sh
pixcam-showcase import-solidcraft-snapshot SNAPSHOT.json
```

Two tests verify the synthetic import and fail-closed behavior for unknown versions, invalid axes and invalid geometry. The adapter never launches SolidCraft, writes a document, sends engine commands, opens a network connection or infers missing associations.

## Precise live-bridge blocker

An end-to-end active-document run is **not verified on this workstation** because no `solidcraft-cli` executable is installed. The public protocol establishes that a bridge is possible, but an honest acceptance run still requires:

1. SolidCraft v0.17.x and `solidcraft-cli` from the same build;
2. an app launched with the control endpoint;
3. read-only calls to `inspect_design`, then `body_topology` for selected bodies;
4. capture into the documented pixCAM snapshot envelope;
5. comparison of imported topology with the visible source document.

There is also a product-level limit: v0.17 documents timeline definitions and current body topology separately. It does not document a single evaluated geometry object per timeline feature. The adapter therefore refuses to invent a timeline-feature-to-face relation. It can import body geometry now; exact per-feature evaluated geometry remains blocked on a stable host association or explicit provenance supplied by the caller.

## Future WASM/WIT boundary

[Issue #1221](https://github.com/tangency-labs/solidcraft/issues/1221) remains an open design proposal whose description explicitly asks for agreement before implementation. Its proposed `solidcraft:plugin@1.0.0` world and `run(command, args)` host import are not treated as shipped.

If that contract lands, the current adapter can sit behind a thin component:

1. request only read permissions;
2. call the same documented inspection commands through `run`;
3. pass their JSON results into the existing versioned importer;
4. return neutral pixCAM JSON or overlay polylines;
5. request no filesystem or network capability for in-memory inspection.

No private API, unrestricted WASI access or parallel CAD mutation surface is proposed.

## Avoiding duplicate CAM work

[Manufacture RFC #1153](https://github.com/tangency-labs/solidcraft/issues/1153) describes a branch with setups, operations, posts, read-back checks and height-field simulation; the v0.17 roadmap still treats Manufacture/CAM as deferred and the issue remains open. pixCAM v0.2 stays on the complementary side of that boundary: coordinate explanation, read-only feature/topology exchange and NC inspection. It does not add SolidCraft CAM operations, postprocessors, material removal or manufacturing document edits.
