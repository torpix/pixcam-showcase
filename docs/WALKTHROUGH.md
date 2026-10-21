# From SolidCraft CAD to Manufacturing Intelligence

Watch [the 36-second v0.3 film](../media/pixcam-solidcraft-bridge-36s.mp4). It records actual browser frames from a Rust adapter connected to an isolated SolidCraft 0.17.0 GUI. No desktop, user document or control token is captured.

1. Open the synthetic native document in a separate SolidCraft GUI.
2. Start pixCAM's loopback adapter with --connect and that app's actual port.
3. Choose CONNECTED and Refresh analysis; confirm bridge-part and 100 × 70 × 30 mm.
4. Inspect CY02: Ø10, Z axis, available span 12 mm.
5. Inspect coaxial CY03/CY04: Ø12/Ø6 and the possible counterbore hypothesis.
6. Select CY05 and Y +90°: its X axis becomes parallel to Machine Z. Access remains unverified.
7. Explore the compound geometric 3+2 orientation and surface-normal map.
8. Export the current report JSON, registry CSV, source snapshot JSON, SVG map and PNG view.

The three v0.3 screenshots show offline DEMO, imported topology and a real GUI CONNECTED capture. The refreshed legacy-laboratory screenshots and historical 72-second film document the separate Aurora Coordinate/NC experience.

To reproduce the isolated GUI test and film:

```sh
cargo build --locked --offline
./target/debug/pixcam-showcase build dist
python3 scripts/live-gui-test.py --record
```

Set PIXCAM_PLAYWRIGHT to an installed module path if Playwright is not available as a package. The harness creates a temporary profile/control directory, dynamically chooses a port, opens only fixtures/bridge-part.solidcraft, verifies no file change and terminates its own app process.

The film demonstrates geometry and planning assistance. It does not demonstrate certified manufacturing, complete hole classification, collision checking or a generated NC program.
