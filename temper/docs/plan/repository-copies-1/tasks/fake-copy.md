+++
id = "fake-copy"
part = "fake-migration"
kind = "feature"
needs = []
files = ["skein/crates/skein-fake-forge-domain/src/api.rs", "skein/crates/skein-fake-forge-domain/src/boundary.rs", "skein/crates/skein-fake-forge-domain/src/domain.rs", "skein/crates/skein-fake-forge-domain/src/creation.rs", "skein/crates/skein-fake-forge-domain/src/copy.rs", "skein/crates/skein-fake-forge-domain/src/faults.rs", "skein/crates/skein-fake-forge-domain/src/observe.rs", "skein/crates/skein-fake-forge-domain/src/store.rs", "skein/crates/skein-fake-forge-domain/src/tests/copy.rs", "skein/crates/skein-fake-forge-domain/src/tests.rs"]
effort = "high"
minutes = 70
+++

# The fake's domain copies and deletes

## Goal

`Write::Copy` and `Write::DeleteRepository` run in the fake's domain as
`skein/docs/design/fake-forge.md`, 2.6 says (contracts S3), so a copy can be slow or cut short.

Acceptance, each a test in the new `src/tests/copy.rs`, one thread, seeded:

- `a_copy_is_there_and_empty_from_its_call_and_whole_at_its_end`: read during the call's latency: the
  repository is there, private, with no branch; after its answer, every branch of the source, its
  default branch, its commits the source's own ids.
- `a_slow_copy_lasts_its_fill`; `a_cut_copy_times_out_and_stays_empty`.
- `a_copy_takes_the_domains_faults`: `unavailable` does nothing; `timeouts` makes it and fails it.
- `a_copy_of_a_name_taken_is_exists_and_makes_nothing`.
- `the_source_hears_nothing`: no head, hook or observation names it.
- `a_deleted_repository_frees_its_name_and_reads_nothing`, and a new repository of that name has a
  new id.
- `a_deleted_copy_ends_its_call`: a `Delete` during a slow copy: the copy's call ends and nothing
  fills a gone repository.
- The domain's fuzz and promise worlds pass with neither write drawn: nothing walks a gone slot
  into a panic. `heavy cargo nextest run -p skein-fake-forge-domain -p skein-fake-forge-protocol -p
  skein-fake-forge-github` pass.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, S3.
- `skein/docs/design/fake-forge.md`, 2.1, 2.3, 2.6 and 3.

## Owns and may change

- The files of the header; `skein/crates/skein-fake-forge-domain/src/*.rs` where a match needs the two
  writes' arms.
- **Readers outside the crate, the arms alone:** `skein/crates/skein-fake-forge-protocol/src/answers.rs`,
  `skein/crates/skein-fake-forge-protocol/src/observations.rs` and their `api::Write` matches:
  `rg 'Write::Dispatch' skein temper`, each answering as a write no route sends yet.
- **`Config` stays `Copy`:** the catalogue is the protocol layer's (`fake-migrate`), and the domain takes
  `fill` with the call.
- **A new call state, beside `State::Landing`** (`domain.rs`, about 742): the call makes the repository
  empty at its arrival and fills it at the alarm. `execute` refuses a repository that does not exist,
  so the call names the source; `unborn_repository` (`scenario.rs`) shows the empty form.
- **Delete:** drop the name and mark the slot gone; do not remove it from the `Slab`: about 124
  `repositories.get...expect` sites assume a slot lives, and `Alarm` holds `Id<Repository>`. Audit what
  walks the slab (`inspect`, the promises) so that a gone repository is passed over.
- Pre-approved:
  - `limits.rs`: the copy charges the room its `has` takes, refused `Full` past it; commits are shared
    and cost nothing. No new limit.
  - If the promises' judge (2.5) reads a repository a test has deleted, it passes over it.
  - `can_land` (`domain.rs`, about 522) may say no for `Copy`: `landing` does not apply to it, and the
    report says so.

## Steps

1. The API, 2. the state and its alarm, 3. delete and the audit, 4. the tests, each biting: fill the copy
at arrival, and walk a gone slot, each once, 5. the implementer's checks; commit and push.

## Report

As the implementer's text asks; what walks the slab, and what you changed there.
