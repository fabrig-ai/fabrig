# engine-sweeps: lockstep replay without repeated state text

2026-11-07 UTC. Base `9695f7550746`; measured code `f0e82711cd32bcd449b68f4ad6394149220add74`. 8 CPUs,
AMD EPYC-Genoa Processor; valgrind 3.24.0. The workspace's unoptimized test
profile and its dependency settings are unchanged. Native CPU is user plus
system with waited-for descendants. Each profile produced one process file;
Ir is its `PROGRAM TOTALS`. No child totals or timer-poll normalization apply.

## Method

Build and locate the suite with:

```sh
heavy cargo nextest list -p temper-engine-domain-world --test suite --message-format json
```

Use its `rust-suites` entry's `binary-path` and `cwd`. For each exact test
name below, from that cwd with `CARGO_MANIFEST_DIR` set to the cwd:

```sh
heavy bash -lc 'cd <cwd> && CARGO_MANIFEST_DIR=<cwd> valgrind --tool=callgrind --callgrind-out-file=<slot>/target/limits-fuzzy/engine-sweeps/<arm>-<label>.%p.out <binary-path> --exact <test name> --nocapture'
callgrind_annotate --inclusive=yes --auto=no <profile>
callgrind_annotate --inclusive=no --auto=no <profile>
callgrind_annotate --inclusive=yes --tree=calling --auto=no <profile>
```

Every execution selected exactly one passing test. Native runs use the same
binary, cwd, environment and exact name through heavy, with the
`resource.RUSAGE_CHILDREN` wrapper from `temper/docs/plan/limits-fuzzy/notes/measuring.md`.
All builds and tests use `TEMPER_BUNDLE=none`. One run per arm is sufficient
for these simulated worlds. Native variation is retained; small intermediate
CPU changes are not claimed as savings when instruction counts alone establish
the gain. Temporary wrappers only measure the test executables.
Raw profiles and measurement scripts are removed after recording these figures.

## Before and after

| Sweep | Worlds or inputs drawn | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---:|---|---:|---:|---:|---|
| `fuzzy::walking::fuzzy_walking_shard_0` | 8 seeds, 2 worlds each | base | 1 | 21,334,206,518 | 2.217662 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_0` | 8 seeds, 2 worlds each | final | 1 | 1,584,596,947 | 0.486608 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_1` | 8 seeds, 2 worlds each | base | 1 | 22,768,256,748 | 2.510219 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_1` | 8 seeds, 2 worlds each | final | 1 | 1,667,866,674 | 0.488216 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_2` | 8 seeds, 2 worlds each | base | 1 | 22,603,153,455 | 2.395202 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_2` | 8 seeds, 2 worlds each | final | 1 | 1,639,838,002 | 0.462534 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_3` | 8 seeds, 2 worlds each | base | 1 | 20,420,165,157 | 2.133657 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_3` | 8 seeds, 2 worlds each | final | 1 | 1,554,405,629 | 0.484399 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_4` | 8 seeds, 2 worlds each | base | 1 | 23,579,272,030 | 2.432868 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_4` | 8 seeds, 2 worlds each | final | 1 | 1,700,780,608 | 0.505550 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_5` | 8 seeds, 2 worlds each | base | 1 | 21,075,888,205 | 2.160036 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_5` | 8 seeds, 2 worlds each | final | 1 | 1,579,339,793 | 0.487149 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_6` | 8 seeds, 2 worlds each | base | 1 | 19,517,047,126 | 2.036504 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_6` | 8 seeds, 2 worlds each | final | 1 | 1,497,467,319 | 0.425731 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_7` | 8 seeds, 2 worlds each | base | 1 | 24,822,020,279 | 2.561331 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_7` | 8 seeds, 2 worlds each | final | 1 | 1,772,540,366 | 0.514498 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_terminal_commit_cut_preserves_complete_replay_and_funding` | 4 seeds, 2 worlds each | base | 1 | 15,408,212,864 | 1.610144 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_terminal_commit_cut_preserves_complete_replay_and_funding` | 4 seeds, 2 worlds each | final | 1 | 1,160,506,724 | 0.347736 | 1 passed native; 1 passed Callgrind |
| `fuzzy::escalation::fifteen_small_escalation_histories_cover_every_story_and_durable_cut` | 15 histories, 2 worlds each | base | 1 | 20,272,440,601 | 3.464388 | 1 passed native; 1 passed Callgrind |
| `fuzzy::escalation::fifteen_small_escalation_histories_cover_every_story_and_durable_cut` | 15 histories, 2 worlds each | final | 1 | 4,442,798,944 | 1.195461 | 1 passed native; 1 passed Callgrind |
| `fuzzy::roles::twelve_small_role_histories_cover_each_holder_owner_order_and_atomic_cut` | 12 histories, 2 worlds each | base | 1 | 22,165,342,372 | 2.391247 | 1 passed native; 1 passed Callgrind |
| `fuzzy::roles::twelve_small_role_histories_cover_each_holder_owner_order_and_atomic_cut` | 12 histories, 2 worlds each | final | 1 | 2,535,350,532 | 0.669363 | 1 passed native; 1 passed Callgrind |

## Hasher arms and isolated changes

`word` is the initial local word hasher, `cheaper-word` removes needless
integer conversions and uses typed eight-byte chunks, `cursor` additionally
scans each role-snapshot trace entry once, and the final arm transfers the
store into the close oracle instead of opening a throwaway store. The SHA-256
arm and initial word arm differ only in their digest function. `stream` starts
from final and compares the replay's Debug chunks against the first string
instead of allocating a second string; it was discarded.

| Sweep | Worlds or inputs drawn | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---:|---|---:|---:|---:|---|
| `fuzzy::walking::fuzzy_walking_shard_0` | 8 seeds, 2 worlds each | sha | 1 | 4,861,640,580 | 0.718900 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_0` | 8 seeds, 2 worlds each | word | 1 | 1,795,559,316 | 0.493798 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_0` | 8 seeds, 2 worlds each | cheaper-word | 1 | 1,625,458,215 | 0.536400 | 1 passed native; 1 passed Callgrind |
| `fuzzy::escalation::fifteen_small_escalation_histories_cover_every_story_and_durable_cut` | 15 histories, 2 worlds each | word | 1 | 5,119,047,425 | 1.270717 | 1 passed native; 1 passed Callgrind |
| `fuzzy::escalation::fifteen_small_escalation_histories_cover_every_story_and_durable_cut` | 15 histories, 2 worlds each | cheaper-word | 1 | 4,519,479,927 | 1.326004 | 1 passed native; 1 passed Callgrind |
| `fuzzy::roles::twelve_small_role_histories_cover_each_holder_owner_order_and_atomic_cut` | 12 histories, 2 worlds each | word | 1 | 3,875,399,820 | 0.811889 | 1 passed native; 1 passed Callgrind |
| `fuzzy::roles::twelve_small_role_histories_cover_each_holder_owner_order_and_atomic_cut` | 12 histories, 2 worlds each | cheaper-word | 1 | 3,632,692,265 | 0.784794 | 1 passed native; 1 passed Callgrind |
| `fuzzy::roles::twelve_small_role_histories_cover_each_holder_owner_order_and_atomic_cut` | 12 histories, 2 worlds each | cursor | 1 | 2,596,623,084 | 0.659093 | 1 passed native; 1 passed Callgrind |
| `fuzzy::walking::fuzzy_walking_shard_0` | 8 seeds, 2 worlds each | stream | 1 | 1,596,839,051 | 0.499453 | 1 passed native; 1 passed Callgrind |
| `fuzzy::escalation::fifteen_small_escalation_histories_cover_every_story_and_durable_cut` | 15 histories, 2 worlds each | stream | 1 | 4,478,528,899 | 1.272966 | 1 passed native; 1 passed Callgrind |
| `fuzzy::roles::twelve_small_role_histories_cover_each_holder_owner_order_and_atomic_cut` | 12 histories, 2 worlds each | stream | 1 | 2,572,658,489 | 0.681220 | 1 passed native; 1 passed Callgrind |

The unkeyed local word hasher is chosen: these digests are compared in one
process and never persisted. It reads all Hash state with a multiply and a
rotate for each word. It need not be stable across targets or runs. SHA-256
remains the design's stable state digest; the cheaper transient comparator is
the task's explicit departure. Exact trace comparisons remain at every
iteration, and complete Debug text remains at each seed's end.

## Profiles

Inclusive rows are direct edges from the replay function, excluding test
harness ancestors. This avoids counting recursive formatter aggregates as
disjoint work. Self rows retain source basenames where symbols have multiple
attributions. These are the largest entries, not an allocation of every
instruction to mutually exclusive causes.


### walking-0: largest inclusive Ir

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| base | 96.04% | 20,489,725,800 | 2,764 | `core::fmt::Write::write_fmt (mod.rs)` |
| base | 1.80% | 384,919,927 | 2,764 | `world::walking::World::iterate (walking.rs)` |
| base | 1.28% | 272,156,344 | 16 | `world::walking::World::close (walking.rs)` |
| base | 0.65% | 138,018,529 | 16 | `world::walking::World::new (walking.rs)` |
| base | 0.08% | 16,010,942 | 1,382 | `alloc::vec::partial_eq::<impl core::cmp::PartialEq<alloc::vec::Vec<U,A2>> for alloc::vec::Vec<T,A1>>::eq (???)` |
| final | 47.05% | 745,593,978 | 1,382 | `world::replay::same_digest (replay.rs)` |
| final | 24.29% | 384,874,106 | 2,764 | `world::walking::World::iterate (walking.rs)` |
| final | 14.60% | 231,278,569 | 16 | `world::walking::World::close (walking.rs)` |
| final | 8.72% | 138,175,252 | 16 | `world::walking::World::new (walking.rs)` |
| final | 2.95% | 46,688,841 | 8 | `world::replay::same_text (replay.rs)` |


### walking-0: largest self Ir

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| base | 13.67% | 2,915,776,360 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (uint_macros.rs)` |
| base | 12.30% | 2,624,198,724 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (const_ptr.rs)` |
| base | 7.77% | 1,656,704,174 | `core::ptr::copy_nonoverlapping::precondition_check (ub_checks.rs)` |
| final | 7.14% | 113,180,517 | `__memcpy_avx_unaligned_erms (memmove-vec-unaligned-erms.S)` |
| final | 6.13% | 97,147,920 | `WordHasher::write_u64 (replay.rs)` |
| final | 5.11% | 80,956,600 | `core::intrinsics::rotate_left (???)` |

### walking-1: largest inclusive Ir

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| base | 96.16% | 21,893,249,484 | 2,928 | `core::fmt::Write::write_fmt (mod.rs)` |
| base | 1.81% | 412,518,854 | 2,928 | `world::walking::World::iterate (walking.rs)` |
| base | 1.20% | 272,128,616 | 16 | `world::walking::World::close (walking.rs)` |
| base | 0.61% | 137,999,295 | 16 | `world::walking::World::new (walking.rs)` |
| base | 0.08% | 18,006,043 | 1,464 | `alloc::vec::partial_eq::<impl core::cmp::PartialEq<alloc::vec::Vec<U,A2>> for alloc::vec::Vec<T,A1>>::eq (???)` |
| final | 47.86% | 798,316,046 | 1,464 | `world::replay::same_digest (replay.rs)` |
| final | 24.73% | 412,465,029 | 2,928 | `world::walking::World::iterate (walking.rs)` |
| final | 13.86% | 231,205,816 | 16 | `world::walking::World::close (walking.rs)` |
| final | 8.28% | 138,141,395 | 16 | `world::walking::World::new (walking.rs)` |
| final | 2.85% | 47,525,931 | 8 | `world::replay::same_text (replay.rs)` |


### walking-1: largest self Ir

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| base | 13.68% | 3,114,777,200 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (uint_macros.rs)` |
| base | 12.31% | 2,803,299,480 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (const_ptr.rs)` |
| base | 7.77% | 1,769,636,370 | `core::ptr::copy_nonoverlapping::precondition_check (ub_checks.rs)` |
| final | 7.02% | 117,035,199 | `__memcpy_avx_unaligned_erms (memmove-vec-unaligned-erms.S)` |
| final | 6.24% | 104,018,760 | `WordHasher::write_u64 (replay.rs)` |
| final | 5.20% | 86,682,300 | `core::intrinsics::rotate_left (???)` |

### walking-2: largest inclusive Ir

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| base | 96.22% | 21,748,548,977 | 2,932 | `core::fmt::Write::write_fmt (mod.rs)` |
| base | 1.74% | 393,392,905 | 2,932 | `world::walking::World::iterate (walking.rs)` |
| base | 1.20% | 272,147,936 | 16 | `world::walking::World::close (walking.rs)` |
| base | 0.61% | 138,006,973 | 16 | `world::walking::World::new (walking.rs)` |
| base | 0.07% | 16,951,879 | 1,466 | `alloc::vec::partial_eq::<impl core::cmp::PartialEq<alloc::vec::Vec<U,A2>> for alloc::vec::Vec<T,A1>>::eq (???)` |
| final | 48.26% | 791,430,234 | 1,466 | `world::replay::same_digest (replay.rs)` |
| final | 23.99% | 393,352,633 | 2,932 | `world::walking::World::iterate (walking.rs)` |
| final | 14.10% | 231,263,958 | 16 | `world::walking::World::close (walking.rs)` |
| final | 8.42% | 138,127,901 | 16 | `world::walking::World::new (walking.rs)` |
| final | 2.85% | 46,666,882 | 8 | `world::replay::same_text (replay.rs)` |


### walking-2: largest self Ir

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| base | 13.69% | 3,093,998,440 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (uint_macros.rs)` |
| base | 12.32% | 2,784,598,596 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (const_ptr.rs)` |
| base | 7.78% | 1,757,689,478 | `core::ptr::copy_nonoverlapping::precondition_check (ub_checks.rs)` |
| final | 7.03% | 115,334,287 | `__memcpy_avx_unaligned_erms (memmove-vec-unaligned-erms.S)` |
| final | 6.29% | 103,120,896 | `WordHasher::write_u64 (replay.rs)` |
| final | 5.24% | 85,934,080 | `core::intrinsics::rotate_left (???)` |

### walking-3: largest inclusive Ir

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| base | 95.86% | 19,574,040,500 | 2,630 | `core::fmt::Write::write_fmt (mod.rs)` |
| base | 1.90% | 387,697,935 | 2,630 | `world::walking::World::iterate (walking.rs)` |
| base | 1.33% | 272,172,445 | 16 | `world::walking::World::close (walking.rs)` |
| base | 0.68% | 138,051,766 | 16 | `world::walking::World::new (walking.rs)` |
| base | 0.08% | 15,418,291 | 1,315 | `alloc::vec::partial_eq::<impl core::cmp::PartialEq<alloc::vec::Vec<U,A2>> for alloc::vec::Vec<T,A1>>::eq (???)` |
| final | 45.89% | 713,256,489 | 1,315 | `world::replay::same_digest (replay.rs)` |
| final | 24.94% | 387,685,065 | 2,630 | `world::walking::World::iterate (walking.rs)` |
| final | 14.88% | 231,251,952 | 16 | `world::walking::World::close (walking.rs)` |
| final | 8.89% | 138,145,160 | 16 | `world::walking::World::new (walking.rs)` |
| final | 3.00% | 46,685,127 | 8 | `world::replay::same_text (replay.rs)` |


### walking-3: largest self Ir

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| base | 13.64% | 2,785,968,440 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (uint_macros.rs)` |
| base | 12.28% | 2,507,371,596 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (const_ptr.rs)` |
| base | 7.75% | 1,583,148,982 | `core::ptr::copy_nonoverlapping::precondition_check (ub_checks.rs)` |
| final | 7.22% | 112,256,669 | `__memcpy_avx_unaligned_erms (memmove-vec-unaligned-erms.S)` |
| final | 5.98% | 92,893,056 | `WordHasher::write_u64 (replay.rs)` |
| final | 4.98% | 77,410,880 | `core::intrinsics::rotate_left (???)` |

### walking-4: largest inclusive Ir

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| base | 96.27% | 22,699,663,861 | 3,042 | `core::fmt::Write::write_fmt (mod.rs)` |
| base | 1.76% | 416,074,543 | 3,042 | `world::walking::World::iterate (walking.rs)` |
| base | 1.15% | 272,092,416 | 16 | `world::walking::World::close (walking.rs)` |
| base | 0.59% | 138,106,390 | 16 | `world::walking::World::new (walking.rs)` |
| base | 0.08% | 18,604,041 | 1,521 | `alloc::vec::partial_eq::<impl core::cmp::PartialEq<alloc::vec::Vec<U,A2>> for alloc::vec::Vec<T,A1>>::eq (???)` |
| final | 48.62% | 826,995,973 | 1,521 | `world::replay::same_digest (replay.rs)` |
| final | 24.46% | 416,074,372 | 3,042 | `world::walking::World::iterate (walking.rs)` |
| final | 13.59% | 231,211,279 | 16 | `world::walking::World::close (walking.rs)` |
| final | 8.12% | 138,147,694 | 16 | `world::walking::World::new (walking.rs)` |
| final | 2.79% | 47,518,253 | 8 | `world::replay::same_text (replay.rs)` |


### walking-4: largest self Ir

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| base | 13.69% | 3,229,006,480 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (uint_macros.rs)` |
| base | 12.32% | 2,906,105,832 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (const_ptr.rs)` |
| base | 7.78% | 1,834,363,326 | `core::ptr::copy_nonoverlapping::precondition_check (ub_checks.rs)` |
| final | 6.95% | 118,191,939 | `__memcpy_avx_unaligned_erms (memmove-vec-unaligned-erms.S)` |
| final | 6.34% | 107,784,768 | `WordHasher::write_u64 (replay.rs)` |
| final | 5.28% | 89,820,640 | `core::intrinsics::rotate_left (???)` |

### walking-5: largest inclusive Ir

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| base | 95.97% | 20,226,858,036 | 2,732 | `core::fmt::Write::write_fmt (mod.rs)` |
| base | 1.85% | 389,886,765 | 2,732 | `world::walking::World::iterate (walking.rs)` |
| base | 1.29% | 272,143,456 | 16 | `world::walking::World::close (walking.rs)` |
| base | 0.65% | 138,037,922 | 16 | `world::walking::World::new (walking.rs)` |
| base | 0.07% | 15,771,666 | 1,366 | `alloc::vec::partial_eq::<impl core::cmp::PartialEq<alloc::vec::Vec<U,A2>> for alloc::vec::Vec<T,A1>>::eq (???)` |
| final | 46.58% | 735,683,064 | 1,366 | `world::replay::same_digest (replay.rs)` |
| final | 24.68% | 389,845,682 | 2,732 | `world::walking::World::iterate (walking.rs)` |
| final | 14.64% | 231,251,630 | 16 | `world::walking::World::close (walking.rs)` |
| final | 8.75% | 138,142,661 | 16 | `world::walking::World::new (walking.rs)` |
| final | 2.95% | 46,650,905 | 8 | `world::replay::same_text (replay.rs)` |


### walking-5: largest self Ir

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| base | 13.66% | 2,878,503,440 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (uint_macros.rs)` |
| base | 12.29% | 2,590,653,096 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (const_ptr.rs)` |
| base | 7.76% | 1,635,583,646 | `core::ptr::copy_nonoverlapping::precondition_check (ub_checks.rs)` |
| final | 7.17% | 113,178,451 | `__memcpy_avx_unaligned_erms (memmove-vec-unaligned-erms.S)` |
| final | 6.07% | 95,848,200 | `WordHasher::write_u64 (replay.rs)` |
| final | 5.06% | 79,873,500 | `core::intrinsics::rotate_left (???)` |

### walking-6: largest inclusive Ir

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| base | 95.78% | 18,693,989,846 | 2,536 | `core::fmt::Write::write_fmt (mod.rs)` |
| base | 1.88% | 367,046,599 | 2,536 | `world::walking::World::iterate (walking.rs)` |
| base | 1.39% | 272,165,082 | 16 | `world::walking::World::close (walking.rs)` |
| base | 0.71% | 138,096,977 | 16 | `world::walking::World::new (walking.rs)` |
| base | 0.07% | 13,667,669 | 1,268 | `alloc::vec::partial_eq::<impl core::cmp::PartialEq<alloc::vec::Vec<U,A2>> for alloc::vec::Vec<T,A1>>::eq (???)` |
| final | 45.39% | 679,677,510 | 1,268 | `world::replay::same_digest (replay.rs)` |
| final | 24.51% | 367,033,765 | 2,536 | `world::walking::World::iterate (walking.rs)` |
| final | 15.45% | 231,329,963 | 16 | `world::walking::World::close (walking.rs)` |
| final | 9.23% | 138,148,020 | 16 | `world::walking::World::new (walking.rs)` |
| final | 3.06% | 45,822,455 | 8 | `world::replay::same_text (replay.rs)` |


### walking-6: largest self Ir

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| base | 13.63% | 2,661,045,000 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (uint_macros.rs)` |
| base | 12.27% | 2,394,940,500 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (const_ptr.rs)` |
| base | 7.75% | 1,512,189,786 | `core::ptr::copy_nonoverlapping::precondition_check (ub_checks.rs)` |
| final | 7.32% | 109,615,143 | `__memcpy_avx_unaligned_erms (memmove-vec-unaligned-erms.S)` |
| final | 5.91% | 88,499,088 | `WordHasher::write_u64 (replay.rs)` |
| final | 4.92% | 73,749,240 | `core::intrinsics::rotate_left (???)` |

### walking-7: largest inclusive Ir

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| base | 96.35% | 23,916,490,105 | 3,184 | `core::fmt::Write::write_fmt (mod.rs)` |
| base | 1.77% | 439,432,856 | 3,184 | `world::walking::World::iterate (walking.rs)` |
| base | 1.10% | 272,069,692 | 16 | `world::walking::World::close (walking.rs)` |
| base | 0.56% | 137,985,404 | 16 | `world::walking::World::new (walking.rs)` |
| base | 0.08% | 20,307,777 | 1,592 | `alloc::vec::partial_eq::<impl core::cmp::PartialEq<alloc::vec::Vec<U,A2>> for alloc::vec::Vec<T,A1>>::eq (???)` |
| final | 49.24% | 872,738,326 | 1,592 | `world::replay::same_digest (replay.rs)` |
| final | 24.79% | 439,394,296 | 3,184 | `world::walking::World::iterate (walking.rs)` |
| final | 13.04% | 231,174,162 | 16 | `world::walking::World::close (walking.rs)` |
| final | 7.80% | 138,176,174 | 16 | `world::walking::World::new (walking.rs)` |
| final | 2.73% | 48,356,745 | 8 | `world::replay::same_text (replay.rs)` |


### walking-7: largest self Ir

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| base | 13.70% | 3,401,635,040 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (uint_macros.rs)` |
| base | 12.33% | 3,061,471,536 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (const_ptr.rs)` |
| base | 7.78% | 1,932,352,862 | `core::ptr::copy_nonoverlapping::precondition_check (ub_checks.rs)` |
| final | 6.86% | 121,571,537 | `__memcpy_avx_unaligned_erms (memmove-vec-unaligned-erms.S)` |
| final | 6.42% | 113,744,784 | `WordHasher::write_u64 (replay.rs)` |
| final | 5.35% | 94,787,320 | `core::intrinsics::rotate_left (???)` |

### terminal: largest inclusive Ir

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| base | 96.17% | 14,817,701,158 | 1,732 | `core::fmt::Write::write_fmt (mod.rs)` |
| base | 2.26% | 348,381,528 | 1,732 | `world::walking::World::iterate (walking.rs)` |
| base | 0.88% | 135,844,240 | 8 | `world::walking::World::close (walking.rs)` |
| base | 0.45% | 68,986,197 | 8 | `world::walking::World::new (walking.rs)` |
| base | 0.08% | 12,314,835 | 866 | `alloc::vec::partial_eq::<impl core::cmp::PartialEq<alloc::vec::Vec<U,A2>> for alloc::vec::Vec<T,A1>>::eq (???)` |
| final | 49.07% | 569,457,066 | 866 | `world::replay::same_digest (replay.rs)` |
| final | 30.02% | 348,368,623 | 1,732 | `world::walking::World::iterate (walking.rs)` |
| final | 9.94% | 115,381,596 | 8 | `world::walking::World::close (walking.rs)` |
| final | 5.95% | 69,046,701 | 8 | `world::walking::World::new (walking.rs)` |
| final | 2.50% | 28,971,254 | 4 | `world::replay::same_text (replay.rs)` |


### terminal: largest self Ir

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| base | 13.66% | 2,104,692,840 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (uint_macros.rs)` |
| base | 12.29% | 1,894,223,556 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (const_ptr.rs)` |
| base | 7.76% | 1,195,207,232 | `core::ptr::copy_nonoverlapping::precondition_check (ub_checks.rs)` |
| final | 6.33% | 73,444,200 | `WordHasher::write_u64 (replay.rs)` |
| final | 5.61% | 65,109,213 | `__memcpy_avx_unaligned_erms (memmove-vec-unaligned-erms.S)` |
| final | 5.27% | 61,203,500 | `core::intrinsics::rotate_left (???)` |

### escalation: largest inclusive Ir

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| base | 90.34% | 18,313,153,944 | 6,758 | `alloc::fmt::format (fmt.rs)` |
| base | 4.67% | 947,341,841 | 6,758 | `world::escalation::World::iterate (escalation.rs)` |
| base | 3.17% | 641,752,882 | 30 | `world::escalation::World::close (escalation.rs)` |
| base | 1.35% | 274,585,784 | 30 | `world::escalation::World::new (escalation.rs)` |
| base | 0.25% | 49,955,003 | 3,379 | `<alloc::string::String as core::cmp::PartialEq>::eq (string.rs)` |
| final | 54.79% | 2,434,014,023 | 3,379 | `world::replay::same_digest (replay.rs)` |
| final | 21.34% | 948,135,493 | 6,758 | `world::escalation::World::iterate (escalation.rs)` |
| final | 12.72% | 565,121,227 | 30 | `world::escalation::World::close (escalation.rs)` |
| final | 6.18% | 274,708,889 | 30 | `world::escalation::World::new (escalation.rs)` |
| final | 2.92% | 129,687,284 | 15 | `world::replay::same_text (replay.rs)` |


### escalation: largest self Ir

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| base | 18.17% | 3,683,114,893 | `__memcpy_avx_unaligned_erms (memmove-vec-unaligned-erms.S)` |
| base | 11.09% | 2,249,095,875 | `<alloc::string::String as core::fmt::Write>::write_str (string.rs)` |
| base | 9.63% | 1,951,726,978 | `<str as core::fmt::Debug>::fmt (macros.rs)` |
| final | 7.02% | 311,671,296 | `WordHasher::write_u64 (replay.rs)` |
| final | 5.97% | 265,328,054 | `__memcpy_avx_unaligned_erms (memmove-vec-unaligned-erms.S)` |
| final | 5.85% | 259,726,080 | `core::intrinsics::rotate_left (???)` |

### roles: largest inclusive Ir

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| base | 87.26% | 19,341,684,147 | 1,162 | `core::fmt::Write::write_fmt (mod.rs)` |
| base | 9.04% | 2,003,585,135 | 24 | `world::roles::World::new (roles.rs)` |
| base | 2.01% | 445,105,627 | 24 | `world::roles::World::close (roles.rs)` |
| base | 0.76% | 168,497,064 | 1,162 | `world::roles::World::iterate (roles.rs)` |
| base | 0.68% | 150,521,379 | 1,162 | `alloc::string::String::shrink_to_fit (string.rs)` |
| final | 38.16% | 967,496,236 | 24 | `world::roles::World::new (roles.rs)` |
| final | 33.74% | 855,338,431 | 581 | `world::replay::same_digest (replay.rs)` |
| final | 15.14% | 383,802,751 | 24 | `world::roles::World::close (roles.rs)` |
| final | 6.65% | 168,552,034 | 1,162 | `world::roles::World::iterate (roles.rs)` |
| final | 4.54% | 115,228,410 | 12 | `world::replay::same_text (replay.rs)` |


### roles: largest self Ir

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| base | 12.94% | 2,867,197,680 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (uint_macros.rs)` |
| base | 11.64% | 2,580,477,912 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to (const_ptr.rs)` |
| base | 7.55% | 1,674,040,094 | `core::ptr::copy_nonoverlapping::precondition_check (ub_checks.rs)` |
| final | 6.99% | 177,222,530 | `__memcpy_avx_unaligned_erms (memmove-vec-unaligned-erms.S)` |
| final | 4.21% | 106,674,552 | `WordHasher::write_u64 (replay.rs)` |
| final | 3.51% | 88,895,460 | `core::intrinsics::rotate_left (???)` |


## Causes, largest first

The full measured set falls from **233,966,005,355 to 21,135,491,538 Ir**
(90.97%) and **25.913258 to 6.067245 native CPU s** (76.59%, 19.846013 s
saved). The eight walking shards together retain all 64 seeds. Every final
shard is below the 10 s guide; the largest is 0.514498 CPU s. Escalation
(1.195461) and roles (0.669363) now exceed every walking shard, so the
walking-first fuzzy override and its longest-tests comment are removed.
The eight shards remain, with the same names and seed ranges.

| Cause | Where | Measured share | What it saved |
|---|---|---|---|
| Render/compare frozen state and manage temporary strings every iteration | `temper/tests/engine/domain/src/{walking,escalation,roles}.rs`, replay loops | Base walking-0 96.09%; escalation 90.58%; roles 87.98% including shrinking | Base to initial word: 19,538,647,202 / 15,153,393,176 / 18,289,942,552 Ir saved respectively, after paying for hashing and the final text comparison |
| Checked word conversions and slice-to-array conversions in hashing | `temper/tests/engine/domain/src/replay.rs`, `WordHasher` | Initial walking-0 `write_isize` 8.05%, `write_usize` 6.69%, `write` 9.23%; these inclusive footprints overlap | Initial word to cheaper-word: 170,101,101 Ir (9.47%) walking-0; 599,567,498 (11.71%) escalation; 242,707,555 (6.26%) roles. Small native differences are noise, not claimed CPU gains |
| Search the growing prior-story trace again for its acknowledgement | `temper/tests/engine/domain/src/roles.rs`, `snapshot` | Initial word roles: snapshot 44.16%; trace-search closure 26.95%; 114,936 `contains` calls | Cheaper-word to cursor: 1,036,069,181 Ir (28.52%) saved. Each new entry is checked once; the waiting-row scan occurs only once ACK and empty pending storage make it relevant |
| Open an unused real store before swapping in the story's store for teardown | `temper/tests/engine/domain/src/oracle.rs`, `close_world` | Cheaper-word walking-0 `Store::new` footprint 5.12%; half its constructor calls are placeholders | Transfer through existing `take_for_drop`: 40,861,268 Ir (2.51%) walking-0; 76,680,983 (1.70%) escalation; 61,272,552 (2.36%) roles, measured against cheaper-word / cheaper-word / cursor respectively. Actual stores, commits, codec and cold-readback oracles remain |

The roles cursor changes observation work only: it returns at the same
iteration, after the same genuine held-chat transactions. Each replay still
constructs and advances both worlds independently. No fixture construction
whose result varies with a seed is shared across seeds. The store transfer
eliminates an unused replacement, not a durable write or read.

## Preserved coverage

The executable code in `tests/suite/fuzzy/{walking,escalation,roles}.rs` is
byte-for-byte unchanged after removing comment lines. Thus seed lists, random
settings, cuts, delays, coverage sets and ending assertions remain. The final
suite adds exactly `walking::a_replay_differing_in_its_root_store_or_counters_fails_its_compare`;
no test names disappear and ignored flags are unchanged.

These are the direct replay-loop call counts, identical before and after.
`same_text` runs once per seed/history before either close; both worlds still
have their original settled, final funding and teardown checks.

| Sweep label | Seed/history cases | World iterations, base = final | World closes, base = final | Final complete-text comparisons |
|---|---:|---:|---:|---:|
| walking-0 | 8 | 2,764 | 16 | 8 |
| walking-1 | 8 | 2,928 | 16 | 8 |
| walking-2 | 8 | 2,932 | 16 | 8 |
| walking-3 | 8 | 2,630 | 16 | 8 |
| walking-4 | 8 | 3,042 | 16 | 8 |
| walking-5 | 8 | 2,732 | 16 | 8 |
| walking-6 | 8 | 2,536 | 16 | 8 |
| walking-7 | 8 | 3,184 | 16 | 8 |
| terminal | 4 | 1,732 | 8 | 4 |
| escalation | 15 | 6,758 | 30 | 15 |
| roles | 12 | 1,162 | 24 | 12 |

The new focused table checks a changed `pages`, a removed deployment store row,
and a root one `iterate` ahead. It requires both complete world texts and the
row label in each panic. Extra rows cover unequal same-length Secret and
Buffer bytes whose Debug texts agree; they must still fail by digest and say
that their text is equal. Inverting digest equality made the test fail on
`pages`; restoring it passed. Temporarily accepting equal Debug text despite
unequal digests made it fail on `hidden bytes`; restoring it passed.

## Hash against Debug

The contracts survey matches the implementations read in `skein-lib`:
container and Buffer capacities and Journal release limits have no setter;
Slab length follows its hashed slots, and Deadlines maintains its by-key index
from the same arm/cancel/expire operations that maintain its hashed by-time
index. Buffer and Secret hash held bytes while Debug omits them. No missing
mutable-state Hash implementation was found, so no skein fix was needed.
The test Store hashes exactly the four fields its Debug prints: applied,
rows, pending and held; the real component remains exercised by each world.

Added the contracts list's 352 temper product derives and 25 world derives.
No additional state derives were required outside that list or outside
these eight packages. Store uses its specified manual Hash; Delayed, listed
in the survey but explicitly excluded by the Store boundary, needs none.
The new test's local Hidden enum derives Hash/Debug as a test fixture.

`meter.check` is unchanged. After `meter.end`, it reconstructs the peak owned
heap from recorded highs, subtracting outputs handed away after the step,
and asserts it is at most the fixed ceiling. The role recovery test still
counts both worlds, fake store/history, referees and trace under
`ROLE_RECOVERY_HEAP_CEILING = 1_700_000`; its focused run and full suite pass.

## Left

`under-causes` owns changes below this world beyond the agreed derive words.
The largest remaining footprints below are **inclusive and overlapping**;
they show where required work remains, not a demonstrated removable gain.

| Function / file | Walking-0 | Escalation | Roles | Owner / reading |
|---|---:|---:|---:|---|
| `jig_protocol_store::down`, `jig/crates/jig-protocol-store/src/lib.rs` | 190,676,595 Ir, 12.03% | 458,764,281, 10.33% | 394,209,227, 15.55% | under-causes; the real store still handles writes and reads through its codec over Files |
| `restore::restore`, `jig/crates/jig-protocol-store/src/restore.rs` | 119,172,280, 7.52% | 297,453,180, 6.70% | 302,540,418, 11.93% | under-causes; nested in store work, so not additive with down |
| `domain::step`, `temper/crates/temper-engine-domain/src/domain.rs` | 222,185,309, 14.02% | 529,466,315, 11.92% | 407,293,003, 16.06% | under-causes; both real roots and their children must still step |
| Derived `Domain::hash`, same file | 483,945,408, 30.54% | 1,254,101,206, 28.23% | 220,751,620, 8.71% | under-causes for a future mechanism; full frozen-state traversal is this task's comparison contract |

No stores, fakes, protocols, budgets, timeouts or build settings were replaced
to avoid these costs. `fuzzy::removal` and `fuzzy::reset` are unchanged.
No runtime fix reaches `fuzzy::endings` or `fuzzy::loads`, so they receive no
separate performance claim.

## Stopped at

The next removable world work is rechecking the already-equal prefix of each
append-only trace. Even removing its **entire** Vec equality cost would save
at most 1.17% of a sweep; required comparison of new entries means the real
possible gain is smaller. It is below about 2%, so the exact accumulated-trace
comparisons stay. Larger rows above are required work or under-causes' scope.

| Sweep label | Complete trace equality Ir | Share of final sweep |
|---|---:|---:|
| walking-0 | 16,025,754 | 1.01% |
| walking-1 | 18,005,136 | 1.08% |
| walking-2 | 16,956,581 | 1.03% |
| walking-3 | 15,415,809 | 0.99% |
| walking-4 | 18,612,309 | 1.09% |
| walking-5 | 15,775,061 | 1.00% |
| walking-6 | 13,680,514 | 0.91% |
| walking-7 | 20,312,978 | 1.15% |
| terminal | 12,317,012 | 1.06% |
| escalation | 51,910,716 | 1.17% |
| roles | 10,668,271 | 0.42% |

The larger-looking final text comparison is required once per seed. Tested a
streaming alternative rather than treating that footprint as a saving: it
added 12,242,104 Ir (+0.77%) walking-0, 35,729,955 (+0.80%) escalation and
37,307,957 (+1.47%) roles. It is discarded. Final code keeps the simple pair
of complete strings only at seed end and on digest mismatch. Native figures
for every accepted final test remain below its 10 s fuzzy guide.
