+++
id = "charter-push"
part = "charter"
kind = "feature"
needs = []
files = ["temper/crates/temper/src/tests/reference_settings.rs"]
effort = "medium"
minutes = 20
+++

# charter-push: the reference coding charter pushes as it goes

## Goal

The reference's coding agent's charter (charter 2 of
`temper/docs/examples/reference/settings.json`) already tells its producer to push as it
goes (N14): deliver a first head as soon as the change builds, then again as it grows. Pin
it, so a later edition of the file cannot drop it unseen.

Acceptance:

- `the_reference_settings_file_decodes` (`temper/crates/temper/src/tests/reference_settings.rs`)
  also asserts that the coding charter's instructions contain `Push as you go: deliver a
  first head as soon as the change builds, then again as it grows.`, and passes.

## Inputs

- `temper/docs/design/domain/agent.md`, section 8 (the producing agent's row: `deliver`
  mid-run, as it goes).
- `temper/docs/plan/reference-m6/notes/common.md`.

## Owns and may change

Owns the file in the header. The settings file and the stock charter's text are not this
task's: if the sentence is not in the file as quoted, stop and ask.

## Steps

1. Add the assertion beside the one on `coding.txt`.
2. `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper --lib -E 'test(=tests::reference_settings::the_reference_settings_file_decodes)'`
   (check that it selected one test; find its full name with `--list` if not). Commit, push.

## Report

As the implementer's text says.
