+++
id = "fe-world-reset"
part = "pg"
kind = "feature"
needs = ["fe-demo-gone", "pd-core-release"]
files = ["temper/tests/engine/domain/**"]
effort = "high"
minutes = 100
+++

## Goal

The engine's world proves the playground on records and its reset whole (draft
`temper/docs/design/draft/playground-reset.md`, 6.1, 6.2, section 8 "the engine's world"
and section 9's first default; `notes/contracts.md` T6, J1, J3 to J5, J8): on the real
root, connector, core and typed store, a start makes the playground's forge, project,
account and adoption; a reset deletes the project (jig's deletion, whole), removes the fake
forge and the account and erases the row; a reset cut by a crash after each step and each
kept act resumes to the same end; afterwards nothing of the project or the forge is read
but what sections 4 and 5 keep. The demo's suite (`tests/suite/demo.rs`) is replaced.

The lead adds to `needs` the jig tasks that make the deletion real in the core (contracts
J3 to J5: the decision, its stages resumed at start, the connector told) and the
account's uses (J8, `pd-accounts-uses`): the stories read their effects.

Acceptance (a new `tests/suite/playground.rs`, registered in `tests/suite/main.rs`;
`tests/suite/demo.rs` deleted and its fixtures moved where still used):
- `a_start_makes_the_fake_forge_project_account_and_adoption`: with `Machine.playground`
  set and the scripted key carried, `StartPlayground` answers `Running` once the fake
  forge connects (the story connects it as serve would); the row names forge, project and
  account; the project is named `Playground` with policy 1; the account serves that project
  alone (a task of another project cannot select it); `playground/widget` is adopted into
  the project; a second start answers `Running` and makes nothing.
- `a_start_beside_a_real_forge_or_while_resetting_is_not_offered`.
- `the_reset_deletes_the_project_removes_the_forge_and_the_account_then_the_row`: a chat in
  the playground's project is live; `EndPlayground` answers `Reset` only once the row is
  erased; the chat is cancelled by the project's deletion; the store then holds no row
  naming the project but J4's keep list (retained act answers, settled spend totals) and no
  row naming the forge but its repositories' while a task names them (none here: the
  project's tasks are erased); the account is gone.
- `a_reset_cut_after_each_step_and_act_resumes_to_the_same_end`: for each cut point (after
  the end's decision; after each step's act is answered; after each of the forge's kept
  acts), crash and cold start; the reset ends with the uncut run's rows.
- `a_start_that_finds_the_playground_running_resets_it`: a running playground, crash, cold
  start: the row reads `Resetting(Project)` and the reset ends whole.
- Each story at two seeds; the whole suite passes.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T4, T6, J1,
  J3 to J5, J8; `notes/fe-surface.md` sections 4 and 5.
- `temper/docs/design/draft/playground-reset.md`, 6.1, 6.2, sections 8 and 9.
- `temper/tests/engine/domain/src/cuts.rs` (`fe-world-remove`'s helpers) and the demo
  suite being replaced.

## Owns and may change

- Owns: `temper/tests/engine/domain/` (`temper-engine-domain-world`).
- May change: nothing else; a fault in the connector, the root or jig is a report item with
  its failing story.
- Pre-approved options:
  - The world's configuration: default, `walking::config` gains a `playground` variant (or
    a helper `walking::playground_config(seed)`) setting `machine.playground` (a Fake record
    with a setup entry), `DemoConfig` (`provider` the scripted provider the world's
    accounts offer, `project_name: "Playground"`, `project_policy: 1`, the adoption of
    `playground/widget`), and `engine.acts`' grant of `make_project`, `add_key`, `adopt`,
    `cancel`, `remove_project`, `remove_account`, as `temper init`'s document does (T4).
  - The scripted key: carried as the demo suite carries it today.

## Steps

1. The configuration helper; `tests/suite/playground.rs`; delete `tests/suite/demo.rs`.
2. Run, one at a time: `heavy cargo nextest run -p temper-engine-domain-world --test suite
   -E 'test(/playground::/)'` (check the count), then the whole `--test suite`; `cargo fmt
   --all`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`;
   `python3 -I -B scripts/old_forms.py` from `temper/`.
3. Commit, push your branch.

## Report

- The cut points walked; any row naming the project or forge afterwards beyond J4's and
  section 4's keep lists.

## Answers

### 2026-11-01: Should fab arrange an upstream fix enforcing project isolation for the playground account (recommended), or revise acceptance to rely on disjoint project model policies / different financial payers? The starting person's task in another project selects the playground account at both seeds. Work is committed and pushed. (playground-reset-r6/fe-world-reset-1, foreman playground-reset-r6/fe-world-reset/foreman-1)

Use disjoint project model policies as design 6.1 requires; retain the acceptance requirement.
`temper/docs/design/draft/playground-reset.md`, 6.1: “the playground's project allows only the scripted model, and no other project allows it”.
J8 requires `Some([people::Use::Party(starting person), people::Use::Project(playground's project)])` and explains that a person's chat has the person as payer.
The same-policy regression does not model the cited design's restriction.
The next run should configure the other project's policy accordingly and prove its task cannot select the playground account, including for the same starting person.
Keep changes within `temper/tests/engine/domain/`, run the required checks, then commit and push.
