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
| `documents-jig` | jig's 17.9: the type, store, check and plan, configuration, client wire | `jig-documents-surface`, `jig-documents-store`, `jig-documents-core`, `jig-documents-config`, `jig-documents-wire` | released |
| `documents-temper` | temper's 13.7: the forge's gather, the root's binding, the settings file, the domain world's stories | `forge-document-gather`, `root-document-binding`, `settings-file-documents`, `domain-documents-stories` | released |

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

- `jig-documents-store` (jig-protocol-store; ~75 min; needs the surface): the settings
  schema's `Document`, the codec, regenerated code and goldens.
- `jig-documents-core` (jig-domain-core; ~90 min; needs the surface): the check (bounds at
  `restore_fit`, the connector at `check_set`), the price, and the plan at the run's charter
  version, with the test that a run on an older revision plans its own documents.
- `jig-documents-config` (jig-protocol-config; ~75 min; needs the surface): the `documents`
  member, `tasks.charter_documents`, Small's 8 sections, the start check naming both limits.
- `jig-documents-wire` (jig client wire, contract, translates; ~90 min; needs the surface and
  the configuration, whose `clients.rs` gets the documents' bound row).
- `root-document-binding` (temper-engine-domain, -smith, -service; ~150 min; needs the surface
  and the gather): the binding, the `Document` kind with its title, kinds 16+, the
  `document` budget, Small's 8 root sections, the room rule (forge sections never evict a
  document), temper's start check, and the readers' tests the larger brief moves.
- `settings-file-documents` (temper-settings-form, client tools; ~90 min; needs the wire): the
  settings file's `documents`, their rendering, the field rows and the help page they render.
- `domain-documents-stories` (temper/tests/engine/domain; ~90 min; needs the binding, the core
  and the store): testing.md 6.4's four stories, end to end.

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
slice 2). Slice 3, the critical path: `forge-document-gather` (~120 min, from the start) and
`jig-documents-surface` (~75) → `root-document-binding` (~150) → `domain-documents-stories`
(~90): about six hours of implementer time. Beside it: `jig-documents-config` (~75) →
`jig-documents-wire` (~90) → `settings-file-documents` (~90), about four hours after the
surface; `jig-documents-store` and `jig-documents-core` in parallel after the surface, both
before the stories.

## Decisions

- **Revisions:** nothing to build (the survey's evidence, spot-checked: `run.rs`'s `prepare`,
  `start.rs`'s `charter_with_history`); only the system world's proof and the status fixes.
- **Where documents live:** on jig's `Charter`, not its `RunPolicy` (fewer literal sites; the
  plan reads them from `charter_of`, at the run's revision).
- **A document's title:** carried by the root's `ForgeBriefKind::Document(words)`, which is never
  stored or sent on a wire; a missing section finds its document by its kind (16 + index), since
  jig's `Missing` carries no token, so jig's brief child does not change.
- **The start check:** at the configuration's and the service's start, never in
  `jig_domain_core::valid`, so in-code fixtures keep their sections and the plan's room guard
  leaves documents out.
- **Sections:** Small's 8 for jig and for temper's root; temper's forge sections evict only
  optional core sections, so 8 holds three required core sections, two documents and the forge's
  three.
- **The repository:** adopted in any project (13.7 says so now), no authority asked (17.9).
- **The stories' world:** the engine's domain world (testing.md 6.4 says so now); the gather's
  own cases are the forge crate's step tests.

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
