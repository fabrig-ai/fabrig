+++
id = "charter-checks"
part = "checks"
kind = "docs"
needs = []
files = ["temper/docs/examples/reference/settings.json", "temper/crates/temper/src/tests/reference_settings.rs"]
effort = "medium"
minutes = 30
+++

# The reference's coding agent runs its lints and focused tests before delivering

## Goal

Charter 2 in the reference's settings file takes smith's shell family, and its instructions say,
in place of the stock's line that it can run nothing, that before each delivery it runs the
lints and the focused tests of each package it changed, as
`temper/docs/design/protocol/configuration.md`, 13.9 (charter 2) has it. `.temper/pre-pr`
stays as it is.

Acceptance:

- In `temper/docs/examples/reference/settings.json`, charter 2's item (`"setting": {"kind":
  "charter", "number": 2}`): `"shell": true`; and in its `instructions`, the sentence `You can
  run nothing.` (in its first paragraph, taken from the stock's `coding.txt`) replaced by
  sentence **S** below, exactly. Nothing else in the file changes: charters 5, 6 and 7, the
  template and the rules stay as they are, and charter 2's other members and paragraphs too.
- `temper/crates/temper/src/tests/reference_settings.rs`, in
  `the_reference_settings_file_decodes`: the coder's assertion that its instructions start
  with the stock's `coding.txt` becomes that they start with the stock's `coding.txt` with
  `You can run nothing.` replaced by S (built in the test from `include_bytes!` of the stock
  text, the stock sentence and S, as `const`s or locals); it asserts `coder.shell()`; its
  other assertions stay.
- `heavy cargo nextest run -p temper --lib -E 'test(=tests::reference_settings::the_reference_settings_file_decodes)'`
  passes, one test selected (find its exact path with `--list` if that name selects nothing).

**S**, exactly (in the JSON string, each `"` escaped as `\"`; a single sentence group, no
paragraph break):

> Before each delivery, run with the shell the lints and the focused tests of each package
> you changed: `cargo clippy -p PKG --all-targets -- -D warnings`, then `cargo nextest run -p
> PKG`, naming each package; fix what they find in your own files, and list a failure outside
> them under Left.

## Inputs

- `temper/docs/plan/reference-m10/notes/common.md`.
- `temper/docs/design/protocol/configuration.md`, 13.9 (charter 2) and section 11 ("What the
  project's checks run at each push").

## Owns and may change

- Owns the two files of this spec's header, for what acceptance names. Task `charter-close`
  edits charter 5's item and the coordinator's assertions in the same two files at the same
  time: leave those lines alone; a conflict at fab's merge is fab's.
- If the settings form refuses `"shell": true` for charter 2 (a rule the form or jig holds),
  stop and ask, quoting the refusal.

## Steps

1. Edit the JSON; check it with `python3 -I -c 'import json,sys;
   json.load(open(sys.argv[1]))' temper/docs/examples/reference/settings.json`.
2. The test's assertions; run it as acceptance says.
3. `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D warnings`; commit and push.

## Report

As the implementer's text asks.
