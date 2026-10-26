# Base runs: temper 07, the moved worlds and the conformance deleted

2026-10-19. Code: temper dd12b8ed9 (the code of 27995fb75, in
`/srv/data/git/runner/src/tmp/planning/temper-base`); skein 064c55f
(`skein-base`, read only: the skein steps 06 to 09 do not exist there, so the
counts of what they copy are temper's). Only this file and
`temper/07-worlds.md` are deliverables. Every prescribed command ran once
; a block holds the command, its directory and the relevant
lines (at most 30, the passing lines of a long run left out, said so). Line
counts of files come from `wc -l` in the base worktree (discovery, not
prescribed commands). Both scratch worktrees are removed (section "The scratch worktrees").

## Discovery and corrections

Names were read with `rg` and `sed -n` in the base worktrees; the code graph
was not relied on (it indexes `main`, and the base is a detached tree).

- **What leaves, at the base.**
  - The client's world, `tests/engine/forge/client` (package
    `temper-engine-forge-client-world`): 10 files, 2,806 lines. One of them,
    `src/translate.rs` (567 lines), is not the world's: `tests/engine/forge/src/lib.rs`
    includes it with `#[path = "../client/src/translate.rs"]`, and its `KEY`
    holds `<!-- temper:key `. The other 9 files (`Cargo.toml`,
    `src/{lib,referee,world}.rs`, `tests/{fuzzy_simulation,job_answer,locate,memory,simulation}.rs`)
    are 2,239 lines.
  - Setup's reconcile world, `tests/forge-setup/reconcile` (package
    `temper-forge-setup-reconcile-world`): 7 files, 900 lines.
  - The protocol world, `tests/engine/protocol-forge` (package
    `temper-engine-protocol-forge-world`): `src/{hooks,joint,machine,peer,pipe}.rs`
    243 + 463 + 320 + 35 + 130 = 1,191 lines; `tests/{fake_protocol,fuzzy_hooks,fuzzy_machine,hooks,machine}.rs`
    113 + 19 + 43 + 215 + 262 = 652 lines. It keeps `src/{fixtures,lib,protocol}.rs`
    (252, 8, 535) and `tests/{fuzzy_protocol,protocol}.rs` (24, 448).
  - `tests/codecs` (package `temper-codecs-world`): `src/forgejo.rs` 725 lines;
    `tests/forgejo_{deliveries,files,items,reviews,setup,start,writes}.rs` 42 + 307 + 257 + 280
    + 309 + 135 + 302 = 1,632; `tests/fuzzy_forgejo.rs` 48; `tests/fixtures.rs` 122.
    Total 2,527 lines in 10 files.
  - `tools/forgejo-conformance`: 377 files, 15,619 lines: 22 `src` files (4,485),
    `transcripts` and `deliveries` (351 files, 10,817 lines), `facts.txt` 221,
    `README.md` 53, `clippy.toml` 13, `Cargo.toml` 30.
- **The conformance corpus has three readers, two in the protocol world.**
  `tests/engine/protocol-forge/src/machine.rs` reads the transcripts,
  `tests/hooks.rs` has `include_bytes!` of `deliveries/push/request`, and
  `tests/codecs/src/forgejo.rs` reads it in 3 places. The first two leave with
  07.1, the third with 07.2, and the tool goes last: this is why the session is
  two increments, not one (skein 06.3's Removes: "only after their remaining
  readers have moved").
- **`tests/codecs/tests/fixtures.rs` is read by one file**, `forgejo_setup.rs`
  (`mod fixtures;`, the only `mod fixtures` in codecs); it is not in skein 06.3's
  Files, which list `forgejo_*`, `fuzzy_forgejo` and `src/forgejo.rs`.
  Temper 07.2 deletes it with its reader; whether skein's copy of
  `forgejo_setup.rs` carries its own is skein 06.3's to say (reply).
- **Another dependency than the README's row: temper 05.2.** The one user of
  the protocol world outside the worlds is `tests/one-process/src/forge.rs`
  (`fixtures::fake_limits()`, one line, and one dependency line in
  `tests/one-process/Cargo.toml`). Temper 05.2 deletes that file (`ForgeHost`).
  If 05.2 has landed, 07.1 has no such user; the Files say what to do if not.
- **Nothing outside Cargo.toml names the tool.** `scripts/`, `.config/` and the
  nextest profiles name no deleted package (the profiles select by test name
  and binary: `fuzzy_*`, `process|live`, `binary(=process)`). Only the
  designer's drafts `docs/design/draft/qa.md` (lines 55, 287) and
  `draft/ci-on-forgejo-17.md` (line 157) still name `tools/forgejo-conformance`.
- **No shared fixture of `temper-engine-forge-world` was used only by the
  deleted parts.** Its items (`REPO`, `LIMITS`, `World`, `fake_config`,
  `translate`, `changes`) have users in `tests/system`, `tests/engine/domain`,
  `tests/conformance` and the kept protocol tests (the `temper_engine_forge_world::` blocks below).
- **The protocol world's fixtures.** The 10 kept tests reach `limits`,
  `protocol_config`, `writer_settings`, `protocol_shared`, `goal_view`,
  `token_absent` and `TOKEN` (`rg -o 'fixtures::\w+'`, block below), which call
  `shared`, `settings`, `WRITER` and `FORGE`. `REPO`, `response`, `branch`,
  `fake_limits`, `hook_limits`, `HOOK_SECRET`, `PING`, `hook_settings`,
  `hooked_protocol_config`, `push_hook` are read only by the moved parts or are
  copied by skein 08.1; `fixtures.rs` goes from 252 to 163 lines.
- **Times differ between runs.** The base runs below were made at light load
  (`base-runs.md` ran under a busy queue): the same tests show 0.155 s here
  against 0.404 s there for the protocol world, 0.222 s against 0.427 s for
  codecs. The session file quotes both; the aim is "no more than at the base".

## Commands at the base

The moved packages, run at the base in their own form (their code is what the
skein steps copy): counts and times are what the moved tests show in their new
homes.

```
$ cargo nextest list -p temper-engine-protocol-forge-world --profile fuzzy
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
temper-engine-protocol-forge-world::fuzzy_hooks a_delivery_split_at_every_point_is_answered_alike
temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once
temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_1
temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_2
temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_3
temper-engine-protocol-forge-world::fuzzy_protocol stories_through_cut_streams_answer_each_call_once
exit 0
```

```
$ cargo nextest run -p temper-engine-protocol-forge-world
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
    Starting 47 tests across 5 binaries (3 binaries skipped via profile.default.default-filter)
        (47 PASS lines omitted)
     Summary [   0.155s] 47 tests run: 47 passed, 0 skipped
exit 0
```

```
$ cargo nextest run -p temper-engine-protocol-forge-world --profile fuzzy
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
    Starting 6 tests across 3 binaries (5 binaries skipped via profile.fuzzy.default-filter)
        PASS [   0.460s] (1/6) temper-engine-protocol-forge-world::fuzzy_hooks a_delivery_split_at_every_point_is_answered_alike
        PASS [   0.526s] (2/6) temper-engine-protocol-forge-world::fuzzy_protocol stories_through_cut_streams_answer_each_call_once
        PASS [   4.026s] (3/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_3
        PASS [   4.048s] (4/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_2
        PASS [   5.159s] (5/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_1
        PASS [   5.191s] (6/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once
     Summary [   5.191s] 6 tests run: 6 passed, 0 skipped
exit 0
```

```
$ cargo nextest run -p temper-engine-protocol-forge-world --test protocol
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
    Starting 9 tests across 1 binary
        PASS [   0.029s] (1/9) temper-engine-protocol-forge-world::protocol honest_costs_equal_the_requests_the_fake_served
        PASS [   0.031s] (2/9) temper-engine-protocol-forge-world::protocol a_created_issue_with_its_answer_lost_is_found_after_restart
        (7 PASS lines omitted)
     Summary [   0.134s] 9 tests run: 9 passed, 0 skipped
exit 0
```

```
$ cargo nextest run -p temper-engine-protocol-forge-world --test fuzzy_protocol --profile fuzzy
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
    Starting 1 test across 1 binary
        (1 PASS lines omitted)
     Summary [   0.213s] 1 test run: 1 passed, 0 skipped
exit 0
```

Per binary of the 47 (from `cargo nextest list`, task 743): hooks 19, machine
14, fake_protocol 5, protocol 9. Leaving with 08.1/09.1: 38 default and 5 fuzzy
(`fuzzy_hooks` 1, `fuzzy_machine` 4). Staying: 9 and 1.

```
$ cargo nextest list -p temper-engine-forge-client-world
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
temper-engine-forge-client-world::job_answer a_job_with_no_failed_step_translates_to_its_whole_logs_end
temper-engine-forge-client-world::locate a_locate_answers_the_forges_id_for_a_repository_by_name
temper-engine-forge-client-world::locate a_locate_of_a_repository_the_forge_lacks_answers_missing
temper-engine-forge-client-world::memory full_live_caches_inbox_stamps_echoes_outbox_and_every_entry_point_fit
(... 27 more lines)
```

The 29 names, per binary: simulation 25, locate 2, job_answer 1, memory 1.

```
$ cargo nextest run -p temper-engine-forge-client-world
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
    Starting 29 tests across 5 binaries (1 binary skipped via profile.default.default-filter)
        (29 PASS lines omitted)
     Summary [   0.025s] 29 tests run: 29 passed, 0 skipped
exit 0
```

```
$ cargo nextest run -p temper-engine-forge-client-world --profile fuzzy
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
    Starting 2 tests across 1 binary (5 binaries skipped via profile.fuzzy.default-filter)
        (2 PASS lines omitted)
     Summary [   0.119s] 2 tests run: 2 passed, 0 skipped
exit 0
```

```
$ cargo nextest list -p temper-forge-setup-reconcile-world
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
temper-forge-setup-reconcile-world::reconcile a_deleted_bot_is_made_again
temper-forge-setup-reconcile-world::reconcile a_deleted_hook_is_made_again
temper-forge-setup-reconcile-world::reconcile a_listing_longer_than_a_page_is_read_to_its_end
temper-forge-setup-reconcile-world::reconcile a_managed_forge_missing_an_org_is_refused_naming_it
(... 19 more lines)
```

Per binary: reconcile 21.

```
$ cargo nextest run -p temper-forge-setup-reconcile-world
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.27s
    Starting 21 tests across 2 binaries (1 binary skipped via profile.default.default-filter)
        (21 PASS lines omitted)
     Summary [   0.075s] 21 tests run: 21 passed, 0 skipped
exit 0
```

```
$ cargo nextest run -p temper-forge-setup-reconcile-world --profile fuzzy
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.33s
    Starting 1 test across 1 binary (2 binaries skipped via profile.fuzzy.default-filter)
        (1 PASS lines omitted)
     Summary [   1.272s] 1 test run: 1 passed, 0 skipped
exit 0
```

```
$ cargo nextest list -p temper-codecs-world
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.35s
temper-codecs-world::client_wire a_core_refusal_carries_jigs_refusal_bytes_unchanged
temper-codecs-world::drift client_wire_schema_matches_generated_code_and_goldens
temper-codecs-world::drift store_schema_matches_generated_code_and_goldens
temper-codecs-world::drift worker_wire_schema_matches_generated_code_and_goldens
(... 58 more lines)
```

Per binary of codecs: client_wire 1, drift 3, worker_wire 1 (staying: 5), and
forgejo_deliveries 4, forgejo_files 6, forgejo_items 5, forgejo_reviews 6,
forgejo_setup 22, forgejo_start 4, forgejo_writes 8 (leaving: 55).

```
$ cargo nextest run -p temper-codecs-world --test forgejo_*
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
    Starting 55 tests across 7 binaries
        (55 PASS lines omitted)
     Summary [   0.154s] 55 tests run: 55 passed, 0 skipped
exit 0
```

```
$ cargo nextest run -p temper-codecs-world --test fuzzy_forgejo --profile fuzzy
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
    Starting 3 tests across 1 binary
        (3 PASS lines omitted)
     Summary [   2.145s] 3 tests run: 3 passed, 0 skipped
exit 0
```

```
$ cargo nextest run -p temper-codecs-world
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.30s
    Starting 60 tests across 12 binaries (4 binaries skipped via profile.default.default-filter)
        (60 PASS lines omitted)
     Summary [   0.222s] 60 tests run: 60 passed, 0 skipped
exit 0
```

```
$ cargo nextest run -p temper-codecs-world --profile fuzzy
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
    Starting 6 tests across 4 binaries (12 binaries skipped via profile.fuzzy.default-filter)
        (6 PASS lines omitted)
     Summary [   2.306s] 6 tests run: 6 passed, 0 skipped
exit 0
```

```
$ cargo nextest run -p forgejo-conformance
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
    Starting 25 tests across 2 binaries
        (25 PASS lines omitted)
     Summary [   0.122s] 25 tests run: 25 passed, 0 skipped
exit 0
```

(Temper 06.1 adds `tools/forgejo-conformance/tests/refusal.rs` with one test,
so the tool has 26 tests at 07.2's start.)

## Users of what the session changes

Each search ran once in `temper-base`; the counts under a block
are made from it.

```
$ rg -n -e temper_engine_protocol_forge_world|temper-engine-protocol-forge-world Cargo.toml crates testing tests tools scripts -g *.rs -g *.toml
dir: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/protocol-forge/tests/protocol.rs:6:use temper_engine_protocol_forge_world::{
tests/engine/protocol-forge/tests/machine.rs:4:use temper_engine_protocol_forge_world::{
tests/engine/protocol-forge/tests/hooks.rs:4:use temper_engine_protocol_forge_world::{
tests/engine/protocol-forge/tests/hooks.rs:192:    let mut world = temper_engine_protocol_forge_world::joint::Joint::hooked(7);
tests/engine/protocol-forge/tests/fuzzy_protocol.rs:2:use temper_engine_protocol_forge_world::{
tests/engine/protocol-forge/tests/fuzzy_machine.rs:3:use temper_engine_protocol_forge_world::{
tests/engine/protocol-forge/tests/fuzzy_hooks.rs:1:use temper_engine_protocol_forge_world::{
tests/engine/protocol-forge/tests/fake_protocol.rs:3:use temper_engine_protocol_forge_world::joint::{Joint, REPO};
tests/engine/protocol-forge/Cargo.toml:2:name = "temper-engine-protocol-forge-world"
tests/one-process/src/forge.rs:57:        let protocol = temper_engine_protocol_forge_world::fixtures::fake_limits();
tests/one-process/Cargo.toml:57:temper-engine-protocol-forge-world = { path = "../engine/protocol-forge" }
tests/forge-setup/reconcile/src/world.rs:37:        let limits = temper_engine_protocol_forge_world::fixtures::fake_limits();
tests/forge-setup/reconcile/src/fixtures.rs:79:    let limits = temper_engine_protocol_forge_world::fixtures::fake_limits();
tests/forge-setup/reconcile/src/exchange.rs:7:use temper_engine_protocol_forge_world::pipe::{Cut, Pipe};
tests/forge-setup/reconcile/Cargo.toml:16:temper-engine-protocol-forge-world = { path = "../../engine/protocol-forge" }
exit 0
```

8 hits are the world's own tests (`protocol, machine, hooks` 2, `fuzzy_protocol,
fuzzy_machine, fuzzy_hooks, fake_protocol`), 1 its manifest, 4 reconcile's world
(deleted with it) and 2 `tests/one-process` (`src/forge.rs`, `Cargo.toml`; temper
05.2 deletes the first). Nothing in `crates/`, `testing/`, `tools/` or `scripts/`.

```
$ rg -n -e temper_engine_forge_client_world|temper-engine-forge-client-world Cargo.toml crates testing tests tools scripts -g *.rs -g *.toml
dir: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/forge/client/tests/simulation.rs:2:use temper_engine_forge_client_world::{REPO, Settings, World};
tests/engine/forge/client/tests/simulation.rs:39:fn run(settings: Settings) -> temper_engine_forge_client_world::Stats {
tests/engine/forge/client/tests/simulation.rs:91:                commit: temper_engine_forge_client_world::translate::commit(head),
tests/engine/forge/client/tests/simulation.rs:135:                commit: temper_engine_forge_client_world::translate::commit(head),
tests/engine/forge/client/tests/simulation.rs:321:            write: api::Write::Merge { number: pull, head: temper_engine_forge_client_world::translate::commit(head) },
(... 18 more lines)
```

21 hits, every one inside the client's world (`simulation.rs` 15, `fuzzy_simulation.rs`
2, `job_answer, locate, memory` 1 each, `Cargo.toml` 1). No other package names it.

```
$ rg -n -e temper_forge_setup_reconcile_world|temper-forge-setup-reconcile-world Cargo.toml crates testing tests tools scripts -g *.rs -g *.toml
dir: /srv/data/git/runner/src/tmp/planning/temper-base
tests/forge-setup/reconcile/tests/reconcile.rs:4:use temper_forge_setup_reconcile_world::{Ended, World, fixtures};
tests/forge-setup/reconcile/tests/fuzzy_reconcile.rs:2:use temper_forge_setup_reconcile_world::{Ended, fixtures};
tests/forge-setup/reconcile/Cargo.toml:2:name = "temper-forge-setup-reconcile-world"
exit 0
```

3 hits, all in reconcile's own tests and manifest.

```
$ rg -n fake_limits tests testing crates tools -g *.rs
dir: /srv/data/git/runner/src/tmp/planning/temper-base
tests/one-process/src/forge.rs:57:        let protocol = temper_engine_protocol_forge_world::fixtures::fake_limits();
tests/forge-setup/reconcile/src/world.rs:37:        let limits = temper_engine_protocol_forge_world::fixtures::fake_limits();
tests/forge-setup/reconcile/src/fixtures.rs:79:    let limits = temper_engine_protocol_forge_world::fixtures::fake_limits();
tests/engine/protocol-forge/src/joint.rs:86:        let fake_limits = fixtures::fake_limits();
tests/engine/protocol-forge/src/joint.rs:117:            service: Service::new(config, &fake_limits, seed).expect("service"),
tests/engine/protocol-forge/src/joint.rs:119:            protocol_env: Env { now, wall: Wall::from_nanos(now.as_nanos()), limits: fake_limits },
tests/engine/protocol-forge/src/hooks.rs:239:    let limits = fixtures::fake_limits();
tests/engine/protocol-forge/src/fixtures.rs:122:pub fn fake_limits() -> temper_fake_forge_protocol::Limits {
exit 0
```

`fake_limits` has 8 hits: its definition (`fixtures.rs`), 3 in the moved `joint.rs`
and `hooks.rs`, 2 in reconcile's world (deleted) and 1 in `tests/one-process/src/forge.rs`
(temper 05.2). After 08.1 the definition is skein's `fixtures::fake_limits`; the
kept tests of the protocol world do not call it.

```
$ rg -l temper_codecs_world tests testing crates tools -g *.rs
dir: /srv/data/git/runner/src/tmp/planning/temper-base
tests/forge-setup/reconcile/src/exchange.rs
tests/codecs/tests/fuzzy_worker_wire.rs
tests/codecs/tests/fuzzy_store.rs
tests/codecs/tests/fuzzy_forgejo.rs
tests/codecs/tests/fuzzy_client_wire.rs
tests/codecs/tests/forgejo_writes.rs
tests/codecs/tests/forgejo_start.rs
tests/codecs/tests/forgejo_setup.rs
tests/codecs/tests/forgejo_reviews.rs
tests/codecs/tests/forgejo_items.rs
tests/codecs/tests/forgejo_files.rs
(... 3 more lines)
```

12 files name `temper_codecs_world`: 11 inside codecs' own tests (the 7
`forgejo_*`, `fuzzy_forgejo` and three that stay) and one outside:
`tests/forge-setup/reconcile/src/exchange.rs` (`forgejo::decode_answer`). It
leaves in 07.1, before 07.2 deletes `forgejo.rs`.

```
$ rg -n mod fixtures|fixtures:: tests/codecs -g *.rs
dir: /srv/data/git/runner/src/tmp/planning/temper-base
tests/codecs/tests/forgejo_setup.rs:1:mod fixtures;
tests/codecs/tests/forgejo_setup.rs:35:    let bot = || Answer::User(fixtures::user(fixtures::BOT, b"bot", false, b"temper:bot"));
tests/codecs/tests/forgejo_setup.rs:36:    let person = || Answer::User(fixtures::user(fixtures::PERSON, b"person", false, b"Person"));
(... 18 more lines)
```

24 lines: `mod fixtures;` once (`forgejo_setup.rs` line 1), the rest its calls. No
other codecs file reads `tests/codecs/tests/fixtures.rs`.

```
$ rg -n forgejo-conformance. -g !tools/forgejo-conformance/** -g !docs/development/** -g !Cargo.lock -g !target/**
dir: /srv/data/git/runner/src/tmp/planning/temper-base
./Cargo.toml:25:    "tools/forgejo-conformance",
./tests/engine/protocol-forge/tests/hooks.rs:154:    let request = include_bytes!("../../../../tools/forgejo-conformance/deliveries/push/request");
./tests/engine/protocol-forge/src/machine.rs:315:            .join("../../../tools/forgejo-conformance/transcripts")
./tests/codecs/src/forgejo.rs:26:    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/forgejo-conformance/transcripts").join(case);
./tests/codecs/src/forgejo.rs:404:    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/forgejo-conformance/transcripts");
./tests/codecs/src/forgejo.rs:503:    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/forgejo-conformance/deliveries").join(case);
./docs/design/draft/qa.md:55:   `tools/forgejo-conformance` checks the facts temper relies on against
./docs/design/draft/qa.md:287:1. **With no agent:** `tools/forgejo-conformance` on a schedule,
./docs/design/draft/ci-on-forgejo-17.md:157:By `tools/forgejo-conformance`, as facts `../protocol/forge.md`,
exit 0
```

9 hits outside the tool and the landed plans: the workspace member, 2 readers in
the protocol world (`src/machine.rs`, `tests/hooks.rs`), 3 in `tests/codecs/src/forgejo.rs`
and 3 in the designer's drafts (`docs/design/draft/qa.md` 55 and 287,
`ci-on-forgejo-17.md` 157; the session does not touch them).

```
$ rg -o -N --no-filename fixtures::[A-Za-z_]+ tests/engine/protocol-forge/tests/protocol.rs tests/engine/protocol-forge/tests/fuzzy_protocol.rs tests/engine/protocol-forge/src/protocol.rs
dir: /srv/data/git/runner/src/tmp/planning/temper-base
fixtures::protocol_config
fixtures::limits
fixtures::writer_settings
fixtures::protocol_shared
fixtures::token_absent
fixtures::writer_settings
fixtures::limits
fixtures::protocol_shared
fixtures::token_absent
fixtures::token_absent
fixtures::token_absent
fixtures::goal_view
fixtures::token_absent
fixtures::TOKEN
fixtures::token_absent
(... 4 more lines)
```

By name: `token_absent` 6, `limits` 2, `protocol_shared` 2, `writer_settings` 2,
`goal_view` 2, `TOKEN` 2, `protocol_config` 1.

```
$ rg -o -N --no-filename temper_engine_forge_world::(\{[^}]*\}|[A-Za-z_]+) tests testing crates tools -g *.rs
dir: /srv/data/git/runner/src/tmp/planning/temper-base
temper_engine_forge_world::fake_config
temper_engine_forge_world::{self as forge_world, translate}
temper_engine_forge_world::REPO
temper_engine_forge_world::translate
temper_engine_forge_world::REPO
(... 54 more lines)
```

57 uses of `temper_engine_forge_world::` items, in 24 files, none in the client's or
reconcile's worlds (the client's world had its own copy): `translate` 24 plain
and 5 in brace imports, `REPO` 14 plus 5 in braces, `fake_config` 5, `LIMITS` 4 plus
1, `changes` 2, `World` 5 in braces. By place: `tests/conformance` 12, `tests/system`
7, `tests/engine/domain` 6, the package's own tests 24, the fake-forge-protocol
fixtures 1, and the protocol world 7 (3 in the moved `joint.rs` and
`fake_protocol.rs`, 4 in the files that stay). The session changes no path of
these: `translate.rs` moves into the package's `src/` unchanged, so its module
path is the same, and no item is used only by the deleted parts.

```
$ bash -c 'for r in /srv/data/git/runner/src/tmp/planning/skein-base /srv/data/git/runner/src/jig /srv/data/git/runner/src/smith /srv/data/git/runner/src/hearth; do printf "%s " "$r"; (cd "$r" && rg -l -e "temper_engine_protocol_forge_world|temper-engine-protocol-forge-world|temper_engine_forge_client_world|temper_forge_setup_reconcile_world|temper_codecs_world|forgejo-conformance|forgejo_conformance|temper_engine_forge_world" -g "*.rs" -g "*.toml" -g "*.sh". | wc -l); done'
dir: /srv/data/git/runner/src/tmp/planning/temper-base
/srv/data/git/runner/src/tmp/planning/skein-base 0
/srv/data/git/runner/src/jig 0
/srv/data/git/runner/src/smith 0
/srv/data/git/runner/src/hearth 0
exit 0
```

Skein (at its base), jig, smith and hearth name none of the moved or deleted
packages, the tool, or the engine-forge world: the deletions are temper's alone.
(A first form of this search, with relative paths, failed on `cd` for smith
and hearth; the block is the rerun with absolute paths.)

## Sketches and the size

The only changed signatures are the **visibility** of `Joint`'s seams, which
skein 08.1 makes (not temper's code), and temper's three imports of them. A
scratch worktree of the base, `scratch-temper-07` (`git worktree add --detach`
of dd12b8ed9), held **stage A**: a stand-in crate `tests/engine/standin-machine`
(package `skein-forge-machine-world`, a workspace member and path dependency
of the protocol world), made by `git mv` of the world's `hooks, joint,
machine, peer, pipe` and a copy of its `fixtures.rs`, so that the code that
stays compiles against a crate of the name and paths skein 08.1 gives it.
The stand-in's `Joint` has the fields `component, env, above, below,
fake_env, answers` and the methods `busy, time, replace`, `checks` public
(08.1's list), `busy` and `checks` with `#[must_use]` as 08.1 says; its
first clippy run failed on `must_use_candidate` for those two (`busy`,
`checks`) until they had the attribute.

Stage A edits in the worktree (`git diff --cached --numstat -M`, the stand-in
and `Cargo.lock` apart): `Cargo.toml` 1/2 (two members out, the stand-in in),
`tests/engine/forge/src/lib.rs` 0/1, `translate.rs` renamed (567 lines,
0/0), `protocol-forge`: `Cargo.toml` 1/0, `src/fixtures.rs` 1/90 (252 -> 163),
`src/lib.rs` 0/5, `src/protocol.rs` 3/3, `tests/fuzzy_protocol.rs` 2/4,
`tests/protocol.rs` 2/4; `tests/one-process/Cargo.toml` 1/1 and
`src/forge.rs` 1/1 (`fake_limits()` from the stand-in; temper 05.2 deletes
these two files, so 07.1 has them only if it has not landed); deletions: the
client's world (9 files, 2,239 lines), reconcile's (7 files, 900) and the
moved parts of the protocol world (10 files, 1,843). Total of the users'
and fixtures' edits: 12 added, 106 + 5 deleted; plus the deleted files.

The edits compiled and ran in the scratch worktree:

```
$ cargo clippy --workspace --all-targets -- -D warnings
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07
error: this method could have a `#[must_use]` attribute
error: this function could have a `#[must_use]` attribute
error: could not compile `skein-forge-machine-world` (lib) due to 2 previous errors
exit 101
```

The two attributes added (`#[must_use]` on `busy` and `checks`), the rerun:

```
$ cargo clippy --workspace --all-targets -- -D warnings
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 27.44s
exit 0
```

```
$ cargo nextest run --workspace
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07
        (198 PASS lines omitted)
     Summary [   6.198s] 1791 tests run: 1791 passed, 1 skipped
exit 0
```

```
$ cargo nextest run --workspace --profile fuzzy
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07
    Finished `test` profile [unoptimized + debuginfo] target(s) in 37.01s
    Starting 42 tests across 22 binaries (113 binaries skipped via profile.fuzzy.default-filter)
        SLOW [> 20.000s] (─────) temper-engine-domain-world::fuzzy_walking fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure
        (42 PASS lines omitted)
     Summary [  21.475s] 42 tests run: 42 passed (1 slow), 0 skipped
exit 0
```

```
$ cargo nextest run -p temper-engine-forge-world
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
    Starting 54 tests across 6 binaries (1 binary skipped via profile.default.default-filter)
        (54 PASS lines omitted)
(... 3 more lines)
```

```
$ cargo nextest run -p temper-engine-forge-world --profile fuzzy
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
    Starting 1 test across 1 binary (6 binaries skipped via profile.fuzzy.default-filter)
        (1 PASS lines omitted)
(... 3 more lines)
```

```
$ cargo nextest run -p temper-engine-protocol-forge-world
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.52s
    Starting 9 tests across 2 binaries (1 binary skipped via profile.default.default-filter)
        PASS [   0.017s] (1/9) temper-engine-protocol-forge-world::protocol honest_costs_equal_the_requests_the_fake_served
        PASS [   0.028s] (2/9) temper-engine-protocol-forge-world::protocol a_created_issue_with_its_answer_lost_is_found_after_restart
        PASS [   0.028s] (3/9) temper-engine-protocol-forge-world::protocol a_merge_with_its_answer_lost_is_found_after_restart_without_merging_twice
        PASS [   0.021s] (4/9) temper-engine-protocol-forge-world::protocol the_api_token_reaches_no_record_request_or_fact
        PASS [   0.039s] (5/9) temper-engine-protocol-forge-world::protocol a_busy_pass_lists_at_most_changed_items_over_the_page_plus_one
        PASS [   0.043s] (6/9) temper-engine-protocol-forge-world::protocol an_external_move_of_a_held_branch_is_reported_as_drift
        PASS [   0.054s] (7/9) temper-engine-protocol-forge-world::protocol idle_forge_call_cost_does_not_follow_old_issue_history
        PASS [   0.086s] (8/9) temper-engine-protocol-forge-world::protocol a_change_produced_opened_checked_queued_and_landed
        PASS [   0.096s] (9/9) temper-engine-protocol-forge-world::protocol a_failed_ci_head_is_repaired_then_checked_again
     Summary [   0.097s] 9 tests run: 9 passed, 0 skipped
exit 0
```

```
$ cargo nextest run -p temper-engine-protocol-forge-world --profile fuzzy
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
    Starting 1 test across 1 binary (2 binaries skipped via profile.fuzzy.default-filter)
        (1 PASS lines omitted)
     Summary [   0.297s] 1 test run: 1 passed, 0 skipped
exit 0
```

The workspace default count is the base's 1,879 less 88 (38 + 29 + 21), the
fuzzy count 50 less 8 (5 + 2 + 1). The fuzzy workspace time is one test's:
`temper-engine-domain-world::fuzzy_walking`
`fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure`, which this
session does not touch, shows SLOW (more than 20 s) in every fuzzy run, base
and scratch: 23.1 s in the base gate, 27.1 s in the base workspace run (the
queue was busy), 21.5 s in stage A, 32.0 s in stage B (the time moves with the
machine's load). The suites' "no more than at the base" is therefore judged by
package (the protocol world's and codecs' blocks), not by the workspace
total. The default run's one slow test, `one_process`
`an_adopted_repository_gets_its_change_pushed_through_git` (over 3 s), shows in
the base workspace run and in stage B; it is not in the packages this session
touches.

**Stage B**, `scratch-temper-07b`, is stage A plus 07.2 staged: `git rm -r
tools/forgejo-conformance` (376 files by `git diff --cached`; 15,663 lines by
numstat, 15,619 by `wc -l`: 77 of its files end without a newline),
`tests/codecs/src/forgejo.rs`, `tests/codecs/tests/{forgejo_*,fuzzy_forgejo,fixtures}.rs`
(2,527 lines in 10 files), `pub mod forgejo;` and its doc lines in
`tests/codecs/src/lib.rs` (0/3), seven lines of `tests/codecs/Cargo.toml`
(0/7: `[dependencies]` the dialect, service, component, `skein-json` and
`serde_json` crates, `[dev-dependencies]` the service and the component), and
the conformance's member line:

```
$ git status --short | awk '{print $1}' | sort | uniq -c      (dir: scratch-temper-07b)
      2 A
    407 D
     12 M
      1 MM
      7 R
$ git diff --cached --numstat -M | awk '$1!="-"{a+=$1;d+=$2} END{print a, d}'
335 22121      (all, with the stand-in's 1,452 lines counted as added and the lock's 25/25)
```

```
$ cargo clippy --workspace --all-targets -- -D warnings
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07b
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 53.14s
exit 0
```

```
$ cargo nextest run --workspace
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07b
        SLOW [>  3.000s] (─────────) temper-one-process-world::one_process an_adopted_repository_gets_its_change_pushed_through_git
        (197 PASS lines omitted)
     Summary [   6.837s] 1711 tests run: 1711 passed (1 slow), 1 skipped
exit 0
```

```
$ cargo nextest run --workspace --profile fuzzy
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07b
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.35s
    Starting 39 tests across 21 binaries (103 binaries skipped via profile.fuzzy.default-filter)
        SLOW [> 20.000s] (─────) temper-engine-domain-world::fuzzy_walking fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure
        (39 PASS lines omitted)
     Summary [  31.956s] 39 tests run: 39 passed (1 slow), 0 skipped
exit 0
```

```
$ cargo nextest run -p temper-codecs-world
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07b
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
    Starting 5 tests across 4 binaries (3 binaries skipped via profile.default.default-filter)
        (5 PASS lines omitted)
     Summary [   0.084s] 5 tests run: 5 passed, 0 skipped
exit 0
```

```
$ cargo nextest run -p temper-codecs-world --profile fuzzy
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07b
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
    Starting 3 tests across 3 binaries (4 binaries skipped via profile.fuzzy.default-filter)
        (3 PASS lines omitted)
     Summary [   0.100s] 3 tests run: 3 passed, 0 skipped
exit 0
```

The gate's other two steps, on stage B (the tree after both increments), after
`touch` of the three changed crates' `lib.rs` so that clippy rebuilt them:

```
$ cargo fmt --all -- --check
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07b
exit 0
```

```
$ cargo clippy --workspace --all-targets -- -D warnings
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07b
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.40s
exit 0
```

## The scratch worktrees

Both were made with `git worktree add --detach` of dd12b8ed9, edited by hand
(stage A, then stage B on top of it), and removed with `git -C
/srv/data/git/runner/src/tmp/planning/temper-base worktree remove --force
/srv/data/git/runner/src/tmp/planning/scratch-temper-07` (and `...-07b`).
`git worktree list | grep -c scratch-temper-07` prints 0, and `git status
--short` in `temper-base` prints nothing.

## The Done-when commands at the base

Each ran once in `temper-base`. The expected result after the
session is in the session file; a command that cannot print its expected result
at the base is run in its base form, or on the scratch tree, and says so.

```
$ cargo check --workspace --all-targets
dir: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.25s
exit 0
```

Finished (the base is built). Stage B's clippy run (block above) checks the same
targets after both increments.

```
$ python3 -I -c from pathlib import Path; print("gone=" + str(not any(Path(p).exists() for p in ("tests/engine/forge/client", "tests/forge-setup", "tools/forgejo-conformance", "tests/codecs/src/forgejo.rs"))))
dir: /srv/data/git/runner/src/tmp/planning/temper-base
gone=False
exit 0
```

`gone=False`: all four paths exist at the base.

```
$ bash -c ls tests/engine/forge/src; ls tests/codecs/src tests/codecs/tests
dir: /srv/data/git/runner/src/tmp/planning/temper-base
changes.rs
lib.rs
world.rs
tests/codecs/src:
forgejo.rs
lib.rs

tests/codecs/tests:
client_wire.rs
drift.rs
fixtures.rs
forgejo_deliveries.rs
forgejo_files.rs
forgejo_items.rs
forgejo_reviews.rs
forgejo_setup.rs
forgejo_start.rs
forgejo_writes.rs
fuzzy_client_wire.rs
fuzzy_forgejo.rs
fuzzy_store.rs
fuzzy_worker_wire.rs
worker_wire.rs
exit 0
```

(Base form of the expected `changes.rs lib.rs translate.rs world.rs`: no
`translate.rs`; codecs holds the `forgejo*` files and `fixtures.rs`.)

```
$ bash -c ls tests/engine/protocol-forge/src tests/engine/protocol-forge/tests
dir: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/protocol-forge/src:
fixtures.rs
hooks.rs
joint.rs
lib.rs
machine.rs
peer.rs
pipe.rs
protocol.rs

tests/engine/protocol-forge/tests:
fake_protocol.rs
fuzzy_hooks.rs
fuzzy_machine.rs
fuzzy_protocol.rs
hooks.rs
machine.rs
protocol.rs
exit 0
```

```
$ rg -n temper_engine_forge_client_world|temper-engine-forge-client-world|temper_forge_setup_reconcile_world|temper-forge-setup-reconcile-world|forgejo-conformance|forgejo_conformance Cargo.toml crates testing tests tools scripts.config -g *.rs -g *.toml -g *.sh
dir: /srv/data/git/runner/src/tmp/planning/temper-base
Cargo.toml:25:    "tools/forgejo-conformance",
tests/engine/protocol-forge/tests/hooks.rs:154:    let request = include_bytes!("../../../../tools/forgejo-conformance/deliveries/push/request");
tests/engine/protocol-forge/src/machine.rs:315:            .join("../../../tools/forgejo-conformance/transcripts")
tests/codecs/src/forgejo.rs:26:    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/forgejo-conformance/transcripts").join(case);
tests/codecs/src/forgejo.rs:404:    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/forgejo-conformance/transcripts");
tests/codecs/src/forgejo.rs:503:    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/forgejo-conformance/deliveries").join(case);
tests/engine/forge/client/tests/simulation.rs:2:use temper_engine_forge_client_world::{REPO, Settings, World};
tests/engine/forge/client/tests/simulation.rs:39:fn run(settings: Settings) -> temper_engine_forge_client_world::Stats {
tests/engine/forge/client/tests/simulation.rs:91:                commit: temper_engine_forge_client_world::translate::commit(head),
tests/engine/forge/client/tests/simulation.rs:135:                commit: temper_engine_forge_client_world::translate::commit(head),
tests/engine/forge/client/tests/simulation.rs:321:            write: api::Write::Merge { number: pull, head: temper_engine_forge_client_world::translate::commit(head) },
(... 24 more lines)
```

33 hits in 15 files at the base (`rg -c`-style tally: `simulation.rs` 15, the
client's other files 6, reconcile 3, the tool 3, `tests/codecs/src/forgejo.rs` 3,
the protocol world 2, the workspace member 1). An earlier form of this search
also matched the protocol world's own package name, which stays; this is the
corrected one.

```
$ rg -n (temper_engine_protocol_forge_world|crate)::(hooks|joint|machine|peer|pipe)\b tests -g *.rs
dir: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/protocol-forge/tests/hooks.rs:192:    let mut world = temper_engine_protocol_forge_world::joint::Joint::hooked(7);
tests/engine/protocol-forge/src/protocol.rs:2:pub use crate::joint::REPO;
tests/engine/protocol-forge/src/protocol.rs:53:                crate::joint::checks(1000),
tests/engine/protocol-forge/tests/fake_protocol.rs:3:use temper_engine_protocol_forge_world::joint::{Joint, REPO};
tests/engine/protocol-forge/src/joint.rs:27:    hook_peers: BTreeMap<Token, crate::hooks::Peer>,
tests/engine/protocol-forge/src/joint.rs:275:            self.hook_peers.insert(socket, crate::hooks::Peer { wire, owner: None, closed: false });
tests/forge-setup/reconcile/src/exchange.rs:7:use temper_engine_protocol_forge_world::pipe::{Cut, Pipe};
exit 0
```

7 hits at the base, all in files that leave (the world's `hooks.rs` test,
`fake_protocol.rs`, `joint.rs` 2, reconcile's `exchange.rs`) or are edited
(`protocol.rs`: the `pub use` of `REPO` and `checks`).

```
$ rg -n #\[path = "\.\./client tests
dir: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/forge/src/lib.rs:5:#[path = "../client/src/translate.rs"]
exit 0
```

```
$ rg -n skein[-_]forge[-_]machine[-_]world Cargo.toml tests -g *.rs -g *.toml
dir: /srv/data/git/runner/src/tmp/planning/temper-base
exit 1
```

Nothing at the base (exit 1). After the session the session file expects 5 lines
(the workspace line, the world's dependency line, the three `use` lines). The
same command on stage B (the stand-in crate plays skein's):

```
$ rg -n 'skein[-_]forge[-_]machine[-_]world' Cargo.toml tests -g '*.rs' -g '*.toml'
dir: /srv/data/git/runner/src/tmp/planning/scratch-temper-07b
tests/engine/standin-machine/src/lib.rs:1://! Scratch stand-in for skein 08.1's skein-forge-machine-world: the moved files, the public seams.
tests/engine/standin-machine/Cargo.toml:2:name = "skein-forge-machine-world"
tests/engine/protocol-forge/Cargo.toml:18:skein-forge-machine-world = { path = "../standin-machine" }
tests/one-process/Cargo.toml:57:skein-forge-machine-world = { path = "../engine/standin-machine" }
tests/engine/protocol-forge/src/protocol.rs:3:use skein_forge_machine_world::joint::{self, Joint, REPO};
tests/one-process/src/forge.rs:57:        let protocol = skein_forge_machine_world::fixtures::fake_limits();
tests/engine/protocol-forge/tests/protocol.rs:1:use skein_forge_machine_world::joint::REPO;
tests/engine/protocol-forge/tests/fuzzy_protocol.rs:1:use skein_forge_machine_world::joint::REPO;
exit 0
```

It shows the world's manifest line and the three `use` lines; the stand-in's own
two lines and the two `tests/one-process` lines (which temper 05.2 deletes) are
the scratch tree's, and the root `Cargo.toml` has no workspace line there (the
stand-in is a path dependency).

The package runs of the expected result (14 and 4) have no base form: the
protocol world holds 47 and 6 and codecs 60 and 6 at the base (the package runs
under Commands at the base); the scratch trees ran 9 and 1 (stage A), 5 and 3
(stage B), i.e. 14 and 4. The workspace runs: 1,879 and 50 at the base,
1,791 and 42 in stage A, 1,711 and 39 in stage B (Sketches and the size).
