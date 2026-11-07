# goal-delivery: the rules every task keeps

- **Failing test first, in the lowest tier.** The test goes in before the code, and the report names how it failed. No test reaches a real host: the fake forge and the scripted model.
- **Focused and fuzzy suites alone** (`AGENTS.md`, 5). Never `tools/tiers/run`, a `<tier>-<tree>` profile or `scripts/browser.sh`. `temper-real-loop-world`'s `suite` binary is focused.
- **No stored format and no protocol changes.** The binding calls in `contracts.md` change none: `ForgeRepository.delivers` and `Charter.delivers` stay in the engine's process, and a change on a run branch is stored as one on a goal branch is. If your work would change a schema, a generated codec, a golden or a wire message, stop and ask: its owner must be told first.
- **Test charters carry the grants.** `walking::charter()` has `modify: true`, and `World::configured` takes it, so a goal in those worlds has the grant. Stay out of `temper/crates/temper-service/src/stock.rs`, `temper/crates/temper-service/src/stock/*.txt` and `temper/docs/examples/reference/`: no stock or reference word, wake, model or effort changes here. `temper/docs/design/draft/reference-code-production.md` stays unedited.
- **`model-catalogue-2` is beside this plan.** It writes `temper/crates/temper-engine-domain/src/route/**`, `jig/crates/jig-protocol-config/**`, `jig/crates/jig-domain-core/src/settings/**` and `temper/crates/temper-service/src/tests/stock*.rs`. Keep your edits in `forge_route.rs` to the functions your task names. A merge there is a catch-up, not a question.
- **`call-room` is the next plan:** the room of a call, a batch, a brief and a report. Keep words and test arguments small so that nothing here leans on them, and raise no bound.
- **Budgets and timeouts are the user's.** No task raises one.
- **`walk = false`:** no task claims a real model ran this.
- **Not yours:** `docs/design/` and `docs/help/`, as the implementer's text says, except where your task's `Owns and may change` names a file.
