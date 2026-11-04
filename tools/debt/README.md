# Debt indexes

Print indexes from debt item files, from the repository's root:

```sh
python3 -I -B tools/debt/debt.py          # all five trees
python3 -I -B tools/debt/debt.py skein    # one tree
python3 -I -B tools/debt/test_debt.py
```

- **What is read:** each tree's `docs/development/debt/D<digits>-*.md`,
  ordered by number, and its `README.md`'s bold highest-ID note.
  Trees print in order: skein, smith, jig, temper, hearth.
- Each row takes the first-line title (whose ID must match the filename),
  kind, value, size and status above the first `## ` heading. A field's
  text after its first ` (` stays in the item file. Reviews are not items.
- Missing or empty fields are reported by path and field on stderr.
  A missing highest-ID note, or one below an item's ID, is also reported.
  Every finding in the requested trees is reported before any index prints.
- Exit 0 indexes printed, 1 findings (no index), 2 usage or a missing
  tree directory. The command reads paths relative to the working directory.
- This command needs no gate.
