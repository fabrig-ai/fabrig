# Standing roles

Draft, 2026-11-08, from a session with the user that day. An agent that owns
an area of a project for the project's life: its rules, its area, its Now,
what wakes it and a budget a week. It works in short sessions and writes its
Now as each one ends. It is the middle kind of `whole-story.md`, section 4's
three kinds of agent session, between a call that runs once and a conversation
a person is in. It adopts jig's `draft/notes-memory.md`, section 4, for a
standing role's Now, as the user took it up on 2026-11-08 for standing roles
alone (that draft's head and status record it). It widens
`../domain/findings.md`, section 6's one triager per project to an owner per
area. It revises `stock-ways.md`, section 2, where a way sets how work starts
and lands and nothing else: a way may now bring standing roles (section 10).
It departs from jig's `domain/tasks.md`, 8.6,
where a charter whose runs resume holds no parts, and from jig's
`domain/engine.md`, 7.4, where the charter alone says whether a run resumes.
It wins where they differ until its plans fold it in. temper/migration carries
out the engine's parts, jig's included, the routing and the sessions; temper/ui
the Team page and a standing role's card (`surfaces.md`, section 6);
temper/tui talking with a standing role in the terminal (`surfaces.md`, 11.3).
Here "a role" in a heading or an example is always a standing role, never a
role of the project's people.

## 1. In one page

- **An owner for an area, for the project's life.** A standing role holds an
  area of a project, said in words. It answers for what happens there:
  findings, reds after landing, the questions its own work asks. A person may
  own an area instead, and Team lists both (section 2).
- **A long life, short sessions.** Each stretch of work is a session. It goes
  on while its work does and its cache is warm, and code ends it. Every run
  writes its Now before it parks, so the session's last run leaves it
  current. The next wake starts fresh from its rules, its Now and what woke
  it (section 3).
- **Its Now is its memory.** A few lines it writes itself: where things stand,
  what it waits for, what it does next. People read it on Team, and Home's
  brief reads the owners' Nows (sections 3 and 7).
- **Routed by a role's judgement** (section 4): the project's triager reads
  what comes in against the areas' words and hands it to its owner, and keeps
  what fits none. Routing by path is later (section 13).
- **Its person** (section 2): who handed it its area requests it, so its
  questions and the decisions it takes in their place reach a person.
- **Bounded** (section 5). It starts goals and changes in its area, answers
  its own work's questions, triages, and proposes changes to its own rules. It
  never accepts its own proposal, and what reaches a person is
  `what-reaches-you.md`'s.
- **Talked with** (section 6), in the terminal or on the web. An instruction
  meant for every case becomes one of its rules, by a proposal accepted in
  place.
- **Parties outside temper** file findings and needs through `temper ctl`,
  routed the same way (section 8).
- **What the engine adds** (section 9): a task that stands, a budget refilled
  each period, sessions, the Now kept with the task, and routing by area.
- **A way may bring standing roles** (section 10). Its acceptance is section
  11's.

## 2. What a standing role is

- **Its rules:** a charter, as every agent has (`../protocol/configuration.md`,
  13.4): instructions for its area's work, in parts by why a run runs
  (`per-wake.md`, section 3), its models and efforts, its tools and its
  documents (jig's `domain/engine.md`, 17.9). They change by a proposal of
  settings, as every setting does (jig's `domain/engine.md`, 17.6).
- **Its area:** what it covers, in words, section 4.
- **Its person:** the person who handed it its area requests it, a party
  (jig's `domain/tasks.md`, section 3). So jig's routes carry its questions,
  escalations and proposals to that person, and the decisions it takes in
  their place are their Decided rows (`what-reaches-you.md`, section 4). When
  that person leaves the project, the project's owners take it over: the
  standing role moves to an owner as its requester, its open questions with
  it (jig's `domain/tasks.md`, section 7).
- **Its Now:** its working notes (jig's `draft/notes-memory.md`, section 4)
  under one name, written by its runs and read whole in every brief. A page at
  most: where things stand, what it waits for and why, what it does next and
  when, what it asked and what it decided.
- **Its wakes:** a wake policy and subscriptions (jig's `domain/tasks.md`, 8.3
  and 8.4) over what concerns its area: landings that touch paths it names, reds
  of its tiers, findings with its labels, its own work's questions and results, a
  person's words, and timers. Porter, the pilot's owner of vsr-rs
  (`whole-story.md`, 5.2), wakes at once on a landing on vsr's main that
  touches `src/` or `include/` and on a red of vsr-rs, every four hours on
  findings labelled `port`, and on Mondays at 06:00 for its report.
- **A budget per period:** an amount a week, carved again each period from
  the project's spend, as a recurring task's is (jig's `domain/authority.md`,
  section 7; jig's `domain/tasks.md`, section 10). What it has left shows on
  Team. Past it, the standing role is held for money and its person is asked,
  since money always waits for a person (`what-reaches-you.md`, section 2).
- **Its report:** its Now at all times, and a report each week (section 7).
- **A person may own an area instead.** An area's owner is a standing role or
  a party.
  Team lists both alike. Handing an area to a role, or back, is a setting
  changed by proposal.
- **The goal's coordinator is one already,** bounded by its goal: it starts
  fresh at each wake, under a wake policy, with its plan in the store
  (`goal-as-designer.md`; `per-wake.md`, section 6). A standing role has the
  same shape, with an area for its scope and the project's life for its own.

## 3. Sessions

- **A session is a stretch of work,** from the wake that opens it to the end
  code decides. Within it, each wake resumes the session's transcript while its
  cache is warm, as a conversation's charter resumes today (jig's
  `domain/engine.md`, 7.4, *Woken again*).
- **Code ends it,** at the first of four:
  - **a quiet span:** no wake for longer than its cache keeps. The span is
    the cache life of its account's provider and model, a little under it:
    an hour where the provider offers one and the account takes it, five
    minutes where that is the provider's default. A model with no prompt
    cache keeps none, so each of its wakes is a fresh run, a session of one;
  - **the context's bound:** a share of its model's window, by default half;
  - **its work closed:** nothing it started is live, and its inbox holds
    nothing it is offered;
  - **a message its parts don't offer:** a message that wakes the task but
    that the session's parts don't cover ends the session, and the next run
    starts fresh with the parts for it.
- **An engine restart doesn't end it** (the user, 2026-11-08). The worker
  and its agent processes keep running and reconnect, so the session goes
  on warm, and what the engine needs to know it again is stored with the
  standing role.
- **Every run writes its Now before it parks.** It is a few lines, cheap to
  write, so whichever run turns out to be the session's last has left it
  current, whatever then ends the session. A run that writes none keeps the
  last Now, and the session's end is a line in its activity.
- **It never finishes.** A standing role's run parks. A run that answers
  finished is committed as parked, its result's words a line in its
  activity, and the standing role goes on (section 9). Only a cancel ends
  it.
- **The next wake starts fresh.** Its brief carries its rules with the parts
  for why it runs (`per-wake.md`, section 3), its Now whole, what woke it and
  its area's state. It carries no transcript: the Now takes the place of a
  transcript's tail (jig's `domain/engine.md`, section 9).
- **Parts and resuming together.** The session's first run reads the parts its
  reasons choose. Later runs in the session resume, and keep the instructions
  the session began with, which keeps the cache (jig's `domain/tasks.md`,
  8.6). They are offered what those parts were for. Any other message keeps
  its place: if it wakes the task, it ends the session (above), and the next
  run reads the parts for it, as `per-wake.md`, section 4 keeps what a run
  isn't offered; if it doesn't, it waits for the next session.
- **Why:** on our measurement, 74% of the long sessions' own spend on
  2026-11-07 went on wakes after the prompt cache had expired, among the long
  sessions we run by hand. Such a wake writes the whole context again, at
  about twenty times a warm wake's cost.
  Compaction loses what nobody chose to keep, and nobody sees what it lost. A
  Now is chosen by the standing role, read by people, and read whole by the next run. A
  fresh run from a small brief is what made the goal's later runs cheap
  (`per-wake.md`, section 1).

## 4. Areas and routing

- **An area** is a project's: a name, what it covers in words, the finding
  label it takes (`area:<name>`, `../domain/findings.md`, section 2) and its
  owner, a setting changed by proposal. Today a project's areas are named per
  repository beside its gates (`../domain/findings.md`, section 9). Words are
  new.
- **A role routes, by judgement** (the user, 2026-11-08): the project's triager
  (`../domain/findings.md`, section 6) reads what comes in against the areas'
  words and its own rules, and hands each to its area's owner by writing that
  label, with why in a line. It is one short run an item, and a project
  changes how it routes by changing words.
- **What fits no area** the triager keeps, and triages itself.
- **What goes to an area's owner:**
  - **A finding,** handed on by the triager, or filed with its area's label,
    which goes straight to the owner with no agent's run. The owner triages it
    as `../domain/findings.md`, section 6's triager does: to a goal, to a new
    goal, to the backlog, or dropped.
  - **A red of no goal's** (`../domain/after-landing.md`, section 7). The
    project's triager is told, as today, and hands it on.
  - **Feedback:** a person's word through Tell temper is a finding, its area
    judged as it is filed (`whole-story.md`, section 2).
  - **Questions** follow the tree, as ever (jig's `domain/tasks.md`, section
    9): a standing role is the requester of what it starts, so its own
    work's questions reach it, and what it can't settle goes on up to its
    person (section 2). Where `what-reaches-you.md`, section 4 lets an agent decide in a
    person's place, the area's owner is that agent for its area.
- **A person owner** gets what is routed to them in Needs you, as a person
  triager is notified today (`../domain/findings.md`, section 6).
- **The project's triager** may be a person or a standing role. Today's
  setting names a task or a role of the project's people
  (`../domain/findings.md`, section 6), a people role, not a standing role; it
  may now name a standing role as well.

## 5. What a standing role may do

- **Its authority** is its task's (jig's `domain/authority.md`, section 3),
  from its project's policy: what it may make, read and spend.
- **It may:**
  - start goals and changes in its area, under its project's way, so that
    they are checked, reviewed and landed as any work is;
  - answer its own work's questions, and decide where `what-reaches-you.md`,
    section 4 lets it;
  - triage the findings and reds routed to it;
  - write its Now and its report;
  - propose changes to its own rules, its wakes, its area's words or its
    budget, as a proposal of settings (jig's `domain/engine.md`, 17.6).
- **It may not:**
  - accept its own proposal: an owner decides it (jig's `domain/engine.md`,
    17.6);
  - change files outside its area. Its changes land under the project's
    rules by path (`what-reaches-you.md`, section 7): one touching paths a
    rule guards waits for that rule's sign-offs, as any change does, and
    its charter's words keep it to its own area in the first place.
    What it needs outside its area it asks of that area's owner, as a
    question or a request for a change;
  - spend past its period's budget;
  - decide what `what-reaches-you.md`, section 2 sends to a person.
- **A merge is never its own.** Its changes land through the project's rules
  as any change does; a merge tool is never an agent's.

## 6. Talking with a standing role

- **A person opens a conversation with it:** `/talk` in the terminal
  (`surfaces.md`, 11.3), "Talk to it" on Team (`surfaces.md`, section 6). The
  person's words wake it and always reach a live run (jig's
  `domain/tasks.md`, 8.3).
- **The talk is a session** (section 3). The person sees its Now at the top,
  and its brief holds its rules, its Now and its area. It resumes while the
  person is there.
- **What lasts goes where it belongs.** An instruction for every case becomes
  a rule: the standing role proposes a change to its own rules, and the person, if an
  owner of the deployment, whose charters are (`configuring.md`, 3.3),
  accepts it in place ("Add it to Porter's rules?"). An instruction for
  the work at hand goes in its Now. A decision becomes a record: an answer, a
  goal, a setting.
- **Leaving ends the session.** The person leaves, the standing role writes
  its Now, and the conversation stays as its activity (jig's
  `domain/engine.md`, 7.2), read on Team, never resumed.
- **Not a chat.** A chat is the person's own and delegates for them. A standing
  role's conversation is its own: its words join its activity, and what it
  starts is its own.

## 7. Reports

- **Its Now on Team** (`surfaces.md`, section 6), with when it was written and
  by which session; and its sessions this week, a line each: when, how long,
  what it did and what it spent.
- **A weekly report,** written at its timer: what landed in its area, what it
  decided, what waits and on whom, what it spent. It is kept with the standing role and
  read on Team.
- **What Home's brief reads.** A standing role's Now is what Home's brief
  reads of its area. The brief itself, its agent and its page are
  `surfaces.md`, section 3's, carried out by temper/ui; this draft changes
  nothing of `../domain/forge.md`, 5.7.

## 8. Parties outside temper

- **Agents outside temper** do work of their own: a deployment's operations,
  the care of its logins, a later lab that proves the code continuously. Each
  sometimes finds something wrong, or needs a person.
- **They file through `temper ctl`,** each a party with a sign-in of its own
  (`../client/tools.md`, section 9):
  - **a finding,** with its kind, its paths or label and its reproduction,
    routed by area like any (section 4);
  - **a need,** a question for a person with its choices and why, sent to
    Needs you as the project's `what-reaches-you.md` says.
- **Such a party files and reads.** It starts no work: what follows a
  finding is its area owner's.

## 9. What the engine adds

Each is a design lag, by its status file.

- **A task that stands** (jig). An agent task made standing is never told
  that nothing is awaited (jig's `domain/tasks.md`, 5.2; jig's
  `domain/engine.md`, 7.4), and ends only when cancelled, as a recurring task
  does (jig's `domain/tasks.md`, section 10). A run of it that answers
  finished is committed as parked (section 3). Two more of jig's rules for
  finite tasks don't hold it: idle with delegates still live, it is not held
  past the stall's bound, since a goal it started may run for days (jig's
  `domain/tasks.md`, 5.5 and section 11); and its tries per class of failure
  count within a session and start again with the next, not over its life
  (jig's `domain/tasks.md`, 5.5).
- **Its person as its requester** (jig and temper). Its requester is a party,
  its person (section 2), so jig's routes for questions, escalations and
  proposals reach a person as they are. Moving it to an owner when its
  person leaves the project is jig's move to a new requester (jig's
  `domain/tasks.md`, section 7), which temper makes when a member leaves.
- **A budget refilled each period** (jig): carved again from the project's
  spend each period, as a recurring task's is, its unspent rest returning to
  the period that funded it (jig's `domain/authority.md`, section 7).
- **Sessions** (jig): resuming within a session and starting fresh across
  them, by section 3's rule; a charter that both resumes and has parts (jig's
  `domain/tasks.md`, 8.6; jig's `domain/engine.md`, 7.4); the quiet span from
  the account's and model's cache life; a session ended by a waking message
  its parts don't offer; a Now written before every park.
- **The Now kept with the task** (jig): a task's working notes, from jig's
  `draft/notes-memory.md`, section 4, whole in every brief, bounded, and read
  by the project's people.
- **Areas, and routing** (temper): areas in words with an owner, as a project's
  setting (`../domain/findings.md`, section 9); the triager's act that hands a
  finding or a red of no goal's to an area's owner, by its label
  (`../domain/findings.md`, sections 2 and 6; `../domain/after-landing.md`,
  section 7).
- **A standing role in the stock** (temper): its charter and its template
  (`../protocol/configuration.md`, 13.4), and its row among the agents temper
  runs (`../domain/agent.md`, section 8).
- **Its pages and acts** (temper's client): Team and a standing role's card,
  and talking with a standing role, on the web and in the terminal
  (`../client/README.md`, 5.3 and section 6); in `temper ctl`
  (`../client/tools.md`, 5.3), `role list`, `role show`, `role talk` (a
  person's words to a standing role, which open or continue its session, its
  answer printed), `finding file` and `need ask`. A rule it proposes is
  decided with `proposal show` and `proposal decide`, as any proposal
  (`configuring.md`, 5.4).

## 10. Stock roles and ways

- **A way may bring standing roles,** revising `stock-ways.md`, section 2,
  as Draft first brings its coordinator and goal template (`stock-ways.md`,
  3.3). Draft first brings a triager, a standing role that routes what comes
  in and owns what has no area, with a small budget a week. A project with no standing role keeps
  its person as the owner of everything, as at Pair.
- **How work runs lists a project's standing roles** with their areas, wakes and
  budgets, beside its way.
- **"Hand an area to a role"** on Team says what the area covers, takes a stock
  standing role's rules or new ones written by asking temper, and sets its
  budget: a proposal an owner accepts. Whoever hands it the area is its
  person (section 2).
- **The stock's standing roles' rules are temper's to write,** as the
  coordinator's are (`../protocol/configuration.md`, 13.4, charter 5): the
  triager first, then the ones the pilot needs, a prover for tests and a
  porter for a follow (`follows.md`, section 2).

## 11. Its acceptance

Plans and their acceptance use focused and fuzzy tests alone; a walk follows
each landing.

- **Focused:**
  - a session's end as a pure function of its last wake, its context, its
    open work and what woke it: a quiet span, the bound, closed work or a
    waking message its parts don't offer ends it, and nothing else;
  - the quiet span taken from the account's and model's cache life, and no
    resume at all for a model with no prompt cache;
  - the next run resumes inside a session and starts fresh after one, its
    brief holding the Now and no transcript; after a session ended by an
    unoffered message, the fresh run reads that message's part;
  - every run writes its Now before it parks, and the session's last Now is
    its last run's;
  - a standing role's run that answers finished is committed as parked, and
    the standing role stays live;
  - routing: a label goes to its area's owner with no agent's run, the
    triager's hand-off reaches the owner it names, with its why, what fits no
    area stays with the triager, a person owner gets a Needs you entry;
  - a standing task is never told that nothing is awaited, is not held past
    the stall's bound while idle with live delegates, counts its tries per
    session, and ends only when cancelled;
  - its questions and Decided rows reach its person; when that person leaves
    the project, it moves to an owner with its open questions;
  - its budget is carved again at a period's start, and a standing role past it is
    held for money;
  - a standing role's own proposal is refused when it decides it.
- **Fuzzy:** random areas and hand-offs, where every finding has exactly one
  owner at any time;
  random wake times, where no run resumes past a quiet span and every fresh
  run follows a written Now or the session's recorded end.
- **A world story,** with a fake model's script (`../domain/testing.md`, 2.1
  and 2.4): on the playground's repository, a standing role owns the tests. A red of its
  tier reaches the triager, which hands it to the role; it triages it, makes the repair change, and the change
  lands. It writes its Now. A wake an hour later starts fresh, its brief
  holding that Now and no transcript. A person talks with it, and an
  instruction becomes a rule by a proposal accepted in place.
- **For v0.1.0:** the same story driven through `temper ctl` on a box, its
  role handed its area and its rule accepted by `proposal decide`, the red's
  finding read with `role show`, and the talk by `role talk` (section 9);
  walked on real backends after landing, as `v0.1.0.md`, section 3 climbs a
  rung.

## 12. Open questions

- **The quiet span's and the bound's defaults,** set from the numbers of the
  first standing roles that run.
- **A standing role's default model and budget,** per stock way.
- **A standing role that reads another project,** as Porter reads vsr and writes
  vsr-rs: its authority across projects is `follows.md`'s.
- **Who may talk with a standing role:** the project's owners alone, or its members
  too, and whether a member's instruction may become a rule.
- **A standing role handing work to another,** by a message, a finding, or a
  change of areas.
- **The weekly report's form:** a note, or a document the standing role shows (jig's
  `draft/notes-memory.md`, section 5).

## 13. Later

- **Routing by path,** a deterministic way beside the triager's judgement
  (the user, 2026-11-08): an area names paths of its repositories, a repository
  whole or prefixes within one; a path has one area, the one whose prefix names
  it longest; and a finding filed against those paths, or a red whose failing
  tests or changed files are in them, goes to the owner with no agent's run.
  Sign-offs by path are another matter, a landing's rule, and stay
  (`what-reaches-you.md`, section 7).
