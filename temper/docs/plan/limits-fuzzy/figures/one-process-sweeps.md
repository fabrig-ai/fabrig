# one-process-sweeps: the two crash sweeps

2026-11-07 UTC. Base `0a921f03b1`; final code `7a3a816e7f`. 16 CPUs, AMD EPYC-Genoa Processor, Valgrind 3.24.0. Test builds and existing Cargo profiles are unchanged.

Instruction counts come first: each profile ran one exact test on its unoptimized test executable and finished with **1 passed**. Each sweep still draws seeds `1..=4`, with one armed crash and a full cold readback per seed. Native CPU is user plus system, including waited-for descendants. There is no fork. One run per revision and arm follows `temper/docs/plan/limits-fuzzy/notes/measuring.md`; CPU samples are observations, not a variance-controlled speedup estimate.

| Sweep | Crash seeds | Arm | Run | Ir | Native CPU s | Native wall s | Result |
|---|---:|---|---:|---:|---:|---:|---|
| Kill | 4 | base | 1 | 14,203,061,119 | 5.627755 | 5.841912 | 1 passed |
| Kill | 4 | final | 1 | 11,290,382,441 | 4.658312 | 4.918599 | 1 passed |
| Power loss | 4 | base | 1 | 13,925,573,290 | 5.190778 | 5.377810 | 1 passed |
| Power loss | 4 | final | 1 | 11,028,384,918 | 4.572799 | 4.579536 | 1 passed |

- kill: Ir falls 20.51%; CPU samples 5.611947 + 0.015808 → 4.598386 + 0.059926 s. Load averages: (28.2724609375, 27.9814453125, 25.22265625) → (28.62353515625, 28.5517578125, 25.24462890625).
- power: Ir falls 20.80%; CPU samples 5.131023 + 0.059755 → 4.556825 + 0.015974 s. Load averages: (14.22412109375, 24.46240234375, 24.66259765625) → (28.5732421875, 28.54248046875, 25.259765625).

## Reproduce

At each recorded revision, from its slot, build and locate the executable:

```sh
mkdir -p target/limits-fuzzy/one-process-sweeps
heavy cargo nextest list -p temper-one-process-world --test suite --message-format json
```

Use that JSON suite's `binary-path` and `cwd` as `<binary>` and `<cwd>` below. Use either exact name:

```text
fuzzy::one_process::drawn_kills_keep_the_conformance_rules_and_answer_adoptions_once
fuzzy::one_process::drawn_power_losses_keep_the_conformance_rules
```

```sh
heavy bash -lc 'cd <cwd> && CARGO_MANIFEST_DIR=<cwd> valgrind --tool=callgrind \
  --callgrind-out-file=<slot>/target/limits-fuzzy/one-process-sweeps/<label>.%p.out \
  <binary> --exact <test> --nocapture'
callgrind_annotate --inclusive=yes --auto=no <profile>
callgrind_annotate --inclusive=no --auto=no <profile>
callgrind_annotate --inclusive=yes --tree=calling --threshold=100 --auto=no <profile>
```

GNU time was absent. For native CPU, recreate this untracked wrapper under `target/limits-fuzzy/`, and run it inside the same `heavy bash -lc` command in place of Valgrind:

```python
import os, resource, subprocess, sys, time
before = resource.getrusage(resource.RUSAGE_CHILDREN)
started = time.monotonic()
code = subprocess.run(sys.argv[1:]).returncode
used = resource.getrusage(resource.RUSAGE_CHILDREN)
print("user", used.ru_utime - before.ru_utime, "system", used.ru_stime - before.ru_stime,
      "wall", time.monotonic() - started, "load", os.getloadavg())
sys.exit(code)
```

Use `python3 <wrapper> <binary> --exact <test> --nocapture`. Every test execution went through `heavy`; no build settings, seeds, deadlines or budgets changed. The temporary wrappers, diagnostics, source copies and raw profiles are removed before push.

## Cuts and retained checks

A temporary `eprintln!` in `world::crash`, removed before commit, recorded both revisions. Every row had window `(449, 585)` before and after; the existing focused adoption test pins those boundaries.

| Sweep | Seed | Base `before` | Final `before` |
|---|---:|---:|---:|
| Kill | 1 | 526 | 526 |
| Kill | 2 | 529 | 529 |
| Kill | 3 | 464 | 464 |
| Kill | 4 | 507 | 507 |
| Power loss | 1 | 526 | 526 |
| Power loss | 2 | 529 | 529 |
| Power loss | 3 | 464 | 464 |
| Power loss | 4 | 507 | 507 |

Call-graph counts also match; these are counts, not sampled time:

| Sweep | Observation | Base calls | Final calls |
|---|---|---:|---:|
| kill | Referee/forge observations | 113,903 | 113,903 |
| kill | Full durable/cold snapshots | 128 | 128 |
| kill | Root checks | 12 | 12 |
| kill | Service starts | 16 | 16 |
| kill | Cold restarts | 4 | 4 |
| power | Referee/forge observations | 115,378 | 115,378 |
| power | Full durable/cold snapshots | 128 | 128 |
| power | Root checks | 12 | 12 |
| power | Service starts | 16 | 16 |
| power | Cold restarts | 4 | 4 |

The sweep test file and `oracle.rs` have empty diffs. `world::crash`, its signature, phases and readers are unchanged. All phases still run to their original endings: no setup tail is cut, and no tail assertion needs another holder. All five `READS` documents (`me`, `inbox`, project, settings and connected forge list) are read from the live service before shutdown, read again after a cold start and compared: 40 document pairs, 80 public document reads across the two sweeps. Full durable snapshots, per-step forge observations and all conformance events remain.

## kill: largest inclusive work functions

Enclosing test/runtime, generic dispatch and world-loop frames are omitted. These selected work functions are sorted by base Ir; nested shares overlap and must not be added.

| Function | Base Ir (share) | Final Ir (share) | Calls base → final |
|---|---:|---:|---:|
| `Person::iterate` | 3,234,053,107 (22.77%) | 3,234,180,099 (28.65%) | 22,747 → 22,747 |
| `service::iterate_stage` | 2,493,905,491 (17.56%) | 2,493,753,519 (22.09%) | 128,355 → 128,355 |
| `programs::service (startup)` | 2,107,087,395 (14.84%) | 648,159,874 (5.74%) | 16 → 16 |
| `engine::iterate` | 1,893,687,254 (13.33%) | 1,893,637,806 (16.77%) | 25,671 → 25,671 |
| `oracle::cold` | 1,494,559,571 (10.52%) | 1,105,522,847 (9.79%) | 4 → 4 |
| `Referee::observe` | 1,482,033,733 (10.43%) | 787,696,101 (6.98%) | 113,903 → 113,903 |
| `fake_person::Person::next` | 1,165,037,578 (8.20%) | 1,164,956,826 (10.32%) | 9,727 → 9,727 |
| `NativeClient::iterate` | 1,156,412,984 (8.14%) | 1,156,216,446 (10.24%) | 16,219 → 16,219 |
| `fixtures::engine` | 908,306,339 (6.40%) | 106,843,090 (0.95%) | 17 → 2 |
| `ServiceObserver::started` | 900,077,477 (6.34%) | 91,552,213 (0.81%) | 16 → 16 |
| `fixtures::codec` | 862,423,057 (6.07%) | 53,896,338 (0.48%) | 16 → 1 |
| `Referee::adoption_window` | 755,110,972 (5.32%) | 59,425,498 (0.53%) | 113,903 → 113,903 |
| `OneProcess::given` | 693,216,908 (4.88%) | 44,376,908 (0.39%) | 16 → 16 |
| `NativeClient::new` | 619,914,455 (4.36%) | 620,043,064 (5.49%) | 8 → 8 |
| `stories::adopted` | 516,534,234 (3.64%) | 1,669,927 (0.01%) | 38,781 → 156 |
| `Referee::check_roots` | 480,580,414 (3.38%) | 480,532,362 (4.26%) | 12 → 12 |
| `write_settings` | 274,841,592 (1.94%) | 34,878,263 (0.31%) | 8 → 8 |
| `documents::settings_file` | 274,133,809 (1.93%) | 34,223,353 (0.30%) | 8 → 1 |
| `client::native_link_limits` | 180,464,176 (1.27%) | 180,459,386 (1.60%) | 648 → 648 |
| `durable::Store::observe` | 136,932,967 (0.96%) | 136,954,414 (1.21%) | 25,671 → 25,671 |

## kill: largest self Ir

| Function | Base Ir (share) | Final Ir (share) |
|---|---:|---:|
| `__memcpy_avx_unaligned_erms` | 1,082,549,952 (7.62%) | 966,737,693 (8.56%) |
| `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` | 310,289,692 (2.18%) | 276,496,136 (2.45%) |
| `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` | 182,777,847 (1.29%) | 181,813,821 (1.61%) |
| `alloc::collections::btree::search::<impl alloc::collections::btree::node::NodeRef<BorrowType,K,V,Type>>::find_key_index` | 170,716,651 (1.20%) | 170,713,651 (1.51%) |
| `<core::option::Option<T> as core::ops::try_trait::Try>::branch` | 166,181,976 (1.17%) | 90,761,995 (0.80%) |
| `core::option::Option<T>::expect` | 161,413,151 (1.14%) | 146,069,432 (1.29%) |
| `core::result::Result<T,E>::expect` | 157,121,902 (1.11%) | 136,683,007 (1.21%) |
| `alloc::collections::btree::navigate::LazyLeafRange<BorrowType,K,V>::init_front` | 155,256,991 (1.09%) | 74,947,386 (0.66%) |
| `core::ptr::const_ptr::<impl *const T>::is_aligned_to` | 116,135,560 (0.82%) | 104,529,220 (0.93%) |
| `<temper_one_process_world::referee::Referee as skein_world::referee::Referee<temper_playground::programs::Process<temper_one_process_world::world::OneProcess>>>::observe` | 101,157,553 (0.71%) | 101,159,373 (0.90%) |

## power: largest inclusive work functions

Enclosing test/runtime, generic dispatch and world-loop frames are omitted. These selected work functions are sorted by base Ir; nested shares overlap and must not be added.

| Function | Base Ir (share) | Final Ir (share) | Calls base → final |
|---|---:|---:|---:|
| `Person::iterate` | 2,598,554,480 (18.66%) | 2,598,370,485 (23.56%) | 23,042 → 23,042 |
| `service::iterate_stage` | 2,526,813,796 (18.15%) | 2,526,647,669 (22.91%) | 129,830 → 129,830 |
| `programs::service (startup)` | 2,107,758,295 (15.14%) | 648,300,389 (5.88%) | 16 → 16 |
| `engine::iterate` | 1,921,321,255 (13.80%) | 1,921,314,572 (17.42%) | 25,966 → 25,966 |
| `oracle::cold` | 1,494,803,518 (10.73%) | 1,105,319,136 (10.02%) | 4 → 4 |
| `Referee::observe` | 1,445,479,163 (10.38%) | 777,054,967 (7.05%) | 115,378 → 115,378 |
| `NativeClient::iterate` | 1,153,461,457 (8.28%) | 1,153,197,353 (10.46%) | 16,514 → 16,514 |
| `NativeClient::new` | 930,574,048 (6.68%) | 930,694,687 (8.44%) | 12 → 12 |
| `fixtures::engine` | 908,335,574 (6.52%) | 106,842,970 (0.97%) | 17 → 2 |
| `ServiceObserver::started` | 900,111,316 (6.46%) | 91,563,194 (0.83%) | 16 → 16 |
| `fixtures::codec` | 862,449,492 (6.19%) | 53,895,997 (0.49%) | 16 → 1 |
| `Referee::adoption_window` | 730,067,181 (5.24%) | 60,365,666 (0.55%) | 115,378 → 115,378 |
| `OneProcess::given` | 693,302,723 (4.98%) | 44,377,996 (0.40%) | 16 → 16 |
| `stories::adopted` | 525,193,032 (3.77%) | 1,669,927 (0.02%) | 39,371 → 156 |
| `fake_person::Person::next` | 513,388,482 (3.69%) | 513,140,090 (4.65%) | 5,301 → 5,301 |
| `Referee::check_roots` | 480,920,480 (3.45%) | 480,548,028 (4.36%) | 12 → 12 |
| `write_settings` | 274,959,342 (1.97%) | 34,972,226 (0.32%) | 8 → 8 |
| `documents::settings_file` | 274,235,916 (1.97%) | 34,223,397 (0.31%) | 8 → 1 |
| `client::native_link_limits` | 187,705,010 (1.35%) | 187,699,974 (1.70%) | 674 → 674 |
| `durable::Store::observe` | 137,178,216 (0.99%) | 137,157,964 (1.24%) | 25,966 → 25,966 |

## power: largest self Ir

| Function | Base Ir (share) | Final Ir (share) |
|---|---:|---:|
| `__memcpy_avx_unaligned_erms` | 1,105,967,622 (7.94%) | 989,853,711 (8.98%) |
| `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` | 318,180,294 (2.28%) | 284,386,738 (2.58%) |
| `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` | 203,033,049 (1.46%) | 202,069,023 (1.83%) |
| `alloc::collections::btree::search::<impl alloc::collections::btree::node::NodeRef<BorrowType,K,V,Type>>::find_key_index` | 181,118,836 (1.30%) | 181,115,836 (1.64%) |
| `<core::option::Option<T> as core::ops::try_trait::Try>::branch` | 168,508,612 (1.21%) | 93,088,631 (0.84%) |
| `core::result::Result<T,E>::expect` | 165,096,923 (1.19%) | 144,649,178 (1.31%) |
| `core::option::Option<T>::expect` | 163,889,751 (1.18%) | 148,546,032 (1.35%) |
| `alloc::collections::btree::navigate::LazyLeafRange<BorrowType,K,V>::init_front` | 158,203,923 (1.14%) | 76,894,024 (0.70%) |
| `<temper_one_process_world::referee::Referee as skein_world::referee::Referee<temper_playground::programs::Process<temper_one_process_world::world::OneProcess>>>::observe` | 98,084,140 (0.70%) | 98,085,960 (0.89%) |

## Causes, largest first

Each step profiles both complete sweeps again. Savings below are differences in whole-sweep Ir, not sums of inclusive footprints. The codec kill run uses 29 sequential dump parts in one process; its total is the sum of their `PROGRAM TOTALS`. Other rows have one profile file per test.

| Step | Kill Ir | Saved since previous | Power-loss Ir | Saved since previous |
|---|---:|---:|---:|---:|
| Base | 14,203,061,119 | — | 13,925,573,290 | — |
| Reuse adoption codec | 13,396,136,524 | 806,924,595 | 13,116,979,306 | 808,593,984 |
| Track cache history and durable positions | 12,694,436,269 | 701,700,255 | 12,443,611,304 | 673,368,002 |
| Reuse typed adoption defaults | 12,044,251,740 | 650,184,529 | 11,791,363,108 | 652,248,196 |
| Refresh adopted count per durable commit | 11,526,201,072 | 518,050,668 | 11,264,605,123 | 526,757,985 |
| Reuse stock settings bytes; preserve initial empty cursor read | 11,290,382,441 | 235,818,631 | 11,028,384,918 | 236,220,205 |

| Cause | Where | Share before its fix (kill / power) | What changed |
|---|---|---:|---|
| Reparse and construct codec per start | `src/fixtures.rs`, `ServiceObserver::started` | 6.07% / 6.19% | Adoption codec construction falls from 16 to 1; every service still starts. |
| Rebuild saved keys and scan unchanged rows per observation | `src/referee.rs`, `world::saved_adoptions` | 5.64% / 5.57% | History position tracks every new private-cache event, including an empty synced table; commit position gates application-row scans. A new cursor reads the initial empty state. |
| Rebuild the same typed defaults per service start | `src/fixtures.rs`, `OneProcess::given` | 5.46% / 5.57% | Construct once for Adoption, copy all default fields and policy-map capacity into each independent start. Other stories keep their factories. |
| Recount adopted rows on every action and completion check | `src/referee.rs` | 4.29% / 4.45% | Refresh with the existing settled-row commit cache, initialize from previous rows and clear on recovery. |
| Regenerate identical stock settings JSON per setup/armed phase | `src/stories.rs`, `write_settings` | 2.39% / 2.44% | Generate once, still write identical bytes on every fresh disk. Net final step includes the initial-cursor correctness repair. |

The existing focused test `world::tests::an_adoption_saves_a_pending_window_before_the_engine_commit` also compares cached codec bounds, every typed-default field (including map capacity), and settings bytes with fresh construction. Every new rule was broken by hand and restored: window end `585→586` failed its boundary assertion; suppressing new history events lost the window; dropping a cached charter failed default equality; changing a codec bound failed codec equality; keeping adopted count zero failed completion with a durable repository present; settings `{}` failed byte equality; suppressing the initial history read failed `None` versus `Some(empty)`. Each restoration passed the same test alone. No test was added, removed or renamed.

## Setup tail and prefix decisions

Temporary non-inlined markers with `--dump-before="*sweep_measure_*"` segmented the codec kill run; they are removed. Setup tails after the pending window total 268,338,682 Ir (2.00% of that sweep). A representative seed tail costs 67,084,924 Ir, including 39,680,470 Ir for its required root check. Even removing every other tail instruction would save at most about 110 million Ir across four seeds, under 1% of the final sweep, before paying early shutdown/drain work. The full tail stays.

Armed prefixes to the cut total 3,215,213,736 Ir, 24.00% of the codec kill sweep. Forking is not used: `Phase::Story` reopens root 2 and spawns a direct client with nonrestartable peers; `Phase::Fault` uses the private startup root, restartable client/peers, and an additional oracle process after `oracle::begin`. The process/root graph and startup sources differ before the cut. Sharing one prefix would change the simulated world. Both set first-start settings for Adoption, so their invariant fixture inputs can be reused without sharing a live prefix. Power-loss prefixes were not separately segmented; its complete sweep was profiled at every step.

## Shared focused benefit

| Exact test | Base Ir | Final Ir | Base CPU s | Final CPU s |
|---|---:|---:|---:|---:|
| `one_process::an_adopted_repository_gets_its_change_pushed_through_git` | 2,913,791,518 | 2,764,434,204 | 0.963688 | 0.965730 |

This one-seed sample saves 149,357,314 Ir (5.13%). Its CPU samples show no gain. The baseline was rebuilt from pristine sources, then the formatted final sources were restored. No focused-only cause was pursued.

## Left

For `under-causes`: these are inclusive footprints outside this world, not estimates of removable work.

| Work and owner | Final kill Ir (share) | Final power Ir (share) | What was read |
|---|---:|---:|---|
| Fake person, `skein/testing/skein-fake-person/src/lib.rs` | 1,164,956,826 (10.32%) | 513,140,090 (4.65%) | `next` repeatedly matches the current step through the tree face; keep all steps and deadline checks. |
| Native startup, `temper/crates/temper-client-shell-terminal/src/client.rs` | 620,043,064 (5.49%) | 930,694,687 (8.44%) | `new` checks capacities, derives view bounds and creates shell/root/view; fresh sign-in and restarted clients still need independent sessions. |
| Native iteration, `temper/crates/temper-client-shell-terminal/src/client.rs` | 1,156,216,446 (10.24%) | 1,153,197,353 (10.46%) | The same real client iterations remain; the world does not skip product work. |
| Service iteration, `temper/crates/temper-service/src/system.rs` | 2,493,753,519 (22.09%) | 2,526,647,669 (22.91%) | Actual service/engine/worker work remains in every deployment and restart. |
| Service root formatting, `temper/crates/temper-service/src/system.rs` | 363,095,161 (3.22%) | 363,122,187 (3.29%) | The world adapter delegates to `Service::fmt_roots`; the entire root is still formatted at all 12 checks. |

## Stopped at

- kill: `client::native_link_limits` costs 180,459,386 (1.60%). Caching its fixed bounds could remove only part of that footprint because returning the view still copies it.
- power: `client::native_link_limits` costs 187,699,974 (1.70%). Caching its fixed bounds could remove only part of that footprint because returning the view still copies it.

This is the next fixed-input construction and is below about 2% in both arms, so it stays. Skipping old commit-vector entries is smaller still: all of `Referee::observe` self Ir (about 101 million) plus all its slice iteration (about 55 million) is only about 1.4% of a final sweep, an upper bound that includes checks a cursor cannot remove. Root text searching/formatting overhead outside the service formatter is about 117 million Ir, around 1%. Full snapshots cost only about 1.7 million Ir, 0.02% or less. None of these footprints justifies another change here; all their checks remain.

## Final checks

- `cargo fmt` (and a package-only formatting pass after the cursor initialization review).
- From `temper/`: `python3 -I -B scripts/old_forms.py`: passes, no baseline changes. The root-directory invocation was corrected after its explicit missing-baseline error.
- `heavy cargo clippy -p temper-one-process-world --all-targets -- -D warnings`: passes.
- `heavy cargo nextest run -p temper-one-process-world`: 51 passed, the same 51 pass names as base.
- `heavy cargo nextest run -p temper-one-process-world --profile fuzzy`: 4 passed, the same four fuzzy names as base.
- The same two existing ignores remain: `story_a::story_a_is_walked_from_the_welcome_to_the_csv_change_landed` (stalled startup) and `story_a_at_its_high_water` (manual memory measurement). Neither was run.
- `cargo tree -i temper-one-process-world --depth 1` lists no dependent package. No public API or cross-tree source changed.
- Only focused/fuzzy tests ran. No browser/process/live tier, walk or throwaway product driver ran.
