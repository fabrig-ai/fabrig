+++
id = "status-format"
part = "status"
kind = "docs"
needs = ["names-docs"]
files = ["hearth/docs/design/protocol/format.md", "hearth/docs/design/protocol/format.status.md", "hearth/docs/design/domain/render.md", "hearth/docs/design/domain/render.status.md"]
effort = "high"
minutes = 50
+++

# Status for the format and render

## Goal

`hearth/docs/design/protocol/format.md` and `hearth/docs/design/domain/render.md`, both M1's, get status files checked against the
code, close with `Open questions`, and lose their build state.

Acceptance: `notes/status.md`, its section "Acceptance, in every status task".

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/notes/status.md`
- `hearth/docs/plan/hearth-docs/notes/contracts.md`, 3
- `DESIGN.md`, 1 and 2
- `hearth/docs/design/README.md`, 3
- `hearth/docs/development/debt/D002-process-worst-case.md`

## Owns and may change

- **Owns:** the four files under `files`.
- **Not to touch:** `hearth/docs/design/STATUS.md`, code.
- **Options:** `notes/status.md` settles what's a gap. A section number already wrong in the text isn't yours: report it.

## Steps

1. **Seed, `format.md`:** Not yet: 11: checks; 10.2: artifacts. Code: `hearth-protocol-config`, `hearth-domain-render` (its `schema`),
   `hearth-protocol-template`, `hearth-protocol-accounts`.
2. **Seed, `render.md`:** no Not yet. Departs, the bullet in `hearth/docs/design/STATUS.md` starting `domain/render.md`, 8.4: carry it
   over, re-checked, in root citations. Code: `hearth-domain-render`, `hearth-protocol-builder`, `hearth-protocol-git`,
   `hearth-protocol-mmdebstrap`, `hearth-client-protocol-records`, `hearth-service-builder`, `hearth/tests/render`.
3. Follow `notes/status.md`, 1 to 4, for each document.
4. Commit (`hearth: format and render's status`), run the checks in `notes/common.md`, push.

## Report

As `notes/status.md` says.
