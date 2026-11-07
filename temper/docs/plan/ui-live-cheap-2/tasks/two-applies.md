+++
id = "two-applies"
part = "live-claude-settings"
kind = "feature"
needs = []
files = ["temper/crates/temper/tests/support/forge_land.rs", "temper/crates/temper/tests/suite/forge_land.rs"]
effort = "high"
minutes = 120
+++

# ui-live-cheap-2: the live Claude Code sitting sets its model, then its charters, in two applies

## Goal

After-landing's `temper::live`
`a_person_asks_claude_code_to_land_a_change_on_the_owned_forge` fails before it reaches
Claude: its one `settings apply` (`--key live-settings`) exits 2 with
`settings apply: This model cannot be chosen: claude-haiku-5-5`. That change sets
`claude-haiku-5-5` at model 64 and names it in charters 1 and 2 at once. jig's clients'
component checks every charter's model against the catalogue *before the core sees the
change* (`jig/crates/jig-protocol-engine-clients/src/request.rs`, the call to
`translate::llm::refused`; `src/translate/llm.rs`, `settings_items` and `settings_model`),
and the model the same change sets isn't in the catalogue yet, so the expected why is
`unconfigured`. A person's path is two acts: Settings' Models adds the model, then a
charter chooses it. The fixture takes that path: the catalogue item in its own apply
first, then the charters, against the revision the first made
(`temper/docs/design/domain/testing.md`, 2.7). `claude-haiku-5-5` stays at `low`.

Acceptance:

- **Red first, with the live refusal.** The new test below, written against
  `live_settings` as it stands (its one change, applied once as the judge applies it:
  `settings apply FILE --against NEWEST --key live-settings`), fails with status 2 and
  stderr exactly `settings apply: This model cannot be chosen: claude-haiku-5-5\n`. Run
  that same apply once with `--json` added and quote the refusal's `why` it prints
  (expected `unconfigured`). The report quotes both. If ctl's `--json` prints no `why`,
  name the why from the component's code path above and say so.
- **The fixture applies two changes, in a person's order.** `live_settings` returns the
  sitting's changes in apply order (sketch below): for Claude Code, the model item alone
  (key `live-model`), then charters 1 and 2 naming it (key `live-settings`); for ChatGPT,
  the charters alone (key `live-settings`), as today. `LiveJudge::settings` writes and
  queues each, the first against the inventory's `newest`, each later one against the
  revision before it plus one. Nothing else of the sitting changes.
- **Green on the fixture's real output.** The new test applies `live_settings`'
  changes, through `live_apply`'s argv, to the joined client world holding the Claude
  deployment's stock catalogue: each apply exits 0 with empty stderr; afterwards
  `settings show model 64 --json` is `claude-haiku-5-5`, `source` `person`, `estimated`
  true, effort `low`, its efforts holding `low`; and `settings show charter N --json`, for
  1 and 2, has `model.name` `claude-haiku-5-5`, `model.effort` `low`, `alternatives` `[]`
  and `rest` false.
- **The table test follows.** `a_fresh_sitting_selects_its_model_in_every_charter`
  (`temper/crates/temper/tests/suite/forge_land.rs`) keeps its three rows, each expecting
  the new return: ChatGPT one change, Claude Code two (the model item, then the
  charters), the unexpected stock row's refusal as it is.
- **It bites.** Put the model item back into the charters' change (one change, as now)
  and see the new test fail with the refusal above; restore. The report names it.
- **The live story compiles:** `heavy cargo nextest run -p temper --test live --no-run`
  (never run: `notes/common.md`).
- The status sentence below replaced, word for word.
- `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py` raises no count; clippy and the
  tests in Steps pass.

## Inputs

- `temper/docs/design/domain/testing.md`, 2.7: *The owned-forge landing story on Claude
  Code* (the two applies, written for this task) and 6.3 (the joined world).
- jig's `jig/docs/design/domain/engine.md`, 17.11: *A number is a position*, *A number
  keeps its name*, *A charter's models are names, resolved*, *Who hears* (a change of the
  catalogue tells the clients' component before its commit, so the second apply's check
  finds the model).
- The code the refusal comes from: `jig/crates/jig-protocol-engine-clients/src/translate/llm.rs`,
  `refused`, `settings_items`, `settings_model`. Read, don't edit (but the option below).
- The world's template: `temper/tests/client/tests/suite/settings_tools.rs`, its
  `configuration()` and `world()`, and its
  `a_settings_file_asking_an_effort_its_model_does_not_offer_is_refused_by_name` (a
  settings file through ctl with `fixtures::give_with_file`, read by
  `fixtures::answered`); `temper/tests/client/src/front.rs`, `Joint::with_engine`.
- The Claude deployment, as the suite already builds it:
  `temper/crates/temper/tests/suite/forge_land.rs`, `initialized_deployment`,
  `configure_claude` and `stock_charters_choose_claude_before_the_demo_model` (the
  engine parsed, `temper_service::stock::with_models` for its catalogue and charters).
- `temper/docs/plan/ui-live-cheap-2/notes/common.md`.

## Owns and may change

Owns:

- `temper/crates/temper/tests/support/forge_land.rs`: `live_settings`, the new
  `LiveChange` and `live_apply`, and `LiveJudge::settings`. Nothing else of the file.
- `temper/crates/temper/tests/suite/forge_land.rs`: the table test's expectations, the
  new test and its world helper, placed beside `stock_charters_choose_claude_before_the_demo_model`.

May also change, only so:

- `temper/crates/temper/Cargo.toml`, `[dev-dependencies]`: the lines the world needs,
  `jig-client-kit.workspace = true` and
  `temper-engine-domain-world = { path = "../../tests/engine/domain" }`, and `Cargo.lock`
  as cargo writes it.
- `temper/tests/client/src/front.rs`: `fn engine_limits()` made `pub` with
  `#[must_use]`, if the world takes the client world's engine limits (the default); no
  other line. If you can build the same limits without it, leave the file alone.
- `temper/docs/design/domain/testing.status.md`, the 2.7 entry: replace

  > in charters 1 and 2 in one settings apply, where 2.7 has two: the
  > clients' component refuses that apply before the core sees it
  > (`settings apply: This model cannot be chosen: claude-haiku-5-5`), and
  > no focused test applies the fixture's changes to an engine. ChatGPT

  with

  > in charters 1 and 2 in a second settings apply, as 2.7 has it; a
  > focused test applies both through ctl on the joined world's Claude
  > catalogue (`crates/temper/tests/suite/forge_land.rs`). ChatGPT

  re-wrapped to the paragraph's width, and nothing else of the file.

Options, each pre-approved:

- **The program account.** Default: before the reads, the test declares the Claude Code
  account as the live story does (`account declare 2 claude_code claude-login --key
  live-program`) and asserts it is answered. If the joined world can't take a program
  declaration without work outside these files, leave it out and say so: no settings
  act reads an account.
- **The world's engine.** Default: `temper_engine_domain_world::walking::config(SEED)`,
  its owners widened as `settings_tools.rs`'s `configuration()` widens them, with its LLM
  facts replaced by the Claude deployment's: the catalogue (`defaults.models`) and
  charters 1 and 2 from `temper_service::stock::with_models`, and `settings.models`,
  `settings.endpoints` and `settings.llm_providers` from
  `jig_protocol_config::core(&engine, ..)`; then `Joint::with_engine(..)` with that config
  twice. Raise the world's engine limits (charters, models, templates) where the stock
  needs them, as `Joint::with_configuring_projects` does; a walking template whose LLM
  choice no longer fits the catalogue loses its choice. If an equivalent route is
  shorter, take it and say which.
- **If the first apply is refused** (out of shape, a reference fault): the model item's
  fields are the fixture's to fix in `live_settings`, keeping its name, its number rule,
  row 3's figures, its efforts with `low` its default, `source` person and `estimated`.
- **If the charters' apply is still refused for its model** once the catalogue holds it,
  that's the core's fault on a person's path: fix it at its lowest place in jig
  (`jig/crates/jig-protocol-engine-clients/src/translate/llm.rs` or
  `jig/crates/jig-domain-core/src/settings/`), with a row in that package's nearest
  existing table test, run that package's focused tests and its direct dependents'
  checks, and say so first in the report.
- **Stock row 2** (`claude-sonnet-5-5`, at `low`) only if the core can't let a Claude
  Code account choose a model outside stock at all; then say why in the report. Fable 5.1
  is never the story's model.

## Steps

1. Read the inputs. Write the new test and its world against the code as it stands
   (one change, one apply keyed `live-settings`), run it alone, and quote its failure and
   the `--json` why:
   `heavy cargo nextest run -p temper --test suite -E 'test(=forge_land::a_fresh_claude_sitting_applies_its_settings_as_a_person_does)'`.
2. The fixture, in `temper/crates/temper/tests/support/forge_land.rs`:

   ```rust
   /// A settings change a fresh sitting applies, and its `settings apply`'s key.
   #[derive(Debug, PartialEq)]
   pub struct LiveChange {
       pub key: &'static str,
       pub file: serde_json::Value,
   }

   /// The settings changes a fresh sitting applies, in order (domain/testing.md, 2.7):
   /// a model taken by name first, alone, at the highest model number the inventory
   /// leaves free, from the stock row it copies (`live-model`); then charters 1 and 2
   /// naming the story's model alone (`live-settings`).
   pub fn live_settings(
       model: LiveModel<'_>,
       inventory: &serde_json::Value,
       charters: [&serde_json::Value; 2],
       row: &serde_json::Value,
   ) -> Result<Vec<LiveChange>, String>;

   /// The `settings apply` of the change at `at` in `live_settings`' order, read from
   /// `file`: the first against the inventory's `newest`, each later one against the
   /// revision the one before it made.
   #[must_use]
   pub fn live_apply(change: &LiveChange, at: usize, newest: u64, file: &str) -> Vec<String>;
   ```

   Each change's `file` is a whole settings file (`{"items":[...]}`), its items as
   today's. `LiveJudge::settings` writes each change to `<key>.json` in
   `self.plan.client` and pushes each `live_apply` argv onto `pending`, in order, then
   sets `LivePhase::SettingsChanged` as now; check that no later phase of the judge
   counts calls from a fixed index past the settings reads (the setup's fixed indices
   come before them).
3. The tests, in `temper/crates/temper/tests/suite/forge_land.rs`:

   ```rust
   /// The Claude deployment's stock catalogue, endpoints, providers and charters 1 and 2
   /// on the joined client world's engine (domain/testing.md, 2.7 and 6.3).
   fn live_claude_world() -> (support::Scratch, jig_client_kit::harness::World<Temper, Joint>);

   #[test]
   fn a_fresh_claude_sitting_applies_its_settings_as_a_person_does() {
       // [the program account, per the option]
       // the judge's four reads through ctl: `settings show --json`,
       // `settings show charter 1 --json`, `... charter 2 --json`, `settings show model 3 --json`
       // live_settings(LiveModel::ClaudeCode { login: Path::new("/held/claude-login") }, ..)
       // each change through fixtures::give_with_file with live_apply's argv, file
       // "<key>.json", its bytes serde_json::to_vec(&change.file): status 0, stderr empty
       // the reads after: model 64, charters 1 and 2, as Acceptance has them
   }
   ```

   Then the table test's expectations for the new return.
4. Bite (Acceptance), then:
   `heavy cargo nextest run -p temper --test suite -E 'test(/^forge_land::/)'` (check the
   count), `heavy cargo nextest run -p temper --test live --no-run`,
   `heavy cargo check -p temper-client-world --tests` if `front.rs` changed, `cargo fmt`,
   `python3 -I -B temper/scripts/old_forms.py`, and
   `heavy cargo clippy -p temper [-p temper-client-world] --all-targets -- -D warnings`.
   Under a jig option, also that package's focused tests and clippy.
5. Commit (why: the live sitting's one apply is refused before the core; a person's two
   acts are accepted) and push.

## Report

As the implementer's text has it, and: the red's stderr and status and the `--json` why,
quoted; which option each choice took; the new test's time as nextest prints it; the bite
and its failure; the files changed outside `files`, each with its one-line reason; and
that the live story was compiled, not run.

## Answers

2026-11-07 (temper/ui). Repair the oracle in jig: your first option. The clients component's refusal before admission is designed, and a final answer with no durable write is right for it. Keep the exemption narrow. Only a Final that answers a refusal the component gave before core admission, with nothing stored, is exempt from Made. Every other Final keeps its durability check. Do it in jig-client-kit's observe.rs or referee.rs, whichever is the lowest place, and hold it with one row in the kit's own tests: such a refusal accepted, and an ordinary Final with no write still refused. Then go on as the task says: the red with the live refusal and its why, then the two applies. Name the kit change in the report, as jig is shared.
