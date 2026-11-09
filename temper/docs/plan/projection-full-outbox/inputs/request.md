# projection-full-outbox: a goal's issue projection waits for room instead of spinning

A real deployment's engine spun at 100% CPU inside one `Engine::step`, logging nothing, answering no client and ignoring SIGTERM. It did so twice: for 3.5 hours on 2026-11-08, and again on 2026-11-09 a minute after five goals were started at once. Its stacks sit under `domain::close -> finish_close -> route_into -> forge_event -> forge::domain::project -> forge_issues::domain::project`. Evidence is outside this repository, so the words here are what you have. **Run, unless marked read.**

**The loop, as read in the code:**
- `route_into` (`temper-engine-domain/src/route.rs`, about 417) pops `domain.work` until it is empty.
- `Work::Forge(ProjectDesired)` reaches `project()` (`temper-engine-domain-forge/src/domain.rs`, about 1559).
- When the issue projection yields an effect but the forge outbox is full (`d.entries.len() == env.limits.entries`, about 4962), it emits `ProjectAfter { goal, when: None }` and changes nothing.
- The root (`route/forge_route.rs`, about 5203) answers `when: None` on a row with no pending entry by pushing a fresh `ProjectDesired` at once, and the outbox is still full.

An entry frees only by a forge commit or settle, which is I/O that can't arrive mid-step, so the loop never ends. The outbox holds 8 entries (`temper-service/src/limits.rs`, about 200), which six goals' forge writes can fill.

## What's wanted, ranked

1. **The failing test first,** in the forge domain's tests or the engine world: fill the forge outbox to its bound, then ask to project a goal. The step ends, with the projection due later and nothing spinning. Add a row where an outbox entry frees and the projection then goes out.
2. **The fix, in `temper-engine-domain-forge`:** on a full outbox, the projection waits. Either set its due time as `fail_projection` does (`projection_due` at `wall + issue_policy.interval`, emitting `ProjectAfter { when: Some(..) }`), or re-emit it when an outbox entry frees. Choose by the code's own idiom, and say which. Also, as a backstop: no `ProjectAfter { when: None }` may re-queue work in the same step that already met a full outbox.
3. **The design says so:** `forge.md` wherever a goal's issue projection and the outbox's bound are written.

## Not in this plan

Any figure: the outbox's bound stays as configured.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test; no test reaches a real host. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** `walk = false`.
