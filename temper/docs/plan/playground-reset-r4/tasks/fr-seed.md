+++
id = "fr-seed"
part = "forge-records"
kind = "feature"
needs = ["fr-surface", "bx-machine"]
files = [
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/seed.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/seed.rs",
  "temper/crates/temper-engine-domain-forge/src/tests.rs",
  "temper/crates/temper-engine-domain/src/translate.rs",
  "temper/crates/temper-engine-domain/src/domain.rs",
  "temper/crates/temper-service/src/startup/application.rs",
  "temper/crates/temper-service/src/startup/seed.rs",
  "temper/crates/temper-service/src/startup.rs",
  "temper/crates/temper-service/src/settings.rs",
  "temper/crates/temper-service/src/system.rs",
  "temper/crates/temper-service/src/startup/checks.rs",
  "temper/crates/temper-service/src/tests/seed.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/restore_fit.rs",
  "temper/crates/temper-service/src/tests/fixtures.rs",
  "temper/crates/temper-service/src/tests/startup.rs",
  "temper/crates/temper-service/src/tests/engine.rs",
  "temper/crates/temper-service/src/tests/setup.rs",
  "temper/crates/temper-service/src/tests/setup_lifecycle.rs",
  "temper/crates/temper-service/src/tests/setup_github.rs",
  "temper/crates/temper-service/src/tests/setup_admin.rs",
  "temper/crates/temper-service/src/tests/setup_adoption.rs",
]
effort = "high"
minutes = 70
+++

## Goal

The document's forges seed `Forge` records at the store's first start,
and from then on the connector's forges are its records (contracts,
D1): its `configured` forges and git identities come from the `Forge`
rows, a removed seeded forge is never seeded again, and the `forges`
subject's rows say `playground` from the record and leave out a
removing forge. The service builds the seed from the document; the
connector tells the service every forge it keeps at each start
(`Request::ForgeKept`, `fr-surface`). A document may name no forge.

The old constructor path (`Config.forges` and `identities`) stays only
for the worlds that build `Configured` by hand, until `fr-seed-worlds`
moves them; the service uses the seed alone.

Acceptance:

- `temper_engine_domain::Config` has `seed: Box<[forge::Forge]>`;
  `forge::Domain::new` takes `seed: &[crate::Forge]` beside today's
  parameters. With `forges` and `identities` both empty, the seed path holds: before restore,
  `configured` and `identities` are the seed's (`Configured::from_record`,
  `identity.number`); a restored `Forge` row replaces its number's; at
  `Event::Restored`, if a `Forges` row was restored, every seeded forge
  with no restored row leaves `configured` and `identities`; if none
  was, this is the store's first start: each seed row and `Forges {
  next: highest + 1 }` (1 with no seed) are saved, in that decision or
  the first one the root allows after (option). Then one
  `Request::ForgeKept` per kept record, in number order.
- With `forges` or `identities` non-empty (a world's hand-built
  `Configured` or `GitIdentity`), today's behaviour holds, nothing
  seeded and no `ForgeKept`; a non-empty seed beside either is refused
  at construction.
- `forges()`'s rows come from `configured` as today, skip a record whose
  `removing` is true, and take `playground` from the record (the demo's
  forge when the record is absent, the old path).
- The service builds the seed (`startup/seed.rs`) from the document's
  forges, identities and setup entries, held as
  `settings::Engine::seed`, with: the file form's names (`record::Files`)
  for a forge not records-only, the identity's login and author (empty
  for null), the setup entry, `playground` true for the demo's forge
  (`application.demo.forge`), `first` none, `removing` false; it passes
  `Config.seed` and an empty `Config.forges`; `startup::application::configured`
  is gone, its readers (`system.rs`, `startup/checks.rs`, service tests)
  using `Configured::from_record` over the seed.
- `application.forges`, `identities` and `setup` may be absent or `[]`;
  `application.forges[]` is bounded by `forge.forges`. A document naming
  no forge starts: its seed is empty.
- A seeded forge `Forge::fits` refuses is refused at the
  document's check at its path (`What::Bound`).
- The tests below pass, and the focused tests of the connector, the root
  and the service.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D1, section 2).
- jig's seeding of projects, the model: `jig/docs/design/domain/core.md`
  ("A project begins in one of two ways").
- `temper/docs/plan/playground-reset-r4/notes/surveys/s1-forges-document.md`,
  sections 1, 3 ("Startup wiring", "`Configured` reverse dependencies")
  and 7.
- The merged `store-families` (`record::Forge`, `Configured::from_record`,
  `Forge::fits`), `fr-limits` (`Limits.forges`), `fr-surface`
  (`Request::ForgeKept`) and `bx-machine` (`settings::Engine::owned`)
  reports.

## Owns and may change

Owned: the files in the header (`seed.rs` in the connector and
`startup/seed.rs`, `tests/seed.rs` in both, new). In the connector's
`domain.rs`: `Domain::new`, the per-forge tables' capacities (to
`l.forges`: `configured`, `writers`, `connections`, `first`, `ended`,
`apps`, `forge_records`, `identities`), `restore`'s `Forge` arm,
`restored`, `forges()`; nothing else.

Neighbours: every `temper_engine_domain::Config` literal and every
`forge::Domain::new` call in temper (worlds, `temper/tests/**`,
`temper/tools/**`, service tests): `seed: Box::new([])` / `&[]` only.
`temper-service/src/tests/*` readers of `application::configured`: the
`from_record` form. Documents: none change.

Pre-approved options:

- If the root allows no save at `Restored`, seed in the root's first
  decision after it (`restart.rs`'s start decision, where it sends
  `SendDemoActs`): add `forge::Event::Seed` there, the one other edit
  allowed in `temper-engine-domain/src/restart.rs`, and say so.
- If the binary's own setup check (`temper/src/setup.rs`, the box
  part's) refuses an absent `application.forges`, leave it and say so
  in the report: `bx-init` handles the template.
- A seeded forge's identity number is the document's; the next made
  forge's (later, `fr-make-forge`) is past the highest kept.

## Steps

1. **Connector** (`seed.rs`, cited to D1): `pub(crate) fn seed(d:
   &mut Domain, out: &mut Queue<Request>)` (first start's saves) and
   `pub(crate) fn kept(d: &Domain, out: &mut Queue<Request>)` (one
   `ForgeKept` per record); `Domain::new`'s new parameter and the
   capacities; `restore`'s `Forge` arm also sets `configured` and
   `identities`; `restored` calls them on the seed path.
2. **Root**: `Config.seed`, passed to `forge::Domain::new` at both
   calls (`domain.rs`'s construction and its validation call).
3. **Service**: `startup/seed.rs` `pub fn seed(members: &Members,
   settings: &settings::Engine) -> Result<Box<[Forge]>, Problems>` (or
   with the arguments the names need), called where `settings::Engine`
   is assembled (`startup.rs`); `settings::Engine::seed`; `system.rs`
   passes it; `application.rs`'s list bound and emptiness.
4. **Tests.**
   - `temper-engine-domain-forge/src/tests/seed.rs`:
     `a_first_start_saves_the_seed_and_the_next_number`,
     `a_later_start_keeps_the_records_not_the_seed` (restore rows for
     1 and 4 with a seed of 1, 2, 3: `configured` is 1 and 4),
     `a_removed_seeded_forge_is_not_seeded_again` (a `Forges` row and
     no row for 2: 2 is not kept),
     `every_kept_forge_is_told_to_the_service_in_order`,
     `a_removing_record_leaves_the_rows`,
     `the_playground_record_says_playground`.
   - `temper-service/src/tests/seed.rs`:
     `the_template_seeds_its_three_forges` (the template's document:
     forge 1 fake and playground, 2 GitHub records-only, 3 owned asked),
     `a_file_form_forge_keeps_its_file_names`,
     `a_document_naming_no_forge_starts_with_an_empty_seed`,
     `a_seed_past_its_bounds_is_refused_at_its_path`.
   - The engine world that walks a restart
     (`temper/tests/engine/domain`, its restart or forked-store tests)
     still passes: run its suite with `-E 'test(restart) | test(fork)'`.
5. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain -p temper-service --all-targets -- -D warnings`;
   their focused tests; `heavy cargo check --tests` for `temper` and
   every world (`temper-engine-domain-world`, `temper-engine-forge-world`,
   `temper-engine-protocol-forge-world`, `temper-system-world`,
   `temper-one-process-world`, `temper-conformance-world`,
   `temper-client-world`, `temper-real-loop-world`,
   `temper-codecs-world`). This changes where the connector's forges
   come from: run `temper-engine-domain-world`'s and
   `temper-system-world`'s focused tests too. Commit and push.

## Report

fab's sections, and `## Interfaces`: `Domain::new`'s signature,
`Config.seed`, `settings::Engine::seed`, `startup::seed::seed`, and
where the first start's saves go (the first option or not).

## Answers

2026-11-01. May fr-seed change the `Forge` and `Writer` rows of
`temper-engine-domain-forge/src/tests/restore_fit.rs`'s bound table from
`CLIENT.repositories` to `LIMITS.forges`? Yes (option A). The table
states the capacities `Domain::new` gives the owned tables, and this task
owns those capacities (`forge_records` and `writers` to `l.forges`), so
their expectations follow them. The file is added to the header; in it,
change those two rows and the `Domain::new` call's new argument, nothing
else. If the fixture's `LIMITS.forges` makes another row of the test
unreachable (a constructor writer outside the restored keys, say), fix
the fixture's rows to fit the new bound and say so in the report.

2026-11-01. May fr-seed migrate the hand-built fixtures that fail under
the seed path and the obsolete missing-setup assertion (A), or leave the
reds to `fr-seed-worlds` (B)? Neither as asked: the old path is chosen
by `forges` or `identities` non-empty, the seed path by both empty
(acceptance amended above), and the service's fixtures build their seed
as the service does. Reason: the old path is `Config.forges` and
`identities` together (Goal), and the connector's fixtures that pass an
identity with no `Configured` (`tests.rs`'s `domain()` and
`routing_domain()`, `adoption.rs`, `reconcile.rs`, `demo.rs`) are
hand-built worlds; `fr-seed-worlds` moves them to records, so the
connector's test files change here only by the `&[]` argument. The
seed path is not weakened: the service passes both empty, and
`tests/seed.rs` passes both empty. A non-empty seed beside either old
parameter is an `api::Error` at construction.

The service uses the seed alone, so its tests' fixtures must hold one:
the files added to the header may change only so that
`fixtures::engine()` (and each setup test that mutates the document's
forges, identities or setup entries in its `settings::Engine`) sets
`seed` from its document by `startup::seed::seed`, recomputed after the
mutation (D1: a fixture naming forges in its document keeps doing so);
no `Forge` record is built by hand there. Two expectations the task
makes plainly wrong change too, and are listed in the report:
`startup.rs`'s `a_missing_setup_or_demo_is_refused_naming_its_path`
drops its `setup` case (absent `setup` is accepted; rename it
`a_missing_demo_is_refused_naming_its_path`), and `engine.rs`'s
`startup_admission_does_not_promise_a_connected_forge` finds its one
`Application` row among the `Forge` range's rows, the seed's `Forge`
rows and `Forges` beside it. Every other assertion stays. If a service
test still fails after its fixture holds the seed, stop and ask with
the test and its log.
