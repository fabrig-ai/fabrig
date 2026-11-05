+++
id = "side-out"
part = "side-out"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/calls.rs", "skein/crates/skein-forge-connection/src/component/github_calls.rs", "skein/crates/skein-forge-connection/src/component/github_logs.rs"]
effort = "high"
minutes = 75
+++

# skein: a call's dialect isn't asked again

## Goal

Finding 2, the first of part 1. `drive_calls` knows the call's dialect, and then 23 sites ask it again:
`let Some(github) = waiting.github() else { return };`. Those branches are dead, and if one were ever taken it would stall
the call to its deadline without a word. `drive_calls` takes the `Side` out of the `Waiting`, hands the dialect's drive a
`&mut GitHubCall` or `&mut ForgejoCall`, and puts the `Side` back when it returns.

Acceptance:

- `rg -n '\.(github|forgejo)\(\)' skein/crates/skein-forge-connection/src/component skein/crates/skein-forge-connection/src/component.rs --glob '!tests.rs'`
  finds nothing.
- `Waiting` has no `github`, `forgejo` or `origin` method. `begun` and `error` stay on it, read only where the `Side` is in
  place: `Component::connect`, `Common::expire`, and `drive_calls` before the take.
- `initialize_github_plan` has no `Err(api::Error::Unavailable)`, and `github_route` no `ok_or(api::Error::Unavailable)`.
- The one mismatch arm stays in `drive_calls`, answers `Unavailable`, and `a_call_of_the_other_dialect_is_unavailable`
  stays and passes unchanged. `skein/docs/design/forge-connection.md`, 7.1 says so in its last sentence, so this overrides
  the finding's "goes".
- `heavy cargo nextest run -p skein-forge-connection` passes with no test changed, beyond a call's spelling.
- Production net about -35; a result worse than -20 is worth a line in the report.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `skein/docs/design/forge-connection.md`, 7.1 and 8.2.

## Owns and may change

- **Owns:** `skein-forge-connection/src/component.rs`, `component/calls.rs`, `component/github_calls.rs`,
  `component/github_logs.rs`.
- **May change, spelling only:** `component/tests.rs`. It builds `GitHubCall` and `ForgejoCall` literals and reads
  `waiting.side` outside a drive, both of which stay valid; a `GitHubCall::new()` may replace a literal.
- **Options:** `#[expect(clippy::too_many_arguments, reason = "...")]` on a function the new parameter pushes over, in the
  wording its neighbours use.

## Steps

1. The sketch of the outer layer, in `component/calls.rs`:

   ```rust
   /// What `assign` did for a call.
   pub(super) enum Assigned {
       /// No connection for it yet.
       Waits,
       /// It keeps the connection it holds.
       Held,
       /// A connection is newly its own: its exchange starts fresh.
       New,
   }

   pub(super) fn assign(&mut self, call, id, origin: Option<u32>, base, env, io) -> Assigned
   ```

   `assign` reads no `Side`: the caller passes the origin, `GitHubCall::origin(&self) -> Option<u32>` for a GitHub call
   (`route?.origin`) and `None` for Forgejo, and on `Assigned::New` the caller sets its call's `exchange` to a new one.
   Pass the origin at the moment of the call: `github_route` may have just set it.

2. In `drive_calls`, after the checks that read `waiting.begun()` and `waiting.error(..)`:

   ```rust
   let side = core::mem::replace(&mut self.common.waiting.get_mut(id).expect("call").side, Side::Unbegun);
   let side = match (side, &mut entry.dialect) {
       (Side::Unbegun, Dialect::GitHub(gh)) => /* a new GitHubCall, then as below */,
       (Side::GitHub(mut github), Dialect::GitHub(gh)) => { /* drive_github_call */ Side::GitHub(github) }
       (Side::Unbegun, Dialect::Forgejo { .. }) => /* assign, begin */,
       (Side::Forgejo(mut forgejo), Dialect::Forgejo { .. }) => { /* assign, pump_call */ Side::Forgejo(forgejo) }
       (Side::Forgejo(_), Dialect::GitHub(_)) | (Side::GitHub(_), Dialect::Forgejo { .. }) => {
           self.common.finish(call, id, Err(api::Error::Unavailable), up);
           Side::Unbegun
       }
   };
   self.common.waiting.get_mut(id).expect("call").side = side;
   ```

   The put-back is safe after a `finish`: `Slab::retire` leaves the entity reachable until `reclaim`, so the side lives
   exactly as long as it does today. Keep the shape, not necessarily this text: two `Unbegun` arms may share code.

3. `begin_forgejo_call` makes the `ForgejoCall`, so it can't take one. Its `waiting.begun()` guard goes: it runs only from
   the `Unbegun` arm. It returns the call as well as what it returns now, because a `Step::Send` on a closed connection
   leaves the call alive with no connection and its plan must be kept; `None` when the page cap isn't known yet, so the
   call stays `Unbegun`. Choose the exact return type; the `Side` that goes back follows it.

4. Every other function of the path takes the call by reference, next to `holder` or `id`: `pump_call`, `call_step`,
   `drive_github_call`, `github_route`, `github_bot_permission`, `begin_github_call`, `initialize_github_plan`,
   `pump_github_call`, `send_github_call`, `hear_github_call`, `github_call_step`, `github_discovered`,
   `follow_github_log`.

5. Nothing between the take and the put-back reads `waiting.side`, `waiting.begun()` or `waiting.error()`: while the side
   is out those would read `Unbegun` and be wrong without a word. Inside a drive, `waiting.begun()` becomes
   `github.plan.is_some()`, and `waiting.error(e)` in `pump_call` becomes `forgejo.plan.error(e)`. Search for them when
   you're done. `Common::expire` runs only outside a drive; check that it does.

6. A comment on `Side` says it's out of its `Waiting` while `drive_calls` drives the call.

7. `heavy cargo nextest run -p skein-forge-connection`; then, because `drive_calls` runs under every forge story,
   `heavy cargo nextest run -p skein-forge-machine-world -p skein-forge-protocol-world`.

8. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge-connection --all-targets --
   -D warnings`; the citations check. Commit and push.

## Report

The common note's items. Refactor only: no new test, so nothing to break by hand. Give the count of `else` branches
removed and the production net.
