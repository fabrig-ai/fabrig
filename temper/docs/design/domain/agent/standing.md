# A standing role

Provisional, 2026-11-11, from `../../draft/standing-roles.md`, sections
2, 3, 5, 6, 7 and 9. A standing role is an agent that owns some of a
project's work for the project's life, works in short sessions, writes
its Now as each ends, and answers to a person: jig's task that stands
(jig's `domain/tasks/standing.md`), run under a charter of its own. This
document says what temper fills in: the stock's charter and template
for one, its person and what follows when that person leaves the
project, what it may do, how a person starts, reads and talks with one, and its reports. What it owns is the
areas that name it as their owner, said in words, and what the
project's triager hands it (`../areas.md`); the stock's standing roles,
the triager first, are charters 14 to 16
(`../../protocol/configuration/stock.md`).

## 1. In one page

- **jig's standing task, temper's charter** (section 2): the stock's
  template 7, *standing role*, makes one, `ended_by` `standing`, labelled
  3, run by the creator's charter, the stock's charter 13 unless they name
  another.
- **Its person** (section 3) is the person who started it, its requester:
  its questions, escalations and proposals reach them, `decided` records
  its Decided rows for them, and when they leave the project temper's
  root moves the role to an owner, its open questions with it.
- **Its sessions and its Now are jig's** (section 4): its runs resume
  while their model's prompt cache is warm, each session's first run
  starting fresh from its Now; the stock's models declare their cache's
  life.
- **Bounded** (section 5): its task's authority, its period's budget, the
  project's rules for its changes; it never accepts its own proposal.
- **Started and read through `temper ctl`** (section 6): `role start`,
  `role list` and `role show`; Team draws it (`../../client/pages/team.md`).
- **Talked with** (section 7): a person's words open a talk, jig's, whose
  session stays warm while they are there; an instruction for every case
  becomes a rule by a proposal of settings an owner accepts; `role talk`
  in `temper ctl`.
- **Its reports** (section 8): its Now at all times, each session's line
  of what it did, a weekly report at its timer, and its Now read by
  Home's brief.

## 2. Its charter and its template

- **Its rules are a charter** (jig's `domain/engine/settings.md`, 9):
  instructions for its work, in parts by why a run runs, its models and
  efforts, its tools and its documents. The stock's charter 13
  (`../../protocol/configuration/stock.md`) is a standing role's own,
  for any project; a deployment's owners add charters of their own, as
  any setting, by proposal, and a role is started under one of them.
- **Charter 13** resumes, with parts (jig's `domain/tasks/messages.md`,
  6), so that its runs work in sessions: its shared instructions say
  what a standing role is and does (section 5) and that it writes its Now
  before every park; a part on `first` has its first session read its
  spec, subscribe to what concerns its work and set its timers; and a part
  on every other kind that chooses parts covers its work, so that after
  its first session every session is offered whatever wakes it. Its wake
  policy wakes it at once on words and notices, on questions, answers and
  timers, and on each delegate's result (`each`), since every goal or
  change it started ending is its business, and on news paced to 30
  minutes; its runs park as soon as they wait.
- **Template 7, a standing role,** run by the charter its creator names:
  `ended_by` `standing` (jig's `protocol/configuration.md`, 3.2), not a
  conversation, listed, kept, untracked, labelled 3, its contract a
  report as template 1's, its budget the creator's, a period's amount,
  and no deadline. Its spec's words say how it works; what it owns is
  the areas naming it as their owner, or as the project's triager
  (`../areas.md`, sections 2 and 6), which its runs read with the
  `finding` tool (`tools.md`).
- **Label 3** marks a standing role, beside 1, a chat, and 2, a goal, so
  that a project's roles are listed by label (section 6).

## 3. Its person

- **Its requester is its person:** the person who started it. jig routes
  its questions, its holds' escalations and its proposals beyond its
  authority to that person's Needs you, as for any task a person made,
  and its own work's questions reach it, as their requester's (jig's
  `domain/tasks/standing.md`, section 3).
- **`decided` records for its person** (`../forge/attention.md`, 5): a
  standing role's requester is a person, so its Decided rows are that
  person's, and Overturn's message wakes it, as a person's words always
  do. A goal or a change a role started asks the role, a task, which
  decides or asks its person in turn.
- **When its person leaves the project,** temper's root moves the role to
  an owner as its requester, with jig's move (jig's
  `domain/tasks/standing.md`, section 3): when the project's roles are
  saved and a live standing role's person no longer holds a role there,
  in the decision after the one that saved them, to the owner whose
  request saved them, or else the project's owner with the lowest
  number. Its open questions go with it to the owner's Needs you: a role
  asks its person by a person task addressed to them, asking in words
  (jig's `domain/tasks/messages.md`, 5), since temper's inbox holds a
  person's answers (jig's `domain/people/README.status.md`), so the root
  makes the move and addresses each of the role's live person tasks
  addressed to the person who left to the owner, one compound decision
  (jig's `domain/tasks/changes.md`, Several operations, one decision),
  all or none. A person task addressed to a role stays as it is. A move
  jig refuses, for room or for funding, leaves the role as it is, and the
  root tries again at the project's next period and its next change of
  roles.
- **Money waits for a person:** past its period's budget it is held, and
  its person is asked (jig's `domain/tasks/standing.md`, section 4).

## 4. Its sessions and its Now

- **jig's, as they are** (jig's `domain/tasks/standing.md`, sections 5
  and 6): a session resumes while its model's cache is warm, ends at the
  first of jig's ends, and the next wake starts fresh, its brief holding
  its rules with the parts for why it runs, its Now whole and what woke
  it, and no transcript. An engine restart ends no session: temper's
  workers and their agent processes keep running and reconnect
  (`../worker/README.md`).
- **The stock's models declare their cache's life** (`cache_life_ms`;
  `../../protocol/configuration/stock.md`): five minutes, each provider's
  default, for every stock model but the scripted one, which declares
  none, so a role on it starts fresh at every wake.
- **Its Now** is the note described `now` in its own scope, which its
  project's people read: Team's card and `role show` read it with jig's
  notes read of the role's scope (jig's
  `client/domain/contract/README.md`, section 4).

## 5. What it may do

- **Its authority is its task's,** from its project's policy for its
  person (jig's `domain/authority.md`, section 6): what it may make,
  read and spend.
- **It may** start goals and changes, with `goal` and `change`
  (`tools.md`), under its project's way, so that they are checked,
  reviewed and landed as any work is: a change of a task with no goal
  branch lands on its repository's landing branch
  (`../forge/change/README.md`); answer its own work's questions, and
  call `decided` where its project's classes let it
  (`../forge/attention.md`, 5); write its Now; propose changes to its
  own rules, its wakes or its budget, as a proposal of settings (jig's
  `domain/engine/settings.md`, 6); and own a follow (`../forge/follows.md`,
  sections 2 and 4): its follow task is its delegate, funded by it, its
  producers' questions and its changes' holds reach it, it acts with
  its `follow` tool, and a red after landing of a follow change reaches
  it by the area it owns.
- **It may not** accept its own proposal: an owner decides it, and its
  own decision of it is refused (jig's `domain/tasks/proposals.md`);
  spend past its period's budget; or decide what
  `../forge/attention.md`, 6's always-waiting kinds send to a person.
  A merge is never its own: its changes land through the project's rules
  as any change does, and no merge tool is an agent's.

## 6. In `temper ctl`

- **`role start`** fills template 7 for a project: its words, its
  budget, and its charter, 13 unless named; a person whose role may not
  make it proposes it, as any batch beyond their role
  (`../../client/ctl/tools.md`, 3).
- **`role list`** lists a project's live standing roles, label 3, a line
  each: its number, its charter, its state, what its period's budget has
  left, and its Now's first line (`../../client/ctl/tools.md`, 2).
- **`role show`** prints one: its task's document, and its Now whole,
  with when it was written (`../../client/ctl/tools.md`, 2).

## 7. Talking with it

From `../../draft/standing-roles.md`, section 6. The talk is jig's (jig's
`domain/tasks/standing.md`, 5.6); temper fills in who may talk, what the
role does with what it hears, and the tools.

- **Who may talk:** whoever may send the role words, its person and the
  project's maintainers and owners (jig's `domain/people/acts.md`, 1), one
  talk at a time. Whose instruction may become a rule is the accepter's
  to say: a rule changes a charter, which the deployment's owners
  decide (below).
- **A talk is a session of its own.** The person's first words end the
  role's open session and begin a fresh one, its brief holding its rules
  with the part for words, its Now whole and what woke it; their later
  words resume it however long they think, until they leave or 30
  minutes pass with neither them nor the role saying anything, its
  lapse. Leaving, or the lapse, ends the session, and the next wake
  starts fresh from the Now the talk's last run wrote.
- **It answers in its ordinary text,** as an Open run replies (`charter.md`,
  5): that reply is what the person reads, then it writes its Now and
  parks. Within its rules it acts as on any wake: it may start a goal or
  a change the talk asks for, as its own work.
- **What lasts goes where it belongs,** by charter 13's instructions
  (`../../protocol/configuration/stock.md`):
  - *an instruction for every case* ("from now on…") it asks to make a
    rule ("Add it to my rules?"), its reply ending with the choices as a
    numbered list, "1. Yes, add it", then one keeping it for the work at
    hand, "2. Just for this port", worded for that work, and "3. Tell it
    something else" last, so that a terminal offers them as a prompt
    (`../../client/tui/pages/talk.md`, 1, *The role's choices*); on yes it
    proposes a change of its
    charter's instructions with `propose_settings`, the rule added in
    its words, its reason naming the talk; it says in its reply that an
    owner of the deployment accepts it, and names the proposal;
  - *an instruction for the work at hand* goes in its Now;
  - *a decision* becomes a record: an answer, a goal or a change, a
    setting proposed.
- **A rule changes its charter,** a deployment's setting, so that every
  task the charter runs takes it: a role started under the stock's
  charter 13 shares it with every role started under 13, which the
  proposal's review shows its accepter. A role whose rules are its own
  runs under a charter of its own (`role start --charter`). Accepted with
  running work moved, the role takes the new revision at its next run,
  which ends its session by its rules (jig's `domain/tasks/standing.md`,
  5.2), so a talk goes on in a fresh session from its Now; accepted with
  running work staying, it takes it at its next move.
- **Not a chat.** Its words join its activity, and what it starts is its
  own: a role is the requester of its work, never the person talking.
- **In `temper ctl`:** `role talk TASK TEXT` sends the words with jig's
  `Talk`, waits until the role has replied, and prints its reply;
  `role talk TASK --leave` leaves (`../../client/ctl/tools.md`, 3). A
  rule it proposed is read with `proposal show` and decided with
  `proposal decide` (`../../client/ctl/tools.md`, 3), as any proposal.
  Team's "Talk to it" and the terminal's `/talk` draw the same talk
  (`../../draft/surfaces.md`, 6 and 11.3).

## 8. Its reports

From `../../draft/standing-roles.md`, section 7.

- **Its Now, at all times** (section 4), with when it was written and by
  which run: the note's `changed` and author, and the task's session
  record on its document (jig's `domain/tasks/standing.md`, 7), say
  whether the session open now wrote it.
- **Each session's line of what it did:** charter 13's runs open their
  Now with one line saying what the session has done so far and where it
  stands, so the session's last `NowWritten` line in its activity says
  what it did (jig's `domain/tasks/standing.md`, section 6). With the
  session's times and its runs' spend, which the activity holds, Team
  draws its sessions this week a line each (`../../draft/surfaces.md`,
  6).
- **A weekly report** at its timer: its first session sets a timer for
  Mondays at 06:00 UTC, repeating each week, and a role that finds none
  in its Now sets it at its next session. At that timer it writes the
  note described `report` in its own scope, whole, replacing last week's:
  the week it covers; what landed in its work, from its goals' and
  changes' results; what it decided, and for whom; what waits, and on
  whom; and what it spent, from its brief's period line (jig's
  `domain/tasks/standing.md`, section 6). A page at most, as its Now.
  Team links it, and `role show --report` prints it
  (`../../client/ctl/tools.md`, 2).
- **Its timers need the subscribe family:** the stock's policy gives it
  to a project's people (`../../protocol/configuration/stock.md`), so
  that a role started by a stock project's member sets its timers and
  subscribes, as section 2 has its first part do.
- **What Home's brief reads:** the status agent's words carry each live
  standing role's Now (`../forge/attention.md`, 3), which is all it reads
  of a role's area; its instructions and Home's page are unchanged.

## 9. The world

- **Focused:** the stock's charter 13 passing jig's settings check with
  its parts and resuming, template 7's shape, and each stock model's
  cache life; in temper's engine world, a role started from template 7
  by a person, its run writing its Now and parking, a later run's brief
  holding that Now; `decided` called by a standing role recording a row
  for its person, and Overturn waking it; a role whose person is removed
  from the project moved to the owner who removed them, its open
  question in that owner's inbox, and a refused move tried again at the
  next period; `role start`, `role list` and `role show` parsed, sent
  and printed.
- **Focused, its talk and reports:** charter 13's instructions holding
  the talk's guidance, the Now's opening line and the weekly report, and
  the stock's policy the subscribe family; in the client's joined world,
  a person's `role talk` reaching a role started from template 7, its
  scripted run replying and proposing a rule with `propose_settings`,
  the reply printed; the person accepting it with `proposal decide`,
  running work moved, and their next words met by a fresh run under the
  new instructions with the Now in its brief; `role talk --leave` ending
  the talk and its session; a stock member's role setting its weekly
  timer and, at it, writing its `report`, which `role show --report`
  prints; and in the engine's world, a status run's words holding each
  live role's Now.
- **Its story on a box** is `../../draft/standing-roles.md`, section 11's,
  once its areas and talking with it have landed.

## 10. Open questions

- **A standing role's default model and budget,** per stock way
  (`../../draft/standing-roles.md`, section 12).
- **Who may start a standing role:** a project's owners alone, or any
  member whose role allows the batch, as today.
- **Who may talk with a standing role** (`../../draft/standing-roles.md`,
  section 12): whoever may send it words today (section 7), and whether
  a member's instruction may become a rule, which today the charter's
  owners decide by accepting it or not.
- **A rule for one role on a shared charter** (section 7): a role under
  the stock's charter 13 changes every role on it; a rule of its own
  would need its own instructions, which jig's tasks keep apart from a
  charter's parts (jig's `domain/tasks/messages.md`, 6).
- **Past weekly reports** (section 8): one `report` note, rewritten each
  week, keeps the newest alone (`../../draft/standing-roles.md`, section
  12, the report's form).
