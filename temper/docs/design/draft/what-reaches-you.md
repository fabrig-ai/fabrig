# What reaches you

Draft, 2026-11-08, from a session with the user that day. What reaches a
person, and what agents decide in their place and tell them afterwards, as
one setting of a project's, of which Away becomes a modifier. It is
`whole-story.md`'s Decide (its section 2). It revises `../domain/forge.md`,
5.8 and 5.9, where Away's kinds are four bits of the person's own, only
goals decide and a project keeps 64 decided rows; `../domain/forge.md`, 8.12
and section 9, where a change waiting on a person's approval holds its
branch's queue; `../domain/agent.md`, 4.3's `decided`; `stock-ways.md`,
section 2, where a way sets how work starts and lands and nothing else, as a
way now also sets the classes' defaults (section 2); what kind 6, a project's
question route (`../protocol/configuration.md`, 13.2), and Start goal's
"Always ask" (`Asks: always`, `../domain/forge.md`, 5.9;
`../client/README.md`, 5.14) mean beside the classes (section 2); and
`../client/README.md`, 3.5's Away and Needs you, and it gives 5.13's
disabled *Telling you* its mechanism. It wins where they differ until its
plans fold it in. temper/ui carries it out: the classes, Away, Needs you,
Decided for you and being told; temper/migration carries rules by path into
the landing (section 7).

How we run our own development today is the model: whoever meets a decision
picks, acts and lists it for the person to review, and it can be undone;
money, what can't be undone and departures from the person's word always
wait. This draft makes that a project's setting, so that nobody blocks on a
person for what they would have waved through, and nothing that matters
passes them by.

## 1. In one page

- **Three classes** (section 2): agents decide, then tell you; asks you;
  always waits for you. Each holds kinds of decision, and a project's way
  sets their defaults. Money past a budget, what can't be undone and
  changes to these rules always wait.
- **Away is a modifier** (section 3), not a list of its own: while it is on,
  "asks you" items are decided with the recommended choice and listed,
  except those marked "even when away". It shows once a project has an item
  it may decide, so a project on You approve has none.
- **Whoever would have asked decides** (section 4): a goal's coordinator, a
  standing role owning the area, any task whose question would reach a
  person. Each decision is a Decided row, for any task, not only goals.
- **Every need says why it reached you,** and offers **Stop asking**
  (section 5): a narrower rule the client composes, which the person
  proposes, or an owner applies, in place.
- **Decided for you** shows for a day on Needs you and Home, then stays in
  the task's history, with Overturn as today (section 6).
- **Rules by path** (section 7): a landing rule may name a path set, so that
  a change touching a protocol core waits for its sign-off, on the forge,
  out of the queue, and the rest lands on its own. temper's landing
  enforces it.
- **Only you** (section 8): sign-ins and logins to renew, listed apart; work
  goes around them.
- **Being told** (section 9): what blocks work is pushed to the person's
  phone at once, the rest comes in a digest; while away, Away decides what
  it may, and what still asks is pushed as ever.

## 2. Three classes

A project's setting, decided by its owners and proposed by Ask temper like
any other (`../protocol/configuration.md`, 13.2; `../domain/agent.md`,
8.4.2): a table of kinds, each in one class.

- **Agents decide, then tell you:** the asker decides, and the decision is
  listed under Decided for you (section 6). Nothing waits.
- **Asks you:** the question reaches the person, and the work that needs the
  answer waits. Other work goes on. Each item is either one Away may decide
  (section 3) or marked **even when away**.
- **Always waits for you:** as "asks you", and Away never decides it
  (section 3). Its kinds are fixed: no setting moves them out.

The kinds, each named in the asker's call (section 4):

| Kind | What it is | Class it may take |
|---|---|---|
| `wording` | words and names inside a spec, a design or a page | any but always |
| `retry` | retrying a failed task, or splitting it in two | any but always |
| `implementation` | a choice the spec leaves open | any but always |
| `tests` | a change that touches only the project's tests | any but always |
| `dependency` | a new dependency | any but always |
| `departure` | a departure from a design that has landed | any but always |
| `paths` | a choice whose change touches a named path set (section 7) | any but always, per path set |
| `money` | funds past a budget, or a budget raised | always |
| `irreversible` | rewriting history, deleting work that hasn't landed, a store's data | always |
| `rules` | a change to this table, a landing rule, a standing role's rules | always |
| `other` | anything the asker can't place | always |

- **A path set is named once** in the project, with its patterns over a
  repository's tree: for instance vsr's protocol core, `include/vsr.h`,
  `src/protocol.*`, `src/transition.*` and its protocol document under
  `docs/` (`whole-story.md`, 5.2). A rule by path (section 7) and the
  `paths` kind both name it.
- **A rule in words narrows a kind:** "fixes that keep the wire format" is
  for agents to follow, not for code to check (`configuring.md`, section 2),
  so a decision taken under such words is always listed.
- **`money`, `irreversible` and `rules` cover** what `other` covers today by
  the charters' words (`../domain/agent.md`, 4.3): funds, sign-ins,
  machines, a plan's go-ahead and anything hard to reverse. A proposal of
  settings and an escalation still wait for their decider, whatever the
  table says (jig's `domain/tasks.md`, section 9).
- **Kind 6 stays the route,** the classes the gate: a project's question
  route (`../protocol/configuration.md`, 13.2) still says which way a
  change's question goes up the tree, through a foreman or to its maker
  (`../domain/forge.md`, 8.11). The classes judge only where a question
  would reach a person.
- **"Always ask" stays a goal's own,** chosen when it starts
  (`../client/README.md`, 5.14): a goal whose words hold `Asks: always` is
  judged as if every item of the table asked, Away included, as 5.9 answers
  such a goal today.

**The defaults,** set by a project's way (`stock-ways.md`, section 2) and
changed afterwards as any setting is:

| Way | Agents decide, then tell you | Asks you; Away may decide | Asks you, even when away |
|---|---|---|---|
| You approve | none | none | `wording`, `retry`, `implementation`, `tests`, `dependency`, `departure` |
| Agents review | `wording`, `retry`, `tests` | `implementation`, `dependency`, `departure` | none |
| Draft first | `wording`, `retry`, `tests` | `implementation`, `dependency`, `departure` | none |

- **You approve decides nothing for the person:** they stay in the loop, so
  a project on it has no Decided rows and no Away to show
  (`whole-story.md`, section 3).
- **Agents review and Draft first** decide the small things, and let Away
  decide the rest, so Decided for you and Away appear from the spectrum's
  Team and Stock on.
- **A path set** takes the class its owners give it, "even when away" most
  often: vsr's protocol core asks its owner even at night.

## 3. Away, a modifier

Away stays a person's switch on a project, with its spell and its end
(`../domain/forge.md`, 5.8). Its four bits of kinds go: what it lets
agents decide is the project's table.

- **While it is on,** an "asks you" item Away may decide is decided with
  the asker's recommended choice and listed. Items marked "even when away"
  ask as ever, and "always waits" kinds are never decided.
- **It shows** once the project's table holds an "asks you" item Away may
  decide: in the rail's foot and on a phone's top bar, as
  `../client/README.md`, 3.5 draws it. With none, as on You approve, there
  is nothing for it to do, and it isn't drawn.
- **Until a time:** `until` stays as 5.8 has it, the next 08:00 by default
  or "I turn it off". Past it, everything asks again.
- **Settings' section** *While you're away, agents may decide*
  (`../client/README.md`, 5.13) becomes a view of the table: each "asks
  you" item with its "even when away" mark, and the "always waits" kinds
  locked, as its last line is today.
- **Several people:** the table is the project's, Away is each person's. A
  question that would reach someone away is decided for them alone; one
  that reaches someone present asks.

## 4. Who decides in your place

- **Whoever would have asked:** the asker's coordinator, for a change's
  question routed up (`../domain/forge.md`, 8.11); the area's owner where a
  standing role owns it (`standing-roles.md`, sections 4 and 5); a goal; a
  chat delegating work. Any task whose question would reach a person may
  call `decided`, which `../domain/forge.md`, 5.9 limits to goals today.
- **`decided` takes a kind and, for a choice that changes files, the paths
  it would touch** (`../domain/agent.md`, 4.3). The judgement is code's, in
  the call's decision: the kind's class, the person's Away, and every path
  set the paths, or the asker's change at its head, touch. The strictest
  class wins: a wording choice inside the protocol core asks its owner.
- **"Ask" names the item that sent it,** so the question that follows
  carries its reason (section 5).
- **Recorded as a Decided row** for the person the question would have
  reached: a goal's requester; a standing role's person, the person who
  handed it the area, else the project's owners (`standing-roles.md`,
  section 2); a chat's person.
- **Bounded per project by a limit, not 64:** widening deciding to every
  task fills 5.9's 64 rows a project in a busy night, and rows still inside
  their day would be erased unseen. A row is kept 7 days, and a project
  holds at most `decided_per_project` of them, a limit setting
  (`../protocol/configuration.md`, section 7), 1,024 by default; past it,
  the oldest row already past its day goes first, and a row still inside
  its day is never erased for room: a call that would need it is answered
  ask. The row's document stays 5.9's.
- **Nothing in the engine answers for an agent:** a question that skips
  the tool waits as ever, and code decides only whether the agent may.

## 5. Why it reached you, and stop asking

- **Each need says why,** in one line under its question: the item that
  sent it and its class, from the decision that said ask: "Reached you
  because it changes vsr's protocol core, which asks you, even when away."
  A need with no item, a question routed to its person by kind 6
  (`../protocol/configuration.md`, 13.2), says so.
- **Stop asking** is one link on the line. It opens, in place, the
  narrowest change that would have decided this need: the item moved to
  "agents decide, then tell you"; or, for a path set, the item with words
  that narrow it ("fixes that keep the wire format"); or the set split.
- **The client composes it,** from the need's reason, with no model, and
  the person proposes it as their own; an owner may apply it directly
  instead. It is never a procedure's proposal, which jig's
  `domain/engine.md`, 17.6 rules out. Ask temper writes one in words when
  the person asks for something else (`../domain/agent.md`, 8.4.2).
- **An owner accepts it in place,** one press, and new questions follow it.
  The need in hand still waits for its answer: the rule changes what comes
  next, never what was asked.
- **Keep asking** says nothing changes; the line stops offering Stop asking
  for that item for a week.
- **Its look** is `surfaces.md`, section 4, and on a phone section 10.

## 6. Decided for you

- **Shown for 24 hours** on Needs you and on Home, newest last, each with
  its question, what was taken, why, and Overturn; then only in the task's
  history, where its call keeps its answer (`../domain/forge.md`, 5.9).
- **Its why** says what allowed it: "Allowed: test-only changes are decided
  and listed."
- **Overturn** stays 5.9's: the row marked, and the client sends the task
  its message, which a standing role, like a goal, is woken by.
- **Ended work:** a row whose task has ended says when, and that a new goal
  can undo it, as `../client/README.md`, 3.5 has it.
- **Across projects:** Needs you's unfiltered page lists every project's
  rows (`surfaces.md`, section 4).

## 7. Rules by path

Landing rules cover branches: a rule's pattern ends at a branch
(`../protocol/configuration.md`, section 3), so a project can't say "a
change touching the protocol core needs my sign-off, the rest lands on its
own" (the Team example on fabrig.ai's home page says so).

- **A rule gains path sets:** a landing rule may name path sets of the
  project's (section 2). It then covers a change only when the change
  touches one of them; with none, it covers every change, as today.
- **What a change touches** is its diff from its merge base with the
  landing branch to the head being merged, so that an update bringing the
  landing branch's own edits to a path set makes no change touch it. A
  change whose files the forge can't give whole, too many to list or a read
  that failed, counts as touching every path set, as an unknown overlap
  wakes in `../domain/forge.md`, 7.3.
- **Enforced by temper's landing:** the forge judges requirement 5, *the
  landing rules* (`../protocol/configuration.md`, 13.3), over the rules
  covering the branch and, for a rule with path sets, the paths the change
  touches at that exact head, a fact the forge reads with the change's
  review facts (`../domain/forge.md`, 8.7). A merge lands only what was
  checked, so a head that newly touches a path set waits again.
- **Out of the queue while it waits:** a change whose merge waits on a
  person's approval leaves its branch's queue until the approval counts at
  its head, then takes its place again after the change then first. The
  rest of the branch lands meanwhile. This revises
  `../domain/forge.md`, 8.12, where the read leaves the branch held by the
  change first in line, and section 9's queue: a sign-off can wait a
  night, and vsr's other changes shouldn't.
- **Its approval** is a rule's `approvals`, as today: a person's review on
  the forge. Needs you's card leads with "Review on GitHub ↗" or "Sign off
  on Forgejo ↗", one tap to the forge's review page, as
  `forge-identity.md`, section 8 has it, its line saying the path set:
  "Signs off changes to vsr's protocol core." Approving in place, posted as
  the person, stays open (section 11).
- **The `paths` kind and the rule are two judgements:** the kind is
  judged when an agent asks, on the paths it declares or its change's head
  (section 4), and says whether a choice may be made for the person; the
  rule is judged at the merge, on the change's diff, and says whether it
  may land. A choice decided inside a path set still lands only with the
  rule's sign-off.
- **Changed live,** as 8.9 steps live changes: a path set added to a rule
  holds the changes it now covers for their approval.
- **The forge's own protection** (a GitHub code owners file, a branch's
  required reviewers) is not written: whether temper also posts the rule
  where the forge enforces it stays `../README.md`, section 8's open
  question.
- **The path** draws the sign-off as a stop for that path set alone
  (`stock-ways.md`, section 4; `surfaces.md`, section 7).

## 8. Only you

What only the person can do and no choice answers: a sign-in to an LLM
provider, a login to renew, a forge's token to replace, a person task
addressed to them that asks for an act rather than a choice (jig's
`domain/tasks.md`, 5.4).

- **Listed apart,** under its own head on Needs you, "Only you", after the
  needs and before Decided for you.
- **Read, not stored:** the accounts' states (`../client/README.md`, 5.5)
  and the person's tasks of that kind, as the approvals' wait is a read
  (`../domain/forge.md`, 8.12).
- **Work goes around them:** runs that need the account wait for it, as
  today, and nothing else does. The line says what waits: "Two agents on
  vsr wait for this login."
- **Early:** a login that ends within two days shows here before it ends.

## 9. Being told

Settings' *Telling you* row is disabled today (`../client/README.md`, 5.13).
It becomes:

- **Pushed at once:** a need that holds work, "asks you" or "always
  waits", sends a push to every device the person turned it on for, through
  the web client's service worker and the browser's push service (Web Push).
  Its payload says how many things need the person and in which project,
  never the question's words; the page reads the rest once opened.
- **A digest for the rest:** decided rows, landings and the brief, once a
  day at the person's hour, or never.
- **While away:** Away decides what it may (section 3) and those rows wait
  for the digest; what still asks, "always waits" and "even when away"
  items, is pushed as ever.
- **Mail where push can't reach:** a browser with no push, or a deployment
  the push service can't be reached from, sends the same short line by the
  deployment's mail settings, if it has them.
- **What push needs,** none of which a box on loopback has:
  - the web client served over HTTPS where the phone reaches it, a
    deployment at a name; a box on loopback is told by mail, or not at all;
  - a VAPID key pair, the key that signs pushes, kept as the deployment's
    secret as its other credentials are (`../protocol/credentials.md`);
  - on iOS, the web app added to the home screen before it may be pushed
    to;
  - for mail, the deployment's SMTP settings and their credential.
- **A device registered by `temper ctl`:** `temper ctl me push add`
  registers a subscription read from a file, so that a story with no
  browser, the box's, can be pushed to a fake push service.
- **The person's own:** which devices, the digest's hour and mail are
  theirs, kept as Away is, not a project's setting.
- **Its look** on a phone is `surfaces.md`, section 10.

## 10. Its acceptance

- **Focused and fuzzy tests in the worlds:**
  - the judgement as a pure function of kind, classes, Away, its marks and
    paths, fuzzed over all of them: an "always waits" kind is never
    decided, the strictest class wins, Away decides only "asks you" items
    it may, and a project on You approve's defaults decides nothing;
  - in the forge's world, a rule with a path set: a change touching it
    waits for its approval out of the queue while an unrelated change on
    the same branch lands, one that doesn't touch it lands, a head that
    newly touches it waits again, an update from the landing branch
    touches nothing, and an unknown file list counts as touching;
  - `decided` called by a goal, a standing role and a chat, each recording
    a row for the right person, and Overturn waking each; the per-project
    bound erasing only rows past their day;
  - Stop asking: the change composed from a need's reason narrows that
    item alone;
  - being told: a fake push service receiving one push per blocking need,
    none for a decided one, and, while away, pushes for "always waits" and
    "even when away" needs alone;
  - the client's needs, Decided and Only you in the gallery's states.
- **The v0.1.0 story,** walked after landing, never by a plan, on a box
  through `temper ctl`, with a fake push service the box is given and a
  device registered by `temper ctl me push add`:
  1. a path set on the playground's repository and a rule naming it with
     an approval: a change touching it asks the person, and its push is
     recorded, while a change elsewhere lands;
  2. on Agents review, with Away off, an implementation question asks the
     person; with Away on, the next one is decided and listed;
  3. Stop asking on a need, accepted: the next question of that item is
     decided and listed;
  4. a goal past its budget waits, Away or not.
- **Its pages** are recorded as journeys, and the user takes them
  (`surfaces.md`, sections 4 and 10).

## 11. Open questions

- **Push or mail by default:** Web Push needs HTTPS, a deployment that can
  reach the push service, and on iOS an installed web app; mail needs the
  deployment's mail settings. Default: push where offered, mail where set,
  nothing otherwise.
- **Approving in place:** whether Needs you's card approves a change
  itself, temper posting the review as the person with a forge credential
  they signed in with, rather than leading to the forge's review page
  (`../README.md`, section 8). Default: the forge's page, one tap away.
- **The table per project or per person:** a team may want one person
  asked about a path set and another not. Default: per project, with
  approvals naming roles as they do.
- **What code decides:** a known flake's rerun (`../domain/findings.md`,
  section 5) asks no one today. Whether it is listed under Decided for you.
  Default: not listed.
- **The forge's own protection** for a path set: `../README.md`, section
  8. Default: temper's landing alone.
