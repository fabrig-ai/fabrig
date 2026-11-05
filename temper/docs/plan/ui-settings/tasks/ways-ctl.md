+++
id = "ways-ctl"
part = "ways-ctl"
kind = "feature"
needs = ["ways-kind"]
files = [
  "temper/crates/temper-settings-form/src/**",
  "temper/crates/temper-client-tools/src/settings.rs",
  "temper/crates/temper-client-tools/src/settings/value.rs",
  "temper/crates/temper-client-tools/src/jig/rows.rs",
  "temper/crates/temper-client-tools/src/tests/jig/settings.rs",
  "temper/crates/temper-service/src/clients/translation.rs",
  "temper/crates/temper-service/src/stock/configuring.txt",
  "temper/crates/temper-engine-smith/src/settings.rs",
  "temper/crates/temper-engine-smith/src/calls.rs",
  "temper/crates/temper-engine-smith/src/tests/settings.rs",
  "temper/docs/help/settings.md",
  "temper/docs/help/project.md",
  "temper/docs/help/reference/**",
  "temper/docs/examples/reference/project.json",
  "temper/crates/temper/src/tests/reference_settings.rs",
  "temper/tests/client/tests/suite/settings_tools.rs",
]
effort = "high"
minutes = 110
+++

# Kinds 6 and 7 through the entrances, `temper ctl`, the configuring agent and the reference

## Goal

A person's or an agent's item of kind 6 or 7 reaches the root; `temper ctl settings show` prints them; a settings
file sets them; the configuring agent knows them; the reference's project shows its goals.

Acceptance:

- **The entrances:** `notes/contracts.md`, *The entrances*: all three take 6 and 7. One function says which kinds
  are temper's, `temper_engine_smith::kinds::typed(kind: u32) -> bool`, and the ranges call it; so a next kind
  has one place.
- **ctl:** `settings show project-ways PROJECT` and `settings show project-questions PROJECT`, a project by its
  number or name as `landing-rule`'s `--project` resolves it, with `--revision` and `--json` as `forge-charter`'s; the
  value is the stored one, or the stock's with none, and says "the stock" so (`temper/docs/design/client/tools.md`, 4).
- **The file:** `read_setting` and `value` take `notes/contracts.md`, *Their names*: a `number` member is refused, `project` is
  required; the ways' five members all required, unknown ones refused; `way` a u32. `FIELDS` gains the two shapes and
  `settings-fields` regenerates.
- **The inventory and the review:** `settings show` and a revision's or proposal's items name them `project-ways P` and
  `project-questions P`, never "application kind 7".
- **The agent's tool:** `settings`' schema (`calls.rs`) and its reader in `temper-engine-smith/src/settings.rs` take both kinds,
  the `project` member allowed and required for them, `number` refused, as `project-policy`'s.
- **The stock's words,** `configuring.txt`: kind 7 and 6 among the structure it may propose, in the agent's terms: what the
  project shows and how its questions go "follow from how the person works". A person who plans big work gets Goals and drafts
  shown; who watches a landing queue, Changes; who steps away and lets agents go on, the Away switch; `way` the way a goal starts
  by, a template of the project's with `tracked` set, or 0 for the first. Its first paragraph's list of structure names them. The
  stock's `chat.txt` and its other tests move only if a test holds their bytes.
- **The documentation:** `temper/docs/help/settings.md` names both in its list of what a setting is named by, and what each does, in
  the voice of the page; `temper/docs/help/project.md` says what a project shows comes from there.
  Run `temper-docs`' tests and `temper`'s help tests.
- **The reference:** `temper/docs/examples/reference/project.json` by `protocol/configuration.md`, 13.9: `{"items": [{"setting": {"kind":
  "project-ways", "project": 1}, "set": {"drafts": true, "goals": true, "changes": true, "away": true, "way": 4}}]}`. A test in
  `temper/src/tests/reference_settings.rs` reads it through the settings form. `settings.json` doesn't change.
- `protocol/configuration.status.md`: its *13.9* on `project.json` goes.
- The tests below pass and bite.

## Inputs

- `temper/docs/plan/ui-settings/notes/common.md`; `temper/docs/plan/ui-settings/notes/contracts.md`, *Their names*, *The entrances*.
- `temper/docs/design/client/tools.md`, 4 (`settings show`'s rows) and 5.3 (the file).
- `temper/docs/design/protocol/configuration.md`, 13.2, 13.4, 13.9. `temper/docs/design/domain/agent.md`, 8.1, the configuring agent.
- `temper/docs/design/protocol/documentation.md`, 2 and the row of tests in its table.

## Owns and may change

Owns `files`. May also change:

- **Tests that hold the stock's bytes or the kinds' lists,** each only for the words: `temper-service/src/tests/stock.rs`,
  `temper-service/src/tests/stock_brief.rs`, `temper/src/tests/help.rs`, `temper-engine-smith/src/tests/kinds.rs`.
- **Design status:** `temper/docs/design/protocol/configuration.status.md`, `temper/docs/design/client/tools.status.md`, for the lines this closes.
- **`docs/help/**`:** a departure from the implementer's text: this task writes the pages it names, and regenerates the rest.

Pre-approved:

- If a `Shape` needs a `Choice` the table's render doesn't have for `"none"|"foreman"|"up"`, use `Value::Words`, as a kind's own words.
- `settings show` of a project unknown or not the person's: as `landing-rule --project` says it, whatever that is.
- If a `settings_tools` world asserts the kinds' list verbatim, change the literal.

## Steps

1. `kinds::typed` and the three call sites. A step test in `temper-service`'s clients' translation: a change with an item of kind 7
   from a client reaches the root typed; and in `temper-engine-smith`'s: an agent's proposal of kind 6.
2. The form: `kinds.rs` (`setting`, `value`, `render`), `file.rs`, `render.rs` (`setting_text`, `encode_setting`), `fields/*` and `fields/tests.rs`.
   `settings_file_tests.rs`: both good and bad shapes, `project_ways_is_read_and_printed_whole`.
3. ctl: `settings.rs` and `settings/value.rs`, `jig/rows.rs`. World, `temper/tests/client/tests/suite/settings_tools.rs`:
   `settings_show_prints_a_projects_ways_and_questions` (the stock, then set, then `--json`, then a file applied);
   `a_settings_file_changes_a_projects_questions_and_a_change_takes_them`.
4. The agent's tool; `configuring.txt`; help; `project.json` and its test.
5. Regenerate `temper/docs/help/reference/*` by the test that prints them, then `cargo fmt`; old forms; clippy and nextest of every package you
   changed and their direct dependents (`temper-docs` and `temper` among them), `heavy cargo check -p temper --tests`. Commit and push.

## Report

As the implementer's text says, with the words you gave the configuring agent, for the reader to check.
