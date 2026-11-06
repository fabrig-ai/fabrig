# goal-delivery: a goal's run writes design, and lands it through a change

`temper/docs/design/draft/goal-as-designer.md` makes temper's goal its own designer: its first run surveys, writes the design and makes the task tree with `change`. Today it can't write: a goal's runs inspect alone. This plan builds that draft's section 4, and the part of section 3 that rests on it (step 2 and the `after` of step 3). The draft wins where other designs differ, and its status file names the sections that lag.

## What's wanted, ranked. Drop from the bottom if it passes six tasks, and say what you dropped

1. **The failing story first, in the lowest tier that holds it:** a goal's run modifies its checkout, delivers to its own run branch `temper/<root>/r<task>`, makes `change {"branch": "run", "landing": "goal", "words": "..."}`, and that change lands into g by g's queue. A change `after` it starts early on its pushed head. Along the way: merges into g go on while the run works, as it takes no writer slot of g; the project's checks run at delivery, as for a producer; the goal's result stays a report; closing the change deletes nothing.

2. **A goal may deliver with no change among its outcomes.** `jig/crates/jig-protocol-charter/src/charter.rs`, near line 115: `deliver` comes from `outcome.change`, and line 132 grants `modify` only with `deliver`. Say in jig's terms what grants them, and what `finish` forms such a run is offered. The mechanism is generic, so jig never names temper; a chat's, an advisor's and a reviewer's runs still inspect alone, and a test holds that.

3. **`"run"` beside `"mine"`:** `temper-engine-domain/src/boundary.rs` `ChangeBranch`, `route/forge_route.rs` `change_branch`, `temper-engine-domain-forge/src/domain.rs` `change_on_held_branch`, and the `change` tool's words. No producing step, its head the branch's tip as read at opening, held by the goal, one at a time. A conflict goes to the merge job and a resolution, as for any change.

4. **Later runs deliver too.** A run starts from g's tip, with whatever the run branch holds that g lacks merged in, so its delivery fast-forwards the run branch. Nothing builds this start today. Walk a second run's delivery, and a run branch left holding what g lacks.

5. **The design says so:** `temper/docs/design/domain/agent.md` 4.3 and 4.3.1, `forge.md` 8.1, 11 and 11.1, jig's `domain/hosts.md` section 8, and the draft's status entries this settles.

## Not in this plan

- The room of a call, a batch, a brief and a report: the next plan, `call-room`. Keep this plan's words and test arguments small, so it doesn't lean on them.
- The words of the stock and reference charters, their wakes, their move to files, and which model or effort a role takes. Test charters carry the grants this needs. Another plan is moving models into the store and changing the stock charters, so stay out of `stock.rs` where you can, and expect a catch-up where you can't.
- `temper/docs/design/draft/reference-code-production.md` stays unedited: tests and a judge read it.

Failing tests first; no test reaches a real host. **A plan runs the focused and fuzzy suites alone (`AGENTS.md`, 5).** A stored format or protocol change is a question first, so its owner can be told; both main areas write `temper-engine-domain` and `forge_route.rs`. Formats change in place before v0.1.0. Budgets and timeouts of test tiers are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). `walk = false`.
