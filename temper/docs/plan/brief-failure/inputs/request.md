# brief-failure: a brief that can't be gathered says why, ends, and survives one section

Found in the first real goal on a deployment, 2026-11-05, at `f9955bf`. A change's CI went red on a real Forgejo; its repair task has been `Retrying`, `retry: refused`, count 25 and rising, for hours, with no attempt, no `run_began`, nothing in the engine's or worker's log, and no reason in `temper ctl`. The change sits idle in `repairing`. The person can't tell why, and nothing ends it.

## The route, from reading (confirm it)

- `retry: refused` comes from `jig_domain_tasks::run::pause`, reached by `preparation_failed` (`jig/crates/jig-domain-tasks/src/run.rs`, about 659 and 680), from `BriefAssembled { ready: false }` or `brief::GatherRequest::Failed`/`Refused` (`jig/crates/jig-domain-core/src/routing/children.rs`, 69; `routing/run/preparation.rs`, `brief_assembled`). A pause has no bound.
- A CI repair's brief plans two forge sections, `Pull` and `Ci` (`temper/crates/temper-engine-domain/src/assemble.rs`, about 458 to 510). Both are required: a required section's failure ends the brief (`jig/crates/jig-domain-brief/src/orchestrated.rs`, `update`, `fail`, and `Missing::Failed` and `Missing::Budget`).
- In `temper/crates/temper-engine-domain-forge/src/brief.rs`, any forge read error at those stages falls to `BriefRead::Failed` (the catch-all arm near line 410), except `TooLarge` at `Compare`, which says `[comparison omitted: forge client limit exceeded]`, and a job's log, which says `[job log could not be read]`.
- The engine's journal shows real reads of that repository refused `InvalidAnswer` around the same time, from an agent's `read_pull_for` and `read_compare` tools. The brief's own reads are the likely culprit: the deployment's API answers are being captured, and the reason will reach this plan as an answer to a task. Don't wait for it.

## The change

1. **It says why.** Whichever way a brief fails, the reason reaches where a person looks: the engine's log line naming the task, the section and the forge's error, as a tool read's `read refused` line does; and `temper ctl task show` and the chip's retry notice. Formats change in place before v0.1.0, so adding it to a record is allowed; keep it bounded, and it carries no LLM-written or secret words.
2. **It ends.** A brief failing for a reason that isn't capacity (a forge read that answers wrongly, a section that can't fit) doesn't pause without bound. Count it as a failure of its class, with its allowance, to a hold that names the reason, as other failures do (`failure`, `Hold::Failures`), so a person sees a held task and can act. A true capacity refusal (a place, an account, a brief slot) keeps today's pause. Settle which is which by `jig/docs/design/domain/engine.md`, 7 and 11, and name each in the design.
3. **One section doesn't sink the brief.** A forge read error on a section the run can do without becomes words in that section, as the job's log does: `[the pull request could not be read: REASON]`, the agent having its forge tools to try again. Decide by `temper/docs/design/domain/forge.md`, 8.4 which sections stay required: a repair with neither its reason nor its pull can't start, and goes to (2).
4. **The cause,** when it reaches this plan: a parser or a check in `skein/crates/skein-forge-client` (`api.rs`, `compared`) refusing a real Forgejo's answer, with that answer as a fixture. finding-323, a real compare listing each file twice, is a landed cousin.

## Tests

A failing test first for each, in the lowest tier: `jig/tests/core` for (1) and (2), `temper/tests/engine` forge world for (3), a story of a CI repair whose `Pull` read answers `InvalidAnswer`, once for a section it can do without, once with both failing and ending held. No test reaches a real host.

## Size

Three or four tasks; jig is shared with hearth, whose manifests name no jig crate, so name the change in `plan.md`. `skein-forge-client` isn't on hearth's surface. fabrig/ui has priority: leave its files alone, and mind the client's wire if it's moving. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Walk

Mark as `walk = true`: on a real Forgejo, a change whose CI goes red gets a repair that starts, and a repair whose brief can't be read ends held with the reason in `temper ctl`.
