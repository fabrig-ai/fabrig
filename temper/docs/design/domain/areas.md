# Areas and their owners

Provisional, 2026-11-11, from `../draft/standing-roles.md`, sections 4,
8 and 10. A project's work is cut into areas said in words, each with
an owner: a standing role (`agent/standing.md`), a person, or a role of
the project's people. What comes in, a finding (`findings.md`), reaches
the owner of its area, or the project's triager while it has none; the
triager reads it against the areas' words and hands it on, or keeps it.
Routing by path is later (`../draft/standing-roles.md`, section 13).

## 1. In one page

- **An area** (section 2) is a project's setting, kind 14: a name, what
  it covers in words, and its owner.
- **The triager** (section 6) is a project's setting too, kind 15: the
  owner of what has no area.
- **An owner** (section 3) is a task, a party, a role of the project's
  people, or the standing role a charter runs in the project.
- **Every finding has exactly one owner** (section 4), derived, never
  stored: its area's owner, else the triager, else the project's
  owners, each where it stands.
- **Routing** (section 5): a finding filed with an area is its owner's
  at once, with no agent's run; one filed without is the triager's,
  which hands it to an area with why in a line, or keeps it.
- **An owner hears** a finding that becomes theirs: a task as news, a
  person in Needs you (section 5).
- **Acts** (section 7): anyone of the project files and reads; a
  finding's owner, or the project's owners, hands it on and triages it
  (`findings.md`, section 6).

## 2. An area

- **Its name:** 1 to 32 bytes of lowercase ASCII letters, digits and
  `-`, starting with a letter, unique in its project. A finding's label
  is `area:` and the name (`findings.md`, section 2).
- **Its words:** what it covers, 1 to 480 bytes of UTF-8 with no control
  character, read by the triager to judge where a finding goes and by
  the area's owner as its scope ("vsr's tests: the C test suite under
  `test/`, its flakes and the tiers' reds").
- **Its owner:** an owner (section 3).
- **A project's areas** are the value of kind 14, at most 16, in the
  order their owners wrote them
  (`../protocol/configuration/settings-kinds.md`, 1). With none, every
  finding is the triager's.
- **Changed as any setting:** by the project's owners, or by proposal
  (jig's `domain/engine/settings-proposals.md`, 1). Handing an area to a role, or
  back, is a change of its owner. A standing role may propose a change
  of its own area's words (`agent/standing.md`, section 5). An area
  removed leaves its findings' `area` as written, and their owner falls
  to the triager by `areas.md`, section 4's chain.

## 3. Owners

- **An owner** is one of four:
  - `task N`: a task of the project, a standing role or any other, such
    as a person's chat (`findings.md`, section 6);
  - `party N`: a person, or a service (`../client/ctl/README.md`,
    section 7), with a role in the project;
  - `role N`: a role of the project's people, whoever holds it, role 0
    its owners (jig's `domain/people/README.md`);
  - `charter N`: the project's standing role run by charter N, so that a
    way can name its triager before the role is started (section 6).
- **Where it stands,** each owner gives a holder, a task, a party or a
  role: `task N` while task N is live; `party N` while the party holds a
  role in the project; `role N` always; `charter N` while a live task of
  the project labelled 3 (`agent/standing.md`, section 2) runs under
  charter N, the one with the lowest number. An owner that does not
  stand gives none.
- **Judged when saved** (`../protocol/configuration/settings-kinds.md`, 1):
  `task N` must be a live task of the project, `party N` a party with a
  role in it, `role N` within jig's `authority.roles`, and `charter N`
  within 1 to 64, a charter's numbers (jig's `domain/engine/settings.md`,
  1), read by the root from the core as a way's template is. What stands later is `areas.md`, section 4's to read.

## 4. Who owns a finding

- **One pure rule,** `owner(area, areas, triager, stands)`, gives the
  holder of a finding from its area, the project's areas, its triager
  and what stands:
  1. its area names one of the project's areas whose owner stands: that
     owner's holder;
  2. else the triager, set and standing: its holder;
  3. else role 0, the project's owners.
- **Always exactly one,** and never stored: the forge keeps a finding's
  area, and the root reads the rule at each act and each read, so a
  change of kind 14 or 15, a task's end or a member's leaving moves the
  findings it concerns in the same decision, with nothing to reconcile.
- **A project with no standing role and no setting** keeps its people
  as the owners of everything, as at Pair: every finding is role 0's.

## 5. Routing

- **Filed with an area** of the project: its owner's at once, state
  `new`, with no agent's run. An area the project lacks refuses the
  filing, naming the project's areas. A person's word through Tell
  temper said on a standing role's page, its card on Team or its talk is
  filed with the first area that role holds
  (`../client/pages/needs-you.md`, 1, *Tell temper*, *Where it goes*).
- **Filed with none:** the triager's, state `new`. A red of no goal's
  (`after-landing.md`, section 7) and any other person's word through
  Tell temper (`../draft/whole-story.md`, section 2), its body naming the
  page it was said on, are filed so, but for a
  red whose landing brought a follow change, filed with the area its
  follow's owner owns where it owns one (`forge/follows.md`, section
  6).
- **Handed on:** the act `hand` names an area of the project and why,
  in a line of 1 to 200 bytes. It sets the finding's area, keeps it
  `new`, and its new owner hears it with the why. The triager hands on
  what it judges an area's; an area's owner may hand on what it judges
  another's.
- **Kept:** what fits no area the triager keeps, and triages as any
  owner does (`findings.md`, section 6).
- **A task hears** through the project's findings topic
  (`forge/topics.md`, 1.1): in the decision that files or hands a
  finding to a task, the root subscribes that task to `findings {
  project }` if it isn't already, as it subscribes a goal to its
  landings, and gives it the news, class `Wakes`, its wake policy
  pacing it (a standing role's news waits up to 30 minutes,
  `agent/standing.md`, section 2). Its words, cut to the deployment's
  `tasks.message_bytes`, the body first:

  ```
  Finding P/N is yours: TITLE
  KIND, SEVERITY, area NAME
  Handed on by WHO: WHY
  BODY
  ```

  `P/N` the project's and the finding's numbers; `no area` where it has
  none; the third line for a hand-off alone, `WHO` as `task N` or
  `party N`; the body cut from its end, marked `[cut]`.
- **Its subscription is apart from the task's room.** The root asks
  for it as the topic jig admits one past the task's
  `tasks.subscriptions` (jig's `domain/tasks/messages.md`, 4), so a
  holder whose own subscriptions fill that bound, as a triager waiting
  on four people's answers does in stock, is subscribed all the same,
  and no later subscription can take its place: the task's own stop at
  the bound, counting it, as they did when it came first. Its news has
  a merged slot in the task's inbox, kept from the subscription on
  (jig's `domain/tasks/messages.md`, 2). A goal a finding is routed to
  is subscribed the same way (`findings.md`, section 6).
- **A finding never waits on it.** A filing, a hand-off or a route is
  written whatever its holder's room, and refused only for its own
  (`findings.md`, section 4). Where the subscription can't be made,
  the holder's inbox having no room for its slot or the forge's
  subscriptions full (`forge.subscriptions`), the finding is written
  and its holder isn't told: it reads it with its `finding` tool's
  `list` (section 7), and its next finding asks for the subscription
  again. A task keeps the subscription until it ends. One without it,
  as a task that held findings before this rule or one whose run
  unsubscribed it, is subscribed at its next finding, under the number
  the forge keeps for it where it keeps one.
- **A person hears** in Needs you: a party's `new` findings whose holder
  is that party, or a role they hold, are listed for them by the forge's
  `findings` read with `mine` (`../client/forge/records.md`), each a card
  (`../client/pages/needs-you.md`, 1). Nothing is sent: the card
  leaves when the finding leaves `new` or stops being theirs.
- **An owner changed** by a save of kind 14 or 15: in the decision after
  the one that saved it, each task holding open findings, `new` or
  `backlog`, that it didn't hold before the save hears one news,
  `Findings on project P are yours now: N1, N2, …`, their numbers in
  order, cut to `tasks.message_bytes` with `and K more`, through the
  same subscription, made as above.

## 6. The triager

- **Kind 15,** an owner, or none. With none, or one that does not stand,
  the project's owners triage (section 4).
- **Its work:** it reads each finding that is its own against the
  areas' words and its own rules, and hands it to the area it fits,
  with why in a line, or keeps it and triages it: one short run an
  item. A project changes how it routes by changing words, the areas'
  or the triager's rules.
- **Its groups** (`../draft/home-for-a-person.md`, 5): of what it
  keeps, the triager groups the findings that one act would settle, by
  what they are, and asks its person one question a group, where its
  `decided` says Ask: "Three bugs in temper, each with a clear fix. Start
  a goal for each?" That call names the kind `findings`, starting work
  on findings (`forge/attention.md`, 6), and no paths, since its choice
  starts work and changes no file itself: so every group's need reads
  "Reached you because it starts work on findings, which asks you unless
  you’re away." on the stock's tables, Home says it once for them all,
  and its Stop asking lets the triager start goals for findings, then
  tell its person (`../client/pages/needs-you.md`, 1, *Why it reached
  you*). Where the call says Decided, the triager carries its
  recommended choice out and its person reads it under Decided for
  you, "Allowed: starting work on findings." Its question is in the line form
  (`agent/agents.md`, 2), with a `Findings:` line naming the group's
  findings and a `Group:` line its plural noun and phrase; its context
  says what they share and what the choices weigh; its choices are its
  acts over the group, the one it recommends first: "A goal for each",
  "One goal for all three", "Backlog them", "Hand them to CI's owner". It
  carries the answer out one act a finding (`findings.md`, section 6),
  with its `finding` tool and the goals it starts, and a reply in words
  as its person's direction. A finding alone, which no other shares an
  act with, is a group of one, its noun singular. Grouping is the
  triager's judgement, in its charter's words (14,
  `../../../crates/temper-engine-service-stock/src/stock/triager.md`):
  code groups nothing and acts on no group, and a group is no record. The
  client draws such a question as a group's card (`../client/pages/needs-you.md`,
  1, *A group of findings*).
- **A way names it** (`../protocol/configuration/settings.md`, 8):
  Draft first's items set kind 15 to `charter 14`, the stock's triager
  (`../protocol/configuration/stock.md`), and the way brings that
  standing role, started with it by the way's act, with the stock's
  weekly budget.

## 7. Acts

| Act | Who may | What it does |
|---|---|---|
| `file` | a party with a role in the project; an agent task of the project, by its tool | a new finding, `new`, its owner by section 4, who hears it (section 5) |
| `hand` | the finding's holder, or a party of it; a project owner | its area set, its why kept, its new owner told |
| `route`, `backlog`, `drop` | the same | `findings.md`, section 6 |
| `repair`, `blame` | the same, on red findings filed under `wait: landed` | `red-repairs.md`, sections 3 and 4 |
| `list`, `show`, `areas` | anyone of the project | the project's findings, one whole, the areas with their owners and holders |

- **A party of a holder:** the party itself; a party holding the role;
  for a task, the task alone, not its requester or its delegates. An
  outside party files and reads; it starts no work, and triages only
  what it owns (`../draft/standing-roles.md`, section 8).
- **Refused** in words: `finding P/N is not yours: its owner is
  HOLDER`; `no area NAME in project P; its areas: A, B`; `the project's
  findings are full: K new` (`findings.md`, section 9). Never for the
  holder's room (section 5).
- **Through** `temper ctl`'s `finding` group (`../client/ctl/tools.md`)
  and the agents' `finding` tool (`agent/tools.md`), each a request of
  the forge's family (`../client/forge/records.md`).

## 8. The world

- **Focused:** the owner rule on each of its three steps and each
  owner's standing; a finding filed with an area reaching its owner
  task as news with no agent's run, and a person owner's Needs you
  listing it; one filed with none reaching the triager; a hand-off
  reaching the area's owner with its why; a finding no area fits staying
  with the triager; an act by one who doesn't own it refused; kinds 14
  and 15 judged; a save moving open findings and telling their new
  owners; a triager whose subscriptions fill `tasks.subscriptions`,
  each on a person task still waiting, taking a filed finding and a
  handed-on one and hearing each through the findings topic; a holder
  whose inbox has no room for the subscription's slot keeping the
  finding unheard.
- **Fuzzy:** random areas, triagers, owners standing or not, filings,
  hand-offs and saves: after each step every finding has exactly one
  holder, the rule's, and each news went to the holder the rule gave
  in that step.

## 9. Open questions

- **A standing role handing work to another** beyond a finding, by a
  message or a change of areas (`../draft/standing-roles.md`, section
  12).
- **An area across projects,** for a follow's owner reading another
  project (`../draft/follows.md`).
