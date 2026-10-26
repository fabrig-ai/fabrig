# Supplement jig-17.s2: a busy watch is opened again, and closed once answered

Added by ruling 319 (the designer's answer of 2026-10-18T05:52:56Z,
ruling 316). Read first: jig's `AGENTS.md`,
`docs/development/workflow.md`; this plan's README, "Rules of this
plan"; jig's `docs/design/client/domain/client.md`, 8.1 ("Counted until the engine
has ended it", "Dropped is not ended", "Bounded") and 8.2 (`me`).

Scope: jig's client, `crates/jig-client-domain`, as client.md 8.1 and
8.2 say at jig `544a57091`, and the tests and mappings of its users
that these rules change (Files).
Needs: nothing.

Shape:
- **Busy is dropped, not ended:** `watches.rs:153`'s `WatchEnd::Busy`
  joins `Dropped | Stopping` (`:148`): opened again after the backoff,
  its handles behind.
- **`me` ends only with the sign-in or its version:** for the watch
  whose `own` is set, every end but `SignedOut` and `NotServed` is
  opened again after the backoff, the link offline until its next
  delivery (`:172-174` already sets offline when nothing ends it).
- **A closing waits for its opening's answer:** `domain.rs:244-247`
  (`unwatch`) and `watches.rs:132-135` (`refuse_watch`) send
  `Down::Close` at once for a named watch. A watch whose opening has no
  answer yet (no `WatchEvent::Opened`, no delivery, no terminal) is
  marked closing and sends nothing; `Opened` then sends its closing,
  and a terminal removes it, as `ended` does for a closing watch
  (`:143-146`).
- **One kept for `me`:** `domain.rs:181`'s `self.watches.len() ==
  env.limits.watches` refuses the application's watch at one fewer
  while the table holds no `own` watch.

- **`Why::Busy` goes:** after the first rule it has no constructor
  (`watches.rs:153` is its only one); the variant leaves `Why`
  (`boundary.rs:83`) with its arms: `examples/ops/client-domain/src/family.rs:145`
  and `:271`, `testing/jig-client-test-domain/src/family/client.rs:204`.
  The engine's `core::reads::Why::Busy` and the ops wire's
  `WatchEnd::Busy` stay.

Files, beyond `crates/jig-client-domain/src`: each test that leaves a
page with its watch unanswered and expects an immediate `Down::Close`
steps `Up::Watched { watch, event: WatchEvent::Opened }` for each such
watch first: in the crate, `tests/watches.rs:28-30` and
`tests/versions.rs:86`; beyond it,
`testing/jig-client-test-domain/src/tests/root.rs:330` (106 lines
already: a helper, `open_families`), `testing/jig-client-test-view/src/tests/native_effect.rs:160`,
`testing/jig-client-test-view/src/tests/native_hold.rs:240`,
`examples/ops/client-domain/src/tests.rs:342` and `:480`.

Test shape: step tests in `crates/jig-client-domain/src/tests/watches.rs`,
each under 20 ms: `a_watch_refused_busy_is_opened_again_after_a_backoff`
(the row `(WatchEnd::Busy, Why::Busy)` of
`a_refused_watch_ends_each_of_its_handles_once`, `:76`, leaves its
table, `ROWS` 4: its expected value changes by the design, named in the
commit body); `me_refused_busy_is_opened_again_and_the_link_is_offline`;
`a_watch_closed_before_its_opening_is_answered_closes_once_opened`;
`a_watch_closed_then_refused_sends_no_closing`;
`the_application_s_watches_leave_one_for_me`.
Proof: the gate.
Size: aim for at most 80 production lines; past it, the report says
why.
Don't: change the link's exchanges, the wire, or the engine's rooms;
change `Ended`'s reasons for an application's watch other than busy.
A further test whose expected value changes with these rules is
refreshed and listed in the commit body.
