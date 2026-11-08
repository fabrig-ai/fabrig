+++
id = "ci-workflows"
part = "cards"
kind = "feature"
needs = ["asked-sentence"]
files = [
  "temper/crates/temper-engine-domain-ways/src/path.rs",
  "temper/crates/temper-engine-domain-ways/src/path/derive.rs",
  "temper/crates/temper-engine-domain-ways/src/tests/derivation.rs",
  "temper/crates/temper-engine-domain-ways/src/tests/fuzzy.rs",
  "temper/crates/temper-engine-domain/src/route/ways.rs",
  "temper/tools/gallery/src/worlds/ways.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 50
+++

# The CI card names the CI workflows' checks

## Goal

The request's task 7 (`temper/docs/plan/look-round-24/inputs/request.md`). CI's card says "Checks: the repository's CI, at the change's head" (`temper-engine-domain-ways/src/path/derive.rs`, `ci`, about line 40). The request's unpictured states ask that code's card say what it checks, "CI's workflows". The forge reads no list of a repository's workflows (`temper/docs/design/domain/forge.md`, 20, a question of its own), but it holds the landing branch's required checks, its protection's `contexts`, which CI folds to when they are present (`temper-engine-domain-forge/src/ci.rs`). The card now names them, and with none says that every check the CI workflows report counts.

Acceptance, by `temper/docs/design/domain/forge.md`, 8.13, *CI's card*, as this cut wrote it, and `temper/docs/plan/look-round-24/notes/contracts.md`, *The engine*:

1. **The input:** `ways::Repository` (`path.rs`) gains `contexts: Box<[Box<[u8]>]>` after `checks`, with its comment; the root fills it in `repository_inputs` (`temper-engine-domain/src/route/ways.rs`, about line 95) from the forge's repository record's protection, `repository.protection.contexts` (`temper-engine-domain-forge/src/boundary.rs`, about line 127; `skein/crates/skein-forge/src/api.rs`, `Protection`), in the forge's order, each context whole.
2. **The card** (`derive.rs`, `ci`): its fact "Checks", built with `cut_on_word(..., VALUE)` (from `asked-sentence`):
   - with contexts: "the CI workflows' checks A, B and C, at the change's head": the contexts joined by ", ", the last by " and " ("A and B" for two, "A" for one);
   - with none: "every check the repository's CI workflows report, at the change's head".
   Its `who`, `name` ("CI") and what it opens stay as they are. The derivation stays pure; it reads `contexts` from its input alone.
3. **The rows, no new test function:** `cards_open_the_setting_and_stay_within_the_bounds` (`tests/derivation.rs`, about line 405) takes the no-contexts words where it asserts today's, and gains two rows of the CI card over the same settings: one context, `ci / test`, giving "the CI workflows' checks ci / test, at the change's head"; three, `build`, `lint`, `test`, giving "the CI workflows' checks build, lint and test, at the change's head". Its bound sweep holds a list of contexts past `VALUE` within it, ending "…". Every literal `ways::Repository` in the crate's tests and the fuzzy suite (`tests/fuzzy.rs`) takes `contexts` (empty, or a few random short ones in the fuzzy generator, so that the invariants see both).
4. **Readers, call sites only:** every other `ways::Repository` literal in temper's tree (`rg -n 'ways::Repository \{|Repository \{' temper/crates/temper-engine-domain temper/tests/engine`) takes `contexts: Box::default()` or the record's, as the code beside it knows; name each.
5. **The gallery's worlds** (`temper/tools/gallery/src/worlds/ways.rs`, about line 259) say the no-contexts words, as the engine would over a fake repository with no required checks.
6. **Status:** delete the **Not yet** entry naming `ci-workflows` in `temper/docs/design/domain/forge.status.md` (8.13, *CI's card*). No other design line.
7. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-engine-domain-ways`, `--profile fuzzy -p temper-engine-domain-ways`, `-p temper-engine-domain`, `-p temper-service`, `-p temper-client-gallery`; `cargo check -p temper-engine-domain-world --tests`; clippy on the packages changed.

One commit or more, each with the trailer `Look-request: ask-temper-ways`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-24/notes/common.md`, `temper/docs/plan/look-round-24/notes/contracts.md`.
- `temper/docs/plan/look-round-24/inputs/request.md`, task 7; the look request's message, *States the pages do not show*, its first bullet (`notes/common.md` says how to read it).
- `temper/docs/design/domain/forge.md`, 8.13, *A stop's card* and *CI's card*; 8.7 (CI passed, the required checks); 20, *A repository's CI workflows by name*.

## Owns and may change

Owns the input's field, its gathering, the CI card, the rows, the gallery's words and the status entry. Not `words.rs`, `cards.rs` or `view.rs`; not the client's view tests' fixture words (`temper-client-view/src/tests/ask.rs`, about lines 294 and 622), which are the view's own fixtures and stay.

Options, each decided:

- **The record keeps no `protection` for a repository adopted before it** (an `Option`, or a default): no contexts; the no-contexts words.
- **A context is not valid UTF-8:** skip it, as `text` refuses one.

## Steps

1. Read the Inputs, `derive.rs`'s `ci`, `repository_inputs` and the rows named above.
2. The field and its gathering (acceptance 1, 4); the card (2); the rows (3); bite: join with ", " alone, see the three-context row fail, restore.
3. The gallery (5), the status entry (6), the runs (7); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main` after committing.
4. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| The CI card names the CI workflows' checks | landed (`derive.rs`: the landing branch's required checks by name) |
| With no required checks | landed: "every check the repository's CI workflows report, at the change's head" |
| The workflows by name, which the forge doesn't read | deferred to `temper/docs/design/domain/forge.md`, 20, *A repository's CI workflows by name* (temper/forge) |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4) |

## Report

The implementer's report: the field and the words as written, the rows and the bite with its failure, the readers changed, the runs' counts, and the account as committed.
