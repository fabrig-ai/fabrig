# Progress: the playground designer

## Specs

final: tasks/pg-words.md
final: tasks/pg-project-page.md
final: tasks/pg-wire.md
final: tasks/pg-client-read.md
final: tasks/pg-playground-pages.md

## Parts

- `playground`: the playground on records, its reset, the demo's removal, and every client page of the run (draft 6.1-6.4, and the client halves of 3, 4, 5).

## Critical path

(in progress)

## Decisions

- A project's Delete goes on a new page, a project's settings, `/projects/<project>/settings` (`Address::ProjectSettings`), linked from the navigation beside "Adoption"; temper's client had no project page (client/README.md, 11). `pg-project-page` makes it, `pg-project-delete` adds Delete.
- Every new client word of the run is added at once by `pg-words` (ids 2200-2237), so the client tasks that run at once never edit `words.rs` (its `ALL` count and `PHRASES`/`SEGMENTS` literals conflict on any two edits).
- Client tasks put their new tests and fixtures in new test files of their own; shared `fixtures.rs` files take only compiler-forced edits.

## Contract problems

- **The scripted model's scripts name forge 1.** `temper init`'s `application.demo.llm` scripts delegate on resource tag `4294967297` (forge 1, repository 1: `temper/crates/temper-init/template/temper.json`, the CSV chat; `temper-init/src/tests/documents.rs`, "forge 1, repository 1"). With forges numbered at runtime and never reused, the playground's fake forge is forge 1 only on a store that never made a forge before it: after a reset, or a set up started over, the next playground's forge is 2 or more and the scripted delegation names a forge that is gone. Proposed fix: the fake LLM, run in the playground's group for forge N (contracts, 3: the group file's entry names N), rewrites the forge half of every `resource` tag its scripts carry whose forge half is the document's playground placeholder, 1, to N (`temper-fake`'s llm script reader, the box designer's crate); the template keeps writing 1. Until it lands, the playground's first start on a fresh store works, a second does not land its CSV change. Cut around: `pg-start` makes the forge whatever its number; the fix is a `temper-fake` task for the box designer (proposed id `bx-script-forge`), which the process story of a second playground needs.
