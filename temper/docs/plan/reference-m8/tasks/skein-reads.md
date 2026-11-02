+++
id = "skein-reads"
part = "skein-reads"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/**", "skein/crates/skein-forge-client/**", "skein/crates/skein-forge-connection/**", "skein/crates/skein-forge-setup/**", "skein/tests/forge/client/**", "skein/tests/forge/conformance/**"]
effort = "high"
minutes = 240
+++

# skein: `Read::Artifact` and `Read::Commits`, Forgejo's dialect

## Goal

skein's forge kit reads a run's artifact by name (one file, stored or deflated, cut to
`max_bytes`) and the commits of a range with their messages, oldest first, paged, in Forgejo's
dialect, with the pinned Forgejo's recorded exchanges; GitHub's dialect answers both
`Refused`. temper's tiers read their runs' reports through the first. The old line holds the
work; this task ports its non-fake half.

Acceptance:

- `skein_forge::api::Read::{Artifact { run, name, max_bytes }, Commits { before, after, page }}`
  and `Answer::Commits { commits, more }` (with its commit item) as
  `skein/docs/design/forge.md`, 3.2 and 3.3 say; the fit rules of 3.7; Forgejo's routes of 5.2
  (`GET R/actions/runs/<run>/artifacts?name=`, then `GET R/actions/artifacts/<id>/zip`; `GET
  R/compare/<b>...<a>?files=false&verification=false`, reversed to oldest first, then paged).
- A zip of more than one file, or compressed otherwise than stored or deflated, is `Refused`;
  sizes and CRC are checked before the cut. The bounded raw-DEFLATE inflater is private to
  `skein-forge` (`notes/contracts.md`, 5): `skein-forge` gains no dependency and `skein-http`
  is unchanged.
- The forge client and connection admit both reads; GitHub's plans answer `Refused` at cost 0.
- The pinned conformance recordings of the artifact exchange land under
  `skein/tests/forge/conformance/transcripts/` with their `facts.txt` lines, and every
  transcript still decodes (`crates/skein-forge/tests/common/forgejo.rs`, `json_cases`).
- The fake forge answers the new requests `Refused` until `skein-fake-reports` gives it reports
  (the precedent: `JobLog` in `skein/crates/skein-fake-forge-protocol/src/routes.rs`).
- temper compiles: `heavy cargo check -p temper-engine-domain -p temper-engine-domain-forge
  -p temper-engine-smith -p temper-protocol-store -p temper --tests` and `heavy cargo check -p
  temper-engine-forge-world --tests`.
- The tests in Steps pass, each count in the report.

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`; `notes/contracts.md`, 5.
- `skein/docs/design/forge.md`, 2.1, 3.2, 3.3, 3.7, 5.2 and 9; `skein/docs/design/forge-github.md`,
  4.5 (GitHub's routes, which this task does not build: it refuses).
- The old code: `temper/docs/plan/reference-m8/notes/old/skein-reads.diff`, the old
  `verdicts/skein-reads-3` squashed (eleven commits: the inflater's move, commit reads and fake
  reports, the conformance probe and recordings, the Forgejo artifact plans, flaky ids), paths
  under `skein/`, the old failed-attempt evidence directories left out. Take its non-fake half;
  `skein-fake-reports` takes the rest.
- What moved since (a survey checked the patch against this tree):
  - test files now live under `tests/*/tests/suite/`:
    `tests/forge/client/tests/{memory,recovery,simulation}.rs` →
    `tests/forge/client/tests/suite/`, `tests/forge/protocol/tests/protocol.rs` →
    `tests/forge/protocol/tests/suite/protocol.rs`,
    `crates/skein-forge/tests/forgejo_files.rs` → `crates/skein-forge/tests/suite/forgejo_files.rs`;
  - heap-sum accounting is gone (`worst_case`, `forgejo_call_heap`, `FIXTURE_HEAP`,
    `kept_run_heap`): drop those hunks; a new memory assertion uses a measured peak ceiling, as
    `skein/crates/skein-forge-connection/src/tests/github_files.rs` does, and
    `tests/forge/client/tests/suite/memory.rs`'s `Measured::new(limits, ceiling)`;
  - new shapes: `Answer::Directory`, `FileKind::DirectoryEntries`
    (`skein-forge/src/forgejo/client/files.rs`), `Request::Contents`
    (`skein-forge-connection/src/connection.rs`), `Read::File` among the unchecked reads in
    `skein-forge-client/src/bounds.rs` (so `Artifact`'s non-empty name is its own arm);
  - `skein/crates/skein-fake-forge-protocol/src/tally.rs`: `ROUTES` is 81 today; recount it and
    its comment for the routes this task adds.

## Owns and may change

- Owns the crates and test packages in the header.
- `skein/crates/skein-fake-forge-protocol/src/{routes.rs,tally.rs}` and
  `skein/crates/skein-fake-forge-domain/src/reads.rs` (or wherever the fake's exhaustive matches
  on the forge's `Read`/`Request` sit): only arms answering the new reads `Refused`.
- temper, each only the arm the compiler asks for (none persists or acts on the reads):
  `temper/crates/temper-engine-domain/src/route/forge_route.rs` (`read_name`: the `None` arm);
  `temper/crates/temper-engine-domain-forge/src/domain.rs` (the answer ignore arm, about line
  4494); `temper/crates/temper-engine-domain-forge/src/restore.rs` (`answer_rows`: count the
  commits); `temper/crates/temper-engine-smith/src/answers.rs` (`forge_answer`'s unreachable
  arm); `temper/crates/temper-protocol-store/src/records/answers.rs` (the "queue does not
  persist" arm); `temper/tests/engine/forge/src/translate.rs` (`read_op`, `answer_as`,
  `pull_answer`: a panic arm naming `skein-fake-reports`, as its line ~366 does for others).
- Not this task's: the fake forge's report model, `fails`, `flaky`, `select`, `Cue::Tests` and
  the fake's artifact serving (`skein-fake-reports`); any design or status file.

Pre-approved options:

- If a recording no longer matches the pinned Forgejo 17.0.0 exchange shape, keep the facts
  that do and say which were dropped; record nothing anew.
- If forge.md 9's comparison fact (commits oldest first with their messages) is shown by an
  existing transcript, say which in the report; do not edit the design.

## Steps

1. Apply the diff on a scratch commit (`git apply --3way --exclude` the fake crates' report
   model as you go, or apply whole and back out the fake half), then re-home the moved tests and
   drop the heap-sum hunks. Read every hunk against today's file.
2. `heavy cargo check -p skein-forge -p skein-forge-client -p skein-forge-connection
   -p skein-forge-setup -p skein-fake-forge-protocol --tests`, then temper's checks (Goal).
3. Tests, each alone first:
   `heavy cargo nextest run -p skein-forge` (lib and `suite`: the artifact and commits plans,
   zip, GitHub's refusals, `forgejo_files`);
   `-p skein-forge-client --lib`; `-p skein-forge-connection --lib`;
   `-p skein-forge-client-world --test suite`; `-p skein-forge-protocol-world --test suite`;
   `-p forgejo-conformance --lib` (`recorded_corpus_has_no_unmasked_uuid_or_log_time`).
4. `cargo fmt`; clippy over every package you changed, temper's included; commit in steps that
   say why; push your branch.

## Report

As the implementer's text says, and also: the old commits taken, adapted or dropped; the
recordings added; the new memory ceilings and their measured peaks; the temper arms added.
