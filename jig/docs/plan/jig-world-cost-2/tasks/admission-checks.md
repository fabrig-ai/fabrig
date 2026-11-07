+++
id = "admission-checks"
part = "admission-checks"
kind = "feature"
needs = ["store-sweeps"]
files = [
  "jig/crates/jig-domain-core/src/settings/charters.rs",
  "jig/crates/jig-domain-core/src/runs/state.rs",
  "jig/crates/jig-domain-core/src/tests/runs/run_choice.rs",
]
effort = "medium"
minutes = 60
+++

# A due agent's readiness asks the cheaper checks first

## Goal

`Core::context_ready` makes a full `run_admission` to answer one bit, at every `take_due` and `due_ready`, on tasks whose answer hasn't changed. This is product code, and a real engine pays it too. Reach the same bit by the cheaper checks first. **No stored answer:** `plan.md`, *Decisions*, says why it can't be shown safe.

Acceptance:

1. `context_ready`'s answer for an agent equals `!Wait` of `run_admission(context, self.run_wall, [])` in every state, and the new test holds that across the cases of step 3.
2. `run_admission`'s own answers, `RunWait`s and `Hold`s are unchanged, and its signature stays. `run_choice`'s too.
3. A due agent's check builds no `RunContext` on the way to its answer, and asks `run_choice` only when no hold and no unusable credential has already answered.
4. The focused tests of `jig-domain-core`, `jig-test-domain`, `jig-ops-domain`, `jig-core-world`, `jig-channel-world` and `jig-tasks-world` (read that name in `jig/tests/tasks/Cargo.toml`) pass. `heavy cargo check -p temper-engine-domain --tests` passes: its root reads `take_due`.
5. If step 2's equality can't be shown from the code at some exit, that exit is left out and the report says why. A stale or wrong admission is a bug, and this saving isn't worth one.

## Inputs

- `jig/docs/design/domain/engine.md`, 7.1 and 8. This task changes no design.
- `jig/docs/design/domain/accounts.md`, 4.3, for the joint check.
- `jig/docs/plan/jig-world-cost-2/plan.md`, *Decisions*.
- `jig/docs/plan/jig-world-cost-2/notes/common.md`.
- `skein/docs/foundation/programming-model.md`, 10: `jig-domain-core` is step code.

## Owns and may change

- **Owns:** the spec's `files`. `jig/crates/jig-domain-core/src/runs/run_choice.rs` may gain the option below.
- **Readers, read at the cut:** `context_ready` is private, read by `take_due` and `due_ready`. Those are read by the roots `jig/testing/jig-test-domain/src/lib.rs`, `jig/examples/ops/domain/src/domain.rs` and `temper/crates/temper-engine-domain/src/domain.rs`, whose signatures don't change. `Domain::quiescent` stays as it is: `finish_owed` walks projections and marks, and `due_ready` is free with nothing due, so neither order is plainly cheaper.
- **Options:**
  - **No host at all:** `fleet.probe(kinds, task, version, models, &[])` being `None` means `run_choice`'s every candidate breaks at its first try, and every path past that returns `Err`, so the answer is a wait. Add this exit before `run_choice`, as a `pub(crate)` beside it, if `chosen_policy` and `fleet_needs::model` read as cheaper than what they save. If not, leave it and say so.
  - `translate::tasks_authority` clones the authority for `RunAsk`. Leave it: `RunAsk`'s shape has readers beyond this task.

## Steps

1. `runs/state.rs`, sharing the ask so that the two can't differ:

   ```rust
   /// `check_run` for a task's current numbers and no writes but `writes`,
   /// under `choice`'s accounts, or none if there's no choice.
   fn run_check(
       &self,
       task: (u64, u32, &tasks::Authority, tasks::Numbers), // or a small struct: yours
       choice: Option<&RunChoice>,
       wall: Wall,
       writes: Box<[authority::Write]>,
       findings: &mut Queue<authority::Finding>,
   ) -> authority::RunAnswer;

   /// Whether a due agent's admission for no writes is anything but a wait
   /// (domain/engine.md, 7.1): what `run_admission` answers, asking the
   /// cheapest first. The account and host check comes last.
   pub(crate) fn agent_ready(&self, task: u64) -> bool;
   ```

   `run_admission` keeps its body's order and answers, through `run_check`.
2. `agent_ready`, from `self.tasks.task(task)`'s row (`project`, `authority`, `numbers`, the fields `run_context` copies), in this order. Each exit is equal to today's answer for the reason given:
   1. `run_check` with no choice, then `denied`. With no accounts, `check_run` always finds `Account`, and everything else it finds doesn't depend on the accounts. So: **a hold here is a hold under any choice** (`denied` gives a hold for any finding but `Account`), so the answer is `true`. Today's oversized check of the accounts can only add a hold.
   2. `!self.credentials_ready(task, self.run_now)`: **a wait under any choice**, since it adds only `Account` and step 1 found nothing else, so the answer is `false`. No choice can be oversized and turn this into a hold: `jig/crates/jig-domain-core/src/limits.rs` refuses `authority.accounts` below `accounts.accounts.min(tasks::MODELS)`, and a choice binds one account per provider.
   3. `run_choice(task)`: `Err(_)` means today's ask is the one step 1 made, whose answer was a wait, so `false`. `Ok(choice)` means `run_check` under the choice and today's match, since a duplicated provider's `Refuse` is still `Account`'s wait. Don't assume `true` here.

   `context_ready`'s `Agent` arm calls `agent_ready(context.task)`. `Job`, `Procedure` and `Person` stay as they are.
3. The test, `a_due_agent_s_readiness_is_its_admission_in_every_state`, in `tests/runs/run_choice.rs`, whose fixtures hold most of these states (`tests/runs/credentials.rs` and `tests/settings/charters.rs` show the rest):

   | State | `run_admission` | `agent_ready` |
   |---|---|---|
   | a usable account and a free host | `Allow` | `true` |
   | no host | `Wait(Host)` | `false` |
   | a limited account | `Wait(Account { .. })` | `false` |
   | an unusable workspace credential | `Wait(Credential)` | `false` |
   | budget spent, and no host | `Hold(Budget)` | `true` |
   | past the authority's deadline, and a credential unusable | `Hold(Deadline)` | `true` |

   Each row asserts both columns, so a row that stops reaching its state fails. The last two are why the checks go in this order. Break it once by swapping exits 1 and 2 of step 2: the last row fails. Restore it, and report that.
4. Narrow runs by name, then the implementer's closing steps for acceptance 4. Commit and push.

## Report

The implementer's report, plus each exit as it landed, whether the host exit went in and why, and the rule broken and the failure seen. Claim no saving, since nothing is measured.
