# Request: pay debt items, twelfth round

From fabrig/debt, 2026-11-04 (the user's continuous tech-debt process; source of truth: each tree's
`docs/development/debt/`). Two independent parts.

Every part, on its own branch:

- reads the item file whole, and the design text it cites;
- makes the change so the item's **Paid when** holds; if the item has a **Measure**, the command
  reads 0 (or the item's target) after;
- deletes any item it pays, in the same branch; updates any status entry the item names as closed by it, and any citation that breaks
  (`tools/citations/citations.py --all` stays at 0);
- writes acceptance searches narrowly: code that legitimately uses the same word for something else
  is exempted, not changed. Never pin an exact HEAD: fab merges the trunk in at each run's start.

**Behaviour stays unchanged.** If a test's visible events or
outcome change, stop and ask. Don't retime a test or rewrite an assertion to make a refactor pass.
Stored formats and breaking public interfaces stop and ask too; additions are fine.

**A test that pins a rule must be able to fail on that rule.** Break the rule by hand, watch the test
go red, restore it, and report what was broken and what failed.

**Keep new code in proportion.** Size caps in tasks are the cut's estimates, not user budgets. Compact
first. If a cap still can't be met without losing wanted behaviour, an answer call may raise it by up
to 25%, stating the measured size and why. Beyond that, ask. Budgets and timeouts stay the user's.

**All parts must be in the plan when the cut ends.**

## 1. skein-link: the spot review of skein D016

`inputs/link-review.md`, findings 1 to 6 and 8 to 10, in order of value. It was read, not run: check each
finding against the code first, and skip a wrong one, saying why. **Leave finding 7 alone:** removing oauth's
plaintext shim retimes requests, which is the user's call. Finding 1's test must fail with `link.fire`
deleted. Finding 2 must keep the bytes out of `Debug`, with a canary test. Production code should end about
65 lines lower, apart from the tests. No debt item closes here.

## 2. Generate the debt index

Each tree's `docs/development/debt/README.md` keeps a hand-written index table. Items paid in parallel in one
tree conflict on its rows, and every conflict costs a rejection and a repair: seen in debt-5, debt-9 and
debt-10. So the table goes:

- A small `tools/debt/` command prints the index of one tree or all five, from each item file's title, kind,
  value, size and status. One command from the repository's root, Python under `-I -B`, no Cargo, no new
  dependencies, exit 0. It needs no gate. If an item file lacks a field, it reports that file and exits 1.
- Each tree's README drops its table and says how to print it. Rewrite the rule at the top: a branch that pays
  an item deletes its file; there is no row to delete. Items that say "and its row" change with it.
- Keep the rest of each README as it is. Item IDs are never reused: note in each README the highest ID used,
  or derive it from history, so a paid ID isn't handed out again.
