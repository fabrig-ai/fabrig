+++
id = "status-run"
part = "status-run"
kind = "feature"
needs = ["project-facts", "status-charter", "status-read"]
files = [
  "temper/crates/temper-engine-domain-forge/src/status.rs",
  "temper/crates/temper-engine-domain-forge/src/status/words.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/status.rs",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route/status_route.rs",
  "temper/crates/temper-engine-domain/src/route/asks.rs",
  "temper/crates/temper-engine-domain/src/admission.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/status.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes.rs",
]
effort = "high"
minutes = 90
+++

# The status agent's run: made, delegated, heard

## Goal

At a new sitting with something new, the forge has an agent write Home's brief: `temper/docs/design/domain/forge.md`, 5.7, whole.

Acceptance:

- **When,** each clause of 5.7's: only at a look that moves `previous`; `landed` above the brief's; the brief over an hour old or none; none in flight; a run in flight over an hour counts as none.
- **Made:** `forge::Request::StatusRun { project, words }`; `start_status_run`, as `start_tier_run`, on `repair_seed`, `Executor::Procedure { connector, code: STATUS_CODE }`, `STATUS_CODE` 5 beside `AFTER_LANDING_CODE`, contract `Report`; `Event::StatusTask { project, task }`. No seed, or `core::Ask::RepairRefused` naming that task: `Event::StatusRefused { project }`, which clears `running`.
- **Delegated:** at `core::Ask::StartProcedure` with `STATUS_CODE` (`asks.rs`, beside `AFTER_LANDING_CODE`), `start_status_step`: its first step's `ProcedureStep.batch` holds one `Delegate`: `Executor::Agent { charter: STATUS_CHARTER }`, 9; the procedure's words; `asks` false; `tasks::Flags::PLAIN`; `Contract::Report`; `child_authority`, then tools 0, no grants, no notes; `wake` default. A restart's second offer doesn't delegate again.
- **No workspace:** `run_workspace` (`forge_route.rs`, `adopted`) selects no repository for a task whose charter is `STATUS_CHARTER`.
- **Heard:** a later step reads its inbox as `foreman::settled` does. `Result(Report)` with a non-empty first paragraph: `Event::StatusHeard { task, report: Some(words) }`; `Failed`, `Cancelled`, a refused delegate, an empty report: `report: None`. Then the procedure's `StepEnd::Result`.
- **The forge hearing:** `task` names `running`: the first paragraph, split at the first blank line and trimmed, is `sentence`, the rest `lede`, each `text::fit` to `BRIEF_SENTENCE_BYTES` and `BRIEF_LEDE_BYTES`, `at` the wall, `landed` `running`'s; saved, `status_subject` marked, `running` cleared. No `sentence`: nothing saved but `running` cleared. A `task` it doesn't name is dropped.
- **Words,** `status/words.rs`, bounded by `tasks.spec_bytes`: the span since the last brief, in hours; the landed count; titles from `landing_logs`, since the brief's `at`, newest first, at most 20; each tier's `record.failing` count and first names. The root adds the goals' lines: a project's live goals, title and `delegates_done` of `delegates_total`, from `core.tasks`.
- `domain/forge.status.md` and `protocol/configuration.status.md`'s first *Not yet* entries go; the reference's stays.

## Inputs

- `temper/docs/plan/ui-home-status/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/domain/forge.md`, 5.6, 5.7, 8.11; `temper/docs/design/domain/after-landing.md`, section 3.
- The models: `route/forge_route/tier_route.rs`, `make_tier_run`, `start_tier_step`; `route/forge_route/foreman.rs`, `delegate`, `settled`; `relay.rs`, how `batch` holds a `Delegate` and a refused delegate is heard (`:152`, `:436`); `asks.rs`, `RepairRefused` and `StartProcedure`.

## Owns and may change

Owns, as `files` lists them. May change the status lines above.

May also change: `temper-engine-domain/src/route.rs` for a `mod` line; `temper/tests/engine/forge/src/world.rs` and `temper-engine-domain-forge/src/tests.rs` for compile-forced arms of the new `Request`s and `Event`s; `temper-engine-domain/src/admission.rs`'s list.

Options, each pre-approved:

- **Telling a first step from a later one:** the row's children, if jig's `TaskRecord` holds them as `change_delegate_status` reads them; else `StatusRunning` gains `delegated: bool`, in `notes/contracts.md`'s record, saved. Choose the first if it serves a restart.
- **The goals' lines** cost more than about 30 lines, or need a query core doesn't have: send them as "N live goals", the count `HasGoals`' query would find, and say so.
- **Failing tests since the last brief:** the forge keeps no times on them, so it is what each tier holds now; that's 5.7's text.
- **A held child never ending:** `running`'s hour is the answer; nothing cancels it, and its late result is dropped.
- **Spend:** `child_authority` returns none at no spend: that is a refused run, nothing to ask.

## Steps

1. **The forge:** `status.rs`'s `looked` asks; `StatusTask`, `StatusRefused`, `StatusHeard`, `status/words.rs`. Unit tests in `tests/status.rs`: `a_new_sitting_with_something_new_asks_a_run`; `a_first_look_asks_none`; `a_look_within_the_sitting_asks_none`; `nothing_new_asks_none`; `a_brief_under_an_hour_old_asks_none`; `a_run_in_flight_asks_none`, then `past_an_hour_it_counts_as_none`; `a_report_splits_into_sentence_and_lede`, with `text::fit`'s bounds; `an_empty_report_saves_nothing`; `a_failed_run_saves_nothing_and_asks_nobody`; `a_late_result_is_dropped`; `a_restored_run_is_still_in_flight`; `its_words_say_what_landed_since_the_brief`, at most 20.
2. **The root:** `status_route.rs`, `forge_route.rs`'s `Request` arm, `asks.rs`, `run_workspace`.
3. **The world,** `suite/forge_routes/status.rs`, shaped as `foremen.rs`, the charter's run answered by a script (`reviews.rs`, `finish_review`), the clock by `World::tick` or `driver.env.now`: `a_look_after_a_landing_runs_the_agent_and_saves_its_brief`, through the document; `the_run_holds_no_tools_no_grants_and_no_workspace`; `its_task_is_unlisted_and_asks_nobody`; `a_refused_run_lets_the_next_sitting_try`; `a_restart_mid_run_still_saves_the_brief`.
4. **Check,** the implementer's text; the citations check; then push.

A change of meaning: a look now may spend. Run `temper-engine-domain-world`'s and `temper-client-world`'s suites that name `Looked` or `status`.

## Report

As `notes/common.md`, Your report, with how a first step was told, and the goals' words.
