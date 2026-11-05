+++
id = "lifecycles-close"
part = "builder-lifecycles"
kind = "docs"
needs = ["builder-child", "builder-resolve", "builder-read", "builder-client", "builder-current", "store-states"]
files = ["hearth/docs/development/debt/D006-lifecycles-in-flags.md"]
effort = "medium"
minutes = 15
+++

# D006 goes

## Goal

The item's **Paid when** holds, so it goes. It has no Measure, and it's a reviewer's reading: **Paid when** says a reviewer finds one state enum per lifecycle, no `closing` or `closed` flag beside a state, and `Protocol` holding one current call's state. These searches stand in, and you read the six types by eye.

Acceptance:

- Each reads nothing, under `hearth/crates/hearth-protocol-builder/src/`:
  - `rg 'closing: bool|closed: bool|closing_resources|struct Closing' git.rs`
  - `rg 'closing: bool|terminal: Option|indices: Option|walker: Option' resolve.rs`
  - `rg 'cache: Option|body: Option|decode: Option|decoded: Option|room: Option|finished_input' read.rs`
  - `rg 'active: bool|finished: bool|closing: Option|broken: Option' client.rs`
  - `rg '^\s+(child|mode|keeping|resolving|call): Option' lib.rs`; `ran` may read here if `builder-current`'s report says it coexists.
- Under `hearth/crates/hearth-domain-store/src/`: `rg '(usable|said|finding): bool|(entering|writing|working): Option' lib.rs` reads nothing.
- `hearth/docs/development/debt/D006-lifecycles-in-flags.md` is deleted.
- `python3 -I -B tools/citations/citations.py --all` reads 0, `git grep -n 'D006-lifecycles' -- hearth` finds nothing, and `python3 -I -B tools/debt/debt.py | rg 'D006'` shows no hearth row.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/builder-states.md`, "Facts stay flags": what stays beside a state, and why.

## Owns and may change

- **Owns:** the file in the header. No code.
- **If a search doesn't read nothing,** stop and ask: the item stays until it does.
- **What's beyond the item** stays open, named in your report: `Stage { Live, Closing }`-style enums aren't flags; `Working` and `Scan` are outside its **Where**.

## Steps

1. The searches; a read of the six types; the deletion; the checks. Commit and push.

## Report

The common note's items; each search's count; what the read of the six types found.

## Answers

### 2026-11-05: Exempt the state-local `Phase::Locking { said: bool }` from the Store acceptance search (recommended), or require store-states to remove that flag before D006 can close? (debt-23/lifecycles-close-1, foreman debt-23/lifecycles-close/foreman-1)

Exempt the state-local `Phase::Locking { said: bool }` from the Store acceptance search; report the exemption and resume the closure review.
`temper/docs/plan/debt-23/notes/common.md`: “Acceptance searches are narrow: code that legitimately uses the same word for something else is exempted, not changed; your report names any exemption the spec does not.”
`skein/docs/foundation/programming-model.md`, 5.4: “each variant holding exactly what that state holds”.
In `hearth/crates/hearth-domain-store/src/lib.rs:27`, `said` belongs to `Phase::Locking`, rather than sitting beside the state.
`start.rs` uses it to arm the waiting-notice timer only on the first Busy event.
The next run records this exemption, completes the six-type review, and, if acceptance otherwise holds, deletes D006, runs the required checks, commits and pushes.
