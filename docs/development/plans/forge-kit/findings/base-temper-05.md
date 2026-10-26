# Base runs: temper 05, temper on skein's fakes and forge peer

2026-10-19. Code: temper dd12b8ed9 (the code of 27995fb75, with the designer's
documentation answers) in `temper-base`; skein 064c55f in `skein-base`. Only this
findings file and `temper/05-fakes.md` are deliverables. The scratch worktree
`scratch-temper-05` (a detached copy of the temper base) held the sketches and
the repin's base form; it is removed at the end, and its `Cargo.lock` was
restored after the repin (0 changes). Every command went
(nextest reports `NEXTEST_DOUBLE_SPAWN=0` from the environment). The queue
was shared with other drafters, so a time taken while it was loaded says so.

## Discovery and corrections

Graph calls (projects `srv-data-git-runner-src-temper`, `-skein`, `-jig`) were
not useful for this code: the one-process world's test crates and the fake crates
were found by `rg` and source reads, and every name below was checked with `rg`
in the base worktrees. Facts at the base that the session file rests on:

- **No peer at the skein base.** `rg 'skein_fake_peers::forge'` in temper prints
  nothing (exit 1); skein-base has no `forge` module under `skein-fake-peers`.
  The peer's API in the session file is skein 05.4's sketch A, uncompiled here.
- **The Done-when count 174** is `rg --count-matches` over the three crates and
  their users together (63 files); outside the three crates the users are 35
  Rust files with 122 lines and 13 manifest lines (users section).
- **`tests/fake-forge` is 457 Rust lines**, not 487 (lines section).
- **`tests/one-process/src/forge.rs` is the whole host (268 lines)**; the host
  has no webhook delivery at the base (`rg webhook tests/one-process/src`
  finds only the document's `"webhooks":null` and `"webhook_secret":null`).
- **The process tier does not run the one-process world.**
  `scripts/proof-one-process.sh` runs the real-binary tier (19 tests, profile
  `process`); the world's stories are joint tests of `temper-one-process-world`.
- **The 3000 literal** occurs three times in `fixtures.rs` (two strings in
  `document()`, one address in `forges()`), once in the host, once in
  `referee.rs` and three times in `stories.rs`.

## Base runs

The gate: focused 1879 passed and 1 skipped; fuzzy 50 passed, 1 slow. The 154 tests of the three fake crates are inside the 1879.

```text
$ scripts/check.sh
directory: /srv/data/git/runner/src/tmp/planning/temper-base
     Summary [   6.794s] 1879 tests run: 1879 passed, 1 skipped
        SLOW [> 20.000s] (─────) temper-engine-domain-world::fuzzy_walking fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure
     Summary [  23.936s] 50 tests run: 50 passed (1 slow), 0 skipped
exit 0
```

The process tier, run once (the script runs the real-binary tier). Only the passes of 5 s or more are listed; 19 passed, 8 slow.

```text
$ TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 scripts/proof-one-process.sh
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Starting 19 tests across 1 binary (6 binaries skipped via profile.process.default-filter)
        PASS [   5.256s] ( 5/19) temper::process a_push_is_delivered_signed_with_the_secret_reconcile_drew
        PASS [   6.042s] (12/19) temper::process temper_box_restarts_a_killed_forgejo_while_the_engine_serves
        PASS [   5.995s] (13/19) temper::process temper_open_adopts_a_repository_by_commands
     Summary [ 145.440s] 19 tests run: 19 passed (8 slow), 0 skipped
exit 0
```

The three crates 05.1 deletes: 154 tests in 0.118 s (the moved tests live in skein: 100 and 56).

```text
$ cargo nextest run -p temper-fake-forge-domain -p temper-fake-forge-protocol -p temper-fake-forge-tests
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Starting 154 tests across 4 binaries
     Summary [   0.118s] 154 tests run: 154 passed, 0 skipped
exit 0
```

Test counts of the three crates to delete: the oneline lines are counted per binary (awk, first field): 96 + 55 + 3 = 154; none is fuzzy.

```text
$ cargo nextest list -p temper-fake-forge-domain -p temper-fake-forge-protocol -p temper-fake-forge-tests --message-format oneline
directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit 0
temper-fake-forge-domain 96
temper-fake-forge-protocol 55
temper-fake-forge-tests::memory 3
```

Counts per binary: fake-git 6 + 8 = 14; one-process 4 (lib) + 5 (`one_process`) = 9; its fuzzy binary (4) is outside the default filter.

```text
$ cargo nextest list -p temper-fake-git -p temper-one-process-world --message-format oneline
directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit 0
temper-fake-git::check 6
temper-fake-git::transcripts 8
temper-one-process-world 4
temper-one-process-world::one_process 5
```

The fake-git crate (05.1 renames 7 lines in 2 files; 05.2 swaps its forge handle): 14 tests in 0.021 s.

```text
$ cargo nextest run -p temper-fake-git
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Starting 14 tests across 3 binaries
     Summary [   0.021s] 14 tests run: 14 passed, 0 skipped
exit 0
```

All 13 packages (the 10 users, the three fake crates), default profile: 606 = 452 + 154.

```text
$ cargo nextest run -p temper-fake-forge-domain -p temper-fake-forge-protocol -p temper-fake-forge-tests -p temper-fake-git -p temper-one-process-world -p temper-conformance-world -p temper-engine-domain-world -p temper-engine-forge-world -p temper-engine-forge-client-world -p temper-engine-protocol-forge-world -p temper-forge-setup-reconcile-world -p temper-system-world -p temper-worker-checkout-world
directory: /srv/data/git/runner/src/tmp/planning/temper-base
     Summary [   2.854s] 606 tests run: 606 passed, 0 skipped
```

Same 13 packages, fuzzy profile: 39 tests (the fake crates have none), 15.543 s.

```text
$ cargo nextest run -p temper-fake-forge-domain -p temper-fake-forge-protocol -p temper-fake-forge-tests -p temper-fake-git -p temper-one-process-world -p temper-conformance-world -p temper-engine-domain-world -p temper-engine-forge-world -p temper-engine-forge-client-world -p temper-engine-protocol-forge-world -p temper-forge-setup-reconcile-world -p temper-system-world -p temper-worker-checkout-world --profile fuzzy
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Starting 39 tests across 17 binaries (51 binaries skipped via profile.fuzzy.default-filter)
     Summary [  15.543s] 39 tests run: 39 passed, 0 skipped
```

The 10 users 05.1 changes (the Proof package list): 452 passed, 3.603 s.

```text
$ cargo nextest run -p temper-conformance-world -p temper-engine-domain-world -p temper-engine-forge-world -p temper-engine-forge-client-world -p temper-engine-protocol-forge-world -p temper-forge-setup-reconcile-world -p temper-system-world -p temper-worker-checkout-world -p temper-one-process-world -p temper-fake-git
directory: /srv/data/git/runner/src/tmp/planning/temper-base
     Summary [   3.603s] 452 tests run: 452 passed, 0 skipped
exit 0
```

The same, fuzzy: 39 passed, 27.144 s while the queue was loaded (15.543 s inside the 13-package run above); one test is slow, as at the gate.

```text
$ cargo nextest run -p temper-conformance-world -p temper-engine-domain-world -p temper-engine-forge-world -p temper-engine-forge-client-world -p temper-engine-protocol-forge-world -p temper-forge-setup-reconcile-world -p temper-system-world -p temper-worker-checkout-world -p temper-one-process-world -p temper-fake-git --profile fuzzy
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Starting 39 tests across 17 binaries (47 binaries skipped via profile.fuzzy.default-filter)
        SLOW [> 20.000s] (─────) temper-engine-domain-world::fuzzy_walking fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure
     Summary [  27.144s] 39 tests run: 39 passed (1 slow), 0 skipped
exit 0
```

Clippy over the 10 users, at the base: clean.

```text
$ cargo clippy -p temper-conformance-world -p temper-engine-domain-world -p temper-engine-forge-world -p temper-engine-forge-client-world -p temper-engine-protocol-forge-world -p temper-forge-setup-reconcile-world -p temper-system-world -p temper-worker-checkout-world -p temper-one-process-world -p temper-fake-git --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.93s
exit 0
```

The one-process world, default: 9 tests in 0.953 s. The Adopt story alone is 0.953 s, over the 0.5 s focused ceiling at the base.

```text
$ cargo nextest run -p temper-one-process-world
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Starting 9 tests across 2 binaries (1 binary skipped via profile.default.default-filter)
        PASS [   0.006s] (1/9) temper-one-process-world::one_process a_secret_in_a_root_fails_the_referee_naming_it
        PASS [   0.006s] (2/9) temper-one-process-world::one_process a_restart_showing_part_of_a_commit_fails_the_referee
        PASS [   0.017s] (3/9) temper-one-process-world referee::durable::tests::a_repair_snapshot_read_restores_the_durable_rows
        PASS [   0.020s] (4/9) temper-one-process-world referee::durable::tests::a_torn_log_write_recovers_the_durable_prefix
        PASS [   0.021s] (5/9) temper-one-process-world referee::durable::tests::a_repair_snapshot_write_yields_no_commit_frames
        PASS [   0.282s] (6/9) temper-one-process-world world::tests::an_adoption_saves_a_pending_window_before_the_engine_commit
        PASS [   0.290s] (7/9) temper-one-process-world::one_process a_person_signs_in_by_the_printed_link_and_a_chat_is_answered
        PASS [   0.301s] (8/9) temper-one-process-world::one_process a_signal_mid_run_drains_and_the_next_start_settles_the_claims
        PASS [   0.953s] (9/9) temper-one-process-world::one_process an_adopted_repository_gets_its_change_pushed_through_git
     Summary [   0.953s] 9 tests run: 9 passed, 0 skipped
```

The one-process world, fuzzy: 4 tests in 2.139 s.

```text
$ cargo nextest run -p temper-one-process-world --profile fuzzy
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Starting 4 tests across 1 binary (2 binaries skipped via profile.fuzzy.default-filter)
        PASS [   0.352s] (1/4) temper-one-process-world::fuzzy_one_process a_chat_reads_then_replies_and_waits
        PASS [   1.731s] (2/4) temper-one-process-world::fuzzy_one_process a_merge_in_progress_is_pushed_with_both_parents
        PASS [   2.115s] (3/4) temper-one-process-world::fuzzy_one_process drawn_kills_keep_the_conformance_rules_and_answer_adoptions_once
        PASS [   2.139s] (4/4) temper-one-process-world::fuzzy_one_process drawn_power_losses_keep_the_conformance_rules
     Summary [   2.139s] 4 tests run: 4 passed, 0 skipped
```

The Done-when check at the base: exit 0.

```text
$ cargo check --workspace --all-targets
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s
exit 0
```

All three exist at the base (the Done-when item prints "No such file" three times afterwards).

```text
$ ls -d testing/temper-fake-forge-domain testing/temper-fake-forge-protocol tests/fake-forge
directory: /srv/data/git/runner/src/tmp/planning/temper-base
testing/temper-fake-forge-domain
testing/temper-fake-forge-protocol
tests/fake-forge
exit 0
```

Per file (15 of the 63 files shown, counts are matches, so the root `Cargo.toml` shows 6 for 4 lines); the sum of the counts is 174 over 63 files. Outside the three crates the same command prints 139 lines (122 Rust, 13 manifest, 4 in the root).

```text
$ rg -n 'temper[-_]fake[-_]forge' Cargo.toml crates testing tests tools -g '*.rs' -g Cargo.toml --count-matches
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Cargo.toml:6
tests/worker/checkout/src/lib.rs:1
tests/worker/checkout/src/forge.rs:22
tests/worker/checkout/Cargo.toml:1
tests/system/src/forge.rs:1
tests/system/src/composed.rs:2
testing/temper-fake-git/src/transport.rs:2
testing/temper-fake-git/src/fixtures.rs:5
tests/system/Cargo.toml:1
tests/one-process/src/stories.rs:21
tests/one-process/src/referee/forge.rs:1
tests/one-process/src/forge.rs:2
tests/one-process/Cargo.toml:2
tests/forge-setup/reconcile/tests/reconcile.rs:1
tests/forge-setup/reconcile/tests/fuzzy_reconcile.rs:1
exit 0
```

The Done-when item prints nothing afterwards: 10 lines in tests plus `lib.rs:12` (`pub use transport::{Forge, Transport};`), 11 at the base.

```text
$ rg -n 'temper_fake_git::Forge|ForgeHost|pub use transport::\{Forge' tests testing crates -g '*.rs'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/one-process/src/forge.rs:11:type Shared = Rc<RefCell<temper_fake_git::Forge>>;
tests/one-process/src/forge.rs:27:pub(crate) struct ForgeHost {
tests/one-process/src/forge.rs:42:impl ForgeHost {
tests/one-process/src/forge.rs:209:impl Host for ForgeHost {
tests/one-process/src/world.rs:131:    Forge(Box<SignalHost<forge::ForgeHost>>),
tests/one-process/src/world.rs:605:    Proc::Forge(Box::new(SignalHost::new(forge::ForgeHost::new(), Some(inherited.signal))))
tests/one-process/src/world.rs:676:            world.spawn(|| Proc::Forge(Box::new(SignalHost::new(forge::ForgeHost::new(), None))));
tests/one-process/src/world.rs:683:            world.spawn(|| Proc::Forge(Box::new(SignalHost::new(forge::ForgeHost::new(), None))));
tests/one-process/src/stories.rs:163:    let forge = std::rc::Rc::new(std::cell::RefCell::new(temper_fake_git::Forge::new(domain, config)));
tests/one-process/src/referee/forge.rs:71:fn green(forge: &temper_fake_git::Forge, repository: &[u8], head: u64) {
12:pub use transport::{Forge, Transport};
```

The three literals of the fixed port in `fixtures.rs` (the Done-when item prints the definition of `FORGE` alone afterwards).

```text
$ rg -n '3000' tests/one-process/src/fixtures.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
121:        "forges":[{"number":1,"kind":"forgejo","api":{"socket":"127.0.0.1:3000"},"transport":"plaintext",
122:            "git":"http://127.0.0.1:3000","webhook_secret":null,"api_token":"secrets/forge.api-token","user":{"file":"secrets/forge-1.user"},"identity":1}],
242:            address: ([127, 0, 0, 1], 3000).into(),
exit 0
```

Nothing at the base (exit 1); the users arrive in 05.2.

```text
$ rg -n 'skein_fake_peers::forge' testing tests -g '*.rs'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit 1
```


## Users of the types and functions the session changes or moves

Counts by `rg` in the temper base, and in skein-base, jig, smith and hearth
where named. `tests/engine/protocol-forge` and `tests/forge-setup/reconcile`
are users 05.1 renames; skein 07, 08 and 09 later move them.

The Rust users outside the three crates: 35 files (per package in the next block), 122 lines; none under `crates/` or `tools/`.

```text
$ rg -l 'temper_fake_forge_(domain|protocol)' crates testing tests tools -g '*.rs' --glob '!testing/temper-fake-forge-*/**' --glob '!tests/fake-forge/**'  # then wc -l; and rg -n... | wc -l
directory: /srv/data/git/runner/src/tmp/planning/temper-base
files: 35
lines: 122
```

Per package, files and lines: 35 files, 122 lines (7 + 3 + 3 + 30 + 22 + 8 + 23 + 3 + 23).

```text
$ rg -c 'temper_fake_forge_(domain|protocol)' crates testing tests tools... | awk  # per package
directory: /srv/data/git/runner/src/tmp/planning/temper-base
testing/temper-fake-git 2 files 7 lines
tests/conformance 2 files 3 lines
tests/engine/domain 3 files 3 lines
tests/engine/forge 9 files 30 lines
tests/engine/protocol-forge 7 files 22 lines
tests/forge-setup/reconcile 5 files 8 lines
tests/one-process 3 files 23 lines
tests/system 2 files 3 lines
tests/worker/checkout 2 files 23 lines
```

The manifests: 5 root lines (3 members at 11, 40 and 47; path entries at 64 and 171) and 13 dependency lines in 10 manifests (the three crates' own manifests, lines 56-58 and 67-68 of the output, are left out here).

```text
$ rg -n 'temper-fake-forge-(domain|protocol)|fake-forge' Cargo.toml crates testing tests tools -g Cargo.toml
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Cargo.toml:11:    "testing/temper-fake-forge-protocol",
Cargo.toml:171:temper-fake-forge-domain = { path = "testing/temper-fake-forge-domain" }
Cargo.toml:40:    "testing/temper-fake-forge-domain",
Cargo.toml:47:    "tests/fake-forge",
Cargo.toml:64:temper-fake-forge-protocol = { path = "testing/temper-fake-forge-protocol" }
testing/temper-fake-git/Cargo.toml:15:temper-fake-forge-domain.workspace = true
testing/temper-fake-git/Cargo.toml:16:temper-fake-forge-protocol.workspace = true
tests/conformance/Cargo.toml:23:temper-fake-forge-domain.workspace = true
tests/engine/domain/Cargo.toml:35:temper-fake-forge-domain.workspace = true
tests/engine/forge/Cargo.toml:17:temper-fake-forge-domain.workspace = true
tests/engine/forge/client/Cargo.toml:13:temper-fake-forge-domain.workspace = true
tests/engine/protocol-forge/Cargo.toml:19:temper-fake-forge-protocol.workspace = true
tests/engine/protocol-forge/Cargo.toml:20:temper-fake-forge-domain.workspace = true
tests/forge-setup/reconcile/Cargo.toml:15:temper-fake-forge-protocol.workspace = true
tests/one-process/Cargo.toml:21:temper-fake-forge-domain.workspace = true
tests/one-process/Cargo.toml:22:temper-fake-forge-protocol.workspace = true
tests/system/Cargo.toml:39:temper-fake-forge-domain.workspace = true
tests/worker/checkout/Cargo.toml:14:temper-fake-forge-domain.workspace = true
```

Only the host imports the crates under local names (`use ... as domain`, `as protocol`); 05.2 deletes that file.

```text
$ rg -n 'temper_fake_forge_(domain|protocol) as ' crates testing tests tools -g '*.rs' --glob '!testing/temper-fake-forge-*/**' --glob '!tests/fake-forge/**'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/one-process/src/forge.rs:9:use temper_fake_forge_domain as domain;
tests/one-process/src/forge.rs:10:use temper_fake_forge_protocol as protocol;
exit 0
```

`fake_limits()` is defined in protocol-forge `fixtures.rs` and called at 5 sites: reconcile `world.rs` and `fixtures.rs`, protocol-forge `joint.rs` and `hooks.rs`, and the host (05.2 copies its 17 values into one-process `forge_limits()`). `forge::set` has 2 sites and `forge::transport` 1 (world.rs).

```text
$ rg -n 'fake_limits|ForgeHost|forge::(set|transport)\b' tests crates testing -g '*.rs'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/forge-setup/reconcile/src/world.rs:37:        let limits = temper_engine_protocol_forge_world::fixtures::fake_limits();
tests/forge-setup/reconcile/src/fixtures.rs:79:    let limits = temper_engine_protocol_forge_world::fixtures::fake_limits();
tests/engine/protocol-forge/src/joint.rs:86:        let fake_limits = fixtures::fake_limits();
tests/engine/protocol-forge/src/joint.rs:117:            service: Service::new(config, &fake_limits, seed).expect("service"),
tests/engine/protocol-forge/src/joint.rs:119:            protocol_env: Env { now, wall: Wall::from_nanos(now.as_nanos()), limits: fake_limits },
tests/engine/protocol-forge/src/hooks.rs:239:    let limits = fixtures::fake_limits();
tests/engine/protocol-forge/src/fixtures.rs:122:pub fn fake_limits() -> temper_fake_forge_protocol::Limits {
tests/one-process/src/world.rs:131:    Forge(Box<SignalHost<forge::ForgeHost>>),
tests/one-process/src/world.rs:503:    Proc::Git(Box::new(temper_fake_git::Git::new(spawn, inherited, forge::transport())))
tests/one-process/src/world.rs:605:    Proc::Forge(Box::new(SignalHost::new(forge::ForgeHost::new(), Some(inherited.signal))))
tests/one-process/src/world.rs:648:    forge::set(Some(transport.clone()));
tests/one-process/src/world.rs:676:            world.spawn(|| Proc::Forge(Box::new(SignalHost::new(forge::ForgeHost::new(), None))));
tests/one-process/src/world.rs:683:            world.spawn(|| Proc::Forge(Box::new(SignalHost::new(forge::ForgeHost::new(), None))));
tests/one-process/src/world.rs:706:    forge::set(None);
tests/one-process/src/forge.rs:27:pub(crate) struct ForgeHost {
tests/one-process/src/forge.rs:42:impl ForgeHost {
tests/one-process/src/forge.rs:57:        let protocol = temper_engine_protocol_forge_world::fixtures::fake_limits();
tests/one-process/src/forge.rs:209:impl Host for ForgeHost {
exit 0
```

The `Proc::Forge` and `fake_forge` sites: world.rs 604-605, 671, 676, 683 and 692; referee.rs 266 and 304 (the lines of the output after the first six are cut off here).

```text
$ rg -n 'Proc::Forge|fake_forge' tests/one-process/src -g '*.rs'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/one-process/src/world.rs:604:fn fake_forge(inherited: &Inherited) -> Proc {
tests/one-process/src/world.rs:605:    Proc::Forge(Box::new(SignalHost::new(forge::ForgeHost::new(), Some(inherited.signal))))
tests/one-process/src/world.rs:671:            world.spawn_restartable(vec![], fake_forge);
tests/one-process/src/world.rs:676:            world.spawn(|| Proc::Forge(Box::new(SignalHost::new(forge::ForgeHost::new(), None))));
tests/one-process/src/world.rs:683:            world.spawn(|| Proc::Forge(Box::new(SignalHost::new(forge::ForgeHost::new(), None))));
tests/one-process/src/world.rs:692:            Proc::Service(_) | Proc::Agent(_) | Proc::Git(_) | Proc::Shell(_) | Proc::Peer(_) | Proc::Forge(_) => None,
tests/one-process/src/referee.rs:266:            Proc::Service(_) | Proc::Agent(_) | Proc::Peer(_) | Proc::Git(_) | Proc::Shell(_) | Proc::Forge(_) => true,
tests/one-process/src/referee.rs:304:                    Proc::Forge(forge) => forge.shutdown(),
```

The fake-git crate's users are the one-process world and the root manifest only. In skein-base, jig, smith and hearth the same search (names `temper_fake_forge`, `temper_fake_git`) finds no file (first lines of the same script).

```text
$ rg -n 'temper_fake_git|temper-fake-git' crates testing tests tools Cargo.toml -g '*.rs' -g Cargo.toml --glob '!testing/temper-fake-git/**'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Cargo.toml:10:    "testing/temper-fake-git",
Cargo.toml:63:temper-fake-git = { path = "testing/temper-fake-git" }
tests/one-process/src/world.rs:128:    Git(Box<temper_fake_git::Git>),
tests/one-process/src/world.rs:503:    Proc::Git(Box::new(temper_fake_git::Git::new(spawn, inherited, forge::transport())))
tests/one-process/src/stories.rs:123:pub(crate) fn forge(story: Story) -> temper_fake_git::Transport {
tests/one-process/src/stories.rs:127:            ..temper_fake_git::fixtures::CONFIG.limits
tests/one-process/src/stories.rs:129:        ..temper_fake_git::fixtures::CONFIG
tests/one-process/src/stories.rs:163:    let forge = std::rc::Rc::new(std::cell::RefCell::new(temper_fake_git::Forge::new(domain, config)));
tests/one-process/src/stories.rs:164:    let mut transport = temper_fake_git::Transport::new(forge, &temper_fake_git::fixtures::table(b"acme
tests/one-process/src/stories.rs:165:    let identity = temper_fake_git::fixtures::identity();
tests/one-process/src/stories.rs:194:pub(crate) fn prepare_merge_branch(transport: &temper_fake_git::Transport) {
tests/one-process/src/referee/forge.rs:10:    transport: temper_fake_git::Transport,
tests/one-process/src/referee/forge.rs:18:    pub(crate) fn new(transport: temper_fake_git::Transport, story: Story) -> Self {
tests/one-process/src/referee/forge.rs:71:fn green(forge: &temper_fake_git::Forge, repository: &[u8], head: u64) {
tests/one-process/src/forge.rs:11:type Shared = Rc<RefCell<temper_fake_git::Forge>>;
tests/one-process/src/forge.rs:12:thread_local! { static SHARED: RefCell<Option<temper_fake_git::Transport>> = const { RefCell::new(None) };
tests/one-process/src/forge.rs:13:pub(crate) fn set(transport: Option<temper_fake_git::Transport>) {
tests/one-process/src/forge.rs:18:pub(crate) fn transport() -> temper_fake_git::Transport {
tests/one-process/src/forge.rs:59:            temper_fake_git::fixtures::table(b"acme/app", None),
tests/one-process/src/fixtures.rs:10:pub const GIT_TOKEN: &[u8] = temper_fake_git::fixtures::TOKEN;
tests/one-process/src/fixtures.rs:11:pub const API_TOKEN: &[u8] = temper_fake_git::fixtures::TOKEN;
tests/one-process/src/fixtures.rs:153:        entries.push((name.into(), Secret::new(temper_fake_git::fixtures::TOKEN.into()))).expect("forg
tests/one-process/Cargo.toml:19:temper-fake-git.workspace = true
```

The sites of fake-git's `Forge` handle (head; the rest are the one-process world's `stories.rs` 163, 176, 191, 195, 220, 235, `referee/forge.rs` 44 and 71, `referee.rs` 269, 285, 311, and the host's `forge.rs` 146 and 172). The fields they use are `domain`, `config`, `now`, `pending` (a push) and, new in 05.2, `hooks` and `next_reply()`; sketch A of skein 05.4 keeps the first four public.

```text
$ rg -n 'temper_fake_git::Forge|fake_git::\{[^}]*Forge|\.forge\.borrow|forge\.pending|\.shared' tests testing crates -g '*.rs'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
testing/temper-fake-git/tests/transcripts.rs:100:    assert!(scenario.transport.forge.borrow().domain.object(1).is_none());
testing/temper-fake-git/tests/transcripts.rs:313:    scenario.transport.forge.borrow_mut().config.limits.body_bytes = 8192;
testing/temper-fake-git/tests/transcripts.rs:335:    let forge = scenario.transport.forge.borrow();
testing/temper-fake-git/tests/transcripts.rs:402:        let forge = scenario.transport.forge.borrow();
testing/temper-fake-git/src/transport.rs:55:        let mut forge = self.forge.borrow_mut();
testing/temper-fake-git/src/transport.rs:64:        let mut forge = self.forge.borrow_mut();
testing/temper-fake-git/src/transport.rs:90:                        forge.pending.push(domain::Request::Reply { to: ReplyTo::new(replied), result });
testing/temper-fake-git/src/transport.rs:93:                request @ domain::Request::Hook { .. } => forge.pending.push(request),
testing/temper-fake-git/src/transport.rs:100:        let forge = self.forge.borrow();
testing/temper-fake-git/src/transport.rs:108:        self.forge.borrow().domain.is_ancestor(ancestor, commit)
testing/temper-fake-git/src/transport.rs:151:        self.forge.borrow().domain.object(commit).expect("stored commit").parent
testing/temper-fake-git/src/transport.rs:154:        self.forge.borrow().domain.object(commit).expect("stored commit").merge_parent
testing/temper-fake-git/src/transport.rs:168:        self.forge.borrow().domain.object(commit).expect("stored commit").message.to_vec()
testing/temper-fake-git/src/transport.rs:171:        let mut forge = self.forge.borrow_mut();
testing/temper-fake-git/src/program.rs:357:        self.transport.forge.borrow_mut().now = now;
testing/temper-fake-git/src/fixtures.rs:243:            let mut forge = scenario.transport.forge.borrow_mut();
```

Line counts: the three crates 8320 + 5309 + 457 = 14,086 Rust lines; the one-process files (the host 268, `world.rs` 745, `fixtures.rs` 303, `stories.rs` 435, `referee.rs` 547) and `transport.rs` 218. Root `Cargo.toml`: members at lines 11, 40 and 47, path entries at 64 and 171, the `skein-fake-peers` git entry at 65; `files()` has `List::with_capacity(9)` at fixtures.rs:145.

```text
$ for d in the three crates: find $d -name '*.rs' | xargs cat | wc -l;  wc -l tests/one-process/src/{forge,world,fixtures,stories,referee,machine}.rs tests/one-process/src/referee/forge.rs testing/temper-fake-git/src/transport.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
testing/temper-fake-forge-domain 8320
testing/temper-fake-forge-protocol 5309
tests/fake-forge 457
   268 tests/one-process/src/forge.rs
   745 tests/one-process/src/world.rs
   303 tests/one-process/src/fixtures.rs
   435 tests/one-process/src/stories.rs
   547 tests/one-process/src/referee.rs
   137 tests/one-process/src/machine.rs
    83 tests/one-process/src/referee/forge.rs
   218 testing/temper-fake-git/src/transport.rs
```


## The repin's base form

The repin is each increment's first commit. At the base, run in the scratch
(a copy of the temper base; `Cargo.lock` was restored afterwards, 0 changes):
`cargo update -p skein-lib` moves skein from `8a51e10b` to the head cargo
resolves, `54c5a8bb`, and the check then fails on what skein's steps since
temper's last repin broke. The README puts that repair in the tools plan's
first repin, which repins jig and temper before temper 01.1; the same two
failures are in findings/base-temper-02.md and base-temper-04.md.

Exit 0: 29 packages locked.

```text
$ cargo update -p skein-lib
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-05
    Updating git repository `https://github.com/forgetic/skein.git`
    Updating crates.io index
     Locking 29 packages to latest Rust 1.95 compatible versions
    Updating skein-channel v0.0.0 (https://github.com/forgetic/skein.git#8a51e10b) -> #54c5a8bb
    ... 27 more skein packages, #8a51e10b -> #54c5a8bb
    Updating skein-world v0.0.0 (https://github.com/forgetic/skein.git#8a51e10b) -> #54c5a8bb
note: pass `--verbose` to see 4 unchanged dependencies behind latest
exit 0
```

Exit 101: two E0004 on `Op::Lock` in `temper-worker-protocol-git/src/files.rs` (296 and 352), five E0004 in jig's `jig-protocol-store` (`Event::Locked` and `Event::Busy`; jig is pinned at `bbea9408`). Neither is in this session's code: the tools plan's first repin repairs them before temper 01.1, hence before this session. `skein-fake-peers` is already a workspace dependency of the root manifest (line 65).

```text
$ cargo check --workspace --all-targets
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-05
error[E0004]: non-exhaustive patterns: `skein_io::kernel::Op::Lock { .. }` not covered
   --> crates/temper-worker-protocol-git/src/files.rs:296:37
error[E0004]: non-exhaustive patterns: `skein_io::kernel::Op::Lock { .. }` not covered
   --> crates/temper-worker-protocol-git/src/files.rs:352:11
error: could not compile `temper-worker-protocol-git` (lib) due to 2 previous errors
error[E0004]: non-exhaustive patterns: `skein_io::file::Event::Locked { .. }` and `skein_io::file::Event::Busy { .. }` not covered
   --> <cargo-checkouts>/jig-aba39083bf77fefe/bbea940/crates/jig-protocol-store/src/directory.rs:79:11
error[E0004]: non-exhaustive patterns: `skein_io::file::Event::Locked { .. }` and `skein_io::file::Event::Busy { .. }` not covered
   --> <cargo-checkouts>/jig-aba39083bf77fefe/bbea940/crates/jig-protocol-store/src/directory.rs:102:11
error[E0004]: non-exhaustive patterns: `skein_io::file::Event::Locked { .. }` and `skein_io::file::Event::Busy { .. }` not covered
   --> <cargo-checkouts>/jig-aba39083bf77fefe/bbea940/crates/jig-protocol-store/src/transcripts/cut.rs:143:11
error[E0004]: non-exhaustive patterns: `skein_io::file::Event::Locked { .. }` and `skein_io::file::Event::Busy { .. }` not covered
   --> <cargo-checkouts>/jig-aba39083bf77fefe/bbea940/crates/jig-protocol-store/src/transcripts/read.rs:724:11
error[E0004]: non-exhaustive patterns: `skein_io::file::Event::Locked { .. }` and `skein_io::file::Event::Busy { .. }` not covered
   --> <cargo-checkouts>/jig-aba39083bf77fefe/bbea940/crates/jig-protocol-store/src/transcripts/write.rs:189:22
error: could not compile `jig-protocol-store` (lib) due to 5 previous errors
exit 101
```


## The sketch in the scratch (`scratch-temper-05`, a detached copy of the base)

The scratch tests the part of 05.2 that the base can run: the engine's
port-0 webhooks listener, learned by the host's service process
(`ServiceHost.hooks`, `hooks_address`), the document's webhooks section, the
webhook secret file and `hooked: true` for the Adopt story. It can't hold the
peer: skein 05.4 isn't in the base. The scratch diff, from `git diff --numstat
-- . ':!Cargo.lock'`, summed on the first column: 43 added lines in four files
(7 `fixtures.rs`, 3 `forge.rs` (a debug print), 1 `stories.rs`, 32 `world.rs`).

Failed: `Op::Bind.addr` and `Done::Bound` are `std::net::SocketAddr` (the kernel's `Addr`), not `skein_io::Addr::Ip` (io's own enum). The sketch uses `SocketAddr` and `.parse()`.

```text
$ cargo clippy -p temper-one-process-world --all-targets -- -D warnings   # first attempt
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-05
error[E0308]: mismatched types
   --> tests/one-process/src/world.rs:131:47
    |
131 |     if let skein_io::kernel::Op::Bind { addr: skein_io::Addr::Ip(asked), .. } = &complete.kind
    |                                               ^^^^^^^^^^^^^^^^^^^^^^^^^         -------------- this expression has type `&skein_io::kernel::Op`
    |                                               |
    |                                               expected `SocketAddr`, found `Addr`

error[E0308]: mismatched types
   --> tests/one-process/src/world.rs:133:49
    |
133 |         && let Ok(skein_io::kernel::Done::Bound(skein_io::Addr::Ip(bound))) = &complete.result
    |                                                 ^^^^^^^^^^^^^^^^^^^^^^^^^     ---------------- this expression has type `&std::result::Result<sk
    |                                                 |
(four E0308 in all: the same two sites in `world.rs` and in its unit test)
```

The sketch A compiles under `-D warnings`: exit 0.

```text
$ cargo clippy -p temper-one-process-world --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-05
    Checking temper-one-process-world v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-temper-05/tests/one-process)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.19s
exit 0
```

Not a plan finding: my script ran the repin's `cargo update` in the scratch while this run compiled, so it built against skein 54c5a8bb and failed in smith and jig (the E0004 above). The lock was restored and the run repeated.

```text
$ cargo nextest run -p temper-one-process-world --no-capture   # the first attempt after the repin ran in the same scratch
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-05

error[E0004]: non-exhaustive patterns: `skein_io::file::Event::Locked { .. }` and `skein_io::file::Event::Busy { .. }` not covered
error[E0004]: non-exhaustive patterns: `skein_io::kernel::Error::Busy` not covered
error: could not compile `smith-protocol-machine` (lib) due to 2 previous errors
error[E0004]: non-exhaustive patterns: `skein_io::file::Event::Locked { .. }` and `skein_io::file::Event::Busy { .. }` not covered
error[E0004]: non-exhaustive patterns: `skein_io::file::Event::Locked { .. }` and `skein_io::file::Event::Busy { .. }` not covered
error[E0004]: non-exhaustive patterns: `skein_io::file::Event::Locked { .. }` and `skein_io::file::Event::Busy { .. }` not covered
```

Failed at once: the secret list `files()` builds has room for 9 entries (`List::with_capacity(9)`, fixtures.rs:145); the webhook secret file is the 10th. Fixed by capacity 10; 05.2's Shape says so.

```text
$ cargo nextest run -p temper-one-process-world --profile fuzzy   # first run of the sketch
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-05
        FAIL [   0.010s] (1/4) temper-one-process-world::fuzzy_one_process a_chat_reads_then_replies_and_waits
  stdout ───

    running 1 test
    test a_chat_reads_then_replies_and_waits ... FAILED

    thread 'a_chat_reads_then_replies_and_waits' (3339890) panicked at tests/one-process/src/fixtures.rs:161:10:
    webhook secret file: ([115, 101, 99, 114, 101, 116, 115, 47, 102, 111, 114, 103, 101, 45, 49, 46, 119, 101, 98, 104, 111, 111, 107, 45, 115, 101, 99, 114, 101, 116], Secret { len: 13 })
     Summary [   0.022s] 4 tests run: 0 passed, 4 failed, 0 skipped
```

With the webhooks section configured, the engine binds `127.0.0.1:0`, `ServiceHost` learns the bound address (one per service start; the signal story restarts the service: two ports), and the Adopt story's domain fires 2 hooks (`HOOKREQ` at Time 0 and one second on) with `hooked: true`. All 10 pass (9 + the new unit test, 0.004 s); the engine started without a `socket_count` refusal. Serial run, queue loaded.

```text
$ cargo nextest run -p temper-one-process-world --no-capture
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-05
    Starting 10 tests across 2 binaries (1 binary skipped via profile.default.default-filter)
HOOKS Some(127.0.0.1:52767)
        PASS [   0.307s] ( 1/10) temper-one-process-world::one_process a_person_signs_in_by_the_printed_link_and_a_chat_is_answered
HOOKS Some(127.0.0.1:52767)
HOOKS Some(127.0.0.1:35455)
        PASS [   0.286s] ( 4/10) temper-one-process-world::one_process a_signal_mid_run_drains_and_the_next_start_settles_the_claims
HOOKREQ at Time(0)
HOOKREQ at Time(1000000000)
HOOKS Some(127.0.0.1:52767)
        PASS [   1.080s] ( 5/10) temper-one-process-world::one_process an_adopted_repository_gets_its_change_pushed_through_git
        PASS [   0.004s] ( 9/10) temper-one-process-world world::tests::a_port_zero_bind_names_the_hooks_address
HOOKS Some(127.0.0.1:35846)
        PASS [   0.232s] (10/10) temper-one-process-world world::tests::an_adoption_saves_a_pending_window_before_the_engine_commit
     Summary [   1.961s] 10 tests run: 10 passed, 0 skipped
exit 0
```

The new unit test of 05.2 (the Proof unit line), in the scratch: 1 passed, 0.004 s, under the 20 ms ceiling.

```text
$ cargo nextest run -p temper-one-process-world -E 'test(a_port_zero_bind_names_the_hooks_address)'
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-05
    Starting 1 test across 2 binaries (9 tests and 1 binary skipped, including 1 binary via profile.default.default-filter)
        PASS [   0.004s] (1/1) temper-one-process-world world::tests::a_port_zero_bind_names_the_hooks_address
     Summary [   0.004s] 1 test run: 1 passed, 9 skipped
exit 0
```

4 passed in 3.697 s under load; the A/B rounds below show the sketch costs nothing.

```text
$ cargo nextest run -p temper-one-process-world --profile fuzzy   # after the capacity fix; the queue was loaded
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-05
        PASS [   0.735s] (1/4) temper-one-process-world::fuzzy_one_process a_chat_reads_then_replies_and_waits
        PASS [   2.951s] (2/4) temper-one-process-world::fuzzy_one_process a_merge_in_progress_is_pushed_with_both_parents
        PASS [   3.527s] (3/4) temper-one-process-world::fuzzy_one_process drawn_kills_keep_the_conformance_rules_and_answer_adoptions_once
        PASS [   3.697s] (4/4) temper-one-process-world::fuzzy_one_process drawn_power_losses_keep_the_conformance_rules
     Summary [   3.697s] 4 tests run: 4 passed, 0 skipped
exit 0
```

Fuzzy time with and without the sketch, alternated twice: base 2.141 s and 2.101 s, scratch 2.177 s and 2.279 s; the webhooks section adds under 0.2 s to the 5 s fuzzy budget.

```text
$ cargo nextest run -p temper-one-process-world --profile fuzzy   # base, then scratch, two rounds
directory: temper-base and scratch-temper-05
$ cargo nextest run -p temper-one-process-world --profile fuzzy   [temper-base, round 1]
     Summary [   2.141s] 4 tests run: 4 passed, 0 skipped
$ cargo nextest run -p temper-one-process-world --profile fuzzy   [scratch-temper-05, round 1]
     Summary [   2.177s] 4 tests run: 4 passed, 0 skipped
$ cargo nextest run -p temper-one-process-world --profile fuzzy   [temper-base, round 2]
     Summary [   2.101s] 4 tests run: 4 passed, 0 skipped
$ cargo nextest run -p temper-one-process-world --profile fuzzy   [scratch-temper-05, round 2]
     Summary [   2.279s] 4 tests run: 4 passed, 0 skipped
```


## What the scratch could not measure

- The peer: skein 05.4's `Peer`, `Forge`, `Endpoint` and `Delivery` are the
  session file's assumptions from skein/05-fakes.md, not compiled here
  (sketch B). Assumed: `Delivery` and `Observation` derive `Clone`, `PartialEq`
  and `Debug`; `Forge.hooks` is a public field; `next_reply()` and the
  reservation form are 05.4's.
- The engine's reaction to a delivery (a hint refreshes a read, a story's
  counts) and the number of deliveries: the base host drops hooks. The 2 hooks
  are the domain's firing at the base; the peer decides how many are written.
- The one-process world on the peer's address: the sim fixes 127.0.0.1:3000
  (the engine's configuration is built when `World::admit` calls `make`, and
  `spawn_restartable` takes a fn pointer, so the world needs a thread-local
  handle). Decision 15 reads as handing the peer's `address()` to the engine;
  the session file does that check in the referee instead and names the question.
- The real-loop world and the tools plan's host with delivery: neither
  exists at the base (`rg` finds no `real-loop` directory; the host has no
  webhook code), so 05.2's Files for them are conditional on the tools plan.

## Scratch removal

`git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-temper-05`
from the temper repository (`git worktree list` first).
