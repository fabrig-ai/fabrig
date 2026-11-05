+++
id = "accounts-names"
part = "world-verbs"
kind = "feature"
needs = []
files = ["jig/tests/accounts/src/lib.rs", "jig/tests/accounts/tests/**"]
effort = "high"
minutes = 45
+++

# jig's accounts world: rows with names

## Goal

Jig D035's largest share, 86 of the Measure's 262: `account(101, 1, HOLDER, b"")` and `source(10, 11, 0, kind)`, with `start_restore.rs` alone at 45.

Acceptance:

- `accounts/src/lib.rs` names the provider once, and gives a builder for the common row, so a row reads what differs: `held(101, b"")`, say. The two `issued` copies merge.
- **Numbers that are the point stay:** a restore's ordering (`account(800, ..)` after `Next { number: 900 }`, `account(20, ..)` and `account(22, ..)` beside `.account(20)`) is what those tests say. Name it where a name says more, and keep the row otherwise.
- `start_restore.rs`'s rows that differ only in data become a table: the request asks for a table or a helper, not copies.
- The Measure over `jig/tests/accounts` reads at most ten, and each left is in the report with why.
- `heavy cargo nextest run -p jig-accounts-world` passes, with no assertion changed. Tests' net lines fall by at least 30.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`

## Owns and may change

- **Owns:** the files in the header.
- **No codemod:** ordering may matter here, so read each row. A table row that would change an order stays as it is.

## Steps

The names; a file at a time. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-accounts-world --all-targets -- -D warnings`; the test above. Commit and push.

## Report

The common note's items; the Measure before and after; what stays. No new rule: nothing to break by hand.
