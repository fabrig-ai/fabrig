# Session 03: the fake forge's jobs with steps, their logs by step, suffix ranges

The fake forge's Actions jobs take the shape Forgejo v17 gives them. A
job has three steps: 0 `Set up job`, 1 one workflow step, 2 `Complete
job`. Where a failed job fails is set per CI context, as a `Failing`:
at its workflow step (the default), with the completion step failing
too, or at no step (a cancelled job). Each step has a log slice of its
own, and the job's log is the three slices in order. The fake's domain
reads a job by its id with its steps, and a log by attempt and
optional step, uncut. Its protocol layer serves `GET
R/actions/jobs/{id}` and `logs?attempt&step`, and applies the request's
byte range to what the domain answered. 03.2 adds suffix ranges
(`bytes=-N`) and v17's answers over an empty log. The forge component
reads none of this until temper 04 (04.1 the job and its step, 04.2
the suffix range). Until then the fake's own HTTP-bytes tests are the
highest layer that runs it.

## Read first

- `docs/design/forge.md`: section 9 (the bullets "The routes of section
  3.2", "Forgejo's quirks, kept" and "Faults move down"); 3.2 (the row
  `Read::Job` and the note "A failed job's log is its failed step's
  end"); 13 (the bullets on a job and a job's plaintext log, under "From
  Forgejo v17.0's API description" and under "To check on v17.0").
- `docs/design/domain/testing.md`: 4.2 (the rule "Forgejo-shaped, not better
  than Forgejo", and "Then a service").
- This plan: `decisions.md` 7, 8 and 9; `code-organization.md` sections
  2 ("The fake's failing step"), 3, 4, 5 and 6;
  `findings/fake-forge-and-brief.md` sections 1, 2, 5 and 6;
  `findings/measurements.md` sections 2 and 3; `findings/answer.md`
  section 1 (its last bullet: the fake has no `Answer::Job`).
- `temper/02-documents.md`, 02.1's Shape: `d::Step`, `d::StepStatus`,
  `server::Answer::Job`, `Document::Job`, `Request::Job`, `JobLog`'s `step`,
  the server's route and encoder, and what 02.1 does to the fake
  protocol's matches. This file's assumptions about 02.1 are listed under
  "Assumed from temper 02.1" below.
- skein's `programming-model.md`, by number only: 4.4 (protocol to
  domain), 6.3 (the worst case). skein's `testing-strategy.md`, by
  number only: 2.1 (step tests), 4 (its opening list: "No shared
  types", "It checks its client"), 4.1 (transcripts).

## What the design asks

- **The routes** (forge.md, 9): the fake's domain grows the reads its
  routes need: "a job by its id (`actions/jobs/{id}`) with its steps,
  its log read by attempt and step (`actions/jobs/{id}/logs?attempt&step`)
  with byte ranges, a suffix range among them".
- **The quirk** (forge.md, 9, "Forgejo's quirks, kept"): "a failed job's
  completion step reports `failure` too. A client that leans on Forgejo
  being better than it is fails in the fake first."
- **The job and its log** (forge.md, 3.2): "Forgejo answers a job
  (`actions/jobs/<job>`) with its steps, those of its latest attempt:
  number 0 is the runner's set-up (`Set up job`), the last its
  completion (`Complete job`), and the workflow's own steps are between.
  The failed step is the first whose `status` is `failure`: the
  completion step reports `failure` whenever the job fails, so it is the
  failed step only when none before it failed." "A job with no step
  failed (one cancelled, or one whose runner was lost) is read whole, its
  end the same way". "a range over an empty log is an empty log (13)".
- **What Forgejo serves** (forge.md, 13): "A job's plaintext log is read
  by `actions/jobs/{job_id}/logs?attempt=`, its latest attempt's without
  it, with `step=` for one step's slice; it is served with byte ranges
  (Go's `http.ServeContent`: `200`, or `206` for a range, a suffix range
  included), over the step's slice when one is named. A step past the
  last, a job never run and a log expired answer `404`."
- **What v17 answered** (`findings/measurements.md`, 3): `Range:
  bytes=-10` over a 112-byte slice, "`206`, `Content-Range: bytes
  102-111/112`, the last 10 bytes"; `bytes=-100000`, "`206`,
  `Content-Range: bytes 0-111/112`, the whole slice"; over an empty
  slice, `bytes=-10` "**`206` with `Content-Range: bytes 0--1/0`** and
  an empty body", and `bytes=0-10` "`200`, empty, no `Content-Range`".
- **Not better than Forgejo** (domain/testing.md, 4.2): "It refuses what
  Forgejo refuses ..., and keeps what Forgejo keeps and no more."
- **Faults move down** (forge.md, 9, domain/testing.md, 4.2): "a call refused
  for the rate is a `429` with `Retry-After`; one failed before it is
  made is a refused connect or a reset before the request is read; one
  made and then timed out is a `5xx` or a dropped connection after the
  request was read; a late answer is a slow one."
- **The plan's shape of it** (decisions.md, 9): "three steps, `Set up
  job`, one workflow step and `Complete job`, each with a log slice of
  its own. Where a job fails is seeded per context: the workflow step
  (the default), or no step (a cancelled job: its workflow and
  completion steps `cancelled`)."

## Choices this plan makes

- **The fake's reads take Forgejo's resource names.** `Read::Job { job
  }` reads the job by its id, with its steps, each carrying its log
  slice. `Read::Log { job, attempt, step }` reads a
  log: it is today's `Read::Job { job, attempt, max_bytes }`, renamed,
  with `step` in place of `max_bytes`. It stays for the protocol layer's
  log route (`logs?attempt&step`); the domain worlds' translation reads
  `Read::Job` alone (below). This matches the fake's `Run`,
  `Runs` and `Jobs` and Forgejo's routes. The rename touches only the
  lines that the field change touches anyway (03.1's Files).
- **Steps are made when read, not stored.** A job's steps and slices
  come from its context's name and its context's `Failing`, when one of
  the two reads asks. Nothing is kept per job beyond today's `Job`.
- **`Failing` is set by the scenario, per context, after the pattern of
  `set_reachable`.** `scenario::set_failing(domain, repository, context,
  failing)` sets it. Each repository keeps one `Failing` per context of
  its checks, each `Workflow` when the repository is added. `Failing` is
  not a field of `api::Checks`, which has 15 struct literals across the
  worlds at the base (`grep -rn 'checks: .*Checks {' --include=*.rs tests testing | wc -l`
  prints 15).
  temper 05 assumes this call (`temper/05-brief.md`, 05.1's Shape).
- **Both reads answer only a job that CI failed.** A job is read, by its
  id or by its log, only when CI's latest status for its context on its
  commit is `Failed`. This is today's rule for the log
  (`testing/temper-fake-forge-domain/src/next.rs:164`). Any other job
  answers `Missing(Job)`, which is a `404`. The component reads only the
  jobs that `Read::Checks` links, and those are CI's failed statuses
  (`next.rs:138`). A pending or passed job read by its id is a question
  (below).
- **The synthetic steps and slices,** for a context `<c>`:

  | Step | Name | `Workflow` | `NoStep` | Its slice |
  |---|---|---|---|---|
  | 0 | `Set up job` | `Success` | `Success` | `set up <c>\n` |
  | 1 | `Run <c>` | `Failure` | `Cancelled` | `failing check <c> 0\n` to `failing check <c> 3\n`, the four lines `next.rs:175-188` makes today |
  | 2 | `Complete job` | `Failure` | `Cancelled` | `job <c> failed\n` (`Workflow`), `job <c> cancelled\n` (`NoStep`) |

  `Run <c>` is the fake's own name for its workflow step; temper 05
  relies on it (`Run build`). The completion's slice is not empty and
  differs from the failing lines. So a client that takes the last
  `failure` step instead of the first reads `job <c> failed`: such a
  client "fails in the fake first" (forge.md, 9). No slice is empty. The
  empty log is 03.2's, tested over an empty body at the response layer.
- **The domain's `max_bytes` goes.** `Read::Log` answers the slice, or
  the job's log, uncut, and `Read::Job`'s steps carry the same slices.
  Their size is bounded by the lines above: at most
  `6 × name_bytes + 91` bytes. The domain's worst case counts that. The
  protocol layer applies the request's range to it. It refuses a body
  longer than `raw_bytes` with `TooLarge` (`413`), as it refuses any raw
  body today (`testing/temper-fake-forge-protocol/src/response.rs:668-672`
  before the range, `:482` after it).
- **translate reads the workflow step's slice.** In the domain worlds,
  `tests/engine/forge/client/src/translate.rs` maps one client call to
  one fake call, and its callers make one call each
  (`tests/engine/forge/client/src/world.rs:439-451`,
  `tests/engine/domain/src/forge_stand.rs:96-119`,
  `tests/system/src/forge.rs:339-417`, and the same at
  `tests/engine/domain/tests/forge_routes.rs:438,524`,
  `tests/conformance/src/systems.rs:164,311`, and
  `tests/engine/forge/src/world.rs:506,520`, which compiles the same
  `translate.rs` through `#[path]`, `lib.rs:4-5`). So `:111-113` sends `Read::Job {
  job: attempt.job }`, and a new arm of `answer` (`:188`) for
  `forge::Answer::Job { steps, .. }` passes the log of the step numbered
  `WORKFLOW_STEP` to `byte_answer` (`:498-509`): the arm is
  `byte_answer(asked, workflow_log(&steps))`, with `fn workflow_log(steps:
  &[forge::Step]) -> &[u8]`, a loop (the workspace forbids closures),
  beside `byte_answer`, since `answer` is 92 of clippy's 100
  `too_many_lines` at the base. `byte_answer` stays as it is:
  the last `max_bytes` bytes, `truncated` when the slice is longer. It
  answers today's `client::Answer::Job { attempt, log, truncated }`.
  That slice is today's log byte for byte, so the briefs of
  the domain and system worlds keep `failing check build 3`
  (`tests/engine/domain/tests/forge_routes.rs:1233-1240`,
  `tests/system/src/forge/stories.rs:256-262`). The job's log would end
  with the completion's line instead. The brief asks at most `ci_budget
  / 5` bytes, 76 in that world, so the failing lines could fall out.
  temper 04.1 replaces this arm when it makes the answer name the
  step (the first `Failure` step's slice, or the slices joined when none
  failed); the read stays `Read::Job`.
- **Ranges follow Go's `http.ServeContent`, as v17 answered them**
  (measurements.md, 3). `bytes=-N` with N at least 1 is a suffix: `206`,
  the last `min(N, T)` bytes, `Content-Range: bytes <T − min(N, T)>-<T −
  1>/<T>`. Over an empty body it answers `206` with nothing and
  `Content-Range: bytes 0--1/0` (Go writes the last byte as first +
  length − 1). A `bytes=F-L` or `bytes=F-` range over an empty body
  answers `200` with nothing (Go ignores a range that misses an empty
  body; measured for `bytes=0-10`). `bytes=-0` and `bytes=-` stay
  refused (`422`), as today: the component never sends them, since the
  client refuses a `max_bytes` of 0
  (`crates/temper-engine-domain-forge-client/src/bounds.rs:29`). A
  `bytes=F-` range starting past a non-empty body stays `416`.
- **The empty body's `200` reaches raw files too.** `raw` serves
  `Request::Raw` and `Request::JobLog` alike (`response.rs:135,334-345`),
  so an empty file read with `bytes=0-N` answers `200` with nothing, as
  v17's `ServeContent` does. No test at the base relies on the `416`:
  `grep -rnw 416 --include=*.rs crates tests testing tools` names the
  fake's own row with a non-empty body (`src/tests/response.rs:135`),
  the code (`response.rs:450`) and an unrelated number
  (`tests/engine/domain/src/escalation.rs:296`).
- **Faults reach the new reads unchanged.** `Read::Job` and `Read::Log`
  are domain calls like any other read. The domain's rate, unavailable,
  timeout and late draws reach them, and so do the protocol's reset, drop
  and `5xx` draws (`lib.rs:35-39`). Nothing is added for them.

## Rules that bind it

- **The fake is not better than Forgejo** (code-organization.md, 5): no
  increment "makes the fake forge better than Forgejo: the fake answers
  what the conformance's facts record (`forge.md`, section 9)". Its
  steps, statuses, `404`s and ranges are those v17 was seen to give
  (measurements.md, 2 and 3; decisions.md, 9).
- **No field its counterpart lacks** (code-organization.md, 5): the
  fake's `Step` has `d::Step`'s fields, `number`, `name` and `status`,
  and its `log`, the bytes `Read::Log` answers for that step; no other.
  The fake's protocol encodes its job through `temper-forge-forgejo`'s
  server without the logs, as `d::Step` has none, and the component's
  client decodes it; the logs are served at the log route alone.
- **A file over 800 lines at the base grows by at most 40 lines in one
  increment** (code-organization.md, 6). The fake domain's
  `src/tests.rs` has 2,150 lines: this session adds no test there. Its
  tests go in `src/tests/actions.rs` (236 lines). `src/api.rs` has 785
  lines, against the 800-line aim: the new types go in `src/actions.rs`
  and are re-exported, and a miss past 800 is noted in the landing's
  report. The landing's report gives `git diff --numstat
  origin/main...HEAD -- testing/temper-fake-forge-domain/src/tests.rs`,
  which prints nothing.

## Assumed from temper 02.1

These names are `code-organization.md`'s, section 2, and the README's
"Needs at start" for this session.
Where 02.1 landed others, its names win, and the author reports the
difference:

- `d::Job { id, run_id, attempt }`, unchanged and `Copy`, as the fake's
  listing at `testing/temper-fake-forge-protocol/src/answers.rs:357`
  builds it (design question 6: a client reads a job's steps as rows).
- `d::Step { number: u32, name: Box<[u8]>, status: d::StepStatus }`,
  where `d::StepStatus` has `Success`, `Failure` and `Cancelled` among
  its variants, written `success`, `failure` and `cancelled`.
- `server::Answer::Job { job: d::Job, steps: Box<[d::Step]> }`,
  encoded with a step written `{"number":…,"name":"…","status":"…"}`,
  in that order, and no `steps` member when there are none.
- `Request::Job { at, job }` for `GET R/actions/jobs/<job>`, and
  `Request::JobLog { at, job, attempt, step: Option<u32> }`, both
  routed by `temper-forge-forgejo`'s server.
- To compile, 02.1 names `Request::Job { .. }` in each exhaustive match
  of the fake protocol: `routes.rs` (`repository`, `op`, `write`),
  `answers.rs`, `response.rs` (`follow`, `select`), `server.rs`
  (`table`) and `tally.rs`. Assumed: `tally::Route::Job`, answered `200`
  in `response::status` (`response.rs:172-213`), with the route map
  sized 40 (`tally.rs:104,114`, `server.rs:432`); `routes::repository`
  names its repository; `response::select` answers `Select::Document`;
  `routes::op` answers it `Err(a::Error::Missing(a::What::Job))`, a
  `404` (02-documents.md, "Choices this plan makes", 5), and ignores `JobLog`'s
  `step`. Where 02.1 did otherwise, 03.1 does these.

## Increments

### 03.1 Jobs with steps, read by id and by step

Scope: the fake's jobs have three steps, a log slice each, and a
`Failing` per context set by the scenario. The domain reads a job by its
id with its steps, each with its log slice, at its latest attempt. It reads a log by attempt and
optional step, uncut, and a step past the last is `Missing(Job)`. The
protocol routes `actions/jobs/{id}` and `logs?attempt&step` to those two
reads, and keeps head ranges as today (forge.md, 9, 3.2 and 13;
decisions.md, 9).
Needs: temper 02.1 only (the session's start).
Files:
- `testing/temper-fake-forge-domain/src/`:
  - `actions.rs`: `Failing`, `Step`, `StepStatus`, `WORKFLOW_STEP`,
    `latest`, `steps`, `log`;
  - `api.rs` (785 lines): `Read::Job` (`:104-110`) becomes `Read::Log`,
    then add `Read::Job`, `Answer::Job`, and the re-export at `:515`.
    `:104`'s doc comment is 01.1's line, which this replaces;
  - `next.rs:150-192`: `job` and `log` replace today's `job`;
  - `reads.rs:67`, and `domain.rs:515-544`, the permission arms;
  - `scenario.rs`: `set_failing`, and its export in `lib.rs:75-78`;
  - `store.rs`: `Repository::failing` (`:20-53`), set in `new`
    (`:129-166`);
  - `limits.rs`: the worst case, in `repository` (`:143-146`) and
    `answer` (`:197-261`).
- `testing/temper-fake-forge-domain/src/tests/actions.rs` (the new tests,
  and `:72-111` rewritten) and `src/tests/next.rs` (`:163` rewritten,
  `:166-190` removed).
- `testing/temper-fake-forge-protocol/src/` (its line numbers are
  origin/main's before 02.1, 3ed817888; 02.1 added one line at each of
  `response.rs:110`, `:158`, `:206`, `routes.rs:98`, `:217`, `:319`,
  `answers.rs:212`, `:275`, `:438`, `server.rs:506`, `tally.rs:32`,
  `:75`, so a line past one of them is that many lines on):
  - `routes.rs`: `:2`, the `Limits` import goes; `op` loses `limits`
    (`:142`); the arms `:215-217`, and `Request::Job` in `op`;
  - `answers.rs:296-394`: `a::Answer::Job`, and `step`;
  - `response.rs`: `Answer::Job { .. }` joins the lists at `:30-60`,
    `:292-319`, `:346-376` and `:390-422`, and gets an arm in
    `bounded::answer` (`:593`), which is 100 of clippy's 100
    `too_many_lines` lines: the job is priced in a new
    `Budget::steps(&[a::Step])` (row 0, then one row per step's name)
    beside `files` (`:585`), and the `Runs` and `Jobs` arms
    (`:646-655`) fold into a new `Budget::rows(len)` to make room;
  - `server.rs:447`, `routes::op`'s call;
  - `tally.rs`, if 02.1 left it as the assumptions above say it does
    not.
  These files are 02.1's first: it adds `Request::Job` there.
- `testing/temper-fake-forge-protocol/src/tests/routes.rs` (`:1`, the
  fixtures' `limits` import goes once `op` loses `limits`; `:14-107`,
  `read_rows`; `routes::op`'s calls at `:113` and `:208`),
  `src/tests/answers.rs` and `src/tests/http.rs`.
- Shared hotspots: the fake protocol's `src/answers.rs`,
  `src/tests/answers.rs` and `src/tests/routes.rs`, which temper 02.2
  (the comparisons' flags) also edits and may edit at the same time.
- Users of the changed reads and answer, found by `git grep -n -e
  'Read::Job' -e 'Answer::Jobs' -- testing tests` (the fake's, not the
  client's):
  - `tests/engine/forge/client/src/translate.rs`, `:111-113`
    (`Read::Job { job: attempt.job }`), and `answer` (`:188-279`), where
    `forge::Answer::Job { steps, .. }` gets an arm beside `File`'s
    (`:240`) that calls `byte_answer` with the workflow step's log
    ("Choices"). temper 04.1 changes this file next;
  - `tests/fake-forge/tests/memory.rs:285-322`, where `outcome` gains
    `Answer::Job { .. }`;
  - `tests/engine/forge/client/tests/simulation.rs:495-505`
    (`bounded_job_bytes_translate_to_a_truncated_attempt_log`), which
    passes `Answer::File` for a `Read::Job`: it passes `Answer::Job`
    with the workflow step's log, so that it reads the new arm.
- Run unchanged, as regressions:
  - the domain world's job brief tests
    (`tests/engine/domain/tests/forge_routes.rs:1178-1294`) and the
    system world's failed-CI story (`tests/system/src/forge/stories.rs:209-326`),
    both through translate;
  - the joint world, `tests/engine/protocol-forge`, which reads no job;
  - `tests/one-process`, whose CI never fails (`fixtures.rs:41`).
Shape:
- `temper_fake_forge_domain::api` re-exports `actions::{Failing, Job,
  Run, Step, StepStatus, WORKFLOW_STEP}` (sketched below). `Step` has
  `d::Step`'s fields, `number`, `name` and `status`, and `log: Box<[u8]>`,
  its slice.
- `api::Read`:
  - `Job { job: u64 }`: the job by its id. It answers `Answer::Job { job:
    Job, steps: Box<[Step]> }`, `job` at its latest attempt (its
    `attempt` as `rerun` set it, `actions.rs:43-48`), each step with
    `number`, `name`, `status` and `log`, the bytes `Read::Log` answers
    for that step;
  - `Log { job: u64, attempt: u32, step: Option<u32> }`: the attempt's
    log, as `Answer::File`, for the protocol layer's log route. With `step` it is that step's slice; without
    one, the three slices in order. A step above 2 is `Missing(What::Job)`.
    So is an attempt other than the job's, as today (`actions.rs:80-91`).
- Both reads answer `Missing(What::Job)` unless CI wrote the latest
  status of the job's context on its commit, as `Failed` (today's gate,
  `next.rs:157-166`, kept in one private function of `next.rs` that both
  reads call).
- `actions::latest(repository: &Repository, id: u64) -> Result<(u64,
  Job), Error>`: the job and its run's commit.
  `actions::steps(context: &[u8], failing: Failing) -> Box<[Step]>` (each
  step with its slice) and
  `actions::log(context: &[u8], failing: Failing, step: Option<u32>) ->
  Result<Box<[u8]>, Error>` give the table of "Choices", reusing
  `append` (`actions.rs:122-126`).
  `next::job(repository: &Repository, job: u64, ci_user: u64) ->
  Result<Answer, Error>` and `next::log(repository: &Repository, job:
  u64, attempt: u32, step: Option<u32>, ci_user: u64) -> Result<Answer,
  Error>` are called from `reads.rs` with `config.ci`.
- `store::Repository` gains `pub(crate) failing: Box<[Failing]>`, one per
  context of `checks.contexts`, in their order, each `Failing::Workflow`
  in `Repository::new`. `scenario::set_failing(domain, repository,
  context, failing)` sets the context's entry. It panics on a context
  the repository's checks lack (`expect("a context of the repository's
  checks")`), as the other setup functions do. It is not observed and
  starts no CI (`scenario.rs:6-7`).
- `limits.rs`:
  - `repository`'s `setup` adds `times(limits.contexts,
    size_of::<Failing>())`;
  - `answer`'s `log` (`:239`) becomes the job's log, `name × 6 + 91`:
    `set up <c>\n` is name + 8, four failing lines 4 × (name + 17), and
    `job <c> cancelled\n` name + 15;
  - `answer` gains `job`: `size_of::<Job>() + 3 × size_of::<Step>() +
    name × 7 + 117`: the three names `Set up job` (10), `Run <c>` (name +
    4) and `Complete job` (12), name + 26, and the three slices, `name ×
    6 + 91` as `log` counts them. It goes in the `.max(…)` chain (`:244-260`).
- `temper_fake_forge_protocol::routes::op(config: &Config, request:
  &Request) -> Result<Op, a::Error>`, with no `limits`:
  - `Request::Job { job, .. }` goes to `Read::Job { job }`;
  - `Request::JobLog { job, attempt, step, .. }` goes to `Read::Log {
    job, attempt, step }`.
- `answers::read` maps `a::Answer::Job { job, steps }` to
  `Answer::Job { job: d::Job { id: job.id, run_id: job.run, attempt:
  job.attempt }, steps }`, each `d::Step { number, name, status }` made
  from the fake's step, its `log` left out, its status through `pub fn step(status:
  a::StepStatus) -> d::StepStatus`, beside `status` (`answers.rs:16`).
  `bounded::answer` prices it as `budget.row(0)`, then `budget.row(<the
  step's name's length>)` per step.
- translate: `client::Read::Job { attempt, .. }` maps to
  `forge::Read::Job { job: attempt.job }`; `answer`'s new arm
  `forge::Answer::Job { steps, .. }` answers `byte_answer(asked, <the log
  of the step whose number is forge::WORKFLOW_STEP>)` ("Choices").
Test shape:
- **Fake domain,** `testing/temper-fake-forge-domain/src/tests/actions.rs`,
  on `failing(reruns)` (`:4-13`: repository `ai/temper`, context `ci`,
  CI failing, job 1 of run 1), `Harness::with` (`src/tests.rs:267`),
  `h.ok`/`h.call` (`:325`, `:296`), `read` (`:188`), `setup()` (`:147`),
  `PERSON` (`:74`), `REPOSITORY` (`:80`), `FIRST` (`:84`), `ENGINE`
  (`:73`). Limits: `LIMITS` and `CALM` (`src/tests.rs:19-71`), as at the
  base; the fake ships in no profile. The reference test, in full:

  ```rust
  #[test]
  fn a_failed_job_read_by_id_has_three_steps_failing_at_the_workflow() {
      let (mut h, _) = failing(0);
      assert_eq!(
          h.ok(PERSON, read(Read::Job { job: 1 })),
          Answer::Job {
              job: Job { id: 1, run: 1, index: 0, attempt: 1 },
              steps: Box::new([
                  Step {
                      number: 0,
                      name: copy_of(b"Set up job"),
                      status: StepStatus::Success,
                      log: copy_of(b"set up ci\n"),
                  },
                  Step {
                      number: 1,
                      name: copy_of(b"Run ci"),
                      status: StepStatus::Failure,
                      log: copy_of(b"failing check ci 0\nfailing check ci 1\nfailing check ci 2\nfailing check ci 3\n"),
                  },
                  Step {
                      number: 2,
                      name: copy_of(b"Complete job"),
                      status: StepStatus::Failure,
                      log: copy_of(b"job ci failed\n"),
                  },
              ]),
          }
      );
  }
  ```

  Each step's `log` is the slice `each_step_of_a_failed_jobs_log_is_a_slice_of_its_own`
  reads by `Read::Log` for that step.

  The new tests, each under 20 ms:
  - `a_failed_job_read_by_id_has_three_steps_failing_at_the_workflow`;
  - `a_cancelled_job_has_its_workflow_and_completion_steps_cancelled`
    (after `crate::set_failing(&mut h.domain, REPOSITORY, b"ci",
    Failing::NoStep)`: `Success`, `Cancelled`, `Cancelled`);
  - `each_step_of_a_failed_jobs_log_is_a_slice_of_its_own`: `Some(0)`
    gives `set up ci\n`; `Some(1)` the four `failing check ci <n>\n`
    lines; `Some(2)` `job ci failed\n`; `None` the three in order, 100
    bytes;
  - `a_log_step_past_the_last_is_missing` (`Some(3)`:
    `Err(Error::Missing(What::Job))`);
  - `a_job_read_by_id_answers_its_latest_attempt` (`failing(1000)`: `Job
    { id: 1, run: 1, index: 0, attempt: 2 }`);
  - `a_passing_ci_jobs_read_by_id_is_missing` (`setup()`, whose CI
    passes, pushed and settled as in `failing`).

  Rewritten:
  - `a_job_log_is_read_by_job_id_and_attempt` (`:72-95`) reads
    `Read::Log { job, attempt, step: None }`: the 100-byte log for job
    1, attempt 1, and `Missing(Job)` for attempt 2 and for job 2. The
    `max_bytes` rows go;
  - `a_rerun_context_answers_its_second_attempt` (`:106-110`) reads
    `Read::Log`, and attempt 2 answers the 100-byte log;
  - `src/tests/next.rs:163` reads `Read::Log { job: 0, attempt: 1,
    step: None }`.
- **Fake protocol,** `testing/temper-fake-forge-protocol/src/tests/`:
  - `routes.rs`, `read_rows` (`:14`) goes from 24 rows to 26:
    `(Request::Job { at: at(), job: 8 }, Route::Job,
    Op::Read(Read::Job { job: 8 }))`, and `JobLog`'s rows with `step:
    None` and `step: Some(1)`, each going to `Read::Log { job: 8,
    attempt: 2, step }`;
  - `answers.rs`, one typed test,
    `a_jobs_steps_keep_their_numbers_names_and_statuses`. Its input is
    `a::Answer::Job` with steps `Success`, `Cancelled` and `Failure`,
    each with a non-empty `log`; the expected value is
    `Answer::Job { job: d::Job { … }, steps: … }` written out, its steps
    with no log, as `d::Step` has none;
  - `http.rs`, on `Harness` (`src/tests/fixtures.rs:60-301`: `new` `:76`
    with context `build` failing, `push` `:283`, `ask` `:240`, `status`
    `:205`, `body` `:214`, `domain_calls` `:202`). Limits: `limits()`
    (`:29-51`, `raw_bytes` 4096) and `temper_engine_forge_world::fake_config()`
    (`tests/engine/forge/src/world.rs:562`, CI user 3), as at the base.
    The new kind of test, the job and its step's log over HTTP bytes,
    in full:

    ```rust
    #[test]
    fn a_failed_jobs_steps_and_failed_steps_log_are_served_over_http() {
        let mut h = Harness::new(fixtures::config());
        assert_eq!(h.push(b"work"), 2);
        h.ask(b"GET", b"/api/v1/repos/org/repo/actions/jobs/1", b"");
        assert_eq!(h.status(), 200);
        let job = h.body();
        for step in [
            b"{\"number\":0,\"name\":\"Set up job\",\"status\":\"success\"}".as_slice(),
            b"{\"number\":1,\"name\":\"Run build\",\"status\":\"failure\"}",
            b"{\"number\":2,\"name\":\"Complete job\",\"status\":\"failure\"}",
        ] {
            assert!(bytes::find(&job, step).is_some(), "{step:?}");
        }
        h.ask(b"GET", b"/api/v1/repos/org/repo/actions/jobs/1/logs?attempt=1&step=1", b"");
        assert_eq!(h.status(), 200);
        assert_eq!(&*h.body(), b"failing check build 0\nfailing check build 1\nfailing check build 2\nfailing check build 3\n");
        assert_eq!(h.domain_calls(), 2);
    }
    ```

    Each under 20 ms.
Proof: the fake service over its HTTP bytes: its server on skein's HTTP
machine, its domain behind it (`src/tests/fixtures.rs:60-301`). This is
one layer above the domain's step tests and the routes' table. Run once,
when the increment is done: `cargo nextest run -p
temper-fake-forge-protocol -E
'test(a_failed_jobs_steps_and_failed_steps_log_are_served_over_http)'`.
It shows a client that asks Forgejo's routes for job 1 getting its three
steps, the workflow step `failure` and the completion `failure` too, and
then that step's four lines alone. The test takes under 20 ms; the
command, with the crate's build, under 2 minutes. No higher: the joint
world (`tests/engine/protocol-forge`, the component over the fake's
HTTP layer) sends no `actions/jobs/{id}` and no `step` until temper 04.1
makes the component read them. temper 04.1 connects this to the joint
world.
Don't: store steps or slices per job; read a job or a log of a context
whose latest CI status is not `Failed`; give `Failing` to `api::Checks`;
add a fault for the new reads; add an `Accept-Ranges` header; change `byte_answer` in translate; add
a test to `testing/temper-fake-forge-domain/src/tests.rs`.
Size: at most 230 lines of production code (about 180 in the fake
domain, 50 in its protocol).
Removes: the domain's cut of a log to its end and its `max_bytes`
(`next.rs:167-169,189-191`, `api.rs:109`, `routes.rs:217`'s
`limits.raw_bytes`); `routes::op`'s `limits`; the test
`a_failed_ci_job_has_a_bounded_log_for_its_current_attempt`
(`src/tests/next.rs:166-190`), whose behaviour goes: "one extra byte
signals truncation to the adapter".

### 03.2 Suffix ranges, and an empty log as v17 answers it

Scope: the fake protocol's `range` parses `bytes=-N`, and `raw` answers
it as v17 did. Over an empty body a suffix answers `206` with `bytes
0--1/0`, and a head range answers `200` (forge.md, 9: "with byte
ranges, a suffix range among them"; 3.2: "a range over an empty log is
an empty log"; measurements.md, 3).
Needs: temper 03.1 (a step's slice served), for its Proof only.
Files: `testing/temper-fake-forge-protocol/src/response.rs`, at `:11-20`
(`Select`), `:120` and `:135` (`select`), `:334-345` and `:426-486`
(`range`, `raw`); `src/server.rs:48,80,381,448-449`, the `range` field,
whose `Select::Raw(_)` matches at `:540` are unchanged. Tests:
`src/tests/response.rs:128-165` and new tests there, and one in
`src/tests/http.rs`. Users of `response::range` and `Select::Raw`, found
by `grep -rn -e 'Select::Raw' -e 'response::range' -e 'response::select'
--include=*.rs testing tests`: the two files above. No world names
them.
Shape:
- `temper_fake_forge_protocol::response::Range` (sketched below):
  `From { first: u64, last: Option<u64> }` for `bytes=F-L` and
  `bytes=F-`, and `Suffix(u64)` for `bytes=-N`.
- `pub fn range(value: Option<&[u8]>) -> Result<Option<Range>,
  a::Error>`: `bytes=-N` with N from 1 to `u64::MAX` is `Suffix(N)`.
  `bytes=-0`, `bytes=-` and the forms refused today stay `Refused`.
- `Select::Raw(Option<Range>)`, `select(config, request, range:
  Option<Range>)`, and `Server::range: Result<Option<Range>,
  api::Error>`.
- `fn raw(limits: &Limits, body: &[u8], range: Option<Range>) ->
  Result<Reply, a::Error>`, with T the body's length:
  - `None`: `200`, the body (unchanged);
  - `From` with T = 0: `200`, nothing, no `content-range` (was `416`);
  - `From` with `first ≥ T > 0`: `416` (unchanged);
  - `From` otherwise: `206`, `bytes F-min(L, T − 1)/T` (unchanged);
  - `Suffix(n)` with T = 0: `206`, nothing, `content-range: bytes
    0--1/0`;
  - `Suffix(n)` otherwise: `206`, the last `min(n, T)` bytes,
    `content-range: bytes <T − min(n, T)>-<T − 1>/<T>`;
  - answered bytes above `raw_bytes`: `TooLarge` (unchanged).
Test shape: `testing/temper-fake-forge-protocol/src/tests/response.rs`,
on `config()`, `limits()`, `at()`, `boxed()` (`src/tests/fixtures.rs:6-54`)
and `header_value` (`:148`); the file's `use crate::response::{self,
Select}` (`:2-5`) gains `Range`. The request is `Request::JobLog { at:
at(), job: 1, attempt: 1, step: Some(3) }`, and the bodies are 112 and
0 bytes, v17's measured sizes (measurements.md, 3). Expected values are
v17's, copied by hand. The reference test, in full:

```rust
#[test]
fn a_suffix_range_answers_206_with_the_slices_last_bytes() {
    let request = Request::JobLog { at: at(), job: 1, attempt: 1, step: Some(3) };
    let slice = [[b'x'; 102].as_slice(), b"0123456789"].concat();
    let reply = response::answer(
        &config(),
        &limits(),
        &request,
        &Select::Raw(Some(Range::Suffix(10))),
        &a::Answer::File(boxed(&slice)),
    )
    .expect("suffix");
    assert_eq!((reply.status, &*reply.body), (206, b"0123456789".as_slice()));
    assert_eq!(header_value(&reply.headers), Some(b"bytes 102-111/112".as_slice()));
}
```

Tests, each under 20 ms:
- `a_suffix_range_answers_206_with_the_slices_last_bytes`;
- `a_suffix_longer_than_the_slice_answers_it_all` (`Suffix(100000)`
  over 112 bytes: `206`, 112 bytes, `bytes 0-111/112`);
- `a_suffix_range_over_an_empty_log_names_no_byte` (`Suffix(10)` over
  0 bytes: `206`, nothing, `bytes 0--1/0`);
- `a_head_range_over_an_empty_log_answers_200` (`From { first: 0, last:
  Some(10) }` over 0 bytes: `200`, nothing, no header);
- `range_syntax_is_bounded_and_rejects_reversed_offsets` (`:152-165`),
  its rows typed as `Range`. `bytes=-2` flips to
  `Ok(Some(Range::Suffix(2)))`. Two rows are added, `bytes=-0` and
  `bytes=-`, both `Err(a::Error::Refused)`.
  `raw_ranges_use_206_and_literal_content_range` (`:128-147`) keeps its
  four rows, typed as `Range::From`;
- in `src/tests/http.rs`, the Proof's test:

  ```rust
  #[test]
  fn a_steps_log_end_is_served_by_a_suffix_range() {
      let mut h = Harness::new(fixtures::config());
      assert_eq!(h.push(b"work"), 2);
      h.request(b"GET /api/v1/repos/org/repo/actions/jobs/1/logs?attempt=1&step=1 HTTP/1.1\r\nHost: 127.0.0.1:3000\r\nAuthorization: token 1111111111111111111111111111111111111111\r\nRange: bytes=-10\r\n\r\n");
      h.run();
      assert_eq!(h.status(), 206);
      assert!(bytes::find(&h.head(), b"content-range: bytes 78-87/88\r\n").is_some());
      assert_eq!(&*h.body(), b"k build 3\n");
  }
  ```
Proof: the fake service over its HTTP bytes, once, when the increment is
done: `cargo nextest run -p temper-fake-forge-protocol -E
'test(a_steps_log_end_is_served_by_a_suffix_range)'`. It shows that
`Range: bytes=-10` on a failed job's workflow step answers `206` with
that 88-byte slice's last 10 bytes and `content-range: bytes
78-87/88`, v17's form (measurements.md, 3). The test takes under 20 ms;
the command, with the build, under 2 minutes. No higher, for two
reasons. The component sends no suffix range until temper 04.2
(`Range::Tail`), which connects this in the joint world. And no test
compares the fake forge's answers with the recorded ones: the
`tests/fake-forge` world holds only the memory test (`tests/memory.rs`);
`tests/codecs` reads the conformance's transcripts through
`temper-forge-forgejo`'s client and server, never through the fake;
`tests/engine/protocol-forge/src/machine.rs:311-319` replays them to the
component. The one comparison of a fake with recordings is git's
(`testing/temper-fake-git/tests/transcripts.rs`). So v17's numbers
enter as literals in the tests above.
Don't: answer a multi-range or `bytes=-0`; add `content-range: bytes
*/T` to the `416`; build a comparison of the fake with the
conformance's transcripts (whether to build one is the planner's
question, not this session's); change
a head range over a non-empty body.
Size: at most 80 lines of production code.
Removes: the refusal of `bytes=-N`; the `416` over an empty body.

## Interface sketches

The fake domain's new types, at the base. Pasted into
`testing/temper-fake-forge-domain/src/actions.rs`, with the re-export
and the export below, they compile:

```rust
// testing/temper-fake-forge-domain/src/actions.rs

/// Where a context's failed job fails (decisions.md, 9): set per context by
/// `scenario::set_failing`, `Workflow` until then.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Failing {
    /// The workflow step fails, and `Complete job` with it, as v17's do.
    Workflow,
    /// No step fails: the job was cancelled, its workflow and completion steps `Cancelled`.
    NoStep,
}

/// One step of a job: `temper_forge_forgejo::document::Step`'s fields, and the step's log slice,
/// which the protocol layer serves at the log route, never in the job document.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Step {
    /// 0 `Set up job`, 1 the job's one workflow step, 2 `Complete job`.
    pub number: u32,
    /// `Set up job`, `Run <context>` or `Complete job`.
    pub name: Box<[u8]>,
    /// What the step came to.
    pub status: StepStatus,
    /// The step's slice of the job's log: what `Read::Log` answers for this step.
    pub log: Box<[u8]>,
}

/// A step's outcome: the three of Forgejo's the fake gives.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StepStatus {
    /// Forgejo's `success`.
    Success,
    /// Forgejo's `failure`.
    Failure,
    /// Forgejo's `cancelled`.
    Cancelled,
}

/// The number of a job's one workflow step.
pub const WORKFLOW_STEP: u32 = 1;
```

```rust
// testing/temper-fake-forge-domain/src/api.rs, replacing `:515`
pub use crate::actions::{Failing, Job, Run, Step, StepStatus, WORKFLOW_STEP};

// testing/temper-fake-forge-domain/src/scenario.rs, exported in lib.rs's
// `pub use scenario::{…}` (`:75-78`)
/// Sets where `context`'s failed jobs fail in `repository`.
pub fn set_failing(domain: &mut Domain, repository: &[u8], context: &[u8], failing: crate::api::Failing) {
    todo!()
}
```

These are edits of existing enums. They compile with 03.1's match arms,
not alone:

```rust
// testing/temper-fake-forge-domain/src/api.rs, in `pub enum Read`
    /// The job `job` at its latest attempt, with its steps. Answered by
    /// [`Answer::Job`]; `Missing(Job)` unless CI failed its context.
    Job {
        job: u64,
    },
    /// The attempt's log, read by the protocol layer's log route: the step
    /// `step`'s slice, or the job's three slices in order. Answered by
    /// [`Answer::File`]; `Missing(Job)` as `Job`, for another attempt, or
    /// for a step past the last.
    Log {
        job: u64,
        attempt: u32,
        step: Option<u32>,
    },

// testing/temper-fake-forge-domain/src/api.rs, in `pub enum Answer`
    /// A job at its latest attempt and its three steps, in order, each with
    /// its log slice; the protocol layer encodes it as `server::Answer::Job`, without the logs.
    Job {
        job: Job,
        steps: Box<[Step]>,
    },
```

The fake protocol's new functions. `step` compiles once temper 02.1 has
landed; `Range` and `range` compile at the base, `range` replacing
today's:

```rust
// testing/temper-fake-forge-protocol/src/answers.rs (03.1)
/// A fake step's outcome in Forgejo's words.
#[must_use]
pub fn step(status: a::StepStatus) -> d::StepStatus {
    todo!()
}

// testing/temper-fake-forge-protocol/src/routes.rs (03.1), replacing `op`
/// Translates the document's fields; a bad count or user names a missing object.
pub fn op(config: &Config, request: &Request) -> Result<Op, a::Error> {
    todo!()
}

// testing/temper-fake-forge-protocol/src/response.rs (03.2)
/// A `Range` header's one byte range, as Go's `http.ServeContent` reads it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Range {
    /// `bytes=F-L`, or `bytes=F-` with no last.
    From { first: u64, last: Option<u64> },
    /// `bytes=-N`: the last N bytes, N at least 1.
    Suffix(u64),
}

/// The request's range, none without a header; a form Forgejo's fake does not read is `Refused`.
pub fn range(value: Option<&[u8]>) -> Result<Option<Range>, a::Error> {
    todo!()
}
```

## Questions to raise when reached

- **A job that CI did not fail, read by its id.** forge.md, 13 records
  that "a job no runner took has no steps", and 3.2 says "one never given
  to a runner has no log (`404`, `MissingJob`)". The fake answers a
  pending job (a `silent` context, or one awaiting its verdict) and a
  passed job `404` at both reads ("Choices"). v17 answers such a job with
  its steps, none for a job waiting for a runner, and a passed job's log
  with its lines. Both reach the component as `MissingJob` today. Raise
  it if a story of temper 04 needs the fake to answer a job with no
  steps, or a passed job's log.

## Done when

Each runs once, after the session's last landing, in `~/src/rust/temper`:

1. Both steps landed: `git log origin/main --grep='^Plan: forgejo-17$'
   --format='%(trailers:key=Step,valueonly,separator=)' | grep -c -E
   '^temper-03\.(1|2)$'` prints 2.
2. The fake's log is cut by no bound of the domain: `git grep -n -w
   max_bytes origin/main -- testing/temper-fake-forge-domain
   testing/temper-fake-forge-protocol` prints nothing.
3. The failing step is seeded per context: `git grep -n -e 'pub enum
   Failing' -e 'pub fn set_failing' origin/main --
   testing/temper-fake-forge-domain/src` prints 2 lines.
4. Suffix ranges and the empty log: `git grep -n -F 'Range::Suffix(2)'
   origin/main -- testing/temper-fake-forge-protocol/src/tests/response.rs`
   prints a line, and `git grep -n -F 'bytes 0--1/0' origin/main --
   testing/temper-fake-forge-protocol/src/tests/response.rs` prints a
   line.
5. The session's tests pass, in a worktree of temper at `origin/main`:
   `cargo nextest run -p temper-fake-forge-domain -p
   temper-fake-forge-protocol -E
   'test(/three_steps|cancelled|slice_of_its_own|step_past_the_last|latest_attempt|passing_ci_jobs|keep_their_numbers|served_over_http|suffix|empty_log/)'`
   runs 13 tests, and none fails.
