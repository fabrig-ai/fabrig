+++
id = "smith-hostio-inspection"
part = "smith-lifecycles"
kind = "feature"
needs = ["smith-hostio-close"]
files = [
  "smith/crates/smith-local-service/src/host_io.rs",
  "smith/crates/smith-local-service/src/host_io/delivery.rs",
]
effort = "high"
minutes = 60
+++

# smith D002, `HostIo`: file inspection as one machine

## Goal

`DeliveryIo` (`smith-local-service/src/host_io/delivery.rs`) has no methods but `new`: it is
`HostIo`'s fields, and `impl HostIo` runs them. One lifecycle sits across five of them: the one
file inspection at a time, a plain capture, a plain comparison or a marker read, held in
`plain`, `markers`, `capture_directories`, `capture_next` and `capture_deadline`. The last three
mean nothing outside a capture, and `capture_deadline` starts as `Time::ZERO` for that reason.
It becomes one state enum, each variant holding what it needs
(`skein/docs/foundation/programming-model.md`, 5.4). This closes `HostIo`'s share of `smith/docs/development/debt/D002-lifecycles-in-flags.md`
with `smith-hostio-close`. Behaviour stays. The item stays open: leave its file.

Acceptance:

- `DeliveryIo` holds 6 fields, from 10: `git`, `inspection`, `snapshots`, `capture_done`,
  `delivery_roots`, `git_environment`.
- `rg -n 'PlainScan|PlainPurpose|capture_directories|capture_next|capture_deadline|delivery\.(plain|markers)' smith/crates/smith-local-service/src`
  reads nothing.
- Matches over `Inspection` are exhaustive: no `_` arm, `matches!`, or `if let` or `let else`
  on it.
- No `pub` or `pub(crate)` signature of `HostIo` changes. `git diff --stat origin/main --
  smith/crates`, against this task's base, lists only `host_io.rs` and `host_io/delivery.rs`.
- `heavy cargo nextest run -p smith-local-service --lib` passes 114 tests: the 113 that
  `smith-hostio-close` leaves and one more. No assertion of an existing test changes but for
  the two lines step 2 names.
- `smith/scripts/old-forms.txt` does not change.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `smith/docs/development/debt/D002-lifecycles-in-flags.md`, whole.
- `skein/docs/foundation/programming-model.md`, 5.4.
- `smith/docs/design/protocol/hosts.md`, 5.5 ("A plain directory's files"): it says what the
  host does and needs no change, so no design or status file moves.
- `smith-local-protocol/src/plain.rs` and `markers.rs`: `Plain` and `Markers` keep their own
  progress; `start` borrows, `from_file` takes `&mut self`, `Plain::finish` takes `self`.
- `smith-local-service/src/tests/plain/simulated.rs` and `tests.rs`
  (`original_conflict_files_are_read_through_the_services_kernel_routes`,
  `plain_status_uses_the_runs_snapshot_and_refreshes_it_before_the_next_run`): they drive this
  through `pub(crate)` methods and do not change.

### What stays, and why

| Field | Reading |
|---|---|
| `git: Option<GitChild>` | One slot, and `GitChild` holds its own progress. The `Option` is the slot's whole state; `git` is not a file inspection and overlaps one freely. |
| `capture_done: Option<bool>` | A one-shot result the service takes (`take_capture_done`); it can wait while `Inspection` has moved on, so it is meaningful in every state. |
| `snapshots: Map<u32, Option<Snapshot>>` | The last capture's results, read by a later comparison; its inner `None` is the value *unknown*. |
| `delivery_roots`, `git_environment` | Configuration. |
| `StoreIo`'s `store`, `cleanup_warning` | The store's own machine in a slot, and a handoff. Unchanged. |

## Owns and may change

- `smith-local-service/src/host_io/delivery.rs`, whole.
- `smith-local-service/src/host_io.rs`: the two test lines of step 2, and one new test.

Not this task's: the root terminal's check in `markers_up`, which stays as
`smith-hostio-close` leaves it and at its place; `store_pass`; the `git` paths (`start_git`,
`git_limits`).

Options, each pre-approved:

- `clippy::large_enum_variant` on `Inspection`: box what it names, or a scoped `#[expect]`
  with a reason, whichever reads smaller.
- `next_capture` and `plain_action` read better cut otherwise: the enum and the table bind,
  the handlers' cut does not.

## Steps

### 1. The machine, in `delivery.rs`

```rust
/// One capture of the plain directories: from `next`, until `deadline`.
#[derive(Debug)]
pub(super) struct Capture {
    directories: List<u32>,
    next: u32,
    deadline: Time,
}

/// The one file inspection at a time; every file terminal that is neither the store's nor a
/// closing root's belongs to it (protocol/hosts.md, section 5.5).
#[derive(Debug)]
pub(super) enum Inspection {
    /// No inspection in flight.
    Idle,
    /// Scanning `directory` for a capture.
    Capturing { capture: Capture, directory: u32, scan: local_protocol::Plain },
    /// Comparing `directory` with its snapshot, for `owner`.
    Comparing { owner: Token, directory: u32, scan: local_protocol::Plain },
    /// Reading files for conflict markers, for `owner`.
    Markers { owner: Token, markers: local_protocol::Markers },
}

impl Inspection {
    pub(super) const fn idle(&self) -> bool;
}
```

`PlainPurpose` and `PlainScan` go. `next_capture` takes the `Capture` by value. A transition
out of a scan uses 5.4's idiom, with `Idle`, which holds nothing, as the placeholder:
`let state = mem::replace(&mut self.delivery.inspection, Inspection::Idle);`.

| Event | `Idle` | `Capturing` | `Comparing` | `Markers` |
|---|---|---|---|---|
| `capture_plain` | `snapshots` and `capture_done` reset, then `next_capture` | panic, today's assert, "file inspection is serial" | the same | the same |
| `next_capture`, a directory with a root | to `Capturing`, file request pushed | | | |
| `next_capture`, a directory with no root | stays `Idle`, `capture_done = Some(false)` | | | |
| `next_capture`, none left | stays `Idle`, `capture_done = Some(true)` | | | |
| `plain_status` | answers at once and stays, or to `Comparing` then `plain_action`, as today | panic, today's assert | the same | the same |
| `start_markers` | to `Markers` then `marker_action`, or `GitResult::Failed` at once | panic, today's assert, "one local file inspection at a time" | the same | the same |
| `PlainAction::Done` | `unreachable!` | snapshot in, `next + 1`, `next_capture` | `PlainStatus { changed: before != snapshot }`, `Idle` | `unreachable!` |
| `PlainAction::Failed` | `unreachable!` | `None` in, `next + 1`, `next_capture` | `PlainStatus { changed: true }`, `Idle` | `unreachable!` |
| a file terminal in `markers_up` | `unreachable!`, today's `expect` | `scan.from_file`, `plain_action` | the same | `markers.from_file`, `marker_action` |

`PlainAction::File` and `MarkerAction::File` push their request and read no state, as today:
the action came from the state's own scan. `MarkerAction::Done` assigns `Idle` and pushes the
`Git` event, also without a match. `PlainAction::Done` and `Failed` in `Idle` or `Markers` are
today's `expect("active plain scan")`.

`next_capture` runs while the state is `Idle`: its caller moved the capture out first. There
is no `follow`: a state implies no effect beyond the request its entry pushed, and
`next_capture` is the capture's next step. Each request keeps its order: the state is set
before `plain_action` or `marker_action`, as `plain` and `markers` are today.

The three asserts become `self.delivery.inspection.idle()` under the same messages.

Run `heavy cargo nextest run -p smith-local-service --lib`; commit with step 2.

### 2. Its tests

- `host_io.rs` ~1085 and ~1217: `adapter.delivery.markers.is_none()` becomes
  `adapter.delivery.inspection.idle()`, messages kept. These read private state; the machine's
  own predicate is what they ask.
- New in `host_io.rs`'s tests, a cell the plain tests do not reach, bound as `hub`:
  `a_capture_of_a_directory_with_no_root_ends_unknown_and_idle`. A hub with no delivery roots,
  `capture_plain` of directory 0: `take_capture_done()` reads `Some(false)` and
  `hub.delivery.inspection.idle()` holds.

### 3. The whole

```
heavy cargo nextest run -p smith-local-service --lib
heavy cargo nextest run -p smith-local-process-world --test suite -E 'not test(/^fuzzy::/)'
cargo fmt
python3 -I -B scripts/old_forms.py          # from smith/
heavy cargo clippy -p smith-local-service --all-targets -- -D warnings
```

Then `common.md`'s citations check; commit and push.

## Report

Add to the implementer's text: `DeliveryIo`'s fields before and after; the lib's test count;
each `unreachable!` cell a test or a world reached (none is expected, and a panic there is
today's panic moved: say which).
