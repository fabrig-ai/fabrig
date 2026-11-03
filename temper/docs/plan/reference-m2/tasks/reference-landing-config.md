+++
id = "reference-landing-config"
part = "reference"
kind = "docs"
needs = []
files = ["temper/docs/examples/reference/settings.json", "temper/docs/examples/reference/landing.json", "temper/crates/temper-service/src/tests/reference_landing.rs", "temper/crates/temper/src/tests/reference_settings.rs"]
effort = "medium"
minutes = 30
+++

# reference-landing-config: the coordinator lands its goal; main's rule asks CI alone

## Goal

The reference's configuration, second edition: the coordinator's charter (charter 5) makes
the goal's landing L once its plan is done, handles L's escalations and makes changes into
its goal branch while L is held; main's landing rule asks CI passed and up to date,
enforced, with no gate beyond CI. `temper/docs/plan/reference-m2/notes/settings.md` holds
the text and the values.

Acceptance:
- `settings.json`'s charter 5 instructions are the note's: the first four paragraphs as
  they are, the fifth replaced by the note's six, byte for byte; nothing else of the file
  changes.
- `landing.json`'s rule 1 (main's) has `"gates": []`; nothing else of the file changes.
- `the_reference_landing_member_parses` expects main's rule with no gates; the settings
  test passes and asserts charter 5's instructions contain `"branch": "mine"` and end with
  the note's last paragraph.

## Inputs

- `temper/docs/design/draft/reference-code-production.md`, 12 (the landing, L) and 14 (the
  coordinator's charter; the landing rule for main).
- `temper/docs/design/draft/reference-landing.md`, 3 (L's way, its bounds and what the
  coordinator does on its hold).
- `temper/docs/design/domain/agent.md`, 4.3.1 (`branch`, `landing`).
- `temper/docs/plan/reference-m2/notes/settings.md`.
- `temper/docs/plan/reference-m2/notes/common.md`.

## Owns and may change

Owns the files in the header. This task departs from the implementer's text in one point:
`temper/docs/examples/` is configuration, not design or help, and is this task's to edit.

- The instructions are one JSON string: escape the note's quotes (`\"branch\": \"mine\"`)
  as the file's existing strings escape theirs; keep its formatting (indentation, key order).
- `reference_settings.rs`: add the two assertions on charter 5's instructions; no other
  change.
- If a test elsewhere reads these two files (`rg 'examples/reference'`), run it; a test
  that pins the old gate or text takes the new value, and the report names it.

## Steps

1. Edit the two files and the two tests.
2. Narrow: `heavy cargo nextest run -p temper-service --lib -E
   'test(=tests::reference_landing::the_reference_landing_member_parses)'` and `heavy cargo
   nextest run -p temper --lib -E
   'test(=tests::reference_settings::the_reference_settings_file_decodes)'` (adjust the
   paths to the modules' real ones; check each selected 1).
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-service -p temper --all-targets -- -D warnings`. Commit ("reference: the
   coordinator lands its goal; main asks CI alone") and push.

## Report

As the implementer's text says.
