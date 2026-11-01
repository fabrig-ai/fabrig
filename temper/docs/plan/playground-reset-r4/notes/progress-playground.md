# Progress: the playground designer

## Specs

final: tasks/pg-words.md
final: tasks/pg-project-page.md
final: tasks/pg-wire.md
final: tasks/pg-client-read.md
final: tasks/pg-playground-pages.md
final: tasks/pg-project-delete.md
final: tasks/pg-forge-remove.md
final: tasks/pg-welcome-states.md
final: tasks/pg-start-over.md
final: tasks/pg-ctl.md
final: tasks/pg-ctl-help.md

## Parts

- `playground`: the playground on records, its reset, the demo's removal, and every client page of the run (draft 6.1-6.4, and the client halves of 3, 4, 5).

## Critical path

(in progress)

## Decisions

- A project's Delete goes on a new page, a project's settings, `/projects/<project>/settings` (`Address::ProjectSettings`), linked from the navigation beside "Adoption"; temper's client had no project page (client/README.md, 11). `pg-project-page` makes it, `pg-project-delete` adds Delete.
- Every new client word of the run is added at once by `pg-words` (ids 2200-2237), so the client tasks that run at once never edit `words.rs` (its `ALL` count and `PHRASES`/`SEGMENTS` literals conflict on any two edits).
- Client tasks put their new tests and fixtures in new test files of their own; shared `fixtures.rs` files take only compiler-forced edits.

## Contract problems

- **The scripted model's scripts name forge 1.** `temper init`'s `application.demo.llm` scripts delegate on resource tag `4294967297` (forge 1, repository 1: `temper/crates/temper-init/template/temper.json`, the CSV chat; `temper-init/src/tests/documents.rs`, "forge 1, repository 1"). With forges numbered at runtime and never reused, the playground's fake forge is forge 1 only on a store that never made a forge before it: after a reset, or a set up started over, the next playground's forge is 2 or more and the scripted delegation names a forge that is gone. Proposed fix: the fake LLM, run in the playground's group for forge N (contracts, 3: the group file's entry names N), rewrites the forge half of every `resource` tag its scripts carry whose forge half is the document's playground placeholder, 1, to N (`temper-fake`'s llm script reader, the box designer's crate); the template keeps writing 1. Until it lands, the playground's first start on a fresh store works, a second does not land its CSV change. Cut around: `pg-start` makes the forge whatever its number. Taken by the box designer as `bx-script-forge`.
- **The client cannot see whether `SetUp` is offered without an owned row.** Contracts 5 gives the `forges` rows `finished` (and `forge-asks` adds `playground`), but the welcome's "Run on this machine" card (`temper-client-view/src/forge/connect.rs`, `welcome_forgejo`) is drawn from an owned row's `offer` today; with forges as records a fresh deployment has no owned row until `SetUp` makes one, so the card cannot know it is offered (`application.owned` present, serve under the box). Proposed: the `forges` subject's record gains a deployment-level `owned: Offer` (`not_offered` with no `application.owned`, `not_under_the_box`, `offered`), filled by the service from `settings::Engine::owned` (`bx-machine`) and the box, in the engine designer's task that lists the rows from records (or `fr-setup`). `pg-connect-kinds` reads it; until it lands the client keeps today's row-based offer for a seeded owned row.
- **The playground's project is made, not seeded.** `application.demo.project` is a number naming a seeded project in the code (contracts 6 keeps it); the reset deletes that project (draft 6.2), and a second start after the reset (offered again: 6.3) has none. `pg-start` makes the playground's project at each start by jig's `MakeProject`, named `Playground` with policy 1 (protocol/setup.md, 8.4: the project the start makes, "the stock policy 1 unless changed"), and keeps its number in the `Playground` row; the document's number is read for its check alone until the member takes the design's `{"name", "policy"}` form (a reader change in `bx-machine`'s files, proposed for `bx-init`). "The playground's project allows only the scripted model" (draft 6.1) is carried by the scripted account's uses naming that project alone; an engine-side refusal of other models there needs a playground policy whose `models` is `["demo"]`, a template change proposed for `bx-init`.
