# temper's stock defaults

The binary holds them, and gives them to jig's builder as `Config`'s
defaults at each start (jig's `domain/engine/settings.md`, 1); `temper-box
init`'s document carries none of them (`../setup/init.md`, 3).

| Setting | Its value |
|---|---|
| charter 1, a chat's | the instructions in the `stock` module's `chat.txt`, which send a chat to the forge's reads, `read_branch` naming the repository, before it says a repository not in its checkout isn't there; a goal's part among them asking for its report (`../../domain/agent/agents.md`, 3), and its goal's part and its chat's calling `decided` before they ask the person anything, a goal whose words hold the line `Asks: always` too, every answer then being Ask (`../../domain/agent/tools.md`); this table holds the structure alone; models: the stock's order (below); hosts `workers`; waiting 10 s; resume; a run of 64 turns and 3,600 s, a reserve of 1 turn and 60 s; a default spend of K (below); the inspect and agents tool families, no modify, no shell; a call's timeout 10 s |
| charter 2, the implementer's | as charter 1, its instructions assembled from the stock's set of charter files, `temper/docs/charters/stock/`, `common` then `implementer`, with no parts (`../../domain/agent/charter.md`, 1; `../../draft/stock-charters.md`, 2 and 9.5), which put what a brief asks it to report in `finish`'s `body` and have a report past that bound cut and said under Left, never asked about (`../../domain/agent/charter.md`, 3), and say what a run does with a shell and without one, the charter naming none, and the modify family, but a run of 800 turns and 28,800 s (8 hours), a reserve of 1 turn and 60 s: five times the longest run fabrig's development has had, 156 turns and 76 minutes; K is unmoved, its floor of 4 units a run turn counting the chat's 64, so a coding agent's run is held by its spend, a run's own, its person funding more (`../../domain/forge/change/repairs.md`), long before its turns |
| charter 4, a reviewer's | its instructions in the `stock` module's `review.txt`; charter 2's models, hosts and run, so 800 turns and 8 hours; the inspect family and the forge's reads alone, no modify, no shell, no agents, no delegation; a review's verdict as its contract (`../../domain/forge/change/gates.md`, 1) |
| charter 9, the status agent's | the instructions in the `stock` module's `status.txt`: the few lines at the top of a project's Home, from the facts in its task (`../../domain/forge/attention.md`, 3); charter 1's models and hosts; a run of at most 4 turns and 300 s, a reserve of 1 turn and 60 s; no inspect, modify, shell or agents, and no host family. The forge's procedure delegates it, its authority holding no tools, grants, notes or delegation, and its run no workspace, so what it can do is finish; the constant `STATUS_CHARTER` names it, no setting of kind 3. Numbered past the reference's charters 5, 7 and 8 (`reference.md`) |
| charter 10, a trial's | the instructions in the `stock` module's `trial.txt`, one word and nothing else (`../../domain/trial.md`, 2); charter 1's models and hosts, so that it selects as a new chat does; a run of at most 2 turns and 120 s, a reserve of 1 turn and 60 s, so one working turn, jig's check holding a run above its reserve; no resume; no inspect, modify, shell or agents, and no host family. It has no output bound of its own, a charter having none: its words ask for one word, and its contract is a report of at most 64 bytes. The root makes its task, in no project (jig's `domain/authority.md`, 6.1), and no policy's delegation names it; the constant `TRIAL_CHARTER` names it, as `STATUS_CHARTER` does, no setting of kind 3. Numbered past the status agent's |
| template 1, a chat | jig's chat, run by charter 1; its report the deployment's `tasks.result_bytes`, read at each start as C is (below) |
| template 2, a goal | jig's goal, its executor chosen; its report as template 1's |
| charter 11, Ask temper's | as charter 1, with the `stock` module's `ask.txt` (`../../draft/ask-temper.md`, section 8, but its Showing part's later steps; `../../domain/agent/agents.md`, 1 and `../../domain/agent/ask-temper.md`), and no inspect, modify, shell or agents; its models Ask temper's order (below); its `tools` the asking and settings families, 3,072, so that its runs are declared `see`, `document`, `remember` and `propose_settings` in its asking form alone, and have no workspace (`../../domain/agent/charter.md`, 2 and `../../domain/agent/tools.md`); it names no repository, and its instructions delegate nothing and effect nothing. Numbered past the trial's |
| charter 5, the goal's agent's | its instructions and parts assembled from the stock's set of charter files, `temper/docs/charters/stock/` (`../../domain/agent/charter.md`, 1; `../../draft/stock-charters.md`, 2 and 9): its instructions `common` then `goal`; its parts `designer`, on `first`, then `coordinator`'s three sections in order, on a change's questions, on results, escalations and words, and on news and timers, the first two naming `first` too; so its first run, the designer, reads `common`, `goal`, `designer` and the parts on questions and on results, and each later wake, the coordinator, a new session for one reason, reads `common`, `goal` and the parts its reasons meet, each once, in order (jig's `domain/tasks/messages.md`, 6); it designs its goal's draft and delivers the design straight to its goal's branch, cuts it into changes it specifies itself, each into its goal's branch with questions `up`, writing its design's status up front, each entry a change will close tagged with it (fabrig's `DESIGN.md`, section 2), answers what they ask, and triages a red on its goal's tier, while its goal's closer lands the goal whole (template 4; `../../domain/forge/branches.md`, 2), with no catch-ups, no command or path of one project and no model named, so that its runs take the stock's order; models: the stock's order; hosts `workers`; its runs starting fresh, as a charter in parts' must (jig's `domain/tasks/messages.md`, 6), and waiting 0, parking without retaining a worker slot or program session; the root admits zero waiting; charter 2's run, 800 turns and 8 hours, a reserve of 1 turn and 60 s; inspect, modify and agents, no shell; charter 1's host families; no documents; the coordinator's wake policy (`reference.md`): questions, failures, the last delegate's result or a failure (`last_or_failure`), its delegates' and its person's words at once, news at most once in 30 minutes, its parts covering every kind that policy wakes it for, `words` among them (jig's `domain/tasks/messages.md`, 6). Numbered as the reference's, whose settings file sets its second edition over it (`reference.md`) |
| template 4, a goal for a draft | `reference.md`'s: named `goal for a draft`, run by charter 5; ended by temper's goal closer, `ended_by` `{"closer": {"connector": 1, "code": 6}}` (jig's `protocol/configuration.md`, 3.2; `../../domain/forge/branches.md`, 2); no LLM choice, so that its runs take the stock's order; tracked at priority 1; its report template 1's; the creator's budget and deadline; labelled as template 2; *its own goal branch* among its resources (jig's `protocol/configuration.md`, 3.3; `../../client/ctl/tools.md`, 3) |
| template 5, Ask temper | a chat named `ask`, flagged conversation and labelled as jig's chat is, its contract and budget template 1's, run by charter 11. Numbered past the reference's template 4 (`reference.md`) |
| charter 12, the drafter's | the instructions in the `stock` module's `drafter.txt`, every byte as written (`../../draft/surfaces.md`, 11.2): a design session with its person, the draft written as the chat's document (`../../domain/agent/agents.md`, 3), one question at a time as numbered choices, its open points under `## Open questions`, a session taken up again from its draft and first open point, and nothing started or built; charter 1's models, hosts, waiting, resume, run, reserve, spend and call timeout; agents, so that its model's sub-agents survey where it has them, but no inspect, modify or shell, so that its runs are given no repository (`../../domain/agent/charter.md`, 2) and read the project's code through the forge's reads alone; its `tools` every family but delegate, settings and asking, 1,022, so that its runs are declared no `goal`, `change` or `delegate` (`../../domain/agent/tools.md`; 8.5). Numbered past Ask temper's |
| template 6, a design session | a chat named `design session`, flagged conversation, kept and labelled as jig's chat is, its contract, budget and grace template 1's, run by charter 12, so that a session is listed among the project's chats and, opened again, resumes from its draft (`../../domain/agent/agents.md`, 4; `../../client/pages/README.md`). Numbered past Ask temper's template 5 |
| charter 13, a standing role's | the `stock` module's instructions for a standing role, kept as charter 5's are, its `## shared` section the instructions and each `## part, on: …` section one part, as charter 5's (jig's `domain/tasks/messages.md`, 6): a standing role owns the work its spec's words name for its project's life, writes its Now, the note described `now` in its own scope, before every park, starts goals and changes for its work, answers its own work's questions, calls `decided` before asking its person, proposes changes to its own rules as a proposal of settings and never accepts one, and parks rather than finishes; a part on `first` alone, to read its spec, subscribe to what concerns its work and set its timers, and a part on every other kind that chooses parts, its work's (`../../domain/agent/standing.md`, section 2); models: the stock's order; hosts `workers`; its runs resuming, in sessions (jig's `domain/tasks/standing.md`, section 5), and waiting 0, parking as soon as it waits; charter 2's run, 800 turns and 8 hours, a reserve of 1 turn and 60 s; no workspace, so no inspect, modify, shell or agents: it reads with the forge's reads; no documents; its wake policy below |
| template 7, a standing role | named `standing role`, run by the charter its creator names, charter 13 from `role start` unless another is named (`../../client/ctl/tools.md`, 3); `ended_by` `standing` (jig's `protocol/configuration.md`, 3.2), not a conversation, listed, kept, untracked; labelled 3; its report template 1's; the creator's budget, a period's amount, and no deadline; no LLM choice, so that its runs take its charter's models |
| policy 1 | no requirements; owner 0 is the escalation role; forge admin (3) maps to maintainer 1, write (2) to member 2, read (1) to observer 3; no forge permission maps to owner. Maintainer: owner authority and period spend, requests 255 (no policy changes), decisions 31. Member: owner task authority, period spend 24K, requests 185 (create, amend, cancel, release, watch), no decisions. Observer: watch alone (128), no decisions, spend, tools, models, grants or delegation. The project and owner: role 0 requests and decides everything, a period's spend of 80 × 24K, 1,920K, for role 0 and of 81 × 24K, 1,944K, for the project, an authority's spend of 24K (below), with the rules' grants and delegation; its ceiling's and role 0's tool families 3,975, delegation, messages, control, reads, settings, notes, recall and asking (jig's `ToolFamilies` Delegate, Message, Control, Read, Settings, Note, Recall and Ask), so that a chat reads the forge it works on, answers its delegates' questions and keeps its document, a goal writes its report (`../../domain/agent/agents.md`, 3), a goal's `change` names its plan's members (`../../domain/agent/tools.md`, 1), and an Ask temper chat (template 5) reads the settings through the asking family's `see` and proposes, a proposal changing nothing until an owner decides it (jig's `domain/engine/settings.md`, 6; `../../domain/agent/agents.md`, 1 and `../../domain/agent/ask-temper.md`); its ceiling's and every role's note scopes a goal's, `Scopes(1)`, but an observer's, none, so that a chat writes its own scope and only it (jig's `domain/engine/briefs.md`, section 2; `../../domain/agent/agents.md`, 3); a goal has its own scope for its report, while an advisor and a change have none (`../../domain/agent/tools.md`, 2); its ceiling's and role 0's delegation 384 tasks 32 deep, jig's `standard` tree and depth, so that one goal's tree holds five times the largest fabrig's development has made, 73 tasks, and five times its depth, 5 |
| the rules | today's: tools 3,975, delegation, messages, control, reads, settings, notes, recall and asking, note scopes a goal's, the forge's grants under `forge`, delegation to charters 1, 2, 4, 5, 9, 11 and 12, the change procedure, the goal's closer (procedure 6), role 0, the merge job (job 3) and the close job (job 4), so that a goal for a draft runs and closes on any project (`../../draft/stock-ways.md`, 3.3), 384 tasks 32 deep, a spend of 1,920K and a period's spend of 1,944K, a run's spend from 2C to K (below); and `settings.md`, 3's two requirements |
| kind 1 | no rule: the rules a deployment's owners add, which every project's merges meet beside its own (`settings.md`, 3) |
| kind 2, rule 1 | for every project: every branch of each of its repositories (`branch`, open: `settings.md`, 2), enforced: CI passed, up to date, and gate 1, an agent's review, blocking, fresh at a clean head; no approvals (`../../draft/stock-ways.md`, 3.1) |
| kind 3 | number 1, the change's charter: charter 2; number 2, its reviews' charter: charter 4; no number 3, the foreman being retired (`settings.md`, 2) |
| kind 6 | none for every project: a change asks no one unless its maker names `up`, the one other route (`../../domain/forge/change/questions.md`, 1) |
| kind 7 | for every project: `drafts` true; `goals` and `changes` false; `way` 0, so that a fresh project shows Home, Needs you and Chats and its chats can write drafts, which it can turn off |
| kind 8 | for every project: Agents review's table (`../../domain/forge/attention.md`, 6): `wording`, `retry` and `tests` decided and listed, `implementation`, `dependency` and `departure` asking, which Away may decide; no path set |
| a period's span | 7 days, from each project's first period, renewed by the calendar (`../../domain/routing.md`, 7), so policy 1's period spend of 1,944K is a week's. A constant of the stock until `settings.md`, 5's second step, which makes it a setting of temper's, the deployment's |

- **Kind 9, each stock charter's words,** given to the forge at each start
  beside kind 3's, each a line in the person's terms, as How work runs'
  Roles says it after when the role runs (`settings.md`, 2):

  | Charter | Its words |
  |---|---|
  | 1, a chat's | talks with you and asks for the changes |
  | 2, the implementer's | makes each change, repairs it and resolves its conflicts |
  | 4, a reviewer's | reviews each change at its head, and holds it until it approves |
  | 5, the goal's agent's | designs a goal's draft, cuts it into changes, and answers what they ask |
  | 9, the status agent's | writes the few lines at the top of Home, from what landed and what waits |
  | 10, a trial's | answers one word, so that a new model is known to run |
  | 11, Ask temper's | explains how work runs here, and proposes changes to it |
  | 12, the drafter's | writes a draft with you, one question at a time, and starts nothing |

  The reference's charters 7 and 8 have none, so Roles says them by their
  instructions' first sentence (`reference.md`).
- **A project's kinds 6, 7, 8 and 10 have no row in the stock:** a project made
  later has its own number, and a stock default names one scope (jig's
  `domain/engine/settings.md`, 5). So their stock is a constant of the code, the
  value of a setting with none stored: the forge answers no value, and
  who reads it, the root for kind 6, the client for kind 7, the forge's
  `decided` and the client for kind 8, and for a run's `settings` read
  (`../../domain/agent/agents.md`, 1) the root for all three, takes the stock.
  Kind 10's stock is none: a project's words are its own, and with
  none How work runs says its way's (`settings.md`, 8).
- **A project's rule 1 is given to the forge,** for the same reason, as
  every project's (`../../draft/stock-ways.md`, 3.1): the stock module
  gives it at each start beside the forge's other defaults, an entry of
  kind 2 for no one scope, of which the forge tells jig nothing, so that
  a test world that gives its own defaults gives its own or none. The
  forge holds it as each project's rule 1 while the project has no value
  of its own set, a removal or a reset of its rule 1 included, among the
  rules a merge into one of the project's repositories meets (`settings.md`, 3), and
  answers it as that setting's value, to the client's read and to a
  run's `settings` read alike; the root takes it where jig knows no
  setting of that number, as it takes kinds 6 and 7's.

- **Each charter's host families** (its `tools`, jig's
  `domain/engine/executors.md`, 1): charter 1, a chat's, names every host
  family but settings; charter 5, the goal's agent's, names the same
  families; charter 2, a
  coding agent's (a change's producer, repairs and resolutions, and the
  implementers a goal's agent delegates), names the read and message
  families alone, since it reads the change and CI, asks its requester,
  and delivers through smith's own `deliver`, never delegating or
  effecting; charter 4, the reviewer's,
  names the read family alone, as
  its row says; charter 9, the status agent’s, and charter 10, a trial’s, name none; charter 12, the drafter's,
  names every family a chat's does but delegate, so that it starts nothing; charter 13, a standing role's, names every family a chat's does and settings, so that it proposes changes to its own rules, and is declared the settings family's tools as a deployment's own charter naming it is; charter 11, Ask temper's, names asking and settings, and is
  declared its own tools alone (`../../domain/agent/tools.md`), so that no stock
  charter is declared the settings family's `settings` and the configuring
  form of `propose_settings`, which a charter of a deployment's own naming
  that family is. As numbers, jig's
  `ToolFamilies` bits: charters 1 and 5 1,023, every family but settings and asking;
  charter 2 514; charter 4 512; charters 9 and 10 0;
  charter 11 3,072, asking 2,048 and settings 1,024; charter 13 2,047,
  charter 1's and settings'; charter 12 1,022,
  charter 1's less delegate's 1. A charter that does not name the delegate
  family is declared none of its tools (`../../domain/agent/tools.md`), so
  charters 2, 4, 9, 10 and 12 are declared no `goal`, `change` or
  `delegate`.
  smith's host part counts a run's brief beside its tools, so a brief
  is rendered within the room its charter leaves. At `temper-box init`'s
  models the host part's bound is that of their smallest window, which
  init's scripted model sets at `gpt-6-luna`'s (`../setup/init.md`, 3): it
  holds the largest stock charter's non-brief host part, charter 5
  with its Report contract, instructions and all four parts,
  and the control family's plan form of `change`
  (`../../domain/agent/tools.md`, 1), with the authorised brief, whose task
  section holds a whole spec of jig's `standard` (`limits.md`), and its 64
  sections' titles and framing, with margin; `temper-service`'s step
  tests hold the figures. The stock brief fits whole; cutting from its
  last section is the fallback.
  Each charter leaves at least 2,048 bytes
  (`../../domain/agent/tools.md`). A run is offered a family's tools
  where its charter names the family and its task's authority holds it,
  and each charter's offered host part is checked against smith's bound
  (`../../domain/agent/tools.md`).
- **The stock charters' wake policies** (jig's `domain/tasks/messages.md`,
  3): charter 5 uses the coordinator's policy (`reference.md`), the last
  delegate's result or a failure, words and notices at once, news paced
  to 30 minutes, with questions, answers and timers, and its parts offer each of those
  kinds to the run it wakes, a task's words among them, so that no kind
  it wakes for is left unread to wake it again after the run (jig's
  `domain/tasks/messages.md`, 6). Every other stock charter uses jig's
  default: words, notices and news at once, the last delegate's result
  or a failure, questions, answers and timers. A deployment sets another
  on the charter as any of its members. Charter 13, a standing role's,
  wakes at once on words and notices, on questions, answers and timers,
  and on each delegate's result (`each`), and on news paced to 30
  minutes; its part on its work offers each of those kinds, so that a
  session begun by any of them is offered the rest
  (`../../domain/agent/standing.md`, section 2).
- **Ask temper is a setting** like any other: template 5 starts it, from
  `chat start --template 5` (`../../client/ctl/tools.md`, 3) and from the
  client's Ask temper, in the rail and the tab bar, which To begin's
  first step opens (`../../client/pages/README.md` and `../../client/view.md`, 3), charter 11 runs it
  (`../../domain/agent/ask-temper.md`), and an owner may change or remove it as
  any template. Its runs read and propose settings; they never change
  one, since jig's `settings` family proposes and an owner accepts
  (jig's `domain/engine/settings.md`, 6).
- **Template 4's work,** kind 11 (`settings.md`, 2), at scope 0 and number 4: no
  design and no close-out, since the stock's charter 5 designs the
  draft itself, delivering its design and status straight to the goal's
  branch, and cuts it into changes it specifies itself, with no
  designers delegated, while its closer settles the status's tags as it
  lands the goal (`../../draft/stock-ways.md`, 3.3, and
  section 4, "plan" in the stock's). So Draft first's path over the stock
  draws "plan" and no close-out, as the look request's does. No other
  stock template has a value of kind 11.
- **Built at each start** by `temper-service`'s `stock` module, as
  typed values, the instructions held in the binary, charters 5 and
  2's compiled in from `temper/docs/charters/stock/` and assembled by
  `temper-charter-set` (`../../domain/agent/charter.md`, 1). Those
  files are code to every gate: compiled in and read by tests, a change
  to them lands through a goal like code, never by hand as
  documentation, Markdown though they are, and they name no model, no
  program and no project's paths (`../../draft/stock-charters.md`, 3); `Service` passes
  jig's kinds' defaults, with the three kinds' declarations (number,
  scope, the forge's connector, the bounds above), to jig's builder in
  place of the document's members, and temper's kinds' defaults (kind 3,
  template 4's kind 11 and every project's rule 1 of kind 2) to the forge (jig's
  `domain/engine/settings.md`, 5), and
  makes the engine as today. A test world builds the service with
  defaults of its own in that field's place, the settings its stories
  need, so that a story not about settings makes no change to reach
  them; the binary always passes the stock.
- **Budgets in runs of the costliest model.** The stock computes C at
  each start: one worst-case completion of the costliest stock model
  among the defaults, in the deployment's units, as jig prices it
  for a run's admission (its window and output at its costliest
  prices), since a person may pick any of them and a charter's order
  falls to any of them: sized by the first alone, K would admit no run on
  Astra 6 where Opus 5.5 comes first. At `temper-box init`'s models that is
  Fable 5.1's and Astra 6's, 10,896, so K is 43,584 and a chat's 24K
  1,046,016. A run is admitted only with two of them free (its reserve,
  reserve turns 1 + 1), so its allowance is K = 4C: its admission and
  room for its working turns; and at least four units for each of a
  charter's run turns (256 for the chat's 64), so that where C is small
  and prices coarse, smith's one-unit floor per completion still leaves
  a run its turns. A delegate is given half of what its
  parent has left (`../../domain/agent/tools.md`, 1, for `change`; a goal
  likewise), so the stock sizes for the work it is used for: a change 5K, its six tasks (a producer, a review, two
  repairs and their reviews) run one after another, each returning what
  it leaves, so that a change whose runs each spent a whole K would be
  held for its budget at its sixth, naming it (`../../domain/forge/change/repairs.md`
  ), and funded by a release or `temper ctl project fund`; an advisor K; a coordinator 11K, its own run and a change; a
  chat 24K, its run, an advisor and two coordinators; the ceilings and
  the owner's and a maintainer's pools 80 × 24K, 1,920K, so that eighty chats or goals, each
  taking a chat's 24K as allowed, are funded at once from the owner's
  pool, five times the fourteen fabrig's development has had at once. The
  project's period and the rules' period spend are one chat's more, 81 × 24K,
  1,944K. A pool reserves its whole budget from the period when it opens, and
  what the deployment itself starts draws on what no pool reserved
  (`../../domain/forge/attention.md`, 3), so that 24K stands outside the pools, and what
  a person may spend is the same as without it. A deployment whose models change
  between starts gets new defaults, a release's revision (`settings.md`, 5, step 1).
- **A fine unit.** smith charges each completion at least one whole
  unit, so `temper-box init` states its models' prices in thousandths of a
  dollar per 1,000,000 tokens, that floor costing a run nothing it
  needs; C, and so every budget above, scales with it.
- **The stock's models,** the binary's, as read from the providers' pages on
  2026-11-06, numbered in this order and for good, the scripted model last, 8,
  its prices its own (`../setup/init.md`, 3). A row whose provider the document
  doesn't configure isn't among the defaults, so the scripted model is there
  where `demo` is. Each has a window of 1,048,576 and an
  output of 8,192, whatever a provider allows; takes the efforts `low`,
  `medium`, `high`, `xhigh` and `max`, low to high, so that a picker has
  one scale, where Terra and Luna also take OpenAI's `none`; and is
  priced for cached input as for input, so that a budget stops early
  rather than late, its provider's page giving no figure on the stock's
  terms; and each but the scripted model declares a prompt cache's life
  of five minutes, `cache_life_ms` 300,000, its provider's default, so
  that a standing role's sessions resume within it, the scripted model
  declaring none (jig's `domain/tasks/standing.md`, 5.3):

  | `name` | `shown` | Provider | `effort` | Input, output |
  |---|---|---|---|---|
  | `claude-opus-5-5` | Opus 5.5 | Anthropic | `high` | 4,000, 20,000 |
  | `claude-sonnet-5-5` | Sonnet 5.5 | Anthropic | `high` | 2,000, 10,000 |
  | `claude-fable-5-1` | Fable 5.1 | Anthropic | `high` | 10,000, 50,000 |
  | `gpt-6.1-sol` | GPT-6.1 Sol | OpenAI | `high` | 2,000, 10,000 |
  | `gpt-6-astra` | GPT-6 Astra | OpenAI | `high` | 10,000, 50,000 |
  | `gpt-6.1-terra` | GPT-6.1 Terra | OpenAI | `medium` | 1,000, 5,000 |
  | `gpt-6-luna` | GPT-6 Luna | OpenAI | `medium` | 100, 500 |
- **The box's limits for that work.** `temper-box init`'s document, and
  `docs/examples/owned.json`, name the `standard` profiles, jig's and
  temper's (`limits.md`; jig's `protocol/limits.md`),
  sized for a project of fabrig's scale: its own development,
  measured over three days (1,834 Codex runs and 636 Claude
  calls), with five times the largest figure seen, rounded up, so that
  a fresh box runs such a project with no override (the user's
  decision, 2026-11-03):

  | What | Largest seen (median) | The default |
  |---|---|---|
  | tasks in one goal's tree, over its life | 73 (2) | `tasks.tree_tasks` 384; the stock's delegation 384 |
  | depth of the work tree | 5 | `tasks.depth` 32; the stock's delegation 32 deep |
  | tasks made in one atomic creation (a plan's tasks at once) | 30 | `tasks.batch` 160 |
  | live tasks in a deployment | 25 (3) | `tasks.tasks` and `tasks.project_tasks` 128 |
  | live tasks under one goal | 15 | 80, within the project's 128 |
  | live goals at once | 14 | `tasks.project_goals` 80 |
  | live direct delegates of one task | 15 | `tasks.delegates` 80, `people.operations` 81 |
  | questions per change or task | 15 (1) | `forge.change_policy.questions` 80; a task's inbox, `tasks.inbox_messages` 82 |
  | runs per task (repairs, retries) | 14 (1) | a change's `repairs`, `resolutions`, `updates` and `tasks` 80 |
  | turns in one run | 156 (37) | the stock's 800; the agent's `max_turns` 800 |
  | one run's wall time | 76 minutes (8) | the stock's 8 hours |
  | tool calls in one model response | 9 (1) | the agent's `calls_per_response` 48 |
  | sub-agents at once in one run | 5 | the agent's `conversations` 26, a run and 25 |
  | a task's spec, a message | 19 kB (5.9 kB) | `tasks.spec_bytes` 131,072; `tasks.message_bytes` 262,144 in the profile, since a result is a message in its parent's inbox; the box's, what its agent takes (below) |
  | a report (a result) | 46 kB (4.9 kB) | `tasks.result_bytes` 262,144 in the profile; the box's, its message bound; templates 1 and 2's report, the deployment's result bound |
  | a run's brief | 200 kB (55 kB) | what the host part leaves at `temper-box init`'s models (`../../domain/agent/tools.md`) |
  | chats or goals funded at once from one owner's pool | 14 | 80 (the budgets, above) |
  | live changes in a project | 18 | `forge.changes` 128, `forge.holds` and `forge.tasks` 128 |
  | conflicted paths in one merge | past 4, the bound then, on a goal's merge (2026-11-11) | `checkout.conflicts` 512 and `git_component.stdout_bytes` 131,072 in both profiles, for a merge of a few hundred conflicted paths (`limits.md`, the checkout's capacity) |

  The stock and the reference's whole delegation (`reference.md`) fit the
  sixteen entries ctl's settings form takes in an authority's list.
  A limit the table does not name moves with the ones it is coupled to
  (a store row, a carrier, an inbox), sized the same way or the least
  the start's checks take. jig's and temper's stored formats changed in
  place for these figures, before the first release (jig's
  `protocol/store/records.md`, 1 and 3). The document's overrides say only
  what differs from the profiles: temper's client's documents
  (`documents.rows` 2, `documents.text_bytes` 512; `../setup/init.md`, 1), the agent's carriers
  (`settings.md`, 7; `../setup/init.md`, 1) with the bounds that follow its message
  (`../setup/init.md`, 1), the forge's repositories and output (`../setup/init.md`,
  3), and the machine's
  concurrency, which is the machine's to size: the worker's 4 slots, so
  that two chats and a goal's agents run side by side, which the hubs'
  and the fleet's slots in `standard` already hold, with
  `checkout.workspaces` and `git_component.operations` 4, the least they
  take (`../../domain/worker/README.md`, section 5); and, at what the box had, the
  clients' streams, `clients.streams` and `clients_http.streams` 8,
  each with four ordinary exchanges beside it in what carries it
  (`clients.exchanges`, `clients_http.calls` and temper's server
  connections, 12), since every `temper ctl` call holds a stream and a
  wait holds it for the wait, so a person and a walk hold three to five
  at once, and the engine's watch rooms follow them (`limits.md`). One call's arguments are bounded by its model's output, not
  by the message bound: the agent's `tool_payload`, and
  `checkout.message_bytes` with it, is the largest smith admits at
  init's models (`settings.md`, 7). smith offers a run a message within the same
  payload, two bytes less for its separator, and the fleet holds a
  host's agent to the engine's message bound at its hello (jig's
  `protocol/limits.md`), so the box's `tasks.message_bytes`
  and `tasks.result_bytes` are its agent's message, the payload less
  two, overridden below the profile's 262,144: a report or message of
  the table's size reaches a run only from a model whose output admits
  that payload (`settings.md`, 7).
- **The stock sets floors under the limits.** Its own values must fit
  the limits they are checked against: a configuration whose
  `core.policy_bytes` is below policy 1's encoding, or whose
  `authority.grants` is below the rules' grants, is refused at the
  start, naming the limit and the stock setting. The `core.run_bytes`
  floor measures the resolved charter, including its parts and catalogue
  models; charter 5 at init's catalogue needs about 27,000 bytes, its
  four files' 24 KB and its models', which `temper-service`'s floor test
  holds exactly. The root holds a run's charter within
  `root.journal.run_bytes`, 65,536 in both of temper's profiles, the
  reference room's (`reference.md`), so that a fresh box admits the
  stock and an overlay of a few KB over it
  (`../../draft/stock-charters.md`, 3); the larger charter 5 leaves its
  brief less of the host part, and `standard`'s
  `root.brief.brief_bytes` follows that room by the rule above. A start's
  problem about a stock setting names it as ctl does, `charter 1.hosts`.
- **The stock charters' models are their first choices and the rest:** a
  stock charter names its first choices, those among the defaults, then
  "the rest", so that a model the catalogue gains needs no charter's change
  (jig's `domain/engine/settings.md`, 11). A task
  given no LLM choice takes the first with an account usable now, at
  that model's default effort (jig's `domain/accounts.md`, 4.3):

  | Charter | First choices, in order | Then |
  |---|---|---|
  | charter 11, Ask temper's | `claude-opus-5-5`, `gpt-6-astra`, `gpt-6.1-sol` | the rest |
  | every other stock charter | `claude-opus-5-5`, `gpt-6.1-sol` | the rest |

  A release whose models differ gives new defaults, which make a revision
  recorded as the release's (jig's `domain/engine/settings.md`, 7); a model a
  person changed or turned off stays as they left it.
- **A model of the catalogue** changes as any setting does: a proposal, which
  Ask temper can make, or an owner's act on Settings, Models
  (`../../client/pages/README.md`). A task keeps the model it has. A model added
  by name takes its provider's dearest stock model's figures, marked estimated
  (jig's `domain/engine/settings.md`, 11).
- **A release that changes a default** reaches every deployment that
  kept it. A setting no change is over follows the binary's default, so
  at the next start the new default makes a revision recorded as the
  release's, with no proposal (jig's `domain/engine/settings.md`, 7): new
  work takes it, running work keeps the revision it runs under (jig's
  `domain/engine/settings.md`, 3), and the release's notes name each default
  it changed. A setting a deployment changed keeps its change; a reset
  removes the stored change, as a change of its own, and the setting
  follows the default again.

## 1. Open questions

- **A landing branch with another name than `main`** (`settings.md`, 8;
  `../../draft/stock-ways.md`, section 2): Draft first's rule 1 names
  `main`, so over a repository whose landing branch is named otherwise
  its rules cover no landing branch, and its path shows none of their
  stops there. Default: later, as the draft has it.

- **A chat checking the forge before declaring a repository absent**
  (`stock.md`, charter 1): the stock directs `read_branch` for a repository
  outside the checkout. Default: the instruction and its phrase assertion;
  a real-model observation follows landing, with no new tool or authority.

- **Ask temper's real-model walk** (`stock.md`): the small chat's walk follows
  landing; the request bound covers
  temper's parts, not Claude Code's own MCP-attached overhead.
