+++
id = "live-report"
part = "live-report"
kind = "feature"
needs = []
files = ["smith/tests/end-to-end/live.rs", "smith/tests/end-to-end/support/live_report.rs", "smith/tests/end-to-end/support/live_tests.rs", "smith/tests/end-to-end/live_configuration.rs"]
effort = "high"
minutes = 90
+++
# A failing live story says why, without credential bytes

## Goal

Every failing `smith::live` story today ends in a message that withholds its reason ("live exec
succeeds; diagnostics withheld", "run reports the expected resume state", "checkout file matches
its prescribed content", "the issuer refresh was kept; generations withheld"), so four standing
reds of the after-landing pair cannot be diagnosed. Make each such failure carry a bounded,
credential-free report of typed facts, built by one pure module that offline tests check.

Acceptance:

- `smith/tests/end-to-end/support/live_report.rs` exists with the surface in
  `smith/docs/plan/live-smith-reds/notes/contracts.md` ("The live report"), compiled into the
  `live` and `live_configuration` binaries.
- Offline tests in `live_configuration` (module `live_tests`) show, red before the module exists
  and green after:
  - a failed exec's typed end reaches the report: synthetic records with a `RunCompleted` of
    status `failed` and a provider failure (class, completion class and status) and an
    `End::Failed`, give a report naming the leg, the failed assertion, the exit code, the end and
    those typed fields;
  - nothing private passes: planted `detail`, `request_id`, a prompt's text, a tool input and a
    standard-error line outside the allow-list appear nowhere in the report; the report is at
    most 4096 bytes even with many records;
  - `file_report` gives each relation of the contract and never the files' bytes;
  - `stderr_lines` keeps the allow-listed lines and counts the rest.
- Every assertion of `smith/tests/end-to-end/live.rs` that can fail after a `smith` process ran
  carries the report as its message (see Steps); the refresh story's message also gives the
  generation before and after (numbers, not credentials).
- fmt, clippy and the focused tests below are green. The live stories themselves run only after
  landing.

## Inputs

- `smith/docs/plan/live-smith-reds/notes/common.md`, then `notes/contracts.md`.
- `smith/docs/design/testing.md`, 2.3 ("The referee reads what shows from outside").
- `smith/docs/design/protocol/events.md`, for the records' fields (read the ones the contract
  names, not the whole document).
- What exists: `smith/tests/end-to-end/live.rs` (`run`, `assert_exec`, `change`, the stories);
  `support/live_run.rs` (the `Judge`'s allow-list: `trust_notice`, `unused_preset_notice`);
  `smith_bench::process::ProcessResult` (`smith/benchmarks/src/process.rs`; it derives `Debug`,
  which prints stdout and stderr: never format it), `smith_bench::End` and `FailureReason`
  (`smith/benchmarks/src/result.rs`), the observer's `sessions()`, `finish()` and `classify`
  (`smith/benchmarks/src/agent/smith/events.rs`), `smith_events::Record` and its events
  (`smith/crates/smith-events/src/records.rs`; the crate's `fixture` feature is on for smith's
  tests, so records can be built directly).

## Owns and may change

- Owns `smith/tests/end-to-end/live.rs`, the new `smith/tests/end-to-end/support/live_report.rs`,
  `smith/tests/end-to-end/support/live_tests.rs` (new tests), and in
  `smith/tests/end-to-end/live_configuration.rs` only the `#[path]` module line for
  `live_report`.
- Not `support/live_run.rs`: the terminal `Judge` keeps its messages (no live story uses it).
- Not `smith/benchmarks` nor `smith/crates/smith-events`: if an enum has no way to be named
  without `Debug` of an `Unknown` variant's bytes, write the `match` in `live_report.rs`.
- Pre-approved:
  - If building a `ProcessResult` is needed anywhere in a test, don't: tests build `Settled`
    directly; `Settled::of` is exercised by the `live` binary's compilation.
  - If an enum the contract names has many variants, a `match` to `&'static str` names in
    `live_report.rs` is the expected form; a wildcard arm is allowed in this test harness for an
    enum smith adds to (`#[non_exhaustive]`), written as `unknown`.
  - If the report needs a fact the contract does not list and it is a typed enum or number of
    a record the contract names, add it and say so; anything carrying bytes from a provider, a
    prompt or a file is a question.
  - If `live.rs`'s `run` must return something for the refresh story, return the completed
    leg's report `String` (the same `exec_report`, `failed` = `"completed"`), and have `change`
    pass it on.

## Steps

1. Write the `live_tests` tests against the contract's signatures; see them fail to compile (the
   red); note it.
2. Write `live_report.rs` (module doc citing testing.md 2.3, saying it keeps no state and prints
   no bytes from a provider, a prompt or a file); add the `#[path = "support/live_report.rs"]`
   module to `live.rs` and `live_configuration.rs`.
3. Wire `live.rs`:
   - `run` takes a `Leg` (story name, account, invocation number); the chat-crossing loop passes
     1, 2, 3; `a_second_run_resumes_the_chat_from_its_files` passes 1 and 2;
   - the check step: `live check must pass` becomes `exec_report` with `Step::Check`, no records,
     no end, plus the account's `credentials.<account>.state` tag read from `check --json`'s
     stdout (the tag only; never its `detail`);
   - `assert_exec`: take the records and the classification first (a `finish()` fault becomes the
     report's `error` line, not a bare `expect`), then every `assert!` there uses
     `exec_report(...)` with its own words as `failed`; each file comparison uses `file_report`;
   - the chat crossing's final file loop uses `file_report`;
   - `a_run_refreshes_its_grant_at_the_issuer`: its message gives the generation before and
     after and the completed run's report.
   Never put a `ProcessResult`, a record or stdout in a message.
4. `heavy cargo nextest run -p smith --test live_configuration -E 'test(/^live_tests::/)'`
   (selected count not zero); `heavy cargo check -p smith --tests` (the `live` binary compiles).
5. `cargo fmt`; `heavy cargo clippy -p smith --all-targets -- -D warnings`;
   `heavy cargo nextest run -p smith --test live_configuration`;
   `heavy cargo nextest run -p smith --test end_to_end`.
6. Commit, push.

## Report

The red of step 1, a sample report from the offline test (the synthetic one: it holds nothing
private), the stories' failure messages as they now read, the tests run (selected, passed), and
implementation and check minutes apart.
