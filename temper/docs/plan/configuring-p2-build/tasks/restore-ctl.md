+++
id = "restore-ctl"
part = "restore"
kind = "feature"
needs = ["settings-wire", "policy-ctl"]
files = ["temper/crates/temper-client-tools/src/jig/rows.rs", "temper/crates/temper-client-tools/src/table.rs", "temper/crates/temper-client-tools/src/jig/runner.rs", "temper/crates/temper-client-tools/src/jig/runner/acts.rs", "temper/crates/temper-client-tools/src/jig/render/outcome.rs", "temper/crates/temper-client-tools/src/jig/render/acts.rs", "temper/crates/temper-client-tools/src/jig/render/document/act_answers.rs", "temper/crates/temper-client-tools/src/jig/render/document/outcomes.rs", "temper/crates/temper-client-tools/src/jig/render/document/me.rs", "temper/crates/temper-client-tools/src/jig/settings_adapter.rs", "temper/crates/temper-client-tools/src/tests/help.rs", "temper/crates/temper-client-tools/src/tests/fixtures.rs", "temper/crates/temper-client-tools/src/tests/jig/settings.rs", "temper/crates/temper-client-tools/src/tests/jig/restore.rs", "temper/crates/temper-client-tools/src/tests/jig.rs", "temper/crates/temper-client-tools/src/tests/golden/**", "temper/crates/temper-settings-form/src/view.rs", "temper/crates/temper-settings-form/src/wire.rs", "temper/crates/temper-settings-form/src/render.rs", "temper/crates/temper-settings-form/src/settings_file_tests.rs", "temper/crates/temper-engine-smith/src/tests/settings.rs", "temper/docs/help/revision.md", "temper/docs/help/configure.md", "temper/docs/help/reference/ctl-revision.md", "temper/docs/design/client/tools.status.md"]
effort = "high"
minutes = 90
+++

# restore-ctl: `temper ctl revision restore REVISION [REASON]`, and `revision show`'s `was:`

## Goal

temper ctl proposes the settings as a revision left them with one keyed call, `revision
restore REVISION [REASON]` (temper's `client/tools.md`, 5.3; contract B5), printing the
proposal and what it leaves out, and its refusals as tools.md words them; `revision show`
prints, for each item, what it replaced (tools.md, 5.2). The help's restore is that one
command.

Acceptance:

- `revision restore 4 "back to the old charter" --key K` sends one `contract::Ask::Restore {
  revision: 4, reason }` under K and prints `proposed restoring revision 4 as proposal p1/7 to
  role 0`, then `left out: SETTING` for each item the answer leaves out (`landing-rule 2
  --project 3`, `project-policy 3`); `--json` prints the answer's record. A replay under K
  prints the first answer.
- Refused, exit 2: forgotten, `temper ctl: revision 1 is forgotten; the oldest kept is
  revision 3` on stderr; too large, `revision restore: too large; revision R's settings pass
  a proposal's bound; restore a later revision`; empty, `revision restore: nothing to restore;
  the settings are as revision R left them`; R 0 or past the newest, `revision restore: no
  revision R; the newest is revision N`; stale otherwise, the settings' stale words. REVISION
  missing or not a number, or no `--key`: exit 1, nothing sent.
- `revision show R` prints under each item a `was:` line: `was: VALUE` (whole), `was: too
  large, N bytes`, `was: default`, `was: removed`, `was: not offered`, and for an
  application's item `was: N bytes`; `--json` gives each item a `replaced` member.
- The row appends at position 62 (`RESTORE_AT`), no existing position moves; the help, its
  goldens and `reference/ctl-revision.md` match the table; `configure.md`'s step 6 restores
  with the one command.
- The status entries below are gone; the focused tests of the changed packages pass.

## Inputs

- `temper/docs/design/client/tools.md`, 5.3 (`revision restore`'s row), 5.2 (`revision show`'s
  row and "A revision's value too large to print"), 6.1 and 6.3 (text, exit status), 7 (keyed
  requests).
- `jig/docs/design/domain/engine.md`, 17.10; `jig/docs/design/client/domain/contract.md`, 3.2
  (`Restore`) and 6 (`revision{n}`'s items).
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B4 and B5; `notes/common.md`.
- `temper/docs/plan/configuring-p2-build/tasks/settings-wire.md` (the contract's `Ask::Restore`,
  `Outcome::Restored`, `Refusal::Forgotten`, `RevisionItem.replaced`, and the arms it left in
  temper) and `policy-ctl.md` (`project-policy P`'s name in text).
- Old code, temper's `configuring-revisions/restore` and `/replaced` (unreachable from your
  slot, quoted from `temper/docs/plan/configuring-p2/notes/next-cut.md`): ctl's `RESTORE` row in
  `jig/rows.rs` (`revision restore`, REVISION, optional REASON, keyed), `runner/acts.rs`
  building `Ask::Restore`, `render/outcome.rs` printing "proposed restoring revision R as
  proposal P/K" and what it left out; `temper-settings-form`'s `view.rs` `enum
  RevisionReplaced { Value, TooLarge(Size), Default, Removed, NotOffered }`, `wire.rs`
  `settings_restore_revision_replaced`, `render.rs` `replaced_text` ("was: ...").

## Owns and may change

- Owned, for the edits in Steps: `temper-client-tools`' `src/jig/rows.rs` (`RESTORE`,
  `RESTORE_AT`), `src/table.rs` (`TOOLS` appended; the `revision` group's line), `src/jig/runner.rs`
  (the dispatch), `src/jig/runner/acts.rs`, `src/jig/render/outcome.rs`, `src/jig/render/acts.rs`
  (`refusal`'s `Forgotten`), `src/jig/render/document/act_answers.rs`, `outcomes.rs`, `me.rs`
  (the JSON of the outcome and the refusal), `src/jig/settings_adapter.rs` (the revision item's
  `replaced`), `src/tests/help.rs` (`GOLDENS`, `USAGES`, a reference test), `src/tests/fixtures.rs`
  (`TOOL_FORMS` alone), `src/tests/jig/settings.rs` (the revision goldens' fixtures), a new
  `src/tests/jig/restore.rs` with its `mod` line, the goldens (`help.txt`, `help_groups.txt`,
  new `help_revision_restore.txt`, `settings_revision*.txt` regenerated, new
  `restore_*.txt`); `temper-settings-form`'s `src/view.rs` (`RevisionItem.replaced`,
  `RevisionReplaced`), `src/wire.rs` (`settings_restore_revision_replaced`), `src/render.rs`
  (`revision_text`'s and `encode_revision`'s `was:`), `src/settings_file_tests.rs`;
  `temper-engine-smith/src/tests/settings.rs` (a revision's text expectation, if one reads it).
- Every arm settings-wire left in temper for the restore's contract variants (`rg 'a restore:
  restore-ctl' temper/crates`, or as its spec names them) gets its body here.
- `docs/help/` (a stated departure from the implementer's "Not yours", these files alone):
  `revision.md` ("How it changes": restoring; "Acting on it": `revision restore`; "Limits": a
  revision records what each item replaced, and a forgotten one cannot be restored),
  `configure.md` section 6 ("Restore what was there": the reset for a default stays; the
  earlier revision's settings by `revision list`, `revision show REVISION`, then `revision
  restore REVISION TEXT --key KEY`, `proposal show PROPOSAL`, `proposal decide PROPOSAL accept
  --running stays --key KEY`; its Expect and If not with the refusals above), and
  `reference/ctl-revision.md` as the table renders it. Read by tests: run `temper-docs`' and
  the help tests.
- Status: remove from `temper/docs/design/client/tools.status.md`'s **Not yet** the 5.2 text
  "; `revision show`'s items print nothing of what each replaced, which jig's revision
  document does not carry" and, from 5.3's list of acts, `revision restore` (", `project hold`
  and `revision restore`" becomes " and `project hold`"). Leave "5.3, beside its acts above:
  `revision restore`'s items of temper's kinds and of a project's policy." to `restore-story`,
  which shows them through the engine.
- Readers whose tests run: the help tests (`USAGES`, `GOLDENS`, the references), `temper-docs`'
  rules, the revision goldens in `src/tests/jig/settings.rs`
  (`whole_settings_document_renderings_are_stable`,
  `setting_text_and_revisions_include_whole_set_values_and_last_page_facts`,
  `non_charter_revision_hints_explicitly_read_the_current_value`), `temper-engine-smith`'s
  settings tests (the host tool `settings` renders a revision through the same form), the
  recovery and pending tests (a keyed request's tool position).
- Options, pre-approved:
  - **The reference page:** no test holds `reference/ctl-revision.md` today; add
    `the_revision_reference_is_what_the_table_renders` beside the settings one, and if the
    page already differs from the table beyond the new row, regenerate it whole and say so.
  - **The `revision` group's line** becomes "Settings revisions: read one or the history, move
    running work to the newest, restore one"; if a docs rule fixes the line's length or words,
    keep the old line and say so.
  - **The JSON** of `revision show`'s item: `"replaced"` is one of `{"set": VALUE}`,
    `{"too_large": N}`, `{"application": N}`, `"default"`, `"removed"`, `"not_offered"`; the
    answer's JSON `{"restored": {"revision": R, "proposal": "p1/7", "to": K, "left_out":
    [SETTING, ...]}}`, SETTING as `encode_setting` writes it. If the existing answer JSON
    (`act_answers.rs`) has a fixed shape per outcome that differs, follow it and say so.
  - If settings-wire named the contract's types otherwise than `Ask::Restore { revision,
    reason }`, `Outcome::Restored { proposal, to, left_out }`, `Refusal::Forgotten { oldest }`
    and `RevisionReplaced { Value(SettingSet), TooLarge(Size), Application(u32), Default,
    Removed, NotOffered }`, build to what it merged and say so.

## Steps

1. `rows.rs`:
   ```rust
   /// The restore's position in the tools table; positions are durable (tests/help.rs).
   pub const RESTORE_AT: u8 = 62;
   pub const RESTORE: Tool = Tool {
       name: b"revision restore",
       line: b"Propose the settings as REVISION left them, to the owners of its items",
       prints: b"`proposed restoring revision R as proposal P to role K`, then each item left out",
       result: b"restore",
       forms: &[&[
           positional(b"revision", b"REVISION", Kind::NumberRange { min: 0, max: 4_294_967_295 }, Required, b"The revision to restore"),
           positional(b"reason", b"REASON", Kind::Text, Optional, b"Why; restores revision R when absent"),
       ]],
       keyed: Keyed::Required,
       area: 1,
       notes: /* forgotten exits 2 naming the oldest kept; too large, restore a later one; read the
                 proposal with proposal show P and decide it with proposal decide */,
   };
   ```
   `table.rs`: `&rows::RESTORE` appended after `&rows::TASK_TEXT`.
2. `runner.rs`: `rows::RESTORE_AT` joins the acts' arm. `runner/acts.rs`: `RESTORE_AT`
   builds `c::Ask::Restore { revision, reason }`, the reason read as `settings propose`'s
   (`words(call, 1, limits)`, empty when absent: jig prefixes "restores revision R: "), and
   retains the party as `Propose` does (`operation.party`), so the answer prints `pN/K`.
3. `render/outcome.rs`: `Operation::of` names it `restore` with `revision: Some(R)`;
   `act_tool` maps `RESTORE_AT`; `proposal_project` gives `None`; the text of
   `c::Outcome::Restored` as Acceptance says, each `left out:` line naming the setting with
   `settings::setting_text` (temper's names, `project-policy P`); `refused_text` for a restore:
   `Refusal::Forgotten { oldest }` writes tools.md's line with the `temper ctl: ` prefix in
   place of the tool's name; `Malformed::Settings { fault: Count, .. }`, too large;
   `Malformed::SettingsUnchanged`, nothing to restore; `Refusal::Stale { newest }` with R 0
   or R > newest, no revision R; other stale, the settings' words. `render/acts.rs`' `refusal`
   gains `Forgotten`'s plain words (`forgotten; the oldest kept is revision O`) for any other
   act. The JSON arms (`act_answers.rs`, `outcomes.rs`, `me.rs`) per the option.
4. `temper-settings-form`: `view.rs` `RevisionItem { setting, edit, replaced: RevisionReplaced }`,
   `pub enum RevisionReplaced { Value(SettingSet), TooLarge(Size), Application { size: u32 },
   Default, Removed, NotOffered }`; `wire.rs` `settings_restore_revision_replaced(value:
   &w::RevisionReplaced, limits: &Limits) -> Result<c::RevisionReplaced, w::Problem>`, read in
   `settings_restore_revision_item` (the host tools' path); `render.rs` `replaced_text` after
   each item's edit lines, and `encode_revision`'s `replaced`. A too-large replaced value
   prints `was: too large, N bytes` alone: the value it names is a past one, which no
   `settings show` call serves unless a held revision reads it, and a restore proposes it
   whole all the same. `temper-client-tools` `settings_adapter.rs`: `form_revision_item`
   fills `replaced` from the contract's.
5. Help: `help.rs` `USAGES` gains `b"revision restore REVISION [REASON] --key KEY\n"` last,
   `GOLDENS` `(Some(jig::rows::RESTORE_AT), "help_revision_restore")`; `fixtures.rs`
   `TOOL_FORMS` 73; regenerate the goldens and `reference/ctl-revision.md` with
   `SKEIN_CODEGEN_WRITE=1`, then run them without it. The pages of `## Owns`.
6. Tests, `temper-client-tools/src/tests/jig/restore.rs` (the harness of
   `tests/jig/settings.rs`: `fixtures::signed_in`, `submit_prepared`, `root.done`):
   - `revision_restore_parses_its_revision_reason_and_required_key` (missing REVISION, a word,
     4294967296, no `--key`: exit 1, nothing sent; the call writes back to the same argv);
   - `revision_restore_sends_one_restore_and_prints_the_proposal_and_what_it_left_out` (text
     and JSON goldens `restore_proposed`, `restore_proposed_json`; `left_out` empty and with
     a landing rule and a project's policy);
   - `a_restore_replayed_under_its_key_prints_the_first_answer`;
   - `revision_restore_refusals_print_tools_words_and_exit_2` (forgotten, too large, nothing
     to restore, no revision R past the newest and 0, stale);
   - in `settings_file_tests.rs` or `tests/jig/settings.rs`:
     `revision_show_prints_what_each_item_replaced` (a revision with a charter set over a
     value, a template over its default, a policy that was removed, a rules reset over a value
     too large, an application item over 12 bytes, a project's policy over its policy: text and
     JSON goldens).
7. Narrow runs, counts checked: `heavy cargo nextest run -p temper-client-tools --lib -E
   'test(/restore/)'`, then `-E 'test(/settings|help|recovery|pending/)'`; `heavy cargo nextest
   run -p temper-settings-form --lib`; `heavy cargo nextest run -p temper-engine-smith --lib -E
   'test(/settings/)'`; `heavy cargo nextest run -p temper-docs --lib`. Then `cargo fmt`;
   `heavy cargo clippy -p temper-client-tools -p temper-settings-form -p temper-engine-smith
   --all-targets -- -D warnings`; the focused tests of those packages; `python3 -I -B
   tools/citations/citations.py --base origin/main`. Commit ("Restore a settings revision with
   one temper ctl call") and push.

## Report

The implementer's text's report, plus: the arms settings-wire left that this task filled; the
exact words of each refusal; the options taken (the reference test, the group line, the JSON).
