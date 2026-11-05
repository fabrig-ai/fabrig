+++
id = "args-close"
part = "payloads"
kind = "feature"
needs = ["args-core-party", "args-core-turns", "args-core-calls", "args-core-checks", "args-tasks", "args-fleet-host", "args-people-views", "args-small", "args-store", "args-ops"]
files = ["jig/docs/development/debt/D016-positional-payloads.md"]
effort = "medium"
minutes = 10
+++

# D016 goes

## Goal

The item's **Paid when** holds, so it goes.

Acceptance:

- `rg -n 'too_many_arguments' jig/crates jig/examples -g '!**/tests/**' -g '!**/tests.rs' -g '!**/generated/**'` reads two lines: `jig-domain-accounts/src/selection.rs` and `jig-protocol-worker-agents/src/sources.rs`, each beside a comment naming it as kept whole on purpose. That's the item's own exception, and this is the item's target (`notes/arguments.md`, "Seams kept whole").
- `jig/docs/development/debt/D016-positional-payloads.md` is deleted.
- `python3 -I -B tools/citations/citations.py --all` reads 0, `git grep -n 'D016-positional' -- jig` finds nothing, and `python3 -I -B tools/debt/debt.py | rg 'D016'` shows no jig row.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/arguments.md`

## Owns and may change

- **Owns:** the file in the header. No code.
- **If another draft has added a site since this plan's base** (the search reads more than two): fold it if it's a line or two in a file no one holds, and say so; otherwise stop and ask, with the sites. The item stays until the search reads two.

## Steps

1. The search; the deletion; the checks. Commit and push.

## Report

The common note's items; the search's count.
