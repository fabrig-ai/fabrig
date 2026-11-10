# Design documents

What whoever writes or reviews design must know, beside `AGENTS.md`:
how each tree's drafts, design documents, status files and execution
files are written and kept. Whoever writes or reviews design reads
this file; everyone else needs only `AGENTS.md`, section 1: the
design is the contract, cited by path from the repository root and
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
- **In flight:** a goal's design writes its status up front. A **Not yet** entry for a section one of the goal's changes will build ends with a tag naming that change, `[in flight: change N]`, N its task number (a goal run outside temper names its change its own way, the tag's form unchanged). When the goal closes, its project's `close` command settles the tags (`temper/docs/design/protocol/tooling.md`, 4.2): it deletes each entry whose change landed into the goal's branch, and drops the tag from each whose change ended without landing, so that the gap stays recorded. A goal writes no other entry for a section its change builds, and its draft's status, its execution file's state and `STATUS.md`'s summaries as they will stand once it lands. The citation checker reads a tagged entry as any other.

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
- Documents and sections move with `tools/citations/move.py`, from a map of where each goes: it moves the text and rewrites every citation to match, so sections are renumbered freely through it (`tools/citations/README.md`). Whoever moves one repairs what the tool lists under **By hand**. A section removed outright leaves its number unused, and whoever removes it fixes every citation to it.
- Citations are checked by code, not by hand: a branch with a broken path, section or status-label citation does not land (`tools/citations/`). Before pushing, run `python3 -I -B tools/citations/citations.py --base origin/main` from the repository root, which lists each broken one by file and line, and spend no review on hunting them.
- The reasons behind drafts go in `draft/notes.md`. Reviews, reports and debt inventories go in the tree's `docs/development/`, never in `docs/design/`.
- Each tree's `docs/design/STATUS.md` indexes its status files under a one-paragraph summary.

## 5. Their shape

A tree's `docs/design/` is read by its directory and file names before any document is opened, so it keeps the shape of the system, not the order it was written in:

- **The tree mirrors the system.** Its layers are directories (`domain/`, `protocol/`, `client/`, `shell/`). Inside one, a part with several documents is a directory named for that part, whose `README.md` is its one page and lists its documents. The design root's `README.md` gives the tree in one page, the reading order, and a line for each file below it.
- **One concept per document, named for it:** a slot's trial is a `trial` document in a `slots/` directory, not a section of a document about slots. The same name in two layers is one subject seen from each, never two subjects.
- **Small enough to read whole.** A document stays under 64 KB, and most are 10 to 40. Past the limit it splits along its concepts. The citation gate refuses a Markdown file under `docs/design/` that crosses the limit, or one already over it that grows, in the trees `tools/citations/sized.txt` lists. A tree joins that list once its layout keeps every file under the limit.
- **New text goes where it belongs.** A designer first finds the document and section the new text belongs in, and moves or renumbers what it must (section 4), rather than appending. Nothing follows a document's "Later" or "Open questions" but more of them. A change that would leave a document over the limit, one concept spread across documents, or a directory whose names no longer say what it holds restructures in the same change, with a map.
- **Testing:** a tree's testing design is at its design root, as `testing.md` or `testing/`, since it crosses the layers. A part whose tests are its own alone, such as a client's, keeps them in its directory.
