# public-causes: public walks and issued-secret checkpoints

2026-11-07 01:04:06 UTC. Before: commit `d00f5f0017`; after: code commit `377f78b971`. 8 CPUs, AMD EPYC-Genoa Processor, Valgrind 3.24.0. Both arms use the unoptimized test profile, including dependencies.

## Method and exact tests

- `identifiers`: `public::every_inaccessible_identifier_is_one_unknown_byte_for_byte`.
- `owners`: `public::every_owner_version_and_kind_answers_as_declared`.
- `m15`: `m15::whole_run_and_renewal_scan_every_fake_issued_token_byte`.

These deterministic simulated worlds ran once natively and once under Callgrind per arm. Every invocation selected one test and reported `1 passed`. Service and agent programs run inside the fake machine, with no native program children. Each profile produced one file; its `PROGRAM TOTALS` Ir is the figure. No patience, clock, deadline, limit, budget or build profile changed. No timer slowdown correction is needed.

The initially available `figures-before.md` had no measurements, so this task measured its own pristine base before editing. The required final merge brings its completed U1 measurement: 18,514,602,590 Ir versus this task’s 18,514,603,106 Ir. The independent unchanged-code spread is 516 instructions, below 0.000003%, far below the reported gains. Its updated public section also identifies the same 19.86% landing scan.

GNU time is absent. Bash’s reserved `time` reports user and system CPU of the waited-for test, including its waited-for descendants; the CPU column sums those fields. Native wall time is separate. Native CPU varies with the machine; instruction counts decide the changes.

For either commit, in the claimed slot, locate that arm’s binary and working directory, then repeat the commands for each exact test above:

```sh
heavy cargo nextest list -p temper-one-process-world --test suite --message-format json
# Read binary-path and cwd from rust-suites.
heavy bash -lc 'cd <cwd> && export CARGO_MANIFEST_DIR="$PWD";
  TIMEFORMAT="native wall=%R user=%U sys=%S";
  time <binary-path> --exact <test-name> --nocapture'
heavy bash -lc 'cd <cwd> && CARGO_MANIFEST_DIR="$PWD" valgrind --tool=callgrind \
  --callgrind-out-file=<slot>/target/limits-worlds/public-causes/<arm>.%p.out \
  <binary-path> --exact <test-name> --nocapture'
callgrind_annotate --inclusive=yes --auto=no --threshold=100 <profile.out>
callgrind_annotate --inclusive=no --auto=no --threshold=100 <profile.out>
```

Executions used `heavy` sequentially, with `TEMPER_BUNDLE=none` from the build configuration. The added dependency is the already locked `aho-corasick` 1.1.5, with `std` and without default features. Cargo generated its one-line `Cargo.lock` entry; no dependency version changed.

## Before and after

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Native wall s | Result |
|---|---|---:|---:|---:|---:|---|
| `identifiers` | before | 1 | 18,514,603,106 | 4.648 | 6.198 | 1 passed |
| `identifiers` | after | 1 | 13,298,375,184 | 4.034 | 5.022 | 1 passed |
| `owners` | before | 1 | 6,377,149,314 | 1.620 | 2.912 | 1 passed |
| `owners` | after | 1 | 4,564,780,427 | 1.388 | 1.887 | 1 passed |
| `m15` | before | 1 | 17,791,335,749 | 3.168 | 5.296 | 1 passed |
| `m15` | after | 1 | 12,661,876,572 | 2.602 | 3.590 | 1 passed |

- `identifiers` saves 5,216,227,922 Ir (28.17%).
- `owners` saves 1,812,368,887 Ir (28.42%).
- `m15` saves 5,129,459,177 Ir (28.83%).

## Intermediate measurements and rejected choices

These were local working diffs, not extra committed implementations. Each invocation selected and passed one test. CPU was measured for the matcher trials, not for the public intermediate arms.

| Story | Arm | Run | Ir | Native CPU s | Decision |
|---|---|---:|---:|---:|---|
| `identifiers` | landing key range | 1 | 14,709,616,454 | — | Keep |
| `identifiers` | also check in-flight I/O first | 1 | 13,699,964,715 | — | Keep |
| `identifiers` | also reuse readiness within a commit | 1 | 13,298,380,344 | — | Keep |
| `m15` | individual owned memmem finders | 1 | 29,193,119,109 | 6.194 | Reject |
| `m15` | retain old matcher groups as needles arrive | 1 | 13,595,355,299 | 2.718 | Reject |

Individual prepared finders still searched each surface once per needle. Unoptimized SIMD/precondition work raised the story to 29,193,119,109 Ir and 6.194 CPU s. The shared byte matcher replaced that trial. Retaining separate matcher groups avoided rebuilding old patterns but added searches: 13,595,355,299 Ir and 2.718 CPU s, 933,478,727 Ir (7.37%) above the retained implementation. It was restored away; all four scanner regressions and the M15 story passed again. The retained matcher builds twice, costing 920,860,241 Ir (7.27%); that parent footprint is not a promised saving.

## Causes, largest first within each story

| Cause | Where | Measured saving / footprint | Decision |
|---|---|---|---|
| Scan every durable row on readiness polls | `public/owners.rs`, `Walk::landed` and stage 2 | Range lookup saves 3,804,986,652 Ir, 20.55% of the original identifier story | Fixed; inclusive bounds cover every commit key and both callers use the helper |
| Compute service predicates before the sufficient in-flight predicate | `src/world.rs`, `ServiceObserver::drain` | Reordering saves 1,009,651,739 Ir, 6.86% of the preceding identifier arm | Fixed; the same four-way disjunction |
| Repeat the range lookup without a durable change | `public/owners.rs`, `Walk::landed` | Commit cache saves 401,584,371 Ir, 2.93% of the preceding arm | Fixed; additions and removals invalidate by `Store::applied` |
| Search every surface once per issued representation | `m15/observer.rs`, `Observer::leak_since` | Search path: 8,519,744,344 to 3,741,155,689 Ir, saving 4,778,588,655 Ir; whole story saves 5,129,459,177 Ir (28.83%) with the shared drain change | Fixed; cached matcher, overlapping matches, original needle order, checked-prefix filtering |

Paths in this table are under `temper/tests/one-process/src/` (except `src/world.rs`, under the package root). The public deployment stays three runs: tracked calibration, then untracked direct and proxied reads. `TRACKED` changes the agent’s delegation. Calibration supplies real public goal/delegate identifiers and checks owner/version/kind rows itself. Sharing the untracked runs at one instant would admit twelve reads into the unchanged eight-read burst. Waiting for refill would lose the single-instant condition. No sharing, larger burst or reduced comparison was introduced. Seeds, identifiers, request order, paths, headers and byte-for-byte assertions stay unchanged.

## Preserved checks and call counts

| Check | Before | After | Function |
|---|---:|---:|---|
| `identifiers`: HTTP answers decoded | 102 | 102 | `temper_one_process_world::public::http::answer` |
| `identifiers`: service worlds | 3 | 3 | `temper_one_process_world::public::service::phase` |
| `m15`: filename/body search entrances | 5,734 | 5,734 | `temper_one_process_world::m15::observer::Observer::leak_since` |
| `m15`: checkpoints including the final scan | 5 | 5 | `temper_one_process_world::m15::observer::Observer::scan` |

The identifier test still asserts all six reads share one instant in each arm, all responses are one unknown, and direct/proxy answers are byte-for-byte equal. The owner walk retains every declared row, complete record consumption, private-marker checks, forge-envelope references and cross-project unknowns. M15 retains four fake-issued tokens, five checkpoints, all surface/count assertions, raw/debug/base64/Basic representations and checks of artifacts predating a token.

The landing regression checks both extreme commit keys, unrelated keys on each side, and readiness after durable additions/removals. Excluding the maximum key failed at “every commit identifier can make the walk ready”; accepting an older commit’s cache failed there too. Skipping binary needles failed with `None` instead of `Some("App key")`. Non-overlapping iteration failed the new late-overlapping-needle regression with `None` instead of `Some("installation token")`. Every mutation was restored.

## Function tables

Top 15 relevant inclusive symbols and top 12 self symbols per arm. Inclusive tables omit libtest/standard-library and world/story/enum dispatch wrappers, retaining application, referee and byte-processing work. Identical demangled symbols are grouped across contexts. Inclusive costs overlap; self costs partition Ir.

### `identifiers` before: largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 20.69% | 3,831,432,839 | 159,386 | `<temper_one_process_world::public::service_checks::Judge as skein_world::referee::Referee<temper_playground::programs::Process<temper_one_process_world::public::service::Deployment>>>::observe` |
| 20.07% | 3,716,311,520 | 137,794 | `temper_one_process_world::public::owners::Walk::ready` |
| 19.86% | 3,677,083,098 | 91,773 | `temper_one_process_world::public::owners::Walk::landed` |
| 18.68% | 3,459,221,291 | 46,596 | `<temper_playground::machine::SignalHost<H> as skein_shell::drive::Host>::iterate` |
| 15.60% | 2,888,452,085 | 23,298 | `<temper_playground::service::ServiceHost<H> as skein_shell::drive::Host>::iterate` |
| 15.60% | 2,888,335,595 | 23,298 | `<temper_playground::service::ServiceState<H> as skein_shell::drive::Host>::iterate` |
| 14.87% | 2,752,697,389 | 23,298 | `<skein_fake_peers::llm::Peer as skein_shell::drive::Host>::iterate` |
| 14.80% | 2,739,344,056 | 9,092 | `<temper_playground::machine::Rooted<H> as skein_shell::drive::Host>::iterate` |
| 14.56% | 2,695,744,645 | 23,298 | `skein_fake_peers::llm::Peer::progress` |
| 13.72% | 2,539,966,461 | 23,298 | `temper_service::system::iterate` |
| 13.64% | 2,525,871,171 | 116,490 | `temper_service::system::iterate_stage` |
| 13.50% | 2,499,938,678 | 3,310 | `skein_fake_llm_protocol::provider::up` |
| 13.48% | 2,495,016,361 | 3,337 | `skein_fake_llm_protocol::provider::resume` |
| 13.39% | 2,478,368,745 | 3,205 | `skein_fake_llm_protocol::provider::http_event` |
| 12.55% | 2,322,920,718 | 15 | `skein_fake_llm_protocol::provider::requested` |

### `identifiers` before: largest by self Ir

| Share | Ir | Function |
|---:|---:|---|
| 3.48% | 643,524,927 | `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::next_unchecked::{{closure}}` |
| 3.39% | 626,778,858 | `__memcpy_avx_unaligned_erms` |
| 3.15% | 583,062,626 | `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<BorrowType,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::next_kv` |
| 3.00% | 555,139,762 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.80% | 518,993,564 | `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<BorrowType,K,V,alloc::collections::btree::node::marker::LeafOrInternal>,alloc::collections::btree::node::marker::KV>>::next_leaf_edge` |
| 2.61% | 482,821,319 | `skein_json::writer::escaped_len` |
| 2.29% | 423,924,206 | `alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut,K,V,NodeType>,alloc::collections::btree::node::marker::KV>::into_kv` |
| 2.29% | 423,245,247 | `alloc::collections::btree::navigate::LazyLeafRange<BorrowType,K,V>::init_front` |
| 2.27% | 420,918,083 | `core::iter::traits::iterator::Iterator::try_fold` |
| 2.18% | 403,653,496 | `skein_json::writer::escaped` |
| 2.09% | 386,168,395 | `<alloc::collections::btree::map::Iter<K,V> as core::iter::traits::iterator::Iterator>::next` |
| 1.91% | 353,281,871 | `alloc::collections::btree::mem::replace` |

### `identifiers` after: largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 26.01% | 3,458,414,206 | 46,596 | `<temper_playground::machine::SignalHost<H> as skein_shell::drive::Host>::iterate` |
| 21.72% | 2,888,476,560 | 23,298 | `<temper_playground::service::ServiceHost<H> as skein_shell::drive::Host>::iterate` |
| 21.72% | 2,888,360,070 | 23,298 | `<temper_playground::service::ServiceState<H> as skein_shell::drive::Host>::iterate` |
| 20.70% | 2,752,561,128 | 23,298 | `<skein_fake_peers::llm::Peer as skein_shell::drive::Host>::iterate` |
| 20.60% | 2,739,311,224 | 9,092 | `<temper_playground::machine::Rooted<H> as skein_shell::drive::Host>::iterate` |
| 20.27% | 2,695,622,773 | 23,298 | `skein_fake_peers::llm::Peer::progress` |
| 19.10% | 2,540,039,780 | 23,298 | `temper_service::system::iterate` |
| 18.99% | 2,525,944,490 | 116,490 | `temper_service::system::iterate_stage` |
| 18.80% | 2,499,816,310 | 3,310 | `skein_fake_llm_protocol::provider::up` |
| 18.76% | 2,494,894,478 | 3,337 | `skein_fake_llm_protocol::provider::resume` |
| 18.64% | 2,478,246,243 | 3,205 | `skein_fake_llm_protocol::provider::http_event` |
| 17.47% | 2,322,856,448 | 15 | `skein_fake_llm_protocol::provider::requested` |
| 17.47% | 2,322,844,274 | 15 | `skein_fake_llm_protocol::documents::request_on` |
| 17.47% | 2,322,843,929 | 15 | `skein_fake_llm_protocol::documents::openai_request` |
| 17.40% | 2,314,432,793 | 9,092 | `<skein_sim_world::writer_pipe::PipeHost<P> as skein_shell::drive::Host>::iterate` |

### `identifiers` after: largest by self Ir

| Share | Ir | Function |
|---:|---:|---|
| 4.08% | 542,973,228 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 4.06% | 539,977,495 | `__memcpy_avx_unaligned_erms` |
| 3.63% | 482,821,319 | `skein_json::writer::escaped_len` |
| 3.04% | 403,653,496 | `skein_json::writer::escaped` |
| 2.32% | 308,282,029 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 2.22% | 294,914,329 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 2.07% | 275,649,951 | `core::result::Result<T,E>::expect` |
| 2.01% | 267,776,554 | `core::option::Option<T>::expect` |
| 1.55% | 206,717,957 | `alloc::collections::btree::search::<impl alloc::collections::btree::node::NodeRef<BorrowType,K,V,Type>>::find_key_index` |
| 1.47% | 195,316,410 | `skein_json::utf8::Utf8::next` |
| 1.42% | 188,322,760 | `alloc::collections::vec_deque::VecDeque<T,A>::pop_front` |
| 1.13% | 150,905,608 | `skein_json::utf8::lead` |

### `owners` before: largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 21.28% | 1,356,790,379 | 54,332 | `<temper_one_process_world::public::service_checks::Judge as skein_world::referee::Referee<temper_playground::programs::Process<temper_one_process_world::public::service::Deployment>>>::observe` |
| 20.67% | 1,318,058,398 | 51,174 | `temper_one_process_world::public::owners::Walk::ready` |
| 20.11% | 1,282,380,204 | 30,613 | `temper_one_process_world::public::owners::Walk::landed` |
| 18.12% | 1,155,482,016 | 15,876 | `<temper_playground::machine::SignalHost<H> as skein_shell::drive::Host>::iterate` |
| 16.13% | 1,028,426,273 | 7,938 | `<temper_playground::service::ServiceHost<H> as skein_shell::drive::Host>::iterate` |
| 16.13% | 1,028,386,583 | 7,938 | `<temper_playground::service::ServiceState<H> as skein_shell::drive::Host>::iterate` |
| 14.39% | 917,863,447 | 7,938 | `<skein_fake_peers::llm::Peer as skein_shell::drive::Host>::iterate` |
| 14.36% | 915,737,674 | 3,202 | `<temper_playground::machine::Rooted<H> as skein_shell::drive::Host>::iterate` |
| 14.09% | 898,707,882 | 7,938 | `skein_fake_peers::llm::Peer::progress` |
| 13.64% | 869,583,997 | 7,938 | `temper_service::system::iterate` |
| 13.56% | 864,781,507 | 39,690 | `temper_service::system::iterate_stage` |
| 13.07% | 833,293,859 | 1,104 | `skein_fake_llm_protocol::provider::up` |
| 13.04% | 831,652,518 | 1,113 | `skein_fake_llm_protocol::provider::resume` |
| 12.95% | 826,081,179 | 1,069 | `skein_fake_llm_protocol::provider::http_event` |
| 12.14% | 774,273,698 | 5 | `skein_fake_llm_protocol::provider::requested` |

### `owners` before: largest by self Ir

| Share | Ir | Function |
|---:|---:|---|
| 3.51% | 224,089,914 | `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::next_unchecked::{{closure}}` |
| 3.38% | 215,346,944 | `__memcpy_avx_unaligned_erms` |
| 3.19% | 203,328,486 | `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<BorrowType,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::next_kv` |
| 3.00% | 191,165,280 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.84% | 181,006,506 | `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<BorrowType,K,V,alloc::collections::btree::node::marker::LeafOrInternal>,alloc::collections::btree::node::marker::KV>>::next_leaf_edge` |
| 2.53% | 161,548,187 | `skein_json::writer::escaped_len` |
| 2.32% | 148,196,342 | `alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut,K,V,NodeType>,alloc::collections::btree::node::marker::KV>::into_kv` |
| 2.32% | 147,649,068 | `alloc::collections::btree::navigate::LazyLeafRange<BorrowType,K,V>::init_front` |
| 2.30% | 146,764,029 | `core::iter::traits::iterator::Iterator::try_fold` |
| 2.12% | 135,080,424 | `skein_json::writer::escaped` |
| 2.10% | 134,043,986 | `<alloc::collections::btree::map::Iter<K,V> as core::iter::traits::iterator::Iterator>::next` |
| 1.93% | 123,063,601 | `alloc::collections::btree::mem::replace` |

### `owners` after: largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 25.31% | 1,155,282,102 | 15,876 | `<temper_playground::machine::SignalHost<H> as skein_shell::drive::Host>::iterate` |
| 22.53% | 1,028,450,264 | 7,938 | `<temper_playground::service::ServiceHost<H> as skein_shell::drive::Host>::iterate` |
| 22.53% | 1,028,410,574 | 7,938 | `<temper_playground::service::ServiceState<H> as skein_shell::drive::Host>::iterate` |
| 20.11% | 917,903,509 | 7,938 | `<skein_fake_peers::llm::Peer as skein_shell::drive::Host>::iterate` |
| 20.06% | 915,724,248 | 3,202 | `<temper_playground::machine::Rooted<H> as skein_shell::drive::Host>::iterate` |
| 19.69% | 898,749,307 | 7,938 | `skein_fake_peers::llm::Peer::progress` |
| 19.05% | 869,618,495 | 7,938 | `temper_service::system::iterate` |
| 18.95% | 864,816,005 | 39,690 | `temper_service::system::iterate_stage` |
| 18.26% | 833,334,613 | 1,104 | `skein_fake_llm_protocol::provider::up` |
| 18.22% | 831,693,471 | 1,113 | `skein_fake_llm_protocol::provider::resume` |
| 18.10% | 826,122,951 | 1,069 | `skein_fake_llm_protocol::provider::http_event` |
| 16.96% | 774,315,599 | 5 | `skein_fake_llm_protocol::provider::requested` |
| 16.96% | 774,311,541 | 5 | `skein_fake_llm_protocol::documents::request_on` |
| 16.96% | 774,311,426 | 5 | `skein_fake_llm_protocol::documents::openai_request` |
| 16.96% | 774,133,130 | 3,202 | `<skein_sim_world::writer_pipe::PipeHost<P> as skein_shell::drive::Host>::iterate` |

### `owners` after: largest by self Ir

| Share | Ir | Function |
|---:|---:|---|
| 4.10% | 187,016,783 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 4.07% | 185,875,501 | `__memcpy_avx_unaligned_erms` |
| 3.54% | 161,548,187 | `skein_json::writer::escaped_len` |
| 2.96% | 135,080,424 | `skein_json::writer::escaped` |
| 2.31% | 105,472,534 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 2.29% | 104,723,755 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 2.04% | 93,256,437 | `core::result::Result<T,E>::expect` |
| 2.00% | 91,099,822 | `core::option::Option<T>::expect` |
| 1.68% | 76,844,755 | `alloc::collections::btree::search::<impl alloc::collections::btree::node::NodeRef<BorrowType,K,V,Type>>::find_key_index` |
| 1.43% | 65,395,514 | `skein_json::utf8::Utf8::next` |
| 1.39% | 63,675,798 | `alloc::collections::vec_deque::VecDeque<T,A>::pop_front` |
| 1.11% | 50,525,852 | `skein_json::utf8::lead` |

### `m15` before: largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 47.89% | 8,519,927,832 | 2,867 | `temper_one_process_world::m15::observer::Observer::check_since` |
| 47.89% | 8,519,744,344 | 5,734 | `temper_one_process_world::m15::observer::Observer::leak_since` |
| 46.78% | 8,322,646,235 | 5 | `temper_one_process_world::m15::observer::Observer::scan` |
| 45.35% | 8,069,209,405 | 8,508 | `<temper_one_process_world::m15::run::Judge as skein_world::referee::Referee<temper_playground::programs::Process<temper_one_process_world::world::OneProcess>>>::act` |
| 44.71% | 7,955,227,685 | 4 | `temper_one_process_world::m15::checkpoint` |
| 44.41% | 7,900,749,445 | 4 | `temper_one_process_world::m15::observer::checkpoint` |
| 25.12% | 4,468,542,468 | 25,524 | `<temper_playground::machine::SignalHost<H> as skein_shell::drive::Host>::iterate` |
| 16.72% | 2,974,300,967 | 8,508 | `<temper_one_process_world::world::Person as skein_shell::drive::Host>::iterate` |
| 14.27% | 2,538,886,559 | 149,973 | `<skein_ui::tree::Node as core::fmt::Debug>::fmt` |
| 8.42% | 1,498,644,099 | 437 | `temper_one_process_world::m15::observer::Observer::check` |
| 7.22% | 1,283,650,317 | 8,508 | `<skein_fake_peers::llm::Peer as skein_shell::drive::Host>::iterate` |
| 7.09% | 1,260,679,442 | 8,508 | `skein_fake_peers::llm::Peer::progress` |
| 6.72% | 1,195,354,299 | 8,508 | `<temper_playground::service::ServiceHost<H> as skein_shell::drive::Host>::iterate` |
| 6.72% | 1,195,311,759 | 8,508 | `<temper_playground::service::ServiceState<H> as skein_shell::drive::Host>::iterate` |
| 6.62% | 1,177,442,420 | 8,071 | `<temper_playground::machine::Rooted<H> as skein_shell::drive::Host>::iterate` |

### `m15` before: largest by self Ir

| Share | Ir | Function |
|---:|---:|---|
| 20.54% | 3,654,723,760 | `core::str::pattern::TwoWaySearcher::next` |
| 4.75% | 845,832,105 | `__memcpy_avx_unaligned_erms` |
| 4.75% | 844,425,854 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 4.48% | 797,150,427 | `<alloc::string::String as core::fmt::Write>::write_str` |
| 3.94% | 701,046,558 | `<core::str::pattern::StrSearcher>::new` |
| 3.90% | 694,625,117 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 2.93% | 521,255,136 | `core::num::<impl usize>::unchecked_add::precondition_check` |
| 1.84% | 327,950,156 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 1.84% | 326,660,456 | `core::ub_checks::maybe_is_nonoverlapping::runtime` |
| 1.82% | 324,461,102 | `skein_lib::bytes::TwoWay::find` |
| 1.57% | 279,198,080 | `core::core_arch::x86::sse2::_mm_loadu_si128` |
| 1.51% | 268,852,314 | `<core::str::pattern::MatchOnly as core::str::pattern::TwoWayStrategy>::use_early_reject` |

### `m15` after: largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 35.20% | 4,457,096,083 | 25,524 | `<temper_playground::machine::SignalHost<H> as skein_shell::drive::Host>::iterate` |
| 30.33% | 3,840,375,572 | 8,508 | `<temper_one_process_world::m15::run::Judge as skein_world::referee::Referee<temper_playground::programs::Process<temper_one_process_world::world::OneProcess>>>::act` |
| 29.55% | 3,741,339,177 | 2,867 | `temper_one_process_world::m15::observer::Observer::check_since` |
| 29.55% | 3,741,155,689 | 5,734 | `temper_one_process_world::m15::observer::Observer::leak_since` |
| 29.43% | 3,726,388,358 | 4 | `temper_one_process_world::m15::checkpoint` |
| 29.38% | 3,719,560,054 | 5 | `temper_one_process_world::m15::observer::Observer::scan` |
| 29.00% | 3,671,942,281 | 4 | `temper_one_process_world::m15::observer::checkpoint` |
| 23.40% | 2,962,865,559 | 8,508 | `<temper_one_process_world::world::Person as skein_shell::drive::Host>::iterate` |
| 22.25% | 2,817,394,268 | 2,944 | `<aho_corasick::ahocorasick::FindOverlappingIter as core::iter::traits::iterator::Iterator>::next` |
| 22.25% | 2,817,197,020 | 2,944 | `aho_corasick::automaton::Automaton::try_find_overlapping` |
| 22.25% | 2,817,170,524 | 2,944 | `aho_corasick::automaton::try_find_overlapping_fwd` |
| 19.96% | 2,527,490,769 | 149,973 | `<skein_ui::tree::Node as core::fmt::Debug>::fmt` |
| 10.14% | 1,283,634,798 | 8,508 | `<skein_fake_peers::llm::Peer as skein_shell::drive::Host>::iterate` |
| 9.96% | 1,260,662,447 | 8,508 | `skein_fake_peers::llm::Peer::progress` |
| 9.44% | 1,195,293,373 | 8,508 | `<temper_playground::service::ServiceHost<H> as skein_shell::drive::Host>::iterate` |

### `m15` after: largest by self Ir

| Share | Ir | Function |
|---:|---:|---|
| 7.99% | 1,011,241,025 | `aho_corasick::automaton::try_find_overlapping_fwd` |
| 6.38% | 807,932,030 | `__memcpy_avx_unaligned_erms` |
| 6.30% | 797,150,427 | `<alloc::string::String as core::fmt::Write>::write_str` |
| 3.54% | 448,730,456 | `<usize as core::slice::index::SliceIndex<[T]>>::index` |
| 2.89% | 365,484,568 | `<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index` |
| 2.63% | 332,903,233 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.09% | 264,757,752 | `<core::fmt::builders::DebugStruct>::field` |
| 1.65% | 208,341,693 | `skein_json::writer::escaped_len` |
| 1.36% | 171,572,925 | `skein_json::writer::escaped` |
| 1.35% | 170,650,452 | `core::cmp::PartialOrd::le` |
| 1.19% | 150,386,344 | `<core::fmt::Formatter>::pad_integral` |
| 1.10% | 139,502,504 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |

## Left

These measured footprints are below the world and belong to `engine-causes`. No code in those crates changed. They are not savings claims.

| Where / function | Identifier after Ir / share | Owner after Ir / share | M15 after Ir / share | Owner |
|---|---:|---:|---:|---|
| `smith/crates/smith-protocol-llm/src/component.rs`: `Component::start` | 2,035,702,252 / 15.31% | 678,667,027 / 14.87% | 1,005,610,350 / 7.94% | `engine-causes` |
| `temper/crates/temper-engine-domain/src/deadline.rs`: `Domain::next_deadline` | 671,174,344 / 5.05% | 231,673,466 / 5.08% | 240,538,559 / 1.90% | `engine-causes` |
| `skein/crates/skein-json/src/parse.rs`: `parse` | 1,879,539,174 / 14.13% | 635,589,246 / 13.92% | 885,466,644 / 6.99% | `engine-causes` |

The identifier story remains at 4.034 native CPU s and M15 at 2.602 s, above the one-process two-second guide. Owners is 1.388 s. `close` carries these figures to the user. No test was merged, moved, weakened or named an exception. Isolated story measurements do not establish the whole tree’s suite CPU total.

## Stopped at

The next public fixture-construction candidate, `fixtures::engine`, is 236,030,770 Ir across four calls: 1.77% of the final identifier story even if all its work disappeared. It stays. The two owner fixture parses total 118,012,422 Ir; if one of these similar parses were reused, the estimated saving is about 59 million Ir (1.29%). M15’s `Observer::context` is 133,599,685 Ir (1.06%). Larger parent wrappers alone do not establish avoidable work. The measured matcher-group trial regressed by 7.37%, so one cached matcher remains. Stop here: the next demonstrated local saving is under about 2%; deployment sharing is constrained above, and the remaining product costs are `engine-causes`’s.
