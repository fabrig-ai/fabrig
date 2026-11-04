+++
id = "notes-and-example"
part = "notes-and-example"
kind = "docs"
needs = []
files = [
  "temper/docs/design/draft/notes.md",
  "temper/docs/design/draft/dashboards.md",
  "temper/docs/design/draft/reference-landing.md",
  "temper/docs/design/draft/reference-case.md",
  "temper/docs/design/draft/reference-code-production.md",
  "tools/citations/allow.txt",
]
effort = "high"
minutes = 40
+++

# `notes.md` keeps its choices; `dashboards.md`'s example is neutral

## Goal

`temper/docs/design/draft/notes.md` loses its run evidence and its mapping from the pipeline, and
keeps every choice with a reason that stands alone: what breaks, what it costs.
`temper/docs/design/draft/dashboards.md`'s example becomes a neutral example project's status page.
This is sections 2 and 3 of `temper/docs/plan/execution-own/inputs/request.md`.

Acceptance:

- `notes.md` has sections 1, 4, 5 and 6, numbered as they are, and every N item it has today, each
  with its choice unchanged. Sections 2 and 3 are gone.
- Neither `notes.md` nor `dashboards.md` holds a counted name (`notes/common.md`, The counted
  names). In `notes.md`, nothing reads "unlike fab", "the run's", "(2.1)" or the like. "the run" is
  left only where it means a coordinator's run (N22).
- Nothing in the repository outside `docs/plan/` cites `notes.md`, 2.x or section 2 or 3.
- `tools/citations/citations.py --all` is at 0, and `--base origin/main` passes.
- The outward check is green. `tests/boundary/outward.txt` is not touched: `execution-lines` takes
  the entries out, and a count may fall below its entry.

## Inputs

- `temper/docs/plan/execution-own/notes/common.md`: its rules hold, and its departures from the
  implementer's text.
- `DESIGN.md`, 4.
- `temper/docs/design/draft/reference-code-production.md`, sections 7 to 9, for the terms N1 and
  N2 use.

## Owns and may change

- **Owns:** `temper/docs/design/draft/notes.md` and `temper/docs/design/draft/dashboards.md`.
- **Readers, only the sentences listed in Steps:** `reference-landing.md`, `reference-case.md` and
  `reference-code-production.md` in `temper/docs/design/draft/`.
- **`tools/citations/allow.txt`:** only as step 4 says.
- **`temper/docs/design/draft/archive/playground-reset.md`** stays where it is, though `notes.md`
  no longer names it. `reference-case.md` still does.
- **Pre-approved:**
  - if a sentence of an owned file holds a lab word Steps does not list, apply
    `notes/common.md` and name it in the report;
  - if `citations.py` finds another citation of `notes.md`'s sections 2 or 3, fix that sentence
    the same way, drop the citation, and name it in the report.

## Steps

### 1. `notes.md`

**Lines 3 to 10:** replace with: 2026-11-04. The reasons behind temper's drafts (`AGENTS.md`,
section 6): the choices they make, each with its reasons. Sections 1, 4, 5 and 6 are the
reference's: `reference-code-production.md`, `reference-landing.md` and `reference-case.md` cite
its choices, N1 to N24, by number. Where the design still lags a draft is that draft's status, not
these notes.

**Section 1:**

- First bullet: "**The reference is one way for a project to develop itself through temper:** a
  person's draft turned into design and a plan by designers, coding agents filling it, a goal
  branch checked along the way and landed whole, then verified after landing."
- Last bullet: its lead becomes "**What the reference holds to, on purpose:**". Keep the list,
  dropping "not specs on a trunk", and write "the close-out before landing, not after it".

**Sections 2 and 3:** delete whole.

**Section 4, each reason.** Items not listed are unchanged (N3, N13, N14, N22, N23). Each entry
says what replaces the sentence that cites the run or names the pipeline; the rest of the item
stays.

- **N1:** Most of a change's questions are about scope: files outside the task, a neighbour's
  fixture, a red the task did not cause. An LLM can settle them. "A change that may not ask ..."
  stays. The bound: "**The bound** (four in the reference) stops a change that keeps asking: past
  it the change is held and escalated, rather than waking its coordinator without end."
- **N2:** A dependent that waits for its dependency to land holds its work behind every merge
  before it; started from the dependency's pushed head, it works alongside. An early start has
  failures of its own, so the reference bounds it: one early start per change, since a second
  repeats a failure its dependencies caused; an answered change waits for its unlanded
  dependencies; an early producer that failed waits for its dependencies; a rerun continued from
  its pushed branch merges the landing branch first; a failed run with no question is held for its
  coordinator; a conflict between the heads refuses the start; and an answer that adds a dependency
  waits for it to land. "A dropped dependency takes ..." stays. This item carries the rules section
  2.1 held, so none may be lost.
- **N4:** delete "The run merged into its trunk lightly and checked along the way (2.1)."
- **N5:** A red along the way is common on a branch many write, and its repair needs no merge
  stopped: merges go on while it runs. "Reds along the way and after landing ..." stays.
- **N6:** Left to the end, catching up is one merge by hand of everything main moved, every
  conflict at once. "The forge's update ..." stays.
- **N7:** Where many write one branch, most conflicts meet in counted baselines, which a recount
  settles with no model, so a model is spent only on what remains.
- **N8:** Batching questions saves wakes, and makes a question that holds up other work wait.
  Waking at once only when a question blocks would keep both, but "at once when it blocks" is no
  wake policy's word (jig's `domain/tasks.md`, 8.3), and is left out.
- **N9:** "jig orders runs waiting for room ..." stays. Then: running merges, repairs and rebases
  before new changes would finish what is started first. "The order matters only ..." stays.
- **N10:** The coordinator and the designers run on Claude Code, and a survey is a read-only
  question, several asked side by side. "Designers are changes' producers ..." stays.
- **N11:** One effort costs most at the wakes that need least: answering takes most of a goal's
  model spend, and at the reference's size (N21) the coordinator wakes about 170 times, fewer where
  messages reach a run already going, each at xhigh. "If the numbers ask: ..." stays.
- **N12, "files on a trunk":** the lead stays "**files on a trunk:**". The coordinator writes and
  pushes them, which a goal's runs may not (`agent.md`, 4.3), and coordination state kept in files
  costs answers and repair sections committed into specs, wakes through a shared file that can be
  missed, and edits to a released spec that no one sees.
- **N15:** "the coordinator needs the report, for a Left list or a designer's list of changes: a
  producer that finishes often leaves something outside its task." Then the contract sentence, its
  "in the run" becoming: a change that builds a test world can meet failures in code it does not
  own.
- **N16:** "... leaves nothing half-said on main. A close-out after the landing and its verdict
  would be a second landing through the queue, and main would hold the code without its status
  until then." "A repair after landing ..." stays.
- **N17:** A gate holding several changes could retry a red test alone before refusing them, so
  one flake does not take the others with it (collateral). "temper reruns failed tests ..." stays.
- **N19:** A plan's changes differ in what they need: most run at high, mechanical ones at medium,
  foremen at medium, and the reference's designers at xhigh with sub-agents at high.
- **N20:** smith's one program backend is Claude Code, and Codex as a second would run on a
  person's ChatGPT login, which is never copied to make it a program. "A key is what smith and jig
  already carry." and "Its umask ..." stay.
- **N21:** "**Why, and the sizes:** a draft of this size is tens of changes, about two hundred runs
  and tens of questions under one coordinator, and a limit met mid-goal stops the goal. The profile
  holds:" and the list stays.

### 2. The readers, in `temper/docs/design/draft/`

- **`reference-landing.md`:**
  - lines 7 to 9: "`notes.md` holds the reasons, cited by number (N1 to N24), and where each item
    of the landing model it drops goes (`notes.md`, section 6)."
  - lines 130 to 133, "**One at a time is enough.**": keep the first sentence, and in place of "The
    evidence agrees: ...": a batch would rarely carry a second change, and bisecting one would have
    nothing to do.
  - lines 147 to 149: "The first keeps the second small: a catch-up left to the end meets every
    conflict at once, by hand, when the goal is otherwise done."
  - section 9's opening: "Three things have no unattended run behind them: a catch-up with main, a
    plan stopped by its bounds, a goal's branch landed with a red after landing routed to a repair.
    They are this reference's first stories:"
  - section 12, "A whole-goal review": "The reference has none; a project may add one by its rules."
- **`reference-case.md`,** lines 9 and 10: "`notes.md`'s N numbers are the choices each step rests
  on." The sentences before it stay.
- **`reference-code-production.md`:**
  - lines 10 to 12: "`notes.md` holds the reasons for each choice, N1 to N24, cited here by number."
  - section 17, "A family's list of changes in g?": "The report fits a family of eleven changes, the
    largest the reference sizes for (N21); revisit past that."
  - section 17, "Should g's tier blame merges": "Triage costs a wake and no machinery; blame would
    need the tier's verdict machinery inside a goal."

### 3. `dashboards.md`

The example is an invented project's status page. It stays unnamed and has no address.

- **Lines 9 and 10:** "This draft's example of such a page is an example project's status page: a
  team running agents on hosts of its own. The page's description is the test's input (section 9)."
  The user's quote above it stays.
- **Line 19:** `"A page like our status page for my deployment"`.
- **Line 58:** "given `status-page.md`, the example project's description of its status page, the
  agent proposes ...".
- **Line 197:** "**The projection is the example page's pace:**".
- **Line 230:** the tile reads "Goals 26 · 18 active, 8 paused".
- **Line 236,** a counted name the request's list leaves out: "host-3: no free slots for 20 min".
- **Lines 325 to 327:** "**The example page fits them.** Every number on it is a count, ...".
- **Lines 369 to 372:** "**The example page as a golden.** `status-page.md`, the example project's
  description of its status page, is the test's data. It goes through a real model as the person's
  description, and ...".
- **Lines 377 to 391, the table:** the header's first cell is "The example's page". These first
  cells change: "goals: active, paused, ended this week"; "what each goal runs now, its last
  activity"; "waiting on a person"; "the lead agent's messages, 24 h"; "gates: count, failures,
  median, p90, by repository"; "builds by kind, the build queue"; "coding runs by model and effort:
  duration". The row for builds has the gap "the project's build tool is its own: temper does not
  read its queue". Every other cell stays.
- **Line 403:** "a dashboard of the example page's panels".
- **Line 422:** "**Public dashboards,** a page for anyone, as the example's status page is, holding
  no names."

### 4. `tools/citations/allow.txt`

- The dashboards entry stays: `status-page.md` is still named as the example project's document, so
  the draft still needs it, and its reason still holds.
- Delete the entry `temper/docs/design/draft/notes.md status.md # ...`: its only target was in
  section 3. The `landing-model.md` entry stays, since section 6 cites it.

### 5. Finish

Read the diff of `notes.md` once whole, against the acceptance. Commit, run `notes/common.md`'s
checks, and push.

## Report

As the implementer's text says, and also: each sentence you changed beyond Steps, each citation
`citations.py` found beyond Steps, and the outward check's count of tests.
