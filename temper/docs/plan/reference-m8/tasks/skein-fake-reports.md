+++
id = "skein-fake-reports"
part = "skein-reads"
kind = "feature"
needs = ["skein-reads"]
files = ["skein/crates/skein-fake-forge-domain/**", "skein/crates/skein-fake-forge-protocol/**", "skein/crates/skein-fake-forge-github/**", "skein/crates/skein-fake-peers/src/forge/**", "skein/tests/fake-forge/**", "skein/tests/forge/protocol/**"]
effort = "high"
minutes = 180
+++

# skein: the fake forge's run reports

## Goal

The fake forge runs a dispatched workflow's tests by id: a workflow names the tests it fails
(`fails`, or `Cue::Tests` reading them from a file at the run's commit), those that fail in a
whole run and pass when selected (`flaky`), the input that selects tests (`select`), and the
artifact it uploads at completion (`report`: a JUnit file or one id per line, stored or
deflated); and it answers `Read::Artifact` and `Read::Commits`. temper's tier stories run on it.

Acceptance:

- `skein/docs/design/fake-forge.md`, 2.3 holds whole: `fails`, `Cue::Tests`, `flaky`, `select`,
  one artifact per completed run in the workflow's format, served on Forgejo's routes
  (`skein/docs/design/forge.md`, 5.2) as the zip the real Forgejo serves, and `Read::Commits`
  over the fake's history, oldest first, paged.
- The new requests that `skein-reads` answered `Refused` in the fake are answered.
- Every existing `Workflow` literal gains the new fields with values that keep its meaning
  (no fails, no flakes, no report, no select): in skein
  (`skein-fake-forge-domain/src/tests/actions.rs`,
  `skein/tests/forge/protocol/tests/suite/protocol.rs`,
  `skein/tests/forge/client/tests/suite/recovery.rs`) and in temper
  (`temper/tests/engine/forge/src/world.rs`, about line 261;
  `temper/tests/engine/domain/src/forge_routes.rs`, about line 686).
- temper's copy of the translation answers them for real
  (`temper/tests/engine/forge/src/translate.rs`: the arms `skein-reads` left panicking), and
  `temper/crates/temper/tests/support/case.rs` (about line 214) lists `Cue::Tests` in its
  exhaustive match.
- The tests in Steps pass, with their counts.

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`; `notes/contracts.md`, 5.
- `skein/docs/design/fake-forge.md`, 2.3; `skein/docs/design/forge.md`, 3.2 and 5.2.
- The old code: `temper/docs/plan/reference-m8/notes/old/skein-reads.diff`, its fake half
  (`skein-fake-forge-domain/src/reports.rs`, the `Workflow` and `Cue` fields, `Kept.artifact`,
  the fake's `Artifacts`/`ArtifactById` routes and zip encoding, flaky ids).
- What moved since: `actions::Kept` gained `pull` (`skein-fake-forge-domain/src/actions.rs`,
  literals in `actions.rs` and `dispatch.rs`); `Cue::PullFile` is new, with `validate_cue`
  (`scenario.rs`) and match sites in `actions.rs`, `ci.rs`, `dispatch.rs` (`Cue::Tests` joins
  each); the fake domain's `limits.rs` lost its heap sums (drop those hunks);
  `skein/tests/fake-forge/tests/suite/memory.rs` measures under `FILLED_FORGE_PEAK_CEILING`
  (`forge.ceiling`, not `forge.bound`): re-express the old filled-forge reads under it.

## Owns and may change

- Owns the crates and test packages in the header.
- `skein/tests/forge/client/tests/suite/*.rs` and
  `skein-fake-forge-domain/src/tests/actions.rs`: `Workflow` literals' new fields only.
- temper: the two `Workflow` literals, `case.rs`'s arm, and `translate.rs`'s new arms, as
  above; nothing else of temper.

Pre-approved options:

- If the filled forge's peak passes `FILLED_FORGE_PEAK_CEILING` with the reports kept, raise the
  ceiling to the measured peak with a margin as the file's other ceilings have, and give both
  numbers in the report.
- If temper's `translate.rs` has no counterpart for a fake answer the old diff serves, map it as
  skein's own translation does, and say so.

## Steps

1. Apply the fake half of the diff on a scratch commit; adapt to today's shapes (Inputs).
2. Tests, each alone first: `heavy cargo nextest run -p skein-fake-forge-domain --lib`;
   `-p skein-fake-forge-protocol --lib`; `-p skein-forge-protocol-world --test suite`;
   `-p skein-forge-client-world --test suite`; `-p skein-fake-forge-tests --test suite`.
3. temper: `heavy cargo check -p temper-engine-forge-world -p temper-engine-domain-world -p
   temper --tests`; `heavy cargo nextest run -p temper-engine-forge-world --test suite`.
4. `cargo fmt`; clippy over every package you changed; commit; push your branch.

## Report

As the implementer's text says, and also: what of the old fake model you changed; the memory
ceilings and peaks; the route count.
