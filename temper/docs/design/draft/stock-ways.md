# Stock ways

Draft, 2026-11-08, from a session with the user that day. Ask temper's
opening shows the ways a project can work as pictures to pick from (the
look request `ask-temper-ways`, `look-requests.md`). The user's design:
the stock ways are pre-made settings in temper's code that a project
opts into, which Ask temper's model sees and draws with its views. This
draft is what must be true for that page to work, and the order it
lands in. The user's word of 2026-11-08: the look request lands only
once all of it works, with no part of the page drawn by hand.

## 1. In one page

- **Three stock ways,** data in the `stock` module: You approve, Agents
  review, which is today's stock, and Draft first. Each is a name, one
  line and the settings of a project's scope it sets (section 2).
- **A project's way is its own:** the stock's landing rule moves from
  the deployment's kind 1 to each project's kind 2, and a project's rule
  may name a branch of each of its repositories (3.1, 3.2).
- **Draft first runs on any project:** the coordinator and the goal
  template for drafts join the stock, generic (3.3).
- **One path,** derived in code from the settings: the stops a change
  takes, drawn by the page, by Ask temper's views and by the terminal
  alike (section 4).
- **One read and one act:** `ways { project }`, and "Use this" as jig's
  `ChangeSettings` of the items that read gives (section 5).
- **Ask temper sees them:** `see ways`, and the views `way NAME`,
  `path`, `proposal` and `charter`, ahead of M2 (section 6).
- **In order:** the rules, then the ways and their paths, then the look
  (section 7).

## 2. The ways

| Way | Its line | What it sets, in the project's scope |
|---|---|---|
| You approve | "An agent makes each change; nothing reaches main until you approve it." | kind 2's rule 1: every branch (3.2), enforced, CI, up to date, no gate, one approval by role 0 at `clean`; rules 2 to 4 removed |
| Agents review | "An agent makes each change, another reviews it, and code lands it." | kind 2's rule 1, the stock's (3.1); rules 2 to 4 removed |
| Draft first | "You and an agent write a design draft; agents build it on its own branch, and it lands whole." | kind 2's rule 1: the branch `main` (3.2), enforced, CI, up to date, no gate, no approval; rule 2: branches open on `temper`, enforced, no CI, no update, no gate, as the reference's two rules (`../protocol/configuration.md`, 13.9); rules 3 and 4 removed; kind 7's `drafts` and `goals` on and its `way` template 4 (3.3); the project's policy's delegation to charter 5 where it lacks it |

- **Ordered** from more of you to more of your agents, as the page's
  tabs are.
- **A way sets how work starts and lands, nothing else:** charters
  (kind 3), models, questions (kind 6), kind 7's `changes` and `away`,
  and a repository's gates and tiers stay as they are, so a person's
  choices of models survive a change of way.
- **Their words** are the stock's, carried by the read (section 5), so
  that the page and `see ways` say the same.
- **The landing branch** is named `main`, as the reference's rules name
  it; a repository whose landing branch has another name is later.

## 3. What changes first

### 3.1 The stock's rule, per project

- **Today** kind 1's rule 1 is the stock (`../protocol/configuration.md`,
  13.4), and rules add (13.3 there): no project can land with less than
  an agent's review, and a goal's branch would wait for CI.
- **Then** kind 1 has no stock rule, and kind 2's rule 1 has one for
  every project, a constant of the code as kinds 6 and 7 have: every
  branch of each of the project's repositories (3.2), enforced, CI, up
  to date, and gate 1, an agent's review, blocking at a clean head.
  Removed or reset, a project's rule 1 goes back to it.
- **A deployment's owners** still add rules of kind 1, which every
  project's merges meet beside its own. A project's owners may now land
  with less than a review in their own project; a deployment that wants
  a floor sets one.
- **Every reader and writer** of kind 1's stock moves with it: the
  forge's held rules, the stock's table, Settings, the words of Needs
  you's approvals (`../client/forge.md`, 4.1), and the worlds and tests
  that assume it.

### 3.2 A project's rule names a branch

- **Today** a rule's pattern begins at `forge`, a host, an owner and a
  name (`../protocol/configuration.md`, 13.2), so a project's rule names
  one repository, and Draft first would take two rules a repository,
  against four a project.
- **Then** a pattern of kind 2 may begin at `branch` instead, and covers
  that branch in each repository of the project, those adopted later
  among them: `{"segments": ["branch"], "last": {"exact": "main"}}`, or
  `{"open": ""}` for every branch. One beginning at `forge` keeps its
  meaning; kind 1's begin at `forge` alone.
- **Judged** against a merge's landing branch past its repository's
  segments, the project's rules being judged for its own merges alone
  (13.3 there). `propose_settings` and Settings write either form, and
  the forge's refusal of a first segment other than `forge` admits
  `branch` for kind 2.

### 3.3 Draft first's roles in the stock

- **Today** the coordinator, charter 5, and the goal template for
  drafts, template 4, are the reference's settings file
  (`../protocol/configuration.md`, 13.9), with one project's commands,
  models and designers, and the rules' and policy 1's delegation is
  changed by hand to reach them.
- **Then** the stock holds charter 5's first edition and template 4,
  generic: the coordinator cuts the draft into changes it specifies
  itself, each into its goal's branch with questions `foreman`, and
  lands the goal whole, with catch-ups and the triage of a red on its
  goal's tier, as 13.9 says, but with no designers, no command or path
  of one project and no model named, so that its runs take the stock's
  order; template 4 as 13.9's, its own goal branch among its
  resources, at the stock's models. The rules' and policy 1's
  delegation reaches charter 5, role 0 and the merge job, as 13.9
  changes them.
- **The reference's file** keeps its own editions, charter 5's second
  with designers, charters 7 and 8, and its charters 2 and 6, applied
  over the stock as today.
- **A project made before** keeps its copy of policy 1, and Draft
  first's items add the delegation it lacks (section 2).

## 4. The path

- **Derived in code** from what applies to a project's landing branch:
  the rules of kinds 1 and 2 covering it, kind 3's charters, kind 7's
  `way` and its template, and the project's landing, by queue or by
  turn, with its tiers (`../domain/landing.md`, 13.1). The result is a
  list of stops, each its actor, a person, an agent, code or a branch's
  bar, its words, and what it opens: a charter, a rule, a gate, a tier,
  the queue.
- **A typical run, with one repair,** on each branch in this order: who
  starts, a person's "you ask" and the change's charter's "produce", or
  for a goal "draft", its template's charter's "plan" and "produce ×n";
  "check" where the rules ask CI; each agent's gate, as "review"; one
  "repair", at the first place one can occur, and "check" again after it
  where the rules ask CI; each approval, "you approve", or the role's
  name for someone else's; "update" and "resolve" where the rules ask
  up to date; "merge"; each tier following the branch, "check" on a
  goal's branch, the coordinator's repair after it while none is drawn,
  and "verify" on main. A goal's branch ends its changes' part with its
  bar, and the goal's landing into main follows by main's rules. Main's
  bar ends the path.
- **What the line leaves out:** a repair or a resolution goes back
  through the checks and judgements it answers, which the line doesn't
  repeat; the stop's card says so.
- **Under `turn`,** the turn's own steps take the queue's
  (`../domain/landing.md`, section 8).
- **No agent's stop below the line:** an update or a merge is always
  code's, whatever the settings.
- **One derivation:** the engine derives, and every client draws what
  it is given, so the page, Ask temper's views and the terminal never
  disagree.

## 5. The page's read and acts

- **`ways { project }`,** a read of the forge's family, watched: each
  stock way with its words, its path over this project and the items it
  would set here; the way the project runs, the one whose path equals
  the project's own, or none, with the project's own path; and whether
  the reader may change the project's settings.
- **"Use this"** sends jig's `ChangeSettings` of the way's items as the
  read gives them, one decision, the person's own, as Settings' change
  of one item is (`../client/README.md`, 5.13). Its refusals are the
  forge's and jig's, in words: an approval with too few eligible people
  names the forge, the role, how many it asks and how many can
  (`../protocol/configuration.md`, 13.2). A person who may not change
  settings sees no act.
- **"Keep it"** does the same for the way the project runs, so that its
  values become the project's own and a release's new stock leaves
  them. It shows while To begin's first step is left, which its change
  marks done (`../client/README.md`, 3.5); once that step is done, the
  current way has no act.
- **"Yours":** a project that runs no stock way shows a fourth tab,
  current, its own path, with no act.
- **A stop's card,** from the read: an agent's charter by name, the
  model a run of it would take now, what it may do, its instructions'
  first sentence; a person's role and what they decide; code's checks,
  the queue or the turn.
- **It replaces** the opening's "What it sets" and "Ways to begin" with
  their cards (`../client/README.md`, 3.5), and `own-way.md`'s card
  (section 2 there), whose reference document stays that draft's.

## 6. Ask temper

- **`see ways`:** section 5's read as text, by `ask-temper.md`, 5.1's
  rules.
- **Views from the same derivation, ahead of M2:** `way NAME`, a stock
  way over the chat's project; `path OWNER/NAME [BRANCH]`; `proposal
  tN/K`, its path before and after; and `charter N`
  (`ask-temper.md`, 6.2). The queue's, a task's tree and its timeline
  wait for M2's reads (`ask-temper.execution.md`).
- **Its instructions' Showing part** for them: a way asked about is
  drawn, not described, and a person who wants their own way gets a
  proposal, drawn before and after.

## 7. In order

| | Lands | Needs |
|---|---|---|
| 1 | the look requests `ask-temper-start`, `to-begin-two-steps` and `night-mode` | nothing |
| 2 | section 3: the stock's rule per project, a rule naming a branch, Draft first's roles | nothing |
| 3 | sections 4 to 6: the ways as data, the path, `ways`, `see ways` and the four views | 2 |
| 4 | the look request `ask-temper-ways`, every state its message names | 3 |

- **Each is complete when it lands,** with its focused and fuzzy tests;
  its walk follows. Steps 3 and 4 may be one plan; step 4 never lands
  before steps 2 and 3 are on main.
- **The walk after step 4:** a fresh box; To begin's Start; each tab
  and a stop's card; "Use this" on You approve, then a change waiting
  for the owner's review on Forgejo; then Draft first, and a goal from a
  chat's draft landing whole.

## 8. Its tests

- **Focused:** the stock's rule 1 per project and its reset; a branch
  pattern over two repositories, one adopted after the rule; the stock's
  delegation to charter 5; the three ways' paths over a project with CI
  and both tiers, exactly the look request's, and with neither, the
  same less "check" and the tiers' stops; `ways`'s current way and
  "Yours"; "Use this" refused for too few approvers; "Keep it"'s change.
- **Fuzzy:** a branch pattern and a full one against random branch
  names; the derivation over random rules, never an agent's stop for an
  update or a merge.

## 9. Decided

- **A way is its landing and how work starts,** not its charters or
  models, so that changing way keeps a person's model choices.
- **The stock's rule moves to each project,** rather than You approve
  keeping an agent's review, since rules add and a project's way must
  be able to loosen the stock.
- **A rule names a branch,** rather than a rule per repository, so that
  a way is data with no repository's name in it.
- **"Keep it" writes the way,** rather than a dismissal of its own, so
  that To begin reads the facts it reads today.
- **The engine derives the paths,** and clients draw them.
- **Tiers stay a repository's,** configured as today, their stops drawn
  where they exist.
