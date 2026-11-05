# Request: pay debt items, twenty-second round

From fabrig/debt, 2026-11-05 (the user's continuous tech-debt process; source of truth: each tree's
`docs/development/debt/`; `python3 -I -B tools/debt/debt.py` prints the index). Two independent parts. fabrig/ui has
priority over builder places (the user, 2026-11-05), so keep the plan small, with no temper client view, no
`temper.css` and no `jig-client-view`. Measure with inline `#[cfg(test)]` items split out and generated code counted apart.
New tests must earn their lines: a table or helper, not copies. A table written to pin a refactor is reduced, before
the plan ends, to the rows that still bite.

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

## 1. jig: the spot review of debt-20

`inputs/debt20-review.md`, findings 1 to 6. It was read, not run: check each against the code first, and skip a wrong one,
saying why. Finding 2 comes first: four tests that can pass with their rule broken must go red when it breaks, or go.
Finding 1 removes about 400 lines of tables that repeat production's lists, keeping the rows that can still fail.
Finding 4: `store.md` gains its two sentences, and `Write::Copy` its tests. No debt item closes here. Net lines should fall.

## 2. Two state items

`skein/docs/development/debt/D021-forgejo-plan-states.md` and `skein/docs/development/debt/D022-oauth-states.md`. Flags
become states as `skein/docs/foundation/programming-model.md`, 5.4 asks, keeping what's asked no more than once (the
dialect isn't asked again, a state holds its own data). Credentials stay out of `Debug`. Items may have changed since
they were written: re-measure first.
