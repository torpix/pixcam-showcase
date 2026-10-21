# Parametric Cycles / v0.4 RC1

These are original geometric positioning algorithms, independent of drilling technology and controller dialects. They run in Rust and in the self-contained offline browser. No solid modelling, Boolean analysis or production NC is involved.

## Circular Pattern

Provide a local center XYZ, exactly one positive radius or pitch diameter, start angle in degrees, positive sweep up to 360°, count and optional direction (default counterclockwise). The feature diameter is independent of the pitch size.

A full circle uses sweep/count and omits the repeated closing position. A sweep within 1e-9 degrees of 360° is considered closed. An open arc uses sweep/(count−1), including both ends; a one-position pattern uses its start only. Clockwise reverses angular progression, not the work-frame axes. Positive angles follow the right-handed local XY plane, viewed from +Z.

## Rectangular Pattern

Provide a local starting XYZ, rows, columns, positive X/Y pitches, rotation in degrees and feature diameter. Positions are generated row-major: row zero then increasing columns. Rotation applies to the spacing vectors around the start point; it does not rotate the start point around the global origin.

Zero is valid for coordinates and angles. Sizes, pitches and counts must be positive. The total limit is 1024 positions, including rows × columns. Centers closer than 1e-7 mm are rejected in local or global coordinates. Inputs/results must be finite; numerically unstable transforms are rejected with a 1e-6 mm round-trip gate. These are numerical checks, not machining tolerances.

## Work frames and registry

`global_from_local` is any finite rigid right-handed 4×4 matrix, row-major, using column vectors. Rigid-axis dot products/determinant are checked to 1e-9. Local −Z becomes the global tool-axis direction. The UI supplies four 3+2 frames and an editable matrix. The geometric transform does not solve physical rotary-axis kinematics.

Each position contains cycle_id, zero-based position_index, stable ID, local/global XYZ, global tool axis, feature diameter, source parameters and GENERATED_GEOMETRY status. This status is distinct from CAD's VERIFIED_GEOMETRY and manufacturing INFERRED_FEATURE.

The combined registry retains the complete imported CAD report and a separate parametric_cycles collection with a shared count. Imported faces are preserved without invented CAD face associations. Rust regenerates cycles from their source parameters and rejects altered positions or duplicate cycle IDs. No cycle writes to SolidCraft.

## CLI

```sh
cargo build --release --locked --offline
./target/release/pixcam-showcase cycle fixtures/cycle-circular.json cycle-output
./target/release/pixcam-showcase cycle fixtures/cycle-rectangular.json grid-output
./target/release/pixcam-showcase registry fixtures/bridge-snapshot.json fixtures/cycle-circular.json
```

The optional output directory receives cycle.json/.csv/.svg/.dxf. JSON and CSV retain full coordinate precision from the registry; the UI rounds only the visible readout. CSV includes source parameters. SVG is local XY, with global XYZ in each entity's metadata. DXF contains global POINT centers and CIRCLE entities whose OCS centers/extrusion normals preserve the tilted work plane, following Autodesk's [OCS contract](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-D99F1509-E4E4-47A3-8691-92EA07DC88F5.htm) and [arbitrary-axis algorithm](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-E19E5B42-0CC7-4EBA-B29F-5E1D595149EE.htm).

Request structure is defined in [cycle-request.schema.json](../schemas/cycle-request.schema.json); numerical rigid-frame, total-count and duplicate constraints are checked by the engine. Unused optional pitch fields normalize to null in Rust exports.

## Limits

No depth, feed, speed, peck/dwell strategy, tool choice, fixture/access approval, material removal, machine certification or controller-cycle expansion. Intersecting feature diameters are allowed: distinct position centers do not imply feasible machining. One cycle is edited at a time in the browser; the Rust combined registry supports multiple uniquely named cycles.
