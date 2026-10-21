# Public release boundary and provenance

This directory is an independent public-safe repository with its own single-root history. The enclosing workspace and its Git history contain private engineering inputs and must remain private. No parent history, source archive, machine configuration, production program or unrelated document is copied into this repository.

v0.3 adds only our original synthetic BridgePlate: the construction commands in fixtures/bridge-part.json, their native SolidCraft document, actual read-only MCP snapshot and derived report. A separate temporary SolidCraft GUI/profile was used for LIVE acceptance. The native CAD ZIP contains document/container JSON and its own model/body caches; the checker inspects JSON members and admits only the reviewed synthetic cache hash.

The retained Aurora dataset, illustrative NC and historical 72-second v0.2 film remain original synthetic content. Regression tests refresh the three legacy-laboratory screenshots within the current page. The three v0.3 bridge screenshots and 36-second film capture only the pixCAM browser viewport; the film's CONNECTED status is backed by the isolated real GUI. They never capture a desktop, existing app window or file chooser. Raster/video provenance and visual review complement signature checks.

The bridge does not log geometry or raw SolidCraft errors. It never reads control token files itself; the pinned CLI handles authentication. inspect_design paths and arbitrary metadata are excluded from captured snapshots. The HTTP session key belongs only to the adapter page, is not a SolidCraft credential, appears in no URL/export, and is regenerated at every adapter start.

## Release gate

`node scripts/privacy-check.mjs` enforces an explicit path allowlist, rejects symlinks/unexpected binaries, checks synthetic contracts and scans text for prohibited data, personal paths and common secrets. Only the two exact pinned local SolidCraft executable paths are exempted from the personal-path check because the integration requires them.

`--history` additionally scans every local Git object, verifies one root, exactly the reviewed v0.1 → v0.2 → v0.3 chain, no remotes, no alternates and a clean public worktree. It checks all text objects, admitted image/video signatures and the exact synthetic native ZIP hash. No general binary/archive exemption is granted.

Ignored target/node_modules/test-results caches are not release materials. Local GUI snapshots and runtime evidence stay in test-results and are not committed. Future connected documents may be private: do not copy their exports, captures or recordings into this public repository.

Publish only a reviewed archive of this independent repository, never the enclosing workspace or workflow logs. No push or publication was performed.
