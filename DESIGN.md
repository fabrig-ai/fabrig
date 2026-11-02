# Design documents

What whoever writes or reviews design must know, beside `AGENTS.md`:
how each tree's drafts, design documents, status files and execution
files are written and kept. fab's charters for the roles that write or
review design carry it; everyone else needs only `AGENTS.md`, section 1:
the design is the contract, cited by path from the repository root and
section.

## 1. Three layers

Each tree's `docs/design/` holds three layers, and each one's status says where the next one lags:

- **Drafts** (`docs/design/draft/`) hold the vision: what the tree is to become. They are the input the design is written from and win over it, unless a draft's status says otherwise. Drafts agree with one another.
- **Design documents** are the contract the code follows. They agree with one another, and with the drafts except where a draft's status records a gap. Every design document closes with a "Later" or "Open questions" section for what it defers or leaves undecided; its designer keeps it and its status.
- **The code** is what is built.

## 2. Status files

Beside every draft and every design document is its status file, `<name>.status.md`. It is dated and records only the gaps as they stand now, by section of its document, never history: what is built or landed leaves it, and git keeps the record. A document with no gap has a status that says so.

- A design document's status: **Not yet** (designed, not built) and **Departs** (built otherwise, with the reason). What the document itself defers (its "Later" or "Open questions") is no gap, and the design text carries no build state.
- A draft's status: **Design lags** (the design document and section that still say otherwise or say nothing, and what), **Design departs** (what the design does instead, why, and who decided) and **Open** (decisions awaiting the user).

```
# <name>: status

<date>. Where <the tree's code | the design> lags or departs from [<name>.md](<name>.md).

- **Not yet:** 3.2: ... 5: ...
- **Departs:** 4.1: ..., because ...
```

## 3. Execution files

A draft being carried out may have an execution file beside it, `<name>.execution.md`: the milestones that carry it into the design and the code, each with the draft's part it executes, what it needs, the steps its walk covers and its state, kept current. It designs nothing: each milestone writes its own design from the draft as it runs. It goes when its draft does.

## 4. Keeping them

- A retired or superseded draft is deleted, not marked. A draft whose vision the design now carries in full is deleted or moved to `draft/archive/`.
- Surviving sections are never renumbered, since code and documents cite them: a removed section leaves its number unused. Whoever removes or moves a document or section fixes every citation to it in the repository.
- Citations are checked by code, not by hand: the queue rejects a branch with any broken path, section or status-label citation (`tools/citations/`). Before pushing, run `python3 -I -B tools/citations/citations.py --base origin/main` from the repository root, which lists each broken one by file and line, and spend no review on hunting them.
- The reasons behind drafts go in `draft/notes.md`. Reviews, reports and debt inventories go in the tree's `docs/development/`, never in `docs/design/`.
- Each tree's `docs/design/STATUS.md` indexes its status files under a one-paragraph summary.
