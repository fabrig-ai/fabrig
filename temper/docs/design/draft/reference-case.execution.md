# reference-case: execution

2026-11-02. How [reference-case.md](reference-case.md) gets built: the
milestones that carry `reference-code-production.md` and
`reference-landing.md` into the design and the code, until the case
walks whole as `v0.1.0.md`'s rung C. It designs nothing: each milestone
executes a part of the drafts and writes its own design as it goes.
`notes.md` holds the reasons for the choices cited here, N1 to N23;
where the design lags each is the drafts' statuses.

Unqualified, `forge.md`, `landing.md`, `after-landing.md`, `agent.md`
and `worker.md` are temper's `../domain/` documents; jig's and smith's
are named as theirs.

## 1. In one page

- **Eleven milestones,** M0 to M10 (section 3). M0 is the ground the
  reference stands on; M1 to M9 each execute a part of the drafts; M10
  is the case whole.
- **Each runs through fab's loop whole, its design included** (section
  2): its input is its entry here and the draft sections it cites; its
  designers write the design into its trunk family by family, feeding
  its implementers as each part is ready. Shared design is kept to the
  least: no milestone designs ahead for another, and where several
  touch one section, each adds its own part (section 4).
- **Each is walked as a rung:** a QA brief of the case's steps it makes
  possible, driven through `temper ctl` alone.
- **A spine and three side tracks** (section 4). The spine is the goal's
  own branch (M1), then four milestones side by side on it (designers,
  questions, early starts, the merge job into it) and its landing (M2).
  Sub-agents, jobs and tiers on main start with M1, needing none of it.
- **Mechanisms in code, judgement in charters:** each milestone says
  what it builds and which of `reference-code-production.md`, section
  14's settings it configures.
- **Each milestone's state** is in section 3's table, kept current. When
  M10 passes, the drafts' statuses hold none of these entries, and this
  file goes with the drafts (`AGENTS.md`, section 9).

## 2. How a milestone runs

A milestone runs through fab's loop whole, its design included, under
fab's rules for a run's lead and its designers (the agents
repository's `fab/roles/cut-run.md`):

1. **Its input:** its entry in section 3 and the draft sections it
   cites. The user starts it as a plan of its own,
   `temper/docs/plan/<milestone>`: its trunk made, the entry its
   request, this file and the drafts the cut's sources.
2. **Design and build, one pipeline.** The cut releases first what
   needs nothing. Its designers, one per family, write the family's
   design into the trunk: the sections the entry's Design names, each
   document's status with a Not yet for what is still to build, its
   Later, and the entries they close in the drafts' statuses. Then the
   specs that cite it, released in batches while the implementers work
   on the earlier ones. Implementers never write the design.
3. **Landed whole:** the trunk is checked and lands through the queue,
   and the plan's retire brings its statuses current.
4. **Its walk:** a rung of the ladder's plan. fabrig/qa writes a brief in
   a rung's form (`v0.1.0.md`, section 3): the case's steps the
   milestone makes possible, with the case's names, on the case's
   backends, through `temper ctl`. A step that needs a later milestone
   is walked in its plainest form or left out, and the brief says
   which. The walk's findings are repaired through the loop; the same
   brief, rerun unchanged, is the pass.
5. **Done** when its walk passes. Walks grow: each brief extends the
   briefs of the milestones it needs, so M10's is the case.

Its designers, its implementers and its walker are separate sessions.

## 3. The milestones

| | Milestone | Executes | Needs | Walks (case steps) | State |
|---|---|---|---|---|---|
| M0 | Ground: `playground-reset.md` closed out; fab's designers write the design | | | | not started; the close-out's owner to be decided; fab's change asked of agents/loop |
| M1 | The goal's own branch | N3, N4, N15, N23; N22 in part | M0 | 1, 3, 7, 19 | not started |
| M2 | The goal lands | N3 for L; N22's `mine` | M1 | 23, 24, 28 | not started |
| M3 | Designers and the cut | N12, N13, N16, N19, N20 | M1 | 3, 4, 6, 22 | not started |
| M4 | Surveys as sub-agents | N10 | M0 | 2, 5 | not started |
| M5 | Questions, and holds passed up | N1, N8 | M1 | 5, 9, 11, 14, 15, 18 | not started |
| M6 | Early starts | N2, N14 | M1 | 4, 8, 10, 13, 17 | not started |
| M7 | Jobs, the merge job, catching up | N6, N7 | M0; M1 for g, M2 for L | 12, 16, 21 | not started |
| M8 | Tiers and verdicts on main | `after-landing.md`, 2 to 6; N5 for main | M0 | 25, 28 | not started |
| M9 | Reds to the goal; g's tier | `after-landing.md`, 7; N5 for g; N22's `repairs` | M2, M8 | 20, 26, 27 | not started |
| M10 | The case | N21; the reference's configuration whole | every one; rung B2 passed | 1 to 28 | not started |

N9 and N17 are the drafts' Later; N11 is accepted and asks no work.
Each entry's Design names the documents and sections its designers
write, from the drafts' statuses; Builds, the trees its implementers
change; Configures, the settings of `reference-code-production.md`,
section 14 it adds.

### M0. Ground

- **The playground's reset closed out** (the user's X5): the adopted
  `playground-reset.md` ported into the design as its status lists,
  then removed. It comes first because the milestones' designers edit
  the same documents (`forge.md`, temper's
  `../protocol/configuration.md`, jig's `domain/tasks.md`).
- **fab's designers write the design** (the user, 2026-11-02): the cut's
  designers write their family's design into the trunk with their
  specs, as section 2 has it, where today they write specs and notes
  alone. agents/loop's change to fab.
- **Done** when `playground-reset.status.md` is gone, with the design
  statuses' Departs entries for the playground's reset, and fab's change
  has landed.

### M1. The goal's own branch

- **Executes:** `reference-code-production.md`, sections 1 and 3 (G
  holds g; every change is G's, made by `change` with its landing
  branch, its dependencies and as a batch), 4 (a goal from the template
  for drafts, its own branch made as its hold is taken), 6 (a change
  into g under the light rules; its result carrying its producer's
  report) and 13 (G's wake policy).
- **Design, jig's:** a template naming *its own goal branch* (jig's
  `protocol/configuration.md`, 3.1); an effect of a hold's taking, and
  of its release (jig's `domain/connectors.md`, 3.3 and 4.6, and
  `domain/tasks.md`, 6.5); a held resource a landing queue orders (jig's
  `domain/tasks.md`, 6.1); writer slots taken by what a run writes
  (jig's `domain/tasks.md`, 6.3, against its `domain/engine.md`, 7.1); a
  charter's default wake policy and the vocabulary it needs (jig's
  `domain/tasks.md`, 8.3, and jig's `protocol/configuration.md`,
  section 3); a
  result's bound past the settings schema's cap.
- **Design, temper's:** g as the goal's, not a change's, a change from
  outside G's subtree refused (`forge.md`, sections 2 and 11, and 11.1);
  `change`'s landing branch, dependencies and batch (`agent.md`, 4.3.1;
  `forge.md`, 8.1); checking and updating by the landing branch's rule
  (`forge.md`, 8.2 steps 3 and 6, 8.7, section 10); a result's words
  carrying the producer's report (`forge.md`, 8.2 step 7).
- **Builds:** jig (templates, holds, writer slots, wake policies,
  limits); temper's forge connector and `change`.
- **Configures:** the coordinator's charter, its first edition (a plan
  of changes G specifies itself, results and Left lists, the goal's
  end); the goal template for drafts; the coding agent's charter (Left
  lists, tests marked ignored for code it does not own: N15); the
  landing rule for goal branches; the merge commit; `.temper/pre-pr`.
- **Walk:** steps 1, 3 (G specifies its changes itself, one after
  another), 7 and 19, then the goal's end with g kept, unlanded. A
  dependent starts once its dependency has landed; no merge into g
  conflicts (the merge job is M7's).

### M2. The goal lands

- **Executes:** `reference-landing.md`, sections 1 to 4 and 7 (L, its
  way into main, what moves g while L lives, its bounds, main's queue,
  a small change); `reference-code-production.md`, section 12 (ready to
  land, L made).
- **Design:** `forge.md`, 8.1 (a change on its requester's held branch)
  and 8.5 (a move by its requester's other delegates is its new head,
  not drift); `forge.md`, section 11 and `after-landing.md`, section 6
  (g deleted as its goal closes if a change landed it, kept otherwise);
  `agent.md`, 4.3.1 (`mine`).
- **Builds:** temper's change procedure on a held branch, and g's
  deletion as G's hold is released.
- **Configures:** the coordinator's charter: L, its escalations, changes
  into g while L is held; the landing rule for main; L's CI, the full
  gate; three repairs.
- **Walk:** steps 23, 24 and 28, L ending at landed until M8; and
  `reference-landing.md`, section 9's second story (L held past its
  repairs, released after a change into g).

### M3. Designers and the cut

- **Executes:** `reference-code-production.md`, sections 2 and 5 (the
  cut, designers as changes, a family's batch as its design lands, a
  spec, a revision of the design) and 12 (the close-out).
- **Design:** `agent.md`, 4.3.1 and `forge.md`, 8.1 and 8.9 (`change`
  naming its producer's charter and an LLM choice per role); temper's
  `../protocol/configuration.md`, 13.2 (the forge's charters); jig's
  `domain/tasks.md`, section 4, jig's `protocol/configuration.md`, 3.1
  and jig's `domain/engine.md`, 17.4 (a batch's LLM choices checked
  against its charter's models); jig's `domain/core.md`, 7.1 (a plan's
  text in the repository's design).
- **Builds:** temper's `change` and change procedure (a producer's
  charter, an LLM choice per role); jig's check of them.
- **Configures:** the designer's charter; the coordinator's charter:
  the cut, the families' batches, revisions, the close-out; each
  change's LLM choices, gpt-6.1-sol on smith's direct backend (N20).
- **Walk:** steps 1 to 6 without surveys, questions or early starts
  (each designer starts once S has landed), and 22; with M2, the goal
  then lands.

### M4. Surveys as sub-agents

- **Executes:** `reference-code-production.md`, section 5, step 2.
- **Design:** smith's `protocol/programs.md`, section 2, settling its
  section 9's open question, and smith's `domain/run.md`, 5.3: a program
  backend's sub-agents, each child a program session of its own at its
  LLM choice.
- **Builds:** smith, alone; the coordinator's and the designer's charters
  name sub-agents.
- **Walk:** a run on Claude Code asking inspect-only sub-agents side by
  side; in the case, steps 2 and 5's survey, once M3's walk stands.

### M5. Questions, and holds passed up

- **Executes:** `reference-code-production.md`, sections 8 (a change
  relays its agents' questions) and 9 (holds escalated to G and passed
  up to the person), and 13's wakes on them.
- **Design:** `agent.md`, 8.2 and `forge.md`, 8.1 (a change's questions;
  the Message family for its producer, repairs and resolutions);
  `agent.md`, 4.3.1 (`questions`); temper's `../protocol/configuration.md`,
  13.2 (the foreman's charter) and section 13 (the `questions` setting
  and its bound).
- **Builds:** temper's change procedure: the relay, a foreman per
  question, the bound, an answer held until a dependency lands; a
  change's open questions in `ctl`.
- **Configures:** the foreman's charter; the coordinator's charter:
  answer, amend, a new change, ask the person, holds; waking on each
  question at once (N8).
- **Walk:** steps 9, 11, 14, 15 and 18; 5's question once M3's walk
  stands.

### M6. Early starts

- **Executes:** `reference-code-production.md`, section 7, and section
  6, step 1's kept start.
- **Design:** `worker.md`, 4.1 and section 5 (a checkout from g's tip
  with its dependencies' pushed heads merged in); `forge.md`, section 8
  (the early start, its once rule, its refusal on a conflict, an
  amendment refused for dropping a dependency started from); `landing.md`,
  4.1 (a dependent started kept); jig's `domain/connectors.md`, 6.1 (a
  procedure that starts kept).
- **Builds:** jig's kept start; temper's change procedure and the
  worker's checkout.
- **Configures:** the coding agent's charter: push as it goes (N14).
- **Walk:** steps 8, 10, 13 and 17; 4's designers kept on S once M3's
  walk stands.

### M7. Jobs, the merge job, catching up

- **Executes:** `reference-code-production.md`, sections 6, step 5 (a
  merge into g refused for a conflict) and 11 (catching up);
  `reference-landing.md`, sections 3, step 6 (L's update in conflict)
  and 5.
- **Design:** jobs are designed and not built (jig's `domain/core.md`,
  3.2, `domain/tasks.md`, 5.3.1, `domain/engine.md`, 7.5 and
  `domain/hosts.md`, 6.9; `worker.md`, 4.4). The merge job is not
  designed: `landing.md`, 5.3, `forge.md`, 8.4 and section 10, temper's
  `../protocol/tooling.md`, sections 1 and 4, and
  `../protocol/configuration.md`, section 13 (`merge_job`); nor a
  catch-up change into g (`forge.md`, section 10 and 11.1), the merge
  job its producer (`agent.md`, 4.3.1).
- **Builds, in two parts:** M7a, jobs in jig and temper's worker, from
  M0, as already designed; M7b, the merge job and its three uses: a
  merge into g and a catch-up (after M1), L's update (after M2).
- **Configures:** `merge_job`, the project's kind that merges and
  recounts its counted baselines; the coordinator's charter:
  catch-ups, landing news paced at most once in 30 minutes (N23).
- **Walk:** steps 12, 16 and 21; `reference-landing.md`, section 9's
  first story (L's update conflicts, settled by the merge job and a
  resolution on g).

### M8. Tiers and verdicts on main

- **Executes:** `reference-landing.md`, section 6 (main's tiers, the
  verdict, L living to cleared): `after-landing.md`, sections 2 to 6,
  designed and not built, with what of sections 10 and 11 they need.
- **Design:** `after-landing.md`, sections 1 and 3: a tier names the
  branches it follows, main's follow main (N5). The rest is designed:
  its families (temper's `../protocol/store.md`), its groups in `ctl`
  (`../client/tools.md`), skein's `Read::Artifact` and `Read::Commits`.
- **Builds:** temper's tiers, runs, verdicts and a change living to
  cleared; skein's reads.
- **Configures:** main's tiers, the slow suites; the verification bound,
  two hours.
- **Walk:** a chat's small fix into main (`reference-landing.md`, section
  7), verified: green clears it; a red reruns, blames, bisects. Then,
  with M2, step 25 and 28's L given its result at cleared.

### M9. Reds to the goal; g's tier

- **Executes:** `reference-code-production.md`, section 10 (g's tier,
  its reds as news, triage, a repair into g while merges go on);
  `reference-landing.md`, section 6 (a red to its goal, triage, a repair
  into main naming it, cleared by it): `after-landing.md`, section 7.
- **Design:** `after-landing.md`, sections 1 to 3 (a goal branch's tier,
  judging no landing, its reds as news); `agent.md`, 4.3.1 and
  `after-landing.md`, section 7 (`repairs`).
- **Builds:** temper's g's tier, a red's news to its goal, `repairs`.
- **Configures:** g's tier (the gate without its slowest suites,
  following goal branches, at most once per 20 minutes); the
  coordinator's charter: triage along the way and after landing, notes
  on what it learns.
- **Walk:** steps 20 and 25 to 27; `reference-landing.md`, section 9's
  third story.

### M10. The case

- **Executes:** what is left: `reference-code-production.md`, section 14
  whole, as one reference profile, its limits sized for a goal of a
  hundred changes (N21: temper's `../protocol/configuration.md`, 13.4
  and section 7, `agent.md`, 4.3.1, jig's `domain/engine.md`, 17.9).
- **Walk:** rung C (`v0.1.0.md`, 2.1 and section 7): `reference-case.md`,
  acts 1 to 5, act by act, then whole; both plans; opened once rung B2
  passes.
- **Done** when C passes and the three drafts' statuses are empty; the
  drafts' own close-out follows (`AGENTS.md`, section 9).

## 4. Order, and what runs side by side

| Wave | Opens when | Side by side |
|---|---|---|
| 0 | now | M0 |
| 1 | M0 is done | M1; M4; M7a; M8 |
| 2 | M1 is done | M2; M3; M5; M6; M7b into g |
| 3 | M2 and M8 are done | M9; M7b for L |
| 4 | every one is done, and B2 passes | M10 |

- **Up to five at once,** in wave 2, each a plan of its own in fab.
- **The longest chain** is M0, M1, M2, M9, M10; M8 joins it if it runs
  late, so it starts in wave 1.
- **No shared design milestone** (the user, 2026-11-02). Where
  milestones extend one section, each adds its own part, and a designer
  that meets another milestone's design there builds on it as it lands:
  `agent.md`, 4.3.1, `change`'s parameters (M1, M2, M3, M5, M7, M9);
  `forge.md`, 8.1 (M1, M2, M3, M5); temper's
  `../protocol/configuration.md`, section 13 (M3, M5, M7, M9); the
  coordinator's charter (all but M8).
- **Walks go one at a time:** they share the held logins, one live run
  at a time (`v0.1.0.md`, 2.2), so they queue where builds run side by
  side.

## 5. The case, step by step

| Step | Milestones | Step | Milestones |
|---|---|---|---|
| 1 | M1 | 15 | M3, M5 |
| 2 | M4 | 16 | M7 |
| 3 | M1, M3 | 17 | M6 |
| 4 | M3, M6 | 18 | M5 |
| 5 | M4, M5 | 19 | M1 |
| 6 | M1, M3 | 20 | M9 |
| 7 | M1 | 21 | M7 |
| 8 | M6 | 22 | M3 |
| 9 | M5 | 23 | M2 |
| 10 | M6 | 24 | M2 |
| 11 | M5, M6 | 25 | M8 |
| 12 | M7 | 26 | M9 |
| 13 | M6 | 27 | M8, M9 |
| 14 | M1, M5 | 28 | M2, M8 |

## 6. Left out

- **The drafts' Later and open questions** (`reference-code-production.md`,
  sections 16 and 17; `reference-landing.md`, sections 11 and 12): the
  numbers reopen them (`notes.md`, section 5).
- **A red of no live goal filed as a finding** (`reference-landing.md`,
  section 6): findings are not built, and the case has none.
- **`after-landing.md`, sections 8 and 9,** trailers and what the forge
  shows: the reference needs neither (`notes.md`, section 6).
- **The web client's pages** for any of this: the walks are `ctl`'s;
  pages are rung W's.
- **After the case:** fabrig's own drafts carried out by temper in place
  of fab, on fabrig's forge, GitHub, a rung behind
  (`v0.1.0.md`, 2.2 and section 6). It gets a brief of its own once M10
  passes.

## 7. Open questions

- **Who carries out the playground's close-out** (M0; the director's
  open work).
- **Real backends from M1's walk,** or fakes until M10? `v0.1.0.md`,
  2.1 has real backends lead, and the held logins make every walk wait
  its turn.
