+++
id = "jig-store-items"
part = "jig"
kind = "docs"
needs = []
files = ["jig/docs/design/protocol/store.md"]
effort = "medium"
minutes = 30
+++
# store.md's items in 10.1 and 15 as numbered headings

## Goal

Code and design cite `protocol/store.md`'s numbered list items as sections (15.1, 15.7, 10.1.2,
10.1.5), but they are list items, not headings, so the citations break. Give the seven steps of
10.1 and the nine items "The domain owes" of 15 numbered headings: item N of 10.1 becomes
10.1.N, item N of 15 becomes 15.N. The words stay; every surviving number stays.

Acceptance:

- After your commit, `python3 -I -B tools/citations/citations.py --all` prints none of the six
  findings `notes/findings.md` gives this task, and none that the list lacks.
- `heavy cargo nextest run -p jig-protocol-store --lib` passes, with a nonzero count: its
  `tests::words` module reads `store.md` (`include_str!`) and pins the refusals' words to it,
  and other test modules call its helpers.

## Inputs

- `temper/docs/plan/citation-burndown/notes/common.md`, then `notes/findings.md`.
- `jig/docs/design/protocol/store.md`, 10.1 and 15, the text you restructure.
- `jig/crates/jig-protocol-store/src/tests/words.rs`: the markers it splits `store.md` on (a
  refusal's quoted line, `"\n\n-"`, `"  > "`). Keep every marker's text and the blank line or
  list item that ends it.
- `DESIGN.md`, 4.

## Owns and may change

- Owns `jig/docs/design/protocol/store.md`, 10.1 and 15 only.
- Leaves as they are: the citing files (`positions.rs`, `retire.rs`, `people.md`, `tasks.md`,
  `generations.rs`), whose numbers become right; citations by step ("store.md 10.1, step 4",
  "10.1 step 5"), which stay true; `store.md`'s own "(15.7)" in 7.2.
- Departs from your text: this task edits `docs/design/` (text only; `notes/common.md`). It
  changes no Rust, so no fmt or clippy.
- Options, pre-approved:
  - Where an item's bold label opens a sentence ("**The stores open** (step 1), and ..."), the
    heading is the label and the paragraph keeps the whole sentence; where the label stands
    alone ("**The lock:** io's `Lock` ..."), the paragraph starts at the words after it, with a
    capital.
  - If a `tests::words` marker would change, keep the marker's text exactly and bend the
    heading's placement instead; if no placement keeps it, stop with a `Question:`.

## Steps

1. 10.1 (a `###` section; its items become `####`):

   ```markdown
   ### 10.1 Generations at open

   <the lead paragraph, unchanged>

   #### 10.1.1 The lock

   io's `Lock` on the deployment's root, held as long as the process (skein's `io.md`, 5). ...

   #### 10.1.2 `active`

   A whole-file load of at most 21 bytes: ...

   ...

   #### 10.1.7 At the binary's levels

   ...

   #### 10.1.8 Its refusals

   Each refusal names the deployment's directory and what to do: ... <the rest of 10.1,
   unchanged, through the `Older` and `Check` paragraph>
   ```

   The text after item 7 ("Each refusal names ...", the refusals' words, the `OpenRefusal` block)
   gets 10.1.8 so it does not read as part of step 7.

2. 15 (a `##` section; its items become `###`):

   ```markdown
   ## 15. What it owes and is owed

   The domain owes 15.1 to 15.9; skein owes 15.10, and the application 15.11.

   ### 15.1 Live before history

   In each child's keys: the tasks child's history and ended tasks after its live families ...

   ...

   ### 15.9 The accounts' family

   ...

   ### 15.10 What skein owes

   io's `OpenWrite` and `Cut` (skein's `io.md`, section 5), for 8.2 and 8.4. ...

   ### 15.11 What the application owes

   Its keys' encoders, its records' codecs, ...
   ```

   Item 1 is "live before history" (15.1, cited as "closed funding sources move to history");
   item 7 is "keys for single reads" (15.7, cited as "positions for single reads"). Check both
   after the edit.
3. Commit. Run `citations.py --all` (acceptance), then
   `heavy cargo nextest run -p jig-protocol-store --lib`. Push.

## Report

`notes/common.md`'s report, plus the headings you wrote for 10.1 and 15 (one line each), and
the test count.
