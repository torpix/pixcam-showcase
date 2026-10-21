# Public release boundary and provenance

The source workspace was audited before preparing this directory. The audit read every one of its **37 tracked working files** and all **46 local Git blobs**, including unreachable objects; history inventory covered **3 commits** across local refs/reflogs. Private technical data and source references were present. That workspace and its history are **not suitable for publication**.

This public variant was assembled in a new directory, with a new Git repository and no copied `.git`, alternates, parent history, source fixtures or archived models. Original files were not overwritten or deleted. The detailed private findings remain outside the public directory, because publishing the findings verbatim would itself disclose identifiers and paths.

Only directly reviewed generic vector/face/transform code and the bounded NC parser were reused. Unused legacy feature-registry/planner code was removed from the public core. No machine-specific module, prior viewer, source dataset, test fixture, old documentation, archive, screenshot or recording was transferred.

All public geometry is the new Aurora definition in `src/showcase.rs`; the illustrative NC was authored anew in `fixtures/aurora.nc`. JSON/CSV/SVG/DXF and the HTML are produced from those inputs. The three screenshots were captured from that HTML in an isolated browser page. The video records only the same page viewport, never the desktop or existing application windows. Their creation does not require any private documents.

## Checks and limits of the audit

`node scripts/privacy-check.mjs` checks all release files against an explicit path allowlist, rejects symlinks and unexpected file formats, scans text for prohibited identifiers, personal absolute paths and common secret signatures, checks the synthetic geometry contract, and verifies media file signatures. With `--history`, it also scans all local Git objects and verifies a single root commit with no remote or object alternates.

This complements provenance review and visual inspection; keyword scanning alone cannot prove that arbitrary technical data is non-confidential. Raster/video content is reviewed visually and by its deterministic synthetic-page capture provenance, not by claiming that a text scan can read every pixel. Dependencies and ignored build/test caches are not release materials. No remote publication has taken place.

Publish only this independent repository's tracked source/materials, or the prepared browser bundle. **Do not upload the enclosing workspace, its history, ignored build directories, private audit report or local workflow logs.** A source archive can be produced from this repository's commit with `git archive`; this excludes `.git`, Cargo build products and test scratch files.
