+++
id = "own-exchange"
part = "own-exchange"
kind = "feature"
needs = ["side-out"]
files = ["skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/calls.rs", "skein/crates/skein-forge-connection/src/component/entry.rs", "skein/crates/skein-forge-connection/src/component/github.rs", "skein/crates/skein-forge-connection/src/component/tests.rs", "skein/crates/skein-forge-connection/src/connection/forgejo.rs", "skein/crates/skein-forge-connection/src/connection/github.rs", "skein/crates/skein-forge-connection/src/connection/tests.rs"]
effort = "high"
minutes = 50
+++

# skein: an own exchange goes where `own` clears

## Goal

Finding 4. A forge's own `Exchange` outlives its connection: `done`, `heard` or the decoder can hold a `Minted` token after
`state.forget()`. It's memory lifetime only, as `Secret` redacts. The exchange is fresh whenever `own` is `None`, and when
the component sheds. The test that hid this stands on what production does.

Acceptance:

- `rg -n 'Exchange::new\(\)' skein/crates/skein-forge-connection/src/component/github.rs` finds nothing, and neither
  `drive_forgejo` nor `drive_github` resets the exchange where they open a connection: it is already fresh.
- Every site where `own` goes `None` (`Component::connect`, `Component::own_closed`, a call's `assign` taking the
  connection), and `Component::shed`, leaves the forge's exchange fresh.
- `an_own_exchange_goes_with_its_connection` passes and fails when any one of the resets is removed.
- `rg -n 'h\.pump\(\);' skein/crates/skein-forge-connection/src/connection/tests.rs` finds nothing.
- Production net at most +12.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `skein/docs/design/forge-connection.md`, 7.1 (this cut wrote the sentence you build), 8.1 and 8.5.

## Owns and may change

- **Owns:** `component.rs`, `component/calls.rs`, `component/entry.rs`, `component/github.rs` of `skein-forge-connection`.
- **May change:** `connection/forgejo.rs` and `connection/github.rs`: one `#[cfg(test)]` reader on each `Exchange`, beside
  `test_github_handoff`, and one to put something in `heard` if the `assign` arm needs it.
- **The approved test changes, and no other:**
  1. `component/tests.rs`: the new test.
  2. `connection/tests.rs`: `github_exchange_handoff_drops_on_loss_close_replacement_and_answer` loses its `h.pump()`.
     Without it `Loss`, `Close` and `Replacement`'s trailing close can't reach the release, since a bare `Connection` has
     no hold on its exchange. So `Loss` and `Close` leave this test for the new one, and it keeps what it can show alone:
     an answer drops the request, and a replacement retains the new one's `retention` and `comparison`. Rename it
     `github_exchange_handoff_is_dropped_at_its_answer_and_replaced`. The `Terminal` enum goes if nothing else uses it.
- **Options:**
  - If `Exchange::pump`'s `Phase::Closed` release turns out to have no other test pinning it, and a call's exchange can be
    observed after a loss in a component test, add that arm to the new test. If it can't be observed, leave the line and
    say so under `## Left`.
  - If removing a reset at open fails a test, keep that reset and say why: the invariant then has a hole to report.

## Steps

1. `component/entry.rs`:

   ```rust
   impl Dialect {
       /// The forge's own exchange goes with its connection (forge-connection.md, 7.1).
       pub(crate) fn forget_exchange(&mut self)
   }
   ```

   Both dialects: a new `Exchange` in the variant.

2. `Component::connect` (beside the `base.own.take()`), `Component::own_closed` (beside `base.own = None`) and
   `Component::shed` (in its loop over the forges) call it. `shed` leaves `own` set until the connection closes; a fresh
   exchange is safe there because after `shed` the lifecycle isn't `Live`, and both `drive_forgejo` and `drive_github`
   close and break before they pump.

3. `assign` clears `own` too, and has no dialect in hand. `drive_calls` covers it in one place:

   ```rust
   let own = entry.base.own;
   // the drive
   if own.is_some() && entry.base.own.is_none() {
       entry.dialect.forget_exchange();
   }
   ```

   This needs `entry.base` and `entry.dialect` free again after the match that `side-out` wrote.

4. Remove the resets at open: `gh.exchange = connection::github::Exchange::new();` in `drive_github`, and
   `*exchange = connection::forgejo::Exchange::new();` in `drive_forgejo`.

5. `component/tests.rs`, in the shape of the tests beside it:

   ```rust
   #[test]
   fn an_own_exchange_goes_with_its_connection() {
       // For GitHub and for Forgejo, each way own ends: the connection's loss (own_closed), a new access (connect), a
       // call taking the idle connection (assign), and the shed. The exchange is held before and fresh after.
   }
   ```

   A `#[cfg(test)]` reader says an exchange is fresh: no `request`, `decoder` or `done`, and `heard` empty. At `assign`
   production's own exchange is already drained, so that arm puts a `Heard` in by a `#[cfg(test)]` helper; otherwise the
   arm couldn't fail.

6. Break each reset in turn and watch that arm fail.

7. `heavy cargo nextest run -p skein-forge-connection`, then `heavy cargo nextest run -p skein-forge-machine-world -p
   skein-forge-protocol-world`.

8. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge-connection --all-targets --
   -D warnings`; the citations check. Commit and push.

## Report

The common note's items, naming each reset broken and the arm that failed.
