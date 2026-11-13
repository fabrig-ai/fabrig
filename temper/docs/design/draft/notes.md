# Notes behind the drafts

2026-11-04. The reasons behind temper's drafts (`AGENTS.md`, section 6):
the choices they make, each with its reasons. Sections 1, 4, 5 and 6 are
the reference's: `reference-code-production.md`, `archive/reference-landing.md`
and `reference-case.md` cite its choices, N1 to N24, by number. Where the
design still lags a draft is that draft's status, not these notes.

Unqualified, `../domain/forge/README.md`, `landing.md`, `after-landing.md`, `../domain/agent/README.md`,
`../domain/worker/README.md` and `findings.md` are temper's `../domain/` documents; jig's
and smith's are named as theirs.

## 1. In one page

- **The reference is one way for a project to develop itself through
  temper:** a person's draft turned into design and a plan by designers,
  coding agents filling it, a goal branch checked along the way and
  landed whole, then verified after landing.
- **Code production maps onto jig with configuration, and onto temper
  with a few changes.** jig's primitives cover the structure (a goal, a
  coordinator called fresh on each wake, batches with dependencies,
  holds and writer slots, questions, person tasks, escalations, notes,
  jobs). temper's change procedure and its after landing cover most of
  the mechanics; what temper offers a coordinator over them is narrower
  than a plan needs (N22), and a goal's own branch is a change's where
  the reference needs it to be the goal's (N3).
- **Landing is the queue temper has,** one change per goal into main:
  update, CI at the exact head, merge at that head, then tiers. The
  landing model's batches, trains, keys per package, provers and
  serializer are not needed (section 6); landing by turn stays, an
  option the reference does not use.
- **The user's answers (2026-11-02):** implementers ask, through a
  foreman and then the coordinator (N1); a goal's changes land into a
  branch of the goal's, which lands whole (N3); one coordinator per
  goal, at xhigh, mostly configuration (N10, N11). On the plan's text: a
  goal branch everybody writes, designers included, who put their
  output in `docs/design` for this project while other projects do as
  they choose; notes only where they are the right tool; and the ideal
  shape before the mechanisms at hand (N3, N12, N13).
- **Notes stay hints** (N12): the plan's text is the project's design,
  so it belongs in the repository, on the goal's branch, versioned and
  landed with the code it governs. Notes keep jig's use: what runs
  learn.
- **What the reference holds to, on purpose:** the goal's branch held by
  the goal, every write of the goal landing into it by change; designers
  as changes, their design landed into the goal's branch; specs as words
  citing the design; no answers or repair sections committed into specs
  (the store holds them); no specs rewritten by an answerer (an amendment,
  visible, instead); reds along the way go to the coordinator as news,
  as reds after landing do; the close-out before landing, not after it;
  designers at xhigh with surveys as Claude sub-agents at high (Opus 5.5).

## 4. The choices, N1 to N24

Each item is a choice the reference drafts make where the existing
design says otherwise or says nothing, and why. The drafts hold the
rules; their statuses say where the design still lags.

### N1. A change's agents may ask

- **The choice:** "a change never asks" stays the default, and a change
  takes a setting that relays its agents' questions: through a foreman,
  which settles what the change's words and the design settle, or
  straight to its requester (`reference-code-production.md`, section 8).
- **Why:** most of a change's questions are about scope: files outside
  the task, a neighbour's fixture, a red the task did not cause. An LLM
  can settle them. A change that may not ask leaves its producer to
  guess or fail. A foreman first keeps the coordinator's wakes for what
  the words and the design do not settle.
  A designer's questions are about the draft, which no foreman can read
  better than the coordinator.
- **The change kept by its question** (jig's `domain/tasks/lifecycle.md`, 8)
  still steps. It merges nothing while its producer works, and a
  repair's or a resolution's question takes it out of its queue until
  answered, which suits a change that cannot merge yet. A procedure
  decides nothing (jig's `domain/tasks/proposals.md`): it relays, and
  the foreman and the requester decide.
- **The bound** (four in the reference) stops a change that keeps asking:
  past it the change is held and escalated, rather than waking its
  coordinator without end.

### N2. Early starts: a kept change and its producer

- **The choice:** a change depending on another change into the same
  landing branch starts kept, and its producer starts early from the
  landing branch's tip with its dependencies' pushed heads merged in
  (`reference-code-production.md`, section 7).
- **Why:** a dependent that waits for its dependency to land holds its
  work behind every merge before it; started from the dependency's pushed
  head, it works alongside. An early start has failures of its own, so
  the reference bounds it: one early start per change, since a second
  repeats a failure its dependencies caused; an answered change waits
  for its unlanded dependencies; an early producer that failed waits for
  its dependencies; a rerun continued from its pushed branch merges the
  landing branch first; a failed run with no question is held for its
  coordinator; a conflict between the heads refuses the start; and an
  answer that adds a dependency waits for it to land. A dropped
  dependency takes its early dependent with it, since the dependent's
  head carries the dependency's unlanded work, so the coordinator
  remakes the dependent rather than amending the
  dependency away.

### N3. The goal's own branch

- **The choice:** the branch is the goal's, not a change's. The goal
  holds it for its whole life; everything of the goal writes it under
  that hold, by change, one writer at a time; its landing is an ordinary
  change into main whose branch is its requester's
  (`reference-code-production.md`, sections 3 and 4;
  `archive/reference-landing.md`, section 3).
- **Why not a change's:** a goal's own branch made "a change of its own"
  (`../domain/forge/branches.md`, 1) is at once the landing branch of its goal's changes
  and the branch of its own change, and three rules stand against that:
  a branch under temper's prefix takes an exclusive hold (`../domain/forge/README.md`,
  section 2), only the holder's delegates write under a hold (jig's
  `domain/tasks/holds.md`, 1), and a forward move by anyone but the change's
  own writers is drift (`../domain/forge/change/README.md`, 3). A landing branch must also
  exist when named (`../domain/forge/branches.md`), so the goal's changes could not be
  made in the batch that makes it. A "collecting change" that held
  nothing, made repairs and catch-ups into its own branch, and was
  closed and reopened by amendment was weighed: it carries a goal's
  lifetime in a change.
- **Why by template:** a goal's own branch is part of the shape of work.
  A small goal names none, and its changes land into main each, as
  `../domain/forge/branches.md`, 1 advises for most work. A goal cannot take a hold once
  active (jig's `domain/tasks/holds.md`, 2), so the choice is made with the
  goal. The change to jig is not temper's alone: an incident template
  could name its own environment the same way.
- **Why everyone writes by change:** direct pushes by many producers
  would race, and a run holding the branch's writer slot for its whole
  length would stop every other writer. Only the landing's own repairs
  and resolution push it directly, when nothing else should land.

### N4. Light rules for changes into a goal's branch

- **The choice:** a change's checking and updating follow its landing
  branch's rule, and the rule for goal branches asks neither: a change
  into one merges at its head once the forge finds it mergeable, and a
  merge refused for a conflict goes to the merge job
  (`reference-code-production.md`, section 6).
- **Why:** the goal's landing into main checks the whole
  (`archive/reference-landing.md`, section 1), and g's tier checks along the way
  (N5); CI and an update on every change into g would cost a gate per
  change for a branch nothing ships from. One rule on `… branch temper`,
  its last segment open, covers every goal's branch (jig's
  `domain/authority.md`, section 4), since a change's own branches and
  saved work are never landing branches. A merge commit keeps g's
  history, each change's merge, readable on main.

### N5. Tiers that name their branches; a goal branch's tier

- **The choice:** a tier names the branches it follows. A goal's branch
  has a tier of its own, which judges no landing; its reds go to the
  goal as news and the coordinator triages them; merges go on
  (`reference-code-production.md`, section 10).
- **Why:** main's slow tiers following every goal's branch would make each
  change into one wait for their verdict. A red along the way is common
  on a branch many write, and its repair needs no merge stopped: merges
  go on while it runs. Reds along the way and after landing reach the
  coordinator the same way, one mechanism.

### N6. Catching up along the way

- **The choice:** the goal catches up by a change into its branch, its
  producer the merge job, made by its coordinator when main's landings
  touch its open work, paced by its wake policy (N23)
  (`reference-code-production.md`, section 11).
- **Why:** left to the end, catching up is one merge by hand of everything
  main moved, every conflict at once. The forge's update is a pull
  request's, which a goal's branch has not until its landing is made.

### N7. The merge job

- **The choice:** a job kind the project declares, its `merge_job`: given
  a branch and a base, it merges the base in, settles the conflicts the
  project's tooling knows, pushes the merge if none remain, and
  otherwise leaves the merge in progress for a resolution. It serves a
  merge into a goal's branch refused for a conflict (N4), a catch-up's
  producing (N6), and an update the forge refused, the goal's landing's
  among them, before a resolution.
- **Why:** where many write one branch, most conflicts meet in counted
  baselines, which a recount settles with no model, so a model is spent
  only on what remains.

### N8. Answers at once

- **The choice:** the coordinator's wake policy wakes on each question,
  with no batching; revisit with the numbers (section 5).
- **Why:** batching questions saves wakes, and makes a question that
  holds up other work wait. Waking at once only when a question blocks
  would keep both, but "at once when it blocks" is no wake policy's word
  (jig's `domain/tasks/messages.md`, 3), and is left out.

### N9. Order of work within a goal

- **The choice:** Later (`reference-code-production.md`, section 16).
- **Why:** jig orders runs waiting for room by their goal's priority,
  then by when they began to wait; running merges, repairs and rebases
  before new changes would finish what is started first. The order
  matters only when capacity is short, which the reference leaves out.

### N10. Surveys as sub-agents on Claude Code

- **The choice:** smith serves its `sub_agent` to a program through its
  MCP server, each child a program session of its own at the model and
  effort the LLM choice names; inspect-only children run side by side
  (`reference-code-production.md`, section 5).
- **Why:** the coordinator and the designers run on Claude Code, and a
  survey is a read-only question, several asked side by side.
  Designers are changes' producers, not sub-agents (N13), so no child
  writes and no child's answer carries a family's design. A survey is a
  saving, not a need: a run without sub-agents reads for itself.

### N11. One coordinator, several kinds of wake

- **The choice:** one task has one LLM choice, so the coordinator answers
  questions, reads each change's result, triages each red of its
  branch's tier and decides each catch-up at xhigh. Accepted (the user,
  2026-11-02).
- **Why:** one effort costs most at the wakes that need least: answering
  takes most of a goal's model spend, and at the reference's size (N21)
  the coordinator wakes about 170 times, fewer where messages reach a
  run already going, each at xhigh. If the numbers ask: an LLM choice per
  kind of wake on the charter, or catch-ups and repairs along the way
  made by a rule rather than judged
  (`reference-code-production.md`, sections 16 and 17).

### N12. The plan lives in the goal's branch; notes stay hints

- **The choice:** the plan's text (the shared shapes, each family's
  design, the status of the code, each spec) is design, in the goal's
  branch; coordination state stays in the store (tasks, words,
  questions, answers, holds); notes keep jig's meaning, what runs learn
  and pass on (`reference-code-production.md`, section 5).
- **Three homes were weighed:**
  - **files on a trunk:** the coordinator writes and pushes them, which
    a goal's runs may not (`../domain/agent/tools.md`), and coordination state kept
    in files costs answers and repair sections committed into specs,
    wakes through a shared file that can be missed, and edits to a
    released spec that no one sees;
  - **notes in the goal's scope:** no writing by the coordinator,
    revisions fenced, readable by every task of the project (jig's
    `domain/engine/briefs.md`, section 2). But notes are hints, "where a note
    and the facts disagree, the facts are right", and the plan's
    contracts bind; a revision notifies no one, so the coordinator
    amends live changes by hand; a spec is copied from its note into a
    change's words; and the plan's text outlives the goal only by a
    second copy into the design;
  - **the project's design in the goal's branch:** written by designers'
    changes (N13), versioned, merged and landed with the code it
    governs; read in each producer's checkout; revised by a change,
    whose landing is a fact every later checkout has. The reference
    takes this one.

### N13. Designers are changes

- **The choice:** a change names its producer's charter among those its
  maker may delegate to, and a designer's change is one whose producer
  runs under the designer's charter. It lands into the goal's branch as
  any change does, with no rule of its own
  (`reference-code-production.md`, sections 2 and 5).
- **Why tasks, not sub-agents:** a design is a commit, and lands as code
  does; a sub-agent's answer would carry a family's design through the
  coordinator's context, and could not be asked about later.

### N14. A producer delivers more than once

- **The choice:** a producer pushes as it goes, each delivery pushing the
  change's branch (`reference-code-production.md`, section 6).
- **Why:** a kept dependent starts from its dependency's pushed head (N2).

### N15. A change's result carries its producer's report

- **The choice:** a change's result words carry its producer's last
  report after its landing's lines, and the coding agent's charter says
  how to finish with what is outside the task
  (`reference-code-production.md`, sections 6 and 14).
- **Why:** the coordinator needs the report, for a Left list or a
  designer's list of changes: a producer that finishes often leaves
  something outside its task. A change's contract requires its checks to
  pass at delivery (jig's `domain/hosts/charters.md`; `../domain/agent/charter.md`, 3),
  and a change that builds a test world can meet failures in code it does
  not own: marking such a test ignored, the fault its reason, keeps the
  checks green without fixing what the change does not own.

### N16. The close-out lands with the goal

- **The choice:** what the goal leaves for the next one (the status
  files, the designs' Later sections, the draft's fate) is written into
  the goal's branch before its landing, by a designer's change
  (`reference-code-production.md`, section 12).
- **Why:** it lands with the code it describes, in one landing checked
  whole, and a goal that never lands leaves nothing half-said on main.
  A close-out after the landing and its verdict would be a second
  landing through the queue, and main would hold the code without its
  status until then. A repair after landing that changes what the
  close-out wrote carries the edit itself.

### N17. A red at the gate that passes alone

- **The choice:** Later (`archive/reference-landing.md`, section 11).
- **Why:** a gate holding several changes could retry a red test alone
  before refusing them, so one flake does not take the others with it
  (collateral). temper reruns failed tests alone for tiers
  (`after-landing.md`, 5.2), and, under the turn, reruns a gate's failed
  jobs once when every
  failing test matches an open flake finding (`findings.md`, section 5);
  under the queue a CI red is a repair. With one change in the queue
  there is no collateral, and a repair that finds nothing to change says
  so. Revisit if flakes cost repairs.

### N19. A change names its agents' LLM choices

- **The choice:** a change's parameters may carry an LLM choice for each
  of its agents' roles (`reference-code-production.md`, section 5).
- **Why:** a plan's changes differ in what they need: most run at high,
  mechanical ones at medium, foremen at medium, and the reference's
  designers at xhigh with sub-agents at high.

### N20. Implementers on gpt-6.1-sol

- **The choice:** the reference's producers run gpt-6.1-sol on smith's
  direct backend, through an OpenAI account with a key; Codex as a
  program backend is Later (`reference-code-production.md`, section 16).
- **Why:** smith's one program backend is Claude Code, and Codex as a
  second would run on a person's ChatGPT login, which is never copied
  to make it a program. A key is what smith and jig already carry.
  Its umask, its logs' mode and its closed stdin are the program
  backend's to keep when it comes.

### N21. Limits for a goal of this size

- **The choice:** the reference deploys with a profile sized for its goal
  (`reference-code-production.md`, section 4).
- **Why, and the sizes:** a draft of this size is tens of changes, about
  two hundred runs and tens of questions under one coordinator, and a
  limit met mid-goal stops the goal. The profile holds:
  - G's delegates: about a hundred changes over the goal's life (the
    designers', the plan's, catch-ups, repairs along the way, the
    close-out, the landing, repairs after landing), and person tasks;
  - a change's tasks: a producer, up to four foremen, merge jobs,
    resolutions, repairs, about twelve;
  - tasks per tree, about eight hundred, and depth: G at the root, a
    change 1 below it, its producer, foreman or resolution 2;
  - G's inbox (one result's room per live delegate: jig's
    `domain/tasks/messages.md`, 2) and open questions per task;
  - the words of a change and the bytes of a result (N15): a designer's
    report lists up to eleven changes; the batch size; and a call's
    input bound, which a `change` call carrying a family's changes must
    pass, one change per call if it must;
  - a charter's documents.

### N22. A coordinator makes changes with all their parameters

- **The choice:** `change` takes, beside the words, what a plan decides
  (`reference-code-production.md`, section 3), rather than a run being
  given the change procedure's `delegate`: the root keeps building the
  delegate from what it knows, and the run names only what the plan
  decides.
- **Why:** the plan's changes go into the goal's branch, on its own held
  branch, with dependencies, as one batch, relaying questions, with LLM
  choices, a producer's charter or the merge job, or naming the red they
  repair (`repairs`, `after-landing.md`, section 7). A run may delegate
  to the change procedure under the stock rules, but cannot know what
  the procedure's delegate needs (`../domain/agent/tools.md`, 1); `change` builds it.

### N23. A charter's default wake policy

- **The choice:** a charter in configuration carries a default wake
  policy, in jig's closed vocabulary (jig's `domain/tasks/messages.md`, 3); the
  coordinator's is `reference-code-production.md`, section 13.
- **Why:** every delegate's result, not only the last: the coordinator
  reads each result for its Left list, and makes a family's batch from
  its designer's report (N15). News batched: a catch-up or a red of g's
  tier needs no answer within minutes, and each wake is a run at xhigh
  (N11).

## 5. What would reopen a choice

The numbers: questions per change and the share the foreman settles (N1,
N8); catch-ups and their conflicts per goal (N6); flakes at the gate
(N17); the coordinator's wakes and their cost at xhigh (N11); the size of
designers' reports (N15, N21).

## 6. The landing model's items

The reference drops the landing model (`landing-model.md`, 2026-10-30,
readable in history at `8a4c5bcd9f`), a plan for many landings at once,
whose items other documents cite by number. Each item, under the
reference:

| Items | Under the reference |
|---|---|
| L1 batches, L2 trains, L3 candidates by a merge chain, L5 keys per package, L6 gate classes, L7 provers and runners, L9 the serializer's durability, L18 the reservation | not needed: one change at a time owns main's queue; landing by turn is `landing.md`'s |
| L4 the project's tooling as a job, no model | needed, as the merge job (N7) |
| L8 a measured failure rerun once before red | Later, for the gate (N17); tiers have it (`after-landing.md`, 5.2) |
| L10 a repair pinned ahead of every landing | temper's branch repair, first in its queue (`../domain/forge/queues.md`) |
| L11 a change's own checks before ready | the producer's checks at delivery (`.temper/pre-pr`) |
| L12 leases on paths across goals | not needed: one branch per goal, held by it (N3); overlaps between goals Later (`archive/reference-landing.md`, section 11) |
| L13 tiers following main, L14 cleared as a setting | needed, as `after-landing.md` has them, each tier naming the branches it follows (N5) |
| L15 findings, L16 QA walks | `findings.md`; the reference uses only a red of no goal's |
| L17 trailers in landing news | `after-landing.md`, section 8; the reference does not need them |
| L19 views and acts | the client's, for what remains: the queue, tiers, verdicts, holds |
| L20 one change in one repository | kept: the reference is one repository |
| J1 owned holds | jig's; the goal's hold on g is one (N3) |
| J2 to J6, J9, J12: drains, inherited holds, parks until an event, moves, successors, holds over admission, lent workspaces | not needed by the reference; jig's designs keep them |
| J7, J8 a conversation's questions | not needed: no chat stands between the goal and its person; jig's `domain/tasks/messages.md`, 5 keeps them |
| J10 an address by role | not needed: questions and escalations go up the tree |
| J11 a fresh run at a context's bound | the coordinator starts fresh on each wake (jig's `domain/engine/executors.md`, 4); a long run on Claude Code is the program's to compact |
| F1 skein's artifact read | needed by tiers (`after-landing.md`, section 4) |
| F2 to F5 GitHub's turn and permissions | not needed: the reference lands by queue |

## 7. The goal as designer

The reasons behind `goal-as-designer.md`, from the user, 2026-11-06.

- **One session designs** (revises N13's default). Where the goal can't write, a
  second session at the same effort must re-read the code to write what the first
  surveyed, which the pipeline's single cut doesn't. Extra designers stay for wide
  plans.
- **The goal may write, through a change** (revises N12's reason against files, that
  a goal's runs may not push). N12's other reasons stand: coordination state stays in
  the store, and g is written by change alone.
- **A spec is the task's** (keeps N12's words, drops "short"). The user finds a spec in
  the git tree a hack and a note an unnecessary indirection, and 64 to 128 KB cheap.
  Short specs were one of the ranked causes of the first comparison's rework.
- **No wake on each result** (revises N23). The pipeline's designer is never woken by a
  result, and the user expects the tree to run itself.
- **The close-out after the landing** (revises N16). It is the pipeline's sequence, and
  the user asked for the same sequence.

## 8. Instructions per wake

The reasons behind `per-wake.md`, from the user, 2026-11-07.

- **Instructions per wake, not an answering task.** The user asked whether temper
  could answer as the pipeline does, a fresh call with an answering prompt, and
  chose instructions per wake over a task added for each question. An answer often
  amends the asking change, makes a prerequisite and adds it to the asker's
  `after`, or revises the design in g: each is its requester's, the goal's (jig's
  `domain/tasks/changes.md`). A task of its own would need that authority, a
  second writer of the goal's plan. The goal's later runs already start fresh,
  so only the instructions differ from the pipeline.
- **Reasons from jig's vocabulary.** The engine already gathers a brief for why
  a run runs, and the store's kinds of message are a closed set, so a part names
  them rather than new words for a wake. Only what woke a run chooses its parts,
  or every run would read the parts for whatever rode along.
- **Kept for the next run, not offered with its part.** A message no part covers
  stays in the inbox, judged again when the run ends, as an unread one is. A part
  given mid-run would put instructions inside a message, and the cut never
  answers a question anyway, as the pipeline's doesn't. Replies to a run's own
  acts, entries, an amendment, a request to finish and a person's words always
  reach it: a run waits on its replies, no policy keeps a decision from whoever
  must make it, and no batch holds a person's words (jig's `domain/tasks/messages.md`,
  3). An entry left undecided wakes the next run, as entries aren't inbox
  messages and nothing would judge one again.
- **A covered entry waits for a change** (the user, 2026-11-08). A goal that
  passed a held repair's question to its person and parked was woken 19 times
  in three minutes by that escalation, each run inspecting it, deciding nothing
  and parking again, a run's cost each time. A run whose part covered the entry
  already chose to leave it, so only the entry's change, or another wake, brings
  it back. An entry the run had no part for still wakes the run that has.
- **Every reason waiting at the claim** (the user, 2026-11-08). A goal run woke
  as its landing's result came due, read only the shared instructions, and
  ended without the work the results part asks: the result arrived between
  its preparation and its claim and was kept back against the parts prepared.
- **Fresh runs only.** A conversation's instructions can't change under it
  without losing its cache and its turns' reading.
- **A later choice, not one per kind** (revises N11's one choice per task). One
  run takes one model, and the need measured is the cut against the rest. It also ends an amendment that
  whoever started the goal had to remember, and the goal couldn't make itself.
- **Focused.** What a role doesn't act on costs every run, and blurs what it does
  act on. The comparisons' goal read 31 KB at each run, a fifth of it rules for
  building and testing it never runs.

## 9. The TUI

The reasons behind `tui.md`, from the user, 2026-11-07.

- **Two jobs, not one in two shells.** "It should not be a replacement for
  the web dashboard, I think the TUI should be more 'working with
  agents'-oriented, and the web UI more 'dashboard overview of where things
  stand, what needs me, progress, etc'." The TUI's pages are its own, not
  the browser's drawn in a terminal, which would be a lesser web client.
- **Unattended once started.** "The expectation is that once a work starts
  it goes on unattended." Steering a goal or a delegate from the TUI was
  offered and left to the web, which may show enough for a quick look.
- **Agents off the person's machine.** Unlike Claude Code, temper's agents
  work on the forge; the user: "that's a big plus".
- **A side pane, no diffs.** A draft being written shows "a side pane like
  the web UI, showing the draft. Pane can be folded, half-screen or full
  screen", where an inline diff was offered.
- **Full screen, not inline.** Claude Code keeps its past in the terminal's
  own scrollback and redraws a live region below it. A pane beside the
  conversation needs the whole screen, and a recorded frame is well defined
  only as a whole screen, so the TUI takes the screen, as `chat-reading.md`
  already scrolls by keys.
- **The browser approves.** "Ideally I start temper and it opens a tab in
  the browser, where I accept and then I can close the tab. Over ssh a cut
  & paste code will be needed." It is jig's built-in, not temper's own
  sign-in ending in `Mint`, because jig mints for an application's
  providers alone and a box's person is signed in by jig's printed link;
  letting `Mint` reach jig's own providers would hand jig's rule to an
  application's code. A browser alone approves, since agents drive
  `temper ctl` as the owner. The code shown on both sides lets a person
  check that the tab is their own terminal's, but on a box it rides the
  tab's address, so that one press approves: a risk taken because
  whoever could ask for an approval is on the machine already. A
  deployment at a name will have the code typed.
- **The bare minimum first.** "First just the bare minimum for e2e, it will
  get us the foundations laid out in the code", on a box alone, with no
  Google and no deployment at a name: "we start same machine, but we'll
  move to named deployment soon". The user then uses it and refines the
  design on its recordings, so the TUI records from the start.
- **In v0.1.0.** The user put the first pass in v0.1.0, which had left the
  terminal client out.

## 10. Ask temper

The reasons behind `ask-temper.md`, from the user, 2026-11-07.

- **One chat, grown from configuring.** The user asked for "an overall
  'embedded' AI in temper": configuring a project's charters and rules to
  their workflow, "where are we at?", and questions about a goal's run, "the
  user can start chats/session with this prompt/model and ask what they
  need", with a single prompt "unless it turns out bloated". The three share
  temper's model and the voice, and differ by a few paragraphs.
- **What they care about most:** "that the model understands temper/jig well
  and can reason accordingly". So the instructions carry temper's model with
  its reasons, at the height fabrig.ai explains it to people, rather than rules
  learnt from findings.
- **Small.** The configuring chat's first request was 52 KB on ChatGPT and 62
  KB through Claude Code, 43 KB of it tools, most of them ones it may never
  call: `delegate`, `amend` and `change` alone were 14 KB. Claude Code's own
  prompt is written for a coding terminal and tells the run of tools it lacks.
  The user asked for no workspace, no file or shell tools and no Claude Code
  prompt.
- **Tools on demand, settled by pruning.** The user asked whether tools could
  be described when asked for. With four or five tools there is little left to defer,
  and a provider's own deferral would be two mechanisms; one reading tool by
  address keeps its declaration a line and its addresses on a page.
- **Not ctl's text.** The user was wary of `temper ctl`'s output for this
  reader. On a playground box it printed the raw authority line in `me`,
  money in two units, charters by number alone, each call's JSON and long
  cursors, and commands to copy: fit for an agent driving temper, not for one
  explaining it. The queries are right, so `see` reads them and answers in its
  own way.
- **Code for mechanisms.** Half of `configuring.txt` taught a model what code
  can do: a note's line arithmetic, copying a repository's host and port, who
  may approve, a large proposal through a note. The tools do it instead.
- **Green field, beside.** "I'd rather start greenfield, we can have some switch
  that keeps what's in place, but let me play with the greenfield version
  myself since it's something that requires human judgement." So no sweep of
  the old instructions, the configuring chat kept, and no golden judging it.
- **It changes nothing but settings,** and most probably replaces the
  configuring chat (the user's answers to the two questions).
- **In the rail, the old chat dropped.** Having read the draft, the user:
  'the "configuring agent" is now the built in temper AI', and "we should find
  another place for it, not Settings". Offered both side by side to compare:
  "just drop the old one". It keeps the name Ask temper with no persona, since
  its instructions have it speak as temper and the landing page already says
  "temper proposes". The rail over a box on Home, which would invite requests
  for work it cannot do.
- **Memory, guarded.** "Adding notes to the tool… would be the model's memory,
  so it can adjust a bit to the person preferences and remember past choices",
  with care: "it tends to create bloat and unexpected behaviors. The best would
  be a notes system with guardrails". jig's `notes-memory.md`, section 3,
  already held it; the user adopted it for this chat, narrowed to its runs and
  shown to the person alone, since a chat's transcript is read by the
  project's roles. Past choices are records already, in the settings' history
  and proposals with their reasons.
- **Pictures like fabrig.ai's.** The user asked that it "communicate very well
  graphically", like the home page's "Four examples, not modes" rail, which
  "depict well what is code, what is LLM, what is human", with "zooming the
  system at various level of detail, for example the landing queue (how it
  works, what knobs are there), the goal execution (their roles, their
  prompt)". Views drawn by code can't be wrong and cost the model a line;
  sketches cover what doesn't exist yet. The look is temper/ui's.

## 11. The deployment's binaries

The reasons behind `deployment-binaries.md`, from the user, 2026-11-08.

- **Two sets, two jobs:** "The current one at https://fabrig.ai/temper/... would be the "try temper via a box" one, stripped and gzipped (I'll update the site accordingly), always pointing to the latest release (main for now). The new set would be the "real deployment" one, under https://fabrig.ai/temper/v{MAJOR}/..., comprised of these binaries (gizip'ed, but not stripped): "temper" just the TUI, "temper-engine" engine plus web, "temper-worker" worker process, "temper-agent" smith. I think that'd be it for a full deployment, unless I'm missing something."
- **Not stripped:** the draft reads it as keeping a deployment's crash reports readable, while a trial download's size matters more. That reading is the draft's, not the user's words.
- **No new CI:** "Our development process stays unchanged for now, no special build jobs for these binaries for now, build-level breakages would be caught when I ask for the one-off release build."
- **The name `temper`** in both sets is the user's to settle (section 5 of the draft).
- **`ctl` beside the TUI:** the user asked for "just the TUI". The draft puts `temper ctl` in the same binary, since it's a client too and a deployment needs one. That's the draft's reading.

## 12. The whole story

The reasons behind `whole-story.md` and the four drafts written with it,
from the user, 2026-11-08.

- **One front end.** "I'll be the first user of temper and ideally
  temper would be my only frontend, replacing most of the things we now
  run by hand." The pieces were there, the dogfooding, the web becoming
  an overview of where things stand and what needs the person, the TUI
  for focused sessions, but not one story joining them.
- **Some things stay outside.** A deployment's operations and the care
  of its logins have no equivalent in temper; they belong to fabrig.ai's
  later Run.
- **A middle ground between sessions.** "Very long running sessions
  should be probably avoided, but maybe we need a middle ground between
  function call-like sessions like the ones producing the code and long
  running ones." Offered standing roles with short sessions that end by
  writing their state, the user: "yes". The measure behind it: on the
  long sessions we run by hand, 74% of their own spend on 2026-11-07 went
  on wakes after the prompt cache had expired.
- **Room for every point of the spectrum.** The vision must also fit
  "other ways of using temper, perhaps at smaller scale", so the pages
  fill as a person hands over more, rather than being drawn for the
  largest case, and two more profiles were drawn: Pair and Team.
- **A pilot that isn't temper.** The user's first: cowsql's vsr, a
  solid C implementation of Viewstamped Replication replacing cowsql's
  raft layer, ported to Rust line by line, continuously, "unsafe is
  fine, guarantees would come from C tests and QA, also replayed on the
  Rust version"; the port, in a repository of its own, will replicate
  skein-kv. On the same deployment, as a new project: a brand new
  deployment would show the same pages, deployment-wide settings aside.
- **Greenfield pages.** "Take it as greenfield, doesn't matter what we
  have now, let's design optimizing for the vision", the rail, Chats and
  Settings included. Shown the sketches, the user asked for them in the
  design whole, and for all of it by v0.1.0: "we have resources (hw and
  subscriptions), so let's try to move fast and parallelize when
  possible", hence the tracks of `whole-story.execution.md`.

## 13. The command line

The reasons behind `cli.md`, from the user, 2026-11-08.

- **What they asked for:** "a fresh look to fabrig's CLI interfaces, in terms of environment variables, command line arguments and build flags". Every convention is "repo-wide", documented under `skein/docs/foundation/` "and enforced by code wherever possible (ideally everywhere)".
- **No features:** "We should have no feature flags, "cargo build" from a fabrig repo checkout builds everything." Asked whether that grows the binaries ("ballpark, no exact measures needed"), the draft's answer is not significantly: a feature that becomes a crate leaves a binary linking what it did, and the recorder and the browser's switches, built always, are small and measured as they go.
- **Two sets:** "user facing (public, stable) and internal (hidden, unstable)", the public ones "advertized via the cli (<binary> {-h,--help,help}) in standardize way", and all of them documented "with standardized format and location (proposed: <tree>/docs/design/cli/<binary>-{env,args}.md) and always be in sync with actual code", and "Env vars common across several binaries go into <tree>/docs/design/cli/common-env.md".
- **Layout:** "All binaries are built from <tree>/bin/<binary>/main.rs", "main.rs is just a few lines of plumbing code (you tell me a reasonable size, I think 100 lines at most)", and "all actual logic lives under <tree>/crates".
- **The public set's quality** can't be checked by code: "elegant, orthogonal and ergonomical".
- **Scope:** "let's limit the scope to temper binaries for this round, other trees can wait", and "this should [be] in the v0.1.0 roadmap".
- **What's internal:** "from a quick look all ctl should be internal (for now), all fakes, and probably "temper forge", not sure if something else."
- **The draft's first look, agreed whole** ("everything you said sounds good"): a marker in internal variables' names, a third set for what tests alone read, `main.rs` at 30 lines, one table per binary rendering the reference documents.
- **The names:** the current single binary "temper" should become "temper-box", and `temper` is the TUI. On delivering the box, the user weighed serving `temper-box` under the name `temper` against `temper box` downloading `temper-box` at its first run, and chose the second: "Let's go for "temper" as TUI only and temper-box downloaded when not found around (~/.cache/temper)."
- **What `temper box` does:** "start the TUI normally, as a regular "temper" run, but also print some message saying it's downloading temper-box and then start temper-box as sub-process" (the user). The draft keeps today's behaviour around it: the box's log in its file, and quitting the TUI stopping the box.
- **The download's time,** the draft's estimate for the box's 24 MB: about 1 to 3 seconds on fiber, cable or a cloud machine, which most of temper's audience has; 12 to 24 seconds on ADSL; 40 seconds to a minute and a half on a weak mobile hotspot. The download line pipes through `gunzip`, since curl has no short form of `--compressed`.
- **The draft's readings,** not the user's words: `ctl` stays in `temper`, internal; `temper box` doesn't look on PATH, where another commit's `temper-box` may be; `serve` and `program-login` are public on `temper-box`, for one person's deployment on one machine; `TEMPER_INTERNAL_BUNDLE=none` stays for development builds' speed.

## 14. Claude through an API key

The reasons behind `claude-api.md`, from the user, 2026-11-11.

- **What they asked for:** "add support for using claude via API token, that would be via smith agent natively (no need to call out to native claude binary), since that's allowed by Anthropic terms of use (please check)", and "Initially it's sufficient to expose this via "temper ctl" (basically you can add an account in temper that uses a Claude API token)."
- **The terms, checked:** Anthropic's pages, read that day, allow a customer's own API key in software built on Claude, and forbid routing a subscription's credentials through it (`claude-api.md`, section 2).
- **One provider, not two:** a second, key-only Claude provider beside Claude Code's was weighed first, as configuration alone. It fails on the catalogue: jig refuses two models of one name, whatever their provider, and charters name a model by its name, so a second provider can't carry its own `claude-opus-5-5`. Making a model's identity its provider and name would be work that one provider never needs. One provider whose source picks the route keeps every piece.
- **In two steps:** "let's build B in two steps as you recommend": step 1 in v0.1.0, and "we'll evaluate the second one later, if it fits in v0.1.0 or not."
- **The draft's readings,** not the user's words: a subscription limited everywhere holds its account's runs, as jig's order already says, rather than spending the key after it, until step 2 weighs that; Claude's cache prices replace the stock's cautious ones, since Anthropic's page gives them and a key pays them; the pages draw the key where they draw OpenAI's, with no new look.

## 15. The crate graph

The reasons behind `crate-graph.md`, from the user, 2026-11-12.

- **What they asked for:** "ideas for our speeding up fabrig builds, from a crates organization perspective", starting from a report on the shared compiler cache's hits for a build of main from scratch.
- **The first read, which the user agreed to:** (1) one feature set for every build; (2) generated code in crates of its own; (3) the crates that change most split along the lines they change; (4) checks that hold the shape, and a figure after landing.
- **Test targets:** asked whether they should be a section, the draft recommended it, from the Cranelift timings, and the user said "go ahead" (`crate-graph.md`, 6).
- **The scope:** "1 should be solved or about to be solved ... start your draft for 2 3 4 (assuming 1 is properly fixed already, otherwise include also what's missing)". The round in flight that day removes the workspace's own features (`cli.md`, 3.2, whose request reads "no features, so every build is reusable", the user, 2026-11-12). External crates' features still follow what a build selects, so the draft takes that part (`crate-graph.md`, 3).
- **The toolchain:** "we're very likely switching to rust nightly plus cranelift", then "will switch within an hour or 2 if the pilot is fine" (the user, 2026-11-12). The pilot failed tests under Cranelift, and the user first kept the draft as it was while it was fixed, then chose "plain LLVM on nightly, better than nothing" (the user, 2026-11-12). So the draft keeps the trial's Cranelift figures as measured, and pins nightly with LLVM as a milestone of its own (`crate-graph.execution.md`, G9).
- **Features, in two layers:** "re feature-unification we can use it if you think it's appropriate but I'd also suggest that we still have our 'own' feature unification (i.e. we have no features, anywhere, which was my original plan, and when thirdy party needs features we use the same ones everywhere)" (the user). So `crate-graph.md`, 3.3, is ours: inherited external dependencies and the root's unions; and 3.4 is Cargo's, for the crates below ours, which no manifest of ours names.
- **The draft's readings,** not the user's words: the browser's nested wasm32 build kept to its own selection; a features crate only if the workspace returns to stable; only the generated record code in codec crates, each large family its own, with schemas, kept texts and patches staying with their owner, whose patches may name its domains; a split landing as preparatory changes, then moves with the old crate re-exporting for a time; build contexts, a platform and whether a build has the dev-dependencies, as the rule's unit, since Cargo's unification keeps them apart; lines as the size's measure, 60,000 of a library's hand-written code and its own tests and 20,000 a shared family, proposed for the user; and embedded files, test worlds and links left for later.

## 16. A gate's compile

The reasons behind `gate-compile.md`, from the user, 2026-11-13.

- **What they asked for:** after `crate-graph.md`'s milestones landed, "you can take measurements yourself, just look at what landed in the last 8 hours via the CI landing job ... and measure build times (or triangulate them if they are mixed with test times)"; then "come up with a follow up draft for improving the situation against your observation", the shared compiler cache's operational problems aside. Then "okay, I like it, let's go for all the changes you propose except the weekly ranking (for now)" (`gate-compile.md`, 6 and 11).
- **The draft's readings,** not the user's words: the gate's build step as the measure, since it builds every test target; units no milestone touched as a rough yardstick for the ones the milestones split, since the toolchain and the builder's load changed too; a split judged by everything that replaced the old units, not by its largest piece; compile per build as what's left to cut; generated tests as integration tests, an exception to step tests; one version's generated code as skein's codec design already says; the first split measured before the next; and a weekly ranking of the next cuts, proposed and left for later.

## 17. Panics

The reasons behind `panics.md`, from the user, 2026-11-13.

- **What they asked:** "I'm told that we have only one place in the codebase where we catch panics. Do you think it'd be good to remove that place and forbid catching them? Cranelift doesn't support them (I'm told), and in general I prefer error handling or abort". Main's code outside tests catches none: the one place is a test module (`panics.md`, 2).
- **Tests too:** "there are tests catching panics, but they are ignored atm. And I think they should be converted to panic-less too."
- **Few tests across a process boundary:** "my only concern is test speed for tests exercising panic and spawning a sub process", asking whether the rule can hold "without 1) having (too many) spawn-sub-process tests 2) keep that number low and stable, only when strictly necessary, and don't 'induce' more of them (implementers or designer see it as a pattern, adopt it and grow it)", since implementers write "tests that bite", and "We'd probably need to add a few lines (hopefully just a sentence or too) to skein's programming-model.md". Hence no test triggers an assertion (`panics.md`, 3.3), and section 6.
- **The toolchain:** "we'll use Rust stable with --release for production and releases" (`panics.md`, 8).
- **The scope and the words:** "Assuming that the risk you mention is handled by your draft (if not, please flag it), include everything in the panic draft, your amendments to programming-model.md are accepted (with your extra wording too)". The risk is the held login's (`panics.md`, 5). The extra wording is the store's contents among what comes from outside the process (3.2).
- **The draft's readings,** not the user's words: the four kinds of section 4 and their counts, a script's rough split; the launcher taking `Lent`'s last case; a count per file before the ban, so that no milestone marks each remaining site with an `#[expect]`; the `#[should_panic]` tests after the 123, since they pass today; a held-login story built with LLVM until its milestone lands.
