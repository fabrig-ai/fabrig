+++
id = "status-run"
part = "status"
kind = "docs"
needs = ["names-docs"]
files = ["hearth/docs/design/protocol/wire.md", "hearth/docs/design/protocol/wire.status.md", "hearth/docs/design/domain/operating.md", "hearth/docs/design/domain/operating.status.md", "hearth/docs/design/domain/development.md", "hearth/docs/design/domain/development.status.md", "hearth/docs/design/domain/artifacts.md", "hearth/docs/design/domain/artifacts.status.md", "hearth/docs/design/domain/reproducible.md", "hearth/docs/design/domain/reproducible.status.md"]
effort = "medium"
minutes = 30
+++

# Status for the documents of later milestones

## Goal

`hearth/docs/design/protocol/wire.md`, `domain/operating.md`, `domain/development.md`, `domain/artifacts.md` and `domain/reproducible.md`
belong to milestones nobody has started. Each gets a one-line status, closes with `Open questions`, and loses any build state.

Acceptance: `notes/status.md`, its section "Acceptance, in every status task".

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/notes/status.md`
- `hearth/docs/plan/hearth-docs/notes/contracts.md`, 3
- `DESIGN.md`, 1 and 2
- `hearth/docs/design/README.md`, 3

## Owns and may change

- **Owns:** the ten files under `files`.
- **Not to touch:** `hearth/docs/design/STATUS.md`, code.
- **Options:** nothing built means a one-line **Not yet** (`notes/status.md`, 3). `domain/reproducible.md` comes after M6 and no milestone builds
  it: its line says so.

## Steps

1. **Seeds:** `wire.md`: 2, 4: the faces and standalone mode; 5, 9: the wire; 7: telemetry; 8: bytes by digest. `operating.md`, `artifacts.md`,
   `reproducible.md`: nothing built. `development.md`: nothing built: mutable machines, capture and apply, Incus, clusters, containers and VMs.
2. `rg` over `hearth/crates` for each document's main types, to confirm nothing is built.
3. Follow `notes/status.md`, 1 to 4, for each document. Expect few build-state sentences: `artifacts.md`, 7.4's precedent and the like are
   design.
4. Commit (`hearth: later milestones' status`), run the checks in `notes/common.md`, push.

## Report

As `notes/status.md` says.
