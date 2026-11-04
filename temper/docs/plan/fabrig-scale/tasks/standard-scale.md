+++
id = "standard-scale"
part = "jig-standard"
kind = "feature"
needs = ["store-ceilings", "override-relations", "smith-calls"]
files = ["jig/crates/jig-protocol-config/src/profile/standard.rs", "jig/crates/jig-protocol-config/src/tests/profile.rs", "jig/crates/jig-protocol-config/src/tests/domain.rs", "jig/crates/jig-protocol-config/src/tests/overrides.rs", "jig/crates/jig-protocol-config/src/tests/provider.rs", "jig/testing/jig-test-service/src/**", "jig/tests/core/tests/suite/settings_memory.rs", "jig/tests/core/tests/suite/settings_proposal_edges.rs", "jig/tests/store-codecs/tests/suite/profiles.rs", "jig/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 120
+++

# jig's standard profile holds a project of fabrig's scale

## Goal

`jig/crates/jig-protocol-config/src/profile/standard.rs` is raised in place to contracts K2:
fabrig's own development with five times the largest figure seen, every limit those figures
couple moved with them (the least that passes), and its store's value, page and chunk sized by
the largest row the raised limits make, its page holding the frame of smith's largest turn
(`jig/docs/design/protocol/configuration.md`, 6). `small` does not change.

Acceptance:

- `standard.rs`'s constants, each a field set in its owner's struct (the others stay as they
  are, or small's through the spreads):
  - `TASKS`: `tasks` 128, `project_tasks` 128, `project_goals` 80, `tree_tasks` 384, `depth`
    32, `delegates` 80, `batch` 160, `inbox_messages` 82, `run_inbox` 82, `spec_bytes`
    131,072, `result_bytes` 262,144, `message_bytes` 262,144, `charters` 12,
    `executor_kinds` 16, `authority_grants` 16, `authority_segments` 8, `hold_segments` 8,
    `contract_choices` 4 (K2); and, coupled: `inbox_bytes` 21,495,808 (82 × 262,144: a
    result's room for each of 80 live delegates, a message's for a question and an answer);
    `batch_bytes` 4,194,304 (five times the largest plan seen, 30 tasks of at most 19 kB,
    under 660 kB with their contracts and authority; 4 MiB); `funders`, `pools`,
    `hold_waiters` and `exemptions` 128, with the live tasks, as both profiles keep them.
  - `PEOPLE`: `operations` 81, `inbox_tasks` 256 (K2), `inbox_recipients` 176 (twice
    `inbox_messages`, the roles' 8 and four: `jig-domain-core`'s `inbox::waiting_capacity`);
    `words` 8,192 (the lead's addition, K2: a person's words, a goal's spec a person writes
    among them, at people-v14's `bytes 8192` ceiling, the most without a format change).
  - `AUTHORITY`: `grants` 16, `executors` 16, `segments` 8 (K2); `batch` stays 4.
  - `NOTES`: `scopes` 19 (three and the grants).
  - `DOCUMENTS`: `lines` 16 (K2, the lead's: a creating offer's every charter and executor),
    and `segments: super::name_segments(&TASKS, &AUTHORITY)` (it inherits small's today).
  - `HUB`: `inbox` 82, with the run's inbox (the most a run's start carries).
  - `STORE_KV`: `value` the largest value `sizes::check_commits` computes for standard
    (exact, as each growth before set it); `page.bytes` the larger of key plus value and
    `jig_protocol_store::sizes::frame(&CORE, 33_554_432)` (smith's largest turn, K3);
    `chunk` `skein_kv::chunk_bytes(key, value)`; each written as a literal, as today, the
    tests below holding them to their rules. `ops`, `commit`, `queued_bytes`, `segment` and
    `snapshot_after` stay: the journal's writes, the key and the map's budget do not move.
  - `CORE.routes`, `JOURNAL` and the map's `budget` stay.
  The comment block above `STORE_KV` gets one line per figure that moved and why (as its
  neighbours), the out-of-date ones ("(batch + 1)^2 = (4 + 1)^2 = 25 bytes") removed.
- Both profiles still pass every check: `jig_domain_core::valid`, `valid_hub`, the clients'
  wire checks, `capacity`, `check_message`, `check_effective_limits`, `writes_bound` within
  `JOURNAL.writes`, and `check_commits`; the policy and amendment maxima stay 43,695 and 8,192
  bytes for both (`configuration.md`, 6 states them).
- `jig/docs/design/protocol/configuration.status.md`: the **Not yet** "6: `standard` is a
  team's, not fabrig's development's ..." removed, the rest of the line kept.

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K1, K2 (with
  `documents.lines`) and K3.
- `jig/docs/design/protocol/configuration.md`, 6 (the bullets "A profile" and "Standard holds
  fabrig's own development"); `jig/docs/design/protocol/store.md`, 5.4 (the bullet "The largest
  value is the largest tasks row") and 8.1 (the frame's ceiling).
- `jig/docs/design/domain/tasks.md`, 8.2 (the inbox's room kept) and 11.

## Owns and may change

- `standard.rs`, and every jig test that pins one of standard's figures these change, each
  edited in its figure: `jig-protocol-config/src/tests/profile.rs` (the page 10,769,629),
  `src/tests/domain.rs` (standard's `(operations, delegates, project_goals)`, now
  `(81, 80, 80)`), `src/tests/overrides.rs` and `src/tests/provider.rs` where a row pins a
  standard figure; `jig/testing/jig-test-service/src/tests.rs`
  (`both_jig_profiles_have_usable_store_edges`, standard's value; commit 901,830,282 and ops
  303,350 stay); `jig/tests/core/tests/suite/settings_memory.rs` and
  `settings_proposal_edges.rs` (their standard figures, which twelve charters move).
- **The test service** (`jig/testing/jig-test-service/src/**`): if a world that runs standard
  is refused because a scripted worker's agent bounds fall below a raised figure (its inbox
  below `run_inbox` 82, say), raise that fixture's bound to the figure; list each.
- If a check refuses standard and its words name a limit this spec does not set, set that limit
  to the least that passes and list it with the check's words, unless it is the machine's
  concurrency (`hub.slots`, `fleet.slots`, `fleet.workers`, `clients.streams`,
  `clients_http.calls` and the like) or a figure this spec gives: then stop and ask with the
  check's words and the figure it needs.
- If a maximum of the policy or amendment envelope differs from 43,695 / 8,192 for standard,
  stop and ask with both figures.
- No other crate; no temper file (temper's documents take `standard` in `init-standard`).

## Steps

1. The constants above; run `heavy cargo nextest run -p jig-test-service --lib -E 'test(=tests::both_jig_profiles_have_usable_store_edges)'`,
   whose failure prints standard's computed value; set `value`, `page`, `chunk` from it and the
   frame; rerun.
2. Tests, in `jig-protocol-config/src/tests/profile.rs`:
   - `standard_holds_fabrig_scale`: asserts each K2 figure and each coupled figure above on
     `profile(Profile::Standard)` (so the contract is pinned in one place);
   - `standard_page_holds_smiths_largest_turn`: standard's `store.store.page.bytes` is at
     least `jig_protocol_store::sizes::frame(&limits.core, 33_554_432)`, and that frame is
     `Some`;
   - `small_is_unchanged`: small's `tasks.tasks`, `tasks.batch`, `tasks.spec_bytes`,
     `tasks.result_bytes` and `STORE_KV.value` are 8, 2, 4,096, 512 and 2,166,708.
3. Narrow: `heavy cargo nextest run -p jig-protocol-config --lib -E 'test(/tests::profile::|tests::domain::/)'`;
   then `-p jig-protocol-config`, `-p jig-test-service`, `-p jig-core-world`,
   `-p jig-store-codecs-world`, `-p jig-store-world`, and `heavy cargo check -p temper-service --tests`.
   Print the store figures: `heavy cargo nextest run -p jig-store-codecs-world --test suite -E 'test(=profiles::question_formats_fit_the_shipped_envelopes)' --no-capture`.
4. The status line; `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy
   -p jig-protocol-config -p jig-test-service --all-targets -- -D warnings`; the citations
   check; commit and push.

## Report

As the implementer's text asks, with: standard's `STORE_KV` before and after (key, value,
page, chunk, ops, commit), the TurnRecord frame at 32 MiB, every limit set beyond this spec's
list and the check that asked for it, and every fixture raised. The figures are the resource
gate's inputs: a value, page or chunk past 100 MB is said so in the report's first lines.

## Rebase (2026-11-03, step merge, foreman fabrig-scale/standard-scale/foreman-1)

resolve both test conflicts by retaining upstream Small sizing and this branch’s scaled Standard sizing.
The merge conflicts with integration commit `46bf4d5a76cad03a3022edfc3f6186e4a17e16f9`, which incorporates main’s `56aadce2d5` RunPolicy.tools sizing fix.
- In `jig/crates/jig-protocol-config/src/tests/profile.rs`, resolve `configured_read_room_is_smaller_than_the_maximum_frame` with Small page `2_168_979` and Standard page `59_570_499`; retain `MAX_FRAME = 33_554_496`.
- Preserve the branch’s added validity and scale tests. Update `small_is_unchanged`’s store value assertion from `2_166_708` to upstream’s `2_166_900`; retain its other assertions. Do not revert upstream’s Small constants.
- In `jig/testing/jig-test-service/src/tests.rs`, resolve `both_jig_profiles_have_usable_store_edges` with Small value `1_156_772` and Standard value `59_568_420`. Keep both profiles’ existing commit and ops figures.
Run the focused profile tests and `both_jig_profiles_have_usable_store_edges` through heavy, then complete the spec’s checks.

## Answers

- 2026-11-03. Q: may standard's store figures follow the current codec (value 59,568,612) rather than
  the rebase note's literals (value 59,568,420, page 59,570,499)? A: yes; the codec governs. The
  acceptance sets `value` to what `sizes::check_commits` computes, exactly, and page and chunk from
  it; the rebase note's literals predate main's `56aadce2d5`, which priced `RunPolicy.tools` and
  raised small's value by the same 192 bytes. Set standard's `STORE_KV` to value `59_568_612`, page
  bytes `59_570_691` (key 2,079 plus value, above the 33,554,496-byte frame), chunk
  `skein_kv::chunk_bytes(2079, 59_568_612)` (59,570,711 if it adds 20 as before; use what it
  computes); move `configured_read_room_is_smaller_than_the_maximum_frame`'s standard page and
  `both_jig_profiles_have_usable_store_edges`'s standard value with them, and the comment above
  `STORE_KV` to say `RunPolicy.tools` is priced. The rest of the rebase note stands. Then run the
  spec's remaining checks.
