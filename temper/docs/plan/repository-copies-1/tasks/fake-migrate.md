+++
id = "fake-migrate"
part = "fake-migration"
kind = "feature"
needs = ["migrate-request", "fake-copy"]
files = ["skein/crates/skein-fake-forge-protocol/src/lib.rs", "skein/crates/skein-fake-forge-protocol/src/service.rs", "skein/crates/skein-fake-forge-protocol/src/routes.rs", "skein/crates/skein-fake-forge-protocol/src/answers.rs", "skein/crates/skein-fake-forge-protocol/src/response.rs", "skein/crates/skein-fake-forge-protocol/src/server.rs", "skein/crates/skein-fake-forge-protocol/src/tally.rs", "skein/crates/skein-fake-forge-protocol/src/errors.rs", "skein/crates/skein-fake-forge-protocol/src/setup.rs", "skein/crates/skein-fake-forge-protocol/src/setup/creation.rs", "skein/crates/skein-fake-forge-protocol/src/setup/migrate.rs", "skein/crates/skein-fake-forge-protocol/src/tests/migrate.rs", "skein/crates/skein-fake-forge-protocol/src/tests.rs"]
effort = "high"
minutes = 60
+++

# The fake's migration route, and its catalogue

## Goal

The fake forge answers `Migrate` and `DeleteRepository` as Forgejo does, copying from a catalogue of
sources under `.invalid` hosts (`skein/docs/design/fake-forge.md`, 4; contracts S4).

Acceptance, in the new `src/tests/migrate.rs`, on `src/tests/creation.rs`'s harness:

- `a_migrate_answers_as_forgejo_with_the_sources_default_branch`: 201, `CreatedRepository`, and the
  repository reads by every route, its git among them, once the call has ended.
- `a_migrate_under_way_reads_as_present_with_no_branch`: its row is reserved at the entrance.
- `a_source_not_in_the_catalogue_is_refused_and_makes_nothing`, and `a_private_source` likewise: each
  with a status and message of `copy-facts`, or, until it lands, Forgejo's as
  `skein/docs/design/forge-setup.md`, 4.10 names them: 422 and the message; the test names which.
- `a_name_taken_is_409`; `a_slow_source_is_slow`; `a_cut_source_answers_504_or_as_a_timed_out_write_does`.
- `a_source_is_read_by_no_route`: `GET` and git of a catalogue's repository are 404.
- `a_catalogue_with_a_host_outside_invalid_is_refused_by_valid`.
- `a_delete_is_204_then_404_and_drops_collaborators_and_hooks`, and an id is never given twice.
- `the_scopes_are_write_repository`: an admin token without it is refused each route.
- `heavy cargo nextest run -p skein-fake-forge-protocol -p skein-fake-forge-github -p
  skein-forge-setup-world` pass; `heavy cargo check --tests -p temper-fake` passes.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, S3 and S4.
- `skein/docs/design/fake-forge.md`, 2.6 and 4.
- `skein/docs/design/forge-setup.md`, 4.3, 4.10 and the scope table of section 8.

## Owns and may change

- The files of the header. `Migrate`'s and `DeleteRepository`'s arms that `migrate-request` left as
  not served become these routes.
- **`Config.sources`:** every `Config` literal gains `sources: Box::new([])`: `rg 'default_paging_num:'
  skein temper`, the field alone, in any tree. `temper-fake`'s seeding is `fake-box`'s.
- **`ROUTES`** in `tally.rs`: 75 routes now fit its 85, and `skein/docs/design/fake-forge.md`, 4's "up to
  74" is the design's, reported, not changed.
- **`CreateRepository` draws no latency and no fault** (`server.rs`, about 604, runs before any domain
  call): `Migrate` must not copy that; it is a domain call, on the source, and stays open in
  `State::Waiting`.
- **`config.repositories`' row** is reserved at the entrance: `answers.rs`, about 65, otherwise leaves a
  timed-out copy unreadable. Ids: `setup/creation.rs` takes `max + 1`, which a delete would reuse: keep
  a high mark.
- Pre-approved:
  - A source's repository has no row in `config.repositories`. If a git route serves it anyway, hide it
    in that route, and say so.
  - The messages are constants in `setup/migrate.rs`, as one source for the tests.

## Steps

1. `Source`, `Config.sources` and `valid`. 2. The routes and the scopes (`missing_scope`, `server.rs`,
about 691). 3. The tests, each biting: skip the row's reservation, and let a host outside `.invalid`
pass, each once. 4. The implementer's checks; commit and push.

## Report

As the implementer's text asks; the status and message each refusal uses, and where each came from.
