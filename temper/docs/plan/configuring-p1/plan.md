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
| `running-revision` | the system world's story: a run keeps its task's revision, a chat moves, an advance moves the rest | `system-running-revision` | released |
| `documents-jig` | jig's 17.9: the type, store, check and plan, configuration, client wire | `jig-documents-surface`, then the store, core, configuration and wire tasks | surface released; the rest to come |
| `documents-temper` | temper's 13.7: the forge's gather, the root's binding, the settings file, the domain world's stories | `forge-document-gather`, then the binding, settings-file and stories tasks | gather released; the rest to come |

## Tasks so far

- `jig-chat-from` (jig-client, ~20 min): `shapes::chat_from(me, project, template, words)`,
  refusing a template that is no conversation's. Design line written by this cut in
  `jig/docs/design/client/domain/client.md`, 19, with its **Not yet** in `client.status.md`.
- `ctl-chat-template` (temper-client-tools, ~60 min, needs `jig-chat-from`): the flag, the
  refusal naming the project's chat templates, the step tests, and the help text given whole.

- `system-running-revision` (temper/tests/system, ~45 min): the revisions survey found jig's
  17.3/17.4 built (a run's `RunCharter` prepared at its task's revision; held revisions; the
  fleet given versions; moves) and temper reading a run's charter only from the assignment, so
  nothing is left to build; agent.status.md's "4.1, 4.6" entry was stale and this cut removed
  it. What was missing is a run keeping the old revision: the story shows it, a conversation
  moving at its next run, and the advance. `tools.md`, 11.4's sentence revised to match.

- `jig-documents-surface` (jig core, tasks, authority; ~75 min): `Document`,
  `Charter.documents`, `tasks.charter_documents`, `BriefCorePlanned.documents` (empty), the
  brief-room and owned-bytes helpers, and every forced literal across jig and temper. The
  shapes are `notes/contracts.md`, section 1.
- `forge-document-gather` (temper-engine-domain-forge; ~120 min): `PlanDocument`, the gather
  at the branch head, the heading's part, the line-end cut, the sections saying why
  (contracts.md, section 3). Needs nothing: nothing sends it until the root's binding.

Design this cut wrote for the documents: `temper/docs/design/protocol/configuration.md`, 13.7
(the budget is `root.brief.budgets.document`, as section 7 has the others; the repository is
the one adopted in any project; a settings file's `forge` is the host);
`jig/docs/design/protocol/configuration.md`, section 3 (the `documents` member's JSON);
`temper/docs/design/domain/testing.md`, 6.4 (the documents' stories run in the engine's domain
world, which has the core's plan and the root's binding, the forge world having neither); and
the status sentences each task deletes as it builds (jig's `engine.status.md`, 17.9; temper's
`configuration.status.md`, 13.7; `agent.status.md`, 4.1; `testing.status.md`, 6.4).

## Critical path, as it stands

`jig-chat-from` (landed) → `ctl-chat-template` (slice 1); `system-running-revision` (landed,
slice 2). Slice 3: `jig-documents-surface` → the core's plan and check → the root's binding
(also after `forge-document-gather`) → the domain world's stories.

## Overlaps with plans beside this one

- reference-m1's `settings-wake-charter`, `settings-wake-clients` and `settings-wake-pace`
  (not merged) edit the files a charter's documents must: jig's `settings/charters.rs`,
  `settings/check.rs`, the settings store's schema, codec, generated code and goldens,
  `jig-protocol-config/src/charters.rs`, the client wire's `client-v19-part1.schema`, its
  generated code and goldens, the contract and both translates, `temper-settings-form/**`,
  `settings_adapter.rs` and `temper-service/src/stock.rs`. No order avoids them: the
  documents' store and wire tasks come after the surface, and whichever plan lands second
  regenerates the generated code and goldens over the other's schema.
- reference-m1's `change-words` edits the forge's `boundary.rs` and `domain.rs`;
  `forge-document-gather` keeps its hunks there to a variant, an event and two arms.

- `jig/crates/jig-client/src/shapes.rs` and its tests: `reference-m1/goal-jig-client` (not
  merged) changes them too; `jig-chat-from` adds one function beside `chat` and its tests at
  the file's end, so a conflict, if any, is local.
