+++
id = "people-tables"
part = "spot-review"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-people/src/tests.rs", "jig/crates/jig-domain-people/src/boundary.rs", "jig/crates/jig-domain-people/src/domain/asks.rs", "jig/crates/jig-domain-people/src/domain/acts.rs", "jig/crates/jig-domain-people/src/domain/deletion.rs"]
effort = "medium"
minutes = 45
+++

# jig-domain-people: the outcome table keeps two `Settled` rows, aliases go, and the project arm is one

## Goal

Findings 1 (the `F::Settled` rows, the aliases) and 6 (first sentence) of `temper/docs/plan/debt-22/inputs/debt20-review.md`. Net lines fall by about 50.

Acceptance:

- `outcome_classes` holds two `F::Settled` rows, not 37, and every row of another family it holds today.
- `rg -n 'OutcomeClass|AskClass' jig/crates/jig-domain-people/src` reads nothing: the tests name `OutcomeFamily` and `AskFamily`. `outcome_classes` and `ask_classes` keep their names. `ask_classes`'s rows stay: `every_ask_only_applies_rosters_with_their_exact_members` reads every one.
- `AskFamily::named_project(self) -> Option<u32>`, `pub(crate)` and `const`, lists every variant, and answers `Some` for `Project` and `Application(ApplicationScope::Project(_))`.
- `domain/asks.rs`'s `project` is `ask.family().named_project().unwrap_or(crate::DEPLOYMENT)`; `domain/deletion.rs`'s `ask_project` is `ask.family().named_project()`.
- `capacity.rs`'s `project` stays as it is: it has a third answer, `Some(DEPLOYMENT)`, for `Restore`, `Deployment` and `Application(Deployment)`, so it can't share the helper. This departs from the finding, which lists it.
- Making `named_project` answer `None` for `Application(ApplicationScope::Project(_))` turns `ask_classes_keep_all_three_scopes_and_answer_reservations` red.
- `heavy cargo nextest run -p jig-domain-people -p jig-people-world` passes.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/inputs/debt20-review.md`, findings 1, 6

## Owns and may change

- **Owns:** the files in the header, and any `#[expect(clippy::too_many_lines)]` that a shorter table leaves unfulfilled.
- **Public interface:** none changes; `named_project` is `pub(crate)`.

## Steps

1. The rows; the aliases; the helper and its two readers.
2. Break the rule by hand as Acceptance lists; restore.
3. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-people --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the rule broken and what failed.
