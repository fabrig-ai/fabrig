+++
id = "charter-lines"
part = "stock"
kind = "feature"
needs = ["words-kind"]
files = [
  "temper/crates/temper-service/src/stock.rs",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/tests/engine/domain/tests/suite/configuring.rs",
  "temper/docs/design/protocol/configuration.status.md",
]
effort = "high"
minutes = 60
+++

# Each stock charter's words, the stock's kind 9

## Goal

Each stock charter carries a line of its own, what it does in the person's
terms, as the forge's stock default of kind 9: given at each start beside kind
3's, so that a fresh deployment holds a line for every charter it ships, and
Roles has it to say once Roles is drawn.

Acceptance, by `temper/docs/design/protocol/configuration.md`, 13.4 (*Kind 9,
each stock charter's words*):

1. **The defaults** (`temper/crates/temper-service/src/stock.rs`, `forge`): after
   kind 3's two items, one kind 9 item per stock charter, scope 0, its number the
   charter's, its value `Value::CharterWords` of the line byte for byte as
   `temper/docs/plan/ui-p3e-words/notes/contracts.md`, section 8, has it, for
   charters 1, 2, 4, 5, 6, 9, 10, 11 and 12 in that order, each sized by
   `temper_engine_smith::kinds::size` as the others are. The list's capacity and
   its `expect`'s words count them. A comment cites 13.4.
2. **Every stock charter has its line:** a test asserts that every charter the
   stock offers (`defaults(..)`'s charters, as the stock's other tests list them)
   has exactly one kind 9 item in `forge`, and that each line fits
   `temper_engine_domain_forge::settings::words_fit(.., LINE_BYTES)`; so a
   charter the stock gains later without a line fails it.
3. **Rows:** `the_binary_gives_the_forges_stock_at_every_start`
   (`temper-service/src/tests/stock.rs`, about line 1458): `ITEMS` 12 and a row
   per kind 9 item, its number and its line; acceptance 2's test beside it, the
   one new test of this task.
4. **Read back:** the engine world's `configuring.rs`, a row where a member reads
   `see settings charter-words 4` on the stock and is answered the stock's line,
   marked the stock's, and `see settings charter-words 7` that none is kept.
5. **Bite:** drop charter 12's line, see acceptance 2's test fail; restore.
6. **Status:** delete the bullet naming `charter-lines` in
   `temper/docs/design/protocol/configuration.status.md` (13.4) (`notes/common.md`,
   *Status files*).

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.2 (*A setting's words*) and
  13.4 (*Kind 9, each stock charter's words*).
- `temper/docs/plan/ui-p3e-words/notes/contracts.md`, sections 1, 3 and 8.
- `temper/docs/plan/ui-p3e-words/notes/common.md`.

## Owns and may change

- Owns: the stock's kind 9 items, the test of acceptance 2, the rows and the
  status bullet.
- **Readers of `stock::forge`** compare it whole or count it
  (`temper-playground`'s `documents.rs` and `tests/stories.rs`, the service's
  `tests/first_model.rs`, `ask_bound.rs` and `welcome_policy.rs`): if one pins its
  length or items, it takes the new figure alone; name each in the report. If a
  limit refuses twelve defaults at a start (a world's settings bound, jig's
  default count), stop with `Question:`, naming the limit and the figures.
- The stock's instructions (`stock/*.txt`, `coordinator.md`) stay as they are: a
  charter's words are its kind 9 item, never its instructions.
- `ask-words` may run beside this task in `tests/stock.rs` and `configuring.rs`:
  keep to your rows.

## Steps

1. Read the inputs; `forge` and its test.
2. The items, the test of acceptance 2, the rows; the bite.
3. A change of what every start gives: run the focused tests of `-p
   temper-service`, `-p temper-playground`, `-p temper-engine-domain-world` and
   `-p temper`, one at a time.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-service --all-targets -- -D warnings`.
5. Commit (why, in the message); `python3 -I -B tools/citations/citations.py --base
   origin/main`; push.

## Report

As the implementer's text says, and: the nine lines as the test reads them; each
reader whose expectation changed.

## Answers

2026-11-08. *May the engine world's shared fixture raise `tasks.charters`
from 4 to 12, or keep 4 and give its own forge defaults without kind 9?*
Raise it to 12. Kind 9's number runs 1 to `tasks.charters`
(`temper/docs/design/protocol/configuration.md`, 13.4's table), so a world
that starts on the whole stock holds at least the stock's highest charter,
12; a fixture that drops kind 9 would test a stock no deployment runs. Set
`limits.tasks.charters = 12` in `temper/tests/engine/domain/src/kinds.rs` and
say why in its comment (the stock's charter lines, kind 9, up to charter 12).
Any other engine-world fixture whose bound refuses the stock's twelve
defaults at a start (`configuring/fixture.rs`'s `CHARTERS`, `settings.rs`,
`ways.rs` and the like) takes the same raise to 12 alone, keeping what each
test checks; a test that sets a small bound to bite on a count gives itself
forge defaults without kind 9 instead, only where raising would blunt its
bite. Name each fixture changed in the report. The stock and its kinds stay
as they are.

## Rebase (2026-11-08, step merge, foreman ui-p3e-words/charter-lines/foreman-2)

resolve Story A’s fixture conflict by regenerating it from the combined code
The merge conflicts only in `temper/tests/playground/tests/fixtures/story-a.log`.
Branch commit `f9e68a8319` refreshed its hashes for the nine stock charter lines.
Integration commit `06e54574b7` (ask-words) refreshed the same hashes for changed Ask temper instructions.
Preserve both source changes when rebasing onto the landing’s current integration tip.
Regenerate the fixture; choosing either side’s hashes or combining them by hand leaves the combined state unrecorded.
The generator is `temper/tests/playground/tests/suite/walk.rs`, enabled by `SKEIN_CODEGEN_WRITE`.
Run through heavy, with `TEMPER_BUNDLE=none`:
`SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-playground-world --test suite -E 'test(=walk::story_a_walks_from_the_welcome_to_the_csv_change_landed)'`
Then verify the focused replay:
`heavy cargo nextest run -p temper-playground-world --test suite -E 'test(=walk::story_a_kept_log_replays_natively)'`
Commit the regenerated fixture and report it as the task’s generated-file repair.
