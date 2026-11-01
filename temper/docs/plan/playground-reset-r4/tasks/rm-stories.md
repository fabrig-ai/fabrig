+++
id = "rm-stories"
part = "forge-removal"
kind = "feature"
needs = ["rm-remove-forge", "rm-box-remove", "rm-forge-secrets", "fr-first-intent"]
files = [
  "temper/tests/engine/domain/tests/suite/forge_routes.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/removal.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/start_over.rs",
  "temper/tests/engine/domain/src/forge_routes.rs",
  "temper/crates/temper-service/src/tests/start_over.rs",
]
effort = "high"
minutes = 60
+++

## Goal

The engine world's stories of a forge's removal and of a set up started
over (draft 8, "The engine's world"; draft 6.4): a removal cut by a
crash after each kept act resumes to the same end, and afterwards
nothing of the forge is read but what draft 4 keeps; an unfinished
forge is removed and set up again; GitHub goes on after a restart from
its kept App records. A gap these stories find in a body of this run is
fixed in that body's module (option).

Acceptance (each a test, below):

- **Crash after each kept act.** A forge with two live changes under
  the box is removed; the world restarts (its forked store, as
  `src/forge_routes.rs`' `restart`) after the decision, after the first
  `Cancel` is answered, after the second, and after the box's
  `removed`: each run ends with the `Forge` row erased, no `Kept` row,
  no `Writer`, `Flight` or `Entry` row of the forge, its `Repository`
  rows kept and ended, the forge in no `forges` row, and each act sent
  exactly once more after each restart until answered.
- **A `failed` box remove** stays kept across a restart and is
  answered by a later `removed`.
- **Start over, owned.** An owned forge set up and cut before its first
  repository (its connection not connected, its intent kept) is removed
  (`RemoveForge`), then `SetUp` again makes a new forge, number past the
  old, set up from the template, its first repository made at its
  connect; the welcome's Default project made by the first set up is
  reused (D5).
- **Start over, GitHub.** A GitHub forge whose manifest converted (its
  App records kept) and whose service restarted before installation:
  after the restart it connects from its records and `Install` (Go on)
  is offered and answered with its link; or removed, its records gone.
- Built and run in `temper-engine-domain-world` (`suite`) and the
  service's focused tests.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D3, D4, D5).
- The draft, sections 4, 6.4 and 8.
- `temper/docs/design/domain/testing.md` (the engine's worlds: which
  world, its referee, how a restart is driven).
- The merged reports of `rm-remove-forge`, `rm-box-remove`,
  `rm-forge-secrets`, `fr-setup`, `fr-connect`, `fr-first-intent`,
  `kept-acts`.

## Owns and may change

Owned: the files in the header (`forge_routes/removal.rs`,
`forge_routes/start_over.rs` and the service's `tests/start_over.rs`
new; `suite/forge_routes.rs` takes their two `mod` lines; the service
test declared by a `#[path]` line at the end of `forge/table.rs`, never in
`src/tests.rs`); `src/forge_routes.rs`: stand helpers (a forge removed,
an act answered, a box line) only.

Pre-approved options:

- A story failing for a gap in a body of this run (`removal.rs`,
  `kept.rs`, `asks.rs`, `make.rs`, `first.rs`, `forge/table.rs`,
  `route/box_link.rs`): fix it there, smallest change, and list each
  under `## Fixed` with the story that showed it.
  A gap in another part's body (jig, the box, the playground) is
  reported, not fixed.
- If the GitHub story needs the setup host's fake GitHub
  (`temper-github-app`, `testing/`) at the service level rather than in
  the engine world, write it in the service's test and say so.

## Steps

1. Stand helpers in `src/forge_routes.rs`.
2. `forge_routes/removal.rs`: `a_removal_cut_after_each_kept_act_ends_whole`
   (one test per cut point, or a table over the four),
   `a_failed_box_remove_is_sent_again_after_a_restart`,
   `nothing_of_a_removed_forge_is_read_but_its_repositories`.
3. `forge_routes/start_over.rs`: `an_unfinished_owned_forge_starts_over_as_a_new_forge`,
   `the_default_project_is_reused_by_the_next_set_up`.
4. Service `tests/start_over.rs`:
   `github_goes_on_from_its_app_records_after_a_restart`,
   `a_removed_github_forge_keeps_no_record`.
5. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world -p
   temper-service --all-targets -- -D warnings`; `heavy cargo nextest run
   -p temper-engine-domain-world --test suite -E 'test(removal::) |
   test(start_over::)'`; the service's test by name; the focused tests of
   any package a fix touched. Commit and push.

## Report

fab's sections, `## Fixed` (each gap fixed, its module, its story),
and `## Interfaces` for any helper made public.

## Answers

2026-11-01. The removal's acts are asked in its own decision and the
first `Cancel` is answered in it, so no saved state holds all three
`Kept` rows: gate the sends behind the commit, or move the cuts to the
connector boundary? Neither. The engine root stays as it is: a kept act
is written in the removal's decision and asked in it (`DemoAct`'s rule,
`temper/docs/design/protocol/setup.md`, 8.4; D4), and an act the core
answers in that decision is committed with its answer, its row never
saved and never sent again, which is whole. No production edit outside
the gap-fix list; the cuts stay at the engine world's root and forked
store. The first acceptance item reads, for this task:

- **Crash after each commit of the removal.** The story records the
  commits from the removal's decision to the `Forge` row's erasure and
  restarts the world (`restart`, a fork of the store at that commit)
  after each one, a table over them. The removal's commit holds the
  `Forge` row `removing`, a `Kept` row for each act not answered in it
  (here the second `Cancel` and `BoxRemove`) and no row for one answered
  in it, whose task is cancelled in the same commit. Each run ends as the
  item says (the `Forge` row erased, no `Kept`, `Writer`, `Flight` or
  `Entry` row of the forge, its `Repository` rows kept and ended, the
  forge in no `forges` row); after each restart every `Kept` row then
  saved is sent exactly once more until answered, and no act answered
  before the cut is sent again. At least one cut must hold a `Kept` row, or the story
  fails.
- **The second `Cancel` must be answered:** drive the world past the
  chat template's grace and the task's end. If it still stays kept, that
  is a gap the story found: fix it if it lies in `kept.rs` or
  `removal.rs`, otherwise report it with the write batches, as the
  option says.
