+++
id = "measure-recheck"
part = "bundle-helpers"
kind = "docs"
needs = ["bundle-temper", "undecoded", "listener-text", "temper-names"]
files = ["temper/docs/development/debt/D001-reference-service-copy.md"]
effort = "medium"
minutes = 10
+++

# D001: the unchanged measure, rechecked

## Goal

D001's own text asks to recheck its unchanged measure after sharing helpers. Do that once every task of this plan that
touches a measured file has merged, and write down what's now true. D001 stays open: this plan can't pay it, and whether
its bound changes is the user's question (`d001-bound` in `plan.toml`). So `notes/common.md`'s "Paying an item" doesn't
apply here: nothing is deleted.

Acceptance:

- The item's **Measure** command is unchanged, and the figure after it is what it reads now.
- **Paid when**, **Kind**, **Value**, **Size** and **Status** are unchanged, as is the item's row in the README.
- A new **Checked** line carries today's date (`date -u +%F`) and your base's short commit.
- **What** gives the new per-file figures, and no longer says the bundle helpers are copied.
- **What pays it back** says what this cut found, in the words under `Inputs`.
- `python3 -I -B tools/citations/citations.py --all` reads 0.

## Inputs

- `temper/docs/plan/debt-10/notes/common.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `temper/docs/development/debt/D001-reference-service-copy.md`, whole.
- What to write under **What pays it back**, as it stands, keeping its last sentence's sense: that the remaining overlap
  was sorted by hand at debt-10's cut, from 356 lines: about 127 generic (braces, imports, signatures), about 176 owned by
  the application under `jig/docs/design/protocol/service.md`, 11 or by this item (passes, tokens, sizes), and about 53
  movable. Every move that keeps section 11 would leave about 310; below 200 needs a clients' part holding the pass, which
  11.6 rules out. The bundle helpers are shared, outside the measure. The question of the bound is the user's.

## Owns and may change

- Owns that one file, and nothing else. No code.

## Steps

1. Run the measure in bash from the repository root, per file and in total, and record both.
2. Edit the item. Citations. Commit, push.

## Report

As the implementer's text says: the measure's new total and per-file figures. No test ran; a docs-only task needs none.
