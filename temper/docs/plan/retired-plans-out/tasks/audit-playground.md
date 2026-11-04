+++
id = "audit-playground"
part = "audit"
kind = "docs"
needs = []
files = ["temper/docs/plan/retired-plans-out/notes/kept-playground.md"]
effort = "high"
minutes = 95
+++

# The kept note of the playground, wasm32, the live smith and temper stories, and the tooling plans

## Goal

The retired plans leave main once what only they record stands in the design (the request's
Task 1). This task is that pass's reading for twenty of the 49 directories: for each, read what the
request names, check every item against the design, and write those the design lacks as
findings in `temper/docs/plan/retired-plans-out/notes/kept-playground.md`, in the form of
`notes/contracts.md`, K1. `keep-decisions` writes the findings into the design afterwards;
this task changes nothing else.

Acceptance:

- `notes/kept-playground.md` exists in K1's form: a `##` section for each directory below, in this
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

- **`temper/docs/plan/retired-plans-out/notes/kept-playground.md`** (new), whole. Nothing else: the
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
   skein/docs/plan/playground-skein
   temper/docs/plan/playground-temper
   temper/docs/plan/playground-decisions
   jig/docs/plan/wasm32-jig
   skein/docs/plan/wasm32-skein
   temper/docs/plan/wasm32-list
   temper/docs/plan/wasm32-more
   temper/docs/plan/wasm32-rustls
   temper/docs/plan/wasm32-temper
   temper/docs/plan/wasm32-temper-2
   temper/docs/plan/wasm32-temper-3
   smith/docs/plan/live-smith-manual
   smith/docs/plan/live-smith-reds
   smith/docs/plan/live-smith-reds-2
   temper/docs/plan/live-temper
   temper/docs/plan/live-temper-stories
   temper/docs/plan/live-temper-stories-2
   temper/docs/plan/openai-manual
   temper/docs/plan/citation-burndown
   temper/docs/plan/collector-starved
   ```

   In each, read what K2 names: `plan.md` whole; `plan.toml`'s `[[question]]` tables
   (`text`, `answer`); every `tasks/*.md`'s `## Answers`, from that heading to the end of the
   file (`git grep -l '^## Answers' -- <dir>/tasks` lists them); `notes/contracts.md`;
   `notes/common.md`. Also:
   - `temper/docs/plan/citation-burndown/notes/findings.md`: only its decisions (what a
     citation is, what the checker leaves out); its findings list is Not design.
   - `skein/docs/plan/playground-skein/notes/contracts.md` is 510 lines: `## Owns and may change`,
     "Long material", applies.
   - The `live-temper` question `pair-timeout`'s answer sets a nextest profile's timeout, a
     budget (`AGENTS.md`, 7: the user's); Held if `temper/docs/design/domain/testing.md` gives it.
   Never `inputs/` but where this list names it, nor `notes/old/` or patches: the code they
   carried is on main or was dropped.

2. **Check and write.** For each item, K2's search, then its decision; a Kept item becomes a
   finding whose **Text** is written in the target section's words, after reading that
   section. The design homes this family most likely meets:

   - skein: `playground.md`, `browser.md`, `ui-web.md`, `tls.md`, `testing.md`, `lib.md`,
     `shell.md`, `world.md` (under `skein/docs/design/`).
   - temper: `playground.md`, `client/browser.md`, `domain/testing.md` (the live tier and its
     stories), `release.md`, `protocol/llm.md`, and their status files (under
     `temper/docs/design/`).
   - jig: `protocol/browser.md`, `protocol/llm.md`; smith: `testing.md`, `benchmarks.md`,
     `protocol/llm.md`, `shell.md` (smith's design is read by tests: the writer runs them).
   - `tools/tiers/design.md` (the CPU summary, a starved run), `tools/citations/README.md`
     (Held only: `tools/citations/` is not a target), `.config/gate/README.md` (Held only).

   Any other document of any tree may hold an item; search, don't assume.

3. **The check before committing.** From the repository root, this prints nothing (it reads
   every finding's **Text**, to the next heading):

   ```sh
   awk '/^- \*\*Text:\*\*/{t=1} /^(- \*\*(Kind|Source|Checked|Target):\*\*|#)/{t=0} t' \
     temper/docs/plan/retired-plans-out/notes/kept-playground.md \
     | grep -niE 'docs/plan|reference-m[0-9]|configuring-p[0-9]|wasm32-(jig|skein|list|more|rustls|temper)|live-smith-|live-temper-stories|playground-(skein|temper|decisions)|\bfab\b|heavy (cargo|env|--)|agents (repo|repository)|fabrig/qa|agents/loop|hel8|director|implementer|answer call|the cut\b|\blead\b'
   ```

   A hit that is a product word (a forge's "lead", a temper "implementer" role) stays, and the
   report names it.

4. Commit (`retired-plans-out: kept-playground, <m> findings over <n> directories`) and push.

## Report

`Done:` with the counts. Then, per directory, one line: kept, held, superseded, not design;
the findings you were least sure of (by number, one line each: why); any item you could not
decide; your minutes reading and writing.
