# Request: pay debt items, fourteenth round

From fabrig/debt, 2026-11-04 (the user's continuous tech-debt process; source of truth: each tree's
`docs/development/debt/`; `python3 -I -B tools/debt/debt.py hearth` prints the index). Two independent parts.

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

## 1. hearth: small items

`hearth/docs/development/debt/D011-ratchet-counts-two-forms.md` first, so new copies stop, then
`D005-private-hex-decimal-octal.md`, `D008-path-modules.md` and `D012-render-test-shapes.md`, one task each.
fabrig/hearth is migrating hearth to the dynamic memory model at the same time, and owns D001 to D003: leave
`hearth/tests/*/tests/suite/memory.rs`, `hearth-service-builder/src/lib.rs` and the machines' `MAX_OUT` forms alone.
If a task needs those, stop and ask.

## 2. smith: two small items

`smith/docs/development/debt/D017-domain-worlds-on-protocols.md` and `D018-domain-formats-text.md`, one task each.
