# Status

2026-11-11. Where hearth's code lags or departs from its design, and its design from its
draft, from the status files below. M1 has render and diff in-process, canonical
encoding and a store without layers; M2 has build keys, planning and ordered
stage, run, seal and discard, with scripted steps and a fake helper
(`hearth/docs/design/domain/render/README.md`, `hearth/docs/design/domain/store/README.md`,
`hearth/docs/design/domain/build/README.md`, `hearth/docs/design/client/README.md`).
Layer operations remain `domain/store/README.status.md`, 2.1, 2.2, 2.3, 4.1 and 4.2; the helper process remains `domain/store/tree-helper.status.md`, 1, 2, 3, 5, 6 and 8; build-step execution remains `domain/build/README.status.md`, 2, 3, 4.1 and 4.3; sandboxes remain `domain/build/sandboxes.status.md`, 1 to 8; pins and collection remain `domain/store/README.status.md`, 5.1 to 5.5 and 6.1. The client reports steps as unavailable. Tool versions remain `domain/render/spec.status.md`, 2.4, and `domain/store/records.status.md`, 4; rebuild times remain `domain/render/README.status.md`, 4.2. The resolution reply remains `domain/render/README.status.md`, 5.2; reconciled-base retention remains `domain/store/README.status.md`, 5.1; mmdebstrap hooks remain `domain/build/packages.status.md`, 4 and 5. Resumed file work's test remains `domain/render/README.status.md`, 5.5, as does the decoder driver's queue departure. Store-record settlement remains `domain/store/README.status.md`, 4.4 and 6.2; second staging remains `domain/store/README.status.md`, 4.6; store lifecycle departures remain `domain/store/README.status.md`, 4 and 6.2, and the builder adapter and keep `domain/render/README.status.md`, 5. The missing activator, steward and link remain `README.status.md`, 4.1; remaining crates and support remain `README.status.md`, 4.2; citation-form departures remain `README.status.md`, 6. Machine, boot and bootstrap work remains `domain/machine/README.status.md`, 3; slots remain `domain/slots/README.status.md` (Not yet, the whole document); the privilege boundary and live changes remain `domain/activator/README.status.md`, 2 to 6; the steward remains `domain/steward.status.md`, 2 to 8; the wire remains `protocol/wire.status.md` (Not yet, the whole document). Operating, development, artifacts and reproducible builds remain their whole-document Not-yet entries in `domain/operating.status.md`, `domain/development.status.md`, `domain/build/artifacts.status.md` and `domain/build/reproducible.status.md`. Tests have the worlds' suites,
git, resolution and render conformance, and tool fakes, but complete-machine worlds remain `testing/worlds.status.md`, 1, and real builds and boot `testing/real-builds.status.md`, 1 to 6; the conformance runner remains `testing/README.status.md`, 3.5. Tier profiles and suite timeouts remain `testing/README.status.md`, 3.5, 4.1 and 4.2; memory-test layout remains `testing/worlds.status.md`, 4; repeated per-seed work remains `testing/README.status.md`, 4.1. Draft departures remain `draft/brainstorm.status.md`, 4.1, 4.6, 10.3, 11.4, 8, 9 and 12; pool placement remains `draft/brainstorm.status.md`, 5.2 (Design lags); open decisions remain `draft/brainstorm.status.md`, 13 (Open).
`hearth/docs/design/protocol/format/README.md` has no gap.

Design documents:

- [`README.md`](README.status.md)
- [`protocol/format/README.md`](protocol/format/README.status.md)
- [`domain/render/README.md`](domain/render/README.status.md)
- [`domain/store/README.md`](domain/store/README.status.md)
- [`domain/build/README.md`](domain/build/README.status.md)
- [`domain/machine/README.md`](domain/machine/README.status.md)
- [`domain/slots/README.md`](domain/slots/README.status.md)
- [`domain/activator/README.md`](domain/activator/README.status.md)
- [`domain/steward.md`](domain/steward.status.md)
- [`protocol/wire.md`](protocol/wire.status.md)
- [`client/README.md`](client/README.status.md)
- [`domain/operating.md`](domain/operating.status.md)
- [`testing/README.md`](testing/README.status.md)
- [`domain/development.md`](domain/development.status.md)
- [`domain/build/artifacts.md`](domain/build/artifacts.status.md)
- [`domain/build/reproducible.md`](domain/build/reproducible.status.md)

Drafts:

- [`draft/brainstorm.md`](draft/brainstorm.status.md)
