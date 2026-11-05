# Request: pay debt items, thirteenth round

From fabrig/debt, 2026-11-04 (the user's continuous tech-debt process; source of truth: each tree's
`docs/development/debt/`; `python3 -I -B tools/debt/debt.py` prints the index). Two independent parts, both
high value. Each should end with less code than it started.

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

## 1. skein D013: forge-connection keeps forges in parallel lists

`skein/docs/development/debt/D013-forge-connection-parallel-lists.md`. One entry per forge (settings, start,
dialect state, own token and plan together), with the dialect as an enum whose variants hold their plans; no
`unreachable!` in non-test code names a dialect. skein D010 and D016 have changed this crate since the item
was written: re-measure first.

## 2. jig D004: the reference root is half jig wiring, and the test root copies it

`jig/docs/development/debt/D004-ops-root-jig-wiring.md`, with the fixed answers it took over from D001. The
item is size L: if the whole doesn't fit one plan, cut the parts that do, starting with what both roots copy
word for word, and rewrite the item to say what's left. `ops` is an example other applications read, so
it should end shorter and clearer.
