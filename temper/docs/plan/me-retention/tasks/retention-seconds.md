+++
id = "retention-seconds"
part = "me-retention"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-tools/src/jig/render/document.rs",
  "temper/crates/temper-client-tools/src/tests/jig/reads.rs",
  "temper/crates/temper-client-tools/src/tests/golden/me.txt",
]
effort = "medium"
minutes = 25
+++

# `me` prints its retention in whole seconds

## Goal

`temper ctl DIR me` prints `retention: 86400000000000 ns` (QA finding 13). Print it in
whole seconds, the form ctl already prints a duration in: `retention: 86400 seconds`, as
the help prints `at most 3600 seconds` and `--timeout`'s `300 seconds when absent`. A
remainder below a second is dropped, as the help's bound drops it.

Acceptance:

- A `me` whose retention is `Duration::from_secs(86_400)` prints the line
  `retention: 86400 seconds`, and no ` ns` anywhere in its text.
- A retention of `Duration::from_nanos(86_400_999_999_999)` prints `retention: 86400 seconds`.
- `src/tests/golden/me.txt` says `retention: 1000 seconds` (the fixture's 1000 s) and is
  otherwise unchanged; `me_json.txt` is byte-identical (JSON keeps `"retention"` in
  nanoseconds).
- `temper-client-tools` lib tests pass; clippy is clean.

## Inputs

- `temper/docs/plan/me-retention/notes/common.md`.
- `temper/docs/design/client/tools.md, 5.2` (the `me` row: it prints the retention) and
  `6.1` (text: plain lines). The design names no form for the retention's line; do not edit
  it; the report says the line is now `retention: N seconds`.
- The form reused, ctl's printed duration: `temper/crates/temper-client-tools/src/jig/help.rs`,
  `Bound::Seconds` (`, at most N seconds`, whole seconds by `checked_div(1_000_000_000)` in
  `src/jig/table.rs`), and the `--timeout` descriptions in `src/jig/rows.rs`. There is no
  shared function to call: the form is a decimal number of whole seconds then ` seconds`.
  Write no "N days"/"N hours" renderer: the request asks for the form ctl already uses.

## Owns and may change

Owns, in `temper/crates/temper-client-tools`:

- `src/jig/render/document.rs`: in `me_text`, the three lines that print the retention
  (now `out.put(b"\nretention: "); out.number(me.retention.as_nanos()); out.put(b" ns\n");`)
  and nothing else of the file. `me_text` is also how a met wait prints `me`
  (`render/document/waiting.rs`), so both change together; that is intended.
- `src/tests/jig/reads.rs`: the new test below, beside `me_prints_the_live_frame`.
- `src/tests/golden/me.txt`: regenerated, never hand-edited.

Read only, for the fixture: `src/tests/fixtures.rs` (`me`, `started`, `EMPTY`, `ME_CALL`,
`PARTY`, `Harness::deliver/submit/answer`). Change nothing there.

Pre-approved options:

- If clippy objects to `/ 1_000_000_000` (`arithmetic_side_effects`), use
  `.checked_div(1_000_000_000).expect("nonzero divisor")` as `src/jig/table.rs` does.
- If another golden or test in this crate holds `retention: ... ns` (the survey found only
  `me.txt`), regenerate it with its owning test under `SKEIN_CODEGEN_WRITE=1`, check its
  diff is that line alone, and list it in the report.
- If a test in another package asserts the old line (none found by `rg 'retention: '`),
  update that line only and list it.

## Steps

1. `me_text`: print the retention in whole seconds. Sketch (the shape inside is yours):

   ```rust
   out.put(b"\nretention: ");
   out.number(me.retention.as_nanos() / 1_000_000_000);
   out.put(b" seconds\n");
   ```

2. `src/tests/jig/reads.rs`, after `me_prints_the_live_frame`:

   ```rust
   /// `me` prints its retention in whole seconds, as ctl's help prints a duration (QA finding 13).
   #[test]
   fn me_prints_its_retention_in_whole_seconds() {
       for retention in [
           skein_lib::Duration::from_secs(86_400),
           skein_lib::Duration::from_nanos(86_400_999_999_999),
       ] {
           let mut root = fixtures::started(skein_lib::List::with_capacity(fixtures::EMPTY));
           let mut me = fixtures::me(fixtures::PARTY);
           me.retention = retention;
           root.deliver(jig_client::contract::Delivery::Me(Box::new(me)));
           let call = root.submit(fixtures::ME_CALL);
           let answer = root.answer(call).expect("me answered from the live frame");
           assert_eq!(answer.status, 0);
           // the line `\nretention: 86400 seconds\n` is in answer.stdout
           //   (skein_lib::bytes::find), and ` ns` is not
       }
   }
   ```

   No `Vec`, `format!` or `String` (the tests' `clippy.toml`).

3. Regenerate the golden:
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-tools --lib -E 'test(=tests::jig::reads::me_prints_the_live_frame)'`
   (check the path with `--list` first; a run that selects nothing proves nothing). The
   test writes both `me.txt` and `me_json.txt`: `git diff --stat` must show `me.txt` alone,
   its line 6 alone changed to `retention: 1000 seconds`.

4. Narrow tests: the new test and `me_prints_the_live_frame` alone, without
   `SKEIN_CODEGEN_WRITE`; then `cargo fmt`;
   `heavy cargo clippy -p temper-client-tools --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-tools --lib`. The text `me` prints changes
   meaning for its readers: check again that no other package's test reads the line
   (`rg -n 'retention: ' --glob '!temper/docs/plan/**'`); if one does, run its package's
   tests too.

5. One commit for the render, its test and the golden; push your branch.

## Report

As the implementer's text says, and also: the shape chosen for the division; the goldens
regenerated; and, under `## Left`, that a refused worker hello's stop bound still prints
`allowed grace N ns` (`src/jig/render/document/llm.rs`, `HelloRefusal::StopBound`), the
same raw form, outside this task.
