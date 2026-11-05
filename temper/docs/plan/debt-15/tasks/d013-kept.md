+++
id = "d013-kept"
part = "d013-kept"
kind = "docs"
needs = []
files = ["hearth/docs/development/debt/D013-records-three-times.md"]
effort = "medium"
minutes = 5
+++

# hearth: D013 is kept

## Goal

The user decided (the request's part 2): `hearth/docs/development/debt/D013-records-three-times.md` is `kept`, for the
layers' own vocabularies. Its **Paid when** allows that.

Acceptance:

- The item's status line reads `- **Status:** kept (the layers' own vocabularies)`, and its **Checked** line carries
  today's date (`date -u`) and your base's commit.
- `python3 -I -B tools/debt/debt.py hearth` exits 0 and prints D013 as `kept`.

## Inputs

- `temper/docs/plan/debt-15/notes/common.md`
- `hearth/docs/development/debt/README.md`, the entry's form.

## Owns and may change

- **Owns:** that item's file, those two lines. Nothing else: no code, and no other item (a plan beside this one is
  changing hearth).

## Steps

1. The two lines.
2. The debt tool and the citations check. Commit and push. No build runs.

## Report

The common note's items; no break applies, since nothing here pins a rule.
