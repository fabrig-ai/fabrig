# design-upkeep: a status page no goal writes, and documents split before they fill

`DESIGN.md` keeps each tree's design in shape with three rules: a status file beside every document, a one-paragraph summary in `STATUS.md`, and a 64 KB limit that the citation gate holds. The first rule holds. The other two don't. This draft makes `STATUS.md` a fixed page and has code roll up the status files. It splits documents well before they reach the limit, and it removes drafts the design already carries in full. The figures are in section 2, and the user's words are in `design-upkeep.notes.md`.

## 1. In one page

- **What went wrong** (section 2):
  - temper's `STATUS.md` grew back into a log, because every goal is told to write its summaries.
  - Documents climb to the limit and leave it by the smallest cut that passes, so they're back at it within days.
  - Drafts the design carries in full stay in `draft/`.
- **`STATUS.md` becomes a fixed page** (section 3): a few lines saying where the status files are and how to roll them up. A script beside the citation checker prints both the page and the roll-up. The gate refuses any page other than the one the script prints, so no goal writes it.
- **Documents split before they fill** (section 4): a document past a mark at 48 KB is due a split, and its pieces come out under 40 KB. Text is never cut just to pass the limit. The roll-up lists the documents that are due, and a goal splits them one area at a time.
- **Finished drafts go** (section 5): the roll-up lists each draft whose status records no gap, and the same goal archives or deletes it.
- **Unchanged:**
  - The status file beside each document, and what its entries say (`DESIGN.md`, 2).
  - The 64 KB limit, its gate, and the trees `tools/citations/sized.txt` lists.
  - `tools/citations/move.py`.
  - `temper/docs/design/client/web/mockups/STATUS.md`, the mockups' own list of gaps, kept under their README's rules.

## 2. What happened

Each line names the main commit its figures were read at: `1815554c04` (2026-11-20 04:47Z) or `b2fdead79c` (2026-11-20 20:16Z).

### 2.1 STATUS.md

- **What was meant:** on 2026-11-02 the gaps moved to a status file beside each document. Each tree's `STATUS.md` was to become an index of those files under a one-paragraph summary (`DESIGN.md`, 4).
- **temper's page:** 4 KB on 11-02 and 86 KB on 11-11. It was cut to 18.5 KB that day, and stands at 65,490 bytes at `b2fdead79c`, 46 bytes under the limit. 199 of the 695 landings on main from 11-11 12:00Z to `1815554c04` edited it.
- **What it holds,** at `1815554c04`:
  - A summary of about 30 paragraphs (31.5 KB). It reads as a log of goals' work and names 30 goals.
  - An index with 314 entries (33 KB). 23 of them carry 14 KB of notes between them, the longest 2.2 KB.
- **The other trees,** at `b2fdead79c`: jig's page is 22.9 KB, up from 17.4 KB on 11-12, its one paragraph grown by appended sentences. skein's is 11.5 KB, smith's 3.6 KB and hearth's 4.3 KB.
- **Why it grows:** `DESIGN.md`, 2, and fabrig's goal charter (`temper/docs/charters/fabrig/goal.md`) ask every goal to write "`STATUS.md`'s summaries" as they will stand once it lands. Nothing says to replace what's there, and nothing checks the page's shape except the limit.
- **What it cost:** goal 5993's landing (PR #4145) was refused at `gate.docs`: the merged page came to 65,797 bytes. Its repair may not edit design documents (fabrig's implementer charter), so the landing waited for the user's word to condense the page.

### 2.2 Documents at the limit

- **The smallest cut:** a document that meets the limit loses just enough to pass.
  - On 11-17, `domain/landing.md` moved its section 13 out to `landing-settings.md`, going from 65,056 to 60,753 bytes. Two days later it was at 65,526.
  - `draft/notes.md` moved test-budgets' reasons into a file beside that draft, going from 65,439 to 63,411 bytes. Two days later it was at 65,531.
- **How many,** counting temper's documents and drafts (the Markdown under `temper/docs/design/` other than status files and `STATUS.md`): at `b2fdead79c`, 45 are over 48 KB, 27 over 56 KB and 16 over 60 KB. Three of them are within 100 bytes of the limit. On 11-11, at `93acbf952c`, 4 were over 60 KB. One status file, `client/pages/README.status.md`, is over 48 KB itself, at 57,007 bytes. Counted the same way, jig has 9 over 56 KB, skein 12, smith 4 and hearth 1.
- **Why:** the gate's refusal already says to split along the document's concepts. But a goal meets the limit in the middle of its own work, at the gate, while other goals edit the same document. So it takes the smallest cut that passes. Nothing splits a document before a goal runs into the limit.

### 2.3 Drafts carried in full

At `b2fdead79c`, 13 of temper's drafts outside `draft/archive/` have a status that records no gap: 74 KB with their status and execution files. One of them, `cli-c8-tools`, says so in prose rather than with the labels. They are:

- `after-landing-reds-1114`, `bench-without-sqlite`, `change-landing-fixes`, `change-stall-after`, `cli-c8-tools`;
- `flake-regate`, `focused-pyramid`, `forge-reads-targeted`, `fuzzy-context`;
- `gate-units`, `project-delete-cascade`, `project-open-superseded` and `queue-smooth`.

`DESIGN.md`, 4, says such a draft is deleted or archived.

### 2.4 What held

The status file beside each document held. temper's design documents' status files went from 673 KB in 116 files on 11-11 to 518 KB in 195 files at `1815554c04`. A file whose gaps belong to its own document, in a shape the rule fixes, stays small.

## 3. STATUS.md, a fixed page

### 3.1 The page

- **What it says:**
  - Each document's and draft's status is beside it, as `<name>.status.md`.
  - What their entries mean (`DESIGN.md`, 2).
  - The command that rolls them up (3.2).

  Nothing on it changes as work lands. The five trees' pages say the same, except for the tree's name.
- **Who writes it:** the script, as `python3 -I -B tools/citations/status.py --page TREE`. The page changes only when the script's text for it changes, which is a change to this rule.
- **Its present text:** before a tree's page is replaced, each statement on it is sorted:
  - A gap in the code that no status file records goes to the status file of the document it concerns, as **Not yet** or **Departs**.
  - Work a document defers goes to that document's "Later" or "Open questions", if it isn't there already. `DESIGN.md`, 2, keeps deferred work out of status files.
  - A draft's lag, departure or open decision goes to that draft's status, as **Design lags**, **Design departs** or **Open**.
  - A statement that only says which goal built what is dropped: git keeps it.
- **Its readers:** each tree's README and hearth's `AGENTS.md` keep their link to the page. Where a line says the page tells how far the code is, or lists what it holds, it now says the page tells where to find that out.

### 3.2 The roll-up

- **A script beside the citation checker,** `tools/citations/status.py`, reading status files from a committed tree. `python3 -I -B tools/citations/status.py [TREE...]` covers every tree when none is named.
- **How it reads a status file,** in `DESIGN.md`, 2's form:
  - **Labels:** a design document's are **Not yet** and **Departs**; a draft's are **Design lags**, **Design departs** and **Open**. Each label's text runs from the label to the next label or to the end of the file, its bullets included.
  - **Entries:** each section label (`3.2:`) in that text counts as one entry. Text with no section label counts as one. A text that reads "none" counts none. A **Not yet** entry ending in `[in flight: change N]` is in flight.
  - **Done:** every label in the file reads none, or the file's only text after its opening line is "No gap."
  - **Unread:** a file in neither form, with prose and no label for instance. The roll-up lists it as unread, never as done.
- **What it prints, for each tree:**
  - The open entries counted by label, with how many **Not yet** entries are in flight.
  - A line for each status file with an open entry, giving its date and counts.
  - What's due: the files over the mark (4.1), largest first; the drafts that are done, outside `draft/archive/` (section 5); and the unread status files.
- **Only a read:** it writes nothing, and no gate runs the roll-up. Its tests sit beside it, as the checker's do.

### 3.3 The check

- **The gate:** in a tree that `sized.txt` lists, the citation checker refuses the tree's `docs/design/STATUS.md` when its text isn't the page `status.py --page TREE` prints. The refusal says "STATUS.md is a fixed page (DESIGN.md, 4): put the gap in its document's status file".
- **By path:** the check covers only the five trees' `docs/design/STATUS.md`. The mockups' `STATUS.md` keeps its own rules.

### 3.4 The rule's words

- **`DESIGN.md`, 2:** the in-flight bullet no longer asks a goal for "`STATUS.md`'s summaries".
- **`DESIGN.md`, 4:** its last bullet reads "Each tree's `docs/design/STATUS.md` is the fixed page `tools/citations/status.py --page TREE` prints: where the status files are and how to roll them up. No goal writes it."
- **fabrig's goal charter:** its **Status** bullet no longer asks for "`STATUS.md`'s summaries".
- **temper's design README, 6:** the status files' line says that `STATUS.md` tells how to roll them up.
- **A branch from before the rule:** a goal's branch may have written the page before it was fixed. That branch's own status files already record its gaps, since `DESIGN.md`, 2, has every goal write them. So at the merge, main's page stands, and the branch's page text is dropped.

## 4. Documents split before they fill

### 4.1 The mark

- **48 KB** (49,152 bytes), below the 64 KB limit, for every file the limit covers. A document or draft over the mark is due a split, and the roll-up lists it.
- **A status file over the mark** follows its document: its entries go with their sections when the document splits.
- **No refusal at the mark:** the limit stays the gate's only refusal. Whatever line the gate refuses at, goals pass it by the smallest cut (2.2), so a refusal at the mark would only bring the same cut sooner. What's missing is a split made before a goal runs into the limit.

### 4.2 A split

- **Along the document's concepts** (`DESIGN.md`, 5), with `tools/citations/move.py` and its map. Each piece comes out under 40 KB. A concept that can't be brought under 40 KB stays under the mark. One that can't be brought under the mark stays whole, and the change says why.
- **Text moves unchanged:** a split only moves text. No text is cut just to pass the limit. Text that is wrong, out of date, or not the design's to say (`design-economy.md`, 4.2) is removed for that reason, in a change of its own, so that each change is reviewed as what it is.
- **What stays keeps its numbers:** the map names no `renumber` for the document left behind. Where a section left, one line names where it went, so a spec or a branch that still cites the old place can find it. The next pass removes those lines.
- **At the limit too:** a goal that meets the limit splits the same way, and the checker's refusal says so: "over the 65,536-byte limit: split it along its concepts, each piece under 40 KB (DESIGN.md, 5); don't cut text to fit".
- **`draft/notes.md`:** this one file holds every draft's reasons. It splits into a notes file beside each draft, `<draft>.notes.md`, and `DESIGN.md`, 4, then puts a draft's reasons there and nowhere else. No shared notes file is left to fill up.

### 4.3 The split pass

- **A goal** that splits temper's files over the mark as 4.2 says, and removes the drafts section 5 lists.
- **One area at a time:** `move.py` rewrites citations across the whole repository. So the goal's changes run one after another, each splitting one area's files: for example the domain, the protocol, the clients, and then the rest. Each change takes its batch from the files over the mark at the commit it starts from, and writes its maps alone. It's accepted when every file of its batch is under the mark.
- **The first pass:** the 45 documents and drafts over the mark at `b2fdead79c` (27 of them over 56 KB), `draft/notes.md` among them; `client/pages/README.status.md` with its document; and the 13 drafts of 2.3.
- **Later passes, by hand for now:** when the user checks in, the drafter brings the roll-up's due list for each tree, a document over 56 KB above all, and they decide together whether to run a pass. A pass they decide on starts as a goal on the user's word. Once a few passes have given experience, the passes are automated (7.1).
- **Goals in flight:** a split moves text that a goal in flight may cite or edit. A citation to a moved section finds its line (4.2). A goal whose branch edited a moved section meets a conflict at its merge. The goal's designer or coordinator resolves it with the map, since a repair may not edit design documents.
- **Other trees:** jig's, skein's, smith's and hearth's files over the mark get passes of their own, decided as temper's are. A tree's pass is one goal, and the trees' goals run one after another, since each rewrites citations across the whole repository (7.2).

## 5. Finished drafts go

- **Due:** a draft in `draft/`, outside `draft/archive/`, whose status is done (3.2). The roll-up lists it.
- **How it goes:** the split pass moves it to `draft/archive/` with `move.py`, or deletes it where nothing cites it (`DESIGN.md`, 4). Its execution file and its notes file go with it.
- **Unread first:** a draft whose status is unread, as `cli-c8-tools`' is today, is brought to the form before it counts as done.

## 6. Order

- **The words before the check:** goals stop being asked to write the page before the gate refuses a page other than the fixed one.
- **On dev.fabrig.ai,** fabrig's goal charter reaches the goals once its charter set is applied there and the live goals are moved to the new revision. A run already going keeps the revision it started with until it ends.
- **The split pass after the roll-up,** since the pass works from the roll-up's list of what's due.

## 7. Open questions

### 7.1 How often split passes run

Decided by the user, 2026-11-21: by hand at first, with no cadence and no standing approval. At the user's check-ins the drafter reminds them of the roll-up's due list, and they decide whether to run a pass (4.3). Once a few passes have given experience, the passes are automated, and their cadence and approval are decided then.

### 7.2 The other trees

Decided by the user, 2026-11-23: jig's, skein's, smith's and hearth's files over the mark get split passes of their own. The drafter proposes them with temper's, from each tree's due list at the user's check-ins (7.1), and a pass the user decides on starts as a goal on their word. Their first passes follow temper's second, one goal per tree (4.3).
