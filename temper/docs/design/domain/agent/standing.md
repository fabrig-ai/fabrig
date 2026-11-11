# A standing role

Provisional, 2026-11-11, from `../../draft/standing-roles.md`, sections
2, 3, 5 and 9. A standing role is an agent that owns some of a
project's work for the project's life, works in short sessions, writes
its Now as each ends, and answers to a person: jig's task that stands
(jig's `domain/tasks/standing.md`), run under a charter of its own. This
document says what temper fills in: the stock's charter and template
for one, its person and what follows when that person leaves the
project, what it may do, and how a person starts and reads one. Its
area, said in words and routed to by the project's triager, is later
work of `../../draft/standing-roles.md`, sections 4, 8 and 10; until then a role's spec words
say what it covers.

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
  and no deadline. Its spec's words are what it covers and how, until
  areas are settings (`../../draft/standing-roles.md`, section 4).
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
  (`../forge/attention.md`, 5); write its Now; and propose changes to its
  own rules, its wakes or its budget, as a proposal of settings (jig's
  `domain/engine/settings.md`, 6).
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

## 7. The world

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
- **Its story on a box** is `../../draft/standing-roles.md`, section 11's,
  once its areas and talking with it have landed.

## 8. Open questions

- **A standing role's default model and budget,** per stock way
  (`../../draft/standing-roles.md`, section 12).
- **Who may start a standing role:** a project's owners alone, or any
  member whose role allows the batch, as today.
