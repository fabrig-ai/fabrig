+++
id = "plans-out"
part = "plans-out"
kind = "docs"
needs = ["qa-page", "acceptance"]
files = [
  "temper/docs/development/v0.1.0-ladder.md",
  "temper/docs/development/reference-walks.md",
  "temper/docs/plan/v0.1.0-ladder/**",
  "temper/docs/plan/reference-walks/**",
  "tests/boundary/outward.txt",
  "temper/docs/development/qa.md",
]
effort = "medium"
minutes = 15
+++

## Goal

What the lab now keeps outside the repository leaves it: the two generated backlogs and the two plan
directories, whose stories `temper/docs/development/acceptance.md` now holds. `qa.md` links that page.

Acceptance:

1. None of these exists: `temper/docs/development/v0.1.0-ladder.md`,
   `temper/docs/development/reference-walks.md`, `temper/docs/plan/v0.1.0-ladder/`,
   `temper/docs/plan/reference-walks/`.
2. `tests/boundary/outward.txt` holds its three comment lines and no entry, and the outward check is
   green.
3. This prints nothing:

   ```sh
   git grep -n -E 'v0\.1\.0-ladder|reference-walks' -- . ':!*/docs/plan/*'
   ```

4. `temper/docs/development/qa.md`, section 3, cites `acceptance.md`, and its eight headings are unchanged.
5. Both checks of `temper/docs/plan/qa-out/notes/common.md` pass.

## Inputs

- `temper/docs/plan/qa-out/inputs/request.md`, section 4 and Bounds.
- `temper/docs/plan/qa-out/notes/common.md`, all of it; it departs from your implementer's text.
- `temper/docs/plan/qa-out/notes/contracts.md`, C3.

## Owns and may change

- **Owns:** the four paths of acceptance 1, to delete.
- **May change:** `tests/boundary/outward.txt`, those two lines only; `temper/docs/development/qa.md`,
  the one sentence of step 3.
- **Touches nothing else.** The other `docs/plan/` directories keep what they name of these paths
  (`notes/common.md`).
- **Options:**
  - If acceptance 3 finds a citation outside `docs/plan/`, or `citations.py` does, it arose since this
    plan was cut. Rewrite it to cite `temper/docs/development/acceptance.md` or drop the clause,
    changing nothing else in that file, and say so in the report.
  - If `temper/docs/development/acceptance.md` is not on your trunk, stop: `Question:`. Nothing is
    deleted before it stands.

## Steps

1. **Check the stories stand:** `test -s temper/docs/development/acceptance.md`.
2. **Delete:**

   ```sh
   git rm -r -q temper/docs/development/v0.1.0-ladder.md temper/docs/development/reference-walks.md \
     temper/docs/plan/v0.1.0-ladder temper/docs/plan/reference-walks
   ```

3. **Link the page.** In `temper/docs/development/qa.md`, section 3, end the first bullet with the
   sentence `Each rung's stories are `acceptance.md`.`
4. **Drop the two lines** from `tests/boundary/outward.txt`: `temper/docs/development/reference-walks.md 1`
   and `temper/docs/development/v0.1.0-ladder.md 2`. Its three comment lines stay.
5. **Check, commit, push.** Commit first: both checks read what git tracks. Run acceptance 2 to 4 and
   the two checks of `notes/common.md`. Push your branch.

## Report

As your implementer's text says. Also: any citation you rewrote, by file and line.
