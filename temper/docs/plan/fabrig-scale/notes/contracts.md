# fabrig-scale: contracts

The names and figures that cross this plan's tasks, as its binding calls. Each entry names the
design that holds it in full; the design wins on wording, this note on names and figures. A
figure marked *derived* is computed by the task that owns it, by the rule given, and reported;
a later task reads it from the code, never from this note.

The target is the draft's table (`temper/docs/plan/fabrig-scale/inputs/request.md`): fabrig's
development as fab runs it, 5x the largest figure seen.

## K1. Profiles, by name

- jig's `standard` profile (`jig/crates/jig-protocol-config/src/profile/standard.rs`) is raised
  in place to K2's figures; `small` is unchanged. No third jig profile.
- temper's application limits gain a second profile, `standard`
  (`temper_service::limits::Profile::Standard`, parsed from `"standard"` by
  `temper-service/src/startup/application.rs` beside `"small"`), small's constants with K4's
  figures.
- `temper init`'s document (`temper/crates/temper-init/template/temper.json`) and
  `temper/docs/examples/owned.json` name `"standard"` on all three `"profile"` lines (the
  engine's, the worker's and the application's), and their overrides say only what differs
  from the profiles (K5).

## K2. jig's `standard`, raised (jig family; `jig/docs/design/protocol/configuration.md`, 6)

At least these, each a configured limit of the tasks, people or authority child:

| Limit | standard | For (the draft's row) |
|---|---|---|
| `tasks.tasks`, `tasks.project_tasks` | 128 | live tasks in a deployment |
| `tasks.project_goals` | 80 | live goals at once |
| `tasks.tree_tasks` | 384 | tasks in one goal's tree, lifetime |
| `tasks.depth` | 32 | depth of the work tree |
| `tasks.delegates` | 80 | live direct delegates of one task |
| `tasks.batch` | 160 | tasks made in one atomic creation |
| `tasks.inbox_messages`, `tasks.run_inbox` | 82 | a result's room for each live delegate, a question's and an answer's |
| `tasks.spec_bytes` | 131,072 | a task's spec |
| `tasks.result_bytes` | 262,144 | a report |
| `tasks.message_bytes` | 262,144 | a message: at least a result, which a parent's inbox holds as one (`check_message`) |
| `tasks.charters` | 12 | the stock's five and the reference's three, with room |
| `tasks.executor_kinds`, `authority.executors` | 16 | the stock's and the reference's executors |
| `tasks.authority_grants`, `authority.grants` | 16 | the stock's fourteen grants |
| `authority.segments`, `tasks.authority_segments`, `tasks.hold_segments` | 8 | the forge's deepest path |
| `tasks.contract_choices` | 4 | a foreman's verdict's three choices, a person task's four |
| `people.operations` | 81 | delegates plus one; at least project goals |
| `people.inbox_tasks` | 256 | twice the live tasks |
| `people.words` | 8,192 | a person's words, a goal's spec a person writes among them: people-v14's ceiling, the most without a format change |
| `documents.lines` | 16 | a creating offer's every charter (12) and executor (16): the engine refuses an offer past it (`jig-domain-core/src/acts/bounds.rs`); at most 32 (`jig-protocol-config/src/clients.rs`) |

- Every limit these couple to (relations jig's checks hold, store rows, the inbox's bytes, the
  funders and pools 80 chats at once need, the notes' scopes over the grants, the recipients
  over the inbox) moves with them, *derived* by the jig family's tasks, the least that passes
  or 5x the largest seen where the survey gives one.
- The machine's concurrency in `standard` (hub, fleet and worker slots, workers, clients'
  streams and calls) is jig's own choice for a team; init sets its own (K5).
- The store's figures (`STORE_KV`, the journal's, routes) are *derived* by the jig family, the
  map's `budget` staying 268,435,456, within the resource gate (the draft: tens of MB resident,
  up to about 1 GB on disk).
- **tasks-v9's shape** follows jig's `protocol/store.md`, 5.1 (a collection the domain bounds
  by its bytes in all is one piece; a record's worst case is the sum): the jig family's design
  says how, and its tasks build it.

## K3. Transcript ceilings (task `transcript-ceilings`)

- smith's largest turn T is at most 16,777,216 bytes, smith's channel's `turn_body` ceiling
  (smith's `protocol/limits.md`, 4, `turn-in-channel`). So, in place: the store's core-v6
  `TurnRecord.transcript` and jig's worker wire v9 `Turn.body`, each one turn, `bytes 16777216`;
  the worker wire's `Assignment.transcript`, the worker's resumed transcript (resume plus twelve
  bytes a four-byte step, at most 67,108,880 at that T), `bytes 134217728`. jig's client wire's
  `TurnCarried` stays: a larger turn reaches a client as `too_large`.
- jig's `standard` page holds a TurnRecord frame of a transcript of 16,777,216 bytes, so any
  turn smith admits is stored (the jig family sizes it).

## K4. temper's application `standard` (task `forge-profile`)

small's constants, with:

| Limit | standard |
|---|---|
| `forge.tasks`, `forge.holds`, `forge.changes` | 128 |
| `forge.client.resources` | 128 (at least `forge.holds`) |
| `forge.landings` | 32 |
| `forge.change_policy.questions`, `.tasks`, `.repairs`, `.resolutions`, `.updates` | 80 |
| `forge.issue_policy.plan_items` | 384 |

- The forge's open questions kept on a change's record: at most 128
  (`temper-engine-domain-forge-change`'s `QUESTIONS_OPEN`, store-v11's `Questions.open`), so
  `change_policy.questions` up to 127 passes (task `forge-questions`).
- The root's other limits the start's checks couple (loads, journal, the forge's output and
  facts) move with them, *derived*, the least that passes. The brief's budgets are K8's (task
  `brief-room`), small's until then.

## K5. init's documents (task `init-standard`)

- The three profiles `"standard"` (K1).
- **The agent** (`temper-init/template/agent.json` over smith's `standard`): `max_turns` 800,
  `calls_per_response` 48, `conversations` 26 (a run and 25 sub-agents), `inbox` 82 (jig's
  `standard` `tasks.run_inbox`, which the fleet holds the agent's inbox to); `tool_payload` the
  largest smith's derivation admits for init's models (skein-llm's input within answer: about
  10,662 bytes at an output of 8,192), *derived*; `checkout.message_bytes` equal to it;
  `hub.event_bytes` two less.
- **T**, the agent's largest turn, *derived* by smith's measurement with that agent, and the
  carriers of `temper/docs/design/protocol/configuration.md`, 13.8's table from it, written by
  the `startup::turn_bounds` writer.
- **The box's message and result bounds follow its agent:** smith offers a run a message within
  the tool payload, two bytes less, and the fleet holds the agent to the engine's
  `tasks.message_bytes` at its hello; so init's `tasks.message_bytes` and `tasks.result_bytes`
  are `tool_payload` − 2 (*derived*, about 10,660), overridden below `standard`'s 262,144
  (`temper/docs/design/protocol/configuration.md`, 13.4 and 13.8). `tasks.spec_bytes` stays the
  profile's 131,072 (the brief carries a spec, K8).
- **The machine's concurrency stays init's:** the worker's `slots` 1 and every hub, fleet and
  clients figure init takes today (small's or its overrides) is kept by override where
  `standard` sets another.
- Overrides that stay because they are temper's client's or this machine's: `documents.rows`
  2, `documents.text_bytes` 128 (`documents.lines` is the profile's 16, K2), `hub.accounts` 4, `hub.event_bytes`,
  the T carriers, `forge.repositories` and `forge.client.repositories` 3, `forge.output` 82;
  every other override equal to the profile's figure is dropped.

## K6. The stock (task `stock-scale`; `temper/docs/design/protocol/configuration.md`, 13.4)

- Charter 1 (and charter 3, the configuring agent's, where the stock holds it): a run of 64
  turns and 3,600 s; charters 2, 4 and 6: 800 turns and 28,800 s; every reserve 1 turn and 60 s.
- `CHAT_TOOLS` gains jig's control family, 4, beside the families it has.
- Delegation (the rules, policy 1's ceiling, role 0): 384 tasks, 32 deep.
- A chat's spend stays 24K (role 0's authority spend, which a chat or goal takes as allowed);
  the rules' and policy 1's ceilings and periods and role 0's period are 80 × 24K, so 80 chats
  or goals are funded at once from the owner's pool.
- Templates 1 and 2's report: the deployment's `tasks.result_bytes`, read at each start as C is
  (so the stock's floor on `tasks.result_bytes` always holds).

## K8. The room (`temper/docs/design/domain/agent.md`, 4.3 and 4.6)

- init's scripted model, `demo`, takes `gpt-6-luna`'s window, 1,048,576, in both init documents
  (`template/temper.json`'s `engine.llm.models` and `template/agent.json`'s endpoint), so the
  smallest window over init's models is luna's and the host part's bound is luna's room
  (*derived* by smith; about 400,000 bytes): task `init-standard`, with the agent, so the
  stock's brief keeps fitting whole as the agent's own part grows. The rest of K8 is task
  `brief-room`'s.
- The stock's `CHAT_TOOLS` gains the control family (4) in this task, since control's plan form of
  `change` (1,939 bytes more in charters 1 and 2) is what the room must hold.
- temper's `standard` brief: its task section a whole spec (131,072) with its contract's and
  requester's lines; `root.brief.brief_bytes` and `root.journal.result_bytes` equal, the most the
  host part leaves the largest stock charter (charter 1, a Change contract, control) with its 64
  sections' titles and at least small's margin, 1,129 bytes (*derived*).

## K9. What clients receive (`temper/docs/design/client/README.md`, 2; jig's `client/domain/objects.md`, 11 and `pages.md`, 3)

- temper's shells (terminal, browser, `temper ctl`): `authority` 16, `rows` 16 (ctl's 32),
  `delegates` 80; the settings form and ctl's settings adapter: `authority` and
  `delegation_kinds` 16 (task `client-capacities`).
- A task's document lists its live delegates up to `tasks.delegates`, not `documents.lines`
  (task `delegates-document`).
- jig's client wire v19 `InboxSelection.goals` `list 64`, and a client receives the whole list,
  not its `rows` (task `inbox-depth`).

## K7. The override refusal (task `override-relations`; `jig/docs/design/protocol/configuration.md`, 6)

- A failed relation between two configured limits is `What::Capacity { member, configured,
  requires, required }` at `limits.overrides.<member>`, the first in a fixed order, and no
  other override of the family is marked `What::Override` for it.
