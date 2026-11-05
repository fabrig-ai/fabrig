# Status

2026-11-05. Where hearth's code lags or departs from its design, and its design from its
draft, from the status files below. M1 has render and diff in-process, canonical
encoding and a store without layers; M2 has build keys, planning and ordered
stage, run, seal and discard, with scripted steps and a fake helper
(`hearth/docs/design/domain/render.md`, `hearth/docs/design/domain/store.md`,
`hearth/docs/design/domain/build.md`, `hearth/docs/design/client/README.md`).
Real layer operations, the helper process, build-step runners and sandboxes,
pins and collection remain unbuilt; the client reports steps as unavailable.
Tool versions and rebuild times are missing, and the resolution reply,
reconciled-base retention and mmdebstrap hooks have unsettled
departures in those documents. Resumed work's order has no test, the
store step's order no named predicate, and one test driver keeps capped
queues beside the JSON decoder
(`hearth/docs/design/domain/render.md`). The store reports unreadable or invalid layer
records as strays and refuses a second staging request, and its scan and chain,
with the builder's store adapter and keep, hold lifecycles in flags
(`hearth/docs/design/domain/store.md`, `hearth/docs/design/domain/render.md`). The activator, steward
and their link, remaining crates and support are absent; printed errors,
task data and lint reasons keep citations without the repository-root prefix
(`hearth/docs/design/README.md`). Machine layout, boot and bootstrap, slots
and live changes, the privilege boundary, the persistent steward and the wire
remain unbuilt (`hearth/docs/design/domain/machine.md`,
`hearth/docs/design/domain/slots.md`, `hearth/docs/design/domain/activator.md`,
`hearth/docs/design/domain/steward.md`, `hearth/docs/design/protocol/wire.md`),
as do operating, development machines, artifacts and reproducible builds
(`hearth/docs/design/domain/operating.md`,
`hearth/docs/design/domain/development.md`,
`hearth/docs/design/domain/artifacts.md`,
`hearth/docs/design/domain/reproducible.md`). Tests have the worlds' suites,
git, resolution and render conformance, and tool fakes, but lack the complete
M2 fake machine, joined system world, real builds and boot test; conformance
has no automatic runner. Tier profiles, memory tests and shared suite
timeouts depart from the design, the focused suite stands over its 30 CPU s,
by about 15 on one record and 0.2 or about 2 on the gate's, and measured costs await the user
(`hearth/docs/design/domain/testing.md`). The design departs
from the draft in its three processes, subordinate ids, offline sandboxes,
development loop, live changes and milestone split; pool placement still
lags, and budgets, credentials, check policy and other decisions remain open
(`hearth/docs/design/draft/brainstorm.md`).
`hearth/docs/design/protocol/format.md` has no gap.

Design documents:

- [`README.md`](README.status.md)
- [`protocol/format.md`](protocol/format.status.md)
- [`domain/render.md`](domain/render.status.md)
- [`domain/store.md`](domain/store.status.md)
- [`domain/build.md`](domain/build.status.md)
- [`domain/machine.md`](domain/machine.status.md)
- [`domain/slots.md`](domain/slots.status.md)
- [`domain/activator.md`](domain/activator.status.md)
- [`domain/steward.md`](domain/steward.status.md)
- [`protocol/wire.md`](protocol/wire.status.md)
- [`client/README.md`](client/README.status.md)
- [`domain/operating.md`](domain/operating.status.md)
- [`domain/testing.md`](domain/testing.status.md)
- [`domain/development.md`](domain/development.status.md)
- [`domain/artifacts.md`](domain/artifacts.status.md)
- [`domain/reproducible.md`](domain/reproducible.status.md)

Drafts:

- [`draft/brainstorm.md`](draft/brainstorm.status.md)
