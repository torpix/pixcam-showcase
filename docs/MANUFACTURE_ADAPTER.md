# Future manufacture adapter data contract

Status: exploratory, schema version 1. This is a neutral JSON interchange
proposal, not a SolidCraft plugin ABI and not a commitment to WASM/WIT.

## Public discussion context

The proposal was checked against four upstream threads on 2026-10-21:

- [#1153](https://github.com/tangency-labs/solidcraft/issues/1153) describes a
  broad Manufacture/CAM direction with setups, operations, tools, simulation,
  and posts, while explicitly asking maintainers to settle scope and design.
- [#2652](https://github.com/tangency-labs/solidcraft/issues/2652) recommends an
  interface-first split and a CAD/CAM boundary where document/settings enter
  and computed path segments leave; toolpaths are not persisted in CAD.
- [#2613](https://github.com/tangency-labs/solidcraft/pull/2613) proposes
  `CamBackend`/`MachineBackend` host traits with fake in-process test hosts and
  leaves network/protocol adapters outside the core contract.
- [#2645](https://github.com/tangency-labs/solidcraft/pull/2645) explored a
  loopback job companion but was closed over runtime licensing/dependency
  concerns, reinforcing a backend-neutral exchange format.

The stable common ground is data, provenance, frames, validation, and explicit
uncertainty—not a premature transport or plugin ABI.

## Contract

The standalone [JSON Schema](../schemas/manufacture-adapter.schema.json) uses the
identifier `pixcam.manufacture-adapter/1`. A payload contains:

- `geometry`: neutral entity records with stable source references and capture
  provenance;
- `manufacturing_features`: interpretations that reference geometry rather
  than relabel it as verified manufacturing intent;
- `coordinate_frames`: right-handed `global_from_local` 4×4 transforms with
  explicit millimetre units;
- `position_patterns`: evaluated local positions plus original parametric
  inputs, so consumers can choose evaluation or reconstruction;
- `validation`: document-level findings and per-record `status` values that
  separate evidence class, validation state, and uncertainty.

IDs are payload-local strings. Source references remain opaque strings; an
adapter must not infer topology identity from names alone. Every feature and
pattern names its coordinate frame. Every interpreted record carries one of
`VERIFIED_GEOMETRY`, `GENERATED_GEOMETRY`, `INFERRED_FEATURE`, or `UNKNOWN` and
a validation state of `valid`, `partial`, `invalid`, or `unvalidated`.

JSON Schema checks structure and finite JSON-number shape, but it cannot prove
that matrices are rigid/right-handed, references exist, revisions match, or
uncertainty is physically meaningful. Implementations must perform those
cross-record and numerical checks separately and fail closed.

## Evolution rules

- Increment the major suffix for breaking field/meaning changes.
- Additive fields require consumers to ignore unknown fields only where the
  schema explicitly permits them.
- Never silently reinterpret units, frame direction, provenance class, or
  validation state.
- Transport authentication, job lifecycle, toolpaths, posts, and machine
  commands belong in separate versioned contracts.
- A future WIT mapping may wrap this data only after upstream interfaces are
  approved; this project does not define that ABI.

Run `python3 scripts/manufacture-schema-test.py` for positive and negative
contract examples.
