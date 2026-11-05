+++
id = "oauth-client-phase"
part = "oauth-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-oauth/src/machine.rs", "skein/crates/skein-oauth/src/machine/**", "skein/crates/skein-oauth/src/machine_tests.rs", "skein/crates/skein-oauth/src/machine_tests/**"]
effort = "high"
minutes = 60
+++

# skein-oauth: the client's `Phase` holds its exchange

## Goal

The client's clause of `skein/docs/development/debt/D022-oauth-states.md`: `Phase` holds its `Exchange`, with no `Option<Exchange>` beside it and no comparison of phases. No public interface changes, and behaviour stays. Production grows by about 25 lines; the cut's cap is 40.

Acceptance:

- `rg -n 'self\.phase (==|!=)|self\.active\b|Option<Exchange>' skein/crates/skein-oauth/src/machine.rs skein/crates/skein-oauth/src/machine` reads nothing. It reads 26 at this plan's base.
- `Client` is `{ limits, phase, next_id }`. `next_id` stays there: ids survive `Reset`.
- `Phase` is as sketched: `Idle` and `Done` hold nothing, and `Phase` loses `Copy`, `Eq` and `Hash`.
- Each `Phase` transition uses `skein/docs/foundation/programming-model.md`, 5.4's idiom: move out, leaving `Phase::Done`, and assign the match's result back. No `if let` or tuple scrutinee on `Phase`.
- `Phase`'s `Debug` is hand-written and prints today's text, such as `AwaitHttp { id, deadline }`: **never the exchange**, whose `visit` is a carrier. `Exchange` has no `Debug`. `Client`'s `Debug` stays as it is.
- The pins below pass on the base's code first, in their own commit, and unchanged after.
- A start outside `Idle` still answers as `skein/docs/design/oauth.md`, 3 says, and the running exchange goes on. **The item's last sentence is stale:** today's code answers `Failed { Limit }` or `Revoked { confirmed: false }`, and that is what stays.
- `heavy cargo nextest run -p skein-oauth -p skein-oauth-world` passes, `CLIENT_REFRESH_HEAP_CEILING` and `SIGN_IN_KEYS_HEAP_CEILING` included. If a ceiling goes over, stop and ask with the measurement. `heavy cargo nextest run -p skein-oauth-accounts-world --test suite -E 'test(debug_hides_every_oauth_credential_and_carrier)'` passes: its `oauth::Client` row holds a client in `AwaitKeys` with a `visit`. `heavy cargo check -p skein-oauth-accounts --tests` passes.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `skein/docs/development/debt/D022-oauth-states.md`
- `skein/docs/foundation/programming-model.md`, 5.4
- `skein/docs/design/oauth.md`, 3, 4

## Owns and may change

- **Owns:** the files in the header. `Phase` and `Exchange` are private, and only `machine.rs`, `machine/fetch.rs` and `machine/revoke.rs` read them. No test reads them.
- **Pins go in** `machine_tests.rs`, one table. At most 60 new lines.
- **Left out, not needed for the item:** splitting `Exchange` by stage, one free function per cell. The status keeps that departure.

## Steps

1. **Pins first,** on the base's code, as rows. Search first: skip a row an existing test holds, and say so.
   - A start refused in `AwaitKeys`, `AwaitHttp` and `Backoff`, with the exchange going on (`skein/tests/oauth/tests/suite/world.rs`'s `a_start_outside_idle_is_refused_at_once` holds the other two phases).
   - A wrong-id `Http` in `AwaitKeys`, and any `Http` in `Backoff`, keep the exchange.
   - `Redirected` outside `AwaitRedirect` emits nothing and changes nothing.
   - `Cancel` in `Idle` or `Done` is silent.
2. **The shape:**

   ```rust
   enum Phase {
       Idle,
       AwaitKeys { exchange: Exchange, id: u64, deadline: Time },
       AwaitRedirect { exchange: Exchange },
       AwaitHttp { exchange: Exchange, id: u64, deadline: Time },
       Backoff { exchange: Exchange, due: Time },
       Done,
   }
   struct Client { limits: ClientLimits, phase: Phase, next_id: u64 }
   ```

   `http_request` and `read_keys` hold the exchange on `Err` today (`machine.rs`, about 665), so they answer what to hold next, for example `enum Awaits { Redirect, Keys { id, deadline }, Http { id, deadline } }` with `fn hold(self, exchange: Exchange) -> Phase`; yours to choose inside.
3. **Traps:**
   - The placeholder is `Done`, so an early return that doesn't assign back silently ends the exchange. Check before moving out, or assign in every arm: a wrong id, `now < due`, `now < deadline`, a refused start.
   - `finish` reads `active` to see a revocation. Only the `AwaitHttp` tick still holds a revoking exchange without `Cancelled`, so move the `revoking()` split there. `Cancel` on a revocation stays `Failed { Cancelled }`.
   - The arms for `active` being `None` beside an active phase are dead, and go.
4. Break a pin by hand: make a start in `AwaitHttp` drop the exchange, see its row red; restore.
5. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-oauth --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the search's count; the rule broken and what failed; the pins an existing test already held.
