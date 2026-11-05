+++
id = "away-forge"
part = "away-forge"
kind = "feature"
needs = ["home-base"]
files = [
  "temper/crates/temper-engine-domain-forge/src/away.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/away.rs",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/limits.rs",
  "temper/crates/temper-engine-domain-forge/src/reconcile.rs",
  "temper/crates/temper-engine-domain-forge/src/restore.rs",
  "temper/crates/temper-engine-domain-forge/src/views.rs",
  "temper/crates/temper-engine-domain-forge/src/adoption.rs",
  "temper/crates/temper-protocol-store/schema/store-v11.schema",
  "temper/crates/temper-protocol-store/src/generated/**",
  "temper/crates/temper-protocol-store/golden/**",
  "temper/crates/temper-protocol-store/src/keys.rs",
  "temper/crates/temper-protocol-store/src/records.rs",
  "temper/crates/temper-protocol-store/src/records/away.rs",
  "temper/crates/temper-protocol-store/src/tests/**",
  "temper/tests/codecs/tests/suite/fuzzy/store.rs",
  "temper/tests/codecs/tests/support/store.rs",
]
effort = "high"
minutes = 90
+++

# The forge's `Away` and `Decided`, the switch, the judgement and Overturn

## Goal

The connector keeps the switch and what was decided: `temper/docs/design/domain/forge.md`, 5.8 and 5.9, whole, at the forge's
boundary and in the store. The root, the service and the tool come in `away-route` and `decided-tool`.

Acceptance:

- `notes/contracts.md`, *The store* and *The forge*, hold exactly, but `CallPayload`'s arm, which is `decided-tool`'s.
- **`Event::SetAway`** does each case of 5.8's list, at its wall, with `landed` from `ProjectFacts` at a new spell, and 0
  with no facts.
- **`Event::Decided`** judges as 5.9 says and always pushes one `Request::Decided` for its `to` and `key`, `took` the
  option's words or none. Its row is saved only where it lets the call.
- **At most 64 a project:** the 65th erases the project's lowest number in its decision, and numbers go on rising.
- **`Event::Overturn`** does each case of 5.9's list.
- **`Domain::document`** answers `Document::Aways` for `AWAY_SUBJECT`, and `Document::Decisions` for
  `decided_subject(project)` in the project's scope: each the asking party's alone, `Decisions`' `took` the option's words.
- **Marks:** `AWAY_SUBJECT` at a row saved or erased, `decided_subject` likewise; nothing at a call or request that wrote
  nothing.
- **Erased** in `adoption.rs`'s project release, beside `dismissed::erase_project` and `status::erase_project`.
- **Restored and refused** as `Dismissed` is: `Away` at most `l.dismissed()`; `Decided` at most a new `l.decided()`, 64 times
  `judge_projects`, saturating; and a project's rows past 64 refused at restart.
- `temper/docs/design/protocol/store.status.md` and `domain/forge.status.md` lose what this closes; 5.8 and 5.9 stay *Not yet*
  for what `away-route` and `decided-tool` add.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/domain/forge.md`, 5.5, 5.8 and 5.9; `temper/docs/design/protocol/store.md`, sections 2 and 3.
- The models: `temper-engine-domain-forge/src/dismissed.rs` and `status.rs`, and commit `6841e695bf`, which added `ProjectFacts`
  and `Looked` whole: read its `--stat`, its readers are this task's.

## Owns and may change

Owns, as `files` lists them. Status lines as above. May also change:

- **Compiler-forced arms,** each only for that: `temper-engine-domain/src/route/panels.rs`, `src/admission.rs`,
  `src/route/playground.rs`, `src/route/forge_route.rs` (`Request::Decided`, an arm that does nothing, its comment naming
  `decided-tool`), `src/tests/resume_fixture.rs`; `temper-fake/src/check.rs`, `src/tests/check.rs`;
  `temper/tests/engine/forge/src/world.rs`, `temper/tests/engine/protocol-forge/src/protocol.rs`; `temper-service/src/family.rs`'s
  `document`, whose new arms answer `None` until `away-route`.

Options, each pre-approved:

- **A store byte:** 38 and 39; if taken when you start, the next free, in `keys.rs` and the design's table.
- **A new limit:** `l.decided()` is a product of limits held, so no configuration line is needed. If `reconcile.rs`'s budget or
  `temper-service/src/limits.rs` asks for one, add it, and regenerate `temper/docs/help/reference/limits.md` if it moves.
- **`took` past `options`:** a decoded call never holds one; `Event::Decided` answers ask, writing nothing, rather than panic.
- **A row's `Decided` bytes** in `lib.rs`' `row_bytes`: the sum of its texts, as `ProjectFacts`' brief is.

## Steps

1. **The store:** schema, `SKEIN_CODEGEN_WRITE=1 TEMPER_BUNDLE=none heavy cargo nextest run -p temper-protocol-store`, then
   unset; `keys.rs`, encoders and decoders in key order; `records.rs`, `records/away.rs`.
2. **`away.rs`:** the types, then:
   ```rust
   pub(crate) fn set_away(d: &mut Domain, env: &Env<Limits>, request: Token, token: Token, party: u64, project: u32, on: bool, until: Option<Wall>, kinds: u8, out: &mut Queue<Request>);
   pub(crate) fn decided(d: &mut Domain, env: &Env<Limits>, to: ReplyTo, key: NamedCall, party: u64, project: u32, task: u64, call: DecidedCall, out: &mut Queue<Request>);
   pub(crate) fn overturn(d: &mut Domain, request: Token, token: Token, party: u64, project: u32, number: u32, out: &mut Queue<Request>);
   pub(crate) fn erase_project(d: &mut Domain, project: u32, out: &mut Queue<Request>);
   ```
3. **Wiring:** `domain.rs`'s fields, capacities, events, restore and `RecordKind`; `lib.rs`' `key_of` and `row_bytes`;
   `reconcile.rs`; `restore.rs`; `views.rs`.
4. **Tests,** `src/tests/away.rs`, shaped as `tests/dismissed.rs` and `tests/status.rs`:
   `a_new_spell_takes_the_wall_and_the_landed_count`; `on_again_keeps_the_spell`; `off_writes_when_it_went`;
   `past_until_it_is_not_on_and_nothing_is_written`; `on_past_until_is_a_new_spell`;
   `off_past_until_keeps_the_end_at_until`; `off_with_no_row_keeps_the_choices`; `the_same_again_writes_nothing`;
   `past_the_bound_a_new_row_is_too_large`; `decided_with_the_switch_on_saves_a_row_and_answers_its_words`;
   `decided_with_the_switch_off_answers_ask_and_writes_nothing`; `a_kind_not_ticked_answers_ask`;
   `other_never_decides_whatever_the_bits`; `the_sixty_fifth_erases_the_lowest_and_numbers_rise`;
   `overturn_marks_once_then_refuses`; `overturn_of_anothers_row_is_unknown`; `the_documents_are_the_asking_partys_own`;
   `decisions_say_the_options_words`; `both_are_erased_with_the_project`; `rows_past_their_bound_are_refused_at_restart`.
   In `temper-protocol-store`'s `tests/keys.rs` and `tests/records.rs`, and the codecs' fuzzy `store.rs`: both families, as
   `Looked`'s.
5. **Check,** the implementer's text, with `temper-engine-domain`, `temper-service`, `temper-fake` as direct dependents (`heavy
   cargo check -p ... --tests`), `python3 -I -B tools/citations/citations.py --base origin/main`, then push.

## Report

As `notes/common.md`, Your report.
