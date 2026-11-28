# stock-charters: three stock roles for a goal, and a deployment's overlay

Draft, 2026-11-12, from a session with the user that day. A goal runs
three roles: the designer, the coordinator and the implementer. temper's
stock says them in an older edition, with a foreman beside them; the
reference profile carries five charters of its own; and dev.fabrig.ai,
the deployment that develops this repository, runs a third set, kept
outside it. This draft makes one set: the three roles written as plain
Markdown, a file each and a common one, built into temper as its stock;
and a deployment's own lines added over them as an overlay that leaves
the stock's words in place. The user's words: "a set of stock charters
people can reuse (and that would be shipped in temper builtin) plus an
overlay with fabrig-specific things", "fabrig as dev.fabrig.ai", and
"those 3 prompts, one per file", with a common one beside them.

## 1. In one page

- **Three roles, five files** (section 2): `common`, `goal`, `designer`,
  `coordinator` and `implementer`, assembled into two charters: the
  goal's agent (charter 5), whose first run is the designer and whose
  later wakes are the coordinator, and the implementer (charter 2). jig
  sends each run the shared instructions and only the parts for why it
  runs, so no agent is told to pick its part.
- **The stock is these files** (section 3), under
  `temper/docs/charters/stock/`, compiled in, in place of today's
  coordinator and coding texts. The foreman goes, its charter and its
  question route; the reviewer stays as it is. The files are code to
  every gate.
- **An overlay adds a deployment's lines** (section 4): a change of a
  charter that names the fields it replaces and adds text after the
  stock's instructions and parts, kept over the stock, so a release's
  new stock reaches the deployment with the overlay in place. A
  settings file gives an overlay's text as directories of the same five
  files.
- **fabrig's overlay** (section 5), for dev.fabrig.ai, is
  `temper/docs/charters/fabrig/`: fabrig's rules for agents on its code,
  and its settings for the two charters. The deployment's machines stay
  outside this repository, in an overlay of its own applied with
  fabrig's. The charters kept outside this repository retire.
- **The reference profile** (section 6) drops its five charter editions
  for overlays that carry its fields alone.
- **The texts** are `archive/stock-charters-texts.md`, section 1, and `stock-charters.md`, section 10, moved verbatim by the first
  milestones (`stock-charters.execution.md`).

## 2. The roles and their files

| Role | Who | Runs |
|---|---|---|
| designer | the goal's agent, its first run | once: one session, until it parks |
| coordinator | the goal's agent, every later wake | a new session for each wake, with no memory, for one reason |
| implementer | each change's coding agent | one spec, until it finishes |

A set of charter files is a directory of Markdown files, each named
`<file>.md`, all five present in the stock and each optional in an
overlay:

| File | Read by | Becomes |
|---|---|---|
| `common` | every run of the three roles | the start of both charters' instructions |
| `goal` | the designer and the coordinator | charter 5's instructions, after `common` |
| `designer` | the designer | charter 5's part on `first` |
| `coordinator` | the coordinator, and the designer for questions and results | charter 5's other parts, one per section |
| `implementer` | the implementer | charter 2's instructions, after `common` |

- **The assembly:** charter 5's instructions are `common` then `goal`;
  its parts are `designer`, on `first`, then each section of
  `coordinator` in order. A section starts at a line `## part, on: `
  followed by its reasons, comma-separated (jig's
  `domain/tasks/messages.md`, 6), and runs to the next such line or the
  file's end, other headings included. Charter 2's instructions are
  `common` then `implementer`, with no parts. Texts join with one blank
  line, every byte as written but the blank lines that open and close a
  file or a section.
- **What each run reads** (jig's `domain/tasks/messages.md`, 6): the
  shared instructions, then each part whose reasons meet its own, in
  order, each once. So the designer reads `common`, `goal`, `designer`,
  and the coordinator's parts on questions and on results, which name
  `first` too, since it is driven while it designs. A coordinator woken
  by a question reads `common`, `goal` and the question part alone. A
  part never leans on another part's text, which its run may not have:
  what the designer and the coordinator both need is in `goal`.
- **Fresh runs:** charter 5's runs start fresh, as an ordinary task's
  charter in parts must: a task that doesn't stand, under a charter
  whose runs resume, is sent its instructions alone.
- **Every wake has a part:** each kind that chooses parts, of those
  charter 5's wake policy wakes it for, is among some part's reasons.
  The always-offered kinds choose none; `goal` says what to do with
  them.
- **Nothing dropped:** a Markdown file in a set's directory that is none
  of the five is refused by name, and so is text in `coordinator` before
  its first section, or a section whose reasons are empty, repeated or
  unknown: at the build for the stock, at apply for an overlay. Other
  files in the directory, a settings file among them, are no part of the
  set.

## 3. The stock

- **Built in:** the stock module compiles `temper/docs/charters/stock/`
  as charters 5 and 2's instructions and parts, in place of today's
  coordinator and coding texts (`../protocol/configuration/stock.md`).
  Their other fields stay the stock's: the stock's order of models, the
  hosts, the runs, the tool families. The texts name no model, no
  program and no project's paths.
- **The foreman goes:** charter 6 and its text, the forge's charter 3
  (kind 3, number 3), and `foreman` as a question route (kind 6;
  `../domain/forge/change/questions.md`, 1): a change's questions go
  `up` to the goal's agent, or nowhere. The implementer now checks its
  spec itself before it asks, which is most of what a foreman settled.
  Its whole reach goes: the forge's constant for it and the authority
  its delegate took, the relay's foreman states and their stored form,
  the `change` tool's schema and decoder, the settings forms and pages,
  How work runs' roles, Ask temper's options, and the stock rules'
  delegation to it. A stored `foreman` is refused, not read, naming its
  setting, with no compatibility before v0.1.0. The reference's foreman
  goes with it: its charter 6, and its charter 5's `foreman` route, now
  `up`. Where `goal-as-designer.md` and `configuring.md` keep a foreman,
  this draft retires it; `reference-code-production.notes.md` (N1) keeps the reasons it had.
- **The reviewer stays:** charter 4, and the stock's rule 1 with its
  agent's review at every branch, as they are.
- **Kind 9's words:** charter 5's become "designs a goal's draft, cuts
  it into changes, and answers what they ask"; charter 6's row goes.
- **Room for it:** charter 5's files come to about 24 KB, which with its
  models passes a stock box's `root.journal.run_bytes`, 24,576, where
  today's stock needs 13,123 bytes (`../protocol/configuration/stock.md`'s
  floors). The stock's bound rises to 65,536, the reference room's, so
  that a fresh box admits the stock and an overlay of a few KB; the
  milestone measures the new floor, and the room smith gives a received
  charter (`../../examples/reference/README.md`, A call's room), rather
  than take one bound for both.
- **Code to every gate:** the stock's files are compiled in and tests
  read them, so a change to them lands through a goal like code, never
  by hand as documentation, Markdown though they are (the user,
  2026-11-12). An overlay's files are documentation (section 5).

## 4. An overlay

- **What it is:** a change of a charter that has a default, kept over
  that default rather than replacing it (jig's
  `domain/engine/settings.md`, 1). It names the fields it replaces
  (models, tool families, hosts, waiting, resume, a run's bounds, spend,
  documents), and holds text added after the default's: instructions,
  after the default's instructions with one blank line, and parts, after
  the default's parts, selected by their reasons as any part is. The
  current charter is the default with its overlay over it.
- **Kept over the stock:** each start gives jig the binary's defaults
  and lays each overlay over its own again, so a release's new stock
  reaches the deployment with the overlay in place. A reset erases the
  overlay; a set replaces it whole, with a charter or another overlay.
- **Checked whole:** the charter it makes is checked as any charter is,
  its stored policy and parts within `root.journal.run_bytes` and its
  parts within `tasks.charter_parts`, at the change and at each start.
  A start whose new default leaves an overlaid charter past a bound is
  refused, naming the setting, as a default past its bounds is. temper's
  judgment of a received charter's room runs on the charter the overlay
  makes too, the parts a run can select included, at apply, at a
  proposal and after a restore, as it runs on a whole charter today.
- **Revisions:** a run reads its revision's charter: the overlay over the
  default that revision held (jig's `domain/engine/settings.md`, 3). A
  task already running keeps its words when a release changes the
  default; new work reads the new one.
- **Every path a change takes:** an overlay is proposed, decided,
  withdrawn when stale, kept in the history and restored as any change
  of settings is (jig's `domain/engine/settings-proposals.md`, 1, and `domain/engine/settings.md`, 10).
- **Read:** `settings show charter N` prints the charter as its runs read
  it and marks what the overlay replaced and added; with `--json`, the
  overlay whole, its fields and its added text as literal text, naming
  no directory, so that it applies back anywhere.
- **In a settings file** (`../client/ctl/tools-acts.md`, 1): an overlay item
  names the charter, the fields it replaces, and `charters`, directories
  of a set, whose files give its text by `stock-charters.md`, section 2's assembly. A
  directory is relative to the file that names it; a file read from
  standard input names its directories absolutely or is refused. One
  directory serves both charters: charter 5 takes `common`, `goal`,
  `designer` and `coordinator`; charter 2 takes `common` and
  `implementer`.
- **Stacked at apply:** files applied in one call join their overlays of
  one charter, the directories' text in file order and a later file's
  fields over an earlier's. So a deployment adds lines of its own to a
  public set without copying it. A set, a reset or a removal of a
  charter beside an overlay of it in the same call is refused.

## 5. fabrig's overlay, on dev.fabrig.ai

- **The set:** `temper/docs/charters/fabrig/`, section 10's files, and
  beside them `settings.json`: the overlays of charters 5 and 2, naming
  that directory, with the fields below, and template 4's models, Opus
  5.5 at xhigh for the designer and at high for later wakes, with GPT-6.1
  Sol at high for sub-agents.

  | Field | Charter 5, the goal's agent | Charter 2, the implementer |
  |---|---|---|
  | model | Opus 5.5 at xhigh, GPT-6.1 Sol at high its alternative | GPT-6.1 Sol at high |
  | shell, agents | both | a shell, no agents |
  | runs | fresh, waiting 0 | resuming, waiting 10 s |
  | a run | 800 turns and 24 hours, a reserve of 1 turn and 300 s | the same |
  | spend | 1,000,000,000 | the same |

- **Documentation, not code:** no test reads the set, and it lands by
  hand as documentation. `settings apply`'s check is its gate: an
  overlay past a bound is refused, and nothing changes.
- **The deployment's own lines:** what dev.fabrig.ai's machines and
  programs need (the build admission behind `cargo`, where checkouts
  live, its programs' sandboxes: `dogfood-hacks.md`, 2.2 and 2.4) go in
  an overlay set kept with the deployment, outside this repository,
  applied with fabrig's in one call (section 4). Neither names the
  other.
- **Applied on landing:** the deployment applies both sets after each
  landing that touches `temper/docs/charters/fabrig/`. A new release
  needs no apply: the overlays stand over its stock.
- **The switch is one decision:** each project's policy first loses
  charters 6 and 7, then one `settings apply` sets both sets' overlays
  over charters 5 and 2, drops charters 6 and 7 from the rules and policy
  1, and resets charters 6 and 7. A refusal leaves everything as it was;
  the deployment never runs the bare stock without its own lines. Its
  other stored settings (models, landing rules, policies' other members)
  stay unless the set replaces them.
- **What retires:** the deployment's charters kept outside this
  repository and the generator that made them, once the switch holds,
  and `dogfood-hacks.md`, 2.10, whose entry goes with them.
- **Before the stock changes there,** nothing stored on dev.fabrig.ai
  names the foreman's route (kind 3's number 3, kind 6's `foreman`); its
  stored charter 6 stays until the switch.

## 6. The reference profile

The reference's settings (`../../examples/reference/README.md`) drop
their five charter editions (`../protocol/configuration/reference.md`):
charters 5 and 2 become overlays that carry the reference's fields alone,
its models, documents, budgets and shell, and no text, so that the
reference runs the stock's words; charters 6, 7 and 8 go. Its template
4, landing rules, limits and room stay.

## 7. What carries it

- `../protocol/configuration/stock.md`: charters 5 and 2 from the stock's
  files, charter 6 gone, kinds 3, 6 and 9.
- `../protocol/configuration/settings-kinds.md`, 1 and
  `../domain/forge/change/questions.md`, 1: no foreman.
- jig's `domain/engine/settings.md`, 1 and 2, and `domain/engine/settings-store.md`, 1: an overlay as a change
  kept over its default.
- `../client/ctl/tools-acts.md`, 1: an overlay item, its directories, and
  files stacked in one call; `settings show`'s marks.
- `../domain/agent/charter.md`, 1: the stock's instructions from a set of
  files.
- `../protocol/configuration/reference.md` and
  `../../examples/reference/README.md`: the reference's overlays.
- `../../help/charter.md` and the settings reference pages.

## 8. Acceptance

Focused and fuzzy suites alone, each failing first:

- the assembly: a set's files made into charters 5 and 2 as section 2
  says; a heading inside a section kept in it; the blank lines that open
  and close a file dropped; an unknown Markdown file, text before
  `coordinator`'s first section, and an empty, repeated or unknown
  reason each refused by name; the section parser fuzzed;
- the stock's charter 5: a first run reads `common`, `goal`, `designer`
  and the parts on questions and on results; a question's wake reads the
  question part alone; a red's news the red part alone; a wake for a
  question and a result reads both parts, once each, in order; an
  `answer` reaches a run with no part of its own; every kind that
  chooses parts, of those its wake policy wakes it for, among some
  part's reasons;
- the stock's texts name no model: no `claude-` or `gpt-` in them;
- a fresh box's start admitting the stock's charters at init's
  catalogue, under the stock's floors;
- the foreman gone: kind 3's number 3 not offered; a single and a batch
  `change` naming `"questions": "foreman"` refused by name; kind 6
  offering none and up alone; Settings, How work runs' roles and Ask
  temper's options naming no foreman; a stored route refused naming its
  setting, at a start and at a restore; the reference's charter 5
  making its changes with `up`;
- an overlay: the fields it names replacing the default's and the rest
  following it; its text after the default's instructions and its parts
  after the default's parts; checked whole at the change; refused at
  apply and at a proposal where the charter it makes leaves a received
  run too little room; kept across a restart; a new default at a start
  reaching the overlaid charter, the overlay kept, while a task already
  running keeps its words; a start whose default leaves it past
  `root.journal.run_bytes` refused, naming the charter; proposed,
  accepted, refused, withdrawn when stale and restored; a reset
  returning it to the default;
- a settings file: an overlay item's directories assembled in order;
  fabrig's whole set, its `settings.json` included, applied; two files'
  overlays of one charter joined, text in order and the later's fields
  winning; a set beside an overlay of one charter refused; relative
  directories from standard input refused; `settings show charter N
  --json` naming no directory, applied back, and still taking a new
  default after;
- the reference: its settings applied over the stock at the reference's
  room, charters 5 and 2 within `root.journal.run_bytes`, tests reading
  the charter a run receives rather than an item's text.

## 10. fabrig's texts

Each file whole, as `temper/docs/charters/fabrig/` holds it, added after
the stock's by section 4.

### 10.1 The common file

```
# fabrig

These lines add fabrig's rules to the ones above.

- **Its rules for agents:** the root `AGENTS.md` and each tree's `AGENTS.md`; whoever writes or reviews design reads `DESIGN.md` too.
- **Before v0.1.0** a stored format, protocol or codec changes in place, with no version bump or migration: an additive change that every existing row still decodes isn't hard to reverse, and the change's report says so.
- **Suites:** a plan's runs use the focused and fuzzy suites alone: never `tools/tiers/run`, a `<tier>-<tree>` profile or `scripts/browser.sh`, even where a spec says to. A story in a slow tier is written and compiled (`--no-run`), not run, and the report says so; after-landing runs it, and a red comes back as a repair, which may run the flagged test, by name, alone. No walk, render check or throwaway driver: a plan is complete when it lands. `temper-client-journeys again` is generation, not a walk: a branch that leaves a journey stale runs it, and on `a key mark caught a moving page` retries it at most three times.
- **Budgets and timeouts are the user's,** never raised by a role. Over budget is red (`skein/docs/foundation/testing-strategy.md`, 8.1), repaired by asking, in order: does the test belong to its tier (moving it to escape a budget is raising it, forbidden); can it be made faster; if neither, the measurements go up as a question. The wall hard limit is a backstop: a run past it is starved, a warning, never red on that account; the tests it didn't run are unobserved, never passed, and the report names them.
- **Known reds,** failing on g and owned by no live change, are listed in g's `.temper/known-reds.md` (`- test <binary_id> <name>`, `- lint <package> <code> <file>`), at most 64, which delivery's checks pass.
```

### 10.2 The goal file

```
- **A coding change** also carries `"producer": 2` and `"llm": {"producer": {"main": {"model": "gpt-6.1-sol", "effort": "high"}, "sub_agents": []}}`.
- **Editing:** edit a file only after reading it with Read; a shell `cat` doesn't count.
- **Status** follows `DESIGN.md`, section 2. A draft's status says **Design lags**, **Design departs** and **Open**; write it, and `STATUS.md`'s summaries, as they will stand once the goal lands.
- **Known reds:** a goal note `known reds` holds each one's since and why, under the same keys as g's `.temper/known-reds.md`. Drop one once fixed. Specs list those in their packages.
- **A spec** gives each requirement's tier, focused or fuzzy; a slow tier's story only compiles. It keeps a branch in its tree, but for a required field: the producer's branch carries the other tree's lines that name it, each edited only in the field's name, type path or value. It repeats the standing rules: fakes, fixtures, what not to touch. It stays within about 60 KB, leaving room in the 63,740-byte payload.
```

### 10.3 The designer file

```
- **Surveys** run at gpt-6.1-sol, high. A reverse-dependency survey takes the tests that read a site from `.config/gate/read-by-tests.txt`'s globs.
- **Your design's rules:** `DESIGN.md` holds for every design file you write: read it first.
```

### 10.4 The coordinator file

```
## part, on: first, question, question_answered, question_ended, question_owed

- **A red the asking change inherited** that no live change owns becomes a known red: add it to the goal's note and to g's `.temper/known-reds.md`, deliver, and answer with its entry line for the implementer to add.
```

### 10.5 The implementer file

```
# fabrig's checks

- **What you run,** where the lines above say: `cargo clippy -p PKG --all-targets -- -D warnings` on what you wrote, while errors or lints last; a new or failing test alone, `cargo nextest run -p PKG --test BIN -E 'test(=NAME)'` or `--lib`, adding `--profile fuzzy` only for a fuzzy test; a repository tool's own Python tests, for a tool you changed, run directly with `python3 -I -B`. These lines win over `AGENTS.md`'s on testing. A run that selects no test exits 4. Read a failure's log before anything reruns; no unchanged rerun, except once after exit 137.
- **When done,** once: `cargo fmt`; for each tree the branch touches, `(cd <tree> && python3 -I -B scripts/old_forms.py)` from the tree's root, with `--write` only to lower a count for what you removed: old-form counts only fall, and a raised count is red; then `deliver`. Its checks, `.temper/pre-pr`, run clippy on the packages you changed and the focused tests of them and their direct dependents, in every tree.
- **Edits:** a scoped `#[expect(clippy::<lint>, reason = "...")]` on an item you wrote or changed, where reshaping it isn't your task. Generated files include goldens, prints, `docs/help/reference` and counted baselines. `docs/design/`, drafts and `docs/help/` aren't yours, but a help page your change or merge pushes past its byte bound may be condensed, losing no fact and keeping the bound.
- **A red you didn't cause:** g's `.temper/known-reds.md` and your spec's known reds pass. When an answer confirms one isn't yours, add the entry line it gives to that file, then deliver.
```

## 11. Later

- **A project's own overlay,** from its repository: a set of charter
  files under its `.temper/`, read as jig's charter documents are (jig's
  `domain/engine/settings.md`, 9), so that one deployment serves
  projects with rules of their own.
- **Ask temper proposing an overlay** where a person's rule adds to the
  stock, in place of a whole charter.
- **The other stock charters** (the chat, the drafter, Ask temper, the
  status agent, the trial, the reviewer, a standing role) as sets of
  files of their own.
- **Unsaying a stock line:** an overlay only adds; a deployment that
  must drop a stock line sets the charter whole.

## 12. Open questions

- **An overlay's parts:** parts of their own after the stock's, selected
  by their reasons, or merged into the stock's part with the same
  reasons. Default: parts of their own, which select the same way and
  need no matching.
- **The stock implementer's shell:** the stock's charter 2 keeps no
  shell, its text saying what to run with one and without. Default:
  unchanged.
