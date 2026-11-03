+++
id = "profile-limits"
part = "profile"
kind = "feature"
needs = ["charter-triage", "charter-checks", "change-tasks-limit", "template-resources"]
files = ["temper/docs/examples/reference/limits.json", "temper/docs/examples/reference/agent.json", "temper/docs/examples/reference/settings.json", "temper/crates/temper/src/tests/reference_settings.rs", "temper/crates/temper/tests/suite/startup.rs", "temper/crates/temper/tests/support/owned.rs"]
effort = "high"
minutes = 150
+++

# The reference as one profile a box is set up with, sized for a hundred changes

## Goal

`temper/docs/examples/reference/` becomes the reference's whole profile: `limits.json` and
`agent.json`, what the reference sets over `temper init`'s document and agent configuration,
sized for one goal of a hundred changes; the settings file's charters, template and
coordinator sized to them; and tests that start init's document with the profile over it and
hold the settings to it, as `temper/docs/design/protocol/configuration.md`, 13.9 ("The
profile, whole" and "Checked as it is read") has it.

Acceptance:

- **`limits.json`** (new): `{"engine": {"limits": {"overrides": {…}}}, "application":
  {"limits": {"overrides": {…}}}}`, the members 13.9's table names with the reference's values,
  each in the document's own override name (section 7): in the engine's, `tasks.tree_tasks`
  1024, `tasks.tasks` 64, `tasks.project_tasks` 64, `tasks.delegates` 32, `people.operations`
  33, `tasks.inbox_messages` 40, `tasks.run_inbox` 40, `tasks.inbox_bytes` 163840,
  `tasks.result_bytes` 4096, `tasks.batch` 4, `authority.batch` 4, `people.batch` 4,
  `authority.executors` 16, `tasks.executor_kinds` 16, `tasks.charters` 12; in the
  application's, `forge.change_policy.tasks` 8, `forge.changes` 32, `forge.holds` 32,
  `forge.tasks` 64, `checkout.message_bytes` 16386. Nothing else, but what the next bullet
  forces.
- **`agent.json`** (new): `{"profile": {"declared": {"tool_payload": 16386}, "policy":
  {"max_turns": 64}}}`.
- **They start:** merged over `temper init`'s files as 13.9 says (an object's members over
  the file's, any other value replacing it), the document and the agent configuration pass
  the start's admission: in `temper/crates/temper/tests/suite/startup.rs`, a test
  `the_reference_profile_passes_startup_admission`, built as
  `checked_in_deployment_profiles_pass_startup_admission`'s case for
  `crates/temper-init/template/temper.json` is, through a helper `bounded_reference` in
  `temper/crates/temper/tests/support/owned.rs` beside `bounded_template` that writes init's
  files, merges the two files over its `temper.json` and `agent.json` (a small recursive
  merge of `serde_json::Value`s in the helper), and returns the same `Bounded`; its
  `admission()` is `Ok`.
  - **Pre-approved, sizes only:** where admission names a limit that must move with these (a
    store page or value bound, a hub's or carrier's bound of the agent's largest call, a check
    between two limits), add that limit to `limits.json` at the least value that passes, and
    say so in the report. Where a value of the table cannot pass at all (a schema's bound, a
    page that cannot be overridden), take the largest that passes, set the settings file's
    dependants (template 4's report words, charter 5's batch number below) to it, and report
    the bound and the value taken, for the design's table to follow. A budget, a spend, a period, a
    time or a timeout never moves: if admission asks one to, stop and ask.
- **The settings file** (`settings.json`):
  - charters 5, 7 and 2 take `"turns": 64`; charters 6 and 8 keep 16;
  - template 4: `"contract": {"report": {"words": 4096}}` (or the `tasks.result_bytes` taken),
    and `"resources": [{"connector": 1, "symbol": 1}]`, the forge's *its own goal branch*, in
    the form `template-resources` made;
  - charter 5's `instructions`: in the paragraph that starts `On your first run, read the
    draft,`, `Make the first batch in one \`change\` call, every change with` becomes `Make
    the first batch with \`change\`, every change with`; and right after the paragraph that
    starts `On a designer's result,` a new paragraph, exactly:

    > A `change` call makes at most four changes. Make a larger batch in several calls, a
    > later call naming an earlier one's changes in "after" by the task numbers its answer
    > gave. Where a call is refused for its size, make one change to a call.

    (with `tasks.batch` taken lower than 4, its number in place of `four`). Nothing else in
    the file changes.
- **The settings hold to the profile:** in `temper/crates/temper/src/tests/reference_settings.rs`,
  `TURNS` becomes the reference's `agent.json` `max_turns` for charters 5, 7 and 2, read from
  the file (`include_bytes!` and `serde_json`, or a constant with an assertion that it equals
  the file's), and init's 16 for charters 6 and 8; a new test
  `the_reference_settings_fit_the_reference_profile` decodes `settings.json`, `limits.json`
  and `agent.json` and asserts: the charters' count with the stock's four is within
  `tasks.charters`; each charter's documents within 2 (small's `tasks.charter_documents`);
  each charter's turns within `max_turns`; template 4's report words within
  `tasks.result_bytes`, which is within the stock's `tasks.message_bytes` (4096); template 4
  names `{"connector": 1, "symbol": 1}`; charter 5's instructions contain `A \`change\` call
  makes at most`.
- `heavy cargo nextest run -p temper --lib -E 'test(/tests::reference_settings::/)'` and
  `heavy cargo nextest run -p temper --test suite -E 'test(/the_reference_profile_passes_startup_admission|checked_in_deployment_profiles_pass_startup_admission/)'`
  pass, each selecting its tests (find exact names with `--list`).

## Inputs

- `temper/docs/plan/reference-m10/notes/common.md`.
- `temper/docs/design/protocol/configuration.md`, 13.9 ("The profile, whole", its table;
  "Checked as it is read"), section 7 (overrides by dotted name) and 13.4 (the box's limits).
- `temper/docs/design/domain/agent.md`, 4.3.1 (`forge.change_policy.tasks`).
- `temper/docs/design/client/tools.md`, 5.3 (a template's `resources`).

## Owns and may change

- Owns the files of this spec's header for what acceptance names.
- `temper/docs/examples/reference/landing.json`, `temper-init`'s templates and
  `docs/examples/owned.json` are not yours: the reference is settings and overrides over
  init's, never a copy of it.
- If the startup suite's helper cannot write init's files and read them back as `Bounded`
  for a merged pair (its constructors take only a path), add the helper's own constructor
  from the two merged files, nothing else of `owned.rs`.

## Steps

1. The two files; the helper and the admission test; run it alone; settle what admission names
   as the pre-approved bullet says.
2. The settings file's edits; the settings test's assertions and the new test; run them.
3. `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D warnings`; the two runs of
   acceptance; commit and push.

## Report

As the implementer's text asks, with each limit admission made you add or lower and why.
