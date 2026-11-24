# temper's shell tests moved below

2026-11-24. The record that `temper/docs/design/draft/slow-tests-down.md`, 3.2 asks of its goal D14 (5.3): each test of `temper-forge-setup-shell::suite`, `temper-fetch-shell::real`, `temper-box-shell::real`, `temper-shell::real`, `temper-client-shell-tui::real`, `temper-service-shell::real` and `temper-engine-shell::suite`, what becomes of it, and beside each assertion that leaves it the lower test that holds it. It was read again at `17f69c0c01`, from the survey `slow-tests-survey-2026-11-23-temper-2.md`, groups 1 and 3, and seven read-only surveys of that commit. Where each kind of check lives, and why what stays stays, is `temper/docs/design/testing/shells.md`, 2 to 4.

- **Fates:** *Stays*, unchanged; *Shortened*, kept above with the rows or assertions named, the rest held below; *Extended*, kept above with an observation added; *Gone*, every assertion held below.
- **Holders:** *Added* marks a holder the goal's change adds, under the name given here; every other holder already ran at that commit. A change that places an added holder under another name or in another binary edits its row here. Binaries are named as `tools/tiers/layers.toml` names them, a library's own tests by its package's name. Line numbers are that commit's.
- **An added table keeps every row** of the slow table it holds, with each row's result, its words and the files it leaves, compared exactly where the slow row compared them.

## 1. `temper-forge-setup-shell::suite`

In `temper/crates/temper-forge-setup-shell/tests/suite/`. Its `boundary.rs` is already `temper-forge-setup-shell::step`'s and stays there.

| Test | Fate | Holders |
|---|---|---|
| `fetch.rs` `local_tls_fetch`:80, 8 rows | Shortened to `right` and `roots` | `temper-fetch-shell` `fetch::tests::pinned_fetch_rows` *Added*, the other six (1.1); `fetch::tests::fetch_request_heads` *Added*, each row's GET head |
| `fetch.rs` `local_tls_bottle_fetch`:262, 13 rows | Shortened to `right` | `temper-fetch-shell` `fetch::tests::bottle_fetch_rows` *Added*, the other twelve (1.2); `fetch::tests::fetch_request_heads` *Added*, both hops' heads; `temper-forge-setup-shell` `process::tests::extraction_rows` *Added*, the verdicts of 1.2's extraction rows |
| `fetch.rs` `the_lines_reach_the_writer_in_order`:503, 2 rows | Gone | `temper-fetch-shell` `fetch::tests::fetch_progress_rows` *Added*: with `Content-Length: 3`, the writer's bytes exactly `fetched runner 0 of 3 bytes\nfetched runner 3 of 3 bytes\n`; chunked, `fetched runner 0 bytes\nfetched runner 3 bytes\n`; each publishing `abc`, no part left |
| `fetch.rs` `a_writer_that_refuses_fails_no_fetch`:515 | Gone | `fetch::tests::fetch_progress_rows` *Added*, its refusing row: a writer refusing every write stays empty, the fetch succeeds, `abc` published, no part left |
| `fetch.rs` `claude_fetches_once_and_refuses_a_wrong_digest_or_existing_size`:523, 4 rows | Gone | `temper-fetch-shell` `fetch::tests::claude_download_rows` *Added*, every row (1.3) |
| `fetch.rs` `concurrent_claude_commands_make_one_request`:584 | Stays | |
| `files.rs` `a_file_is_written_whole_private_and_renamed_over`:8 | Gone | `temper-forge-setup-shell::step` `files::a_file_is_written_whole_private_and_renamed_over` *Added*, on the fake machine: `before` then `after` at one path; the name reads `after`, mode `0600` asked, a handle kept on the first node still reading `before`, one entry in the directory; the exclusive `.partial`, its write, sync, rename and the parent's sync in order |
| `files.rs` `clear_removes_a_first_preparations_files_and_never_backups`:21, 8 entries | Gone | `temper-forge-setup-shell::step` `files::clear_removes_a_first_preparations_files_and_never_backups` *Added*: the same eight entries laid on the fake machine; `forgejo/` and `runner/` with their files, `admin_token`, `person_password`, `person_read.header` and `person_write.header` gone; `state` and `backups/dump.zip` reading `kept` |
| `files.rs` `a_renamed_file_exists_under_its_new_name_only`:47 | Gone | `temper-forge-setup-shell::step` `files::a_renamed_file_exists_under_its_new_name_only` *Added*: the destination absent before; after, the source absent and the destination reading `PKcontents` |
| `prepare.rs` `a_later_preparation_leaves_the_row_a_runner_declared`:52 | Gone | `temper-forge-setup-shell::step` `prepare::a_later_preparation_leaves_the_row_a_runner_declared` *Added*, through `prepare::run_with` over a host whose commands' captures record each registration's argv and keep the runner's row: the first registration with `temper-host:host`; the row the runner declared, `temper-host`, kept by a rerun at the same settings and by one at capacity 2, whose YAML starts at capacity 2 and names `kept-runner`; with the YAML removed, registration again with the configured label, the UUID restored |
| `prepare.rs` `person_headers_are_whole_private_and_each_missing_header_is_remade_alone`:80, 4 rows | Gone | `temper-forge-setup-shell::step` `prepare::person_headers_are_whole_private_and_each_missing_header_is_remade_alone` *Added*, the host's files the fake machine's through `files::write_private`: both fresh headers exactly `Authorization: token private-token\n` at `0600`, the password at `0600`; each header missing in turn remade with the fresh line, the other keeping `Authorization: token kept-token\n` and its node |
| `prepare.rs` `an_interrupted_first_preparation_discards_both_headers_before_starting_again`:107, 2 rows | Gone | `temper-forge-setup-shell::step` `prepare::an_interrupted_first_preparation_discards_both_headers_before_starting_again` *Added*: `prepared` false and two abandoned headers; the rerun replaces both with the fresh line and leaves a state decoding `prepared == true` |
| `prepare.rs` `changed_runner_capacity_needs_the_lock_and_keeps_the_registration`:127 | Gone | `temper-forge-setup-shell::step` `prepare::changed_runner_capacity_needs_the_lock_and_keeps_the_registration` *Added*, the runner's lock held on the fake machine: identical YAML not written; capacity 2 `Failed::Held(File::RunnerLock)`, the YAML unchanged; after release, the YAML's first line at capacity 2, every line after the first two unchanged, the runner's secret not written |
| `locks.rs` `a_second_lock_is_refused_while_the_first_is_held`:5 | Gone | `temper-forge-setup-shell::step` `locks::a_second_lock_is_refused_while_the_first_is_held` *Added*, on the fake machine's locks: a second acquisition exactly `Refused::Held`; after the first's handle closes, a new one acquired |
| `locks.rs` `an_opened_file_lock_follows_its_inode_and_releases_with_its_handle`:15 | Gone | `temper-forge-setup-shell::step` `locks::an_opened_file_lock_follows_its_inode_and_releases_with_its_handle` *Added*: renamed between its open and its lock, the renamed name held, a replacement at the old name locked apart, the renamed one acquired once the first handle closes; no second open by path |
| `locks.rs` `a_child_holds_the_lock_as_its_stdin_after_the_parent_lets_go`:31 | Stays | |
| `commands.rs` `a_command_ends_with_its_status_and_both_streams`:5 | Stays | |
| `commands.rs` `output_past_the_bound_is_cut_and_the_command_ends`:13 | Gone | `temper-forge-setup-shell::step` `commands::output_past_the_bound_is_cut_and_the_command_ends` *Added*, `command::Running` over scripted turns: 100,000 zero bytes on each stream at bound 17; code 0, exactly 17 zero bytes each, both pipes read to their ends |
| `commands.rs` `a_program_that_is_not_there_is_reported_not_run`:22 | Gone | `temper-forge-setup-shell::step` `commands::a_program_that_is_not_there_is_reported_not_run` *Added*: a refused spawn exactly `Err(command::Error::Spawn(skein_io::Error::Other))` |
| `commands.rs` `a_source_command_deadline_closes_a_live_child_and_settles_both_pipes`:31 | Stays | |

### 1.1 A pinned fetch's rows

`pinned_fetch_rows` *Added* runs each row through `run_with`'s host over the cache's facts and scripted answers (`temper/docs/design/testing/shells.md`, 3.4), comparing the result or its exact words, the final file's bytes and its asked mode or its absence, the part file's absence, and whether a `Get` was asked; `fetch_request_heads` *Added* compares each asked `Get`'s head, `GET /binary HTTP/1.1`, `Host` with its port, `User-Agent: temper`, `Connection: close`:

- `wrong`: `xyz` against the `abc` pin; `Digest`, the URL and both full SHA-256 values; no final or part.
- `matching`: a cached `abc` taken unchanged; no `Get`, no progress, no part.
- `present`: a cached `xyz` refused, `<file>: does not match the pinned digest`, its bytes unchanged; no `Get`, no progress, no part.
- `leftover`: a longer part file left behind truncated; `abc` published at `0755`, no stale suffix, no part.
- `404`: an empty 404, exactly `<url>: HTTP status 404`; no final or part.
- `chunked`: no length, `ab` then `cd` past bound 3, exactly `<url>: body past setup.fetch_bytes`; no final or part.

### 1.2 A bottle's rows

`bottle_fetch_rows` *Added*, each row's typed refusal or success, both temporary files (`.part`, `.archive`) absent, the 32,768 `x` bytes published at `0755` or no final file; each row's two heads in `fetch_request_heads` *Added*, the first `/bottle` with `Authorization: Bearer QQ==` where the row has the token, the second `/object?signature=a%2Fb` with none:

- `mirror`: no token on either hop, the same success.
- `leftover`: both temporary files longer than their replacements, truncated; the exact bytes published.
- `archive digest`: the right archive against a wrong pin; `Digest`, nothing extracted, nothing published.
- `binary digest`: the member's digest not its pin; `Digest`, neither temporary file nor final left.
- `archive length`: a declared length one past the bound; `Bytes`, no file opened.
- `archive stream`: chunked, one past the bound; `Bytes`, the archive removed.
- `extraction bound`: the member one byte past the bound; `Bytes`, both temporary files removed.
- `missing member`: `bin/missing`; `Extraction`, nothing left.
- `empty member`: `bin/empty` and even the empty digest's pin; `Extraction`, nothing left.
- `corrupt archive`: `broken` at its archive pin, extraction failing; `Extraction`, nothing left.
- `second redirect`: the second hop's `307` to `https://storage.test/object`; `Redirect`, no third `Get`, no files.
- `gone`: an empty 404 from the second origin; `Gone`, not `Status`; nothing left.

`extraction_rows` *Added* makes the verdict of `binary digest`, `extraction bound`, `missing member`, `empty member` and `corrupt archive` from the member's bytes in pieces and the child's end (`temper/docs/design/testing/shells.md`, 3.5): the digest of the bytes, two 16 KiB pieces at the bound and one byte past it, a clean exit with no byte, a missing member's and a corrupt archive's failed exits; the child killed and waited for when the copy stops.

### 1.3 Claude Code's rows

`claude_download_rows` *Added*, through `claude_code_with` with the download over 1.1's host, each `Get` the head of `GET /binary`:

- `missing`: `abc` at the pin's size published, the writer's bytes exactly `fetching Claude Code\nfetched claude-code 0 of 3 bytes\nfetched claude-code 3 of 3 bytes\n`; metadata, prepare, lock, metadata again and download in that order; a second call silent, asking metadata alone, no lock, roots or `Get`.
- `digest`: `xyz` at the pinned size; `Digest`, the URL and both digests, one `Get`, no final or part.
- `bound`: a declared four bytes against size 3, exactly `<url>: body past the pinned size`; one `Get`, no part opened, no final.
- `size`: a cached file of another size, `<file>: does not match the pinned digest`, its bytes kept; metadata alone, no `Get`, no part.

## 2. `temper-fetch-shell::real`

In `temper/crates/temper-fetch-shell/tests/real/box_binary/tests.rs`.

| Test | Fate | Holders |
|---|---|---|
| `candidate_is_run_and_bounded`:17, 8 cases | Shortened to `checked`, a real child printing `checked\n` only for `--version` with an empty `HOME` | `temper-fetch-shell` `box_binary::tests::candidate_rows` *Added*, the other seven over the candidate's facts (`temper/docs/design/testing/shells.md`, 3.6): a matching identity exiting 0, exactly `temper-box 0.0.0 (0123456789ab)\n` and success; another identity exiting 0 and 1, `another\n` with success and without; 257 bytes on stdout, failure with exactly 256 kept, and 256 accepted; a regular file at `0600`, a directory and a missing path, never run, empty stdout and failure |
| `lock_cleanup_and_publication_order`:157, `publish` and `refuse` | Shortened to `publish` | `box_binary::tests::a_staged_build_of_another_identity_is_refused_under_the_lock` *Added*: the lock taken before the cache's read, the three stale parts removed, a staged child exiting 0 with `wrong build\n` failing the lookup; no `WriteSum`, `RenameSum` or `Publish`, no final binary, no part left, the lock held throughout. `publish` holds the lock's and the cache's native modes |

## 3. `temper-box-shell::real`

In `temper/crates/temper-box-shell/tests/real/`. Each *Added* holder is a `temper-box-shell` library test over the facts `temper/docs/design/testing/shells.md`, 3.7 to 3.10 names.

| Test | Fate | Holders |
|---|---|---|
| `fake/tests.rs` `retained_seed_outputs_survive_later_refusal_and_rollback`:8 | Gone | `fake::tests::admission::retained_seed_outputs_survive_later_refusal_and_rollback` *Added*, over `Disk` with distinct node identities: the first start leaving the seed's bytes and observations; the second accepting the matching output, creating a file at `0600`, refusing the observations by name; the rollback removing only the new file, the retained output's bytes and node and the observations' bytes kept; the effects in order |
| `fake/tests.rs` `unsafe_or_different_retained_seed_outputs_are_named_and_refused`:44, 6 rows | Gone | `fake::tests::admission::unsafe_or_different_retained_seed_outputs_are_named_and_refused` *Added*, every row, each refusal naming the output's path with nothing owned: `bytes`, `seed` changed to `other`; `symlink`, a link to matching bytes, never opened; `hardlink`, two links; `mode`, `0644`; `oversize`, the read asked at the expected length plus one and answered with one byte more; `directory` |
| `fake_backends/tests.rs` `given_scripts_join_at_each_start_without_replacing_the_model`:35 | Gone | `fake_backends::tests::given_scripts_join_at_each_start_without_replacing_the_model` *Added*: A's and B's `cued` and `asked` joined in order, the fallback `X` kept when B has none, replaced by B's `otherwise: []` on restart; a plain start removing the join and both flags; strict alone adding `--strict` with no join; only `fake model` taking the argv suffix after its six words; the join at `0600`; `llm.json` unchanged |
| `fake_backends/tests.rs` `a_reservation_that_loses_its_port_takes_another`:101 | Gone | `fake_backends::tests::a_reservation_that_loses_its_port_takes_another` *Added*: one `AddrInUse`, then exactly one retry reserved at a nonzero port. `peer_reservations_transfer_once_and_close_with_preparation` holds a reserved port held natively |
| `fake_backends/tests.rs` `peer_reservations_transfer_once_and_close_with_preparation`:120 | Stays | |
| `fake_backends/tests.rs` `explicit_bundle_leaves_existing_empty_source_configuration_untouched`:142 | Gone | `fake_backends::tests::explicit_bundle_leaves_existing_empty_source_configuration_untouched` *Added*: no effect under the source's directory; the selected bundle, the effective worker secret, no source configuration, validation passing. `peer_reservations_transfer_once_and_close_with_preparation` holds the listeners |
| `fake_backends/tests.rs` `fresh_preparation_is_private_and_isolates_imported_resources`:157 | Shortened to the native modes, `0700` roots, configuration and data, `0600` marker, lock, configuration, GitHub, OAuth and LLM documents and worker secret, and a second preparation refused, `another fake-backend box` | `fake_backends::tests::fresh_preparation_is_private_and_isolates_imported_resources` *Added*: configuration and data apart from the source's, configuration at `D/box/fake-backends/config`, the cache shared, no `workers[0].agent`, the provisioned bundle selected, validation passing, each file's mode asked |
| `fake_backends/tests.rs` `reuse_atomically_replaces_bundle_and_preserves_every_other_edit`:196 | Shortened to the rename: a handle held on the old document reading its old bytes, the name a new node at `0600` | `fake_backends::tests::reuse_atomically_replaces_bundle_and_preserves_every_other_edit` *Added*: only the bundle member replaced; the returned bytes the reparsed document; the marker, the GitHub, OAuth and model documents, the worker secret and the retained data unchanged; a later unflagged start keeping the selection; staging at `0600` exclusively, its write, sync and rename in order, no staging left |
| `fake_backends/tests.rs` `repeating_the_same_bundle_preserves_document_bytes_and_inode`:234 | Gone | `fake_backends::tests::repeating_the_same_bundle_preserves_document_bytes_and_inode` *Added*: no publication effect, the bytes kept, the returned bytes the parsed document, validation passing |
| `fake_backends/tests.rs` `invalid_replacement_bundle_preserves_retained_document`:250 | Gone | `fake_backends::tests::invalid_replacement_bundle_preserves_retained_document` *Added*: the refusal naming `index.html`, no staging opened, the document's bytes kept |
| `fake_backends/tests.rs` `invalid_retained_configuration_refuses_a_valid_replacement_without_mutation`:260 | Gone | `fake_backends::tests::invalid_retained_configuration_refuses_a_valid_replacement_without_mutation` *Added*: an appended `[application] workers = []` refused naming `application`, before any staging, with a valid replacement; the bytes kept |
| `fake_backends/tests.rs` `fake_adapter_is_retained_outside_fixed_worker_configuration`:272 | Gone | `fake_backends::tests::fake_adapter_is_retained_outside_fixed_worker_configuration` *Added*: no `application.workers`; `fake-claude.path` holding the adapter's path at `0600`; with it absent from a prepared sandbox, the error naming the record, no `PATH` lookup |
| `fake_backends/tests.rs` `own_app_key_is_the_seeded_apps_private_pem`:286 | Gone | `fake_backends::tests::own_app_key_is_the_seeded_apps_private_pem` *Added*: `D'/github-app.pem` the generated GitHub document's App key, its RSA PEM header, `0600` asked, its bytes kept on reuse |
| `fake_backends/tests.rs` `a_new_sandbox_takes_the_nine_ports_its_test_named_and_a_restart_keeps_them`:326 | Shortened to the native bind: slots 0, 4, 5, 6, 7 and 8 held while prepared, all nine bound after the drop | `fake_backends::tests::a_new_sandbox_takes_the_nine_ports_its_test_named_and_a_restart_keeps_them` *Added*: the nine named ports in the documents and the peers, a restart with another choice keeping them |
| `fake_backends/tests.rs` `a_named_port_held_elsewhere_refuses_a_new_sandbox_naming_it`:345, 2 rows | Stays | |
| `tests/upgrade.rs` `checkpoint_checks_take_only_the_checkpoint_lock_and_leave_data_unchanged`:127, `--check` and `--workers` | Gone | `upgrade::tests::checkpoint_checks_hand_their_request_to_the_engine` *Added*: the standalone preflight writing nothing under an existing `--data`; `Request::Check` and `Request::Workers` each handed to the engine's prepared run unchanged, with the configured deployment. `temper-engine-shell::suite` `checkpoint_checks_take_only_the_checkpoint_lock_and_leave_data_unchanged`, *Extended* (7): each request through `run_prepared`, 0 with the deployment held, 1 with the checkpoint's lock held, the tree unchanged |
| `init/tests.rs` `a_port_another_init_listens_on_between_bind_and_listen_gets_a_kernel_port`:21 | Stays | |
| `init/tests.rs` `a_kernel_chosen_port_another_takes_before_listen_gets_another`:42 | Stays | |
| `init/tests.rs` `init_finds_ports_and_programs_and_writes_exact_modes_in_order`:59, 6 rows | Shortened to one native init from kernel-chosen ports: boxed, reread through setup, its real modes and the git link's target, two held listeners matching the document, and a second init asking those ports given others | `init::tests::init_finds_ports_and_programs_and_writes_exact_modes_in_order` *Added*, every row over init's facts: for two deployments, each port's preference kept or fallen back (free, held, the same preference twice), Claude found or not, git missing or not with its link written; the secret, the git link and the configuration in that order at `0600`, the link and `0644`; `config`, `data`, `cache`, `demo` and `demo/bin` at `0700`; no `demo/llm.key` or `charters`; `clients.bundle` null and `clients.public` present and null; no `application.workers`, `machine`, `demo` or `engine.llm`; `missing_git`; Claude `fetched` and the agent preflighted with its tools |
| `init/tests.rs` `init_renders_only_listen_unless_its_next_port_is_taken`:206, 3 rows | Gone | `init::tests::init_renders_only_listen_unless_its_next_port_is_taken` *Added*, every row: the exact bytes at `0644`, the configuration last, no former file, the clients port; bare and boxed with webhooks at clients + 1 and `listen` alone; boxed with it taken and the webhooks' address written out |
| `init/tests.rs` `a_bare_configuration_passes_the_same_checks_as_serve`:251, 2 rows | Gone | `init::tests::a_bare_configuration_passes_the_same_checks_as_serve` *Added*, both rows: setup and naming admitted with a scripted provider, no `machine`, `welcome` or `demo`; `Person::Found`, no login; the secret then the configuration; two distinct ports matching the reservations; Claude `fetched` with it absent and present, the agent preflighted with its tools |
| `init/tests.rs` `init_keeps_an_existing_provider_key_even_without_openssl`:294 | Gone | `init::tests::init_keeps_an_existing_provider_key_even_without_openssl` *Added*: the existing key's node and size kept, no OpenSSL lookup or command, no write; no sign-in warning; the reread naming `provider.key` and no clients |
| `init/tests.rs` `init_makes_a_private_3072_bit_provider_key_before_writing_its_document`:320 | Stays | |
| `program_login/tests.rs` `the_printed_line_preserves_shell_arguments_from_another_directory`:8 | Stays | |
| `tests.rs` `id_login_agrees_with_the_effective_users_passwd_entry`:13 | Stays | |
| `tests.rs` `the_standalone_operations_check_retains_no_descriptors`:27 | Stays | |
| `tests/box_kinds.rs` `a_held_descriptor_is_refused_as_held`:5 | Stays | |
| `tests/box_link_form.rs` `the_box_reads_its_stdout_and_term`:5, 5 rows | Shortened to the pseudo-terminal with `TERM=xterm-256color` and `/dev/null` | `box_command::tests::the_box_reads_its_stdout_and_term` *Added*, every row over the terminal's verdict and the environment's rows: a terminal with `TERM=xterm-256color`, OSC 8; with `HOME=/h` and `TERMCAP=dumb` and no `TERM`, OSC 8; with `TERM=dumb`, bare; no terminal with `TERM=xterm-256color`, bare, for the file's row and `/dev/null`'s |

## 4. `temper-shell::real`, gone

In `temper/crates/temper-shell/tests/real/`. The binary goes, with its lines in `tools/tiers/layers.toml`, `.config/nextest.toml` and `tools/tiers/packages.toml`.

| Test | Fate | Holders |
|---|---|---|
| `directories/tests.rs` `roots_are_private_and_checked_at_their_own_names`:5, 6 rows | Gone | `temper-shell` `directories::tests::roots_are_private_and_checked_at_their_own_names` *Added*, every row over `temper/docs/design/testing/shells.md`, 3.11's facts: `absent`, the parent and the root made `0700`; `private`, kept at `0700`; `public`, `link` and `file`, each refused with exactly `PATH: must be a private directory of this user's, not reached through a link`; `parent-link`, the root made through the linked ancestor and `0700` through the name given |
| `link_file/tests.rs` `keeping_replaces_the_whole_private_file_and_forgetting_is_quiet`:11 | Gone | `temper-shell` `link_file::tests::keeping_replaces_the_whole_private_file_and_forgetting_is_quiet` *Added* (3.12): the URL and one LF at `0600`, both missing parents at `0700`; `short` replacing it, exactly `short\n` and one entry; forgotten, removed; forgotten again, quiet |
| `link_file/tests.rs` `keeping_refuses_an_insecure_or_foreign_parent_without_disclosing_the_url`:30, 2 rows | Gone | `temper-shell` `link_file::tests::keeping_refuses_an_insecure_or_foreign_parent_without_disclosing_the_url` *Added*, both rows, `0770` with the user's owner and `0700` with another's: the refusal starting with the target's path and `: `, no URL byte in it, no file left |
| `link_file/tests.rs` `reading_refuses_nonprivate_foreign_and_oversize_files_and_strips_one_lf`:54, 8 cases | Gone | `temper-shell` `link_file::tests::reading_refuses_nonprivate_foreign_and_oversize_files_and_strips_one_lf` *Added*, every case, the read asked for the target, the user and 4,096 bytes: absent, no link; `final LF`, the URL; `one LF only`, `two\n`; `no LF`, `bare`; `group-readable` (`Unsafe(Mode(0o040))`), `foreign owner` (`Unsafe(Owner)`) and `past MOST` (`TooLarge`), no link; `at MOST`, the 4,096 bytes whole |
| `lookup/tests.rs` `git_lookup_skips_missing_nonexecutable_and_nonabsolute_entries`:5 | Gone | `temper-shell` `lookup::tests::git_lookup_skips_missing_nonexecutable_and_nonabsolute_entries` *Added* (3.13): a missing absolute candidate before a regular `git` at `0700`, that `git`; at `0600`, none; only empty and relative entries, no candidate and none found |

## 5. `temper-client-shell-tui::real`, gone

In `temper/crates/temper-client-shell-tui/tests/real/start/tests.rs`. The binary goes, with its lines as in 4.

| Test | Fate | Holders |
|---|---|---|
| `origin_directories_keep_components_private_and_unshortened`:5, 4 rows | Gone | `temper-client-shell-tui` `start::tests::origin_directories_keep_components_private_and_unshortened` *Added*, every row through `start::deployment::prepare_with` on its fake files, the verdict `private_root_with`'s: `at/127.0.0.1/8367`, `at/::1/8367` without its brackets, `at/dev.fabrig.ai/443` and a 253-byte name unshortened at 443; each made on the first preparation and not the second, `at`, the host's and the port's directories at `0700` |
| `origin_directories_refuse_a_public_or_linked_parent`:27, 2 rows | Gone | `temper-client-shell-tui` `start::tests::origin_directories_refuse_a_public_or_linked_parent` *Added*: an existing `at` at `0755`, and `at` a link to the TUI's root, each refused |

## 6. `temper-service-shell::real`, gone

In `temper/crates/temper-service-shell/tests/real/`. The binary goes, with its lines as in 4.

| Test | Fate | Holders |
|---|---|---|
| `tests.rs` `a_push_key_refuses_nonprivate_files_naming_its_member`:2 | Gone | `temper-service-shell` `tests::a_push_key_refuses_nonprivate_files_naming_its_member` *Added*, over `read_files_with_asked_using`: `application.push.key`, `push.pem`, required, private, bound 1,024, the user's; a mode refusal starting `application.push.key:`, then the fixture's bytes accepted. `skein-ring-tests::suite` `startup::a_file_with_group_or_other_bits_is_refused_unread_as_mode` holds the native mode's refusal |
| `providers/tests.rs` `restore_and_changes_are_whole_private_and_unchanged_bytes_are_not_written`:9 | Gone | `temper-service-shell` `providers::tests::restore_and_changes_are_whole_private_and_unchanged_bytes_are_not_written` *Added* (3.14): an absent copy written, `true`, at `0600`, read back to the exact values; equal bytes, `false`, no effect at all; the endpoint's port 8080 changed to 8081, `true`, the changed values, no `providers.setting.new` left |

## 7. `temper-engine-shell::suite`

In `temper/crates/temper-engine-shell/tests/suite/`.

| Test | Fate | Holders |
|---|---|---|
| `upgrade.rs` `stopped_preparation_names_bad_held_lines_without_opening_or_changing_the_store`:32, 2 rows | Gone | `temper-engine-shell` `stopped::tests::stopped_preparation_names_bad_held_lines_without_opening_or_changing_the_store` *Added* (3.15): `not.a.limit 1`, starting `limits-held line 1: not.a.limit is no limit of this release`; `people.identity_bytes 4294967296`, starting `limits-held line 1: people.identity_bytes (setting limits):`; neither containing `put limits-held`; no store opened or adopted |
| `upgrade.rs` `checkpoint_checks_take_only_the_checkpoint_lock_and_leave_data_unchanged`:53, `Check` and `Workers` | Extended: with the checkpoint's lock held, each request also refused through a fresh stopped preparation and `run_prepared`, 1, the tree unchanged | |
| `upgrade/tests.rs` `deployment_rehearsal_with_accounts_workers_and_forge_reaches_its_report`:14 | Gone | `temper-one-process-world::suite` `configured_accounts_rehearsal_starts_only_the_store_and_reaches_its_report`: the report, exit 0 through the restores, the source unchanged. `temper-engine-shell` `upgrade::tests::a_rehearsals_sweep_takes_only_stale_unlocked_scratch` *Added* (3.16): the stale `upgrade-00000000000000aa` removed, an outside file's links from 2 to 1; the busy `…bb`, as old, kept on its lock's refusal with no removal asked, `in use` unchanged; `upgrade-x`, `other`, `…aA` and `…aaa` skipped before any metadata, lock or removal; the file `…cc` and the link `…dd` kept, unfollowed, with no lock or removal; only the stale entry gone. `upgrade::tests::a_rehearsals_end_removes_its_own_scratch` *Added*: the run's scratch removed after its report, on 0 and on 1 |
| `upgrade/tests.rs` `a_rehearsals_own_scratch_is_swept_only_after_its_lock_is_released`:63 | Stays | |
| `upgrade/tests.rs` `own_scratch_removal_releases_all_contents_and_outside_links`:87 | Gone | `temper-engine-shell` `upgrade::tests::own_scratch_removal_releases_all_contents_and_outside_links` *Added*, through `remove_with` on the fake machine: the whole tree gone, no diagnostic, the outside file reading `checkpoint bytes` at one link, the removal before the lock's release |
| `upgrade/tests.rs` `a_refused_rehearsal_leaves_no_own_scratch_in_the_cache`:102 | Gone | `temper-one-process-world::suite` `a_held_source_refuses_a_rehearsal_after_its_scratch_is_made` *Added*: the source held, `Busy` through the store-first opening, the report, exit 1. `temper-engine-shell` `upgrade::tests::a_rehearsals_end_removes_its_own_scratch` *Added*, its row of 1 |
| `upgrade/tests.rs` `deployment_back_with_accounts_workers_and_forge_reports_no_previous_generation`:119 | Gone | `temper-one-process-world::suite` `a_configured_floor_goes_back_to_no_previous_generation` *Added*: the configured floor going back, `BackRefused::NoPrevious`, its report and exit 1. `jig-store-world::suite` `back_refuses_committed_missing_previous_and_unreadable_manifest` holds the store's decision |
| `checkpoint.rs` `stopped_checkpoint_stands_with_its_record_and_removal_reports_missing_twice`:7 | Shortened to the take and the first removal | `temper-engine-shell` `checkpoint::tests::a_missing_hold_program_and_a_second_removal_end_as_stopped` *Added*: `no-such-command` 127, no child run, the hold released before the end; a second removal of a removed name, 1. `temper-engine-service-component` `checkpoint::tests::failed_spawn_and_missing_program_release_the_holder_before_finishing` and `jig-protocol-store` `named_holds_block_other_readers_removal_and_retention_until_close` hold the release and the missing name |
| `restore.rs` `stopped_restore_switches_generation_and_keeps_source_and_safety_and_refuses_missing`:7 | Shortened to the take and the restore | `temper-engine-shell` `restore::tests::restore_terminals_render_exact_lines_and_exits_for_each_binary`, a row *Added*: `CheckpointName { store: u64::MAX, suffix: 0 }` missing, exit 1. `jig-store-world::suite` `source_and_deployment_refusals_leave_the_live_pair_unchanged` holds the transition |
| `lines/tests.rs` `the_playground_start_makes_a_private_random_key_before_reading_it`:4 | Stays | |
| `forge/tests.rs` `prepare_flags_write_one_private_setup_and_later_flags_must_agree`:19 | Shortened to the first preparation's native write: `forge-1/setup` at `0600` under a `0700` parent | `temper-engine-shell` `forge::tests::prepare_flags_rows` *Added*, over `forge::setup_file_with`'s facts and the setup's bounded bytes: the first setup, owned, `http://127.0.0.1:3000`, both fetch flags, capacity 1 and the default runner's path; a repeat with the same flags and one with none returning it with no write; `127.0.0.1:3001` refused naming the file's 3000 and the flag's 3001, the bytes kept; a relative `bin/forgejo` compared resolved for forge 2; forge 3's `run` with no setup naming `forge-3/setup`, no write; version 2 bytes naming `forge-1/setup` and `Version(2)`; fake forge 7 on 3007 for `ada`; `forge::configure` taking the directories, the forge and the flags to `Command::Forge` with `prepare` first |
| `forge_sign_in/tests.rs` `credentials_are_private_whole_and_left_in_place_on_a_second_run`:8 | Stays | |
| `serve/program_environment.rs` `the_words_survive_a_shell`:6 | Stays | |
| `serve/program_environment.rs` `the_widest_directories_fit_the_wire`:42 | Gone | `temper-engine-shell` `serve::program_environment::tests::the_widest_directories_fit_the_wire` *Added*, unchanged: three paths of 4,096 apostrophes and `temper`, exactly 49,206 bytes, round-tripping at `w::CEILINGS` |
