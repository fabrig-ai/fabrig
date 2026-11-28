# Settings in the store

How work is run leaves the document for the store: the document keeps
what a start needs before it can read its store, and what belongs to
the machine, and everything else that configures work is a setting,
changed while temper runs and kept in the store, as projects already
are. The mechanism is jig's (jig's `domain/engine/settings.md`
; `jig/docs/design/domain/connectors.md`, section 17): a setting is a kind, a scope
and a number; the current settings are the binary's defaults with the
stored changes over them; a change is one decision, checked whole and
refused whole; charters are pinned per revision, everything else is
read at its newest. This section says what temper adds: its own kinds,
how the forge judges them, its stock defaults, and the members that go.
The pages a person meets it on are `../../client/pages/settings.md`, 2.

## 1. What leaves the document

| Member today | Becomes | Its default (`stock.md`) |
|---|---|---|
| Models in the deployment and agent files, refused under passed defaults | jig's models | temper's stock models, numbered (`stock.md`) |
| jig's `engine.charters` | jig's charters | temper's stock charters: a chat's (1), the implementer's (2), a reviewer's (4), the goal's agent's (5), the status agent's (9), a trial's (10) and Ask temper's (11) |
| jig's `engine.templates` | jig's templates | jig's chat (1) and goal (2), a goal landed whole (4), and temper's Ask temper (5) |
| jig's `engine.policies` | jig's policies projects are made from | the stock policy (1) |
| jig's `engine.rules` | jig's rules | temper's stock rules, the landing's two requirements among them (3) |
| `application.landing` | temper's kinds 1 and 2: the deployment's landing rules, and each project's (`settings-kinds.md`, 1) | each project's rule 1: CI passed, up to date and gate 1's agent review on every branch of each of its repositories; no deployment rule |
| `application.charter` | temper's kind 3: the forge's charter (`settings-kinds.md`, 1) | charter 2 |

- **What stays** is `engine.md`'s other members and jig's other parts:
  the directories, the listeners, the workers and their secrets' files,
  a worker alone's limits (the engine's are settings, `limit-settings.md`), the first owners and the sign-in providers, `clients`,
  `sign_in` and `webhooks`. The LLM providers and their endpoints
  (`engine.llm`) are jig's `provider` kind, and the playground's
  scripted provider the box's constant (`providers.md`). Of the stock's
  providers each has its `keep` and `drop` (jig's
  `protocol/configuration-engine.md`, section 1): OpenAI's keep `gpt-*` and drop `*audio*`,
  `*image*`, `*realtime*`, `*search*`, `*transcribe*`, `*tts*` and the dated
  `*-20??-??-??`; Anthropic's keep `claude-*` and drop the dated `*-20??????`. The
  playground's scripted provider has neither, and no list. jig's
  `engine.projects`,
  the projects a first start seeds, stays jig's member; `temper-box init`'s
  is `[]` (../setup/init.md, 6), and a document's seeded project names its
  policy as `policy: 1` (jig's `protocol/configuration-engine.md`, section 1),
  so the seed copies the stock's policy 1 and no document carries a
  copy of it (5, step 1). Before opening the store, the copied seed is
  checked against the stock rules and configured authority bounds, its
  references, and `core.policy_bytes`. A refusal names the policy member
  and the executor count or snapshot bytes beside the configured limit;
  an unknown reference names its member.
- **A project's own policy** is not a setting: it is in the store
  already, and its owners change it (jig's `domain/people/acts.md`, 2),
  part by part, or whole as an item of a change of settings (jig's
  `domain/engine/settings.md`, 2), so that one proposal sets a project up
  beside the deployment's settings: the charters and templates its
  work runs under, its landing rules, and its policy, with its roles,
  delegations, tasks and depth. Such a set is decided by one person
  who owns both the deployment and the project, in one decision; a
  restored version sets the policy back as it set the rest (jig's
  `domain/engine/settings.md`, 10). What a project adds to how its changes
  land is its landing rules, a setting of kind 2, decided by its
  owners.

## 3. How the landing rules are enforced

- **Two standing requirements** in temper's stock rules, on the forge's
  merge (kind 4) over `{"segments": ["forge"], "last": "open", "value":
  ""}`, every landing branch of every forge: the forge's requirement 5,
  *the landing rules*, guarded and marked as one that must be guarded;
  and its requirement 6, *up to date by the landing rules*, observed
  with a freshness of 0. Their parameters are 0. They replace the
  requirements 1 to 4 that each rule's criteria became, one per
  criterion, in the rules and in each seeded project's policy
  (`../../domain/forge/change/gates.md`, 2).
- **A rule that does not enforce is advice:** its gates, blocking or
  not, are asked for and their verdicts shown on the change's panel,
  none holding the merge, and its approvals are not asked; only rules
  that enforce hold a merge, below. Built with 5's second step, when
  the forge holds the rules itself.
- **Judged over the rules the forge holds then.** At a merge, the forge
  takes every rule covering its branch that enforces, the deployment's
  and the project's, a project's rule beginning at `branch` covering the
  branch of that name in each of the project's repositories (`settings-kinds.md`, 1), a rule
  naming path sets only where the change touches one of them at the
  merge's head (`../../domain/forge/change/gates.md`, 2), and
  its rule 1 the stock's while its owners have set none (`stock.md`):
  requirement 5 meets when each one's CI, gates and
  approvals are met, waits when one waits and none refuses, and refuses
  when one refuses; requirement 6 likewise for each one's up to date. A
  branch no enforcing rule covers meets both. The forge keeps on the
  change the rule and the criterion it last waited on or refused, which
  the change's panel shows and its hold says (`../../domain/forge/change/gates.md`,
  2): the words are the forge's, kept with its change, whatever a
  verdict carries through jig.
- **So a change of landing rules touches nothing of jig's:** no rule,
  no project's policy. A project is covered from its making by its own
  rule 1, the stock's (`stock.md`), and by the rules a deployment's owners
  add, which every project's merges meet beside its own; a project's
  rules changed reach its next merge. So a project's owners may land
  with less than an agent's review in their own project, and a
  deployment that wants a floor sets one in kind 1
  (`../../draft/archive/stock-ways.md`, 3.1).
- **jig's rules are the deployment owners' to change,** the two
  requirements among them. A change that removes one leaves landing to
  the change procedure's waits alone; the review page says so under
  "temper enforces" (`../../client/pages/settings.md`, 2).

## 4. The members gone

- **Refused.** temper passes its defaults, so `jig-protocol-config`
  reads no `engine.charters`, `templates`, `policies` or `rules` (jig's
  bridge, `jig/docs/design/domain/engine/settings-store.md`, 1, serves an application that passes
  none), and temper reads no `application.landing` or `charter`. A
  document naming one is refused as any unknown member is (`README.md`, section 1),
  naming its path and saying where it went: "`engine.charters`:
  settings live in the store; read them with `temper ctl settings show`".
  `engine.md`'s `landing` and `charter`, and `Config`'s `landing` and
  `forge_charter` fields, go with them, in 5's second step.
- **The workers are settings too** (jig's `domain/engine/workers.md`;
  `../../draft/config.md`, 6.7): `engine.hosts.workers` is refused by
  name, "workers are settings in the store", and no digest file is
  read. An engine alone's owner adds each worker with `temper ctl worker
  add` (`../../client/ctl/workers.md`); the box passes its own, `local`
  (`../cluster.md`, 1).
- **A seeded project names its policy by number.** jig's
  `engine.projects` stays, but temper refuses a project whose policy is
  inline, naming its path: "`engine.projects[0].policy`: name a policy,
  `policy: 1`; change it with `temper ctl settings apply`". A policy is
  then written in one form alone, the settings file's
  (`../../client/ctl/tools-acts.md`, 1), and the document's older forms of an
  authority, a pattern and a role's permissions (milliseconds, text
  segments) meet nothing.
- **Configuration is not imported into a store.** Removed members
  are refused by name; the binary supplies the stock, and an
  operator's changes are proposals after the start ("Tell temper how
  you work"). A store below the floor follows its named refusal
  (`../floor.md`, 4), not a settings import.
- **Across jig and temper:** jig reads the four members for an
  application that passes no defaults (its bridge, which stays jig's),
  and refuses them for one that passes its own. temper passes its own
  from 5's first step, which needs nothing new of jig's settings;
  its second needs jig's application settings.
- **Their writers name none of them:** `temper-box init`'s document and
  files (`../setup/init.md`, 5 and 6); the box, which has no document of
  its own but `temper-box init`'s (`../setup/box.md`);
  `docs/examples/owned.json` and its instructions; the cluster
  fixture's engine documents, whose stories apply their own values as a
  settings file through ctl after sign-in; the fake backends' document
  rewriter (`temper-fake`'s backends); the tests' documents and
  builders (`../../testing/checks.md`, 4). A generated document
  elsewhere, fabrig's, drops the six members; what it set differently
  from the stock values becomes a proposal after the deployment's first
  start.

## 5. In steps

The members leave in three steps, each landing whole, so that temper's
side never waits on the part of jig it does not use:

1. **jig's four kinds.** The `stock` module passes jig's builder
   charters 1 and 2, templates 1 and 2, policy 1 and the rules as
   `Config`'s defaults (`stock.md`), the rules delegating to charters 1 and 2
   and the change procedure until step 3's charter comes, and the document's
   `engine.charters`, `templates`, `policies` and `rules` are refused
   (4). The landing rules stay `application.landing` until the
   second step, so policy 1 and the rules keep the requirements 1 to 4
   each rule's criteria become today (`../../domain/forge/change/gates.md`, 2),
   derived at each start from the document as now: a document whose
   landing rules change between starts gives new defaults, which make
   a revision recorded as the release's (jig's `domain/engine/settings-store.md`,
   1). Until step 2, the rules' default carries those requirements,
   so once the rules hold a stored change, a landing rule edited in the
   document reaches no decision until the rules are reset or changed
   again; step 2 ends that, the rules' standing requirements reading
   the forge's rules (3). The document's checks that read the four members (`engine.md`:
   `application.charter` among the charters, a landing rule's
   approvals' roles in the policies) read the stock defaults instead,
   which the binary knows before the store is read. A change that
   leaves the forge's charter unoffered (dropping the rules' delegation
   to it too, which jig's references require) is accepted in this step,
   and the forge's changes are refused until it is offered again: step
   2 refuses such a change, kind 3 naming its charter (`settings-kinds.md`, 1). The
   forge's review charter is the stock's charter 4 until then, fixed by
   the binary, with no document member of its own: its settings change
   as any charter's, and step 2 makes it kind 3's number 2. `temper-box
   init`'s document
   loses the four members and its `charters/`, and every other writer
   of them (4) moves with it. A seeded project names `policy: 1`
   in place of an inline policy, so the stock reaches its first chat;
   the interim inline copy with tools 513, and the test that holds it
   equal to the stock, go with it. A document may seed no project at
   all, its first owners the deployment's alone, as jig's seed allows,
   holding no escalation role until a project's policy gives them one;
   `temper-box init`'s seeds none, its projects made at runtime on policy 1
   (../setup/init.md, 6); a world or a person that needs a wider project
   raises the rules and policy 1 with one change of settings, then makes
   it, or widens an existing project's ceiling within the new rules
   (jig's `domain/authority.md`, section 6). A deployment whose document carried
   the stock values starts on the same settings, and makes no
   revision.
2. **temper's kinds,** on jig's application settings (jig's
   `domain/engine/settings-application.md`, 1): kinds 1 to 3 (`settings-kinds.md`, 1), judged by the
   forge; `application.landing`'s rules and `charter` refused; 3's two
   standing requirements in the rules, and policy 1 without
   requirements, as `stock.md` has them. `application.landing` keeps its
   projects' ways, checks and tiers (`engine.md`) until kinds 4 and 5;
   the document's checks that read a landing rule's approvals' roles go
   with the rules, the forge judging a rule as `settings-kinds.md`, 1 says. Its writers
   (4) move with it: their landing rules become stock or a world's
   defaults, or a settings file applied after the start.
3. **Ask temper:** charter 11 and template 5 (`stock.md`), with the run's
   asking tools, `see`, `document`, `remember` and `propose_settings`' asking form
   (`../../domain/agent/tools.md` and `../../domain/agent/ask-temper.md`), and the rules' and policy 1's
   delegation to charter 11. A deployment made before takes them as new
   defaults, by a revision recorded as the release's; a project made
   before keeps its own policy, a copy (jig's `domain/engine/settings.md`, 2),
   and gains charter 11 by a proposal its owner accepts, or is made
   again, as nothing before v0.1.0 is kept compatible.

## 6. A charter's documents

A rule a person wrote down in a repository, and that only changes how
an agent behaves, reaches the runs that should follow it from where it
was written, by jig's charter documents (jig's `domain/engine/settings.md`,
9): a chat has no workspace and reads no repository's `AGENTS.md`,
so its charter names the documents, and a rule that lands reaches its
next run.

- **A document** is a branch of an adopted repository, by its resource
  name (`../../domain/forge/README.md`, section 2: `forge`, the host, the owner,
  the repository, `branch` and the branch's segments), a path, and
  optionally a heading. temper's stock charters name none.
- **The forge gathers each** for a run of the charter, as a brief
  section of its own (`../../domain/root-translation.md`, 2.2): the file at the
  branch's head as the forge resolves it when it gathers the section,
  one commit named in the section and kept for its whole life, read as
  `read_forge what=file` reads one (`../../domain/forge/README.md`, section 2). With a
  heading, the part from the first Markdown heading line whose text,
  after its `#`s and one space, equals the heading byte for byte, to
  the line before the next heading of its level or above. Cut to the
  section's allotment at the last line end within it, saying how many
  bytes were left out, as the brief's other cuts do. Its budget is
  `root.brief.budgets.document`, beside `ci`, `reviews` and `pull`
  (`limits.md`), 4,096 bytes in `small`, bounding each document section
  the forge prepares. The repository is the one adopted, in any
  project, under the name's host, owner and repository: the charter's
  owners chose its documents, and no authority is asked of the run
  (jig's `domain/engine/settings.md`, 9). A repository not
  adopted, a branch, file or heading not there, or a read that failed,
  is a section saying so.
- **In a settings file** (`../../client/ctl/tools-acts.md`, 1), a charter's value
  carries `"documents": [{"forge": F, "repository": "owner/name",
  "branch": B, "path": P, "heading": H}]`, `heading` optional, `F` the
  forge's host as its resources name it, which ctl turns into the
  branch's resource name, `forge F owner name branch` and B's
  segments, and back.
- **Ask temper** (`../../domain/agent/agents.md`, 1) proposes a charter that
  names a document, where the person keeps a rule in an adopted
  repository, in place of copying it into the charter's instructions,
  the document's repository named `OWNER/NAME` and its forge written by
  `propose_settings` (`../../domain/agent/ask-temper.md`, 2); a change landed
  there reaches the next run.

A document section begins `At BRANCH's head COMMIT`, then the selected text
on the next line. A cut appends `[N bytes cut]`, or `[more than N bytes
cut]` when the file read itself was truncated; a later cut renders the
retained text again without reading another head. Missing resources read
`Repository OWNER/NAME on HOST is not adopted.`, `No branch BRANCH in
OWNER/NAME.`, `No file PATH in OWNER/NAME at BRANCH's head COMMIT.` (a
folder too), or `No heading "HEADING" in PATH at BRANCH's head COMMIT.`. A
failed or refused read is a failed brief read, rather than one of these
successful explanatory reads (2026-11-04).

A stored document whose resource name is no forge branch renders with
`"connector"` and `"name": [segments]`, its `path` and optional `heading`,
preserving what was stored. A settings file naming a document in that raw
form is refused as out of shape; the accepted input form is the forge,
repository and branch form above (2026-11-04).

## 8. The stock ways

The ways a project can work, pre-made settings of its own scope that it
opts into (`../../draft/archive/stock-ways.md`, sections 2 and 5): data in the
`stock` module, given to the engine at each start beside the forge's
defaults (`stock.md`), so that a test world gives its own or none.

| Way | Its headline | Its line | Its items, in the project's scope |
|---|---|---|---|
| You approve | "You approve every change" | "An agent makes each change; nothing reaches main until you approve it." | kind 2's rule 1: every branch (`branch`, open: 2), enforced, CI, up to date, no gate, one approval by role 0, `clean`; rules 2 to 4 removed; kind 8: You approve's table (`../../domain/forge/attention.md`, 6) |
| Agents review | "Agents review every change, and code lands it" | "An agent makes each change, another reviews it, and code lands it." | kind 2's rule 1, the stock's (`stock.md`); rules 2 to 4 removed; kind 8: Agents review's table |
| Draft first | "Draft first, then agents build it on its own branch" | "You and an agent write a design draft; agents build it on its own branch, and it lands whole." | kind 2's rule 1: `branch`, exact `main`, enforced, CI, up to date, no gate, no approval; rule 2: `branch`, open `temper`, enforced, no CI, not up to date, no gate, no approval; rules 3 and 4 removed; kind 7: the project's own, or the stock's with none, with `drafts` and `goals` on and `way` 4; kind 8: Draft first's table; the project's policy, where its delegation lacks charter 5, the goal's closer (procedure 6), role 0, the merge job (job 3) or the close job (job 4), with them added to its ceiling's and role 0's delegation; kind 15: `charter 14`, the stock's triager (`../../domain/areas.md`, section 6). It brings one standing role, the triager (below) |

- **In this order,** from more of the person to more of their agents, as
  the page's tabs are. Each way's name, headline and line are its words,
  the stock's, carried by the read (below), so that the page and `see
  ways` say the same. Its headline is How work runs' heading while the
  project runs it (`../../client/pages/settings.md`, 2): the words saved with a
  stock way (`../../draft/whole-story.execution.md`, P3e), at most 120 bytes.
- **A way's items** are what jig's `ChangeSettings` carries (jig's
  `domain/engine/settings.md`, 2), computed by the root for one project from
  the way and that project's settings: a rule *set* to its value, each
  rule the way names none of *removed*; kind 7 *set* to the project's
  value with the way's members laid over it; kind 8 *set* to the way's
  table, whole (`Classes::stock`, `../../domain/forge/attention.md`, 6), so that a
  way sets what reaches the project's people
  (`../../draft/what-reaches-you.md`, section 2); a project's policy item
  only where its delegation lacks what the way needs. Every value is
  set whole, never reset, so that a way's values become the project's
  own and a release's new stock leaves them.
- **Nothing else:** charters, models, questions (kind 6), kind 7's
  `changes`, a charter's or the project's words (kinds 9 and
  10), a project's areas (kind 14), a repository's gates and tiers, and the project's
  landing stay as they are, so that a person's choices of models survive
  a change of way. You approve and Agents review set no kind 7: a project
  that ran Draft first keeps its goals' way, and their paths over it
  start with a goal. You approve and Agents review set no kind 15
  either: a project's triager stays as it is.
- **The standing roles a way brings** (`../../draft/standing-roles.md`,
  section 10; `../../draft/archive/stock-ways.md`, section 2): each way lists
  them, each a `WayRole { charter, words, budget }`: the charter it runs
  by, the words its spec takes, what it covers and how, and its budget
  a week, in the deployment's unit. Draft first brings the triager:
  charter 14, its words the stock's (`stock.md`), its budget the
  stock's triager budget; the other ways bring none. A way's act starts
  each of its roles the project lacks, a live task labelled 3 run by
  that charter, as `role start` does (`../../domain/agent/standing.md`,
  section 6), after its items, the person who chose the way its person;
  a role the project already runs is left as it is. The items alone, as
  a proposal or a settings file sends them, start no role: until one
  runs, kind 15's `charter 14` does not stand, and the project's owners
  triage (`../../domain/areas.md`, section 4).
- **Each way's path** over the project is `../../domain/forge/change/branch-path.md`'s
  derivation over the project's settings with the way's items laid over
  them. The way the project runs is the first whose path equals the
  project's own, stop for stop; none, where none does.
- **The forge's `ways` read** (`../../client/forge/records.md`;
  `../../client/forge/served.md`) gives each way
  with its words, its path, its items and the standing roles it brings,
  each with whether the project runs one, the way the project runs or
  none, the project's own path, the project's words (kind 10) or none,
  its rules by path (`../../domain/forge/change/branch-path.md`, *Rules
  by path*), and whether the reader may change the
  project's settings: an owner of the project, as jig's `ChangeSettings`
  asks (jig's `domain/people/acts.md`, 1). "Use this" and "Keep it" send
  its items (`../../client/pages/settings.md`, 2).

## 9. Open questions

- **A real provider's list** (`settings.md`, 1): checking the configured patterns and
  account grant against a real provider follows landing. Default: fake
  lists through the worker and agent, with the account's retained names,
  catalogue decision and unchanged accounting checked together. A fake
  establishes no real-provider result. Later pages and changes to a known
  model's figures remain jig's `domain/engine/README.md`, section 10's work.

- **Requirements an application fixes** (`settings.md`, 3). The landing rules'
  two requirements are in jig's rules, which the deployment's owners may
  change, so landing can be left unenforced by a change of rules.
  Default: changeable, and said plainly on the review page; jig could
  let an application mark requirements no change removes.
