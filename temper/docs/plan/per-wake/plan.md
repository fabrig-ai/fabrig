# per-wake: a run reads the instructions for why it runs

The draft's jig half (`temper/docs/design/draft/per-wake.md`, sections 2 to 5, in the
order of its section 9, item 1): a run's reasons, a charter's parts, what a run is
offered, an undecided entry's wake, and a task's `later` choice. The design is written
first, in jig's `domain/tasks.md` (3, 7, 8.2, 8.3, the new 8.6, 9, 13), `domain/engine.md`
(7.1, 7.2, 7.3, 7.4, 9, 16), `domain/hosts.md` (6.4, 9), `protocol/configuration.md` (3,
3.1, 6), `protocol/store.md` (5) and `client/domain/contract.md` (3.2, 5.1), each with its
status's **Not yet**, one bullet per task that closes it; the draft's status keeps what is
temper's and the cut's three departures. The shapes that cross tasks are
`notes/contracts.md`; the standing rules `notes/common.md`.

## Parts and tasks

Eight tasks, the request's cap; nothing of its seven items is dropped.

| Part | Task | Needs | What | Request item |
|---|---|---|---|---|
| surface | `surface` | | the types and fields every task builds on, forced readers empty | 3, 4, 7 (shapes) |
| formats | `store` | surface | the settings, tasks and people families keep parts, `later`, reasons | 3, 4, 7 |
| formats | `config` | surface | a charter's `parts` and a template's `later` in the document, refused at their path | 4, 7 |
| formats | `wire` | surface | the client wire 19 and both translations carry parts and `later` | 4, 7 |
| offering | `offering` | surface | reasons at activation, kept across attempts; relays and the fence take only what was offered; an undecided entry wakes | 3, 5, 6 |
| parts | `parts-core` | surface | the core's check of parts; a run's instructions, start and claim from the parts its reasons meet | 4, 5 |
| later | `later` | offering, parts-core | `later` checked at batch, amendment, template, restore; taken after the first activation; the document's next-run choice | 7 |
| story | `story` | offering, parts-core, later | the story in the core world, a table of its rows | 1 |

The design (item 2) is the cut's, pushed with the first batch.

## Critical path

`surface` (120 min), then `store`, `config`, `wire`, `offering` and `parts-core` in parallel,
one writer per crate (the longest, `offering`, 150), then `later` (140), which writes the
tasks child and the core again and so follows `offering` and `parts-core`, then `story`
(150): about 560 implementer-minutes. The formats are off the path: the core world keeps
domain records, so the story needs no codec.

## Overlaps with drafts beside this one

- `limits-fuzzy` (integration) lists `jig/crates/jig-domain-*/src/**` for its merged
  `state-hash`: this plan's `surface`, `offering`, `parts-core` and `later` write
  `jig-domain-tasks`, `jig-domain-core` and `jig-domain-people`, which can't be avoided; its
  `engine-sweeps` lists `temper/crates/temper-engine-domain/src/**` and
  `temper/tests/engine/domain/**`, and its `under-causes` `temper/tests/real-loop/src/**`,
  where `surface` edits a forced literal only (`later: None`, `parts: Box::new([])`).
- `look-round-16`'s open tasks list temper's client pages; `wire` edits only
  `temper-client-domain/src/pages/choice.rs`'s forced literal, which none of them lists.
