+++
id = "records"
part = "records"
kind = "docs"
needs = []
files = ["jig/docs/development/debt/D004-ops-root-jig-wiring.md", "skein/docs/design/link.status.md"]
effort = "medium"
minutes = 10
+++

# Record two of the user's decisions of 2026-11-06

## Goal

Two records, in words the spec gives. No code.

Acceptance:

- `jig/docs/development/debt/D004-ops-root-jig-wiring.md` says the user deferred rebuilding the test root on 2026-11-06, its status is `kept`, and it stays open.
- `skein/docs/design/link.status.md`'s Departs entry for `skein-oauth-accounts/src/exchange.rs` says the user left it, and why.
- `python3 -I -B tools/debt/debt.py jig` prints D004 as `kept`, and exits 0. `python3 -I -B tools/citations/citations.py --all` stays at 0.

## Inputs

- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`, whole.
- `skein/docs/design/link.md`, 1, *A stream with a close rule*.
- `temper/docs/plan/debt-26/notes/common.md`.

## Owns and may change

Owns the two files in `files`. The status file's edit is a stated departure from the implementer's text. Nothing else: `jig/docs/design/domain/root.md`, `hosts.md` and their status files cite D004 by a path that doesn't change, and what they say stays true (the order is the user's; the default holds).

If the entry in `link.status.md` no longer reads as below, keep whatever else it has gained and change only its reason.

## Steps

In D004's header:

- `- **Checked:** 2026-11-06`
- `- **Status:** kept (the user deferred rebuilding the test root, 2026-11-06; what is left below aligns the two roots or waits on that same decision)`

Run the item's **Measure** and put the number you read in place of `3,563 at the checked trunk`; do the same for the `ops` measures `3,563 non-test lines` in **What**, and `route.rs`'s `1,400` if it has moved. Leave `3,836 at the plan's base`, and **Paid when** whole.

In **What**, the first bullet becomes:

> - The user deferred rebuilding the test root on 2026-11-06. This item stays open for it, `kept` until they take it up.

Leave the other bullets.

In `skein/docs/design/link.status.md`, the date line says `2026-11-06`, and the entry reads:

> - **Departs:** 1: `skein-oauth-accounts/src/exchange.rs` reads `encrypted` for a
>   move, not a size: it pumps at once when its link isn't, because removing that
>   retimes requests. The user left it so, 2026-11-06.

Run the two tools named in the acceptance. Commit, push.

## Report

As your text says: the numbers you read, and that no test ran because no code changed.
