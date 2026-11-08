+++
id = "drafter-stock"
part = "drafter-stock"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/stock.rs",
  "temper/crates/temper-service/src/stock/drafter.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/limits.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
  "temper/crates/temper-service/src/tests/room.rs",
  "temper/crates/temper/tests/suite/creating.rs",
  "temper/crates/temper/src/tests/reference_settings.rs",
  "temper/crates/temper-init/src/tests/documents.rs",
  "temper/docs/help/settings.md",
  "temper/docs/help/template.md",
  "temper/docs/help/chat.md",
  "temper/docs/design/protocol/configuration.status.md",
  "temper/docs/design/domain/agent.status.md",
]
effort = "high"
minutes = 90
+++

# The drafter's charter and the design session's template in the stock

## Goal

The stock gains charter 12, the drafter's, and template 6, a design session that
runs it, and its delegation names charter 12, so that a person may start one
(`temper/docs/design/protocol/configuration.md`, 13.4;
`temper/docs/design/domain/agent.md`, 8.5). The charter's instructions are
`temper/docs/plan/ui-drafter/inputs/drafter.txt`, byte for byte.

Acceptance:

1. **The instructions:** `temper/crates/temper-service/src/stock/drafter.txt`, a
   copy of `temper/docs/plan/ui-drafter/inputs/drafter.txt` made with `cp` and
   checked with `cmp` (`notes/contracts.md`, *The charter*: 2,874 bytes, sha256
   `1f447743…5ed900`). No byte of it changes. If any stock bound or test refuses
   it (a host part, `core.run_bytes`, a receiving bound), stop with `Question:`,
   naming the bound and the bytes (`notes/common.md`, *The drafter's words*).
2. **The charter, the template and the delegation,** in `stock.rs`, exactly as
   `temper/docs/plan/ui-drafter/notes/contracts.md` has them: `DRAFTER_TOOLS`,
   `charter`'s `tools` match gaining `12 => DRAFTER_TOOLS`, `fn drafter(models:
   &[CatalogueModel]) -> Charter` beside `ask_charter`, `fn design(result_bytes:
   u32) -> Template` beside `ask`, both added to `checked_with`'s lists after
   Ask temper's, and `authority::Executor::Charter(12)` after `Charter(11)` in
   `ceiling`'s kinds. Each new item's comment cites its design section, as its
   neighbours do. `CHAT_TURNS`' and `CHAT_TIME`'s comments say the drafter's row
   shares them.
3. **The drafter's test,** a new test `the_drafter_is_charter_12_and_template_6`
   in `temper/crates/temper-service/src/tests/stock.rs`, beside
   `ask_temper_is_charter_11_and_template_5` and built as it is:
   - charter 12 equals charter 1 with `number` 12, the instructions
     `include_bytes!("../stock/drafter.txt")`, `inspect`, `modify` and `shell`
     false, `agents` true and `tools` `Tools(1022)`: so its hosts, models, run,
     reserve, waiting, resume and spend are a chat's;
   - its instructions, byte for byte: their length is 2,874 and
     `skein_lib::sha256::digest` of them is the pinned sha256 of the plan's input
     (above), written out in the test as bytes, so a change of any byte of the
     stock's copy fails it;
   - its tools name no delegate family (`tools.0 & ToolFamilies::JIG.delegate.0
     == 0`), no settings and no asking;
   - template 6 equals template 1 with `number` 6, `name` `design session` and
     `executor` `Charter(12)`;
   - the rules' ceiling, policy 1's ceiling and roles 0, 1 and 2 name
     `Executor::Charter(12)`; role 3 names none.
4. **The stock's figures and bounds with it added,** in the tests that hold them
   (this cut's survey; each change is the list or count alone):
   - `tests/stock.rs`: `ask_temper_is_charter_11_and_template_5`'s charter numbers
     gain 12, its tools 1022, its templates 6;
     `stock_chats_inspect_and_delegate_while_coders_modify` has charter 12 inspect
     nothing and use agents; the list at about line 948; and any other test there
     that lists the stock's charters or templates or counts them
     (`every_stock_charter_names_its_first_choices_and_the_rest`, the floors'
     tests `each_floor_refuses_one_below_the_stocks_need` and
     `each_charters_run_bytes_floor_names_that_charter`): charter 12 takes the
     stock's first choices, `FIRST_CHOICES`;
   - `tests/room.rs`, `stock_charters_leave_at_least_2048_bytes`, and
     `tests/stock_brief.rs`: charter 12 meets them as every stock charter does;
     where one pins a figure per charter, it gains charter 12's;
   - `tests/limits.rs`, where it counts the stock's charters, templates or
     executors;
   - `temper/crates/temper/tests/suite/creating.rs` (about line 127):
     `engine.charters.len()` 9;
   - `temper/crates/temper-init/src/tests/documents.rs`
     (`the_box_has_room_for_reference_creating_executors`): the stock's list gains
     12 and the expected union reads `[1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12]`, still
     within `documents.lines` and `tasks.charters`;
   - `temper/crates/temper/src/tests/reference_settings.rs` (about line 340),
     where it counts from `stock.charters`.
   If a floor refuses the stock at `temper init`'s document or a test world's
   limits once the drafter is added (`tasks.charters`, `core.templates`,
   `authority.executors`, `core.policy_bytes`, `core.run_bytes`), stop with
   `Question:`, naming the limit, its configured figure and the one needed: the
   box's limits are sized for this (`configuration.md`, 13.4) and none is raised
   here.
5. **The help** says what the stock now holds, in its own pages' words, within
   their bounds (`notes/common.md`):
   - `temper/docs/help/settings.md`, about line 38: the stock's charters 1, 2, 4,
     5, 6, 9, 10, 11 and 12, its templates 1 to 6 but 3 with 6 for a design
     session, a sentence for charter 12 ("Charter 12 is the drafter: it writes a
     draft with you as a design session's document, reads code through the
     forge, and starts nothing."), and the rules' eight charter executors (1, 2,
     4, 5, 6, 9, 11 and 12); about line 186, the floors: nine charters, five
     templates and fifteen executor kinds (eight charters, three procedures, one
     job and three roles);
   - `temper/docs/help/template.md`, after its line on template 5: "Template 6 is
     a design session, run by charter 12, the drafter; `chat start --template 6`
     starts it.";
   - `temper/docs/help/chat.md`, after its *Ask temper* bullet: a bullet
     *A design session,* template 6 under charter 12 in the stock: the drafter
     designs one piece of work with you, one question at a time, keeps the draft
     as the chat's document and starts nothing; you start a goal from the draft;
     a session opened again goes on from its draft and its open points.
   A help test that pins the old lists or counts takes the new ones; regenerate
   any `temper/docs/help/reference/*` page the change makes stale.
6. **The status:** delete the bullets `**Not yet:** 13.4: charter 12, the
   drafter's, ...` in `temper/docs/design/protocol/configuration.status.md` and
   `**Not yet:** 8.5: the stock holds no drafter's charter ...` in
   `temper/docs/design/domain/agent.status.md` (`notes/common.md`, *Status
   files*).

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.4: the rows *charter 12, the
  drafter's*, *template 6, a design session* and *the rules*; *Each charter's
  host families*; *The stock sets floors under the limits*.
- `temper/docs/design/domain/agent.md`, 8.5.
- `temper/docs/plan/ui-drafter/notes/contracts.md`, *Numbers*, *The charter*,
  *The template*, *The delegation*.
- `temper/docs/plan/ui-drafter/notes/common.md`.

## Owns and may change

- Owns: `temper-service`'s `stock.rs` and `stock/drafter.txt`; the tests and help
  pages in `files`, their lists, counts and sentences named above; the two status
  bullets.
- Not yours: `tools_for`'s declarations (`delegate-held`) and `run_workspace`
  (`no-workspace`), which build in parallel; this task's tests read the stock's
  values, not a run's declarations or workspace. `docs/examples/reference/` is
  `reference-widening`'s.
- Readers you may change, the expectation alone, if they pin the stock's charter
  or template lists, counts or the rules' executors (a `rg` for `[1, 2, 4, 5, 6,
  9, 10, 11]`, `[1, 2, 4, 5]`, `Charter(11)` and `fourteen` over `temper/` finds
  them): any in `temper-service`, `temper`, `temper-init`, `temper-client-tools`
  and `temper-docs`. Name each in the report.
- A page that lists a project's templates from the stock (How work runs' ways, a
  journey recorded on a fake box) now shows template 6: if a journey's replay
  check in `temper-client-journeys` fails for it, regenerate it with
  `temper-client-journeys again` (`notes/common.md`) and name it; a golden or
  print likewise, by its own tool.

## Steps

1. Read the inputs, then `stock.rs`'s `checked_with`, `charter`, `ask_charter`,
   `ceiling`, `chat` and `ask`.
2. `cp temper/docs/plan/ui-drafter/inputs/drafter.txt
   temper/crates/temper-service/src/stock/drafter.txt`, then `cmp` them and
   `sha256sum` the copy.
3. Write the test (acceptance 3), then the stock; break it once (a byte of the
   copy, then the tools mask) and see the test fail for each; restore.
4. Narrow tests: `heavy cargo nextest run -p temper-service --lib -E
   'test(=tests::stock::the_drafter_is_charter_12_and_template_6)'`, then each
   test of acceptance 4 by name.
5. The stock is what every deployment starts with, a meaning its readers see: run
   the focused tests of `-p temper-service`, `-p temper`, `-p temper-init`, `-p
   temper-client-tools`, `-p temper-docs` and `-p temper-client-journeys`
   (`heavy cargo nextest run -p ...`, one at a time).
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-service -p temper -p temper-init --all-targets -- -D warnings`.
7. Commit (why, in the message) and push.

## Report

As the implementer's text says, and: the drafter's bytes and sha256 as the test
reads them; each reader whose list or count changed; the help pages' sizes before
and after against their bounds; anything regenerated.
