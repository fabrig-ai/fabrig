+++
id = "jig-tables"
part = "spot-review"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/tests.rs", "jig/crates/jig-domain-tasks/src/limits.rs", "jig/crates/jig-domain-people/src/tests.rs"]
effort = "medium"
minutes = 25
+++

# jig: two tables lose what can't fail

## Goal

Finding 8 of `temper/docs/plan/debt-24/inputs/debt22-review.md`, checked against the code, but its third copy, which the cut skips (`temper/docs/plan/debt-24/plan.md`).

Acceptance:

- `jig-domain-tasks/src/tests.rs`: `F::Separate(slot)`'s row reads `true` for its `separate` column. `Separate` has exactly the four variants the `.contains(&slot)` lists, so that's always true, and the array goes. `slot == Separate::FinishRequested` stays.
- `jig-domain-tasks/src/limits.rs`: the `const _` assertion on `OWED_WORDS` has a message, and its `#[expect(clippy::missing_assert_message, ..)]` goes.
- `jig-domain-people/src/tests.rs`: `ask_classes` keeps two `F::Project(7)` rows, not 26. `project`, `capacity::project`, `deletion::ask_project` and `reserves_answer` all read `ask.family()`, so a row beyond the first says only what `Ask::family`'s one arm says. **The two it keeps are `Ask::SetRoles`, which `every_ask_only_applies_rosters_with_their_exact_members` needs, and `Ask::RemoveProject`.** Every row of another family stays: they're what tells the families apart.
- **Rows `models::selectors` reads go last.** `tests.rs`'s `for (ask, _) in ask_classes()` asserts no row has a model selector, and `selectors` has its own arms for `Create`, `Amend`, `Propose`, `Compound`, `ChangePolicy`. Break each of those arms by hand (answer "any") and run `-p jig-domain-people --lib`: if the only test to go red is that loop on the row you'd cut, keep that row too, and say so. The rest go.
- `ask_classes`'s `#[expect(clippy::too_many_lines, ..)]` goes if the function falls under the lint, and its reason stops saying "every request variant" if it stays.
- `heavy cargo nextest run -p jig-domain-tasks --lib` and `-p jig-domain-people --lib` pass, with the same assertions on the rows left.
- Tests' net lines fall by about 50.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/inputs/debt22-review.md`, finding 8
- `jig/crates/jig-domain-people/src/boundary.rs`, `Ask::family`
- `jig/crates/jig-domain-people/src/domain/asks.rs`, `classifications`

## Owns and may change

- **Owns:** the files in the header. A helper or import only the cut rows read goes with them.
- **`settings::moves` stays,** with `codec/tasks/models.rs`'s `moves` and `codec/people.rs`'s `model_moves`: the three read three generated types, and step code holds no generic item.

## Steps

1. The three edits; the hand breaks of `selectors`' arms.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-tasks -p jig-domain-people --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; each hand break and what it turned red; the rows kept beyond the two, and why.
