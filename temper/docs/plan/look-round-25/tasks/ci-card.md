+++
id = "ci-card"
part = "workflows"
kind = "feature"
needs = ["path-stops", "roles-view", "workflows-read"]
files = [
  "temper/crates/temper-engine-domain-ways/src/path.rs",
  "temper/crates/temper-engine-domain-ways/src/path/derive.rs",
  "temper/crates/temper-engine-domain-ways/src/tests/derivation.rs",
  "temper/crates/temper-engine-domain-ways/src/tests/fuzzy.rs",
  "temper/crates/temper-engine-domain-ways/src/tests.rs",
  "temper/crates/temper-engine-domain/src/route/ways.rs",
  "temper/crates/temper-engine-domain/src/route/ways/tests.rs",
  "temper/tools/gallery/src/worlds/ways.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 60
+++

# CI's card names the repository's CI workflows

## Goal

The request's task 3, second bullet, its card half: "the CI card naming the repository's CI workflows". CI's card says only what CI's checks are (`temper-engine-domain-ways/src/path/derive.rs`, `ci`). With `workflows-read` the forge's repository record holds the names of the workflows on the landing branch (`temper/docs/design/domain/forge.md`, 4.7); the root now gathers them, but those a test gate or a tier dispatches, and the card names them in a fact "Workflows" before "Checks".

Acceptance, by `temper/docs/design/domain/forge.md`, 8.13, *CI's card*, as this cut wrote it, and `notes/contracts.md`, *The forge's CI workflows*:

1. **The input:** `ways::Repository` (`path.rs`, about line 135) gains `pub workflows: Box<[Box<[u8]>]>` after `contexts`, with its comment in the struct's style ("the landing branch's CI workflows by file, as the forge holds them, the dispatched left out").
2. **The gathering** (`temper-engine-domain/src/route/ways.rs`, `repository_inputs`, about line 95): `workflows` is the record's `workflows` (`temper_engine_domain_forge::Repository`, from `workflows-read`), in its order, leaving out each name equal, byte for byte, to the `workflow` of a gate or of a tier in `domain.forge.repository_gates(project, repository.provider)`, every tier whatever it follows (the `other` skip applies to the drawn tiers alone). No repository gates: the record's names whole.
3. **The card** (`derive.rs`, `ci`): where at least one workflow name is UTF-8, a fact "Workflows" first, the names that are, joined by ", " and the last by " and " ("A and B" for two, "A" for one), through `fact`, so past `VALUE` it is cut on a word with "…"; then "Checks", its words as they are, the contexts joined the same way. With no such name, the card is as today, "Checks" alone. Its `who`, `name` ("CI"), what it opens, and the `Stop` fields `path-stops` gives it (`needed: false`, `over` empty) stay. The derivation stays pure. One join helper serves both facts, beside `ci` in `derive.rs` (not `cards.rs`, which `roles-view` owns).
4. **The rows, no new test function** (Steps, 3).
5. **Readers, call sites only:** every other `ways::Repository` literal takes `workflows` (`rg -n 'Repository \{' temper/crates/temper-engine-domain-ways temper/crates/temper-engine-domain/src/route`): `tests.rs`'s `derive_keeps_the_branch_and_repository` (about line 356), `Box::default()`. No literal outside these two crates names it.
6. **The gallery's worlds** (`temper/tools/gallery/src/worlds/ways.rs`, the `Check` arm, about line 281): the card's facts are `("Workflows", "ci.yml")` then the "Checks" fact as it stands, as the engine gives them over a repository whose landing branch holds `.forgejo/workflows/ci.yml` and no required check. No state and no test added there.
7. **Status:** in `temper/docs/design/domain/forge.status.md`, delete the **Not yet** entry naming `ci-card` and `look-round-25` (8.13, *CI's card*), and, with it, the sentence "The workflows by name are section 20's." of the 8.13 paragraph. No other design line.
8. Runs through `heavy`, one at a time, counts in the report: `cargo nextest run -p temper-engine-domain-ways` (its `tests/fuzzy.rs` is a unit test, which this run holds), `-p temper-engine-domain`, `-p temper-engine-domain-world`, `-p temper-service`, `-p temper-client-gallery`; clippy on every package changed.

One commit or more, each with the trailer `Look-request: ask-temper-ways`; the last carries the account (Steps, 5).

## Inputs

- `temper/docs/plan/look-round-25/notes/common.md`, `temper/docs/plan/look-round-25/notes/contracts.md` (*The forge's CI workflows*; *The engine's path*, for `Stop`'s fields).
- `temper/docs/plan/look-round-25/inputs/request.md`, task 3, second bullet; the look request `ask-temper-ways`'s message, *States the pages do not show*, its first bullet (`git show 98322b3565`).
- `temper/docs/design/domain/forge.md`: 8.13, *A stop's card* (the bounds) and *CI's card*; 4.7 (what the record holds, and *Every workflow there is kept*); 20, *A workflow that runs on no pull request*.
- `temper/docs/design/domain/landing.md`, 7.1, and `temper/docs/design/domain/after-landing.md`, section 2: a test gate's and a tier's workflow, dispatched on temper's own branches.

## Owns and may change

Owns the input's field, its gathering, CI's card, the rows named below, the gallery's `Check` arm and the status entry and sentence. Not `cards.rs`, `words.rs` or the root's `view.rs`; not the client's view fixtures (`temper-client-view/src/tests/ask.rs`, about lines 293 and 643), which are the view's own and stay; not the mockups' "Checks" words.

`path-stops` restructures `derive.rs` and these tests before this task; build on its shapes as merged, leaving its rows as they are. `workflows-read` gives the record's `workflows` and already set `workflows: Box::default()` in `route/ways/tests.rs`'s `repository`; this task sets names there.

Options, each decided:

- **A name equal to a gate's or a tier's workflow but for case or a directory prefix:** kept; the match is whole bytes, as dispatch names a workflow by its file.
- **A name not UTF-8:** skipped whole, as a context is.
- **Every name left out:** no "Workflows" fact.

## Steps

1. Read the Inputs, `derive.rs`'s `ci` as `path-stops` left it, `repository_inputs`, and the rows named below.
2. The field and its gathering (acceptance 1, 2), the card (3), the literals (5); sketched:

   ```rust
   // path.rs, ways::Repository, after `contexts`
   pub workflows: Box<[Box<[u8]>]>, // the landing branch's CI workflows by file, as the forge holds them, the dispatched left out

   // derive.rs, beside `ci`
   /// `A, B and C`: the parts joined by ", ", the last by " and "; none for no part.
   fn joined(parts: &[&[u8]]) -> Option<Box<[u8]>>;
   ```

3. **The rows:**
   - `tests/derivation.rs`, `cards_open_the_setting_and_stay_within_the_bounds`: the rows without workflows keep "Checks" alone; new rows over the same settings: `deployment` with workflows `ci.yml`, giving `[("Workflows", "ci.yml"), ("Checks", "every check the repository's CI workflows report, at the change's head")]`; `three_contexts` with workflows `build.yml`, `lint.yml`, `test.yml`, giving `[("Workflows", "build.yml, lint.yml and test.yml"), ("Checks", "the CI workflows' checks build, lint and test, at the change's head")]`. Its bound sweep sets `workflows` to three copies of the long branch, as it does `contexts`, and asserts a "Workflows" fact past `VALUE` ends "…". The invalid-UTF-8 case gives `workflows` `[[0x80], b"ci.yml"]` and expects `("Workflows", "ci.yml")` before its "Checks".
   - `tests/fuzzy.rs`, `paths_end_at_a_bar_and_only_code_lands`: the generator's literal takes `workflows`, drawn as `contexts` are, 0 to 3 of `ci.yml`, `lint.yml` and `release.yaml`, so the invariants see a two-fact card.
   - `temper-engine-domain/src/route/ways/tests.rs`: `repository` (about line 59) holds `workflows` `checks.yml`, `ci.yml`, `fuzzy.yml`; `inputs_gather_the_repository_start_and_readers_roles` asserts the gathered `repository.workflows` is `[ci.yml]` (the fixture's gate dispatches `checks.yml`, its tier `fuzzy.yml`).
   - Bites: join with ", " alone, see the three-workflow row fail; drop the leave-out, see the gathering row fail; restore each.
4. The gallery (6), the status (7), the runs (8); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main` after committing. No journey reprints: no recording or print holds CI's card.
5. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| The CI card naming the repository's CI workflows | landed (`derive.rs`: "Workflows", before "Checks") |
| With none read, or all dispatched | landed: no "Workflows" fact; "Checks" as before |
| A test gate's or a tier's dispatched workflow | left out by the root (`route/ways.rs`; `temper/docs/design/domain/forge.md`, 8.13) |
| A workflow that runs on no pull request, and a workflow's own `name` | deferred to `temper/docs/design/domain/forge.md`, 20, *A workflow that runs on no pull request* (temper/forge) |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4) |

## Report

The implementer's report: the field, the gathering and the words as written, the rows and the bites with their failures, the readers changed, the runs' counts, and the account as committed.
