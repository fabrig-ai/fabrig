+++
id = "sv-skein-setup"
part = "fs"
kind = "interface"
needs = []
files = ["skein/crates/skein-forge-setup/**", "skein/tests/forge/setup/**"]
effort = "high"
minutes = 75
+++

## Goal

skein's forge setup host takes forges added and removed after its start (`notes/contracts.md`,
T7): `host::Request::Add { forge, settings }` and `host::Request::Remove { forge }` for its
per-forge settings. Its start-time list stays the initial set. skein only: nothing here names
temper.

Acceptance:
- `skein_forge_setup::host::Request` gains `Add { forge: u16, settings: host::Forge }` and
  `Remove { forge: u16 }`, with doc comments saying what this spec says.
- An admitted `Add` holds the forge as one handed at start: a later `Run` naming it runs as
  for a start-time forge.
- An `Add` the host cannot hold changes nothing and says nothing: while closing (after
  `Close`/`Abort`); a number already held; `forge` differing from `settings.number()`;
  `limits.forges` forges already held; or settings that start's per-forge checks refuse
  (`limits::check`'s loop body: plaintext name, address, request head, the GitHub documents
  and metadata for a GitHub forge, `given::check`/`github::given::check`, `reachable`, the
  capacity bound). `Host::admits` says why, by the same check, so the owner can check first.
- `Remove` of a held forge: each live run of the forge ends at once with its one
  `Event::Stop`, as at its deadline (`cut`: its connection aborted, `Why::Down`), the same
  pass; the one-time states bound to the forge (forge-setup.md, 5.7) are forgotten; then the
  forge is held no more. `Remove` of a number not held is ignored. The number may be added
  again afterwards.
- A `Run` naming a forge not held (never added, or removed) answers at once
  `Event::Stop { run, failure: Failure { forge, step: Stage::Admin, subject: plan's name,
  why: Why::Unexpected } }` instead of today's `unreachable!("owner names a configured
  forge")`, so a removal racing the owner's run never panics.
- Start-time behaviour is unchanged for every existing test.

## Inputs

- `skein/docs/design/forge-setup.md`, sections 5.7, 6.3, 6.5.
- `skein/docs/foundation/programming-model.md`, section 1.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T7.
- `skein/AGENTS.md`.

## Owns and may change

- Owns: `skein/crates/skein-forge-setup/` and its world `skein/tests/forge/setup/` (package
  `skein-forge-setup-world`).
- May change, call sites only: any file the compiler names because of a derive this task
  drops (below). temper's readers of `host::Request`
  (`temper/crates/temper-service/src/setup.rs`, `setup/lifecycle.rs`) use `if let` and
  construct it; none compares it.
- Not `skein/docs/design/`: the report proposes forge-setup.md 6.3's new text (forges added
  and removed at runtime; the unknown-forge run's stop), and the lead writes it.
- Pre-approved options:
  - `host::Request` derives `Clone, PartialEq, Eq, Debug` today; `host::Forge` and
    `host::Transport` derive only `Debug` (`skein_tls::Config` is `Clone` but not
    `PartialEq`). Default: derive `Clone` on `Forge` and `Transport`, and drop `PartialEq,
    Eq` from `Request` (keeping `Clone, Debug`); fix the comparisons the compiler names
    (tests only, expected). If `Clone` cannot be derived on `Transport`, drop `Clone` from
    `Request` too and say so.
  - The per-forge check: factor the loop body of `host/limits.rs::check` into
    `pub(super) fn forge(l: &Limits, user_agent: &[u8], forge: &Forge) -> Result<(),
    Unusable>`, used by start and `Add`, so they cannot drift. `Unusable` keeps its
    variants: `Forges` for a full set, `Duplicate` for a held number; add
    `Unusable::Number { forge: u16 }` for `forge` differing from `settings.number()` and
    route it in temper's `temper/crates/temper-service/src/setup/settings.rs::problems`
    (it matches `Unusable` exhaustively) beside `Duplicate` (compiler-forced).
  - States: `host/states.rs` has `clear` for all; add `pub(super) fn forget(&mut self,
    forge: u16)` dropping the forge's bindings (same order kept for the rest).

## Steps

1. `src/host/boundary.rs`: the two variants:

   ```rust
   /// A forge held from now on, as one handed at start is (6.3). Ignored, with nothing
   /// said, when the host cannot hold it: see `Host::admits`.
   Add { forge: u16, settings: Forge },
   /// A forge held no more: its live runs stop `Down` at once, its one-time states are
   /// forgotten. Ignored for a forge not held.
   Remove { forge: u16 },
   ```

2. `src/host.rs`:
   - `pub fn admits(&self, limits: &Limits, forge: &Forge) -> Result<(), Unusable>`:
     `Forges` when `limits.forges` are held, `Duplicate` when the number is held, else the
     per-forge check against the held `user_agent`.
   - `down`: `Add` pushes `settings` into `self.settings.forges` when admitted (and `forge
     == settings.number()`); `Remove` stops each live run of the forge as the `Abort` arm
     does for every run (take the run out, `cut` it unless it already holds its
     `Done`/`Stop`, push that event), forgets its states, and drops it from
     `settings.forges` (rebuild the `List` without it, order kept); `Run` naming a forge not
     held pushes the `Unexpected` stop above. Each ends with `drive` as today.
   - `configured` returns `Option<&Forge>`; its two callers handle `None` (`Run`: the stop
     above; `drive_run`: a run whose forge is gone cannot be live, since `Remove` ends its
     runs: `expect` with that reason).
3. Unit tests in `src/tests/host.rs` (its `World`, `settings`, `forge`, `limits`), names:
   - `a_forge_added_after_start_runs_its_plans`
   - `an_add_past_forges_or_of_a_held_number_changes_nothing`
   - `admits_names_the_same_refusal_as_start_for_a_bad_address`
   - `an_add_of_a_github_forge_checks_the_github_limits` (GitHub fixtures from
     `src/tests/github_fixtures.rs`)
   - `a_removal_stops_a_live_run_down_and_aborts_its_connection`
   - `a_removal_forgets_the_forges_one_time_states_and_keeps_anothers` (a GitHub forge's
     `Install` state, as `tests/github_install.rs` draws one; an `Accept` naming it after the
     removal and re-add stops `Unknown`)
   - `a_run_for_a_forge_not_held_stops_unexpected`
   - `a_removed_forge_added_again_runs_afresh`
4. The setup world (`skein/tests/forge/setup`, `tests/suite/host.rs`): one story,
   `a_forge_added_reconciles_and_its_removal_mid_run_stops_it_down`, on the world's fake
   Forgejo (`src/world.rs`; add a constructor beside `World::managed` that builds the host
   with no forges and `Add`s the world's forge): reconcile to `Done`,
   run a second reconcile, `Remove` while it waits on the forge, see its `Stop` with
   `Why::Down` and every connection closed.
5. Narrow runs, one at a time:
   - `heavy cargo nextest run -p skein-forge-setup --lib -E 'test(/tests::host::(a_forge_added|an_add_|admits_|a_removal|a_run_for_a_forge_not_held|a_removed_forge)/)'`
   - `heavy cargo nextest run -p skein-forge-setup-world --test suite -E 'test(=host::a_forge_added_reconciles_and_its_removal_mid_run_stops_it_down)'`
   Check the counts (8 and 1). Then whole: `heavy cargo nextest run -p skein-forge-setup`,
   `heavy cargo nextest run -p skein-forge-setup-world`, and
   `heavy cargo check -p temper-service --tests`.
6. `cargo fmt`; `heavy cargo clippy -p skein-forge-setup -p skein-forge-setup-world
   --all-targets`; `python3 -I -B scripts/old_forms.py` from `skein/` (no count rises).
7. Commit (why: setup's forges added and removed at runtime, for temper's forges in its
   store) and push your branch.

## Report

- The derive option taken and the comparisons it changed.
- The proposed text for forge-setup.md 6.3 (a short paragraph).
