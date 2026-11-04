# Request: finish jig D001 as the user decided

From fabrig/debt, 2026-11-04. The user answered `d001-scope` with (c): `Activate` stays in its decision,
so a release stays one commit, and only `PrepareAgent` leaves the core's vocabulary. One part.

`jig/docs/development/debt/D001-roots-sequence-core-steps.md`. Its **Paid when** can't hold as
written: of its five arms, only two could leave the core without a design change, and the user chose
to remove one.

1. `Now::PrepareAgent` and `Event::PreparedAgent` go, as `inputs/held-core-prepare-agent.md` says, but
   that spec was written on top of `core-due-activation`, which is dropped. It must stand alone on main:
   `Now::Activate` stays, activations run in the same decision as today, and the added `Busy` route the
   question named is built here. The branch `debt-5/core-prepare-agent` on origin is for reading only.
2. Rewrite D001's **Paid when** to what stands (a `Now` value stays when any root does real work with
   it), confirm the measure reads 39, and delete the item and its row in the same branch.
   The fixed answers ops and the test root keep to `BriefCorePlanned` and `WorkspaceRequest` belong to
   jig D004; add that to D004's text.

**Behaviour stays unchanged.** Don't change `reviews.rs`'s same-commit release check, or any test's
visible events or outcome. If one changes, stop and ask: don't retime a test or rewrite an assertion
to make the change pass. Stored formats and breaking public interfaces stop and ask too.

**A test that pins a rule must be able to fail on that rule.** Break the rule by hand, watch the test
go red, restore it, and report what was broken and what failed.

**Size caps in tasks are the cut's estimates, not user budgets.** Compact first. If a cap still can't be
met without losing wanted behaviour, an answer call may raise it by up to 25%, stating the measured
size and why. Beyond that, ask. Budgets and timeouts stay the user's.

**All parts must be in the plan when the cut ends.** Keep `tools/citations/citations.py --all` at 0.
