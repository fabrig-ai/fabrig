# The whole story

Draft, 2026-11-08, from a session with the user that day. temper as a
person's one front end: what a person does with it and where, at every
point of fabrig.ai's spectrum from one person approving every change to
areas owned by standing agents; the kinds of agent session that carry
it; and the parts still missing, each with a draft of its own. It is the
top of four drafts written with it, `standing-roles.md`,
`what-reaches-you.md`, `follows.md` and `surfaces.md`, and it frames
`tui.md`, `ask-temper.md` and `stock-ways.md`, which keep their own. Its
mockups are section 9's. The user: "I'll be the first user of temper and
ideally temper would be my only frontend", and they asked for all of it
by v0.1.0 (`v0.1.0.md`, section 5). Its milestones are
`whole-story.execution.md`. Reasons: `notes.md`, section 12.

## 1. In one page

- **A person does three things:** direct, decide and know. Agents and
  code do the rest (section 2).
- **Two surfaces, two jobs.** The web says where things stand and what
  needs the person, on a laptop or a phone. The terminal is where they
  work with agents, one thing at a time (sections 6 and 7; `tui.md`,
  section 2).
- **One set of pages, filled as the person hands over more.** Pair,
  Team, Stock, At scale and standing roles are settings, so a page shows
  only what the project's way uses (section 3).
- **Three kinds of agent session:** calls, conversations, and standing
  roles between them, which own an area for the project's life and wake
  in short sessions that end by writing their Now (section 4;
  `standing-roles.md`).
- **What reaches a person is a setting:** agents decide and tell them,
  ask them, or wait for them, by kind and by path, and every need says
  why it reached them and how to stop it (`what-reaches-you.md`).
- **Work can follow a source:** each landing in one project makes a
  change in another, owned by a role (`follows.md`).
- **Three stories** hold it together: a newcomer at Pair, a pilot on
  cowsql's vsr with its Rust port, and a project's own development
  moving in (section 5).
- **The pages are drawn:** the web, the phone and the terminal at three
  points of the spectrum. They are the design's target where they differ
  from the greenfield mock (section 9; `surfaces.md`).
- **All of it by v0.1.0,** in tracks that run side by side
  (`whole-story.execution.md`).

## 2. Three acts, two surfaces

| Act | What the person does | Where |
|---|---|---|
| Direct | says what to build, and how work runs | the terminal: a design session that ends in a draft and a goal, talking with a role, Ask temper; the web: How work runs, changed by proposal |
| Decide | answers what only they should | Needs you, on a phone in one tap: the recommended choice first, why it reached them, a way to stop being asked; Decided for you, to overturn |
| Know | sees where things stand, and says what's off | Home's brief across projects; Work, each goal's report in its agent's words; Team, each owner's Now; Tell temper, a finding sent to its area's owner |

- **The same objects in the same words** on both surfaces: what one
  decides, the other shows decided (`tui.md`, section 2).
- **Setting up is the web's:** How work runs and Deployment. Working
  with agents is the terminal's. The phone is for deciding and knowing.
- **Agents never touch the person's machine.** They work on the forge
  (`tui.md`, section 1).

## 3. The spectrum

fabrig.ai's four examples are settings, not modes; a fifth step hands
areas to standing roles.

| Step | The person says | What appears |
|---|---|---|
| Pair | "nothing reaches main until I've approved it" | a chat makes a change; Needs you holds approvals, each one tap from the forge's review; Work lists chats and their changes; Home's brief; no Team, no Away, no Decided for you |
| Team | "agents review, one of us signs off; payments/ needs two of us" | sign-offs by path, and who can still sign, in Needs you; Team lists people and the area each owns; Decided for you and Away, since agents now decide wording and retries |
| Stock | "what do you do by default?" | agents review and land; Needs you holds questions alone |
| At scale | "a draft becomes a goal, landed whole" | drafts and goals in Work |
| Standing roles | "Prover owns the tests, Porter the port" | standing roles beside people on Team; follows; the brief rolled up from owners' Nows |

- **Each step adds one thing,** and the step before offers it: How work
  runs suggests the next when it is earned, such as "let agents land
  test-only changes" or "hand an area to a role".
- **Which rail items and sections each step shows** is
  `surfaces.md`, section 12.
- **Derived, never a profile.** No page is drawn per step: each part
  of a page shows by a rule over the project's settings and what exists
  (`surfaces.md`, section 12). A project between two steps, or mixing
  them, gets exactly its parts, so the three points drawn are
  illustrations, not views.
- **Moving along it is a change of settings,** proposed and accepted
  like any (`configuring.md`, section 4): new work follows the new
  settings, running work keeps what it started under, and the pages
  show both side by side until the old work is done.
- **Drawn at three points:** Pair, Team, and at scale with roles
  (section 9).

## 4. Three kinds of agent session

- **Calls:** one job, one run, then done: producers, reviewers,
  foremen, surveys. Code writes their brief. A person meets what they
  made, never them.
- **Conversations:** a person is there: a chat, a design session, Ask
  temper, talking with a role. They last as long as the person stays,
  and end in something kept: a draft, a proposal, a rule.
- **Standing roles,** between the two. A role lives for the project's
  life, with its rules, its area, its Now, what wakes it and a budget
  per period. Its sessions are short: each covers one stretch of work,
  goes on while its cache is warm, is ended by code, and closes with
  the role writing its Now. The next starts fresh from its rules, its
  Now and what woke it (`standing-roles.md`, sections 2 and 3).
- **Why not long sessions:** on the long sessions we run by hand to
  develop fabrig, 74% of their own spend on 2026-11-07 went on wakes
  that came after the prompt cache had expired, each writing the whole
  context again. Compaction keeps a session small but loses what nobody
  sees. A written Now is cheap to read, and the person can read it too.
- **Why not calls alone:** an area needs an owner across goals: a red
  after landing with no live goal, a finding, a question about a port.
  Today one triager per project holds that (`../domain/findings.md`,
  section 6).
- **The goal's coordinator is one already,** bounded by its goal: fresh
  at each wake, with instructions for why it woke (`per-wake.md`,
  section 1).

## 5. Three stories

### 5.1 A newcomer, at Pair

1. They download temper, run `temper box`, try the playground, then
   adopt their own repository.
2. Ask temper's opening offers the ways as pictures; they pick You
   approve (`stock-ways.md`, section 2).
3. In the terminal, or a chat on the web, they ask for a change. The
   chat delegates it; an agent produces it, CI checks it, and failures
   are repaired.
4. On their phone, Needs you shows the change waiting for their
   approval, with CI's state. One tap opens the
   forge's review, where they approve (`forge-identity.md`, section 8).
5. Home says what landed; Work lists the chat and its change.
6. Once enough has landed, How work runs suggests the next step: test
   changes landing on their own. It changes nothing until accepted.

Drawn as the Pair set (section 9).

### 5.2 A pilot: vsr, vsr-rs and cowsql

The user's first project that isn't temper, on dev.fabrig.ai, where
fabrig's own goals already run as its development moves in
(`v0.1.0.md`, section 6 keeps the whole move for after the tag), one
project each:

- **vsr** (github.com/cowsql/vsr): a C11 library for Viewstamped
  Replication, with a deterministic core that does no I/O and an
  io_uring I/O layer; its CI runs ASan and UBSan, `-Werror`,
  clang-tidy, cppcheck, a fuzz smoke test and coverage. The aim is a
  solid C implementation. Its way is Draft first; changes to its
  protocol core (`include/vsr.h`, `src/protocol.*`, `src/transition.*`
  and its protocol document) wait for the person's sign-off (`what-reaches-you.md`, section 7); a role,
  Prover, owns its tests and its triage; its tiers after landing run
  long fuzzing, MSan and coverage.
- **vsr-rs:** a repository of its own, vsr ported to Rust line by line,
  continuously. unsafe is fine: its guarantees come from vsr's C tests
  and QA, replayed on the Rust build. A role, Porter, owns it, and it
  follows vsr (`follows.md`, section 8). It will replicate skein-kv, so
  fabrig comes to depend on it, a relation shown where both are
  projects of one deployment (`follows.md`, section 7).
- **cowsql:** its raft-based replication replaced by vsr, designed with
  the person in design sessions, then carried out as a goal.

The person's week: design sessions in the terminal, for cowsql's move
and vsr's open questions; the phone in the morning, for the brief, a
protocol-core question with its recommended answer, a budget to raise;
Team, for Prover's and Porter's Nows; How work runs, to tune what
reaches them. Drawn as the at-scale set (section 9).

What it needs beyond today: rules by path, standing roles, follows,
and workers whose machine builds the project: vsr needs a C toolchain,
and Linux 6.18 or newer for its io_uring tests (`../domain/worker/jobs.md`
, for what a job runs). Goals whose changes span two projects'
repositories are later (the user, 2026-11-08): until then, work on vsr
and on cowsql runs as a goal per repository.

### 5.3 A project's own development moving in

fabrig is developed today by a pipeline and long sessions run by hand.
Each thing they do has its place in temper:

| By hand today | In temper | Its draft |
|---|---|---|
| plans cut by a designer and run by coding agents, questions through a foreman | a goal, its coordinator, its changes | `reference-code-production.md`, `goal-as-designer.md`, `per-wake.md` |
| a landing queue, tiers after landing | landing, tiers | `archive/reference-landing.md` |
| long sessions owning areas, woken by their mail, timers and watches | standing roles and what wakes them | `standing-roles.md` |
| messages between sessions, each with a class: now, by a time, for information | messages between tasks, and wake policies | jig's `domain/tasks/messages.md`, 3 |
| a status written for the person, rolled up | Home's brief, from the owners' Nows | `surfaces.md`, section 3 |
| decisions taken in the person's place, listed for review | Decided for you | `what-reaches-you.md`, section 6 |
| design sessions with the person | design sessions in the terminal, then Draft first | `surfaces.md`, 11.2 |
| a deployment's operations, its logins' care | outside temper | section 10 |

The whole move follows v0.1.0 (`v0.1.0.md`, section 6). These drafts
make the pages and the mechanisms it needs, and the pilot (5.2) uses
them first.

## 6. The web

- **The rail:** Home · Needs you · Work · Team · Ask temper, then How
  work runs, then Away · Deployment · the person (`surfaces.md`,
  section 2).
- **What changes from the greenfield mock:** Goals, Changes and the
  chats that started work fold into Work; Team is new; Settings splits
  into How work runs, the project's process, and Deployment, its
  accounts, forges, workers and members; Home and Needs you work across
  projects; Chats leave the rail, since a conversation lives with what
  it is about and happens in the terminal.
- **The pages:** Home (`surfaces.md`, section 3), Needs you (4), Work
  (5), Team (6), How work runs (7), Deployment (8), Ask temper and Tell
  temper (9), and the phone's four tabs (10).

## 7. The terminal

- **The launchpad:** the brief, where the person left off, what needs
  them, the team, and the commands that start things (`surfaces.md`,
  11.1).
- **A design session:** a chat with the drafter, its draft in the side
  pane, choices numbered, a goal started from the draft with the
  project's ways (`surfaces.md`, 11.2).
- **Talking with a role:** the role's Now at the top; a standing
  instruction becomes one of its rules by proposal; leaving ends its
  session, and it writes its Now (`surfaces.md`, 11.3).
- These revise `tui.md`, section 3, where they differ.

## 8. The parts

| Part | Its draft | Carried out by |
|---|---|---|
| The rail, Home across projects, Needs you, Work, Team, How work runs, Deployment, Tell temper, the phone | `surfaces.md`, sections 2 to 10 | temper/ui |
| The launchpad, design sessions, talking with a role | `surfaces.md`, section 11 | temper/tui |
| The drafter, a design session's charter | `surfaces.md`, 11.2 | temper/ui |
| What reaches a person, Away, Decided for any task, being told | `what-reaches-you.md` | temper/ui |
| Rules by path, enforced at landing | `what-reaches-you.md`, section 7 | temper/migration |
| Standing roles: sessions, Now, wakes, budgets per period, areas and routing | `standing-roles.md` | temper/migration; Team's page temper/ui; talking in the terminal temper/tui |
| Follows, their lag and parity | `follows.md` | temper/migration; lag on the pages temper/ui |
| Ask temper reading work, and its memory | `ask-temper.execution.md`, M2 and M4 | temper/ui |
| Ways and their paths | `stock-ways.md` | temper/ui |
| The pilot's projects and workers | section 5.2 | temper/migration, with the deployment's operators |

## 9. The mockups

Drawn with the user on 2026-11-08. They are the design's target where
they differ from the greenfield mock, as that mock is over
`../client/mockups/` (`../client/web/greenfield/README.md`). Names, numbers
and questions in them are made up. They join the design with the
execution's M0.

- **At scale, with standing roles:** a Sunday morning on one deployment
  with four projects, vsr, vsr-rs, cowsql and fabrig.
  `../client/web/whole-story/`: `index.html` (the shape), `home.html`,
  `needs.html`, `work.html`, `team.html`, `how.html`,
  `phone-home.html`, `phone-need.html`, `phone-tell.html`, with
  `whole-story.css` and `whole-story.js`. `../client/tui/whole-story/`: `launchpad.html`,
  `design-session.html`, `role.html`.
- **Pair:** one person, one project, You approve.
  `../client/web/whole-story/pair/`: `first-run.html`, `home.html`,
  `needs.html`, `work.html`, `how.html`, `phone-home.html`;
  `../client/tui/whole-story/pair/launchpad.html`.
- **Team:** three people, one project, Agents review with sign-offs by
  path. `../client/web/whole-story/team/`: `home.html`, `needs.html`,
  `work.html`, `team.html`, `how.html`, `phone-home.html`;
  `../client/tui/whole-story/team/launchpad.html`.
- **What they bind:** the pages' parts, words, order and look. Where a
  draft and a mockup disagree on a mechanism, the draft wins and the
  mockup is redrawn.

## 10. What stays outside temper, and its door

- **Outside:** a deployment's operations (its machines, backups and
  upgrades) and the care of its logins, which fabrig.ai's Run is to
  take on later; QA as a continuous lab, fabrig.ai's Prove, also later.
- **Their door:** an agent outside temper is a party. Through
  `temper ctl` it files a finding or a need, routed by area like any
  (`standing-roles.md`, section 8), and reads what it must.
- **What only the person can do** for them, a sign-in or a login to
  renew, shows apart in Needs you, while work goes around it
  (`what-reaches-you.md`, section 8).

## 11. v0.1.0

- **All of it is in v0.1.0, by rank** (the user, 2026-11-08): `v0.1.0.md`,
  section 5 lists the part, and each draft's acceptance section says
  its bar. A milestone ranked nice in `whole-story.execution.md`,
  section 3, is built last and may follow v0.1.0.
- **E and V stand beside the ladder,** as the TUI's first pass does
  (`v0.1.0.md`, section 5): neither waits for the rung below it.
- **E, the whole story,** on a box, through `temper ctl`, walked after
  its milestones land, one story gathering each draft's own
  (`standing-roles.md`, section 11; `what-reaches-you.md`, section 10;
  `follows.md`, section 9; `surfaces.md`, section 14):
  - Pair: a change waiting for the person's approval in Needs you, and
    landed once approved on the forge;
  - Team: a change under a path rule waiting for two sign-offs while an
    unrelated change lands;
  - what reaches a person: Away deciding an "asks you" item and listing
    it, and asking with Away off; Stop asking narrowing a rule; a budget
    overrun waiting; a push for the blocking need alone;
  - standing roles: a red routed to its area's role, triaged and
    repaired by a change it makes, its Now written, a later wake
    starting fresh from it, and a rule added by talking with it;
  - follows: a source landing making a follow change, landed with its
    check, and lag shown while one is held.
- **V, the pilot:** vsr, vsr-rs and cowsql adopted on dev.fabrig.ai,
  Prover and Porter running, and vsr-rs keeping up with vsr's landings,
  its check green, through a week of the person's use; its findings are
  findings like any other (section 5.2).
- **The pages and frames:** each recorded as a journey and taken by the
  user, as the ladder's pages are (`v0.1.0.md`, section 9).

## 12. Open questions

- **Being told:** Web Push to an installed web app, or mail, as the
  default (`what-reaches-you.md`, section 11).
- **Goals across projects:** a goal whose changes land in two projects'
  repositories, and whose authority covers both (section 5.2).
- **A role's default model and budget,** per stock way
  (`standing-roles.md`, section 12).
- **Follows across deployments or forges:** later (`follows.md`,
  section 6).

## 13. Later

- **From the mockups' audit** (the user, 2026-11-08), drawn nowhere now:
  raising a role's budget for one week alone; pausing it until a day;
  "Remind me tomorrow"; nudging a person; sign-offs by named people or
  overlapping groups, beyond a role's count; inviting a person from the
  client; suggestions backed by counts; a budget's week starting on a
  set day, and a tier on a schedule; one brief across projects written
  per person; when a login ends, from its provider.

- **Approving in place:** an approval or a sign-off given from Needs you and posted on
  the forge as the person, with a forge credential they signed in with,
  rather than one tap to the forge's own review page (temper's
  `../README.md`, section 8). The forge's page is v0.1.0's (the user,
  2026-11-08).
