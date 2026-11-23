# jig's slow tests moved down: their holders

2026-11-23, at `b574a13436a`, for `temper/docs/design/draft/slow-tests-down.md`, 3.2 (its execution file's D5; `temper/docs/development/slow-tests-survey-2026-11-23-smith-jig.md`, groups 1, 2, 4 and 9). For each slow test that jig's goal removes or shortens: where each of its assertions is held below, or the lower test the goal adds. What stays above is listed last. Where a test is new, the change that adds it names it in its report.

## Retired: duplicates

| Slow test | Its assertion | Held by |
|---|---|---|
| `jig-client-browser-world::browser_skein_stories` `a_panic_at_start_reloads_then_says_jig_failed` | a panic at start reloads, then says the application failed | skein's `skein-ui-web-tests::browser_stories` `a_panic_reloads_then_says_it_failed` (the browser's reload and the failure page) |
| | at most three reloads in a minute | `skein-ui-web`'s `a_fourth_panic_in_a_minute_says_the_application_failed` |
| | jig's shell installs that limit, under its key, with jig's words | a new step test in `jig-client-shell-web` over the restart part jig's look gives the shell; the words are also `the_credential_dialog_s_words_are_jig_s` |
| `browser_jig_stories` `jig_s_pending_request_story_holds_for_jig_test`, shortened to its reload and warning | the pending request kept across the reload and made once after it | `jig-client-protocol-world::suite` `a_reload_restores_a_pending_request_from_the_small_store_s_bytes`; `jig-client-world::suite` `a_reload_retains_the_request_before_resending` |
| `jig-test::process` `jig_test_serves_smiths_agent_modes`, its signed-out row | exit 1 reads as not signed in | `smith-protocol-program`'s `each_status_reads_as_programs_md_says`; the child's exec hands on exit and answer as the signed-in row shows |
| `jig-test::process` `jig_test_serves_its_bundle_s_page_under_the_reference_policy` | the page is served with the reference policy | `jig-test-service`'s `each_answer_carries_the_reference_policy_s_five_fields` |
| | the page's bytes | `jig-test-service`'s `the_page_is_revalidated_by_its_etag`; `jig-test`'s `a_bundle_directory_is_read_with_its_one_digest_directory` |
| | the shipped binary serves its bundle | `browser_tier` `a_person_opens_the_printed_link_and_sees_their_inbox` |

## Moved, every assertion with it

| Slow test | Its lower place |
|---|---|
| `jig-engine-clients-world::suite`, its five provider tests | `jig-engine-clients-world::protocol`, same names, with a committed test key and a fixed client secret in place of `openssl`'s; the scratch key file's own mode check goes with the scratch file |
| `jig-test::process`'s `a_premature_exit_names_its_status_milestone_and_safe_stderr_tail` | `jig-test::step`, same name |

## Stays above

- `browser_attacks`: the cross-port, cross-site and framed-page stories, the browser's own rules.
- `browser_jig_stories`: the two reloads and their `beforeunload` results.
- `browser_tier`: the printed link opening the shipped bundle.
- `jig-test::process`: the relay's usage, the program child refused with no parent, one signed-in program child, the killed binary's restart, and SIGTERM with a sign-in under way.
- `jig-test::real`: the native private-file reader's group bits, leaf link and linked directory.
