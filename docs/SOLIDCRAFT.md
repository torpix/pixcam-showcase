# SolidCraft context and integration boundary

Issue descriptions checked on **2026-10-21**. The following compares stated scope, not independently certified implementations or a claim that a branch has shipped in SolidCraft main.

| Area | This pixCAM showcase | SolidCraft Manufacture RFC |
|---|---|---|
| Purpose | Inspect synthetic feature metadata and illustrative NC | Proposes integrated 2.5D manufacturing |
| Geometry | Nine explicit annotations, three face projections | Author reports operations from sketch profiles and planar faces |
| Operations | Proposed orientation/tool groups only | Author reports facing, contours, pockets and drilling on a branch |
| NC | Read a bounded subset; no production generation | Author reports declarative posts and read-back checks |
| Visualization | Colored paths and nominal outlines | Author reports height-field stock removal and deviation checks |
| Integration | Independent Rust crate and offline HTML | RFC describes shared engine commands and Manufacture UI |

These Manufacture claims are from [RFC #1153](https://github.com/tangency-labs/solidcraft/issues/1153), whose author describes a working `feat/cam` branch and requests maintainer guidance. The issue remains open in the checked snapshot; it is not evidence that the described features have landed on main. pixCAM can contribute a small inspection/data-exchange experiment without duplicating the proposed full CAM system.

The discussion has moved beyond the initial RFC: [this follow-up in #1221](https://github.com/tangency-labs/solidcraft/issues/1221#issuecomment-6210789519) reports active Manufacture work covering mill/turn, part handling, additive workflows, probing/inspection, fabrication and machine/post/template catalogs. It explicitly treats Manufacture as core document/command work alongside the research bank, not something waiting for WASM plugins. Those additional implementation and merge claims were not independently checked against main in this task. The table above deliberately describes the original RFC baseline; it is not an exhaustive inventory of all current Manufacture work.

[Plugin proposal #1221](https://github.com/tangency-labs/solidcraft/issues/1221) describes a versioned WIT component interface, shared engine command execution, permissions and runtime limits. Its body presents these as design decisions and milestones, not a stable plugin ABI guarantee. This showcase does not load a SolidCraft plugin or claim compatibility with a released WIT interface.

[The later panel/overlay discussion](https://github.com/tangency-labs/solidcraft/issues/1221#issuecomment-6278997181) also asks how plugin workspaces and path overlays might be exposed. Consequently, a custom pixCAM panel/viewport is a proposed integration dependency, not an API this demo assumes already exists.

[Research request #1621](https://github.com/tangency-labs/solidcraft/issues/1621) asks for CAM knowledge and verification evidence, with public sources and no private/company data. This deliverable is a narrow executable demonstration and an explicit limitations record, not a substitute for that research or evidence of real-machine accuracy.

## Proposed integration (not implemented)

1. Discover the actual supported SolidCraft engine commands and version at connection time. Obtain a user-selected, public-safe model snapshot through documented read commands; never invent command IDs. Map explicit feature provenance to a versioned pixCAM JSON adapter. No automatic recognition is currently implemented.
2. Keep analysis read-only. Return inspection results and neutral polylines to an overlay/panel. Existing JSON/SVG/DXF files are the first exchange boundary; a native overlay requires a separately tested host adapter.
3. When a supported plugin contract is available, place that adapter behind the agreed WIT exports and shared command dispatcher. Request no filesystem/network permissions for in-memory inspection. Respect host memory/fuel limits and surface errors. Do not add a parallel model mutation API.
4. Route future accepted manufacturing edits through the host's undoable, journaled commands only. This would be a new task: the current demo neither edits SolidCraft documents nor generates operations or NC.

The MCP tools are available now as an independent stdio process; their names are **pixCAM tool names**, not SolidCraft command names. Host ABI compatibility, native rendering, CAD import and an end-to-end SolidCraft plugin remain unverified. No SolidCraft code, third-party postprocessor or commercial reference asset was imported for this showcase.
