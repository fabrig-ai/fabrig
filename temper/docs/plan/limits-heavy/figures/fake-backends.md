# fake-backends: figures

Counts are instructions (Ir) of the test and its traced children together; listing processes are excluded. `unknown` means not individually profiled. The source at the base has 47 tests in this module; all remain, with two added.

| Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---:|---:|---|---|
| `temper::lib::fake_backends::tests::chosen_ports_refuses_all_but_nine_nonzero_ports` | 6,219,759 | 6,221,247 | Process/loader floor; no preparation fixture | stays: all five invalid-port cases |
| `temper::lib::fake_backends::tests::fresh_preparation_is_private_and_isolates_imported_resources` | 3,123,457,580 | 879,454,153 | Optional RSA provider-key generation and repeated source setup checks | Explicit fixture environment; reuse admitted bundle limits |
| `temper::lib::fake_backends::tests::retained_peer_edits_are_checked_without_replacement` | 19,892,711,440 | 2,481,957,851 | Six complete preparations, six RSA keys, repeated source checks | One prepared fixture, restore each peer document; explicit environment; reuse limits |
| `temper::lib::fake_backends::tests::missing_required_retained_documents_refuse_without_writes` | 19,819,608,616 | 1,525,518,072 | Six complete preparations, six RSA keys, repeated source checks | One prepared fixture, restore each required document; explicit environment; reuse limits |
| `temper::lib::fake_backends::tests::oversized_source_bundle_refuses_before_sandbox_creation` | unknown | 172,875,213 | New regression test; source admission and early bundle refusal | Holds the admitted source bundle bound and refusal before sandbox writes |
| `temper::lib::fake_backends::tests::preparation_does_not_discover_optional_programs_outside_its_environment` | unknown | 497,843,702 | New regression test; fresh preparation without optional executables | Holds explicit optional-program facts and absence of an unneeded provider key |
| `temper::lib::fake_backends::tests::peer_reservations_transfer_once_and_close_with_preparation` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::explicit_bundle_initializes_without_creating_absent_source_configuration` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::explicit_bundle_leaves_existing_empty_source_configuration_untouched` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::explicit_bundle_overrides_an_unavailable_source_bundle_without_rewriting_source` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::explicit_bundle_imports_init_configuration_without_a_source_bundle` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::empty_configuration_refuses_before_creating_data_or_cache` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::initialized_source_without_bundle_refuses_before_sandbox_mutation` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::partial_sandbox_refuses_with_explicit_reset_instructions` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::reuse_keeps_effective_edits_peer_addresses_and_data` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::reservations_avoid_every_port_the_peer_check_refuses` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::reuse_atomically_replaces_bundle_and_preserves_every_other_edit` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::repeating_the_same_bundle_preserves_document_bytes_and_inode` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::invalid_replacement_bundle_preserves_retained_document` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::invalid_retained_configuration_refuses_a_valid_replacement_without_mutation` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::staging_file_collision_leaves_the_retained_document_untouched` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::reuse_refuses_a_missing_effective_bundle_and_preserves_edits` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::current_init_import_ignores_charter_files_and_rebases_paths` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::unsupported_imports_refuse_with_the_public_member_named` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::effective_commands_reopen_the_retained_directories` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::missing_provisioned_directory_refuses_before_data_or_cache_mutation` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::oversized_explicit_bundle_refuses_before_creating_data_or_cache` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::unsupported_provisioned_file_refuses_before_sandbox_creation` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::linked_provisioned_page_refuses_before_reading_its_target` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::source_public_asset_links_are_refused_before_reading_them` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::import_preserves_disabled_git_and_adds_the_fake_program` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::fake_github_machine_passes_ordinary_admission_without_document_forges` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::foreign_retained_adapter_refuses_without_resetting_or_rewriting_it` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::source_adapter_is_a_bounded_fact_without_path_discovery` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::own_app_key_is_the_seeded_apps_private_pem` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::changed_source_bytes_or_directory_require_explicit_reset_without_writes` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::a_new_sandbox_takes_the_nine_ports_its_test_named_and_a_restart_keeps_them` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::a_named_port_held_elsewhere_refuses_a_new_sandbox_naming_it` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |

## Found and not fixed

- **Repeated protocol admission and JSON conversion.** In the final fresh test, protocol parsing is 511,114,381 Ir (58.1%); `setup::check` is 397,821,765, `Source::read` 303,629,763, `Json::decode` 189,530,715, and the main recursive JSON writer 139,690,933. These inclusive figures overlap. In retained-peer edits, parsing is 1,520,897,042 and source admission 1,063,751,758; in missing documents, parsing is 971,376,362 and source admission 1,062,774,350. Every changed retained/source document still needs admission. Some consumers parse the same bytes again: sharing a checked protocol document and typed deployment through setup, step assembly and peer validation would address that remaining repetition while preserving refusal order. This is remaining work above the common note's approximate stopping threshold, recorded here rather than claimed cheap.
- **Production provider-key generation.** The traced fresh baseline spends 2,169,860,694 Ir in OpenSSL, of which 2,150,272,957 is inclusive RSA generation. Its work is random: the six children in the peer-edit baseline total 13,935,005,798 Ir, and those in missing documents total 14,827,327,225. A real environment that needs a new provider key still generates its fresh 3072-bit key. These preparation tests hold no rule about that provider key; its generation has its focused test in `init::tests::init_makes_a_private_3072_bit_provider_key_before_writing_its_document`. The GitHub App key is separate, seeded, and its existing test remains.
- **Small initialization and reservation costs stay.** In the initial parent-only fresh profile, source `init::write` is 3,986,792 Ir, sandbox `write_with_program` 8,358,880, and peer `reserve` 5,076. File publication and held listeners are required by the checks. There is no profile evidence for changing their algorithm or replacing fresh worker secrets.

## Tests that went

None.

## Belonging

Read every body in `temper/crates/temper/src/fake_backends/tests.rs`:

- Argument grammar, bundle path bounds, echo and printed command tests stay focused: direct configuration and formatting calls hold their decisions without a process.
- Reservation retry, refusal, ownership and chosen-port tests stay focused: direct calls with injected bind outcomes or local listeners hold descriptor ownership and address choices, without supervising a process.
- Fresh preparation, source import, isolation and ordinary admission tests stay focused: scratch documents exercise preparation and validation directly; no path through `main` is needed.
- Retained edits, identity, required documents, locking and reset tests stay focused: direct preparation calls hold the retained sandbox's admission and refusal rules.
- Bundle selection, replacement, staging, bounds, links and missing provisioning tests stay focused: files and direct calls hold validation order, atomic publication and preservation on refusal.
- Adapter provisioning and seeded App key tests stay focused: direct calls check identity, resource bytes and file modes; they start no adapter or peer.

These groups confirm `temper/docs/plan/limits-heavy/plan.md`, Belonging: no test needs the process tier, and none spawns temper.

## Measurement and changes

All primary before/after profiles used the same slot and unoptimized test build, one named test at a time:

```sh
NEXTEST_DOUBLE_SPAWN=0 TEMPER_BUNDLE=none \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --tool=callgrind --trace-children=yes --callgrind-out-file=$PWD/target/limits-heavy/NAME.%p.cg" \
heavy cargo nextest run -p temper --lib -E 'test(=fake_backends::tests::NAME)'
```

Before is the pristine task base; after is the completed change. Both selected exactly one test in the default profile and passed. The floor has no children; the other three baselines have respectively one, six and six OpenSSL children. The final profiles have no children. RSA prime searches make the before totals observed samples, rather than deterministic constants. The profile identifies the cause without depending on CPU timing.

The parent-only profiles separate the other changes from random key generation. Reusing the fixture reduced peer edits from 5,958,103,389 to 2,556,035,237 Ir and missing documents from 4,992,271,989 to 1,599,680,105. Keeping source admission's bundle limits then reduced a fresh parent from 953,772,289 to 880,194,066 Ir; the final explicit-environment parent is 879,454,153. Intermediate profiles omitted child tracing and are not the totals in the main table.

The environment is an explicit input to the private preparation core, as it already is to init. The command wrapper still supplies the real environment. The test helper supplies no optional executables, matching its source initializer: these assertions check preparation, peers, imports and retention, and do not need a generated provider key. No environment variable is mutated and no credential is borrowed or read for a fixture.

Each multi-case test prepares once. Peer edits restore the original public document bytes and private mode after every case; the successful edit still reserves its replacement while the fresh listeners are held. Missing documents restore the removed document after the unchanged-listing, unchanged-marker and absent-document assertions. All six cases of each test still run, with the same refusal or preservation assertions.

`Source::read` keeps the bundle limits from its successful setup check; preparation consumes them without encoding and checking the same normalized source again. Empty sources retain the small-profile limits. This saves one source encode/check for a person's first fake-backend preparation from an initialized source. It does not speed ordinary box initialization or `temper::suite`'s ordinary startup path. Actual provider-key generation remains unchanged.

The new bundle regression failed when early bundle admission was bypassed: the sandbox directory existed after refusal. The new environment regression failed when initialization read the real environment instead of its supplied facts: optional git appeared in the effective document. Restoring each rule made its named test pass. All 49 module tests also passed together.
