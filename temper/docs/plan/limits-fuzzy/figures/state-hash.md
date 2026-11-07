# state-hash: complete-state formatting in the engine sweeps

2026-11-07 07:32:18 UTC, base commit `d5a82f76bc86`, 16 CPUs,
AMD EPYC-Genoa Processor, valgrind 3.24.0. Test profile: the workspace's
unchanged unoptimized `test` profile, with its existing dependency overrides.

| Sweep | Worlds or inputs drawn | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---:|---|---:|---:|---:|---|
| `fuzzy::walking::fuzzy_walking_shard_0` | 8 seeds, two lockstep worlds each | base | 1 | 21,334,207,274 | 2.549967 (2.541961 + 0.008006) | 1 passed natively; 1 passed under callgrind |
| `fuzzy::escalation::fifteen_small_escalation_histories_cover_every_story_and_durable_cut` | 15 histories, two lockstep worlds each | base | 1 | 20,272,441,207 | 3.511566 (3.263597 + 0.247969) | 1 passed natively; 1 passed under callgrind |

Each profile produced one process file; Ir is its `PROGRAM TOTALS`, with no
child totals to add. Native wall seconds were 2.550728 and 3.512414;
callgrind test wall seconds were 121.43 and 163.03. Native load averages
were (15.96, 25.50, 24.99) and (14.03, 24.77, 24.76). There are no real-time
polls to normalize in these simulated worlds.

## Method

Built and located the suite with
`heavy cargo nextest list -p temper-engine-domain-world --test suite --message-format json`.
From the suite's reported cwd, set `CARGO_MANIFEST_DIR` to that cwd and ran
`heavy bash -lc '... valgrind --tool=callgrind --callgrind-out-file=<slot>/target/limits-fuzzy/state-hash/<label>.%p.out <binary> --exact <test name> --nocapture'`
once for each full test name above. Both executions selected exactly one test.
All builds and executions used `TEMPER_BUNDLE=none`.

Read each file with `callgrind_annotate --inclusive=yes --auto=no`,
`--inclusive=no --auto=no`, and `--inclusive=yes --tree=calling --auto=no`.
The inclusive tables below retain the largest direct calls from each
`run_replayed`, rather than the test harness ancestors. Their edge counts
separate whole-world formatting from the formatting below `World::iterate`.
Native runs used the same binary, cwd, environment and exact test names,
through heavy, with the `resource.RUSAGE_CHILDREN` wrapper in
`notes/measuring.md` (GNU time was absent). No build settings changed.

The raw inclusive annotation starts with recursive formatting aggregates:
walking's `DebugStruct::field'2` is 77,812,136,187 Ir (364.7%),
`Formatter::debug_struct_fields_finish'2` 53,211,428,123 (249.4%), and
`<&dyn Debug as Debug>::fmt'2` 46,291,645,313 (217.0%). Escalation's same
three entries are 51,048,145,412 (251.8%), 34,084,994,163 (168.1%), and
30,263,679,000 (149.3%). These recursive aggregates count nested visits;
they are not disjoint shares and are not the gate numerator.

## Walking: the largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 99.93% | 21,318,767,362 | 8 | `walking::run_replayed` (parent) |
| 96.04% | 20,489,725,800 | 2,764 | `run_replayed → core::fmt::Write::write_fmt` |
| 1.80% | 384,919,927 | 2,764 | `run_replayed → walking::World::iterate` |
| 1.28% | 272,156,344 | 16 | `run_replayed → walking::World::close` |
| 0.65% | 138,018,529 | 16 | `run_replayed → walking::World::new` |
| 0.08% | 16,010,942 | 1,382 | `run_replayed → Vec::eq` (trace comparison) |
| 0.05% | 11,253,703 | 1,382 | `run_replayed → String::eq` (state comparison) |

## Walking: the largest by self Ir

Source locations distinguish separate instruction attributions with the same
symbol; the entries below preserve those distinctions.

| Share | Ir | Function |
|---:|---:|---|
| 13.67% | 2,915,776,360 | `const_ptr::is_aligned_to` (`core/src/num/uint_macros.rs`) |
| 12.30% | 2,624,198,724 | `const_ptr::is_aligned_to` (`core/src/ptr/const_ptr.rs`) |
| 7.77% | 1,656,704,174 | `copy_nonoverlapping::precondition_check` (`core/src/ub_checks.rs`) |
| 7.03% | 1,499,326,160 | `from_raw_parts::precondition_check` (`core/src/ub_checks.rs`) |
| 5.89% | 1,257,387,078 | `Vec::append_elements` |
| 5.71% | 1,218,161,655 | `copy_nonoverlapping::precondition_check` (`core/src/ptr/mod.rs`) |
| 4.16% | 887,296,065 | `__memcpy_avx_unaligned_erms` |
| 3.20% | 682,174,850 | `maybe_is_nonoverlapping::runtime` (`core/src/intrinsics/mod.rs`) |
| 2.74% | 584,721,300 | `maybe_is_nonoverlapping::runtime` (`core/src/num/uint_macros.rs`) |

## Escalation: the largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 99.89% | 20,249,231,969 | 15 | `escalation::run_replayed` (parent) |
| 90.34% | 18,313,153,944 | 6,758 | `run_replayed → alloc::fmt::format` |
| 4.67% | 947,341,841 | 6,758 | `run_replayed → escalation::World::iterate` |
| 3.17% | 641,752,882 | 30 | `run_replayed → escalation::World::close` |
| 1.35% | 274,585,784 | 30 | `run_replayed → escalation::World::new` |
| 0.25% | 49,955,003 | 3,379 | `run_replayed → String::eq` (state comparison) |
| 0.06% | 13,165,412 | 15 | `run_replayed → drop_in_place<World>` |
| 0.04% | 7,103,917 | 6,758 | `run_replayed → drop_in_place<String>` |

## Escalation: the largest by self Ir

| Share | Ir | Function |
|---:|---:|---|
| 18.17% | 3,683,114,893 | `__memcpy_avx_unaligned_erms` |
| 11.09% | 2,249,095,875 | `<String as fmt::Write>::write_str` (`alloc/src/string.rs`) |
| 9.63% | 1,951,726,978 | `<str as Debug>::fmt` (`core/src/slice/iter/macros.rs`) |
| 5.53% | 1,120,755,090 | `DebugStruct::field'2` |
| 5.48% | 1,110,923,424 | `Formatter::pad_integral` |
| 4.44% | 899,601,241 | `<String as fmt::Write>::write_str` (`alloc/src/vec/mod.rs`) |
| 2.63% | 532,604,282 | `<str as Debug>::fmt` (`core/src/fmt/mod.rs`) |
| 2.22% | 450,607,191 | `<String as fmt::Write>::write_str` (`alloc/src/raw_vec/mod.rs`) |
| 2.03% | 411,453,120 | `Formatter::pad_integral::write_prefix` |

## Gate

Gate: holds, 96.09% walking; 90.58% escalation; 95.20% weighted by 18 and 3.5 CPU s.

Walking's numerator is 20,489,725,800 + 11,253,703 = 20,500,979,503 Ir:
`run_replayed`'s own 2,764 `write_fmt` calls for `first` and `replay`, plus
its 1,382 state `String::eq` calls. It excludes the trace's `Vec::eq`,
`iterate` and everything beneath `iterate`, including input/output trace
formatting. The walk removes each trace before formatting its two worlds.

Escalation's numerator is 18,313,153,944 + 49,955,003 = 18,363,108,947 Ir:
`run_replayed`'s own 6,758 `format` calls for `first` and `replay`, plus its
3,379 state `String::eq` calls. Its worlds retain their traces inside the
state text; that repeated rendering is part of these direct calls. Formatting
new trace entries in `iterate` is excluded. String destruction and the
helpers' own setup instructions are also excluded, conservatively.

Divide these numerators by their respective `PROGRAM TOTALS`, then weight:
`(18 × 0.9609440482 + 3.5 × 0.9058163622) / 21.5 = 0.9519697737`.
Both individual shares exceed 25%. Applied to the task's historical CPU
weights, this is 20.47 CPU s of eligible work, not a measured saving:
reading and hashing state still costs work, and the replacement belongs to
`engine-sweeps`.

## Causes, largest first

| Cause | Where | Share of each sweep | What it saved, or whose it is |
|---|---|---|---|
| Format and compare the complete state at every iteration | `temper/tests/engine/domain/src/{walking,escalation}.rs`, `run_replayed` | walking 96.09%; escalation 90.58% | Gate authorizes 286 `Hash` derives in skein/jig; the digest consumer is `engine-sweeps`' |
| Drive both worlds, including trace production | same files, `World::iterate` | walking 1.80%; escalation 4.67% | `engine-sweeps` / later `under-causes`; footprint, no isolated removable gain measured |
| Close both worlds | same files, `World::close` | walking 1.28%; escalation 3.17% | `engine-sweeps` / later `under-causes`; required teardown, no removable gain established |
| Construct both worlds | same files, `World::new` | walking 0.65%; escalation 1.35% | Required replay setup; not a demonstrated optimization |

## Left

The comparison in `temper/tests/engine/domain/src/{walking,escalation}.rs`
is `engine-sweeps`' work (96.09% / 90.58%). No consumer or test changes are
made here. Escalation's `iterate` (4.67%) and `close` (3.17%) are footprints
outside this task; this profile establishes no saving from them. Their calls
and ownership are retained above for the later tasks.

## Stopped at

The next direct walking cause, `iterate`, is 1.80%, below about 2% even
before separating required work from any removable cost. Escalation's next
cause is 4.67%, followed by close at 3.17%; both are left to their owners,
not declared negligible. This task stops at the derives allowed by the gate.
They add unused impls and change no executed path, so no post-edit timing or
instruction reduction is claimed. The eight seeds and fifteen histories
remain unchanged; the eventual digest comparison must measure its own arm.
