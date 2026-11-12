# What reaches a person

## 1. A person's facts

What is true of a person, kept by the engine so that it holds on every
device and across restarts, each a `PersonFact`: `ToBegin { project }`, "Skip"
on that project's "To begin" (`../../draft/to-begin.md`, section 5); and `GoalFrom
{ project, chat, goal, at }`, that the person started goal `goal` from chat `chat`'s
document, at the client's clock's `at` (`../../client/pages/README.md`); and `KeepAsking
{ project, subject, at }`, that the person pressed Keep asking on a need's reason
for the item of `subject` (6) at the client's clock's `at`, so that its Stop
asking is not offered for 7 days (`../../draft/what-reaches-you.md`, section 5). It owes nothing to the forge: the
connector keeps it because `Stored` holds every family of temper's, as it does
`Binding` (`../../protocol/store.md`, section 2).

- **Its rows** are held in memory, each its party and its fact, restored whole
  at the restart step, at most 8 times `bindings` times `judge_projects`; a row
  past that is refused at restart, as each family's is.
- **`Dismiss { what }`** (`../../client/forge/records.md`) is admitted by the
  core, the root naming `what`'s project as its scope: a party with a
  role there, in a project not being deleted: a service with a role
  may dismiss too, having no Home, its rows doing no harm. The core's
  ask names the party, and the root hands the connector the request
  with it. In the decision:
  - a `KeepAsking` where the party holds one of that project and subject
    with another `at`: that row erased and the new one saved, the subject
    `dismissed` marked, answered dismissed;
  - a row held already: answered dismissed, nothing written;
  - the bound reached, all rows' or the party's own (`DISMISSED`, the document's):
    answered too large, nothing written;
  - otherwise the row saved, the subject `dismissed` marked, answered
    dismissed, the answer held behind the commit.

  The answer is held as a set up's is, in memory, and is unknown after
  a restart; the row says what stands.
- **Its document:** the subject `dismissed` is one number,
  `DISMISSED_SUBJECT`, outside the task numbers as `landings_subject`'s
  are, of the party's own scope. The core's ask for a document names the
  watcher's party, and the connector answers that party's rows in key
  order: each watcher's own. A mark makes every watcher's document
  again, so a fact costs each signed-in client one small document;
  a subject per party would make the root read the sign-in on every
  watch.
- **Erased with its project:** the release of a project being deleted
  (jig's `domain/connectors.md`, section 2) erases its rows in its
  decision, beside its settings, and marks the subject when any went.
  jig erases no party (`../../protocol/store.md`, section 7).

## 2. What changed since a person last looked

What a project's Home says of its landings and of the person's last
look (`../../client/pages/README.md`): two families of temper's, kept by the
connector as `Dismissed` is (1), and a document computed at its read.

- **`ProjectFacts`,** one row a project that has landed anything, held
  in memory, restored whole at the restart step, at most
  `judge_projects`:
  - `landed`, the count of the changes landed, into every landing
    branch of the project. A merge to a goal's branch counts: a change
    that lands on a goal's branch and the goal's own landing on main
    are two;
  - `last_landed`, the wall time of the latest, the time of its
    change's decision;
  - `brief`, none or what the status agent last wrote (3): `sentence`,
    `lede`, `at` and `landed`, the count it covers.

  A landing is counted in the decision that first logs its change in
  the landing log (`../after-landing.md`, section 10), once a change; the
  row is saved and the subject `status` marked there. A project with no
  repository left in `repositories` counts nothing. Past the bound a
  landing is not counted, as `Landed`'s are not past theirs
  (`../../protocol/store.md`, section 7).
- **`Looked`,** one row a party and a project, held in memory, restored
  whole, at most `bindings` times `judge_projects`:
  - `previous` and `latest`, walls: the end of the sitting before this
    one, none on a party's first, and this sitting's last look;
  - `seen`, the `landed` count at `previous`, and `next_seen`, the count
    at `latest`, which `seen` becomes when `previous` next moves. Both
    are kept because a sitting's first look must compare what landed
    with where the person stopped: a `seen` taken at that look would be
    the count that look reads, and say that nothing landed.
- **`Looked { project }`** (`../../client/forge/records.md`) is admitted by
  the core as `Dismiss` is, the root naming the project as its scope.
  In the decision, with `now` the wall and `landed` the row's count, 0
  with no row:
  - no row, the bound reached: answered too large, nothing written;
  - no row: `previous` none, `latest` now, `seen` and `next_seen`
    `landed`;
  - `latest` under 30 minutes old, or ahead of `now`: `latest` now,
    `next_seen` `landed`, so a sitting keeps one "last looked";
  - otherwise a new sitting: `previous` the old `latest`, `seen` the old
    `next_seen`, `latest` now, `next_seen` `landed`.

  Each saves the row, marks the subject `status` and answers looked, the
  answer held behind the commit as `Dismiss`'s is and unknown after a
  restart. The 30 minutes are the limit `forge.sitting_ms`, `SITTING` in each
  profile. A story reaches a second sitting on its world's clock, never by
  waiting (`../../testing/process-tier.md`).
- **Its document:** the subject `status`, `status_subject(project)`, one
  number a project, outside the task numbers as `changes_subject`'s are,
  of the project's scope. The core's ask for a document names the
  watcher's party, as for the subject `dismissed` (1), and the
  connector computes it at that read:
  - `landed`, `last_landed` and `brief`, the row's; 0 and none with no
    row;
  - `previous` and `seen`, the party's `Looked`; none and `landed` with
    no row, so nothing reads as new;
  - `queued`, the project's changes the subject `changes` counts
    (`../../client/forge/records.md`): each in temper's queue, ready or owning
    its turn, and each in the forge's own merge queue (`queues.md`, 1);
  - `red`, whether the forge's red record holds for the home
    repository's landing branch, as that document's `red` for that
    branch: a live repair of the branch (`queues.md`), or a landing a tier
    still holds red (`../after-landing.md`, 5.3). False with no home;
  - `branch`, the home repository's landing branch, its default branch
    (`Repository.settings`); empty with no home.

  It is marked wherever the subject `changes` is (a queue, a branch's
  red, a merge), and at a look and a brief. A look makes the document
  again for every watcher of the project's, each its own party's; a
  subject per party would make the root read the sign-in on every
  watch (1).
- **Erased with its project,** in the same decision as its dismissals
  (1), the subject marked when any went. jig erases no party, so a
  `Looked` row stays until its project goes, as `Dismissed` does
  (`../../protocol/store.md`, section 7).

## 3. The status agent

The few lines at the top of a project's Home, written by an agent from
what the engine keeps, at most once a sitting and only with something
new to say. Its charter is the stock's 9
(`../../protocol/configuration/stock.md`).

- **When:** in the decision of a `Looked` that moves `previous` (2),
  with `landed` above the brief's `landed`, the brief over an hour old or
  none, and no run in flight for the project. A party's first look moves
  no `previous` and starts none. There are no timers: nobody looking, or
  nothing landed, spends nothing.
- **In flight:** `ProjectFacts.running`, none or `{ task, landed, since }`,
  saved in that decision, `task` none until the root has made it, so a
  restart keeps it. A run in flight over an hour counts as none, so one
  that is held or waits on an account never stops the next, and its late
  result is dropped, as `running` no longer names it.
- **Made** as a tier run's task is (`../after-landing.md`, section 3):
  `Request::StatusRun { project, words }`; the root makes a procedure task
  of the project's period, code `STATUS_CODE`, on `repair_seed`, as it has
  an agent to fund, and answers `Event::StatusTask { project, task }`, or
  `Event::StatusRefused { project }` with no seed or a refused admission.
- **What funds it:** what the project's period has unreserved, up to the
  rules' run cap, which a run returns what it leaves of. A stock project's
  period holds one chat's 24K beyond its owner's pool
  (`../../protocol/configuration/stock.md`), so a fresh project's briefs run from
  it, with what the forge's own repairs take. A member's pool reserves from that
  room too, so the first member to spend in a period can use it up. Its payer is
  the project: a key serves it as any project's work, and a sign-in's plan while
  its holder is the project's only member (jig's `domain/accounts.md`, 4.2).
  With neither, its run waits, as *In flight* has it.
- **Its run:** the procedure's first step delegates to charter 9 with the
  task's words, unlisted, no person asked (`asks` false), its authority
  narrowed to no tools, grants, notes or delegation, and no workspace: a run of this charter selects no repository. A
  project whose policy's delegation doesn't name charter 9 has its
  delegate refused, which is a refused run. It is kept as a tier run's task
  is: jig forgets a result only for a person's request.
- **Its words,** made by the forge, and the root adding the goals and
  the standing roles, which are core's: the span since the last brief,
  the landed count, the titles of the changes the landing log holds since
  the last brief (at most 20), the project's live goals each with its
  delegates done and total, the failing tests each tier of the project's
  repositories holds, and each live standing role's Now. They say only
  what the engine keeps.
- **The roles' Nows** (`../agent/standing.md`, section 8): after the
  goals, a line "Standing roles:", then for each of the project's live
  standing roles, label 3, by number, at most 8, a line "Role N:" and its
  Now's body, read as `role show` reads it, the note described `now` in
  its scope, cut to 1,024 bytes at a character's edge, or "no Now yet".
  The words are fitted to the spec's bound as before, the roles' part
  cut first from its end. A Now the root cannot read is left out, and the
  run goes on: a brief never waits on a role.
- **Heard** at the procedure's next step, from its inbox: the report's first paragraph is `sentence`, the rest
  `lede`, cut by `text::fit` to 200 and 600 bytes, saved to
  `ProjectFacts.brief` with `at`, the wall, and `landed`, `running`'s, and
  the subject `status` marked. A failed, cancelled, refused or empty run
  saves nothing and asks nobody: Home keeps the computed words, and the
  next chance is the next sitting. The procedure then ends.
## 4. The Away switch

A person's switch on one project: while it is on, what the project's
classes let Away decide (6, the class `Ask`) is decided for that
person, and shown afterwards (5). It is the person's own, no setting of
the project: the classes are the project's, Away each person's. It owes
nothing to the forge: the connector keeps it as it keeps a dismissal
(1). No engine answers for an agent: the switch only changes what
`decided` says.

- **Its row,** `Away { party, project }`, at most one a party a project,
  held in memory, restored whole at the restart step, at most `bindings`
  times `judge_projects`; a row past that is refused at restart, as each
  family's is. It holds:
  - `since`, a wall: when it last went on;
  - `until`, a wall or none: when it goes off by itself, which the
    client works out (`../../client/pages/README.md`), none for "I turn it
    off";
  - `landed`, `ProjectFacts.landed` (2) at `since`, 0 with no facts;
  - `off`, a wall or none: when it went off, none while it has not.
- **On at a wall** when `off` is none and `until` is none or after that
  wall. Past `until` nothing is written: `decided` judges Away off, the
  client draws it off, and the spell's end reads as `until`.
- **`SetAway { project, on, until }`** (`../../client/forge/records.md`) is
  admitted by the core as `Dismiss` is: the project its scope, a party
  with a role there, in a project not being deleted. In the decision, at
  its wall:
  - `on`, the row on: `until` replaced, the spell kept;
  - `on`, no row or the row not on: a new spell: `since` the wall,
    `landed` the project's count, `off` none, `until` as asked;
  - not `on`, the row on: `off` the wall, `until` as asked;
  - not `on`, the row not on: `until` as asked, and `off` its `until`
    where it ran out with none written, so the spell's end stays where it
    was;
  - not `on`, no row: a row with `since` and `off` the wall, `landed` the
    count: Settings' choice of `until` kept while off;
  - a row to make with the bound reached: answered too large, nothing
    written;
  - a row that comes out the same: answered, nothing written.

  Otherwise the row saved, the subject `away` marked, answered, the
  answer held behind the commit, held as a set up's is, in memory, and
  unknown after a restart; the row says what stands.
- **Its document:** the subject `away`, one number, `AWAY_SUBJECT`,
  `1 << 60`, outside the task numbers, of the party's own scope. The
  connector answers the watcher's party's rows in key order, as 1's.
- **Erased with its project,** as 1's, marking the subject when any
  went. jig erases no party (`../../protocol/store.md`, section 7).

## 5. Decided for you

What an agent decided for a person, and why each question that asked
reached them, kept so that Needs you and Home say so
(`../../client/pages/README.md`) and the person can overturn a decision. The
agent decides: the host tool `decided` (`../agent/tools.md`) asks the
connector whether it may, and its answer is live, so a running task needs
no brief or wake (`../../draft/what-reaches-you.md`, sections 4 and 6).

- **The caller's person,** which the root finds and hands over: the
  person the question would have reached. Of a task whose requester is a
  person, a goal, a chat or any task a person made, that person; of a
  task whose requester is a task, the person of the chat serving it
  (jig's `served_chat`) where that chat's requester is a person. A
  standing role's requester is its person, so it is that person
  (`../agent/standing.md`, section 3). Any other caller is answered ask by the root, with nothing
  asked of the connector: its questions go to its requester, a task,
  which asks in turn. The root hands over too whether the caller's words
  hold a line `Asks: always`, as `always`.
- **The judgement,** in the call's decision: 6's `judge` of the
  project's classes (kind 8, or its stock), the call's `kind`, `paths`
  and `under`, the party's `Away` row for the task's project on at the
  decision's wall (4), and `always`. A `took` past the options is
  answered ask, whatever the judgement, with no reach and no row.
- **Its answer is kept as the call's,** `CallPayload::Decided { took,
  reach }`: the taken option's words, or none for ask, and the reach. A
  call answered replays that answer whatever the switch or the classes
  say by then, and writes nothing again: calls are named and decided once
  (`../agent/tools.md`), as `change`'s are.
- **Its row:** every call the connector judges with its `took` among the
  options writes one, decided or asked, so that a question that follows says why it came (6;
  `../../client/pages/README.md`). `Decided { project, number }`: `party`, the
  person's; `task`, the caller; `at`, the decision's wall; `question`, at
  most 400 bytes; `options`, 2 to 6, each at most 120 bytes, and none for
  an asked row; `took`, an index into them, none for an asked row; `why`,
  at most 400 bytes, empty for an asked row; `reach`, the judgement's;
  `overturned`. `number` is one above the project's highest, 1 with none.
- **Kept 7 days, bounded by a limit:** a decision that writes a
  project's row first erases that project's rows older than 7 days
  (`DECIDED_KEPT_NS`). A project holds at most `forge.decided_per_project`
  rows (`../../protocol/configuration/limits.md`), 1,024: at the bound,
  the project's oldest row past its day (24 hours, `DECIDED_DAY_NS`) is
  erased in the same decision; with none past its day, no row is
  written and the call is answered ask, its reach as judged, since a row
  still inside its day is never erased for room. So the family is
  bounded by `decided_per_project` times `judge_projects`, and a row past
  that is refused at restart.
- **Its document:** the subject `decided`, per project,
  `(1 << 62) | (1 << 35) | project`, beside `status_subject` (2), of
  the project's scope. It answers the asking party's rows alone, as 2's
  is the asking party's own: what was decided for one person is not
  another's to see or overturn. `rows`, the decided ones, each with its
  taken option's words in place of the options and the index; `asked`,
  the asked ones, each its number, task, `at` and reach; each list the
  newest `DECISIONS_SHOWN`, 128, in number order, and `cut` where either
  left a row out.
- **`Overturn { project, number }`** (`../../client/forge/records.md`) is
  admitted as `SetAway` is. In the decision: no such row, another
  party's, or an asked row: refused, unknown; `overturned` already:
  refused, overturned; otherwise `overturned` set, the row saved, the
  subject `decided` marked, answered. It sends no word to the task: the
  client does, once answered, by its existing message, so that a refused
  message leaves the row as it stands.
- **Erased with its project,** as 1's, marking its subject.
- **Nothing answers for anyone.** A question that skips the tool waits as
  ever, and held tasks and proposals always wait, away or not. A
  change's question passed up (`change/questions.md`, 1) reaches its maker, which calls
  `decided` before it would ask its person, as any caller does.

## 6. What reaches a person

What an agent may decide for a person, and what reaches them, is one
setting of a project's, its classes (`../../protocol/configuration/settings.md`,
2, kind 8), judged by one pure function that the forge's `decided`
(5) and the client (`../../client/pages/README.md`) both call: the crate
`temper-protocol-reaches`, a step crate on `skein-lib` and the client
wire's records alone (`../../draft/what-reaches-you.md`, sections 2 to 5).

- **The kinds,** `Kind`, named in the asker's call as the draft's table
  names them: `wording`, `retry`, `implementation`, `tests`, `dependency`
  and `departure`, which a project's classes may move; `money`,
  `irreversible`, `rules` and `other`, *fixed*, which always wait. The
  draft's `paths` is no call's kind: a call names its own kind and the
  paths it would touch, and each path set those touch is an item of its
  own beside the kind.
- **The classes,** `Class`, strictest last: `Decide`, agents decide, then
  tell you; `Ask`, asks you, and Away may decide (4); `AskAway`, asks
  you, even when away; `Wait`, always waits for you, the fixed kinds'
  alone.
- **The table,** `Classes { items, sets }`:
  - `items`, at most 20, each `Item { subject, words, class }`: `subject`
    a movable kind or a path set's number (`Subject::Kind`,
    `Subject::Set`); `words` empty for the subject's own item, or at most
    120 bytes of a rule in words that narrows it ("fixes that keep the
    wire format"), for agents to follow and no code to check, so that a
    decision taken under them is listed as any other; `class` one of the
    first three. An item is numbered by its place, from 1. A subject has
    at most one item without words; a subject with none is `AskAway`.
  - `sets`, at most 6, each `PathSet { number, name, patterns }`:
    `number` 1 to 6, unique in the table; `name`, 1 to 40 bytes, as the
    person reads it ("vsr's protocol core"); `patterns`, 1 to 6, each 1 to
    80 bytes, over the paths of any repository of the project.
  - `Classes::check` refuses, naming the first fault by item or set: more
    items or sets than the bounds; an item of a fixed kind, of `Wait`, of
    words past 120 bytes, naming a set the table lacks, or a second item
    without words of its subject; a set numbered twice or outside 1 to 6,
    a name or a pattern empty or past its bound, no pattern or more than
    6, a pattern not well formed (below).
- **A pattern** is segments joined by `/`, matched against a path's own:
  a segment `**` matches no segment or any run of them; `*` inside a
  segment matches any run of bytes but `/`; every other byte matches
  itself. Well formed: no empty segment, no leading or trailing `/`, and
  `**` only as a whole segment. So `include/vsr.h`, `src/protocol.*` and
  `docs/protocol/**`. A call's path is relative to its repository's root,
  1 to 256 bytes, with no leading `/`; a call names at most 16.
- **The stock tables,** `Classes::stock(Way)`, the draft's defaults
  (`../../draft/what-reaches-you.md`, section 2): *You approve*, no item, so
  every movable kind asks even when away; *Agents review* and *Draft
  first*, `wording`, `retry` and `tests` at `Decide`, then
  `implementation`, `dependency` and `departure` at `Ask`, in that order;
  no set in any. A project whose kind 8 holds no value has Agents
  review's, the stock way of kind 2's stock rule 1
  (`../../protocol/configuration/stock.md`).
- **The judgement,** `judge(classes, ask) -> Reach`, of an
  `Ask { kind, paths, under, away, always }`: the call's kind, its
  paths, `under` the number of an item with words it says its choice
  falls under (0 for none), whether the person's Away is on at the
  decision's wall (4), and whether the asker's words hold `Asks:
  always`:
  - *its subjects:* the kind, and each set one of whose patterns matches
    one of the paths;
  - *each subject's class:* a fixed kind's `Wait`; otherwise its own
    item's, or `AskAway` with none, but where `under` names an item with
    words of that subject, which gives its class instead. An `under`
    naming anything else is not used;
  - *`Asks: always`* lifts every class below `AskAway` to `AskAway`, as
    the goal's own choice (`../../client/pages/goal.md`, 3);
  - *the strictest class wins;* its reason the kind where the kind's
    class is the strictest, otherwise the lowest-numbered set at it, or
    `Always` where only the lift made it;
  - *decided* where that class is `Decide`, or `Ask` with Away on;
    otherwise ask.

  Its answer, `Reach { kind, class, reason, under, sets, matched, away
  }`: `reason` `Reason::Kind`, `Reason::Set(number)` or `Reason::Always`;
  `under` the item used, 0 for none; `sets` the touched sets, bit
  `number − 1` each; `matched` the reason set's patterns a path matched,
  bit `i` for its pattern `i`, 0 for any other reason; `away` whether Away
  was on. `Reach::decided` says which. On the wire and in the store a
  reach is the record `Reach`, and a table `ProjectClasses`
  (`../../protocol/configuration/settings.md`, 2).
- **Away may decide** while the table holds an item of `Ask`
  (`Classes::away_may_decide`): what the client's switch is shown by
  (`../../client/pages/README.md`).
- **Stop asking,** `offers(classes, reach)` and `narrowed(classes, reach,
  offer, words)`: the narrowest changes that would have decided a reach
  that asked, each a whole table the person proposes or an owner applies
  (`../../client/pages/README.md`; `../../draft/what-reaches-you.md`, section
  5):
  - `Offer::Decide(subject)`, for a reason `Kind` of a movable kind: the
    item `under` named moved to `Decide`, or with none the kind's own
    item, added at the end where it has none;
  - `Offer::Words(set)`, for a reason `Set`: an item of that set with the
    person's words, 1 to 120 bytes, at `Decide`, added at the end;
  - `Offer::Split(set)`, for a reason `Set` whose set holds two patterns
    or more of which `matched` is some but not all: the matched patterns
    leave it for a new set at the lowest free number, named the set's
    name and ", in part", cut at a character to 40 bytes, with an item
    at `Decide`; the set keeps its number, its other patterns and its
    items.

  None for a fixed kind, for `Always`, or for a reach decided; an offer
  the table has no room for is left out. Where a second subject asked as
  strictly, the narrowed table still asks, and the need it raises next
  names that one.

## 7. Being told

What reaches a person while they aren't looking: a push to each device
they turned it on for, as each need that holds work reaches them
(`../../draft/what-reaches-you.md`, section 9). It is the person's own,
as Away is (4): their devices are rows the connector keeps (7.1), the
root pushes as jig's core tells it a need arrived (7.2), and the
deployment turns push on in its configuration (7.3). Mail and the
digest are designed (7.4) and not built.

### 7.1 A person's devices

- **Its row,** `Device { party, number }`, the party's own, held in
  memory and restored whole at the restart step, at most `DEVICES`, 8,
  a party and 8 times `bindings` in all; a row past either is refused at
  restart, as each family's is. It holds:
  - `endpoint`, the push service's URL for that browser's subscription,
    1 to 1,024 bytes that skein's `Endpoint::parse` takes (skein's
    `net/push.md`, section 2);
  - `p256dh`, 65 bytes, and `auth`, 16 bytes, the subscription's keys;
  - `label`, 1 to 60 bytes of UTF-8, what the person knows it by,
    "Safari on iPhone";
  - `at`, the wall it was added or last replaced at.

  `number` is one above the party's highest, 1 with none. An endpoint
  and its keys let whoever holds them push to that browser, so no log
  prints them and no one but the party reads them.
- **`AddDevice { endpoint, p256dh, auth, label }`**
  (`../../client/forge/records.md`), of the party's own scope, is
  admitted for a person; a service, which no need reaches, is refused
  `person`. In the decision, at its wall, the first that holds:
  - push off on this deployment (7.3): refused, `off`;
  - an endpoint that doesn't parse, or whose origin isn't one of the
    deployment's push services (7.3): refused, `service`;
  - a `p256dh` whose first byte isn't 4, or a label empty, past its
    bound or not UTF-8: refused, `malformed`;
  - the party holds a row of that endpoint: its keys and label
    replaced and `at` the wall, its number kept, as a browser
    subscribing again does; with the same keys and label, answered with
    nothing written;
  - another party holds a row of that endpoint: that row erased in the
    same decision, then the new row as below, so that a browser one
    person signed out of and another signed in to pushes for the second
    alone;
  - the party at `DEVICES`, or the family at its bound: refused, `too
    many`;
  - otherwise the row saved under its new number.

  The subject `devices` marked, answered `added { number }`, the answer
  held behind the commit and kept as `SetAway`'s is (4).
- **`RemoveDevice { number }`,** of the party's own scope: no such row
  of the party's, refused `unknown`; otherwise the row erased, the
  subject marked, answered `removed`.
- **Its document:** the subject `devices`, one number, `DEVICES_SUBJECT`,
  `1 << 59`, outside the task numbers, of the party's own scope. The
  connector answers the watcher's party's rows in number order, each `{
  number, label, at, endpoint }`, never the keys: the browser finds its
  own row by its subscription's endpoint, and a page shows the
  endpoint's origin alone.
- **Erased by nothing else:** a device is no project's, and jig erases
  no party (`../../protocol/store.md`, section 7). A member removed from
  every project keeps theirs, and no need reaches them, so nothing is
  pushed.

### 7.2 A push

- **What starts one:** jig's core tells the root each need's arrival,
  `NeedArrived { party, project, task, needs }`, in the decision that
  made it (jig's `domain/people/README.md`, section 8). Every need holds
  work: a question waits for its answer, a proposal or an escalation for
  its decision, a person task for its result. What Away decides is no
  need, since `decided` answers before a question is asked (5), so while
  a person is away only "always waits" and "even when away" items, and
  questions that skip `decided`, reach them and push
  (`../../draft/what-reaches-you.md`, section 9). A Decided row pushes
  nothing.
- **The root, in that decision,** asks the forge connector for the
  arrival's pushes: one `Push` for each device the party holds, none
  with push off or no device. The connector writes nothing for it. The
  root holds each push as a delivery behind the decision's commit, as
  it holds every delivery, and the service sends it once the commit is
  durable (7.3).
- **`Push { party, device, endpoint, p256dh, auth, topic, payload }`:**
  - `topic`, `p` and the project's number in decimal, `p7`: a phone
    that was offline is given the latest count of each project, not a
    pile of them;
  - `payload`, JSON, `{"title":"temper","body":LINE,"project":N}`:
    `LINE` is "1 thing needs you in NAME" or "COUNT things need you in
    NAME", `COUNT` the arrival's `needs` and `NAME` the project's name as
    the core holds it; `N` the project's number, which the browser opens
    Needs you at (`../../client/web/README.md`, 2.7). Never the
    question's words, its task's title or who asked: the push service
    carries the payload sealed, but a phone shows it on a locked screen.
    A name that would take the payload past skein's `PAYLOAD_BYTES` is
    cut on a UTF-8 boundary and ends "…".
- **Sent once,** with a `TTL` of 86,400 seconds, a day, and `Urgency`
  `high`: the service seals and posts it (skein's `net/push.md`) and
  tells the root `Pushed { party, device, endpoint, answer }`. `Gone`
  erases the device, in a decision of its own, where the party still
  holds it with that endpoint, marking `devices`; every other answer
  changes nothing. No push is sent again: a lost one costs a count that
  the next push, or the page, gives.
- **Bounded:** at most `PUSHES_IN_FLIGHT`, 16, being sent at once, each
  within 10 seconds from its connection to its answer; a push past them
  is dropped, never queued, since a burst of needs is a burst of the
  same few counts.
- **A sign-off's wait** (`change/questions.md`, 2) is a read of the
  forge's, not a jig need, and pushes nothing: rules by path reshape it
  (`../../draft/what-reaches-you.md`, section 7).

### 7.3 The deployment's push

- **Off unless configured:** `application.push` absent or `null`
  (`../../protocol/configuration/engine.md`) is off: no device can be
  added, nothing is pushed, and the deployment's read says so
  (`../../client/forge/records.md`), so the page says this deployment
  doesn't push (`../../client/pages/told.md`).
- **On, with three members:** `key`, a data path to the VAPID key,
  PKCS#8 PEM of a P-256 key (skein's `net/push.md`, section 3), private,
  0o600, read at start, a key skein's `Vapid::from_pkcs8_pem` refuses
  refusing the start, naming the file; `subject`, the `mailto:` or
  `https:` URI the push services may reach the operator at; and
  `services`, the origins pushes may go to, by default Google's,
  Mozilla's and Apple's: `https://fcm.googleapis.com`,
  `https://updates.push.services.mozilla.com` and
  `https://web.push.apple.com`. A browser whose push service is another
  (Microsoft Edge's, under `notify.windows.com`) is refused until the
  deployment names its origin.
- **The service sends each push:** it matches the endpoint's origin
  against `services` (a push whose origin has left them is dropped),
  seals it with the key and subject (skein's `seal`), connects to the
  origin's host as it connects to a forge, over TLS for `https` with the
  trust roots it gives forges, posts it, reads the status, closes, and
  tells the root skein's `answer` of it.
- **What else a phone needs,** which no configuration gives: the web
  client served over HTTPS at the deployment's name
  (`application.clients.public`; `../../protocol/sign-in.md`, section 2),
  since a browser subscribes only on a secure origin, and a box on
  loopback is one for its own machine alone; and on iOS, the web app
  added to the Home Screen first (`../../client/pages/told.md`).
- **The key is kept for good:** browsers subscribe with its public half,
  which the deployment's read gives clients, so a new key leaves every
  device subscribed to the old one: each push to it is refused, and the
  person turns it on again on their page.
- **A box** pushes only if its configuration is given push, as any
  deployment's; a story with no browser registers a device with `temper
  ctl push add` (`../../client/ctl/tools.md`, 3) whose endpoint is a fake
  push service's (skein's `testing/fakes/push.md`).

### 7.4 Mail and the digest

Designed, not built: the milestone's nice part, left when push took
the changes it was given.

- **Mail where push can't reach:** a need that pushes nothing, because
  the person has no device, is sent by mail, the same line as a push's
  body, to the address the person set, if the deployment has mail on.
- **The digest:** decided rows, landings and the status agent's brief
  for each of the person's projects, by mail once a day at the person's
  hour, or never; while away, Away's decisions wait for it.
- **The person's own:** a row `Told { party }`, set by the party's
  record as `SetAway` is: `mail`, an address of 3 to 254 bytes, or none;
  `digest`, the next wall it is sent at, which the client works out as
  it works out `until` (4), or none for never.
- **The deployment's mail:** `application.mail` absent or `null` is
  off; on, `host`, `port` (465 for TLS from the start, 587 for
  STARTTLS), `from`, an address, `user`, and `password`, a data path,
  private, 0o600, read at start.
## 8. Open questions

- **"The brief on Home" in Settings** (`attention.md`, 3): the status agent always runs
  for now; the switch is another plan's. Default: it runs.

- **A status run's kept row** (`attention.md`, 3): a task made for the deployment is
  never forgotten, as jig forgets a result only for a person's request. At
  most one a sitting and an hour, each small. Default: kept, until jig
  forgets a deployment's reads.

- **A stock project's room, shared** (`attention.md`, 3; `../../protocol/configuration/stock.md`):
  the chat's 24K outside the pools is what no pool reserved, and no brief's own: the
  forge's repairs draw on it and a member's first spend in a period reserves it, and
  a shared project's plan doesn't serve the run. Home then keeps its computed
  sentence, and nothing tells the owners why. Default: so; they can fund the period.

- **What lands on a goal's branch and then on main** (`attention.md`, 2) counts twice,
  each a change landed. Default: so, said here and in the design of Home.

- **What ends the Away switch** (`attention.md`, 4): `until`, by the client's next
  08:00, and a hand. Quiet hours and notifications are the client's
  later parts (`../../client/README.md`, section 5); neither is the
  switch's.

- **The classes per person** (`attention.md`, 6; `../../draft/what-reaches-you.md`,
  section 11): a team may want one person asked about a path set and
  another not. Default: the classes are the project's and Away each
  person's, approvals naming roles as they do.

- **What code decides, listed** (`attention.md`, 5): a known flake's rerun
  (`../findings.md`, section 5) asks no one and writes no row. Whether
  Decided for you lists it. Default: not listed.
