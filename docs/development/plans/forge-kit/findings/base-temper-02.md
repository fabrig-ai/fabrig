# Base runs: temper 02, temper on `skein-forge-client`

2026-10-19. Code: temper dd12b8ed904edced9e65af9fa62cc769c5a09905 (the code of
27995fb75, with the designer answers), run in `/srv/data/git/runner/src/tmp/planning/temper-base`;
skein 064c55f (docs only here: skein 02.2 is not on origin/main, so the kit's
`effect_key` and `measure` could not be compiled at the base). Deliverables:
`temper/02-client.md` and this file. The scratch worktree
`tmp/planning/scratch-temper-02` (detached at dd12b8ed9) held the sketch; it is removed below.
Every run went. Outputs are cut to the relevant lines (at most 30
per command); full logs are in `/srv/data/git/runner/src/tmp/logs/`, named by task.

## What the base showed beyond the prompt

- The prompt's `temper-engine-protocol-store` is `temper-protocol-store` (21 lines).
  Its per-crate lines (20/21/22/21/10/32/13) match the base for the lines that name
  the crate; the lines that reach client state through an alias (`client::`,
  `forge_client::`) number 561 in 48 files, and every one changes (section "Users").
- Only 2 of the client crate's 76 tests are `EffectPurpose` tests and move; the other
  74 leave with the crate and are skein 02.1's. The workspace gate drops 76, adds 5.
- The 15-package set the Proof uses does not contain the client crate: it is 921 at
  the base and 926 after (+5), not 921 - 76 + 5. The gate (1879) does contain it: 1808.
- `jig_domain_core::EffectPurpose` is a different type (jig's, an effect's `origin` at
  `forge_route.rs`); only the client's `EffectPurpose` moves. Nothing in skein, smith
  or hearth names the client crate or `EffectPurpose`; skein's `docs/design/STATUS.md`
  has two "not planned" rows that name the crate (the designer's file, untouched).
- The two agreement tests of protocol-forge compare a `size_of` measure with the
  client's; see "Unsettled" (the running measure).
- Skein 02.3 needs only a test-order adaptation: `restart.rs` already refuses on
  false from `bind_deployment`. Skein 02.4's key fixtures in `simulation.rs` and
  `fuzzy_simulation.rs` are unframed at the base (`deployment:goal`, `[entry]`, the
  world's namespace `world`).
- No dependency beyond the README's Needs (skein 02.2) except the running measure,
  written in the file's Needs as an open question.

## The crate that moves (base form of the proofs)

The client crate exists at the base, so its proofs are run on it directly; the moved
crate (`temper-engine-domain-forge`, which gains `purpose`) is run too.

```text
$ cargo nextest list -p temper-engine-domain-forge-client
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
exit 0
temper-engine-domain-forge-client tests::effect_keys_are_stable_by_purpose_and_separate_deployments
temper-engine-domain-forge-client tests::projection_history_keys_keep_task_position_and_history_family
(76 test lines: count of lines starting `temper-engine-domain-forge-client `)
exit 0
```
The 2 `EffectPurpose` tests move (names kept); 74 leave with the crate.

```text
$ cargo nextest run -p temper-engine-domain-forge-client
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Starting 76 tests across 1 binary
        PASS [   0.007s] (11/76) temper-engine-domain-forge-client tests::effect_keys_are_stable_by_purpose_and_separate_deployments
        PASS [   0.007s] (69/76) temper-engine-domain-forge-client tests::projection_history_keys_keep_task_position_and_history_family
     Summary [   0.084s] 76 tests run: 76 passed, 0 skipped
```
Base: 76 tests, 0.084 s, every one far under 20 ms.

```text
$ cargo nextest run -p temper-engine-domain-forge-client effect_keys_are_stable_by_purpose_and_separate_deployments projection_history_keys_keep_task_position_and_history_family
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.32s
    Starting 2 tests across 1 binary (74 tests skipped)
        PASS [   0.004s] (1/2) temper-engine-domain-forge-client tests::projection_history_keys_keep_task_position_and_history_family
        PASS [   0.006s] (2/2) temper-engine-domain-forge-client tests::effect_keys_are_stable_by_purpose_and_separate_deployments
     Summary [   0.008s] 2 tests run: 2 passed, 74 skipped
exit 0
```
The moved tests' base times: 0.006 s and 0.004 s.

```text
$ cargo nextest run -p temper-engine-domain-forge
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Starting 22 tests across 1 binary
     Summary [   0.015s] 22 tests run: 22 passed, 0 skipped
exit 0
```
Base: 22 tests in 0.015 s; 22 + 5 = 27 after.

```text
$ cargo nextest list -p temper-engine-domain-forge-client -p temper-engine-domain-forge
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
exit 0
(counts: 76 client lines, 22 domain-forge lines; counted with grep of the package name at the line start)
```

## The packages' set and the gates at the base

The 15 packages hold the client's users and their worlds; the client crate is not in them.

```text
$ cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-smith -p temper-protocol-store -p temper-service -p temper-engine-forge-client-world -p temper-engine-forge-world -p temper-engine-protocol-forge -p temper-engine-protocol-forge-world -p temper-engine-domain-world -p temper-conformance-world -p temper-codecs-world -p temper-client-world -p temper-system-world -p forgejo-conformance
directory: /srv/data/git/runner/src/tmp/planning/temper-base
     Summary [   4.823s] 921 tests run: 921 passed, 0 skipped
```
Base: 921 tests in 4.823 s (the slowest, 1.412 s, `held_and_decision_commit_cuts_restore_paged_real_children_and_exact_scripts` in domain-world, is not touched by the increment). After 02.1: 926.

```text
$ cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-smith -p temper-protocol-store -p temper-service -p temper-engine-forge-client-world -p temper-engine-forge-world -p temper-engine-protocol-forge -p temper-engine-protocol-forge-world -p temper-engine-domain-world -p temper-conformance-world -p temper-codecs-world -p temper-client-world -p temper-system-world -p forgejo-conformance --profile fuzzy
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Starting 39 tests across 18 binaries (64 binaries skipped via profile.fuzzy.default-filter)
        SLOW [> 20.000s] (─────) temper-engine-domain-world::fuzzy_walking fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure
     Summary [  20.410s] 39 tests run: 39 passed (1 slow), 0 skipped
```
Base: 39 tests, one of 20.4 s (`fuzzy_walking`), unchanged by the increment.

```text
$ scripts/check.sh
directory: /srv/data/git/runner/src/tmp/planning/temper-base
     Summary [   3.754s] 1879 tests run: 1879 passed, 1 skipped
     Summary [  16.803s] 50 tests run: 50 passed, 0 skipped
exit 0
```
The gate runs the focused tier (1879 tests, 1 skipped, 3.754 s) and the fuzzy tier (50 tests, 16.803 s), then clippy and format. After 02.1: 1808 focused, 50 fuzzy.

```text
$ TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 scripts/proof-one-process.sh
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Starting 19 tests across 1 binary (6 binaries skipped via profile.process.default-filter)
     Summary [ 149.083s] 19 tests run: 19 passed (8 slow), 0 skipped
exit 0
```
Base: 19 tests, 149.083 s alone (145.694 s in `base-runs.md`); 8 slow, the longest `a_change_lands_by_deliveries_with_every_poll_raised`. Run once. Flaky under load: temper 01 saw a failure, this run none.

```text
$ cargo check --workspace --all-targets
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.53s
```

## Done when, each command at the base

Each runs at the base, where the answer is the opposite of the
Done-when answer (the crate and its users exist); after 02.1 each prints what the
session file says. `rg` exit 1 means no match.

```text
$ rg -n 'temper_engine_domain_forge_client|temper-engine-domain-forge-client' Cargo.toml Cargo.lock crates testing tests tools -g '*.rs' -g Cargo.toml -g Cargo.lock
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-domain/Cargo.toml:22:temper-engine-domain-forge-client.workspace = true
exit 0
(counted from the full log: 227 lines = 193 in .rs, 16 Cargo.lock, 17 in the 15 members' manifests and the root, 1 in the client's own manifest)
```
Base: 227 lines. After 02.1: nothing, exit 1.

```text
$ rg -n 'skein_forge_client( as |::\{.* as )' crates testing tests tools -g '*.rs'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit 1
```
Base: no match, exit 1 (the kit is not in temper yet); after 02.1 the same.

```text
$ rg -n 'answer_bytes_unbounded|effect_bytes|client::(answer_bytes|stored_bytes)\b' crates testing tests tools -g '*.rs'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-domain-forge-client/src/lib.rs:40:pub fn effect_bytes(effect: &Effect) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/lib.rs:41:    bounds::effect_bytes(effect)
crates/temper-engine-domain-forge-client/src/lib.rs:50:pub fn answer_bytes_unbounded(answer: &api::Answer) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/lib.rs:51:    bounds::answer_bytes_unbounded(answer)
crates/temper-engine-domain-forge-client/src/bounds.rs:205:pub(crate) fn answer_bytes_unbounded(value: &Answer) -> Option<u64> {
crates/temper-engine-domain-forge-client/src/bounds.rs:214:pub(crate) fn effect_bytes(effect: &crate::Effect) -> Option<u64> {
crates/temper-engine-protocol-forge/src/tests/reads.rs:489:            temper_engine_domain_forge_client::answer_bytes_unbounded(&answer),
crates/temper-engine-protocol-forge/src/tests/contents.rs:259:            temper_engine_domain_forge_client::answer_bytes_unbounded(&answer),
crates/temper-engine-domain/src/store.rs:690:            Ok(answer) => temper_engine_domain_forge_client::answer_bytes_unbounded(answer),
crates/temper-engine-domain-forge/src/domain.rs:178:                Ok(value) => match client::answer_bytes(value, &limits.client) {
crates/temper-engine-domain-forge/src/lib.rs:150:    match client::effect_bytes(&effect) {
crates/temper-engine-domain-forge/src/lib.rs:277:                Ok(answer) => client::answer_bytes_unbounded(answer),
crates/temper-engine-domain-forge/src/lib.rs:370:            client::effect_bytes(&row.entry.effect)?.checked_add(resource)
crates/temper-engine-domain-forge/src/lib.rs:372:        Stored::Entry { entry, .. } => client::effect_bytes(&entry.effect),
crates/temper-engine-domain-forge/src/lib.rs:373:        Stored::Client(row) => client::stored_bytes(row),
exit 0
```
Base: 15 lines: the client's 6 (definitions and wrappers) and the 9 callers named in the file's Shape (3 `effect_bytes` + 1 `stored_bytes` + 1 `answer_bytes_unbounded` in domain-forge `lib.rs`, `domain.rs` `keep_named_answer`, `store.rs`, two protocol-forge tests). After 02.1: nothing, exit 1.

```text
$ rg -l 'enum EffectPurpose' crates testing tests tools -g '*.rs'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-domain-forge-client/src/identity.rs
exit 0
```
Base: the client's `identity.rs`. After 02.1: `crates/temper-engine-domain-forge/src/purpose.rs` alone (jig's `EffectPurpose` is in jig, not temper).

```text
$ python3 -c 'from pathlib import Path; print("client_copy_absent=" + str(not Path("crates/temper-engine-domain-forge-client").exists()))'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
client_copy_absent=False
exit 0
```
Base: False. After 02.1: True.

## The users of the client (rg, at the base)

Searched in temper (`crates`, `tests`, `tools`, `testing`) and in skein, jig, smith and hearth.

```text
$ rg -l 'temper_engine_domain_forge_client|temper-engine-domain-forge-client' --glob '!crates/temper-engine-domain-forge-client/**' --glob '!target/**' --glob '!docs/**' . | sort | wc -l
directory: /srv/data/git/runner/src/tmp/planning/temper-base
120
```
120 files: 103 `.rs`, 16 `Cargo.toml` (the root and 15 members), `Cargo.lock`. The 15 members: crates temper-engine-domain, -domain-forge, -engine-protocol-forge, -engine-smith, temper-protocol-store, temper-service; tests client, codecs, conformance, system, engine/domain, engine/forge, engine/forge/client, engine/protocol-forge; tools/forgejo-conformance.

Lines per directory, by two counts (rg and awk only). `crate-name` is a line
with the crate's name; `alias-path` a line with `<alias>::` that is not `::api` (the files
alias the crate as `client`, `forge_client` or `d`); `union` is the lines with either:

```text
$ rg -l 'temper_engine_domain_forge_client|temper-engine-domain-forge-client' --glob '!crates/temper-engine-domain-forge-client/**' --glob '!target/**' --glob '!docs/**' . | sort > files
  for f in $(grep '\.rs$' files); do
    aliases=$(rg -o --no-line-number 'temper_engine_domain_forge_client(::\{self)? as [A-Za-z_]+' $f | awk '{print $NF}' | sort -u | tr '\n' '|' | sed 's/|$//')
    crate=$(rg -c 'temper_engine_domain_forge_client' $f)
    if [ -n "$aliases" ]; then
      n=$(rg --pcre2 -c "temper_engine_domain_forge_client|\b($aliases)::(?!api\b)" $f)
      nonapi=$(rg --pcre2 -c "\b($aliases)::(?!api\b)" $f)
    else n=$crate; nonapi=0; fi
    echo "$f|$aliases|${crate:-0}|${nonapi:-0}|${n:-0}" >> rows
  done   # then awk sums column 3, 4, 5 of rows per directory
directory: /srv/data/git/runner/src/tmp/planning/temper-base
files 103 crate-name lines 193 alias non-api lines 523 union 716
temper-engine-domain files=10 crate-name=20 alias-path=3 union=23
temper-engine-domain-forge files=13 crate-name=21 alias-path=286 union=307
temper-engine-protocol-forge files=21 crate-name=32 alias-path=14 union=46
temper-engine-smith files=3 crate-name=22 alias-path=0 union=22
temper-protocol-store files=9 crate-name=21 alias-path=148 union=169
temper-service files=6 crate-name=10 alias-path=7 union=17
tests/client files=1 crate-name=1 alias-path=0 union=1
tests/codecs files=1 crate-name=1 alias-path=1 union=2
tests/conformance files=6 crate-name=10 alias-path=1 union=11
tests/engine files=27 crate-name=37 alias-path=60 union=97
tests/system files=2 crate-name=5 alias-path=0 union=5
tools files=4 crate-name=13 alias-path=3 union=16
```
The prompt's 20/21/22/21/10/32/13 are the crate-name column (`temper-protocol-store` is the
prompt's "engine-protocol-store"; 13 is `tools`, the conformance tool). The union, 716
lines in 103 files, is what the file's Choices cite.

Of those, the lines that reach the client's state (not `api`, not the `locate` vocabulary
that 01.1 moves to `skein_forge`), by the second script (rg -P; alias paths and
crate-name paths): 507 + 54 = 561 lines in 48 files:

```text
$ while IFS='|' read -r path alias c a t; do
    n=$(rg -P -c "(?<![A-Za-z_:])$alias::(?!api\b|locate\b)\w" $path)
    m=$(rg -P -c 'temper_engine_domain_forge_client::(?!api\b|locate\b)\w|temper_engine_domain_forge_client::\{[^}]*\b(?!api\b|locate\b)[A-Z]\w*' $path)
    echo "$path|${n:-0}|${m:-0}"
  done < rows > state   # awk sums column 2 and 3, and per directory
directory: /srv/data/git/runner/src/tmp/planning/temper-base
alias state lines 507 crate-name state lines 54 total 561 in 48 files of 103
temper-engine-domain-forge state-lines=287 files=9
temper-protocol-store state-lines=152 files=6
tests/engine state-lines=69 files=13
temper-engine-smith state-lines=15 files=1
temper-engine-domain state-lines=11 files=4
temper-service state-lines=10 files=4
temper-engine-protocol-forge state-lines=8 files=5
tests/conformance state-lines=4 files=2
tests/system state-lines=2 files=2
tools state-lines=2 files=1
tests/codecs state-lines=1 files=1
tests/client state-lines=0 files=0
```

```text
$ rg -n 'EffectPurpose|effect_key' crates tests tools testing -g '*.rs' -g '!crates/temper-engine-domain-forge-client/**' | sed -E 's/^([^:]+:[0-9]+:)\s+/\1 /' | cut -c1-140
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-domain-forge/src/domain.rs:2675: return client::effect_key(
crates/temper-engine-domain-forge/src/domain.rs:2677: &client::EffectPurpose::ProjectionHistory { goal, task, position, family: 1 },
crates/temper-engine-domain-forge/src/domain.rs:2682: return client::effect_key(
crates/temper-engine-domain-forge/src/domain.rs:2684: &client::EffectPurpose::ProjectionHistory { goal, task, position: revision, family: 2 
crates/temper-engine-domain-forge/src/domain.rs:2697: client::effect_key(namespace, &client::EffectPurpose::Projection { goal, part, number 
crates/temper-engine-domain/src/route/forge_route.rs:325:fn effect_key(domain: &Domain, key: CallKey) -> Box<[u8]> {
crates/temper-engine-domain/src/route/forge_route.rs:326: forge_client::effect_key(
crates/temper-engine-domain/src/route/forge_route.rs:328: &forge_client::EffectPurpose::Call {
crates/temper-engine-domain/src/route/forge_route.rs:405: effect_key(domain, key),
crates/temper-engine-domain/src/route/forge_route.rs:2512: origin: jig_domain_core::EffectPurpose::Call {
```
Two callers, four `effect_key` calls (three in `domain.rs`, one in `forge_route.rs`); the `jig_domain_core::EffectPurpose` at line 2512 is jig's own type. In the client: `lib.rs` 1, `identity.rs` 12, `tests.rs` 18 lines (`rg -c`). `Step` (tag 2) and `Release` (tag 4) have no caller outside the client at the base.

```text
$ for r in skein jig smith hearth; do echo "== $r"; (cd $r && rg -l 'temper-engine-domain-forge-client|temper_engine_domain_forge_client|EffectPurpose' --glob '!target/**' --glob '!tmp/**' .); done
directory: /srv/data/git/runner/src
== skein
./docs/design/STATUS.md
== jig
./examples/ops/domain/src/translate.rs
./crates/jig-domain-core/src/lib.rs
./crates/jig-domain-core/src/effects.rs
./testing/jig-test-domain/src/routes.rs
./crates/jig-protocol-store/src/generated/tasks_v3.rs
./testing/jig-test-store/src/generated/v1.rs
./crates/jig-client-protocol-wire/src/generated/v8.rs
./testing/jig-client-test-protocol-wire/src/generated/v1.rs
== smith
== hearth
```
skein's `docs/design/STATUS.md` rows 56 and 58 name the crate in "not planned" rows (the designer owns that file, untouched). jig's 8 files are jig's own, different `EffectPurpose` (`jig-domain-core`) and its generated copies. No Rust file or manifest in skein, smith or hearth names the temper client.

## The sketch: `purpose.rs`, compiled in the scratch worktree

`tmp/planning/scratch-temper-02` (detached at dd12b8ed9). Skein 02.2 is not on origin/main, so
the kit's `effect_key` could not be named: `purpose.rs` holds a private `stand_in` module
that frames exactly as the client's `effect_key` (version 1, a two-byte namespace length,
the namespace, the identity; `None` for an empty namespace or identity or over the cap), and
the scratch-only test `every_tag_frames_like_the_base_client` compares each tag with the
base client's `effect_key`. After skein 02.2 the call is
`skein_forge_client::effect_key(namespace, &self.identity()?, max_bytes)` and the stand-in goes.
The sketch is 114 lines (about 90 without the stand-in), its test file 120 (two moved tests
of 31, one scratch-only test of 15). Callers changed: `domain.rs` (3 calls) and
`forge_route.rs` (1), 15 insertions and 22 deletions in all.

```text
$ cargo clippy -p temper-engine-domain-forge --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-02
error: use of a disallowed type `alloc::vec::Vec`
 --> crates/temper-engine-domain-forge/src/tests/purpose.rs:6:30
error: use of a disallowed macro `alloc::vec`
 --> crates/temper-engine-domain-forge/src/tests/purpose.rs:7:19
error: use of a disallowed macro `alloc::vec`
  --> crates/temper-engine-domain-forge/src/tests/purpose.rs:18:13
error: use of a disallowed macro `alloc::vec`
  --> crates/temper-engine-domain-forge/src/tests/purpose.rs:20:55
error: use of a disallowed macro `alloc::vec`
  --> crates/temper-engine-domain-forge/src/tests/purpose.rs:23:13
error: use of a disallowed macro `alloc::vec`
  --> crates/temper-engine-domain-forge/src/tests/purpose.rs:27:13
error: use of a disallowed macro `alloc::vec`
  --> crates/temper-engine-domain-forge/src/tests/purpose.rs:31:13
error: use of a disallowed type `alloc::vec::Vec`
  --> crates/temper-engine-domain-forge/src/tests/purpose.rs:15:32
error: could not compile `temper-engine-domain-forge` (lib test) due to 8 previous errors
```
First sketch: the test file used a `Vec` helper and `vec!`. The lib was clean; the 8 errors (2 `Vec` types at lines 6 and 15, 6 `vec!` at 7, 18, 20, 23, 27, 31) were all in `src/tests/purpose.rs`, banned by the lints ("Box<[T]> for owned sequences, skein_lib::List or Queue for bounded ones"). Fixed with `&[u8]` slices, `[u8; N]` consts and `Box::from(&b"ab"[..])`.

```text
$ cargo clippy -p temper-engine-domain-forge --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-02
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.39s
```
Clean after the fix (the second sketch).

```text
$ cargo clippy -p temper-engine-domain-forge -p temper-engine-domain --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-02
    Checking temper-engine-domain-forge v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-temper-02/crates/temper-engine-domain-forge)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.56s
exit 0
```
Clean, with the callers changed and the two moved tests added (an earlier run, before the moved tests, was clean too).

```text
$ cargo nextest run -p temper-engine-domain-forge tests::purpose
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-02
    Starting 6 tests across 1 binary (22 tests skipped)
        PASS [   0.002s] (1/6) temper-engine-domain-forge tests::purpose::every_tag_frames_like_the_base_client
        PASS [   0.003s] (2/6) temper-engine-domain-forge tests::purpose::a_history_key_is_the_framed_forty_five_bytes
        PASS [   0.003s] (3/6) temper-engine-domain-forge tests::purpose::projection_history_keys_keep_task_position_and_history_family
        PASS [   0.003s] (4/6) temper-engine-domain-forge tests::purpose::identity_bytes_pin_tags_one_to_five
        PASS [   0.003s] (5/6) temper-engine-domain-forge tests::purpose::a_call_key_is_the_framed_forty_four_bytes
        PASS [   0.003s] (6/6) temper-engine-domain-forge tests::purpose::effect_keys_are_stable_by_purpose_and_separate_deployments
     Summary [   0.003s] 6 tests run: 6 passed, 22 skipped
exit 0
```
Five real tests plus the scratch-only parity test `every_tag_frames_like_the_base_client`: the vectors equal the base client's bytes for all five tags (Call and ProjectionHistory by literal and by comparison, Step, Projection, Release by comparison). Each test 2 to 3 ms.

## The running measure against the kit's widths

The two agreement tests of `temper-engine-protocol-forge` (`the_running_answer_size_agrees_with_the_clients_measure`
in `tests/reads.rs`, ten answers; `contents_sizes_agree_with_the_clients_measure` in
`tests/contents.rs`, four) compare `reads::size::answer`, a `size_of`-based measure, with the
client's `answer_bytes_unbounded` (the base: both `size_of`). Skein 02.2 replaces the client's
side with fixed widths (`SUMMARY_BYTES=128`, `REVIEW_BYTES=128`, `STATUS_BYTES=160`,
`COMMENT_BYTES=96`, `REMARK_BYTES=64`, `FILE_BYTES=64`, `COMMIT_BYTES=32`,
`COLLABORATOR_BYTES=16`, `STRING_BYTES=16`, `ID_BYTES=8`, each const-asserted to be at least
the type's size). The `size_of` of the base's types, from a temporary test in the scratch
worktree (a `scratch_sizes_of_the_running_measure` test printing `size_of::<api::Summary>()`
and the rest; not kept):

```text
$ cargo nextest run -p temper-engine-domain-forge-client scratch_sizes_of_the_running_measure
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-02
    Starting 1 test across 1 binary (76 tests skipped)
SIZES summary=96 review=104 status=128 comment=72 remark=56 file=48 commit=32 collaborator=16 string=16 id=8
     Summary [   0.004s] 1 test run: 1 passed, 76 skipped
exit 0
```
Against the widths: Summary 96 < 128, Review 104 < 128, Status 128 < 160, Comment 72 < 96, Remark 56 < 64, File 48 < 64; Commit 32, Collaborator 16, the string header 16 and the id 8 equal. So any answer holding a Summary, Review, Status, Comment, Remark or File is priced higher by the kit than by the running `size_of` measure, and the equality the two tests assert fails unless the plans' measure also switches to the widths. Skein 01.3's Don't says "skein 02.2 switches them to the measure (decision 24)"; skein 02.2's Files do not list the plans (`reads/size.rs`, `contents.rs`, `actions.rs`, `reads/done.rs`) and its tests line says only that "their byte expectations take these widths". That is a sequencing or design question; this session names it and settles nothing (the file's Open question: the call sites change, and if the running side then disagrees the author stops; no assertion is loosened and no bound raised).

## The repin, in its base form (scratch worktree)

The first commit of 02.1 is `cargo update -p skein-lib`, then `cargo check
--workspace --all-targets`. Skein 02.2 is not on origin/main, so the repin cannot reach it;
the commands were run in the scratch worktree (with the sketch in it) to see what the
repin does today. The scratch's `Cargo.lock` was restored afterwards (`git checkout
Cargo.lock`), and nothing of it is in the plan.

```text
$ cargo update -p skein-lib
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-02
Updating git repository `https://github.com/forgetic/skein.git`
   064c55f..54c5a8b  HEAD -> origin/HEAD
Locking 29 packages to latest Rust 1.95 compatible versions
Updating skein-lib v0.0.0 (https://github.com/forgetic/skein.git#8a51e10b) -> #54c5a8bb
(the other 28 skein-* packages of that source move with it)
exit 0; task 832, ran 4s
```
```text
$ git diff --stat -- Cargo.lock
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-02
 Cargo.lock | 58 +++++++++++++++++++++++++++++-----------------------------
 1 file changed, 29 insertions(+), 29 deletions(-)
```
```text
$ cargo check --workspace --all-targets
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-02
error[E0004]: non-exhaustive patterns: `skein_io::kernel::Op::Lock { .. }` not covered
  --> crates/temper-worker-protocol-git/src/files.rs:296:37   (and :352, two errors)
error: could not compile `temper-worker-protocol-git` (lib) due to 2 previous errors
error[E0004]: non-exhaustive patterns: `skein_io::file::Event::Locked { .. }` and
  `skein_io::file::Event::Busy { .. }` not covered
  --> jig-protocol-store (jig#bbea9408) src/transcripts/write.rs:189:22   (five errors)
error: could not compile `jig-protocol-store` (lib) due to 5 previous errors
exit 101; task 840, ran 39s
log: /srv/data/git/runner/src/tmp/logs/20261019T024452-840.log
```

What this shows, for the planner (settles nothing):

- The repin as the prompt and README write it does not check at today's temper `main`:
  skein's `Op::Lock` breaks `temper-worker-protocol-git` (two matches), and jig, still
  pinned by temper at bbea9408, breaks on `file::Event::Locked`/`Busy` (five matches in
  `jig-protocol-store`). The first is the same failure as findings/base-temper-04.md
  (which saw it at skein #25d2551e); the second is new here, at skein #54c5a8bb.
- Neither is this session's to repair: the README's Rules put the tools plan's first
  repin (which repins jig and temper onto skein's `main` and repairs what broke) before
  temper 01.1, hence before 02.1. By 02.1 the repin commit should be a no-break
  `cargo check`, its repairs being only what skein 02.2 itself breaks (the nine
  measure callers; the file's Shape lists them). 02.1's Needs says so.
- The lockfile move is 29 packages of one git source (29 insertions, 29 deletions),
  as at temper 04's repin.
## Unsettled, named for the designer or the planner

1. **The running measure of the plans.** The two agreement tests (above) cannot keep
   their values once the client's side is the kit's fixed-width measure and the plans'
   `reads/size.rs` stays `size_of`-based: six row types price lower than their widths.
   Skein 01.3's Don't says skein 02.2 switches the plans' `size_of` uses; skein 02.2's
   Files do not list them, and skein 03.3 prices page sizes only. Either skein 02.2 (or a
   temper step before 02.1) moves the plans onto `skein_forge::measure`, or the two
   tests are rewritten by the designer's ruling. The session file names it as an open
   question in 02.1 and in its Needs; it loosens no assertion and raises no bound.
2. **The kit's `effect_key` and `measure` were not compiled.** Skein 02.2 is not on
   origin/main at this base; `purpose.rs` was sketched with a stand-in that frames
   exactly as the base client does, and the golden vectors are the base client's bytes.
   The author compiles the one-line replacement against the repinned kit.
3. **A skein 02.3 or 02.4 already on origin/main** changes the repin's repairs only as the
   file's Shape says (a test-order change in the restore tests; framed key fixtures in
   `simulation.rs` and `fuzzy_simulation.rs`). Neither could be run at the base.
4. **The process tier is flaky under load** (temper 01 saw one failure; this run, alone,
   passed 19 of 19 in 149.083 s against 145.694 s in `base-runs.md`). It runs once.
5. **Slow tests at the base, not touched here:** six of the 921 run above 0.5 s and 48
   above 0.15 s (`PASS` lines of the set's log); the three slowest are
   `held_and_decision_commit_cuts_restore_paged_real_children_and_exact_scripts` (1.412 s),
   `terminal_restart_replays_complete_state_with_delayed_store_and_page_terminals`
   (0.943 s) and `walking_story_replays_exactly_with_store_and_page_latency` (0.770 s),
   all in `temper-engine-domain-world`. The budget is the focused world under 0.5 s;
   02.1 changes none of them, and leaves their time to the session that next changes them.

## The scratch worktree

`git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-temper-02`,
run from `tmp/planning/temper-base`; `git status --short` there prints nothing afterwards,
and `git worktree list` no longer names the scratch. The repin trial changed the scratch's
`Cargo.lock` only, and the scratch is gone with it.
