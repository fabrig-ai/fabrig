+++
id = "tiers-reports"
part = "tiers-forge"
kind = "feature"
needs = ["tiers-runs", "skein-reads", "skein-fake-reports"]
files = ["temper/crates/temper-engine-domain-forge/src/tiers.rs", "temper/crates/temper-engine-domain-forge/src/tiers/**", "temper/crates/temper-engine-domain-forge/src/tests/tiers.rs", "temper/crates/temper-engine-domain-forge/src/tests/tiers/**", "temper/tests/engine/forge/**"]
effort = "high"
minutes = 300
+++

# A red tier run: its report read, its failures rerun, its blame bisected

## Goal

A failed tier run's failing tests are read from its artifact; the failed tests run once more,
alone, at the same tip (flakes); new failures over several landings are bisected by probes,
the failing tests alone at each landing's commit, oldest first; past `failing_tests` the
failures are many and blamed by landing alone. All driven by the pure policy's `Next`.

Acceptance (`temper/docs/design/domain/after-landing.md`, sections 2, 4 and 5.2):

- **The report:** a whole run, a rerun or a probe that failed reads its artifact by the tier's
  `artifact` name (`skein_forge::api::Read::Artifact { run, name, max_bytes }`, `max_bytes` the
  most a report of `failing_tests` ids of `test_id_bytes` can take in the tier's format), then
  `temper_engine_domain_forge_tiers::report` and `failed`; an absent or unreadable report, or one
  naming no test, is a fault (retried). A passed run reads nothing.
- **A rerun** (`Part::Rerun`): dispatched again on the run's branch, at the same tip, with the
  input `temper_tests`, the ids one per line; a whole run's reruns and probes never make
  receipts.
- **A probe** (`Part::Probe`): a branch of its own at the landing's commit,
  `temper/tier/NAME.N.K` (K the probe's number in the run, kind 9), dispatched with
  `temper_tests` (or with none for `Selection::All`), deleted when the probe ends, by the same
  release as the run's branch; its operation numbers in `EffectPurpose::Tier` (contracts.md, 4).
- **Restart:** a rerun or probe in flight survives a cold restart (its state on `TierRow`).
- The forge world's stories (`temper/tests/engine/forge/tests/suite/tiers.rs`, new, with the
  fake forge's `fails`, `Cue::Tests`, `flaky`, `report`, `select`): a red run bisected over
  three landings, one changing nothing the tier reads (skipped), two tests failing from the
  second's commit, which owns them, the others cleared; inherited failures (the next run fails
  the same two and clears the landing after); a flake (one test fails, passes alone, the range
  cleared, the flake counted); many failures (past `failing_tests`) blamed by landing, probes
  running the whole tier; a fault (a run lost when its runner restarts) retried, then green; a
  report absent from a failed run, a fault; under a repository whose tier follows main, a push to
  another branch starts no run.
- Packages pass: `heavy cargo nextest run -p temper-engine-domain-forge --lib`;
  `-p temper-engine-forge-world --test suite`.

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`; `notes/contracts.md`, 1, 4 and 5.
- `temper/docs/design/domain/after-landing.md`, sections 2 (dispatch, branches, inputs), 4, 5.2
  and 12 (its stories); `skein/docs/design/fake-forge.md`, 2.3 (the fake's reports).
- The old code: `temper/docs/plan/reference-m8/notes/old/temper-t9-b3b9cf52a.diff` (failed-test
  reports, selected reruns, probe orchestration), `temper-t9-816c390da.diff` (red orchestration,
  durable restart), `temper-t9-87afa0940.diff` (selected flaky tests clear a range),
  `temper-t9-d1b0cdff8.diff` (skein's read variants at temper's boundaries). t9 also sends reds
  to goals as news (`red_news`, `RedLogs`, `tell`, `repairs`): those are M9's, leave them.

## Owns and may change

- Owns the files in the header for this wave; `tiers-verification` runs beside you and owns
  `verification.rs` and the change's facts in `domain.rs`: touch neither.
- `TierRow` and its store record may grow the fields a rerun or probe needs to survive a
  restart (`temper/crates/temper-engine-domain-forge/src/boundary.rs`,
  `temper/crates/temper-protocol-store/**`, regenerated as `tiers-surface` did).
- Not this task's: the root, the panel, M9's news.

Pre-approved options:

- If the fake forge's `select` input name differs from `temper_tests`, configure the world's
  workflow with `temper_tests`; never rename temper's input.
- If a probe's branch cannot be released before the run ends within `resources_per_task`, keep
  at most one probe's branch at a time (probes run one after another anyway) and say so.

## Steps

1. Read t9's commits, then write the report read, reruns and probes in today's `tiers.rs`.
2. Unit tests beside `tiers-runs`', then the forge world's stories.
3. `cargo fmt`; clippy over every package you changed; the tests in Goal, counts reported.
4. Commit in steps that say why; push your branch.

## Report

As the implementer's text says, and also: `max_bytes` as you computed it; the stories and their
seeds; what of t9 you took.
