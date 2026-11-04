# Spot review of debt-7 and debt-8 (2026-11-04)

Read-only reviewer, using the user's quality criteria. Nothing was built or run: every "can fail" and
"should panic" is from reading. Line numbers are at the merge commits.

## Change A: 5bc7d570d3, `PrepareAgent` out of the core

Verdict: clearer, size mostly earned. +201 on `*.rs`: product -27, tests +228 (158 in `upgrade_transcripts.rs`).
About 35 to 45 lines could still go. Sound by reading: `transcript_from`, `.expect` at now.rs:201, `busy`.

A1. `jig-domain-core/src/routing/run/preparation.rs:511`: `route_prepared_agent` names a removed event, is
    `pub(crate)` with one caller, and its guard at :518-521 is dead (`route_activate` checks just before).
    Rename `prepare_agent`, make it private, drop the guard.
A2. `jig-domain-core/src/runs/state.rs:165`: `begin_transcript` derives `window`, then both product callers
    overwrite it (preparation.rs:531, :721). Take `window` as an argument. About -7.
A3. `temper-engine-domain/src/boundary.rs:878`: `Work::Activate` and `Work::Due` differ only in who read
    `ready` and a gate at route.rs:455, whose `stopping` half repeats `stopping::gate` and whose `settling`
    half repeats domain.rs:942 if `domain.work` is empty at that `Resume` (not proved). One
    `Work::Activate { context, ready }`. About -12.
A4. `jig-domain-core/src/tests/runs/notes_room.rs:305`: `a_busy_root_still_prepares_an_agent_with_nothing_to_load`
    branches on its input, and its closed-window row asserts the opposite of the name. Move that row to
    `a_busy_root_keeps_an_agent_with_a_window_to_load_due`. About -6.
A5. `temper/tests/engine/domain/tests/suite/upgrade_transcripts.rs:219`: two new tests end identically, and
    `pending_reply` re-spells `transcript_task`. One helper, about -15 to -20. Both pin an exact `Resume`
    count, so a harmless retiming fails them too.

## Change B: dce14840f3, service parts shipped

Verdict: clearer, size earned. Service sources -3,078 against +1,809 of new crates plus the moved 407-line
server; overlap measure 1,388 to 356. Temper's hidden `Debug` survived; `Server` and `Routed` improved.

B1. **Risk.** `jig-protocol-engine-workers/src/host.rs:46`: `#[derive(Debug)] pub struct Workers` can print
    grant values, against service.md 11: `GrantValue::Credential { token }` reaches `Lower::Write(OutputDown::Send
    { bytes })` in `out`, `io` and `Connection.below`, and all of those derive `Debug`. Nothing formats it
    today; temper's old copy derived it too. Hand-written `Debug` that counts, and a canary test.
B2. **Bug, pre-existing, now shared.** `jig-protocol-engine-clients/src/host.rs:253`: `address_text` returns
    `None` for non-loopback IPv6, yet `printed_link` (http lib.rs:276), temper's clients.rs:383 and jig's
    clients.rs:241 `.expect` it. temper's configuration.md:359 requires loopback only in standalone or with the
    printed link, so `[::]:7100` should panic on `Listening`. Refuse at `Clients::new` as provider.rs:36 does,
    or print any v6 address. Move `address_text` and `HOST_BYTES` beside `printed_link`.
B3. The biggest economy left: `jig-test-service/src/server/bundle.rs:138`'s `Pages` restates temper's
    `clients/bundle.rs` (`kind`, `target`, `file`, `content`, `field`, `demand`, the chunk pump): jig D005 and
    temper D001. Ship the helpers; routing and cache fields stay with each service. About -90 to -120 per service.
B4. `jig-protocol-worker-agents/src/route.rs:20`: `Invalid` exists three times (also temper-worker-domain
    boundary.rs:176, jig-test-worker boundary.rs:77), mapped by hand, jig's with an `unreachable!`. One enum in
    `jig-domain-host`, which links no smith. About -25.
B5. Tests that may not bite (`jig-protocol-accounts/src/host_tests.rs`): `trust_bounds_refuse_tls_alone_and_none_checks_nothing`
    tests the helper only: delete the check at host.rs:90 and nothing fails (from grep). Add a test through
    `provider_templates` with a TLS endpoint. `endings_leave_in_commit_order_behind_their_own_commit`: a
    `durable > 0` mutation passes; add a `committed(57)` step. `a_lost_listener_changes_nothing_for_the_reference`:
    `assert_eq!(service.end(), None)` can't fail, since `workers.up` is called directly.
B6. Leftovers from the move: `Option<&mut Runtime>` plus `mem::replace` (accounts host.rs:71, :80): pass by
    value. `Hosted::close` (store host.rs:165): `Queue::uncapped()` assignments. `Hosted::secrets` (:113): one
    match. `place` (clients host.rs:125) and `Workers::socket` are `pub` for one temper test each.
    temper's `Page::Service` is kept only for a test (tests/server.rs:236).
B7. The parts don't share one shape: rename store's `Host` to `Drivers` (beside `Hosted`). Accounts' `output`
    panics when empty, workers' returns `None`. Finish reads differ: `is_empty`, `idle`, `closed`.
    `host.component.mark_durable()` is spelled at 20 sites: one accessor. The `(Token, Answer)` unwrap repeats
    four times in each `worker.rs`: a helper saves about 8 per service. `route.rs`, `sources.rs` and `Workers`
    have few method docs, unlike accounts' host.
