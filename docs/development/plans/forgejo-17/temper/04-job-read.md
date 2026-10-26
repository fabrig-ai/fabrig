# Session 04: `Read::Job` answers the failed step and its log's end

The forge component reads a failed job as v17 lets it (forge.md, 3.2):
the run, then the job by its id with its steps (`actions/jobs/<job>`),
the failed step chosen as the first whose `status` is `failure`, then that
step's log alone (`step`), or the job's log when no step failed. The
answer names the step as a value of its own, `api::Step { number, name }`
in `Answer::Job`, which the answer's users carry in the same
increment: the client's bound, the component's measure, the store's
`JobAnswer` (in place, the store's version unchanged), the agent's `job`
tool, and the forge client world's translation. 04.1 lands the step with
the log still read from its start; 04.2 reads the log's end, by a suffix
range, and a `200` read through, keeping its last `max_bytes` bytes as it
streams. Each increment is proved in the joint world, the real component
over the fake forge's HTTP layer, and once on the throwaway Forgejo 17.0.0
and runner 13.2.0, through the conformance tool's component mode.

## Read first

- forge.md, 3.2: the `Read::Job` row and the notes "Job attempts" and
  "A failed job's log is its failed step's end"; 3.3, the bullets "A
  string is kept whole, or not at all" and "A malformed answer"; 3.4,
  the table's rows `404` and "an answer past the component's bounds";
  7, the limits `body_bytes_in` and `string_bytes`, and "The worst
  case"; 8, the forge connection's table and the paragraph after it; 9,
  the first and third bullets; 10, "The real Forgejo check"; 13, the
  bullets on a job and its plaintext log.
- domain/forge.md, section 2, "Tools"; 8.4, "A repair" (temper 05
  writes the brief's words; 04.1 only keeps the brief compiling).
- decisions.md 4, 6, 7, 8, 9, 10; code-organization.md, sections 2, 3,
  4, 5 and 6; README, "Rules of this plan".
- findings/component.md, sections 1, 3 and 4; findings/answer.md,
  sections 1, 2 and 3; findings/measurements.md, sections 2 and 3;
  findings/fake-forge-and-brief.md, sections 1, 2, 3 and 6.
- temper 02's file: 02.1's Shape (`d::Step`, `Row::Step`,
  `d::StepStatus`, `Document::Job`, `Request::Job`, `Request::JobLog`'s
  `step`, the job's decoder). temper 03's file: 03.1's Shape (the fake's
  three steps, their logs, `Failing`, the fake domain's job answer) and
  03.2's (the suffix range, the empty range).
- skein's `programming-model.md`: 4.3, 6.2, 6.3 (by number only).
  skein's `testing-strategy.md`: 2.1, 2.5, 4.1 (by number only).

## What the design asks

- **The read** (forge.md, 3.2, `Read::Job`): "`GET R/actions/runs/<run>`;
  `GET R/actions/jobs/<job>`; `GET
  R/actions/jobs/<job>/logs?attempt=<attempt>&step=<number>` with `Range:
  bytes=-<max_bytes>`, `step` the failed step's, absent when no step
  failed | the failed step, its `number` and `name`, none when no step
  failed; the last `max_bytes` bytes of that step's log, or of the job's
  whole log when no step failed; `truncated` when `Content-Range` names
  bytes before them; `MissingJob` when a read answers `404`, the run's
  `commit_sha` is not the attempt's head, or the job's `run_id` or
  `attempt` is not the attempt's".
- **The failed step** (forge.md, 3.2, "A failed job's log is its failed
  step's end"): "Forgejo answers a job (`actions/jobs/<job>`) with its
  steps, those of its latest attempt: number 0 is the runner's set-up
  (`Set up job`), the last its completion (`Complete job`), and the
  workflow's own steps are between. The failed step is the first whose
  `status` is `failure`: the completion step reports `failure` whenever
  the job fails, so it is the failed step only when none before it
  failed. The steps are read as they stream, and only the failed one is
  kept, the others skipped as they pass: a job of any number of steps is
  read, its steps costing time, not memory. Its log alone is read
  (`step`), and of it the end, by a suffix range, since a step tells its
  failure last. A job with no step failed
  (one cancelled, or one whose runner was lost) is read whole, its end
  the same way; one never given to a runner has no log (`404`,
  `MissingJob`). The job's `attempt` is its latest: one other than the
  call's (the job ran again since) is `MissingJob`, as a log Forgejo no
  longer keeps is (`404`). A `200` is read through, its last `max_bytes`
  bytes kept as it streams, within `body_bytes_in` like any body
  (section 7); a range over an empty log is an empty log (13). The log's search (`q`) and its NDJSON form are not read: neither
  answers a range."
- **Names past their cap** (3.3): "a name, a title, a context, a
  description, a link or a path is kept within `string_bytes`; a longer
  one makes the answer malformed"; "but for a job step's name, which
  Forgejo makes of the step's whole script when the workflow names none:
  a longer one is carried empty, and the step is named by its number
  alone, so that a long script never makes a job unreadable".
- **Malformed answers** (3.3): "a document that does not decode, a field
  missing, a number out of range, a commit id of the wrong length, a
  string past its cap, a body past `body_bytes_in`, a log read through
  included"; a malformed answer "fails a read as `InvalidAnswer`".
- **The answer's bound** (3.3): "Raw contents are read within what is
  left of `answer_bytes`: a file's after its path, a job's log after its
  step's name, so the answer never passes it".
- **Bounds** (7): "`head_bytes`, `body_bytes_in`: the largest response
  head and body read (the body is streamed, so this caps time and bytes,
  not memory)". (3.4): "an answer past the component's bounds |
  `TooLarge`".
- **The connection** (8): "A connection the forge marked `Connection:
  close`, or one that read a raw body past `max_bytes`, goes to Closed
  after its answer, not to Idle."
- **The agent's tool** (domain/forge.md, 2, "Tools"): "CI on a head, and
  a failed job's output, its failed step's end".
- **The real check** (forge.md, 10): the conformance tool "drives the
  facts of section 13 through plain requests and the forge component".
- **The plan** (decisions.md 6): "`Answer::Job { attempt, step:
  Option<Step>, log, truncated }`, with `Step { number: u32, name:
  Box<[u8]> }` in the forge client's `api`. `step` is none when no step
  failed. The job's other steps are not kept". (7): "The failed step is
  found by the component ... Nothing above the component reads the
  steps." (8): "The component's range becomes a choice of two, the head
  (`bytes=0-N`, a file's) and the end (`bytes=-N`, a log's); `truncated`
  is `Content-Range`'s first byte above 0, or bytes dropped from a
  `200`." (4): "`JobAnswer` changes in place in the schema `origin/main`
  holds at the increment's landing".

## Choices this plan makes

- **The job's listing leaves `Read::Job`.** Its plan reads the run, then
  `Request::Job`, then the log; `Read::Checks` keeps the listing
  (`runs/<run>/jobs`) for the job at a status's index. The job document's
  `id`, `run_id` and `attempt` are checked against the call's attempt,
  each mismatch `MissingJob`, `id` included as the base's listing check
  includes it (`actions.rs:227-228`).
- **The step is kept as the rows stream** (design question 6): of the
  job's `d::Row::Step`s (temper 02.1), the first whose `status` is
  `d::StepStatus::Failure`, moved into an `api::Step` the plan holds
  until the log answers; the others are dropped as they come, so a job
  of any number of steps is read. A job with no step rows (no runner
  took it) has no failed step: its log is asked without `step`, and
  Forgejo's `404` is `MissingJob`. A step's name past `string_bytes`
  comes empty from temper 02.1's decoder (design question 1): the
  answer carries it so, never refused.
- **04.1 reads the step's log from its start** (`Range: bytes=0-<max>`,
  as the base reads the job's log); between 04.1's landing and 04.2's,
  the answer is the failed step's first `max_bytes` bytes. 04.2 turns it
  to the end.
- **The step's name counts in the answer's bound** as a status's context
  does: the client's `bounds.rs` and the component's `reads/size.rs` add
  its length to the log's. The client keeps no bound per field; the
  component's `string_bytes` caps the name when it is decoded (temper
  02.1's decoder). The log's raw room is `answer_bytes` less the step's
  name, as a file's is less its path (`contents.rs:115-116`; design
  question 2): the answer always fits the client's bound, and the
  client and the agent's tool do not change.
- **The store:** a record `Step { number: u32 name: bytes 1048576 }`
  beside `JobAnswer`, which gains `step: option Step` after `attempt`,
  in the schema file `ls crates/temper-protocol-store/schema/` names;
  the limit `step_name` is `forge.client.answer_bytes`, as
  `status_context` is.
- **The agent's words** (`crates/temper-engine-smith/src/answers.rs`):
  `Job <job> run <run>, failed step <number> <name>:` and a line feed
  (`failed step <number>:` when the name is empty), or `Job <job> run
  <run>, no step failed:` and a line feed; then the
  log. 04.1 keeps `\n[log truncated]` after the log, as today; 04.2,
  where the bytes dropped are the log's first ones, puts
  `[earlier log bytes cut]` and a line feed before the log instead.
- **The brief** (`brief.rs:308`) binds `step: _` in 04.1 and changes no
  word; temper 05 writes its words.
- **The range is a choice of two** in `plan::Exchange`: `Range::Head(n)`
  writes `bytes=0-<n>` and keeps a raw answer's first bytes and one
  more, as today; `Range::Tail(n)` writes `bytes=-<n>` and keeps the
  answer's last `n` bytes as they stream, for a `206` and a `200` alike.
- **A tail read past `body_bytes_in`** stops there and is a malformed
  answer, `InvalidAnswer`, as a JSON body past `body_bytes_in` is (the
  tokenizer's `length`, `limits.rs:288`), as forge.md 3.3 now says
  (design question 5).
- **A tail read that dropped bytes closes its connection** after its
  answer (forge.md, 8, quoted above); one that dropped none goes Idle.
- **`Content-Range` gets an exclusive end,** so that the empty range has
  a value: `header::ContentRange { first, end, total }`, `bytes 0-3/7`
  read as `{ first: 0, end: 4, total: Some(7) }`, and `bytes 0--1/0` as
  `{ first: 0, end: 0, total: Some(0) }`. Other negative last bytes
  stay refused (`bytes 0--1/2` is none, as
  `crates/temper-forge-forgejo/src/tests.rs:327` has it).
- **The conformance's oracle is a plain request.** The component facts
  compare the component's answer with what the tool reads itself from
  the throwaway Forgejo (`f.get`, `f.exchange`): the failing job's
  document for the step, the step's log for its end. No step number,
  name or log byte is written as a literal there, so the facts hold
  whatever workflow temper 02 leaves in `runner.rs`.

## Rules that bind it

- **One type per thing, changed in place with its users**
  (code-organization.md, 5): `Answer::Job` gains `step` and each user
  findings/answer.md, section 1 lists changes in 04.1; `JobAnswer`
  changes in the schema `origin/main` holds; `Exchange`'s `range` changes
  type with its two producers and its consumer in 04.2. No second
  answer, record, exchange or job document. Checked by the build (a
  struct pattern without `..` fails to compile) and the Pulse's `types`
  line.
- **The store's version does not move** (README, "Rules of this plan";
  decisions.md 4): `STORE_VERSION` stays as `origin/main` has it; a
  conflict in `crates/temper-protocol-store/schema/` takes `origin/main`'s
  file, applies 04.1's two lines to it and regenerates. Checked by the
  README's "Done when", item 8.
- **The recording tool runs where a Proof says, once,** and once more after each fix aimed at a cause its output
  names; the third failure with the same symptom is a question. `facts.txt` is written by the tool only, and committed when
  it changed.
- **Files over 800 lines at the base grow by at most 40 lines** in one
  increment (code-organization.md, 6): here
  `crates/temper-engine-domain-forge-client/src/api.rs` (844) and
  `tests/engine/forge/client/tests/simulation.rs` (836); the landing's
  report gives `git diff --numstat origin/main...HEAD -- <file>` for
  each.

## Increments

### 04.1 The job read through its steps; the answer names the step

Scope: `Read::Job` reads the job by its id and its failed step's log, and
answers the step (forge.md, 3.2, the row and the note "A failed job's log
is its failed step's end"; decisions.md 6, 7); `api::Step` and
`Answer::Job`'s `step`, carried by the client's bound, the component's
measure, the store, the agent's `job` tool and the forge client world
(domain/forge.md, 2, "Tools"); the conformance's fact
`component-read-job` names the step.
Needs: nothing beyond the session's start (temper 02.1's job document
and requests; temper 03.1's fake jobs with steps, served over HTTP).
Files: `crates/temper-engine-protocol-forge/src/actions.rs` (371 lines
since temper 02.1: `Mode`, `Stage` :13-26, `Plan` :28-44, the job row
:212-232, the run :270-284, the listing :285-309, `log` :357-370), `src/reads/size.rs:113`;
`crates/temper-engine-domain-forge-client/src/api.rs` (844 lines:
`Answer::Job` :155-156, `JobAttempt` :374-381; `accepts` :511-514
unchanged), `src/bounds.rs:152`; `crates/temper-protocol-store/schema/`
(the file `ls` names, `store-v1.schema:84-85` at the base; shared with
client-api), its generated module and goldens (`src/generated/v1.rs`,
`golden/v1/` at the base), `src/records.rs:142`,
`src/records/answers.rs:76-84, 115-121, 223-226`;
`crates/temper-engine-smith/src/answers.rs:527-537`;
`crates/temper-engine-domain-forge/src/brief.rs:308`;
`tools/forgejo-conformance/src/component/reads.rs:91-110`. Tests:
`crates/temper-engine-protocol-forge/src/tests/contents.rs` (:253-299,
:300-316, :403-486; a shared hotspot: temper 02.2 also edits it, at
:116-144, and may edit it at the same time), `crates/temper-engine-domain-forge-client/src/tests.rs:398-438`,
`crates/temper-protocol-store/src/tests/fixtures.rs:458-468` (790
lines), `src/tests/records.rs:103-131`,
`crates/temper-engine-smith/src/tests.rs`,
`tests/engine/forge/client/src/translate.rs` (`answer`'s arm for
`forge::Answer::Job`, which temper 03.1 adds, and `byte_answer`'s
`Read::Job` arm, `:508-517` since temper 03.1) and
`tests/simulation.rs:495-514` (836 lines),
`tests/engine/protocol-forge/src/joint.rs:50-104`, `src/protocol.rs:46-53`,
`tests/fake_protocol.rs`.
Shape:
- `temper_engine_domain_forge_client::api::Step { number: u32, name:
  Box<[u8]> }`, derives as `Answer`'s (`:129`), beside `JobAttempt`;
  `Answer::Job { attempt: JobAttempt, step: Option<Step>, log:
  Box<[u8]>, truncated: bool }` in place (sketch below). `Step` mirrors
  a `d::Step` (temper 02.1), its `status` dropped; its `name` is empty
  when Forgejo's passed `string_bytes`.
- `actions.rs`: `Stage::Jobs` becomes `Stage::Job`; `Plan.found` goes;
  `Plan.step: Option<api::Step>` comes. `Mode::Job`'s `Run` arm sends
  `r::Request::Job { at, job: attempt.job }` at `Stage::Job`. A new arm
  for `d::Row::Step(step)`, at `Stage::Job` only (else `InvalidAnswer`):
  when `self.step` is none and `step.status` is
  `d::StepStatus::Failure`, `self.step` becomes `api::Step { number,
  name }`, its name moved; any other step is dropped. A new arm for
  `d::Document::Job(job)`, at `Stage::Job` only (else `InvalidAnswer`):
  `job.id`, `job.run_id` or `job.attempt` other than the attempt's is
  `MissingJob`; then `Stage::Log` and `Exchange { request:
  r::Request::JobLog { at, job: attempt.job, attempt: attempt.attempt,
  step: <the step's number, none when no step> }, range: Some(max) }`.
  In `Mode::Job`, `d::Row::Job` and `d::Document::Listed` are
  `InvalidAnswer`. `log()` answers `Answer::Job { attempt, step:
  self.step.take(), log, truncated }`, the bytes and `truncated` as
  today. `actions::Plan::raw_room(&self, shared: &Shared) -> u32` is
  `answer_bytes` less the kept step's name's length, as
  `contents.rs:115-116`'s; `reads.rs:257-262` returns it when the read
  holds an actions plan (design question 2).
- `bounds.rs:152` and `reads/size.rs:113`: `Answer::Job` counts its
  log's length plus its step's name's length, 0 with no step.
- The store: `record Step { number: u32 name: bytes 1048576 }` and
  `record JobAnswer { attempt: JobAttempt step: option Step log: bytes
  1048576 truncated: bool }` (sketch below); regenerated by
  `SKEIN_CODEGEN_WRITE=1 cargo nextest run -p temper-codecs-world
  --test drift`, then the same command without the variable passes,
  once each. `records.rs`: `step_name: forge.client.answer_bytes`.
  `records/answers.rs`: `encode_step(&d::Step) -> w::Step` and
  `decode_step(w::Step) -> d::Step` beside `encode_job_attempt`; the
  `Job` arms carry `step` both ways. `STORE_VERSION` unchanged.
- `answers.rs` (smith): the `Job` arm writes `Job `, the job, ` run `,
  the run, then `, failed step ` and the step's number, then a space and
  its name when the name is not empty, or `, no step failed`; then
  `:\n`, the log, and `\n[log truncated]` when `truncated`.
- `brief.rs:308`: the pattern binds `step: _`; nothing else changes.
- `translate.rs`: the client's `Read::Job { attempt, max_bytes }` maps
  to the fake's `Read::Job { job: attempt.job }`, as temper 03.1 leaves
  it: one fake call per client call, as its callers make
  (`tests/engine/forge/client/src/world.rs:439-451`,
  `tests/engine/domain/src/forge_stand.rs:96-119`,
  `tests/system/src/forge.rs:339-417`, and the same at
  `tests/engine/domain/tests/forge_routes.rs:438,524`,
  `tests/conformance/src/systems.rs:164,311`, and
  `tests/engine/forge/src/world.rs:506,520`, which compiles the same
  `translate.rs` through `#[path]`, `lib.rs:4-5`). `translate::answer`
  is 92 of clippy's 100 `too_many_lines` at the base, 93 after 03.1: the
  arm calls a helper beside `byte_answer`, `fn job_answer(asked:
  &client::Op, job: &forge::Job, steps: &[forge::Step]) ->
  client::Answer`, written as loops, which replaces 03.1's
  `workflow_log` (it goes). The helper first asserts the fake's job is
  the attempt asked (`assert_eq!` on its `attempt`): the read by id
  drops the attempt, no world reads a stale one (every world's checks
  set `reruns: 0`), and the component answers `MissingJob` for one, so
  a world that met one would diverge silently. The arm is
  `forge::Answer::Job { job, steps } => job_answer(asked, &job, &steps)`;
  the helper takes the first of `steps` whose
  `status` is `forge::StepStatus::Failure`, and gives `step:
  Some(client::Step { number, name })` and that step's `log`; with none
  failed (`Failing::NoStep`), `step: None` and the steps' logs joined in
  order (the job's whole log). It then keeps the last `max_bytes` bytes,
  `truncated` when it cut, as `byte_answer`'s `Read::Job` arm
  (`:508-517`) does since temper 03.1; that arm goes, `Read::Job` joins
  `byte_answer`'s panic, and `byte_answer` keeps `Read::File`'s arm.
- `reads.rs` (conformance) `actions()`: after `component-read-job`'s
  call, the tool reads `GET R/actions/jobs/<job>` itself (`f.get`) and
  takes the first step whose `status` is `"failure"`; the fact holds when
  the answer's `attempt` is the call's, its `step` is `Some` with that
  `number` and `name`, and its log holds `TEMPER_CONFORMANCE_FAILED_JOB`,
  as today. The fact's name is unchanged.
Test shape: the component's tests in `src/tests/contents.rs`, on
`fixtures::known()` (`tests/fixtures.rs:463`), `fixtures::ok`
(`:351`), `fixtures::status` (`:373`), `Harness::request_lines`
(`:222`) and the constants `ATTEMPT`, `RUN` (`:302-303`); `JOBS` stays
for the Checks tests (`:317-402`). Add `JOB`, the job document: `id`
10, `run_id` 7, `attempt` 3, `status` `"failure"`, and three `steps`:
0 `Set up job` `success`, 1 `build` `failure`, 2 `Complete job`
`failure`, with any other field temper 02.1's decoder requires, as its
own job literal has it. Limits: the component's fixture
(`tests/fixtures.rs:5-37`, `string_bytes` 128, beside the shipped 512 of
`crates/temper-service/src/forge.rs:120`), the client's `LIMITS`
(`crates/temper-engine-domain-forge-client/src/tests.rs:10-34`,
`answer_bytes` 4096, beside the shipped 65,536), the store's
`fixtures::LIMITS` (`answer_bytes` 4096). The reference test:
```rust
#[test]
fn the_failed_step_is_the_first_whose_status_is_failure() {
    // `format!` is a disallowed macro in step crates: each row names its log's line whole.
    type StepRow = (&'static str, [&'static str; 3], u32, &'static [u8], &'static [u8]);
    let rows: [StepRow; 3] = [
        ("workflow before completion", ["success", "failure", "failure"], 1, b"build",
            b"GET /api/v1/repos/fixture/specimen/actions/jobs/10/logs?attempt=3&step=1 HTTP/1.1"),
        ("set-up", ["failure", "cancelled", "failure"], 0, b"Set up job",
            b"GET /api/v1/repos/fixture/specimen/actions/jobs/10/logs?attempt=3&step=0 HTTP/1.1"),
        ("completion alone", ["success", "success", "failure"], 2, b"Complete job",
            b"GET /api/v1/repos/fixture/specimen/actions/jobs/10/logs?attempt=3&step=2 HTTP/1.1"),
    ];
    for (name, statuses, number, step, log) in rows {
        let mut h = fixtures::known();
        h.call(1, fixtures::REPO, Op::Read(Read::Job { attempt: ATTEMPT, max_bytes: 4 }));
        h.answer(fixtures::FORGE, &fixtures::ok(RUN));
        h.answer(fixtures::FORGE, &fixtures::ok(&job(statuses)));
        h.answer(fixtures::FORGE, b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc");
        h.run(Duration::from_millis(10));
        let lines = h.request_lines(fixtures::FORGE).into_boxed();
        assert_eq!(lines[3].as_ref(), b"GET /api/v1/repos/fixture/specimen/actions/jobs/10 HTTP/1.1", "{name}: job");
        assert_eq!(lines[4].as_ref(), log, "{name}: the step's log");
        let step = Some(Step { number, name: Box::from(step) });
        let expected = Answer::Job { attempt: ATTEMPT, step, log: Box::from(&b"abc"[..]), truncated: false };
        let answer = h.answered(1).expect("job");
        assert_eq!((answer.cost, answer.result.clone()), (3, Ok(expected)), "{name}: answer");
    }
}
```
`job(statuses)` is a helper of the file that writes `JOB` with the three
steps' statuses given, by pushing its pieces into a `skein_lib::List<u8>`
(`format!` is disallowed). Tests, each under 20 ms:
`the_failed_step_is_the_first_whose_status_is_failure` (above);
`a_job_with_no_failed_step_reads_its_log_without_a_step` (statuses
`success`, `cancelled`, `cancelled`: the log line ends
`logs?attempt=3 HTTP/1.1`, `step` none);
`a_job_of_another_id_run_or_attempt_is_missing_job` (replaces
`a_job_without_its_id_and_attempt_is_missing_job`, rows on `JOB`'s
`id` 9, `run_id` 8, `attempt` 2; cost 2);
`a_job_read_answered_404_is_missing_job_at_each_exchange` (its rows
`run`, `job`, `log`, costs 1, 2, 3);
`a_step_name_is_read_up_to_string_bytes_and_empty_past_it` (two rows
at the fixture's `string_bytes` 128: a 128-byte name, at the limit,
answers the step with that name and the log `abc`, cost 3; a 129-byte
name, one past, answers the step with an empty name and the log `abc`,
cost 3; design question 1);
`a_failed_step_after_more_steps_than_inner_is_found` (the fixture's
`inner` 8: a job of twelve steps, 0 to 9 `success`, 10 and 11
`failure`, answers step 10, its log line ending
`logs?attempt=3&step=10 HTTP/1.1`, cost 3; design question 6);
`a_jobs_log_room_is_answer_bytes_less_its_steps_name` (`max_bytes`
the fixture's client `answer_bytes`, step 1 `build`, and a `200` log of
`answer_bytes` bytes: the log kept is `answer_bytes` less 5 bytes,
`truncated`, and the answer is `Ok`, within the client's bound; design
question 2);
`contents_sizes_agree_with_the_clients_measure` gains the row `"job with
its step"` (log `abc`, step 1 `build`: 8);
`a_jobs_verified_log_uses_range_and_content_range_reports_more` and
`a_job_log_at_another_head_is_missing_job` keep their names, on `JOB`,
their expected answers with step 1 `build`; the first's request check
(`src/tests/contents.rs:472` on origin/main df37f0329) reads
`logs?attempt=3&step=1 HTTP/1.1` where it read `logs?attempt=3
HTTP/1.1`. In the client's `tests.rs`:
`job_logs_are_bounded_and_pinned_to_the_failing_attempt` gives its first
answer step 1 `build`; new
`a_job_answers_step_name_counts_against_answer_bytes` (`max_bytes` 4096:
a 4095-byte log and a 1-byte name answer; a 4096-byte log and a 1-byte
name are `TooLarge`). In the store: `answers_without_item_pieces()`
becomes 19 answers, its job `step: Some(Step { number: 21, name:
Box::new([22]) })` and a second job with `step: None`, both through
`each_named_call_payload_round_trips_unchanged`; new
`a_job_steps_name_past_its_limit_fails_to_decode` (a 4097-byte name,
`Undecodable`, beside `an_answer_piece_past_its_limit_fails_to_decode`);
`the_application_sizes_are_the_largest_key_and_row` keeps 35,403 (the
job answer's worst case, about 8.3 KB at `answer_bytes` 4096, stays
below the change row). In smith's `tests.rs`:
```rust
#[test]
fn a_job_answer_names_its_failed_step_before_its_log() {
    use temper_engine_domain_forge_client::api;
    let attempt = api::JobAttempt { head: [1; 32], run: 7, job: 8, attempt: 2 };
    let step = Some(api::Step { number: 1, name: Box::from(&b"build"[..]) });
    let read = api::Answer::Job { attempt, step, log: Box::from(&b"failing check build 3\n"[..]), truncated: false };
    let rendered = answer(&engine::CallAnswer::ForgeRead(Box::new(Ok(read))));
    assert_eq!(rendered.text(), b"Job 8 run 7, failed step 1 build:\nfailing check build 3\n", "step, then log");
    assert!(!rendered.error(), "a read's answer is no error");
}
```
`a_job_answer_with_no_failed_step_says_none_failed` (`Job 8 run 7,
no step failed:\n` and the log), and
`a_job_answer_with_an_empty_step_name_names_its_number_alone` (`Job 8
run 7, failed step 1:\n` and the log; design question 1). In the forge client world:
`bounded_job_bytes_translate_to_a_truncated_attempt_log`
(`simulation.rs:495-514`) passes `translate::answer` a
`temper_fake_forge_domain::api::Answer::Job` in place of its
`Answer::File(b"error")`: steps 0 `Set up job` `Success` (log `set
up\n`), 1 `Run build` `Failure` (log `error`), 2 `Complete job`
`Failure` (log `job failed\n`), for `Read::Job { attempt, max_bytes: 4
}`; it expects `Answer::Job { attempt, step: Some(Step { number: 1,
name: Box::from(&b"Run build"[..]) }), log: Box::from(&b"rror"[..]),
truncated: true }`. New beside it,
`a_job_with_no_failed_step_translates_to_its_whole_logs_end` (steps
`Success`, `Cancelled`, `Cancelled` with logs `ab`, `cd`, `ef`,
`max_bytes: 4`: `step: None`, log `cdef`, `truncated: true`); under
20 ms each, and `simulation.rs` grows by at most 40 lines net (Rules).
In the joint
world, `Joint::with` takes its repository's `raw::Checks` as a seventh
parameter, the literal at `joint.rs:70-78` moved into `fn checks(passes:
u32) -> raw::Checks`; its callers, `Joint::cutting` (`joint.rs:47`,
which `Joint::new` calls) and `Protocol::cutting` (`protocol.rs:46`),
pass `checks(1000)` as today; `Joint::failing(seed:
u64) -> Joint` passes `checks(0)`, on temper 03.1's default
`Failing::Workflow`, and its other arguments as `Joint::cutting(seed, 0,
no faults)` gives them (settings with `user = 1`, `fixtures::shared()`).
A `match` on the failed step with a `None` arm is written `if let ...
else` (clippy's `single_match_else`). The joint test, in `tests/fake_protocol.rs`, under
0.5 s:
```rust
#[test]
fn a_failed_job_read_names_its_failed_step_and_reads_its_log() {
    let mut joint = Joint::failing(5);
    let pushed = joint.push(b"work", b"change");
    joint.run_for(Duration::from_secs(1));
    let head = temper_engine_forge_world::translate::commit(pushed);
    joint.call(1, REPO, Op::Read(Read::Checks { commit: head }));
    joint.run_for(Duration::from_secs(1));
    let Some(Ok(Answer::Checks(statuses))) = joint.answered(1).map(|a| &a.result) else {
        panic!("checks: {:?}", joint.answered(1))
    };
    let attempt = statuses[0].job.expect("the failed status links its job");
    joint.call(2, REPO, Op::Read(Read::Job { attempt, max_bytes: 4096 }));
    joint.run_for(Duration::from_secs(1));
    let answered = joint.answered(2).expect("job answer");
    assert_eq!(answered.cost, 3, "run, job, the step's log");
    let step = Some(Step { number: 1, name: Box::from(BUILD_STEP_NAME) });
    let expected = Answer::Job { attempt, step, log: Box::from(BUILD_STEP_LOG), truncated: false };
    assert_eq!(answered.result, Ok(expected));
}
```
`BUILD_STEP_NAME` and `BUILD_STEP_LOG` are constants of the test file:
the fake's workflow step's name and its log slice for the context
`build`, copied by hand from temper 03.1's
`testing/temper-fake-forge-domain` code.
Proof: two, each once.
- Joint layer: `cargo nextest run -p
  temper-engine-protocol-forge-world -E
  'test(a_failed_job_read_names_its_failed_step_and_reads_its_log)'`:
  the real component, over the fake forge's HTTP, reads a failed CI's
  job by its id, picks the workflow step, and answers its name with its
  log's bytes; under 0.5 s.
- The real Forgejo, after the increment's commit:
  `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  cargo run -p forgejo-conformance -- --scratch
  target/forgejo-component --out tools/forgejo-conformance --component`
  (about 60 s; 24 s in the spike, findings/tools.md, 2). Bound: 10
  minutes for the task, the build included, as 01.1's (`01-pin.md`,
  01.1's Proof, item 1). What counts:
  `grep -E '^(fails|unchecked) component-'
  tools/forgejo-conformance/facts.txt` prints nothing, and `grep -c
  '^holds component-' tools/forgejo-conformance/facts.txt` prints 39,
  as before the run: 04.1 changes `component-read-job` in place and
  adds no fact. Nothing else is wanted. As in 01.1 (`01-pin.md`, 01.1's
  Test shape and Don't): a literal a re-masking moved is read from the
  new transcripts by hand, and a fact that changes outcome is a
  question, with its `seen` text. The component mode writes only
  `facts.txt`'s `component-` block
  (`tools/forgejo-conformance/src/component.rs:16-23`), and no
  transcript.
Don't: read the job's listing in `Read::Job`; keep a job's other steps
(decisions.md 6); read a step above the component (decisions.md 7);
change the brief's words (temper 05); change the range (04.2); add a
per-field bound to the client, or change its admission of `max_bytes`
(design question 2: the component's raw room keeps the answer within
the bound); change `read_job`'s description or schema
(`crates/temper-engine-smith/src/calls.rs:35`); change the conformance's
workflow (`runner.rs:68`) or record transcripts.
Size: at most 110 lines: 10 more than planned, for the log's raw room
(design question 2) and an empty name's words (question 1).
Removes: `actions.rs`'s `found` and its jobs-listing stage in
`Mode::Job`.

### 04.2 The log's end

Scope: the log's end by a suffix range, and a `200` read through keeping
its last `max_bytes` bytes as it streams (forge.md, 3.2; decisions.md
8); `truncated` from `Content-Range`'s first byte, or bytes dropped; a
range over an empty log is an empty log (forge.md, 3.2 and 13); a file's
read keeps its head; a new component fact on v17.
Needs: temper 03.2: the fake protocol answers `bytes=-<n>` with `206`
and the slice's last bytes, and `Content-Range: bytes 0--1/0` over an
empty log.
Files: `crates/temper-engine-protocol-forge/src/plan.rs:5-17`,
`src/connection.rs` (686 lines: `Body` :38-49, `Receiving` :84-89,
`start` :156-220, the raw arm :354-375, `Done` :378-396, `response`
:456-468, `raw_demand` :476-485, `decoder` :632-637), `src/grow.rs`,
`src/actions.rs` (the head :136-146, the log's exchange, `log`
:357-370), `src/contents.rs:102-105` (the consumer,
`src/component/calls.rs:219`, passes `request.range` unchanged);
`crates/temper-forge-forgejo/src/header.rs:4-38`;
`crates/temper-engine-smith/src/answers.rs` (the `Job` arm);
`tools/forgejo-conformance/src/component/reads.rs:91-110`. Tests:
`crates/temper-engine-protocol-forge/src/tests/connection.rs` (:217
and :285 pass `Some(Range::Head(n))`; the `None`s stay; :15-25, `Grow`'s
test, passes no range),
`src/tests/contents.rs`, `crates/temper-forge-forgejo/src/tests.rs:316-331`,
`crates/temper-engine-smith/src/tests.rs`,
`tests/engine/protocol-forge/tests/fake_protocol.rs`.
Shape:
- `plan::Range { Head(u32), Tail(u32) }` and `plan::Exchange { request,
  range: Option<Range> }` (sketch below). `contents.rs`'s file read
  sends `Range::Head(max)`; `actions.rs`'s log sends `Range::Tail(max)`.
- `Connection::start(&mut self, env, forge, request, range:
  Option<Range>)`: `Head(n)` writes `Range: bytes=0-<n>` and reads as
  today; `Tail(n)` writes `Range: bytes=-<n>` and the response's raw
  body goes to `Receiving::Tail { tail: grow::Tail, read: u64,
  remaining: Option<u64> }`, the tail holding at most `n.min(raw_cap)`
  bytes. Its arm writes each piece into the tail and demands the next
  as `raw_demand` does for `Receiving::Raw` (`chunk`, at most what
  `remaining` says), whether or not the tail is full; past
  `body_bytes_in` bytes `read`, it answers
  `Heard::Done(Body::Malformed(decode::Malformed::Bound { field: b"log"
  }))` and goes Closed. At `http::Event::Done`, `Body::Raw { bytes:
  tail.finish(), more: tail.dropped() }`, and Closed when bytes were
  dropped (forge.md, 8), else Idle. `Body::Raw`'s doc: `more` when bytes
  followed the cap (a head) or came before those kept (a tail).
- `grow::Tail` (sketch below): `new(first, max)`, `write(piece)`,
  `dropped()`, `finish()`; it grows as `Grow` does until it holds `max`
  bytes, then each new byte takes the oldest one's place. The `raw`
  term of `limits::worst_case` (`limits.rs:321`, twice `answer_bytes`)
  is not changed: it prices a raw body of at most `raw_cap` bytes
  (`connection.rs:153`, `answer_bytes`, or less once `raw_room` sets it,
  `:598-601`) and its doubling overlap, and the tail's `max` is
  `n.min(raw_cap)`. No test at the base reads a tail's size, so `Tail`
  gets the `#[cfg(test)] pub(crate) fn capacity(&self) -> u32` that
  `Grow` has (`grow.rs:55-60`), and two tests hold the bound (Test
  shape): `the_tail_keeps_the_last_max_bytes_across_writes` and
  `a_tail_read_keeps_at_most_the_raw_room`.
- `actions.rs`: at `Stage::Log`, the head sets `truncated` when
  `Content-Range`'s `first` is above 0; `log()` answers the bytes as the
  connection kept them, `truncated` when `more` or `Content-Range` said
  so.
- `header::ContentRange { first, end, total }` and `content_range`
  (sketch below): `end` exclusive; `bytes 0--1/0` is `{ first: 0, end:
  0, total: Some(0) }`; the other forms the base refuses stay refused.
- `answers.rs` (smith): when `truncated`, `[earlier log bytes cut]\n`
  goes after the header line and before the log; the trailing `\n[log
  truncated]` goes.
- `reads.rs` (conformance): a new fact,
  `component-read-job-answers-the-failed-steps-end`, after
  `component-read-job`: `Read::Job { attempt, max_bytes: 16 }`; the tool
  reads the step's log itself, `f.exchange("GET",
  "<R>/actions/jobs/<job>/logs?attempt=<attempt>&step=<number>", None,
  Some(&f.writer.token), None)`, the number from the document 04.1's fact
  read; the fact holds when the answer's `step` has that number, its log
  is 16 bytes, the tool's body ends with them, and `truncated` is true.
  Without `TEMPER_FORGEJO_RUNNER` it is `unchecked` beside the other two
  (`reads.rs:93`).
Test shape: `src/tests/connection.rs` on `fixtures::ConnectionHarness`
(`tests/fixtures.rs:402-462`), `fixtures::ok`, and the helpers of
`src/tests/connection.rs`, `contains` (`:229`) and `raw_request()`
(`:237`); `src/tests/contents.rs` on 04.1's `JOB`. Limits:
the component's fixture; one test sets `h.env.limits.body_bytes_in` to 8,
beside the fixture's 1,048,576, so that a 9-byte body passes it. The
reference test:
```rust
#[test]
fn a_tail_read_keeps_a_200s_last_bytes_and_closes_when_some_were_dropped() {
    let rows: [(&str, &[u8], &[u8], bool, Phase); 3] = [
        ("short", b"ab", b"ab", false, Phase::Idle),
        ("max", b"abc", b"abc", false, Phase::Idle),
        ("longer", b"abcdef", b"def", true, Phase::Closed),
    ];
    for (name, response, expected, more, phase) in rows {
        let mut h = fixtures::ConnectionHarness::new();
        h.connection.start(&h.env, &h.forge, raw_request(), Some(Range::Tail(3))).expect("valid raw request");
        let heard = h.answer(&fixtures::ok(response));
        let kept = Heard::Done(crate::connection::Body::Raw { bytes: Box::from(expected), more });
        assert_eq!(heard.last(), Some(&kept), "{name}: the last bytes");
        assert_eq!(h.connection.phase, phase, "{name}: connection state");
        assert!(contains(&h.sent, b"Range: bytes=-3\r\n"), "{name}: suffix range");
    }
}
```
Tests, each under 20 ms: the reference test;
`the_tail_keeps_the_last_max_bytes_across_writes` (`Tail::new(2, 4)`,
writes `ab`, `cde`, `f`: `cdef`, dropped; writes `ab` alone: `ab`, not
dropped; the bound at its limit and one past, as
`the_capped_buffer_doubles_and_refuses_a_write_past_its_cap`,
`src/tests/connection.rs:14-24`, holds `Grow`'s: `capacity()` is 2
after `ab`, 4 after `cde`, at `max`, and 4 after `f`);
`a_tail_read_keeps_at_most_the_raw_room` (the margin's floor: after
`h.connection.raw_room(8)`, `Some(Range::Tail(16))` and a 20-byte `200`
`abcdefghijklmnopqrst` answer `Body::Raw { bytes: mnopqrst, more: true
}`, Closed: the tail's `max` is `n.min(raw_cap)`, never above the
`raw_cap` the `raw` term doubles);
`a_chunked_tail_read_keeps_the_last_bytes_across_pieces`
(chunks `ab`, `cde`, `Tail(3)`: `cde`, `more`, Closed);
`a_tail_read_past_body_bytes_in_is_malformed_and_closes`;
`a_job_log_asks_a_suffix_range_and_bytes_before_it_are_truncated`
(replaces `a_jobs_verified_log_uses_range_and_content_range_reports_more`;
`Range: bytes=-4`; rows `206` `bytes 0-2/3` `abc`: not truncated, `206`
`bytes 16-19/20` `abcd`: truncated, `200` `abcdef`: `cdef`, truncated);
`an_empty_log_answered_with_an_empty_range_is_an_empty_log` (`206`,
`Content-Length: 0`, `Content-Range: bytes 0--1/0`: log empty, not
truncated, cost 3); in `crates/temper-forge-forgejo/src/tests.rs`,
`an_empty_range_over_an_empty_log_reads_as_empty` (`bytes 0--1/0`:
`Some(ContentRange { first: 0, end: 0, total: Some(0) })`; `bytes
0--1/2` and `bytes 1--1/0`: none), and
`a_content_range_reads_its_first_last_and_total` renamed
`a_content_range_reads_its_first_end_and_total`, its literals' `last: 3`
and `last: 6` as `end: 4` and `end: 7`;
`raw_reads_keep_the_range_and_refuse_bytes_past_remaining_answer_room`
keeps its rows with `Range::Head`. Smith's
`a_job_answer_names_its_failed_step_before_its_log` takes `truncated:
true` and expects `Job 8 run 7, failed step 1 build:\n[earlier log bytes
cut]\nfailing check build 3\n`. In the joint world, under 0.5 s:
`a_failed_jobs_log_end_comes_through_a_suffix_range`, as 04.1's joint
test with `max_bytes: 8`, expecting the last 8 bytes of
`BUILD_STEP_LOG`, written as a literal, and `truncated: true`.
Proof: two, each once.
- The real Forgejo, after the increment's commit: the conformance
  command of 04.1's Proof, with its bound (10 minutes, the build
  included) and its rule on moved literals and facts that change
  outcome. What counts: `grep -E '^(fails|unchecked)
  component-' tools/forgejo-conformance/facts.txt` prints nothing, `grep
  -c '^holds component-' tools/forgejo-conformance/facts.txt` prints 40,
  one more than the 39 of 04.1's run, and `grep -x 'holds
  component-read-job-answers-the-failed-steps-end'
  tools/forgejo-conformance/facts.txt` prints the line. The new
  `facts.txt` is committed with the increment.
- Joint layer: `cargo nextest run -p
  temper-engine-protocol-forge-world -E
  'test(a_failed_jobs_log_end_comes_through_a_suffix_range)'`: the real
  component asks the fake's HTTP layer for a suffix range and answers the
  failed step's last 8 bytes, `truncated`; under 0.5 s.
Don't: change a file's read (`Range::Head`, `bytes=0-<n>`, its first
bytes kept); send `Range` on a request other than a log or a file; keep
more than `max` bytes of a log at once; read a log's `q`, `qi` or NDJSON
form; change the fake forge.
Size: at most 140 lines.
Removes: `actions.rs`'s `truncated` from `total > last + 1`, and the
head cut of the log in `log()`.

## Interface sketches

Who owns what: temper 04.1 owns `api::Step` and `Answer::Job`'s `step`
(the component makes them; the client bounds them; the store keeps them;
smith and temper 05 render them). temper 04.2 owns `plan::Range` (a
plan asks for it; the connection writes its header and keeps the bytes
it names) and `grow::Tail`. A call still ends with one `Answered`.

One file, compiling as a module of a std crate that links the workspace's
crates, with `Box` from the prelude:

```rust
//! Sketches of temper 04: the job answer's step (04.1), the log's end (04.2).
//! Each module stands for the crate module of the same path.
#![expect(unused_variables, reason = "a sketch's bodies are unreachable; its parameters are its signature")]
#![expect(dead_code, reason = "a sketch's private fields and crate items are its shape")]

pub mod temper_engine_domain_forge_client {
    // temper_engine_domain_forge_client::api (04.1): `Step` beside `JobAttempt`.
    pub mod api {
        use ::temper_engine_domain_forge_client::api::JobAttempt;

        /// A job attempt's failed step: the first of its steps whose `status` is `failure`
        /// (forge.md, 3.2). The component finds it; nothing above the component reads steps.
        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub struct Step {
            /// Forgejo's `number`: 0 is `Set up job`, the last `Complete job`.
            pub number: u32,
            /// Forgejo's `name` as Forgejo wrote it, within the component's `string_bytes`.
            pub name: Box<[u8]>,
        }

        /// `Answer`, its `Job` variant changed in place (`api.rs:155-156`); the others as at the base.
        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub enum Answer {
            /// One pinned attempt's failed step, none when no step failed, and the last bytes of
            /// that step's log (the job's log when none failed) within `max_bytes`; `truncated`
            /// when bytes came before them. Until 04.2, the log's first bytes.
            Job { attempt: JobAttempt, step: Option<Step>, log: Box<[u8]>, truncated: bool },
        }
    }
}

pub mod temper_engine_protocol_forge {
    // temper_engine_protocol_forge::plan (04.2)
    pub mod plan {
        use ::temper_forge_forgejo::request;

        /// The bytes a raw answer is asked for (forge.md, 3.2); a plan chooses, the connection writes it.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub(crate) enum Range {
            /// `bytes=0-<n>`: a file's first `n` bytes and one more, which shows that more came.
            Head(u32),
            /// `bytes=-<n>`: a log's last `n` bytes; a `200` is read through, its last `n` kept.
            Tail(u32),
        }

        /// One exchange of a call's plan.
        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub(crate) struct Exchange {
            pub request: request::Request,
            pub range: Option<Range>,
        }
    }

    // temper_engine_protocol_forge::grow (04.2): `Tail` beside `Grow`.
    pub mod grow {
        use ::skein_lib::List;

        /// The last `max` bytes of a body of unknown length, kept as it streams (forge.md, 3.2,
        /// "A `200` is read through, its last `max_bytes` bytes kept as it streams").
        #[derive(Debug)]
        pub struct Tail {
            bytes: List<u8>,
            start: u32,
            max: u32,
            dropped: bool,
        }
        impl Tail {
            /// Starts at `first` bytes (at most `max`), growing as `Grow` does up to `max`.
            #[must_use]
            pub fn new(first: u32, max: u32) -> Tail {
                unreachable!("sketch")
            }
            /// Appends `piece`; once `max` bytes are held, each new byte takes the oldest one's place.
            pub fn write(&mut self, piece: &[u8]) {
                unreachable!("sketch")
            }
            /// Whether a byte was dropped: bytes came before those kept.
            #[must_use]
            pub fn dropped(&self) -> bool {
                unreachable!("sketch")
            }
            /// The bytes kept, oldest first, in a box of their length.
            #[must_use]
            pub fn finish(self) -> Box<[u8]> {
                unreachable!("sketch")
            }
        }
    }
}

pub mod temper_forge_forgejo {
    // temper_forge_forgejo::header (04.2)
    pub mod header {
        /// A `206`'s `Content-Range`; `bytes 0--1/0`, Forgejo's range over an empty log, is the
        /// empty range at 0 (forge.md, 13).
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct ContentRange {
            /// The first byte answered.
            pub first: u64,
            /// One past the last byte answered; `first` for the empty range.
            pub end: u64,
            /// The representation's length; none for `*`.
            pub total: Option<u64>,
        }
        /// `bytes F-L/T` with `F <= L < T`, `bytes F-L/*`, or `bytes 0--1/0`; anything else none.
        #[must_use]
        pub fn content_range(value: &[u8]) -> Option<ContentRange> {
            unreachable!("sketch")
        }
    }
}
```

The store's schema (04.1), in the file `ls
crates/temper-protocol-store/schema/` names, `JobAnswer`'s line replaced
and `Step`'s added before it:

```
record Step { number: u32 name: bytes 1048576 }
record JobAnswer { attempt: JobAttempt step: option Step log: bytes 1048576 truncated: bool }
```

## Questions to raise when reached

None. The designer answered the three found while planning, and a
fourth that reaches this session, on 2026-10-16 (forge.md 3.2 and 3.3):

1. **A step's name past `string_bytes`** (design question 1): carried
   empty, never refused; the step is named by its number (Choices, "The
   step is kept as the rows stream"; 04.1's tests).
2. **`max_bytes` up to `answer_bytes`, plus the step's name** (design
   question 2): the log's raw room is `answer_bytes` less the step's
   name, as a file's is less its path; the client does not change
   (Choices, "The step's name counts in the answer's bound").
3. **A `200` log past `body_bytes_in`** (design question 5): the bound
   holds for a log read through as for any body; past it the answer is
   malformed, `InvalidAnswer`, the connection closed, as 04.2's
   `a_tail_read_past_body_bytes_in_is_malformed_and_closes` holds.
4. **A job of more steps than `inner`** (design question 6): read
   whole, its steps as rows, the failed one kept (Choices, "The step is
   kept as the rows stream").

## Done when

Each command runs once, after 04.2 has landed, in `~/src/rust/temper`
reading `origin/main`.

1. `git log origin/main --grep='^Plan: forgejo-17$'
   --format='%(trailers:key=Step,valueonly,separator=)' | grep -c -E
   '^temper-04\.[12]$'` prints 2.
2. `git grep -n -F 'step: Option<Step>' origin/main --
   crates/temper-engine-domain-forge-client/src/api.rs` prints a line.
3. `git grep -n -F 'step: option Step' origin/main --
   crates/temper-protocol-store/schema/` prints a line, and `git log
   origin/main --all-match --grep='^Plan: forgejo-17$'
   --grep='^Step: temper-04' -p -- crates/temper-protocol-store/src/lib.rs
   | grep -E '^[-+].*STORE_VERSION'` prints nothing.
4. `git grep -n -F 'Range::Tail' origin/main --
   crates/temper-engine-protocol-forge/src/actions.rs` prints a line, and
   `git grep -n -F 'Range::Head' origin/main --
   crates/temper-engine-protocol-forge/src/contents.rs` prints a line.
5. `git show origin/main:tools/forgejo-conformance/facts.txt | grep -c -x
   -E 'holds component-read-job(-answers-the-failed-steps-end)?'` prints
   2, and `git show origin/main:tools/forgejo-conformance/facts.txt |
   grep -E '^(fails|unchecked) component-'` prints nothing.
