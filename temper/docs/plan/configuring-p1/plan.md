# configuring-p1: the plan

2026-11-02. Phase P1 of `temper/docs/design/draft/configuring.execution.md`: a charter's
documents, each run reading its charter at its task's revision, and `chat start --template`.
Design: none new (jig's `engine.md`, 17.3 and 17.9; temper's `configuration.md`, 13.7,
`agent.md`, 4.1 and 4.6, `tools.md`, 5.3); this cut writes only where a cited section or a
status is incomplete.

## Parts

| Part | What | Tasks | State |
|---|---|---|---|
| `chat-template` | jig's `shapes::chat_from`; temper's `chat start --template` | `jig-chat-from`, `ctl-chat-template` | released |
| documents, revisions | a charter's documents (jig, temper); runs at their task's revision | to come, after the surveys | surveying |

## Tasks so far

- `jig-chat-from` (jig-client, ~20 min): `shapes::chat_from(me, project, template, words)`,
  refusing a template that is no conversation's. Design line written by this cut in
  `jig/docs/design/client/domain/client.md`, 19, with its **Not yet** in `client.status.md`.
- `ctl-chat-template` (temper-client-tools, ~60 min, needs `jig-chat-from`): the flag, the
  refusal naming the project's chat templates, the step tests, and the help text given whole.

## Critical path, as it stands

`jig-chat-from` → `ctl-chat-template` (slice 1). The documents and revisions parts follow in
later batches and will hold the critical path.

## Overlaps with plans beside this one

- `jig/crates/jig-client/src/shapes.rs` and its tests: `reference-m1/goal-jig-client` (not
  merged) changes them too; `jig-chat-from` adds one function beside `chat` and its tests at
  the file's end, so a conflict, if any, is local.
