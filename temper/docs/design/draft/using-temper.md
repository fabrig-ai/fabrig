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
fold it in. Its milestones are `using-temper.execution.md`.

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
  repository and tells the person nothing, a project's checks and
  toolchain that only an operator can set, and a deployment the person
  can't reach away from the machine it runs on.

## 2. Where things stand

As read on main at `cd05436198`, 2026-11-16.

- **Landed:** the whole story's pages, its rules by path, standing roles,
  follows and the terminal's launchpad, design session and talking with a
  role (`whole-story.execution.md`, section 3); Ask temper's opening, its
  ways as pictures, its views of a path, a way, a proposal and a charter,
  and its memory (`ask-temper.execution.md`); the welcome and the
  playground (`../client/pages/welcome.md`); a draft landed in a
  repository starting a goal (`../domain/forge/drafts.md`).
- **In flight:** milestones and Home for a person
  (`home-for-a-person.execution.md`), none of it on main yet.
- **Not started:** E and V1 to V5 (`whole-story.execution.md`, section 3);
  Ask temper's M3b; configuring's rung D; the release's file, checksums
  and fresh-machine walk (`../release.status.md`).
- **Not walked:** most landed milestones' walks and render checks still
  follow. Seven journeys are recorded, three on the web and four in the
  terminal, all on fakes (`../client/web/journeys.md`, section 9;
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
| 4, Plan | a milestone, from a large draft | its strip | its goals' questions, a held goal | a milestone | "big work is a milestone, and each goal lands when done" |
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
| Start | hands over work of their rung's size | Change; "+ New draft or change"; "Start goal" on a landed draft | `/change`, `/draft` then `/goal` | `temper ctl chat start`, `goal start`, `draft ready` |
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
| any | lower | a red after landing traced to a kind of change agents landed alone | "Ask you again for changes like this", as Keep asking narrows a rule (`what-reaches-you.md`, section 5) |

- **Down is as easy as up:** Keep asking, a rule by path, or "Give the
  area back" (`surfaces.md`, section 6), each one proposal. A person who
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
- **What the second door lacks:** a page that is the words a person gives
  their agent, and a way to reach a deployment that isn't on the agent's
  machine, since `temper ctl` signs in on loopback
  (`../client/ctl/README.md`, section 6) (section 11, G7).

## 5. The journeys

Each is a person's steps and what it shows of fabrig.ai's line, "Say how
you work. Agents do the rest. Code holds the line." Section 8 says where
each stops today.

### 5.1 J0, the try: ten minutes

1. The site's one line downloads `temper` and runs `temper box`. The
   terminal shows the box starting in front of its services, then its
   welcome, and the browser opens on it (`tui.status.md`, *Design
   departs*).
2. "Try the playground": the Demo project on fakes, its suggested requests
   answered, and "Add CSV export to widget" delegated, checked, reviewed
   and landed (`v0.1.0.md`, 2.1, rung A).
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
3. **temper says whether a first change can pass,** before any work. From
   what adoption reads already (`../domain/forge/adoption.md`) and a few
   reads more: main's last CI run; the workflows that run on a pull
   request; branch protection that would refuse temper's merge; a reviewer
   identity for You approve; whether the repository has an `AGENTS.md` and
   a delivery check, `.temper/pre-pr` (`../domain/agent/charter.md`,
   section 4). Each line says what to do. New (G2).
4. **temper learns the repository, as its first change:** "Teach temper
   this repository", offered first. Its agent reads the CI workflows, the
   build files and the docs, and writes the delivery check, which runs the
   repository's own build and tests, and the part of `AGENTS.md` temper's
   agents need. It is a pull request like any, so the person's first review
   is of temper learning their code, a harmless change that walks the whole
   loop once. New (G2).
5. **Its way:** Ask temper's opening offers the ways as pictures
   (`stock-ways.md`, section 2), You approve first for a repository that
   has code (section 13).
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
   `AGENTS.md` holds, its build and test commands, its CI workflow and
   delivery check, and its first features, each a part that lands apart.
3. **The foundation's goal:** `/goal` starts it. Its first change replaces
   the starter workflow, which only echoes success, with the real build and
   tests, and writes `AGENTS.md` and the delivery check; every later change
   passes them (G3).
4. **The features:** a goal each, or one milestone (rung 4).

**The starting rung is the project's, not the person's:** a repository
with no code has nothing to approve line by line, and its design is what
matters, so a new project starts at rung 3; an existing one starts at rung
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
  loopback alone (`../client/tui/README.md`, section 1), so a person whose
  temper runs on a server forwards its port (G5).
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
| Adoption | the repository adopted, its protection, merge styles, collaborators and workflows read | nothing says whether a first change can pass: main's CI, the workers' toolchain, a delivery check, a reviewer identity for You approve (G2) |
| A way | Ask temper's three pictures, "Use this", a proposal accepted | a project's gates and tiers are the deployment's document, so neither Ask temper nor a page sets them, and a change of them is a restart (`../protocol/configuration/engine.md`) (G4) |
| A first change | `/change`, or Change on the web; its steps; Needs you | the workers build only what their machine holds, and nothing says so before a run fails (G4); an approval waiting sends no push, and the forge doesn't ask for the review (G5) |
| A new project | "Make an empty one" on Forgejo; on GitHub, made there and adopted | the starter workflow echoes success; no build, test, `AGENTS.md` or delivery check is written (G3) |
| A design session | `/draft`, the draft beside it, `/goal` | built, and recorded on fakes |
| A milestone | none yet | in flight (`home-for-a-person.execution.md`, M1 to M3) |
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
| an execution file's milestones, each a goal, started in order on the person's approval | a milestone, its goals waiting on each other (`milestones.md`, section 6) | in flight |
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
4. **Teach temper vsr** (5.2, step 4): the first change writes the delivery
   check from its CI, `./bootstrap`, configure with `--enable-werror` and
   sanitizers, `make check` and `make lint`, and an `AGENTS.md` from its
   `DESIGN.md`, its development document and its handovers' working
   agreement. The user reviews it.
5. **Its way,** through Ask temper: "Draft first; changes to include/vsr.h,
   src/protocol.*, src/transition.* and the protocol document wait for my
   sign-off; the rest lands after CI." A rule by path, proposed and
   accepted (`what-reaches-you.md`, section 7).
6. **Its tiers:** long fuzzing, MSan and coverage after landing. Today
   they are entries of the deployment's document, so an operator's edit and
   a restart of dev.fabrig.ai (G4).
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

**Week two:** vsr-rs, made on GitHub and adopted into its own project; a
follow of vsr owned by Porter, its check vsr's tests built against the
port (`follows.md`, section 8). **Week three:** cowsql's design sessions.

### 9.4 What the user would miss

- **The terminal on a laptop** reaches dev.fabrig.ai only over a forwarded
  port (6).
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

- **How they come in:** through their own agent first (section 4), and the
  playground; temper's terminal once they want design sessions kept.
- **What is said plainly,** as fabrig.ai's hard questions are: what a
  change costs on their plan, and that agents in parallel reach a plan's
  limits sooner; that a box stops with its laptop; that temper is as good
  as their CI; that a GitHub App needs an organization's admin.
- **What they climb toward** is section 9: temper built by temper, readable
  on dev.fabrig.ai (`first-release.md`, section 3).

## 11. The gaps

Ranked: the first five are on v0.1.0's happy path, the person's first day
and the pilot; the rest are for the climb.

1. **G1, the journeys walked.** E and V haven't started, and no journey
   goes from the download to a person's own change landed. J1 and J2 are
   recorded on fakes as the client's journeys are, then walked on real
   backends after landing; V is the user's own week (section 12).
2. **G2, an adopted repository that can take a first change.** Adoption
   reads its branch rules, merge styles, collaborators and workflow names
   (`../domain/forge/adoption.md`, sections 4 and 7) and tells the person
   nothing of whether a change can pass. A readiness card on the project's
   Home, from those reads and a few more, each line with its act; and
   "Teach temper this repository", a stock first change that writes the
   delivery check and `AGENTS.md`'s part from the repository's own CI and
   docs, the agent judging and the person approving.
3. **G3, a new project's foundation.** "Make an empty one" ends at a README
   and a workflow that echoes success. A design session for a new project,
   whose goal writes its build, tests, CI, `AGENTS.md` and delivery check;
   no catalogue of scaffolds, since a drafter and a goal write one to fit.
4. **G4, a project's checks and toolchain as the project's.** Its gates and
   tiers are entries of the deployment's document, read at start
   (`../protocol/configuration/engine.md`), and a run is placed by its
   models and logins alone, so every worker must hold every project's
   toolchain. Gates and tiers as settings a proposal changes; and a
   project naming what its runs need, placed on the workers that hold it,
   with a run that can't be placed saying so.
5. **G5, reached away from the machine.** A box is on loopback, and so are
   the terminal's and `temper ctl`'s sign-ins; a waiting approval sends no
   push; there is no mail. The forge asking the person for their review,
   so the forge's own notifications reach them; a sign-off's push; and the
   help's way from a box to a machine that stays on, with the terminal over
   a forwarded port.
6. **G6, a large draft as a milestone,** from Work's draft card or `/goal`,
   once milestones land (section 13).
7. **G7, the second door:** a help page that is the words a person gives
   their coding agent to hand work to temper, and `temper ctl` reaching a
   deployment on another machine (section 4).
8. **G8, what the newcomer reads:** temper's README, which still says
   Forgejo alone and ChatGPT later; the install script; the help's GitHub
   walk, which never makes the project it adopts into.
9. **G9, the climb's offers** (3.4): How work runs' suggestions, each a rule
   over counts code keeps (`surfaces.md`, section 12), and the chat's "Make
   it a draft?".
10. **G10, a change's cost said before it starts,** on the person's plan, so
    that starting three at once is a choice made knowing it.

## 12. Its acceptance

Plans use the focused and fuzzy suites alone; walks come after landing.

- **Each gap's milestone** carries focused and fuzzy tests in the worlds
  the parts it touches already use, with the fake forge and the fake model:
  the readiness card's lines from adoption's reads; the first change's
  brief and its pull request; a gate or tier changed by a proposal and read
  by the next landing without a restart; a run placed on a worker holding
  what its project needs, and one that can't be, saying so; the forge's
  review request; the sign-off's push.
- **J1 and J2 as journeys** (`../client/web/journeys.md`;
  `../client/tui/journeys.md`), recorded where a fake box reaches their
  states: J1 from the welcome to a first change landed after its review,
  J2 from an empty repository to the foundation's goal started. Recording
  is generation, which a task runs. The user takes their frames.
- **After landing:** J0 to J3 walked on real backends from the download,
  and V, the user's own week on vsr.

## 13. Open questions

- **An existing repository's first way:** You approve offered first, or the
  stock's Agents review. Default: You approve first for a repository with
  code; Agents review stays the stock for a project made empty.
- **"Teach temper this repository":** offered, or made at adoption.
  Default: offered, first on the project's Home, since it is a pull request
  the person reviews.
- **A milestone from a draft in the client:** milestones are made through
  `temper ctl` by the person's own scripts (`milestones.md`, section 6).
  Default: so for v0.1.0, with the recipe a help page; the client's act is
  later, as `milestones.md`, section 11 has it.
- **The second door:** a help page, or a packaged skill for coding agents.
  Default: one help page, whose text a person gives their agent.
- **The pilot's deployment:** dev.fabrig.ai beside fabrig's own, or its own.
  Default: dev.fabrig.ai, as `whole-story.md`, 5.2 has it.
- **vsr's ring tests:** a tier on a worker with Linux 6.18 or newer, or kept
  out of temper until there is one. Default: out until the user decides on
  the hardware.
- **From a box to a server:** the help's setup from nothing, or a box's
  store moved. Default: the help for v0.1.0; moving a box's store later.

## 14. Later

- **The terminal reaching a deployment over TLS,** with no forwarded port.
- **A person's backlog as their first requests:** the forge's open issues
  offered when the project is new.
- **A design session in the terminal's form on a phone.**
- **Goals across projects,** as `whole-story.md`, section 12 keeps.
- **Approving in place,** as `surfaces.md`, section 16 keeps.
