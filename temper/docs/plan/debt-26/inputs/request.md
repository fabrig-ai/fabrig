# Request: pay debt items, twenty-sixth round

From fabrig/debt, 2026-11-06 (the user's continuous tech-debt process; source of truth: each tree's
`docs/development/debt/`; `python3 -I -B tools/debt/debt.py` prints the index). Four independent parts. fabrig/ui has
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

## 1. Records of the user's decisions (2026-11-06)

- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`: rebuilding the test root is deferred by the user's word, 2026-11-06. The item stays open, status `kept` for that part if it's all that's left.
- `skein/crates/skein-oauth-accounts/src/exchange.rs`'s plaintext shim contradicts `skein/docs/design/link.md`, section 1. The user left it: removing it retimes requests. Record it as a Departs entry in `skein/docs/design/link.status.md`, naming the file and why.
- `Some(_) | None` in the Forgejo plans (`inputs/debt22-review.md`, finding 2), decided by fabrig/debt under the user's standing word, for the user's later review: one rule in `skein/docs/design/forge.status.md` ("the exchange's decoder picks the kind", a departure from `skein/docs/foundation/programming-model.md`, 10.3), `_ =>` under a named `#[expect]`, the five wrappers and their `Option` parameters gone, and the row lists too if they fall out cleanly.

## 2. New debt items

D030 and D031 closed mostly by moving code (`inputs/debt24-review.md`, findings 1 and 2). Open two new items in their trees, each with the next free ID (a paid ID is never reused), citing the review:

- skein: two harnesses over forge-connection sit in one package, with near-twin tests. Pay it here: a seam-gated fixtures module both read, and cut twins, ignoring harness bookkeeping asserts when comparing. Delete the item if paid in substance; otherwise leave it saying what's left.
- jig: worlds are shims over kits holding the worlds, with the modules mutually cyclic, so a fixture change still breaks dependents. Open it; don't pay it here. Correct `skein/docs/design/testing.md`'s "in any tree" at its line 386, with the gate's real reach (`tests/boundary/tests/skein_worlds.rs:72`).

## 3. Leftovers

`inputs/debt24-review.md`, findings 3 and 4: the seam's shapes, and leftover copies. It was read, not run: check each against the code first, and skip a wrong one, saying why. Net lines should fall.

## 4. The spot review of debt-25

`inputs/debt24-review.md`, its second half ("Spot review of debt-25"), findings 1 to 6. Finding 1's rows must go red on the realistic break, printing the raw-bytes fields. Finding 3 undoes a move an earlier review asked for, because it cost more than it paid: fabrig/debt's decision. Check each against the code first, as above. Net lines should fall.
