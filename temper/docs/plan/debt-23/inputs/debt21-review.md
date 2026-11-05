# Spot review of debt-21 (2026-11-05, merge 9a769cbde1)

Read-only reviewer, using the user's quality criteria. Nothing was run: whether pins bite and mutants survive is by reading.
Line numbers are at the merge.

Verdict: a good merge. D009 is complete and clean; no secret reaches `Debug` or a log; none is copied or held longer. The Hooked
and Connection enums are a real improvement. Of hooked-states' +100, about +70 is earned.

1. **Bug (latent): `connect` forgets a close.** `skein-forge-connection/src/connection.rs:330-334`. `Start::Closing` only happens after `close`
   or `abort`, so the state is already `Closed { closing: true }`, but the arm rebuilds it with `closing: false, ended: None`. `step` then
   skips the link pump (:355) and `github.rs:926` doesn't break. `calls.rs:433-437` re-closes in the same `drive`, so no outward
   difference found. Fix: `Start::Closing => state`, and pin `c.close(..); c.connect(..); assert!(c.closing())`.
2. **Gap: H6 is pinned on one edge.** `skein/tests/forge/machine/tests/suite/hooks.rs`: nothing advances time across `Done`. Mutants should
   survive at `hooks/connection.rs:426` (keeping `until`) and :377 (recomputing after a late offer). One test: send at t=9, open at t=10;
   refuse, offer at t=18, closed at t=19.
3. **Owner's call: D020's deadline is closed by renaming.** `connection.rs:96-102,152,215-238`: nine `hold`/`rest` calls in `component*` still choose
   the deadline, against forge-connection 8.1. `until` sits in six variants, and `deadline` mirrors it. Turn `follow`'s match into `fn deadline(&self)`,
   delete the field and the calls, fold `hold` into `rest`, about -17. fabrig/debt decided: do it.
4. `Closed.exchange`, `Idle.exchange`, `Closed.until` hold data 8.1 says they don't; the last two exist for tests
   (`a_connected_notice_keeps_the_in_flight_exchange`, `a_start_answer_can_hold_a_closed_connection_until_close_clears_it`). Drop all three and those tests, about
   -25. `Receiving::Json { body, next }` makes `discard_redirect`'s resets one line, -5.
5. `hooks/connection.rs` slack, about -34: `NextOwed` (:36) to `Option<http::Reuse>`; `Hint { value, offer }` (:50) is an `Option` plus a flag: one
   enum only if H5's outward order holds; `call` :182-197 with :211-213 through a `refuse` helper, dropping `signature.expect`; no-op `follow()` at :272, :280,
   :330; the `pump` guard at :306-308; rename `Answering.hint` to `held`.
6. **Owner's call: hand `Debug` is D009's biggest cost.** `skein-oauth-accounts/src/boundary.rs:75-164`: 90 lines hide `Redirected.uri` and `Visit.url`;
   `oauth::Request` and the fake's `Event` and `Body` about 45 more on three fields. Make those five fields `Secret` and derive `Debug`, about -130.
   About 106 lines across the trees name the variants, and only sites reading the bytes change. fabrig/debt decided: do it.
7. D009 noise, about -15: `component/sign_in.rs:174-177` and `skein-fake-oauth/src/lib.rs:482-489` hand-write `Option::clone` (`Secret` is `Clone`), and
   `post`'s new expects go; `record.rs`'s `secret` helper was re-inlined; `expose_optional` (fake `lib.rs:751`) duplicates `secret_ref` (`record.rs:273`);
   `temper-service/src/clients/google.rs:262`; `component/github.rs:847` tests one value twice.
8. jig D017: `jig-domain-tasks/src/closing.rs:270` clones `Ending` before `begin_effects`' early return: gate first. `jig-domain-core/src/settings/application.rs`:
   `named` returns `(u32, u32, u32)`: a small `Name` struct. `translate` returns `None` on a value mismatch, so `keep_proposal` files such an item under jig
   where it used to abort, and `drop_project`, `restore` skip silently: keep the abort on the value mismatch.

Caveat to record: oauth types had no `Debug` before; `subject`, `metadata`, `granted_scopes`, `issued_client_id`, `Redirect.error` now print raw. Not credentials
under oauth.md 4, but lib.md 6's carrier sentence reads wider: accept in writing or redact. `debug.rs` has no `Component` or `Issuer` rows.
