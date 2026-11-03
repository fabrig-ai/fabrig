+++
id = "questions-panel"
part = "panel"
kind = "feature"
needs = ["relay-surface"]
files = ["temper/crates/temper-engine-domain-forge/src/panel.rs", "temper/crates/temper-engine-domain-forge/src/tests/panel.rs", "temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema", "temper/crates/temper-client-protocol-wire/src/generated/v1.rs", "temper/crates/temper-client-protocol-wire/golden/**", "temper/crates/temper-client-protocol-wire/src/tests/**", "temper/crates/temper-service/src/family.rs", "temper/crates/temper-service/src/tests/family.rs", "temper/crates/temper-client-tools/src/change.rs", "temper/crates/temper-client-tools/src/tests/change.rs", "temper/crates/temper-client-tools/src/tests/golden/**", "temper/docs/help/reference/ctl-change.md", "temper/docs/design/client/README.status.md", "temper/docs/design/client/forge.status.md", "temper/docs/design/client/tools.status.md"]
effort = "high"
minutes = 75
+++

# questions-panel: a change's questions on its panel and in `ctl change show`

## Goal

A change's panel carries its questions (contract C8): its route, the questions taken of its
bound, and its open questions with where each stands, from its record's `questions` (task
`relay-surface`). The wire's `ChangePanel` carries them to clients, and `temper ctl change
show` prints them. Nothing fills a change's open questions yet (task `relay-up` does); the
tests build rows that hold them.

Acceptance:

- `temper-engine-domain-forge/src/panel.rs`: C8's `PanelQuestions`, `PanelQuestion` and
  `PanelStands`, and `Panel.questions: PanelQuestions` after `updates`, made from
  `row.questions` (route; `taken` used of `change_policy.questions`; the first 8 open
  questions in order, a `Held` one shown as `Held` without its words). `Panel::within`
  counts each line's bytes against `QUESTION_LINE`.
- The wire (`client-wire-v1.schema`): C8's enums and records, `questions: Questions`
  appended last to `versioned record ChangePanel`, regenerated with `heavy cargo run -p
  skein-codegen -- temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema
  temper/crates/temper-client-protocol-wire/src/generated`, keeping `golden/**`; the wire's
  fixtures and round trips build a panel with one open question of each stands.
- `temper-service/src/family.rs`'s `panel` maps them field for field.
- `ctl`: `temper-client-tools/src/change.rs`'s `text` prints C8's lines after the repairs
  line and before `digest:`; its `encode` adds C8's JSON member. The goldens
  `change_ready.txt` and `change_preparing.txt` gain `questions: none`; a new golden
  `change_questions.txt` shows a change with route `foreman`, `2 of 4 taken`, one question
  `with foreman 15` and one `taken`.
- `docs/help/reference/ctl-change.md` says, where it lists what `change show` prints, "its
  questions: the route, those taken of its bound, and each open question with where it
  stands" (a stated departure from the implementer's text: this task edits that help page),
  and `help_change_show.txt` follows it if the help golden renders that page.
- Status: remove `client/README.status.md`'s 4.6 entry ("`ChangePanel`'s questions, and a
  hold past its questions"), `client/forge.status.md`'s 9.2 sentence, and
  `client/tools.status.md`'s "5.5, area 6: `change show`'s questions".

## Inputs

- `temper/docs/design/client/README.md`, 4.6 (`ChangePanel`, its questions);
  `temper/docs/design/client/forge.md`, 9.2 ("What it carries"); `temper/docs/design/client/tools.md`,
  5.5, area 6 (`change show`); `temper/docs/design/domain/forge.md`, 8.8 and 8.11 ("On its
  record and its panel").
- `temper/docs/plan/reference-m5/notes/common.md`; `notes/contracts.md`, C2 and C8.

## Owns and may change

Owns the files in the header.

- Readers that build a `Panel` or a wire `ChangePanel` by its fields, edited only to add
  `questions` at its default (route none, nothing taken, none open):
  `temper-client-domain`'s forge change tests and fixtures, `temper-client-view`'s change
  tests, `temper/tests/client/**` fixtures, `temper-engine-domain-forge`'s other tests.
  `rg 'ChangePanel \{|Panel \{' temper` lists them; name each. The web client's page shows
  no questions yet (its design is rung W's): `temper-client-domain` decodes and ignores
  them.
- If the wire's code generator spells an enum variant with a value otherwise, use its
  spelling and say so. If `ChangePanel`'s versioned record cannot take an appended member
  without a new wire version, stop and ask.

## Steps

1. Panel types and their making; wire schema and regeneration; family's mapping; ctl's text
   and JSON; goldens; the help page; the status lines.
2. Tests:
   - `temper-engine-domain-forge/src/tests/panel.rs`: `a_panel_shows_its_questions` (a row
     with route `Foreman`, `taken` 2 and three open questions, one `Held`, gives
     `taken` 2 of the limit and three `PanelQuestion`s, the held one without words);
     `a_panel_shows_at_most_eight_questions`.
   - `temper-service/src/tests/family.rs`: the panel's questions map to the wire's.
   - `temper-client-tools/src/tests/change.rs`: `change_show_prints_its_questions` against
     `change_questions.txt`; the JSON member in the existing JSON test.
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/panel/)'`,
   `heavy cargo nextest run -p temper-client-protocol-wire`, `heavy cargo nextest run -p
   temper-service --lib -E 'test(/family/)'`, `heavy cargo nextest run -p
   temper-client-tools --lib -E 'test(/change/) | test(/help/)'`, `heavy cargo nextest run
   -p temper-docs` (check each count).
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-forge -p temper-client-protocol-wire -p temper-service -p
   temper-client-tools --all-targets -- -D warnings`; `heavy cargo check --tests` for `-p
   temper-client-domain` and `-p temper-client-view`. Commit ("Show a change's questions on
   its panel") and push.

## Report

As the implementer's text says.
