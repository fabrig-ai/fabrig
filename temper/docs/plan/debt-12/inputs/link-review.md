# Spot review of skein D016 (2026-11-04, merge 682451a1bf)

Read-only reviewer, using the user's quality criteria. Nothing was run: every "would stay green" is by
reading. Line numbers are at the merge.

Verdict: the copies really went (lines naming transport internals in the four files fell from 177 to 10)
and the change is good. `Link` reads correct on `close_notify` order, abort, half-close and the close bound.
Production +57, tests +670 (20 new tests, none moved). About 65 production lines are avoidable, which would
land near break-even. D016's file sizes were whole files, not transport, so the expected saving was too high.

1. **Gap: forge-connection's close bound has no test.** `skein-forge-connection/src/component.rs:581-583`, `:613`.
   The other three owners pin the stalled close. Deleting `link.fire` here leaves everything green while a
   stalled peer holds `Lifecycle::Closing` open forever. Add a machine test (about 20 lines) and a room hold on
   `Peer` like setup's `hold_room`.
2. **Risk: `Upload` and `Link` derive `Debug` over secret bytes.** `skein-link/src/upload.rs:6`, `link.rs:47`.
   `Upload` holds the OAuth token POST body (`exchange.rs:234`); `HttpRequest` refuses `Debug` for that body
   (`skein-oauth/src/machine.rs:87`), and `oauth.md` says such bytes stay out of `Debug`. Not live: no owner
   derives it and `clippy::use_debug` denies formatting it. Hand-write both, lengths only, with a test.
3. **Risk: forge's closing loop can spin on an orphaned `heard`.**
   `skein-forge-connection/src/component/calls.rs:121-126`. For a closed connection `ready()` includes
   `!heard.is_empty()` (`connection.rs:906-907`), but `pump` never pops `heard` and `close` doesn't clear it.
   Narrow, and bounded by the socket's close. Test `c.link.has_work()` in both `if`s.
4. `link.rs` carries about 30 lines it can derive (`:41-45`, `:54`, `:83-85`, `:139-164`, `:167-197`, `:228-236`):
   `held` and `Ask` restate `ending` while `socket` is `None`. Derive `Copy` on `Ending`, match
   `(self.ending, event)` in `pump`, add one `end(next, below)`.
5. A missing primitive, about 25 lines: "is the close deadline due" is spelled out at `skein-forge-setup/src/host.rs:312-315`,
   `:377-380`, `skein-llm-connection/src/component.rs:474-477`, `skein-oauth-accounts/src/component.rs:1253-1256`. Add
   `Link::due(now)`. `link_env` is 7 lines in four crates, with a fifth inline at llm `component.rs:880`: one
   `env.with(skein_link::Limits { .. })`.
6. Dead code from the migration, about 10 lines: forge `connection.rs:364-366`, `component.rs:536`; llm
   `component.rs:373-375`, `:384-386`. Make `Link::socket()` test-only or remove it.
7. oauth's plaintext shim contradicts `link.md` section 1 (`exchange.rs:265-267`, `:334-336`). Leave it: removing it
   retimes requests, which is the user's call.
8. **Risk:** forge's `pump` moves the link above its guard (`connection.rs:383-401`, `:480`), so link events run while
   `answer_pending` or `Phase::Closed`. It heals by reading. Run `link.pump` above the guard only when `closing`.
9. `Upload`'s face loses information, about 15 lines (`upload.rs:17`; forge `connection.rs:770`; setup
   `host/connection.rs:432`): have owners test `next == Down::Finish`, and add `Upload::of(Option<Box<[u8]>>)`.
10. Weak pins: `an_idle_tls_connection_sends_close_notify_before_socket_close`
    (`skein/tests/llm-connection/tests/suite/failures.rs:614`) can't fail on order: rename it, or drop `Stream`s after
    `Close`. Setup's bound at `skein/tests/forge/setup/src/host.rs:712-722` allows two `close_timeout`s where one is the
    rule. Nothing drives `Up::Failed` or `Up::End` during `Flushing`, or `close` after TLS `Failed` (`link.md` 3.3).
