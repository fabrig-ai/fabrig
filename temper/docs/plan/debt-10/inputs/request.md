# Request: pay debt items, tenth round

From fabrig/debt, 2026-11-04 (the user's continuous tech-debt process; source of truth: each tree's
`docs/development/debt/`). Three independent parts, from the spot review of debt-7 and debt-8 (`inputs/debt7-8-review.md`), which was read and not run: check each finding against the code as it stands first, and skip a wrong one, saying why. Each pays its items, so that each item's
**Paid when** holds, and nothing else.

Every part, on its own branch:

- reads the item file whole, and the design text it cites;
- makes the change so the item's **Paid when** holds; if the item has a **Measure**, the command
  reads 0 (or the item's target) after;
- **deletes the item's file and its row** in that tree's `docs/development/debt/README.md`, in the
  same branch;
- updates any status entry the item names as closed by it, and any citation the deletion breaks
  (`tools/citations/citations.py --all` stays at 0);
- writes acceptance searches narrowly: code that legitimately uses the same word for something else
  is exempted, not changed. Never pin an exact HEAD: fab merges the trunk in at each run's start.

**Behaviour stays unchanged, except where part 1 says otherwise.** If a test's visible events or
outcome change, stop and ask. Don't retime a test or rewrite an assertion to make a refactor pass.
Stored formats and breaking public interfaces stop and ask too; additions are fine.

**A test that pins a rule must be able to fail on that rule.** Break the rule by hand, watch the test
go red, restore it, and report what was broken and what failed.

**Keep new code in proportion.** Size caps in tasks are the cut's estimates, not user budgets. Compact
first. If a cap still can't be met without losing wanted behaviour, an answer call may raise it by up
to 25%, stating the measured size and why. Beyond that, ask. Budgets and timeouts stay the user's.

**All parts must be in the plan when the cut ends.**

## 1. Findings B1 and B2: credentials and a panic

- **B1:** `Workers` gets a hand-written `Debug` that counts, with a canary test, as service.md 11 asks.
  While there, check every other type in the shipped parts that can hold those bytes.
- **B2:** first show the panic with a test on `[::]:7100`. If it's real, fix it: refuse at `Clients::new`,
  as `provider.rs` does, unless `temper/docs/design/protocol/configuration.md` means such an address to work, in
  which case print any v6 address. This is a bug fix, so the behaviour change is allowed here. If the design
  doesn't settle it, stop and ask. `address_text` and `HOST_BYTES` move beside `printed_link`.

## 2. jig D005 and temper D001: the bundle helpers

`jig/docs/development/debt/D005-accounts-host-copied.md` and
`temper/docs/development/debt/D001-reference-service-copy.md`, with finding B3. Both items' **Paid when** hold
and both items go. Then B4.

## 3. Tidying

Findings A1 to A5 and B5 to B7. B5 comes first: a test that can't fail on its rule is the point.
For B7, keep to the renames, the accessor and the helper; a shared trait across parts is out of scope.
For A3, prove `domain.work` is empty at that `Resume` first; if it isn't, skip A3. This part should end
with fewer lines than it started.
