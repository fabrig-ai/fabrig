+++
id = "brief-words"
part = "brief-words"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-domain-brief/src/**",
  "jig/tests/brief/**",
]
effort = "high"
minutes = 75
+++

# The brief child says why it failed, and how that counts

## Goal

`jig-domain-brief` ends a brief with `Failed { why }` or `Refused`, which say nothing a person can read and don't tell capacity from a fault. Give each ending a `Failure` by the table in `jig/docs/design/domain/engine.md`, 9.1. This task carries the surface other tasks build on: **what a failure does to a task doesn't change here**, and every brief failure still pauses.

Acceptance:

- `notes/contracts.md`, 1 is built as written, its readers compile, and all of today's behaviour holds outside the child's own vocabulary.
- Each refusal and failure of the child's own says the fate and words below, each pinned by a test in the brief world or the child's unit tests.
- A connector's `Failure` on a required section comes out unchanged on `Request::Failed`.
- `jig/docs/design/domain/connectors.status.md`'s "9: a connector saying why" **Not yet** entry stays: `forge-sections` closes it.

## Inputs

- `temper/docs/plan/brief-failure/notes/common.md`, `temper/docs/plan/brief-failure/notes/contracts.md`, 1.
- `jig/docs/design/domain/engine.md`, 9 and 9.1.
- `jig/AGENTS.md`.

## Owns and may change

Owns `jig/crates/jig-domain-brief/src/**` and `jig/tests/brief/**`.

May also change, each only as the compiler forces it (a field, a pattern, a literal):

- `jig/crates/jig-domain-core/src/routing/children.rs`, `tag_brief`: its pattern only; the route to `PreparationFailed` stays.
- `jig/crates/jig-domain-core/src/routing/task/save.rs`: its patterns, if they need it.
- `jig/crates/jig-domain-core/src/routing/run/preparation.rs`, the document's `Planned::Connector`: `name: b"document"`.
- `jig/crates/jig-domain-core/src/tests/runs/brief_documents.rs`, `jig/crates/jig-domain-core/src/tests/routing/deadlines.rs`: their literals.
- This task is one of the plan's required fields across trees (a departure from "a branch stays in its tree"), so temper's lines naming them:
  - `temper/crates/temper-engine-domain/src/assemble.rs`: both `Planned::Connector` literals, and the `!gatherable` `Missing`; add `pub(crate) fn forge_brief_name(kind: &ForgeBriefKind) -> &'static [u8]` beside `forge_brief_kind`, with the names in `notes/contracts.md`, 5.
  - `temper/crates/temper-engine-domain/src/route/forge_route.rs`, the one `Missing` under `BriefSized`.
  - `temper/crates/temper-engine-domain/src/route/forge_route/tests.rs`, `temper/crates/temper-engine-domain/src/tests/documents.rs`: their literals and patterns.
  - In both `Missing`s the connector says nothing yet: `GatherFailure { fate: GatherFate::Capacity, section: forge_brief_name(&row.kind), words: b"could not be gathered" }`, so temper's briefs pause as they do today. `forge-sections` replaces them.

Options, pre-approved:

- A reader the survey missed that names `Planned::Connector`, `GatherEvent::Missing`, `GatherRequest::Failed` or `Refused` by its fields: edit it only for the field, and list it.
- A connector section with no fitting name in a test: `name: b"section"`.

Leave `Missing`, `Placed::Missing` and what they render alone: optional sections keep their markers.

## Steps

1. Build the types of `notes/contracts.md`, 1. `update`'s `size: Option<u32>` becomes `Result<u32, Failure>`.
2. Give each ending its failure:

   | Where | Fate | Section | Words |
   |---|---|---|---|
   | `begin`: this brief already gathering, or the table full | capacity | `brief` | `no brief slot is free` |
   | `begin`: more sections than the limit, or a count past `u32` | permanent | `brief` | `it has more sections than a brief may hold` |
   | `begin`: budget past `brief_bytes` | permanent | `brief` | `its budget is past the limit` |
   | `begin`: deadline at or before now | permanent | `brief` | `its deadline has passed` |
   | `begin`: not `distinct` | permanent | `brief` | `a section is planned twice` |
   | `begin`: core text past `brief_bytes` | permanent | its `core_name` | `its text is past the brief's limit` |
   | `update`: a required section, the connector's `Failure` | the connector's | the connector's | the connector's |
   | `update`: a required section's `Ready` or `Cut` past its bound | permanent | its `name` | `answered past its bound` |
   | `fire`, `allocate` when expired: a required section late | transient | the first such section's `name` | `not gathered by its deadline` |
   | `allocate`: `planned::plan` finds no plan | permanent | `brief` | `its required sections do not fit its budget` |
   | `allocate`: a required core text that can't be cut, a required connector's `CutTo` of 0 | permanent | its name | `does not fit the brief's budget` |

   `Missing` on `Request::Failed` stays what it is today at each of these.
3. Tests: in `jig/tests/brief/tests/suite/simulation.rs`, a capacity refusal against a table-full brief and a permanent refusal for too many sections; a connector's `Failure` passing through; the late and budget rows. `Failure::reason` is `pull: words`. The referee still holds.
4. `cargo fmt`; old forms; `heavy cargo clippy -p jig-domain-brief -p jig-domain-core -p temper-engine-domain --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-brief -p jig-brief-world`, then `-p jig-domain-core --lib`, `-p temper-engine-domain --lib`. Check each count.
5. Commit, push.

## Report

As the implementer's text has it, and the readers changed beyond the list above.
