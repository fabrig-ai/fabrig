+++
id = "copy-surface"
part = "copy-plan"
kind = "interface"
needs = ["migrate-request"]
files = ["skein/crates/skein-forge-setup/src/host/boundary.rs", "skein/crates/skein-forge-setup/src/host/run.rs", "skein/crates/skein-forge-setup/src/host/settings.rs", "skein/crates/skein-forge-setup/src/host.rs", "skein/crates/skein-forge-setup/src/given.rs", "skein/crates/skein-forge-setup/src/limits.rs", "skein/crates/skein-forge-setup/src/item.rs", "skein/crates/skein-forge-setup/src/reconcile.rs", "skein/crates/skein-forge-setup/src/create.rs", "skein/crates/skein-forge-setup/src/tests/limits.rs", "skein/crates/skein-forge-setup/src/tests/given.rs"]
effort = "medium"
minutes = 40
+++

# The surface of `Copy`

## Goal

`skein-forge-setup`'s public shapes for a copy exist and every reader compiles (contracts S2), so that
the plan's body, the host's words and temper's owner each build on them apart. A `Copy` run stops at
once, `Unexpected`, until `copy-machine`.

Acceptance:

- Every shape of contracts S2 exists. `Limits::copy_deadline` is checked: zero is
  `Place::CopyDeadline`, and `deadline > copy_deadline` names `Place::Deadline`, then
  `Place::CopyDeadline`; `backoff <= backoff_max < deadline` stands as it does.
- `Outcome::Created` carries `branch`, the default branch `Create` holds.
- `given::check_copy` refuses a plan that is no `Copy` (`Place::Plan`), holds the org, the name and the
  files to `create_fields`, and holds `from` to `Place::CopyFrom`: it starts `https://`, has at least
  one byte after, at most `string_bytes`, every byte visible ASCII. The host asserts it, as it does
  `check_create`.
- `Run::new` takes a `Copy`'s deadline from `Times::copy_deadline`; a test in `src/tests/host.rs` shows
  it: a `Copy` admitted at time 0 is cut at `copy_deadline`, not `deadline`, once `copy-machine` is
  there; here, `Plan::Copy` is `writes()`, `name()` is `b"copy"`, `dialect()` is `Forgejo`, and a
  GitHub forge stops it `Unexpected`.
- `heavy cargo nextest run -p skein-forge-setup -p skein-forge-setup-world` pass;
  `heavy cargo check --tests -p temper-service -p temper-forge-setup -p temper-fake` passes.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, S2.
- `skein/docs/design/forge-setup.md`, 4.4, 4.10, 6.3 and 6.4.

## Owns and may change

- The files of the header, and `skein/crates/skein-forge-setup/src/tests/*.rs` where a literal needs
  `copy_deadline` or a pattern `branch`.
- **Fixtures, the field alone:** `skein/tests/forge/setup/src/fixtures.rs`, and every `limits::Limits`
  literal `rg 'backoff_max:'` finds in skein.
- **temper, the producer's lines alone:** `temper/crates/temper-forge-setup/src/limits.rs` (`plan()`
  and `small()`, `copy_deadline` 1800 s, or the test fixture's own small value),
  `temper/crates/temper-forge-setup/src/tests/given.rs`,
  `temper/crates/temper-forge-setup/src/given.rs` and
  `temper/crates/temper-service/src/setup/settings.rs` (the `Place` arms: `Place::CopyDeadline` and
  `Place::CopyFrom` beside `Place::Deadline` and `Place::CreateRepository`),
  `temper/crates/temper-service/src/setup/lifecycle.rs` (`Outcome::Created { id, .. }` where it reads
  the id; `Outcome::Copied { .. }` in its or-pattern of outcomes no plan of the service's has;
  `Event::Said { .. }` ignored; `Item::Copying` as an item the service does not keep; `Plan::Copy` in
  `kind`), `temper/crates/temper-service/src/setup.rs` (`Plan::Copy` in its match), and each `Why`
  match: `temper/crates/temper-service/src/setup/lifecycle.rs` and
  `temper/crates/temper-service/src/setup/public.rs`, `Taken`, `Empty` and `TimedOut` beside `Unexpected`,
  whose words they take for now. `temper/crates/temper-fake/src/backends/tests.rs`, if its `Created`
  pattern needs `..`.
- `skein/crates/skein-forge-setup/src/reconcile/tokens.rs`, the `Why` arm of `unexpected`.
- `skein/crates/skein-forge-setup/src/host/connection.rs`, `forgejo_row_bound`: `Request::Migrate` and
  `Request::DeleteRepository` are in its `Some(0)` arm. In the `None` arm they compile, and `open` then
  refuses the send and halts the run `Unexpected`; if `migrate-request` left them there, move them.
- Pre-approved:
  - `Machine` gains no variant here: `forgejo_machine` returns `Err(about(.., Why::Unexpected))` for
    `Plan::Copy`, and `copy-machine` replaces that arm.
  - `item.rs`: `Copying` needs no decoder beyond its byte: `pub fn copying(bytes: &[u8]) -> bool`,
    true for exactly `[1]`.
  - If a test pins the count of `Place`s, `Why`s or items, it takes the new value.
  - temper's `SetupStage` needs no change: `Copy`'s stages are `Create`'s.

## Steps

1. The enums and fields of S2. 2. `Times` and `Run::new`. 3. `check_copy`, on `check_create`'s pattern,
and the host's assertion. 4. The compiler's arms. 5. The tests, each biting: remove the host's use of
`copy_deadline`, and let `check_copy` take an `http://`, each once. 6. The implementer's checks; commit
and push.

## Report

As the implementer's text asks; each temper line you changed.
