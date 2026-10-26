# Session 02: v17's documents: a job with its steps, a step's log by a suffix range, the comparisons' flags, and their facts

`temper-forge-forgejo` learns what Forgejo 17.0 adds that the design
adopts, on both sides: a job read by its id with its latest attempt's
steps (`GET R/actions/jobs/<job>`), a job's log narrowed to one step
(`&step=`), and the comparisons' `files` and `verification` flags. The
recorder of `temper-throwaway-forgejo` sends a suffix range;
`tools/forgejo-conformance` records the captures forge.md, section 13
lists "to check" and writes their facts; the codecs world decodes each
capture to a literal document or request. The component sends the
comparisons' flags (02.2) and nothing else new: it reads a job's steps
in temper 04, and the fake forge serves them in temper 03. 02.1 comes
first, since temper 03 and 04 wait on it; 02.2 lands after it (its
captures follow 02.1's in the run). That a job waiting for a runner
has no steps is not checked here: `later.md` (decisions.md, 13).

## Read first

- temper's `docs/design/forge.md`: 3.2 (the rows `Read::Compare`,
  `Read::Job`, `Write::OpenPull`; the notes "A failed job's log is its
  failed step's end" and "Comparisons ask for what they keep"); 3.3 (the
  bullets "A string is kept whole, or not at all" and "A malformed
  answer"); 9 (the first bullet, as far as it names
  `temper-forge-forgejo`'s server side); 10 (the bullets "Step tests" for
  `temper-forge-forgejo`, "Transcripts are format-faithful" and "The real
  Forgejo check"); 13 (the API description's bullets on a job and a
  job's log, and "To check on v17.0", its first three bullets).
- This plan: README, "Rules of this plan" and "Done when" item 4;
  code-organization.md, sections 2, 3, 5 and 6; decisions.md, 7 and 10;
  `findings/component.md`, 1, 2 and 4; `findings/tools.md`, 2 and 3;
  `findings/measurements.md`, 2 and 3; `findings/answer.md`, 3.
- The raw captures behind measurements.md (not versioned):
  `~/src/rust/tmp/temper/forgejo-17-plan/facts-probe/` (`job`,
  `failure-suffix-10`, `silent-suffix-10`, `compare-files-false`,
  `compare-reverse-both-false`); the spike's
  `~/src/rust/tmp/temper/forgejo-17-spike/temper/tools/forgejo-conformance/transcripts/`
  (`v17-job`, `v17-log-step`, `v17-log-setup`, `v17-log-cleanup`,
  `v17-log-past-last-step`, `v17-job-rerun-pending-job`) and its
  `src/runner.rs:382-404` (the runner held with `SIGSTOP`); Forgejo
  v17's API description, `~/src/rust/tmp/forgejo-17-swagger.json`
  (`ActionRunJob`, `ActionRunJobStep`, `repoGetActionJob`,
  `repoGetActionJobLogs`).
- skein's `docs/foundation/programming-model.md`: 6.3, 8. skein's
  `docs/foundation/testing-strategy.md`: 2.1, 4.1, 8.
- temper's `docs/development/workflow.md`, 1.

## What the design asks

- **The job** (forge.md 3.2, "A failed job's log is its failed step's
  end"): "Forgejo answers a job (`actions/jobs/<job>`) with its steps,
  those of its latest attempt: number 0 is the runner's set-up (`Set up
  job`), the last its completion (`Complete job`), and the workflow's own
  steps are between. The failed step is the first whose `status` is
  `failure` [...]. The steps are read as they stream, and only the
  failed one is kept, the others skipped as they pass: a job of any
  number of steps is read, its steps costing time, not memory. Its log
  alone is read (`step`), and of it the end, by a suffix range" (the
  streaming: design question 6, answered 2026-10-16). The row `Read::Job` sends "`GET R/actions/jobs/<job>`;
  `GET R/actions/jobs/<job>/logs?attempt=<attempt>&step=<number>` with
  `Range: bytes=-<max_bytes>`, `step` the failed step's, absent when no
  step failed". This session puts those requests and the job document in
  place on both sides; the component's use of them is temper 04's.
- **The comparisons** (3.2): `Read::Compare` sends "`GET
  R/compare/<before>...<after>?verification=false`; `GET
  R/compare/<after>...<before>?files=false&verification=false`";
  `Write::OpenPull` "`GET R/compare/<base's commit.id>...<head's
  commit.id>?files=false&verification=false`, of which only
  `total_commits` is kept". The note: "Every comparison sends
  `verification=false`, since no commit's signature is read; one read
  only for its `total_commits` (`OpenPull`'s, and `Compare`'s second)
  also sends `files=false`, so Forgejo lists no commit's files. [...]
  `Compare`'s first comparison, which keeps them, never sends it. The
  answer is decoded as before: what the call does not keep is skipped as
  it streams."
- **Strings** (3.3): "a name, a title, a context, a description, a link
  or a path is kept within `string_bytes`; a longer one makes the answer
  malformed"; "but for a job step's name, which Forgejo makes of the
  step's whole script when the workflow names none: a longer one is
  carried empty, and the step is named by its number alone, so that a
  long script never makes a job unreadable" (design question 1); a
  malformed answer "fails a read as `InvalidAnswer`".
- **Both sides** (10): "Step tests for `temper-forge-forgejo`: each
  decoder against transcripts of a throwaway Forgejo v17.0 [...]; golden
  bytes for each encoder [...]. Server encoders meet the client decoders,
  and the client encoders meet the server decoders." The fake forge (9)
  serves "a job by its id (`actions/jobs/{id}`) with its steps, its log
  read by attempt and step (`actions/jobs/{id}/logs?attempt&step`) with
  byte ranges, a suffix range among them" "from the server side of
  `temper-forge-forgejo`": this session writes that server side; temper
  03 serves it.
- **To check** (13), "each before the component relies on it":
  - "that a job's steps are numbered from 0, `Set up job` first and
    `Complete job` last; that a failed workflow step is `failure`, and
    the completion step `failure` with it; that a job waiting for a
    runner has no steps;"
  - "that a step's log (`step`) holds that step's lines alone, and that a
    range over it answers `206`: a suffix range with the slice's last
    bytes, and a `Content-Range` naming where they start; what a range
    answers over an empty log; that a step past the last answers `404`;"
  - "that a comparison sent `files=false` lists no commit's files, and
    whether it leaves out its own `files` too; that `verification=false`
    changes nothing the calls keep;"

  measurements.md, 3 answered the open one: `files=false` keeps the
  comparison's own `files`, empty (`[]`), so the comparison's decoder,
  which requires `files` (`client/files.rs:40-42`), reads it as it is.

## Choices this plan makes

1. **One job document, its steps as rows** (code-organization.md, 2;
   design question 6): `d::Job { id, run_id, attempt }` is unchanged
   and keeps `Copy`. A job read by its id answers `Document::Job(Job)`,
   and each of its steps goes up before it as a `Row::Step(Step)` as it
   streams, in Forgejo's order, as a comparison's commits and files do:
   no list of steps is held, and no bound counts them. A job with no
   `steps` member (no runner took it) has no step rows. A run's listing
   of jobs gives no step rows: steps in its rows, if Forgejo sent any,
   are read and dropped (a step in a listing row with a bad `status`
   fails the listing, as any malformed member does). One decoder table serves the job at the root and as a
   listing row. The server's `Answer::Job { job: Job, steps: Box<[Step]>
   }` holds the steps it writes.
2. **Bounds and words, as 06's other documents:** a job's steps are
   counted by no bound (Choices, 1); a step's `name` past `string_bytes`
   is read empty, never refused (forge.md 3.3, design question 1): it is
   read as `Read::Body`, which the walker caps at `string_bytes` for a
   step as it does for `Kind::Error` (`decode/walk.rs:363-367`), and its
   `Body::Long` (`:469-474`) is the empty name; a `status` outside Forgejo's eight
   words is `Malformed::Value { field: b"status" }`; a `number` past
   `u32` is `Malformed::Range { field: b"number" }`. The eight words are
   Forgejo's Actions statuses: `unknown`, `waiting`, `running`,
   `success`, `failure`, `cancelled`, `skipped`, `blocked`; the v17 API
   description's `ActionRunJobStep.status` names six of them ("success,
   failure, running, waiting, skipped, cancelled, ...").
3. **The server writes `steps` only when the job has some**
   (`Answer::Job`'s): a waiting job carries none, and a listing's rows
   (`Answer::Jobs`, of `Job`) never do, as Forgejo's carry none; the
   codecs world holds
   the server's answers to each transcript's shape
   (`tests/codecs/tests/forgejo_files.rs:188-202`).
4. **`JobLog { step: None }` writes no `step`:** the job's log, as at the
   base. The component sends `step: None` until temper 04.1.
5. **The fake, until temper 03.1,** answers `Request::Job` `404`
   (`a::Error::Missing(a::What::Job)`), and ignores `JobLog`'s step
   (`routes.rs:217`'s `..`): its domain has no job by its id yet, and
   nothing sends either request to it before temper 04.1.
6. **Comparisons:** `files: true` writes `?verification=false`, `files:
   false` writes `?files=false&verification=false`, in the design's
   order. The server reads `files` (`false` is `files: false`; absent or
   another value is `files: true`, Forgejo's default) and leaves
   `verification` unread: no request carries another value, since the
   encoder writes it on each comparison, so the round trip holds without
   a field.
7. **The fake's `files=false` comparison** answers its commits and
   `files: []`, as v17 does (measurements.md, 3).
8. **The recorder's range** is `temper_throwaway_forgejo::Range {
   Head(u64), Tail(u64) }`: `Head(n)` writes `bytes=0-<n>`, `Tail(n)`
   writes `bytes=-<n>`, the names temper 04.2 gives the component's
   range (code-organization.md, 2).
9. **The captures' places in the run,** each after the base's: 02.1's in
   `runner.rs`, after `actions-job-log-range` (they read the runner's
   jobs); 02.2's in `lib.rs`, after `runner::run` (they need no runner).
10. **The suffix and the empty log:** the suffix range is 10 bytes
    (`SUFFIX`), smaller than the failing step's slice (123 bytes in the
    spike's `v17-log-step`). The empty log is step 1 of the passing job
    (`run: exit 0`). Not observed directly: the probe's `run: "true"`
    slice was empty, and the spike's slices put a step's `⭐ Run Main`
    line in step 0's slice and a failure's `RUN exit status` line in the
    step's own, so a passing step that prints nothing has an empty slice;
    the fact `empty-log-suffix-range-answers-206-empty` shows it.
11. **Each recording runs both modes,** in one `sh -c`: the
    plain mode writes `facts.txt` from its own facts (`lib.rs:33-36`),
    without the `component-` block, which only the component mode writes
    back (`component.rs:19-21`, `facts.rs:503-526`). The command, from
    the increment's worktree, on a committed tree:

    ```sh
    TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 \
    TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 \
    sh -c 'cargo run -p forgejo-conformance -- --scratch target/forgejo-conformance --out tools/forgejo-conformance && cargo run -p forgejo-conformance -- --scratch target/forgejo-component --out tools/forgejo-conformance --component'
    ```

    About 70 s after the tool's build (38 s and 24 s in the spike).
    Bound: 10 minutes for the task, the build included, as 01.1's
    (`01-pin.md`, 01.1's Proof, item 1). A
    failed run may leave its scratch directory: `rm -rf
    target/forgejo-conformance target/forgejo-component` before the next.
    The order within an increment: the code and its unit tests, a commit,
    the recording, a commit of what the tools wrote, then the codecs
    world's rows, read from those transcripts by hand. As in 01.1
    (`01-pin.md`, 01.1's Test shape and Don't): each literal a re-masking
    moved is read from the new transcripts by hand, and a fact that
    changes outcome (a `holds` now `fails`, or the reverse) is a
    question, with its `seen` text.
12. **In the codecs world, the flagged comparisons replace `compare` and
    `compare-reverse` in `cases()`:** the encoder writes no comparison
    without flags, so those rows cannot hold
    `file_requests_encode_as_the_tool_sent_them`. Their transcripts stay,
    as Forgejo's defaults: 02.2's facts and the fuzzy suite read them, and
    so does the mutation row `("compare", "sha", …)`.

## Rules that bind it

- **Transcripts and `facts.txt` are written by the tools only,** never by
  hand, and a new capture is appended after the existing ones, since
  masking numbers ids and times in order of appearance (README, "Rules of
  this plan"; code-organization.md, 5). A shifted mask moves a literal
  the codecs world reads, and its tests fail.
- **The recording tools run only where a Proof line says,** through
  once, and once more after each fix aimed at a cause
  their output names; the third failure with the same symptom is a
  question (README, "Rules of this plan"). `facts.txt` is read after the
  run: a `fails` line still exits 0.
- **No field its counterpart lacks:** the client's decoder and the
  server's encoder of `temper-forge-forgejo` carry the same job and the
  same comparison request, and the fake answers what the facts record, no
  better (code-organization.md, 5). Expected values in tests are
  literals, read from the transcripts by hand.

## Increments

### 02.1 A job with its steps, and a step's log

Scope: Forgejo 17's job document on both sides of `temper-forge-forgejo`
(forge.md 3.2, "A failed job's log is its failed step's end"; 9; 10):
`Request::Job`, `JobLog`'s `step`, the job with its steps, its client
decoder, server route and encoder; the recorder's suffix range; the
conformance's captures and facts of 13's first two "to check" bullets
(but the waiting job: `later.md`); the codecs world decoding them.

Needs: nothing beyond the session's start (temper 01.1: the pin on
17.0.0 and 13.2.0, the conformance's transcripts recorded on them, and
the codecs world's literals following them).

Files:
- `crates/temper-forge-forgejo/src/`: `document.rs` (`Job` `:334-340`,
  `Document` `:83-102`, beside `style`/`style_text` `:193-213`);
  `request.rs:142-150`; `client.rs:48-54`, `:93-128`, `:133`;
  `decode/walk.rs:363-367` (the `Read::Body` cap);
  `client/actions.rs:1-135`; `client/files.rs:95-104`; `server.rs`
  (`Answer` `:64-106`, `:137`); `server/actions.rs:24-45`, `:76-89`. The
  exhaustive matches a new `Request` reaches: `client/writes.rs:123,245`,
  `client/items.rs:151`, `client/start.rs:36,106`,
  `client/reviews.rs:125`, `server/writes.rs:279,390,434`; those a new
  `server::Answer` reaches: `server/start.rs:128`, `server/files.rs:139`,
  `server/reviews.rs:130`, `server/branches.rs:125`,
  `server/items.rs:237`. Tests: `src/tests.rs`.
- `crates/temper-engine-protocol-forge/src/`: `connection.rs:645-686`
  (`raw_request`); `reads.rs:658-699` (`page_bound`); the six `d::Document`
  matches, `actions.rs:304-317`, `contents.rs:269-282`, `reads.rs:646-655`
  and `:719-736`, `reads/done.rs:85-92`, `writes.rs:318-333`;
  `actions.rs:300` (`step: None`); the `d::Row` matches,
  `actions.rs:233-242`, `contents.rs:207-217`, `reads.rs:388`,
  `:624-629` and `:708-716`, `writes.rs:196-207`. Shared with temper 04
  (`actions.rs`).
  Tests: `src/tests.rs`, beside `:307-315`.
- `testing/temper-fake-forge-protocol/src/`: `routes.rs:79-115`,
  `:142-226`, `:297-318`; `answers.rs:213,275,437` and `:357`;
  `response.rs:111` (`follow`), `:154` (`select`), `:204` (`status`);
  `server.rs:432`, `:507` (`table`); `tally.rs` (`Route`, `route`, `:104`,
  `:114`); `service.rs:158`. Tests: `src/tests/routes.rs:102`. Shared with
  temper 03 (`routes.rs`, `answers.rs`).
- `testing/temper-throwaway-forgejo/src/lib.rs:64-85` (`Outgoing`),
  `http.rs:16-18`; tests `src/tests.rs`. temper 01.2 (git's transcripts)
  builds on this crate and may change it.
- `tools/forgejo-conformance/src/fixture.rs:74-83`, `cases.rs:24,201`,
  `runner.rs` (`NAMES` `:15-22`, `:102-105`, `capture` `:162-174`);
  `README.md` (its count of Actions transcripts: six, then eleven);
  `transcripts/` and `facts.txt`, written by the tool.
- `tests/codecs/tests/forgejo_files.rs` (the imports `:6`, `jobs()`
  `:20-22`, `log()` `:36-38`, `cases()` `:39-116`, `expected()`
  `:117-155`, the mutations `:204-232`); the `server::Answer` lists of
  `forgejo_items.rs:166`, `forgejo_reviews.rs:138`, `forgejo_start.rs:55`
  and `forgejo_writes.rs:159`; `forgejo_writes.rs`'s `failure_cases()`
  (`:233-258`). `Job` keeps `Copy` (design question 6), so
  `client/actions.rs:79` and `forgejo_files.rs:253` do not change.

The users above were found with `git grep -n -e 'Request::JobLog' -e
'Answer::Jobs' -e 'Document::Runs' -e 'Job { id' -e '39' -e 'range:
Option<(u64, u64)>' -e 'Some((0, 3))' -e 'Row::Job(' -- '*.rs'` at the
base; its other hits are the fake domain's own `Answer::Jobs` and `Job`
(`testing/temper-fake-forge-domain`, `tests/engine/forge/client`,
`tests/fake-forge`), which this session does not change. component.md, 1
lists most, and leaves out `service.rs:158`.

Shape:
- `temper_forge_forgejo::document`: `Step`, `StepStatus`,
  `Row::Step(Step)` and `Document::Job(Job)`, as the sketch; `Job`
  unchanged; `pub(crate) fn step_status(text: &[u8]) -> Option<StepStatus>`
  and `pub(crate) fn step_status_text(status: StepStatus) -> &'static
  [u8]` beside `style`/`style_text`.
- `request::Request::Job { at: Repo, job: u64 }` (`GET
  R/actions/jobs/<job>`); `JobLog { at, job, attempt: u32, step:
  Option<u32> }` (`…/logs?attempt=<attempt>`, then `&step=<step>` when
  some).
- The client's encoder (`client/files.rs`): `Job` as `out.numbered(at,
  b"/actions/jobs/", *job)`; `JobLog`'s `&step=`. `client::encode` sends
  `Job` to `files::encode`.
- The client's decoder (`client/actions.rs`; `decode::Step` is in scope
  there, so the document's is written `document::Step`): `ActionKind`
  gains `Steps` (an array) and `Step` (its element); `fields(Job)` adds
  `steps`, optional, `Read::Array(Kind::Action(ActionKind::Steps))`;
  `fields(Step)` is `number` (`Read::Number`), `name` (`Read::Body`)
  and `status` (`Read::Text`), required; `element(Steps)` is `Step`.
  The walker's `Read::Body` cap (`decode/walk.rs:363-367`) is
  `string_bytes` for `Kind::Action(ActionKind::Step)` as for
  `Kind::Error`; a step's `Body::Kept(name)` is its name, a
  `Body::Long(_)` an empty name. `Actions` holds the step being read. A
  step's end, with the root `Job`, answers `Step::Row(Row::Step(step),
  Request::Next)`, as a run's end answers its row with the root
  `Listing`; with the root `Jobs` the step is dropped. A job's end
  answers it as a row only when the root is `Jobs` (the base answers one
  always, `client/actions.rs:78-80`: without the check, a job read at
  the root sends a `Row::Job` up before its steps); with the root `Job`,
  `Move::Done` answers `Document::Job(job)`.
  `client::decoder` maps `Request::Job` to `Kind::Action(ActionKind::Job)`.
  If clippy's `large_enum_variant` names `DecoderState::Actions`, it is
  boxed, as its siblings are.
- The server (`server/actions.rs`): the route `[jobs, job]` is
  `Request::Job`; `[jobs, job, logs]` reads `step` when present, as a
  `u32` (`Unrouted::Malformed` otherwise). `server::Answer::Job { job: Job,
  steps: Box<[Step]> }`; one `fn job(json: &mut Encoder, value: &Job,
  steps: &[Step])` writes `id`, `run_id`, `attempt`, and `steps` (each
  `number`, `name`, `status`) only when there are some; `Answer::Jobs`
  writes each row through it with no steps.
- The component: `raw_request` classes `Request::Job` with the JSON
  requests (`false`), and `page_bound` with the unpaged ones (`None`):
  each match classifies the shared enum's requests, and both answers are
  true whoever sends it. `d::Document::Job(_)` joins each match's arm for
  a document the plan did not ask for (`InvalidAnswer` in `actions.rs`,
  `contents.rs` and `reads/done.rs`; nothing in `writes.rs`,
  `keep_document` and `long`), as `Document::Run` does in the plans that
  never send `Request::Run`. `d::Row::Step(_)` joins each `d::Row`
  match the same way: `InvalidAnswer` in `actions.rs`'s rows
  (`:233-242`), `contents.rs:207-217` and `reads.rs:388`; nothing in
  `writes.rs:196-207` and `reads.rs:624-629`; `false` in `long`'s arm,
  `reads.rs:708-716`.
  `actions.rs:300` sends `step: None`. No arm reads a job or a step
  before temper 04.1. The component's `worst_case` (`limits.rs:311`) is
  not changed: a step is one row being built and one delivered, its name
  one string of at most `string_bytes`, within the envelope's 32 scalar
  strings and its rows (`:322-334`).
- The fake: `Route::Job`, so 40 routes (`tally.rs:104`, `:114`,
  `server.rs:432` and `service.rs:158` say 40); `routes::op` answers
  `Request::Job` with `Err(a::Error::Missing(a::What::Job))` (Choices, 5);
  `routes::repository` gives its path; `response.rs:204` answers it 200;
  the other arms join their lists; `answers.rs:357`'s `d::Job` is
  unchanged.
- The recorder: `temper_throwaway_forgejo::Range`, `Outgoing.range:
  Option<Range>` (the sketch); `http.rs` writes `Range: bytes=0-<n>` for
  `Head(n)` and `Range: bytes=-<n>` for `Tail(n)`; the parameters
  `fixture.rs:80`, `cases.rs:24` and `runner.rs:169` take `Option<Range>`
  where they took `Option<(u64, u64)>`, and `cases.rs:201` and
  `runner.rs:104` pass `Range::Head(3)` where they passed `(0, 3)`.
- The conformance (`runner.rs`), in a function of its own, since
  `run_with` is 92 lines and clippy's pedantic `too_many_lines` stops at
  100: `fn job_steps(f: &Fixture, out: Option<&Path>, mask: &mut Mask,
  secrets: &Secrets, jobs: &[Value], whole: &Exchange) ->
  Result<Vec<Fact>>`, called after `actions-job-log-range`, its facts
  appended after the six. `whole` is the `actions-job-log` capture's
  `Exchange`, which `run_with` keeps (the base drops it into
  `Value::Null`) through a helper `fn exchange(f, out, mask, secrets,
  name, target, range: Option<Range>) -> Result<Exchange>` that records
  and returns it; `capture` calls it. The captures, in this order:
  1. `actions-job`: `GET {R}/actions/jobs/<the failed job's id>`, the job
     `run_with` found (`name == "fail"`);
  2. `actions-job-log-step`: `GET {R}/actions/jobs/<id>/logs?attempt=<its
     attempt>&step=<failed>`, `failed` the `number` of the first step of
     `actions-job` whose `status` is `failure`;
  3. `actions-job-log-step-suffix`: the same, with
     `Some(Range::Tail(SUFFIX))`, `const SUFFIX: u64 = 10`;
  4. `actions-job-log-past-last-step`: the same log with
     `&step=<last + 1>`, `last` the largest `number`;
  5. `actions-job-log-empty-suffix`: `GET {R}/actions/jobs/<pass>/logs?attempt=<its
     attempt>&step=1` with `Some(Range::Tail(SUFFIX))`, `pass` the job
     named `pass` in `jobs`.

  `NAMES` grows to 13, the seven facts below appended in this order, so
  that without a runner they read `unchecked` as the six do. With two
  helpers, `contains(hay: &[u8], needle: &[u8]) -> bool` (false for an
  empty needle) and `content_range(exchange: &Exchange) -> Option<String>`
  (the `Content-Range` field's value, its name read in any case):
  - `job-steps-number-from-0-set-up-first-complete-last`: `steps.len() >=
    3`; each `steps[i]["number"]` is `i`; `steps[0]["name"] == "Set up
    job"`; the last step's `name` is `"Complete job"`;
  - `failed-workflow-step-and-completion-step-are-failure`:
    `steps[0]["status"] == "success"`, `0 < failed < last`, and the last
    step's `status` is `"failure"`;
  - `step-log-holds-that-steps-lines-alone`: the step's capture answers
    200; its body holds `TEMPER_CONFORMANCE_FAILED_JOB`, is shorter than
    `whole`'s body and a run of its bytes, and holds neither `Run Main`
    (the set-up step's last line) nor `Job failed` (the completion
    step's);
  - `step-log-suffix-range-answers-206-with-its-last-bytes`: the suffix
    capture answers 206, the step's body is longer than `SUFFIX` bytes,
    and the suffix capture's body is its last `SUFFIX` bytes;
  - `step-log-suffix-content-range-names-its-first-byte`: the suffix
    capture's `content_range` is `bytes <n - 10>-<n - 1>/<n>`, `n` the
    step's body's length;
  - `empty-log-suffix-range-answers-206-empty`: the empty capture answers
    206 with an empty body and the `content_range` `bytes 0--1/0`
    (measurements.md, 3);
  - `step-past-the-last-is-404`: the past-last capture answers 404.

  Each fact's `seen` text names the statuses, lengths and `Content-Range`
  it read. In the component mode, `run_with` makes these exchanges and
  records none, as for its other captures.

Test shape:
- `crates/temper-forge-forgejo/src/tests.rs`, on `fixtures::limits()`
  (`src/tests/fixtures.rs:12-21`, `inner` 64 and `string_bytes` 1024, the
  crate's test values) and `fixtures::decode` (`:55-57`); each under
  20 ms. The reference test:

  ```rust
  #[test]
  fn a_job_reads_its_steps_numbers_names_and_statuses() {
      use crate::document::{Job, Row, Step, StepStatus};
      let limits = fixtures::limits();
      let request = Request::Job {
          at: crate::request::Repo { owner: Box::from(*b"fixture"), name: Box::from(*b"specimen") },
          job: 2,
      };
      let body = br#"{"id":2,"run_id":1,"attempt":1,"name":"fail","status":"failure","steps":[{"number":0,"name":"Set up job","status":"success","started":"2020-01-01T00:00:14+05:30"},{"number":1,"name":"exit 1\n","status":"failure"},{"number":2,"name":"Complete job","status":"failure"}]}"#;
      let decoded = fixtures::decode(client::decoder(&request, &limits).unwrap(), body, &limits);
      assert_eq!(decoded.document, Ok(Document::Job(Job { id: 2, run_id: 1, attempt: 1 })));
      assert_eq!(
          decoded.rows.as_slice(),
          [
              Row::Step(Step { number: 0, name: Box::from(*b"Set up job"), status: StepStatus::Success }),
              Row::Step(Step { number: 1, name: Box::from(*b"exit 1\n"), status: StepStatus::Failure }),
              Row::Step(Step { number: 2, name: Box::from(*b"Complete job"), status: StepStatus::Failure }),
          ],
          "each step goes up as a row, in Forgejo's order"
      );
  }
  ```

  The others: `a_job_without_a_steps_member_reads_no_steps` (`{"id":4,
  "run_id":2,"attempt":0,"status":"waiting"}`: the document, no row);
  `a_job_encodes_its_steps_only_when_it_has_some` (golden bytes:
  `server::encode(&Answer::Job { job, steps: …one step… })` is
  `{"id":2,"run_id":1,"attempt":1,"steps":[{"number":0,"name":"Set up
  job","status":"success"}]}`, and `Answer::Job` with no steps is
  `{"id":2,"run_id":1,"attempt":1}`);
  `each_step_status_reads_back_from_the_servers_word` (a table of the
  eight `StepStatus` values: a one-step job the server encodes decodes to
  the same job); `a_step_status_outside_forgejos_words_is_malformed`
  (`"status":"done"`: `Err(Malformed::Value { field: b"status" })`);
  `a_job_reads_more_steps_than_inner_as_rows` (`limits.inner = 2`: a
  job of three steps reads `Ok(Document::Job(…))` and three step rows;
  design question 6); `a_step_name_past_string_bytes_reads_empty`
  (`limits.string_bytes = 8`; two rows: a step named `12345678`, at the
  limit, keeps its name; one named `123456789`, one past, reads as a
  step row with an empty name, its `number` and `status` kept, and the
  document `Ok`; design question 1).
- `testing/temper-throwaway-forgejo/src/tests.rs`, under 20 ms:

  ```rust
  #[test]
  fn a_range_is_sent_as_a_head_or_a_suffix() {
      for (range, line) in [(Range::Tail(10), "Range: bytes=-10\r\n"), (Range::Head(3), "Range: bytes=0-3\r\n")] {
          let outgoing = Outgoing { range: Some(range), ..Outgoing::get("/api/v1/version", None) };
          let sent = String::from_utf8(http::request(3000, &outgoing, None)).expect("UTF-8 request");
          assert!(sent.contains(line), "{range:?}: {sent}");
      }
  }
  ```
- The codecs world, `tests/codecs/tests/forgejo_files.rs`: the rows
  below, appended to `cases()` in the run's order, so that
  `file_transcripts_decode_to_their_documents_and_rows`,
  `file_requests_encode_as_the_tool_sent_them`,
  `file_requests_route_on_the_servers_side_to_the_same_request` and
  `file_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape`
  read them (each under 0.5 s); the values are the spike's `v17-job`, and
  the author reads them from its own `actions-job` transcript:

  ```rust
  fn failing_job() -> Answer {
      Answer::Job {
          job: Job { id: 2, run_id: 1, attempt: 1 },
          steps: Box::from([
              Step { number: 0, name: Box::from(*b"Set up job"), status: StepStatus::Success },
              Step {
                  number: 1,
                  name: Box::from(*b"printf 'TEMPER_CONFORMANCE_FAILED_JOB\\n'\nexit 1\n"),
                  status: StepStatus::Failure,
              },
              Step { number: 2, name: Box::from(*b"Complete job"), status: StepStatus::Failure },
          ]),
      }
  }
  fn step_log(job: u64, step: u32) -> Request {
      Request::JobLog { at: forgejo::specimen(), job, attempt: 1, step: Some(step) }
  }
  // cases(), after ("actions-job-log-range", log(), None, 206):
  ("actions-job", Request::Job { at: forgejo::specimen(), job: 2 }, Some(failing_job()), 200),
  ("actions-job-log-step", step_log(2, 1), None, 200),
  ("actions-job-log-step-suffix", step_log(2, 1), None, 206),
  ("actions-job-log-past-last-step", step_log(2, 3), None, 404),
  ("actions-job-log-empty-suffix", step_log(1, 1), None, 206),
  // expected():
  Answer::Job { job, steps } => (Document::Job(job), steps.into_vec().into_iter().map(Row::Step).collect()),
  ```

  `log()` gains `step: None`; `jobs()` is unchanged.
  `file_transcript_mutations_are_malformed_naming_the_field` gains
  `("actions-job", "status", 5, Malformed::Value { field: b"status" })`
  (`5 => json["steps"][0][field] = serde_json::json!("finished")`) and
  `("actions-job", "number", 6, Malformed::Range { field: b"number" })`
  (`6 => json["steps"][1][field] = serde_json::json!(4_294_967_296u64)`).
  `forgejo_writes.rs`'s `failure_cases()` gains
  `("actions-job-log-past-last-step", 404, b"step index out of range",
  None)`. The fuzzy suite's
  `forgejo_decoders_answer_every_mutated_transcript_without_panicking`
  and `forgejo_server_routes_answer_every_mutated_request_without_panicking`
  (`tests/codecs/tests/fuzzy_forgejo.rs`) route the new captures
  unchanged; `actions-job`, a 2xx JSON capture, needs 02.1's route
  (`tests/codecs/src/forgejo.rs:361-377`).
- `testing/temper-fake-forge-protocol/src/tests/routes.rs:102`: `step:
  None`; no test of the fake is added (its job route is temper 03.1's).

Proof: the real Forgejo 17.0.0 and runner 13.2.0, through the recording
(Choices, 11), once, on the increment's committed code: Forgejo's job
with its steps and a step's log by a suffix range are captured, and
`grep -c -E '^holds (job-steps-number-from-0-set-up-first-complete-last|failed-workflow-step-and-completion-step-are-failure|step-log-holds-that-steps-lines-alone|step-log-suffix-range-answers-206-with-its-last-bytes|step-log-suffix-content-range-names-its-first-byte|empty-log-suffix-range-answers-206-empty|step-past-the-last-is-404)$'
tools/forgejo-conformance/facts.txt` prints 7; `grep -E '^fails '
tools/forgejo-conformance/facts.txt | cut -d: -f1` prints the base's
four; `grep -c '^holds component-' tools/forgejo-conformance/facts.txt`
prints 39. About 70 s after the build; bound: 10 minutes, the build
included (Choices, 11). The codecs world decodes the
recorded job in the gate; temper 04.1 connects the job read to the
engine, and temper 03.1 to the fake. If `empty-log-suffix-range-answers-206-empty`
fails because the `exit 0` step's slice holds bytes, it is a question with
the capture, not a change of the workflow.

Don't: send `Request::Job` or a step's log from the component, or serve
either from the fake's domain (temper 04.1, 03.1); read `bytes 0--1/0` in
`header.rs:19-38` (temper 04.2: no test of this session reads a
`Content-Range` through it; the codecs world checks status lines and
bodies); add a second job type; change the workflow at
`runner.rs:60`, whose jobs the transcripts' literals rest on; capture a
log's search (`q`, `qi`) or its NDJSON form.

Size: at most 370 lines (ruling 103; the plan said 290: documents 45,
the decoder 50, the server 30, the request and the encoder 10, the
exhaustive arms 50, the recorder 15, the conformance 90). The parts are
a guide; the conformance came to about 110 once rustfmt laid out its
captures and checks. The total is the rule, counted by
code-organization.md, section 6's command, once before landing, less
the three doc comment lines steer 1 corrects.

Removes: the recorder's `Option<(u64, u64)>`.

### 02.2 The comparisons' flags

Scope: the comparisons ask for what they keep (forge.md 3.2, the rows
`Read::Compare` and `Write::OpenPull`; the note "Comparisons ask for
what they keep"): `Request::Compare`'s `files` and `verification=false`
on each comparison, both sides; the component's three comparisons; the
fake's `files=false` answer; the captures and facts of 13's third "to
check" bullet; the component mode recorded again, since its requests
changed.

Needs: temper 02.1 (its captures follow 02.1's in the run).

Files:
- `crates/temper-forge-forgejo/src/request.rs:122-126`,
  `client/files.rs:73-79`, `server/files.rs:30-38`; tests `src/tests.rs`.
- `crates/temper-engine-protocol-forge/src/contents.rs:97` (`files:
  true`), `:256` (`files: false`), `writes.rs:508` (`files: false`);
  tests `src/tests/contents.rs:116-144`, `src/tests/writes.rs:283-296`.
  Shared hotspot: `src/tests/contents.rs`, which temper 04.1 also edits
  and may edit at the same time.
- `testing/temper-fake-forge-protocol/src/answers.rs:362-371`; tests
  `src/tests/answers.rs`, `src/tests/routes.rs:85` (`files: true`);
  `routes.rs:203` reads `{ before, after, .. }` and stays. Shared
  hotspots: `src/answers.rs`, `src/tests/answers.rs` and
  `src/tests/routes.rs`, which temper 03.1 also edits and may edit at
  the same time.
- `tools/forgejo-conformance/src/cases.rs` (`flags`), `facts.rs`
  (`flags`), `lib.rs:24-25`; `transcripts/` and `facts.txt`, written by
  the tool.
- `tests/codecs/tests/forgejo_files.rs:53-72` (the two rows replaced by
  three).

Users: `git grep -n -e 'Request::Compare' -e '/compare/' -- '*.rs'` at
the base prints the lines above, the exhaustive lists (which match `{ ..
}` and stay) and `tools/forgejo-conformance/src/cases.rs:195-196` and
`facts.rs:136` (the plain mode's own requests, which stay).

Shape:
- `Request::Compare { at, before, after, files: bool }` (the sketch).
- The encoder: after `/compare/<before>...<after>`, `?verification=false`
  when `files`, `?files=false&verification=false` when not.
- The route: `files: query_value(&target.query, b"files").as_deref() !=
  Some(b"false".as_slice())` (Choices, 6).
- The component: `Read::Compare`'s first comparison `files: true`, its
  second `files: false`; `OpenPull`'s `files: false`.
- The fake: `answers::read`'s `a::Answer::Comparison` arm answers `files:
  Box::from([])` when the request is `Request::Compare { files: false,
  .. }`, `files(values)` otherwise; its commits and `total_commits` as
  before. `answers::read` is 96 of clippy's 100 `too_many_lines` at the
  base: the check goes in a helper outside it (`fn compared(request:
  &Request, values: ...) -> Box<[ChangedFile]>`), and the arm calls it
  in its one `files:` line.
- The conformance: `pub fn flags(f: &Fixture, out: &Path, mask: &mut
  Mask, secrets: &Secrets) -> Result<()>` in `cases.rs` records, with the
  writer's token, through `Recorder::w`: `compare-verification-false`
  (`{R}/compare/{main}...{head}?verification=false`),
  `compare-files-false`
  (`{R}/compare/{main}...{head}?files=false&verification=false`) and
  `compare-reverse-files-false`
  (`{R}/compare/{head}...{main}?files=false&verification=false`), `main`
  and `head` the fixture's `f.main` and `f.head`, as `compare` at
  `cases.rs:195-196`. `pub fn flags(out: &Path) -> Result<Vec<Fact>>` in
  `facts.rs` reads them, and `compare` and `compare-reverse`, with
  `case`:
  - `compare-files-false-lists-no-commits-files`: `compare-files-false`
    answers 200 with at least one commit, and each commit's `files` is
    null or absent;
  - `compare-files-false-keeps-its-own-files-empty`:
    `compare-files-false`'s and `compare-reverse-files-false`'s top-level
    `files` are `[]`;
  - `compare-verification-false-changes-nothing-kept`:
    `compare-verification-false` answers 200, and its `total_commits`,
    its list of `commits[].sha` and its list of `files[].filename` equal
    `compare`'s;
  - `compare-files-false-keeps-total-commits-and-shas`:
    `compare-files-false`'s `total_commits` and list of `commits[].sha`
    equal `compare`'s, and `compare-reverse-files-false`'s
    `total_commits` equals `compare-reverse`'s.

  `lib.rs::run` calls `cases::flags`, then `facts::flags`, after
  `runner::run`, the four facts appended after the runner's.

Test shape:
- `crates/temper-forge-forgejo/src/tests.rs`, under 20 ms:

  ```rust
  #[test]
  fn a_comparison_sends_verification_false_and_files_false_when_asked() {
      let limits = fixtures::limits();
      let before = id::read(b"0100000000000000000000000000000000000000").unwrap();
      let after = id::read(b"0300000000000000000000000000000000000000").unwrap();
      // `format!` is a disallowed macro here: the two targets are written whole.
      let targets: [(bool, &[u8]); 2] = [
          (true, b"/api/v1/repos/fixture/specimen/compare/0100000000000000000000000000000000000000...0300000000000000000000000000000000000000?verification=false"),
          (false, b"/api/v1/repos/fixture/specimen/compare/0100000000000000000000000000000000000000...0300000000000000000000000000000000000000?files=false&verification=false"),
      ];
      for (files, target) in targets {
          let request = Request::Compare {
              at: crate::request::Repo { owner: Box::from(*b"fixture"), name: Box::from(*b"specimen") },
              before,
              after,
              files,
          };
          let encoded = client::encode(&request, &limits).unwrap();
          assert_eq!(encoded.target.as_ref(), target, "{files}");
          let crate::server::Route::Ready(routed) =
              crate::server::route(encoded.method, &encoded.target, &limits).unwrap()
          else {
              panic!("a comparison routes without a body");
          };
          assert_eq!(routed, request, "{files}");
      }
  }
  ```
- `testing/temper-fake-forge-protocol/src/tests/answers.rs`, on
  `fixtures::{at, boxed, config}`, under 20 ms:

  ```rust
  #[test]
  fn a_files_false_comparison_keeps_commits_and_lists_no_files() {
      let input = a::Answer::Comparison {
          base: 8,
          head: 9,
          contains_base: true,
          files: Box::new([a::ChangedFile { path: boxed(b"src"), before: None, after: Some(boxed(b"a")) }]),
          commits: Box::new([9]),
      };
      for (files, listed) in [(true, 1), (false, 0)] {
          let request = Request::Compare { at: at(), before: routes::object(8), after: routes::object(9), files };
          let Ok(Answer::Compare { compare, commits, files: changed }) = answers::read(&config(), &request, &input)
          else {
              panic!("a comparison");
          };
          assert_eq!((compare.total_commits, commits.as_ref()), (1, [routes::object(9)].as_slice()), "{files}");
          assert_eq!(changed.len(), listed, "{files}");
      }
  }
  ```
- The component's request lines follow, in the tests that pin them:
  `comparisons_read_both_directions_and_both_file_sides` asserts
  `lines[2]`, `GET
  /api/v1/repos/fixture/specimen/compare/0300000000000000000000000000000000000000...0100000000000000000000000000000000000000?verification=false
  HTTP/1.1`, beside `lines[3]`, now ending
  `...0300000000000000000000000000000000000000?files=false&verification=false
  HTTP/1.1`; `open_pull_with_nothing_to_merge_compares_and_writes_nothing`
  finds `…/compare/0100000000000000000000000000000000000000...0100000000000000000000000000000000000000?files=false&verification=false `.
- The codecs world: `cases()`'s `compare` and `compare-reverse` rows give
  way to three, appended after 02.1's, their values those the `compare`
  and `compare-reverse` rows hold at the session's start (the same
  commits; `sha(11)` is the spike's re-recorded value for the base's
  `sha(10)`, `findings/tools.md`, 3), read again from the new transcripts
  by hand:

  ```rust
  (
      "compare-verification-false",
      Request::Compare { at: forgejo::specimen(), before: forgejo::sha(6), after: forgejo::sha(4), files: true },
      Some(Answer::Compare {
          compare: Compare { total_commits: 3 },
          commits: Box::from([forgejo::sha(4), forgejo::sha(11), forgejo::sha(8)]),
          files: Box::from([file(2), file(1), file(0)]),
      }),
      200,
  ),
  (
      "compare-files-false",
      Request::Compare { at: forgejo::specimen(), before: forgejo::sha(6), after: forgejo::sha(4), files: false },
      Some(Answer::Compare {
          compare: Compare { total_commits: 3 },
          commits: Box::from([forgejo::sha(4), forgejo::sha(11), forgejo::sha(8)]),
          files: Box::from([]),
      }),
      200,
  ),
  (
      "compare-reverse-files-false",
      Request::Compare { at: forgejo::specimen(), before: forgejo::sha(4), after: forgejo::sha(6), files: false },
      Some(Answer::Compare { compare: Compare { total_commits: 0 }, commits: Box::from([]), files: Box::from([]) }),
      200,
  ),
  ```

Proof: the real component on the throwaway Forgejo 17.0.0, through the
recording (Choices, 11), once: the component sends its comparisons with
the flags, and `grep -c -E '^holds (component-read-compare|component-write-open-pull|component-write-open-pull-no-changes|component-costs-equal-requests-sent)$'
tools/forgejo-conformance/facts.txt` prints 4; `grep -E '^(fails|unchecked)
component-' tools/forgejo-conformance/facts.txt` prints nothing; and the
plain mode's `grep -c -E '^holds (compare-files-false-lists-no-commits-files|compare-files-false-keeps-its-own-files-empty|compare-verification-false-changes-nothing-kept|compare-files-false-keeps-total-commits-and-shas)$'
tools/forgejo-conformance/facts.txt` prints 4. About 70 s after the
build; bound: 10 minutes, the build included (Choices, 11). The protocol world (`tests/engine/protocol-forge`, the component
on the fake's HTTP layer) runs the comparisons' stories with the flags in
the gate.

Don't: send `files=false` on `Read::Compare`'s first comparison; record
`compare` or `compare-reverse` with flags (they stay Forgejo's
defaults); make the fake refuse a comparison without flags; give
`Request` a field for `verification`.

Size: at most 70 lines.

Removes: the component's comparisons without flags; the codecs world's
`compare` and `compare-reverse` rows (their transcripts stay).

## Interface sketches

02.1 owns the job document, `Request::Job`, `JobLog`'s `step`,
`server::Answer::Job` and the recorder's `Range`; 02.2 owns
`Request::Compare`'s `files`. temper 03.1 uses `Answer::Job` and
`d::Step` in the fake (its arm for `Request::Job` replaces Choices 5's,
and it reads `JobLog`'s `step`); temper 04.1 sends `Request::Job` and
`JobLog { step: Some(_) }` from the component and keeps, of the job's
`Row::Step`s, the first whose `status` is `StepStatus::Failure`
(decisions.md, 7; design question 6).

`Step`, `StepStatus` and `Range` compile standalone in their modules at
the base. `Job` is unchanged; the enum excerpts compile with the arms
02.1 and 02.2 add.

```rust
// crates/temper-forge-forgejo/src/document.rs (02.1)

// `Job { id, run_id, attempt }` is unchanged, `Copy`: a job read by its id sends its latest
// attempt's steps up before it, each a `Row::Step` (design question 6).

/// One step of a job's latest attempt, a row of `GET R/actions/jobs/<id>` (forge.md 3.2, "A
/// failed job's log is its failed step's end").
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Step {
    /// `number`: 0 is `Set up job`, the last `Complete job`; what a log's `step=` names.
    pub number: u32,
    /// `name`: the workflow's `name:`, or the step's command text; empty when Forgejo's passed
    /// `string_bytes` (forge.md 3.3).
    pub name: Box<[u8]>,
    /// `status`.
    pub status: StepStatus,
}

/// A step's `status`, in Forgejo's words for an Actions status.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StepStatus {
    /// `unknown`.
    Unknown,
    /// `waiting`.
    Waiting,
    /// `running`.
    Running,
    /// `success`.
    Success,
    /// `failure`: the failed step is the first with it; the completion step has it whenever one did.
    Failure,
    /// `cancelled`.
    Cancelled,
    /// `skipped`.
    Skipped,
    /// `blocked`.
    Blocked,
}

// `Document` gains, after `Run(Run)`:
//     /// `GET R/actions/jobs/<id>`, its steps gone up before it as rows.
//     Job(Job),
// `Row` gains, after `Job(Job)`:
//     /// A step of `GET R/actions/jobs/<id>`.
//     Step(Step),
// and `server::Answer`, after `Run(..)`:
//     Job { job: crate::document::Job, steps: Box<[crate::document::Step]> },
```

```rust
// crates/temper-forge-forgejo/src/request.rs (excerpt: the variants 02 adds or changes; the others stay)
pub enum Request {
    /// `GET R/actions/jobs/<job>` (02.1).
    Job { at: Repo, job: u64 },
    /// `GET R/actions/jobs/<job>/logs?attempt=<attempt>`, then `&step=<step>` when some (02.1 adds `step`).
    JobLog { at: Repo, job: u64, attempt: u32, step: Option<u32> },
    /// `GET R/compare/<before>...<after>?verification=false`, or
    /// `?files=false&verification=false` when `files` is false (02.2 adds `files`).
    Compare { at: Repo, before: crate::id::ObjectId, after: crate::id::ObjectId, files: bool },
}
```

```rust
// testing/temper-throwaway-forgejo/src/lib.rs (02.1)

/// The `Range` field of a recorded request.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Range {
    /// `bytes=0-<n>`: a file's first `n + 1` bytes.
    Head(u64),
    /// `bytes=-<n>`: a log's last `n` bytes.
    Tail(u64),
}

#[derive(Clone)]
pub struct Outgoing<'a> {
    pub method: &'a str,
    pub target: &'a str,
    pub token: Option<&'a Token>,
    pub body: Option<&'a [u8]>,
    pub range: Option<Range>,
}
```

## Questions to raise when reached

None. The designer answered the two found while planning, on
2026-10-16 (forge.md 3.2 and 3.3):

- **A job of more steps than `inner`** (design question 6): no bound
  counts a job's steps; each goes up as a row as it streams, and the
  component keeps the failed one (Choices, 1).
- **A step named by its command text** (design question 1): a name past
  `string_bytes` is read empty, never refused, and the step is named by
  its number (Choices, 2).

## Done when

Each command runs once, in `~/src/rust/temper`, reading `origin/main`.

1. The two increments landed: `git log origin/main
   --grep='^Plan: forgejo-17$'
   --format='%(trailers:key=Step,valueonly,separator=)' | grep -c -x -E
   'temper-02\.[1-2]'` prints 2.
2. The facts of forge.md, section 13 this session checks hold (the
   README's "Done when", item 4): `git show
   origin/main:tools/forgejo-conformance/facts.txt | grep -c -E '^holds
   (job-steps-number-from-0-set-up-first-complete-last|failed-workflow-step-and-completion-step-are-failure|step-log-holds-that-steps-lines-alone|step-log-suffix-range-answers-206-with-its-last-bytes|step-log-suffix-content-range-names-its-first-byte|empty-log-suffix-range-answers-206-empty|step-past-the-last-is-404|compare-files-false-lists-no-commits-files|compare-files-false-keeps-its-own-files-empty|compare-verification-false-changes-nothing-kept|compare-files-false-keeps-total-commits-and-shas)$'`
   prints 11.
3. No other fact fails: `git show
   origin/main:tools/forgejo-conformance/facts.txt | grep -E '^fails ' |
   cut -d: -f1` prints `fails pull-for-names-the-newest-of-several`,
   `fails open-pull-no-changes-is-422`, `fails
   delete-missing-branch-is-404` and `fails
   effective-protection-name-for-a-writer`, and nothing else.
4. The component's facts stayed: `git show
   origin/main:tools/forgejo-conformance/facts.txt | grep -E
   '^(fails|unchecked) component-'` prints nothing, and `git show
   origin/main:tools/forgejo-conformance/facts.txt | grep -c '^holds
   component-'` prints 39 or more.
5. The captures are there: `git ls-tree -d --name-only origin/main
   tools/forgejo-conformance/transcripts/ | grep -c -x -E
   'tools/forgejo-conformance/transcripts/(actions-job|actions-job-log-step|actions-job-log-step-suffix|actions-job-log-past-last-step|actions-job-log-empty-suffix|compare-verification-false|compare-files-false|compare-reverse-files-false)'`
   prints 8.
6. The component sends the flags: `git grep -c -F
   'files=false&verification=false' origin/main --
   crates/temper-engine-protocol-forge/src/tests/contents.rs
   crates/temper-engine-protocol-forge/src/tests/writes.rs` prints two
   lines, each with a count of at least 1.
7. The fake's tally counts 40 routes: `git grep -n -e '39 routes' -e
   'with_capacity(39)' -e 'worst_case(39)' origin/main -- testing` prints
   nothing.
