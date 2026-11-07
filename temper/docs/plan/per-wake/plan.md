# per-wake: a run reads the instructions for why it runs

The draft's jig half (`temper/docs/design/draft/per-wake.md`, sections 2 to 5, in the
order of its section 9, item 1): a run's reasons, a charter's parts, what a run is
offered, an undecided entry's wake, and a task's `later` choice. The design is written
first, in jig's `domain/tasks.md` (3, 7, 8.2, 8.3, the new 8.6, 9, 13), `domain/engine.md`
(7.1, 7.2, 7.3, 7.4, 9, 16), `domain/hosts.md` (6.4, 9), `protocol/configuration.md` (3,
3.1, 6), `protocol/store.md` (5) and `client/domain/contract.md` (3.2, 5.1), each with its
status's **Not yet**; the draft's status names what stays temper's. The shapes that cross
tasks are `notes/contracts.md`; the standing rules `notes/common.md`.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| surface | `surface` | | the types and fields, forced readers empty |
| formats | `store` | surface | the settings, tasks and people families keep parts, `later`, reasons |
| formats | `config` | surface | a charter's `parts` and a template's `later` in the document, refused at their path |
| formats | `wire` | surface | the client wire 19 and both translations carry parts and `later` |
| offering | `offering` | surface | reasons at activation, kept across attempts; relays and the fence take only what was offered; an undecided entry wakes |
| parts | `parts-core` | surface | the core's check of parts; a run's instructions, start and claim from the parts its reasons meet |

## Critical path, as it stands

`surface`, then `store`, `config`, `wire`, `offering` and `parts-core` in parallel (one
writer per crate), then `later` (which needs the tasks child and the core free again: it
writes both), then the story in the core world. `later` and the story follow in the next
push.

## Overlaps with drafts beside this one

None of this plan's files are among the other drafts' listed files. `surface`'s forced
readers touch temper's `temper-engine-domain`, `temper-engine-smith`,
`temper-protocol-store` and `temper-service` in a field's default only.
