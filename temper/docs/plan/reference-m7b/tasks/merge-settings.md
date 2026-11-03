+++
id = "merge-settings"
part = "merge-kind"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/startup/application/landing.rs", "temper/crates/temper-service/src/tests/startup.rs", "temper/crates/temper-engine-domain/src/translate.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/landing.rs", "temper/crates/temper-engine-domain-forge/src/tests/landing.rs"]
effort = "medium"
minutes = 60
+++

## Goal

A project names its merge command: the document's `application.landing.projects[]` entry reads
`merge_job`, which reaches the forge per project (`notes/contracts.md`, C5). Nothing acts on it
yet; the change procedure's use is `change-merge-steps`' and `change-merge-route`'s.

Acceptance:

- `"merge_job": ".temper/merge"` in a project's landing entry parses, and the engine's
  `ProjectLanding` holds `Some(b".temper/merge")`; absent, `None`;
- an empty path, an absolute one (`/x`), one with a `..` segment, one with a NUL, and one past
  4,096 bytes each refuse the start at `application.landing.projects[i].merge_job`;
- the forge answers `merge_job(project)` with the path for the project that named it and `None`
  for another.

## Inputs

- `temper/docs/design/protocol/configuration.md, 3` (the bullet "The merge job's command");
  `temper/docs/design/domain/landing.md, 13.1` (kind 4's `merge_job` row).
- `temper/docs/plan/reference-m7b/notes/contracts.md`, C5; `notes/common.md`.

## Owns and may change

- `temper/crates/temper-service/src/startup/application/landing.rs`: `project_landing` reads
  `merge_job` (an optional string member, beside `by`; the file's own `optional_*` helpers and
  `read::form`/`What::Bound` style for the refusals).
- `temper/crates/temper-engine-domain/src/translate.rs`: `ProjectLanding::merge_job`;
  `project_landing` calls the forge's setter when it is `Some`.
- `temper/crates/temper-engine-domain-forge/src/domain.rs`: the per-project field and C5's
  `project_merge_job` and `merge_job`. Keep it beside the project's way, in the structure that
  already holds it; a new map is allowed if none fits, bounded as that structure is.
- Tests: `temper/crates/temper-service/src/tests/startup.rs` (beside
  `landing_configuration_reads_tiers_and_project_settings_by_queue_and_turn`) and
  `temper/crates/temper-engine-domain-forge/src/tests/landing.rs`.
- Readers: every `engine::ProjectLanding { .. }` literal in temper (four today, found with
  `rg 'ProjectLanding \{'`) gains `merge_job: None`, field only.

## Steps

1. The parse and its refusals; the engine type's field; the forge's setter and reader.
2. Tests:
   - `landing_configuration_reads_a_projects_merge_job` (startup): present and absent;
   - `a_merge_job_path_out_of_bounds_refuses_the_start_at_its_member` (startup): the five
     refusals of the acceptance, each at its path;
   - `a_projects_merge_job_is_kept_per_project` (forge's `tests/landing.rs`).
   Each doc comment cites `temper/docs/design/protocol/configuration.md, 3`.
3. `heavy cargo nextest run -p temper-service --lib -E 'test(/landing_configuration|merge_job/)'`,
   `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/merge_job/)'`, then
   the three packages' lib tests; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-service -p temper-engine-domain -p temper-engine-domain-forge
   --all-targets -- -D warnings`.
4. Commit and push.

## Report

As the implementer's text says.

## Answers

2026-11-03. **May the task change `temper-engine-domain-forge/src/landing.rs` to keep
`merge_job` beside the project's way?** Yes. `landing::State::projects` is the structure that
already holds the project's way, so the path goes there: its row gains a fourth member,
`Option<Box<[u8]>>`, the project's `merge_job`; no new map. `Domain::project_landing` (in
`landing.rs`) writes the row with `None`, so a reconfigured project without the member holds
none; `translate::project_landing` then calls `Domain::project_merge_job` when the path is
`Some`. `project_merge_job` sets the member of the existing row and returns false where the
project has no row (it inserts nothing, so the map's bound is unchanged); `merge_job` reads it.
The setter and reader may live in `domain.rs` or beside `after_landing` in `landing.rs`,
whichever reads better. The tuple patterns the new member breaks (`landing.rs`, `tiers.rs`,
`verification.rs`, `tests/landing.rs`) gain a `_`, field only, as the Readers bullet allows for
`ProjectLanding` literals. The forge's lint step is `-p temper-engine-domain-forge`, as step 3
says.
