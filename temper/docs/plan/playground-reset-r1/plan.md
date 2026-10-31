# The plan: playground-reset-r1

The draft `temper/docs/design/draft/playground-reset.md`, sections 3 to 7,
cut into parts in its order, contracts first. `status.md` beside this file
says where the code stands. Tasks are in `tasks/`, their common rules in
`notes/common.md`.

## Parts

| Part | Draft | Tree | Needs |
|---|---|---|---|
| `runtime-forges` | 3, 4 (their skein side) | skein | none |
| `forges-store` | 3 | temper | `runtime-forges` |
| `forge-removal` | 4, 6.4 | temper | `forges-store` |
| `project-deletion` | 5 | jig (and temper's hook) | none |
| `playground` | 6.1 to 6.3 | temper | `forge-removal`, `project-deletion` |

Section 7's item 5, the LLM providers, is later (the draft's own order).

## Decisions

The lead's calls where the draft leaves a choice or the code forces one.
Tasks cite them by number.

- **D1. skein first.** skein's forge component and setup host take their
  forges once, at construction. A forge made at runtime needs both to add
  and remove a forge at runtime; that generic mechanism goes into skein
  before temper uses it (`AGENTS.md`, 1). Tasks `skein-connection-forges`
  and `skein-setup-forges`.
- **D2. The document's forges stay readable, as configured forges beside
  the kept ones.** The draft has forges leave `temper.json`. `temper init`
  writes none, and every forge a person makes (the welcome, Connect a
  forge, the playground) is a record of the store. But a document that
  still names `application.forges`, `identities` and `setup` keeps working
  as today: those are configured forges, numbered by the document, which
  the kept forges' numbering steps over and which cannot be removed
  (`RemoveForge` refuses one, *not offered*). Reason: the owned example
  (`temper/docs/examples/owned.json`), the cluster and GitHub fixtures,
  and the process tier's support writers all configure forges in the
  document; moving them is a migration of its own, with no behaviour the
  draft asks for. Their leaving the document is later.
- **D3. A removed forge's changes are cancelled by the root's existing
  deployment cancel** (`Work::Tasks(Control { by: Party::Deployment {
  project }, control: Cancel { .. } })`, as `landing_route.rs` does), kept
  as acts sent again at each start; an answer that the task has ended
  counts as done. jig gains no cancel act.
- **D4. The draft's open questions take their defaults** (section 9): a
  box restarted in the playground resets it at serve's start; a removed
  forge's repositories' names are kept while a task names them; owners
  delete their project, the deployment the playground's; spend stays in
  the totals; the welcome's Default project is reused by the next set
  up's first repository.
- **D5. Schemas move in sequence.** Each part's store or wire change is
  one task's, at the next version, after the earlier part's has merged;
  no two concurrent tasks change one schema.
