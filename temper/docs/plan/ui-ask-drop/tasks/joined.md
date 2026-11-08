+++
id = "joined"
part = "joined"
kind = "migration"
needs = []
files = ["temper/tests/client/**"]
effort = "high"
minutes = 240
+++

# joined: the joined client world's configuring stories run Ask temper

## Goal

The joined client world (`temper/tests/client`, the package `temper-client-world`) runs
its configuring stories with stock charter 11 and template 5 in place of the stock's
charter 3 and template 3: ctl starts Ask temper with `chat start --template 5`, the
run reads through `see` and writes its document through `document`, and its proposals
take the asking form. What each story checks of the client, ctl and the engine stays.
After this task nothing in `temper/tests/client` reads stock charter 3 or template 3,
so that the task `stock` can remove them.

Acceptance:

- No `charter == 3`, `template.number == 3`, `Executor::Charter(3)`, `--template 3`,
  `Tool::Settings` or `note` call by the Ask temper run, `with_configuring`,
  `configuring_world`, `configuring_agent` or `configuring_reply` in `temper/tests/client`;
  `tests/suite/configuring_chat.rs` is `tests/suite/asking_chat.rs`.
- The two Ask temper stories that skip when template 5 is absent
  (`the_page_starts_the_asking_chat`, `start_over_closes_and_opens_a_fresh_ask`, ~36 and
  ~313) assert template 5 is there instead of returning.
- `heavy cargo nextest run -p temper-client-world --test suite -E 'test(/^asking_chat::/) | test(/^chat_accept::/) | test(/^settings::/) | test(/^settings_tools::/) | test(/^builders::/)'`
  passes with the base's count (the renamed module's tests counted under their new
  names); its CPU time no more than 10% above the base's (report both).

## Inputs

- `temper/docs/plan/ui-ask-drop/notes/common.md` and `notes/contracts.md` (binding:
  *What moves to charter 11*, its constants for this world).
- `temper/docs/design/domain/testing.md`, 6.3 and 6.4's row "the client and the engine,
  joined".
- `temper/docs/design/domain/agent.md`, 8.4.1 to 8.4.3 (`see`, the asking
  `propose_settings`, `document`).
- `temper/docs/design/draft/ask-temper.md`, section 9.

## Owns and may change

- Owns `temper/tests/client/**`: `src/fixtures.rs`, `src/front.rs`,
  `tests/suite/configuring_chat.rs` (renamed), `tests/suite/main.rs`'s `mod` line,
  `tests/suite/settings.rs`, `settings_tools.rs`, `chat_accept.rs`, `builders.rs`, and
  any other file there the renames reach.
- Never `temper/crates/**`. A client or ctl behaviour that differs for template 5 (a
  word, a refusal) is checked as it is today; if it looks wrong, report it under
  `## Left` and keep the assertion on today's behaviour.
- Options, pre-approved:
  - **The fixture** `asking_configuration` (was `configuring_configuration`) copies stock
    charter 11 and template 5 into the world's small config where it copied 3, delegates
    to `Executor::Charter(11)`, and adds the asking family
    (`jig_domain_core::ToolFamilies::JIG.ask`) beside settings to the rules' ceiling and
    the owner's (and a member's) authority. `Joint::with_asking` and
    `with_asking_projects` replace `with_configuring*`; `fixtures::asking_world`,
    `asking_member`, `asking_agent` and `Joint::asking_reply` replace theirs.
  - **`journal.run_bytes`** (9,019 for charter 3's instructions and framed policy):
    measure charter 11's and set it to that figure, its comment saying so.
  - **A story's settings reads** (`Tool::Settings { read: SettingsRead::Inventory | Setting {..} }`)
    become `Tool::See` with the address (`settings`, `settings charter 1`), parsing the
    answer as `see` gives it; a proposal from `Tool::ProposeSettings` with a `note`, if
    any, becomes inline items or parts (`more`). The start-over story's `note` write
    (~323) becomes `document`'s write.
  - **`settings_tools.rs`'s review story** (~409–477) reads and proposes charter 11
    where it read charter 3 (`settings show charter 11`, the file's item, the review's
    lines).
  - **The ctl key** `configure` in `chat start ... --key configure` becomes `ask`.
  - Test names saying "configuring" say "asking" (`an_asking_chat_started_through_ctl_proposes_and_an_owner_accepts`,
    `an_asking_chat_accepts_its_own_proposal`); a name a design or status cites (`rg`
    it in `temper/docs/design/`) is renamed too and listed in the report, for the lead.

## Steps

1. Run the base's stories once and keep the count and CPU seconds.
2. Fixture first (`src/fixtures.rs`, `src/front.rs`), then each story, running it alone
   after each move (`heavy cargo nextest run -p temper-client-world --test suite -E 'test(=asking_chat::NAME)'`).
3. Once: `cargo fmt`; `heavy cargo clippy -p temper-client-world --all-targets -- -D warnings`;
   the acceptance's run; `python3 -I -B temper/scripts/old_forms.py`. The two stories
   that no longer skip must bite: remove template 5 from the fixture by hand, see each
   fail on its assertion, restore.
4. Commit ("Run the joined world's configuring stories as Ask temper") and push.

## Report

Each story with its old and new name and what changed in its calls; the measured
`run_bytes`; the test count and CPU seconds before and after; renamed tests a design
cites; anything under `## Left`.
