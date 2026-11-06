+++
id = "roots-quiet"
part = "roots-quiet"
kind = "feature"
needs = []
files = [
  "smith/crates/smith-agent-shell/src/config/native.rs",
  "smith/crates/smith-agent-shell/src/config/tests/native.rs",
  "smith/tests/end-to-end/live_tests.rs",
  "smith/tests/end-to-end/support/live_run.rs",
  "smith/tests/end-to-end/support/live_report.rs",
  "smith/tests/local-process/src/referee/live.rs",
]
effort = "medium"
minutes = 30
+++

# Smith's trust-roots line says nothing on a healthy machine

## Goal

`smith: trust roots: 120 taken, 1 skipped, 0 malformed blocks, 0 other blocks` prints at each start,
twice in `temper program-login`, and nobody acts on it. Print it only when a block is malformed.

Acceptance:

- With no malformed block, `trust_sources` writes nothing to its writer, whatever was skipped and
  whatever other blocks there were.
- With a malformed block, it writes the one line as it stands, all four counts.
- `smith`'s and `temper`'s tests pass with the line gone, and the live tiers' allow-lists no longer
  admit a line with `0 malformed blocks`.

## Inputs

- `temper/docs/plan/first-usable-model/notes/common.md`.
- `skein/docs/design/shell.md`, 6.2, *Startup says nothing of its roots unless a block is malformed*.
- `temper/docs/design/protocol/credentials.md`, section 3, `temper program-login`'s failure line.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper/tests/suite/fake.rs`, `check_run`, about line 1322: drop the line from the
  expected stderr, nothing else.
- `temper/crates/temper-box/src/tests/logs.rs`: the three sample strings at about lines 319, 358 and
  366 read `0 malformed blocks`; make each `1 malformed blocks`, so that they say what can still be
  printed, and leave the tests' shape.

Options, each pre-approved:

- **Any other service printing these counts** (`rg 'malformed blocks'`): if one exists in skein,
  smith or temper, give it the same rule and say so in the report.
- **`trust_notice`'s three copies:** tighten each in place. Don't share them: they sit in three test
  crates.

## Steps

1. In `smith/crates/smith-agent-shell/src/config/native.rs`, `trust_sources`, guard the `writeln!`:

   ```rust
   if malformed > 0 {
       drop(writeln!(errors, "smith: trust roots: {} taken, {} skipped, {} malformed blocks, {} other blocks", ...));
   }
   ```

2. In `smith/crates/smith-agent-shell/src/config/tests/native.rs`:
   - `resolving_reports_trust_only_to_the_supplied_writer` becomes
     `resolving_says_nothing_of_well_formed_roots`: the same document, `errors` empty.
   - New `a_malformed_block_is_reported_once_to_the_supplied_writer`: a `trust` array with
     `tests/protocol-llm/fixtures/root.der` in `der`, and in `pem` a scratch file holding one
     certificate block whose body isn't a certificate, as
     `trust_arrays_keep_one_combined_certificate_bound` writes one. If `skein-shell`'s reader counts
     that block as a certificate that `skein-tls` skips rather than as malformed, use a body that isn't
     base64. Assert the whole line, `1 malformed blocks`, once.
3. `trust_notice` in `smith/tests/end-to-end/support/live_run.rs`,
   `smith/tests/end-to-end/support/live_report.rs` and `smith/tests/local-process/src/referee/live.rs`:
   the malformed count must parse and be above zero.
4. `smith/tests/end-to-end/live_tests.rs`: keep the allowed sample (it has `1 malformed blocks`); add
   to `rejected` `"smith: trust roots: 12 taken, 0 skipped, 0 malformed blocks, 0 other blocks"`.
5. The readers named above.

Tests, through `heavy`, one command at a time:

- `heavy cargo nextest run -p smith-agent-shell --lib -E 'test(native)'`
- `heavy cargo nextest run -p temper --test process -E 'test(check_run)'`
- `heavy cargo nextest run -p temper-box --lib -E 'test(refusal)'`
- `smith/tests/end-to-end/live_tests.rs`' allow-list test, by name, in its package.

Then fmt, `old_forms.py` for smith and temper, clippy on the packages you changed, their focused tests,
commit and push.

## Report

As the implementer's text says, and the `rg` you ran for other printers.
