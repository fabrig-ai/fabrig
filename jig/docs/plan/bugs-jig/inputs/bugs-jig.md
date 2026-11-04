# bugs-jig: the jig bugs the debt backfill routed

From `coord/debt/for-director.md` (fabrig/debt, 2026-11-03; the user approved routing these). Paths starting with a crate's name are under `jig/crates/`; line numbers are at `a6a8155d35` or older, so locate each again.

Rules for this plan:

- Keep to the code's bugs below; nothing else from the debt list. Low priority: small tasks, no reshaping.
- Re-check every bug marked "not re-checked" or "partly checked" against main first, by survey. One that no longer holds is dropped and named in the report, with what was looked at.
- Every fix carries a test that fails on the old code.
- Bug 3 asks whether the code or the design is wrong. Decide from `temper/docs/design/domain/agent.md` and jig's charter design; a design change is a design task in both trees, not a question, unless the design genuinely can't settle it.
- Bug 1 is the widest. If it needs more than the wire, `people::Member` and the template identity it names, cut it as its own part.
- Keep every part in the plan. Wait for a designer before reporting `Done:`.

### jig (owner: jig)

1. **Close grace taken from the first conversation template.** Seen. `jig/examples/ops/domain/src/route.rs`, the `infrastructure::Request::Close` arm (line 1073). Fixing it means carrying a task's template identity through the client wire, `people::Member`, `tasks::New` and `TaskRecord`, and `engine.md` says a task keeps no template: design first. (Also jig debt D004.)
2. **A deleted project's watches may never end.** Partly checked: `close_views` (`jig-domain-core/src/routing/view/routes.rs`, line 81) is called only from `tests/reads/closed.rs`. Nobody confirmed that no other path ends them.
3. **The charter's deliver rail.** Seen. `jig-protocol-charter/src/assignment.rs` grants `policy.modify` whatever the delivery, but temper's `temper/docs/design/domain/agent.md` says a run that cannot deliver cannot write. A bug, or a design change in both trees.
4. **The client aborts on wire-valid records.** Seen, partly paid. `jig-client/src/facts.rs` still `.expect`s `ended_at` (line 40) and `retry` (line 63), and both are optional on the wire.
5. **Config's Small fallback.** Not re-checked. `jig-protocol-config/src/lib.rs` (line 88) starts from `Profile::Small` before it parses `limits`. Nobody checked whether problems still cascade from it.
6. **The test service rebuilds smith's `SourceName` with an `expect`.** Seen, test kit only. `testing/jig-test-service/src/program_sources.rs`, line 113, `.expect("same source name grammar")`.
7. **The test service's `Resources` derives `Debug`, and its environment entries may hold credentials.** Seen (temper's triage). `jig/testing/jig-test-service/src/program_sources.rs`, `#[derive(Debug)]`. temper's copy writes a `Debug` that shows only counts.
8. **A component stream can stay Closing for good.** Not re-checked. Source: `coord/qa-ladder/log.md`, 11-01 04:04Z ("core's Unwatch of an unknown watcher answers nothing").
9. **The quality run's hazards.** None re-checked; each is one line in `jig/docs/development/debt/reviews/2026-10-31-jig-quality-run.md`:
   - project 0 doubles as the deployment sentinel;
   - people request tokens and core flight handles share values;
   - authority's model mapping is asymmetric with people's;
   - a period can open for a closed project;
   - notes keep a deleted project's entries;
   - the test service's token nibbles (review bug 6);
   - the document bound for `Recipient::Roles` (review bug 8);
   - views' `Fact::Delivered` comes at request time;
   - `WatchEnd::Stopping` is terminal in the client domain but jig-client retries it;
   - the store clones the model catalogue for every row.

