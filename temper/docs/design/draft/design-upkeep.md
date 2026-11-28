# design-upkeep: a status page no goal writes, and documents split before they fill

`DESIGN.md` keeps each tree's design in shape with three rules: a status file beside every document, a one-paragraph summary in `STATUS.md`, and a 64 KB limit that the citation gate holds. The first rule holds. The other two don't. This draft makes `STATUS.md` a fixed page and has code roll up the status files. It splits documents well before they reach the limit, and it removes drafts the design already carries in full. The figures are in section 2, and the user's words are in `design-upkeep.notes.md`.

## 1. In one page

- **What went wrong** (section 2):
  - temper's `STATUS.md` grew back into a log, because every goal is told to write its summaries.
  - Documents climb to the limit and leave it by the smallest cut that passes, so they're back at it within days.
  - Drafts the design carries in full stay in `draft/`.
- **`STATUS.md` becomes a fixed page** (section 3): a few lines saying where the status files are and how to roll them up. A script beside the citation checker prints both the page and the roll-up. The gate refuses any page other than the one the script prints, so no goal writes it.
- **Documents split before they fill** (section 4): a document past a mark at 48 KB is due a split, and its pieces come out under 40 KB. Text is never cut just to pass the limit. The roll-up lists the documents that are due, and a goal splits them one area at a time.
- **Finished drafts go** (section 5): the roll-up lists each draft whose status records no lag and no open decision, and the same goal archives or deletes it.
- **Unchanged:**
  - The status file beside each document, and what its entries say (`DESIGN.md`, 2).
  - The 64 KB limit, its gate, and the trees `tools/citations/sized.txt` lists.
  - `tools/citations/move.py`.
  - `temper/docs/design/client/web/mockups/STATUS.md`, the mockups' own list of gaps, kept under their README's rules.
- **After the first passes** (2.5): a draft with only **Design departs** left is done and goes to the archive with them (3.2, section 5). A document keeps its text in numbered sections, and one whose opening passes 8 KB is due to be numbered (4.1, 4.2).
- **After the third pass** (2.6): each pass removes the earlier passes' "where it went" lines, and the checker refuses a citation to a section that holds only one (4.2). `move.py` carries a literal's exception in `tools/citations/allow.txt` with it, and the roll-up leaves out the pages generated from a table (3.2).

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
  - The shared notes file moved test-budgets' reasons into a file beside that draft, going from 65,439 to 63,411 bytes. Two days later it was at 65,531.
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

### 2.5 What the first passes showed

Read at `d50c06b8e3` (2026-11-24 18:27Z), against the first pass's landing, `0359224cd6` (2026-11-22 22:53Z), and `42e1e8fcdf` (2026-11-21 19:00Z).

- **The page held:** all five trees' `STATUS.md` stayed the fixed page, 830 to 839 bytes, through the 142 landings since it was fixed, and none of them edited one.
- **The splits held:** 85 files across the five trees were over the mark on 11-21, 46 of them over 60 KB. At `d50c06b8e3`, 2 are over it, temper's `domain/worker/checkouts.md` at 54,743 bytes and smith's `testing.md` at 51,305, and none is over 56 KB. No file came near the limit, so no goal had to cut one.
- **Goals write as before:** the 60 landings that touched design documents before the rules made 23 documents and added 452 KB, and the 60 after the first pass made 20 and added 466 KB. Documents climb back to the mark within days, as 4.3 expects: temper's documents over 40 KB went from 19 to 21, and `checkouts.md` grew 5.6 KB in 12 hours.
- **No draft went:** a draft's status counted as done only when every label read none. But a **Design departs** entry never closes, since goals leave a draft's text as it is. Since the first pass the roll-up has listed no draft as done, and none was archived, while 51 of temper's 122 drafts outside `draft/archive/` had only departures left: 751 KB of `draft/`'s 2.4 MB.
- **Openings that hold the document:** 34 documents' text before their first numbered section is over 8 KB (19 of them temper's, 11 jig's, 3 skein's and 1 hearth's), 20 of them over 16 KB. `checkouts.md` is a 43.7 KB opening list and two sections, and the three goals that grew it since the pass each added an item to that list. `move.py` moves numbered sections, so a split can't take such an opening apart as it stands.

### 2.6 What the third pass left

Read at `2f50ccd03d` (2026-11-28 12:46Z), against the third pass's landing, `c7f89289f6` (2026-11-24 23:28Z), 190 landings apart.

- **Over the mark again:** six files, none on 11-24: temper's `draft/jig-domain-api.md` at 62,359 bytes, `client/journey-writer.md` at 60,764, `testing/workloads.md` at 58,247, `domain/forge/branches.md` at 50,536 and `protocol/configuration/stock.md` at 49,658, and skein's `testing/browser.md` at 53,541. Documents over 40 KB went from 39 to 45.
- **Where the text went:** the five trees' design documents, drafts and status files aside, grew 616 KB, to 13.8 MB, and gained 18 files. 467 KB went into documents that were already there, and 23 new documents took 255 KB. `journey-writer.md` went from 18,720 to 60,764 bytes in eight landings over two and a half days, as look requests and their render check were designed into its section 6, now 27 KB. jig's `domain/engine/vocabulary.md` went from 9,411 to 42,317 bytes, its section 2, the engine, at 30 KB. The text read is current design, not a log.
- **Status files held:** 811 KB to 820 KB.
- **Lines left behind:** 351 "where it went" lines in 92 files. No pass was told to remove the earlier passes' lines, as 4.2 asks.
- **What the passes don't reach:** a new concept is designed into the nearest document's section, not a document of its own (`DESIGN.md`, 5). Nothing at the writer acts before the limit. A hot document goes from the mark to the limit in a few landings, faster than passes decided at the user's check-ins.

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
  - **Done:** every label in the file reads none, or the file's only text after its opening line is "No gap." A draft's status is also done when its **Design lags** and **Open** read none, whatever its **Design departs** hold: they record what the design does instead, and nothing closes them (2.5).
  - **Unread:** a file in neither form, with prose and no label for instance. The roll-up lists it as unread, never as done.
- **What it prints, for each tree:**
  - The open entries counted by label, with how many **Not yet** entries are in flight. A status file under `draft/archive/` counts none: its draft has gone.
  - A line for each status file with an open entry, giving its date and counts.
  - What's due: the files over the mark (4.1), largest first; the documents whose opening is past 8 KB (4.1), largest opening first; the drafts that are done, outside `draft/archive/` (section 5); and the unread status files. A page generated from a table, whose opening line says so ("Generated from"), isn't listed among the openings: its text is its renderer's, and no goal adds to it by hand (`design-upkeep.status.md`, Design departs, 4.1 and 4.2).
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
- **The opening:** a document's or a draft's text before its first numbered section, all of it where there is none. Past 8 KB, it is due to be numbered, and the roll-up lists it among what's due (3.2). Status, notes and execution files aren't read for it.
- **A status file over the mark** follows its document: its entries go with their sections when the document splits.
- **No refusal at the mark:** the limit stays the gate's only refusal. Whatever line the gate refuses at, goals pass it by the smallest cut (2.2), so a refusal at the mark would only bring the same cut sooner. What's missing is a split made before a goal runs into the limit.

### 4.2 A split

- **Along the document's concepts** (`DESIGN.md`, 5), with `tools/citations/move.py` and its map. Each piece comes out under 40 KB. A concept that can't be brought under 40 KB stays under the mark. One that can't be brought under the mark stays whole, and the change says why.
- **Text moves unchanged:** a split only moves text. No text is cut just to pass the limit. Text that is wrong, out of date, or not the design's to say (`design-economy.md`, 4.2) is removed for that reason, in a change of its own, so that each change is reviewed as what it is.
- **What stays keeps its numbers:** the map names no `renumber` for the document left behind. Where a section left, one line names where it went, so a spec or a branch that still cites the old place can find it. The next pass removes those lines, under their headings, and points what still cites them at where the text went. Meanwhile the checker refuses a citation to a section that holds only such a line, saying where the text went, so none is written.
- **Exceptions follow:** a literal's exception in `tools/citations/allow.txt` moves with it. `move.py` rewrites the exception's file to the literal's new place, as it rewrites citations.
- **A long opening first:** before a document is split, an opening past 8 KB goes into numbered sections under headings of its own, its text unchanged, and a `renumber` entry of the map numbers them with the sections after them, so that every citation follows. Its pieces then move as any section does. A document under the mark with such an opening is numbered and not split.
- **At the limit too:** a goal that meets the limit splits the same way, and the checker's refusal says so: "over the 65,536-byte limit: split it along its concepts, each piece under 40 KB (DESIGN.md, 5); don't cut text to fit".
- **The shared notes file:** this one file holds every draft's reasons. It splits into a notes file beside each draft, `<draft>.notes.md`, and `DESIGN.md`, 4, then puts a draft's reasons there and nowhere else. No shared notes file is left to fill up. What the first pass left of it, its headings and their lines, goes with the fourth pass's cleanup of those lines.

### 4.3 The split pass

- **A goal** that splits temper's files over the mark as 4.2 says, and removes the drafts section 5 lists.
- **One area at a time:** `move.py` rewrites citations across the whole repository. So the goal's changes run one after another, each splitting one area's files: for example the domain, the protocol, the clients, and then the rest. Each change takes its batch from the files over the mark at the commit it starts from, and writes its maps alone. It's accepted when every file of its batch is under the mark.
- **The first pass:** the 45 documents and drafts over the mark at `b2fdead79c` (27 of them over 56 KB), the shared notes file among them; `client/pages/README.status.md` with its document; and the 13 drafts of 2.3.
- **Later passes, by hand for now:** when the user checks in, the drafter brings the roll-up's due list for each tree, a document over 56 KB above all, and they decide together whether to run a pass. A pass they decide on starts as a goal on the user's word. Once a few passes have given experience, the passes are automated (7.1).
- **Goals in flight:** a split moves text that a goal in flight may cite or edit. A citation to a moved section finds its line (4.2). A goal whose branch edited a moved section meets a conflict at its merge. The goal's designer or coordinator resolves it with the map, since a repair may not edit design documents.
- **The third pass,** 2026-11-24 (2.5): temper's `checkouts.md`, its opening numbered first, then smith's `testing.md`, a goal each. Then a goal that brings the roll-up to 3.2's and 4.1's new readings and acts on what they list: temper's drafts with only departures left archived, and every tree's long openings numbered, one tree a change.
- **The fourth pass,** 2026-11-28 (2.6): temper's files over the mark, but `draft/jig-domain-api.md`, a draft its drafter is still writing; temper's finished drafts and unread status files; skein's `testing/browser.md`; and in temper and skein, the earlier passes' lines removed (4.2), the first pass's shared notes file with them. A goal each, skein's after temper's. Then a goal for the tools: the roll-up's generated pages (3.2), the exceptions that follow and the refused citation (4.2), with the other trees' lines removed.
- **Other trees:** jig's, skein's, smith's and hearth's files over the mark get passes of their own, decided as temper's are. A tree's pass is one goal, and the trees' goals run one after another, since each rewrites citations across the whole repository (7.2).

## 5. Finished drafts go

- **Due:** a draft in `draft/`, outside `draft/archive/`, whose status is done (3.2). The roll-up lists it.
- **A milestone left:** the pass reads each listed draft's execution file. A draft whose execution file still has a milestone to run stays, and the pass's change names it: its status is missing a **Design lags** entry, and only a reader can tell.
- **How it goes:** the split pass moves it to `draft/archive/` with `move.py`, or deletes it where nothing cites it (`DESIGN.md`, 4). Its execution file and its notes file go with it. A draft with **Design departs** entries is archived, not deleted, and they go with it: they say why the design reads otherwise.
- **Unread first:** a draft whose status is unread, as `cli-c8-tools`' is today, is brought to the form before it counts as done.

## 6. Order

- **The words before the check:** goals stop being asked to write the page before the gate refuses a page other than the fixed one.
- **On dev.fabrig.ai,** fabrig's goal charter reaches the goals once its charter set is applied there and the live goals are moved to the new revision. A run already going keeps the revision it started with until it ends.
- **The split pass after the roll-up,** since the pass works from the roll-up's list of what's due.
- **The new readings after the third pass** (4.3): archiving drafts and numbering openings rewrite citations across the repository, as the passes' splits do, so they run after them.
- **The tools after the fourth pass:** its documents are nearest the limit, so they go first. The tools' goal removes the other trees' lines after the passes, and its refused citation then holds what they left.

## 7. Open questions

### 7.1 How often split passes run

Decided by the user, 2026-11-21: by hand at first, with no cadence and no standing approval. At the user's check-ins the drafter reminds them of the roll-up's due list, and they decide whether to run a pass (4.3). Once a few passes have given experience, the passes are automated, and their cadence and approval are decided then. Asked again, 2026-11-28, the user would rather know first what would keep documents in shape at the root, without passes (2.6).

### 7.2 The other trees

Decided by the user, 2026-11-23: jig's, skein's, smith's and hearth's files over the mark get split passes of their own. The drafter proposes them with temper's, from each tree's due list at the user's check-ins (7.1), and a pass the user decides on starts as a goal on their word. Their first passes follow temper's second, one goal per tree (4.3).
