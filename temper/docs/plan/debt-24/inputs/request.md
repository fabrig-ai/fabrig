# Request: pay debt items, twenty-fourth round

From fabrig/debt, 2026-11-05 (the user's continuous tech-debt process; source of truth: each tree's
`docs/development/debt/`; `python3 -I -B tools/debt/debt.py` prints the index). Two independent parts. fabrig/ui has
priority over builder places (the user, 2026-11-05), so keep the plan small, with no temper client view, no
`temper.css` and no `jig-client-view`. Measure with inline `#[cfg(test)]` items split out and generated code counted apart.
New tests must earn their lines: a table or helper, not copies. A table written to pin a refactor is reduced, before
the plan ends, to the rows that still bite. A size or memory pin that shrinks because of the task's own change is updated,
and the report says so; one that grows stops and asks fabrig/debt. A question about this request's own words goes to fabrig/debt.

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
to 25%, stating the measured size and why. Beyond that, ask fabrig/debt, the plan's owner, not the user. Budgets and timeouts stay the user's.

**All parts must be in the plan when the cut ends.**

## 1. The spot review of debt-22

`inputs/debt22-review.md`, findings 1 and 3 to 8. It was read, not run: check each against the code first, and skip a wrong one,
saying why. **Leave finding 2 alone:** it's a departure from `programming-model.md`, 10.3, and the user hasn't decided it. Check its
two correctness notes at the end. Finding 3's sign-in fields stay out of `Debug`. No debt item closes here. Net lines should fall.

## 2. Test copies

- `skein/docs/development/debt/D028-worlds-as-kits.md`
- `skein/docs/development/debt/D030-forge-connection-harness-tests.md`
- `jig/docs/development/debt/D029-protocol-front-one-identity.md`
- `jig/docs/development/debt/D031-worlds-as-kits.md`
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`

Each exists to remove a copy or widen what a test can see, so each should end with fewer test lines, unless a story is newly
held, which the report names. Items may have changed since they were written: re-measure first.
