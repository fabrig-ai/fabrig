+++
id = "pg-welcome-states"
part = "playground"
kind = "feature"
needs = ["pg-playground-pages"]
files = [
  "temper/crates/temper-client-domain/src/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/domain.rs",
  "temper/crates/temper-client-domain/src/tests/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/tests/welcome_states.rs",
  "temper/crates/temper-client-view/src/pages/welcome.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/tests/client/tests/suite/welcome.rs",
]
effort = "high"
minutes = 50
+++

## Goal

The welcome reads the store as empty, unfinished or whole (draft
`temper/docs/design/draft/playground-reset.md`, 6.3) from the `forges`
rows (`finished`, `playground`: contracts, 5) and the accounts: empty
offers "Try the playground" and "Set up temper"; unfinished opens at
the step left undone; whole moves on to the chats page. The playground
is offered while no real forge is kept, and never once one is; a real
account alone does not end it.

Acceptance:

- `pages::welcome::setup(root) -> Option<Setup>`, `Setup::{Empty,
  Unfinished(Step), Whole}`, none while its facts are missing (as
  `decision` waits today: `me`, the playground's read, the accounts and,
  for a person whose deployment role is owner, the `forges` rows). For
  an owner: *whole* when a real forge row (not `playground`) is
  `finished` and a real model is usable (`model_usable`); *empty* when
  no real forge row is kept and no real model is usable; otherwise
  *unfinished*, at `Step::Forge { forge: None }` while no real forge is
  finished, else at `Step::Model { way: None }`. For any other person:
  whole with a usable model, else empty. A refused (`Ended`) forges
  watch counts as no forge, as today;
- `pages::welcome::playground_offered(root) -> bool`: the read answered
  `NotStarted` and no real forge row is kept;
- `decision(root)` is `Some(true)` while the playground runs (or
  resets) or the setup is whole, `Some(false)` otherwise, none while
  `setup` is none;
- opening `/welcome` (the Welcome step), once its facts are in: whole
  or running replaces the address with Chats (as today); unfinished
  replaces it with `/welcome?at=forge` or `?at=model`; empty stays; once
  per opening, as today; a later step never moves;
- the welcome's view draws the playground card on the Welcome step, and
  "Try the playground instead" on the Forge and Model steps, exactly
  when `playground_offered`; the view's own `real_forge_connected` goes;
- the tests below pass, and every existing test of the packages changed,
  expectations updated where they pinned the halves' welcome (a
  connected real forge without a repository, or the demo's forge
  counted as the person's), each named in the report.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 5).
- `temper/docs/design/draft/playground-reset.md`, 6.3 and 6.4 ("An
  account added before stays").
- `temper/docs/design/client/README.md`, 3.5 (`Welcome`) and 5.6 (its
  words; none change).
- `temper/docs/plan/playground-reset-r4/tasks/forge-asks.md` (`ForgeRow`'s
  `finished` and `playground`: until the engine lists rows from records
  they mean "connected with a repository adopted" and "the demo's
  forge"), `tasks/pg-client-read.md` (`Domain::demo()`, the read's
  states).

## Owns and may change

Owned: the `files` above (`tests/welcome_states.rs` is new).

Neighbour files, edits allowed:

- `temper-client-domain/src/forge/forges.rs` and `forge/family.rs`: the
  decoded `Row` gains `finished: bool` and `playground: bool` from the
  wire row (they are ignored there since `forge-asks`), every `Row`
  literal in the temper tree gains them (false, or what the test's row
  means);
- `temper-client-domain/src/forge/listed.rs`: `personal` excludes a row
  by `row.playground` beside the demo's forge test;
- `temper-client-domain/src/tests.rs`: one `mod` line;
- other tests of `temper-client-domain`, `temper-client-view` and
  `temper/tests/client/` whose expectation this changes (the welcome's
  move on a connected forge without a repository; the playground card
  beside a connected forge): updated, each named in the report.

Pre-approved:

- `model_usable` stays as it is (the playground's provider never counts:
  6.1); if a test shows the scripted account counted, fix the call, not
  the rule;
- if the welcome's view needs `setup` to pick its Ready heading, read
  it; otherwise leave Ready as it is.

## Steps

1. **Domain** (`pages/welcome.rs`):

   ```rust
   /// The store as the welcome reads it (draft playground-reset.md, 6.3).
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum Setup { Empty, Unfinished(crate::Step), Whole }
   #[must_use] pub fn setup(root: &Domain) -> Option<Setup>;
   #[must_use] pub fn playground_offered(root: &Domain) -> bool;
   #[must_use] pub fn decision(root: &Domain) -> Option<bool>;   // kept, re-defined
   ```

   `domain.rs`'s `welcome(root, out)`: on the Welcome step, decided
   once, pushes the `Show { replace: true }` of the acceptance.
2. **View** (`pages/welcome.rs`): `playground` is
   `temper_client_domain::pages::welcome::playground_offered(root)`;
   `real_forge_connected` goes.
3. **Tests** (client domain and view unit tests; the client world):
   - `temper-client-domain/src/tests/welcome_states.rs`, with rows built
     as `tests/pages/welcome.rs` builds them (copy its helpers locally):
     `no_forge_and_no_model_is_empty`;
     `a_real_account_alone_is_unfinished_at_forge_and_keeps_the_playground_offered`;
     `a_forge_not_finished_is_unfinished_at_forge_and_withdraws_the_playground`;
     `a_finished_forge_without_a_model_is_unfinished_at_model`;
     `a_finished_forge_and_a_model_is_whole`;
     `the_playgrounds_forge_is_no_real_forge` (a `playground` row,
     connected and finished: empty);
     `a_running_playground_decides_over`;
     `the_welcome_opens_at_the_step_left_undone_once` (Welcome step,
     unfinished at Model: one `Show` of `/welcome?at=model` replacing;
     reopening `/welcome?at=forge` moves nothing).
   - `tests/pages/welcome.rs`: its cases on the new rule (the demo's
     account and forge keep the welcome; a refused watch counts as no
     forge; once per opening), updated where the halves were pinned.
   - `temper-client-view/src/tests/welcome.rs`: the playground card and
     "Try the playground instead" drawn iff offered (a kept GitHub row
     not connected withdraws both).
   - `temper/tests/client/tests/suite/welcome.rs`: its cases on the new
     rule.
   - Narrow runs per file.
4. Checks, once: `cargo fmt`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view -p temper-client-world
   --all-targets -- -D warnings`; focused tests of the three; `heavy
   cargo check --tests` for `temper-client-tools`,
   `temper-client-shell-terminal`, `temper-client-shell-web`,
   `temper-client-gallery`, `temper-one-process-world`, one at a time.
   This changes what `decision` answers, so the focused tests of every
   package reading it run (the three above). Commit, push.

## Report

fab's sections; `## Interfaces`: `Setup`, `setup`, `playground_offered`,
`decision`, `Row.finished`/`Row.playground`; every test whose
expectation changed, by file and name.
