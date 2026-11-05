# start-refuses: a release at a start withdraws its stale proposals without a panic

From the redeploy of 2026-11-05: a restart with a run's pending settings proposal and changed defaults crash-loops. The request (`inputs/request.md`) has the backtrace.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| `server-walk` | `server-walk` | | the walk's two additions, each with its help test |
| `release-withdrawal` | `release-withdrawal` | | the failing test, the fix in the core, the stories of a held revision and a crash |

Critical path: `release-withdrawal`. The two share no file, so they run side by side.

## What was decided

- **The withdrawal stays in the release's decision.** `jig/docs/design/domain/engine.md`, 17.6 has the decision that makes a revision withdraw its stale proposals. A later commit would leave a crash window: the revision made, the defaults no longer differing, so no later start would withdraw a proposal now pending against an older revision. In one decision a crash leaves both or neither.
- **People take a projection event from their link on.** The events are `ChatState` and `Waiting`, in memory for a task that stays live, and a restart rebuilds them. The census is whole from the link, so holding them in a new queue until `start_complete` would buy nothing. Loading still panics.
- **The withdrawal's activation and relay are dropped at `Phase::Link`, not held.** Today they sit behind the panic, and would reach the root before the switch, which section 6 forbids. A held activation would double the one `tasks::Event::Restored` makes for a task left `Due`, and a held relay would go out before the run is adopted. Dropped, the start makes both from the durable records, so a first start and a second after a crash do the same.
- **The gate is in `append_tasks`, not `withdraw_pending`,** to stay off the lines `debt-21` rewrites. It compares `Phase::Link` exactly, because `start_phase()` stays `Some` through phases that serve runtime routes.
- **The walk does not say a restart is required for the sign-in meta.** The request says so; `temper/docs/design/protocol/sign-in.md`, 9.7 and three tests say the meta comes when the forge is made. The walk says what all of them support, and the report carries the difference to the user.
- **No temper story.** The request places the test in `jig/tests/core`, and ops's re-run on a copy of the store is the real proof. The task runs temper's engine domain tests as readers.

## The class, swept

Each path a start's phases take into a domain before people's start has ended. This is from reading the code; the task's stories run the first row, and the existing stories named in its spec the second and third.

| Path | What it sends | Verdict |
|---|---|---|
| `withdraw_stale` at the release, a task's proposal | people's `ChatState` and `Waiting`; the tasks child's `Activate` and `Relay` | the panic and what's behind it: fixed by this plan |
| `withdraw_stale` at the release, a party's proposal | no task row, so no people event | same path; passes today |
| A proposal's application values | connector's `DropProposalSettings` | a settings ask, in the decision, as designed |
| `withdraw_project` | only from `admission/deletion.rs`, a party's request | people answer a request as not ready until their start has ended, so it finds them open; `deletion::advance` returns during a start |
| The release's other items | writes; connector's `SaveSettingsDefaults` and `ForgetReplaced`; authority's `Rules`, the tasks child's `Charters`, the fleet's `Needs`, each asserting no output | no people event |
| `start_link`'s `tasks::Event::Check` | steps the tasks child itself; writes, or `start.waiting_tasks` | no people event |
| `want_accounts` | writes; the rest in `start.held` | no people event |
| The sweep | people's erases | no people event |
| `AdoptRuns` | `start_task_outputs`, `Restored`, the fleet | after `start_complete` |

## Beside this plan

- **hearth:** this changes jig's core and people child. No manifest under `hearth/` names a jig crate, so hearth is untouched.
- **Stored formats, protocols, public interfaces:** none change.

## Overlaps

- `jig/docs/design/domain/engine.md` and `engine.status.md` are also changed by `ui-chat-draft`, `note-parts` and `ui-settings`. Not avoidable: the design has to say this. This plan adds one paragraph to 17.7 and one status entry.
- `jig/crates/jig-domain-people/src/tests/start.rs` gains lines in `ui-chat-draft`. The task expects not to touch it.
- `jig/crates/jig-domain-core/src/routing/settings/dispatch.rs` (`debt-21`) is avoided, as above.
