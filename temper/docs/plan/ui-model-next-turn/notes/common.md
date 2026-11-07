# ui-model-next-turn: for every task

The request: `temper/docs/plan/ui-model-next-turn/inputs/request.md`; its section *For every task of this plan* holds for you, and wins over your spec where they differ. The design is written and on the trunk: `jig/docs/design/domain/tasks.md`, section 7, the bullet *On a task its requester ends, the next run is the next turn*.

- **The cause, as this plan's cut read it.** A temper chat is an Open run (`temper/docs/design/domain/agent.md`, section 4): after a reply it listens for its charter's waiting time (the stock's 10 seconds) before it parks. A model change is an `Amend`, which `jig/docs/design/domain/tasks.md`, section 7 gives to the next run. So:
  - the amendment's own word, `MessageKind::Amendment`, has `WakeRule::Immediate` (`jig/crates/jig-domain-tasks/src/wake.rs`, `rule`): it wakes an idle chat on words the person never typed ("Change next run"), and it is relayed into a listening run, whose old model answers it;
  - the person's next words go to that same live run, on the old model.
- **This was read, not run.** The tests are the proof. If one shows something else, follow what it shows: fix the cause at the lowest tier that shows it, and say so in your report.
- **Not smith.** `smith/crates/smith-domain/src/route.rs`, `resumed_model`, does what `smith/docs/design/domain/session.md`, 8.2 says and doesn't change: a run opened on a new model compacts first, with the window's own model, then answers on the new one. `model-catalogue-2/smith-limits` is changing that code; don't touch it or take on its work. A request naming the old model is expected once after a model change, and only as the summary.
- **Failing tests first,** at the lowest tier: write each test, see it fail for the rule's reason, then the code. Your report names the rule and the failure seen.
- **Only focused and fuzzy run in this plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, a walk or a render check.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely, with no compatibility and no `Question:`; say so in your report. A small edit your task needs just outside its file list is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **A test's oracle or helper that the change's correct behaviour trips is yours to repair,** keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- **Goldens and generated files** your change makes stale are yours to regenerate by their own tool in the same task; on a catch-up, regenerate rather than hand-merge.
