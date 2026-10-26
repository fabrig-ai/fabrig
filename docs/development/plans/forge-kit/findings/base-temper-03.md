# Base runs: temper 03, temper on `skein-forge-connection`
The draft's 03.2, 03.3 and 03.4 (the conformance on skein-json) are temper 06.2, 06.3 and 06.4; its 03.5 is temper 03.2.

2026-10-19. Code: temper dd12b8ed904edced9e65af9fa62cc769c5a09905 (the code of
27995fb75, with the designer's answers), run in
`/srv/data/git/runner/src/tmp/planning/temper-base`; skein 064c55f (docs only here:
skein 03.1 is not on origin/main, so the kit's `skein_forge_connection` could not be
compiled at the base). Deliverables: `temper/03-connection.md` and this file. The
scratch worktree `tmp/planning/scratch-temper-03` (detached at dd12b8ed9) held the
sketches and the conformance recordings; it is removed at the end (last section).
Every run went with unoptimized builds and NEXTEST_DOUBLE_SPAWN=0.
Outputs are cut to the relevant lines (at most 30 per command); full logs are in
`/srv/data/git/runner/src/tmp/logs/`, named by task. reuses task numbers: each
block's log path (the timestamp in its name) is the identity, not the number.
`$S` below is `/srv/data/git/runner/src/tmp/planning/scratch-temper-03`. A block
that says "derived" shows a count made from the lines recorded just before it.

## What the base showed beyond the prompt

- **The repair of skein 02.2's client changes is already done.** At the base the
  measure callers (`stored_bytes`, `effect_bytes`, `answer_bytes_unbounded`) are in
  `temper-engine-domain-forge` (5 lines), `temper-engine-domain` `store.rs` (3) and,
  inside the component crate (which leaves), two tests; `bind_deployment` already
  returns bool with one caller, `restart.rs`. Temper 02.1 changes them before 03.1
  starts, so 03.1's repin repairs nothing of them (section "Users", d08).
- **The prompt's 03.3 (the mapping) cannot wait behind the conformance.** `problems()`
  in `temper-service/src/forge.rs` is an exhaustive match on `Unusable`; skein 03.1 adds
  `Word` and `PlaintextName`, so temper 03.1's repin does not compile without their
  arms. The two arms, their tests and the indexing of `PlaintextName` and `Request`
  therefore land in 03.1 (skein 03.1's "Temper's side" says so). What is left for a
  later increment is the repin onto skein 03.2/03.3 (head check, page sizes).
- **The README says the mapping needs skein 03.2; it needs skein 03.3 too.** Skein
  03.3 prices pages by `measure::answer`, which changes temper's page-size expectations
  (skein 03.3, "Temper's side"). Written in the file's Needs: skein 03.3 (which needs
  03.2). The README row, "Increments that need more" and the critical path
  (`temper 03.1 -> temper 03.2 -> skein 06.2`) name the increments by the prompt's
  numbers and are renumbered (below).
- **The conformance move is larger than decision 27's "about 250 lines".** The 17 files
  that use serde_json hold about 450 lines that build or read a `Value` (d15 to d17:
  30 lines naming serde_json in 17 files, 69 `json!(` sites in 14 files, and at most
  347 lines of `.as_str()`, `["member"]` or `is_string()` reads in 18 files), and the
  helper that replaces the tree (`json.rs`, compiled in the scratch) is 242
  production lines and 57 of tests. That is about 700 production lines, so it is
  three increments, 03.2 to 03.4, each under 300 new lines, and the repin onto skein
  03.2/03.3 (the head check, the page sizes) becomes 03.5. Outside my files: README (row,
  critical path, "the conformance after temper 03.2"), `skein/06-conformance.md`
  (06.2 "Needs: temper 03.2" becomes the last conformance increment, 03.4) and
  decisions.md 27 and its table row name "temper 03.2".
- **The protocol world is a user though the prompt does not list it.**
  `tests/engine/protocol-forge` (`temper-engine-protocol-forge-world`, 47 tests, 13
  lines naming the crate in 7 files; its `settings()` builds the kit's `Forge` with a
  `host:` field, `fixtures.rs` line 100) depends on the crate (`Cargo.toml` line 18). It stays until temper 07.1, so
  03.1 repoints it at `skein_forge_connection` and hands it the new settings.
  `tests/forge-setup/reconcile` needs no changed line (it names the world only).
- **Hotspots.** The README's Rules do not name `temper-service`'s `forge.rs` and
  `settings.rs` (only `temper/01-forge.md` does, for 01.3); 03.1 and 03.5 edit both,
  and 03.1 edits `tests/fixtures.rs::forge_runtime`. The listener's `hooks.rs`
  (README, Rules) is deleted with the crate in 03.1.
- **Host.** The base derives the Host from the `api` address (`conventions::host`); the
  new derivation takes a TLS forge's Host from its server name. Equal for every
  fixture and for `forge.example`; recorded as a behaviour change of skein 03.1 that
  temper's wire test (`the_start_request_carries_the_user_agent_and_the_host`) pins.
- **Head size.** Skein 03.2 adds the user agent's 12 bytes to the head check. By hand,
  the shipped small profile's longest TLS head is far under its `tls.send` of 16 384
  (derived, not run); the repin's run shows it. No limit is raised; a `Request` or
  `PageSize` refusal of the shipped profile at the repin is a design question.
- **Nothing in temper pins a malformed or uppercase signature** (answer 3): d14/f02
  list the signature users; temper's writers emit 64 lowercase hex. The only
  `temper/0.0.0` pin is inside the crate (`a_json_upload_sends_its_body_and_content_headers`),
  which moves to skein with the crate, so temper adds its own wire test.
- **The process tier fails at the base**, twice in this drafting, alone in the gate
  lane (blocks "Process tier"). Reported, not read as a pass.
- **The gate counts do not chain.** The sibling files' gate counts (1791 after 01.4,
  1808 after 02.1) do not start from the base's 1879, so 03.1's Proof states "drops
  112 (117 at the base), adds 5" and names no absolute total.
- **No extra dependency.** `skein-forge-connection` joins the workspace dependencies
  in the repin; `skein-json` is already one (`Cargo.toml` line 121). `serde_json`
  stays a workspace dependency (29 other files in six other packages; d15): only the
  conformance's manifest loses it.

## The crate that moves (base form of the proofs)

The component's crate exists at the base, so each proof of 03.1 is run on it in its base
form; the moved crate (`skein-forge-connection`) must show the same test count and
no slower a time. Skein 03.1 owns those 112 tests (117 at the base: 112 after the
component's start tests moved in temper 01.3 and 01.4) and 3 new ones; none of them
stays in temper.

```text
$ cargo nextest list -p temper-engine-protocol-forge
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
117 tests listed; by module (derived from the listed names):
  connection 20
  contents 23
  hooks 12
  reads 27
  root 17
  total 117
  writes 18
```
The component's tests keep their names: they move with the crate (skein 03.1). Module split
above: root 17, connection 20, contents 23, hooks 12, reads 27, writes 18.

```text
$ cargo nextest run -p temper-engine-protocol-forge
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
    Starting 117 tests across 1 binary
     Summary [   0.099s] 117 tests run: 117 passed, 0 skipped
PASS [   0.002s] ( 10/117) tests::connection::a_json_upload_sends_its_body_and_content_headers
PASS [   0.007s] ( 16/117) tests::a_page_is_the_smallest_of_rows_cap_and_answer_room
PASS [   0.005s] ( 63/117) tests::host_names_include_only_nondefault_ports_and_bracket_ipv6
PASS [   0.005s] ( 66/117) tests::limits_that_disagree_with_the_client_name_both_limits
PASS [   0.002s] ( 93/117) tests::unusable_forge_settings_name_the_forge
PASS [   0.017s] (111/117) tests::writes::creations_carry_the_key_marker_at_the_head_of_their_body
```
Base: 117 tests, 0.099 s. The slowest of the six tests 03.1 names is 17 ms
(`creations_carry_the_key_marker_at_the_head_of_their_body`), under the 20 ms ceiling.

```text
$ cargo nextest run -p temper-service -p temper -p temper-forge-setup-shell -p temper-codecs-world -p temper-one-process-world -p temper-engine-protocol-forge-world -p forgejo-conformance
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Starting 435 tests across 34 binaries (10 binaries skipped via profile.default.default-filter)
Summary [   2.325s] 435 tests run: 435 passed, 0 skipped
per package (derived from the PASS lines of the full log, then the last binary id before '::'):
  forgejo-conformance 25
  temper 58
  temper-codecs-world 60
  temper-engine-protocol-forge-world 47
  temper-forge-setup-shell 24
  temper-one-process-world 9
  temper-service 212
```
The seven packages are the users of the crate (section "Users") and the world. 435 tests,
2.325 s, of which the crate's world is 47 and the conformance's own tests 25. 03.1 adds 5
tests to temper-service (section "Sketches"): 440 after it.

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --profile fuzzy
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
    Starting 6 tests across 3 binaries (5 binaries skipped via profile.fuzzy.default-filter)
        PASS [   0.539s] (1/6) temper-engine-protocol-forge-world::fuzzy_protocol stories_through_cut_streams_answer_each_call_once
        PASS [   0.872s] (2/6) temper-engine-protocol-forge-world::fuzzy_hooks a_delivery_split_at_every_point_is_answered_alike
        PASS [   5.510s] (3/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_3
        PASS [   5.951s] (4/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_1
        PASS [   6.055s] (5/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once
        PASS [   6.184s] (6/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_2
     Summary [   6.185s] 6 tests run: 6 passed, 0 skipped
```
The protocol world's fuzzy tests: 6 tests, 6.185 s (the budget class of about 5 s each is
the world's own; the base is kept, nothing is made cheaper).

```text
$ cargo nextest run -p forgejo-conformance
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
    Starting 25 tests across 2 binaries
        PASS [   0.003s] ( 1/25) forgejo-conformance tests::a_fresh_password_in_a_body_leaves_its_transcript_masked
        PASS [   0.004s] ( 2/25) forgejo-conformance tests::a_made_tokens_value_and_tail_leave_its_transcript_masked
        PASS [   0.004s] ( 3/25) forgejo-conformance tests::a_chunked_response_is_written_unframed_with_its_masked_length
        PASS [   0.004s] ( 4/25) forgejo-conformance tests::a_listed_runner_is_found_by_its_uuid
        PASS [   0.005s] ( 5/25) forgejo-conformance tests::a_fact_read_from_a_case_compares_its_status_and_message
        PASS [   0.005s] ( 6/25) forgejo-conformance tests::a_masked_delivery_keeps_a_content_length_that_matches_its_body
        PASS [   0.006s] ( 7/25) forgejo-conformance tests::a_fact_line_is_holds_fails_or_unchecked_with_its_reason
        PASS [   0.003s] ( 8/25) forgejo-conformance tests::a_raw_token_is_forty_lowercase_digits_and_a_line_feed
        PASS [   0.007s] ( 9/25) forgejo-conformance tests::a_password_is_read_between_its_first_and_last_quote
        PASS [   0.004s] (10/25) forgejo-conformance tests::a_truncated_git_packet_keeps_its_length_attached_to_the_hex_run
        PASS [   0.005s] (11/25) forgejo-conformance tests::a_secret_in_a_case_stops_the_run_naming_the_case
        PASS [   0.003s] (12/25) forgejo-conformance tests::masking_also_rewrites_the_listeners_port_and_delivery_ids
        PASS [   0.004s] (13/25) forgejo-conformance tests::git_packet_ids_share_the_plain_ids_first_appearance_order
        PASS [   0.006s] (14/25) forgejo-conformance tests::a_signature_holds_only_for_the_body_it_signs
        PASS [   0.003s] (15/25) forgejo-conformance tests::masking_rewrites_ports_ids_times_and_dates_by_first_appearance
        PASS [   0.005s] (16/25) forgejo-conformance tests::masking_sorts_the_deprecated_units_set_in_every_team
        PASS [   0.004s] (17/25) forgejo-conformance tests::targets_encode_every_byte_but_unreserved_ones
        PASS [   0.003s] (18/25) forgejo-conformance tests::the_pull_pair_records_any_checking_probe_that_settles
        PASS [   0.010s] (19/25) forgejo-conformance tests::a_sent_request_is_counted_by_its_request_line
        PASS [   0.004s] (20/25) forgejo-conformance tests::the_clone_workflow_runs_steps_only_and_names_the_jobs_token
        PASS [   0.011s] (21/25) forgejo-conformance tests::a_provisioning_failure_keeps_at_most_two_hundred_body_bytes
        PASS [   0.005s] (22/25) forgejo-conformance tests::the_component_block_is_replaced_and_other_lines_kept
        PASS [   0.002s] (23/25) forgejo-conformance tests::the_roll_up_ignores_the_losers_of_the_pairs
        PASS [   0.002s] (24/25) forgejo-conformance tests::the_runner_configuration_names_one_connection_by_its_token_file
        PASS [   0.107s] (25/25) forgejo-conformance tests::the_listener_holds_its_answer_until_the_probe_replies
... (2 more lines cut)
```
The conformance's own tests: 25 tests, 0.118 s (the base form of the unit tests that 03.2 to
03.4 keep green; the json module adds 8 of its own, 4 / 1 / 3 over 03.2 / 03.3 / 03.4).

```text
$ cargo nextest run -p temper-forge-setup-reconcile-world -p temper-system-world
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
    Starting 37 tests across 5 binaries (2 binaries skipped via profile.default.default-filter)
        PASS [   0.007s] ( 1/37) temper-forge-setup-reconcile-world::reconcile a_person_under_the_bots_login_is_refused
        PASS [   0.008s] ( 2/37) temper-forge-setup-reconcile-world::reconcile a_managed_forge_missing_an_org_is_refused_naming_it
        PASS [   0.013s] ( 3/37) temper-forge-setup-reconcile-world::reconcile a_listing_longer_than_a_page_is_read_to_its_end
        PASS [   0.012s] ( 4/37) temper-forge-setup-reconcile-world::reconcile a_team_named_for_the_bot_is_made_with_write_on_every_repository
        PASS [   0.025s] ( 5/37) temper-forge-setup-reconcile-world::reconcile a_deleted_hook_is_made_again
        PASS [   0.012s] ( 6/37) temper-forge-setup-reconcile-world::reconcile a_token_file_holding_a_secret_the_forge_never_made_is_made_again
        PASS [   0.028s] ( 7/37) temper-forge-setup-reconcile-world::reconcile a_revoked_token_is_made_again
        PASS [   0.030s] ( 8/37) temper-forge-setup-reconcile-world::reconcile a_deleted_bot_is_made_again
        PASS [   0.028s] ( 9/37) temper-forge-setup-reconcile-world::reconcile a_team_edited_by_a_person_is_repaired
        PASS [   0.017s] (10/37) temper-forge-setup-reconcile-world::reconcile a_token_file_holding_another_users_token_is_made_again
        PASS [   0.014s] (11/37) temper-forge-setup-reconcile-world::reconcile an_empty_forge_is_given_its_bot_and_both_tokens
        PASS [   0.004s] (12/37) temper-forge-setup-reconcile-world::reconcile no_secret_is_in_any_failure_debug_or_trace
        PASS [   0.015s] (13/37) temper-forge-setup-reconcile-world::reconcile a_token_file_that_answers_as_the_bot_is_kept_untouched
        PASS [   0.042s] (14/37) temper-forge-setup-reconcile-world::reconcile a_removed_team_member_is_added_again
        PASS [   0.045s] (15/37) temper-forge-setup-reconcile-world::reconcile a_second_run_writes_only_the_hooks_patches
        PASS [   0.017s] (16/37) temper-forge-setup-reconcile-world::reconcile an_owned_forge_makes_a_missing_org_for_its_person
        PASS [   0.024s] (17/37) temper-forge-setup-reconcile-world::reconcile an_org_hook_is_made_with_the_secret_the_events_json_and_active
        PASS [   0.015s] (18/37) temper-forge-setup-reconcile-world::reconcile the_webhook_secret_is_drawn_when_missing
        PASS [   0.022s] (19/37) temper-forge-setup-reconcile-world::reconcile the_tokens_hold_the_scopes_of_setup_section_4
        PASS [   0.020s] (20/37) temper-forge-setup-reconcile-world::reconcile two_hooks_with_the_listeners_url_keep_the_lowest_patched_and_the_rest_deleted
        PASS [   0.036s] (21/37) temper-forge-setup-reconcile-world::reconcile the_webhook_secret_is_kept_when_present
        PASS [   0.035s] (22/37) temper-system-world::composed a_committed_chat_runs_through_the_worker_and_smith_and_returns_a_report
        PASS [   0.035s] (23/37) temper-system-world::composed a_persons_stop_crosses_the_worker_and_cancels_smith_before_the_task_answers
        PASS [   0.038s] (24/37) temper-system-world::composed a_withheld_store_commit_holds_the_party_reply_and_worker_assignment
        PASS [   0.049s] (25/37) temper-system-world::composed a_parked_smith_run_resumes_from_the_prefix_restored_through_both_roots
... (14 more lines cut)
```
The two worlds that name the protocol world or the settings seam but need no changed line:
37 tests, 0.248 s (they must show 37 passed after 03.1).


## Users of what 03.1 changes or moves (counts at the base)

Every command ran from the directory shown. The commands name the
crate by its path (`temper_engine_protocol_forge`) because that is what a user writes.
The raw lists are cut by the 28-line limit; a count marked "derived" is made from the
raw list by `cut`, `sort` and `uniq -c`.

```text
$ rg -c 'temper_engine_protocol_forge\b' --glob '*.rs'.
directory: /srv/data/git/runner/src/tmp/planning/temper-base
./tests/engine/protocol-forge/tests/machine.rs:1
./tests/engine/protocol-forge/tests/hooks.rs:1
./tests/engine/protocol-forge/src/protocol.rs:2
./tools/forgejo-conformance/src/component/hosted.rs:1
./tools/forgejo-conformance/src/component.rs:3
./tests/engine/protocol-forge/src/machine.rs:1
./tests/engine/protocol-forge/src/joint.rs:4
./tests/codecs/tests/fuzzy_forgejo.rs:1
./tests/engine/protocol-forge/src/hooks.rs:1
./tests/engine/protocol-forge/src/fixtures.rs:3
./tests/codecs/tests/forgejo_deliveries.rs:1
./crates/temper/src/forge.rs:1
./crates/temper-forge-setup-shell/tests/support/mod.rs:2
./crates/temper/src/lib.rs:1
./tests/codecs/src/forgejo.rs:1
./crates/temper/src/host.rs:1
./crates/temper-service/src/stop.rs:2
./crates/temper-service/src/settings.rs:1
./crates/temper-service/src/route.rs:1
./tests/one-process/src/fixtures.rs:1
./crates/temper-service/src/limits.rs:1
./crates/temper-service/src/lib.rs:1
./crates/temper-service/src/forge.rs:1
./crates/temper-service/src/engine.rs:3
./crates/temper-service/src/system.rs:5
./crates/temper-service/src/tests/fixtures/forge.rs:1
./crates/temper-service/src/tests/forge.rs:1
./crates/temper-service/src/tests/fixtures.rs:1
```
Derived: outside the crate and its world 31 lines in 21 files (temper-service 18 lines in 11
files, crates/temper 3 in 3, tests/codecs 3 in 3, forge-setup-shell 2 in 1, the conformance 4
in 2, tests/one-process 1 in 1); the world `tests/engine/protocol-forge` 13 lines in 7 files.

```text
$ rg -n 'temper-engine-protocol-forge' --glob '*.toml' --glob '!Cargo.lock'.
directory: /srv/data/git/runner/src/tmp/planning/temper-base
./crates/temper/Cargo.toml:23:temper-engine-protocol-forge.workspace = true
./tests/codecs/Cargo.toml:12:temper-engine-protocol-forge.workspace = true
./tests/codecs/Cargo.toml:19:temper-engine-protocol-forge.workspace = true
./tests/one-process/Cargo.toml:17:temper-engine-protocol-forge.workspace = true
./tests/one-process/Cargo.toml:57:temper-engine-protocol-forge-world = { path = "../engine/protocol-forge" }
./crates/temper-service/Cargo.toml:10:temper-engine-protocol-forge.workspace = true
./tests/forge-setup/reconcile/Cargo.toml:16:temper-engine-protocol-forge-world = { path = "../../engine/protocol-forge" }
./tests/engine/protocol-forge/Cargo.toml:2:name = "temper-engine-protocol-forge-world"
./tests/engine/protocol-forge/Cargo.toml:18:temper-engine-protocol-forge.workspace = true
./crates/temper-forge-setup-shell/Cargo.toml:20:temper-engine-protocol-forge.workspace = true
./Cargo.toml:35:    "crates/temper-engine-protocol-forge",
./Cargo.toml:89:temper-engine-protocol-forge = { path = "crates/temper-engine-protocol-forge" }
./tools/forgejo-conformance/Cargo.toml:11:temper-engine-protocol-forge.workspace = true
./crates/temper-engine-protocol-forge/Cargo.toml:2:name = "temper-engine-protocol-forge"
```
```text
$ rg -n 'temper-engine-protocol-forge' Cargo.toml   (then:  rg -n 'tests/engine/protocol-forge|protocol-forge' Cargo.toml)
directory: /srv/data/git/runner/src/tmp/planning/temper-base
35:    "crates/temper-engine-protocol-forge",
89:temper-engine-protocol-forge = { path = "crates/temper-engine-protocol-forge" }
--
13:    "tests/engine/protocol-forge",
35:    "crates/temper-engine-protocol-forge",
89:temper-engine-protocol-forge = { path = "crates/temper-engine-protocol-forge" }
```
Manifests that change in 03.1: 6 packages (temper-service line 10, crates/temper line 23, the
shell line 20, the conformance line 11, tests/codecs lines 12 and 19, tests/one-process line 17)
plus the world (line 18) and the root (member line 35, path line 89). Cargo.lock names the
crate on 8 lines (g-f03, derived: `Cargo.lock:8`); it changes only by `cargo update`.

```text
$ rg -l 'temper-engine-protocol-forge|temper_engine_protocol_forge' --glob '!*.rs' --glob '!*.toml' --glob '!Cargo.lock' --glob '!docs/**'.
directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit exit 1 (no file other than .rs, .toml, Cargo.lock and docs names the crate)
```

The other repositories (counts per file; exit 1 is no match):

```text
$ rg -c 'temper_engine_protocol_forge|temper-engine-protocol-forge'.   (in skein-base, jig, smith, hearth)
directory: /srv/data/git/runner/src/tmp/planning/skein-base, /srv/data/git/runner/src/{jig,smith,hearth}
skein-base:   ./docs/design/STATUS.md:1
jig:          exit exit 1 (no match)
smith:        exit exit 1 (no match)
hearth:       ./docs/development/plans/render/findings/skein.md:1
```
skein-base names the crate once in docs/design/STATUS.md and hearth once in a plan finding;
neither is code. No change in jig, smith or hearth.

```text
$ rg -n '\b(stored_bytes|effect_bytes|answer_bytes_unbounded|bind_deployment)\b' --glob '*.rs'.
directory: /srv/data/git/runner/src/tmp/planning/temper-base
./crates/temper-engine-domain-forge-client/src/domain.rs:52:    pub fn bind_deployment(&mut self, id: [u8; 16], limits: &Limits) -> bool {
./crates/temper-engine-domain-forge-client/src/lib.rs:35:pub fn stored_bytes(row: &Stored) -> Option<u64> {
./crates/temper-engine-domain-forge-client/src/lib.rs:36:    bounds::stored_bytes(row)
./crates/temper-engine-domain-forge-client/src/lib.rs:40:pub fn effect_bytes(effect: &Effect) -> Option<u64> {
./crates/temper-engine-domain-forge-client/src/lib.rs:41:    bounds::effect_bytes(effect)
./crates/temper-engine-domain-forge-client/src/lib.rs:50:pub fn answer_bytes_unbounded(answer: &api::Answer) -> Option<u64> {
./crates/temper-engine-domain-forge-client/src/lib.rs:51:    bounds::answer_bytes_unbounded(answer)
./crates/temper-engine-domain-forge-client/src/bounds.rs:205:pub(crate) fn answer_bytes_unbounded(value: &Answer) -> Option<u64> {
./crates/temper-engine-domain-forge-client/src/bounds.rs:214:pub(crate) fn effect_bytes(effect: &crate::Effect) -> Option<u64> {
./crates/temper-engine-domain-forge-client/src/bounds.rs:235:pub(crate) fn stored_bytes(row: &crate::Stored) -> Option<u64> {
./crates/temper-engine-domain-forge/src/lib.rs:150:    match client::effect_bytes(&effect) {
./crates/temper-engine-domain-forge/src/lib.rs:214:pub fn stored_bytes(record: &Stored) -> Option<u64> {
./crates/temper-engine-domain-forge/src/lib.rs:277:                Ok(answer) => client::answer_bytes_unbounded(answer),
./crates/temper-engine-domain-forge/src/lib.rs:370:            client::effect_bytes(&row.entry.effect)?.checked_add(resource)
./crates/temper-engine-domain-forge/src/lib.rs:372:        Stored::Entry { entry, .. } => client::effect_bytes(&entry.effect),
./crates/temper-engine-domain-forge/src/lib.rs:373:        Stored::Client(row) => client::stored_bytes(row),
./crates/temper-engine-domain-forge/src/domain.rs:263:    pub fn bind_deployment(&mut self, id: [u8; 16], limits: &Limits) -> bool {
./crates/temper-engine-domain-forge/src/domain.rs:264:        if !self.client.bind_deployment(id, &limits.client) {
./crates/temper-engine-domain/src/restart.rs:397:            if !domain.forge.bind_deployment(deployment.id, &env.limits.forge) {
./crates/temper-engine-domain/src/store.rs:506:        Record::Core(jig_domain_core::Record::Tasks(row)) => jig_domain_tasks::stored_bytes(row),
./crates/temper-engine-domain/src/store.rs:513:        Record::Forge { row, .. } => temper_engine_domain_forge::stored_bytes(row),
./crates/temper-engine-domain/src/store.rs:690:            Ok(answer) => temper_engine_domain_forge_client::answer_bytes_unbounded(answer),
./crates/temper-engine-protocol-forge/src/tests/reads.rs:489:            temper_engine_domain_forge_client::answer_bytes_unbounded(&answer),
./crates/temper-engine-protocol-forge/src/tests/contents.rs:259:            temper_engine_domain_forge_client::answer_bytes_unbounded(&answer),
```
Derived: the measure callers outside the crate and its client are 5 lines in
`temper-engine-domain-forge/src/lib.rs`, 3 in `temper-engine-domain/src/store.rs` and one
`bind_deployment` caller in `restart.rs` (it already tests the bool); two more are in the
crate's own tests (`tests/reads.rs`, `tests/contents.rs`), which leave with it. Temper 02.1
repairs the first three before 03.1 starts, so 03.1's repin repairs none of them.

```text
$ rg -n 'settings::Webhooks|hooks::Webhooks|pub use.*Webhooks' --glob '*.rs'.
directory: /srv/data/git/runner/src/tmp/planning/temper-base
./tests/engine/protocol-forge/src/fixtures.rs:223:    settings.webhooks = Some(temper_engine_protocol_forge::hooks::Webhooks {
./crates/temper-service/src/tests/fixtures/forge.rs:138:    settings.webhooks = Some(kit::hooks::Webhooks { listener: io::Addr::Ip(([127, 0, 0, 1], 7200).into()), secrets });
./crates/temper-engine-protocol-forge/src/tests/fixtures.rs:482:        Some(crate::hooks::Webhooks { listener: skein_io::Addr::Ip(([127, 0, 0, 1], 7200).into()), secrets });
./crates/temper-service/src/startup/application.rs:595:    pub webhooks: Option<settings::Webhooks>,
./crates/temper-service/src/startup/application.rs:701:        webhooks = Some(settings::Webhooks { listener: *listener, secrets });
./crates/temper-service/src/settings.rs:69:pub use temper_engine_protocol_forge::hooks::Webhooks;
./crates/temper-engine-protocol-forge/src/settings.rs:13:    pub webhooks: Option<crate::hooks::Webhooks>,
./crates/temper-service/src/forge.rs:24:    webhooks: Option<settings::Webhooks>,
```
```text
$ rg -n 'temper_engine_protocol_forge as|use temper_engine_protocol_forge' --glob '*.rs' --glob '!crates/temper-engine-protocol-forge/**' --glob '!tests/engine/protocol-forge/**'.
directory: /srv/data/git/runner/src/tmp/planning/temper-base
./crates/temper/src/lib.rs:279:    use temper_engine_protocol_forge::settings::Transport;
./crates/temper/src/host.rs:60:                    use temper_engine_protocol_forge::{StartRead, Stopped};
./tests/forge-setup/reconcile/src/exchange.rs:7:use temper_engine_protocol_forge_world::pipe::{Cut, Pipe};
./crates/temper/src/forge.rs:229:    use temper_engine_protocol_forge::{conventions, limits};
./crates/temper-service/src/settings.rs:69:pub use temper_engine_protocol_forge::hooks::Webhooks;
./crates/temper-service/src/tests/forge.rs:6:use temper_engine_protocol_forge::{self as kit, StartRead, Stopped};
./crates/temper-service/src/route.rs:4:use temper_engine_protocol_forge as forge;
./crates/temper-service/src/forge.rs:9:use temper_engine_protocol_forge as kit;
./crates/temper-service/src/tests/fixtures/forge.rs:3:use temper_engine_protocol_forge as kit;
./crates/temper-service/src/engine.rs:386:    use temper_engine_protocol_forge as kit;
./tools/forgejo-conformance/src/component/hosted.rs:13:use temper_engine_protocol_forge::{self as kit, Component, Event, Request, Stopped};
```
```text
$ rg -n 'kit::(Shared|Settings|Limits)\b|settings::(Forge|Settings|Transport)\b|Transport::(Plaintext|Tls)' --glob '*.rs' --glob '!crates/temper-engine-protocol-forge/**' --glob '!tests/engine/protocol-forge/**'.
directory: /srv/data/git/runner/src/tmp/planning/temper-base   (the second list: rg -c '\bkit::' <four files>)
derived per file (lines):
  16 crates/temper-service/src/forge.rs
  11 crates/temper-service/src/startup/application.rs
  7 tools/forgejo-conformance/src/component/hosted.rs
  7 crates/temper-service/src/clients.rs
  6 crates/temper/src/lib.rs
  5 crates/temper-service/src/tests/fixtures.rs
  3 crates/temper/src/sign_in.rs
  3 crates/temper-service/src/workers.rs
  2 tests/system/src/composed/link.rs
  2 crates/temper/tests/support/sign_in.rs
  2 crates/temper-service/src/translate.rs
  2 crates/temper-service/src/tests/clients_unwatch.rs
  2 crates/temper-service/src/startup/checks.rs
  1 tests/one-process/src/fixtures.rs
  1 crates/temper/tests/support/mod.rs
  1 crates/temper/src/setup.rs
  1 crates/temper/src/forge.rs
  1 crates/temper-service/src/tests/family.rs
  1 crates/temper-service/src/family.rs
  1 crates/temper-service/src/assignment.rs
  1 crates/temper-service/src/accounts.rs
e01 (\bkit:: per file):
  tools/forgejo-conformance/src/component/hosted.rs:12
  crates/temper-service/src/tests/forge.rs:18
  crates/temper-service/src/tests/fixtures/forge.rs:5
  crates/temper-service/src/forge.rs:38
```
The pattern over-matches: `clients.rs` and `workers.rs` alias jig's `jig_protocol_engine_clients` and
`jig_protocol_engine_workers` as `kit`, and `application.rs` and `tests/system/src/composed/link.rs` use
temper-service's own `settings::Transport` (`settings.rs`, line 31), which stays. The files that build the
component's settings are 7: `forge.rs`, `tests/forge.rs`, `tests/fixtures/forge.rs`, `tests/fixtures.rs`
(`forge_runtime`), `crates/temper/src/lib.rs` (`forges()`), the conformance `hosted.rs` and the one-process
`fixtures.rs` (derived by reading the paths of the list above).

```text
$ rg -n '\bkit\b' crates/temper-service/src/engine.rs   (then:  rg -n '\bforge::' crates/temper-service/src/route.rs)
directory: /srv/data/git/runner/src/tmp/planning/temper-base
386:    use temper_engine_protocol_forge as kit;
392:            kit::Event::Answered(answer) => {
394:                    engine.forge.down(kit::Request::Returned(answer));
400:                    engine.forge.down(kit::Request::Refused(
406:            kit::Event::Hint(hinted) => {
417:                    engine.forge.down(kit::Request::RefusedHint(kit::hooks::Hinted { connection, hint }));
421:            kit::Event::ListenerLost => stop::hooks(engine),
422:            kit::Event::Stopped { forge, why } => stop::forge(engine, forge, why),
423:            kit::Event::Closed => engine.forge.closed = true,
--
22:            engine.forge.down(forge::Request::Call { call, repository, op });
26:            engine.forge.down(forge::Request::Returned(
27:                crate::forge::back(event).expect("the routing hands the root only the component's answers"),
```
```text
$ rg -o --no-filename 'temper_engine_protocol_forge::[a-z_]+::[A-Za-z_]+' --glob '*.rs' --glob '!crates/temper-engine-protocol-forge/**' --glob '!tests/engine/protocol-forge/**'.
directory: /srv/data/git/runner/src/tmp/planning/temper-base
temper_engine_protocol_forge::settings::Transport
temper_engine_protocol_forge::limits::http
temper_engine_protocol_forge::limits::documents
temper_engine_protocol_forge::settings::Transport
temper_engine_protocol_forge::limits::worst_case
temper_engine_protocol_forge::hooks::Webhooks
temper_engine_protocol_forge::limits::documents
temper_engine_protocol_forge::limits::documents
temper_engine_protocol_forge::limits::documents
temper_engine_protocol_forge::settings::Transport
```
```text
$ rg -n 'limits::(http|documents|worst_case|hook_http|hook_documents|check|page_size)\b|\bMAX_OUT\b' --glob '*.rs' --glob '!crates/temper-engine-protocol-forge/**'.
directory: /srv/data/git/runner/src/tmp/planning/temper-base
limits:: lines per file, derived (the pattern also matches other crates' own limits::check and limits::http):
./tests/engine/protocol-forge/src/protocol.rs:289:        let mut output = Queue::with_capacity(forge::MAX_OUT);
./tests/engine/protocol-forge/src/joint.rs:121:            fake_out: Queue::with_capacity(fake::MAX_OUT),
./crates/temper/src/forge.rs:267:        http: limits::http(&profile.forge_component, false, &io),
./crates/temper/src/forge.rs:268:        documents: limits::documents(&profile.forge_component),
./tools/forgejo-conformance/src/component/hosted.rs:131:        let heap = kit::limits::worst_case(&limits, &shared, 1).ok_or("component heap")?
./crates/temper-service/src/worker_git.rs:45:        let io = slots.checked_mul(component::MAX_OUT.io).ok_or(problem(b"git", What::Bound))?;
./crates/temper-service/src/worker_git.rs:51:                kernel: Queue::with_capacity(component::MAX_OUT.kernel),
./crates/temper-service/src/worker_git.rs:96:        .checked_add(Queue::<io::Request>::worst_case(slots.checked_mul(component::MAX_OUT.io)?)?)?
./crates/temper-service/src/worker_git.rs:97:        .checked_add(Queue::<kernel::Submit>::worst_case(component::MAX_OUT.kernel)?)?
./crates/temper-service/src/system.rs:594:        .checked_add(temper_engine_protocol_forge::limits::worst_case(
./crates/temper-service/src/system.rs:600:            batch.checked_mul(temper_engine_protocol_forge::MAX_OUT.above)?,
./crates/temper-service/src/system.rs:603:            u64::from(batch.checked_mul(temper_engine_protocol_forge::MAX_OUT.above)?)
./crates/temper-service/src/system.rs:607:            batch.checked_mul(temper_engine_protocol_forge::MAX_OUT.below)?,
./crates/temper-service/src/forge.rs:163:            above: Queue::with_capacity(batch.checked_mul(kit::MAX_OUT.above).expect("forge above room")),
./crates/temper-service/src/forge.rs:164:            io: Queue::with_capacity(batch.checked_mul(kit::MAX_OUT.below).expect("forge io room")),
./crates/temper-service/src/forge.rs:170:        self.above.room() >= batch.checked_mul(kit::MAX_OUT.above).expect("forge batch above")
./crates/temper-service/src/forge.rs:171:            && self.io.room() >= batch.checked_mul(kit::MAX_OUT.below).expect("forge batch io")
./crates/temper-service/src/forge.rs:313:        kit::limits::check(&config.limits.forge_component, &shared(&config.limits, &config.jig.limits.io), &prepared)
./crates/temper-forge-setup-shell/tests/support/mod.rs:74:        http: temper_engine_protocol_forge::limits::http(&profile, false, &io),
./crates/temper-forge-setup-shell/tests/support/mod.rs:75:        documents: temper_engine_protocol_forge::limits::documents(&profile),
./crates/temper-forge-setup-shell/src/reconcile.rs:374:        needs: Queue::with_capacity(reconcile::MAX_OUT),
./crates/temper-forge-setup-shell/src/prepare.rs:44:    let mut out = Queue::with_capacity(prepare::MAX_OUT);
```
```text
$ rg -n ForgeRuntime --glob '*.rs'.
directory: /srv/data/git/runner/src/tmp/planning/temper-base
./crates/temper/src/lib.rs:278:fn forges(directory: &Path, document: &skein_json::Document) -> Result<List<service::ForgeRuntime>, String> {
./crates/temper/src/lib.rs:305:            .push(service::ForgeRuntime {
./crates/temper-service/src/system.rs:36:        forges: List<crate::ForgeRuntime>,
./crates/temper-service/src/system.rs:46:        forges: List<crate::ForgeRuntime>,
./tests/one-process/src/fixtures.rs:237:pub(crate) fn forges() -> List<temper_service::ForgeRuntime> {
./tests/one-process/src/fixtures.rs:240:        .push(temper_service::ForgeRuntime {
./crates/temper-service/src/lib.rs:6:pub use forge::ForgeRuntime;
./crates/temper-service/src/forge.rs:12:pub struct ForgeRuntime {
./crates/temper-service/src/forge.rs:23:    runtime: List<ForgeRuntime>,
./crates/temper-service/src/forge.rs:301:    runtime: List<ForgeRuntime>,
./crates/temper-service/src/forge.rs:321:pub(crate) fn roots(config: &settings::Engine, runtime: &List<ForgeRuntime>) -> Result<u64, Problems> {
./crates/temper-service/src/startup.rs:192:    forges: List<crate::ForgeRuntime>,
./crates/temper-service/src/tests/fixtures.rs:757:pub fn forge_runtime() -> skein_lib::List<crate::ForgeRuntime> {
./crates/temper-service/src/tests/fixtures.rs:760:        .push(crate::ForgeRuntime {
```
```text
$ rg -n 'forge::(settings|shared|small|prepare|problems|indexed)\(' --glob '*.rs' crates tests tools
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-service/src/limits.rs:255:            forge_component: crate::forge::small(&FORGE),
crates/temper-service/src/tests/forge.rs:96:        forge::problems(refusal, &mut found);
crates/temper-service/src/tests/forge.rs:150:    forge::problems(kit::Unusable::SecretBytes { bytes: 16 }, &mut found);
crates/temper-service/src/tests/forge.rs:159:    forge::problems(
crates/temper-service/src/tests/fixtures/forge.rs:135:        crate::forge::settings(&config.forges, super::forge_runtime(), None, &mut found).expect("settings");
crates/temper-service/src/tests/fixtures/forge.rs:141:        kit::Component::new(settings, &layer.env.limits, &crate::forge::shared(&config.limits, &config.jig.limits.io))
crates/temper-service/src/system.rs:53:        let (forge_settings, forge_roots) = crate::forge::prepare(&settings, forges)?;
crates/temper-service/src/system.rs:54:        let shared = crate::forge::shared(&settings.limits, &settings.jig.limits.io);
crates/temper-service/src/system.rs:596:            &crate::forge::shared(&settings.limits, &settings.jig.limits.io),
```
```text
$ rg -n 'host:|\.host\b|conventions::|api_token' --glob '*.rs' <five files>   (lines about jig_domain_host cut)
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/protocol-forge/src/fixtures.rs:100:            host: Box::from(&b"127.0.0.1:3000"[..]),
tests/engine/protocol-forge/src/fixtures.rs:234:        host: Box::from(b"127.0.0.1:7200".as_slice()),
crates/temper-service/src/forge.rs:56:        let Some(host) = kit::conventions::host(endpoint.address, name) else {
crates/temper-service/src/forge.rs:67:                token: forge.api_token.clone(),
crates/temper-service/src/forge.rs:253:            found.push(forge_path(u32::from(forge)).member(b"api_token"), What::Bound);
tools/forgejo-conformance/src/component/hosted.rs:125:                host: format!("127.0.0.1:{}", f.instance.port()).into_bytes().into(),
crates/temper-service/src/tests/fixtures.rs:95:            api_token: Secret::new(Box::from(&b"api-token"[..])),
crates/temper-service/src/tests/fixtures.rs:544:        host: b"forge.example".as_slice().into(),
tests/one-process/src/fixtures.rs:122:            "git":"http://127.0.0.1:3000","webhook_secret":null,"api_token":"secrets/forge.api-token","user":{"file":"secrets/forge-1.user"},"identity":
```
Of those, `host:` is read as the kit's `Forge.host` only in the protocol world's
`fixtures.rs` (`settings()`, line 100) and the conformance `hosted.rs` (line 125); the world's line 234 is the
fake forge's delivery endpoint and the temper-service `fixtures.rs:544` is a `ForgeRepository` of the engine:
both stay.

```text
$ rg -n 'temper/0\.0\.0|USER_AGENT' --glob '*.rs' --glob '*.md' --glob '*.txt' --glob '!docs/**' --glob '!transcripts/**' --glob '!deliveries/**'.
directory: /srv/data/git/runner/src/tmp/planning/temper-base   (then: rg -n 'the_start_request|User-Agent|user_agent' --glob '*.rs' crates/temper-service crates/temper tests/one-process tests/codecs tools)
./crates/temper-engine-protocol-forge/src/tests/connection.rs:272:    assert_eq!(h.sent.as_ref(), b"DELETE /api/v1/repos/fixture/specimen/pulls/7/requested_reviewers HTTP/1.1\r\nHost: 127.0.
./crates/temper/src/forge.rs:259:        user_agent: conventions::USER_AGENT,
./crates/temper-engine-protocol-forge/src/conventions.rs:6:pub const USER_AGENT: &str = "temper/0.0.0";
./crates/temper-engine-protocol-forge/src/connection.rs:175:        headers.push(field(b"User-Agent", crate::conventions::USER_AGENT.as_bytes())).expect("five header fields");
--
crates/temper/src/forge.rs:259:        user_agent: conventions::USER_AGENT,
```
The only pin of `temper/0.0.0` is inside the crate (`connection.rs:272`, a test that moves
to skein as `kit/0.0.0`); `crates/temper/src/forge.rs` is the one other use. Temper therefore
adds its own wire test (`the_start_request_carries_the_user_agent_and_the_host`, 03.1).

```text
$ rg -n '\bUnusable\b' --glob '*.rs' --glob '!crates/temper-engine-protocol-forge/**'.
directory: /srv/data/git/runner/src/tmp/planning/temper-base
derived per file:
  27 crates/temper-service/src/forge.rs
  7 crates/temper-worker-protocol-git/src/component.rs
  6 crates/temper-service/src/tests/forge.rs
  2 crates/temper-worker-protocol-git/src/tests/files.rs
  2 crates/temper-worker-protocol-git/src/tests/component.rs
  2 crates/temper-worker-protocol-git/src/boundary.rs
  1 crates/temper-worker-protocol-git/src/tests/fixtures.rs
  1 crates/temper-worker-protocol-git/src/lib.rs
  1 crates/temper-service/src/worker_git.rs
(the worker-git crate's `Unusable` is its own type)
```
The kit's `Unusable` is named in `temper-service/src/forge.rs` (27 lines: the `problems()` match,
`indexed()`, `prepare`) and its tests (6 lines); the worker-git crate's lines are another type.

```text
$ rg -n -i signature --glob '*.rs' tests/engine/protocol-forge crates/temper-service/src crates/temper/tests tests/one-process tests/codecs   (lines of the signature helpers cut)
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/protocol-forge/tests/hooks.rs:32:fn a_wrong_signature_is_answered_403_and_hints_nothing() {
tests/engine/protocol-forge/tests/hooks.rs:81:fn a_missing_signature_is_answered_403_without_reading_a_body() {
crates/temper/tests/process.rs:85:    assert_eq!(delivery.signature, support::webhooks::signature(&forgejo.secret(), &delivery.body));
tests/engine/protocol-forge/src/hooks.rs:228:    let signature =
crates/temper-service/src/tests/fixtures/forge.rs:151:    head.put(b"POST /hooks/1 HTTP/1.1\r\nHost: loopback\r\nContent-Length: 34\r\nX-Forgejo-Event: pull_request\r\nX-Forgejo-Signature: "
crates/temper-service/src/tests/fixtures/forge.rs:154:            head.put(&[b"0123456789abcdef"[usize::from(nibble)]]).expect("signature");
--
./testing/temper-fake-forge-protocol/src/tests/deliveries.rs:11:const PUSH_HEAD: &[u8] = b"POST /hooks/1 HTTP/1.1\r\nHost: 127.0.0.1:7200\r\
./testing/temper-fake-forge-protocol/src/tests/deliveries.rs:124:    assert_eq!(&*post.head, b"POST /hooks/1 HTTP/1.1\r\nHost: 127.0.0.1:720
./crates/temper/tests/support/webhooks.rs:72:            write!(stream, "X-Forgejo-Signature: {}\r\n", signature(SECRET, self.body)).expect(
./crates/temper/tests/support/webhooks.rs:191:                "x-forgejo-signature" => delivery.signature = value.trim().into(),
./tests/engine/protocol-forge/src/hooks.rs:230:    let mut head = format!("POST {} HTTP/1.1\r\nHost: 127.0.0.1:7200\r\nContent-Length: {}\r\
./crates/temper-service/src/tests/fixtures/forge.rs:151:    head.put(b"POST /hooks/1 HTTP/1.1\r\nHost: loopback\r\nContent-Length: 34\r\nX-F
./tools/forgejo-conformance/src/tests.rs:174:    let mut received = Received { head: b"X-Forgejo-Signature: 22e3b1f3fb90d9e7f5814c8af105f7f7
./tools/forgejo-conformance/src/deliveries.rs:136:    skein_lib::sha256::equal(received.header("X-Forgejo-Signature").as_bytes(), signature.
./tools/forgejo-conformance/src/deliveries.rs:138:            received.header("X-Hub-Signature-256").as_bytes(),
./crates/temper-forge-forgejo/src/delivery.rs:96:pub const SIGNATURE_HEADER: &[u8] = b"X-Forgejo-Signature";
```
Answer 3 (a malformed signature is 403 at the head): only the protocol world's two tests name a
signature outcome (`a_wrong_signature_is_answered_403_and_hints_nothing`, `a_missing_signature_is_answered_403_without_reading_a_body`);
both are 403 already. No temper test pins a malformed or uppercase signature, so the answer changes no temper line.

```text
$ rg -c 'temper_engine_protocol_forge|\bforge::|\bkit::' --glob '*.rs' tests/engine/protocol-forge
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/protocol-forge/tests/protocol.rs:1
tests/engine/protocol-forge/tests/machine.rs:2
tests/engine/protocol-forge/tests/hooks.rs:3
tests/engine/protocol-forge/tests/fuzzy_protocol.rs:1
tests/engine/protocol-forge/tests/fuzzy_machine.rs:1
tests/engine/protocol-forge/tests/fuzzy_hooks.rs:1
tests/engine/protocol-forge/tests/fake_protocol.rs:1
tests/engine/protocol-forge/src/protocol.rs:11
tests/engine/protocol-forge/src/machine.rs:1
tests/engine/protocol-forge/src/joint.rs:4
tests/engine/protocol-forge/src/hooks.rs:1
tests/engine/protocol-forge/src/fixtures.rs:3
```
The world `tests/engine/protocol-forge` stays until temper 07.1 and names the crate on 13 lines;
03.1 repoints it (47 tests at the base, section "The users run").

### serde_json (03.2 to 03.4)

```text
$ rg -c serde_json --glob '*.rs' --glob '*.toml' --glob '!Cargo.lock'.
directory: /srv/data/git/runner/src/tmp/planning/temper-base
derived: files in tools/forgejo-conformance:
  1 tools/forgejo-conformance/Cargo.toml
  2 tools/forgejo-conformance/src/cases.rs
  1 tools/forgejo-conformance/src/component/reads.rs
  1 tools/forgejo-conformance/src/component/writes.rs
  3 tools/forgejo-conformance/src/deliveries.rs
  3 tools/forgejo-conformance/src/facts.rs
  1 tools/forgejo-conformance/src/fixture.rs
  2 tools/forgejo-conformance/src/hooks.rs
  1 tools/forgejo-conformance/src/job_clone.rs
  2 tools/forgejo-conformance/src/lib.rs
  2 tools/forgejo-conformance/src/mask.rs
  2 tools/forgejo-conformance/src/orgs.rs
  3 tools/forgejo-conformance/src/registration.rs
  2 tools/forgejo-conformance/src/runner.rs
  1 tools/forgejo-conformance/src/setup.rs
  1 tools/forgejo-conformance/src/tests.rs
  1 tools/forgejo-conformance/src/tokens.rs
  1 tools/forgejo-conformance/src/users.rs
outside the conformance (derived): 29 files, 22 sources in 6 packages and 7 manifests (the root's too)
```
```text
$ rg -n 'skein-json|serde_json|skein-forge' tools/forgejo-conformance/Cargo.toml Cargo.toml
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Cargo.toml:119:serde_json = "=1.0.154"
Cargo.toml:121:skein-json = { git = "https://github.com/forgetic/skein.git" }
tools/forgejo-conformance/Cargo.toml:10:serde_json.workspace = true
```
serde_json stays a workspace dependency (line 119) because the other packages use it
(`crates/temper` tests, `tests/codecs`, `tests/one-process`, `tools/live-login`, `tools/git-transcripts`,
`testing/temper-throwaway-forgejo`: derived, 6 packages); only the conformance's manifest line leaves.

```text
$ rg -c 'json!\(' tools/forgejo-conformance/src
directory: /srv/data/git/runner/src/tmp/planning/temper-base
json! sites (derived total: 69 in 14 files):
tools/forgejo-conformance/src/users.rs:1
tools/forgejo-conformance/src/job_clone.rs:2
tools/forgejo-conformance/src/tokens.rs:6
tools/forgejo-conformance/src/hooks.rs:3
tools/forgejo-conformance/src/fixture.rs:9
tools/forgejo-conformance/src/setup.rs:1
tools/forgejo-conformance/src/runner.rs:1
tools/forgejo-conformance/src/facts.rs:14
tools/forgejo-conformance/src/component/writes.rs:1
tools/forgejo-conformance/src/registration.rs:1
tools/forgejo-conformance/src/deliveries.rs:23
tools/forgejo-conformance/src/component/reads.rs:4
tools/forgejo-conformance/src/orgs.rs:2
tools/forgejo-conformance/src/cases.rs:1
```
```text
$ rg -c '\.as_(str|u64|bool|array)\(|is_(string|boolean|null)\(|\["[a-z_.]+"\]|\[[0-9]\]' tools/forgejo-conformance/src
directory: /srv/data/git/runner/src/tmp/planning/temper-base
read sites (derived total: 347 in 18 files):
tools/forgejo-conformance/src/facts.rs:75
tools/forgejo-conformance/src/deliveries.rs:51
tools/forgejo-conformance/src/runner.rs:45
tools/forgejo-conformance/src/hooks.rs:28
tools/forgejo-conformance/src/component/writes.rs:25
tools/forgejo-conformance/src/orgs.rs:19
tools/forgejo-conformance/src/component/reads.rs:17
tools/forgejo-conformance/src/registration.rs:15
tools/forgejo-conformance/src/cases.rs:15
tools/forgejo-conformance/src/tokens.rs:14
tools/forgejo-conformance/src/users.rs:13
tools/forgejo-conformance/src/fixture.rs:11
```
```text
$ sh -c 'echo src=$(find crates/temper-engine-protocol-forge/src...) tests=... manifest=... files=...'   (non-test, test, manifest lines and file count)
directory: /srv/data/git/runner/src/tmp/planning/temper-base
src=6365 tests=3178 manifest=23 files=32
```
The crate that is deleted: 6365 non-test lines, 3178 test lines, a 23-line manifest, 32 files
(which skein 03.1 moves at the same sizes; 03.1 here removes them from temper).

```text
$ python3 -c 'from pathlib import Path; print("component_copy_absent=" + str(not Path("crates/temper-engine-protocol-forge").exists()))'
directory: /srv/data/git/runner/src/tmp/planning/temper-base   (base form of Done when: the crate exists, so False; after 03.1 it prints True)
component_copy_absent=False
```
```text
$ rg -n '16_384|21\b' --glob '*.rs' crates/temper-service/src/forge.rs crates/temper-service/src/limits.rs   (and: rg -n 'b"temper"|word')
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-service/src/limits.rs:200:        run_bytes: 16_384,
crates/temper-service/src/forge.rs:121:        head_bytes: 16_384,
crates/temper-service/src/forge.rs:127:        body_bytes: 16_384.max(forge.client.op_bytes.saturating_mul(2).saturating_add(21)),
crates/temper-service/src/forge.rs:141:        tls: skein_tls::client::Limits { read: 16_384, send: 16_384, records: skein_tls::client::MAX_RECORD },
crates/temper-service/src/limits.rs:197:        // At least jig small people.words: the root's worst case.
```
The small profile's +21 is `small()` in `forge.rs`: `16_384.max(op_bytes*2 + 21)`. The word-derived
size is `skein_forge::marker::length(b"temper", op_bytes)` = `2*op_bytes + 6 + 15`: the same 21 for
`temper`, so the number does not move for temper (the Test shape pins 100 -> 16_384 and 10_000 -> 20_021).


## The gate

```text
$ scripts/check.sh
directory: /srv/data/git/runner/src/tmp/planning/temper-base (committed tree)

     Summary [   6.668s] 1879 tests run: 1879 passed, 1 skipped
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
    Starting 50 tests across 26 binaries (122 binaries skipped via profile.fuzzy.default-filter)
        SLOW [> 20.000s] (─────) temper-engine-domain-world::fuzzy_walking fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure
     Summary [  23.234s] 50 tests run: 50 passed (1 slow), 0 skipped
```
Gate at the base: 1879 tests passed (1 skipped) in 6.668 s, then the fuzzy profile 50 in
23.234 s with one slow test (`temper-engine-domain-world` `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure`,
over 20 s), 34 s alone, exit 0. The gate checks the budgets (15 s and 60 s); it printed no budget line, so
the Proof says: exit 0, the two Summary lines, and the counts after the step. 03.1 drops the crate's 117
tests and adds 5 (temper-service) and the other increments add the sketched tests; the siblings'
gate counts (1791 after 01.4, 1808 after 02.1) do not chain from the base 1879 (the sibling steps
also move tests), so the Proof states the delta, not a total.

## The conformance recordings and the stable-fact check (03.2 to 03.4 prove with them)

Both modes were recorded from the base into `$S/target` (the gate lane takes a committed tree only,
so the recordings ran from `temper-base` with `--scratch` and `--out` pointing into the scratch.
The throwaway Forgejo and runner are the binaries of README Rules; TEMPER_FORGEJO and
TEMPER_FORGEJO_RUNNER were exported). Neither temper nor the plan has a conformance proof script
at the base (`scripts/` holds check.sh, proof-live.sh and proof-one-process.sh); skein 06.4's
`proof-forgejo-conformance.sh` is skein's.

```text
$ cargo run -p forgejo-conformance -- --scratch $S/target/conformance-instance --out $S/target/conformance-out
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running `target/debug/forgejo-conformance --scratch /srv/data/git/runner/src/tmp/planning/scratch-temper-03/target/conformance-instance --out /srv/data/git/runner/src/tmp
rc=0 wall=336s
provisioning lines: 8 (HTTP 201 each, cut)
```
```text
$ cargo run -p forgejo-conformance -- --component --scratch $S/target/conformance-component --out $S/target/conformance-out
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running `target/debug/forgejo-conformance --component --scratch /srv/data/git/runner/src/tmp/planning/scratch-temper-03/target/conformance-component --out /srv/data/git/r
rc=0 wall=41s
provisioning lines: 6 (HTTP 201 each, cut)
```
Recording: 100 s (alone 98 s); the component mode: 30 s (alone 28 s). The README's 106 s + 22 s are the
base-skein-06 numbers; both modes are well under 5 min.

```text
$ bash -o pipefail -c '<the stable-fact line of skein/06-conformance.md, Proof, line "bash -o pipefail -c 'set -eu; digest=...'">' sh $S/target/conformance-out/facts.txt
directory: /srv/data/git/runner/src/tmp/planning/temper-base   (the full line is /tmp/t03/stable-cmd.txt; it is written in full once, in the session file's Done when)
timing fails merge-old-head-is-409-head-out-of-date: HTTP 405: "Please try again later"
timing holds delivery-pull-reads-not-mergeable-while-checked
timing fails delivery-pull-is-settled-when-its-push-is-delivered: job statuses=[true, true], pull mergeability=[[false, true], [false, true]]
timing holds delivery-pull-is-checking-when-its-push-is-delivered
timing holds webhook-deliveries
facts=218 stable=213 holds=207 fails=6 timing=5 unchecked=0
d1f7ba2bb323b8944e05d2e05bc1c83396fb7bfffadbcefea4faad64ec18347d  -
```
```text
$ bash -o pipefail -c '<the same line>' sh tools/forgejo-conformance/facts.txt
directory: /srv/data/git/runner/src/tmp/planning/temper-base   (the committed corpus)
timing holds merge-old-head-is-409-head-out-of-date
timing holds delivery-pull-reads-not-mergeable-while-checked
timing fails delivery-pull-is-settled-when-its-push-is-delivered: job statuses=[true, true], pull mergeability=[[false, true], [false, true]]
timing holds delivery-pull-is-checking-when-its-push-is-delivered
timing holds webhook-deliveries
facts=218 stable=213 holds=207 fails=6 timing=5 unchecked=0
d1f7ba2bb323b8944e05d2e05bc1c83396fb7bfffadbcefea4faad64ec18347d  -
```
Both print the digest d1f7ba2b…18347d and the counts of Done when 7. The timing outcomes differ between
the fresh and the committed run in one fact: `merge-old-head-is-409-head-out-of-date` failed on the fresh
run (HTTP 405 "Please try again later") and holds in the committed corpus; it is a timing fact, so the
check passes either way. A step's Proof therefore reads the counts and the digest, not the timing lines.

```text
$ sh -c 'cd tools/forgejo-conformance && diff -rq --exclude=response transcripts "$1"/transcripts; echo "diff exit $?"' sh $S/target/conformance-out
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Files transcripts/merge-conflict/request and /srv/data/git/runner/src/tmp/planning/scratch-temper-03/target/conformance-out/transcripts/merge-conflict/request differ
Files transcripts/merge-stale-head/request and /srv/data/git/runner/src/tmp/planning/scratch-temper-03/target/conformance-out/transcripts/merge-stale-head/request differ
diff exit 1
```
At the base a fresh recording's requests differ from the committed transcripts in exactly two files,
`merge-conflict` and `merge-stale-head`: the masked head id placeholder (`…0e` committed against `…0a`, and
`…0d` against `…0e`). A second fresh recording (f1, 101 s) differed from the first in the same two files
(`…0a` against `…0e`, and `…0e` against `…0d`), so the two files vary from run to run at the base:
the Proof expects exactly these two names and no other.

```text
$ sh c5.sh   (diff -r --exclude=response transcripts, then deliveries, against the fresh recording; cut)
directory: /srv/data/git/runner/src/tmp/planning/temper-base/tools/forgejo-conformance
8c8
< {"Do":"merge","head_commit_id":"000000000000000000000000000000000000000e"}
> {"Do":"merge","head_commit_id":"000000000000000000000000000000000000000a"}
8c8
< {"Do":"merge","head_commit_id":"000000000000000000000000000000000000000d"}
> {"Do":"merge","head_commit_id":"000000000000000000000000000000000000000e"}
transcripts: diff exit 1
23c23
< Content-Length: 3871
> Content-Length: 3875
72c72
<     "ssh_url": "ssh://free@localhost/hooked/specimen.git",
>     "ssh_url": "ssh://runner@localhost/hooked/specimen.git",
101c101
<     "wiki_ssh_url": "ssh://free@localhost/hooked/specimen.wiki.git",
>     "wiki_ssh_url": "ssh://runner@localhost/hooked/specimen.wiki.git",
23c23
< Content-Length: 3864
> Content-Length: 3868
72c72
<     "ssh_url": "ssh://free@localhost/hooked/specimen.git",
>     "ssh_url": "ssh://runner@localhost/hooked/specimen.git",
101c101
<     "wiki_ssh_url": "ssh://free@localhost/hooked/specimen.wiki.git",
>     "wiki_ssh_url": "ssh://runner@localhost/hooked/specimen.wiki.git",
23c23
< Content-Length: 3815
> Content-Length: 3819
```
The deliveries differ on every recording in the forge's own noise only: the ssh user (`free` against
`runner`, a machine fact), ids, `Content-Length` following them, and `updated_at` seconds. The Proof therefore
diffs `transcripts` only and does not diff `deliveries`; the stable-fact check covers the deliveries.


## The process tier at the base (it failed twice, differently; reported, not a pass)

`TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64` and
`TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64`
(`$WS` is `/srv/data/git/runner/src`) were exported for the three runs below. The README's `$HOME/src/rust/...` forgejo
paths do not exist on this machine.

```text
$ scripts/proof-one-process.sh   (TEMPER_FORGEJO and TEMPER_FORGEJO_RUNNER set)
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Starting 19 tests across 1 binary (6 binaries skipped via profile.process.default-filter)
        PASS [  84.971s] ( 1/19) temper::process a_change_lands_by_deliveries_with_every_poll_raised
        PASS [   4.067s] ( 2/19) temper::process a_native_chat_reaches_the_model
        FAIL [   4.554s] ( 3/19) temper::process a_persons_change_lands_on_the_forge_and_is_found_after_a_restart
    test a_persons_change_lands_on_the_forge_and_is_found_after_a_restart ... FAILED
    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 18 filtered out; finished in 4.55s
    thread 'a_persons_change_lands_on_the_forge_and_is_found_after_a_restart' (3269679) panicked at <cargo-checkouts>/skein-336d5ae5eecfa0f9/8a51e10/testing/skein-world/src/li
    binary exited while tree processes still ran: [3270296]
     Summary [  93.593s] 3/19 tests run: 2 passed (2 slow), 1 failed, 0 skipped
        FAIL [   4.554s] ( 3/19) temper::process a_persons_change_lands_on_the_forge_and_is_found_after_a_restart
```
```text
$ cargo nextest run -p temper --profile process --no-fail-fast   (the profile the script runs, all 19 tests)
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Starting 19 tests across 1 binary (6 binaries skipped via profile.process.default-filter)
        PASS [  83.213s] ( 1/19) temper::process a_change_lands_by_deliveries_with_every_poll_raised
        PASS [  22.919s] ( 3/19) temper::process a_persons_change_lands_on_the_forge_and_is_found_after_a_restart
        FAIL [  30.150s] ( 6/19) temper::process a_running_engine_lends_the_record_temper_sign_in_kept
    test a_running_engine_lends_the_record_temper_sign_in_kept ... FAILED
    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 18 filtered out; finished in 30.14s
    thread 'a_running_engine_lends_the_record_temper_sign_in_kept' (3302309) panicked at crates/temper/tests/support/sign_in.rs:277:9:
    sign-in world deadline; stopped: false; empty hosts: [true, true, false, true, false, false, false]; issuer deadline: None; stderr: 
     Summary [ 174.042s] 19 tests run: 18 passed (8 slow), 1 failed, 0 skipped
```
```text
$ cargo nextest run -p temper --profile process a_running_engine_lends_the_record_temper_sign_in_kept   (the e1 failure, rerun alone)
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
    Starting 1 test across 1 binary (18 tests and 6 binaries skipped, including 6 binaries via profile.process.default-filter)
        PASS [   0.324s] (1/1) temper::process a_running_engine_lends_the_record_temper_sign_in_kept
     Summary [   0.324s] 1 test run: 1 passed, 18 skipped
```
Read: the script run failed `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` (the world's
"binary exited while tree processes still ran") after 93.6 s of 3 of 19 tests; the `--no-fail-fast` run passed that
test (22.9 s) and failed `a_running_engine_lends_the_record_temper_sign_in_kept` (the sign-in world deadline,
30.15 s; 18 passed, 174 s), which then passed alone in 0.324 s. Two different tests, each passing on a rerun.
That is a base fact about the tier (README Done when 6 expects exit 0): the step's Proof runs the script once,
reruns a failure once alone, and reports a failure that persists; it never counts a failure as a pass.
The base-runs.md figure is 19 tests in 145.694 s; the three runs here took 95 s alone (3 tests), 175 s and 1 s.

## Sketches compiled in the scratch

skein 03.1 is not on origin/main, so the kit's `Word`, `PlaintextName`, flat `Shared` and `Transport::Plaintext{name}`
do not exist at the base. The scratch (`$S`, detached at dd12b8ed9) stands in with the two new variants on the
crate's `Unusable` (`limits.rs`: `Word`, `PlaintextName { forge: u16 }`) and compiles the temper-service side that does
not depend on the rest: the two `problems()` arms, the three-variant `indexed()`, a `sent()` accessor on the
fixture face and the five tests. The tests that need the flat `Shared` (after the step) are compiled in their base form
(`shared.client == client`) and say so. The conformance `json` module (242 lines of production, 57 of tests) is
compiled against skein-json at 8a51e10, the pin of the base.

```text
$ cargo nextest run -p temper-service the_component_s_new_refusals a_refusal_naming_a_forge the_start_request_carries
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-03
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.77s
    Starting 3 tests across 1 binary (212 tests skipped)
        PASS [   0.005s] (1/3) temper-service tests::forge::the_component_s_new_refusals_are_problems_naming_their_members
        PASS [   0.006s] (2/3) temper-service tests::forge::a_refusal_naming_a_forge_is_indexed_to_the_configured_forge
        PASS [   0.012s] (3/3) temper-service tests::forge::the_start_request_carries_the_user_agent_and_the_host
     Summary [   0.012s] 3 tests run: 3 passed, 212 skipped
```
```text
$ cargo nextest run -p temper-service the_small_profile_keeps_the_body_room the_shared_numbers_are_the_clients
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-03   (the second test in its base form)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.23s
    Starting 2 tests across 1 binary (215 tests skipped)
        PASS [   0.006s] (1/2) temper-service tests::forge::the_shared_numbers_are_the_clients_plain_numbers
        PASS [   0.008s] (2/2) temper-service tests::forge::the_small_profile_keeps_the_body_room_the_word_derives
     Summary [   0.008s] 2 tests run: 2 passed, 215 skipped
```
```text
$ cargo clippy -p temper-service --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-03
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.13s
```
```text
$ cargo nextest run -p forgejo-conformance json::tests
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-03
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.48s
    Starting 6 tests across 2 binaries (25 tests skipped)
        PASS [   0.002s] (1/6) forgejo-conformance json::tests::a_body_that_is_not_one_document_is_refused
        PASS [   0.002s] (2/6) forgejo-conformance json::tests::a_string_is_escaped_as_serde_json_wrote_it
        PASS [   0.002s] (3/6) forgejo-conformance json::tests::members_out_of_order_or_repeated_are_refused
        PASS [   0.002s] (4/6) forgejo-conformance json::tests::a_missing_member_or_the_wrong_kind_reads_as_none
        PASS [   0.002s] (5/6) forgejo-conformance json::tests::an_object_is_written_compact_in_sorted_key_order
        PASS [   0.002s] (6/6) forgejo-conformance json::tests::a_parsed_body_reads_members_items_and_scalars_by_name
     Summary [   0.003s] 6 tests run: 6 passed, 25 skipped
```
```text
$ cargo clippy -p forgejo-conformance --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-03   (json.rs whole, with json/tests.rs; the reading half alone: clippy exit 0, 1.72 s)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.76s
```
The scratch ran h1 to h3 earlier (an earlier sketch of the same modules, since replaced by h4 to h9; their
outputs are not used above). Tests in the scratch: 3 + 2 = 5 new temper-service tests (212 at the base -> 217).
The temper-service test count after 03.1 is 217; the crate's 117 are skein's.

```text
$ cargo nextest run -p temper-service tests::forge   (base: first two lines; the same in the scratch with the five new tests: last two)
directory: /srv/data/git/runner/src/tmp/planning/temper-base; /srv/data/git/runner/src/tmp/planning/scratch-temper-03
    Starting 10 tests across 1 binary (202 tests skipped)
     Summary [   0.028s] 10 tests run: 10 passed, 202 skipped
    Starting 15 tests across 1 binary (202 tests skipped)
     Summary [   0.026s] 15 tests run: 15 passed, 202 skipped
```
The `tests::forge` tests of temper-service: 10 at the base (0.028 s), 15 in the scratch (0.026 s).


### The conformance's three increments: why by direction, and the leaves compiled

The 17 source files reach each other only through `Fixture::{api, get, branch, pull, put}`
and `api_answer` (every one returns `serde_json::Value` and `api` takes `&Value`), `lib.rs::json`,
`facts.rs::Case.body`, `registration::listed`, `runner.rs::rows` and the `deliveries.rs` helpers.
Callers of the `api` family at the base: 146 lines in 14 files (the counts below). A split by module
therefore does not build: converting `fixture.rs` forces every caller in the same commit. The split is
by direction, after a first increment for what does not touch the family:

```text
$ rg -c '\.(api|get|branch|pull|put)\(' tools/forgejo-conformance/src   (derived per file, summed: 146 lines, 14 files)
directory: /srv/data/git/runner/src/tmp/planning/temper-base
facts 43, fixture 28, component/writes 22, deliveries 19, cases 12, component/reads 7, runner 7,
job_clone 5, orgs 2, registration 2, mask 2, users 1, tokens 1, setup 1, hooks 1
```
- **03.2**, the module and the leaves that parse or write bytes of their own: `json.rs` whole (reading half
  about 190 lines with `has_key` moving to 03.3, `write` 15), `mask.rs::sorted_units` (a `Vec<String>` read and
  re-written: now `Json::parse`, sort, `json::write`), `hooks.rs::body` (`serde_json::to_string(events)`), and
  `registration.rs`'s runner-file read (`from_slice(&fs::read(file)?)`, two reads). None passes through the family.
- **03.3**, the reads: the family returns `Json`, `Case.body`, `listed`, `rows`, `no_secret` (a walk over
  every object: `Json::has_key("secret")`), and the 347 read sites (at most; counts above).
- **03.4**, the writes: `api(.., body: &[u8])`, the 69 `json!` sites on `json::object` (flat, sorted keys) or
  `json::write` (nested), `Put`, `object`, and `serde_json` leaves the manifest.

```text
$ cargo nextest run -p forgejo-conformance   (scratch: json.rs whole + has_key + mask.rs::sorted_units on skein-json)
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-03
        PASS [   0.002s] (26/33) forgejo-conformance tests::masking_sorts_the_deprecated_units_set_in_every_team
        PASS [   0.104s] (33/33) forgejo-conformance tests::the_listener_holds_its_answer_until_the_probe_replies
     Summary [   0.113s] 33 tests run: 33 passed, 0 skipped
```
```text
$ cargo nextest run -p forgejo-conformance json::tests   (the 8 module tests; 25 base tests skipped)
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-03
     Summary [   0.004s] 8 tests run: 8 passed, 25 skipped        (each 0.002 to 0.003 s)
$ cargo clippy -p forgejo-conformance --all-targets -- -D warnings      -> Finished, exit 0 (1.76 s)
$ cargo fmt --check -p forgejo-conformance                               -> exit 0 (after `cargo fmt` on the two test lines)
```
The 8 tests split 4 / 1 / 3 over 03.2 / 03.3 / 03.4: 03.2 the first three and
`an_array_of_strings_is_written_compact_and_escaped`; 03.3 `a_key_is_found_at_any_depth_and_a_value_is_not_a_key`;
03.4 the object and escape tests. The conformance has 25 tests at the base: 29, 30, 33 after the three.

### The base form of the Done-when commands (not recorded above)

```text
$ cargo check --workspace --all-targets
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s
```
(0.20 s: the target directory was built by the runs above.)
```text
$ rg -n 'temper_engine_protocol_forge\b|temper-engine-protocol-forge($|[^-])' Cargo.toml Cargo.lock crates testing tests tools -g '*.rs' -g Cargo.toml -g Cargo.lock
directory: /srv/data/git/runner/src/tmp/planning/temper-base          (exit 0; counted per package with -c, 63 lines in 38 files)
Cargo.lock 8, Cargo.toml 2, crates/temper 4 (4 files), crates/temper-engine-protocol-forge 1 (its manifest's name),
crates/temper-forge-setup-shell 3 (2), crates/temper-service 19 (12), tests/codecs 5 (4), tests/engine/protocol-forge 14 (8),
tests/one-process 2 (2), tools/forgejo-conformance 5 (3)
```
The pattern leaves out `_world` and `-world`, so the world's own name does not match. The world stays until
temper 07.1, but it names the crate (14 lines in 8 files, the manifest included), and 03.1 repoints those
lines onto `skein_forge_connection`: after 03.1 the command, which searches `tests` too, prints nothing.
```text
$ rg -n 'temper_engine_protocol_forge( as |::\{.* as )' crates testing tests tools -g '*.rs'      (base form of the alias check)
directory: /srv/data/git/runner/src/tmp/planning/temper-base          (6 lines in 6 files)
crates/temper-service/src/tests/forge.rs:6:use temper_engine_protocol_forge::{self as kit, StartRead, Stopped};
tools/forgejo-conformance/src/component/hosted.rs:13:use temper_engine_protocol_forge::{self as kit, Component, Event, Request, Stopped};
crates/temper-service/src/tests/fixtures/forge.rs:3:use temper_engine_protocol_forge as kit;
crates/temper-service/src/route.rs:4:use temper_engine_protocol_forge as forge;
crates/temper-service/src/forge.rs:9:use temper_engine_protocol_forge as kit;
crates/temper-service/src/engine.rs:386:    use temper_engine_protocol_forge as kit;
```
Lines that use those aliases: `forge.rs` 39, `tests/forge.rs` 19, `tests/fixtures/forge.rs` 6, `hosted.rs` 13,
`engine.rs` 9 (from line 386), `route.rs` 2 (`forge::Request`): 88; the 25 other lines that name the crate in the
users are direct paths. About 113 changed lines in the users outside the world, 14 in the world, and the
construction of `Settings` and `Forge` in 7 files (about 30): about 170 changed lines in all.
```text
$ rg -n 'serde_json|json!' tools/forgejo-conformance -g '*.rs' -g Cargo.toml        (base form: prints the lines)
directory: /srv/data/git/runner/src/tmp/planning/temper-base          (96 lines in 18 files: 17 sources and the manifest)
```
```text
$ rg -c --sort path 'temper/0\.0\.0' crates tests tools -g '*.rs'          (base form of the literal check)
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-protocol-forge/src/conventions.rs:1
crates/temper-engine-protocol-forge/src/tests/connection.rs:1
```
After 03.1 the same command prints `crates/temper-service/src/forge.rs:1` (the `USER_AGENT` constant) and
`crates/temper-service/src/tests/forge.rs:1` (the wire test's expected bytes), in that order.
```text
$ rg -n 'conventions::(host|USER_AGENT)' crates tests tools -g '*.rs'       (base form: 6 lines in 4 files)
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper/src/forge.rs:258:        host: conventions::host(address, None).expect("checked address"),
crates/temper/src/forge.rs:259:        user_agent: conventions::USER_AGENT,
crates/temper-engine-protocol-forge/src/tests.rs:102:        assert_eq!(conventions::host(address, server), Some(Box::from(expected)), "{name}");
crates/temper-engine-protocol-forge/src/settings.rs:26:    /// `Host`'s value, made by `conventions::host`.
crates/temper-engine-protocol-forge/src/connection.rs:175:        headers.push(field(b"User-Agent", crate::conventions::USER_AGENT.as_bytes())).expect("five header fields");
crates/temper-service/src/forge.rs:56:        let Some(host) = kit::conventions::host(endpoint.address, name) else {
```
The two `crates/temper/src/forge.rs` lines are the setup shell's `reconcile_settings`; 03.1 turns them into
`skein_forge::host::host(address, None)` and `temper_service::USER_AGENT`, so the command prints nothing after it.
```text
$ rg -n '(temper_engine_protocol_forge|skein_forge_connection)( as |::\{.* as )' crates testing tests tools -g '*.rs'
directory: /srv/data/git/runner/src/tmp/planning/temper-base          (the alias check in its final form: 6 lines in 6 files, the same as above)
crates/temper-service/src/tests/forge.rs:6, tests/fixtures/forge.rs:3, engine.rs:386, route.rs:4, forge.rs:9, hosted.rs:13
```

### The repin, in its base form (`cargo update -p skein-lib`)

skein 03.1 is not on skein's origin/main (93052c6: no crate with `forge` in its name under `crates/`), so the
repin cannot reach `skein_forge_connection`. Run in the scratch worktree, as the sibling findings did:
```text
$ cargo update -p skein-lib
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-03
    Updating git repository `https://github.com/forgetic/skein.git`
   54c5a8b..93052c6  HEAD       -> origin/HEAD
    Updating skein-llm-world v0.0.0 (https://github.com/forgetic/skein.git#8a51e10b) -> #93052c6b
    ... (29 skein packages, all 8a51e10b -> 93052c6b)
$ cargo check --workspace --all-targets
error[E0004]: non-exhaustive patterns: `skein_io::kernel::Op::Lock { .. }` not covered
   --> crates/temper-worker-protocol-git/src/files.rs:352:11
error: could not compile `temper-worker-protocol-git` (lib) due to 2 previous errors
```
The update exits 0 and the check fails on skein's later kernel (`Op::Lock`, and a second error in the same crate),
which is the repair of the tools plan's first repin, not of this plan: README Rules say a temper increment repairs
in its repin commit what skein's steps since temper's last repin broke. `Cargo.lock` restored afterwards.

### The 03.5 sketch (the `Request` arm and the two head tests), compiled in the scratch

The scratch holds the base form: the base's component omits the user agent from the head, so the boundary is
1_048_576 - 192 - 27 = 1_048_357 (27 = Host `forge.example:3000` 18 + token `api-token` 9). After the repin onto
skein 03.2 the head also counts the 12-byte agent: 1_048_345 and 1_048_346.
```text
$ cargo nextest run -p temper-service a_head_past_the_request_room the_longest_head_of_the_shipped_forge
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-03
    Starting 2 tests across 1 binary (217 tests skipped)
        PASS [   0.007s] (1/2) temper-service tests::forge::a_head_past_the_request_room_is_a_problem_naming_its_members_and_the_forge
        PASS [   0.010s] (2/2) temper-service tests::forge::the_longest_head_of_the_shipped_forge_fits_its_room_to_the_byte
     Summary [   0.011s] 2 tests run: 2 passed, 217 skipped
$ cargo clippy -p temper-service --all-targets -- -D warnings          -> Finished, exit 0
$ cargo fmt --check -p temper-service                                    -> exit 0
```
Shape of the arm in the scratch (`crates/temper-service/src/forge.rs`): `Unusable::Request { forge }` pushes
`forge_path(u32::from(forge)).member(b"api_token")` with `What::Bound`, then names `forge_component.target_bytes`,
`forge_component.token_bytes`, `forge_component.tls.send` and `io.output`; `indexed` gains `Request { forge }` in
its or-pattern. The shipped profile's worst head: 4096 target + 258 Host + 512 token + 12 agent + 192 fixed = 5070
bytes, against `tls.send` 16_384 and `io.output` 1_048_576 (derived by hand from the shipped profile).

## Unsettled (named in the reply)

- Numbering: skein's design and the README name "temper 03.2" for the conformance on skein-json; this file makes
  it 03.2 to 03.4 (the json module, the reads, the writes) and puts the repin onto skein 03.2/03.3 in 03.5. The
  README sessions row of `skein/06-conformance.md`, the README critical path, decisions.md 27 and its table row,
  and `skein/06-conformance.md` (the lines naming "temper 03.2") then read "temper 03.4".
- The conformance is about 700 production lines moved onto skein-json, not decision 27's "about 250": the 69
  `json!` sites and about 347 read sites are code in substance. The file counts the mechanical accessor rewrites of
  03.3 as moved (350 lines changed in place, as temper 02.1 counted changed callers) and says that counted as new
  the increment is about 400.
- `Word` (maps to the component-wide `forge_component` member) and `Request` (adds `api_token` and `io.output`) are
  this file's choices; the design says only "broaden the explanation and index".
- A `Request` or `PageSize` refusal of the shipped profile at the 03.5 repin is a design question; the file changes
  no bound and returns it.
- The process tier failed at the base twice, in different tests, each passing on a rerun alone.
- A TLS forge's Host now comes from its server name (decision 25), not from the base's `server_name.is_some()`
  default: equal for every fixture.
- The README's Rules do not name temper-service's `forge.rs` and `settings.rs` as hotspots (temper 01.3 does).
- No extra dependency: skein-json is already a workspace dependency; `serde_json` stays one (6 other packages).
- There is no temper conformance proof script at the base; the stable-fact check is the command written out in
  the file's Done when.
- `tests::forge` counts: 10 at the base; temper 01.3 adds the owner unit (11 at 03.1's start); 03.1 adds 5 (16) and
  03.5 adds 2 (18). No other temper session file adds to temper-service's `tests/forge.rs`.
- temper-service's `startup/application.rs` has its own `Webhooks` (line 31); the kit's is reached through
  `settings::Webhooks` (two lines there, and `settings.rs` line 69's `pub use`), which 03.1 re-points.
- The README's row for temper 03 says "03.3: skein 02.2" for the page sizes (sic); this file names skein 03.3, which
  prices them, and so 03.5 needs skein 03.3.

## Scratch removed

The scratch worktree `tmp/planning/scratch-temper-03` (uncommitted: the stand-in variants, the `Request` arm, the
five scratch tests and `tools/forgejo-conformance/src/json.rs` with its 8 tests) was removed with `git
worktree remove --force` from `tmp/planning/temper-base`, whose status is clean (0 changed files). Nothing of it
is committed or landed; the session file carries the sketches that compiled.
