+++
id = "fr-sk-connection"
part = "forge-records"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/**"]
effort = "high"
minutes = 100
+++

## Goal

skein's forge connection component takes forges at runtime: a forge
added after construction and a forge removed, so temper can configure a
forge from a record when it is made or restored, and drop it when it is
removed (temper's draft `temper/docs/design/draft/playground-reset.md`,
sections 3 and 4). Today the component's forges are fixed at `new`
(`settings.forges`, positional tables, Forgejo and GitHub tables sized only
if a forge of that kind is configured) and `Request::Connect` for an
unconfigured forge is ignored. This is the generic mechanism skein lacks
(fabrig `AGENTS.md`, section 1: skein before its consumer).

Acceptance:

- `skein_forge_connection::Request::Add { forge: settings::Forge }`:
  admitted while live; its number not configured and fewer than
  `Limits::forges` configured, else ignored with a `Fact` naming it (an
  owner's error the owner never makes: temper bounds its records by the
  same limit). The forge then behaves exactly as one handed at `new`:
  with `access`, its start runs; without, it runs no start and its calls
  answer `NotConnected` until `Request::Connect`.
- `Request::Remove { forge: u16 }`: its calls in flight end at once with
  their one `Event::Answered` as at their deadline (`NotConnected`), its
  connections close, its held answers and hints are dropped, its GitHub
  installation state and own plans go, and its slot is free for a later
  `Add` of another number. No event for the forge follows. A number not
  configured is ignored.
- Tables no longer depend on the kinds configured at `new`: the Forgejo
  and GitHub tables and the GitHub connection reserve are sized from the
  limits for every forge slot (`Limits::forges`), so an `Add` never
  grows anything; `limits::check` and the owned-bytes estimate
  (`settings::owned_bytes`) price that room. Per-forge state moves from
  positional `List`s indexed by settings position to a table keyed by
  the forge's number (or a slot index found by number), whichever keeps
  the code smaller.
- A component made with no forges at all is valid (`limits::check`
  refuses it today only if it does; it must not).
- Tests (the crate's library tests, a new `src/tests/runtime_forges.rs`):
  `an_added_forgejo_starts_and_answers_calls_as_a_configured_one`,
  `an_added_github_app_starts_and_mints_as_a_configured_one`,
  `an_added_forge_without_access_answers_not_connected_until_connect`,
  `a_removed_forge_ends_its_calls_once_and_emits_nothing_after`,
  `a_slot_freed_by_removal_takes_another_number`,
  `an_add_past_the_forge_bound_or_of_a_configured_number_is_ignored`,
  `a_component_with_no_forges_starts_and_closes`; every existing test
  passes unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md`.
- `skein/docs/design/forge-connection.md`, sections 2.1 (what crosses),
  3 (start: 3.1 what the owner hands it, 3.5 access at runtime) and 7
  (entities and limits).
- `skein/docs/foundation/programming-model.md`, section 1 (and its
  bounded-tables rules the component keeps).

## Owns and may change

- Owns: `skein/crates/skein-forge-connection/**`.
- Nothing outside it. temper's use (`temper-service/src/forge.rs`) is
  `fr-svc-forges`'; a `Request` match outside the crate that the compiler
  forces (none is expected: temper builds requests, it does not match
  them) gets its arm and is named in the report.
- Options: if `Remove` while a call is in flight cannot end the call at
  once without breaking the component's one-answer rule, end it as
  `Close` ends calls, at the drain, and say so; if a slot keyed by number
  needs a `Map` the crate's rules allow, use it.

## Steps

1. Tables sized for every slot; per-forge state found by number.
2. `Add`, `Remove`; their facts.
3. Tests in `src/tests/runtime_forges.rs`, built on `src/tests/fixtures.rs`.
4. Narrow: `heavy cargo nextest run -p skein-forge-connection --lib -E
   'test(/runtime_forges/)'`.
5. `cargo fmt`; `heavy cargo clippy -p skein-forge-connection
   --all-targets -- -D warnings`; focused tests of
   `skein-forge-connection`; `heavy cargo check -p temper-service --tests`
   (its direct dependent). Old-forms from `skein/` (`python3 -I -B
   scripts/old_forms.py`). Commit and push.

## Report

As `notes/common.md` says; under `## Design`, what
`skein/docs/design/forge-connection.md` should gain (sections 2.1, 3.5,
7.2) for its designer.
