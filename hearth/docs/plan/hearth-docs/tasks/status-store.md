+++
id = "status-store"
part = "status"
kind = "docs"
needs = ["citations-docs"]
files = ["hearth/docs/design/domain/store.md", "hearth/docs/design/domain/store.status.md", "hearth/docs/design/domain/build.md", "hearth/docs/design/domain/build.status.md"]
effort = "high"
minutes = 55
+++

# Status for the store and the build

## Goal

`hearth/docs/design/domain/store.md` and `build.md` are where the line runs through M2: part is built and part isn't. They get status
files checked against the code section by section, close with `Open questions`, and lose their build state.

Acceptance: `notes/status.md`, its section "Acceptance, in every status task".

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/notes/status.md`
- `hearth/docs/plan/hearth-docs/notes/contracts.md`, 3
- `DESIGN.md`, 1 and 2
- `hearth/docs/design/README.md`, 3

## Owns and may change

- **Owns:** the four files under `files`.
- **Not to touch:** `hearth/docs/design/STATUS.md`, code.
- **Options:** `notes/status.md` settles what's a gap. A section number already wrong in the text isn't yours: report it.

## Steps

1. **Seed, `store.md`:** built: 3.1, 3.2; 2.1, 2.4, 4.4, 6.1, 6.4 to 6.6, 6.8, 8.1, 8.2; 6.2, 5.3; 4.2, 7.3. Not yet: other layers and
   store; 10: peers. "Other" is the seed's vagueness: replace it with the sections it means, read from the code. 10 is `[M6]`: no entry
   unless a started milestone needs it. Code: `hearth-domain-store`, `hearth-domain-tree`, `hearth-protocol-tree`, `hearth-lib`,
   `hearth-protocol-accounts`.
2. **Seed, `build.md`:** built: 3.3, 4.1, 4.3: keys, plan, stage, seal, discard; steps are refused. Not yet: steps and the remaining
   build: replace it with sections. Code: `hearth-domain-build`, `hearth-domain-builder`, `hearth-protocol-builder`,
   `hearth-service-builder`, `hearth/tests/builder`.
3. Follow `notes/status.md`, 1 to 4, for each document.
4. Commit (`hearth: store and build's status`), run the checks in `notes/common.md`, push.

## Report

As `notes/status.md` says.
