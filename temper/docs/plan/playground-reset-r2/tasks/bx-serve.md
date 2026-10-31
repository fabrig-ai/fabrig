+++
id = "bx-serve"
part = "box-groups"
kind = "feature"
needs = ["fr-service", "bx-steps"]
files = [
  "temper/crates/temper-service/src/box_groups.rs",
  "temper/crates/temper-service/src/route/box_link.rs",
  "temper/crates/temper-service/src/tokens.rs",
  "temper/crates/temper-service/src/tests/box_groups.rs",
  "temper/crates/temper-service/src/tests/box_link.rs",
]
effort = "high"
minutes = 90
+++

## Goal

Serve's side of the box's groups (`notes/contracts.md`, 3;
`temper/docs/design/draft/playground-reset.md`, 3 and 4): before it asks
`start N`, serve writes `<data>/box/groups/N` (0600 in a 0700 directory,
a temporary file renamed) from forge N's record; and it gives the
service a primitive that sends `remove N` and hands back the box's
`removed N` (or `failed N WHY`), which `rm-service` (D4) routes the
domain's kept `RemoveGroup` act to and from. Without a box (no pipe
adopted), a start asks nothing as today and a removal is answered
removed at once.

The names are fixed (D4's specs use them):

```rust
// temper/crates/temper-service/src/box_groups.rs
/// Writes forge N's group file, then asks `start N` (contract 3).
pub(crate) fn start_group(engine: &mut crate::engine::Engine, forge: u16, request: skein_lib::Token);
/// Sends `remove N`; its answer arrives through `take_removed`.
pub(crate) fn remove_group(engine: &mut crate::engine::Engine, forge: u16);
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GroupRemoved { Removed, Failed(Box<[u8]>) }
/// The box's answers to `remove`, oldest first.
pub(crate) fn take_removed(engine: &mut crate::engine::Engine) -> Option<(u16, GroupRemoved)>;
```

Acceptance, `temper-service`'s lib target:

- `tests::box_link`: `remove_writes_remove_and_reads_removed`
  (`Link::remove(7)` writes `remove 7\n`; the reply `removed 7\n`
  gives `Out::Removed { forge: 7 }`); `a_failed_removal_is_its_answer`
  (`failed 7 busy\n` while a remove is asked gives `Out::RemoveFailed`);
  `removes_queue_behind_a_start_and_a_stop` (one line in flight, as
  `held`/`stop` queue today; a second remove of the same forge is
  sent once); `a_closed_pipe_drops_pending_removals` (no answer: the
  kept act is sent again at the next start); every existing
  `tests::box_link` test unchanged.
- `tests::box_groups` (new), driving `Service` as `tests::system` does
  (its staged submissions and completions):
  - `start_group_writes_the_file_then_asks_start`: with a box adopted
    and a data root, `start_group(N)` submits, in order, the `box` and
    `box/groups` directories (0700, `Exists` accepted), the temporary
    file (create, 0600), its write, its close, the rename to `N`, and
    only after the rename's completion writes `start N\n` on the pipe;
    the bytes written are `temper_forge_setup::group::compose` of forge
    N's definition's `setup` member, and `group::read` accepts them.
  - `the_playground_group_file_says_so`: `"playground": true` for a
    `SetUpGroup` ask (today's demo or playground group) and for the
    forge the `Playground` row names; false for a `SetUp` (owned).
  - `a_failed_write_refuses_the_ask_and_asks_nothing`: an error on any
    step: the ask answered as a busy link answers it today, the forge's
    state `SetUpFailed` with a reason naming the group file, no
    `start N` line.
  - `a_forge_without_a_record_or_setup_is_refused` (no definition, or
    its `setup` null: no file, the ask refused as above).
  - `without_a_box_start_group_asks_as_today` and
    `without_a_box_remove_group_is_removed_at_once`.
  - `remove_group_answers_through_take_removed`: `remove 7` written,
    `removed 7` read: `take_removed` gives `(7, Removed)` once, then
    none; `failed 7 busy` gives `(7, Failed(b"busy"))`.
- The service's focused tests pass with the route arms changed (the
  `SetUp` and `SetUpGroup` arms now write the file first).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md` (what you may
  change without asking).
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 2 (the
  definition; the `Playground` row), 3 (the file and the pipe), 4
  (the machine's ports: the definition's listens are already them).
- `temper/docs/design/draft/playground-reset.md`, 3, 4.
- `temper/docs/design/protocol/setup.md`, 7.1 (the pipe), 8.4 (the
  demo's group ask).
- `temper/docs/plan/playground-reset-r2/tasks/bx-steps.md` and
  `bx-machine.md` (`group::compose`, `group::path`, `group::DIRECTORY`;
  the box's `remove N` / `removed N` / `failed N WHY`).
- The merged `fr-service`'s report and code: where forge N's
  `ForgeRecord` (contract 2) is read in the service (the domain's
  accessor `forge-families` gave, or fr-service's runtime registry),
  and what the `SetUp`/`SetUpGroup` arms of `route.rs` became.
- Code: `temper/crates/temper-service/src/route.rs` (the box arms),
  `route/box_link.rs` whole, `system.rs` (`adopt_box`, `complete`, the
  submissions drain near line 604, the operations count near line
  957), `store.rs` (how the store adopts its root and submits, the
  model for a small file writer), `tokens.rs`;
  `temper/crates/temper/src/lib.rs`'s `serve` (its `adopt_box` call).

## Owns and may change

- Owns `src/box_groups.rs` (new), `src/route/box_link.rs`, and their
  tests (`src/tests/box_groups.rs`, new, with its `mod` line in
  `src/tests.rs`; `src/tests/box_link.rs`, `src/tests/qa_box_link.rs`
  where a literal or reply changes).
- `src/tokens.rs`: a new prefix `BOX_GROUPS` (the next free nibble,
  `12 << 60`) for the writer's operations.
- Pre-approved hooks in files `forge-records` owns, call sites only:
  - `src/route.rs`: the `SetUpGroup` and `SetUp` arms call
    `box_groups::start_group(engine, forge, request)` in place of
    `engine.box_link.ask(...)` plus its drain (whatever else the arm
    does stays); add `mod box_groups;` where the crate declares its
    modules (`src/lib.rs` or `src/route.rs`, as `box_link` is).
  - `src/system.rs`: `adopt_box` gains a parameter `data: Fd`, the data
    directory's root, adopted with the io's root adoption as the store
    adopts its own (refused like the pipes when it cannot be adopted);
    `complete` routes tokens of the `BOX_GROUPS` prefix to
    `box_groups::complete`; the submissions drain takes the writer's
    submissions as it takes the store's; the operations count adds
    `box_groups::OPERATIONS`.
  - `temper/crates/temper/src/lib.rs`'s `serve`: under `--box`, open
    `<data>` with `skein_shell::open_root` (as `box_command.rs` does)
    and pass it to `adopt_box`, closing it on refusal as the pipes are.
- `notes/scope.md` covers the rest the change forces (a field the
  engine's box state needs lives in `box_link::Link`, not in
  `engine.rs`, so `engine.rs` stays untouched; if it cannot, add one
  field there and say so).
- Not yours: the domain's kept acts and `RemoveGroup` request (D4's
  `rm-domain`), `rm-service`'s routing of them to `remove_group` and of
  `take_removed` to the domain; the box (`bx-box`).

## Steps

- `box_link.rs`: `Link` gains `remove(&mut self, forge: u16, out:
  &mut Queue<Out>)`, queued as `stop` is (one line in flight; a bounded
  list of pending removals, capacity `group`-independent: 8, a ninth
  refused with `Out::Named` and `Out::RemoveFailed { why: b"busy" }`),
  `Out::Removed { forge }` and `Out::RemoveFailed { forge, why }`,
  `Reply::Removed(u16)` parsed from `removed N\n` with the same number
  rules as `started`; a `failed N WHY` while the line in flight is a
  remove is `RemoveFailed`; `closed` drops pending removals. A
  removal queue `removed: Queue<(u16, GroupRemoved)>` (capacity 8, the
  oldest dropped with a `Named` log line when full) lives in `Link`,
  and `drain` pushes onto it for `Removed`/`RemoveFailed`, through
  `box_groups::removed(engine, forge, result)` (one function, so
  `rm-service` can turn it into the domain's event if it prefers).
- `box_groups.rs`, its doc citing
  `temper/docs/design/draft/playground-reset.md, 3` and contracts 3:
  - `start_group`: no box root adopted (no box): `engine.box_link.ask`
    and `drain` as the arms did. Otherwise: the forge's record's
    definition (JSON `{"forge": F, "identity": I, "setup": S}`, contract
    2), its `setup` member's bytes located with `skein_json`, `S` null
    or absent or no record: refused. `P` true when the ask is a
    `SetUpGroup` or the `Playground` row names the forge (pass it in
    from the arm: `start_group` may take a `playground: bool`
    argument if the arm knows it better; say so). The file:
    `group::compose(S, forge, P)`. One write at a time: a phase machine
    (as `store.rs`'s driver submits) through `box`, `box/groups`
    (`MakeDirectory`, 0700, `Exists` is fine; opened `Directory`), a
    temporary `N.tmp` (`Create`, 0600, truncating), write, close,
    `Rename` to `N` (replacing), close the directories; then the ask
    as today. Pending starts while one writes: a bounded queue (4),
    beyond it refused busy. A failure: the forge's state
    `Connection::SetUpFailed { why }` with `group file: WHY` and the
    ask answered refused as `Out::Answered` with `Refused::Busy` is
    answered today.
  - `remove_group`: no pipe: `GroupRemoved::Removed` queued at once.
    Otherwise `engine.box_link.remove(forge, ..)` and its drain.
  - `complete(engine, complete)`, `OPERATIONS` (the writer's in-flight
    bound: 1), and the submissions it hands `system.rs`.
- Tests: `tests::box_link` as its tests drive `Link` today (bytes in,
  `Out`s out); `tests::box_groups` through `Service` with a box
  adopted (see `tests::system`'s box adoption at its lines 172 and 239)
  and a fake data root fd, completing each staged submission by hand
  (`publish_staged`, `complete_staged`), the forge's record seeded the
  way `fr-service`'s tests seed one.

Then: `cargo fmt`; `heavy cargo clippy -p temper-service -p temper
--all-targets -- -D warnings`; narrow: `heavy cargo nextest run -p
temper-service --lib -E 'test(/box_groups::|box_link::/)'` (check the
count); then the focused tests of the service and its dependents:
`heavy cargo nextest run -p temper-service -p temper
-p temper-one-process-world -p temper-system-world` (adopt_box's
callers; use the package names the workspace gives those worlds).
Old forms from `temper/`: `python3 -I -B scripts/old_forms.py`.
Commit and push your branch.

## Report

- The accessor you used for forge N's definition, and how `P` is
  decided.
- Every caller of `adopt_box` changed (tests and worlds included).
- Whether `box_groups::removed` pushes onto the link's queue only, or
  something else, so `rm-service` knows what to drain.
