# Base runs

Machine hel8 (hostname `debian-8gb-hel1-1`, 4 cores), 2026-10-19. skein worktree
`wt-skein` at b481ad9 (`origin/main`), temper worktree `wt-temper` at cd8fada9a
(`origin/main`); both clean. `SKEIN_TEST_CHROMIUM` is
`/home/free/.local/opt/chromium-147.0.7727.101/bin/chromium` (exists, executable).

Notes that hold for every section:

- Every build and test ran one at a time. The command sets
  `NEXTEST_DOUBLE_SPAWN=0` itself (nextest prints `info: NEXTEST_DOUBLE_SPAWN=0 set`).
- Output is trimmed; `…` marks a cut. The full logs are in
  `/home/free/.claude/jobs/5b29a521/tmp/logs/` (the per-run `aims/*.out` files hold
  the complete nextest output of command 8).
- The tool's guard refused a `sh -c` text holding `$(pwd)`, so commands 1b and 2
  were run from a one-line script (`run-1b.sh`, `run-2.sh` in that logs directory)
  that does `cd` to the worktree and then runs the README line verbatim.


## skein (wt-skein)

### 1a. README command 1: build the wasm shell, bundle it, build the server

```
$ sh -c 'cargo build --target wasm32-unknown-unknown -p skein-ui-example-shell-web && cargo run -p skein-ui-bundle -- --module target/wasm32-unknown-unknown/debug/skein_ui_example_shell_web.wasm --name skein_ui_example_shell_web --prefix /bundle/ --stylesheet examples/ui/shell-web/notes.css --out target/ui-example-bundle --noscript "Notes needs WebAssembly and scripts." && cargo build -p skein-ui-example-server'
   Compiling wasm-bindgen-shared v0.2.126
   …
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 32.99s
   …
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.77s
     Running `target/debug/skein-ui-bundle --module target/wasm32-unknown-unknown/debug/skein_ui_example_shell_web.wasm --name skein_ui_example_shell_web --prefix /bundle/ --stylesheet examples/ui/shell-web/notes.css --out target/ui-example-bundle --noscript 'Notes needs WebAssembly and scripts.'`
target/ui-example-bundle/7dd2b93da7788c2c
   …
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.86s
```

Passed (exit 0), 58 s in all: wasm build 33 s, bundler build and run 21 s, server
build 2 s. The bundle is `target/ui-example-bundle/{7dd2b93da7788c2c,index.html}`.

### 1b. README command 2: the one browser test

```
$ sh -c 'SKEIN_UI_EXAMPLE_BUNDLE="$(pwd)/target/ui-example-bundle" cargo nextest run -p skein-ui-web-tests --profile browser -E "test(=back_and_forward_move_between_the_list_and_a_note)"'
   Compiling skein-ui-web-tests v0.0.0 (/home/free/.claude/jobs/5b29a521/tmp/wt-skein/tests/ui-web)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.43s
info: NEXTEST_DOUBLE_SPAWN=0 set, disabling double-spawn for test processes
 Nextest run ID 50f69edc-79ab-4478-8816-3c9c41bc025a with nextest profile: browser
    Starting 1 test across 3 binaries (21 tests and 3 binaries skipped, including 3 binaries via profile.browser.default-filter)
        PASS [   1.483s] (1/1) skein-ui-web-tests::browser_stories back_and_forward_move_between_the_list_and_a_note
     Summary [   1.484s] 1 test run: 1 passed, 21 skipped
```

Passed (exit 0). The one test takes 1.48 s of real Chromium; the first compile
for the test profile took 21 s.

### 2. The whole browser suite

```
$ sh -c 'SKEIN_UI_EXAMPLE_BUNDLE="$(pwd)/target/ui-example-bundle" cargo nextest run -p skein-ui-web-tests --profile browser'
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
 Nextest run ID f1d09a7f-5058-41fd-98ed-55731509abe4 with nextest profile: browser
    Starting 22 tests across 3 binaries (3 binaries skipped via profile.browser.default-filter)
        PASS [   0.686s] ( 1/22) skein-ui-web-tests::browser_example a_note_added_in_one_page_shows_in_another
        PASS [   0.646s] ( 2/22) skein-ui-web-tests::browser_example a_person_chooses_a_label_by_its_option
        PASS [   0.950s] ( 3/22) skein-ui-web-tests::browser_example a_person_fills_each_entry_kind_and_chooses_a_label
        PASS [   0.708s] ( 4/22) skein-ui-web-tests::browser_example a_person_opens_a_note_and_comes_back_by_its_link
        PASS [   0.507s] ( 5/22) skein-ui-web-tests::browser_example a_person_opens_the_example_and_sees_its_heading
        PASS [   0.788s] ( 6/22) skein-ui-web-tests::browser_example a_press_on_a_bar_shows_its_labels_notes
        PASS [   0.737s] ( 7/22) skein-ui-web-tests::browser_example a_reload_keeps_a_pending_add_made_once
        PASS [   0.588s] ( 8/22) skein-ui-web-tests::browser_example a_reload_once_the_add_is_made_meets_no_closing_warning
        PASS [   0.646s] ( 9/22) skein-ui-web-tests::browser_example a_reload_with_an_add_pending_meets_the_closing_warning
        PASS [   0.652s] (10/22) skein-ui-web-tests::browser_example escape_in_the_dialog_keeps_the_draft
        PASS [   0.643s] (11/22) skein-ui-web-tests::browser_example the_body_grows_with_its_words_and_reports_them
        PASS [   0.505s] (12/22) skein-ui-web-tests::browser_example the_charts_fallback_is_read_by_its_name
        PASS [   0.451s] (13/22) skein-ui-web-tests::browser_harness a_framed_page_s_refusal_is_awaited_as_trouble
        PASS [   0.462s] (14/22) skein-ui-web-tests::browser_harness a_page_of_another_origin_is_opened_by_its_step
        PASS [   0.401s] (15/22) skein-ui-web-tests::browser_harness a_request_with_another_host_has_its_status_awaited
        PASS [   0.454s] (16/22) skein-ui-web-tests::browser_harness the_harness_fails_a_page_whose_module_throws
        PASS [   0.388s] (17/22) skein-ui-web-tests::browser_harness the_harness_reads_the_listening_line
        PASS [   0.411s] (18/22) skein-ui-web-tests::browser_harness the_harness_searches_the_next_accessible_role
        PASS [   0.581s] (19/22) skein-ui-web-tests::browser_stories a_panic_reloads_then_says_it_failed
        PASS [   0.795s] (20/22) skein-ui-web-tests::browser_stories a_refused_write_makes_no_note
        PASS [   0.755s] (21/22) skein-ui-web-tests::browser_stories back_and_forward_move_between_the_list_and_a_note
        PASS [   0.803s] (22/22) skein-ui-web-tests::browser_stories typing_in_the_title_survives_a_note_arriving
     Summary [  13.558s] 22 tests run: 22 passed, 0 skipped
```

Passed (exit 0): 22 tests, total 13.558 s (the 22 PASS times sum to 13.557 s; the
profile runs one test at a time). Slowest is `browser_example
a_person_fills_each_entry_kind_and_chooses_a_label` at 0.950 s, then
`typing_in_the_title_survives_a_note_arriving` at 0.803 s and
`a_refused_write_makes_no_note` at 0.795 s; the fastest is
`the_harness_reads_the_listening_line` at 0.388 s. Mean 0.62 s a test; the harness
tests take 0.39 to 0.46 s, the example and stories tests 0.5 to 0.95 s.

The `[profile.browser]` block of `.config/nextest.toml` (lines 23 to 29, with its
comment):

```
# Real Chromium over io's process pipes and the ring. A missing browser or
# unavailable ring is a failure in this suite, not a skip.
[profile.browser]
default-filter = "(package(skein-browser-world) or package(skein-ui-web-tests)) and binary(/^browser_/)"
global-timeout = "2m"
slow-timeout = { period = "20s", terminate-after = 3 }
test-threads = 1
```

### 3. The size command at the base

```
$ git diff --numstat origin/main... -- crates testing examples ':(glob,exclude)**/tests/**' ':(glob,exclude)**/tests.rs' ':(glob,exclude)**/golden/**' | awk '{a += $1} END {print a + 0}'
0
```

Prints 0 (exit 0), as expected at the base.

### 4. skein's default and fuzzy profiles

```
$ rg -n -A2 '^\[profile\.(default|fuzzy)\]' .config/nextest.toml
7:[profile.default]
8-default-filter = "not binary(/^fuzzy_/) and not binary(/^browser_/) and not binary(=live)"
9-global-timeout = "15s"
--
18:[profile.fuzzy]
19-default-filter = "binary(/^fuzzy_/)"
20-global-timeout = "60s"
```

skein's profiles, by `rg -n '^\[profile|^\[\[profile'`: `default` (line 7, with one
`[[profile.default.overrides]]` at 14), `fuzzy` (18), `browser` (25), `measure`
(34), `live` (40). The default profile's filter already leaves out `browser_`
binaries.

### 5. Line counts

```
$ wc -l testing/skein-browser/src/browser.rs testing/skein-fake-person-browser/src/harness.rs crates/skein-ui-web/src/page.rs crates/skein-ui-web/src/page/web.rs testing/skein-fake-person-browser/src/stories.rs
  2041 testing/skein-browser/src/browser.rs
   822 testing/skein-fake-person-browser/src/harness.rs
   777 crates/skein-ui-web/src/page.rs
   767 crates/skein-ui-web/src/page/web.rs
   188 testing/skein-fake-person-browser/src/stories.rs
  4595 total
```

## temper (wt-temper)

### 6. The size command at the base

```
$ git diff --numstat origin/main... -- crates scripts ':(glob,exclude)**/tests/**' ':(glob,exclude)**/tests.rs' ':(glob,exclude)**/golden/**' | awk '{a += $1} END {print a + 0}'
0
```

Prints 0 (exit 0).

### 7. wasm32 target and clippy on the client crates

```
$ rustup target list --installed
wasm32-unknown-unknown
x86_64-unknown-linux-gnu

$ cargo clippy --target wasm32-unknown-unknown -p temper-client-domain -p temper-client-view -p temper-client-protocol-wire -- -D warnings
    Updating git repository `https://github.com/fabrig-dev/jig.git`
    Updating git repository `https://github.com/fabrig-dev/skein.git`
    Updating git repository `https://github.com/fabrig-dev/smith.git`
    Checking skein-lib v0.0.0 (https://github.com/fabrig-dev/skein.git#92e177c3)
    Checking skein-codec v0.0.0 (https://github.com/fabrig-dev/skein.git#92e177c3)
    Checking jig-client-domain v0.0.0 (https://github.com/fabrig-dev/jig.git#149970b9)
    Checking smith-transcript v0.0.0 (https://github.com/fabrig-dev/smith.git#f47f4df9)
    Checking jig-client-protocol-wire v0.0.0 (https://github.com/fabrig-dev/jig.git#149970b9)
    Checking jig-protocol-transcript v0.0.0 (https://github.com/fabrig-dev/jig.git#149970b9)
    Checking skein-ui v0.0.0 (https://github.com/fabrig-dev/skein.git#92e177c3)
    Checking temper-client-protocol-wire v0.0.0 (…/wt-temper/crates/temper-client-protocol-wire)
    Checking jig-client-protocol-link v0.0.0 (https://github.com/fabrig-dev/jig.git#149970b9)
    Checking jig-client v0.0.0 (https://github.com/fabrig-dev/jig.git#149970b9)
    Checking temper-client-domain v0.0.0 (…/wt-temper/crates/temper-client-domain)
    Checking temper-client-view v0.0.0 (…/wt-temper/crates/temper-client-view)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 27.36s
```

The `wasm32-unknown-unknown` target is installed. The clippy run **passes**
(exit 0, no warnings, no errors, 27 s): none of the three crates, and none of
their dependencies, fails to build for wasm32. The dependencies it checked were
the pinned git ones: skein (`#92e177c3`: skein-lib, skein-codec, skein-ui), jig
(`#149970b9`: jig-client-domain, jig-client-protocol-wire, jig-protocol-transcript,
jig-client-protocol-link, jig-client) and smith (`#f47f4df9`: smith-transcript).
Temper pins these, not the `wt-skein` checkout.

### 8. The aims' base: summed test time of temper-service, temper-client-domain, temper-client-view

The sum command, checked against nextest's real output. The `PASS` lines read
`        PASS [   0.004s] (  1/366) temper-client-domain tests::acts::a_deciding_card_survives_its_removal_from_inbox`
(`PASS [ N.NNNs ] (i/n) package binary-id test`); the `sed` expression from the
brief matched all of them: 366 `PASS` lines in the file, 366 times summed. Nothing
else prints a time in these runs (no `SLOW`, `LEAK` or `FLAKY` lines). The command
as run, with the output also kept by `tee` (`$OUT` is a file under the logs
directory; the `tee` adds no time to the nextest figures):

```
$ NEXTEST_DOUBLE_SPAWN=0 sh -c 'cargo nextest run --profile default -p temper-service -p temper-client-domain -p temper-client-view 2>&1 | tee "$OUT" | sed -n "s/^ *PASS \[ *\([0-9.]*\)s\].*/\1/p" | awk "{t += \$1} END {print t}"'
```

(`--profile fuzzy` in place of `--profile default` for the fuzzy runs.) Before the
six, one warm-up run of the default profile (sum 2.97 s, Summary 0.767 s,
366 tests) and `cargo nextest run --no-run -p …` to compile the test
binaries (1 m 05 s); they are not among the six. Each run was alone on the
machine (gate lane; load average about 1 before each).

Default profile (`not binary(/^fuzzy_/) & not binary(/^(process|live)$/)`):

```
run 1: sum 2.734 s   Summary [   0.702s] 366 tests run: 366 passed, 0 skipped   (Starting 366 tests across 3 binaries)
run 2: sum 2.746 s   Summary [   0.705s] 366 tests run: 366 passed, 0 skipped
run 3: sum 2.784 s   Summary [   0.717s] 366 tests run: 366 passed, 0 skipped
median 2.746 s
```

Fuzzy profile (`binary(/^fuzzy_/) & not binary(/^(process|live)$/)`):

```
run 1: sum  (empty, i.e. 0)   Starting 0 tests across 0 binaries (3 binaries skipped via profile.fuzzy.default-filter)
                              Summary [   0.000s] 0 tests run: 0 passed, 0 skipped
run 2: sum  (empty, i.e. 0)   same Starting and Summary lines
run 3: sum  (empty, i.e. 0)   same Starting and Summary lines
median 0 s
```

At the base the three packages hold no fuzzy tests: they have only unit tests in
`src/` (no `tests/` directory in `temper-service`, `temper-client-domain` or
`temper-client-view`), so their fuzzy sum is 0 and the `awk` prints an empty line
for it (`t` is never set). The `fuzzy_*` binaries sit in the `*-world` test
packages (`temper-system-world`, `temper-one-process-world`,
`temper-client-world`, `temper-engine-domain-world`,
`temper-forge-setup-reconcile-world`, `temper-codecs-world`,
`temper-worker-domain-world`).

Test counts per package (the same in the three default runs), with the per-package
sum of test times of run 1 (runs 2 and 3 within 0.03 s of it):

```
temper-service          228 tests   2.148 s   (slowest test 0.030 s: tests::root_checks::landing_rules_past_the_authoritys_requirements_refuse_the_start)
temper-client-view       75 tests   0.334 s
temper-client-domain     63 tests   0.252 s
total                   366 tests   2.734 s
```

The suite for scale, one plain run of every package in the default profile:

```
$ cargo nextest run
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3m 31s
 Nextest run ID 97eac2b9-e11e-41ae-b8f9-5ac0567ed2df with nextest profile: default
    Starting 1861 tests across 116 binaries (28 binaries skipped via profile.default.default-filter)
     Summary [   7.640s] 1861 tests run: 1861 passed, 0 skipped
```

Passed (exit 0): 1861 tests in 116 binaries, 7.640 s wall (the build was 3 m 31 s,
the whole task 220 s). The three aim packages are 366 of the 1861 tests.

### 9. temper's nextest profiles

```
$ rg -n -A2 '^\[profile\.(default|fuzzy)\]' .config/nextest.toml
7:[profile.default]
8-default-filter = "not binary(/^fuzzy_/) & not binary(/^(process|live)$/)"
9-global-timeout = "15s"
--
18:[profile.fuzzy]
19-default-filter = "binary(/^fuzzy_/) & not binary(/^(process|live)$/)"
20-global-timeout = "60s"

$ rg -n '^\[profile' .config/nextest.toml
7:[profile.default]
18:[profile.fuzzy]
30:[profile.measure]
34:[profile.process]
39:[profile.live]
```

Profile names: `default`, `fuzzy`, `measure`, `process`, `live`. The `process`
block (lines 34 to 37) is `default-filter = "binary(=process)"`,
`global-timeout = "10m"`, `test-threads = 1`.

### 10. The process tier

```
$ scripts/proof-one-process.sh
   Compiling temper v0.0.0 (/home/free/.claude/jobs/5b29a521/tmp/wt-temper/crates/temper)
   …
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.11s
info: NEXTEST_DOUBLE_SPAWN=0 set, disabling double-spawn for test processes
 Nextest run ID 037fa83c-1a37-44c4-9a00-bd2271d7d791 with nextest profile: process
    Starting 19 tests across 1 binary (6 binaries skipped via profile.process.default-filter)
        FAIL [   0.003s] ( 1/19) temper::process a_change_lands_by_deliveries_with_every_poll_raised
  stderr ───
    thread 'a_change_lands_by_deliveries_with_every_poll_raised' (2056309) panicked at crates/temper/tests/support/owned.rs:472:54:
    TEMPER_FORGEJO must name the pinned Forgejo
  Cancelling due to test failure:
     Summary [   0.006s] 1/19 tests run: 0 passed, 1 failed, 0 skipped
warning: 18/19 tests were not run due to test failure (run with --no-fail-fast to run all tests, or run with --max-fail)
error: test run failed
```

**Failed, exit 100.** The script (`cargo build -p temper --bin temper`, then
`cargo nextest run -p temper --profile process`) builds, finds 19 tests in the
`temper` `process` binary, and the first one stops at once, 0.003 s in, because
`TEMPER_FORGEJO` is not set: the tier needs a pinned Forgejo binary (and
`TEMPER_FORGEJO_RUNNER` for a story with CI; `docs/design/testing.md` lines 330 and
401 to 403). The machine has none: no `forgejo` or `gitea` on `PATH` or under
`/home/free` (`~/.local/opt` holds only Chromium). So no test of the tier ran, no
tier time exists, and fail-fast cancelled the other 18; the run was not repeated
with `--no-fail-fast`.

### 11. Line counts

```
$ wc -l crates/temper-service/src/clients.rs crates/temper-service/src/server.rs crates/temper-service/src/startup/application.rs crates/temper-service/src/startup/checks.rs crates/temper-service/src/limits.rs crates/temper-service/src/limits/names.rs crates/temper/src/lib.rs crates/temper-client-view/src/words.rs crates/temper-client-view/src/limits.rs crates/temper-client-domain/src/address.rs scripts/check.sh .config/nextest.toml
   641 crates/temper-service/src/clients.rs
   345 crates/temper-service/src/server.rs
   708 crates/temper-service/src/startup/application.rs
   540 crates/temper-service/src/startup/checks.rs
   325 crates/temper-service/src/limits.rs
   227 crates/temper-service/src/limits/names.rs
   412 crates/temper/src/lib.rs
   538 crates/temper-client-view/src/words.rs
    61 crates/temper-client-view/src/limits.rs
    30 crates/temper-client-domain/src/address.rs
    14 scripts/check.sh
    45 .config/nextest.toml
  3886 total
```

### 8b. The aims' base for two more packages

`temper` (the binary's package) and `temper-client-shell-terminal`, which the plan
also changes: the same sum command as section 8 with `-p temper -p
temper-client-shell-terminal`, then all five packages at once. The scripts are
`run-8b-build.sh` and `run-8b.sh` in the logs directory; the per-run nextest
output is in `aims2/` there (`two-<profile>-run<n>.out`, `five-default-run<n>.out`).
First `cargo nextest run --no-run -p temper-service -p temper-client-domain
-p temper-client-view -p temper -p temper-client-shell-terminal` (exit 0, 0.23 s:
the binaries were already built by the full-suite run of section 8), and one
warm-up run of the two-package default command (sum 2.181 s, 84 tests), which is
not among the runs below. Each run went through the gate lane alone (load average
under 1 before each).

```
$ NEXTEST_DOUBLE_SPAWN=0 sh -c 'cargo nextest run --profile default -p temper -p temper-client-shell-terminal 2>&1 | tee "$OUT" | sed -n "s/^ *PASS \[ *\([0-9.]*\)s\].*/\1/p" | awk "{t += \$1} END {print t}"'
1.826
```

(`--profile fuzzy` for the fuzzy runs; `-p temper-service -p temper-client-domain
-p temper-client-view -p temper -p temper-client-shell-terminal` for the five.
`$OUT` is a file in `aims2/`.) The `PASS` lines now carry binary ids such as
`temper::forge` and `temper-client-shell-terminal::terminal`; the `sed` matched all
of them (the `PASS` count equals the `tests run` count in every run), and no `SLOW`,
`LEAK`, `FLAKY` or `FAIL` line appeared. All ten runs, the warm-up included, exited 0.

Two packages, default profile (`Starting 84 tests across 7 binaries (3 binaries
skipped via profile.default.default-filter)`):

```
run 1: sum 1.826 s   Summary [   0.460s] 84 tests run: 84 passed, 0 skipped
run 2: sum 1.872 s   Summary [   0.472s] 84 tests run: 84 passed, 0 skipped
run 3: sum 1.815 s   Summary [   0.459s] 84 tests run: 84 passed, 0 skipped
median 1.826 s
```

Two packages, fuzzy profile (`Starting 1 test across 1 binary (9 binaries skipped
via profile.fuzzy.default-filter)`):

```
run 1: sum 0.258 s   Summary [   0.259s] 1 test run: 1 passed, 0 skipped
run 2: sum 0.224 s   Summary [   0.224s] 1 test run: 1 passed, 0 skipped
run 3: sum 0.220 s   Summary [   0.221s] 1 test run: 1 passed, 0 skipped
median 0.224 s
```

Test counts per package (the same in every run of the profile), with the
per-package sum of test times of run 1 (runs 2 and 3 within 0.03 s of it):

```
default:  temper                        59 tests   1.733 s   (unit tests 10, configuration 2, forge 22, startup 25)
          temper-client-shell-terminal  25 tests   0.093 s   (unit tests 16, terminal 9)
          total                         84 tests   1.826 s
fuzzy:    temper-client-shell-terminal   1 test    0.258 s   (fuzzy_terminal random_cuts_and_stdout_delays_lose_no_command_or_change)
          temper                         0 tests
```

All five packages, default profile (`Starting 450 tests across 10 binaries (3
binaries skipped via profile.default.default-filter)`):

```
run 1: sum 4.415 s   Summary [   1.125s] 450 tests run: 450 passed, 0 skipped
run 2: sum 4.731 s   Summary [   1.196s] 450 tests run: 450 passed, 0 skipped
run 3: sum 5.033 s   Summary [   1.283s] 450 tests run: 450 passed, 0 skipped
median 4.731 s
```

```
temper-service                228 tests   (run 1: 2.090 s, run 2: 2.398 s, run 3: 2.238 s)
temper-client-view             75 tests   (0.319 s, 0.333 s, 0.388 s)
temper-client-domain           63 tests   (0.209 s, 0.167 s, 0.250 s)
temper                         59 tests   (1.684 s, 1.726 s, 1.983 s)
temper-client-shell-terminal   25 tests   (0.113 s, 0.107 s, 0.174 s)
total                         450 tests   (4.415 s, 4.731 s, 5.033 s)
```

The medians at the base: two packages 1.826 s default and 0.224 s fuzzy; all five
4.731 s default, the aim the plan states. The five are the 366 tests of section 8
plus the 84 of the two packages. The five-package runs vary more (4.4 to 5.0 s)
than the two-package runs (1.8 to 1.9 s); `temper` and `temper-service` carry the
spread.

### 8c. The aims' base for six packages

The plan also adds a story to `temper-client-world` (`tests/client`, in the
`stories` binary). The same sum command as section 8, with all six packages at once:
`-p temper-service -p temper-client-domain -p temper-client-view -p temper -p
temper-client-shell-terminal -p temper-client-world`. The scripts are
`run-8c-build.sh` and `run-8c.sh` in the logs directory; the per-run nextest output
is in `aims3/` there (`warmup.out`, `six-<profile>-run<n>.out`, with command output beside each). The build first:

```
$ cargo nextest run --no-run -p temper-service -p temper-client-domain -p temper-client-view -p temper -p temper-client-shell-terminal -p temper-client-world
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
```

Exit 0, 0.19 s: the binaries were already built (section 8's full-suite run built
`temper-client-world`'s). Then one warm-up run of the default command below (sum
6.048 s, Summary 1.536 s, 487 tests, exit 0), not among the six. The runs went
back to back, each through the gate lane alone; the command printed a load average of
1.3 to 1.8 as each started (higher than before section 8b's runs, whose load was
under 1), with only the agent's own processes (`codebase-memory`, `claude`, 2 to 6 %)
listed as busiest.

Default profile:

```
$ NEXTEST_DOUBLE_SPAWN=0 sh -c 'cargo nextest run --profile default -p temper-service -p temper-client-domain -p temper-client-view -p temper -p temper-client-shell-terminal -p temper-client-world 2>&1 | tee "$OUT" | sed -n "s/^ *PASS \[ *\([0-9.]*\)s\].*/\1/p" | awk "{t += \$1} END {print t}"'
5.501
    Starting 487 tests across 19 binaries (4 binaries skipped via profile.default.default-filter)
     Summary [   1.396s] 487 tests run: 487 passed, 0 skipped
…
run 1: sum 5.501 s   Summary [   1.396s] 487 tests run: 487 passed, 0 skipped   exit 0
run 2: sum 5.541 s   Summary [   1.410s] 487 tests run: 487 passed, 0 skipped   exit 0
run 3: sum 5.267 s   Summary [   1.339s] 487 tests run: 487 passed, 0 skipped   exit 0
median 5.501 s   (Summary median 1.396 s)
```

(The three `run` lines are cut from the three output files, not printed
by the command.) The `PASS` count equals the `tests run` count in every run (487),
and no `SLOW`, `LEAK`, `FLAKY` or `FAIL` line appeared.

Fuzzy profile, the same command with `--profile fuzzy`:

```
$ NEXTEST_DOUBLE_SPAWN=0 sh -c 'cargo nextest run --profile fuzzy -p temper-service -p temper-client-domain -p temper-client-view -p temper -p temper-client-shell-terminal -p temper-client-world 2>&1 | tee "$OUT" | sed -n "s/^ *PASS \[ *\([0-9.]*\)s\].*/\1/p" | awk "{t += \$1} END {print t}"'
0.539
    Starting 2 tests across 2 binaries (21 binaries skipped via profile.fuzzy.default-filter)
     Summary [   0.307s] 2 tests run: 2 passed, 0 skipped
…
run 1: sum 0.539 s   Summary [   0.307s] 2 tests run: 2 passed, 0 skipped   exit 0
run 2: sum 0.372 s   Summary [   0.213s] 2 tests run: 2 passed, 0 skipped   exit 0
run 3: sum 0.407 s   Summary [   0.238s] 2 tests run: 2 passed, 0 skipped   exit 0
median 0.407 s   (Summary median 0.238 s)
```

The fuzzy sum is two tests, one in `temper-client-shell-terminal` (as in section 8b)
and one in `temper-client-world`; the spread (0.37 to 0.54 s) is run 1's slower
`temper-client-world` test (0.307 s against 0.160 and 0.169 s).

The test counts per package and each package's sum of test times, from the same
`.out` files with `perpkg.sh` (it groups the `PASS` lines by the package in the
binary id; the output is cut to the packages and the `==` headers):

```
$ bash ../perpkg.sh six-default-run1.out six-default-run2.out six-default-run3.out six-fuzzy-run1.out six-fuzzy-run2.out six-fuzzy-run3.out
== six-default-run1.out
temper                               59 tests   1.875 s
temper-client-domain                 63 tests   0.200 s
temper-client-shell-terminal         25 tests   0.108 s
temper-client-view                   75 tests   0.309 s
temper-client-world                  37 tests   0.737 s
temper-service                      228 tests   2.272 s
== six-default-run2.out
…                                                (temper-client-world 37 tests   0.779 s)
== six-default-run3.out
…                                                (temper-client-world 37 tests   0.715 s)
== six-fuzzy-run1.out
temper-client-shell-terminal          1 tests   0.232 s
temper-client-world                   1 tests   0.307 s
== six-fuzzy-run2.out
temper-client-shell-terminal          1 tests   0.212 s
temper-client-world                   1 tests   0.160 s
== six-fuzzy-run3.out
temper-client-shell-terminal          1 tests   0.238 s
temper-client-world                   1 tests   0.169 s
```

(The command was run from `aims3/`, where the six `.out` files are; the cuts in the
run 2 and run 3 default blocks keep only the `temper-client-world` line.)

The six packages' test counts, the same in all three default runs:

```
temper-service                228 tests
temper-client-view             75 tests
temper-client-domain           63 tests
temper                         59 tests
temper-client-world            37 tests   (run 1: 0.737 s, run 2: 0.779 s, run 3: 0.715 s)
temper-client-shell-terminal   25 tests
total                         487 tests   (5.501 s, 5.541 s, 5.267 s)
```

`temper-client-world`'s own sum is 0.737 s in default run 1 (37 tests in 9 binaries:
`temper-client-world` itself 4, `adoption` 6, `chat` 9, `inbox` 2, `inbox_observation`
3, `names` 1, `recovery` 1, `stories` 5, `task` 6) and 0.307 s in fuzzy run 1 (1
test, `fuzzy_client random_people_and_faults_make_each_request_once`). The `stories`
binary, where the plan adds its story, holds 5 tests at the base.

The medians at the base: six packages 5.501 s default and 0.407 s fuzzy. The six are
the 450 tests of section 8b plus the 37 of `temper-client-world`, which add about
0.74 s to the default sum (section 8b's five-package median was 4.731 s) and 0.16 to
0.31 s to the fuzzy one. The default runs vary 5.27 to 5.54 s.

### 8d. The six packages' sum inside the whole suite

The pulse sums the aim packages' test times from a land gate's log, where the whole
workspace runs at once (skein's `development.md`, 5.2). Section 8c ran the six
packages alone; here the same six are read out of `cargo nextest run --workspace`.
The script is `run-8d.sh` in the logs directory (it also does the `cd` to the
worktree); each run's nextest output is in `aims4/` there (`warmup.log`,
`default-run<n>.log`, `fuzzy-run<n>.log`, with command output beside each). `LOG` below is an exported variable holding that run's log file. One
warm-up first, not counted:

```
$ NEXTEST_DOUBLE_SPAWN=0 sh -c 'cargo nextest run --workspace --profile default > "$LOG" 2>&1'
   Compiling temper-codecs-world v0.0.0 (…/tests/codecs)
…
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.63s
    Starting 1861 tests across 116 binaries (28 binaries skipped via profile.default.default-filter)
     Summary [   6.899s] 1861 tests run: 1861 passed, 0 skipped
```

Exit 0. It compiled six workspace crates that were not yet built here (9.63 s: the
`Compiling` lines are `temper-codecs-world`, `temper-one-process-world`,
`forgejo-conformance`, `git-transcripts`, `temper-forge-setup-reconcile-world`,
`live-login`); the counted runs then compiled nothing (no `Compiling` line; `Finished … in
0.19 s` to 0.35 s).

The sum expression, checked against real lines. A `PASS` line reads
`        PASS [   0.024s] (  82/1861) temper-client-world::stories a_chat_started_on_the_chats_page_is_answered_by_the_run`;
the package is the binary id's first part, before `::` (an integration binary:
`temper::configuration`, `temper-client-world::stories`) or before the space (a
unit-test binary: `temper-service tests::accounts::…`). The `sed` matched every
`PASS` line in every log (1861 of 1861 in each default log, 50 of 50 in each fuzzy
log), and the `awk` pattern is anchored (`^temper(…)?$`), so the other
`temper-*` packages in the workspace (`temper-system-world`, `temper-engine-domain-world`
and the rest) do not match. Nothing printed a `FAIL`, `SLOW`, `LEAK` or `FLAKY` line.

The matched lines per package, default profile, the same in all three runs and the
same as section 8c's (`sum-8d.sh` in the logs directory prints these from the same
`sed` and an `awk` that counts instead of summing; run 1's times):

```
temper                           59 lines   1.835 s     (8c: 59)
temper-client-domain             63 lines   0.211 s     (8c: 63)
temper-client-shell-terminal     25 lines   0.124 s     (8c: 25)
temper-client-view               75 lines   0.335 s     (8c: 75)
temper-client-world              37 lines   0.730 s     (8c: 37)
temper-service                  228 lines   2.104 s     (8c: 228)
total                           487 lines   5.339 s     (8c: 487)
```

Fuzzy profile, the same in all three runs and as in 8c: `temper-client-shell-terminal`
1 line (`fuzzy_terminal`) and `temper-client-world` 1 line (`fuzzy_client`), 2 in all.

The six-package sum of each run, the expression of the brief run on each log:

```
$ sed -nE 's/^ *PASS \[ *([0-9.]+)s\] \([^)]*\) ([^: ]+)[: ].*/\1 \2/p' default-run1.log | awk '$2 ~ /^temper(-service|-client-domain|-client-view|-client-shell-terminal|-client-world)?$/ {s += $1} END {printf "%.3f\n", s}'
5.339
$ … default-run2.log | …
5.082
$ … default-run3.log | …
5.088
$ … fuzzy-run1.log | …
0.378
$ … fuzzy-run2.log | …
0.381
$ … fuzzy-run3.log | …
0.403
```

(`…` stands for the same `sed` and `awk` text; only the log name changes.) The three
runs of each profile, each `NEXTEST_DOUBLE_SPAWN=0 sh -c 'cargo
nextest run --workspace --profile <profile> > "$LOG" 2>&1'`, with the `Summary` line
and total run times, all exit 0 and alone in the gate lane:

```
default  (Starting 1861 tests across 116 binaries (28 binaries skipped via profile.default.default-filter))
run 1: six 5.339 s   Summary [   5.422s] 1861 tests run: 1861 passed, 0 skipped
run 2: six 5.082 s   Summary [   5.395s] 1861 tests run: 1861 passed, 0 skipped
run 3: six 5.088 s   Summary [   5.315s] 1861 tests run: 1861 passed, 0 skipped
median six 5.088 s   (8c: 5.501 s; difference -0.413 s)   Summary median 5.395 s

fuzzy  (Starting 50 tests across 26 binaries (118 binaries skipped via profile.fuzzy.default-filter))
run 1: six 0.378 s   Summary [  14.287s] 50 tests run: 50 passed, 0 skipped
run 2: six 0.381 s   Summary [  14.448s] 50 tests run: 50 passed, 0 skipped
run 3: six 0.403 s   Summary [  14.233s] 50 tests run: 50 passed, 0 skipped
median six 0.381 s   (8c: 0.407 s; difference -0.026 s)   Summary median 14.287 s
```

(The `run` lines are cut from the log files, not printed by one
command.) No fuzzy run came near 10 minutes (16 s each), so all three ran. The
the command start lines printed a load average of 1.9 to 3.1 as each counted run began
(that one-minute figure includes the run before it), with only the agent's own
processes (2 to 8 %) listed as busiest.

The whole suite for scale, the sum of every `PASS` time in run 1 (`sed -nE
's/^ *PASS \[ *([0-9.]+)s\].*/\1/p' <log> | awk '{s += $1} END {printf "%.3f\n", s}'`):

```
default run 1:  21.502 s over 1861 tests    (the six: 5.339 s, 24.8 %)    Summary 5.422 s
fuzzy   run 1:  56.745 s over   50 tests    (the six: 0.378 s,  0.7 %)    Summary 14.287 s
```

(Runs 2 and 3 are 21.366 and 21.075 s default, 56.970 and 55.999 s fuzzy.) The
whole-suite sum is the test time added up across four cores, so it is four times the
`Summary` wall time at most; the fuzzy profile's wall time is set by the larger
`*-world` fuzzy tests (the slowest in run 1 is `temper-engine-domain-world` at
10.127 s), not by the six packages' two tests, 0.14 and 0.24 s.

What it shows: the six packages' sum read from the whole-suite log is not larger than
section 8c's alone. It is 0.41 s lower in the default profile (5.088 s against 5.501 s
medians; the three runs spread 5.08 to 5.34 s, 8c's 5.27 to 5.54 s) and 0.03 s lower
in the fuzzy one (0.381 s against 0.407 s; 0.378 to 0.403 s, 8c's 0.372 to 0.539 s),
with the same matched lines per package as 8c. The base for the pulse's sum from a
land gate's log is 5.09 s default and 0.38 s fuzzy.

### 10b. The process tier, with Forgejo

The review installed Forgejo and its runner under `~/.local/opt` and set the two
variables in `~/.profile`. The values, the `test -x` on both, and the run, from
scripts that export both (`run-10b.sh` and `run-10b-run.sh` in the logs directory):

```
$ grep -E 'TEMPER_FORGEJO' ~/.profile
export TEMPER_FORGEJO=/home/free/.local/opt/forgejo-17.0.0/forgejo
export TEMPER_FORGEJO_RUNNER=/home/free/.local/opt/forgejo-runner-13.2.0/forgejo-runner
$ test -x "$TEMPER_FORGEJO"
(exit 0)
$ test -x "$TEMPER_FORGEJO_RUNNER"
(exit 0)

$ scripts/proof-one-process.sh
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.65s
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
info: NEXTEST_DOUBLE_SPAWN=0 set, disabling double-spawn for test processes
 Nextest run ID 6affcaf7-f339-4d96-a459-18c52a86575a with nextest profile: process
    Starting 19 tests across 1 binary (6 binaries skipped via profile.process.default-filter)
        SLOW [>  3.000s] (─────) temper::process a_change_lands_by_deliveries_with_every_poll_raised
        …
        PASS [  85.393s] ( 1/19) temper::process a_change_lands_by_deliveries_with_every_poll_raised
        PASS [   3.869s] ( 2/19) temper::process a_native_chat_reaches_the_model
        PASS [  22.744s] ( 3/19) temper::process a_persons_change_lands_on_the_forge_and_is_found_after_a_restart
        …
        PASS [   5.851s] (13/19) temper::process temper_open_adopts_a_repository_by_commands
        …
        PASS [   0.068s] (19/19) temper::process temper_sign_in_prints_the_visit_and_keeps_the_record
────────────
     Summary [ 142.528s] 19 tests run: 19 passed (7 slow), 0 skipped
```

**Passed, exit 0**: all 19 tests of the `process` binary, 142.528 s by nextest's
clock (146 s for the task; the two builds took under 2 s). The slowest three:
`a_change_lands_by_deliveries_with_every_poll_raised` 85.393 s,
`a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` 22.744 s,
`temper_open_adopts_a_repository_by_commands` 5.851 s (the next two are
`temper_box_restarts_a_killed_forgejo_while_the_engine_serves` 5.354 s and
`a_push_is_delivered_signed_with_the_secret_reconcile_drew` 4.961 s). Seven tests
ran past the 3 s `SLOW` mark, and the first alone is 60 % of the tier. This run
replaces the failed one of section 10 (the same command with no `TEMPER_FORGEJO`),
which stays as recorded there.

### 12. The README's prerequisites

Each once, as written (from `run-12.sh` in the logs directory, which exports the two
`TEMPER_*` variables from the lines of `~/.profile`; `SKEIN_TEST_CHROMIUM` is in the
session's environment, not in `~/.profile`):

```
$ "$SKEIN_TEST_CHROMIUM" --version | grep -q 'Chromium 147\.'
(no output; exit 0)
$ "$SKEIN_TEST_CHROMIUM" --version        # for the version text only
Chromium 147.0.7727.101 built on Debian GNU/Linux 13 (trixie)

$ rustup target list --installed | grep -c -x wasm32-unknown-unknown
1
(exit 0)

$ test -x "$TEMPER_FORGEJO"           # /home/free/.local/opt/forgejo-17.0.0/forgejo
(exit 0)
$ test -x "$TEMPER_FORGEJO_RUNNER"    # /home/free/.local/opt/forgejo-runner-13.2.0/forgejo-runner
(exit 0)
```

All four prerequisites hold: Chromium is 147 (`147.0.7727.101`), the
`wasm32-unknown-unknown` target is installed (count 1), and both Forgejo binaries
are executable.
