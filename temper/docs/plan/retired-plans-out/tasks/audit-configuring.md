+++
id = "audit-configuring"
part = "audit"
kind = "docs"
needs = []
files = ["temper/docs/plan/retired-plans-out/notes/kept-configuring.md"]
effort = "high"
minutes = 90
+++

# The kept note of configuring, the first run's projects, connecting forges, the welcome's chats and public reading

## Goal

The retired plans leave main once what only they record stands in the design (the request's
Task 1). This task is that pass's reading for twelve of the 49 directories: for each, read what the
request names, check every item against the design, and write those the design lacks as
findings in `temper/docs/plan/retired-plans-out/notes/kept-configuring.md`, in the form of
`notes/contracts.md`, K1. `keep-decisions` writes the findings into the design afterwards;
this task changes nothing else.

Acceptance:

- `notes/kept-configuring.md` exists in K1's form: a `##` section for each directory below, in this
  order, with its five summary lines (Read, Kept, Held, Superseded, Not design), and each
  finding with its five fields (Kind, Source, Checked, Target, Text).
- Every item K2 names in each directory was decided (Held, Superseded, Not design or Kept), and
  the summary lines account for them.
- No finding's **Text** names a plan, a role of the lab or a name of the lab (`notes/common.md`,
  "What text added outside `docs/plan/` may say"): the check of `## Steps`, 3 prints nothing.
- No other file changes. One commit (more if you push work in progress), pushed.

## Inputs

- `temper/docs/plan/retired-plans-out/notes/common.md`: the standing rules.
- `temper/docs/plan/retired-plans-out/notes/contracts.md`, K1 and K2: the note's form and how
  each item is decided. K3 says what the writer does with it.
- `temper/docs/plan/retired-plans-out/inputs/request.md`, "Task 1": the request.
- `DESIGN.md`, whole: where a decision, a gap and an open choice belong.

## Owns and may change

- **`temper/docs/plan/retired-plans-out/notes/kept-configuring.md`** (new), whole. Nothing else: the
  directories you read, the design and every other file stay as they are.
- **Departures from the implementer's text:** no fmt, clippy, old-forms or test run (the branch
  changes one Markdown file inside `docs/plan/`, which neither the citations checker nor the
  outward check reads); and `docs/design/` is read, never written.
- **A directory absent at your base:** its section says "absent at `<hash>`" alone.
- **An item another directory decided again** (a later plan revisiting it): judge each where you
  find it; the writer merges duplicates across notes.
- **An answer that only widens or narrows a task's files or scope:** Not design.
- **Long material:** `notes/contracts.md` and `notes/common.md` are mostly code shapes and run
  rules (Not design); read each item's heading and first lines, and the whole item where it
  states a rule of behaviour, format, bound or ownership.

## Steps

1. **The directories, in this order** (each from the repository root):

   ```
   temper/docs/plan/configuring-p1
   temper/docs/plan/configuring-p2
   temper/docs/plan/configuring-p2-build
   temper/docs/plan/configuring-p3
   temper/docs/plan/first-run-projects
   temper/docs/plan/forgejo-connect
   temper/docs/plan/open-default
   temper/docs/plan/look-welcome-chats
   temper/docs/plan/public-reading
   temper/docs/plan/me-retention
   temper/docs/plan/repo-links
   temper/docs/plan/fab3-fixes
   ```

   In each, read what K2 names: `plan.md` whole; `plan.toml`'s `[[question]]` tables
   (`text`, `answer`); every `tasks/*.md`'s `## Answers`, from that heading to the end of the
   file (`git grep -l '^## Answers' -- <dir>/tasks` lists them); `notes/contracts.md`;
   `notes/common.md`. Also:
   - `temper/docs/plan/configuring-p2/notes/next-cut.md` and
     `temper/docs/plan/public-reading/notes/next-cut.md`, whole: decisions handed to a later cut.
   - `temper/docs/plan/first-run-projects/inputs/request.md` and
     `temper/docs/plan/public-reading/inputs/request.md`: only the decisions taken while the
     user was away (the drafts' status files cite them: `draft/first-run-projects.status.md`,
     9; `draft/first-release.status.md`, 5.3), each an open choice unless the design has since
     settled it.
   Never `inputs/` but where this list names it, nor `notes/old/` or patches: the code they
   carried is on main or was dropped.

2. **Check and write.** For each item, K2's search, then its decision; a Kept item becomes a
   finding whose **Text** is written in the target section's words, after reading that
   section. The design homes this family most likely meets:

   - temper: `draft/configuring.md`, `protocol/configuration.md`, `protocol/setup.md`,
     `protocol/credentials.md`, `protocol/sign-in.md`, `client/README.md`, `client/forge.md`,
     `client/tools.md`, `domain/agent.md`, `draft/first-run-projects.md`,
     `draft/first-release.md`, `draft/v0.1.0.md`, and their status files (all under
     `temper/docs/design/`).
   - jig: `protocol/configuration.md`, `domain/engine.md` (17), `domain/people.md` (13, public
     reading), `protocol/client.md`, `client/domain/*.md` (under `jig/docs/design/`).
   - skein: `forge-setup.md`, `forge-connection.md` (connecting a forge).

   Any other document of any tree may hold an item; search, don't assume.

3. **The check before committing.** From the repository root, this prints nothing (it reads
   every finding's **Text**, to the next heading):

   ```sh
   awk '/^- \*\*Text:\*\*/{t=1} /^(- \*\*(Kind|Source|Checked|Target):\*\*|#)/{t=0} t' \
     temper/docs/plan/retired-plans-out/notes/kept-configuring.md \
     | grep -niE 'docs/plan|reference-m[0-9]|configuring-p[0-9]|wasm32-(jig|skein|list|more|rustls|temper)|live-smith-|live-temper-stories|playground-(skein|temper|decisions)|\bfab\b|heavy (cargo|env|--)|agents (repo|repository)|fabrig/qa|agents/loop|hel8|director|implementer|answer call|the cut\b|\blead\b'
   ```

   A hit that is a product word (a forge's "lead", a temper "implementer" role) stays, and the
   report names it.

4. Commit (`retired-plans-out: kept-configuring, <m> findings over <n> directories`) and push.

## Report

`Done:` with the counts. Then, per directory, one line: kept, held, superseded, not design;
the findings you were least sure of (by number, one line each: why); any item you could not
decide; your minutes reading and writing.
