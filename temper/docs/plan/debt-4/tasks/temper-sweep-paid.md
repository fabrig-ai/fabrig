+++
id = "temper-sweep-paid"
part = "temper-forms"
kind = "docs"
needs = ["temper-sweep-engine", "temper-sweep-forge", "temper-sweep-service", "temper-sweep-client", "temper-sweep-worker", "temper-sweep-worlds", "temper-env-now"]
files = [
  "temper/docs/development/debt/D003-widening-try-from.md",
  "temper/docs/development/debt/D004-env-with.md",
  "temper/docs/development/debt/README.md",
  "skein/docs/development/debt/D002-index-and-length-widening.md",
  "skein/docs/development/debt/D003-env-rewraps.md",
  "jig/docs/development/debt/D043-length-widening.md",
]
effort = "medium"
minutes = 15
+++

# temper D003 and D004: read both measures, and delete the items

## Goal

The sweeps are merged: read both items' **Paid when** on the trunk, sweep what other drafts' merges
brought in since, and delete both items, their rows and the citations of them.

Acceptance:

- The items' two **Measure** commands read 0 (they print nothing, or `awk`'s empty sum), and
  `rg -c --type rust -g '!**/generated/**' '"u32 fits usize"' temper` prints nothing.
- `temper/docs/development/debt/D003-widening-try-from.md` and `D004-env-with.md` are gone, with
  their rows in `temper/docs/development/debt/README.md`.
- `python3 -I -B tools/citations/citations.py --all` reads 0.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/temper-forms.md`,
  `AGENTS.md`.
- Both items, whole. No design or status file names them.

## Owns and may change

- Owns: the two items and the README's two rows.
- May change: the three citing paragraphs below, in another tree's debt files, as the steps say.
- May change: any Rust site in `temper/` a search still lists, by the note's table, each edited
  only for its form: another draft's merge may have brought one in. Each package you edit then
  runs its clippy and its tests as the note's `What runs` says.

## Steps

1. Run the three searches over `temper` as a whole; sweep what they list.
2. Delete the two files and the two rows.
3. Each of these cites a deleted item; say what stays true, without the path:
   - `skein/docs/development/debt/D002-index-and-length-widening.md`, the sentence "jig's and
     temper's sites are their trees' items, ...": "jig's sites are its tree's item,
     `jig/docs/development/debt/D043-length-widening.md`; the helper exists, so it does not wait
     on this one."
   - `skein/docs/development/debt/D003-env-rewraps.md`, the sentence ending "not worth an item);
     temper's are temper's own item, ...": it ends at "not worth an item)."
   - `jig/docs/development/debt/D043-length-widening.md`, the sentence "skein's and temper's
     sites are their own items ...": "skein's sites are its own item (skein's D002)."
4. Run the citations check with `--all`. Commit and push.

## Report

Both measures, the citations count, and any site step 1 swept, with the draft that brought it.
