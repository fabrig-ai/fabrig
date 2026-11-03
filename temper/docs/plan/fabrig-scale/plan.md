# fabrig-scale: plan

temper's defaults sized so a fresh box from `temper init` runs a project of fabrig's scale with
5x headroom (`inputs/request.md`): jig's store profile and its formats, the agent, the stock
settings and the forge's limits, every pinned test, and the resource gate's figures in the
status file. Written by the cut as it releases; each batch brings it up to date.

The standing rules are `notes/common.md`; the names and figures that cross tasks,
`notes/contracts.md`.

## What the surveys found (the walls)

- **tasks-v9's shape:** its largest task row holds (batch+1)² members, each priced as counts
  times largest items, so a batch of 160 makes skein refuse the schema (worst case past u32),
  and a 256 kB result, priced thirteen times over in a verdict's fields, breaks it at batch 4.
  jig's `protocol/store.md`, 5.1 already prices a collection the domain bounds by its bytes as
  one piece; tasks-v9 departs from it. The jig family (a designer) reshapes it, in place.
- **48 calls in one response:** smith's transcript schema holds 64 message blocks and 3 MiB of
  opaque bytes; 48 calls need 98 and about 6.4 MB (`smith-calls`).
- **A turn of smith's** is at most 16 MiB (its channel's `turn_body`), but the store's frame and
  the worker wire hold 8 MiB (`transcript-ceilings`).
- **80 questions on a change:** the forge keeps at most 16 open (`forge-questions`).
- **A tool call's payload** is bounded by the model's output (skein-llm: input within answer):
  at init's models' 8,192 output tokens, 10,662 bytes. The agent's `tool_payload` and
  `checkout.message_bytes` take that, not 128 kB, and, since smith offers a message within the
  payload and the fleet holds the agent to the engine's message bound, so do the box's message
  and result bounds (Decisions).
- **The override refusal:** `validate_members` names three relations and marks every other
  override "invalid" when any other fails (`override-relations`).

## The design this cut wrote

- `temper/docs/design/protocol/configuration.md`: 7 (two profiles, `standard`'s figures);
  13.4 (the stock's runs, control, delegation 384 deep 32, ceilings and periods 80 × 24K,
  templates' report the deployment's result bound; the box's limits as a table of the draft's
  figures; the box's message and result bounds its agent's); 13.8 (T at 48 calls per response,
  the store's page, one call's payload); 13.9 (the reference trimmed to `tasks.tree_tasks`, its
  charters at the stock's run); section 11's open question on the reference's small sizes
  settled and removed. Status: one **Not yet** for 7, 13.4, 13.8 and 13.9; the reference's
  Departs removed.
- `temper/docs/design/protocol/setup.md`: 8.1 (the owned example on `standard`, its agent's
  figures), 8.3 (sized for real work; the demo at luna's window), 16 (good defaults settled
  for charters and limits). Status: **Not yet** for 8.1 and 8.3, and the resource gate's
  figures, not measured.
- `temper/docs/design/domain/agent.md`: 4.1, 4.3, 4.3.1 (the stock gives control), 4.6 (the
  demo's room). Status: **Not yet** 4.3, 4.6.
- `temper/docs/design/domain/forge.md`: the resource capacity and 8.11's bound in `standard`.
  Status: **Not yet** 8.11.
- jig's `protocol/configuration.md`, 6 (the relations); `protocol/store.md`, 8.1 and smith's
  `protocol/transcript.md`, 12 (the frame's ceiling); `client/domain/objects.md`, 11 and
  `pages.md`, 3 (what a client receives).
- The jig family (a designer): `protocol/store.md`, 5.1 ("The tasks family's pieces", "No batch
  inside a batch") and 5.4 ("The largest value is the largest tasks row"); `domain/tasks.md`, 4
  and 11 (`tasks.batch_bytes` and its refusal); `protocol/configuration.md`, 6 ("Standard holds
  fabrig's own development") and 9. Status: `store.status.md`, `tasks.status.md`,
  `configuration.status.md`.

## Decisions

- **What clients receive** (survey D): the settings form held twelve grants and kinds, below
  the stock's fourteen, so the configuring agent's read of policy 1 already fails; a chat's
  document was refused past `documents.lines` live delegates, at most 32; a creating offer is
  refused past `documents.lines` entries, so `standard` takes 16 (12 charters, 16 executors);
  the wire's inbox selection held 32 goals against a depth of 32. Each is a task of part
  `clients`.

- jig's `standard` is raised in place to the table (no production code names it); temper's
  application gains `standard` beside `small`; init writes `"standard"` on its three lines
  (contracts K1).
- jig's `standard` `tasks.message_bytes` is 262,144, not the table's 128 kB: a result is a
  message in its parent's inbox, and jig holds a result within the message bound
  (`check_message`).
- Store formats change in place, as the user decided; existing stores are refused at open as
  jig's `store.md`, 5.3 says.
- **The box's message and result bounds are its agent's.** smith offers a run a message within
  the agent's tool payload, and the fleet holds the agent to the engine's message bound; the
  payload is bounded by the model's output (8,192 tokens at init: about 10,662 bytes). So
  init overrides `tasks.message_bytes` and `tasks.result_bytes` to the payload less two, below
  the profile's 262,144, and the templates' report reads the deployment's result bound. The
  draft's 128 kB message and 256 kB report are reached only with a model whose output admits
  them and smith's messages bounded by the window's offer: question `message-room`.
- **The room:** init's scripted model takes `gpt-6-luna`'s window so the host part is luna's
  (about 400,000 bytes), holding the stock's tools with control and a brief whose task section
  is a whole spec. The draft's 1 MB brief is past any room init's models give: question
  `brief-room`.

## Overlaps not avoided

- `realistic-defaults`, cut in flight, is an earlier edition of this draft on the same files
  (question `realistic-defaults`).
- `configuring-p3` (integration) owns `temper-service`'s `stock.rs`, `limits.rs` and their tests,
  `jig-protocol-config` pieces, and jig's client wire v19 output; `public-reading-build`
  (integration) touches `jig-protocol-config` and `temper-service/src/limits.rs`. This plan's
  first batch keeps off them but for `jig-protocol-config`'s `domain.rs` and its tests
  (`override-relations`); the stock and limits tasks come later, after their tasks.

## Parts and tasks

| Part | Task | Needs | State |
|---|---|---|---|
| refusal | `override-relations` (jig): every relation between two limits named | | released |
| ceilings | `transcript-ceilings` (jig): a turn of 16 MiB in the store's frame and the worker wire | | released |
| ceilings | `smith-calls` (smith): 48 calls per response, 26 conversations derive | transcript-ceilings | released |
| ceilings | `forge-questions` (temper): 128 open questions on a change | | released |
| forge-scale | `forge-profile` (temper): the application's `standard` profile; the refusal named through temper | forge-questions, override-relations | released |
| clients | `client-capacities` (temper): the shells and the settings form receive 16 grants, kinds and entries, 80 delegates | | released |
| clients | `delegates-document` (jig): a chat's document lists its live delegates up to `tasks.delegates` | | released |
| clients | `inbox-depth` (jig): the wire's inbox selection holds 64 goals, every client receives them | | released |
| jig-rows | `batch-bytes` (jig): `tasks.batch_bytes`, a batch's bytes, admitted and refused | override-relations, transcript-ceilings | released |
| jig-rows | `tasks-pieces` (jig): tasks-v9 priced by pieces, no batch inside a batch | batch-bytes, forge-profile | released |
| jig-rows | `store-ceilings` (jig): tasks-v9's and core-v6's ceilings for the draft's figures | tasks-pieces | released |
| jig-standard | `standard-scale` (jig): `standard` raised to K2, its store sized by the largest row | store-ceilings, override-relations | released |
| box | `init-standard` (temper): init's document and the owned example on `standard`, the agent at 800 turns, 48 calls, 26 conversations, inbox 82, the demo at luna's window, T | standard-scale, forge-profile, smith-calls, client-capacities | released |
| room | `brief-room` (temper): control in the stock; `standard`'s brief holds a whole spec | init-standard | released |
| stock | `stock-scale` (temper): the stock's runs, delegation, pool, report; QA's funding raise gone; the task row's words | brief-room | released |
| reference | `reference-trim` (temper): the reference sets `tasks.tree_tasks` alone; its charters at the stock's run | stock-scale | released |
| gate | `fresh-box` (temper): starts with no override, `me`, 80 chats, a tree of 384 | stock-scale | released |
| gate | `measure-box` (temper, docs): the resource gate measured, written in `setup.status.md` | fresh-box, reference-trim | released |

Slices: 1 (refusal, ceilings), 2 (forge-scale), 3 (clients), 4 (jig-rows), 5 (jig-standard,
box), 6 (room, stock, reference, gate; the walk's, `walk = true`: QA's rung rm3).

## For answer calls

- `batch-bytes`' spec says no `jig-domain-core` production file but `routing/run/payloads.rs`;
  its `route_answer_payload` must take the tasks child's limits from its one call site in
  `routing/mod.rs`: approve that call site's edit.
- `measure-box` writes a `**Measured**` line in `setup.status.md`: a stated departure from
  `DESIGN.md`, 2 (a status holds gaps), because the user asked the figures there.
- The jig designer's defaults, by the user's 5x rule: `tasks.batch_bytes` 4 MiB in `standard`
  (five times the largest plan seen, 30 tasks of at most 19 kB), 128 KiB in `small`; the inbox
  stays a counted list. Its arithmetic puts `standard`'s largest row, so its store value, page
  and chunk, near 57 MB (the inbox 44.6 MB at 262,144-byte messages); nothing allocates it, but
  skein-io's read buffer is a read's `max` (zeroed), so `measure-box` reads the box's real
  figures against the gate.

## Critical path, as it stands

`override-relations` → `batch-bytes` (120) → `tasks-pieces` (180) → `store-ceilings` (75) →
`standard-scale` (120) → `init-standard` (240) → `brief-room` (150) → `stock-scale` (150) →
`reference-trim` (120) or `fresh-box` (150) → `measure-box` (120): about 22 implementer-hours
end to end. Everything else runs beside it: the ceilings, `forge-profile` and the clients'
tasks finish well before `standard-scale`.
