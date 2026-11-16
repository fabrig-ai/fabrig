# Using temper

Draft, 2026-11-16, from a session with the user that day. How a person
actually starts using temper and goes on using it, over the first ten
minutes, the first day, the first month and at scale, for a new project
and an existing one, and what stops them on the day v0.1.0 is out. The
user: what's missing is "putting the pieces together in a coherent UX
story that can span different use cases", from pairing with an agent to
"full factory with milestones like fabrig". Its probe is the user's own
case: "I want to use temper to develop vsr at scale, in the same way I
develop temper itself" (`whole-story.md`, 5.2). It meets a market too:
"somebody smart enough to 'get it' and that could progressively climb the
ladder of automated programming". It frames `whole-story.md`, which says
what each surface holds, by when and why a person meets each part, and
adds what no draft has: the journeys, the two doors in, and the seams
between parts that each work. It wins where they differ until its plans
fold it in. Its milestones are `using-temper.execution.md`. The user
answered its open questions the same day (section 13), added the
repository's `.temper/` (section 15), and asked for the second door from
another machine first (section 16).

## 1. In one page

- **The parts are built; the journeys aren't** (section 2). Nearly every
  milestone of the whole story has landed. Its two stories, E on a box and
  V the pilot, haven't started, and no recorded journey goes from the
  download to a person's own change landed.
- **One ladder** (section 3): a person climbs by handing over bigger
  things, a change, a goal, a milestone, an area, and reading coarser
  summaries of them, a diff, a headline, a strip, a Now. Five verbs stay
  the same at every rung: Start, Answer, Look, Tell, Tune.
- **People and paths are dimensions, not rungs:** a colleague joins at any
  rung, and a rule by path keeps part of a tree lower than the rest (3.3).
- **Each rung is earned, offered and reversible** (3.4).
- **Two doors** (section 4): temper's own terminal and web, and the
  person's own coding agent driving `temper ctl`, which is how fabrig is
  developed today. Both reach the same drafts, goals and needs.
- **The second door works from another machine** (section 16): `temper
  ctl` and the terminal reach a deployment at its public address, signed
  in by the person's browser. The person tells their agent one line, where
  agents already look, its `AGENTS.md`: hand temper work with `temper
  ctl`, and read `temper help` first. It is the first thing to build.
- **A repository's `.temper/` holds everything temper needs to drive it**
  (section 15): its checks, its jobs, its known reds, and temper's guide,
  `.temper/AGENTS.md`, which adds to the repository's own `AGENTS.md` and
  wins over it for temper's agents. The repository says what its code
  needs; the store says what its people decide.
- **Five journeys** (section 5): the try; an existing repository's first
  day, at Pair; a new project from a sentence, starting with a design
  session; the climb over a month; and the factory, which is the pilot.
- **The starting rung is the project's:** an existing repository starts at
  Pair, a new one with a design session (5.3).
- **The terminal** is where a person directs, in sittings, beside their
  own coding agent rather than in its place (section 6).
- **Simple at scale** is six rules, the inbox first (section 7).
- **Release day and the pilot,** step by step: what a person meets and
  where it stops (sections 8 and 9).
- **The market** (section 10): developers who already live in a coding
  agent and have met its ceiling.
- **The gaps** (section 11) are mostly seams: adoption that reads the
  repository and never says in one place whether a change can pass, a project's checks and
  toolchain that only an operator can set, and a deployment the person
  can't reach away from the machine it runs on. v0.1.0 needs the second
  door from another machine, the newcomer's reading, `.temper/`, the
  readiness card, the review reaching the person and a new project's
  foundation, then the journeys recorded. Teaching temper a repository
  comes after.

## 2. Where things stand

As read on main at `cd05436198`, 2026-11-16, and again at `02b5a4c045`
that afternoon.

- **Landed:** the whole story's pages, its rules by path, standing roles,
  follows and the terminal's launchpad, design session and talking with a
  role (`whole-story.execution.md`, section 3); Ask temper's opening, its
  ways as pictures, its views of a path, a way, a proposal and a charter,
  and its memory (`ask-temper.execution.md`); the welcome and the
  playground (`../client/pages/welcome.md`); a draft landed in a
  repository starting a goal (`../domain/forge/drafts.md`).
- **Since the first read:** Home for a person's H1, A1, B1, N1, M1, F1 and
  F2 have landed; its M2 and M3, temper's milestones and the pages', have
  not started (`home-for-a-person.execution.md`). A person can hand temper
  a branch they pushed, `temper ctl change open`, and temper lands it as
  any change (`mechanical-changes.md`; `../domain/forge/change/opened.md`).
- **Not started:** E and V1 to V5 (`whole-story.execution.md`, section 3);
  Ask temper's M3b; configuring's rung D; the release's file, checksums
  and fresh-machine walk (`../release.status.md`).
- **Not walked:** most landed milestones' walks and render checks still
  follow. Ten journeys are recorded at `5b6698daa7`, three on the web and
  seven in the terminal, all on fakes (`../client/web/journeys.md`, section 9;
  `../client/tui/journeys.md`, section 9). None goes from the download to
  a person's own repository, their change, its approval and its landing.
  Standing roles have never run on a real project.
- **So the work now is the person's path through the parts,** and its
  proof, more than new parts.

## 3. One ladder

### 3.1 The size of what a person hands over

| Rung | The person hands over | and reads | It needs them for | On the forge | They say |
|---|---|---|---|---|---|
| 1, Pair | one change at a time | its diff, in the forge's review | every approval | a pull request | "nothing reaches main until I've approved it" |
| 2, Delegate | changes, several at once | what landed, on Work | questions alone | pull requests that temper merges | "agents review, and code lands it" |
| 3, Draft | a goal, from a design session | its report's headline | the draft's questions, a held goal | an issue | "a draft becomes a goal, landed whole" |
| 4, Plan | a milestone, from a large draft | its strip | its goals' questions, a held goal | its goals' issues alone (`milestones.md`, section 2) | "big work is a milestone, and each goal lands when done" |
| 5, Own | an area, or a follow, to a standing role | its owner's Now, and the brief | what passes what reaches them | an owner, as a CODEOWNERS file names one, in words | "Prover owns the tests; Porter the port" |

- **Each rung is the one below at a larger size:** a goal is a set of
  changes and a milestone a set of goals (`milestones.md`, section 2); an
  area is what its owner's goals and changes keep. Nothing below goes
  away: a project with milestones still asks for one change in a chat.
- **fabrig.ai's examples, by size:** Pair is rung 1, Stock rung 2, At
  scale rungs 3 and 4, and standing roles rung 5 (`whole-story.md`,
  section 3). Team is no rung (3.3).
- **What a person reads grows coarser as they climb:** a diff, a headline,
  a strip, a Now, the brief. Each is a line above the one below it and
  opens to it, so a person reads at the size they handed over and opens
  the next only when something is off.

### 3.2 Five verbs, at every rung

| Verb | What the person does | On the web and the phone | In the terminal | Through their own agent |
|---|---|---|---|---|
| Start | hands over work of their rung's size | Change; "+ New draft or change"; "Start goal" on a landed draft | `/change`, `/draft` then `/goal` | `temper ctl change open` for a branch it pushed, `chat start`, `goal start`, `draft ready` |
| Answer | decides what reaches them | Needs you, one tap on a phone | `n`, `/needs` | `temper ctl inbox`, `task answer` |
| Look | sees where things stand | Home's brief, Work, Team | the launchpad, `/brief` | `temper ctl task show` |
| Tell | says what's off | Tell temper | `/tell` | a finding filed through `temper ctl` (`standing-roles.md`, section 8) |
| Tune | changes how work runs | Ask temper, How work runs | `/ask` | `temper ctl proposal decide` |

- **The nouns grow, the verbs don't.** A person who has started a change,
  answered a need, read Home, told temper something and asked it to work
  differently has learned temper. What follows is the same acts on bigger
  things.
- **Talking with a role** is Start and Look at an area's size: `/talk
  NAME` asks an owner about its area and gives it standing instructions
  (`standing-roles.md`, section 6).
- **No page or command is added for them:** the rail and the launchpad
  already offer each (`surfaces.md`, sections 2 and 11.1). They are how the
  help, the site and Ask temper explain temper.

### 3.3 People and paths

- **People join at any rung:** as a signer at rung 1 or 2, Team's
  sign-offs (`whole-story.md`, section 3); as a member who starts goals at
  rung 3; as an area's owner at rung 5. Team lists people and roles alike
  (`surfaces.md`, section 6).
- **Part of a tree can stay lower:** a rule by path
  (`what-reaches-you.md`, section 7) keeps vsr's protocol core at rung 1,
  the person signing off each change to it, while the rest of vsr lands at
  rung 4. A project isn't at a rung; each part of it is, and How work runs
  says so in its headline: "Draft first, with the protocol core asking
  you" (`surfaces.md`, section 7).
- **That is how "mixed is ordinary"** (`surfaces.md`, section 12) reads to
  a person: trust given per path, as to a new colleague.

### 3.4 Climbing

- **A rung is offered when the facts earn it,** never chosen up front: How
  work runs offers it as a proposal the person may make (`surfaces.md`,
  section 12), and nothing changes until they accept. Code counts; where
  an offer needs judgement, the agent in the conversation makes it, by its
  charter's words.

| From | To | Offered when | The offer |
|---|---|---|---|
| 1 | 2, on some paths | a run of approvals given unchanged on changes touching only those paths | "Let agents land changes to tests/ and docs/" |
| 1 or 2 | 3 | a request too large for one change, as the chat's agent judges it, or several changes started on one subject | the chat's "Make it a draft?", opening a design session |
| 3 | 4 | a draft whose parts can land apart | "Start it as a milestone" (section 13) |
| 2 to 4 | 5 | the person triaging findings or reds of one area, again and again | "Hand tests to a role", Team's act (`surfaces.md`, section 6) |
| any | lower | a red after landing traced to a kind of change agents landed alone | "Ask you again for changes like this": a rule by path moving the item back to asking, composed as Stop asking's change is, in reverse (`what-reaches-you.md`, section 5). New |

- **Down is as easy as up:** a rule by path that asks again, from that
  offer or from Ask temper in the person's words, or "Give the area back"
  (`surfaces.md`, section 6), each one proposal. A person who
  climbs too fast climbs back without losing anything.
- **Nobody needs to know the next rung exists.** A person at Pair meets
  rung 2 as an offer about their own tests, not as a mode to read about.

## 4. Two doors

- **temper's own:** the terminal (`temper`, bare), the web and the phone
  (`whole-story.md`, sections 6 and 7).
- **The person's own agent:** a coding agent in the person's checkout
  drives `temper ctl` and reads `temper help`, both written for agents
  (`agent-docs.md`), as any party outside temper does (`whole-story.md`,
  section 10). This is how fabrig is developed today: drafts written with
  an agent in sessions outside temper and landed in the repository as
  documents; goals started from them through `temper ctl`, "Read the draft
  at PATH and carry out its request"; and temper producing, checking and
  landing them (`whole-story.md`, 5.3).
- **They meet at the same objects.** A draft landed in a repository and
  said ready (`../domain/forge/drafts.md`) is on Work's Drafts whichever
  door wrote it, with "Start goal"; a goal started through `temper ctl` is
  on Home and Work as one started by `/goal` is; its questions reach Needs
  you, and the phone, either way.
- **Who comes through which:** a person who lives in a coding agent comes
  through theirs first. They keep their tool for what they do by hand, and
  hand temper what they would otherwise watch: work that should run
  without them and land without breaking main. temper's terminal is for the
  person who wants temper to keep the conversation, its sittings and its
  draft. Neither is lesser: fabrig itself has come through the second door
  for weeks.
- **One habit carries over:** the sessions that develop fabrig are woken by
  mail, timers and watches, and each owns an area; a standing role is that
  habit inside temper (`standing-roles.md`). A person moves an area through
  the door when they want temper to own its waking, its Now and its budget.
- **The agent's own work lands through temper too:** a branch it pushed
  becomes a change with `temper ctl change open`, checked, reviewed and
  landed in turn like any other change, with no goal
  (`mechanical-changes.md`, section 1). That is the second door's rung 1:
  the person keeps writing code with their agent, and temper keeps main
  green.
- **What the second door lacks:** a way to reach a deployment that isn't on
  the agent's machine, since `temper ctl` signs in on loopback
  (`../client/ctl/README.md`, section 6), and the words that point the
  agent at it: section 16 (section 11, G7).

## 5. The journeys

Each is a person's steps and what it shows of fabrig.ai's line, "Say how
you work. Agents do the rest. Code holds the line." Section 8 says where
each stops today.

### 5.1 J0, the try: ten minutes

1. The site's one line downloads `temper` and runs `temper box`. The
   terminal shows the box starting in front of its services, then its
   welcome and its sign-in link, which the person opens in a browser on
   that machine; the terminal never opens it (`../client/tui/box.md`).
2. "Try the playground": the Demo project on fakes, its suggested requests
   answered, and "Add CSV export to widget" delegated, checked, reviewed
   and landed (`first-run-projects.md`). It is not v0.1.0's rung A, which
   runs on real backends (`v0.1.0.md`, 2.1).
3. "Set up temper →": Forge, Model, Ready (`first-run-projects.md`,
   section 4).

It shows the line held: a change produced, checked by code, reviewed by
an agent and merged at the commit that was checked, with no merge tool an
agent's. It shows nothing of the person's own code.

### 5.2 J1, an existing repository: the first day

A maintainer of a repository on GitHub, with its CI, who works with a
coding agent every day.

1. **Set up:** `temper box`; Forge, GitHub, by installing temper's App on
   the repository, which needs the organization's admin; Model, their
   Claude or ChatGPT plan, Claude's signed in through the terminal the box
   lends (`../client/pages/welcome.md`).
2. **Adopt:** Ready makes the project and, on GitHub, adopts nothing, so
   Home offers what the App reaches (`first-run-projects.md`, section 4).
3. **temper says whether a first change can pass,** before any work, on
   one card. Adoption already warns of a read-only installation and of
   protection it couldn't read, and says what branch rules mean for
   landing (`../domain/forge/adoption.md`); the card gathers those and a
   few reads more: main's last CI run; the workflows that run on a pull
   request; branch protection that would refuse temper's merge; a reviewer
   identity for You approve; whether the repository has an `AGENTS.md`, and
   what its `.temper/` holds, a delivery check, `.temper/pre-pr`, above all
   (`../domain/agent/charter.md`, section 4; section 15). Each line says
   what to do. A read that failed, a CI that has never run and a file it
   couldn't fetch are "unknown", never "ready" or "absent". Whether a
   worker holds the repository's toolchain it can't know until G4, and
   says so. New (G2).
4. **temper learns the repository:** the card's `.temper/` line opens
   `temper help repository`, and the person, or their own agent, writes the
   delivery check from it, a script running the repository's own build and
   tests. A repository without one has no checks before a delivery (smith's
   `domain/run.md`, 8.1), and its CI is the only gate. After v0.1.0,
   "Teach temper this repository" writes it for them (G2): a stock first
   change, offered first on Home, whose agent reads the CI workflows, the
   build files and the docs and writes `.temper/pre-pr` and
   `.temper/AGENTS.md`, and nothing outside `.temper/`. It is a pull
   request like any, so the person's first review is of temper learning
   their code, a harmless change that walks the whole loop once.
5. **Its way:** Ask temper's opening offers the ways as pictures
   (`stock-ways.md`, section 2), You approve first for a repository that
   has code (the user, section 13).
6. **A first change:** in the terminal, `/change Fix #123: --json drops
   null fields`. The chat hands it to a change; its steps show a line each;
   the person can leave.
7. **Their review:** the forge asks them for it, so GitHub's own mail and
   app reach their phone with no route to the box (G5). They ask for a
   change there; the change repairs from their words
   (`../domain/forge/change/gates.md`, 1); they approve; temper merges at
   that commit.
8. **The second hour:** three issues at once. They land in turn through the
   queue, each brought up to date and checked again, while the person works
   on something else. Home says what landed.

It shows agents doing the rest of a change, and work in parallel never
breaking main, which a coding agent alone in a terminal doesn't give.

### 5.3 J2, a new project: from a sentence

1. **A project and an empty repository:** on a box's own Forgejo, "Make an
   empty one" (`../domain/forge/adoption.md`, section 5); on GitHub the
   person makes it there and adopts it.
2. **A design session first:** `/draft A command-line tool that keeps my
   dotfiles in sync across machines, in Rust.` The drafter asks one
   question at a time: the language and toolchain, what is in and out of a
   first version, how it is tested and released (`surfaces.md`, 11.2). Its
   draft is the project's foundation: what it is, its layout, the rules its
   `AGENTS.md` holds, its build and test commands, its CI workflow, its
   `.temper/`, and its first features, each a part that lands apart.
3. **The foundation's goal:** `/goal` starts it. Its first change replaces
   the starter workflow, which only echoes success, with the real build and
   tests, and writes `AGENTS.md` and `.temper/pre-pr`; every later change
   passes them (G3). Since it changes `.temper/`, it waits for the person's
   approval in every stock way (15.4), so on GitHub the person must be a
   reviewer the forge counts (`../domain/forge/change/gates.md`, section
   1), as in J1.
4. **The features:** a goal each, or one milestone (rung 4).

**The starting rung is the project's, not the person's:** a repository
with no code has nothing to approve line by line but its foundation, and
its design is what matters, so a new project starts at rung 3; an existing one starts at rung
1, since its code is someone's. It shows "say how you work" and "say what
to build" at once.

### 5.4 J3, the climb: the first month

1. **Week one, at Pair:** after a run of unchanged approvals on tests, How
   work runs offers to let them land on their own (3.4); accepted, Needs
   you shrinks to the rest.
2. **Week two, a draft:** a request runs long; the chat offers a design
   session; the first goal lands whole overnight, if the deployment stays
   up overnight. A box on a laptop stops when its lid closes, so the first
   goal that runs for hours is when a person moves temper to a machine that
   stays on (G5).
3. **Week three, Away:** an evening off; Decided for you in the morning,
   one of them overturned (`what-reaches-you.md`, sections 3 and 6).
4. **Week four, an owner:** the same kind of red reaches the person three
   times; Team offers to hand tests to a role; they hand it to Prover from
   the stock with a weekly budget, and talk to it once, to give it a rule
   (`standing-roles.md`, sections 6 and 10).

It shows the ladder climbed by evidence, each step reversible.

### 5.5 J4, the factory

The user's own, on vsr, vsr-rs and cowsql: section 9.

## 6. The terminal, in practice

- **For sittings in which a person directs:** a design session, a talk with
  a role, a chat that asks for a change, Ask temper. Not for watching: once
  work starts it goes on unattended, and the web and the phone say where it
  stands (`tui.md`, section 2).
- **Beside the person's coding agent, not in its place.** temper's terminal
  edits no checkout, and its agents work on the forge (`tui.md`, section
  1). A person keeps their coding agent for what they do by hand and at
  once, and temper for what should run without them. Their agent can hand
  temper work too (section 4).
- **A sitting at rung 1,** illustrative:

  ```
   pocket: 2 changes landed overnight; csv-export waits on your review.
                                               the status agent · 07:42
   Where you left off
     1  pocket · a chat · --json drops nulls · waiting on your review
   Needs you
     ✓  pocket · csv-export · Review on GitHub ↗
  ❯ /change Fix #131: the config path ignores XDG_CONFIG_HOME
  ● Live · pocket · Claude Opus · high                  1 needs you · n to walk
  ```

  `n` walks to the review on the forge; the new change starts; the person
  closes the terminal.
- **A sitting at rung 4 or 5,** illustrative: the brief across projects;
  `n` walks two needs, one a protocol-core sign-off; Enter on "vsr · a
  design session · store phase 4 · 2 open points" resumes it, the draft in
  the side pane, the drafter's next question numbered; `/goal`, or a
  milestone (section 13); `/talk prover` "why has the fuzz tier been red
  since yesterday?", and "from now on, run the fuzz smoke on every change
  to src/io" becomes a proposal of Prover's rules (`surfaces.md`, 11.3).
  Half an hour, a few times a week.
- **On another machine's deployment,** the terminal reaches an engine on
  loopback alone today (`../client/tui/README.md`, section 1), so a person
  whose temper runs on a server forwards its port. Once the second door
  works from another machine, it opens at the deployment's public address,
  signed in by the person's browser, as `temper ctl` does (section 16).
- **Run in a checkout,** `temper` opens on the project holding that
  repository (`tui.md`, section 3), so the person's two tools meet in one
  directory. Not built.

## 7. Simple at scale

1. **One inbox.** Needs you is the only page that asks anything; every
   other page may go unread. The measure of temper's UX at any size is how
   many cards wait there, and whether each is answered in one tap, with
   its why (`what-reaches-you.md`, section 5).
2. **The verbs stay; the nouns grow** (3.2).
3. **Read at the size you handed over** (3.1), each summary a line above
   the next and opening to it.
4. **Nothing to set up front:** a project starts on a stock way, and every
   part of a page shows once it has content (`surfaces.md`, section 12;
   `to-begin.md`).
5. **Earned, offered, reversible** (3.4), per path (3.3).
6. **The same objects, through every surface and every door** (section 4),
   so that what a person learns on one is true on the others.

## 8. Release day

What a person meets on main today, step by step, and the first place it
stops. Each stop is a gap of section 11.

| Step | What the person meets | Where it stops |
|---|---|---|
| The download | the site's line; `temper box` brings the terminal up, then the browser's welcome | no release is published: its file, checksums and fresh-machine walk are unbuilt (`../release.status.md`); a checkout's install script installs `temper-box` alone (G8) |
| The try | the playground: Demo, its scripted requests, a change landed | nothing, on fakes |
| A model | ChatGPT's sign-in, or Claude's through Claude Code in the terminal the box lends | with no terminal to lend, a command typed by hand |
| A forge | Forgejo on this machine, or GitHub's App and its installation | the help's GitHub walk adopts into a project it never makes (`../../help/github.md`, section 4) (G8) |
| Adoption | the repository adopted, its protection, merge styles, collaborators and workflows read | nothing says whether a first change can pass: main's CI, the workers' toolchain, a delivery check, a reviewer identity for You approve (G2); no page says what `.temper/` holds (G11) |
| A way | Ask temper's three pictures, "Use this", a proposal accepted | a project's gates and tiers are the deployment's document, so neither Ask temper nor a page sets them, and a change of them is a restart (`../protocol/configuration/engine.md`) (G4) |
| A first change | `/change`, or Change on the web; its steps; Needs you | the workers build only what their machine holds, and nothing says so before a run fails (G4); an approval waiting sends no push, and the forge doesn't ask for the review (G5) |
| A new project | "Make an empty one" on Forgejo; on GitHub, made there and adopted | the starter workflow echoes success; no build, test, `AGENTS.md` or delivery check is written (G3) |
| Their own agent | `temper help`, `temper ctl` signed in, `change open` for a branch it pushed | on the deployment's machine alone, since `temper ctl` refuses an address off loopback, and no page gives the person the words for their agent (G7) |
| A design session | `/draft`, the draft beside it, `/goal` | built, and recorded on fakes |
| A milestone | none yet | jig's part landed; temper's milestones and the pages' not started (`home-for-a-person.execution.md`, M1 to M3) |
| An owner | Team's "Hand an area to a role", `/talk` | built, and never run on a real project (G1) |
| The phone | Home, Needs you, Ask; push, once an owner has set it up over HTTPS | a box on loopback is out of a phone's reach; no mail or digest (G5) |
| The overnight goal | a goal from a draft, landing whole | a box stops with its laptop; moving temper to a machine that stays on is an operator's setup from nothing (`../../help/server.md`) (G5) |
| Reading about it | temper's README and help | the README says Forgejo alone and ChatGPT later (`../../../README.md`) (G8) |

## 9. The pilot, as fabrig develops temper

### 9.1 vsr today

Read from its repository on GitHub, 2026-11-16:

- **A C11 library on autotools,** with a `DESIGN.md`, a protocol document
  and an adapter reference, and no `AGENTS.md`.
- **Its CI** (`.github/workflows/ci.yml`) on Ubuntu 24.04 runners: clang 18
  and gcc with sanitizers, an analysis job with clang-tidy, cppcheck and
  valgrind, a fuzz smoke test and coverage.
- **Its I/O layer needs Linux 6.18 or newer at run time,** as its
  development document says, and its ring tests were last run by hand on a
  machine with 7.2, as its handover of 2026-09-30 says.
- **It is developed in agent sessions that hand over by documents:**
  `docs/handover/` holds a brief per session and per module, and three
  branches were left unverified by agents paused at wrap-up.

That is the picture temper replaces, as it replaced fabrig's own: a
handover becomes a goal's report and a role's Now; a branch left half done
becomes a change that lands or is held with its reason.

### 9.2 What each habit becomes

| Today, by hand, for temper or for vsr | For vsr, in temper | State |
|---|---|---|
| a design session with an agent, outside temper, from a laptop or a phone | `/draft` in the terminal, or Draft on the web | built; not used on a real project |
| the draft landed as a document, said ready | `temper ctl draft ready`, then Work's Drafts | built; ready is said through `temper ctl` alone |
| an execution file's milestones, each a goal, started in order on the person's approval | a milestone, its goals waiting on each other (`milestones.md`, section 6) | not started (`home-for-a-person.execution.md`, M2) |
| sessions watching goals and answering what they ask as the person would | the coordinator's wakes; Away and Decided for you | built |
| the landing queue | temper's landing | built and in daily use |
| sessions owning what follows landing: reds, tiers, budgets | tiers after landing, the triager, Prover | built; a project's tiers are the deployment's document |
| a report per area and one roll-up the person reads on a phone | Team's Nows; Home's brief across projects; the phone | built |
| sessions woken by mail, timers and watches | a standing role's wakes | built; unused |
| the person's rules, in files the sessions read | How work runs; a role's rules; the repository's `AGENTS.md` | built |
| a machine with the right kernel, run by hand | a worker holding vsr's toolchain, and a tier placed there | absent (G4) |

### 9.3 Its first week, practically

**Day 0, an hour, mostly the user's own acts:**

1. **Where:** dev.fabrig.ai, a project per repository (`whole-story.md`,
   5.2), beside fabrig's own. temper's GitHub App on cowsql's
   organization, installed by its admin.
2. **Workers:** vsr's build tools on every worker, since a run is placed by
   its models and logins alone (jig's `domain/engine/fleet.md`), and
   Linux 6.18 or newer for its ring tests, which a worker on Debian 13's
   kernel, 6.12, lacks. Until the hardware is there, CI's jobs on GitHub's runners are the
   gate, and the ring tests stay out of temper.
3. **Adopt** cowsql/vsr into the project vsr.
4. **vsr's `.temper/`** (5.2, step 4; section 15): `pre-pr` from its CI,
   `./bootstrap`, configure with `--enable-werror` and sanitizers, `make
   check` and `make lint`; and `.temper/AGENTS.md` from its `DESIGN.md`,
   its development document and its handovers' working agreement. The
   user's own agent writes them, or Teach temper once it is built, and the
   user reviews them. None of vsr's own files changes.
5. **Its way,** through Ask temper: "Draft first; changes to include/vsr.h,
   src/protocol.*, src/transition.* and the protocol document wait for my
   sign-off; the rest lands after CI." A rule by path, proposed and
   accepted (`what-reaches-you.md`, section 7).
6. **Its tiers:** long fuzzing, MSan and coverage after landing. Today
   they are entries of the deployment's document, so an operator's edit and
   a restart of dev.fabrig.ai (G4); once gates and tiers are `.temper/`'s,
   a change to it (15.2).
7. **Prover:** Team, "Hand an area to a role": tests, fuzzing and triage,
   from the stock, with a weekly budget.

**Then, each day:**

- **A design session** from the handover's next steps, `/draft store
  phase 4: the three store fixes and the recovery fuzzer`, ends in a
  draft; its parts become a milestone once milestones land, and goals
  started in order until then. Each lands on its own.
- **The three branches left half done** become goals, each named in its
  draft: finish, check, land, or say why not.
- **Mornings, on the phone:** the brief across temper and vsr; a
  protocol-core sign-off, by "Review on GitHub ↗", with no push for it
  today (G5).
- **`/talk prover`** when the fuzz tier is red.
- **What the user writes by hand** with their own agent, on their laptop,
  is pushed and handed over with `temper ctl change open`, and lands
  through the same queue (section 16).

**Week two:** vsr-rs, made on GitHub and adopted into its own project; a
follow of vsr owned by Porter, its check vsr's tests built against the
port (`follows.md`, section 8). **Week three:** cowsql's design sessions.

### 9.4 What the user would miss

- **The terminal and `temper ctl` on a laptop** reach dev.fabrig.ai only
  over a forwarded port, until the second door works from another machine
  (section 16). Then the laptop's `temper` follows dev.fabrig.ai's builds,
  since a client of another version is refused (16.3).
- **A design session from the phone** is the web's chat with its draft,
  not the terminal's.
- **Every tier or gate added** is a restart that waits for the user (G4).
- **The kernel:** vsr's ring tests need hardware the user decides on.
- **Spend:** vsr's goals share the deployment's logins with fabrig's own,
  so its weekly budget is set against theirs.

This is V1 to V5 (`whole-story.execution.md`, section 3), with what stands
between them and today named.

## 10. Meeting the market

- **Who:** developers who work with a coding agent every day and have met
  its ceiling: one agent in one terminal, watched; every change reviewed by
  them; agents in parallel colliding on main. Smart enough to see that the
  work has moved from writing code to checking it, landing it and deciding
  what to build.
- **What pulls them,** in their words: run several agents without breaking
  main; stop watching; keep my process, as settings I can read; on my own
  machine, with my own plan, open source.
- **Four moments that convince,** each a journey's:

| When | What they see | Its journey |
|---|---|---|
| ten minutes | a change lands by itself, checked at the commit that merged | J0 |
| the first hour | three of their own issues land in turn while they do something else | J1 |
| the first week | a design session's goal lands whole overnight | J3 |
| the first month | they stop reviewing tests, because temper earned it | J3 |

- **How they come in:** through their own agent first (sections 4 and 16),
  and the playground; temper's terminal once they want design sessions kept.
- **What is said plainly,** as fabrig.ai's hard questions are: what a
  change costs on their plan, and that agents in parallel reach a plan's
  limits sooner; that a box stops with its laptop; that temper is as good
  as their CI; that a GitHub App needs an organization's admin.
- **What they climb toward** is section 9: temper built by temper, readable
  on dev.fabrig.ai (`first-release.md`, section 3).

## 11. The gaps

Each by the name it was first given. For v0.1.0, in the order to start
them: G7, G8 and G11, small and apart from each other; then G2's
readiness card, G5 and G3; then G1, once those have landed. The pilot
needs G4 as well. After v0.1.0: G2's teaching, G6, G9 and G10 (the
user, 2026-11-16: teaching temper a repository is "nice to have, not
blocking for v0.1.0").

- **G1, the journeys walked.** E and V haven't started, and no journey
  goes from the download to a person's own change landed. J1 and J2 are
  recorded on fakes as the client's journeys are, then walked on real
  backends after landing; V is the user's own week (section 12).
- **G2, an adopted repository that can take a first change.** Adoption
  reads its branch rules, merge styles, collaborators and workflow names
  (`../domain/forge/adoption.md`, sections 4 and 7), and warns of a
  read-only installation and of protection it couldn't read, but nothing
  says in one place whether a first change can pass. For v0.1.0, a
  readiness card on the project's Home, from those reads and warnings and
  a few reads more, each line with its act, and "unknown" where a read
  failed. After it, "Teach temper this repository", a stock first change
  that writes `.temper/pre-pr` and `.temper/AGENTS.md` from the
  repository's own CI and docs, the agent judging and the person
  approving.
- **G3, a new project's foundation.** "Make an empty one" ends at a README
  and a workflow that echoes success. A design session for a new project,
  whose goal writes its build, tests, CI, `AGENTS.md` and `.temper/`; no
  catalogue of scaffolds, since a drafter and a goal write one to fit.
- **G4, a project's checks and toolchain as the project's.** Its gates and
  tiers are entries of the deployment's document, read at start
  (`../protocol/configuration/engine.md`), and a run is placed by its
  models and logins alone, so every worker must hold every project's
  toolchain. Gates and tiers changed without a restart, from `.temper/`
  or as settings (15.2); and a project naming what its runs need, placed
  on the workers that hold it, with a run that can't be placed saying so.
- **G5, reached away from the machine.** A box is on loopback; a waiting
  approval sends no push; there is no mail. The forge asking the person
  for their review, so the forge's own notifications reach them; a
  sign-off's push; and the help's way from a box to a machine that stays
  on. The terminal's and `temper ctl`'s reach are G7's.
- **G6, a large draft as a milestone,** from Work's draft card or `/goal`,
  once milestones land (section 13).
- **G7, the second door from another machine:** `temper ctl` and the
  terminal reaching a deployment at its public address, signed in by the
  person's browser, and the words a person gives their agent (section 16).
- **G8, what the newcomer reads:** temper's README, which still says
  Forgejo alone and ChatGPT later; the install script; the help's GitHub
  walk, which never makes the project it adopts into.
- **G9, the climb's offers** (3.4): How work runs' suggestions, each a rule
  over counts code keeps (`surfaces.md`, section 12), and the chat's "Make
  it a draft?".
- **G10, a change's cost said before it starts,** on the person's plan, so
  that starting three at once is a choice made knowing it.
- **G11, the repository's `.temper/`** (section 15): no page says what it
  holds, temper's runs read no guide of temper's own, and a change to it
  reaches the person no more than any other change.

## 12. Its acceptance

Plans use the focused and fuzzy suites alone; walks come after landing.

- **Each gap's milestone** carries focused and fuzzy tests in the worlds
  the parts it touches already use, with the fake forge and the fake model:
  the readiness card's lines from adoption's reads; the first change's
  brief and its pull request; a gate or tier changed and read by the next
  landing without a restart; a run placed on a worker holding what its
  project needs, and one that can't be, saying so; the forge's review
  request; the sign-off's push.
- **The second door from another machine** (section 16): `temper ctl` and
  the terminal signed in at a fake deployment's `https://` address, over
  TLS with a test root; an approval asked from another host, its code
  typed on the page, which marks what the client says of itself; sustained
  anonymous asks never keeping the person out; the terminal's credential
  and kept requests never sent to another origin; a client of another
  version refused, naming the deployment's.
- **`.temper/`** (section 15): `.temper/AGENTS.md` in a run's prompt after
  the root's guide, under its line; a reviewer's guides read from the
  branch the change lands into; a change touching `.temper/` asking
  the person in each stock way; `temper help repository`'s section held to
  the help's own checks.
- **J1 and J2 as journeys** (`../client/web/journeys.md`;
  `../client/tui/journeys.md`), recorded where a fake box reaches their
  states: J1 from the welcome to a first change landed after its review,
  J2 from an empty repository to the foundation's goal started. Recording
  is generation, which a task runs. The user takes their frames.
- **After landing:** J0 to J3 walked on real backends from the download,
  the second door walked from a laptop against dev.fabrig.ai, and V, the
  user's own week on vsr.

## 13. Open questions

The user answered the first seven on 2026-11-16, and their answers stand
as decided.

- **An existing repository's first way:** You approve first for a
  repository with code; Agents review stays the stock for a project made
  empty. Decided: "makes sense".
- **"Teach temper this repository":** offered, first on the project's
  Home, since it is a pull request the person reviews. Decided, and after
  v0.1.0: "nice to have, not blocking for v0.1.0".
- **A milestone from a draft in the client:** through `temper ctl` by the
  person's own scripts for v0.1.0 (`milestones.md`, section 6), with the
  recipe a help page; the client's act later, as `milestones.md`, section
  11 has it. Decided.
- **The second door:** one help page, not a packaged skill. Decided. The
  user asked how it reaches a person's agent, for every kind of handover
  and not milestones alone: by one line in the `AGENTS.md` the agent
  already reads, pointing it at `temper ctl` and `temper help` (16.4).
- **The pilot's deployment:** dev.fabrig.ai, as `whole-story.md`, 5.2 has
  it. Decided.
- **vsr's ring tests:** out of temper until the user decides on the
  hardware. Decided.
- **From a box to a server:** the help's setup for v0.1.0; moving a box's
  store later. Decided.

Open, each with its default:

- **A change to `.temper/`:** asks the person in every stock way, or lands
  as its way has it. Default: it asks, by a rule by path the stock
  carries, which a project may narrow or drop (15.4).
- **Where gates and tiers live:** in `.temper/`, read from the branch a
  change lands into, or in the store, as the kinds that
  `../protocol/configuration/engine.md` plans for them. Default:
  `.temper/`, since they describe the repository's CI and change with it,
  settled in a draft of their own beside `config.md` (15.2).
- **A browser's approval between hosts:** jig leaves it open, defaulting
  to loopback alone (jig's `protocol/client/README.md`, section 12).
  Default: enabled at a deployment's public `https://` address, the code
  typed on the page rather than carried in its address (16.3).

## 14. Later

- **A deployment serving its clients' builds,** so that a laptop's
  `temper` follows a deployment that redeploys often (16.3).
- **The project from the checkout:** `temper ctl` taking a checkout's
  remote for `--project` and `--repository`.
- **"Connect your agent" on the web:** the sign-in command and the line
  for `AGENTS.md`, ready to copy (16.2).
- **A skill or a server for coding agents,** generated from the help,
  never written beside it (16.4).
- **A project's charter overlay under `.temper/`** (`stock-charters.md`,
  section 11).
- **A person's backlog as their first requests:** the forge's open issues
  offered when the project is new.
- **A design session in the terminal's form on a phone.**
- **Goals across projects,** as `whole-story.md`, section 12 keeps.
- **Approving in place,** as `surfaces.md`, section 16 keeps.

## 15. The repository's `.temper/`

The user, 2026-11-16: "we should have .temper/ repo folder where
everything temper-specific lives (probably an AGENTS.md as well, which
augments or overrides the top-level repo AGENTS.md), .temper/ would also
hold delivery checks etc, really anything temper needs to drive the
repo."

### 15.1 The repository's, and the store's

- **The repository says what its code needs; the store says what its
  people decide.** What changes when the code changes lives in
  `.temper/`: how a change is checked, what temper's jobs run, what
  temper's agents should know, what a run needs. It lands as a change, so
  a change adding a test suite adds its check in the same commit. Who
  decides, what reaches whom and what may be spent stays in the store, the
  people's (`configuring.md`, section 3): ways and rules by path, roles
  and their areas, follows, budgets, members. So no change to a
  repository changes who decides.
- **Each part is read from a tree of its own:**
  - a producer's guide and its delivery check, from its own checkout, so
    a change may change both, as today (`../domain/agent/charter.md`,
    section 4). The check is no security boundary but an early catch
    inside the run that can fix it; the line is the forge's CI and the
    gates (smith's `domain/run.md`, 8.1);
  - the merge job, from the merge in progress, as today
    (`../protocol/tooling.md`, 4.1);
  - a reviewer's guides, from the branch the change lands into, so that
    no change instructs its own review (15.3). New;
  - gates and tiers, once they are `.temper/`'s, from the branch the
    change lands into, so a change is held to the gates it found. How a
    train holding a change to them cuts over is their own draft's
    (`using-temper.execution.md`, section 2).
- **One folder,** so a maintainer sees temper's whole footprint at a
  glance, and a repository that leaves temper deletes one folder. What
  lives outside it is the repository's own, its CI and its `AGENTS.md`,
  which temper reads and never needs.

### 15.2 What it holds

| Path | What | Today |
|---|---|---|
| `.temper/pre-pr` | the delivery check, run on a run's checkout before anything is pushed (`../domain/agent/charter.md`, section 4) | built |
| `.temper/merge` | the merge job, run from the merge in progress (`../protocol/tooling.md`, 4.1) | built; its path named by the deployment's document, `merge_job` (`../protocol/configuration/engine.md`) |
| `.temper/close` | the close job, a goal's close step | built; likewise, `close_job` |
| `.temper/AGENTS.md` | temper's guide (15.3) | new (G11) |
| gates and tiers | the forge's checks a change waits for, the workflows run after landing, and their words | the deployment's document (`../protocol/configuration/engine.md`); here by a draft of their own (section 13) |
| what its runs need | a toolchain, a kernel, a machine, which placement reads | absent (G4) |
| a charter overlay | the project's own rules over the stock charters | later (`stock-charters.md`, section 11) |

A goal's known reds are no file of the folder: the goal keeps them as
records, and a delivery's check reads them from the worker
(`../domain/forge/reds.md`).

### 15.3 temper's guide

- **Read after the repository's own.** A run's prompt holds each workspace
  directory's guide, `AGENTS.md` (smith's `domain/run.md`, 3.3). At the
  repository's root, `.temper/AGENTS.md` follows the root's, under a line
  saying it is temper's and wins where the two differ.
- **A reviewer reads both as the branch the change lands into has them,**
  not as the change has them, so a change can't tell its own reviewer what
  to pass. A producer, a repair and a resolution read their checkout's, as
  today.
- **The repository's `AGENTS.md` stays for every agent,** the person's own
  among them; `.temper/AGENTS.md` is for temper's alone: how to deliver
  here, what never to run, what a goal's report should say.
- **Either may be absent:** a repository with no `AGENTS.md`, as vsr's, has
  temper's alone; one with neither runs on the charters.

### 15.4 A change to it reaches the person

- **A change to `.temper/` changes what temper's agents are told and what
  its jobs run,** so every stock way asks the person for one, by a rule by
  path the stock carries (`what-reaches-you.md`, section 7). A project
  narrows or drops it as any rule, as one whose agents tune their own
  checks every day would.
- **It is no security boundary.** A check's code may live outside the
  folder, as fabrig's `.temper/pre-pr` runs `tools/pre-pr/`, and so may
  the CI workflows and tests it mirrors. The line stays the forge's CI and
  the gates. A person who wants a check's code to ask them too adds its
  paths to the same rule, and `temper help repository` says so.

### 15.5 What a person does with it

- **J1:** the readiness card's `.temper/` line opens `temper help
  repository`, which says what each file is and shows one; the person or
  their agent writes `.temper/pre-pr` (5.2, step 4).
- **J2:** the foundation's goal writes it, and the person approves it
  (5.3).
- **The pilot:** vsr's `.temper/` is written and reviewed on day 0, and
  none of vsr's own files changes (9.3).
- **fabrig's own repository** has `.temper/pre-pr`, `merge` and `close`
  already, and its `AGENTS.md` serves every agent.

## 16. The second door from another machine

The user, 2026-11-16, as the first thing to start: "temper (ctl) working
against a remote repo". Read here as a person's own agent, in a checkout
on their machine, driving a temper that runs on another.

### 16.1 Today

- **`temper ctl` refuses an address off loopback,** "the engine's address
  must be on loopback", and speaks HTTP alone. It signs in by a box's
  printed link or a service's file (`../client/ctl/README.md`, section 6).
- **The terminal is on loopback alone too** (`../client/tui/README.md`,
  section 1), and signs in by the link or by the person's browser
  approving it on the approval page (`../client/README.md`, 4.1;
  `../client/pages/README.md`, section 2). It keeps one directory for
  whatever address it opens.
- **A browser's approval between hosts** is jig's open question, defaulting
  to loopback alone (jig's `protocol/client/README.md`, section 12;
  `protocol/client/sign-in.md`).
- **So fabrig's own sessions** run on dev.fabrig.ai's machine and reach it
  on loopback, and a person on a laptop forwards a port.

### 16.2 The person's steps

Illustrative; its milestone settles the forms.

1. **Sign in, once per deployment:** `temper ctl ~/.temper/dev sign-in
   --address https://dev.fabrig.ai` prints the approval page's address
   and a code, and waits. The person, signed in on the web, opens the
   page, types the code, reads what the deployment knows of the ask, and
   approves; ctl prints who it is signed in as. `temper
   https://dev.fabrig.ai` signs the terminal in the same way, into that
   deployment's own place (16.3).
2. **Tell their agent, once:** a line in the `AGENTS.md` the agent already
   reads, the repository's or their own:

   ```
   ## temper
   Work on this repository lands through temper at https://dev.fabrig.ai.
   Hand it work with `temper ctl ~/.temper/dev`; read `temper help` first.
   ```

3. **Then the agent works as section 4 says:** a branch it pushed, `change
   open`; a draft landed, `draft ready` and `goal start`; the person's
   needs, `inbox` and `task answer`; and waits on what it handed over.

### 16.3 Its rules

- **`https://` at the deployment's public address,** whatever serves its
  TLS, with the machine's roots, as temper's GitHub connection does
  (`../protocol/configuration/engine.md`); `http://` stays for loopback
  alone.
- **A credential goes only to the deployment that gave it.** ctl's
  directory holds one deployment already, and every call names it. The
  terminal keeps a credential and its kept requests per deployment, by
  its origin, and opening another origin never sends the old one's
  credential or requests.
- **Who may ask, and how often, before a deployment enables it in
  public.** An ask is anonymous today, and a deployment holds four at
  once for ten minutes each (jig's `protocol/client/sign-in.md`), so four
  strangers' asks keep the person out. Between hosts, asking, polling and
  guessing are bounded so that sustained anonymous asks can't, and U6's
  acceptance shows it.
- **What the page can vouch for.** The page tells apart what the
  deployment attests, the ask's time and, behind a proxy it trusts, the
  address it came from, from what the client says of itself, its name and
  machine, shown as the client's own words. The code is typed on the
  page, never carried in its address, so a person who didn't just ask
  approves nothing.
- **An approval guards against strangers, not against the person's own
  agent.** A browser's cookie and a bearer carry the same secret (jig's
  `protocol/client/default-http.md`), so a program holding the person's
  credential can approve as a browser would. Whether approving should
  need more than that credential is U6's to settle; the draft promises no
  more than it gives.
- **The sign-in is the person's,** with their roles: their agent acts as
  them, as fabrig's sessions act as the deployment's first owner today. A
  service, made by `party make` and signed in by `sign-in --service
  FILE`, stays for an agent that should act as itself, at the same
  addresses.
- **A client of another version is refused,** with the versions the
  deployment serves (`redeploys.md`, section 8): ctl and the terminal say
  which build the deployment runs, and the person upgrades. A deployment
  on main redeploys often, so a laptop's `temper` follows its builds
  (section 14).
- **The terminal and ctl together:** both take the same addresses and sign
  in the same way, so a person learns it once.

### 16.4 The words for the agent

- **One line, where agents already look,** as the user suggested: point
  the agents at `temper ctl`. A coding agent reads its `AGENTS.md` before
  anything; the line points it at `temper ctl` and `temper help`, and the
  help does the rest. `temper help` is already where an agent starts
  (`../../help/index.md`), and it ships in the binary the agent runs, so
  its words are always that build's.
- **A walk for the driver:** `temper help` gains the walk an agent follows
  to hand work over: signing in, a pushed branch opened as a change, a
  draft said ready and its goal started, needs answered, and waits. It is
  the second door's help page (section 13), written once for every
  handover, not for milestones alone. A milestone from an execution file
  joins it with `temper ctl milestone`, which
  `home-for-a-person.execution.md`'s M2 brings.
- **No skill or server packaged for agents in v0.1.0:** either would copy
  what the help says and drift from the binary. One generated from the
  help can come later (section 14).
