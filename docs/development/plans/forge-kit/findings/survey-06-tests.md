# Survey 06: test costs of moving temper's forge crates into skein

Read only, 2026-10-19. Reference machine: 8 threads (development.md 11.5). Paths are under /srv/data/git/runner/src.

```
$ cat skein/.config/nextest.toml temper/.config/nextest.toml skein/scripts/check.sh temper/scripts/check.sh
$ grep -n '^\[profile' -A4 skein/Cargo.toml temper/Cargo.toml
$ sed -n 2438,2456p skein/docs/foundation/development.md; sed -n 2509,2515p ...; sed -n 444,520p skein/docs/foundation/testing-strategy.md
$ git -C temper log -12 --format='%h %cI %(trailers:key=Gate-Result,valueonly)' origin/main -- crates tests testing   # same for skein
$ awk -f /tmp/sum-pkg.awk tmp/temper/client-api/logs/20261018T224544-734.log   # PASS lines summed per package, per nextest run
```

## (a) Budgets and build profile
- Set in `.config/nextest.toml`, key `[profile.<name>] global-timeout` (a suite past it fails). skein: default 15s, fuzzy 60s (slow-timeout 3s / 20s); also browser 2m (test-threads 1), live 10m, measure 1h (no budget). temper: default 15s, fuzzy 60s, process 10m, live 20m, measure 1h. Docs: skein testing-strategy.md 8; temper docs/development/workflow.md lines 45-63.
- Both `scripts/check.sh` export `NEXTEST_DOUBLE_SPAWN=0`, then run `cargo nextest run --workspace` and `--profile fuzzy`. skein's also runs fmt, clippy, wasm checks, taste.sh (on skein-tls dirs only) and the browser profile; temper's runs fmt, clippy, the two suites.
- skein Cargo.toml profiles: `[profile.dev]` panic=abort, overflow-checks; `[profile.dev.package.skein-ui-bundle] opt-level = 3` (a tool; its comment is misplaced above it, it is rustls's); `[profile.dev.package.rustls] debug-assertions = false`. No `[profile.test]`, no `"*"` override, no opt-level for dependencies.
- temper Cargo.toml: `[profile.dev]` the same; `[profile.dev.package.skein-codegen] opt-level = 3` only. No forge crate has an override, so none moves.

## (b) Gate logs
- temper: newest passing gate log on disk is `tmp/temper/client-api/logs/20261018T224544-734.log` (land temper, gate lane, ended 2026-10-18T22:45:45Z, exit 0, "ran alone for 38s"). It gated 3a3f9bb9d, the last code commit on origin/main (27995fb75, Markdown only). The gate of 8b4387fd4 (citation edits: 1879 tests / 7.022 s; 50 / 15.268 s; 50 s) has no log on disk.
- temper suites: focused 1879 tests, 120 binaries, wall 7.557 s (budget 15; sum of test times 58.15 s); fuzzy 50 tests, 26 binaries, wall 24.690 s (budget 60; sum 155.4 s, one test 24.69 s: domain-world fuzzy_walking, stays). Last 5 landings: focused 6.1-7.6 s, fuzzy 15.3-24.8 s.
- Per package, summed test seconds (focused n/s; fuzzy n/s), same log:
```
temper-forge-forgejo                  84  0.35   named crates
temper-engine-domain-forge-client     76  0.31
temper-engine-protocol-forge         117  1.28
temper-forge-setup                    98  0.71
temper-forge-setup-shell              24  1.71
temper-fake-forge-domain              96  0.51
temper-fake-forge-protocol            55  0.26
temper-throwaway-forgejo              13  0.08
  subtotal                           563  5.22
temper-fake-forge-tests                3  0.03   test crates
temper-forge-setup-reconcile-world    21  0.54   fuzzy  1  1.68
temper-engine-forge-client-world      29  0.30   fuzzy  2  0.46
temper-engine-protocol-forge-world    47  1.82   fuzzy  6 30.38
  subtotal                           100  2.68   fuzzy  9 32.52
-- not on the list, but they test the moved crates (decide) --
temper-codecs-world forgejo_* (7 bin) 55  1.27   fuzzy_forgejo 3 11.10
temper-engine-forge-world             54  0.52   fuzzy  1  0.09  (tests engine-domain-forge, -change, -issues)
forgejo-conformance (tools/)          25  0.21
temper bins forge 22/2.00, startup 24/2.67, configuration 2/0.10 (CLI; use setup-shell)
temper-engine-domain-world forge_routes 33 3.95; temper-fake-git 14 0.14
```
- Tests over 0.3 s. Focused: temper-forge-setup-shell::boundary 0.681 `std_process_and_file_locks_appear_only_in_the_process_module`; ::reconcile 0.312 `a_run_ends_at_its_deadline_when_nothing_answers`; temper::forge 0.334 `reconcile_makes_a_missing_forge_directory_private`; temper::startup 0.920 `a_managed_entrys_absent_admin_token_file_does_not_stop_temper_serve`, 0.813 `serve_prepares_once_on_the_owned_example`; domain-world::forge_routes 0.359, 0.345. None in the other named crates (lib tests, fakes, throwaway, forge-client-world focused).
- Fuzzy over 0.3 s: protocol-forge-world::fuzzy_machine 7.662, 7.459, 7.274, 6.622 (`a_response_cut_at_each_byte_ends_its_call_once`, `_seed_1`..`_seed_3`); fuzzy_hooks 0.739 `a_delivery_split_at_every_point_is_answered_alike`; fuzzy_protocol 0.627 `stories_through_cut_streams_answer_each_call_once`; reconcile-world::fuzzy_reconcile 1.680 `random_initial_forges_converge_and_a_second_run_writes_only_hook_patches`; forge-client-world::fuzzy_simulation 0.406 `seeded_faults_restarts_and_keyed_effects`; codecs-world::fuzzy_forgejo 3.421, 3.769, 3.913.
- skein: newest gate log on disk is `tmp/temper/client-api/logs/20261016T220313-595.log` (land skein, ended 2026-10-16T22:03:14Z, exit 0, "ran alone for 76s"). Focused 2464 tests, 196 binaries, 5.945 s (sum 46.96 s; longest 2.31 s conformance-ring); fuzzy 99 tests, 41 binaries, 39.090 s (sum 280.8 s; five tests over 20 s: echo-world 37.5 and 24.5, io-world 28.2, 26.9, 23.6); browser 38 tests, 23.729 s. Newer, from the Gate-Result trailer of 0e6f44e (2026-10-18T16:57Z, log gone): 2578 tests / 6.659 s; 104 / 30.919 s; 38 / 23.221 s; 103 s in all. Fuzzy wall over recorded gates: 30.9-48.0 s. Local skein main d120820 differs from origin/main b4cc6eb by forge design Markdown only.
- Projection (wall is about sum/8 + 0.1 s; both suites match that): the named crates and test crates add 663 focused tests, +7.9 s sum (+1.3 s with codecs forgejo_*), about +1 s wall: 6.0-6.7 s becomes 7-8 s of 15. They add 9 fuzzy tests, +32.5 s sum (+11.1 s with fuzzy_forgejo), about +4.1 (+5.5) s wall: 39.1 s becomes 43-45 s, and the worst recorded 48.0 s becomes 52-54 s of 60. The fuzzy side is the tight one; the four 7 s fuzzy_machine tests may want a `priority` override in skein's fuzzy profile (temper gives its longest test priority 20).

## (c) Suites by nextest filter (a moved test keeps its suite by its binary's file name)
- skein: default `not binary(/^fuzzy_/) and not binary(/^browser_/) and not binary(=live)`; fuzzy `binary(/^fuzzy_/)`; browser `(package(skein-browser-world) or package(skein-ui-web-tests)) and binary(/^browser_/)`; live `package(skein-llm-world) and binary(=live)`. Override: `kind(test)` priority 10 in default.
- temper: default `not binary(/^fuzzy_/) & not binary(/^(process|live)$/)`; fuzzy `binary(/^fuzzy_/) & not binary(/^(process|live)$/)`, plus priority 20 on `test(=fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure)`; process `binary(=process)`; live `binary(/^(process|live)$/) & not binary(=process)`. `process` and `live` binaries are only in the `temper` package (crates/temper/tests), so none of the moved crates has one.
- Moved fuzzy binaries (keep the `fuzzy_` prefix): protocol-forge-world fuzzy_hooks, fuzzy_machine, fuzzy_protocol; reconcile-world fuzzy_reconcile; forge-client-world fuzzy_simulation (and, if moved, engine-forge-world fuzzy_simulation, codecs-world fuzzy_forgejo). Every other test file is focused. No moved crate has `[[test]]` or `harness = false`.

## (d) testing-strategy.md 8, slow tests
1. Budgets are enforced (global timeout): focused 15 s, fuzzy 1 min; a breach is fixed by cheaper tests or by moving randomized ones to fuzzy; raising a budget is an explicit decision, not a fix.
2. A slow test is understood, never hidden: find the cause (scan, hidden I/O, sleep, repeated setup, copying) with a profile and fix it there; build settings never stand in, so test builds stay unoptimized, dependencies included (a tool a test runs may be optimized).
3. A per-crate `[profile.dev.package.<crate>]` is allowed for one hot leaf crate only, when a profile shows a cost no test can avoid, with the user's  sign-off cited in a comment beside it.

## (e) What skein's gate would trip on
- Nothing is enforced by a list or allowlist: no deny.toml, no cargo-deny, no test reads Cargo.toml or Cargo.lock (only the prebuild and a git-URL check read manifests), no boundary crate (jig has jig-boundary; skein has none), no serde check (no manifest in skein names serde).
- Clippy `-D warnings --workspace --all-targets`: root clippy.toml (step-crate subset) equals temper's but for three reason strings; `[workspace.lints]` are the same. Ordinary-Rust crates need their own clippy.toml, which must move with them: temper-forge-setup-shell, temper-throwaway-forgejo, forgejo-conformance have one; skein's `tests/clippy.toml` covers crates under tests/ (it bans HashMap, thread::spawn, env::var).
- Cargo.toml: `crates/*` is a glob, but `testing/*` and `tests/*` members and every `[workspace.dependencies]` entry are listed by hand; no check compares them, a missing one fails the build.
- External dependencies: skein's Cargo.toml allows only io-uring, libc, rustls (and the wasm tools), by comment and programming-model.md 2.1, 3. temper-throwaway-forgejo and forgejo-conformance use `serde_json =1.0.154`; no other named crate has an external dependency. Nothing in the gate rejects it, review does.
- Upstream-only edges (no gate check, but cargo cannot resolve them): moved crates name jig-protocol-config (forge-setup, setup-shell, reconcile-world), temper-service (setup-shell, protocol-forge-world), temper-codecs-world, temper-engine-forge-world, temper-engine-domain-forge{,-change,-issues}, jig-domain-people.
- Size: files over 800 lines (taste.sh and development.md 893 are review rules; check.sh runs taste.sh on skein-tls only): domain-forge-client keep.rs 1377, outbox.rs 1079, api.rs 850; fake-forge-domain src/tests.rs 2084; engine/forge tests/simulation.rs 1723; engine/forge/client tests/simulation.rs 858.
