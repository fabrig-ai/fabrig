+++
id = "report-lines"
part = "report-lines"
kind = "feature"
needs = []
files = ["smith/tests/end-to-end/support/live_report.rs", "smith/tests/end-to-end/support/live_tests.rs"]
effort = "medium"
minutes = 45
+++
# The live report keeps the host's agent-failure line and names an unknown notice's kind

## Goal

In the whole after-landing run at main 46e6e70d9, the chat crossing's Claude leg failed so:

```
story=a_chat_crosses_from_chatgpt_to_claude_then_back account=claude invocation=2 step=Exec
failed=live exec succeeds
exit=Code(1) forced=No live_pids=0  error=none  end=Failed reason=Agent
stderr: bytes=393 lines=5 kept=3 omitted=2
session=2 run.completed status=Failed turns=0 failure.class=Agent ended=Signal(9)
session=1 session.started / session=1 notice kind=unknown
session=2 session.started / session=2 notice kind=unknown / session=2 session.ended exit=1 answered=true
```

The spawned `smith-agent` was killed by signal 9 before its run started, and the report cannot say
by whom: the host prints `run failed: the agent <fixed words> (<exit>)[; terminated|; killed]:
<detail>` on standard error (`smith/crates/smith-local-protocol/src/terminal.rs`, the
`local::Failure::Agent` arm; `terminal/failure.rs`, `end` and `detail`), and `allowed_line` in
`support/live_report.rs` drops it. `; killed` tells a host-forced kill (with the fault in the fixed
words) from an outside one. Every report also shows `notice kind=unknown`: the session-start
warning is a notice kind `smith-events` does not know (`NoticeKind::Unknown`, read from
`not_contained`), and `notice_kind` drops its name.

Make the report keep the agent-failure line without its detail, and name an unknown notice by its
kind, as smith/docs/design/testing.md, 2.3 ("A failure says why, in typed facts") now says.

Acceptance:

- `stderr_lines` keeps, for each `AgentFailure`/`Fault` wording and each exit form the terminal
  writes (`(exit N)`, `(signal N)`, `(exit not observed)`, `(not spawned)`, `(inline run
  settled)`, each with no suffix, `; terminated` or `; killed`), the line cut before its `: `
  detail, and drops a line whose words or numbers are not of those forms.
- `notice kind=<kind>` names an unknown kind when it is a bounded identifier (at most 32 bytes of
  `a-z`, `0-9` and `_`), and prints `unknown` otherwise; a known kind prints as today.
- `live_tests.rs` checks both, and the existing report tests stay green.

## Inputs

- `smith/docs/plan/live-smith-reds-2/notes/common.md` (read it first).
- `smith/docs/design/testing.md`, 2.3 ("A failure says why, in typed facts").
- `smith/docs/design/protocol/events.md`, the notices section (find `not_contained`).

## Owns and may change

- `smith/tests/end-to-end/support/live_report.rs`: `allowed_line`, `stderr_lines` (a kept line
  may be the cut form of the line, not the whole line), `notice_kind`, and new private helpers.
- `smith/tests/end-to-end/support/live_tests.rs`: new tests and new rows in
  `stderr_keeps_only_complete_allow_listed_forms`.
- Pre-approved:
  - If the terminal prefixes the line other than `smith: ` on standard error, match what it
    writes (read `smith/src/main.rs` and the exec path that prints terminal lines) and say so.
  - If `NoticeKind::Unknown` holds no text you can read, keep `unknown` and say so.
  - If cutting a kept line breaks the report's 4 KiB bound test, keep the bound and say which
    lines were omitted.

## Steps

1. Read the inputs, `live_report.rs` (`stderr_lines`, `allowed_line`, `budget_line`,
   `notice_kind`) and `live_tests.rs`.
2. Tests first, run red: in `stderr_keeps_only_complete_allow_listed_forms` (or a new test beside
   it, `stderr_keeps_an_agent_failure_without_its_detail`), lines such as
   `smith: run failed: the agent ended without answering (signal 9); killed: private tail` kept as
   `smith: run failed: the agent ended without answering (signal 9); killed`, and rejected forms
   (unknown words, a non-number exit, a suffix other than the two); and
   `an_unknown_notice_is_named_by_its_kind`: a record whose notice kind is unknown `not_contained`
   reports `notice kind=not_contained`, one whose kind text is 40 bytes or holds `-` or a space
   reports `notice kind=unknown`. Build the records as `live_tests.rs` builds its synthetic records.
   `heavy cargo nextest run -p smith --test live_configuration -E 'test(/stderr|notice/)'`: red.
3. Implement in `live_report.rs`; rerun step 2 green.
4. `cargo fmt`; `heavy cargo clippy -p smith --all-targets -- -D warnings` (add
   `--features end-to-end` if the end-to-end targets are not checked without it).
5. `heavy cargo nextest run -p smith --test live_configuration`, then
   `heavy cargo nextest run -p smith --features end-to-end --test end_to_end -E 'test(/live/)'`.
6. Commit, push.

## Report

The red run of step 2, the files changed, the line prefix you matched, the test runs (selected,
passed), and implementation and check minutes apart.
