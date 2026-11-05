+++
id = "codegen-close"
part = "codegen-once"
kind = "docs"
needs = ["names-once", "diff-on-plan", "emit-structure"]
files = ["skein/docs/development/debt/D035-codegen-diff-and-emit.md"]
effort = "medium"
minutes = 5
+++

# skein: D035 is paid

## Goal

Check `skein/docs/development/debt/D035-codegen-diff-and-emit.md`'s **Paid when** on the merged code, and delete the item.

Acceptance:

- `rg -c 'fn (snake|pascal)\b' skein -g '*.rs' | awk -F: '{s+=$2} END{print s}'` reads 2; `rg -c 'fn rust_type\b' skein -g '*.rs' | awk -F:
  '{s+=$2} END{print s}'` reads 1.
- `rg -n 'split_bound_functions|split_record_functions' skein -g '*.rs'` finds nothing; `skein/testing/skein-codegen/src/diff/matching.rs` and
  `compare.rs` don't exist.
- `skein/docs/design/codec.status.md` says "No gap."
- D035 is deleted, and `python3 -I -B tools/citations/citations.py --all` reads 0.

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `skein/docs/development/debt/D035-codegen-diff-and-emit.md`

## Owns and may change

- **Owns:** the item's file. **If a search reads otherwise,** don't fix the code: report `Question:`, with the reading.
- **jig's mirrors request** (`jig/docs/development/debt/D007-contract-written-by-hand.md`) stays as it is: it's unblocked, not paid.

## Steps

1. The searches. Delete the item; run the citations check. No build. **This departs from the implementer's text:** no code changes.
   Commit and push.

## Report

The common note's items that apply, and each search's reading.
