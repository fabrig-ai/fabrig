# debt-7: the service parts' contracts

The binding calls for the tasks of parts `service-accounts`, `service-worker` and `service-clients`. The
contract is `jig/docs/design/protocol/service.md`, 11: read it first. The items are
`jig/docs/development/debt/D005-accounts-host-copied.md` and
`temper/docs/development/debt/D001-reference-service-copy.md`.

`T` is `temper/crates/temper-service/src`, `J` is `jig/testing/jig-test-service/src`.

## How a part moves

- **A jig task moves.** The code leaves `J` for its product crate, in one branch, and `jig-test-service`
  keeps a thin adapter: its tokens, its root's events, its routing. Its tests keep passing unchanged at
  every commit. There is never a third copy.
- **A temper task deletes.** It deletes `T`'s copy and composes the part, and keeps an adapter the same
  way. It needs that part's jig task.
- **A branch stays in its tree.** A jig task changes no temper file and a temper task no jig file. A
  temper need that the shipped part can't meet is a `Question:`, whose answer is a jig task.
- **One writer per crate.** `jig-test-service` and `temper-service` each take one task at a time. The chains
  are: `jig-accounts-host`, `jig-worker-agents`, `jig-workers-host`, `jig-server-part`, `jig-store-host`,
  `jig-clients-host`; and the same for temper, each also needing its jig task.
- **No second reader moves by hand.** Readers of a moved item in jig's or temper's tests take a re-export or a
  path edit, whichever is fewer lines: `pub use jig_protocol_..::host::Accounts` beside the adapter is fine.

## The seam

- **No trait, no generic item, no closure, no function pointer** (`skein/docs/foundation/programming-model.md`,
  10.3). Values at `new`, typed outputs the owner pulls. `jig_protocol_store::host::Host` is the form.
- **No part names `root::Event`.** The common case, `events: &mut Queue<root::Event>`, becomes a pull: the part
  hands out `domain::Event` in order, the adapter wraps and pushes it in the same pass. Order stays as it is.
- **A difference between the services stays a difference.** "Behaviour stays unchanged" holds per service. Where
  the survey found one (`Where the copies differ`, below), the part reports the fact and each owner acts as
  today, or the difference is a value. Neither service moves to the other's behaviour, unless the list says so.
- **`cfg(test)` does not cross crates.** A test hook on a shared type stays on the service's adapter, or
  becomes a small public read if the adapter can't hold it. temper's `Store::secret_requests`, `forge_loads`,
  `erases`, `hold_mark` and `held_mark` stay in temper.

## Credentials

temper's hidden `Debug` stays, for every type that holds a credential. `Accounts` has no `Debug`, with the
`#[expect(missing_debug_implementations, reason = "...")]` temper's has. `Resources` keeps the hand-written
`Debug` that counts its entries, and its comment. `Clients` keeps its `expect` in temper, and jig's hand-written
`Debug` hiding `links` stays pinned by its test. A `Debug` no test reads is still a type's contract here: no
derive is added to a type that holds a `Secret`, a token or an environment.

## Where the copies differ

From the cut's survey, by reading; nothing was run. Each line says which holds. A test's visible events or
outcome changing is a stop, as the request says, even on a line below.

- **`Effect::InvalidSecret`:** the part reports it; jig pushes `PageRefused`, temper `Corrupt`, each as today.
- **The output-room hold:** temper holds an account event when its root queue has no room. The part offers a
  one-slot `hold`, and only temper calls it.
- **`aborting`:** temper's holds in the shared host. It changes jig only on abort paths.
- **Trust-root bounds:** `Runtime`'s bound is an `Option`; jig passes its bounds, temper `None`, which skips
  the check, as today.
- **A peer's end during a call:** a value of the server's `Limits`; each service keeps its behaviour.
  `jig/docs/design/protocol/client.status.md`, 17.4's **Departs** stays true, and follows the file's new path.
- **The server's queues:** temper's capacity 8 goes to jig's uncapped: an assert goes, nothing else.
- **`Closed` on the listener, `held`, `finished`, `valid`:** temper's, in the shared server. jig never calls
  the additions. A listener's `Closed` clearing `listening` changes jig only on a path no jig test reaches.
- **A listener closing, in the workers' side:** the part reports it; temper stops, jig ignores it, each as today.
- **Delivery evidence:** temper's decoder is a shipped function; jig keeps using the evidence raw.
- **`Store::secrets`' immediate set:** a value; `SecretsNotReady` stays jig's alone.
- **`endings`:** the bound is a value: temper's stays `journal.commits`, jig's `u32::MAX`.
- **The path bound of a program:** a value: 4096 in temper, 1024 in jig.

## Tests

- **A composition test stays where it is,** reading the adapter. A test moves only if it tests the part alone and
  can't compile where it was; it moves with the code, and its assertions don't change.
- **Moving a temper test into a jig crate moves its CPU into jig's focused budget,** and moving a test to escape
  a budget is raising it. Prefer leaving it as a composition test. If you move any, report how many, and each
  tier's figure from your run; over budget is red (`skein/docs/foundation/testing-strategy.md`, 8.1).
- **Tests that cite design:** `tests::engine::the_store_root_closes_after_delayed_driver_completions`,
  `a_cancellation_waiting_unpublished_keeps_its_completed_targets_route`,
  `tests::clients::sign_in_endings_follow_their_own_commits_under_load`,
  `tests::accounts::an_early_account_request_waits_for_the_previous_moves_dispatch` and
  `a_stalled_link_consumer_drops_nothing_and_keeps_order` are named in design documents. Keep their names and
  modules, or fix the citation in the same branch; that departs from the implementer's text for that line alone.
- A jig task runs `jig-test-service`'s tests, and the worlds that read it: `jig-one-process-world`,
  `jig/tests/channel`, `jig/tests/engine-clients` and `jig/tests/client-protocol` by their package names. A temper
  task runs `temper-service`'s and the direct dependents' tests.

## The measure

D001's **Measure** reads 1,388 at this plan's base, not the 1,369 the item says. Every temper task reports it
after its change, with the command as the item gives it, per file. Nothing here aims a line at that number; the
last task, `service-items`, reads it and says what's left.

## Status

Each jig task removes its part from `jig/docs/design/protocol/service.status.md`'s **Not yet**, 11, and updates
its date line. `temper-clients-host` removes the 2 entry from `temper/docs/design/protocol/README.status.md`, if it
holds no copy by then. This departs from the implementer's text for those two files alone.
