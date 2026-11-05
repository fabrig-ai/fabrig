+++
id = "oauth-states-close"
part = "oauth-states"
kind = "docs"
needs = ["oauth-client-phase", "accounts-entries"]
files = ["skein/docs/development/debt/D022-oauth-states.md", "skein/docs/design/oauth.status.md", "skein/docs/design/link.md"]
effort = "medium"
minutes = 15
+++

# D022 goes, and oauth's status says what's left

## Goal

The item's **Paid when** holds, so it goes, with what the status says of it.

Acceptance:

- These read nothing: `rg -n 'self\.phase (==|!=)|self\.active\b|Option<Exchange>' skein/crates/skein-oauth/src/machine.rs skein/crates/skein-oauth/src/machine`; `rg -n '\b(announced|held|rejected|attempted|due): (bool|true|false)\b|\brejection\b' skein/crates/skein-oauth-accounts/src/component.rs skein/crates/skein-oauth-accounts/src/component`; `rg -o '(self|component)\.(accounts|names|sources|bindings|keepers)\b' skein/crates/skein-oauth-accounts/src/component.rs skein/crates/skein-oauth-accounts/src/component`.
- `skein/docs/development/debt/D022-oauth-states.md` is deleted.
- `skein/docs/design/oauth.status.md`'s **Departs** on 3 reads, whole:

  > - **Departs:** 3: `skein/docs/foundation/programming-model.md`, 5.4 asks
  >   for one explicit state enum per lifecycle, each state holding exactly
  >   its data, no lifecycle in flags or `Option`s beside it, and one handler
  >   per cell; `skein-oauth-accounts/src/exchange.rs` keeps `pending_grant`,
  >   `aborted` and `file_active` beside `Exchange::stage`, and neither
  >   crate's machine keeps one handler per cell; no reason is recorded.

  The item's **Where** named `exchange.rs`, and its **Paid when** doesn't, so the status keeps that clause without the item. Its date becomes today's. No other entry changes.
- `skein/docs/design/link.md`, 5's line reads `(`D023` of `skein/docs/development/debt/`)`.
- `python3 -I -B tools/citations/citations.py --all` reads 0, and `git grep -n 'D022-oauth-states' -- skein` finds nothing.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`, with this departure: this task edits the two design files above, in the words above.
- `DESIGN.md`, 2, 4

## Owns and may change

- **Owns:** the files in the header. No code.
- **If a search doesn't read nothing,** stop and ask: the item stays until it does.

## Steps

1. The searches; the three files.
2. `python3 -I -B tools/debt/debt.py | rg D022` finds nothing; the citations check. Commit and push.

## Report

The common note's items; each search's count.
