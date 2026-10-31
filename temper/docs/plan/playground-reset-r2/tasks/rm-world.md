+++
id = "rm-world"
part = "forge-remove"
kind = "feature"
needs = ["rm-project"]
files = [
  "temper/tests/engine/domain/src/removal.rs",
  "temper/tests/engine/domain/tests/suite/removal.rs",
  "temper/tests/engine/domain/tests/suite/fuzzy/removal.rs",
]
effort = "high"
minutes = 75
+++

## Goal

The engine's world (`temper-engine-domain-world`) proves a forge's
removal ends whole however a crash cuts it: one story, a connected
forge with adopted repositories and live changes removed, is run once
to its end, then again once per durable commit with the store crashed
right after that commit, the root rebuilt from the store and started,
and run to its end; every run ends the same, and nothing of the forge
is read afterwards but what draft section 4 keeps.

Acceptance:

- `removal::a_removed_forge_ends_whole` (focused): the story uncut:
  `RemoveForge` answered `Removing`; each change task ended as a
  cancel ends it; `Request::RemoveGroup` and `Request::EraseSecrets`
  released once each, after their commit, and answered by the test as
  the service would (`forge::Event::GroupRemoved`, `SecretsErased`);
  then no `Forge` record, no `KeptAct`, no `Writer` or `Flight` row of
  the forge in `d.store.rows`; the repository rows kept (hidden: the
  forges document and every repository list omit them; the chat's
  task still reads the repository's name by tag).
- `fuzzy::removal::a_removal_crashed_after_each_commit_ends_as_uncut`
  (fuzzy profile): for every commit `k` of the uncut run, the cut run
  ends with the same store rows of the forge connector and the same
  task phases; a kept act answered before the cut is not sent again
  after it; one answered after it is sent again under the same key and
  jig answers it once (the core's retained act answer).
- `removal::a_restart_mid_removal_resends_the_kept_acts_after_core_opens`:
  the requests and the core acts after a restart come after the
  restart's core open, under the rows' keys.
- `removal::an_ungranted_cancel_keeps_its_row_and_notes_it`: a
  configuration whose `engine.acts` lacks `cancel` keeps the `Cancel`
  rows across a restart, with the note; granting it and restarting
  ends the removal.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 2.
- `temper/docs/design/draft/playground-reset.md`, 4 and 8 ("The
  engine's world").
- `skein/docs/foundation/testing-strategy.md`, section 1 and its
  sections on fuzzy tests and seeds (one thread, one loop, a seed
  replays).
- `temper/docs/design/domain/root.md`, 9.1 (restart).
- The world: `temper/tests/engine/domain/src/direct.rs` (`Driver`:
  `send`, `advance`, `pump`, `settle`) and `src/store.rs` (`Store`:
  `apply`, `crash`, `fork`, `pending`, `rows`);
  `tests/suite/demo.rs` (its `restart`, `kept_acts`, `acted` helpers
  and `demo_end_committed_before_its_answer_is_resent_with_the_same_key_and_closed_once`:
  the pattern this sweep generalizes).

Assumed: rm-domain's `forge::Ask::RemoveForge`, `RemoveAnswer`,
`ApplicationAnswer::RemoveForge`, `Request::{RemoveGroup,
EraseSecrets}`, `forge::Event::{GroupRemoved, SecretsErased}`, and
forge-families' `Stored::{Forge, KeptAct}`; fr-domain's way to have a
forge record in a test store (seed the `Forge` row, or `Connect`; use
whichever fr-domain's own world tests use).

## Owns and may change

Owns the files in the header: a helper module `src/removal.rs` of the
world crate (the story's setup, the service's scripted answers, the
crash-and-restart step, the comparison of rows), its suite file and its
fuzzy file. May change the world's `src/lib.rs` and
`tests/suite/main.rs` (one `mod` line each, in order), and the world's
other `src/*.rs` only to make a helper public that the story needs.
`notes/scope.md` applies. No production code: a fault the sweep finds
in the forge connector or root is fixed in `remove.rs`/`kept.rs` only
if it is a one-line slip; otherwise stop and report it as the first
line, `Question:`, with the commit number and the rows that differ.

## Steps

1. `src/removal.rs`:

   ```rust
   pub struct Removal { pub driver: Driver, pub forge: u16, pub changes: Box<[u64]>, pub chat: u64 }
   pub fn story(seed: u64) -> Removal;            // forge connected, two repositories, two changes, a chat naming one
   pub fn remove(r: &mut Removal);                // sends RemoveForge as the deployment owner
   pub fn serve(r: &mut Removal);                 // answers RemoveGroup/EraseSecrets requests seen since last call
   pub fn run_to_end(r: &mut Removal);            // pump, serve, fire timers until quiet
   pub fn crash_after(r: &mut Removal, commit: u64, seed: u64); // apply through `commit`, crash, rebuild, restart
   pub fn forge_rows(r: &Removal) -> Box<[(engine::Key, engine::Record)]>; // the connector's rows naming the forge
   ```

   Rebuild the root from the store as `demo.rs`'s `restart` does
   (`Driver::from_store_at` or its kin). Count commits from the store's
   applied commit numbers.
2. The four tests. The fuzzy test runs every `k` of one uncut run (its
   seed from the world's seed source, printed on failure); keep its
   wall time within the fuzzy profile's per-test budget: measure it and
   say the figure in the report.
3. Then, one at a time:
   - `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/^removal::/)'`;
   - `heavy cargo nextest run -p temper-engine-domain-world --test suite --profile fuzzy -E 'test(/removal/)'`;
   - `heavy cargo nextest run -p temper-engine-domain-world`;
   - `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`;
   - `python3 -I -B scripts/old_forms.py` from `temper/`.
4. Commit and push.

## Report

The standard report, plus: the number of commits the uncut story
makes; the fuzzy test's wall time; every difference the sweep found
before it passed and where it was fixed.
