# Request: pay debt items, sixteenth round

From fabrig/debt, 2026-11-05 (the user's continuous tech-debt process; source of truth: each tree's
`docs/development/debt/`; `python3 -I -B tools/debt/debt.py` prints the index). Three independent parts, from
the spot review of debt-13 (`inputs/debt13-review.md`), which was read and not run: check each finding against
the code as it stands first, and skip a wrong one, saying why. Measure with inline `#[cfg(test)]` items split
out: a test moved out of a production file is not a production saving.

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

## 1. skein-forge-connection: findings 2, 4, 5, 8, 9 and 10

Finding 2 first: the dialect isn't asked again, so `Side` comes out of `Waiting` in `drive_calls`. Finding 4: the
own `Exchange` goes wherever `own` clears, and its test stands on what production does. `skein/docs/development/debt/`
may hold nothing for these: they're follow-ups, and no debt item closes.

## 2. jig: findings 3, 6 and 7

Finding 3 needs `Now::Link(..)`, nesting the six link variants, which touches 13 core construction sites: temper's
root follows if it matches them. If that's a public break, stop and ask fabrig/debt. Where a shared helper doesn't pay
(finding 6), move it back to the root, and update `jig/docs/design/domain/root.md` where it names one.

## 3. jig D004: the parts that need no decision

`jig/docs/development/debt/D004-ops-root-jig-wiring.md`: `ops`'s delegate decoding and alert wording move to its
protocol layer, the fixed-refusal template shrinks without moving capability choices into jig, and temper's
consumers use the shared parts (job assignments, settled relays, messages, sender labels, `Class`). The test root
keeps its shape: rebuilding it is deferred, and D004 stays open for that. Rewrite the item to say so.
