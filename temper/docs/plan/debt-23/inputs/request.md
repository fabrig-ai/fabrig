# Request: pay debt items, twenty-third round

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
A size or memory pin that shrinks because of the task's own change is updated, and the report says so; one that grows stops and asks. "The plan's owner" means fabrig/debt, not the user: a question about my request's own words comes to me.

## 1. The spot review of debt-21

`inputs/debt21-review.md`, findings 1 to 8. It was read, not run: check each against the code first, and skip a wrong one,
saying why. Finding 1, the latent bug, comes first, with its pin failing before the fix. Findings 3 and 6 are decided:
derive the deadline from the state, and make the five fields `Secret` and derive `Debug`. Finding 8's value-mismatch abort
comes back; say so in `jig-domain-core`'s status if the spec had recorded otherwise. Record or redact the review's caveat
about oauth fields that now print raw. No debt item closes here. Net lines should fall.

## 2. Items

- `skein/docs/development/debt/D023-llm-client-states.md`
- `jig/docs/development/debt/D016-positional-payloads.md`
- `hearth/docs/development/debt/D006-lifecycles-in-flags.md`

Items may have changed since they were written: re-measure first. fabrig/hearth is migrating hearth's memory model: stay
out of `hearth-service-builder/src/lib.rs` and the machines' `MAX_OUT` forms; if D006 needs them, stop and ask fabrig/debt.
