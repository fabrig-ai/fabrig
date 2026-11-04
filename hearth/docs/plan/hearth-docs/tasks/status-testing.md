+++
id = "status-testing"
part = "status"
kind = "docs"
needs = ["names-docs"]
files = ["hearth/docs/design/domain/testing.md", "hearth/docs/design/domain/testing.status.md"]
effort = "high"
minutes = 60
+++

# Status for testing

## Goal

`hearth/docs/design/domain/testing.md` holds most of what `STATUS.md` says today, since hearth's tests joined the root's tiers. It gets a
status file checked against the worlds, closes with `Open questions`, and loses its build state.

Acceptance: `notes/status.md`, its section "Acceptance, in every status task". Beyond it, each measured figure of the seed's 9.1 stays
with its date, and `domain/testing.md`, 11 still holds what the figures await.

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/notes/status.md`
- `hearth/docs/plan/hearth-docs/notes/contracts.md`, 3
- `DESIGN.md`, 1 and 2
- `skein/docs/foundation/testing-strategy.md`, 8.1
- `.config/nextest.toml`, `.config/gate/README.md`, `tools/tiers/design.md`
- `hearth/docs/development/debt/D003-memory-tests-on-formulas.md`

## Owns and may change

- **Owns:** the two files under `files`.
- **Not to touch:** `hearth/docs/design/STATUS.md`, code, `.config/`, and any budget or timeout: a figure is the user's.
- **Options:** the document's profiles, budgets and the like stay as design and the root's differences go in **Departs**, as the seed
  has them. A section number already wrong in the text isn't yours: report it.

## Steps

1. **Seed:** the row's Not yet: 6.6: the boot test and the other tiers; 3.11: the shared fake machine. Built: 2.9, 5, 10. "Other tiers"
   is the seed's vagueness: replace it with sections. Carry over, re-checked, in root citations, every bullet of
   `hearth/docs/design/STATUS.md` that starts `domain/testing.md`: the **Not yet** on 5.5 and the **Departs** on 5.5 and 9.2, on 8.4 and on
   9.1. Code: `hearth/tests/*`, `hearth/testing/*`, the root's tier profiles.
2. Follow `notes/status.md`, 1 to 4.
3. Commit (`hearth: testing's status`), run the checks in `notes/common.md`, push.

## Report

As `notes/status.md` says.
