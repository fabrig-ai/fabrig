+++
id = "status-index"
part = "status"
kind = "docs"
needs = ["status-format", "status-store", "status-testing", "status-overview", "status-machine", "status-run", "status-draft"]
files = ["hearth/docs/design/STATUS.md"]
effort = "medium"
minutes = 20
+++

# STATUS.md is the index

## Goal

The status files hold the gaps, so `hearth/docs/design/STATUS.md`'s table and bullets go: it becomes the one-paragraph index `DESIGN.md`, 4
asks for, like every other tree's.

Acceptance:

- `STATUS.md` is one dated summary paragraph, then the links: every `*.status.md` under `hearth/docs/design/` once, none missing. The archive
  isn't listed.
- Every entry of the old file's "Not yet" and "Departs" is in a status file, or a status task's report dropped it: check by reading the old file
  against the status files, since `hearth/docs/design/STATUS.md` at your base has it all.
- Every design document under `hearth/docs/design/` has a status file, and `brainstorm.md` too:

  ```sh
  git ls-files 'hearth/docs/design/*.md' | grep -v -E 'STATUS\.md|\.status\.md|/archive/' | sed 's/\.md$/.status.md/' | xargs ls
  ```

- Every one closes with `Open questions`: `git grep -n -E '^## [0-9]+\. What is open' -- hearth/docs/design` prints nothing.
- `grep -c '^|' hearth/docs/design/STATUS.md` prints 0: the table is gone.
- `python3 -I -B tools/citations/citations.py --all` reads 0 and `outward` passes.

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/notes/contracts.md`, 3
- `DESIGN.md`, 4
- `smith/docs/design/STATUS.md`: the form, a paragraph and then the links under `Design documents:`
- Each status file under `hearth/docs/design/`

## Owns and may change

- **Owns:** `hearth/docs/design/STATUS.md`. Writing `docs/design/` departs from the implementer's text, as `notes/common.md` says.
- **Not to touch:** the status files: an entry that's wrong goes in your report. `hearth/docs/development/debt/**`: its citations of `STATUS.md`
  still resolve.
- **Options:** a status file's date stays its own.

## Steps

1. Write the file. The title stays `# Status`. Open with the date and `Where hearth's code lags or departs from its design, and its design from its
   draft, from the status files below.` The paragraph then says, in the status files' words, how far the code stands: what M1 and M2 have built,
   the largest gaps, the departures, the documents with no gap. It cites each by root path and holds no list of sections.
2. `Design documents:`, then `Drafts:`, each a list like `- [`domain/store.md`](domain/store.status.md)`, in `hearth/docs/design/README.md`, 2's
   reading order.
3. Commit (`hearth: STATUS.md indexes the status files`), run the checks in `notes/common.md`, push.

## Report

The count of status files; what the read against the old file found.
