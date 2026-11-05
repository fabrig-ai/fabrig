+++
id = "hook-slack"
part = "spot-review"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/src/hooks/connection.rs", "skein/tests/forge/machine/tests/suite/hooks.rs"]
effort = "high"
minutes = 45
+++

# skein-forge-connection: a webhook connection's deadline is pinned across its `Done`, and its slack goes

## Goal

Findings 2 and 5 of `temper/docs/plan/debt-23/inputs/debt21-review.md`, checked against the code. Nothing advances time across `Done`, so a mutant keeping `until` there, or recomputing it after a late offer, survives. One test pins both. Then the slack goes, about -30 lines in `hooks/connection.rs`.

Acceptance, the pin first:

- A test in `skein/tests/forge/machine/tests/suite/hooks.rs`, beside `a_body_keeps_the_deadline_that_started_at_accept`. Its shape, with `hook_deadline` 10 s: a delivery sent at t=9, answered `204` and refused by the root; open at t=10 (so `Done` restarted the deadline); `Request::Offer` at t=18, the hint goes up; still open just before t=19; closed at t=19 (so a late offer didn't restart it). The world's calls: `run_for`, `send`, `refuse`, `advance_to`, `request(Request::Offer)`, `peer(..).closed`, `hints()`.
- **It bites both ways, before any slack goes:** in `event`'s `http::Event::Done` arm, keep the old `until`: the test goes red at t=10. In `pump`'s hint arm, before its `follow()`, restart a `Head`'s `until`: `if let State::Head { until, .. } = &mut self.state { *until = env.now.saturating_add(env.limits.hook_deadline); }`: red at t=19. The report quotes both.

Then in `hooks/connection.rs`:

- `NextOwed` goes: `next: Option<http::Reuse>`. `None` is `Awaited`; its comment on `Close` moves to the field.
- `call`'s `status` chain and its `Option<u16>` go, through a small `refuse(&mut self, env, status, until, held) -> State` that holds the respond, discard and `Answering` lines, with `let Some(..) else` for the secret and the signature, so `secret.expect` and `signature.expect` go. The checks keep their order: secret 404, method 405, signature 403.
- `refused`, `offer` and `pump`'s hint arm lose their `follow()`: none changes an `until`.
- `pump`'s `if self.sent() { return; }` goes: its only caller, `hooks.rs`'s `drive`, already asks `!c.sent()`, and `stream` feeds nothing once closing. Check no other caller exists, tests included; if one does, leave the guard and say so.
- `Answering.hint` is renamed `held`, as `Head` and `Body` have it, and `held()` and `event`'s arms shorten.
- `Hint { value, offer }` stays: all six pairs are reachable. A `Done` with no hint leaves `Ready` with no value, and a `204` over a refused hint can leave `Refused` with none, so one enum would list the same pairs. This is why finding 5 said "only if".
- `heavy cargo nextest run -p skein-forge-connection --lib`, `-p skein-forge-machine-world` and `-p skein-forge-protocol-world` pass, with no assertion changed but the new test's.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/inputs/debt21-review.md`, findings 2 and 5
- `skein/docs/design/forge-connection.md`, 8.4; `skein/docs/design/forge-connection.status.md`'s 8.4 entry, which this doesn't change

## Owns and may change

- **Owns:** the files in the header. `Hooked` keeps its own `deadline` and `follow`: `Closing::Sent` has no `until`, so it can't be derived as `Connection`'s is.
- **Stays off** `connection.rs` and `component*`: `connect-close`, `connection-deadline` and `connection-states` hold them.
- **If `refuse` doesn't shorten `call`,** leave `call` and say so. **If a slack item changes a test,** restore it and say so: it isn't slack.

## Steps

1. The test; both mutants; restore.
2. The slack, an item at a time, running `-p skein-forge-machine-world --test suite -E 'test(/hook/)'` or this world's whole `hooks` module between items. Check the count: nextest's filter must select them.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-connection -p skein-forge-machine-world --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; each mutant and what failed; what of the slack didn't go, and why.

## Answers

2026-11-05. **Accept the second mutant surviving, or change the test's shape?** Neither: the test stands as pushed (`done_restarts_the_deadline_and_a_late_offer_keeps_it`, keep the commit) and the second mutant changes, as the acceptance above now says. The first spelling can't bite in any scenario: `Done` sets the offer `Ready`, so one `pump` at one `env.now` offers the hint and passes from `Answering` to `Head`; recomputing `until` in that arm gives the same time. No test shape delays that step, so don't look for one. The rule is that a late offer doesn't restart the deadline, and a late offer meets `Head`, which is where the new mutant breaks it: at t=18 `until` becomes t=28, and the test goes red at t=19. The report quotes this mutant and says the first one is equivalent, and why. If this mutant passes too, stop and ask.

2026-11-05. **`pump`'s `if self.sent()` guard.** It stays, under this spec's own exception: `HookHarness::pump` and `removal_discards_queued_notifications_and_listener_hints` call `pump` directly, and no test changes for slack. Say so in the report. The rest of the slack goes as listed.
