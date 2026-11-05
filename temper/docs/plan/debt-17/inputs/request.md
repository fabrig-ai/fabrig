# Request: pay debt items, seventeenth round

From fabrig/debt, 2026-11-05 (the user's continuous tech-debt process; source of truth: each tree's
`docs/development/debt/`; `python3 -I -B tools/debt/debt.py` prints the index). Small independent items, one task
each. fabrig/ui has priority over builder places (the user, 2026-11-05), so keep this plan to these tasks, with no
temper client view and no `temper.css`. Measure with inline `#[cfg(test)]` items split out.

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

In `skein/docs/development/debt/`: `D003-env-rewraps.md`, `D005-civil-calendars.md`, `D006-lru-tables.md`,
`D008-http-head-parsing.md`, `D019-channel-machine-flags.md`, `D026-crate-tests-directories.md`,
`D034-codec-double-generation.md` and `D044-io-token-tags.md`.

In `jig/docs/development/debt/`: `D027-engine-clients-cursor-format.md`.

In `hearth/docs/development/debt/`: `D008-path-modules.md`, its last two `#[path]`s. fabrig/hearth is migrating
hearth's memory model, so stay out of `hearth-service-builder/src/lib.rs` and the machines' `MAX_OUT` forms; if a
`#[path]` needs them, say why beside it and stop there.

Items may have changed since they were written: re-measure first. An item already paid gets deleted, saying what
paid it. Apart from D019, which turns flags into states, each should end with less code than it started.
