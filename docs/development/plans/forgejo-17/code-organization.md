# forgejo-17: code organization

2026-10-16. Crates, names, shared interfaces, fixtures, the plan's
"Don't" list and its size limits. No crate is added, renamed or removed.

## 1. Crates

| Crate | What changes | Sessions |
|---|---|---|
| `testing/temper-throwaway-forgejo` | `VERSION` and `RUNNER_VERSION`, each checked exactly against the binary's first `--version` line; the recorder's suffix range (`src/http.rs`) | 01, 02 |
| `tools/forgejo-conformance` | the runner checked by `RUNNER_VERSION`; the captures and facts of `forge.md`, section 13; the component's facts for `Read::Job` and the comparisons; its transcripts and `facts.txt` recorded on v17 | 01, 02, 04 |
| `tools/git-transcripts` | its header names Forgejo's version and digest; the redirect's fact reads a recorded fetch; its transcripts and `facts.txt` recorded on v17 | 01 |
| `crates/temper-forge-forgejo` | `Request::Compare`'s flags; `Request::Job`; `JobLog`'s `step`; the job document with its steps (client decoder, server encoder, route); `Content-Range`'s `bytes 0--1/0` (`header.rs`) | 02, 04 |
| `testing/temper-fake-forge-domain` | jobs with three steps, a seeded failing step, a log per step, a job read by its id | 03 |
| `testing/temper-fake-forge-protocol` | the routes `actions/jobs/{id}` and `logs?attempt&step`; suffix ranges; the log served whole | 03 |
| `crates/temper-engine-domain-forge-client` | `api::Step` and `Answer::Job`'s `step`; its bound | 04 |
| `crates/temper-engine-protocol-forge` | the job read through the job's steps; the range's two kinds; a `200`'s end kept as it streams; the comparisons' flags sent | 02, 04 |
| `crates/temper-protocol-store` | `JobAnswer`'s step, in the schema `main` holds (decisions.md, 4) | 04 |
| `crates/temper-engine-smith` | the `job` tool's answer names the step | 04 |
| `crates/temper-engine-domain-forge` | the brief names the failed step and its log's end | 04 (compiles), 05 |
| `tests/codecs` | the v17 transcripts' literals; the new captures decoded | 01, 02 |
| `tests/engine/protocol-forge` | the joint world's job read against the fake's steps | 04 |
| `tests/engine/forge/client` | `translate.rs`'s job answer: the fake's renamed reads (03.1), then with its step (04.1) | 03, 04 |
| `tests/fake-forge` | the fake's memory test, following its jobs with steps | 03 |
| `tests/engine/domain`, `tests/system` | the brief's step in the failed-CI stories | 05 |
| `tests/worker/protocol-git`, `testing/temper-fake-git` | follow git's transcripts recorded again, if a literal moved | 01 |

## 2. Names

- **Versions:** `temper_throwaway_forgejo::VERSION` (`"17.0.0"`) and
  `RUNNER_VERSION` (`"13.2.0"`). Nothing else in code names a Forgejo
  or runner version; doc comments say "the pinned Forgejo"
  (`temper_throwaway_forgejo::VERSION`), never a number.
- **Forgejo's documents** (`temper_forge_forgejo::document`, `d::` in
  the component): the existing `d::Job { id, run_id, attempt }` is
  unchanged, `Copy`; `d::Step { number: u32, name: Box<[u8]>, status:
  d::StepStatus }`; a job read by its id is `Document::Job(d::Job)`,
  its steps delivered before it as rows, `Row::Step(d::Step)`, as they
  stream (design question 6: a job of any number of steps is read); the
  server's `Answer::Job { job: d::Job, steps: Box<[d::Step]> }`. No
  second job type.
- **Requests** (`temper_forge_forgejo::request`): `Request::Job { at,
  job }` (`GET R/actions/jobs/<job>`); `Request::JobLog { at, job,
  attempt, step: Option<u32> }`; `Request::Compare { at, before, after,
  files: bool }` (`files: false` sends `files=false`; every comparison
  sends `verification=false`).
- **The answer** (`temper_engine_domain_forge_client::api`): `Step {
  number: u32, name: Box<[u8]> }`; `Answer::Job { attempt, step:
  Option<Step>, log, truncated }`.
- **The range** (`temper_engine_protocol_forge::plan`): `Exchange {
  request, range: Option<Range> }`, `enum Range { Head(u32), Tail(u32)
  }`: `Head(n)` writes `bytes=0-<n>` (a file's, as today), `Tail(n)`
  writes `bytes=-<n>` (a log's end).
- **The fake's failing step** (`temper_fake_forge_domain`): `enum
  Failing { Workflow, NoStep }`, seeded per CI context, `Workflow` by
  default: the workflow step fails (as v17 was seen to,
  `findings/measurements.md`), or no step does (a cancelled job, which
  `forge.md`, 3.2 names).
- **Tests** are named for the behaviour, about eight words
  (development.md, 5.6), in the crate's existing test modules.

## 3. Shared interfaces and their owners

Each is defined by one increment; the others use it by name.

| Interface | Owner |
|---|---|
| `VERSION`, `RUNNER_VERSION` | temper 01.1 |
| `d::Step`, `d::StepStatus`, `Row::Step`, `Document::Job`, `Answer::Job`, `Request::Job`, `JobLog`'s `step` | temper 02.1 |
| the recorder's suffix range (`testing/temper-throwaway-forgejo/src/http.rs`) | temper 02.1 |
| `Request::Compare`'s `files`, and `verification=false` on every comparison | temper 02.2 |
| the fake's `Failing`, its steps and their logs | temper 03.1 |
| the fake protocol's suffix range | temper 03.2 |
| `api::Step`, `Answer::Job`'s `step`, `JobAnswer`'s step in the store; `translate.rs`'s mapping of `Read::Job` to the fake's job read | temper 04.1 |
| `Range { Head, Tail }`, a `200`'s end kept | temper 04.2 |
| the brief's words for the failed step | temper 05.1 |

## 4. Fixtures

Each crate's existing fixtures module grows; none is redefined.

- `crates/temper-engine-protocol-forge/src/tests/fixtures.rs` and the
  job constants of `src/tests/contents.rs:302-316` (`ATTEMPT`, `RUN`,
  `RUNS`, `JOBS`, `FAILING`, `failing()`).
- `crates/temper-forge-forgejo/src/tests.rs`.
- `tests/codecs/tests/forgejo_files.rs`'s `cases()`, `run()`, `jobs()`,
  `log()` (`:17-116`).
- `testing/temper-fake-forge-domain/src/tests/actions.rs`'s `failing()`
  (`:4-13`); `testing/temper-fake-forge-protocol/src/tests/response.rs`
  (`:129-165`), `src/tests/routes.rs`.
- `tests/engine/protocol-forge/src/fixtures.rs` and `joint.rs`.
- `tests/engine/domain/tests/forge_routes.rs`'s `change_world` (`:633`),
  `configured_policy` (`:78-311`).
- `crates/temper-protocol-store/src/tests/fixtures.rs` (`:464-468`).

Limits in tests come from the shipped profile's builders, as they do at
the base (`crates/temper-service/src/limits.rs`,
`crates/temper-service/src/forge.rs`); a test that needs another value
names it beside the profile's, with the reason.

## 5. The plan's "Don't" list

No increment:

- adds a type beside an existing one for the same thing: it changes the
  type in place, and its users in the same increment (`d::Job`, not a
  second job document; `Answer::Job`, not a second answer);
- adds a field or a predicate that its counterpart across a boundary
  lacks: the fake's server and the component's client decode and encode
  the same documents, from `temper-forge-forgejo`;
- writes a test that checks several behaviours under one name;
- adds a lint exception in tests;
- writes an expected value computed by the code under test: a
  transcript's literal is read from the transcript, by hand;
- writes the words the pulse forbids (README, "Pulse");
- edits a transcript or a `facts.txt` by hand, or inserts a capture
  before an existing one;
- reads a log's search (`q`, `qi`) or its NDJSON form, writes a re-run,
  or subscribes to a webhook (`draft/ci-on-forgejo-17.md`; `later.md`);
- moves `STORE_VERSION` (decisions.md, 4);
- makes the fake forge better than Forgejo: the fake answers what the
  conformance's facts record (`forge.md`, section 9);
- names a Forgejo or runner version in code, but `VERSION` and
  `RUNNER_VERSION`.

## 6. Size limits and the count

- **Aims:** a file of at most 800 lines and a test function of at most
  40; a miss is noted in the landing's report, and the landing stands.
- **Rule: a file over 800 lines at the base grows by at most 40 lines**
  in one increment: for each such file the increment changes, `git diff
  --numstat origin/main...HEAD -- <file>` prints added and removed
  lines, and added minus removed is at most 40; the landing's report
  gives the two numbers. Code past that goes in a new module beside the
  file. At the base these include
  `crates/temper-engine-domain-forge-client/src/keep.rs` (1377),
  `src/outbox.rs` (1079) and `testing/temper-fake-forge-domain/src/tests.rs`
  (2150); `wc -l` names the others.
- **An increment's size** is its hand-written production lines,
  counted once before it lands, in its worktree:

  ```sh
  git diff -M --numstat origin/main...HEAD -- crates testing tools \
    ':!*/tests.rs' ':!*/tests/*' ':!*/generated/*' ':!*/golden/*' \
    ':!*/schema/*' ':!tools/*/transcripts/*' ':!tools/*/facts.txt' \
    | awk '{a+=$1} END {print a}'
  ```

  `tests/` (worlds), generated code, goldens, schemas, transcripts and
  facts are not counted. Each landing's report gives the number
.
