+++
id = "temper-docs-guide"
part = "temper-items"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-docs/src/**",
  "temper/docs/design/protocol/documentation.status.md",
  "temper/docs/development/debt/D007-docs-guide-kind.md",
  "temper/docs/development/debt/README.md",
]
effort = "medium"
minutes = 20
+++

# temper D007: `temper_docs::Kind` loses `Guide`

## Goal

`temper-docs` declares the page kinds of `temper/docs/design/protocol/documentation.md`, 2.2, and no other. `Kind::Guide` goes with its arms, its transitional names and its tests; the paging and search fixtures move to `Kind::Reference`, a kind that exists and keeps `Guide`'s bound and its no-template arm. This task closes the item.

Acceptance:

- `rg -n 'Guide|GUIDES|guides?\b' temper/crates/temper-docs/src` reads nothing.
- `temper_docs::Kind` has five variants; `Kind::bound` and `Kind::word` give the five others their values as before.
- The Departs entry for 2.2 and 3 is gone from `temper/docs/design/protocol/documentation.status.md`, and its **Not yet** entry stays.
- `temper/docs/development/debt/D007-docs-guide-kind.md` and its row are gone.
- `heavy cargo nextest run -p temper-docs` passes, its count above zero; `heavy cargo check -p temper-engine-smith -p temper-client-tools -p temper --tests` passes; `heavy cargo clippy -p temper-docs --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `temper/docs/development/debt/D007-docs-guide-kind.md`, whole.
- `temper/docs/design/protocol/documentation.md`, 2.2 and 3.

## Owns and may change

- Owns: `temper-docs/src/lib.rs`, `temper-docs/src/tests.rs`, `temper-docs/src/tests/{checks,rules,paging_search}.rs`.
- Status: `temper/docs/design/protocol/documentation.status.md`, the Departs entry only, and its date line.
- Debt: the item's file and its row in `temper/docs/development/debt/README.md`.
- No other crate names `Kind::Guide` or matches on `Kind`; `temper-client-tools/src/help.rs` writes `Kind: reference` as text, and nothing changes there. `temper/docs/help/` holds no guide page, and this task does not touch it.
- `Kind::Reference` keeps its exemption from the voice and placeholder rules: `written` stays false for it. Paying this item removes a kind, and changes no rule for a kind that has pages.

## Steps

1. `src/lib.rs`: remove the variant and its `word` arm; `Self::Reference => PAGE_BYTES`; the docs of `PAGE_BYTES` ("a reference's"), `word`, `bound` (five numbers) and `PAGES` ("reference pages, each by name") say five kinds.
2. `src/tests.rs`: `KINDS: [Kind; 5]`, `GUIDES` goes.
3. `src/tests/checks.rs`: the two arms lose `Guide`.
4. `src/tests/rules.rs`: `each_guide_is_one_of_the_six_names` goes; the bound test's arm loses `Guide`; the two "guides exempt" assertions become `fixture(Kind::Reference, ..)` with `"references exempt"`.
5. `src/tests/paging_search.rs`: every `fixture(Kind::Guide, ..)` becomes `Kind::Reference`; the kinds' word list loses `"guide"`.
6. Status, debt file and row; `python3 -I -B tools/citations/citations.py --all`.
7. Run what the acceptance names; commit (`D007: temper-docs keeps only the design's page kinds`, saying why) and push.

## Report

The acceptance search's output, the test count, and anything outside this list you had to touch.
