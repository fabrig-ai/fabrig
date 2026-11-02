# live-smith-reds: binding shapes

## The live report (task `live-report`)

A failing `smith::live` story says why in its panic message, from typed facts only
(smith/docs/design/testing.md, 2.3: the referee reads exit codes, standard error and the event
stream "read as smith-events records, never as text"). New module
`smith/tests/end-to-end/support/live_report.rs`, compiled into the `live` binary and the
`live_configuration` binary (by `#[path]`, as `support/live.rs` is), with this public surface:

```rust
/// Which story, account and invocation a report is about.
pub struct Leg<'a> {
    pub story: &'a str,
    pub account: &'a str,
    /// 1-based, in the story's order of `smith` invocations.
    pub invocation: usize,
    pub step: Step,
}

pub enum Step { Check, Exec }

/// The settled process's content-free evidence, taken from a `smith_bench::process::ProcessResult`.
pub struct Settled<'a> {
    pub exit: smith_bench::Exit,
    pub forced: smith_bench::Forced,
    /// `ProcessResult::error`, a harness fault in fixed words.
    pub error: Option<&'a str>,
    pub live_pids: usize,
    pub stderr: &'a [u8],
}

impl<'a> Settled<'a> {
    pub fn of(result: &'a smith_bench::process::ProcessResult) -> Self;
}

/// One bounded report (at most 4096 bytes) naming the leg, the failed assertion and the typed facts.
pub fn exec_report(
    leg: &Leg<'_>,
    failed: &str,
    settled: &Settled<'_>,
    records: &[&smith_events::Record],
    end: Option<&smith_bench::End>,
) -> String;

/// A prescribed file's mismatch, as a relation between expected and found, never their bytes.
pub fn file_report(path: &[u8], expected: &[u8], found: Option<&[u8]>) -> String;

/// Standard error's byte and line counts, and only its lines of a fixed, content-free form.
pub fn stderr_lines(stderr: &[u8], account: &str) -> (usize, usize, Vec<String>);
```

What a report may hold, and nothing else:

- the leg, the failed assertion's words, `exit`, `forced`, `error`, the live pid count;
- standard error's byte and line counts and its allow-listed lines: those `live_run.rs`'s `Judge`
  accepts (`smith: local host started`, `smith: agent started`, the process-group line,
  `smith: chat chat`, `trust_notice`, `unused_preset_notice`), the budget line
  `smith: run failed: out of {turns|spend|time}: used N of M` with N and M numbers, and the
  credential lines smith prints from
  `smith_local_shell::credentials::Credential::{NotSignedIn, SignInAgain, ProgramNotSignedIn { .. }}.line(account)`,
  under the prefix smith prints them with (as `unused_preset_notice` matches its warning);
- the `smith_bench::End` variant, with `FailureReason` or `which` or `what`, never `detail` or
  `setup`;
- per session: its `RunStarted` `resumed`, `messages` and backend; its `RunCompleted` `status`,
  `turns`, and `failure`'s typed fields (`class`, `attempts`, `state`, `exit`/`ended`, `used`,
  `bound`, `turn`, `completion.class`, `completion.evidence`, `completion.phase`,
  `completion.status`), `which` and `reason` (smith's own words), never `detail`, `code` or
  `request_id`; `ConversationClosed.end`; each `Notice`'s `kind`; each `ToolCompleted`'s tool
  name (only `finish`, `deliver` or a built-in's), `verdict` and `delivery`;
  `CheckCompleted`'s `exit` and `passed`; `SessionEnded`'s `exit` and `answered`;
- an enum is written by its variant name, an `Unknown(..)` variant as `unknown`; never a
  record's, a `ProcessResult`'s or an `Unknown` variant's `Debug`, never stdout, a prompt, a
  completion, a tool input or result.

`file_report`'s relation is one of: `missing`, `empty`, `expected without its final newline`,
`expected followed by N more bytes`, `a prefix of expected (N of M bytes)`,
`differs at byte N (expected M bytes, found K)`, each with the escaped path.
