# Security and privacy status

pixCAM 0.4.1 is an experimental local application, not a certified machining
or security product. Its connected attack surface is intentionally narrow.

## Enforced controls

- SolidCraft process selection is explicit and never installs or replaces it.
- Only server `solidcraft` 0.17.0 with MCP 2025-03-26 is accepted.
- The MCP client allowlists four read-only tools and denies mutation before a
  request is sent.
- MCP messages, body counts, notification counts, request time, and HTTP bodies
  are bounded. Failed MCP sessions are killed and not reused.
- Active-app connections accept only explicit `127.0.0.1:PORT` endpoints.
- The browser adapter binds only loopback and checks exact Host, same Origin,
  fetch site, content type, route, and a fresh per-process session key.
- There is no CORS, general filesystem server, or generic MCP tool proxy.
- Snapshot sanitization removes CAD paths and arbitrary metadata; raw CLI/tool
  errors and control credentials are not returned to the browser.

Automated tests cover absent/bad executables, version/protocol mismatch,
timeout, disconnect, malformed JSON, wrong JSON-RPC envelopes, suppressed
secret-like server errors, non-loopback addresses, mutation-tool denial,
invalid geometry, and cycle numeric/count boundaries.

## Privacy boundary

Only synthetic Aurora and BridgePlate fixtures belong in this repository.
Machine configurations, controller programs, customer geometry, control
tokens, and captures of real user documents must not be committed. The privacy
gate uses an explicit public-file allowlist, checks the native fixture hash,
scans common secret/private-path markers, and can scan the reachable Git
history with `node scripts/privacy-check.mjs --history`.

## Residual risk

The real SolidCraft executable and documents are outside pixCAM's trust
boundary. A malicious or compromised executable selected by the user can return
hostile data or behave differently from SolidCraft. Resource bounds reduce but
do not eliminate local denial-of-service risk. Browser exports are user-initiated
local files. Geometry evidence and manufacturing hypotheses are not safety
certification, collision proof, or production machining simulation.

Report a suspected vulnerability privately to the repository owner rather than
posting credentials, proprietary designs, or exploit details in a public issue.
