+++
id = "cancel-release"
part = "tasks-operation"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/**", "jig/crates/jig-domain-core/src/vocabulary.rs", "jig/crates/jig-domain-core/src/routing/party/controls.rs", "jig/crates/jig-domain-core/src/routing/party/decisions.rs", "jig/crates/jig-domain-core/src/routing/run/calls.rs", "jig/crates/jig-domain-core/src/routing/run/controls.rs", "jig/crates/jig-domain-core/src/routing/effect/procedure.rs"]
effort = "high"
minutes = 110
+++

# jig: a cancel and a release of a task each have one type

## Goal

The first of three steps of `jig/docs/development/debt/D012-tasks-operation-vocabulary.md`. `Cancel` and `Release` become public types
(`temper/docs/plan/debt-18/notes/contracts.md`, 2). `Operation` wraps them, `Event::Control` splits into `Event::Cancel` and
`Event::Release`, `Control` and `ProcedureCancel` go, and `NamedAction::Control` carries a `Cancel`. No rule changes, and no entrance
changes what it checks, in what order, or what it answers.

Acceptance:

- `rg -w Control jig/crates/jig-domain-tasks/src` and `rg -n 'tasks::Control|ProcedureCancel' jig temper -g '*.rs'` find nothing.
- Every test passes, its assertions unchanged. A test changes only its spelling of the event, the operation or the action.
- `heavy cargo check --workspace --all-targets` compiles.
- By hand, the `Release` refusal for `Party::Deployment` in `control::apply` is removed, a test goes red, and it's restored; the same for
  `permitted` moved after `check_cancel`.
- The branch's net lines fall by at least 50 (estimate: -90).

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `temper/docs/plan/debt-18/notes/contracts.md`, 2: **what differs between entrances stays**
- `jig/docs/development/debt/D012-tasks-operation-vocabulary.md`: its `ProcedureInput` is stale, and the contract's list stands.
- `jig/docs/design/domain/tasks.md`, 7
- `jig/docs/design/domain/people.md`, 5.1

## Owns and may change

- **Owns:** the files in `files`. `control::Cancel`, the private input to `check_cancel`, gives way to the public type plus context
  arguments; `check_release`'s positional arguments likewise.
- **Reader files, spelling only:** tests in `jig-domain-tasks` and `jig-domain-core`; `jig/tests/tasks/**`; `jig/tests/core/**`;
  `jig/crates/jig-protocol-tools/src/{decode,tests}.rs`; `jig/testing/jig-test-service/src/tests/engine.rs`; and in temper, as this
  plan's decision and only in the name or type path, `temper/crates/temper-engine-domain/src/route/forge_route.rs`,
  `.../route/forge_route/landing_route.rs`, `.../route/forge_route/tests.rs` and `.../route/tools.rs`.
- **Not readers:** `skein_channel::Control` in `jig/tests/channel`, `jig/tests/engine-workers`, `jig/tests/worker-link`; the rest of the
  contract's list. No stored or wire format is touched: `rg` finds these types in no protocol crate. If one turns up, stop and ask.
- **A public interface:** this step and the next two change `jig_domain_tasks::Event` and `jig_domain_core::NamedAction`, which the
  item's **Paid when** asks for and this plan decided (`plan.md`). The crates are `publish = false`, and the compiler finds every reader.
- **If `NamedAction::Control` turns out to build a release,** stop and ask: this cut found only cancels.
- `Event` and `domain.rs` are also changed by another plan, in one line each, far from these arms. Expect no conflict.

## Steps

1. The types, in `control.rs`; `Operation`'s two variants; `compound.rs`'s and `procedure.rs`'s arms over them; `ProcedureDecision`'s
   `cancels`; `Event`'s two variants and `domain.rs`'s dispatch; `NamedAction::Control { cancel }` and what reads it (`run/calls.rs`, about
   line 633; `run/controls.rs` fills `message`).

2. The readers, by compiler. `heavy cargo check -p jig-domain-tasks --tests`, then outward.

3. Break the two rules by hand, as the acceptance says.

4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-tasks -p
   jig-domain-core -p jig-protocol-tools -p jig-tasks-world -p jig-core-world -p jig-test-service -p temper-engine-domain --all-targets -- -D warnings`;
   `heavy cargo nextest run -p jig-domain-tasks -p jig-domain-core -p jig-protocol-tools -p jig-tasks-world -p jig-core-world -p jig-test-service -p
   temper-engine-domain -p temper-engine-domain-world`; `heavy cargo check --workspace --all-targets`. **This departs from the implementer's
   text:** 39 manifests read this crate, so one workspace check stands for naming them; it isn't a test run. The citations check.
   Commit and push.

## Report

The common note's items, each public item changed, what you broke and what failed.
