# Draft for SolidCraft CAM #1153 — not posted

Target: [Manufacture/CAM discussion #1153](https://github.com/tangency-labs/solidcraft/issues/1153).

Hi! I have prepared pixCAM v0.4 RC1 as a small, standalone companion experiment for SolidCraft. It is intended to complement the existing Manufacture work: its scope is read-only geometry inspection, explainable coordinate transforms and parametric position layouts.

The current demo connects to SolidCraft 0.17.0 through a Rust MCP client restricted to inspection tools, imports cylindrical surfaces and axes, and separates verified geometry from manufacturing hypotheses and unknowns. Circular and rectangular positioning patterns now run in arbitrary rigid 4×4 work frames, with immediate 2D/3D preview, local/global coordinates, source parameters, and JSON/CSV/SVG/DXF exchange.

The public RC uses only original synthetic examples. The offline demo needs no SolidCraft installation. Headless and isolated GUI connections, reconnect behavior, transforms and exports have been tested. It does not edit CAD documents or generate production NC.

I would welcome feedback on a small neutral exchange boundary for geometry, position patterns and provenance that could be useful alongside Manufacture. I am not proposing a competing operations/postprocessor pipeline or changes to SolidCraft's manufacturing document format.

The package and description are prepared locally for review; neither has been published.
