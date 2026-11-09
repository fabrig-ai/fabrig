+++
id = "tier-words"
part = "tier"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/startup/application/landing.rs",
  "temper/crates/temper-service/src/tests/startup.rs",
  "temper/crates/temper-service/src/tests/reference_landing.rs",
  "temper/crates/temper-engine-domain/src/translate.rs",
  "temper/crates/temper-engine-domain/src/route/ways.rs",
  "temper/crates/temper-engine-domain/src/route/ways/tests.rs",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/landing.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-ways/src/path.rs",
  "temper/crates/temper-engine-domain-ways/src/path/derive.rs",
  "temper/crates/temper-engine-domain-ways/src/tests/derivation.rs",
  "temper/crates/temper-engine-smith/src/see.rs",
  "temper/crates/temper-engine-smith/src/tests/see.rs",
  "temper/docs/examples/reference/landing.json",
  "temper/docs/design/protocol/configuration.status.md",
  "temper/docs/design/domain/forge.status.md",
  "temper/docs/design/domain/agent.status.md",
]
effort = "high"
minutes = 100
+++

# A tier's description, from the configuration to its card

## Goal

A tier in the configuration document may carry a `description`, what it checks in
words; the start reads and bounds it, the forge keeps it with the tier, and the
tier's stop on How work runs' path says it on its card as "Checks", as `see
settings gates` does for Ask temper. The reference's two tiers carry theirs. This
is P3e's tier's description (`temper/docs/design/draft/whole-story.execution.md`,
P3e); nothing draws anything new in the client, which draws a card's facts as the
wire gives them.

Acceptance:

1. **The member,** `temper/docs/design/protocol/configuration.md`, section 3, *A
   tier's `description`*: `tier` in
   `temper/crates/temper-service/src/startup/application/landing.rs` reads an
   optional `description`; absent is none (an empty `Box`); present, it is 1 to
   120 bytes of UTF-8 holding no `char` for which `char::is_control` is true,
   and anything else refuses the start at that member, as the tier's other
   members are refused (its path and a problem naming the bound). The 120 is
   `pub const DESCRIPTION_BYTES: usize = 120;` beside `Tier` in
   `temper-engine-domain-forge/src/boundary.rs`, exported as `Tier` is, citing
   section 3; the forge's own check of a configured tier's bounds
   (`temper-engine-domain-forge/src/landing.rs`, about line 114) refuses a
   description past it as it refuses a name past its bound.
2. **Carried:** `TierCheck` (`temper-engine-domain/src/translate.rs`),
   `temper_engine_domain_forge::Tier` (`boundary.rs`) and
   `temper_engine_domain_ways::Tier` (`path.rs`) gain `pub description:
   Box<[u8]>`, last, empty for none (`notes/contracts.md`, section 7);
   `project_landing` and `repository_inputs` (`route/ways.rs`) copy it.
3. **The card,** `temper/docs/design/domain/forge.md`, 8.13, *A tier's card*:
   `tiers` in `temper-engine-domain-ways/src/path/derive.rs` adds, after the fact
   "Runs", a fact "Checks" whose value is the description whole, where it is not
   empty; a tier with none keeps its one fact. On a goal's branch and on the
   landing branch alike.
4. **`see settings gates`** (`temper-engine-smith/src/see.rs`, the tier line,
   about line 290): `, checks DESCRIPTION` after `follows ...` and before the
   ignored paths, where the tier has one
   (`temper/docs/design/domain/agent.md`, 8.4.1).
5. **The reference:** `temper/docs/examples/reference/landing.json`'s tiers carry
   `"description": "the slow suites"` (`slow`) and `"description": "the gate
   without its slowest suites"` (`goal`), as
   `temper/docs/plan/ui-p3e-words/notes/contracts.md`, section 8, has them;
   `tests/reference_landing.rs`' expected tiers take them.
6. **Rows,** no new test function:
   - the start's refusals of a tier's members (`temper-service/src/tests/startup.rs`;
     `rg -n 'span_ms|artifact' temper/crates/temper-service/src/tests/startup.rs`
     finds the table): a description accepted and carried; one empty, one of 121
     bytes and one holding a line break (`\n` in the JSON) each refused at
     `.description`;
   - the derivation (`temper-engine-domain-ways/src/tests/derivation.rs`, the row
     about line 557 whose card has `facts(&[("Runs", "the tier slow after each
     landing")])`): a row where the tier has a description, its card's facts
     "Runs" then "Checks"; the existing row, with none, keeps one fact;
   - `see settings gates` (`temper-engine-smith/src/tests/see.rs`, about line
     304): a tier with a description, its line ending in `, checks ...`.
7. **Bites:** drop the "Checks" fact, see the derivation row fail; accept a
   description of 121 bytes, see its row fail; restore each.
8. **Status:** delete the bullets naming `tier-words` in
   `temper/docs/design/protocol/configuration.status.md` (3, 13.9),
   `temper/docs/design/domain/forge.status.md` (8.13) and
   `temper/docs/design/domain/agent.status.md` (8.4.1) (`notes/common.md`,
   *Status files*).

## Inputs

- `temper/docs/design/protocol/configuration.md`, section 3 (*`landing`*, *A
  tier's `description`*) and 13.9 (*The tiers*).
- `temper/docs/design/domain/forge.md`, 8.13, *A tier's card*.
- `temper/docs/design/domain/agent.md`, 8.4.1, the `settings gates` row.
- `temper/docs/plan/ui-p3e-words/notes/contracts.md`, sections 1, 7 and 8.
- `temper/docs/plan/ui-p3e-words/notes/common.md`.

## Owns and may change

- Owns: the `description` member, its bound and its refusals; the three types'
  field; the tier's card's "Checks"; the `see` line; the reference's two
  descriptions; the rows; the three status bullets.
- Compile-forced, the new field alone (`description: Box::default()` or the
  fixture's own), in any file of temper's tree that builds one of the three
  types; the survey found: `temper-engine-domain-forge/src/tests/landing.rs` and
  `src/tests/verification.rs`; `temper-engine-domain/src/route/forge_route.rs`
  and `route/forge_route/tests.rs`; `temper-engine-domain-ways/src/tests/fuzzy.rs`;
  `temper-engine-smith/src/tests/see.rs`; `temper/tests/engine/domain/src/tier_runs.rs`
  and `tests/suite/ways.rs`; `temper/tests/client/src/front/ways.rs`. Name each in
  the report. `forge_route.rs` is the coding runs' seam: the field's literal
  alone there.
- **Stored:** if the forge stores a tier in the store (a record of
  `temper-protocol-store`), the description is stored with it, the store's schema
  changing in place with no compatibility, its generated code and goldens
  regenerated; say so. If the tier is held from the configuration alone, nothing
  is stored.
- **Help:** if a `temper/docs/help/` page lists a tier's members (`rg -n
  'span_ms' temper/docs/help`), it gains `description` in its words, within its
  bound; a `docs/help/reference/*` page made stale is regenerated by its tool.
- Not yours: the wire, How work runs' drawing and the stock ways
  (`words-wire`, `way-headlines`, `how-words`), which build beside this task in
  other files of the same crates.

## Steps

1. Read the inputs, then `tier` and `check` in `landing.rs`, `project_landing`,
   `repository_inputs`, and `tiers` in `derive.rs`.
2. The member and its refusal rows; then the field through the three types
   (`heavy cargo check -p temper-service --tests` while errors last); the card and
   its row; the `see` line and its row; the reference.
3. The bites (acceptance 7).
4. Narrow tests: `heavy cargo nextest run -p temper-engine-domain-ways --lib`, `-p
   temper-engine-smith --lib -E 'test(/see/)'`, `-p temper-service --lib -E
   'test(/startup|reference_landing/)'`.
5. The tier reaches every reader of the forge's `Tier`: run the focused tests of
   `-p temper-engine-domain-forge`, `-p temper-engine-domain`, `-p
   temper-engine-domain-ways`, `-p temper-engine-smith`, `-p temper-service`, and
   `heavy cargo check --tests -p temper-engine-domain-world -p temper-client-world
   -p temper` (the world packages' names as their manifests give them).
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy`
   for every package changed, `--all-targets -- -D warnings`.
7. Commit (why, in the message); `python3 -I -B tools/citations/citations.py --base
   origin/main`; push.

## Report

As the implementer's text says, and: the member's refusal words; each
compile-forced file; whether the tier is stored; the rows and the bites.
