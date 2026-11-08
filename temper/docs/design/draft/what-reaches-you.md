# What reaches you

Draft, 2026-11-08, from a session with the user that day. What reaches a
person, and what agents decide in their place and tell them afterwards, as
one setting of a project's, of which Away becomes a modifier. It is
`whole-story.md`'s Decide (its section 2). It revises `../domain/forge.md`,
5.8 and 5.9, where Away's kinds are four bits of the person's own and only
goals decide; `../domain/agent.md`, 4.3's `decided`; and `../client/README.md`,
3.5's Away and Needs you, and it gives 5.13's disabled *Telling you* its
mechanism. It wins where they differ until its plans fold it in. temper/ui
carries it out: the classes, Away, Needs you, Decided for you and being told;
temper/migration carries rules by path into the landing (section 7).

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
  except those marked "even when away".
- **Whoever would have asked decides** (section 4): a goal's coordinator, a
  standing role owning the area, any task whose question would reach a
  person. Each decision is a Decided row, for any task, not only goals.
- **Every need says why it reached you,** and offers to stop asking
  (section 5): a narrower rule, proposed in place, which an owner accepts
  with one press.
- **Decided for you** shows for a day on Needs you and Home, then stays in
  the task's history, with Overturn as today (section 6).
- **Rules by path** (section 7): a landing rule may name a path set, so that
  a change touching a protocol core waits for its sign-off and the rest
  lands on its own. temper's landing enforces it.
- **Only you** (section 8): sign-ins and logins to renew, listed apart; work
  goes around them.
- **Being told** (section 9): what blocks work is pushed to the person's
  phone at once, the rest comes in a digest, and Away keeps it quiet but
  for what always waits.

## 2. Three classes

A project's setting, decided by its owners and proposed by Ask temper like
any other (`../protocol/configuration.md`, 13.2; `../domain/agent.md`,
8.4.2): a table of kinds, each in one class.

- **Agents decide, then tell you:** the asker decides, and the decision is
  listed under Decided for you (section 6). Nothing waits.
- **Asks you:** the question reaches the person, and the work that needs the
  answer waits. Other work goes on.
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
| `irreversible` | rewriting history, a forced push, deleting work that hasn't landed, a store's data | always |
| `rules` | a change to this table, a landing rule, a role's rules | always |
| `other` | anything the asker can't place | always |

- **A path set is named once** in the project, with its patterns over a
  repository's tree: for instance vsr's protocol core, `include/vsr.h`,
  `src/protocol.*` and its protocol document under `docs/`
  (`whole-story.md`, 5.2). A rule by path (section 7) and the `paths` kind
  both name it.
- **A rule in words narrows a kind:** "fixes that keep the wire format" is
  for agents to follow, not for code to check (`configuring.md`, section 2),
  so a decision taken under such words is always listed.
- **`money`, `irreversible` and `rules` cover** what `other` covers today by
  the charters' words (`../domain/agent.md`, 4.3): funds, sign-ins,
  machines, a plan's go-ahead and anything hard to reverse. A proposal of
  settings and an escalation still wait for their decider, whatever the
  table says (jig's `domain/tasks.md`, section 9).

**The defaults,** set by a project's way (`stock-ways.md`, section 2) and
changed afterwards as any setting is:

| Way | Agents decide, then tell you | Asks you |
|---|---|---|
| You approve | `wording`, `retry` | `implementation`, `tests`, `dependency`, `departure` |
| Agents review | `wording`, `retry`, `implementation`, `tests` | `dependency`, `departure` |
| Draft first | `wording`, `retry`, `implementation`, `tests` | `dependency`, `departure` |

A project made before keeps what its questions route and its Away rows say
until its owners pick a way again (`../protocol/configuration.md`, 13.10).

## 3. Away, a modifier

Away stays a person's switch on a project, with its spell and its end
(`../domain/forge.md`, 5.8). Its four bits of kinds go: what it lets
agents decide is the project's table.

- **While it is on,** an "asks you" kind is decided with the asker's
  recommended choice and listed, unless the table marks it **even when
  away**. "Always waits" kinds are never decided.
- **"Even when away"** is a mark on an "asks you" item, a path set's most
  often: vsr's protocol core asks its owner even at night.
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
  reached: a goal's requester, a role's person (the owner who started it,
  `standing-roles.md`, section 2), a chat's person. The row's bounds and
  its document stay 5.9's.
- **Nothing in the engine answers for an agent:** a question that skips
  the tool waits as ever, and code decides only whether the agent may.

## 5. Why it reached you, and stop asking

- **Each need says why,** in one line under its question: the item that
  sent it and its class, from the decision that said ask: "Reached you
  because it changes vsr's protocol core, which always asks you." A need
  with no item, a question routed to its person by kind 6
  (`../protocol/configuration.md`, 13.2), says so.
- **Stop asking** is one link on the line. It opens, in place, a proposal
  of the narrowest change that would have decided this need: the item moved
  to "agents decide, then tell you"; or, for a path set, the item with words
  that narrow it ("fixes that keep the wire format"); or the set split.
  Code makes the proposal from the reason, with no model (jig's
  `domain/engine.md`, 17.6); Ask temper writes one in words when the person
  asks for something else (`../domain/agent.md`, 8.4.2).
- **An owner accepts it in place,** one press, and new questions follow it.
  The need in hand still waits for its answer: the rule changes what comes
  next, never what was asked.
- **Keep asking** says nothing changes; the line stops offering it for that
  item for a week.
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
  project's (section 2). It then covers a change only when the change's
  diff at the head being merged touches one of them; with none, it covers
  every change, as today.
- **Enforced by temper's landing:** the forge judges requirement 5, *the
  landing rules* (`../protocol/configuration.md`, 13.3), over the rules
  covering the branch and, for a rule with path sets, the paths the change
  touches at that exact head, a fact the forge reads with the change's
  review facts (`../domain/forge.md`, 8.7). A merge lands only what was
  checked, so a head that newly touches a path set waits again.
- **Its approval** is a rule's `approvals`, as today: a person's review on
  the forge, shown under Needs you while it waits
  (`../domain/forge.md`, 8.12), its line saying the path set: "Signs off
  changes to vsr's protocol core."
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
- **Quiet while away:** while Away is on, only what always waits is pushed.
  What still asks waits in Needs you for the morning.
- **Mail where push can't reach:** a browser with no push, or a deployment
  the push service can't be reached from, sends the same short line by the
  deployment's mail settings, if it has them.
- **The person's own:** which devices, the digest's hour and mail are
  theirs, kept as Away is, not a project's setting.
- **Its look** on a phone is `surfaces.md`, section 10.

## 10. Its acceptance

- **Focused and fuzzy tests in the worlds:**
  - the judgement as a pure function of kind, classes, Away, its marks and
    paths, fuzzed over all of them: an "always waits" kind is never
    decided, the strictest class wins, Away decides only unmarked "asks
    you" items;
  - in the forge's world, a rule with a path set: a change touching it
    waits for its approval, one that doesn't lands, a head that newly
    touches it waits again;
  - `decided` called by a goal, a standing role and a chat, each recording
    a row for the right person, and Overturn waking each;
  - "stop asking": the proposal made from a need's reason narrows that
    item alone;
  - being told: a fake push service receiving one push per blocking need,
    none for a decided one, and none but "always waits" while away;
  - the client's needs, Decided and Only you in the gallery's states.
- **The v0.1.0 story,** on a box through `temper ctl`, with a fake push
  service the box is given:
  1. a path set on the playground's repository and a rule naming it with
     an approval: a change touching it asks the person, and its push is
     recorded;
  2. Away on: a wording question is decided and listed, and the person
     overturns nothing;
  3. "stop asking" on a need, accepted: the next question of that item is
     decided and listed;
  4. a goal past its budget waits, Away or not.
- **Its pages** are recorded as journeys, and the user takes them
  (`surfaces.md`, sections 4 and 10).

## 11. Open questions

- **Push or mail by default:** Web Push needs a browser that allows it and
  a deployment that can reach the push service; mail needs the
  deployment's mail settings. Default: push where offered, mail where set,
  nothing otherwise.
- **The table per project or per person:** a team may want one person
  asked about a path set and another not. Default: per project, with
  approvals naming roles as they do.
- **What code decides:** a known flake's rerun (`../domain/findings.md`,
  section 5) asks no one today. Whether it is listed under Decided for you.
  Default: not listed.
- **The forge's own protection** for a path set: `../README.md`, section
  8. Default: temper's landing alone.
