+++
id = "cl-world-playground"
part = "client"
kind = "feature"
needs = ["pg-reset", "cl-world-forges", "cl-play-view"]
files = [
  "temper/tests/client/**",
]
effort = "high"
minutes = 85
+++

## Goal

The joined client world's playground on the real engine (draft 6.1-6.3):
its demo shim goes (`Joint.demo`, the synthetic start outcomes and
`demo_read`); `StartPlayground` makes the engine's playground (the
world composing the fake forge's `NewForge` and answering its group as
the box would), the read answers from the engine's `Playground` row, and
`EndPlayground` resets it whole: the project deleted, the forge removed,
the scripted account removed, then none. Its stories: the playground's
pages with nothing to add, the banner's link, "Leaving the playground…"
until the reset answers, then `/welcome?at=forge` on an empty store, and
the welcome's three states against the engine.

Acceptance (`temper-client-world`, `tests/suite/`):

- `demo_start.rs` (renamed `playground.rs` if the suite's names allow,
  else kept): `try_the_playground_starts_it_and_opens_its_chats`;
  `the_playground_offers_nothing_to_add` (Forges and Accounts drawn
  without add controls; Connect a model's line);
  `set_up_temper_resets_the_playground_and_opens_the_forge_step` (the
  banner's link; "Leaving the playground…" while the row reads
  resetting; then the address `/welcome?at=forge`, the forges watch
  empty, the scripted account gone, the playground's project gone from
  `Me`); `a_reload_while_resetting_reads_leaving_and_ends_the_same_way`.
- `welcome.rs`: `the_welcome_reads_empty_unfinished_and_whole` (three
  engine states, one story each or one walk through them);
  `the_playground_is_not_offered_once_a_real_forge_is_kept`.
- `stories.rs`/`qa_all` cases of the old demo end ("Ended with the
  playground", the halves) are deleted; others pass (each listed).
- `fuzzy/client.rs` passes with the end among the people's acts.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 2 (the row's
  steps) and 5.
- `temper/docs/plan/playground-reset-r2/tasks/pg-start.md`
  (`Ask::StartPlayground { new }`, `PlaygroundConfig`,
  `configure_playground`, `Event::PlaygroundMade`), `pg-reset.md` (the
  reset's acts and answers, `Ask::EndPlayground`, `EndAnswer`),
  `forge-families.md` (`Domain::playground()`), `fr-registry.md`
  (`stock::playground`'s definition, for the world's own composition),
  `cl-play-domain.md`, `cl-play-view.md`, `cl-welcome.md`,
  `cl-world-forges.md`.
- `temper/docs/design/draft/playground-reset.md`, 6.1-6.3.

## Owns and may change

Owns `temper/tests/client/**`. Calls the engine's public items as the
`pg-`, `rm-`, `fr-` and `jp-` tasks built them; never edits the engine.
A story an engine item does not yet allow (for example a restart in the
middle of the reset, if the world cannot restart its engine) is kept to
what exists and named in the report.

## Steps

1. **The shim goes (src/front.rs, src/front/account_acts.rs):**
   `Joint.demo`, `demo_starts`, `demo_removals`, `demo_reads`,
   `demo_outcomes`, `hold_demo_read`/`release_demo_read` and
   `demo_fixture` go, or become thin views of the engine (a story that
   holds a read keeps a hold on the engine's read answer).
2. **The start:** the wire's `StartPlayground` → `Ask::StartPlayground
   { new }`, `new` the world's fake forge projection (number given by the
   decision) and `{}`; the world's playground configuration by
   `configure_playground` (its scripted provider, project name and
   policy as `first_run_projects` sets them today); the group's start
   answered as the box would; the adoption of `playground/widget` by the
   engine.
3. **The read and the end:** the wire's read `playground` answered from
   `Domain::playground()` (running: its forge, project, provider and the
   configuration's cues; resetting; none), as `pg-service` does in the
   service; `EndPlayground` → `Ask::EndPlayground`, answered
   `Resetting`/`EndRefused`; the reset's kept acts flow through the
   world's in-process jig core (the project's deletion, the account's
   removal) and through `cl-world-forges`' group and secrets answers
   (the forge's removal).
4. **Stories:** the tests above.
5. **Checks:** `cargo fmt`; `heavy cargo clippy -p temper-client-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-client-world`, then once `--profile fuzzy`.
6. Old forms from `temper/`; commit and push.

## Report

- Each story deleted or rewritten.
- How long (in the world's time) the reset took, and how many reads the
  client sent while leaving (for `READ_AGAIN`).
