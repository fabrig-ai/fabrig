+++
id = "brief-counts"
part = "brief-counts"
kind = "feature"
needs = ["brief-words", "retry-reason"]
files = [
  "jig/crates/jig-domain-core/src/routing/children.rs",
  "jig/crates/jig-domain-core/src/routing/mod.rs",
  "jig/crates/jig-domain-core/src/routing/run/preparation.rs",
  "jig/crates/jig-domain-core/src/routing/run/jobs.rs",
  "jig/crates/jig-domain-core/src/routing/run/results.rs",
  "jig/crates/jig-domain-core/src/vocabulary.rs",
  "jig/tests/core/tests/stories/brief_failure.rs",
  "jig/tests/core/tests/suite/stories.rs",
  "jig/testing/jig-test-domain/src/routes.rs",
]
effort = "high"
minutes = 150
+++

# A brief's failure counts by why, and ends held

## Goal

Every way a task's preparation fails goes to `PreparationFailed`, a pause without bound. Route each by the table in `jig/docs/design/domain/engine.md`, 9.1: capacity pauses as it does, saying why; the rest count in their class to a hold that names the reason. The root hears each failure for its log.

Acceptance:

- A failing test first for each of stories 1 and 2 below, in `jig/tests/core`; the report quotes each failure on the code before the route.
- All stories pass; each ends in the restart oracle.
- Each row of 9.1 that the core owns goes where the table says, each with those words.
- `tasks::Event::PreparationFailed` and `PreparationUnread` are gone, with their functions in `run.rs`; `PreparationPaused` and `PreparationCounted` are what's left.
- **This departs from the implementer's text,** for two entries alone: delete `jig/docs/design/domain/engine.status.md`'s "9.1" **Not yet** entry and `jig/docs/design/domain/tasks.status.md`'s "5.2, 5.5" **Not yet** entry. Temper's connector still answers capacity until `forge-sections`: that is forge's status, not these.

## Inputs

- `temper/docs/plan/brief-failure/notes/common.md`; `temper/docs/plan/brief-failure/notes/contracts.md`, 1 to 3.
- `jig/docs/design/domain/engine.md`, 7.1, 9 and 9.1; `jig/docs/design/domain/tasks.md`, 5.5.
- `skein/docs/foundation/testing-strategy.md`, section 1. `jig/AGENTS.md`.

## Owns and may change

Owns the files in `files`.

May also change:

- `jig/crates/jig-domain-tasks/src/{boundary,domain,run}.rs`: remove `PreparationFailed` and `PreparationUnread`. Their tests move to `PreparationPaused` or `PreparationCounted`, keeping what they assert: `jig/crates/jig-domain-tasks/src/tests/revisions.rs`, `jig/crates/jig-domain-tasks/src/tests/startup/reports.rs`, `jig/tests/tasks/tests/suite/{added,lifecycle,holds}.rs`, `jig/tests/tasks/src/world.rs`, and `jig/crates/jig-domain-core/src/routing/flight.rs`'s arm.
- `jig/crates/jig-domain-core/src/routing/effect/note.rs`, the `NoteRoute::Brief` arm's `Full` and its other refusals: two lines. **Another draft is changing this file:** touch nothing else.
- `Now::BriefFailed`'s arm, empty, wherever `Now` is matched in full: `jig/tests/client/src/joint/engine.rs`, `jig/tests/core/src/client_peer.rs`, `jig/tests/ops/src/joint/engine.rs`, `jig/examples/ops/domain/src/route.rs`, `jig/testing/jig-test-service/src/route.rs`, `jig/testing/jig-test-service/src/clients/translation.rs`, `jig/testing/jig-test-store/src/route.rs`, and any other the compiler names.
- `ready: true` becomes `failure: None`: `jig/crates/jig-domain-core/src/tests/runs/windows.rs`, `jig/examples/ops/domain/src/route.rs`.
- temper's lines naming these, a departure from "a branch stays in its tree" for a required field:
  - `temper/crates/temper-engine-domain/src/assemble.rs`, its three `ready: false`, by 9.1's row of a root that can't put a brief together, all `GatherFate::Permanent`: the lost section's `brief` with `a section's owner no longer holds it`; `plan_home_access`'s `home` with `no room for its required section`; the room check's `brief` with `no room for its required forge sections`. `ready: true` becomes `failure: None`.
  - `temper/crates/temper-engine-domain/src/route/now.rs`: `core::Now::BriefFailed { .. } => {}`. `forge-sections` fills it.
  - `temper/crates/temper-engine-domain/src/tests/documents.rs`: its pattern.
  - A temper story that leans on a pause it no longer gets: **read why before you change it.** Where 9.1 says it now counts, change its expectation and name it in the report. Where not, it's a `Question:`.

Options, pre-approved:

- The world's fault is yours: `jig-test-domain` gains a test-only `script_brief_failure(Option<brief::GatherFailure>)` beside `script_workspaces`; while set, `Now::BriefCorePlanned` answers `BriefAssembled { failure: Some(..) }`. A better seam you find is fine; name it.
- The child's own route (`tag_brief`) is covered by a unit test in `jig/crates/jig-domain-core/src/tests/`, with a plan holding a required connector section and a `GatherEvent::Missing`.
- A story can't reach a row of 9.1 through the world (`no attempt number is left`, a full proof table): a unit test of the routing function, and say so.
- A story whose claim the start settles opts out of the oracle as its neighbours do, with `world.broken()` and the same named finding. No new reason to opt out.

Leave alone: `jig/crates/jig-domain-notes/**`, `wait_for_account` and `PreparationWaiting`, which stay as they are (9.1), and `Now::RunPreparationFailed`, which still tells the root to drop what it holds.

## Steps

1. The vocabulary of `notes/contracts.md`, 3. `Work::PreparationFailed` and `route_preparation_failed` take the failure.
2. One function, beside `route_preparation_failed`, turns a `(task, GatherFailure)` into the tasks event and `Now::BriefFailed`, using `failure.reason()`. Every site below calls it:

   | Site | Fate | Section | Words |
   |---|---|---|---|
   | `tag_brief`, the child's `Failed` and `Refused` | the child's | the child's | the child's |
   | `brief_assembled`, the root's failure | the root's | the root's | the root's |
   | `brief_assembled`, no `fresh` run number | permanent | `run` | `no attempt number is left` |
   | `preparation.rs`, `jobs.rs`: no proof room | capacity | `run` | `no room for its proof` |
   | `results.rs`: no read flight | capacity | `results` | `no read flight is free` |
   | `results.rs`: a dependency's result is gone | transient | `results` | `a dependency's result could not be read` |
   | `start_brief`: `read_brief_part` gives `None` | permanent | `task`, `results`, `transcript` by the part | `could not be read within its bounds` |
   | `TranscriptLoad::Failed`, a page `append_transcript` refuses | transient | `transcript` | `its window could not be loaded` |
   | `transcript_unread`, where it counted | transient | `transcript` | `its window could not be read` |
   | `route_claim_refused` | capacity | `claim` | `refused` |
   | `note.rs`, `Refusal::Full` | capacity | `notes` | `no room for the index` |
   | `note.rs`, every other refusal but `Busy` | permanent | `notes` | `the index could not be read` |

3. Stories, new `jig/tests/core/tests/stories/brief_failure.rs`, in `jig/tests/core/tests/suite/stories.rs`:
   1. `a_brief_that_cannot_be_read_counts_to_a_hold_naming_it`: permanent; the notice says the reason, count 1 of the allowance; past it `Held`, `Hold::Failures(Permanent)`, the words on the task's document's hold; no assignment; `Now::BriefFailed` once for each failure.
   2. `no_room_for_a_brief_pauses_past_any_allowance_and_then_runs`: capacity; the refused count passes every class's allowance and the task is never held; the notice says why; clear the script and the run starts.
   3. `a_release_counts_again_and_runs`: from story 1's hold, clear the script, release; the run starts.
   4. `a_late_section_counts_as_transient`: the child's `Late`, through `tag_brief` or the script.
4. `cargo fmt`; old forms for jig; `heavy cargo clippy -p jig-domain-core -p jig-domain-tasks -p jig-test-domain -p temper-engine-domain --all-targets -- -D warnings`.
5. This changes what a shared item means: run its readers. `heavy cargo nextest run -p jig-domain-core -p jig-domain-tasks -p jig-core-world -p jig-tasks-world -p jig-test-domain -p temper-engine-domain -p temper-engine-domain-world -p temper-service`. Check each count.
6. Commit, push.

## Report

As the implementer's text has it, with each story's first failure and each temper story whose expectation moved.
