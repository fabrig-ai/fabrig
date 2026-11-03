+++
id = "policy-ctl"
part = "policy-item"
kind = "feature"
needs = ["settings-wire"]
files = ["temper/crates/temper-settings-form/src/file.rs", "temper/crates/temper-settings-form/src/view.rs", "temper/crates/temper-settings-form/src/wire.rs", "temper/crates/temper-settings-form/src/render.rs", "temper/crates/temper-settings-form/src/fields.rs", "temper/crates/temper-settings-form/src/fields/jig.rs", "temper/crates/temper-settings-form/src/fields/render.rs", "temper/crates/temper-settings-form/src/fields/tests.rs", "temper/crates/temper-settings-form/src/settings_file_tests.rs", "temper/crates/temper-client-tools/src/jig/settings_adapter.rs", "temper/crates/temper-client-tools/src/jig/rows.rs", "temper/crates/temper-client-tools/src/jig/render/document/proposal.rs", "temper/crates/temper-client-tools/src/jig/render/document/settings.rs", "temper/crates/temper-client-tools/src/jig/runner/proposal.rs", "temper/crates/temper-client-tools/src/tests/jig/settings.rs", "temper/crates/temper-client-tools/src/tests/golden/**", "temper/crates/temper-engine-smith/src/settings.rs", "temper/crates/temper-engine-smith/src/tests/settings.rs", "temper/crates/temper-service/src/startup/problems.rs", "temper/docs/help/settings.md", "temper/docs/help/configure.md", "temper/docs/help/policy.md", "temper/docs/help/reference/ctl-settings.md", "temper/docs/help/reference/settings-fields.md", "temper/docs/design/client/tools.status.md"]
effort = "high"
minutes = 90
+++

# policy-ctl: a project's own policy as an item of temper ctl's settings file

## Goal

temper ctl's settings file names a project's own policy, `{"setting": {"kind":
"project-policy", "project": P}, "set": POLICY}` (configuring-p2's contract P1; temper's
`client/tools.md`, 5.3), decoded by `temper-settings-form` to the item jig's client sends
(`Setting::ProjectPolicy { project }`, settings-wire's), its `edit` and a second one refused
before anything is sent; `proposal show` prints the policy now and as it would be, whole; the
help says so. Every `unreachable!("a project's policy: policy-ctl")` arm in temper gets its
body.

Acceptance:

- `temper ctl settings apply FILE` and `settings propose FILE` with a `project-policy` item
  send one `ChangeSettings` / `Propose` whose item is `contract::Setting::ProjectPolicy {
  project: P }` set to the policy decoded as a `policy` item's value is; a run's
  `propose_settings` (the same form) carries it too.
- Refused before sending, exit 1, the item numbered from 1: `"edit": "removed"` or `"reset"`
  on it; a second `project-policy` item in one file (another project's included); `project`
  missing or 0; a `number` member.
- `proposal show` of a settings proposal naming one prints `project-policy P · set · now
  POLICY · would be POLICY`, each whole, and its JSON carries both; an engine refusal naming
  the item prints `project-policy P` as its setting.
- `settings-fields` lists the kind, the page regenerated; the help pages below say how to set a
  project's policy and who decides it; the help goldens and reference pages match.
- The status entry below is gone; the focused tests of the changed packages pass.

## Inputs

- `temper/docs/design/client/tools.md`, 5.3 (**Settings FILE is JSON**, its `project-policy`
  item and refusals, `proposal show`'s project's policy), 5.2 (`proposal show`).
- `jig/docs/design/domain/engine.md`, 17.2 (a project's own policy as an item) and 17.6 (its
  proposal, who decides); `jig/docs/design/client/domain/contract.md`, 3.2 and 5.1 (the
  review's item).
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B1 and B5;
  `temper/docs/plan/configuring-p2/notes/contracts.md`, P1; `notes/common.md`.
- `temper/docs/plan/configuring-p2-build/tasks/settings-wire.md` (the client contract's and
  the wire's `ProjectPolicy` arms this task reads) and `policy-surface.md` (the arms it left).
- As the pattern: `temper/docs/plan/reference-m3/tasks/kinds-ctl-form.md`, which added
  temper's kinds to the same form.

## Owns and may change

- Owned, for the edits in Steps: in `temper-settings-form`, `src/view.rs` (`Setting::
  ProjectPolicy { project: u32 }`), `src/file.rs` (`setting`, `item`, `decode_root`),
  `src/wire.rs` (`settings_wire_setting`, `settings_restore_setting`), `src/render.rs`
  (`setting_text`, `encode_setting`, a revision item's too-large hint), `src/fields.rs`
  (`shape_of_kind`), `src/fields/jig.rs` (the `kind` word, the `project` member's line),
  `src/fields/render.rs` (`Value::Setting`'s words), `src/fields/tests.rs` (the generator),
  `src/settings_file_tests.rs`; in `temper-client-tools`, `src/jig/settings_adapter.rs`
  (`form_setting`, `client_setting`), `src/jig/rows.rs` (`CHANGE_SETTINGS`' notes alone),
  `src/jig/render/document/proposal.rs`, `src/jig/render/document/settings.rs`,
  `src/jig/runner/proposal.rs`, `src/tests/jig/settings.rs`, its goldens
  (`src/tests/golden/help_settings_apply.txt` regenerated, new `settings_review_project_policy_*`);
  `temper-engine-smith/src/settings.rs` (`refusal`'s setting arm) and its `src/tests/settings.rs`;
  `temper-service/src/startup/problems.rs` (the stock floor's setting arm).
- Every other `unreachable!("a project's policy: policy-ctl")` arm in temper (find them with
  `rg 'policy-ctl' temper/crates`), in that arm alone: where a setting is named in text, it
  reads `project-policy P`; where it is translated, it carries `ProjectPolicy { project }`;
  where no path reaches it (a stock floor names no project's policy), the body names it as
  text the same way, never a panic. List each in the report.
- `docs/help/` (a stated departure from the implementer's "Not yours", these files alone):
  `settings.md`, `configure.md` (step 2 and step 3), `policy.md` ("How it changes" and
  "Acting on it", one sentence each); `reference/ctl-settings.md` and
  `reference/settings-fields.md` only as their tests render them. These pages are read by
  tests (`.config/gate/read-by-tests.txt`): run `temper-docs`' and the help tests.
- Status: remove from `temper/docs/design/client/tools.status.md`'s **Not yet** the text "a
  settings file's item of a project's own policy (`temper-settings-form` decodes none), and "
  (leaving "5.3, beside its acts above: `revision restore`'s items of temper's kinds and of a
  project's policy.", which `restore-story` closes).
- Readers of what changes, whose tests run: `temper-settings-form`'s tests (the form, FIELDS);
  `temper/crates/temper/src/tests/settings_fields.rs` (the page and the fields' paths);
  `temper-client-tools`' help tests (`src/tests/help.rs`: goldens, `the_settings_reference_
  is_what_the_table_renders`); `temper-docs`' rules (`src/tests/rules.rs`: placeholders, voice);
  `temper-engine-smith`'s settings tests (the run's form and refusal words).
- Options, pre-approved:
  - **A project's policy too large in a review or a revision:** if settings-wire's
    `setting{…}` read answers a `ProjectPolicy` item (now, or a pending proposal's), resolve a
    too-large review value with it as `runner/proposal.rs` does for the others; if it does not,
    skip that read and print `too large, N bytes` alone, and say so in the report. A revision's
    too-large project's policy prints `too large, N bytes` with no `settings show` hint (no
    read serves a project's whole policy): say so.
  - **The review's origin** of a project's policy item: print none in text (`now POLICY ·
    would be POLICY`, no parenthesis), since a project's policy is no setting; keep the JSON's
    `origin` as the review gives it.
  - If settings-wire named the contract's or the wire's variant otherwise than
    `Setting::ProjectPolicy { project }` / `w::Setting::ProjectPolicy(w::ProjectPolicySetting)`,
    build to what it merged and say so.
- Not this task's: `revision restore` and `revision show`'s replaced values (`restore-ctl`);
  the joined stories (`policy-story`).

## Steps

1. `temper-settings-form`:
   - `view.rs`: `Setting` gains `ProjectPolicy { project: u32 }`, last, a line citing
     `jig/docs/design/domain/engine.md`, 17.2.
   - `file.rs` `setting`: `kind == b"project-policy"` takes `project` (required, from 1:
     `error(b"project", b"from 1")`, missing `project: missing member`) and no `number`
     (`error(b"number", b"project-policy has no number")`, as the rules' "rules has no
     number"), before the `project` member is refused for jig's other kinds. `item`: its `set`
     is `values::setting_policy` as a `policy`'s, as `SettingValue::Policy`; an `edit` on it
     is `error(b"edit", b"a project's policy is set alone")`. `decode_root`: a second
     `ProjectPolicy` item, whatever its project, is `error(b"setting", b"one project's policy
     per file")` at its index, beside the duplicate check. Messages keep the form's `settings
     file item N: MEMBER: RULE` shape.
   - `wire.rs`: `settings_wire_setting` and `settings_restore_setting` map the arm to and from
     settings-wire's `w::Setting::ProjectPolicy`; the value is the `policy` arm's.
   - `render.rs`: `setting_text` prints `project-policy P`; `encode_setting` writes
     `{"kind": "project-policy", "project": P}`; the revision too-large hint as the option says.
   - FIELDS: the `kind` word `project-policy` (line: "A project's own policy, set alone and at
     most once; its value is a policy."), `project`'s line "For a landing rule, a project's
     number, from 1, the deployment's when left out; for a project's policy, its project,
     required."; `shape_of_kind(b"project-policy") => Some(Shape::Policy)`; `Value::Setting`'s
     words name `project-policy`. `fields/tests.rs`' generator: for `project-policy` it writes
     `project` and no `number`, and its edit-only form sets the value (an `edit` on it is
     refused). Regenerate `docs/help/reference/settings-fields.md` from the page the
     `temper` crate's `the_page_is_what_fields_renders` prints.
2. `temper-client-tools`:
   - `settings_adapter.rs`: `form_setting` / `client_setting` map `ProjectPolicy { project }`.
   - `render/document/proposal.rs`: a `ProjectPolicy` review item prints as above, its values
     through `settings::named_value_inline` as a `policy`'s; `render/document/settings.rs`
     needs no vocabulary name for it (the form's `setting_text` serves).
   - `runner/proposal.rs`: the too-large reads, per the option.
   - `rows.rs` `CHANGE_SETTINGS.notes`: one sentence, "`{\"kind\": \"project-policy\",
     \"project\": P}` sets project P's own policy, set alone, once a file; it needs an owner of
     P, and with the deployment's items one owner of both." (wrapped as the notes are), then
     regenerate `reference/ctl-settings.md` and `help_settings_apply.txt` with the tests'
     `SKEIN_CODEGEN_WRITE=1` and run them again without it.
3. `temper-engine-smith/src/settings.rs` `refusal`: `s::Setting::ProjectPolicy { project }`
   maps to the form's arm, so a run hears `out of shape; item N; project-policy P: FAULT`.
   `temper-service/src/startup/problems.rs`: the arm names `project-policy P`.
4. Help pages: `settings.md` ("What it is": a project's own policy is no setting, yet an item
  of a change, named `project-policy P`; "How it changes": the deployment's items need its
  owners, a project's policy that project's owners, a change of both one person who owns both;
  `settings show` lists no project's policy, `project show` reads its spends); `configure.md`
  step 2 (a file setting `PROJECT`'s policy from `settings show policy 1 --json`, with jq as
  the charter's example, and "set alone, at most once a file"), step 3 (who decides it);
  `policy.md` (a project's own policy is changed by its owners with `project fund` or whole as
  a settings file's item). Placeholders as the walk declares them (`PROJECT` is already used).
5. Tests:
   - `temper-settings-form/src/settings_file_tests.rs`:
     `a_project_policy_item_decodes_to_its_setting_and_a_policy_value` (the stock-like policy
     JSON a `policy` item takes, round-tripped through `render::encode_setting` and the value's
     encoder); `a_project_policy_edit_a_second_one_or_a_bad_project_is_refused_naming_the_item`
     (removed, reset, two of them with different projects, `project` 0, `project` missing, a
     `number`: each `settings file item N: ...`, N from 1).
   - `fields/tests.rs`' existing generator tests pass with the new kind
     (`every_generated_file_decodes`, `a_path_resolves_through_its_shapes`).
   - `temper-client-tools/src/tests/jig/settings.rs`:
     `a_settings_file_with_a_project_policy_sends_its_item` (apply and propose: the sent ask's
     one item `c::Setting::ProjectPolicy { project: 1 }`, its value the file's);
     `a_project_policy_item_refused_by_the_form_sends_nothing` (exit 1, nothing in
     `root.sends`); `proposal_show_prints_a_project_policy_now_and_as_it_would_be` (text and
     JSON, goldens `settings_review_project_policy_text` and `_json`);
     `an_engine_refusal_naming_a_project_policy_prints_its_setting` (`Malformed::Settings`
     with `ProjectPolicy`, exit 2, `settings apply: out of shape; item 0; project-policy 1:
     rules`; and `Refusal::Role`, not theirs, printed in the engine's words, exit 2).
   - `temper-engine-smith/src/tests/settings.rs`:
     `a_runs_proposal_carries_a_project_policy_item` (beside
     `settings_proposal_items_use_the_file_decoder`) and the refusal's words in
     `settings_refusals_keep_ctls_words_without_a_client_dependency`.
6. Narrow runs, each count checked: `heavy cargo nextest run -p temper-settings-form --lib`;
   `heavy cargo nextest run -p temper-client-tools --lib -E 'test(/settings/)'`, then `-E
   'test(/help/)'`; `heavy cargo nextest run -p temper-engine-smith --lib -E
   'test(/settings/)'`; `heavy cargo nextest run -p temper --lib -E 'test(/settings_fields/)'`;
   `heavy cargo nextest run -p temper-docs --lib`; `heavy cargo nextest run -p temper-service
   --lib -E 'test(/stock|problems/)'`. Then `cargo fmt`; `heavy cargo clippy -p
   temper-settings-form -p temper-client-tools -p temper-engine-smith -p temper-service --all-targets
   -- -D warnings`; the focused tests of those packages and `temper`'s; `python3 -I -B
   tools/citations/citations.py --base origin/main`. Commit ("Name a project's own policy in
   temper ctl's settings file") and push.

## Report

The implementer's text's report, plus: each `policy-ctl` arm given a body (file, function,
body); the options taken (too-large reads, the origin); the exact refusal words the form
prints.
