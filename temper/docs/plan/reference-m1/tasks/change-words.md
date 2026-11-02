+++
id = "change-words"
part = "change"
kind = "feature"
needs = ["change-batch", "change-by-rule", "goal-forge-symbol"]
files = ["temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-protocol-store/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/result.rs", "temper/crates/temper-engine-domain-forge/src/result_tests.rs", "temper/crates/temper-engine-domain/src/route.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-protocol-store/schema/store-v11.schema", "temper/crates/temper-protocol-store/src/generated/**", "temper/crates/temper-protocol-store/golden/v11/**", "temper/crates/temper-protocol-store/src/records/change.rs", "temper/crates/temper-protocol-store/src/tests/**", "temper/tests/engine/domain/tests/suite/forge_routes/result.rs", "temper/tests/engine/domain/tests/suite/forge_routes/light_rules.rs", "temper/docs/design/domain/forge.status.md", "temper/docs/design/domain/agent.status.md"]
effort = "high"
minutes = 100
+++

# change-words: a landed change's result carries its producer's report

## Goal

A landed change's result words carry, after its landing's three lines, its producer's last
report, then what the landing passed, the whole tail cut from the end to the change's
contract's bound, the deployment's `tasks.result_bytes` (contract C6). A change held for a
conflict under rules asking no update (task `change-by-rule`) says so in its hold's words.

Acceptance:

- `temper_engine_domain_forge::ChangeRow` gains `pub report: Box<[u8]>`, doc "Its producing
  task's last successful result words (domain/forge.md, 8.2 step 7)", empty until a producer
  succeeds; stored: `report: bytes 1048576` appended to `record ChangeRow` in
  `store-v11.schema`, in place (temper's `protocol/store.md`, section 4: the version stays
  until the first release), its generated code, translations (`src/records/change.rs`) and
  goldens regenerated as the store's tests require.
- The forge's `Event::ProducedTitle` gains `report: Box<[u8]>`; `route.rs`'s
  `keep_produced_title` fills it from the producer's `TaskResult::Change`'s `words`; the
  forge's `produced_title` keeps the title as today (first only) and the report each time
  the child is the change's current producing delegate, saving the row.
- `ChangeResult::reported(&self, report: &[u8], head: &Commit, bound: u32) -> ResultText`
  in `result.rs`: `Report:\n`, the report, `\n`, then `words(head, bound)`'s text, as one
  `ResultText` (`Report:` and its line left out when the report is empty).
  `change_landing_words` (`forge_route.rs`) appends `reported(...)` fitted to the room after
  the three lines, where it appends `words(...)` today.
- The change's contract's words are `env.limits.tasks.result_bytes` (`chat_change`), not
  `512.min(...)`.
- `hold_change_decision` (`forge_route.rs`): for `forge_change::Hold::Resolutions` where
  `domain.forge.requires(repository, &row.base).up_to_date` is false, the hold carries words
  (`StepEnd::HoldFrom` with `holds::source(domain, None, words)`, as the other worded holds
  there): `The merge into BASE conflicts, and its landing rule asks no update: settle the
  conflict on BRANCH and release the change, or cancel it.`
- The stories below pass; every story of `forge_routes` passes, a story whose expected
  result words change only because a report now precedes the landing's facts, or because the
  bound is the world's `result_bytes`, updated to the new words.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.2 step 7 (the producer's last report under
  `Report:`; the bound) and step 6 (the conflict's hold under rules asking no update).
- `temper/docs/design/domain/agent.md`, 4.3.1 (the delegate's bullet: the contract's words
  bounded by `tasks.result_bytes`).
- `temper/docs/design/domain/forge.status.md`: its **Not yet** entry for 8.2 step 7, which
  this task removes; `agent.status.md`: the 4.3.1 entry's last clause ("a change's contract
  bounds its words at 512 bytes"), which this task removes.
- temper's `temper/docs/design/protocol/store.md`, section 4 (the store version and an
  in-place schema change).
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C5 and C6.

## Owns and may change

Owns the files in the header.

- **Order:** the lead puts this task after `goal-forge-symbol`, so the goal family's
  `goal-forge-branch` (the forge's `domain.rs`, `boundary.rs`) and `goal-forge-symbol` (the
  root's `forge_route.rs`) are on your base: build on their hunks there, never undo them.
- Reader and neighbour files: every `ChangeRow { .. }` literal (`rg 'ChangeRow \{'
  temper/crates temper/tests`: the forge's fixtures and tests, `forge_route/tests.rs`,
  `temper/tests/engine/forge`, `temper/tests/engine/protocol-forge`, `temper-fake`'s
  tests, the domain world's stories), `report: Box::new([])` only; every
  `Event::ProducedTitle { .. }` site, its new field only.
- Keep the hunks in `forge_route.rs` to `chat_change`'s contract line,
  `change_landing_words` and `hold_change_decision`; in the forge's `domain.rs` to
  `produced_title`. Both files and the store's schema are changed on plan reference-m8's
  trunk too.
- Options: if the store's tests pin ChangeRow's largest value or the store's sizes (`rg
  'producer_failure' temper/crates/temper-protocol-store`), update the pinned figure for the
  new field and name it. If a producer's result words reach the root by another record than
  `keep_produced_title`'s, take them where the title is taken and say so.

## Steps

1. The field, the event, the words, the bound and the hold's words.
2. Tests:
   - `result_tests.rs`: `a_report_precedes_what_the_landing_passed` (report, then
     `checks at`), `a_long_report_is_cut_from_the_end_and_marked` (a report past the bound:
     the prefix kept, the evidence gone, the cut marked as `fitted` marks it),
     `an_empty_report_adds_no_line`.
   - `temper/tests/engine/domain/tests/suite/forge_routes/result.rs`:
     `a_landed_change_carries_its_producers_report`: the producer finishes with words
     `Done: edited README.\n## Left\n- none`; the change's result words hold the three lines,
     `Report:`, those words, then `checks at`.
   - `light_rules.rs` (task `change-by-rule`'s file):
     `a_conflict_under_rules_asking_neither_says_why_it_holds`: the held change's hold words
     are the words above.
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E
   'test(/report/)'`; `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/forge_routes::result::/) | test(/light_rules::/)'`. Then `heavy cargo nextest run -p
   temper-protocol-store`, `-p temper-engine-domain-forge`, `-p temper-engine-domain --lib`,
   `-p temper-engine-domain-world --test suite -E 'test(/forge_routes::/)'`, and `heavy cargo
   check --tests` for each package naming `ChangeRow` (`temper-engine-forge-world` and the
   others `rg` finds: `heavy cargo check -p <package> --tests`).
4. The status edits. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-engine-domain-forge -p temper-engine-domain -p temper-protocol-store -p
   temper-engine-domain-world --all-targets -- -D warnings`. Commit ("Carry a change's
   producer's report in its result") and push.

## Report

As the implementer's text says, with the store's schema line as written.

## Answers

2026-11-02. **Widen to the store's bound and the forge's payload sum, or leave them to
another task?** Widen (A): this task's own acceptance (every `forge_routes` story, and the
dependents its schema change breaks at startup) cannot hold without them. Both files are
added to the header.

- `temper/crates/temper-protocol-store/src/lib.rs`, `bounds`: `change_row_report:
  forge.client.op_bytes`, beside the other forge-bounded fields. The root's validity
  (`temper-engine-domain/src/limits.rs`, `op_bytes < tasks.result_bytes` refused) makes it
  cover the producer's words; `bounds`' signature stays.
- `temper/crates/temper-engine-domain-forge/src/lib.rs`, `stored_bytes`: the
  `Stored::Change` sum adds `bytes(&row.report)?`.
- Recompute the row-size pins after the bound, name the new figures, and rerun the
  dependents that failed (`temper::suite`'s `configuration::`, `admission_edges::`,
  `box_listeners::`, `fake::check_run`) before the broad run. Change nothing else in
  either file.
