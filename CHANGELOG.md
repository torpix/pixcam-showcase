# Changelog

## 0.4.1 - 2026-10-21

### Added

- Portable SolidCraft CLI discovery via `--solidcraft-cli PATH`,
  `PIXCAM_SOLIDCRAFT_CLI`, or `solidcraft-cli` on `PATH`, in that order.
- Linux CI for build, tests, formatting, Clippy, JSON Schemas, synthetic
  fixtures, exports, deterministic demo generation, and privacy checks.
- Setup, troubleshooting, security, contribution, and architecture guidance
  for new users.

### Changed

- Removed the developer-specific SolidCraft installation path.
- Improved missing executable, bad path, MCP timeout/disconnect, JSON, and
  compatibility diagnostics.
- Made the privacy gate suitable for a normal public clone with an `origin`
  remote and an evolving single-root public history.

### Compatibility

- SolidCraft CLI 0.17.0 and MCP protocol 2025-03-26 are the only tested
  connected configuration. SolidCraft 0.18.x is not claimed as supported.
- The existing `v0.4.0-rc.1` tag and release are unchanged.

This remains an experimental geometry and manufacturing-analysis showcase. It
does not provide production machining simulation, collision proof, or
production NC generation.
