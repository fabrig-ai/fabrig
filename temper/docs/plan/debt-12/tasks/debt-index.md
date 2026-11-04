+++
id = "debt-index"
part = "debt-index"
kind = "feature"
needs = []
files = ["tools/debt/**", "skein/docs/development/debt/README.md", "smith/docs/development/debt/README.md", "jig/docs/development/debt/README.md", "temper/docs/development/debt/README.md", "hearth/docs/development/debt/README.md", "hearth/docs/design/domain/testing.md"]
effort = "high"
minutes = 40
+++

# Print each tree's debt index, and drop the tables

## Goal

Items paid in parallel in one tree conflict on its README's table. The table goes: a small Python command
prints the index from the item files, and a branch that pays an item touches its file alone. One task for the
tool and the five READMEs, so no commit has a README that names a command that isn't there.

Acceptance:

- `python3 -I -B tools/debt/debt.py` from the repository's root prints all five indexes and exits 0;
  `python3 -I -B tools/debt/debt.py skein` prints one.
- An item file that lacks a field is reported by path and field, and the command exits 1.
- `python3 -I -B tools/debt/test_debt.py` passes, and each rule below was broken once by hand and seen red.
- No README keeps a table: `rg -n '^\|' {skein,smith,jig,temper,hearth}/docs/development/debt/README.md` finds
  nothing.
- `rg -n 'its row|row (below|here)' {skein,smith,jig,temper,hearth}/docs/development/debt/*.md` finds nothing.
  `reviews/` are dated records and stay as they are.
- Each README says how to print its index and holds the highest ID used, and the rest of it is as it was.
- `python3 -I -B tools/citations/citations.py --all` reads 0.
- `heavy cargo nextest run -p fabrig-boundary --test outward` passes.
- No Cargo manifest, no third-party import, no line added to `.config/gate/`: the request says it needs no
  gate.

## Inputs

- `temper/docs/plan/debt-12/notes/common.md`
- `temper/docs/plan/debt-12/inputs/request.md`, section 2
- `tools/citations/README.md` and `tools/lints/test_lints.py`, for how a tool here reads and how its tests
  build a repository per test
- The entry format, in any tree's `docs/development/debt/README.md`

## Owns and may change

- **Owns:** `tools/debt/` (new: `debt.py`, `test_debt.py`, `README.md`) and the five
  `docs/development/debt/README.md`.
- **May change:** `hearth/docs/design/domain/testing.md`, one citation (step 4), and nothing else there.
- **Not to touch:** the item files `D*.md`: none says "its row" at this base. `reviews/`. A table's rows are
  not moved anywhere: the files hold them.
- **Options:**
  - If an item file on your base lacks a field, that file is wrong and the command's report is right: add the
    missing line to that file from the table's row, and name it in the report.
  - If an item's row and its file disagree beyond the cases in step 3, the file wins, and the report names the
    row.
  - If a table at your base has an ID the files lack, or the reverse, the files win, and the report names it.

## Steps

1. `tools/debt/debt.py`, about 70 lines at most, `#!/usr/bin/env python3`, standard library only, a module
   docstring in the shape of `tools/citations/citations.py`'s (usage, exit codes). It runs from the repository's
   root and reads paths relative to the working directory, so a test runs it in a built repository with
   `cwd=`.

   ```text
   python3 -I -B tools/debt/debt.py [TREE]     TREE: skein smith jig temper hearth; none: all five
   ```

   - Exit 0: the index. For each tree in that order, a heading `## <tree>`, a blank line, the table and a blank
     line between trees:

     ```md
     | ID | title | kind | value | size | status |
     |---|---|---|---|---|---|
     | D002 | Index and length conversions are written by hand beside lib's helpers | primitives | medium | M | open |
     ```

   - **Items** are `<tree>/docs/development/debt/D<digits>-*.md`, in order of their number. `README.md` and
     `reviews/` are not items.
   - **The title** is the first line, `# <ID>: <title>`, whose ID is the file's.
   - **The fields** are the lines `- **Kind:** ...`, `- **Value:** ...`, `- **Size:** ...`,
     `- **Status:** ...` above the file's first `## ` heading. A field's value is its text up to the first
     ` (`: `high (a new field costs one edit)` prints `high`, `open (waits on skein's D044)` prints `open`,
     `dead code` prints `dead code`. The reason stays in the file.
   - **Exit 1:** a file that lacks one of the five, or whose field is empty, is reported on stderr, one line
     for each lack: `<path>: no <title|kind|value|size|status>`. Every such file of the trees asked for is
     reported, and no table is printed.
   - **The README's note:** each README holds the bold line of step 3. A README that lacks it, or whose ID is
     below a file's, is reported the same way (`<path>: no highest ID`, `<path>: highest ID D012 is below D013`)
     and exits 1. This is the one hand-kept line left, and it's what stops a paid ID being handed out again.
   - **Exit 2:** a usage line on stderr for anything else on the command line, or for a tree's directory that
     isn't there (run from elsewhere).

2. `tools/debt/test_debt.py`, about 110 lines at most, `unittest`, each test on a repository built in a
   temporary directory, `debt.py` run by `subprocess` under `-I -B` with `cwd=` there. One test each:

   - one tree's table, in order of number, with `D010` after `D002` and a tail cut from each of value, size and
     status, and `dead code` whole;
   - no tree prints all five, headed, in the order above;
   - a file lacking each of the five is named with what it lacks, exit 1, no table on stdout, and two bad files
     are both named;
   - `README.md` and `reviews/2026-11-04-x.md` are not read as items;
   - a README with no note, and one whose note is below a file's ID, exit 1;
   - an unknown tree, and two trees, exit 2;
   - this repository, `cwd=` its root: exit 0 and a row for every item file.

   Break each by hand once: the field check, the first-` (` cut, the order by number, the note's check.

3. The READMEs. Find each ID's highest use:

   ```sh
   git log --format= --name-only -- TREE/docs/development/debt/ | grep -oE '/debt/D[0-9]+' | sort -u | tail -1
   ```

   At this plan's base: skein D048, smith D019, jig D043, temper D012, hearth D013. Take what the command
   prints on your base, if it's higher.

   Before you delete a table, run the command on the tree and compare every row with it. At this plan's base
   they agree but for one, skein's D044, whose title the table lags: the file wins. Temper's table has no links
   and the others do; that is no difference.

   - **The rule at the top.** Replace the sentence that says a branch deletes an item's file and its row:

     - skein: `- **The branch that pays an item deletes its file;** there is no row to delete, and git keeps the history.`
       (wrapped as the file wraps);
     - smith, hearth, temper: `- **The branch that pays an item deletes its file;** there is no row to delete. Git keeps the history.`;
     - jig, in its first paragraph: `A branch that pays an item deletes its file; there is no row to delete (git keeps the history).`

   - **The note.** A bullet of its own, on one line, in the list that says IDs are never reused:

     ```md
     - **The highest ID used is D048.** The branch that adds an item raises it, so a paid ID isn't handed out again.
     ```

     In jig, which says that outside a list, it goes in the list below section 2's entry format.

   - **The index.** The heading stays (`## 4. Index` in skein, `## 3. Index` in jig, `## Index` in the others:
     surviving sections aren't renumbered). The table is replaced by:

     ```md
     `python3 -I -B tools/debt/debt.py skein` prints it, from the repository's root, from each item file's
     title, kind, value, size and status; with no tree, all five (`tools/debt/README.md`).
     ```

   Everything else in each README stays as it is.

4. `hearth/docs/design/domain/testing.md`, in section 11's "The newer model", cites the README for items D001 to
   D003, which it no longer lists: `(`hearth/docs/development/debt/README.md`, D001 to D003)` becomes
   `(`hearth/docs/development/debt/`, D001 to D003)`. That line's reflow only. This departs from the
   implementer's text, for that line.

5. `tools/debt/README.md`, about 25 lines at most, in the shape of `tools/citations/README.md`: the commands,
   what is read, the exit codes, and that it needs no gate.

6. Commit and push. Tool and tests first; the READMEs and the citation in a second commit.

## Report

The report of your implementer text, and: the rules you broke and what failed; each highest ID you wrote and
the command's output; each row that disagreed with its file; the lines of `debt.py`, `test_debt.py` and
`README.md`.
