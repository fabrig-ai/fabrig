# Base runs

At temper 27995fb75 and skein d120820, in clean detached worktrees, through
directly, with `NEXTEST_DOUBLE_SPAWN=0`. temper's run and skein's first run
went at the same time (load 12 on 8 cores); skein's fuzzy suite was then
run again alone, and that rerun is its base.

## temper, by package

```
$ cd ~/src/tmp/planning/temper-base && sh ~/src/tmp/planning/base-temper.sh
  # for each package p: cargo nextest run -p $p; cargo nextest run -p $p --profile fuzzy
== temper-forge-forgejo default             Summary [   0.062s] 84 tests run: 84 passed
== temper-engine-domain-forge-client default Summary [   0.051s] 76 tests run: 76 passed
== temper-engine-protocol-forge default     Summary [   0.149s] 117 tests run: 117 passed
== temper-forge-setup default               Summary [   0.140s] 98 tests run: 98 passed
== temper-fake-forge-domain default         Summary [   0.086s] 96 tests run: 96 passed
== temper-fake-forge-protocol default       Summary [   0.078s] 55 tests run: 55 passed
== temper-throwaway-forgejo default         Summary [   0.015s] 13 tests run: 13 passed, 1 skipped
== forgejo-conformance default              Summary [   0.143s] 25 tests run: 25 passed
== temper-engine-protocol-forge-world default Summary [   0.404s] 47 tests run: 47 passed
== temper-engine-protocol-forge-world fuzzy Summary [   7.709s] 6 tests run: 6 passed
== temper-engine-forge-client-world default Summary [   0.075s] 29 tests run: 29 passed
== temper-engine-forge-client-world fuzzy   Summary [   0.258s] 2 tests run: 2 passed
== temper-forge-setup-reconcile-world default Summary [   0.122s] 21 tests run: 21 passed
== temper-forge-setup-reconcile-world fuzzy Summary [   1.315s] 1 test run: 1 passed
== temper-fake-forge-tests default          Summary [   0.010s] 3 tests run: 3 passed
== temper-codecs-world default              Summary [   0.427s] 60 tests run: 60 passed
== temper-codecs-world fuzzy                Summary [   4.347s] 6 tests run: 6 passed
  # the crates above have no fuzzy tests where no fuzzy line is shown (0 tests run)
== workspace default                        Summary [   5.993s] 1879 tests run: 1879 passed, 1 skipped
== workspace fuzzy                          Summary [  22.803s] 50 tests run: 50 passed (1 slow)
```

The package names of the worlds: `tests/engine/protocol-forge` is
temper-engine-protocol-forge-world, `tests/engine/forge/client`
temper-engine-forge-client-world, `tests/forge-setup/reconcile`
temper-forge-setup-reconcile-world, `tests/fake-forge`
temper-fake-forge-tests, `tests/codecs` temper-codecs-world and
`tools/forgejo-conformance` forgejo-conformance.

The fuzzy tests over a second (the per-test lines of the same run):

```
PASS [   7.481s] temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_3
PASS [   7.544s] temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_1
PASS [   7.635s] temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once
PASS [   7.709s] temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_2
PASS [   1.315s] temper-forge-setup-reconcile-world::fuzzy_reconcile random_initial_forges_converge_and_a_second_run_writes_only_hook_patches
PASS [   3.079s] temper-codecs-world::fuzzy_forgejo forgejo_delivery_decoders_answer_every_mutated_delivery_without_panicking
PASS [   3.702s] temper-codecs-world::fuzzy_forgejo forgejo_decoders_answer_every_mutated_transcript_without_panicking
PASS [   4.347s] temper-codecs-world::fuzzy_forgejo forgejo_server_routes_answer_every_mutated_request_without_panicking
```

The four `fuzzy_machine` tests are over the fuzzy ceiling's 5 s aim at
the base. They move as they are in skein 03.4 (decisions.md).

## skein, the workspace

```
$ cd ~/src/tmp/planning/skein-base && sh ~/src/tmp/planning/base-skein.sh
  # three times: cargo nextest run --workspace; cargo nextest run --workspace --profile fuzzy
     Summary [  11.476s] 2578 tests run: 2578 passed (1 slow), 0 skipped
     Summary [  10.311s] 2578 tests run: 2578 passed, 0 skipped
     Summary [  11.037s] 2578 tests run: 2578 passed (1 slow), 0 skipped
  # (the fuzzy runs of this pass hit the 60 s budget under temper's run's load; rerun alone below)
```

The default suite's times are under load; temper's last gate gives skein
6.66 s alone (findings/survey-06-tests.md).

```
$ cd ~/src/tmp/planning/skein-base && sh -c 'for i in 1 2 3; do NEXTEST_DOUBLE_SPAWN=0 cargo nextest run --workspace --profile fuzzy --no-fail-fast 2>&1 | grep -E "Summary|FAIL|TIMEOUT|SLOW|TERMINATING"; done'
     Summary [  26.158s] 104 tests run: 104 passed (2 slow), 0 skipped
     Summary [  35.774s] 104 tests run: 104 passed (5 slow), 0 skipped
     Summary [  39.266s] 104 tests run: 104 passed (5 slow), 0 skipped
  # slow (> 20 s): skein-echo-world::fuzzy_scenarios (2), skein-io-world::fuzzy_scenarios (2), skein-io-world::fuzzy_cuts (1)
```

## temper, the process tier

```
$ cd ~/src/tmp/planning/temper-base && TEMPER_FORGEJO=$HOME/src/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$HOME/src/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 scripts/proof-one-process.sh
     Summary [ 145.694s] 19 tests run: 19 passed (8 slow), 0 skipped
```

On the run's machines the binaries are under `$HOME/src/rust/tmp/temper/forgejo/` (README, Rules).

### The process tier again, during drafting (10-19)

Two reruns at the same base failed while drafting agents ran beside
them, the temper 06 drafter among them, on the same tier:

- the temper 01 draft's, at about 01:45Z: 5 passed, then
  `a_running_engine_lends_the_record_temper_sign_in_kept` missed the
  sign-in world's 30 s deadline; 154.170 s, exit 100
  (`findings/base-temper-01.md`).
- the planner's, the gate at load 1.56, 02:05Z to 02:10Z (log
  `~/src/tmp/logs/20261019T020951-750.log`): the first test,
  `a_change_lands_by_deliveries_with_every_poll_raised`, failed at
  5.229 s after its repository stage: skein-world's "binary exited
  while tree processes still ran".

Three later runs at the same base, each in the isolated run (alone in
the queue), passed: 19 of 19 in 149.083 s (`findings/base-temper-02.md`),
145.248 s (`base-temper-06.md`) and 145.440 s (`base-temper-05.md`). The
two failures came with drafting agents' builds beside them, before the
gate lane held the queue; the tier is load-sensitive, and each session runs
it once, as its Proof says. Its flakiness under load
goes to the run's supervisor as a known risk (README, Rules).
