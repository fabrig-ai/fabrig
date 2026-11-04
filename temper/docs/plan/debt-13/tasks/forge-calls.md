+++
id = "forge-calls"
part = "forge-exchange"
kind = "feature"
needs = ["forge-dialect"]
files = ["skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/calls.rs", "skein/crates/skein-forge-connection/src/component/github_calls.rs", "skein/crates/skein-forge-connection/src/component/github_logs.rs", "skein/crates/skein-forge-connection/src/tests/**", "skein/crates/skein-forge-connection/src/component/tests.rs"]
effort = "high"
minutes = 75
+++

# A call's plan sits in its dialect's variant

## Goal

`Waiting` holds a plan in a two-variant enum and three `Option`s only GitHub reads, and each use re-checks the
dialect with an `unreachable!`. Here a call has one `Side`, chosen where it meets its forge, and everything below
reads its own variant.

Acceptance:

- `rg -n 'CallPlan' skein/crates/skein-forge-connection/src` finds nothing.
- `rg -n 'unreachable!\(.*(Forgejo (admission|heard|step|pump)|GitHub plan)' skein/crates/skein-forge-connection/src
  -g '!**/tests/**'` finds nothing.
- `rg -n 'fn (forgejo|github)_plan' skein/crates/skein-forge-connection/src` finds nothing.
- `Side` is matched against `Dialect` in one place, `drive_calls`, and the cross arm finishes the call
  `api::Error::Unavailable`, with a test that builds that pairing and sees the answer.
- `Auth::App => unreachable!("ordinary GitHub plans use Installation or no credential")` is gone: `send_github_call`
  answers that encoding `api::Error::Refused`, as it answers one that fails to encode.
- `heavy cargo nextest run -p skein-forge-connection -p skein-forge-machine-world -p skein-forge-protocol-world -p
  github-conformance -p skein-fake-peers-tests -p temper-engine-protocol-forge-world` passes, the same tests as at
  base.
- The crate's production lines fall from what `forge-dialect` left.

## Inputs

- `temper/docs/plan/debt-13/notes/common.md`
- `temper/docs/plan/debt-13/notes/contracts.md`, 2
- `skein/docs/design/forge-connection.md`, 3.1, 4.1, 7.1 and 8.2

## Owns and may change

- **Owns:** `Waiting` and `Side` in `component.rs`; `component/calls.rs`, `component/github_calls.rs`,
  `component/github_logs.rs`.
- **May change:** `component.rs`'s `shed`, `connect` and any other reader of `Waiting`'s four fields, those lines
  alone. `src/component/tests.rs` for the new test.
- **Not to touch:** `connection.rs`. `connection::Heard`, `Refusal` and `Exchange` stay as they are, with their
  `unreachable!`s at `call_heard`, `hear_github_call` and the two `start` calls: `forge-exchange` removes them.
- **Options:**
  - A function that holds `&mut self` can't also take `&mut ForgejoCall`: read the variant with `Waiting::forgejo()` or
    `github()`, inside the function, by `let ... else`. A `None` takes the path of an unbegun call.
  - If `waiting.plan = None; waiting.route = None` in a discovery's re-begin needs the call's `discovery` to survive,
    it does: `GitHubCall.plan` and `route` go `None`, and `discovery` and `discovered` stay.
  - `Box` on `ForgejoCall` is `notes/contracts.md`'s, for `forge-exchange`'s exchange in it. Keep it.

## Steps

1. `component.rs`: `Side`, `ForgejoCall`, `GitHubCall`, and on `Waiting`:

   ```rust
   fn begun(&self) -> bool;                         // what `plan.is_some()` asked, for either dialect
   fn forgejo(&mut self) -> Option<&mut ForgejoCall>;
   fn github(&mut self) -> Option<&mut GitHubCall>;
   ```

   `Waiting::origin` reads `GitHubCall.route`.
2. `drive_calls` is the one place. After the checks it has today (revoked, the forge's start), it matches:

   ```rust
   match (&waiting.side, &mut entry.dialect) { ... }
   ```

   Same dialect, or `Unbegun` with either: that dialect's path, as `drive_github_call` and `assign` + `pump_call` run
   today. A variant that isn't its forge's: `finish(call, id, Err(api::Error::Unavailable), up)`.
3. `CallPlan::Forgejo(plan) => plan.error(..)` and its `CallPlan::GitHub(_) | None => error` become one method on
   `Waiting`, used by `expire`, `drive_calls` and `pump_call`.
4. `call_step`, `pump_call`, `hear_github_call`, `github_call_step`, `follow_github_log` and the rest read the variant.
   `github_plan` and `forgejo_plan` go.
5. The test: a `Waiting` of the other dialect put under a forge, through a `#[cfg(test)]` helper on `Component`,
   driven, and answered `Unavailable`. Break the arm by hand and see the test red; restore.
6. `cargo fmt`, `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`, the tests above, commit,
   push.

## Report

What changed, the commit, what ran with counts, the rule broken and what failed, and the net lines, production and
tests apart.
