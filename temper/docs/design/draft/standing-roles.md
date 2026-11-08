# Standing roles

Draft, 2026-11-08, from a session with the user that day. An agent that owns
an area of a project for the project's life: its rules, its area, its Now,
what wakes it and a budget a week. It works in short sessions and writes its
Now as each one ends. It is the middle kind of `whole-story.md`, section 4's
three kinds of agent session, between a call that runs once and a conversation
a person is in. It adopts jig's `draft/notes-memory.md`, section 4, for a
role's Now. It widens `../domain/findings.md`, section 6's one triager per
project to an owner per area. It departs from jig's `domain/tasks.md`, 8.6,
where a charter whose runs resume holds no parts, and from jig's
`domain/engine.md`, 7.4, where the charter alone says whether a run resumes.
It wins where they differ until its plans fold it in. temper/migration carries
out the engine's parts, jig's included, the routing and the sessions; temper/ui
the Team page and a role's card (`surfaces.md`, section 6); temper/tui talking
with a role in the terminal (`surfaces.md`, 11.3).

## 1. In one page

- **An owner for an area, for the project's life.** A standing role holds an
  area of a project: some paths of its repositories and the findings labelled
  for it. It answers for what happens there: findings, reds after landing, the
  questions its own work asks. A person may own an area instead, and Team
  lists both (section 2).
- **A long life, short sessions.** Each stretch of work is a session. It goes
  on while its work does and its cache is warm, and code ends it. As it ends,
  the role writes its Now. The next wake starts fresh from its rules, its Now
  and what woke it (section 3).
- **Its Now is its memory.** A few lines it writes itself: where things stand,
  what it waits for, what it does next. People read it on Team, and the status
  agent writes Home's brief from the owners' Nows (sections 3 and 7).
- **Routed by area** (section 4). What has no area goes to the project's
  triager, now the owner of what nobody else owns.
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
- **A way may bring roles** (section 10). Its acceptance is section 11's.

## 2. What a standing role is

- **Its rules:** a charter, as every agent has (`../protocol/configuration.md`,
  13.4): instructions for its area's work, in parts by why a run runs
  (`per-wake.md`, section 3), its models and efforts, its tools and its
  documents (jig's `domain/engine.md`, 17.9). They change by a proposal of
  settings, as every setting does (jig's `domain/engine.md`, 17.6).
- **Its area:** paths and labels, section 4.
- **Its Now:** its working notes (jig's `draft/notes-memory.md`, section 4)
  under one name, written by its runs and read whole in every brief. A page at
  most: where things stand, what it waits for and why, what it does next and
  when, what it asked and what it decided.
- **Its wakes:** a wake policy and subscriptions (jig's `domain/tasks.md`, 8.3
  and 8.4) over what concerns its area: landings that touch its paths, reds of
  its tiers, findings with its labels, its own work's questions and results, a
  person's words, and timers. Porter, the pilot's owner of vsr-rs
  (`whole-story.md`, 5.2), wakes at once on a landing on vsr's main that
  touches `src/` or `include/` and on a red of vsr-rs, every four hours on
  findings labelled `port`, and on Mondays at 06:00 for its report.
- **A budget per period:** an amount a week, carved again each period from
  the project's spend, as a recurring task's is (jig's `domain/authority.md`,
  section 7; jig's `domain/tasks.md`, section 10). What it has left shows on
  Team. Past it, the role stops and asks, since money always waits for a
  person (`what-reaches-you.md`, section 2).
- **Its report:** its Now at all times, and a report each week (section 7).
- **A person may own an area instead.** An area's owner is a role or a party.
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
- **Code ends it,** at the first of three:
  - **a quiet span:** no wake for longer than the cache keeps, a setting of
    the role's charter, by default 50 minutes, inside a provider's hour;
  - **the context's bound:** a share of its model's window, by default half;
  - **its work closed:** nothing it started is live, and its inbox holds
    nothing it is offered.
- **Ending, it writes its Now.** Before the quiet span passes the cache's
  life, the engine wakes the session once more, a warm wake, with a notice
  that the session ends. Its run writes its Now and finishes. A session ended
  by its bound or its closed work writes its Now in the run that ends it. A
  session that ends with no Now written keeps the last one, and its end is a
  line in its activity.
- **The next wake starts fresh.** Its brief carries its rules with the parts
  for why it runs (`per-wake.md`, section 3), its Now whole, what woke it and
  its area's state. It carries no transcript: the Now takes the place of a
  transcript's tail (jig's `domain/engine.md`, section 9).
- **Parts and resuming together.** The session's first run reads the parts its
  reasons choose. Later runs in the session resume, and keep the instructions
  the session began with, which keeps the cache (jig's `domain/tasks.md`,
  8.6). They are offered what those parts were for. Any other message keeps
  its place and wakes the next session, which reads the parts for it, as
  `per-wake.md`, section 4 keeps what a run isn't offered.
- **Why:** on our measurement of 2026-11-07, 74% of the long sessions we run by
  hand spent their own share on cold wakes: a wake after the cache expired
  writes the whole context again, at about twenty times a warm wake's cost.
  Compaction loses what nobody chose to keep, and nobody sees what it lost. A
  Now is chosen by the role, read by people, and read whole by the next run. A
  fresh run from a small brief is what made the goal's later runs cheap
  (`per-wake.md`, section 1).

## 4. Areas and routing

- **An area** is a project's: a name, paths of its repositories (a repository
  whole, or prefixes within one), the finding labels it takes
  (`area:<name>`, `../domain/findings.md`, section 2) and its owner. Today a
  project's areas are named per repository beside its gates
  (`../domain/findings.md`, section 9). Areas by path are new.
- **A path has one area:** the area whose prefix names it longest. The
  project's area with no paths, "No area yet" on Team, takes the rest.
- **What goes to an area's owner:**
  - **A finding** with its label, or filed against its paths. The owner
    triages it as `../domain/findings.md`, section 6's triager does: to a goal,
    to a new goal, to the backlog, or dropped.
  - **A red of no goal's** whose failing tests or changed files are in its
    paths (`../domain/after-landing.md`, section 7). Today the project's
    triager is told; the area's owner is told instead.
  - **Feedback:** a person's word through Tell temper is a finding, routed the
    same way (`whole-story.md`, section 2).
  - **Questions** follow the tree, as ever (jig's `domain/tasks.md`, section
    9): a role is the requester of what it starts, so its own work's questions
    reach it. Where `what-reaches-you.md`, section 4 lets an agent decide in a
    person's place, the area's owner is that agent for its paths.
- **A person owner** gets what is routed to them in Needs you, as a person
  triager is notified today (`../domain/findings.md`, section 6).
- **The project's triager** becomes the owner of what has no area: a role or a
  person, as the project's setting names it today.

## 5. What a role may do

- **Its authority** is its task's (jig's `domain/authority.md`, section 3),
  from its project's policy: what it may make, read and spend.
- **It may:**
  - start goals and changes in its area, under its project's way, so that
    they are checked, reviewed and landed as any work is;
  - answer its own work's questions, and decide where `what-reaches-you.md`,
    section 4 lets it;
  - triage the findings and reds routed to it;
  - write its Now and its report;
  - propose changes to its own rules, its wakes, its area's paths or its
    budget, as a proposal of settings (jig's `domain/engine.md`, 17.6).
- **It may not:**
  - accept its own proposal: an owner decides it (jig's `domain/engine.md`,
    17.6);
  - change files outside its area. A change that must is asked of that area's
    owner, or left as a finding for it;
  - spend past its period's budget;
  - decide what `what-reaches-you.md`, section 2 sends to a person.
- **A merge is never its own.** Its changes land through the project's rules
  as any change does; a merge tool is never an agent's.

## 6. Talking with a role

- **A person opens a conversation with it:** `/talk` in the terminal
  (`surfaces.md`, 11.3), "Talk to it" on Team (`surfaces.md`, section 6). The
  person's words wake it and always reach a live run (jig's
  `domain/tasks.md`, 8.3).
- **The talk is a session** (section 3). The person sees its Now at the top,
  and its brief holds its rules, its Now and its area. It resumes while the
  person is there.
- **What lasts goes where it belongs.** An instruction for every case becomes
  a rule: the role proposes a change to its own rules, and the person, if an
  owner, accepts it in place ("Add it to Porter's rules?"). An instruction for
  the work at hand goes in its Now. A decision becomes a record: an answer, a
  goal, a setting.
- **Leaving ends the session.** The person leaves, the role writes its Now,
  and the conversation stays as the role's activity (jig's
  `domain/engine.md`, 7.2), read on Team, never resumed.
- **Not a chat.** A chat is the person's own and delegates for them. A role's
  conversation is the role's: its words join the role's activity, and what the
  role starts is its own.

## 7. Reports

- **Its Now on Team** (`surfaces.md`, section 6), with when it was written and
  by which session; and its sessions this week, a line each: when, how long,
  what it did and what it spent.
- **A weekly report,** written at its timer: what landed in its area, what it
  decided, what waits and on whom, what it spent. It is kept with the role and
  read on Team.
- **Home's brief from the owners' Nows.** The status agent
  (`../domain/forge.md`, 5.7) reads each owner's Now beside the facts it reads
  today. A brief across projects is then what the owners say, rolled up, not a
  count.

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
- **Their role files and reads.** It starts no work: what follows a finding
  is its owner's.

## 9. What the engine adds

Each is a design lag, by its status file.

- **A task that stands** (jig). An agent task made standing is never told
  that nothing is awaited (jig's `domain/tasks.md`, 5.2; jig's
  `domain/engine.md`, 7.4), and ends only when cancelled, as a recurring task
  does (jig's `domain/tasks.md`, section 10). Its requester is the deployment,
  on its project's behalf (jig's `domain/tasks.md`, section 3).
- **A budget refilled each period** (jig): carved again from the project's
  spend each period, as a recurring task's is, its unspent rest returning to
  the period that funded it (jig's `domain/authority.md`, section 7).
- **Sessions** (jig): resuming within a session and starting fresh across
  them, by section 3's rule; a charter that both resumes and has parts (jig's
  `domain/tasks.md`, 8.6; jig's `domain/engine.md`, 7.4); the warm last wake
  that writes the Now.
- **The Now kept with the task** (jig): a task's working notes, from jig's
  `draft/notes-memory.md`, section 4, whole in every brief, bounded, and read
  by the project's people.
- **Areas by path, and routing** (temper): areas with paths and an owner, as a
  project's setting (`../domain/findings.md`, section 9); findings and reds of
  no goal's routed to their area's owner (`../domain/findings.md`, section 6;
  `../domain/after-landing.md`, section 7).
- **A role in the stock** (temper): its charter and its template
  (`../protocol/configuration.md`, 13.4), and its row among the agents temper
  runs (`../domain/agent.md`, section 8).
- **Its pages and acts** (temper's client): Team and a role's card, and
  talking with a role, on the web and in the terminal (`../client/README.md`,
  5.3 and section 6); `role list`, `role show`, `finding file` and `need ask`
  in `temper ctl` (`../client/tools.md`, 5.3).

## 10. Stock roles and ways

- **A way may bring roles** (`stock-ways.md`, section 2), as Draft first
  brings its coordinator and goal template (`stock-ways.md`, 3.3). Draft
  first brings a triager role, the owner of what has no area, with a small
  budget a week. A project with no role keeps its person as the owner of
  everything, as at Pair.
- **How work runs lists a project's roles** with their areas, wakes and
  budgets, beside its way.
- **"Hand an area to a role"** on Team names the area's paths, takes a stock
  role's rules or new ones written by asking temper, and sets its budget: a
  proposal an owner accepts.
- **The stock's roles' rules are temper's to write,** as the coordinator's are
  (`../protocol/configuration.md`, 13.4, charter 5): the triager first, then
  the roles the pilot needs, a prover for tests and a porter for a follow
  (`follows.md`, section 2).

## 11. Its acceptance

Plans and their acceptance use focused and fuzzy tests alone; a walk follows
each landing.

- **Focused:**
  - a session's end as a pure function of its last wake, its context and its
    open work: a quiet span, the bound or closed work ends it, and nothing
    else;
  - the next run resumes inside a session and starts fresh after one, its
    brief holding the Now and no transcript;
  - routing: the longest prefix wins, a label routes, what has no area goes to
    the triager, a person owner gets a Needs you entry;
  - a standing task is never told that nothing is awaited, and ends only when
    cancelled;
  - its budget is carved again at a period's start, and a role past it is
    held for money;
  - a role's own proposal is refused when the role decides it.
- **Fuzzy:** random areas and paths, where every path has exactly one owner;
  random wake times, where no run resumes past a quiet span and every fresh
  run follows a written Now or the session's recorded end.
- **A world story,** with a fake model's script (`../domain/testing.md`, 2.1
  and 2.4): on the playground's repository, a role owns `tests/`. A red of its
  tier is routed to it; it triages it, makes the repair change, and the change
  lands. It writes its Now. A wake an hour later starts fresh, its brief
  holding that Now and no transcript. A person talks with it, and an
  instruction becomes a rule by a proposal accepted in place.
- **For v0.1.0:** the same story driven through `temper ctl` on a box, then
  walked on real backends after landing, as `v0.1.0.md`, section 3 climbs a
  rung.

## 12. Open questions

- **The quiet span's and the bound's defaults,** set from the numbers of the
  first roles that run.
- **A role that reads another project,** as Porter reads vsr and writes
  vsr-rs: its authority across projects is `follows.md`'s.
- **Who may talk with a role:** the project's owners alone, or its members
  too, and whether a member's instruction may become a rule.
- **A role handing work to another role,** by a message, a finding, or a
  change of areas.
- **The weekly report's form:** a note, or a document the role shows (jig's
  `draft/notes-memory.md`, section 5).
