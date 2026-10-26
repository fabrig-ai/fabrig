# Findings for skein 10: the split of skein 01's 01.5

No base run was made: the base forms are recorded in `base-skein-01.md` (the 01.5 estimate table,
"Answers 7-12 and 35, traced against the base", the Date change and the skein-http Date check).
This file records the split, the per-test estimates, the Size sums and the names checked.

## Source

`/srv/data/git/runner/src/tmp/planning/drafts/01.5.v1.md`, skein 01's 01.5 "Pure plan tests and the
settled differences" before the split: 27 new tests (12 in calls.rs, 15 in edges.rs), 220 production
plus 880 tests and fixtures = about 1,100 new, 0 moved, five behaviour changes (Q7, Q8, Q9, Q10, Q12)
and two pinned ones (Q11, Q35). Its own Size line asks for a planner ruling on the excess; the ruling
is this session. No test or behaviour is dropped; the split adds one test (28).

What the split forces, beyond the move of text:

- The source's second reference test (`another_word_has_no_key`) was in edges.rs, which 10.2 now
  creates before 10.3: 10.2 carries its own Run-driven reference (`pull_files_stops_at_a_stale_head`)
  and 10.3 keeps the source's reference. 10.1 keeps the source's other reference, plus a Run one.
- Pinned behaviour first (10.1-10.3), changes after (10.4, 10.5), so every change shows against a
  green test of the neighbouring behaviour. The source mixed them in one file.
- `fixtures::Run` must serve 10.4 and 10.5 without editing 10.1's tests. Three choices keep that:
  the default head carries a `Date` (else Q10 breaks the 19 read cases); canned resolve answers
  carry the id asked (else Q12 breaks them); the PullFiles case has fewer rows than its page (else
  Q10's X-HasMore default turns its `more`).
- `absent_optional_headers_use_the_fallback` has two halves with different behaviour: the 429 half
  is pinned (base already `RateLimited { after: 0 }`), the X-HasMore half changes behaviour (Q10).
  Split into `rate_limited_without_retry_after_waits_zero` (10.3) and
  `pull_files_without_has_more_is_more_only_when_full` (10.5); the session file says so in 10.3's
  Shape. The source's name is not kept for either half, since each half pins one thing.
- Q7's write arm: `failure::before_write` converts only Timeout and InvalidAnswer to Unavailable, so a
  bare Timeout for OpenPull's reads before its POST would become Unavailable, not the TooLarge Q7
  keeps. 10.4 prescribes `if before_write() { TooLarge } else { Timeout }` in the writes.rs Bound arm.
- 10.5 repairs `a_body_past_body_bytes_is_carried_empty_without_its_key` in skein's world (08.1
  moved it). The repair is a separate `fixtures::dated_response`, not a Date in `response`:
  `fuzzy_machine`'s byte-cut sweeps run 7.5-7.7 s against a 7.8 s ceiling and a Date line in its three
  heads would lengthen them. Not a Reply change: skein-http writes the Date on the wire and refuses an
  owner-supplied one (`base-skein-01.md`), so only raw scripted bytes lack it.

## Test-to-increment table

Lines are estimates (the per-test lines of 01.5 are not itemised in `base-skein-01.md`, only the file
totals 430 calls.rs, 350 edges.rs, 100 shared and 220 production; these split those totals).

| Test | File | Inc | Behaviour | Lines |
|---|---|---|---|---|
| read_calls_follow_canned_exchanges | calls | 10.1 | pinned | 144 |
| user_search_uses_id_and_one_result | calls | 10.1 | pinned | 18 |
| page_size_is_passed_to_each_listing | calls | 10.1 | pinned | 26 |
| branch_slashes_and_2554_have_golden_targets | calls | 10.1 | pinned | 22 |
| long_body_revision_has_big_endian_golden | calls | 10.1 | pinned | 9 |
| shared fixtures::Run, tests.rs, imports | shared | 10.1 | none | 70 |
| write_calls_follow_canned_exchanges | calls | 10.2 | pinned | 99 |
| refusals_match_status_and_whole_message | calls | 10.2 | pinned | 36 |
| first_failure_ends_the_plan | calls | 10.2 | pinned | 16 |
| unseen_write_is_unavailable | calls | 10.2 | pinned | 9 |
| seen_write_is_timeout | calls | 10.2 | pinned | 9 |
| open_pull_read_failure_is_unavailable | calls | 10.2 | pinned | 22 |
| reviewers_are_added_before_removed | calls | 10.2 | pinned | 20 |
| pull_files_stops_at_a_stale_head | edges | 10.2 | pinned | 18 |
| job_mismatch_is_missing_job | edges | 10.2 | pinned | 24 |
| shared Run additions (fail, answer_with) | shared | 10.2 | none | 20 |
| deliveries_map_subscribed_events_to_hints | edges | 10.3 | pinned | 52 |
| irrelevant_delivery_has_no_hint | edges | 10.3 | pinned | 10 |
| markers_accept_words_at_both_bounds | edges | 10.3 | pinned | 20 |
| malformed_word_is_refused | edges | 10.3 | pinned | 8 |
| another_word_has_no_key | edges | 10.3 | pinned | 6 |
| success_statuses_match_each_exchange (Q11) | edges | 10.3 | pinned | 34 |
| merge_read_429_is_timeout (Q35) | edges | 10.3 | pinned | 16 |
| rate_limited_without_retry_after_waits_zero (half) | edges | 10.3 | pinned | 10 |
| shared imports | shared | 10.3 | none | 4 |
| write_inner_overflow_is_timeout (Q7) | edges | 10.4 | change | 24 test + 30 prod |
| file_cuts_to_remaining_answer_room (Q9) | edges | 10.4 | change | 30 test + 40 prod |
| repository_id_mismatch_is_invalid_answer (Q12) | edges | 10.4 | change | 32 test + 65 prod |
| shared imports | shared | 10.4 | none | 3 |
| answering_404_keeps_the_path (Q8) | edges | 10.5 | change | 28 test + 60 prod |
| items_without_date_is_invalid_answer (Q10) | edges | 10.5 | change | 20 test + 10 prod |
| pull_files_without_has_more_is_more_only_when_full (Q10 half) | edges | 10.5 | change | 18 test + 15 prod |
| world repair (dated_response, one script) | machine world | 10.5 | none | 12 |
| shared imports | shared | 10.5 | none | 3 |

Tests: 5 + 9 + 8 + 3 + 3 = 28 (the source's 27 plus the one from the split). Behaviour changes: Q7, Q9,
Q12 in 10.4; Q8, Q10 (two parts) in 10.5; each has its named test. Q11 and Q35 are pinned in 10.3.

Production per change (from `base-skein-01.md`'s 220, and its Date change of 10 production + 25 test
lines): Q7 30, Q9 40, Q12 65, Q8 60, Date 10, X-HasMore 15 = 220.

## Size sums

| Inc | Tests and fixtures | Production | World | Total new | Size line |
|---|---|---|---|---|---|
| 10.1 | calls 230 (the tables' asserts, after the review), Run 70 = 300 | 0 | 0 | 300 | about 300 |
| 10.2 | calls 211, edges 42, Run 20 = 273 | 0 | 0 | 273 | about 270 |
| 10.3 | edges 156, imports 4 = 160 | 0 | 0 | 160 | 160 |
| 10.4 | edges 86, Run 15 (after the review), imports 3 = 104 | 135 | 0 | 239 | about 240 |
| 10.5 | edges 66, imports 3 = 69 | 85 | 12 | 166 | about 165 |
| Sum | 906 | 220 | 12 | 1,138 | |

Sum check against the source: 430 + 350 + 100 = 880 tests and fixtures, 220 production, 1,100; the
extra 12 is the world repair the source assigned to temper's repin. None passes about 300 (the
largest is 10.1 at 289); all five are under the bound, so the planner's excess ruling needs no
further split. Moved lines: 0 in each (a file with no base source counts all new, organization 6).
Cumulative filtered counts: 5, 14, 22, 25, 28.

## Names checked

Base worktrees: temper `/srv/data/git/runner/src/tmp/planning/temper-base` (dd12b8e), skein
`/srv/data/git/runner/src/tmp/planning/skein-base` (064c55f). skein-forge itself does not exist at the
skein base (skein 01 creates it); its names are skein 01's sketches, checked in
`skein/01-forge.md` lines 130-146 and 250-262. Paths below are under
`crates/temper-engine-protocol-forge/src/` unless said.

- Plan code the changes touch (temper base, rg-checked now):
  - `writes.rs:254` Bound arm; `:287` `fn before_write`; `:349` `Stage::Merged => Timeout`; `:367`
    the stage set after the merge; `:166` the expected status 200 for the read stages.
  - `reads.rs:86`, `:292` `Time::ZERO` (the Items head without Date); `:100`, `:122`
    `RepositoryById`; `:257` `raw_room`; `:646` `keep_document`.
  - `reads/done.rs:108` `repository_done`; `:217` `search_done`.
  - `contents.rs:106` `Range::Head`; `:117` `raw_room`; `:148` `has_more.unwrap_or(false)`; `:338`
    `fn raw`.
  - `actions.rs:316` `Range::Tail`; `:369` `raw_room`.
  - `failure.rs:28` `classify`; `:76` `before_write`.
  - `connection.rs:25` `struct Fields` (`:31` `has_more`); `:371` the Job log read-through
    `Malformed::Bound { field: b"log" }`; `:447` `response`; `:456` `X-HasMore`.
  - `reading.rs:66` `fn revision`; `tables.rs:48` `keep_repository`; `marker.rs:5-31` (base `decode`
    takes the body only; skein 01.3 adds the word, as the reference test has it).
  - `hooks/hint.rs:7` `fn hint(forge, delivery)` (skein 01.3: `hint(forge, word, delivery)`).
- Dialect (`crates/temper-forge-forgejo/src/`): `decode.rs:15` `Malformed::Bound`; `client.rs:28`
  `struct Encoded`; `time.rs:125` `fn http_date` (IMF-fixdate only); `tests/fixtures.rs:15` `user`,
  `:28` `small`, `:57` `struct Decoded`, `:61` `decode`, `:64` `decode_cut`.
- Plan fixtures: `crates/temper-engine-protocol-forge/src/tests/fixtures.rs:44` `REPO`, `:399`
  `specimen` (both go to skein's `forgejo/tests/fixtures.rs` in 01.3).
- Machine world (temper base): `tests/engine/protocol-forge/src/fixtures.rs:109` `response`
  (no Date line); `tests/engine/protocol-forge/tests/machine.rs:133`
  `a_body_past_body_bytes_is_carried_empty_without_its_key`, scripting
  `Response::Bytes(fixtures::response(200, ..))` for its Items head. skein 08.1 moves both to
  `tests/forge/machine/{src/fixtures.rs,tests/machine.rs}`; the 10.5 repair uses that path.
- skein base: `crates/skein-lib/src/time.rs:28` `Wall::from_nanos`; `crates/skein-http/src/server.rs:312`
  and `server/response.rs:3,28` (the Date the server writes and refuses from an owner).
- Not found at the base, by design: `plans/tests/{calls,edges}.rs`, `fixtures::Run`,
  `fixtures::dated_response` and the names of all 28 new tests (grep: no match in either base, apart
  from the machine test above). They are new here. `OP_BYTES` has no match under the plans' crate;
  it is the operation-key bound 512 of the protocol fixtures (`limits()` at
  `tests/engine/protocol-forge/src/fixtures.rs:6`), and the session file names the value (512).

## Open points

- Q7 and the Job log read-through. forge.md 5.3 lists "a log read through" among malformed answers
  (InvalidAnswer on a read). Answer 7 maps a bound overflow to TooLarge on a read; the arm matches the
  whole `Malformed::Bound` variant, so the Job log read-through (`connection.rs:371`) follows it. 10.4
  states this and sends the planner here; either the designer says the read-through stays InvalidAnswer
  (then the read arm tests `field`), or forge.md 5.3's list is amended. Not settled by the source.
  Planner's ruling: forge.md 5.3 names it ("a body past the component's `body_bytes_in`, a log read
  through included"), so it stays InvalidAnswer; 10.4's read arm tests the field and its test pins it.
- `fixtures::Run`'s method names (`known`, `new`, `with`, `begin`, `answer`, `answer_with`, `fail`,
  `sent`, `tables`) and `dated_response` are sketches; the author may rename them when no test
  changes meaning.
- The machine world's 42 passes only once skein 08 completes (38 after 08.1 alone, per
  `skein/08-machine-world.md`); the 10.5 Proof says so.
- `base-skein-01.md`'s cumulative counts (126 after 01.5) are stale: with 01.5 out, skein 01 ends at
  99 and skein 10 adds 28, so the crate holds 127 after skein 10 (before skein 06.3's own tests).
- Per-test lines are estimates; if an author's 10.1 or 10.2 passes about 300, the planner's next
  split is the harness (Run, 70 lines) as its own increment.
