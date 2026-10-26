# Each command of this plan, run once at the base

2026-10-16, on x1, by the planners' sub-agents (forgejo-17's
retrospective, proposal 1; development.md 5.4): temper at `e40485aa7`,
skein at `04c1562` (its crates equal `ec686b8a`), each from a scratch
worktree at that commit, the cargo commands. The
`runs/<repo>-NN.txt:LINE` of each entry names the planner's list of the
session's commands; each entry carries its command whole.

## Summary

- **skein 01:** 17 `base` commands, 17 as expected. The README's x1 gate
  (`bash -c '… target/check-x1.sh'`): exit 0, the browser suite 37
  passed and the one filtered test skipped, 116 s alone.
- **temper 01 to 15:** 83 `base` commands. 77 as expected at once; four
  were the planner's list escaping `|` as `\|` (the session file's
  commands are right, and as expected once unescaped); two were plan
  defects, fixed: `scripts/check.sh` takes 238 s cold at `e40485aa7` (32
  and 48 s warm), not 120 s (temper 03); the jig pin in `Cargo.lock` is
  `94eaa03405ed`, which client-api moved (temper 05). After fix pass 4,
  the 11 changed commands ran again: 11 as expected (the `done` diffs
  of temper 04 with `B=HEAD`, there being no plan commit at the base).
- **decisions.md 5's limits:** none of the new keys exists at the base;
  each holds against configuration.md 7 by its arithmetic and one probe
  (below). The sockets test at `crates/temper-service/src/tests/checks.rs:182`
  moves to `git_component.operations` 9 in temper 05.3, as its Tests say.
- **The sketches under `cargo clippy --all-targets -- -D warnings`:**
  skein 4, temper 80 (sketch lines). Those naming code an earlier
  increment lands were checked by name (ready-check), and some compiled
  against stubs as well. Defects found: lint-level (`#[must_use]`,
  `# Errors`, `large_enum_variant`, the disallowed `Default` derive,
  `format!` and `Option::map` in step code, `partial_pub_fields`,
  `ip_constant`, `unused_qualifications`), one wrong path (06's
  `client::api::Hint`), one import (12.1's `machine::Order`) and one
  `let … else` syntax error (02); each fixed in the session file
  (fix pass 5) and compiled again clean.
- **Placement, for an author compiling a sketch:** temper's and
  skein's workspaces deny `allow_attributes`; a scratch allow needs a
  reason (`#![expect(…, reason = "…")]`).

## skein: the base commands

### 01.1 (runs/skein-01.txt:3)
`cargo nextest run -p skein-oauth-accounts-world a_refreshed_record_is_stored_privately_before_it_is_lent`
expected: 1 test, 1 passed, under 0.5 s — observed: Summary 1 test run: 1 passed, 52 skipped; PASS [0.004s]; exit 0 — **match**

### 01.1 (runs/skein-01.txt:5)
`cargo nextest run -p skein-io -p skein-io-world -E '...4 filters...'`
expected: 4 tests, 4 passed — observed: Summary [0.331s] 4 tests run: 4 passed, 310 skipped; exit 0 — **match**

### 01.1 (runs/skein-01.txt:6)
`for n in 55 56 63 64 65; do head -c $n /dev/zero | tr '\0' a | sha256sum; done`
expected: five digests 9f4390f8..., b35439a4..., 7d3e74a0..., ffe054fe..., 635361c4... — observed: the same five lines, each "  -"; exit 0 — **match**

### 01.1 (runs/skein-01.txt:7)
`printf ab | sha256sum`
expected: fb8e20fc...0603 — observed: fb8e20fc2e4c3f248c60c39bd652f3c1347298bb977b8b4d5903b85055620603; exit 0 — **match**

### 01.1 (runs/skein-01.txt:8)
`printf '%s' abcdbcde...nopq | sha256sum`
expected: 248d6a61...06c1 — observed: 248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1; exit 0 — **match**

### 01.1 (runs/skein-01.txt:9)
`printf '%s' abcdefgh...nopqrstu | sha256sum`
expected: cf5b16a7...d9d1 — observed: cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1; exit 0 — **match**

### 01.1 (runs/skein-01.txt:10)
`head -c 1000000 /dev/zero | tr '\0' a | sha256sum`
expected: cdc76e5c...2cd0 — observed: cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0; exit 0 — **match**

### 01.2 (runs/skein-01.txt:13)
`for n in 63 64 65; do printf abc | openssl dgst -sha256 -hmac "$(head -c $n /dev/zero | tr '\0' k)" -r; done`
expected: 23670528..., ae0c0e4a..., ed378e5d... each ' *stdin' — observed: the same three lines with ' *stdin'; exit 0 — **match**

### 01.2 (runs/skein-01.txt:14)
`printf 'Hi There!' | openssl dgst -sha256 -mac HMAC -macopt hexkey:0b0b...0b -r`
expected: 5b5b4571...b891a *stdin — observed: 5b5b45718bbcc73dcc6578f7067d021ba39266484b0b24d1ea44c8a853ab891a *stdin; exit 0 — **match**

### 01.2 (runs/skein-01.txt:15)
`cargo nextest run -p skein-oauth-world public_sign_in_refresh_rotation_and_old_refresh_replay`
expected: 1 test, 1 passed, under 0.5 s — observed: 1 test run: 1 passed, 7 skipped; PASS [0.002s]; exit 0 — **match**

### 01.2 (runs/skein-01.txt:16)
`cargo nextest run -p skein-oauth wrong_state_and_redirect_are_terminal_without_exchange`
expected: 1 test, 1 passed — observed: 1 test run: 1 passed, 26 skipped; PASS [0.001s]; exit 0 — **match**

### 01.3 (runs/skein-01.txt:17)
`cargo nextest run -p skein-oauth -p skein-ui-bundle`
expected: all pass, incl. pkce RFC 7636 tests, the_page_matches_its_golden, the_digest_covers_paths_and_bytes — observed: Summary [0.016s] 40 tests run: 40 passed, 0 skipped; pkce tests present (3), both bundler tests PASS; exit 0 — **match**

### 01.3 (runs/skein-01.txt:18)
`cargo nextest run -p skein-oauth-world public_sign_in_refresh_rotation_and_old_refresh_replay`
expected: 1 test, 1 passed, under 0.5 s — observed: 1 test run: 1 passed, 7 skipped; PASS [0.002s]; exit 0 — **match**

### 01.3 (runs/skein-01.txt:19)
`cargo build --target wasm32-unknown-unknown -p skein-ui-example-shell-web && cargo run -p skein-ui-bundle --... --noscript '...'`
expected: exit 0; bundling wall time recorded — observed: exit 0, output dir target/ui-example-bundle/9b02acf966dcb1fd; bundler binary run alone (debug, 3 reps): 1.82 s, 2.00 s, 1.96 s; the command's run step (incl. cargo) 5 s — **match** (timing recorded: ~1.9 s bundler alone)

### done (runs/skein-01.txt:27)
`git diff ec686b8a origin/main -- crates/skein-io/src/digest.rs crates/skein-oauth/src/pkce.rs crates/skein-ui-bundle/src/digest.rs | grep -c -E '^-\s*pub (fn|struct|enum|const|mod|use|type|trait)\b'`
expected: 0 — observed: prints 0 (grep -c exit 1 because of no match) — **match**

### done (runs/skein-01.txt:31)
`git diff --quiet ec686b8a origin/main -- crates/skein-ui-bundle/src/tests/golden`
expected: exit 0 — observed: exit 0 — **match**

### done (runs/skein-01.txt:32)
`sh -c 'export NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3; do cargo nextest run --workspace 2>&1 | grep Summary; done'`
expected: three Summary lines per checkout (medians within 0.5 s across ec686b8a and origin/main) — observed at base (origin/main) only: 4.452 s, 5.163 s, 5.485 s, each 2461 tests passed; median 5.163 s; exit 0. The ec686b8a side was not run (task covers base only) — **match** for the base side

## MISMATCH lines
none

## skein: the gate on x1

```
     Summary [  21.204s] 37 tests run: 37 passed, 1 skipped
exit 0
```

## temper: the base commands

### 01.1 (runs/temper-01.txt:1)
`cargo nextest run -p temper-forge-forgejo -p temper-codecs-world -p temper-fake-forge-protocol`
expected: exit 0 (at ready: the 7 tests of tests::accounts and the changed forgejo_start row pass; under 30 s once built) — observed: exit 0; 111 tests run: 111 passed (summary 0.078 s) — **match**

### 01.1 (runs/temper-01.txt:3)
`cargo nextest run -p temper-engine-protocol-forge-world --test fake_protocol`
expected: exit 0; 5 tests passed, among them the_component_starts_and_lists_items_through_the_fakes_bytes (under 30 s once built) — observed: exit 0; 5 tests run: 5 passed (0.010 s) — **match**

### 01.1 (runs/temper-01.txt:4)
`cargo nextest run -p temper-engine-protocol-forge -E 'test(the_worst_case_counts_answers_and_refuses_overflow)'`
expected: exit 0; 1 test passed (the memory guard of 01.1's Shape) — observed: exit 0; 1 test run: 1 passed (0.003 s) — **match**

### 01.1 (runs/temper-01.txt:6)
`test "$(grep -rn 'Kind::Users' crates/temper-forge-forgejo/src | wc -l)" = 17`
expected: exit 0 (the 17 mentions that mark the exhaustive Kind lists, in 8 files) — observed: exit 0 (count is 17) — **match**

### 01.1 (runs/temper-01.txt:7)
`test "$(grep -rnE 'User \{ id[:,]' --include=*.rs crates testing tests tools | grep -v 'Heard::User' | grep -vc 'Unusable::User')" = 20`
expected: exit 0 (20 lines at the base: the 19 literals of the Files list less the multi-line one at tests/codecs/src/forgejo.rs:221, plus document.rs:179 and engine-protocol-forge/src/start.rs:25, another type) — observed: exit 0 (count is 20) — **match**

### 01.1 (runs/temper-01.txt:8)
`test "$(wc -l < crates/temper-forge-forgejo/src/server/writes.rs)" = 460`
expected: exit 0 (the file the Rules watch for 700 lines) — observed: exit 0 (460 lines) — **match**

### 01.1 (runs/temper-01.txt:9)
`test "$(wc -l < testing/temper-fake-forge-protocol/src/response.rs)" = 741`
expected: exit 0 (the file the Rules watch for 780 lines) — observed: exit 0 (741 lines) — **match**

### 01.2 (runs/temper-01.txt:13)
`test "$(grep -c '^fails ' tools/forgejo-conformance/facts.txt)" = 4`
expected: exit 0 (the base's four known fails stay four) — observed: exit=0; no output — **match**

### 01.2 (runs/temper-01.txt:14)
`test "$(grep -c '^holds component-' tools/forgejo-conformance/facts.txt)" = 40`
expected: exit 0 — observed: exit=0; no output — **match**

### 01.2 (runs/temper-01.txt:17)
`test "$(grep -c 'with_capacity(40)' testing/temper-fake-forge-protocol/src/tally.rs)" = 1`
expected: exit 0 (the tally capacity that 01.2 names ROUTES = 64; 40 Route variants at the base) — observed: exit=0; no output — **match**

### 01.2 (runs/temper-01.txt:18)
`test "$(grep -rn 'Request::DeleteBranch' --include='*.rs' crates testing | wc -l)" = 28`
expected: exit 0 (the exhaustive Request lists the new variants enter) — observed: exit=0; no output — **match**

### done (runs/temper-01.txt:27)
`test "$(grep -c '^fails setup-' tools/forgejo-conformance/facts.txt)" = 0`
expected: exit 0 (0 at the base too) — observed: exit=0; no output — **match**

### done (runs/temper-01.txt:28)
`test "$(grep -c '^fails ' tools/forgejo-conformance/facts.txt)" = 4`
expected: exit 0 — observed: exit=0; no output — **match**

### done (runs/temper-01.txt:29)
`test "$(grep -c '^holds component-' tools/forgejo-conformance/facts.txt)" = 40`
expected: exit 0 — observed: exit=0; no output — **match**

### done (runs/temper-01.txt:32)
`test -z "$(git ls-files 'crates/temper-forge-forgejo/*.rs' 'testing/temper-fake-forge-protocol/*.rs' 'testing/temper-throwaway-forgejo/*.rs' 'tools/forgejo-conformance/*.rs' 'tests/codecs/*.rs' | xargs wc -l | awk '$1 > 800 && $2 != "total"')"`
expected: exit 0 (no file over 800 lines; true at the base) — observed: exit=0; no output — **match**

### done (runs/temper-01.txt:33)
`test "$(git log origin/main --grep='^Plan: forge-setup$' -p --format= -- '*.rs' | grep -E '^\+' | grep -ciE 'actual|genuine|legacy|compat|workaround')" = 0`
expected: exit 0 (vacuous at the base) — observed: exit=0; no output — **match**

### 02.1 (runs/temper-02.txt:1)
`grep -rn 'Request::DeleteBranch' --include='*.rs' crates testing`
expected: prints 28 lines (the exhaustive Request lists the new variants join, in 17 files) — observed: exit 0; 28 lines in 17 files — **match**

### 02.1 (runs/temper-02.txt:2)
`grep -rn 'Answer::CombinedStatus' --include='*.rs' crates testing tests`
expected: prints 18 lines (the exhaustive Answer lists, in 14 files) — observed: exit 0; 18 lines in 14 files — **match**

### 03.1 (runs/temper-03.txt:1)
`"$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64" --version \| head -1`
expected: forgejo version 17.0.0+gitea-1.22.0 (release name 17.0.0) built with GNU Make 4.4.1, go1.26.9 : bindata, timetzdata, sqlite, sqlite_unlock_notify — observed: exit=0; 1 lines — **match**

### 03.1 (runs/temper-03.txt:2)
`"$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64" --version`
expected: forgejo-runner version v13.2.0 — observed: exit=0; 1 lines — **match**

### 03.1 (runs/temper-03.txt:3)
`grep -rnE '"17\.0\.0"\|"13\.2\.0"' crates testing tools tests --include='*.rs'`
expected: base: two lines, testing/temper-throwaway-forgejo/src/binary.rs:9 and :11; ready: two lines, both in crates/temper-forge-forgejo/src/pin.rs — observed: exit=1; no output — **MISMATCH: the command carries a markdown-escaped `\|`, which grep -E reads as a literal pipe: exit 1, no output (two lines expected)**

### 03.1 (runs/temper-03.txt:4)
`grep -rnE 'fn names_(forgejo\|runner)' crates testing tools tests --include='*.rs'`
expected: base: two lines, testing/temper-throwaway-forgejo/src/binary.rs:14 and :19; ready: two lines, both in crates/temper-forge-forgejo/src/pin.rs — observed: exit=1; no output — **MISMATCH: same `\|` escape: exit 1, no output (two lines expected)**

### 03.1 (runs/temper-03.txt:5)
`grep -rnE '\b(RUNNER_VERSION\|names_forgejo\|names_runner)\b\|temper_throwaway_forgejo::VERSION' crates testing tools tests --include='*.rs'`
expected: users at base, 16 lines: throwaway src/tests.rs 2,40,51,56,61,62,63,64,66; src/lib.rs 5; src/binary.rs 11,14,19,20,51; tools/forgejo-conformance/src/runner.rs 203; no other file (bare VERSION also at tests.rs 36-49 and tests/throwaway.rs 2,25) — observed: exit=1; no output — **MISMATCH: same `\|` escape: exit 1, no output (16 lines expected)**

### 03.1 (runs/temper-03.txt:7)
`TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 sh -c 'cargo run -p forgejo-conformance -- --scratch target/forgejo-conformance --out tools/forgejo-conformance && cargo run -p forgejo-conformance -- --scratch target/forgejo-component --out tools/forgejo-conformance --component'`
expected: exit 0 from a clean tree (about 70 s of recordings after the build); same outcome at base and ready; the tree is dirty after (git checkout -- tools/forgejo-conformance) — observed: exit 0; ran 79 s (77 s alone) incl. build — **match**

### 03.1 (runs/temper-03.txt:8)
`head -2 tools/forgejo-conformance/facts.txt`
expected: after the recording: line 1 begins "# forgejo forgejo version 17.0.0+gitea-1.22.0 (release name 17.0.0)" and ends "sha256 5a3b07e124ed70b4e46ee3939515f003bd3cc85352257a9a4607a527251042ca"; line 2 is "# runner forgejo-runner version v13.2.0" — observed: exit 0; line 1 begins '# forgejo forgejo version 17.0.0+gitea-1.22.0 (release name 17.0.0)' and ends sha256 5a3b07e1...42ca; line 2 '# runner forgejo-runner version v13.2.0' (tree restored first, so this reads the committed facts.txt) — **match**

### 03.1 (runs/temper-03.txt:9)
`grep -c '^holds ' tools/forgejo-conformance/facts.txt`
expected: after the recording: 109 — observed: 109 (committed facts.txt) — **match**

### 03.1 (runs/temper-03.txt:10)
`grep -c '^holds component-' tools/forgejo-conformance/facts.txt`
expected: after the recording: 40 — observed: 40 (committed facts.txt) — **match**

### 03.1 (runs/temper-03.txt:11)
`grep -E '^(fails\|unchecked) ' tools/forgejo-conformance/facts.txt \| cut -d: -f1`
expected: after the recording: fails pull-for-names-the-newest-of-several / fails open-pull-no-changes-is-422 / fails delete-missing-branch-is-404 / fails effective-protection-name-for-a-writer / unchecked webhook-deliveries, nothing else — observed: exit=1; 7 lines — **MISMATCH: `\| cut` is passed to grep as arguments: 'grep: invalid argument : for --directories', exit 1 (the escaped pipe cannot run as written)**

### 03.2 (runs/temper-03.txt:12)
`grep -n -e 'fn command' -e 'fn run\b' -e 'fn token' testing/temper-throwaway-forgejo/src/instance.rs`
expected: base: instance.rs:84 (command), :94 (run), :116 (token) — observed: exit=0; 3 lines — **match**

### 03.2 (runs/temper-03.txt:13)
`grep -rn -e 'self\.token(' -e 'instance\.token(' testing/temper-throwaway-forgejo tools crates/temper/tests --include='*.rs'`
expected: users of the throwaway's token at base: testing/temper-throwaway-forgejo/src/instance.rs:79 and :168 only — observed: exit=0; 2 lines — **match**

### 03.3 (runs/temper-03.txt:22)
`grep -rn -e 'RegistrationToken' -e 'registration-token' -e 'registration_token' crates testing tools tests --include='*.rs'`
expected: base, 3 lines: tools/forgejo-conformance/src/runner.rs:206 and :210, src/write.rs:9 — observed: exit=0; 3 lines — **match**

### 03.3 (runs/temper-03.txt:23)
`grep -rn 'TEMPER_FORGEJO_RUNNER' crates testing tools tests --include='*.rs'`
expected: base, 5 lines: runner.rs:55 and :60, component/reads.rs:57, :93 and :96 — observed: exit=0; 5 lines — **match**

### done (runs/temper-03.txt:34)
`grep -c 'binary.rs' tools/forgejo-conformance/README.md tools/git-transcripts/README.md`
expected: base: "tools/forgejo-conformance/README.md:1" and "tools/git-transcripts/README.md:1"; ready: ":0" for both — observed: exit=0; 2 lines — **match**

### done (runs/temper-03.txt:35)
`cargo nextest run -p temper-forge-forgejo -p temper-throwaway-forgejo -p forgejo-conformance`
expected: exit 0 at base and at ready — observed: exit 0; 55 tests run: 55 passed, 1 skipped — **match**

### done (runs/temper-03.txt:41)
`grep -rlE "generated random password is '[^<]\|0123456789abcdef0123456789abcdef01234567\|fedcba9876543210fedcba9876543210fedcba98" tools/forgejo-conformance/facts.txt tools/forgejo-conformance/transcripts`
expected: prints nothing (exit 1), at base and at ready — observed: exit=1; no output — **match**

### done (runs/temper-03.txt:42)
`scripts/check.sh`
expected: exit 0 (base: 120 s cold, findings/measurements.md) — observed: exit=0; 202 lines — **MISMATCH: ran 238 s alone (log 20261016T191332-0.log), not about 120 s cold; exit 0, all pass (later reruns of the same gate: 32 s and 48 s)**

### 04.1 (runs/temper-04.txt:1)
`cargo nextest run -p temper-throwaway-forgejo -p forgejo-conformance`
expected: exit 0 (the existing tests pass at the base; the 7 new tests of 04.1 join them) — observed: exit 0; 18 tests run: 18 passed, 1 skipped — **match**

### 04.1 (runs/temper-04.txt:2)
`TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 sh -c 'cargo run -p forgejo-conformance -- --scratch target/forgejo-conformance --out tools/forgejo-conformance && cargo run -p forgejo-conformance -- --component --scratch target/forgejo-conformance --out tools/forgejo-conformance'`
expected: exit 0 within 150 s alone (at the base: about 70 s, 1 unchecked webhook-deliveries; it rewrites 34 transcripts, restore with git checkout -- tools/forgejo-conformance/transcripts; after 04.1: no unchecked, 21 delivery facts, deliveries/ with at least 21 directories) — observed: exit 0; ran 70 s (67 s alone); the output's tail shows no unchecked line, so the 1 unchecked webhook-deliveries was not visible in the output — **match**

### done (runs/temper-04.txt:4)
`cargo nextest run -p temper-throwaway-forgejo -p forgejo-conformance -p temper-forge-forgejo -p temper-codecs-world`
expected: exit 0 — observed: exit 0; 89 tests run: 89 passed, 1 skipped — **match**

### done (runs/temper-04.txt:5)
`cargo nextest run -p temper-codecs-world --profile fuzzy`
expected: exit 0 — observed: exit 0; 5 tests run: 5 passed — **match**

### done (runs/temper-04.txt:6)
`grep -c '^unchecked' tools/forgejo-conformance/facts.txt`
expected: prints 0 after 04.1 (1 at the base) — observed: prints 1, exit 0 (the base value) — **match**

### done (runs/temper-04.txt:7)
`grep -x 'holds webhook-deliveries' tools/forgejo-conformance/facts.txt`
expected: prints the line after 04.1 (nothing and exit 1 at the base) — observed: no output, exit 1 (the base value) — **match**

### done (runs/temper-04.txt:8)
`grep -c '^\(holds\|fails\) delivery-' tools/forgejo-conformance/facts.txt`
expected: prints 21 after 04.1 (0 at the base) — observed: prints 0, exit 1 (the base value) — **match**

### done (runs/temper-04.txt:9)
`grep '^fails delivery-' tools/forgejo-conformance/facts.txt | grep -v -e 'job-end-\(follows\|precedes\)-its-commit-status' -e 'pull-is-\(settled\|checking\)-when-its-push-is-delivered' -e 'branch-deletion-sends-no-push'`
expected: prints nothing — observed: exit=1; no output — **match**

### done (runs/temper-04.txt:10)
`grep -c -x -e 'holds delivery-job-end-follows-its-commit-status' -e 'holds delivery-job-end-precedes-its-commit-status' tools/forgejo-conformance/facts.txt`
expected: prints 1 after 04.1 (0 at the base) — observed: prints 0, exit 1 (the base value) — **match**

### done (runs/temper-04.txt:11)
`grep -c -x -e 'holds delivery-pull-is-settled-when-its-push-is-delivered' -e 'holds delivery-pull-is-checking-when-its-push-is-delivered' tools/forgejo-conformance/facts.txt`
expected: prints 1 after 04.1 (0 at the base) — observed: prints 0, exit 1 (the base value) — **match**

### done (runs/temper-04.txt:13)
`grep -c 'left unchecked until webhooks' tools/forgejo-conformance/README.md`
expected: prints 0 after 04.1 (1 at the base) — observed: prints 1 (the base value) — **match**

### done (runs/temper-04.txt:15)
`git diff origin/main --stat -- tools/forgejo-conformance/transcripts`
expected: prints nothing — observed: exit=0; no output — **match**

### done (runs/temper-04.txt:16)
`git diff origin/main --name-only -- crates/temper-forge-forgejo tools/forgejo-conformance testing/temper-throwaway-forgejo tests/codecs | grep '\.rs$' | xargs -r grep -nE 'actual|genuine|legacy|compat|workaround'`
expected: prints nothing — observed: exit=0; no output — **match**

### done (runs/temper-04.txt:17)
`git diff origin/main --name-only --diff-filter=A -- crates/temper-forge-forgejo tools/forgejo-conformance testing/temper-throwaway-forgejo tests/codecs | grep '\.rs$' | xargs -r wc -l | awk '$2 != "total" && $1 > 800'`
expected: prints nothing — observed: exit=0; no output — **match**

### 05.1 (runs/temper-05.txt:3)
`grep -o 'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' Cargo.lock | sort -u`
expected: three lines: jig 2900e9b3bb67, skein 8a10437fa557, smith 29fa97c9cdfd at the base; after the repin the skein line is the sha above — observed: jig 94eaa03405ed3868657697023fe9b80a0ea0b30c, skein 8a10437fa55735f83502cd189b0314fbcf3af5fc, smith 29fa97c9cdfdcd1a0d0bc28e270de5be221c0928 — **MISMATCH: jig line is 94eaa03405ed, expected 2900e9b3bb67 (skein 8a10437fa557 and smith 29fa97c9cdfd match)**

### 05.1 (runs/temper-05.txt:5)
`printf 'what do ya want for nothing?' | openssl dgst -sha256 -hmac Jefe`
expected: SHA2-256(stdin)= 5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843 (RFC 4231 case 2, the literal of the signature test) — observed: exit=0; 1 lines — **match**

### 05.1 (runs/temper-05.txt:7)
`git grep -n 'default_paging_num:' -- '*.rs'`
expected: 8 lines; the Config literals among them are testing/temper-fake-forge-protocol/src/tests/fixtures.rs:22, testing/temper-fake-git/src/fixtures.rs:109 and tests/engine/protocol-forge/src/fixtures.rs:156 — observed: 8 lines, exit 0; includes tests/fixtures.rs:22, temper-fake-git/src/fixtures.rs:109, protocol-forge/src/fixtures.rs:156 — **match**

### 05.2 (runs/temper-05.txt:10)
`git grep -n -e 'start_backoff_max:' -e 'Settings { forges' -- '*.rs'`
expected: 9 lines: limits.rs:44 (the field) and the eight literal sites listed in Files — observed: 9 lines, exit 0 — **match**

### 05.3 (runs/temper-05.txt:15)
`git grep -l '"forges"' -- '*.rs'`
expected: 8 files; the three that write a whole document (tests/fixtures/startup.rs, tests/support/configuration.rs, one-process fixtures.rs) are named in Files — observed: 8 files, exit 0 — **match**

### 05.3 (runs/temper-05.txt:16)
`git grep -n -w NotBuilt -- crates`
expected: 6 lines at the base: application.rs:343, problems.rs:14, :74 and :169, tests/startup.rs:106 and :111; nothing after 05.3 — observed: 6 lines, exit 0 (application.rs:343, problems.rs:14 and :169, tests/startup.rs:106 and :111) — **match**

### done (runs/temper-05.txt:25)
`git grep -n -e '^sha2' -e '^hmac' -e '^subtle' -- crates/temper-engine-protocol-forge/Cargo.toml crates/temper-service/Cargo.toml testing/temper-fake-forge-protocol/Cargo.toml`
expected: prints nothing (exit 1), at the base and after — observed: exit=1; no output — **match**

### 6.2 (runs/temper-06.txt:2)
`git grep -n 'hinted: Duration::from_secs' -- crates testing tests tools`
expected: 10 lines at the base, the ten literals of client::Limits (one per file); each gains a settle line after slow — observed: 10 lines, exit 0 — **match**

### done (runs/temper-06.txt:5)
`cargo nextest run -p temper-fake-forge-domain -p temper-engine-forge-world -p temper-engine-domain-forge-client -p temper-engine-domain-forge -p temper-engine-domain-forge-change -p temper-service -p temper-conformance-world`
expected: passes at the base (no settle.rs, hooks.rs or settling yet); passes after the session, with the change crate's two settle_window tests among them — observed: exit 0; 426 tests run: 426 passed — **match**

### done (runs/temper-06.txt:6)
`git grep -n -e 'Change::Status' -e 'Change::Wiki' -e 'Hook::status' -e 'Hook::wiki' -- crates testing tests tools`
expected: 13 lines at the base; nothing and exit 1 after — observed: 13 lines, exit 0 — **match**

### done (runs/temper-06.txt:7)
`git grep -n 'settle::window' -- crates/temper-engine-domain-forge/src/domain.rs`
expected: exit 1 and no output at the base; one line after — observed: exit=1; no output — **match**

### done (runs/temper-06.txt:8)
`git grep -c 'forge.client.settle_ms' -- crates/temper-service/src/limits/names.rs`
expected: exit 1 and no output at the base; crates/temper-service/src/limits/names.rs:1 after — observed: exit=1; no output — **match**

### done (runs/temper-06.txt:9)
`scripts/check.sh`
expected: exits 0 at the base (about 120 s cold); exits 0 after the session — observed: exit 0; ran 32 s — **match**

### 07.3 (runs/temper-07.txt:3)
`export TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64; sh -c 'NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper --profile process -E "test(=temper_open_adopts_a_repository_by_commands)"'`
expected: 1 test run: 1 passed, within 30 s (13.1 s at the base; the tree committed first); after 07.3 the same command runs on the id read from secrets/forge-1.user — observed: exit 0; 1 test run: 1 passed, PASS [12.767s] (summary 12.767 s) — **match**

### done (runs/temper-07.txt:12)
`git grep -c -E 'temper-forge-setup|temper-box' origin/main -- crates/temper-service/Cargo.toml`
expected: prints nothing (exit 1) — observed: exit=1; no output — **match**

### 08.3 (runs/temper-08.txt:5)
`env TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 scripts/proof-one-process.sh`
expected: exit 0; at the base "6 tests run: 6 passed" in about 45 s (findings/forge-and-tiers.md 4: 42.4 s); after 08.3 every test passes, at most 90 s — observed: exit 0; 6 tests run: 6 passed (summary 43.304 s; ran 53 s with build) — **match**

### 08.3 (runs/temper-08.txt:6)
`env TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 sh -c 'export NEXTEST_DOUBLE_SPAWN=0; cargo build -p temper --bin temper && cargo nextest run -p temper --profile live'`
expected: exit 0; Summary "1 test run: 1 passed" (live binary, fake mode: TEMPER_RUN_LOGS unset); at the base and after 08.3, at most 90 s — observed: exit 0; 1 test run: 1 passed (23.803 s; ran 26 s) — **match**

### done (runs/temper-08.txt:8)
`git grep -n -F 'actions: false' origin/main -- crates/temper/tests/support/forgejo.rs`
expected: at the base prints one line (forgejo.rs:19, exit 0); after 08.3 prints nothing (exit 1) — observed: exit=0; 1 lines — **match**

### done (runs/temper-08.txt:12)
`git grep -l -E 'process::(Command|Child|Stdio|exit|abort)|CommandExt|try_lock|lock_shared' origin/main -- 'crates/*/src/*' ':!*/tests/*' ':!*/tests.rs'`
expected: at the base prints nothing (exit 1); after 08.1 prints only origin/main:crates/temper-forge-setup-shell/src/process.rs (exit 0) — observed: exit=1; no output — **match**

### done (runs/temper-08.txt:13)
`git grep -c --all-match -F -e 'temper forge prepare CONFIG.json FORGE' -e 'temper forge run CONFIG.json FORGE' origin/main -- crates/temper/src/lib.rs`
expected: at the base prints nothing (exit 1); after 08.2 prints origin/main:crates/temper/src/lib.rs:1 — observed: exit=1; no output — **match**

### done (runs/temper-09.txt:13)
`git ls-files -z -- crates/temper-forge-setup testing/temper-fake-forge-protocol tests/forge-setup | xargs -0 wc -l | awk '$1 > 800 && $2 != "total"'`
expected: prints nothing (at the base and at the end) — observed: exit=0; no output — **match**

### done (runs/temper-09.txt:14)
`! git grep -nE 'std::|skein_io|skein_http|format!' -- crates/temper-forge-setup/src/reconcile.rs crates/temper-forge-setup/src/reconcile`
expected: exit 0, prints nothing (at the base the paths do not exist) — observed: exit=0; no output — **match**

### done (runs/temper-09.txt:15)
`! git grep -nEi 'actual|genuine|legacy|compat|workaround' -- crates/temper-forge-setup/src/reconcile.rs crates/temper-forge-setup/src/reconcile crates/temper-forge-setup/src/tests/reconcile.rs testing/temper-fake-forge-protocol/src/setup.rs testing/temper-fake-forge-protocol/src/setup testing/temper-fake-forge-protocol/src/tests/setup.rs tests/forge-setup`
expected: exit 0, prints nothing (at the base the paths do not exist; the fake's older tests under src/tests use "actual" and are not in these paths) — observed: exit=0; no output — **match**

### done (runs/temper-10.txt:8)
`scripts/check.sh`
expected: exit 0 (120 s cold at the base) — observed: exit 0; ran 48 s — **match**

### 11.1 (runs/temper-11.txt:1)
`git grep -n 'Forge::Throwaway' -- crates/temper/tests`
expected: prints three lines at the base, all in forgejo.rs (:29, :115, :228): the variant's users that 10.2 changes (it loses token and user) and 11.1 changes (it gains webhooks) — observed: 3 lines, forgejo.rs:29, :115, :228 — **match**

### done (runs/temper-11.txt:13)
`grep -cF 'not binary(/^(process' .config/nextest.toml`
expected: prints 2 — observed: exit=0; 1 lines — **match**

### done (runs/temper-11.txt:14)
`! grep -rn -e actual -e genuine -e legacy -e compat -e workaround crates/temper/tests/process.rs crates/temper/tests/support`
expected: exit 0, prints nothing — observed: exit=0; no output — **match**

### 12.1 (runs/temper-12.txt:1)
`git grep -n -E 'temper_box|temper-box' -- crates tests testing tools`
expected: prints nothing (no code names the new crate at the base) — observed: exit=1; no output — **match**

### done (runs/temper-13.txt:8)
`grep -rnE 'Command::new|try_lock|\.exec\(' crates/temper/src`
expected: prints nothing (grep exits 1); checked at the base — observed: exit=1; no output — **match**

### 14.1 (runs/temper-14.txt:2)
`git grep -n 'workshop\|temper-bot' -- crates/temper/tests`
expected: lists the tests naming the tier's org and bot (forgejo.rs:22 and :121 'temper-bot' at the base, exit 0; 08.3 and 10.2 may add 'workshop'); each becomes acme / temper — observed: 2 lines, forgejo.rs:22 and :121 'temper-bot', exit 0 — **match**

### 14.1 (runs/temper-14.txt:3)
`git grep -n '"owned"' -- crates/temper/tests ':!crates/temper/tests/support/owned.rs'`
expected: prints nothing at the base (exit 1); at the ready check it lists the owned members that 08.3, 10.2 and any 11 landed first write by hand, which 14.1 removes — observed: exit=1; no output — **match**

### done (runs/temper-14.txt:11)
`git grep -n '127.0.0.1:3000' -- crates/temper/tests ':!crates/temper/tests/support/owned.rs'`
expected: prints nothing (exit 1); checked at the base — observed: exit=1; no output — **match**

### done (runs/temper-15.txt:4)
`git grep -n -E '"v?[0-9]+\.[0-9]+\.[0-9]+"' origin/main -- crates/temper-forge-setup/src crates/temper-forge-setup-shell/src ':!*/tests/*' ':!*/tests.rs'`
expected: prints nothing (exit 1); at the base the two crates do not exist, so it is checked at the ready check — observed: exit=1; no output — **match**

## Mismatches

- runs/temper-03.txt:3: the command carries a markdown-escaped `\|`, which grep -E reads as a literal pipe: exit 1, no output (two lines expected)
- runs/temper-03.txt:4: same `\|` escape: exit 1, no output (two lines expected)
- runs/temper-03.txt:5: same `\|` escape: exit 1, no output (16 lines expected)
- runs/temper-03.txt:11: `\| cut` is passed to grep as arguments: 'grep: invalid argument : for --directories', exit 1 (the escaped pipe cannot run as written)
- runs/temper-03.txt:42: ran 238 s alone (log 20261016T191332-0.log), not about 120 s cold; exit 0, all pass (later reruns of the same gate: 32 s and 48 s)
- runs/temper-05.txt:3: jig line is 94eaa03405ed, expected 2900e9b3bb67 (skein 8a10437fa557 and smith 29fa97c9cdfd match)

## Second run, after fix pass 4

Base: temper e40485aa7 (scratch worktree temper-forge-setup-base). Commands whose text was already in the first run, and `scripts/check.sh`, were not rerun. The runs/temper-03.txt greps carry plain `|` in the file now, so no `\|` rewrite was needed. Nothing was written into the worktree.

### 01.? (runs/temper-01.txt:26)
`test "$(grep -cE '^fails setup-(create-user|created-user|user-of|a-non-admin|the-admin-token|every-recorded|a-user-made|admin-route|token-|a-made-token|a-git-token|admin-token|api-token|git-token)' tools/forgejo-conformance/facts.txt)" = 0`
expected: exit 0 (0 at the base too) — observed: exit 0 — **match**

### 03.1 (runs/temper-03.txt:3)
`grep -rnE '"17\.0\.0"|"13\.2\.0"' crates testing tools tests --include='*.rs'`
expected: base: two lines, binary.rs:9 and :11 — observed: testing/temper-throwaway-forgejo/src/binary.rs:9 (VERSION) and :11 (RUNNER_VERSION), exit 0 — **match**

### 03.1 (runs/temper-03.txt:4)
`grep -rnE 'fn names_(forgejo|runner)' crates testing tools tests --include='*.rs'`
expected: base: two lines, binary.rs:14 and :19 — observed: binary.rs:14 and :19, exit 0 — **match**

### 03.1 (runs/temper-03.txt:5)
`grep -rnE '\b(RUNNER_VERSION|names_forgejo|names_runner)\b|temper_throwaway_forgejo::VERSION' crates testing tools tests --include='*.rs'`
expected: 16 lines (tests.rs 2,40,51,56,61,62,63,64,66; lib.rs 5; binary.rs 11,14,19,20,51; runner.rs 203) — observed: exactly those 16 lines, exit 0 — **match**

### 03.1 (runs/temper-03.txt:10)
`grep -E '^(fails|unchecked) ' tools/forgejo-conformance/facts.txt | cut -d: -f1`
expected: (a `base` line describing the state after the recording) fails pull-for-names-the-newest-of-several / fails open-pull-no-changes-is-422 / fails delete-missing-branch-is-404 / fails effective-protection-name-for-a-writer / unchecked webhook-deliveries — observed: those five lines, in that order, nothing else; the base already holds them, consistent with the "base" expectation — **match**

### 04.? (runs/temper-04.txt:8)
`grep -c '^holds delivery-' tools/forgejo-conformance/facts.txt`
expected: 21 minus the next count after 04.1 (0 at the base) — observed: 0, exit 1 (grep -c with no match) — **match**

### 04.? (runs/temper-04.txt:9)
`grep -c '^fails delivery-' tools/forgejo-conformance/facts.txt`
expected: 2, or 3 after 04.1 (0 at the base) — observed: 0, exit 1 — **match**

### 04.? (runs/temper-04.txt:17)
`git log --all-match --grep='^Plan: forge-setup$' --format=%H | tail -1`
expected: nothing at the base — observed: no output, exit 0 — **match**

### 04.? (runs/temper-04.txt:18) run with B=HEAD
`git diff HEAD --stat -- tools/forgejo-conformance/transcripts`
expected: prints nothing — observed: no output, exit 0 (B=HEAD, as no plan commit exists at the base, so trivially empty) — **match**

### 04.? (runs/temper-04.txt:19) run with B=HEAD
`git diff HEAD --name-only -- crates/temper-forge-forgejo tools/forgejo-conformance testing/temper-throwaway-forgejo tests/codecs | grep '\.rs$' | xargs -r grep -nE 'actual|genuine|legacy|compat|workaround'`
expected: prints nothing — observed: no output, exit 0 (B=HEAD) — **match**

### 04.? (runs/temper-04.txt:20) run with B=HEAD
`git diff HEAD --name-only --diff-filter=A -- crates/temper-forge-forgejo tools/forgejo-conformance testing/temper-throwaway-forgejo tests/codecs | grep '\.rs$' | xargs -r wc -l | awk '$2 != "total" && $1 > 800'`
expected: prints nothing — observed: no output, exit 0 (B=HEAD) — **match**

### Mismatches (second run)

None: 11 commands run, 11 matched.

## decisions.md 5's limits

None of the new keys exist at the base (no setup., box., hook*, settle_ms in
temper-service limits.rs, forge.rs `small`, or the forge component's Limits).
Existing: the component's connections/deadline/body_bytes checks (temper-
engine-protocol-forge limits.rs `check`) and checks::sockets (startup/
checks.rs: jig's engine+worker io.sockets, 16+16=32; no temper constant yet).
No check takes the new values, so each is arithmetic. One throwaway probe
(`cargo nextest run -p temper-service -E 'test(zz_probe_sockets)'
--no-capture`, reverted) read the fixtures' small profile: clients.connections
4, accounts 1 (handed in), worker slots 1, git operations 3, forge calls 2,
lifetime 10 s, io intake 1048576, forge body_bytes max(16384, 2*512+21)=16384.

- sockets 48, "hooks within the sockets left" (7): base count 4+1+2+1(account)
  +1+1*4(slots)+3*4(git) = 25; +2 forge connections = 27; left 48-27 = 21
  >= hooks 4 (20 with the webhook listener's socket). Whole demand 32 <= 48.
  **holds** at slots 1 (each slot adds 4: fails past 5 slots; check the owned
  example's slots).
- hook_head_bytes 8192 <= io intake 1048576 (HTTP's largest read). **holds**
- hook_body_bytes 1048576 >= forge body_bytes 16384 (largest write + marker).
  **holds**
- hook_deadline_ms 10000 <= forge.client.lifetime 10 s (7). **holds** (equal;
  the base `Deadline` refusal is only past lifetime)
- hook_secret_bytes 128 >= 32 (7), and >= reconcile's 64 digits. **holds**
- hooks 4 and every hook limit above 0 (7). **holds**
- forge.client.settle_ms 30000 above 0 (7). **holds**
- setup.backoff_ms 100 <= setup.backoff_max_ms 5000. **holds**
- setup.backoff_max_ms 5000 < setup.deadline_ms 120000. **holds**
- box.backoff_ms 500 <= box.backoff_max_ms 30000. **holds**
- box.backoff_max_ms 30000 < box.window_ms 300000. **holds**
- box.restarts 5 <= 64. **holds**
- setup.orgs 8, box.grace_ms 10000: no cross-check stated. **holds**

Side effect (not a failure): base test
`one_ios_entities_hold_both_services_and_their_children` (tests/checks.rs)
expects `engine.limits.io.sockets: 33 bytes, past its bound of 32`; with
temper's sockets 48 that demand fits, so the plan must change its overrides
or expectation.

## The sketches: skein

Placement (mine, not the sketches'): crates/skein-lib/src/sha256.rs (01.1 and 01.2 blocks joined), `pub mod sha256;` in lib.rs (a private module leaves the `#[expect(clippy::new_without_default)]` unfulfilled, since the lint needs an exported item), tests/sha256.rs with `mod sha256;` in tests.rs, each `mod` carrying `#[allow(clippy::todo, unused, dead_code)]`; the workspace lints deny `allow` attributes without reason, so lib.rs also got a crate-level `#![allow(clippy::allow_attributes, clippy::allow_attributes_without_reason)]`; the test's `crate::sha256::Hmac` written `sha256::Hmac` (unused_qualifications; the sketch already imports `sha256`). All four built together.

Command for all four: `cargo clippy -p skein-lib --all-targets -- -D warnings`

### 01.1 skein-lib Sha256, Debug, digest (runs/skein-01.txt:34)
compile — as above — **clean**

### 01.1 skein-lib EDGES, hex, messages_at_the_padding_edges_digest_as_sha256sum_prints (runs/skein-01.txt:35)
compile — as above — **clean** (except the qualification note below, in the 01.2 test)

### 01.2 skein-lib Hmac, Debug, equal (runs/skein-01.txt:36)
compile — as above — **clean**

### 01.2 skein-lib RFC_4231, rfc_4231_cases_1_to_7_give_their_published_macs (runs/skein-01.txt:37)
compile — as above — **DEFECT (minor): the test writes `crate::sha256::Hmac::new(key)` beside `use crate::sha256;`, which `unused_qualifications` rejects under -D warnings; should read `sha256::Hmac::new(key)`, at skein/01-sha256.md:~241 (the `let mut mac` line of the 01.2 test block)**

## DEFECT lines
- runs/skein-01.txt:37: 01-sha256.md 01.2 test, `crate::sha256::Hmac::new` is an unnecessary qualification (clippy -D warnings); write `sha256::Hmac::new`.

## Recompiled after fix pass 5

Worktree skein-forge-setup-base, placed as at the top of this record (sha256.rs from the 01.1 and 01.2 impl blocks, tests/sha256.rs from both test blocks, `pub mod sha256;` / `mod sha256;`, mod-line allows with reasons), session file text as now, with `sha256::Hmac::new(key)`. `cargo clippy -p skein-lib --all-targets -- -D warnings`.

- 01.2 skein-lib RFC_4231, rfc_4231_cases_1_to_7_give_their_published_macs (runs/skein-01.txt:37): **clean** (all four sketches together).

## The sketches: temper-a1

Placement notes (not defects). The workspace denies `allow_attributes` and `allow_attributes_without_reason`, so the
prescribed `#![allow(clippy::todo, unused, dead_code)]` needs `reason = ".."` and a first line
`#![allow(clippy::allow_attributes, clippy::allow_attributes_without_reason, reason = "..")]` in each sketch file.
Where a sketch redeclares a base item (Request, Method, Page, Document, Instance) it went into its own module (`sk01_*`,
`sk02_*`) and compiled alone. A test that calls `client::encode`/`decoder` on a new variant was compiled in a second
pass against the base crate with the sketched types and variants added to the real enums and `Request::X { .. } => todo!()`
arms in the exhaustive matches of temper-forge-forgejo (and the two `User` members added to the crate's literals); that
tree was then reverted. For 01.2's world test, `forgejo::profile_limits()` and `forgejo::setup_cases()` (both landed by 01.2
itself, session lines 71, 163) were stubbed in `tests/codecs/src/forgejo.rs`. For 02.1's conformance sketch `Recorder` was made
`pub(crate)` (01.2 does it, session 01 line 162).

### 01.1 temper-forge-forgejo request.rs: Method, Page, Request variants (runs/temper-01.txt:33)
compile — module sk01_request; `cargo clippy -p temper-forge-forgejo --all-targets -- -D warnings` — **clean**

### 01.1 temper-forge-forgejo document.rs: User, AccessToken (runs/temper-01.txt:34)
compile — module sk01_document; same command — **DEFECT: `#[derive(Clone, Default, ..)]` on `User` is refused by clippy.toml's disallowed macro `core::default::Default` ("only the derives in section 10.2"), at the session file's line 350** (also what 01.1's Shape at line 107 and the 20 `..Default::default()` literals at line 106 rest on). Clean once `Default` is dropped.

### 01.2 temper-throwaway-forgejo instance.rs: Instance::exchange_basic (runs/temper-01.txt:35)
compile — module sk01_instance; `cargo clippy -p temper-throwaway-forgejo --all-targets -- -D warnings` — **clean**

### 01.1 temper-forge-forgejo tests/accounts.rs: a_top_level_user_keeps_admin_flag_and_full_name (runs/temper-01.txt:36)
compile — second pass (User with `is_admin`, `full_name` added to the base); `cargo clippy -p temper-forge-forgejo --all-targets -- -D warnings` — **clean**

### 01.2 temper-codecs-world tests/forgejo_setup.rs: setup_requests_encode_as_the_tool_sent_them (runs/temper-01.txt:37)
compile — `cargo clippy -p temper-codecs-world --lib --test forgejo_setup -- -D warnings` (stubs of `profile_limits`, `setup_cases`; User literal in forgejo.rs adjusted) — **clean**

### 01.3 temper-forge-forgejo tests/tokens.rs: a_made_token_holds_its_value_and_shows_only_its_length (runs/temper-01.txt:38)
compile — second pass (`Request::CreateToken`, `Document::Token`, `AccessToken` added); same command as 01.1's test — **DEFECT: `token.value.as_ref().map(Secret::len)` uses the disallowed method `Option::map` ("no closures: match"), at the session file's line 298; `format!("{:?}", token.value)` is the disallowed macro `alloc::format` ("no formatting in step code"), at line 299** (the only two errors).

### 02.1 temper-forge-forgejo request.rs: Page, HookSpec, Request variants (runs/temper-02.txt:17)
compile — module sk02_request (names `TeamSpec` through sk02_document); `cargo clippy -p temper-forge-forgejo --all-targets -- -D warnings` — **clean**

### 02.1 temper-forge-forgejo document.rs: Access, Units, TeamSpec, Team, Org, Hook, access, access_text (runs/temper-02.txt:18)
compile — module sk02_document; same command — **clean**

### 02.1 forgejo-conformance src/orgs.rs: record, facts (runs/temper-02.txt:19)
compile — module sk02_orgs; `cargo clippy -p forgejo-conformance --all-targets -- -D warnings` (`Recorder` made `pub(crate)`) — **clean**

### 02.1 temper-forge-forgejo tests/orgs.rs: a_team_request_writes_its_booleans_and_four_units_as_literals (runs/temper-02.txt:20)
compile — second pass (TeamSpec, Units, Access, `Request::CreateTeam` added); `cargo clippy -p temper-forge-forgejo --all-targets -- -D warnings` — **clean**

### 02.2 temper-forge-forgejo tests/hooks.rs: a_hook_requests_debug_text_shows_no_secret_bytes (runs/temper-02.txt:21)
compile — second pass (HookSpec, `Request::CreateOrgHook` added); same command — **DEFECT: `format!("{:?}", Request::CreateOrgHook { .. })` is the disallowed macro `alloc::format` ("no formatting in step code"), at the session file's line 369** (the only error).

### 03.1 temper-forge-forgejo pin.rs (runs/temper-03.txt:39)
compile — `pub mod pin;`; `cargo clippy -p temper-forge-forgejo -p temper-throwaway-forgejo -p forgejo-conformance --all-targets -- -D warnings` — **clean**

### 03.1 temper-forge-forgejo tests/pin.rs (runs/temper-03.txt:40)
compile — `mod pin;` in tests.rs; same command — **clean**

### 03.2 temper-throwaway-forgejo cli.rs (runs/temper-03.txt:41)
compile — `mod cli;`; same command — **clean**

### 03.2 temper-throwaway-forgejo tests/throwaway.rs additions (runs/temper-03.txt:42)
compile — as the new test file tests/sk03_throwaway.rs (its own `use` lines); same command — **clean**

### 03.2 forgejo-conformance cli.rs (runs/temper-03.txt:43)
compile — `mod cli;`; same command — **clean**

### 03.3 forgejo-conformance registration.rs (runs/temper-03.txt:44)
compile — `mod registration;`; same command — **clean**

### 03.3 forgejo-conformance tests.rs: the_runner_configuration_names_one_connection_by_its_token_file (runs/temper-03.txt:45)
compile — `#[cfg(test)] mod sk03_tests;` with its `use crate::registration::configuration`; same command — **clean**

### 04.2 temper-forge-forgejo delivery.rs (runs/temper-04.txt:21)
compile — `pub mod delivery;`; `cargo clippy -p temper-forge-forgejo --all-targets -- -D warnings` — **clean**

### 04.2 temper-forge-forgejo client/delivery.rs (runs/temper-04.txt:22)
compile — `mod delivery;` in client.rs; same command — **clean**

### 04.2 temper-forge-forgejo server/delivery.rs (runs/temper-04.txt:23)
compile — `mod delivery;` in server.rs; same command — **clean**

## DEFECT lines
- 01-users-tokens.md:350 `User` derives `Default`: disallowed macro `core::default::Default` (also lines 106-107).
- 01-users-tokens.md:298 `Option::map` disallowed method in tokens.rs reference test.
- 01-users-tokens.md:299 `format!` disallowed macro in tokens.rs reference test.
- 02-orgs-hooks.md:369 `format!` disallowed macro in hooks.rs reference test.

## Recompiled after fix pass 5

Worktree temper-forge-setup-sketch2, placements as above (the sk01_*/sk02_* modules; the 01.3 and 02.2 tests as a second pass with `Request::CreateToken`, `Request::CreateOrgHook`, `HookSpec`, `Document::Token`, `AccessToken` added to the real types and `todo!()` arms in the exhaustive matches), session file text as now. `cargo clippy -p temper-forge-forgejo --all-targets -- -D warnings`.

- 01.1 document.rs: User, AccessToken (runs/temper-01.txt:34): **clean** (no `Default`).
- 01.3 tests/tokens.rs: a_made_token_holds_its_value_and_shows_only_its_length (runs/temper-01.txt:38): **clean**.
- 02.2 tests/hooks.rs: a_hook_requests_secret_shows_only_its_length (runs/temper-02.txt:21): **DEFECT: syntax error, "right curly brace `}` before `else` in a `let...else` statement not allowed" on `let Request::CreateOrgHook { hook, .. } = Request::CreateOrgHook { org: Box::from(*b"ensemble"), hook } else { panic!("a hook") };`, at the session file's line 369 (02-orgs-hooks.md)**. Clean once the initializer is parenthesized (`= (Request::CreateOrgHook { .. }) else`) or bound by a separate `let request = ...;` first.

Planner 3, after the recompile: 02-orgs-hooks.md's test now binds `let request = Request::CreateOrgHook { … };` and then
`let Request::CreateOrgHook { hook, .. } = request else { panic!("a hook") };`, the form the recompile found clean.

## The sketches: temper-a2

Placement: each sketch as a new module (`sk_*.rs`, or `#[test]` file) with `#![expect(clippy::todo, ..., reason = "sketch")]`
(the crates deny `allow_attributes`, so `expect`, trimmed to what fires). Commands are `cargo clippy -p <crate> --all-targets -- -D warnings`.

### 05.1 temper-fake-forge-protocol delivery.rs (runs/temper-05.txt:27)
compile — `cargo clippy -p temper-fake-forge-protocol --all-targets -- -D warnings` — **clean**

### 05.2 temper-engine-protocol-forge hooks.rs (runs/temper-05.txt:28)
compile — `cargo clippy -p temper-engine-protocol-forge --all-targets -- -D warnings` — **clean**

### 6.1 temper-fake-forge-domain api.rs: Change, Ended (runs/temper-06.txt:10)
compile (as `sk_api.rs`, beside `Check`/`JobRef` imported) — `cargo clippy -p temper-fake-forge-domain --all-targets -- -D warnings` — **clean**

### 6.1 temper-fake-forge-domain boundary.rs: Request (runs/temper-06.txt:11)
compile (as `sk_boundary.rs`; `Change`, `Ended` imported from the api sketch) — same command — **clean**

### 6.1 temper-fake-forge-domain scenario.rs: check (runs/temper-06.txt:12)
compile (as `sk_scenario.rs`) — same command — **clean**

### 6.1 temper-engine-forge-client-world translate.rs: hint (runs/temper-06.txt:13)
compile (as `sk_translate.rs`) — `cargo clippy -p temper-engine-forge-client-world --all-targets -- -D warnings` — **clean**

### 6.1 temper-engine-forge-world world.rs: impl World (runs/temper-06.txt:14)
compile (as `sk_world.rs`) — `cargo clippy -p temper-engine-forge-world --lib -- -D warnings` —
**DEFECT: `client::Hint` does not exist (`Hint` is `client::api::Hint`; the crate root has only `pub mod api`), at 06-settle.md:307**;
**DEFECT: `hooked`, `hints`, `wall` lack `#[must_use]` (clippy::must_use_candidate, pedantic denied by -D warnings), at 06-settle.md:306-308**.
With both fixed: clean.

### 6.1 temper-engine-forge-world tests/hooks.rs (runs/temper-06.txt:15)
compile (with the world sketch) — `cargo clippy -p temper-engine-forge-world --test sk_hooks -- -D warnings` — **clean**

### 6.2 temper-engine-forge-world tests/settle.rs (runs/temper-06.txt:16)
compile (with the world sketch; `changes::{change_row, change_step}` are 6.1's move from simulation.rs, stubbed with the base's signatures,
which match the sketch's use; 6.1's Files lists `src/changes.rs`, and the base's `change_row() -> top::ChangeRow`,
`change_step(&mut World, u64, change::Status, Box<[change::GateReport]>) -> change::Decision` agree) — same crate, `--test sk_settle` — **clean**

### 6.2 temper-engine-domain-forge settle.rs (runs/temper-06.txt:17)
compile (as `sk_settle.rs`) — `cargo clippy -p temper-engine-domain-forge --all-targets -- -D warnings` —
**DEFECT: `#[derive(Default)]` on `Settle` is a disallowed macro (clippy.toml: "only the derives in section 10.2; a default can be a live state"), at 06-settle.md:331**.
Without `Default`: clean (the rest, `fold` included, passes).

### 07.1 temper-forge-setup setup.rs (runs/temper-07.txt:17)
compile (new crate `crates/temper-forge-setup`, deps skein-lib, skein-json, jig-protocol-config, temper-forge-forgejo; `limits::{Limits, small}` and `problem::Problems` stubbed as the file's Shape says) — `cargo clippy -p temper-forge-setup --all-targets -- -D warnings` —
**DEFECT: `Setup::entry` lacks `#[must_use]` (must_use_candidate), at 07-prepare.md:257**. Otherwise clean.

### 07.1 temper-forge-setup check.rs (runs/temper-07.txt:18)
compile — same command — **clean**

### 07.1 temper-forge-setup step.rs (runs/temper-07.txt:19)
compile — same command —
**DEFECT: `Step { pub name, pub arguments, long }` mixes pub and private fields (clippy::partial_pub_fields, denied by the workspace), at 07-prepare.md:287**;
**DEFECT: `Step::new`, `is_long` and `steps` lack `#[must_use]` (must_use_candidate), at 07-prepare.md:289-290 and 293**. (A boxed_local on `new`'s arguments came of the `todo!()` body: not counted.)

### 07.1 temper-forge-setup tests/read.rs (runs/temper-07.txt:20)
compile (with `fixtures` stubs: `document`, `TWO`) — same command —
**DEFECT: `Ipv4Addr::new(127, 0, 0, 1)` is clippy::ip_constant (use `Ipv4Addr::LOCALHOST`), at 07-prepare.md:97**. Otherwise clean.

### 07.2 temper-forge-setup state.rs (runs/temper-07.txt:21)
compile — same command — **DEFECT: `encode` lacks `#[must_use]` (must_use_candidate), at 07-prepare.md:305**. Otherwise clean.

### 07.2 temper-forge-setup place.rs (runs/temper-07.txt:22)
compile — same command — **DEFECT: `Place::path` and `work_path` lack `#[must_use]` (must_use_candidate), at 07-prepare.md:320-321**. Otherwise clean.

### 07.2 temper-forge-setup tests/golden.rs (runs/temper-07.txt:23)
compile (with fixtures stubs `owned()`, `place()`, `secrets()`, a `APP_INI_WITH_RUNNER` const, and `ini::text` stubbed from 07.2's Shape) — same command — **clean**

### 07.4 temper-forge-setup spec.rs (runs/temper-07.txt:24)
compile — same command — **DEFECT: `command`, `admin_token`, `version` lack `#[must_use]` (must_use_candidate), at 07-prepare.md:337-339**. Otherwise clean.

### 07.4 temper-forge-setup prepare.rs (runs/temper-07.txt:25)
compile (with state.rs, place.rs, spec.rs) — same command — **DEFECT: `Prepare::new` lacks `#[must_use]` (must_use_candidate), at 07-prepare.md:382**. Otherwise clean.

### 07.4 temper-forge-setup tests/prepare.rs (runs/temper-07.txt:26)
compile (with fixtures stubs `Drive::{new, send}`, `first()`, `owned()`) — same command — **clean**

## DEFECT lines
- 06-settle.md:307 `client::Hint` should be `client::api::Hint`.
- 06-settle.md:306-308 `World::hooked`, `hints`, `wall` need `#[must_use]`.
- 06-settle.md:331 `derive(Default)` on `Settle` is disallowed by clippy.toml.
- 07-prepare.md:287 `Step` mixes pub and private fields (partial_pub_fields).
- 07-prepare.md:97 `Ipv4Addr::new(127, 0, 0, 1)` (ip_constant): use `Ipv4Addr::LOCALHOST`.
- 07-prepare.md:257, 289-290, 293, 305, 320-321, 337-339, 382 missing `#[must_use]` on pub fns/methods returning values (must_use_candidate): `Setup::entry`, `Step::new`/`is_long`/`steps`, `state::encode`, `Place::path`/`work_path`, `spec::{command, admin_token, version}`, `Prepare::new`.

## Recompiled after fix pass 5

Same placements (06's world and settle blocks and 07's seven src blocks and tests/read.rs replaced by the session files' current text, trimmed `#![expect]` to what fires). Commands: `cargo clippy -p temper-engine-forge-world --lib -- -D warnings`, `-p temper-engine-domain-forge --all-targets`, `-p temper-forge-setup --all-targets`, all `-- -D warnings`.

- 6.1 temper-engine-forge-world world.rs: impl World (06-settle.md:300-318): **clean**
- 6.2 temper-engine-domain-forge settle.rs (06-settle.md:322-381; `const NEW`, no Default): **clean**
- 07.1 setup.rs (07-prepare.md:236-258): **clean**
- 07.1 step.rs (07-prepare.md:282-294; private fields, accessors): **clean**
- 07.1 tests/read.rs (07-prepare.md:84-99; `Ipv4Addr::LOCALHOST`): **clean**
- 07.2 state.rs (07-prepare.md:297-307): **clean**
- 07.2 place.rs (07-prepare.md:310-323): **clean**
- 07.4 spec.rs (07-prepare.md:326-340): **clean**
- 07.4 prepare.rs (07-prepare.md:343-384): **clean**

Remaining DEFECT lines: none.

## The sketches: temper-b

Scratch: ~/src/rust/tmp/plan/temper-forge-setup-sketch. Placement notes (mine, not defects): the workspace and crate lints
deny `allow_attributes`/`allow_attributes_without_reason`, so each crate root (and each test root) carries
`#![allow(clippy::allow_attributes, clippy::allow_attributes_without_reason)]` besides the sketch's own allow; 11.2's `mod.rs`
block went in a module `support/landing.rs` (its `use` lines clash with mod.rs's); the base's test
`a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` was removed since 11.2's sketch redefines it; docs/examples
files for 14.1's include_bytes! are one-byte placeholders. `needless_pass_by_value` / `boxed_local` fired only on todo!() bodies
and were allowed (not counted). Where a ready-check item could be compiled against stubs of the earlier increment's names
(Entry, File, Step, Reply, 10.2's fns), I did so as a bonus and record what it found.

### 08.1 temper-forge-setup-shell process.rs/command.rs/prepare.rs (runs/temper-08.txt:14)
ready-check — grep of 07-prepare.md sketches and skein-io — **names agree**: Spec (spec.rs, with `output`), Ran, Failure, Prepare (prepare.rs), Place, File (place.rs), skein_io::{Limits, Error} at the base.

### 08.2 temper-forge-setup run.rs (runs/temper-08.txt:15)
ready-check — **names agree**: Place, Owned, Spec, Which, State all as 07's sketches give them; `runner: bool` against `Owned.runner: Option<Runner>`, `Command::Web|Daemon` exist.

### 08.2 temper-forge-setup-shell run.rs (runs/temper-08.txt:16)
ready-check — **names agree**: run::{Launch, Refusal} are the sketch above's, command::Error is 08.1's sketch 2.

### 08.3 temper-throwaway-forgejo Instance::attach (runs/temper-08.txt:17)
compile — `cargo clippy -p temper-throwaway-forgejo --all-targets -- -D warnings` — **DEFECT: 08-run.md:370, the doc comment has no `# Errors` section (clippy::missing_errors_doc: "docs for function returning Result missing # Errors section"); the crate's other pub fns return Result with one. All names resolve; nothing else fires once missing_errors_doc is allowed.**

### 09.1 temper-forge-setup reconcile.rs sketch A (runs/temper-09.txt:16)
ready-check — **names agree** (setup::Entry 07.1, place::File 07.2, skein_lib::{Queue, Secret}, temper_forge_forgejo::{document::{Document, Row}, request::Request} at the base). Bonus compile against stubs of Entry/File, as a pub module, with `cargo clippy -p temper-forge-setup --all-targets -- -D warnings`: **DEFECT: 09-reconcile-machine.md:282 and :296, clippy::large_enum_variant on `Event` and `Reply` (Document is 224 bytes at the base; the other variants 16); box Document or the variant. DEFECT: 09-reconcile-machine.md:340, `Reconcile::new` lacks #[must_use] (clippy::must_use_candidate).**

### 09.1 temper-forge-setup tests/reconcile.rs reference test (runs/temper-09.txt:17)
ready-check — **names agree**: File::AdminToken, Event/Need/Reply/Failure/Stage/Why::Unmarked and `Need::Send { request, token }`/`Need::Stop` as sketch A; fixtures `Run::{managed, step, forge, bot}`, `secret`, `ADMIN`, `admin() -> User`, `person(login, full_name) -> User` each with the signature the Test shape gives; `Request::User` and `Document::User(User)` exist at the base.

### 09.2 temper-fake-forge-protocol setup.rs sketch B (runs/temper-09.txt:18)
compile — `cargo clippy -p temper-fake-forge-protocol --all-targets -- -D warnings` (as `mod setup;`) — **clean**

### 09.2 temper-forge-setup-reconcile-world tests/reconcile.rs (runs/temper-09.txt:19)
ready-check — **names agree**: World::{managed, reconcile, store, file, secrets_absent}, Ended::Done are in 09.2's world text; `Store::find(&[u8]) -> Option<&Account>` (admin, full_name, id), `token_user(&[u8]) -> Option<u64>` as sketch B; bot `temper`, full name `temper:bot`, id 2 after `temper-admin` id 1.

### 10.1 temper-forge-setup-shell exchange.rs Next, Gather, authorization, pause (runs/temper-10.txt:16)
ready-check — **names agree** (Reply is 09.1's `reconcile::Reply`; skein_json::tokenizer::Request, temper_forge_forgejo::decode::Step and document::Row exist at the base). Bonus compile in a stub shell crate (own clippy.toml as crates/temper's) with the 09.1 sketch as its dependency: pause's body passes clippy; **DEFECT: 10-reconcile.md:233, clippy::large_enum_variant on `Next` (Reply is 224 bytes; same root as 09.1's Reply).**

### 10.1 temper-forge-setup-shell reconcile.rs Settings, Refused, bound, reconcile (runs/temper-10.txt:17)
ready-check — **names agree**: Entry 07.1, File 07.2, Failure 09.1 (`reconcile::Failure`); skein_http::client::Limits, temper_forge_forgejo::limits::Limits, skein_io::Limits, skein_shell::Clock at the base. Bonus compile: clean apart from the todo-body artifact.

### 10.1 temper reconcile_settings (runs/temper-10.txt:18)
ready-check — **names agree**: `temper_service::startup::Named` is at the base; `check::Checked` is 07.1's; 07.3's `setup::check(bytes) -> Result<(Named, Checked), String>` feeds it; forge.rs is 08.1's.

### 10.1 temper-forge-setup-shell tests/reconcile.rs reference test (runs/temper-10.txt:19)
ready-check — **names agree**: `reconcile(Settings, &Clock) -> Result<(), Refused>` and `Refused::line()` as the sketch; Peer and settings are 10.1's fixtures (support/mod.rs), named in the Test shape.

### 10.1 temper-forge-setup-shell tests/exchange.rs reference test (runs/temper-10.txt:20)
ready-check — **names agree**: `exchange::pause(backoff, cap, deadline, attempt: u32, began, now)`; bonus: compiled in the stub crate and `cargo nextest run -p temper-forge-setup-shell` passes (1 test).

### 10.2 temper binaries, tier_configuration (runs/temper-10.txt:21)
ready-check — **names agree** with 08.1's owned.rs (no clash with Names/document/configuration/prepare/files/scripted/finished) and with 13.1/14.1/11.1's uses (`tier_configuration(&Path) -> SocketAddr`, `binaries() -> (PathBuf, PathBuf)`). Bonus compile (appended to the 14.1 module): **DEFECT: 10-reconcile.md:322 and :325, `binaries` and `tier_configuration` lack #[must_use] (clippy::must_use_candidate).**

### 10.2 temper Throwaway::job_state (runs/temper-10.txt:22)
compile — `cargo clippy -p temper --all-targets -- -D warnings` — **DEFECT: 10-reconcile.md:331, `job_state` lacks #[must_use] (clippy::must_use_candidate; 11.1's methods have it).**

### 10.2 temper .forgejo/workflows/check.yaml (runs/temper-10.txt:23)
not Rust — `python3 -c 'import yaml,sys; yaml.safe_load(open(sys.argv[1]))' check.yaml` — **clean**: one job `check` on `temper-host`, `on: push: branches: ['temper/**']` only (PyYAML reads the key `on` as True, a YAML 1.1 quirk; Forgejo's parser is not affected).

### 11.1 temper support/webhooks.rs Delivery, receive (runs/temper-11.txt:16)
ready-check — **names agree**: the module is 05.3's (05-webhooks.md names `support/webhooks.rs`, "temper 11.1 extends it"), which gives `webhooks::signature(secret, body)`, `configuration`, `served`, `free_address`; no `Delivery`/`receive` there to clash.

### 11.1 temper support/forgejo.rs impl Throwaway { webhooks, secret, push } (runs/temper-11.txt:17)
compile — `cargo clippy -p temper --all-targets -- -D warnings` (appended to the base file) — **clean**

### 11.1 temper process.rs a_push_is_delivered_signed_with_the_secret_reconcile_drew (runs/temper-11.txt:18)
ready-check — **names agree**: `Throwaway::{webhooks, secret, push, stop}` (sketch above), `webhooks::{receive, signature}` (11.1 / 05.3), `support::WITHIN` and `skein_lib::bytes::find` at the base.

### 11.2 temper process.rs the two stories and fn landing (runs/temper-11.txt:19)
compile — `cargo clippy -p temper --all-targets -- -D warnings` (replacing the base test of the first story's name) — **clean**

### 11.2 temper support/mod.rs LANDING, RAISED_MS, raise_paces (runs/temper-11.txt:20)
compile — `cargo clippy -p temper --all-targets -- -D warnings` (as module support/landing.rs) — **clean**

### 12.1 temper-box limits.rs sketch 1 (runs/temper-12.txt:11)
compile — `cargo clippy -p temper-box --all-targets -- -D warnings` (new no_std crate, deps skein-lib) — **clean**

### 12.1 temper-box machine.rs sketch 2 (runs/temper-12.txt:12)
ready-check — **names agree**: `temper_forge_setup::Step` is re-exported at the root (07-prepare.md:60) with `Step::new(name, arguments, long)` and `is_long()`; skein_io::kernel::Exit at the base. Bonus compile against a Step stub: clean.

### 12.1 temper-box tests/machine.rs reference test (runs/temper-12.txt:13)
ready-check — **names agree**, with one gap: `use crate::Order;` needs a root re-export of `Order`, which no text of 12.1 states (lib.rs is listed only as a file; the sketch puts Order in machine.rs). Fixtures `Rig, FORGEJO, PREPARE` are 12.1's. Not counted as a defect (a one-line `pub use`), noted here.

### 12.1 temper-box tests/boundary.rs reference test (runs/temper-12.txt:14)
ready-check — **names agree**: jig-boundary's `manifest::manifests(&Path) -> Vec<CrateManifest>` with `name: Option<String>`, `dependencies: Vec<String>` (jig tests/boundary/src/manifest.rs:12-13,106).

### 12.2 temper-box host.rs sketch 3 (runs/temper-12.txt:15)
ready-check — **names agree**: Step (07.1), `limits::{Limits, Unfit}` (12.1 sketch 1), skein_io::{Limits, kernel::Fd} at the base; 13.1 calls `Service::new`, `Settings` as given. Bonus compile against the Step stub: clean.

### 12.2 temper-box tests/host.rs reference test (runs/temper-12.txt:16)
ready-check — **names agree**: Exit::Code(u8), Time::saturating_since, Duration ordering at the base; fixtures `Harness, once, long` are 12.2's.

### 13.1 temper box_command.rs run (runs/temper-13.txt:12)
compile — `cargo clippy -p temper --all-targets -- -D warnings` (as `mod box_command;`) — **clean**

### 13.1 temper tests/support/boxed.rs Plan, Outcome, run, status (runs/temper-13.txt:13)
compile — `cargo clippy -p temper --all-targets -- -D warnings` (`pub mod boxed;`) — **clean**

### 13.1 temper process.rs temper_box_prints_the_link_and_a_stop_leaves_no_process (runs/temper-13.txt:14)
ready-check — **names agree**: `owned::tier_configuration` (10.2) and `boxed::{run, Plan::Stop, status}` as sketched; `printed_link`, `Scratch::new`, `skein_lib::bytes::find` at the base. Bonus compile with 10.2's sketch D appended: clean.

### 13.2 temper process.rs temper_box_restarts_a_killed_forgejo_while_the_engine_serves (runs/temper-13.txt:15)
ready-check — **names agree**: `Plan::KillForgejo`, `forgejo_pids`, `engine_served`, `left` are in sketch 2. Bonus compile: clean.

### 14.1 temper tests/support/owned.rs EXAMPLE, SHIPPED, Owned, ... lay (runs/temper-14.txt:18)
compile — `cargo clippy -p temper --all-targets -- -D warnings` (module alone, placeholder example files) — **DEFECT: 14-example.md:251, :256, :261, :266, `Owned::example`, `Owned::tier`, `application`, `example_document` lack #[must_use] (clippy::must_use_candidate, 4 errors); nothing else fires.**

### 14.1 temper tests/startup.rs serve_on_the_owned_example_asks_only_for_the_files_reconcile_makes (runs/temper-14.txt:19)
compile — `cargo clippy -p temper --all-targets -- -D warnings` (with the owned.rs sketch) — **clean** (`Scratch::startup`, `prepare_in`, `fs` exist at the base)

### 15.1 temper-forge-setup upgrade.rs Course, course, backup_name (runs/temper-15.txt:6)
compile — `cargo clippy -p temper-forge-setup --all-targets -- -D warnings` (new no_std crate) — **clean**

## DEFECT lines
- 08-run.md:370 (08.3 attach): doc lacks `# Errors` (missing_errors_doc).
- 09-reconcile-machine.md:282, :296 (09.1 sketch A): large_enum_variant on Event and Reply (Document is 224 bytes).
- 09-reconcile-machine.md:340 (09.1 sketch A): Reconcile::new lacks #[must_use].
- 10-reconcile.md:233 (10.1 Next): large_enum_variant (Reply inside).
- 10-reconcile.md:322, :325 (10.2 binaries, tier_configuration): lack #[must_use].
- 10-reconcile.md:331 (10.2 job_state): lacks #[must_use].
- 14-example.md:251, :256, :261, :266 (14.1 owned.rs): four fns lack #[must_use].

Aside, found while stubbing 07's names (07-prepare.md, not my files): :287 `Step` mixes pub and private fields (clippy::partial_pub_fields), and its `new`/`is_long`; :320-321 `Place::path`/`work_path` lack #[must_use].

## Recompiled after fix pass 5

Same scratch worktree and placements, each replaced by the session file's current block; the extra allows I had added for the
defects (`missing_errors_doc`, `large_enum_variant`, `must_use_candidate`) were removed, so only `todo`, `unused`,
`dead_code` (and `needless_pass_by_value` on todo bodies) remain. Command per crate: `cargo clippy -p <crate> --all-targets -- -D warnings`.

- 08.3 temper-throwaway-forgejo Instance::attach (08-run.md:364-373): **clean**
- 09.1 temper-forge-setup reconcile.rs sketch A (09-reconcile-machine.md:256-347): **clean** (its `#[expect(clippy::large_enum_variant)]` on Event and Reply are fulfilled; `Reconcile::new` has `#[must_use]`)
- 10.1 temper-forge-setup-shell exchange.rs Next (10-reconcile.md:223-256): **clean** (against the 09.1 sketch as dependency)
- 10.2 temper binaries, tier_configuration (10-reconcile.md:321-326), appended to the 14.1 owned.rs module: **clean**
- 10.2 temper Throwaway::job_state (10-reconcile.md:331), in support/forgejo.rs, with the file's `must_use_candidate` allow removed: **clean**
- 14.1 temper tests/support/owned.rs (14-example.md:219-273, with 10.2's two fns): **clean**
- 12.1 temper-box tests/machine.rs reference test (12-box-machine.md:107-121): `use crate::machine::Order;` resolves (it is a unit test, `src/tests/machine.rs`, with the machine.rs sketch 2 at 304-349 and stub fixtures `Rig`, `FORGEJO`, `PREPARE`): **clean**. Placement note: `src/tests.rs` not `tests/mod.rs` (clippy::mod_module_files).

Remaining DEFECTs: none.
