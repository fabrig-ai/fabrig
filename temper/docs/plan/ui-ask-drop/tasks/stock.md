+++
id = "stock"
part = "stock"
kind = "feature"
needs = ["domain-configuring", "domain-settings", "joined"]
files = ["temper/crates/temper-service/**", "temper/crates/temper/tests/**", "temper/crates/temper/src/tests/reference_settings.rs", "temper/crates/temper/src/tests/settings_fields.rs", "temper/crates/temper-init/src/tests/documents.rs", "temper/crates/temper-playground/src/documents.rs", "temper/tests/real-loop/src/fixtures.rs"]
effort = "high"
minutes = 240
+++

# stock: template 3 and charter 3 leave the stock

## Goal

temper's stock holds no configuring chat: charter 3, template 3 and `configuring.txt`
go, with `Executor::Charter(3)` from the rules' and policy 1's delegation, and every
test, count and figure that names them follows. Charter 11 and template 5, Ask temper,
take their place, their numbers kept. The tasks this one needs have moved every
scripted run of charter 3 to charter 11; what is left is the stock and the tests that
count it.

Acceptance:

- `temper/crates/temper-service/src/stock.rs` is `notes/contracts.md`'s *The stock after
  the drop*: charters 1, 2, 4, 6, 9, 10 and 11; templates 1, 2 and 5; the ceiling's
  delegation charters 1, 2, 4, 6, 9 and 11, procedures 2, 3 and 4, roles 0, 1 and 2;
  `ASK_FIRST_CHOICES` in place of `CONFIGURING_FIRST_CHOICES`; no `configuring_charter`,
  `configuring`, `CONFIGURING_TOOLS` or match arm 3; `stock/configuring.txt` deleted.
  Comments saying "configuring" for charter 3 go or say Ask temper.
- `rg -n 'Charter\(3\)|charter == 3|number == 3|configuring\.txt|CONFIGURING' temper/crates/temper-service temper/crates/temper/tests temper/crates/temper/src/tests temper/crates/temper-init temper/crates/temper-playground`
  finds only numbers 3 that are not the configuring charter (`hello_refused.rs`'s
  synthetic shortfalls, `stock/kinds.rs`' application kind 3), each named in the report.
- `STANDARD_BRIEF`'s `brief_bytes` stays 354,988; its comment and
  `tests/stock_brief.rs`' derivation say it is charter 1's room (356,958) less 1,472 bytes
  for 64 titles and framing, with a margin, where they said charter 3's.
- The focused tests of every package you change pass, with the counts the report
  states: `heavy cargo nextest run -p temper-service --lib`,
  `heavy cargo nextest run -p temper --lib -E 'test(/reference_settings/) | test(/settings_fields/)'`,
  `heavy cargo nextest run -p temper --test suite -E 'test(/creating/) | test(/startup/)'`,
  `heavy cargo nextest run -p temper-init --lib`, `heavy cargo nextest run -p temper-playground --lib`;
  and, as a change of meaning (the stock its readers build on), the focused suites of
  the stock's readers: `heavy cargo nextest run -p temper-engine-domain-world --test suite`,
  `heavy cargo nextest run -p temper-client-world --test suite`,
  `heavy cargo nextest run -p temper-one-process-world --lib`,
  `heavy cargo nextest run -p temper-fake --lib`, `heavy cargo nextest run -p temper-stage-one --lib`.
  `temper-real-loop-world` and the system world's slow stories are compiled
  (`heavy cargo check -p temper-real-loop-world --tests`, `heavy cargo check -p temper-system-world --tests`),
  not run.
- The design's **Not yet** lines this task builds (below) are deleted.

## Inputs

- `temper/docs/plan/ui-ask-drop/notes/common.md` and `notes/contracts.md` (binding).
- `temper/docs/design/protocol/configuration.md`, 13.1, 13.4 (the stock's tables, its
  tools and models, its delegation) and section 11 (the standard brief).
- `temper/docs/design/domain/agent.md`, 4.3 (the settings family's tools, declared to no
  stock charter), 8.1 and 8.4.
- `temper/docs/design/draft/ask-temper.md`, section 9.

## Owns and may change

- Owns `temper/crates/temper-service/**`: `src/stock.rs`, `src/stock/configuring.txt`
  (deleted), `src/limits.rs` (`STANDARD_BRIEF`'s comment), and its tests:
  - `src/tests/stock.rs`: `the_configuring_agent_is_charter_3_and_template_3` becomes
    `ask_temper_is_charter_11_and_template_5`, its charter-3 and template-3 rows gone,
    charter numbers `[1, 2, 4, 6, 9, 10, 11]`, tools `[1023, 514, 512, 512, 0, 0, 3072]`,
    templates `[1, 2, 5]`; `each_floor_refuses_one_below_the_stocks_need` (counts 7, 3,
    12; the template-name floor and the alternatives floor name the stock's first
    setting past them now; the policy's bytes re-measured); `a_stock_policy_past_its_limit_...`;
    `every_stock_charter_names_its_first_choices_and_the_rest` (`number == 11` alone);
    `the_foremans_charter_...`, `stock_budgets_...`, `the_reviewer_keeps_...` (find a
    charter by its number, not its index); `stock_and_init_admit_the_landing_procedure_...`
    (no `Executor::Charter(3)`); `stock_words_hold_no_placeholder_a_run_cannot_fill`
    (over the stock's texts, `ask.txt` among them, with no `configuring.txt`); every
    `configuring_txt_*` test deleted with its file (its rules go with charter 3; `ask.txt`
    is the draft's section 8 as written).
  - `src/tests/stock_brief.rs`: charter 3's rows and its `number == 3` branches go; the
    standard brief's derivation names charter 1.
  - `src/tests/fixtures.rs` (~9, ~60) and `src/tests/root_checks.rs` (~13): the embedded
    stock-sized overrides 8 charters, 4 templates and 13 executor kinds become 7, 3 and
    12, the policy-size comment re-measured.
  - `src/tests/checks.rs` (~130, ~478): charter 3's line leaves the expected model-error
    lists. `src/tests/welcome_policy.rs` (~208, ~426): charter 3 and its run row leave the
    offered list; the joined policy's size re-measured (today 5,854).
  - `src/tests/first_model.rs`: template 3's preview rows go; the tests named for "the
    stock configuring chat" are named for Ask temper, keeping `(5, 11)` and Astra first;
    the previews' count follows.
  - `src/tests/startup.rs` (~430): labels `[1, 2, 1]` for templates 1, 2 and 5.
  - `src/tests/room.rs` (~8, ~70): its `CHARTER = 3` is a stock charter that remains,
    11 by default, the 2,047 and 2,048 bytes boundaries kept, the diagnostic re-measured.
  - `src/tests/workers.rs` (~1107, ~1136) and `src/tests/fixtures/worker_peers.rs`
    (~723–975): `ConfiguringChat` becomes `AskingChat`, its executor charter 11, its
    reads `see settings` and `see settings charter 1` with `more`, its tests named for
    Ask temper, their recovery checks kept.
  - `src/tests/limits.rs` (~191, ~202): unchanged figures (354,988), comments only.
- Reader files of other crates, the edits named:
  - `temper/crates/temper/src/tests/reference_settings.rs` (~521): `Charter(3)` leaves the
    reference's expected delegation, the lists' counts in their names and comments one
    fewer. `temper/crates/temper/src/tests/settings_fields.rs` (~219): `STOCK_VALUES` 13
    (one model, seven charters, three templates, the policy and the rules).
  - `temper/crates/temper/tests/suite/creating.rs` (~82): six offered stock charters, no
    "configuring"; `tests/support/configuration.rs` (~41, ~56, ~69): overrides 7, 3, 12
    and six offered charters; `tests/suite/startup.rs` (~33): the policy's size as
    re-measured.
  - `temper/crates/temper-init/src/tests/documents.rs` (~15, ~23): no stock charter 3 in
    the reference room's seed; the expected set re-derived.
  - `temper/crates/temper-playground/src/documents.rs` (~175, ~416): the five operations
    that remove charter 3 and template 3 and their references go, since the stock holds
    none; the comment on seven slots says the stock's seven.
  - `temper/tests/real-loop/src/fixtures.rs` (~355): the generated stock minimum
    overrides 8, 4, 13 become 7, 3, 12 if the start checks them as minima; if larger is
    harmless, leave them and say so. Compile only.
  - Generated files the change makes stale (the limits or configuration reference pages,
    a counted baseline) are regenerated by their own tests and named; none is expected.
- Not yours: `temper/crates/temper/src/tests/help.rs`, `temper/docs/help/**` (the task
  `help`), the engine and client worlds (their tasks moved them; a red there that
  this task's correct change trips is yours to repair by `notes/common.md`'s oracle
  rule, named in the report).

## Status lines to delete

This task deletes these **Not yet** bullets whole, each once what it says is built (a
stated departure from the implementer's text, `notes/common.md`), and no other status
line:

- `temper/docs/design/protocol/configuration.status.md`: the bullet beginning
  "**Not yet:** 13.1, 13.4, 13.6 step 3: the stock still holds charter 3 and template 3",
  and the bullet beginning "**Not yet:** 13.4, 11: the standard brief, 354,988 bytes, is
  derived from charter 3's room".
- `temper/docs/design/domain/agent.status.md`: the bullet beginning "**Not yet:** 4.3,
  8.1: the stock still holds charter 3", and the bullet beginning "**Not yet:** 4.3:
  `the_stock_brief_with_a_whole_spec_fits_both_receiving_bounds`".

The standard brief's derivation, with charter 1's room, leaves a margin of 498 bytes
(356,958 − 1,472 − 354,988): the derivation's margin constant says 498.

## Steps

1. The stock first (`stock.rs`, `configuring.txt`, `limits.rs`); `heavy cargo check -p temper-service --tests`.
2. The service's tests, then the readers above, one package at a time, each with its
   narrowest test after its edit; a figure that moves is re-measured by the test that
   prints it (`stock_brief.rs` prints its rows), never guessed.
3. Once: `cargo fmt`; `heavy cargo clippy -p temper-service -p temper -p temper-init -p temper-playground -p temper-real-loop-world --all-targets -- -D warnings`;
   the acceptance's runs; `python3 -I -B temper/scripts/old_forms.py` (with `--write` only
   to lower what you removed). No new test, so no bite check; `ask_temper_is_charter_11_and_template_5`
   must still fail if charter 11 leaves the stock: check it once by hand, restore.
4. Commit ("Drop the configuring chat from the stock") and push.

## Report

The stock's lists before and after; each figure re-measured (old → new: the policy's
bytes, room's diagnostic, stock_brief's rows); each test deleted and why; each reader
edit; the readers' suites' counts; the `rg` output's remaining lines and why.
