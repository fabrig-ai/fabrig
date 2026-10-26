# The suites' times at the base

2026-10-19, on debian-16gb-hel1-1 (8 vCPU), the run's machine: temper at
`cd4727819` (`origin/main`, past the tools plan's temper-repin
`8c02b3b9e`), in a detached worktree, through the gate lane, alone (load
6.7 at the start, 9.5 at the end: two planning sub-agents were reading).
Three runs of each suite, after one build of each:

```
$ cd ~/src/rust/tmp/planning/real-loop-base && sh -c 'export NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3; do cargo nextest run --workspace >focused-$i.log 2>&1; grep Summary focused-$i.log; cargo nextest run --workspace --profile fuzzy >fuzzy-$i.log 2>&1; grep Summary fuzzy-$i.log; done'
     Summary [   3.580s] 1861 tests run: 1861 passed, 0 skipped
     Summary [   8.653s] 50 tests run: 50 passed, 0 skipped
     Summary [   2.834s] 1861 tests run: 1861 passed, 0 skipped
     Summary [   9.841s] 50 tests run: 50 passed, 0 skipped
     Summary [   3.016s] 1861 tests run: 1861 passed, 0 skipped
     Summary [   7.996s] 50 tests run: 50 passed, 0 skipped
```

| Suite | Median | Swing |
|---|---|---|
| temper focused | 3.02 s | 0.75 s |
| temper fuzzy | 8.65 s | 1.85 s |

## The packages this plan changes

forge-kit lands on the same `main` during the run, so the aims are
measured as the summed test times of the packages this plan's sessions
change (development.md, 5.2, "Pulse"), from the same logs: each test's
time from its `PASS [ … ]` line, summed by package (the first word after
the test's count, before `::` or a space):

```
$ for k in focused fuzzy; do for i in 1 2 3; do sed -nE 's/^ *PASS \[ *([0-9.]+)s\] \([^)]*\) ([^: ]+)[: ].*/\1 \2/p' $k-$i.log | awk -v run=$k-$i '{sum[$2]+=$1; n[$2]++} END {for (p in sum) printf "%s %s %.3f %d\n", run, p, sum[p], n[p]}'; done; done
```

| Package | Tests | Focused, three runs | Median | Fuzzy median |
|---|---|---|---|---|
| `temper` (`crates/temper`) | 59 | 2.270, 1.952, 1.804 s | 1.95 s | none |
| `temper-client-tools` | 224 | 1.330, 1.495, 1.582 s | 1.50 s | none |
| `temper-client-world` (`tests/client`) | 37, 1 fuzzy | 2.644, 0.736, 0.789 s | 0.79 s | 0.13 s |
| `temper-engine-domain-world` (`tests/engine/domain`) | 201, 7 fuzzy | 6.559, 4.980, 5.666 s | 5.67 s | 11.72 s |
| `temper-client-shell-cli` (new, temper 05) | | | 0 | 0 |
| `temper-real-loop-world` (new, temper 08) | | | 0 | 0 |
| Sum | | | 9.90 s | 11.86 s |

The client world's first run (2.64 s) is a cold start of its binaries;
the other two agree. The engine's domain world's fuzzy time is mostly
one test's (`findings/fuzzy-walking.md`).

## What a story through the whole service costs today

The nearest thing to a real-loop story on `main` is the one-process
world's (`tests/one-process`, simulated, `World::new_controlled`), the
same three runs:

| Story | Times |
|---|---|
| `a_person_signs_in_by_the_printed_link_and_a_chat_is_answered` | 0.349, 0.241, 0.228 s |
| `a_signal_mid_run_drains_and_the_next_start_settles_the_claims` | 0.334, 0.251, 0.263 s |
| `an_adopted_repository_gets_its_change_pushed_through_git` | 0.936, 0.955, 0.876 s |

So a sign-in and a chat to its reply, or a drain, costs about a quarter
of a second through the whole service; an adoption pushed through fake
git, about 0.9 s, near the real-loop story's 1 s aim (testing.md 6.1).
The real loop adds the real kernel's loopback and a hosted program per
`temper ctl` call: temper 08.2 measures what that costs.

## The process tier

Once, at the same commit, through the gate lane, alone (load 5.3 at the
start), with `TEMPER_FORGEJO` (Forgejo 17.0.0) and
`TEMPER_FORGEJO_RUNNER` (runner 13.2.0) set:

```
$ cd ~/src/rust/tmp/planning/real-loop-base && sh -c 'export TEMPER_FORGEJO=… TEMPER_FORGEJO_RUNNER=… NEXTEST_DOUBLE_SPAWN=0; scripts/proof-one-process.sh >process-1.log 2>&1; grep Summary process-1.log'
     Summary [ 152.878s] 19 tests run: 19 passed (8 slow), 0 skipped
```

The tier runs one test at a time (`threads 1`), so its time is the sum of
its stories'. The largest:

| Story | Time |
|---|---|
| `a_change_lands_by_deliveries_with_every_poll_raised` | 86.39 s |
| `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` | 23.78 s |
| `a_push_is_delivered_signed_with_the_secret_reconcile_drew` | 7.52 s |
| `temper_open_adopts_a_repository_by_commands` | 6.83 s |
| `temper_box_restarts_a_killed_forgejo_while_the_engine_serves` | 6.10 s |
| `a_native_chat_reaches_the_model` | 4.83 s |
| the other 13 | 17.42 s in all |

The tools plan's run measured 150.9 s at 09:05Z, with the same 19. The
tier's aim is 3 min (testing.md 2.6): this plan may add about 27 s.
