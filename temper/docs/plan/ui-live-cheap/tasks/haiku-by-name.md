+++
id = "haiku-by-name"
part = "live-haiku"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/support/forge_land.rs",
  "temper/crates/temper/tests/suite/forge_land.rs",
  "temper/docs/design/domain/testing.status.md",
]
effort = "high"
minutes = 75
+++

# The live Claude Code story takes Haiku by name, at low effort

## Goal

`a_person_asks_claude_code_to_land_a_change_on_the_owned_forge` (`temper/crates/temper/tests/live.rs`) runs every charter it selects on stock row 3, `claude-fable-5-1`, at its default `high`: the dearest row in the stock catalogue. Make a fresh sitting take `claude-haiku-5-5` by name, as a person adds a Claude subscription's model, and select it at `low` effort in every charter the story selects, so Claude Code runs as `--model claude-haiku-5-5 --effort low`. The ChatGPT story stays on stock row 7, `gpt-6-luna`, exactly as it is.

This task departs from the implementer's text in one way: it removes one **Not yet** entry from `temper/docs/design/domain/testing.status.md` (acceptance 5), which the cut wrote for it.

Acceptance:

1. **Haiku is added by name in the sitting's one settings change.** On a fresh first sitting (`first && !kept`), the `settings apply` the story already makes holds, first, an item that sets a model setting: kind `model`, at the highest number in 1 to `jig_domain_tasks::MODELS` (64) that the sitting's `settings show --json` read names for no model; its value the `settings show model 3 --json` read's value (stock row 3, `claude-fable-5-1`, the Anthropic provider's dearest stock model, whose endpoint, window, output and prices a model added by name takes: jig's `domain/engine.md`, 17.11), with `number` that number, `model.name` `claude-haiku-5-5`, `model.effort` `low`, `model.efforts` kept, `shown` and `description` null, `picker` true, `withdrawn` false, `source` `person` and `estimated` true.
2. **Every charter the story selects names it.** Charters 1 and 2 take that item's `model` whole, as they take the stock row's model now, with the same `alternatives`, `rest` and `time` as now; the chat starts with `--model claude-haiku-5-5`. The story still requires the model read after startup to be the row it expects (`claude-fable-5-1` for Claude Code, `gpt-6-luna` for ChatGPT) and fails, naming it, when it isn't.
3. **Nothing else of the story moves:** the ChatGPT story's change and reads are as now (row 7, no model item); the Claude Code story's reads, commands, accounts and launch are as now but for the model it names and its effort. `stock.rs` and the stock catalogue are untouched: no row is added.
4. **A focused test holds the selection:** one table test in `temper/crates/temper/tests/suite/forge_land.rs`, one row per `LiveModel` variant and one refusal row (Step 3), passing; it bites (implementer's text). `heavy cargo check -p temper --tests` compiles the `live` and `process` binaries; the live story is not run.
5. The **Not yet** entry for 2.7 that begins "the Claude Code story selects stock row 3" is gone from `temper/docs/design/domain/testing.status.md`, the rest of the file unchanged.
6. fmt, `old_forms.py` for temper, `heavy cargo clippy -p temper --all-targets -- -D warnings` clean; `temper`'s `suite` binary's forge-land tests pass (Step 4).

## Inputs

- `temper/docs/plan/ui-live-cheap/inputs/request.md`, all of it, and its *For every task of this plan*: they bind.
- `temper/docs/design/domain/testing.md`, 2.7, *The owned-forge landing story on Claude Code* (this cut's lines: Haiku taken by name, at `low`) and *After every landing* (the cheapest models).
- `jig/docs/design/domain/engine.md`, 17.1 (*A setting no default has is added by a change that sets it*) and 17.11 (*A number is a position*, *A number keeps its name*, *A charter's models are names, resolved*, *An owner's act*): a model added by name takes the highest free number and the provider's dearest stock model's figures, marked estimated. A charter's model resolves to its name's catalogue record whole, so the record's default effort is the run's.
- `jig/docs/design/client/domain/contract.md`, 5.1, *the settings*: `settings show`'s read names each setting by kind and number, defaults included, and carries no value.
- `temper/docs/help/reference/ctl-settings.md`: `settings show KIND NUMBER --json` prints a value a settings file's `set` takes; one `settings apply` is one decision, checked whole.
- What this cut found, read only:
  - temper ctl has no verb for `ChangeModel` (*Add a model by name*); `settings apply` with a `model` item is how ctl adds one. jig's core takes a change that sets a model at a number no default has (`jig/crates/jig-domain-core/src/settings/tests.rs`, `a_number_keeps_its_name`, which adds model 9 so), and checks the set whole, so the model and the charters naming it go in one file.
  - A model setting's value form is `{"number","model":{...},"shown","description","picker","withdrawn","source","estimated"}`, `source` one of `stock`, `list`, `person` (`temper/crates/temper-settings-form/src/settings_file_values.rs`, near 1054; `settings_file_tests.rs`, `the_settings_form_round_trips_a_model`).
  - Stock row 3 declares `low, medium, high, xhigh, max`, default `high` (`temper/crates/temper-service/src/stock.rs`, `MODELS`). Claude Code takes `--effort` only when the run's model has one (`smith/crates/smith-protocol-program/src/backend/start.rs`, near 349): a model with no efforts, as the Models page's add makes, would drop `--effort` from the launch, so the added model keeps the row's efforts with `low` its default.
  - The story's settings phase is `LiveJudge::settings` in `temper/crates/temper/tests/support/forge_land.rs`: it reads `settings show --json`, `charter 1`, `charter 2` and `model N` (`LiveJudge::setup`, the four queued reads), then writes `settings.json` and applies it against the read's `newest`.

## Owns and may change

**Owned:** the `files` above.

- `temper/crates/temper/tests/support/forge_land.rs`: `LiveModel`'s `name` and `number` and what you add beside them, the new `live_settings`, `LiveJudge::settings`. Nothing else in the judge changes. Put `live_settings` just after `configure_live_model`, before `pub struct LivePlan`: the draft `live-change-call` has edited this file near `replies_after` and in `find_delegates`, `check_landing`, so stay off those.
- `temper/crates/temper/tests/suite/forge_land.rs`: the new test, just after `a_claude_plan_declares_without_import_or_sign_in`; nothing else. `live-change-call` adds tests after `an_imported_records_path_is_printed_as_file` and after `a_landing_referee_reads_committed_tools_beyond_the_conversation`: stay off those places.
- `temper/docs/design/domain/testing.status.md`: acceptance 5's entry only.

**Readers, unchanged:** `temper/crates/temper/tests/live.rs` (it calls `name()` on the ChatGPT model alone, for `retain_live_model`); `retain_live_model`, `configure_live_model`, `live_commands`, `stock_charters_choose_claude_before_the_demo_model` (the stock charters before the sitting's change: their alternatives still name `claude-fable-5-1`, rightly).

**Options, each with its default:**

- **The inventory's lines.** Read `settings show --json`'s record shape from its encoder, not from a guess (`rg -n 'SettingsDocument' temper/crates/temper-client-tools/src`, or a test that prints the record). Default: the lines are `settings[i].setting`, each `{"kind": ..., "number": ...}`. If the read names no model lines at all (defaults left out), take 64 when no line names it, and say so in the report.
- **No free number** (every number 1 to 64 named): `live_settings` returns `Err("no free model number for claude-haiku-5-5")`; the story fails, naming it.
- **The copied row lacks `low`** among its efforts: `Err` naming it; never add an effort the row doesn't declare.
- **A kept store** (`kept`): the story reads no settings and makes no change, as now; its deployment holds the model from the sitting that made it. Leave that path as it is.
- **The chat's `--effort`:** default none: the chat takes the model's default, `low`. If you find `chat start --model` without `--effort` takes another effort than the model's default, add `--effort low` to that one `chat start` and say so.
- **A help or design line** that names the Claude Code story's model as Fable: none is known (`docs/help/` names Fable only as a stock row). If you find one, change only that line and name it.

## Steps

1. **`LiveModel`** (sketch; the bodies are yours):

   ```rust
   impl LiveModel<'_> {
       /// The model every charter the story selects runs, and the chat's `--model`.
       #[must_use]
       pub const fn name(self) -> &'static str; // "gpt-6-luna" | "claude-haiku-5-5"
       /// The stock row read after startup, by number and name: the selected
       /// model's own, or the provider's dearest, whose figures a model taken
       /// by name copies (jig's `domain/engine.md`, 17.11).
       const fn number(self) -> &'static str; // "7" | "3"
       const fn row(self) -> &'static str; // "gpt-6-luna" | "claude-fable-5-1"
       /// A model taken by name: its default effort; `None` for a stock row.
       const fn by_name(self) -> Option<&'static str>; // None | Some("low")
   }
   ```

2. **`live_settings`**, a pure function the judge calls and the test reads:

   ```rust
   /// The settings change a fresh sitting applies (domain/testing.md, 2.7):
   /// charters 1 and 2 name the story's model alone. A model taken by name is
   /// added in the same change, at the highest model number the inventory
   /// leaves free, from the stock row it copies.
   pub fn live_settings(
       model: LiveModel<'_>,
       inventory: &serde_json::Value, // `settings show --json`
       charters: [&serde_json::Value; 2], // `settings show charter 1|2 --json`
       row: &serde_json::Value,       // `settings show model N --json`
   ) -> Result<serde_json::Value, String>; // `{"items": [...]}`
   ```

   `LiveJudge::settings` keeps reading the revision from the inventory and writing `settings.json`, and builds the file with `live_settings`, so its call order and count are unchanged. Keep the current refusal text for a wrong row ("the selected live stock model") or reword it to name the row; the test pins whichever you keep.

3. **The test**, `a_fresh_sitting_selects_its_model_in_every_charter`, beside `a_claude_plan_declares_without_import_or_sign_in`: build small reads by hand (an inventory naming models 1 to 8 and charters 1 and 2; two charter reads with a `setting` and a `value` holding `model`, `alternatives`, `rest`, `time`; a row read with `setting` and a `value` in the model form above, `model.efforts` `["low","medium","high","xhigh","max"]`, `effort` `"high"`), then one table:
   - `LiveModel::ChatGpt { account: None }`, row `gpt-6-luna`: two items, the charters, each `model` the row's;
   - `LiveModel::ClaudeCode { login: Path::new("/held/claude-login") }`, row `claude-fable-5-1`: three items, the first `{"kind":"model","number":64}` with the value of acceptance 1, both charters' `model` equal to its `model`;
   - Claude Code with the row read naming another name: `Err`, with its text.

   Bite it once: drop the effort override (or the `name` change) and see the Claude Code row fail; restore.
4. Runs, through `heavy`, one at a time:
   - `heavy cargo nextest run -p temper --test suite -E 'test(=forge_land::a_fresh_sitting_selects_its_model_in_every_charter)'` while you work (check the path nextest prints, and the count);
   - at the end, `heavy cargo nextest run -p temper --test suite -E 'test(/^forge_land::/)'`: the forge-land focused tests, the readers of `LiveModel`;
   - `heavy cargo check -p temper --tests`;
   - `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p temper --all-targets -- -D warnings`.

   Never the `live` or `process` binaries' tests, nor `scripts/proof-live.sh`: no worker holds the login, and after-landing runs the story at every landing.
5. Remove acceptance 5's status entry. Commit (why: the live tier runs on the cheapest models, the user's word of 11-01) and push.

## Report

The implementer's report, plus: the inventory's line shape you found and the number your code takes on a fresh deployment; the rule the test's bite broke and the failure it showed; and that the live story was compiled, not run.
