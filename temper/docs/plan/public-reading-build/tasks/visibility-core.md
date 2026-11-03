+++
id = "visibility-core"
part = "jig-public"
kind = "feature"
needs = ["visibility-surface", "visibility-wire"]
files = ["jig/crates/jig-domain-core/src/admission/visibility.rs", "jig/crates/jig-domain-core/src/routing/party/projects.rs", "jig/crates/jig-domain-core/src/routing/party/people.rs", "jig/crates/jig-domain-core/src/acts/requests.rs", "jig/crates/jig-domain-core/src/admission/deletion.rs", "jig/crates/jig-domain-core/src/start/restore.rs", "jig/crates/jig-domain-core/src/documents/small.rs", "jig/crates/jig-domain-core/src/tests/admission/visibility.rs", "jig/crates/jig-domain-people/src/domain/**", "jig/crates/jig-domain-people/src/tests/projects.rs"]
effort = "high"
minutes = 120
+++

# The core decides a project's visibility, keeps it, and says so in every `me`

## Goal

An owner makes a project public or private: the core decides `SetVisibility`, commits the
`Stored::Public` row or erases it, answers once durable, restores the rows at start, erases
the row with the project, and every `me` of the project's parties says whether it is public
(`jig/docs/design/domain/people.md`, 5.1's `SetVisibility` row and 13.2). The public index
(`public-index`) and the public reads (`public-reads-core`) build on the one predicate this
task gives.

Acceptance:

- The people child routes `SetVisibility` to the core as it routes `RemoveProject` (its
  refusal from `visibility-surface` removed), and answers the core's decision once durable,
  kept under the request's key as every outcome is.
- The core decides it (`routing/party/projects.rs`, `visibility_request`, beside
  `rename_request`), admitted outside the authority's acts as `RemoveProject` is in
  `routing/party/people.rs` (no policy gives it to another role): a project that is not open
  (`Core::open_project` false: unknown, project 0, being deleted) is refused
  `Refusal::Unknown`; a party whose role there is not `Owner` is refused
  `Refusal::Role { held }`; otherwise it is done: made public, `Write::Save(Record::People(
  Stored::Public { project }))`; made private, `Write::Erase` of the row's key; the outcome
  `people::Outcome::VisibilitySet { project, public }`; `core.touch_project_parties(project)`.
  Setting the visibility a project already has is done with the same outcome (whether it
  rewrites the row is yours; the report says).
- The state, in a module of its own, `src/admission/visibility.rs`:

  ```rust
  /// Which projects are public (jig/docs/design/domain/people.md, 13.2).
  pub(crate) struct Visibility { public: Map<u32, ()> /* or the crate's bounded set */ }
  impl Core {
      /// Public, open and not being deleted: the one test every public read and `me` makes.
      pub(crate) fn public_project(&self, project: u32) -> bool;
  }
  ```

  bounded by the projects' limit as `Core.names` is; a field `visibility` on `Core`.
- Restored at start: `Record::People(Stored::Public { project })` is taken by the core in
  `start/restore.rs`, as `Stored::Policy` is, never handed to the people child; a row naming
  project 0 refuses the restore. A store with no public row restores every project private.
- Erased with the project: deletion's rows step (`admission/deletion.rs`, `rows`) erases the
  public row beside the policy's and forgets the project's visibility; from the decision that
  admits the deletion, `public_project` is false (it tests `open_project`).
- `FrameProjectRole.public` is `core.public_project(role.project)` (`documents/small.rs`).
- Unit tests in `jig-domain-core/src/tests/admission/visibility.rs` (a module line in
  `tests/admission.rs`), each through the core's step as `tests/admission/deletion.rs` drives
  it:
  - `an_owner_makes_a_project_public_then_private`: the row saved then erased, the outcome
    both times, every party's `me` re-delivered with `public` true then false;
  - `a_member_setting_visibility_is_refused_and_nothing_is_written`;
  - `project_zero_an_unknown_project_and_a_deleting_one_are_refused_alike` (the same
    refusal, no write);
  - `a_restored_public_row_makes_the_project_public` and
    `a_store_without_public_rows_restores_every_project_private`;
  - `deletion_ends_publicity_at_its_decision_and_erases_the_row`.
- In `jig-domain-people`: `visibility-surface`'s
  `set_visibility_is_refused_until_it_is_decided` becomes
  `set_visibility_is_routed_to_the_core_for_its_project` (`src/tests/projects.rs`, or where
  that test sits).

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, section 2.
- `jig/docs/design/domain/people.md`, 5.1's table (`SetVisibility`'s row), 5.2 (deletion,
  "Closed at once"), 13.2.
- As they stand: `jig-domain-core/src/routing/party/people.rs` (how `RemoveProject` is
  admitted and routed), `routing/party/projects.rs` (`rename_request`), `admission/deletion.rs`
  (`open_project`, `begin`, `rows`), `start/restore.rs` (`Stored::Policy`),
  `routing/view/marks.rs` (`touch_project_parties`), `documents/small.rs`; the people child's
  `domain/requests.rs` and `domain/start.rs`; `visibility-surface`'s and `visibility-wire`'s
  reports (the row's key and the people `Key` variant).

## Owns and may change

- Owns the files in the header.
- `jig/crates/jig-domain-core/src/lib.rs`: the `Core` field `visibility` and its constructor
  line alone, and `mod visibility;` in `src/admission.rs`; `public-vocabulary` may edit
  `lib.rs`'s module and re-export lines at the same time: keep your hunks local.
- `jig/crates/jig-domain-core/src/tests/admission.rs`: the module line alone.
- `jig/crates/jig-domain-core/src/routing/view/marks.rs` and `start/script.rs`,
  `start/census.rs`, `start/cursor.rs`: a `Stored::Public` / `Key::Public` arm only if the
  compiler or a restore check names it (the sweep never erases a public row; a public row
  counts as the deletion's row does).
- `jig/tests/core/src/referee/offers.rs`, `jig/tests/core/tests/suite/deletion.rs`,
  `jig/tests/people/src/world.rs`, `jig/testing/jig-conformance/src/core/naming.rs`: only if
  an arm `visibility-surface` wrote there no longer compiles or no longer holds now that the
  request is decided, edited for that alone.
- Pre-approved: if the people child refuses project 0 earlier with another refusal (it has no
  project role there), keep it and say which. If the people `Key` has no variant for the
  public row, add `Key::Public(u32)` appended last in `jig-domain-people/src/boundary.rs`,
  keyed as `visibility-surface`'s store key is, and say so. If a restored public row can
  precede its project's policy row in key order, check its project once the load phase ends
  rather than at the row, and say where.

## Steps

1. The state and predicate; the people child's routing; the core's decision; until
   `heavy cargo check -p jig-domain-core --tests` passes.
2. The restore and the deletion's erase; `FrameProjectRole.public`.
3. The unit tests above, each run alone first
   (`heavy cargo nextest run -p jig-domain-core --lib -E 'test(/visibility::/)'`,
   `heavy cargo nextest run -p jig-domain-people --lib -E 'test(/set_visibility/)'`).
4. This changes what `SetVisibility` answers (refused before, decided now): run the tests of
   its readers too, `jig-core-world` (`jig/tests/core`), `jig-people-world`
   (`jig/tests/people`) and `jig-protocol-engine-clients`.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; clippy on every package you
   changed; the focused tests of `jig-domain-core`, `jig-domain-people`, `jig-test-domain`,
   `jig-test-service`; `heavy cargo check -p temper-engine-domain --tests`; the citations
   check.
6. Commit (why: an owner's visibility decided, kept and shown in `me`, which the public index
   and reads build on); push your branch.

## Report

As the implementer's text says, and also: the refusals as given (project 0, unknown, deleting,
member), whether a repeated visibility rewrites the row, where a restored row's project is
checked, and every arm outside the header's files.
