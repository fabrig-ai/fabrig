# Base runs: temper 06, temper on skein's throwaway Forgejo

2026-10-19. Code: temper dd12b8ed9 (the code of 27995fb75, in
`/srv/data/git/runner/src/tmp/planning/temper-base`); skein 064c55f
(`skein-base`, read only: skein's throwaway does not exist there). Only this
file and `temper/06-throwaway.md` are deliverables. Every command ran once
; a block holds the command, its directory and the relevant
lines (at most 30). The scratch worktree is removed (last section).

## Discovery and corrections

The code graph (`srv-data-git-runner-src-temper`, `-skein`) answered nothing
for the throwaway's symbols: its temper index covers an older tree, and the
skein index has no throwaway. Every name below was read with `rg` and
`sed -n` in the base worktrees.

- The throwaway crate is 934 lines in 9 files (Cargo.toml, clippy.toml,
  `src/{binary,cli,http,instance,lib,tests}.rs`, `tests/throwaway.rs`). It holds
  13 running tests and 1 ignored one, `a_throwaway_forgejo_starts_serves_its_admin_and_is_removed`
  (needs `TEMPER_FORGEJO`); all 14 move to skein 06.1 under their names.
- temper's process-tier support does **not** hand paths to `Instance::attach`.
  `support/owned.rs::attach(directory, port, running)` reads the admin
  token file and calls `Instance::attach(child, port, scratch, token,
  &Settings::small())`; it never names a binary. The env reads
  (`TEMPER_FORGEJO`, `TEMPER_FORGEJO_RUNNER`) are in `binaries()`,
  `prepared_processes` and `reconcile_configuration`, and feed temper's own
  `forge prepare` and `forge reconcile`, not the throwaway. Skein's 06.1
  text agrees ("`support/owned.rs::attach` keeps its arguments"). The
  prompt's "hands the paths to skein's throwaway" is therefore not code:
  the change in owned.rs is its imports only; the login now comes from the API.
- The base `attach` hard-codes login `temper-admin`; no temper test reads
  `instance.admin().login` (`.admin()` is read for `.token` at
  owned.rs `assert_converged` and forgejo.rs `person_token`). The process
  test `a_prepared_forge_runs_forgejo_and_its_runner_once_each`'s check of
  `admin["login"] == "temper-admin"` (process.rs) reads the API of a forge
  made by temper's own `forge prepare`, not the throwaway's handle.
- The throwaway holds two temper words only: `APP_NAME = temper throwaway`
  in `instance.rs` and the `temper-admin` login in `attach`. No recorded
  corpus holds either (`rg -i 'temper throwaway'` outside the crate and
  `docs/development`: nothing). The git corpus holds 15 `temper`
  hits in 15 of its 169 files, all `refs/temper/fetched` (temper's own ref,
  handed by temper's cases; the last block below).
- What temper hands the kit in this session: the `TEMPER_FORGEJO` path to
  `Forgejo::from_path` (git-transcripts, conformance), and nothing from the
  process tier's `attach`; no user agent or word (the throwaway sends no
  User-Agent).
- `tools/forgejo-conformance` (18 files import the throwaway, 30 lines) is
  not in the README row, but temper 07 deletes it after this session, and it
  must build in between: it is repointed here (Choices, session file).
- `Cargo.toml` lists the crate as a workspace member and as a path
  dependency; three manifests use it: `crates/temper` (dev-dependencies),
  `tools/git-transcripts`, `tools/forgejo-conformance`.
- Both tool READMEs still name `crates/temper-forge-forgejo/src/pin.rs`
  (git-transcripts README line 10, conformance README line 7); temper 01.2
  deletes that file and its Files do not list the READMEs.
- skein 06.1 keeps `pub use skein_forge::forgejo::pin::{RUNNER_VERSION,
  names_runner}` in the kit ("re-exported", fake-forge.md 6), while temper
  01.2 removes the temper throwaway's re-export and repoints its "seven
  indirect user lines" to `skein_forge::forgejo::pin`. At the end of 01.2
  those five lines in owned.rs, process.rs, runner.rs and tests.rs no longer
  name the throwaway, so 06.1 does not change them.

## Commands at the base

```
$ cargo nextest list -p temper-throwaway-forgejo     # the crate skein 06.1 copies
dir: /srv/data/git/runner/src/tmp/planning/temper-base
temper-throwaway-forgejo tests::a_basic_request_records_its_scheme_and_holds_no_credential
temper-throwaway-forgejo tests::a_chunked_answer_is_read_whole_and_unframed
temper-throwaway-forgejo tests::a_fresh_secret_is_64_hexadecimal_digits_and_never_repeats
temper-throwaway-forgejo tests::a_range_is_sent_as_a_head_or_a_suffix
temper-throwaway-forgejo tests::a_recorded_request_holds_no_token
temper-throwaway-forgejo tests::an_output_shows_its_lengths_and_no_text
temper-throwaway-forgejo tests::the_app_ini_allows_the_webhook_host_it_is_given
temper-throwaway-forgejo tests::the_app_ini_names_loopback_sqlite_and_the_page_cap
temper-throwaway-forgejo::throwaway a_binary_printing_another_version_is_refused_naming_it
temper-throwaway-forgejo::throwaway a_failed_work_path_command_keeps_its_exit_code_and_both_streams
temper-throwaway-forgejo::throwaway a_scratch_directory_that_is_not_empty_is_refused
temper-throwaway-forgejo::throwaway a_work_path_is_refused_when_its_scratch_is_not_empty
temper-throwaway-forgejo::throwaway a_work_path_runs_commands_in_the_scratch_with_its_environment
(13 listed; the 14th, ignored, is not listed by default)
```

```
$ cargo nextest run -p temper-throwaway-forgejo
dir: temper-base
Starting 13 tests across 2 binaries (1 test skipped)
Summary [   0.020s] 13 tests run: 13 passed, 1 skipped
(slowest 0.019 s: a_failed_work_path_command_keeps_its_exit_code_and_both_streams)
```

```
$ cargo nextest run -p git-transcripts       # list showed the same 7
dir: temper-base
Starting 7 tests across 2 binaries
tests::a_redirect_to_the_old_name_fails_the_redirect_fact, a_renamed_fetch_exiting_non_zero_fails_the_redirect_fact,
a_renamed_fetch_warned_of_its_redirect_holds_the_fact, a_renamed_fetch_without_git_s_redirect_warning_fails_the_fact,
masking_rewrites_ids_ports_and_the_scratch_path, no_argument_holds_a_token_or_its_base64,
the_environment_is_given_whole_and_holds_the_header_only_for_remote_operations
Summary [   0.016s] 7 tests run: 7 passed, 0 skipped
```

```
$ cargo nextest run -p forgejo-conformance   # list showed the same 25 (all in src/tests.rs)
dir: temper-base
PASS [   0.110s] (25/25) forgejo-conformance tests::the_listener_holds_its_answer_until_the_probe_replies
Summary [   0.116s] 25 tests run: 25 passed, 0 skipped
```

```
$ cargo nextest run -p temper
dir: temper-base
PASS [   0.322s] (58/58) temper::forge reconcile_makes_a_missing_forge_directory_private
Summary [   0.369s] 58 tests run: 58 passed, 0 skipped
(the five test binaries forge, configuration, startup, process, live each
 compile support/{owned,forgejo}.rs through support/mod.rs; the default profile
 runs forge/configuration/startup and lib tests, 58 in all)
```

```
$ cargo nextest list -p temper --profile process
dir: temper-base
19 tests, all `temper::process`, among them a_prepared_forge_runs_forgejo_and_its_runner_once_each,
an_older_recorded_forgejo_is_dumped_then_upgraded, temper_box_restarts_a_killed_forgejo_while_the_engine_serves,
temper_sign_in_prints_the_visit_and_keeps_the_record
```

```
$ cargo check --workspace --all-targets     # the base form of the repin's second command
dir: temper-base
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.28s
(the repin's first command, `cargo update -p skein-lib`, is not run: skein 06.1
 has not landed, and an update at the base would change Cargo.lock for nothing)
```

```
$ scripts/check.sh > /tmp/claude-600/temper06-gate.out
dir: temper-base
Summary [   4.914s] 1879 tests run: 1879 passed, 1 skipped
Starting 50 tests across 26 binaries (122 binaries skipped via profile.fuzzy.default-filter)
SLOW [> 20.000s] temper-engine-domain-world::fuzzy_walking fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure
Summary [  23.448s] 50 tests run: 50 passed (1 slow), 0 skipped
(fmt and clippy -D warnings passed: exit 0)
```

```
$ TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 \
  TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 \
  scripts/proof-one-process.sh              (WS=/srv/data/git/runner/src)
dir: temper-base
Starting 19 tests across 1 binary (6 binaries skipped via profile.process.default-filter)
PASS [  83.157s] ( 1/19) a_change_lands_by_deliveries_with_every_poll_raised
PASS [  24.945s] ( 3/19) a_persons_change_lands_on_the_forge_and_is_found_after_a_restart
PASS [   2.131s] ( 4/19) a_prepared_forge_runs_forgejo_and_its_runner_once_each
PASS [   0.368s] ( 6/19) a_running_engine_lends_the_record_temper_sign_in_kept
PASS [   1.833s] ( 7/19) an_older_recorded_forgejo_is_dumped_then_upgraded
PASS [   6.116s] (12/19) temper_box_restarts_a_killed_forgejo_while_the_engine_serves
PASS [   0.077s] (19/19) temper_sign_in_prints_the_visit_and_keeps_the_record
Summary [ 145.248s] 19 tests run: 19 passed (8 slow), 0 skipped
(this run passed; base-runs.md's 145.7 s run passed too. base-temper-01.md records
 another run that failed at test 6 on a sign-in deadline: the tier is not a stable base)
```

```
$ TEMPER_FORGEJO=$WS/.../forgejo-17.0.0-linux-amd64 cargo run -p git-transcripts -- \
    --git /usr/bin/git --scratch target/git-transcripts-06 --out target/git-transcripts-06-out
dir: temper-base                                  (the recorder, outside the committed corpus)
Running `target/debug/git-transcripts --git /usr/bin/git --scratch target/git-transcripts-06 --out target/git-transcripts-06-out`
42 case directories + facts.txt written
$ diff -r target/git-transcripts-06-out/transcripts tools/git-transcripts/transcripts | grep -c '^diff -r'
9      (8 stderr files, 1 stdout file; byte counts of `POST git-upload-pack (351 bytes)` against 357,
        curl's "Failed to connect to 127.0.0.1 port 3000" against "127.0.0.1:3000")
$ diff target/git-transcripts-06-out/facts.txt tools/git-transcripts/facts.txt
2c2: < # git version 2.47.3   > # git version 2.53.0
```

The corpus was recorded with git 2.53.0, this machine has 2.47.3 (`git --version`),
so the committed corpus is no proof here: the Proof compares two recordings
on one machine. The same tree recorded twice (second run `target/debug/git-transcripts... --out target/git-transcripts-06b-out`, 15 s)
gives `diff -r target/git-transcripts-06-out target/git-transcripts-06b-out`: no output,
exit 0. The recording is deterministic on one machine. The output directories are removed.

## Users of what the session changes

```
$ bash -c 'rg -n "temper_throwaway_forgejo|temper-throwaway-forgejo" Cargo.toml crates tools tests scripts testing --glob "!testing/temper-throwaway-forgejo/**" | wc -l; rg -l... | wc -l'
dir: temper-base
46      lines in
27      files        (Cargo.toml 2, crates/temper/Cargo.toml 1, crates/temper/tests/{process 2, support/forgejo 2,
                      support/owned 5}, tools/git-transcripts {Cargo.toml 1, cases 1, lib 1}, tools/forgejo-conformance
                      {Cargo.toml 1 and 18 source files, 30 lines})
```

```
$ bash -c 'rg -n "Forgejo::from_env|Refusal::Unset|Instance::attach|owned::attach" crates tools tests testing -g "*.rs"'
dir: temper-base
testing/temper-throwaway-forgejo/tests/throwaway.rs:36   from_env (the ignored test; moves with skein 06.1)
tools/git-transcripts/src/lib.rs:76                      from_env
tools/forgejo-conformance/src/lib.rs:27                  from_env
tools/forgejo-conformance/src/component.rs:17            from_env
crates/temper/tests/support/owned.rs:402                 Instance::attach (the only caller)
crates/temper/tests/support/forgejo.rs:31, process.rs:272   owned::attach (2 callers)
3 temper callers of from_env besides the ignored test; 1 of Instance::attach; 2 of owned::attach
```

```
$ bash -c 'rg -n "TEMPER_FORGEJO" crates tools tests testing scripts.config -g "!*.md"'
dir: temper-base
15 lines in .rs files: owned.rs 5 (binaries(), prepared_processes, reconcile_configuration),
 throwaway crate 3 (binary.rs:25, throwaway.rs:34 and :36), conformance 7 (runner.rs 3, deliveries.rs 1,
 component/reads.rs 3, all TEMPER_FORGEJO_RUNNER); none in scripts/ or .config/
```

```
$ bash -c 'for r in skein-base jig smith hearth; do rg -l "temper_throwaway_forgejo|temper-throwaway-forgejo|TEMPER_FORGEJO|Forgejo::from_env|Instance::attach|skein_throwaway_forgejo" -g "*.rs" -g Cargo.toml -g "*.sh" $r | wc -l; done'
dir: /srv/data/git/runner/src/tmp/planning/skein-base, /srv/data/git/runner/src/{jig,smith,hearth}
0 0 0 0      (no other repository names the throwaway, its variables, from_env or attach)
```

```
$ bash -c 'rg -n "temper-throwaway-forgejo|temper_throwaway_forgejo|throwaway" docs/design docs/foundation AGENTS.md scripts.config'
dir: temper-base
only prose naming `skein-throwaway-forgejo` or "the throwaway Forgejo" (docs/design/domain/testing.md,
protocol/{setup,README,forge}.md, client/tools.md); no live document names the temper crate.
docs/development/plans/** (landed plans) name it: history, not edited.
```

## Sketches and the size

The sketches changed no signature of a moved crate: they are two call
sites (`git-transcripts::run`, `forgejo-conformance::{forgejo, run,
component::record}`) that read `TEMPER_FORGEJO` and call `Forgejo::from_path`,
which has the same signature in temper's crate and in skein 06.1's.
They compiled in the scratch worktree `scratch-temper-06` (a detached
worktree of dd12b8ed9) against the **temper** crate, since skein's cannot
exist at the base; the crate-name swap itself is mechanical (below).

```
$ cargo clippy -p git-transcripts --all-targets -- -D warnings
dir: scratch-temper-06 (with the sketch edit in lib.rs::run)
Checking git-transcripts v0.0.0... Finished `dev` profile in 3.22s ; exit 0
$ cargo clippy -p forgejo-conformance --all-targets -- -D warnings
dir: scratch-temper-06 (with lib.rs::forgejo and component.rs::record)
Checking forgejo-conformance v0.0.0... Finished `dev` profile in 1.97s ; exit 0
```

With the two refusal tests added (`tools/{git-transcripts,forgejo-conformance}/tests/refusal.rs`):

```
$ cargo clippy -p git-transcripts -p forgejo-conformance --all-targets -- -D warnings
Finished `dev` profile in 0.44s ; exit 0
$ cargo nextest run -p git-transcripts -p forgejo-conformance
Starting 34 tests across 6 binaries
PASS [   0.016s] ( 9/34) forgejo-conformance::refusal an_unset_forgejo_variable_is_refused_naming_it
PASS [   0.020s] (15/34) git-transcripts::refusal an_unset_forgejo_variable_is_refused_naming_it
Summary [   0.133s] 34 tests run: 34 passed, 0 skipped        (7 + 25 + 2)
```

The expected stderr, measured: `Error: "TEMPER_FORGEJO must name the pinned Forgejo"` and a newline, exit 1.

The size, measured by the mechanical swap in the same worktree (`sed` of
`temper_throwaway_forgejo` to `skein_throwaway_forgejo` over the 27 files,
the three manifest lines and the root `Cargo.toml` lines, `git rm -r
testing/temper-throwaway-forgejo`, then `git diff --cached --numstat`):

```
$ git diff --cached --numstat -- . ':!testing/temper-throwaway-forgejo'   (awk totals)
TOTAL added 56 deleted 49 in 27 files   (that includes the sketch edits)
  crates/temper: Cargo.toml 1/1, process.rs 2/2, support/forgejo.rs 2/2, support/owned.rs 5/5
  tools/git-transcripts: Cargo.toml 1/1, cases.rs 1/1, lib.rs 3/2
  tools/forgejo-conformance: Cargo.toml 1/1, lib.rs 10/3, component.rs 3/3, 16 other sources 1..5 each
  Cargo.toml (root): 1 added / 2 deleted
$ git diff --cached --numstat -- testing/temper-throwaway-forgejo | awk ...
934 lines deleted in 9 files
```

Of the 49 deleted lines, 45 are repointed one-for-one (path or crate name),
3 are the `from_env` calls replaced by the helper (4 added lines) and 1 is the
workspace member line. Of the 56 added, 7 are the conformance helper.
Five of the 45 are the pin lines that temper 01.2 has already repointed
(owned.rs `use`, process.rs 2, runner.rs `names_runner`, tests.rs
`RUNNER_VERSION`): 06.1 changes about 40. With the two refusal tests (about
20 lines) the new lines are about 31.

The scratch worktree was removed: `git -C temper-base worktree remove
--force .../scratch-temper-06`; `git worktree list | grep -c scratch-temper-06`
prints 0. The recorder's output directories under `temper-base/target` were
removed; `git status --short` there prints nothing.

## The Done-when commands at the base

Each ran once (task 751; the corpus count is task 761), in
`temper-base`. The expected result after the session is in the session file.

```
$ rg -n 'temper_throwaway_forgejo|temper-throwaway-forgejo' Cargo.toml crates testing tests tools -g '*.rs' -g Cargo.toml | wc -l
48      (46 outside the crate; its own two: Cargo.toml `name` and tests/throwaway.rs `use`)
$ python3 -I -c 'from pathlib import Path; print("throwaway_copy_absent=" + str(not Path("testing/temper-throwaway-forgejo").exists()))'
throwaway_copy_absent=False
$ rg -c 'skein-throwaway-forgejo' Cargo.toml crates/temper/Cargo.toml tools/git-transcripts/Cargo.toml
(no output, exit 1: the crate is not a dependency at the base)
$ rg -n 'Forgejo::from_env|SKEIN_FORGEJO|skein_throwaway_forgejo::(VERSION|RUNNER_VERSION|names_runner)' crates tools -g '*.rs'
tools/git-transcripts/src/lib.rs:76:    let forgejo = Forgejo::from_env().map_err(|e| format!("{e:?}"))?;
tools/forgejo-conformance/src/lib.rs:27:    let forgejo = Forgejo::from_env().map_err(|e| format!("{e:?}"))?;
tools/forgejo-conformance/src/component.rs:17:    let forgejo = Forgejo::from_env().map_err(|e| format!("{e:?}"))?;
$ cargo nextest run -p git-transcripts --test refusal       # cannot run at the base: the target does not exist
error: no test target named `refusal` in `git-transcripts` package      (exit 101)
(its base form is the package run above: 7 passed)
$ rg -n '#\[ignore' -g '*.rs' crates tools tests testing
testing/temper-throwaway-forgejo/tests/throwaway.rs:34:#[ignore = "needs TEMPER_FORGEJO"]
      (the only ignored test of the workspace: the gate's one skipped test is the throwaway's)
$ rg -o -I '[A-Za-z/_.-]*temper[A-Za-z/_.:-]*' tools/git-transcripts/transcripts tools/git-transcripts/facts.txt | sort | uniq -c
     15 refs/temper/fetched       (15 of 169 files hold it)
```
