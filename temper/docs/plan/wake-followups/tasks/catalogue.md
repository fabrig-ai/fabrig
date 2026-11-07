+++
id = "catalogue"
part = "catalogue"
kind = "feature"
needs = []
files = ["temper/tests/engine/domain/src/store.rs", "temper/tests/engine/domain/tests/suite/forge_routes/per_wake.rs"]
effort = "medium"
minutes = 60
+++

# wake-followups: the engine's domain world reopens its store with the catalogue it was last given

## Goal

`forge_routes::per_wake::the_reference_goal_reads_its_cut_then_its_questions_part`
(`temper/tests/engine/domain/tests/suite/forge_routes/per_wake.rs`) passes its three rows,
then fails the world's restart oracle with "root startup or store failure", so it is
`#[ignore]`d. The cause is the world's store, not the product: `Store::catalogue`
(`temper/tests/engine/domain/src/store.rs`) gives the live jig store component the root's
new model names (`jig_protocol_store::Store::catalogue`), but `Store::crash` reopens a
component over `self.models`, the world's scripted `walking::models()`, so after a crash
template 4's model indices no longer resolve. The world keeps the catalogue it was last
given and reopens with it, as the service's store keeps one component across its life
(`temper/crates/temper-service/src/store.rs`, `down`'s `Catalogue` arm); the ignore is
lifted.

Acceptance:

- `Store` keeps the names of the last `Store::catalogue` call; `Store::crash` gives the
  reopened component those names (by `catalogue` after `protocol::Store::over`, or by an
  equivalent, before `open`), and `Store::fork`'s fresh store takes them before it replays
  `durable`. A store that was never given a catalogue reopens exactly as today.
- The `#[ignore = "forge world's Store::crash reopens ..."]` line on
  `the_reference_goal_reads_its_cut_then_its_questions_part` is gone, with no other change
  to `per_wake.rs`.
- `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
  'test(/forge_routes::per_wake/)'` passes, selecting one test.
- It bites: with the reopen's catalogue taken out again, the story fails at the restart
  oracle with "root startup or store failure"; restore it. The report quotes the failure.
- The world's focused suite passes: `heavy cargo nextest run -p temper-engine-domain-world`.
  This changes what a crash reopens with for every story of the world that sends a
  catalogue (`escalation.rs`, `roles.rs`, `forge_routes.rs`, `walking.rs`, `direct.rs`
  route `StoreRequest::Catalogue` to `Store::catalogue`), so the whole package runs, not
  one binary.
- The report gives the story's CPU time beside
  `reference_template_four_starts_on_the_configured_forge`'s
  (`forge_routes/goal_branch.rs`), both from one run.

## Inputs

- `temper/docs/plan/wake-followups/notes/common.md`.
- `temper/docs/design/protocol/configuration.md`, 13.9: template 4's bullet and "Charter 5
  in parts", what the story holds.
- jig's `jig/docs/design/domain/engine.md`, 17.11: today's models reach the store before a
  commit that holds them, which `Store::catalogue` carries.

## Owns and may change

- Owns the two files in the header.
- Neighbours: none. The five fixtures that route `StoreRequest::Catalogue` already call
  `Store::catalogue` and stay as they are.
- Pre-approved: if `take_for_drop` or another constructor in `store.rs` needs the kept
  names to stay coherent, carry them there too. If, with the catalogue kept, the story
  fails elsewhere, the failure is outside this task's cause: put the ignore back with the
  new failure as its reason, report `Done:` with the failure quoted under `## Left`, and
  say which of the story's rows passed. If another story of the world fails because a
  crash now reopens with its catalogue, read it: if it asserted the scripted catalogue
  after a crash it is wrong and its assertion follows the kept catalogue (a reader file,
  edited only there, named in the report); otherwise stop with a `Question:`.

## Steps

1. Lift the ignore and run the story: see it fail at the restart oracle.
2. Keep the catalogue in `Store` and reopen and fork with it; run the story: it passes.
3. Bite it as above.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-world --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-domain-world`.
5. Commit (why: a crash in the engine's domain world reopened its store with the scripted
   catalogue, so a story whose root changed its models failed the restart oracle); push
   your branch.

## Report

As the implementer's text says, and also: the story's CPU time beside its neighbour's,
and any story whose outcome the kept catalogue changed.
