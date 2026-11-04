# Spot review of debt-3's smith change (2026-11-04, at 4d39c94f9f)

Read-only reviewer, using the user's quality criteria, on `git diff 4d39c94f9f^1 4d39c94f9f -- smith/`.

Verdict: clearer than before and worth keeping. 25 fields gone (`Stages` 55 to 44, `Domain` 27 to 22,
`HostIo` 34 to 30, `DeliveryIo` 10 to 6). Net +1,693 lines: reviewer's split +810 production, +883
tests (26 new tests). Production growth is mostly the cost of 5.4's exhaustive forms and is earned.
About 200 lines could still go without changing any machine, about half in tests. No correctness bug.
All findings are style; most repeat a spec's own sketch, so a follow-up, not a spec breach.

1. `LiveUnsent` splits the design's `Live` in two (`service/teardown.rs`): 10 arms treat `Live | LiveUnsent`
   alike. Use `Live { failure: Option<Failure> }`, as `agent.md` section 6 names one phase. About -15.
2. Delivery tests repeat setup (`smith-local-domain/src/parent_tests.rs`): add `call_delivery(..)`,
   `saved_intent(..)` and a notice helper, about -80, and the `too_many_lines` expect goes. In
   `smith-agent-service/src/service/tests/lifecycles.rs` share `decoded_start(mounts)` with `tests.rs`
   and an `assert_cancel`, about -18.
3. `Roots` assigns across arms (`service/roots.rs` `close_next`, `adopt`): return from the match as
   `Close::root_answered` reads, or use a `left` counter as `host_io/close.rs` does. About -20.
4. `Line` shadows `Last` (`smith-agent-service/src/terminal.rs`): `remember` builds a `Last` directly. About -15.
5. Test scaffolding in production (`host_io.rs` forwards `spawned`, `adopted_input`, `hold_read`,
   `read_held`): delete, -16. Add `Child::process_mut()`, about -8. Reduce `Child::spawned`, about -6.
6. `plain_action` has two near-identical arms (`host_io/delivery.rs`): one match, about -10.
7. Taking the idle history is written twice (`smith-local-domain/src/chat/machine.rs`): `Chat::take_idle()`. About -10.
8. Three builders for one answer record (`delivery/machine.rs` `save_answer`, `reconcile.rs`): one
   `save_answer(name, delivery, landed, after, out)`. About -15.
9. Transitions take three forms: one rule: `Copy` state uses `match *self`; a same-state change matches
   `self` in place. About -8.
10. Naming: `Roots::reported`, `Close::root_answered`, `Close::shutdown` mutate but read as queries
    (use `take_report`, `latch_shutdown`). Panic text at `host_io/delivery.rs` "marker inspection" is stale.
