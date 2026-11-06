# Hearth focused suite: CPU and instruction profile

## Before

Code base: `39b490f13846b81fe78c5834c7852ffd28a25155`. Measured on branch `hearth-speed-2/profile`, HEAD `8d07171e3c436b7ef77cc0efa93c31ace08d3a06`; its only differences from the code base are this plan's files. No checkout, build setting, test, assertion or seed changed. Machine: `debian-32gb-hel1-11`, from `heavy hostname`; date: 2026-11-06 UTC.

Method: the unmodified throwaway collector driver from `hearth/docs/plan/hearth-speed-2/notes/common.md`, “The method”, using `tools/tiers/design.md`, 7.3 and 10.2. Each test's user plus system CPU includes waited and reaped descendants; compilation, listing, nextest and the accounting supervisor are excluded. `NEXTEST_DOUBLE_SPAWN=0`, `TEMPER_BUNDLE=none`, default profile. Both runs passed every test, with no starvation, unobserved tests, leaks, incomplete attempts or retries. These are builder measurements; the queue's 30.5 CPU s is a different measurement and is not compared numerically with these.

| Run | User s | System s | CPU s | Suite wall s | Tests |
| --- | ---: | ---: | ---: | ---: | ---: |
| base-1 | 28.872563 | 7.255428 | 36.127991 | 2.854501 | 2189 |
| base-2 | 29.788955 | 7.759747 | 37.548702 | 2.987484 | 2189 |

The remaining before tables use the lower CPU run, **base-1**. Its 42.019734 s summed per-test wall time is different from the 2.854501 s suite wall span: tests run concurrently. The inherited suite is over its 30 CPU s budget; this task records causes and changes no code.

### By family

| Family | Tests | User s | System s | CPU s | Suite CPU |
| --- | ---: | ---: | ---: | ---: | ---: |
| Builder suite | 106 | 9.023537 | 1.131725 | 10.155262 | 28.11% |
| Tree suite | 54 | 8.978199 | 0.197561 | 9.175760 | 25.40% |
| Real git | 183 | 3.422426 | 3.557110 | 6.979536 | 19.32% |
| All the rest | 1737 | 4.234002 | 2.090007 | 6.324009 | 17.50% |
| Render suite | 109 | 3.214399 | 0.279025 | 3.493424 | 9.67% |

### By binary

| Binary ID | Tests | User s | System s | CPU s |
| --- | ---: | ---: | ---: | ---: |
| `hearth-builder-world::suite` | 106 | 9.023537 | 1.131725 | 10.155262 |
| `hearth-tree-world::suite` | 54 | 8.978199 | 0.197561 | 9.175760 |
| `hearth-builder-world::real` | 183 | 3.422426 | 3.557110 | 6.979536 |
| `hearth-render-world::suite` | 109 | 3.214399 | 0.279025 | 3.493424 |
| `hearth-domain-render` | 619 | 0.911572 | 0.803192 | 1.714764 |
| `hearth-mmdebstrap-world::suite` | 23 | 0.671542 | 0.036070 | 0.707612 |
| `hearth-config-world::suite` | 22 | 0.584686 | 0.045447 | 0.630133 |
| `hearth-protocol-builder` | 195 | 0.220381 | 0.211192 | 0.431573 |
| `hearth-git-world::suite` | 20 | 0.364885 | 0.032897 | 0.397782 |
| `hearth-client-protocol-records` | 154 | 0.168937 | 0.120324 | 0.289261 |
| `hearth::end_to_end` | 4 | 0.114750 | 0.150517 | 0.265267 |
| `hearth-template-world::suite` | 9 | 0.238673 | 0.018250 | 0.256923 |
| `hearth-protocol-config` | 88 | 0.161084 | 0.075084 | 0.236168 |
| `hearth-protocol-template` | 86 | 0.111595 | 0.061849 | 0.173444 |
| `hearth-protocol-tree` | 75 | 0.074304 | 0.078218 | 0.152522 |
| `hearth-fake-machine` | 74 | 0.074725 | 0.074466 | 0.149191 |
| `hearth-protocol-accounts` | 21 | 0.108592 | 0.019594 | 0.128186 |
| `hearth-domain-tree` | 53 | 0.062345 | 0.039123 | 0.101468 |
| `hearth-domain-store` | 55 | 0.050081 | 0.049525 | 0.099606 |
| `hearth-protocol-mmdebstrap` | 51 | 0.056823 | 0.039318 | 0.096141 |
| `hearth-fake-mmdebstrap` | 27 | 0.039051 | 0.035836 | 0.074887 |
| `hearth-protocol-git` | 35 | 0.038774 | 0.024557 | 0.063331 |
| `hearth-domain-builder` | 17 | 0.021227 | 0.039227 | 0.060454 |
| `hearth::refusals` | 2 | 0.026176 | 0.030145 | 0.056321 |
| `hearth-fake-git` | 25 | 0.028194 | 0.023746 | 0.051940 |
| `hearth-test-mirror` | 19 | 0.030780 | 0.017885 | 0.048665 |
| `hearth-lib` | 24 | 0.020139 | 0.024428 | 0.044567 |
| `hearth-service-builder` | 8 | 0.005547 | 0.020844 | 0.026391 |
| `hearth` | 15 | 0.012779 | 0.008341 | 0.021120 |
| `hearth-domain-build` | 7 | 0.011932 | 0.004441 | 0.016373 |
| `hearth-accounts-world::suite` | 1 | 0.011244 | 0.003748 | 0.014992 |
| `hearth-recordings` | 5 | 0.009682 | 0.000000 | 0.009682 |
| `hearth-conformance-world::suite` | 3 | 0.003502 | 0.001743 | 0.005245 |

34 hearth packages were selected; 33 binaries executed focused tests. Packages or binaries with no selected test do not contribute an attempt.

### Twenty most costly tests

| Binary ID / test | User s | System s | CPU s |
| --- | ---: | ---: | ---: |
| `hearth-tree-world::suite / memory::walk::walking_wide_and_deep_layers_holds_no_more_than_its_ceiling` | 1.094285 | 0.019896 | 1.114181 |
| `hearth-tree-world::suite / walk::each_edit_alone_walks_to_the_oracles_digests_and_file` | 0.900703 | 0.000000 | 0.900703 |
| `hearth-builder-world::suite / domain::a_step_that_fails_in_any_layer_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath` | 0.798651 | 0.000000 | 0.798651 |
| `hearth-builder-world::suite / domain::a_changed_config_part_builds_config_alone_staged_from_the_published_software_layer` | 0.768377 | 0.004001 | 0.772378 |
| `hearth-builder-world::suite / domain::a_helper_request_that_fails_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath` | 0.737854 | 0.007976 | 0.745830 |
| `hearth-tree-world::suite / walk::an_io_failure_at_the_parent_stream_ends_the_walk_and_never_panics` | 0.704350 | 0.000000 | 0.704350 |
| `hearth-tree-world::suite / memory::walk::walking_generated_layers_against_a_parent_holds_no_more_than_its_ceiling` | 0.660351 | 0.004002 | 0.664353 |
| `hearth-tree-world::suite / hash::wide_and_deep_layers_hash_to_the_oracles` | 0.647707 | 0.011994 | 0.659701 |
| `hearth-tree-world::suite / walk::generated_edits_walk_to_the_oracles_digests_and_file` | 0.462582 | 0.007975 | 0.470557 |
| `hearth-builder-world::suite / domain::a_second_build_of_the_same_spec_stages_and_runs_nothing` | 0.457930 | 0.004052 | 0.461982 |
| `hearth-tree-world::suite / trees::a_change_for_another_path_is_refused` | 0.432340 | 0.000000 | 0.432340 |
| `hearth-builder-world::suite / goldens::each_error_case_prints_its_errors_after_the_healthy_spec` | 0.299877 | 0.129691 | 0.429568 |
| `hearth-tree-world::suite / walk::wide_and_deep_layers_walk_against_themselves_and_their_edits` | 0.407721 | 0.003997 | 0.411718 |
| `hearth-tree-world::suite / hash::generated_layers_hash_to_the_oracles_digests_and_file` | 0.389311 | 0.003972 | 0.393283 |
| `hearth-builder-world::suite / domain::a_build_from_an_empty_store_publishes_the_empty_packages_software_and_config_layers` | 0.370153 | 0.003980 | 0.374133 |
| `hearth-tree-world::suite / walk::a_walk_with_nothing_changed_is_no_change_with_the_parents_tree_digest_and_nothing_hashed` | 0.369605 | 0.003974 | 0.373579 |
| `hearth-tree-world::suite / hash::each_bound_fails_at_its_path` | 0.307170 | 0.028292 | 0.335462 |
| `hearth-tree-world::suite / memory::walk::walking_a_layer_at_its_limits_against_a_parent_holds_no_more_than_its_ceiling` | 0.327232 | 0.007981 | 0.335213 |
| `hearth-builder-world::suite / properties::a_production_edit_leaves_the_lab_specs_through_real_git` | 0.196546 | 0.127528 | 0.324074 |
| `hearth-builder-world::suite / protocol::a_full_pipe_holds_its_writer_until_read` | 0.309262 | 0.000000 | 0.309262 |

## Profile

Valgrind/Callgrind 3.24.0, `--trace-children=yes`, one exact test at a time through nextest's tracer, which removes nextest timeouts and supplies the runtime paths. Output is under `target/hs2/`; every test runs in its package directory and uses an absolute output filename. Self and inclusive annotations use `--auto=no --threshold=100` (the full-coverage extension of the task's 99% skim) so the self buckets reconcile to the complete instruction count. `Ir` counts user-space instructions, **not kernel time**. `strace` is absent: syscall counts and the causes of system CPU are **unknown**.

Samples are ranks 1–3, the lower middle test (descending rank floor(n/2)+1), and the cheapest test in base-1. All the rest also samples ranks 4–5 because it spans multiple binaries. Test CPU and system CPU below always come from base-1, not the much slower instrumented run. Family coverage is the sum of those collected test CPU values divided by the family's CPU. No sample is claimed to represent every test.

Buckets assign annotated self instructions by their source file, with explicit precedence for the loader, allocator/counting allocator, hashing and libtest harness. Hearth tests inside a production crate are test code. Compiler allocator wrappers are allocator cost. Rust core/alloc/std includes generic work called from hearth or skein; it is not evidence of ownership until the inclusive call chain is read. Every sample also has exact test-function before/after dumps: all pre-entry and post-return instructions except loader instructions go to the two harness buckets, including Rust/libc descendants. The body retains source attribution; libtest instructions within it have a separate direct self bucket. Loader includes late resolution and teardown as well as startup. Skein's own SHA-256 remains in the skein/Rust buckets, not the external sha1/sha2 bucket. Every residual category is shown, including ones below 1%.

Function tables pool the sampled processes' annotated rows: **self** rows partition instructions, **inclusive** rows overlap callers and callees and must never be summed. Self costs combine all source records of the same function. Inclusive costs use the largest source view of each function per process, then pool processes, so inlined source views are not added twice. The source column prefers a repository definition when available, otherwise the largest source contributor; Callgrind's numbered symbol variants remain distinct. Values are measured pooled counts, not extrapolated family counts. The twenty inclusive rows are restricted to hearth/skein functions, excluding compiler-generated allocator entry points.

Family CPU shares in “What it found” are explicitly **estimates**: for a function, weight each sample's inclusive instruction fraction by that sample's collected user CPU, divide by sampled user CPU, then multiply by the family's user CPU. Divide that estimated cost by family/suite CPU for the two shares. Thus no kernel CPU is assigned by an instruction profile. The sample is deliberately selected, not random; extrapolation error is unknown, especially for the heterogeneous rest. Overlapping inclusive costs are not additive.

Callgrind's incremental dumps use `calls=0` for active call edges spanning a dump. This annotation version otherwise treats their cost as self instructions. Analysis-only copies replace those zero call counts with one before annotation; **Ir and the original files are unchanged**, and call counts are not used. Every process's self rows and buckets then sum exactly to its recorded Ir. No inherited pre-exec dump is included. Function-boundary Ir is used consistently below; pure in-memory samples agree with the original unsplit counts to within thousands of instructions, while real I/O scheduling can change the executed trace.

Some git/zlib functions have only addresses: their function names and source lines are **unknown**, because symbols are unavailable. Address rows retain the loaded object where known, and zlib/unresolved costs get separate buckets. No debug packages were installed.

### Builder suite

5 tests sampled; collected CPU 2.355603 s covers **23.20%** of this family. Recorded pool: **19,171,225,918 Ir**. All sampled tests passed under Callgrind.

| Rank / binary / test | CPU s | System s | Recorded Ir | Family CPU covered |
| --- | --- | --- | --- | --- |
| 1 / `hearth-builder-world::suite / domain::a_step_that_fails_in_any_layer_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath` | 0.798651 | 0.000000 | 6,344,688,149 | 7.864% |
| 2 / `hearth-builder-world::suite / domain::a_changed_config_part_builds_config_alone_staged_from_the_published_software_layer` | 0.772378 | 0.004001 | 6,539,678,437 | 7.606% |
| 3 / `hearth-builder-world::suite / domain::a_helper_request_that_fails_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath` | 0.745830 | 0.007976 | 6,104,594,108 | 7.344% |
| 54 / `hearth-builder-world::suite / simulation::diff_resolve_resolves_each_changed_key_once_in_turn_before_printing` | 0.036307 | 0.004034 | 179,397,879 | 0.358% |
| 106 / `hearth-builder-world::suite / goldens::the_keys_line_is_the_one_hearth_24_pins` | 0.002437 | 0.000000 | 2,867,345 | 0.024% |

**By bucket, self Ir.** Complete measured pool; no CPU extrapolation.

| Bucket | Self Ir | Recorded Ir |
| --- | --- | --- |
| Rust core | 9,711,135,154 | 50.655% |
| Rust alloc | 7,078,291,558 | 36.921% |
| Hearth tests and kits | 788,253,608 | 4.112% |
| Allocator (including counting allocator) | 578,919,937 | 3.020% |
| libc / system libraries (other) | 333,694,707 | 1.741% |
| Rust std | 326,974,452 | 1.706% |
| Skein (excluding counting allocator) | 222,522,423 | 1.161% |
| Hearth production | 77,074,476 | 0.402% |
| Hashing (sha1 / sha2) | 20,664,693 | 0.108% |
| Process startup / loader | 10,173,442 | 0.053% |
| Unresolved | 9,906,709 | 0.052% |
| Other | 7,773,514 | 0.041% |
| Harness before test | 3,666,511 | 0.019% |
| Harness after test | 2,174,734 | 0.011% |
| Harness (direct self instructions) | 0 | 0.000% |

**Twenty hottest functions by self Ir.**

| Function | Source | Self Ir | Recorded Ir |
| --- | --- | --- | --- |
| `alloc::collections::btree::search::<impl alloc::collections::btree::node::NodeRef<BorrowType,K,V,Type>>::find_key_index` | `hearth/testing/hearth-fake-machine/src/lib.rs` | 2,531,034,969 | 13.202% |
| `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` | `Rust library/core/src/iter/adapters/enumerate.rs` | 2,331,548,677 | 12.162% |
| `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` | `hearth/testing/hearth-fake-machine/src/lib.rs` | 1,242,592,968 | 6.482% |
| `alloc::collections::btree::search::<impl alloc::collections::btree::node::NodeRef<BorrowType,K,V,alloc::collections::btree::node::marker::LeafOrInternal>>::search_tree` | `hearth/testing/hearth-fake-machine/src/lib.rs` | 922,070,676 | 4.810% |
| `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<BorrowType,K,V,alloc::collections::btree::node::marker::LeafOrInternal>,alloc::collections::btree::node::marker::KV>>::next_leaf_edge` | `Rust library/alloc/src/collections/btree/node.rs` | 580,867,683 | 3.030% |
| `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::next_unchecked::{{closure}}` | `Rust library/alloc/src/collections/btree/navigate.rs` | 550,763,298 | 2.873% |
| `core::cmp::impls::<impl core::cmp::Ord for u64>::cmp` | `Rust library/core/src/cmp.rs` | 535,586,319 | 2.794% |
| `alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut,K,V,NodeType>,alloc::collections::btree::node::marker::KV>::into_kv` | `Rust library/alloc/src/collections/btree/node.rs` | 510,376,045 | 2.662% |
| `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<BorrowType,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::next_kv` | `hearth/testing/hearth-fake-machine/src/lib.rs` | 499,244,592 | 2.604% |
| `<core::ops::range::Range<usize> as core::slice::index::SliceIndex<[T]>>::get_unchecked::precondition_check` | `Rust library/core/src/ub_checks.rs` | 441,101,310 | 2.301% |
| `<usize as core::slice::index::SliceIndex<[T]>>::get_unchecked::precondition_check` | `Rust library/core/src/ub_checks.rs` | 439,521,786 | 2.293% |
| `alloc::collections::btree::navigate::LazyLeafRange<BorrowType,K,V>::init_front` | `Rust library/alloc/src/collections/btree/navigate.rs` | 433,021,674 | 2.259% |
| `alloc::collections::btree::mem::replace` | `Rust library/alloc/src/collections/btree/mem.rs` | 432,326,588 | 2.255% |
| `<hearth_fake_machine::linux::Node as core::cmp::Ord>::cmp` | `hearth/testing/hearth-fake-machine/src/linux.rs` | 292,128,615 | 1.524% |
| `<T as core::borrow::Borrow<T>>::borrow` | `Rust library/core/src/borrow.rs` | 284,561,940 | 1.484% |
| `<alloc::collections::btree::map::Iter<K,V> as core::iter::traits::iterator::Iterator>::next` | `Rust library/alloc/src/collections/btree/map.rs` | 282,646,778 | 1.474% |
| `alloc::collections::btree::map::BTreeMap<K,V,A>::get` | `Rust library/alloc/src/collections/btree/map.rs` | 257,339,171 | 1.342% |
| `<alloc::collections::btree::dedup_sorted_iter::DedupSortedIter<K,V,I> as core::iter::traits::iterator::Iterator>::next` | `hearth/testing/hearth-fake-machine/src/lib.rs` | 219,562,726 | 1.145% |
| `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Dying,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::deallocating_next` | `hearth/testing/hearth-fake-machine/src/lib.rs` | 217,442,526 | 1.134% |
| `hearth_fake_machine::linux::ledger::<impl hearth_fake_machine::linux::Machine>::call` | `hearth/testing/hearth-fake-machine/src/linux/ledger.rs` | 214,007,393 | 1.116% |

**Twenty heaviest hearth or skein functions by inclusive Ir.**

| Function | Source | Inclusive Ir | Recorded Ir |
| --- | --- | --- | --- |
| `hearth_builder_world::domain::Run::finish` | `hearth/tests/builder/src/domain.rs` | 17,321,600,436 | 90.352% |
| `hearth_builder_world::fixtures::worlds::Shared::answer` | `hearth/tests/builder/src/fixtures/worlds.rs` | 16,924,546,907 | 88.281% |
| `hearth_builder_world::files::Files::call` | `hearth/tests/builder/src/files.rs` | 16,858,718,931 | 87.938% |
| `hearth_fake_machine::behind::answer` | `hearth/testing/hearth-fake-machine/src/behind.rs` | 16,787,606,438 | 87.567% |
| `hearth_fake_machine::behind::Behind::call` | `hearth/testing/hearth-fake-machine/src/behind.rs` | 16,760,932,465 | 87.428% |
| `hearth_fake_machine::hearth::Machine::call` | `hearth/testing/hearth-fake-machine/src/hearth.rs` | 16,756,792,918 | 87.406% |
| `hearth_fake_machine::linux::ledger::<impl hearth_fake_machine::linux::Machine>::call` | `hearth/testing/hearth-fake-machine/src/linux/ledger.rs` | 16,753,517,868 | 87.389% |
| `hearth_builder_world::domain::Run::answer` | `hearth/tests/builder/src/domain.rs` | 14,992,214,055 | 78.202% |
| `suite::domain::each_failure` | `hearth/tests/builder/tests/suite/domain.rs` | 12,443,342,263 | 64.906% |
| `suite::domain::fails_cleanly` | `hearth/tests/builder/tests/suite/domain.rs` | 11,786,218,161 | 61.479% |
| `hearth_builder_world::domain::store::Store::answer` | `hearth/tests/builder/src/domain/store.rs` | 11,117,053,919 | 57.988% |
| `hearth_builder_world::domain::layers::Layers::helper` | `hearth/tests/builder/src/domain/layers.rs` | 8,759,272,288 | 45.690% |
| `suite::domain::a_changed_config_part_builds_config_alone_staged_from_the_published_software_layer::{{closure}}` | `hearth/tests/builder/tests/suite/domain.rs` | 6,536,753,944 | 34.097% |
| `suite::domain::a_changed_config_part_builds_config_alone_staged_from_the_published_software_layer` | `hearth/tests/builder/tests/suite/domain.rs` | 6,536,753,909 | 34.097% |
| `hearth_builder_world::fixtures::worlds::each_settled` | `hearth/tests/builder/src/fixtures/worlds.rs` | 6,536,738,061 | 34.097% |
| `hearth_builder_world::fixtures::worlds::each_scenario` | `hearth/tests/builder/src/fixtures/worlds.rs` | 6,536,738,052 | 34.097% |
| `hearth_builder_world::fixtures::worlds::each_settled::{{closure}}` | `hearth/tests/builder/src/fixtures/worlds.rs` | 6,506,515,562 | 33.939% |
| `suite::domain::a_step_that_fails_in_any_layer_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath::{{closure}}` | `hearth/tests/builder/tests/suite/domain.rs` | 6,341,718,502 | 33.079% |
| `suite::domain::a_step_that_fails_in_any_layer_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath` | `hearth/tests/builder/tests/suite/domain.rs` | 6,341,718,467 | 33.079% |
| `suite::domain::a_helper_request_that_fails_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath::{{closure}}` | `hearth/tests/builder/tests/suite/domain.rs` | 6,101,623,841 | 31.827% |

### Tree suite

5 tests sampled; collected CPU 2.765453 s covers **30.14%** of this family. Recorded pool: **21,647,700,800 Ir**. All sampled tests passed under Callgrind.

| Rank / binary / test | CPU s | System s | Recorded Ir | Family CPU covered |
| --- | --- | --- | --- | --- |
| 1 / `hearth-tree-world::suite / memory::walk::walking_wide_and_deep_layers_holds_no_more_than_its_ceiling` | 1.114181 | 0.019896 | 10,434,800,367 | 12.143% |
| 2 / `hearth-tree-world::suite / walk::each_edit_alone_walks_to_the_oracles_digests_and_file` | 0.900703 | 0.000000 | 6,264,714,804 | 9.816% |
| 3 / `hearth-tree-world::suite / walk::an_io_failure_at_the_parent_stream_ends_the_walk_and_never_panics` | 0.704350 | 0.000000 | 4,645,375,557 | 7.676% |
| 28 / `hearth-tree-world::suite / trees::bytes_after_the_end_are_refused_inside_the_header_and_outside_it` | 0.044115 | 0.000000 | 301,184,386 | 0.481% |
| 54 / `hearth-tree-world::suite / hash::an_item_the_writer_refuses_fails_the_walk_naming_the_batchs_first_path` | 0.002104 | 0.002104 | 1,625,686 | 0.023% |

**By bucket, self Ir.** Complete measured pool; no CPU extrapolation.

| Bucket | Self Ir | Recorded Ir |
| --- | --- | --- |
| Rust core | 9,803,630,364 | 45.287% |
| Allocator (including counting allocator) | 2,473,347,435 | 11.425% |
| Skein (excluding counting allocator) | 2,435,274,860 | 11.250% |
| Rust alloc | 1,601,503,472 | 7.398% |
| Hearth production | 1,558,404,793 | 7.199% |
| Hearth tests and kits | 1,316,265,939 | 6.080% |
| Rust std | 1,315,802,697 | 6.078% |
| libc / system libraries (other) | 1,109,670,471 | 5.126% |
| Unresolved | 25,853,711 | 0.119% |
| Harness before test | 3,563,755 | 0.016% |
| Process startup / loader | 2,963,279 | 0.014% |
| Harness after test | 872,672 | 0.004% |
| Other | 547,352 | 0.003% |
| Hashing (sha1 / sha2) | 0 | 0.000% |
| Harness (direct self instructions) | 0 | 0.000% |

**Twenty hottest functions by self Ir.**

| Function | Source | Self Ir | Recorded Ir |
| --- | --- | --- | --- |
| `skein_lib::sha256::compress` | `skein/crates/skein-lib/src/sha256.rs` | 1,748,177,100 | 8.076% |
| `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` | `hearth/tests/tree/src/lib.rs` | 1,128,662,884 | 5.214% |
| `hearth_tree_world::fs::Fs::links` | `hearth/tests/tree/src/fs.rs` | 785,398,818 | 3.628% |
| `core::ptr::const_ptr::<impl *const T>::is_aligned_to` | `Rust library/core/src/num/uint_macros.rs` | 712,702,540 | 3.292% |
| `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::find` | `hearth/tests/tree/src/lib.rs` | 691,426,774 | 3.194% |
| `std::thread::local::LocalKey<T>::try_with` | `Rust library/std/src/thread/local.rs` | 618,691,333 | 2.858% |
| `core::ptr::copy_nonoverlapping::precondition_check` | `skein/testing/skein-heap/src/lib.rs` | 595,485,159 | 2.751% |
| `__memcmp_avx2_movbe` | `./string/../sysdeps/x86_64/multiarch/memcmp-avx2-movbe.S` | 550,019,184 | 2.541% |
| `core::ops::function::FnOnce::call_once'2` | `Rust library/core/src/ops/function.rs` | 541,105,903 | 2.500% |
| `__memcpy_avx_unaligned_erms` | `./string/../sysdeps/x86_64/multiarch/memmove-vec-unaligned-erms.S` | 517,069,779 | 2.389% |
| `alloc::vec::partial_eq::<impl core::cmp::PartialEq<[U]> for alloc::vec::Vec<T,A>>::eq` | `Rust library/alloc/src/vec/partial_eq.rs` | 514,520,050 | 2.377% |
| `skein_heap::padded` | `skein/testing/skein-heap/src/lib.rs` | 391,106,457 | 1.807% |
| `hearth_protocol_tree::decode::Decoder::item` | `hearth/crates/hearth-protocol-tree/src/decode.rs` | 372,623,424 | 1.721% |
| `std::thread::local::LocalKey<T>::initialize_with` | `Rust library/std/src/thread/local.rs` | 358,195,380 | 1.655% |
| `core::alloc::layout::Layout::from_size_align` | `Rust library/core/src/alloc/layout.rs` | 311,559,381 | 1.439% |
| `core::ptr::alignment::Alignment::new_unchecked::precondition_check` | `Rust library/core/src/num/uint_macros.rs` | 299,150,130 | 1.382% |
| `hearth_tree_world::fs::Fs::lookup::{{closure}}` | `hearth/tests/tree/src/fs.rs` | 298,734,652 | 1.380% |
| `<A as core::slice::cmp::SlicePartialEq<B>>::equal_same_length` | `Rust library/core/src/slice/cmp.rs` | 293,619,383 | 1.356% |
| `<usize as core::slice::index::SliceIndex<[T]>>::index` | `Rust library/core/src/slice/index.rs` | 290,391,550 | 1.341% |
| `core::ub_checks::maybe_is_nonoverlapping::runtime` | `Rust library/core/src/intrinsics/mod.rs` | 270,106,108 | 1.248% |

**Twenty heaviest hearth or skein functions by inclusive Ir.**

| Function | Source | Inclusive Ir | Recorded Ir |
| --- | --- | --- | --- |
| `hearth_tree_world::helper::Helper::answer` | `hearth/tests/tree/src/helper.rs` | 13,836,336,275 | 63.916% |
| `suite::memory::walk::walking_wide_and_deep_layers_holds_no_more_than_its_ceiling::{{closure}}` | `hearth/tests/tree/tests/suite/memory.rs` | 10,433,175,462 | 48.195% |
| `suite::memory::walk::walking_wide_and_deep_layers_holds_no_more_than_its_ceiling` | `hearth/tests/tree/tests/suite/memory.rs` | 10,433,175,427 | 48.195% |
| `suite::memory::walk::peak` | `hearth/tests/tree/tests/suite/memory.rs` | 10,399,214,636 | 48.038% |
| `suite::memory::walk::peak_against` | `hearth/tests/tree/tests/suite/memory.rs` | 10,399,214,608 | 48.038% |
| `hearth_tree_world::helper::walk_with` | `hearth/tests/tree/src/helper.rs` | 10,073,865,993 | 46.536% |
| `hearth_tree_world::helper::walk` | `hearth/tests/tree/src/helper.rs` | 6,585,529,189 | 30.421% |
| `suite::walk::each_edit_alone_walks_to_the_oracles_digests_and_file::{{closure}}` | `hearth/tests/tree/tests/suite/walk.rs` | 6,263,191,453 | 28.932% |
| `suite::walk::each_edit_alone_walks_to_the_oracles_digests_and_file` | `hearth/tests/tree/tests/suite/walk.rs` | 6,263,191,418 | 28.932% |
| `hearth_tree_world::against::checked` | `hearth/tests/tree/src/against.rs` | 5,708,816,308 | 26.371% |
| `suite::walk::an_io_failure_at_the_parent_stream_ends_the_walk_and_never_panics::{{closure}}` | `hearth/tests/tree/tests/suite/walk.rs` | 4,643,952,178 | 21.452% |
| `suite::walk::an_io_failure_at_the_parent_stream_ends_the_walk_and_never_panics` | `hearth/tests/tree/tests/suite/walk.rs` | 4,643,952,143 | 21.452% |
| `hearth_protocol_tree::tree::read::Reader::next` | `hearth/crates/hearth-protocol-tree/src/tree/read.rs` | 4,551,204,802 | 21.024% |
| `hearth_protocol_tree::tree::read::Reader::take` | `hearth/crates/hearth-protocol-tree/src/tree/read.rs` | 4,521,657,268 | 20.887% |
| `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::alloc` | `skein/testing/skein-heap/src/lib.rs` | 3,920,488,591 | 18.110% |
| `hearth_tree_world::helper::Helper::page` | `hearth/tests/tree/src/helper.rs` | 3,643,811,446 | 16.832% |
| `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::dealloc` | `skein/testing/skein-heap/src/lib.rs` | 3,517,392,208 | 16.248% |
| `hearth_tree_world::fs::Fs::stat` | `hearth/tests/tree/src/fs.rs` | 3,380,212,365 | 15.615% |
| `hearth_protocol_tree::tree::read::Reader::read_entry` | `hearth/crates/hearth-protocol-tree/src/tree/read.rs` | 2,896,927,619 | 13.382% |
| `hearth_tree_world::helper::Helper::against` | `hearth/tests/tree/src/helper.rs` | 2,884,739,422 | 13.326% |

### Real git

5 tests sampled; collected CPU 0.481560 s covers **6.90%** of this family. Recorded pool: **330,034,902 Ir**. All sampled tests passed under Callgrind.

| Rank / binary / test | CPU s | System s | Recorded Ir | Family CPU covered |
| --- | --- | --- | --- | --- |
| 1 / `hearth-builder-world::real / part_changes::each_changed_part_exits_with_changes` | 0.178397 | 0.065083 | 135,402,943 | 2.556% |
| 2 / `hearth-builder-world::real / part_changes::changed_parts_report_their_rebuild_kind_in_json` | 0.175325 | 0.072065 | 136,904,708 | 2.512% |
| 3 / `hearth-builder-world::real / a_binary_file_answers_binary_with_no_hunk_line` | 0.094689 | 0.065491 | 37,495,377 | 1.357% |
| 92 / `hearth-builder-world::real / a_read_of_a_setting_its_task_lacks_exits_with_configuration_failure` | 0.029939 | 0.026716 | 18,275,940 | 0.429% |
| 183 / `hearth-builder-world::real / a_scratch_repository_without_git_on_the_path_fails_by_name_not_by_skip` | 0.003210 | 0.002945 | 1,955,934 (parent; child unknown) | 0.046% |

**By bucket, self Ir.** Complete measured pool; no CPU extrapolation.

| Bucket | Self Ir | Recorded Ir |
| --- | --- | --- |
| Rust core | 72,536,529 | 21.978% |
| libc / system libraries (other) | 55,863,767 | 16.927% |
| Process startup / loader | 44,579,745 | 13.508% |
| Skein (excluding counting allocator) | 38,080,599 | 11.538% |
| Hearth production | 31,444,999 | 9.528% |
| Git implementation | 23,626,895 | 7.159% |
| Rust alloc | 20,235,794 | 6.131% |
| Allocator (including counting allocator) | 19,869,772 | 6.021% |
| zlib (symbols unavailable) | 17,444,108 | 5.286% |
| Other | 2,088,253 | 0.633% |
| Harness before test | 1,605,340 | 0.486% |
| Rust std | 1,450,219 | 0.439% |
| Unresolved | 455,164 | 0.138% |
| Harness after test | 357,019 | 0.108% |
| Hearth tests and kits | 336,887 | 0.102% |
| Unwinder | 59,605 | 0.018% |
| Harness (direct self instructions) | 207 | 0.000% |
| Hashing (sha1 / sha2) | 0 | 0.000% |

**Twenty hottest functions by self Ir.**

| Function | Source | Self Ir | Recorded Ir |
| --- | --- | --- | --- |
| `__memset_avx2_unaligned_erms` | `./string/../sysdeps/x86_64/multiarch/memset-vec-unaligned-erms.S` | 26,772,659 | 8.112% |
| `_dl_relocate_object_no_relro` | `./elf/./elf/do-rel.h` | 18,827,851 | 5.705% |
| `__memcpy_avx_unaligned_erms` | `./string/../sysdeps/x86_64/multiarch/memmove-vec-unaligned-erms.S` | 13,775,771 | 4.174% |
| `do_lookup_x` | `./elf/./elf/dl-lookup.c` | 9,008,935 | 2.730% |
| `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` | `hearth/crates/hearth-lib/src/lib.rs` | 5,994,613 | 1.816% |
| `_int_malloc` | `./malloc/./malloc/malloc.c` | 5,787,843 | 1.754% |
| `skein_lib::sha256::compress` | `skein/crates/skein-lib/src/sha256.rs` | 5,589,000 | 1.693% |
| `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` | `Rust library/core/src/iter/adapters/enumerate.rs` | 4,724,836 | 1.432% |
| `_dl_lookup_symbol_x` | `./elf/./elf/dl-lookup.c` | 4,686,984 | 1.420% |
| `core::result::Result<T,E>::expect` | `Rust library/core/src/result.rs` | 4,665,149 | 1.414% |
| `free` | `./malloc/./malloc/malloc.c` | 4,572,026 | 1.385% |
| `core::slice::<impl` | `Rust library/core/src/slice/mod.rs` | 4,225,203 | 1.280% |
| `core::ptr::const_ptr::<impl *const T>::is_aligned_to` | `Rust library/core/src/num/uint_macros.rs` | 3,983,692 | 1.207% |
| `0x00000000002cde30` | `??? [git]` | 3,421,644 | 1.037% |
| `inflate` | `??? [libz.so.1.3.1]` | 3,327,507 | 1.008% |
| `0x0000000000011a30` | `??? [libz.so.1.3.1]` | 3,253,809 | 0.986% |
| `malloc` | `./malloc/./malloc/malloc.c` | 3,191,287 | 0.967% |
| `0x0000000000011930` | `??? [libz.so.1.3.1]` | 2,414,224 | 0.732% |
| `strcmp` | `./string/../sysdeps/x86_64/multiarch/../multiarch/strcmp-sse2.S` | 2,351,332 | 0.712% |
| `0x000000000000fbc0` | `??? [libz.so.1.3.1]` | 2,283,193 | 0.692% |

**Twenty heaviest hearth or skein functions by inclusive Ir.**

| Function | Source | Inclusive Ir | Recorded Ir |
| --- | --- | --- | --- |
| `hearth_builder_world::fixtures::run` | `hearth/tests/builder/src/fixtures.rs` | 195,884,328 | 59.353% |
| `hearth_builder_world::fixtures::run_holding` | `hearth/tests/builder/src/fixtures.rs` | 195,882,104 | 59.352% |
| `skein_shell::drive::drive` | `skein/crates/skein-shell/src/drive.rs` | 193,144,830 | 58.523% |
| `hearth_builder_world::fixtures::diff` | `hearth/tests/builder/src/fixtures.rs` | 187,014,667 | 56.665% |
| `hearth_builder_world::fixtures::part_changes::changed` | `hearth/tests/builder/src/fixtures/part_changes.rs` | 173,198,712 | 52.479% |
| `<hearth_builder_world::fixtures::Holding as skein_shell::drive::Host>::iterate` | `hearth/tests/builder/src/fixtures.rs` | 162,550,168 | 49.252% |
| `<hearth::Client as skein_shell::drive::Host>::iterate` | `hearth/crates/hearth/src/lib.rs` | 162,137,736 | 49.127% |
| `hearth_service_builder::iterate` | `hearth/crates/hearth-service-builder/src/lib.rs` | 162,120,266 | 49.122% |
| `real::part_changes::changed_parts_report_their_rebuild_kind_in_json::{{closure}}` | `hearth/tests/builder/tests/real/part_changes.rs` | 87,525,793 | 26.520% |
| `real::part_changes::changed_parts_report_their_rebuild_kind_in_json` | `hearth/tests/builder/tests/real/part_changes.rs` | 87,525,758 | 26.520% |
| `real::part_changes::each_changed_part_exits_with_changes::{{closure}}` | `hearth/tests/builder/tests/real/part_changes.rs` | 85,735,561 | 25.978% |
| `real::part_changes::each_changed_part_exits_with_changes` | `hearth/tests/builder/tests/real/part_changes.rs` | 85,735,526 | 25.978% |
| `hearth_service_builder::domain_stage` | `hearth/crates/hearth-service-builder/src/lib.rs` | 66,157,085 | 20.045% |
| `hearth_domain_builder::resume` | `hearth/crates/hearth-domain-builder/src/lib.rs` | 57,479,436 | 17.416% |
| `hearth_domain_render::resume` | `hearth/crates/hearth-domain-render/src/lib.rs` | 56,866,172 | 17.230% |
| `hearth_domain_render::call::Call::resume` | `hearth/crates/hearth-domain-render/src/call.rs` | 56,241,170 | 17.041% |
| `hearth_service_builder::protocol_up` | `hearth/crates/hearth-service-builder/src/lib.rs` | 56,218,337 | 17.034% |
| `hearth_domain_render::call::resolution::<impl hearth_domain_render::call::Side>::resolved_answer` | `hearth/crates/hearth-domain-render/src/call/resolution.rs` | 48,577,552 | 14.719% |
| `hearth_domain_render::call::templates::<impl hearth_domain_render::call::Side>::answer_for` | `hearth/crates/hearth-domain-render/src/call/templates.rs` | 46,327,584 | 14.037% |
| `hearth_domain_render::call::Side::answer` | `hearth/crates/hearth-domain-render/src/call.rs` | 46,315,633 | 14.034% |

**By process.** These tests drive `hearth::Client` in the test process (`hearth/tests/builder/src/fixtures.rs:134`). They do not exec the `hearth` binary, so its measured process count and Ir here are **0**.

| Process | Executed processes | Recorded Ir | Sample Ir |
| --- | --- | --- | --- |
| test | 5 | 212,763,014 | 64.467% |
| git | 136 | 117,271,888 | 35.533% |
| hearth | 0 | 0 | 0.000% |

The `hearth` child of the separate `hearth::end_to_end` binary belongs to All the rest; its successful execution Ir is unknown (see the process section).

**Fixture commands versus client commands.** `Repository::make_on` invokes init/add/commit; `Repository::next` invokes add/commit again.

| Git verb | Processes | Ir | Git Ir | Role |
| --- | --- | --- | --- | --- |
| `commit` | 20 | 42,608,012 | 36.333% | fixture setup |
| `add` | 20 | 23,472,417 | 20.015% | fixture setup |
| `rev-parse` | 37 | 16,845,202 | 14.364% | fixture HEAD and client revision lookups |
| `cat-file` | 29 | 14,945,052 | 12.744% | client command |
| `ls-tree` | 19 | 13,589,670 | 11.588% | client command |
| `init` | 10 | 5,387,946 | 4.594% | fixture setup |
| `diff` | 1 | 423,589 | 0.361% | client command |

Init/add/commit together: **71,468,375 Ir**, 60.94% of git instructions and 21.65% of the recorded sample. Family CPU estimates below assign user CPU only; 3.557110 s of family system CPU remains unassigned.

The median test also dumped immediately before `hearth::Client::open`: **1,975,024 of 11,163,011 test-process Ir (17.69%)** ran before the client started, including loader, harness, scratch/filesystem preparation and parent-side git launch work. Its setup git processes added **4,714,987 Ir**; client git commands added **2,227,886 Ir**. Thus **6,690,011 of 18,105,884 recorded Ir (36.95%)** ran before client startup, including setup children. This is that test's boundary; the corresponding boundary for every other real-git test is unknown. The cheapest test intentionally never starts the client.

Fork dumps before exec retain parent counters. They are excluded even when the same PID later execs git, whose fresh counters are included. The instruction delta between fork and exec, and in the failed-exec child of the cheapest test, is **unknown**. Recorded Ir means parent plus executed children, not a claim that unknown deltas are zero. Syscall 424 warnings did not fail these tests, but limit interpretation of kernel-adjacent paths.

### Render suite

5 tests sampled; collected CPU 0.463849 s covers **13.28%** of this family. Recorded pool: **3,162,177,374 Ir**. All sampled tests passed under Callgrind.

| Rank / binary / test | CPU s | System s | Recorded Ir | Family CPU covered |
| --- | --- | --- | --- | --- |
| 1 / `hearth-render-world::suite / properties::a_production_edit_never_changes_a_lab_spec` | 0.192039 | 0.000000 | 1,135,950,083 | 5.497% |
| 2 / `hearth-render-world::suite / errors::three_hundred_broken_documents_answer_256_errors_then_the_count` | 0.132912 | 0.003909 | 1,057,186,957 | 3.805% |
| 3 / `hearth-render-world::suite / properties::a_collision_names_the_same_pair_whichever_task_arrives_first_0` | 0.128052 | 0.000000 | 934,989,842 | 3.666% |
| 55 / `hearth-render-world::suite / diff_reuse::a_blob_read_as_something_else_is_read_again` | 0.008718 | 0.004359 | 31,575,368 | 0.250% |
| 109 / `hearth-render-world::suite / golden_harness::a_witness_must_match_its_files_line_to_twelve_digits` | 0.002128 | 0.000000 | 2,475,124 | 0.061% |

**By bucket, self Ir.** Complete measured pool; no CPU extrapolation.

| Bucket | Self Ir | Recorded Ir |
| --- | --- | --- |
| Rust core | 1,944,224,507 | 61.484% |
| Hearth production | 443,151,747 | 14.014% |
| Allocator (including counting allocator) | 226,309,950 | 7.157% |
| Rust alloc | 139,072,081 | 4.398% |
| Skein (excluding counting allocator) | 136,831,899 | 4.327% |
| libc / system libraries (other) | 127,541,331 | 4.033% |
| Rust std | 121,465,206 | 3.841% |
| Hearth tests and kits | 9,687,472 | 0.306% |
| Process startup / loader | 8,145,750 | 0.258% |
| Harness before test | 3,673,429 | 0.116% |
| Unresolved | 1,375,139 | 0.043% |
| Harness after test | 678,577 | 0.021% |
| Other | 20,286 | 0.001% |
| Hashing (sha1 / sha2) | 0 | 0.000% |
| Harness (direct self instructions) | 0 | 0.000% |

**Twenty hottest functions by self Ir.**

| Function | Source | Self Ir | Recorded Ir |
| --- | --- | --- | --- |
| `core::slice::<impl` | `Rust library/core/src/slice/mod.rs` | 292,036,851 | 9.235% |
| `core::ptr::const_ptr::<impl *const T>::is_aligned_to` | `Rust library/core/src/num/uint_macros.rs` | 260,643,786 | 8.243% |
| `core::slice::raw::from_raw_parts::precondition_check` | `hearth/crates/hearth-lib/src/lib.rs` | 193,715,488 | 6.126% |
| `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` | `hearth/crates/hearth-lib/src/lib.rs` | 92,704,183 | 2.932% |
| `__memcmp_avx2_movbe` | `./string/../sysdeps/x86_64/multiarch/memcmp-avx2-movbe.S` | 90,628,114 | 2.866% |
| `core::slice::<impl [T]>::split_at_unchecked` | `Rust library/core/src/ub_checks.rs` | 66,192,854 | 2.093% |
| `<core::str::lossy::Utf8Chunks as core::iter::traits::iterator::Iterator>::next` | `Rust library/core/src/str/lossy.rs` | 59,007,929 | 1.866% |
| `core::ptr::copy_nonoverlapping::precondition_check` | `skein/testing/skein-heap/src/lib.rs` | 58,795,900 | 1.859% |
| `std::thread::local::LocalKey<T>::try_with` | `Rust library/std/src/thread/local.rs` | 56,805,090 | 1.796% |
| `<core::option::Option<T> as core::ops::try_trait::Try>::branch` | `Rust library/core/src/option.rs` | 55,804,782 | 1.765% |
| `<A as core::slice::cmp::SlicePartialEq<B>>::equal_same_length` | `Rust library/core/src/slice/cmp.rs` | 51,820,662 | 1.639% |
| `hearth_domain_render::revision::class` | `hearth/crates/hearth-domain-render/src/revision.rs` | 50,807,421 | 1.607% |
| `core::ops::function::FnOnce::call_once'2` | `Rust library/core/src/ops/function.rs` | 49,551,664 | 1.567% |
| `core::slice::memchr::memchr` | `Rust library/core/src/slice/memchr.rs` | 43,446,817 | 1.374% |
| `hearth_domain_render::revision::name` | `hearth/crates/hearth-domain-render/src/revision.rs` | 41,879,838 | 1.324% |
| `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` | `Rust library/core/src/iter/adapters/enumerate.rs` | 39,812,336 | 1.259% |
| `skein_heap::padded` | `skein/testing/skein-heap/src/lib.rs` | 38,876,929 | 1.229% |
| `skein_lib::sha256::compress` | `skein/crates/skein-lib/src/sha256.rs` | 37,268,280 | 1.179% |
| `core::slice::cmp::<impl core::cmp::PartialEq<[U]> for` | `Rust library/core/src/slice/cmp.rs` | 34,801,002 | 1.101% |
| `__memcpy_avx_unaligned_erms` | `./string/../sysdeps/x86_64/multiarch/memmove-vec-unaligned-erms.S` | 34,725,851 | 1.098% |

**Twenty heaviest hearth or skein functions by inclusive Ir.**

| Function | Source | Inclusive Ir | Recorded Ir |
| --- | --- | --- | --- |
| `hearth_test_render::parent::drive` | `hearth/testing/hearth-test-render/src/parent.rs` | 2,875,770,788 | 90.943% |
| `hearth_test_render::parent::run_with` | `hearth/testing/hearth-test-render/src/parent.rs` | 2,847,786,967 | 90.058% |
| `hearth_domain_render::resume` | `hearth/crates/hearth-domain-render/src/lib.rs` | 1,875,218,397 | 59.301% |
| `hearth_domain_render::call::Call::resume` | `hearth/crates/hearth-domain-render/src/call.rs` | 1,838,055,359 | 58.126% |
| `hearth_test_render::properties::render` | `hearth/testing/hearth-test-render/src/properties.rs` | 1,809,463,850 | 57.222% |
| `hearth_domain_render::call::resolution::<impl hearth_domain_render::call::Side>::resolved_answer` | `hearth/crates/hearth-domain-render/src/call/resolution.rs` | 1,710,417,480 | 54.090% |
| `hearth_domain_render::call::templates::<impl hearth_domain_render::call::Side>::answer_for` | `hearth/crates/hearth-domain-render/src/call/templates.rs` | 1,649,089,826 | 52.150% |
| `hearth_domain_render::call::Side::answer` | `hearth/crates/hearth-domain-render/src/call.rs` | 1,648,835,522 | 52.142% |
| `suite::properties::a_production_edit_never_changes_a_lab_spec::{{closure}}` | `hearth/tests/render/tests/suite/properties.rs` | 1,133,392,216 | 35.842% |
| `suite::properties::a_production_edit_never_changes_a_lab_spec` | `hearth/tests/render/tests/suite/properties.rs` | 1,133,392,181 | 35.842% |
| `hearth_test_render::properties::run` | `hearth/testing/hearth-test-render/src/properties.rs` | 1,087,209,589 | 34.382% |
| `suite::errors::three_hundred_broken_documents_answer_256_errors_then_the_count::{{closure}}` | `hearth/tests/render/tests/suite/errors.rs` | 1,054,712,508 | 33.354% |
| `suite::errors::three_hundred_broken_documents_answer_256_errors_then_the_count` | `hearth/tests/render/tests/suite/errors.rs` | 1,054,712,473 | 33.354% |
| `hearth_test_render::parent::run` | `hearth/testing/hearth-test-render/src/parent.rs` | 1,038,325,709 | 32.836% |
| `hearth_domain_render::revision::class` | `hearth/crates/hearth-domain-render/src/revision.rs` | 985,651,412 | 31.170% |
| `suite::properties::a_collision_names_the_same_pair_whichever_task_arrives_first_0::{{closure}}` | `hearth/tests/render/tests/suite/properties.rs` | 932,455,309 | 29.488% |
| `suite::properties::a_collision_names_the_same_pair_whichever_task_arrives_first_0` | `hearth/tests/render/tests/suite/properties.rs` | 932,455,274 | 29.488% |
| `hearth_test_render::properties::collision_order` | `hearth/testing/hearth-test-render/src/properties.rs` | 932,455,267 | 29.488% |
| `hearth_domain_render::step` | `hearth/crates/hearth-domain-render/src/lib.rs` | 892,637,463 | 28.229% |
| `hearth_domain_render::revision::Documents::declaration` | `hearth/crates/hearth-domain-render/src/revision.rs` | 872,378,113 | 27.588% |

### All the rest

7 tests sampled; collected CPU 0.728043 s covers **11.51%** of this family. Recorded pool: **5,967,439,126 Ir**. All sampled tests passed under Callgrind.

| Rank / binary / test | CPU s | System s | Recorded Ir | Family CPU covered |
| --- | --- | --- | --- | --- |
| 1 / `hearth-config-world::suite / declarations::every_task_example_decodes_as_a_declaration_cut_anywhere` | 0.168414 | 0.003827 | 1,399,635,788 | 2.663% |
| 2 / `hearth-git-world::suite / recordings::cut_and_raised_recordings_fail_or_decode_as_recorded` | 0.151937 | 0.004106 | 1,258,122,565 | 2.403% |
| 3 / `hearth-git-world::suite / recordings::cut_and_raised_recordings_fail_or_decode_as_recorded_at_other_cuts` | 0.149393 | 0.008075 | 1,250,139,261 | 2.362% |
| 4 / `hearth-config-world::suite / declarations::generated_declarations_hand_up_each_string_where_it_was_written` | 0.130709 | 0.003844 | 1,185,303,936 | 2.067% |
| 5 / `hearth-template-world::suite / templates::generated_templates_compile_as_the_reference_compiles_them` | 0.124155 | 0.003879 | 872,543,116 | 1.963% |
| 869 / `hearth-protocol-config / tests::declaration::a_read_s_task_or_setting_out_of_form_is_refused_at_it` | 0.002117 | 0.000000 | 944,107 | 0.033% |
| 1737 / `hearth / tests::build_takes_one_target_and_render_s_options` | 0.001318 | 0.000000 | 750,353 | 0.021% |

**By bucket, self Ir.** Complete measured pool; no CPU extrapolation.

| Bucket | Self Ir | Recorded Ir |
| --- | --- | --- |
| Rust core | 2,943,454,847 | 49.325% |
| Allocator (including counting allocator) | 853,585,031 | 14.304% |
| Hearth tests and kits | 593,196,023 | 9.941% |
| Rust std | 496,776,995 | 8.325% |
| Skein (excluding counting allocator) | 413,427,812 | 6.928% |
| Rust alloc | 314,243,852 | 5.266% |
| Hearth production | 258,383,266 | 4.330% |
| libc / system libraries (other) | 84,538,548 | 1.417% |
| Harness before test | 4,124,098 | 0.069% |
| Process startup / loader | 3,280,307 | 0.055% |
| Unresolved | 1,156,906 | 0.019% |
| Harness after test | 844,367 | 0.014% |
| Other | 427,074 | 0.007% |
| Hashing (sha1 / sha2) | 0 | 0.000% |
| Harness (direct self instructions) | 0 | 0.000% |

**Twenty hottest functions by self Ir.**

| Function | Source | Self Ir | Recorded Ir |
| --- | --- | --- | --- |
| `hearth_config_world::reference::position` | `hearth/tests/config/src/reference.rs` | 345,265,545 | 5.786% |
| `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` | `??? [suite-e1a220223360ae8a]` | 301,754,323 | 5.057% |
| `core::ptr::const_ptr::<impl *const T>::is_aligned_to` | `Rust library/core/src/num/uint_macros.rs` | 239,990,064 | 4.022% |
| `std::thread::local::LocalKey<T>::try_with` | `Rust library/std/src/thread/local.rs` | 231,615,534 | 3.881% |
| `core::ptr::copy_nonoverlapping::precondition_check` | `skein/testing/skein-heap/src/lib.rs` | 222,818,098 | 3.734% |
| `core::ops::function::FnOnce::call_once'2` | `Rust library/core/src/ops/function.rs` | 202,378,047 | 3.391% |
| `skein_heap::padded` | `skein/testing/skein-heap/src/lib.rs` | 158,780,623 | 2.661% |
| `std::thread::local::LocalKey<T>::initialize_with` | `Rust library/std/src/thread/local.rs` | 137,414,060 | 2.303% |
| `core::alloc::layout::Layout::from_size_align` | `Rust library/core/src/alloc/layout.rs` | 126,486,259 | 2.120% |
| `core::ptr::alignment::Alignment::new_unchecked::precondition_check` | `Rust library/core/src/num/uint_macros.rs` | 111,876,780 | 1.875% |
| `core::ub_checks::maybe_is_nonoverlapping::runtime` | `Rust library/core/src/intrinsics/mod.rs` | 101,402,514 | 1.699% |
| `<core::option::Option<T> as core::ops::try_trait::Try>::branch` | `Rust library/core/src/option.rs` | 101,229,987 | 1.696% |
| `free` | `./malloc/./malloc/malloc.c` | 97,241,266 | 1.630% |
| `std::thread::local::LocalKey<T>::with` | `Rust library/std/src/thread/local.rs` | 87,545,384 | 1.467% |
| `core::result::Result<T,E>::expect` | `Rust library/core/src/result.rs` | 76,561,254 | 1.283% |
| `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::dealloc` | `skein/testing/skein-heap/src/lib.rs` | 69,970,992 | 1.173% |
| `__memcpy_avx_unaligned_erms` | `./string/../sysdeps/x86_64/multiarch/memmove-vec-unaligned-erms.S` | 69,626,154 | 1.167% |
| `<alloc::string::String as core::fmt::Write>::write_str` | `Rust library/alloc/src/string.rs` | 66,919,736 | 1.121% |
| `core::result::Result<T,E>::ok` | `Rust library/core/src/result.rs` | 64,591,420 | 1.082% |
| `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::alloc` | `skein/testing/skein-heap/src/lib.rs` | 63,243,247 | 1.060% |

**Twenty heaviest hearth or skein functions by inclusive Ir.**

| Function | Source | Inclusive Ir | Recorded Ir |
| --- | --- | --- | --- |
| `hearth_git_world::fixtures::check_recording_mutations` | `hearth/tests/git/src/fixtures.rs` | 2,505,522,703 | 41.987% |
| `hearth_test_git::replay::replay` | `hearth/testing/hearth-test-git/src/replay.rs` | 2,471,767,008 | 41.421% |
| `hearth_config_world::world::declare` | `hearth/tests/config/src/world.rs` | 2,456,496,128 | 41.165% |
| `hearth_config_world::world::check` | `hearth/tests/config/src/world.rs` | 2,429,343,609 | 40.710% |
| `hearth_test_git::hunks::diff` | `hearth/testing/hearth-test-git/src/hunks.rs` | 2,189,336,996 | 36.688% |
| `hearth_config_world::reference::Parser::value'2` | `hearth/tests/config/src/reference.rs` | 1,842,162,412 | 30.870% |
| `hearth_config_world::world::run` | `hearth/tests/config/src/world.rs` | 1,645,971,199 | 27.583% |
| `hearth_config_world::reference::Parser::object'2` | `hearth/tests/config/src/reference.rs` | 1,587,270,682 | 26.599% |
| `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::alloc` | `skein/testing/skein-heap/src/lib.rs` | 1,461,483,594 | 24.491% |
| `suite::declarations::every_task_example_decodes_as_a_declaration_cut_anywhere::{{closure}}` | `hearth/tests/config/tests/suite/declarations.rs` | 1,398,288,463 | 23.432% |
| `suite::declarations::every_task_example_decodes_as_a_declaration_cut_anywhere` | `hearth/tests/config/tests/suite/declarations.rs` | 1,398,288,428 | 23.432% |
| `hearth_protocol_git::hunks::up` | `hearth/crates/hearth-protocol-git/src/hunks.rs` | 1,395,329,191 | 23.382% |
| `suite::recordings::cut_and_raised_recordings_fail_or_decode_as_recorded::{{closure}}` | `hearth/tests/git/tests/suite/recordings.rs` | 1,256,751,284 | 21.060% |
| `suite::recordings::cut_and_raised_recordings_fail_or_decode_as_recorded` | `hearth/tests/git/tests/suite/recordings.rs` | 1,256,751,249 | 21.060% |
| `suite::recordings::cut_and_raised_recordings_fail_or_decode_as_recorded_at_other_cuts::{{closure}}` | `hearth/tests/git/tests/suite/recordings.rs` | 1,248,771,503 | 20.926% |
| `suite::recordings::cut_and_raised_recordings_fail_or_decode_as_recorded_at_other_cuts` | `hearth/tests/git/tests/suite/recordings.rs` | 1,248,771,468 | 20.926% |
| `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::dealloc` | `skein/testing/skein-heap/src/lib.rs` | 1,233,393,306 | 20.669% |
| `suite::declarations::generated_declarations_hand_up_each_string_where_it_was_written::{{closure}}` | `hearth/tests/config/tests/suite/declarations.rs` | 1,183,936,439 | 19.840% |
| `suite::declarations::generated_declarations_hand_up_each_string_where_it_was_written` | `hearth/tests/config/tests/suite/declarations.rs` | 1,183,936,404 | 19.840% |
| `suite::templates::generated_templates_compile_as_the_reference_compiles_them::{{closure}}` | `hearth/tests/template/tests/suite/templates.rs` | 871,242,331 | 14.600% |

The rest spans many binaries. Its sample covers 11.51% of family CPU; extrapolation error is unknown. In particular, the numerous cheap `hearth-domain-render` library tests are not represented by the hottest config/git loops. The process-floor table measures the cheapest test of every executing binary instead.

## A test's process

Each binary's cheapest base-1 test was run with `--dump-before=<exact test function>` and `--dump-after=<exact test function>`. Callgrind's incremental dumps separate **before entry**, the entire test function including its callees, and work after it returns. The loader's self instructions in the before dump are startup; every other before instruction is harness work, including its Rust and libc descendants. “After” includes harness teardown and any loader work after the test. These columns partition the **parent process**; child instructions are separate from it. CPU remains the collector's complete test-tree CPU. File size is the unstripped executable's byte size, not resident memory.

| Binary / cheapest test | Binary bytes | CPU s | Parent Ir | Startup Ir | Harness before Ir | Test function Ir | After Ir |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `hearth-recordings / tests::a_further_mask_replaces_its_literal_after_the_scratch` | 7,148,520 | 0.001719 | 718,993 | 388,189 | 281,318 | 15,962 | 33,524 |
| `hearth-protocol-accounts / tests::read::the_passwd_master_reads_to_its_entries` | 7,200,608 | 0.001531 | 972,357 | 383,011 | 284,720 | 269,941 | 34,685 |
| `hearth-lib / tests::hash::h_of_empty_is_the_digest_store_md_gives` | 7,270,736 | 0.001575 | 741,571 | 381,772 | 283,667 | 43,650 | 32,482 |
| `hearth-domain-build / tests::a_refused_stage_fails_the_build_at_once_with_no_discard` | 7,294,432 | 0.001639 | 844,475 | 381,162 | 280,706 | 149,379 | 33,228 |
| `hearth-protocol-git / tests::invocation::each_command_s_vector_is_render_md_2_2_s_word_for_word` | 7,574,048 | 0.001510 | 794,066 | 390,044 | 290,576 | 78,366 | 35,080 |
| `hearth-protocol-template / tests::templates::a_template_past_the_limit_is_refused_at_the_first_byte_past` | 7,703,056 | 0.001545 | 743,970 | 413,403 | 297,752 | 265 | 32,550 |
| `hearth-protocol-mmdebstrap / tests::output::inst_lines_past_the_limit_fail_the_pipe` | 7,707,488 | 0.001551 | 825,559 | 397,010 | 293,229 | 102,290 | 33,030 |
| `hearth-accounts-world::suite / memory::a_read_at_its_limits_holds_no_more_than_its_ceiling` | 7,731,752 | 0.014992 | 28,332,902 | 399,417 | 704,331 | 27,140,149 | 89,005 |
| `hearth-test-mirror / tests::pin::a_pin_out_of_form_is_refused_naming_its_line` | 7,913,592 | 0.001747 | 904,046 | 419,947 | 283,628 | 167,558 | 32,913 |
| `hearth-domain-tree / tests::bounds::the_walk_bound_is_a_sum_that_fits_or_nothing` | 8,064,248 | 0.001530 | 731,123 | 403,412 | 290,847 | 4,993 | 31,871 |
| `hearth-domain-store / tests::scan::a_failed_scan_leaves_no_index` | 8,490,104 | 0.001534 | 807,052 | 398,899 | 293,876 | 78,539 | 35,738 |
| `hearth-template-world::suite / expressions::a_filter_after_a_test_is_refused_at_its_pipe` | 8,512,904 | 0.002066 | 1,303,010 | 457,858 | 699,214 | 61,054 | 84,884 |
| `hearth-protocol-tree / tests::frame::a_framed_body_has_the_header_store_md_gives_byte_for_byte` | 8,547,416 | 0.001613 | 850,092 | 431,006 | 291,891 | 93,758 | 33,437 |
| `hearth-protocol-config / tests::document::a_nonobject_placement_is_refused_at_its_value` | 8,599,264 | 0.001531 | 957,269 | 439,106 | 294,384 | 189,464 | 34,315 |
| `hearth-client-protocol-records / tests::boot::boot_parameters_and_mutability_print_as_render_md_9_2_shows` | 9,435,592 | 0.001457 | 851,472 | 435,097 | 309,349 | 72,172 | 34,854 |
| `hearth-tree-world::suite / hash::an_item_the_writer_refuses_fails_the_walk_naming_the_batchs_first_path` | 12,695,856 | 0.002104 | 1,625,686 | 592,872 | 707,759 | 238,959 | 86,096 |
| `hearth-git-world::suite / hunks::an_end_or_a_fault_before_the_read_answers_the_read` | 13,891,328 | 0.001577 | 1,273,519 | 469,395 | 711,183 | 9,899 | 83,042 |
| `hearth-mmdebstrap-world::suite / mmdebstrap::a_pipe_that_breaks_fails_the_reader_with_its_fault` | 14,606,928 | 0.002089 | 4,678,129 | 487,700 | 709,114 | 3,392,499 | 88,816 |
| `hearth-config-world::suite / world::a_failed_stream_fails_the_decode_with_its_fault` | 15,096,016 | 0.001733 | 1,523,520 | 516,005 | 708,849 | 213,456 | 85,210 |
| `hearth-fake-machine / tests::ledger::a_refusal_is_recorded_once_without_moving_bytes` | 16,291,528 | 0.001629 | 821,884 | 436,926 | 296,927 | 53,929 | 34,102 |
| `hearth-conformance-world::suite / resolve_stanzas::matching_stanzas_keep_index_order_and_all_fields_without_duplicates` | 16,705,912 | 0.001673 | 831,996 | 392,183 | 283,471 | 122,985 | 33,357 |
| `hearth::refusals / an_unknown_option_is_refused_on_standard_error_in_the_call_s_form` | 16,754,800 | 0.019810 | unknown¹ | 642,586 | 283,385 | unknown¹ | unknown¹ |
| `hearth-fake-git / tests::domain::laying_a_repository_makes_only_its_directory_and_head` | 16,842,104 | 0.001731 | 867,578 | 470,202 | 288,263 | 74,732 | 34,381 |
| `hearth::end_to_end / end_to_end_render` | 16,931,784 | 0.053368 | unknown¹ | 658,855 | 283,447 | unknown¹ | unknown¹ |
| `hearth-fake-mmdebstrap / tests::face::hearth_s_vector_reads_into_its_simulate_request` | 17,005,376 | 0.001832 | 892,573 | 505,617 | 284,218 | 66,015 | 36,723 |
| `hearth / tests::build_takes_one_target_and_render_s_options` | 20,942,024 | 0.001318 | 750,353 | 405,814 | 284,150 | 26,395 | 33,994 |
| `hearth-domain-builder / tests::a_produced_file_enters_the_store_with_its_rendered_origin` | 25,739,032 | 0.002298 | 1,259,075 | 878,987 | 289,485 | 57,673 | 32,930 |
| `hearth-domain-render / tests::packages::a_name_out_of_policys_form_is_refused_at_its_entry` | 25,981,392 | 0.001782 | 1,134,962 | 667,981 | 404,490 | 29,762 | 32,729 |
| `hearth-protocol-builder / tests::translate::repository::each_repository_kind_translates_and_back` | 27,801,624 | 0.001732 | 1,107,375 | 753,984 | 318,854 | 2,285 | 32,252 |
| `hearth-service-builder / tests::valid::startup_refuses_a_watermark_of_nothing` | 34,357,232 | 0.002721 | 1,608,130 | 1,218,662 | 287,902 | 60,009 | 41,557 |
| `hearth-render-world::suite / golden_harness::a_witness_must_match_its_files_line_to_twelve_digits` | 47,042,824 | 0.002128 | 2,475,124 | 1,627,590 | 735,260 | 31,273 | 81,001 |
| `hearth-builder-world::real / a_scratch_repository_without_git_on_the_path_fails_by_name_not_by_skip` | 48,732,520 | 0.003210 | 1,955,934 | 1,420,725 | 321,078 | 167,158 | 46,973 |
| `hearth-builder-world::suite / goldens::the_keys_line_is_the_one_hearth_24_pins` | 60,622,856 | 0.002437 | 2,867,345 | 2,032,837 | 733,798 | 17,990 | 82,720 |

¹ `hearth::end_to_end / end_to_end_render` and `hearth::refusals / an_unknown_option_is_refused_on_standard_error_in_the_call_s_form` passed in both ordinary collectors but failed under Callgrind at `hearth/testing/hearth-test-repository/src/binary.rs:30`, `binary starts: Child(Other(95))`. Their completed before-entry dumps still measure the process floor; successful body/whole-process Ir and the launched `hearth` child's Ir are **unknown**. This is a profiling limitation, not a finding about either test. Other tests from the rest family were profiled successfully. The exact unsupported operation behind errno 95 is unknown; no workaround or setting change was made.

31 other floor profiles passed. Some real-git traces warn about unhandled amd64 syscall 424 but still pass; the profiles retain that caveat. For the missing-git test, the failed-exec child's dump inherits the parent's counters before fork. That dump is not added to the parent: its incremental instruction cost is **unknown**. This avoids counting process startup twice.

**Across all 2,189 tests, estimated pre-entry work is 2,526,801,881 Ir:** 1,656,033,358 loader instructions and 870,768,523 harness instructions. This multiplies each binary's observed cheapest-test before-entry count by that binary's executed test count; it is an estimate because test names, scheduling and argument processing vary. It excludes fixture setup inside the test function, child startup and teardown after it. It uses a before-entry measurement for all 33 binaries, including the two whose later bodies could not complete.

**CPU before entry is unknown:** the collector reports a process-tree total, not phases, and Callgrind instruction counts do not measure kernel CPU. Repeating each binary's cheapest observed **whole test** CPU for every test would cost 4.368991 CPU s (12.09% of base-1), a labelled floor proxy including those tests' own work and descendants, not a measured CPU cost of startup. The typical `--lib` example, `hearth-protocol-config`, is 8,599,264 bytes and costs 0.001531 CPU s as a whole process; its measured pre-entry portion is 733,490 Ir. The much larger builder suite is 60,622,856 bytes with 2,766,635 pre-entry Ir. File size and relocations differ; size alone is not asserted to cause that difference.

## What it found

Families are ordered by their measured CPU, largest first; rows within a family by estimated CPU. Every numeric share here is an **estimate** under the user-CPU weighting stated above. Inclusive paths overlap: these are costs reached through a function, not independent savings, and the rows must not be added. Kernel CPU, the effect of changing any function, and extrapolation error are unknown. No repair choice is made here.

| Family | Function / file:line | Repeated work or scan observed in code | Ownership | Estimated family CPU | Estimated suite CPU | Estimated CPU s |
| --- | --- | --- | --- | --- | --- | --- |
| Builder suite | `hearth_fake_machine::linux::ledger::<impl hearth_fake_machine::linux::Machine>::call` — `hearth/testing/hearth-fake-machine/src/linux/ledger.rs:4` | For every fake operation, collect every node/change-counter into a BTreeMap, then scan all current nodes and all old keys to update the process changed set. Read-only and refused operations take this same path. | shared hearth test kit | 77.35% | 21.74% | 7.855 |
| Builder suite | `hearth_builder_world::domain::layers::copy` — `hearth/tests/builder/src/domain/layers.rs:211` | Recursively create directories, list names, read and write each file for a fake snapshot; those operations all pass through the ledger above. Recursive variants and that ledger overlap this inclusive row. | test code | 18.92% | 5.32% | 1.922 |
| Builder suite | `suite::domain::rendered_keys` — `hearth/tests/builder/tests/suite/domain.rs:137` | Build and finish a separate resolving world to get THIN’s expected keys/config part, repeated inside per-seed and per-failure checks. | test code | 4.40% | 1.24% | 0.446 |
| Tree suite | `hearth_tree_world::helper::Helper::page` — `hearth/tests/tree/src/helper.rs:346` | Call Fs::list for every page; it resolves the path and clones every directory name before selecting and boxing the requested page. | test code | 13.94% | 3.54% | 1.279 |
| Tree suite | `hearth_tree_world::helper::Helper::digests` — `hearth/tests/tree/src/helper.rs:382` | Read the entire accumulated tree file twice with new Readers: once for entries/tree digest, again for changes. Each pass decodes items and allocates their values. | test code | 10.10% | 2.57% | 0.927 |
| Tree suite | `hearth_tree_world::fs::Fs::links` — `hearth/tests/tree/src/fs.rs:148` | At every stat, scan every node and every directory name and compare the referenced inode to count links. | test code | 10.08% | 2.56% | 0.925 |
| Tree suite | `hearth_tree_world::fs::Fs::lookup` — `hearth/tests/tree/src/fs.rs:135` | Split the path and linearly search each directory’s name vector; repeated by stat, list, reads and attributes. | test code | 9.98% | 2.53% | 0.916 |
| Real git | `Repository::make_on` / `Repository::next` — `hearth/testing/hearth-test-repository/src/lib.rs:50` / `:82` | Create scratch working trees and run git init/add/commit; next writes the edits and adds/commits again. This estimate covers the child init/add/commit commands only; parent fixture code and pre-exec deltas are not assigned to it. | shared hearth test kit | 10.57% | 2.04% | 0.738 |
| Real git | `hearth_domain_render::revision::Documents::declaration` — `hearth/crates/hearth-domain-render/src/revision.rs:291` | The same listing scan used by the render family also runs through the real client. | production code | 1.20% | 0.23% | 0.084 |
| All the rest | `hearth_protocol_git::hunks::append` — `hearth/crates/hearth-protocol-git/src/hunks.rs:199` | For each delivered chunk, push each byte separately into the retained pending line; the mutation/replay tests repeat the path across cuts and seeds. | production code | 7.41% | 1.30% | 0.468 |
| All the rest | `hearth_config_world::reference::position` — `hearth/tests/config/src/reference.rs:288` | For each requested position, scan the prefix from byte zero to the offset to count line/column, including repeated calls while independently parsing objects and values. | test code | 6.83% | 1.20% | 0.432 |
| Render suite | `hearth_domain_render::revision::Documents::declaration` — `hearth/crates/hearth-domain-render/src/revision.rs:291` | Scan entries zipped with declarations and call class(path) on each until the task matches. Cache priming and other per-target paths repeat the scan. | production code | 25.77% | 2.49% | 0.900 |
| Render suite | `hearth_domain_render::exports::cache::Cache::prime` — `hearth/crates/hearth-domain-render/src/exports/cache.rs:32` | On each resume, start scanning targets/tasks again, find declarations, test whether each pair is needed/already cached, and return after adding one missing pair. Includes the declaration scan above. | production code | 19.05% | 1.84% | 0.666 |
| Render suite | `hearth_domain_render::errors::Collector::push` — `hearth/crates/hearth-domain-render/src/errors.rs:158` | For each error, walk all retained errors for stable insertion, then walk them again to recount retained bytes. The 300-broken-document sample repeats this over eight seeds. | production code | 6.09% | 0.59% | 0.213 |

`hearth/tests/tree/src/fs.rs:135` and `:148` use the fake filesystem’s vectors, not the production tree walker’s data structures. Builder’s ledger belongs to hearth’s fake-machine kit, not skein’s simulator. Render’s declaration/cache/error scans are production paths. Those ownership distinctions come from reading the named functions, not from the leaf core/alloc names.

### Shared kits

These are measured costs in skein, left for the plan’s scope decision. No skein, smith, jig or temper file changes. Counting allocation includes System/libc descendants and is not the same quantity as the allocator’s self bucket; allocation and free rows also overlap callers listed above. Wire includes its formatted fragment/down/up traces and byte buffering; Intake::take allocates a new exact-length delivery, copies from up to two VecDeque slices, and drains the consumed bytes. SHA-256 compress expands 64 message words and executes 64 rounds for every block, without fixture caching.

| Family | Shared function / file:line | Work | Estimated family CPU | Estimated suite CPU |
| --- | --- | --- | --- | --- |
| Builder suite | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::alloc` — `skein/testing/skein-heap/src/lib.rs:317` | count/pad and allocate | 4.45% | 1.25% |
| Builder suite | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::dealloc` — `skein/testing/skein-heap/src/lib.rs:334` | count and free | 3.81% | 1.07% |
| Builder suite | `skein_lib::sha256::compress` — `skein/crates/skein-lib/src/sha256.rs:237` | message schedule and compression | 0.77% | 0.22% |
| Tree suite | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::alloc` — `skein/testing/skein-heap/src/lib.rs:317` | count/pad and allocate | 18.03% | 4.58% |
| Tree suite | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::dealloc` — `skein/testing/skein-heap/src/lib.rs:334` | count and free | 15.97% | 4.06% |
| Tree suite | `skein_lib::sha256::compress` — `skein/crates/skein-lib/src/sha256.rs:237` | message schedule and compression | 8.72% | 2.22% |
| Real git | `skein_lib::sha256::compress` — `skein/crates/skein-lib/src/sha256.rs:237` | message schedule and compression | 0.86% | 0.17% |
| Render suite | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::alloc` — `skein/testing/skein-heap/src/lib.rs:317` | count/pad and allocate | 11.26% | 1.09% |
| Render suite | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::dealloc` — `skein/testing/skein-heap/src/lib.rs:334` | count and free | 9.45% | 0.91% |
| Render suite | `skein_lib::sha256::compress` — `skein/crates/skein-lib/src/sha256.rs:237` | message schedule and compression | 1.24% | 0.12% |
| All the rest | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::alloc` — `skein/testing/skein-heap/src/lib.rs:317` | count/pad and allocate | 16.22% | 2.84% |
| All the rest | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::dealloc` — `skein/testing/skein-heap/src/lib.rs:334` | count and free | 13.69% | 2.40% |
| All the rest | `skein_world::stream::Wire::answer` — `skein/testing/skein-world/src/stream.rs:86` | fragment delivery, intake and formatted tracing | 8.99% | 1.57% |
| All the rest | `skein_lib::intake::Intake::take` — `skein/crates/skein-lib/src/intake.rs:225` | allocate/copy/drain one delivery | 5.50% | 0.96% |

No smith, jig or temper function appeared in these measured profiles. Their cost outside the sampled traces is unknown, not inferred to be zero. Shared-kit shares for unprofiled tests are estimates only.

The focused suite remains over budget at this base. Both ordinary runs passed; this profile supplies measured causes for later tasks and claims no CPU saving. The queue’s figure after any repair, repair instruction counts, and a post-repair suite measurement are unknown because this task changes no code.

Artifacts retained locally under `target/hs2/`: `base-1/` and `base-2/` collectors plus summary JSON, `samples.json`, `cg/` unsplit profiles, `phase/` sampled function-boundary profiles, `floor/` cheapest-test boundaries, `boundary/` the pre-client trace, original tracer logs, annotation text and reconciled `phase-profiles.json`. Raw profiles remain unchanged; `annotation-inputs/` contains the explicitly normalized annotation copies. These target artifacts are not committed. The two collector CPU figures and all tables needed by the repair tasks are recorded here.

## Repairs

The aim is already met at the start of `last-round`: **74.51%** of the base CPU figure, below the required 85%. No further repair is made. These start figures are also the final figures.

Measured on **debian-32gb-hel1-11**, identified by `heavy hostname`, on 2026-11-06 UTC. The base is `39b490f13846b81fe78c5834c7852ffd28a25155`; the start is `044c2039d99f1b1c674a94ee9391428ddf0d447d`, after the family repairs merged. Both sides use the unchanged throwaway driver in `hearth/docs/plan/hearth-speed-2/notes/common.md`, “The method”, on this same machine, twice back to back per side. The slot was detached at the base for its two runs, then returned to the task branch. No build setting changed.

User plus system CPU includes each test's waited and reaped descendants; compilation, listing, nextest and the accounting supervisor are excluded. All 34 hearth packages were selected in each run. Every selected test passed, with one attempt each, no retries, starvation, unobserved tests, leaks or incomplete accounting. The four additional tests at the start belong to the merged repairs; this task changes no test, assertion, case, seed or count.

| Revision / run | User s | System s | CPU s | Suite wall s | Tests |
| --- | ---: | ---: | ---: | ---: | ---: |
| Base / base-1 | 27.062149 | 6.511324 | 33.573473 | 3.542120 | 2189 |
| Base / base-2 | 28.564805 | 5.632392 | 34.197197 | 5.241953 | 2189 |
| Start / start-1 | 21.156585 | 3.860159 | 25.016744 | 4.260360 | 2193 |
| Start / start-2 | 21.125782 | 4.311042 | 25.436824 | 4.712970 | 2193 |

The prescribed comparison takes the lower whole-suite CPU run on each side: **25.016744 / 33.573473 = 74.5134%**, a reduction of **8.556729 CPU s (25.4866%)**. The 85% threshold is **28.537452 CPU s**, leaving **3.520708 CPU s** of headroom. The queue's post-repair CPU figure is unknown: these are builder figures, not a queue run.

### Families

Both runs' CPU figures follow. User and system columns use base-1 and start-1, the lower whole-suite runs; family minima are not chosen independently.

| Family | Base tests | Start tests | Base-1 CPU s | Base-2 CPU s | Start-1 CPU s | Start-2 CPU s | Base-1 user s | Base-1 system s | Start-1 user s | Start-1 system s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Builder suite | 106 | 106 | 9.769513 | 9.784501 | 5.419252 | 5.412345 | 8.704639 | 1.064874 | 4.844197 | 0.575055 |
| Tree suite | 54 | 55 | 8.251019 | 8.610498 | 6.752063 | 6.989357 | 8.034640 | 0.216379 | 6.552452 | 0.199611 |
| Real git | 183 | 184 | 6.310449 | 6.285294 | 4.086104 | 4.038851 | 3.245028 | 3.065421 | 2.525307 | 1.560797 |
| Render suite | 109 | 109 | 3.432730 | 3.518854 | 2.806401 | 3.011307 | 3.111021 | 0.321709 | 2.578296 | 0.228105 |
| All the rest | 1737 | 1739 | 5.809762 | 5.998050 | 5.952924 | 5.984964 | 3.966821 | 1.842941 | 4.656333 | 1.296591 |

No gap remains to the aim, so the stop rule leaves the remaining options unmodified. Their post-repair instruction counts and potential savings are unknown; no new Callgrind profile is needed for a measurement-only stop. The earlier shared-kit estimates remain estimates, not measured savings at this start.

Local collector artifacts are `target/hs2/last-round-base-1/`, `last-round-base-2/`, `last-round-start-1/` and `last-round-start-2/`; complete driver output is in `target/last-round-{base,start}-{1,2}.log`.
