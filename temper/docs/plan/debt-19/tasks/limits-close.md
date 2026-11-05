+++
id = "limits-close"
part = "limits-once"
kind = "docs"
needs = ["limits-core", "limits-link"]
files = ["jig/docs/development/debt/D009-limits-mapped-by-hand.md", "jig/docs/development/debt/D007-contract-written-by-hand.md"]
effort = "medium"
minutes = 10
+++

# jig: D009 closes, and what's left of it goes to D007

## Goal

`jig/docs/development/debt/D009-limits-mapped-by-hand.md`'s **Paid when** holds once `limits-core` and `limits-link` have landed: check it, and delete the item.

Acceptance:

- `rg -o '\.min\(w::CEILINGS' jig/crates/jig-client-protocol-link/src/limits.rs | wc -l` reads 0, and `rg -n '(tasks\.revisions|tasks\.versions|fleet\.versions)' jig temper -g '*.rs'` finds nothing.
- The item is deleted. `python3 -I -B tools/debt/debt.py jig` exits 0, and the citations check reads 0.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `jig/docs/development/debt/D009-limits-mapped-by-hand.md`
- `temper/docs/plan/debt-19/plan.md`, Decisions: D009

## Owns and may change

- **Owns:** the files in `files`, and any citation of D009 that breaks.
- **`D007`'s **What pays it back** says D008 and D009 are its short-term steps. Add to its **What**, in one bullet each:**
  - the link's `limits.rs` lists each class's members by hand, once per class: bound classes in the schema generate it;
  - `temper-settings-form` (`lib.rs`, `fields.rs`, `kinds.rs`) holds its own map with 36 `.min(CEILINGS`, and `jig-protocol-engine-clients/src/translate` narrows with 22 more at encoding. Count them first, since they may have moved.
- **The six client `Limits` types stay,** one per concept, as the plan says, and nothing changes for them.
- **No code changes.** If a search above doesn't hold, that's a `Failed:` naming it.

## Steps

1. Run the searches.
2. Edit `D007`, delete `D009`. The citations check. Commit and push.

## Report

The common note's items and the counts you wrote.
