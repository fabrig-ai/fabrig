+++
id = "close"
part = "close"
kind = "docs"
needs = ["constants", "resumed-order", "hand-lists"]
files = ["hearth/docs/development/debt/D001-room-checks-and-max-out.md", "hearth/docs/development/debt/D004-service-drives-io-by-hand.md", "hearth/docs/design/domain/render.status.md", "hearth/docs/design/domain/testing.status.md", "hearth/docs/design/STATUS.md"]
effort = "medium"
minutes = 15
+++

# D001 and D004 are paid

## Goal

Both items' **Paid when** hold, so their files go, and the status files say what is true now.

Acceptance:

- Each clause of both **Paid when** is checked by command or by reading, and the report gives what each showed. A clause that doesn't hold stops this task: a `Question:`, with what's left.
  - D001: `git grep -nE 'MaxOut|MAX_OUT' -- 'hearth/crates/*'` prints nothing; `iterate`'s only `room()` is `submissions`'; the queues between the layers are `Queue::uncapped()`; each order in `hearth/docs/design/domain/render.md`, 8.5 names its predicate or machine state and its test, but for the one row `resumed-order` settled.
  - D004: `git grep -nE '\bfn tag\b|io::resume' -- hearth/crates/hearth-service-builder/src/lib.rs` prints nothing; `file_layer::owns`, `stage::up` and `stage::down` are there.
- `hearth/docs/development/debt/D001-room-checks-and-max-out.md` and `D004-service-drives-io-by-hand.md` are deleted.
- `git grep -nE 'D001-room|D004-service' -- hearth ':!hearth/docs/plan'` prints nothing, and `python3 -I -B tools/citations/citations.py --all` is at 0.
- `hearth/docs/design/domain/render.status.md`: the **Not yet** on 8.4, and on 8.5's `admits`, go. The **Departs** on 8.5, on the store step's predicate, goes. Nothing else in the file changes but what those two citations need.
- `hearth/docs/design/domain/testing.status.md`: the **Not yet** on 8.4 and 8.5, "a client that stops reading", goes.
- `hearth/docs/design/STATUS.md`: the summary says what's true: the builder's capped queues and the store step's predicate leave it, and resumed work's order keeps its clause or loses it, by what `resumed-order` found.
- Every status file you touch is dated today.

## Inputs

- `hearth/docs/plan/hearth-memory-6/notes/common.md`
- `DESIGN.md`, 2 and 4: a status file records only the gaps as they stand
- `hearth/docs/development/debt/README.md`: the branch that pays an item deletes its file, and the highest ID doesn't fall
- `hearth/docs/design/domain/render.md`, 8.4, 8.5 and 10

## Owns and may change

- **Owns:** the files under `files`. `Departs from your text:` this edits `docs/design/` status files, as it is what this task is.
- **Not the design documents' sections.** The cut wrote `render.md`, 8.4 and 8.5, and `testing.md`, 8.4, 8.5 and 11, first. If one has drifted from the code, it's a `Question:`: don't edit it to match.
- **What D001 alone still says.** Before deleting, read it once for a fact nothing else holds. The search is in `resumed-order`'s result. `domain_stage`'s step loop is in `render.md`, 10. `JSON_QUEUE` is in `render.status.md`. A fact found nowhere else goes in the report under `## Left`. It isn't a reason to keep the file.
- **The other trees' words about this file:** `skein/docs/design/io.status.md` and `skein/docs/development/debt/D049-file-layer-thin-forms.md` name `hearth-service-builder/src/lib.rs` and aren't citations of D001. skein isn't yours: leave them, and name them in the report.
- **`hearth/docs/plan/`** cites D001 in its records. The citation check skips plan directories, and they stay.

## Steps

1. Check each clause.
2. Delete both files, edit the status files, and run the citation check on the commit.
3. Commit and push. The commit says what paid them.

## Report

What each clause showed, the files deleted, what the status files now say, and anything left.
