# temper's stock defaults

The binary holds them, and gives them to jig's builder as `Config`'s
defaults at each start (jig's `domain/engine/settings.md`, 1); `temper-box
init`'s document carries none of them (`../setup/init.md`, 6).

## 1. Settings and their defaults

Moved to [`stock-settings.md`](stock-settings.md), section 1.

## 2. Stock charters and project settings

- **Kind 9, each stock charter's words,** given to the forge at each start
  beside kind 3's, each a line in the person's terms, as How work runs'
  Roles says it after when the role runs (`settings-kinds.md`, 1):

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

  A charter a deployment adds has none, so Roles says it by its
  instructions' first sentence.
- **A project's kinds 6, 7, 8 and 10 have no row in the stock:** a project made
  later has its own number, and a stock default names one scope (jig's
  `domain/engine/settings-application.md`, 1). So their stock is a constant of the code, the
  value of a setting with none stored: the forge answers no value, and
  who reads it, the root for kind 6, the client for kind 7, the forge's
  `decided` and the client for kind 8, and for a run's `settings` read
  (`../../domain/agent/agents.md`, 1) the root for all three, takes the stock.
  Kind 10's stock is none: a project's words are its own, and with
  none How work runs says its way's (`settings.md`, 8).
- **A project's rule 1 is given to the forge,** for the same reason, as
  every project's (`../../draft/archive/stock-ways.md`, 3.1): the stock module
  gives it at each start beside the forge's other defaults, an entry of
  kind 2 for no one scope, of which the forge tells jig nothing, so that
  a test world that gives its own defaults gives its own or none. The
  forge holds it as each project's rule 1 while the project has no value
  of its own set, a removal or a reset of its rule 1 included, among the
  rules a merge into one of the project's repositories meets (`settings.md`, 3), and
  answers it as that setting's value, to the client's read and to a
  run's `settings` read alike; the root takes it where jig knows no
  setting of that number, as it takes kinds 6 and 7's.

## 3. Charter tools and wake policies

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
  its row says (`stock-settings.md`, section 1); charter 9, the status agent’s, and charter 10, a trial’s, name none; charter 12, the drafter's,
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
  init's scripted model sets at `gpt-6-luna`'s (`../setup/init.md`, 6): it
  holds the largest stock charter's non-brief host part, charter 5
  with its Report contract, instructions and all four parts,
  and the control family's plan form of `change`
  (`../../domain/agent/tools-change.md`, 1), with the authorised brief, whose task
  section holds a whole spec of jig's `standard` (`limits.md`), and its 64
  sections' titles and framing, with margin; `temper-service`'s step
  tests hold the figures. At the stock agent's 48,000-token output,
  the brief's bound, `standard`'s `root.brief.brief_bytes`, is the room
  that host part leaves, less 1,472 bytes for its 64 sections' titles
  and framing and 66 bytes spare: charter 5's text and the bound move
  together, by as many bytes, the bound the other way. The stock brief
  fits whole; cutting from its last section is the fallback.
  Each charter leaves at least 2,048 bytes
  (`../../domain/agent/tools.md`). A run is offered a family's tools
  where its charter names the family and its task's authority holds it,
  and each charter's offered host part is checked against smith's bound
  (`../../domain/agent/tools.md`).
- **The stock charters' wake policies** (jig's `domain/tasks/messages.md`,
  3): charter 5 uses the coordinator's policy, the last
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
  (`../../domain/agent/standing.md`, section 2). Charters 14 to 16, the
  stock's standing roles, take charter 13's policy, its models, hosts,
  run and tools, so that they differ by their instructions alone; the
  rules' and policy 1's delegation reach them as they reach charter 13.

## 4. Ask temper and its template

- **Ask temper is a setting** like any other: template 5 starts it, from
  `chat start --template 5` (`../../client/ctl/tools-acts.md`, 1) and from the
  client's Ask temper, in the rail and the tab bar, which To begin's
  first step opens (`../../client/pages/README.md` and `../../client/view-navigation.md`, 1), charter 11 runs it
  (`../../domain/agent/ask-temper.md`), and an owner may change or remove it as
  any template. Its runs read and propose settings; they never change
  one, since jig's `settings` family proposes and an owner accepts
  (jig's `domain/engine/settings-proposals.md`, 1).
- **Template 4's work,** kind 11 (`settings-kinds.md`, 1), at scope 0 and number 4: no
  design and no close-out, since the stock's charter 5 designs the
  draft itself, delivering its design and status straight to the goal's
  branch, and cuts it into changes it specifies itself, with no
  designers delegated, while its closer settles the status's tags as it
  lands the goal (`../../draft/archive/stock-ways.md`, 3.3, and
  section 4, "plan" in the stock's). So Draft first's path over the stock
  draws "plan" and no close-out, as the look request's does. No other
  stock template has a value of kind 11.

## 5. Budgets and accounting

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
  `domain/engine/settings-application.md`, 1), and
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
  Fable 5.1's, its window at its cache write's price, 13,517 (Astra 6's is
  10,896), so K is 54,068 and a chat's 24K 1,297,632. A run is admitted only with two of them free (its reserve,
  reserve turns 1 + 1), so its allowance is K = 4C: its admission and
  room for its working turns; and at least four units for each of a
  charter's run turns (256 for the chat's 64), so that where C is small
  and prices coarse, smith's one-unit floor per completion still leaves
  a run its turns. A delegate is given half of what its
  parent has left (`../../domain/agent/tools-change.md`, 1, for `change`; a goal
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

## 6. Models and prices

- **The stock's models,** the binary's, as read from the providers' pages on
  2026-11-06, numbered in this order and for good, the scripted model 8,
  its prices its own (`../setup/init.md`, 6), and GPT-6.2 Sol, read on
  2026-11-21 and added since, 9. The eight of OpenAI and Anthropic
  are every deployment's defaults, the stock's providers being every
  deployment's, and the scripted model is the box's, beside its scripted
  provider (`providers.md`, 2 and 4). Each has a window of 1,048,576 and an
  output of 8,192, whatever a provider allows; takes the efforts `low`,
  `medium`, `high`, `xhigh` and `max`, low to high, so that a picker has
  one scale, where Terra and Luna also take OpenAI's `none`; is
  priced for cached input as for input, with no cache write of its own,
  so that a budget stops early rather than late, where its provider's
  page gives no figure on the stock's terms, which is OpenAI's for all
  but GPT-6.2 Sol, whose page gives both at short context
  (`../../draft/model-speeds.md`, section 2); and each
  but the scripted model declares a prompt cache's life
  of five minutes, `cache_life_ms` 300,000, its provider's default, so
  that a standing role's sessions resume within it, the scripted model
  declaring none (jig's `domain/tasks/standing.md`, 5.3):

  | `name` | `shown` | Provider | `effort` | Input, output | Cached, cache write |
  |---|---|---|---|---|---|
  | `claude-opus-5-5` | Opus 5.5 | Anthropic | `high` | 4,000, 20,000 | 200, 5,000 |
  | `claude-sonnet-5-5` | Sonnet 5.5 | Anthropic | `high` | 2,000, 10,000 | 200, 2,500 |
  | `claude-fable-5-1` | Fable 5.1 | Anthropic | `high` | 10,000, 50,000 | 250, 12,500 |
  | `gpt-6.1-sol` | GPT-6.1 Sol | OpenAI | `high` | 2,000, 10,000 | as input, none |
  | `gpt-6-astra` | GPT-6 Astra | OpenAI | `high` | 10,000, 50,000 | as input, none |
  | `gpt-6.1-terra` | GPT-6.1 Terra | OpenAI | `medium` | 1,000, 5,000 | as input, none |
  | `gpt-6-luna` | GPT-6 Luna | OpenAI | `medium` | 100, 500 | as input, none |
  | `gpt-6.2-sol` | GPT-6.2 Sol | OpenAI | `high` | 2,000, 10,000 | 100, 2,500 |
- **Two stock models declare faster speeds** (jig's
  `domain/engine/settings-catalogue.md`, 1), at their providers' prices
  for them, in the same unit, read on 2026-11-21 and 2026-11-27
  (`../../draft/model-speeds.md`, section 2):

  | Model | Speed | Input, output | Cached, cache write |
  |---|---|---|---|
  | `gpt-6.2-sol` | `fast`, also `priority` | 4,000, 20,000 | 200, 5,000 |
  | `gpt-6.2-sol` | `ultrafast` | 12,000, 60,000 | 600, 15,000 |
  | `claude-opus-5-5` | `fast` | 8,000, 40,000 | 400, 10,000 |

  Ultrafast is six times standard, its list price from 2027-03-01, the
  user's figure of 2026-11-27: until then it overcounts by half, so a
  budget stops early rather than late, and nothing held needs repricing
  on that date. No stock model names a default speed, and no stock
  charter or template chooses one: a faster speed costs more, so it is
  a deployment's choice, by a set of its model or a task's choice. The
  others declare none until their providers' prices for one are read;
  Sonnet 5.5 and Fable 5.1 have no faster speed. A deployment declares
  more by a set.
- **Claude's cache prices are Anthropic's,** read on 2026-11-11: a cache
  read $0.20 per million tokens for Opus 5.5 and Sonnet 5.5 and $0.25 for
  Fable 5.1, and a cache write 1.25 times the input price, for the
  five-minute cache smith's marks ask for (skein's `llm/wire.md`, 1.7),
  since Anthropic's page gives them and a key pays them. They are the same
  whichever way a run reaches Claude, Claude Code's program or a key:
  a subscription's runs bill nobody, but their spend is counted at the
  same prices, so that a project's budget means the same either way
  (`../../draft/claude-api.md`, section 5). jig prices a completion's
  worst case at the dearest of its input kinds' prices, so Claude's cache
  write raises C (Budgets in runs of the costliest model, above).
- **Claude's output stays 8,192.** On the direct route it bounds thinking
  and the answer together, since `max_tokens` is the declared output
  (skein's `llm/wire.md`, 1.7), as OpenAI's reasoning models' output
  already does. Up to Anthropic's 128,000 would raise every Claude run's
  worst-case completion, and so C and every budget below, and the
  agent's largest turn (`settings-output.md`, 1), for a cut no run has shown; the
  walk on a real key after `claude-api.md`'s step 1 shows whether a
  delegated change at the stock's efforts is cut (`stock.md`, section 9).

## 7. Limits and floors

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
  | a run's brief | 200 kB (55 kB) | what the host part leaves at `temper-box init`'s models (`../../domain/agent/room.md`, 1) |
  | chats or goals funded at once from one owner's pool | 14 | 80 (the budgets, above) |
  | live changes in a project | 18 | `forge.changes` 128, `forge.holds` and `forge.tasks` 128 |
  | conflicted paths in one merge | past 4, the bound then, on a goal's merge (2026-11-11) | `checkout.conflicts` 512 and `git_component.stdout_bytes` 131,072 in both profiles, for a merge of a few hundred conflicted paths (`limits.md`, the checkout's capacity) |

  The stock's counts that dev.fabrig.ai's documents override are sized
  at twice its figures where those run above this table's defaults
  (`limits.md`, sized from what), and the stock's carriers are derived
  (`derived.md`). The table's defaults hold elsewhere.

  The stock and the reference's whole delegation (`reference.md`) fit the
  sixteen entries ctl's settings form takes in an authority's list.
  A limit the table does not name moves with the ones it is coupled to
  (a store row, a carrier, an inbox), sized the same way or the least
  the start's checks take. jig's and temper's stored formats changed in
  place for these figures, before the first release (jig's
  `protocol/store/records.md`, 1 and 3). The document's overrides say only
  what differs from the profiles: temper's client's documents
  (`documents.rows` 2, `documents.text_bytes` 512; `../setup/init.md`, 4), the agent's carriers
  (`settings-output.md`, 1; `../setup/init.md`, 4) with the bounds that follow its message
  (`../setup/init.md`, 4), the forge's repositories and output (`../setup/init.md`,
  6), and the machine's
  concurrency, which is the machine's to size: the worker's 4 slots, so
  that two chats and a goal's agents run side by side, which the hubs'
  and the fleet's slots in `standard` already hold, with
  `checkout.workspaces` and `git_component.operations` 4, the least they
  take (`../../domain/worker/agent-processes.md`, section 1); and, at what the box had, the
  clients' streams, `clients.streams` and `clients_http.streams` 8,
  each with four ordinary exchanges beside it in what carries it
  (`clients.exchanges`, `clients_http.calls` and temper's server
  connections, 12), since every `temper ctl` call holds a stream and a
  wait holds it for the wait, so a person and a walk hold three to five
  at once, and the engine's watch rooms follow them (`limits.md`). One call's arguments are bounded by its model's output, not
  by the message bound: the agent's `tool_payload`, and
  `checkout.message_bytes` with it, is the largest smith admits at
  init's models (`settings-output.md`, 1). smith offers a run a message within the same
  payload, two bytes less for its separator, and the fleet holds a
  host's agent to the engine's message bound at its hello (jig's
  `protocol/limits.md`), so the box's `tasks.message_bytes`
  and `tasks.result_bytes` are its agent's message, the payload less
  two, overridden below the profile's 262,144: a report or message of
  the table's size reaches a run only from a model whose output admits
  that payload (`settings-output.md`, 1).
- **The stock sets floors under the limits.** Its own values must fit
  the limits they are checked against: a configuration whose
  `core.policy_bytes` is below policy 1's encoding, or whose
  `authority.grants` is below the rules' grants, is refused at the
  start, naming the limit and the stock setting. Policy 1's complete
  snapshot is 6,455 bytes, below `standard`'s 16,384-byte
  `core.policy_bytes`; its ceiling and the rules name 21 executor kinds, including prepare job 1.
  `standard` holds 22 executor kinds in both `authority.executors` and
  `tasks.executor_kinds`, and charter numbers through 16 in
  `tasks.charters`, as init's document does. A seeded project uses those
  same bounds; it needs no policy-byte override or wider store codec.
  The `core.run_bytes`
  floor measures the resolved charter, including its parts and catalogue
  models; charter 5 sets it at init's catalogue, its four files and its
  models', which `temper-service`'s floor test holds exactly. The root holds a run's charter within
  `root.journal.run_bytes`, 65,536 in both of temper's profiles, the
  reference room's (`reference.md`), so that a fresh box admits the
  stock and an overlay of a few KB over it
  (`../../draft/stock-charters.md`, 3); the larger charter 5 leaves its
  brief less of the host part, and `standard`'s
  `root.brief.brief_bytes` follows that room by the rule above. A start's
  problem about a stock setting names it as ctl does, `charter 1.hosts`.

## 8. Model selection and releases

- **The stock charters' models are their first choices and the rest:** a
  stock charter names its first choices, those among the defaults, then
  "the rest", so that a model the catalogue gains needs no charter's change
  (jig's `domain/engine/settings-catalogue.md`, 1). A task
  given no LLM choice takes the first with an account usable now, at
  that model's default effort (jig's `domain/accounts-uses.md`, 1.3):

  | Charter | First choices, in order | Then |
  |---|---|---|
  | charter 11, Ask temper's | `claude-opus-5-5`, `gpt-6-astra`, `gpt-6.1-sol` | the rest |
  | every other stock charter | `claude-opus-5-5`, `gpt-6.1-sol` | the rest |

  A release whose models differ gives new defaults, which make a revision
  recorded as the release's (jig's `domain/engine/settings-store.md`, 1); a model a
  person changed or turned off stays as they left it.
- **A model of the catalogue** changes as any setting does: a proposal, which
  Ask temper can make, or an owner's act on Settings, Models
  (`../../client/pages/README.md`). A task keeps the model it has. A model added
  by name takes its provider's dearest stock model's figures, marked estimated
  (jig's `domain/engine/settings-catalogue.md`, 1).
- **A release's new model that a deployment already holds by name,** as
  one that added its own `gpt-6.2-sol` before this stock did, is left out at the start,
  before its checks: the deployment keeps its own model at its own
  number, and what names it goes on naming it (jig's
  `domain/engine/settings-store.md`, 1).
- **A release that changes a default** reaches every deployment that
  kept it. A setting no change is over follows the binary's default, so
  at the next start the new default makes a revision recorded as the
  release's, with no proposal (jig's `domain/engine/settings-store.md`, 1): new
  work takes it, running work keeps the revision it runs under (jig's
  `domain/engine/settings.md`, 3), and the release's notes name each default
  it changed. A setting a deployment changed keeps its change; a reset
  removes the stored change, as a change of its own, and the setting
  follows the default again.

## 9. Open questions

- **A landing branch with another name than `main`** (`settings.md`, 8;
  `../../draft/archive/stock-ways.md`, section 2): Draft first's rule 1 names
  `main`, so over a repository whose landing branch is named otherwise
  its rules cover no landing branch, and its path shows none of their
  stops there. Default: later, as the draft has it.

- **A chat checking the forge before declaring a repository absent**
  (`stock-settings.md`, section 1, charter 1): the stock directs `read_forge what=branch` for a repository
  outside the checkout. Default: the instruction and its phrase assertion;
  a real-model observation follows landing, with no new tool or authority.

- **Ask temper's real-model walk** (`stock.md`): the small chat's walk follows
  landing; the request bound covers
  temper's parts, not Claude Code's own MCP-attached overhead.

- **Claude's output on a key** ("Claude's output stays 8,192"): whether
  Claude's models declare more, up to Anthropic's 128,000, against the
  budgets and the largest turn a larger output raises. Default: 8,192,
  until the walk on a real key shows a delegated change cut.
