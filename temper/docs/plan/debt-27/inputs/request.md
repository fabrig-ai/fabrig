# Request: pay debt items, twenty-seventh round

From fabrig/debt, 2026-11-06 (the user's continuous tech-debt process; source of truth: each tree's
`docs/development/debt/`; `python3 -I -B tools/debt/debt.py` prints the index). Independent small items, one task each. fabrig/ui has
priority over builder places (the user, 2026-11-05), so keep the plan small, with no temper client view, no
`temper.css` and no `jig-client-view`. Measure with inline `#[cfg(test)]` items split out and generated code counted apart.
New tests must earn their lines: a table or helper, not copies. A table written to pin a refactor is reduced, before
the plan ends, to the rows that still bite. A size or memory pin that shrinks because of the task's own change is updated,
and the report says so; one that grows stops and asks fabrig/debt. A question about this request's own words goes to fabrig/debt.

A measure reading 0 doesn't pay an item. A renamed flag, a tuple that only beats a lint, or `Some(_) | None` passes the search and pays nothing: if the honest shape isn't reachable, keep the item open or record the departure.

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

## The items

- `skein/docs/development/debt/D004-utf8-checks.md`
- `skein/docs/development/debt/D007-joined-and-measure.md`
- `skein/docs/development/debt/D040-dead-public-api.md`
- `skein/docs/development/debt/D041-workspace-duplicates.md`
- `skein/docs/development/debt/D042-shell-backend-copies.md`
- `jig/docs/development/debt/D011-channel-mechanics-copied.md`
- `jig/docs/development/debt/D037-copied-goldens.md`
- `jig/docs/development/debt/D041-unused-parameters.md`
- `jig/docs/development/debt/D043-length-widening.md`
- `smith/docs/development/debt/D020-hand-set-token-tags.md`

Items may have changed since they were written: re-measure first. An item already paid gets deleted, saying what paid
it. Stay out of `jig-client-view`. These exist to remove code, so each should end with fewer lines than it started.
Size S: if one turns out bigger, stop and ask fabrig/debt rather than grow it.
