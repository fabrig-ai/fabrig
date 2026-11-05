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
