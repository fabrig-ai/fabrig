# Session skein 02: `skein-forge-client`

Move the bounded client, then apply its admission, identity and recovery contracts.

**Needs at start:** temper 01.1's client on skein 01.1's vocabulary. Copy
temper's `origin/main` at 02.1's start; record its commit (decision 2).
Paths/counts below are at the base. 02.2 owns both measures (decision 24).

## Read first

- skein `docs/design/forge-client.md`, 2–10, and `forge.md`, 2.1, 3.7,
  at `25d2551`; designer answers 7 and 14–22 (all settled).
- skein `docs/foundation/programming-model.md`, 3, 4.5, 6.3, 9, 10;
  `docs/foundation/testing-strategy.md`, 2.1, 2.2, 8, 10.
- This plan's README (sessions, Rules, Done when), `code-organization.md`,
  1–3, 5–8, and `decisions.md`, 2–5, 14, 18, 19, 21, 24.
- Findings: `survey-02-client.md`, `trace-client.md`, `base-runs.md`, `base-skein-02.md`.

## What the design asks

- 2: the child domain, commit-before-release, bounded output and admission.
- 3–4: atomic working sets, durable scans, hints, drift and delivered news.
- 5–6: one fresh-read terminal, retries, four classes and their shared budget.
- 7: opaque keys, authenticated writers, lanes, recovery and echo suppression.
- 8: restore, read afresh, then settle; saved attempts retain their clocks.
- 9: purposes and policy stay with the owner.
- 10: step coverage here; durable-parent and saturation tests in skein 07.1.

## Choices this plan makes

- `api.rs` keeps only `Shape`, `accepts` and its helpers in this crate;
  vocabulary and `locate`: skein 01.1; counters and locate tests stay here.
- Two purpose tests stay temper's: 76−2=74 moved; no vocabulary tests leave.
- Four increments: move, measure, splits and behavior, restart coverage.
- Answers 7 and 14–22: base matches get regression tests, differences a Shape; repins repair temper.

## Rules that bind it

- No rewrite of a moved part. Only the forced changes listed in 02.1's
  Shape accompany the copy; behavior changes start in 02.2.
- No system name, downstream dependency, re-export or alias of vocabulary
  types. The only dependencies are `skein-lib` and `skein-forge`.
- Moved tests keep names/cost: 76 took 0.027 s here (survey 0.051 s),
  0.002–0.005 s each. New units <20 ms, unoptimized; no new world/fuzzy suite.

## Increments

### 02.1 The client moved, with opaque effect identities

Scope: sections 2–9 as implemented; section 7.1's owner-supplied identity.
Needs: the session's start capability, temper 01.1's vocabulary cutover.
Files: from temper `crates/temper-engine-domain-forge-client/` to new
  skein `crates/skein-forge-client/`: `Cargo.toml`, `src/{lib,boundary,
  bounds,calls,domain,facts,identity,keep,limits,outbox}.rs`, the remaining
  answer-check part of `src/api.rs`, `src/tests.rs`, `src/tests/{keep,
  outbox,locate}.rs`. New `src/tests/contracts.rs`. Changed shared
  `Cargo.toml` (workspace dependency; membership is `crates/*`) and
  `Cargo.lock`. Exclude only `locate.rs` and vocabulary definitions moved by 01.1.
Shape:
- Keep the public `Domain`, boundary types, `Limits`, `Fact`, `Priority`,
  `recovery`, `step`, `fire`, `resume`, queries and all four byte counters
  unchanged through 02.1. Private `api`; vocabulary/locate use `skein_forge::{api,locate}`.
  No public forwarding module or type aliases.
- Keep `bounds::{bytes,optional,rows,summary,files,answer_heap,pull}` and
  public `answer_bytes_unbounded`, its bounds implementation and the other
  wrappers as at the base; 02.2 replaces them. No measure moved in 01.1.
- Replace purpose encoding with this interface; the frame is version 1,
  two-byte big-endian namespace length, namespace, then opaque identity:
  ```rust
  // crates/skein-forge-client/src/identity.rs
  pub fn effect_key(namespace: &[u8], identity: &[u8], max: u32)
      -> Option<alloc::boxed::Box<[u8]>>;
  ```
  Empty/over-65535 namespace, empty identity or total over max: None.
  `EffectPurpose`, encoding and two tests stay temper's until temper 02.1;
  three callers in domain-forge's `domain.rs`, one in engine-domain's
  `route/forge_route.rs`. That step changes them and their golden vectors.
- Reword the five jig doc lines in `lib`, `boundary` (two), `identity`
  and `outbox`; remove the purpose-only comment. Cite `forge-client.md`
  instead of owner documents. Test namespace/frames use kit and its length.
Test shape: retain 74 tests, names unchanged, including `tests::locate`.
  The two excluded names are `effect_keys_are_stable_by_purpose_and_separate_deployments`
  and `projection_history_keys_keep_task_position_and_history_family`.
  Seven new `tests/contracts.rs` tests (<20 ms) use `tests.rs::{LIMITS,configured}`
  and `identity::historical`:

| Test | Fixture and literal outcome |
|---|---|
| `keys_frame_the_namespace_and_opaque_identity` | Reference below. |
| `invalid_key_frames_are_refused` | Empty namespace, 65536-byte namespace, empty identity, or seven-byte frame capped at 6: None. |
| `the_largest_namespace_keeps_its_identity` | kit repeated 21845 times (65535 bytes), identity a, max 65539: length 65539, prefix [1,255,255], final a. |
| `opaque_identities_are_stable_and_distinct` | kit/a twice: identical seven bytes; identity b or namespace b changes them. |
| `binding_replaces_the_namespace_before_restore` | Configured; bind [7;16]: true; frame with that id believed. |
| `binding_without_configuration_changes_nothing` | Domain::new; bind [7;16]: false, configuration None. |
| `binding_past_op_bytes_preserves_the_namespace` | Configured kit; op_bytes=3 (normal 256); bind [7;16]: false, namespace kit. |

  ```rust
  // crates/skein-forge-client/src/tests/contracts.rs
  #[test]
  fn keys_frame_the_namespace_and_opaque_identity() {
      const KEY_BYTES: u32 = 7; // version + length + kit + one identity byte
      let key = crate::effect_key(b"kit", b"a", KEY_BYTES);
      assert_eq!(key.as_deref(), Some(b"\x01\x00\x03kita".as_slice()));
  }
  ```
Proof: unit, `cargo nextest run -p skein-forge-client`: 81 tests
  pass, zero skipped, <0.5 s, each <20 ms; Progress precedes Call (callers: temper 02.1, skein 07.1).
Don't: split `keep.rs` or `outbox.rs`, fix their behavior, or retarget
  temper's 15 dependent crates here. Do not rename the moved test whose
  existing name begins `actual_cost`; preservation takes precedence.
Size: about 4,300 lines moved, 120 new; production only.
Removes: purpose API from the skein copy; no temper file or byte counter.

### 02.2 The measure

Scope: forge.md 3.7 and forge-client.md 2.3; answer 14, decision 24.
Needs: skein 02.1's client, with its counters, on skein 01.1's vocabulary; skein 01.3's plans.
Files: shared `crates/skein-forge/src/{lib,tests}.rs`, new `src/measure.rs`;
  client `src/{lib,bounds,measure}.rs` and `src/tests/{contracts,keep}.rs`; if already
  copied by skein 03.1, `crates/skein-forge-connection/src/tests/{reads,contents}.rs`; the
  plans' `crates/skein-forge/src/forgejo/plans/{reads,reads/size,reads/done,contents,actions}.rs`.
Shape:
- Create skein-forge's `measure.rs`, with `pub mod measure` in `lib.rs`.
  Count every owned string's length and each list element's fixed width,
  checked through u64, overflow None; no row/page/path validation or non-list width.
  ```rust
  // crates/skein-forge/src/measure.rs
  pub fn op(value: &Op) -> Option<u64>;
  pub fn answer(value: &Answer) -> Option<u64>;
  // Borrowed parts: callers need no owned Op/Answer or allocation.
  pub fn write(value: &Write) -> Option<u64>;
  pub fn summary(value: &Summary) -> Option<u64>;
  pub fn pull(value: &Pull) -> Option<u64>;
  // crates/skein-forge-client/src/measure.rs
  pub fn stored(row: &Stored) -> Option<u64>;
  pub fn effect(effect: &Effect) -> Option<u64>;
  pub fn answer_rows(value: &Answer, rows: u32) -> bool;
  ```
- Width constants (bytes): `SUMMARY_BYTES=128`, `REVIEW_BYTES=128`,
  `STATUS_BYTES=160`, `COMMENT_BYTES=96`, `REMARK_BYTES=64`, `FILE_BYTES=64`,
  `COMMIT_BYTES=32`, `COLLABORATOR_BYTES=16`, `STRING_BYTES=16`, `ID_BYTES=8`.
  Const-assert each >= its type's size, with text; STRING: `Box<[u8]>`, ID: `u64`.
- Client constants: `STAMP_BYTES=16`, `LABEL_BYTES=16`, `REVIEWER_BYTES=8`.
  Const-assert stamp >= size_of::<Delivery>() and size_of::<Echo>(),
  label >= size_of::<Box<[u8]>>(), reviewer >= 8 and size_of::<u64>().
  Stored counts stamps/echoes, cached labels/reviewers and strings; effect
  counts its write plus `Condition::Merge`'s base.
- `answer_rows` returns true iff every answer list, including nested labels,
  reviewers and contexts, is <= rows; scalar answers pass even at zero.
  Admission adds `measure::answer` and its cap; `bounds` keeps page/path/
  max_bytes, reviewer/cache row checks and refusal order.
  Borrowed helpers need no clone/client types; `summary/pull` add only nested list widths.
  Remove client `answer_bytes/answer_bytes_unbounded/stored_bytes/effect_bytes` and
  byte walkers; physical `worst_case` still counts native container storage.
- The plans' running measure takes these widths (one measure, forge.md 3.7): their 18
  `size_of` uses (reads/size.rs's rows, the Branches/Collaborator/Status/Comment rooms,
  File and Commit sizes) name `measure`'s constants, so `reads::size::answer` equals
  `measure::answer` (base Summary 96, Review 104, Status 128, Comment 72, Remark 56, File 48).
- **temper's side:** next repin changes `crates/temper-engine-domain-forge/src/lib.rs`'s
  `effect_bytes` calls (base 150, 370, 372) to `measure::effect`, its `stored_bytes` (373)
  to `measure::stored` and its unbounded call (277) to `skein_forge::measure::answer`;
  `src/domain.rs`'s `keep_named_answer` (178) uses `skein_forge::measure::answer` AND
  `skein_forge_client::measure::answer_rows(value, limits.client.rows)` before comparing
  with `limits.client.answer_bytes`. `crates/temper-engine-protocol-forge/src/tests/`
  `{reads.rs:489,contents.rs:259}` use `skein_forge::measure::answer`; reads.rs's agreement
  test passes unchanged; contents.rs's (255) expects pull files 68 and comparison 100 (52, 84
  at the base), and `raw_sides_at_the_answer_bound_fit_and_one_byte_more_is_too_large`
  (171) sides of 4031 and 4032 (4047, 4048: File 64, not 48).
  `crates/temper-engine-domain/src/store.rs`'s `call_answer_bytes` switches its forge arm
  to `skein_forge::measure::answer` (its three callers unchanged); skein 03.1's copies likewise.
  The component's page sizes (`limits.rs::largest_row`, `size_of`) take the widths in 03.3.
Test shape: retain 81 client tests; add three in `tests/contracts.rs` reusing `tests.rs::{LIMITS,
  configured}` and keep's saved-row fixtures (relocated in 02.3); units <20 ms; overrides isolate caps.
- `stored_measure_uses_fixed_widths`: empty Repository -> Some(0); live
  branch kit -> Some(3); add one comment Delivery and one Echo -> Some(35),
  one review Delivery -> Some(51), cached Summary with label kit and all
  other strings empty -> Some(70); cached Pull kit/kit, reviewers [7,8]
  -> Some(92). Scalar Summary/Pull add no struct width.
- `effect_measure_uses_fixed_widths`: SetReviewers [7,8], None -> Some(16);
  OpenPull empty title/body, kit head/base, None -> Some(6);
  Merge number 1/head [1;32], `Condition::Merge { base: kit }` -> Some(3).
- `answer_admission_uses_fixed_widths`: fresh matching reads: one Items
  Summary with title kit -> 131; one Comment with body kit/key a in an
  otherwise empty Item -> 100; Pull kit/kit, [7,8] -> 22; Job step kit/log
  ab -> 5. At each byte cap one successful Read; cap minus 1 -> TooLarge.
  Five empty Branches -> 80 bytes but rows=4 refuses; also five empty
  labels in Item, five reviewers in Pull, five empty Protection contexts
  refuse with byte room. `answer_rows` is false there; Done/rows=0 is true.
  Dialect tests in `skein-forge/src/tests.rs` (inline values, unmentioned strings/lists empty)
  expect Some(bytes); each answer's also asserts `plans::reads::size::answer` (rows unlimited) agrees:

| Test | Value and expected bytes (the named widths above, never size_of) |
|---|---|
| `measure_answer_summary` | Items with one title kit: SUMMARY_BYTES+3 = 131; same scalar Item: 3. |
| `measure_answer_review` | Reviews with one body kit: REVIEW_BYTES+3 = 131. |
| `measure_answer_status` | Statuses and Checks, one context kit/description ab/url x: STATUS_BYTES+6 = 166. |
| `measure_answer_comment` | Item, one comment body kit/key a: COMMENT_BYTES+4 = 100. |
| `measure_answer_remark` | Remarks, one path kit/body ab: REMARK_BYTES+5 = 69. |
| `measure_answer_file` | PullFiles and Compare, one path kit/before a/after bc: FILE_BYTES+6 = 70. |
| `measure_answer_commit` | Compare with one commit: COMMIT_BYTES = 32; scalar Commit: 0. |
| `measure_answer_collaborator` | Collaborators with one user: COLLABORATOR_BYTES = 16. |
| `measure_answer_string` | Branches [kit]: STRING_BYTES+3 = 19; Item label kit: 19; Protection branch kit/context x: STRING_BYTES+4 = 20. |
| `measure_answer_reviewer` | Pull head/base kit and reviewer 7: ID_BYTES+6 = 14. |
| `measure_op_reviewers` | SetReviewers [7,8]: 2*ID_BYTES = 16. |
| `measure_op_strings` | Locate owner ab/name kit: 5; OpenPull empty title/body, kit head/base: 6. |

  ```rust
  // crates/skein-forge/src/tests.rs
  #[test]
  fn measure_op_reviewers() {
      const TWO_IDS_BYTES: u64 = 16;
      let op = crate::api::Op::Write(crate::api::Write::SetReviewers {
          number: 1, reviewers: alloc::boxed::Box::new([7, 8]),
      });
      assert_eq!(crate::measure::op(&op), Some(TWO_IDS_BYTES));
  }
  ```
Proof: unit, `cargo nextest run -p skein-forge-client`: 84 passed,
  zero skipped, <0.5 s, each <20 ms; admission driven through step (no harness until skein 07.1).
Proof: unit, `cargo nextest run -p skein-forge -E 'test(measure_)'`:
  12 passed, each <20 ms, <0.5 s; other vocabulary/dialect tests unselected.
Don't: fold rows into byte measurement, change refusal order, add serde,
  retain old public wrappers, or change native worst-case accounting.
Size: about 220 lines moved, at most 310 new production lines across both crates (310 with the tests, 30 of them the plans' widths).
Removes: `answer_bytes`, `answer_bytes_unbounded`, `stored_bytes`, `effect_bytes`, byte walkers, layout widths.

### 02.3 The splits, preconditions, identity guard and answer refusal

Scope: forge-client.md 2, 2.3, 3, 5, 7.2 and 8; answers 15–17 and 20.
Needs: skein 02.2's fixed measure and separate row admission.
Files: client `src/{calls,domain,keep,outbox}.rs`; `keep.rs` -> `keep.rs`
  and `keep/{restore,poll,answer,echo}.rs`; `outbox.rs` -> `outbox.rs` and
  `outbox/{answer,find,retry}.rs`. `src/tests{.rs,/keep.rs,/outbox.rs,
  /contracts.rs}`, new `src/tests/fixtures{.rs,/keep.rs,/outbox.rs}`.
Shape:
- Relocate whole functions, no rewrite: `keep` keeps state/admission;
  `restore` takes `restore_live` through `fresh_done`; `poll` takes
  `pump` through `nudge_resource`; `answer` takes `answered_repository`
  through `rest_resource`; `echo` takes `is_echo` through `made`.
  `outbox` keeps state/admission/sending; `answer` takes `answered`,
  `branch_found` through `merge_verified`; `find` takes `Listing`,
  `listed` through `compared`; `retry` takes `remaining`, `not_found`,
  `failed`, `backoff`, `retain_uncertain`. Use module-qualified calls and
  `pub(super)` only as required, no facade re-exports. Resulting lines:
  keep ~380, restore ~90, poll ~240, answer ~500, echo ~165;
  outbox ~380, answer ~250, find ~160, retry ~290; each below 800.
- Preserve the base's refusal order (answer 15): Keep count TooLarge >
  restore Busy > each watch's size TooLarge, participation Refused, duplicate
  Refused, in input order > capacity Busy; Make unready Busy > missing
  writer Forbidden > invalid TooLarge > full Busy; Read TooLarge > Busy.
- Answer 16 preserves assertions: make's duplicate/non-increasing number;
  restore_live/restore_repository's duplicate row or Restore after Restored;
  calls::answered's unknown/already-answered token. Keep their position after
  existing admission checks. Remake's ordering exception, repeated Restored,
  PauseOutbox/ReadAfresh/SettleOutbox and internal invariants stay as at base.
- `Domain` gets private `identity_used: bool`, initially false, set before
  Restore, Keep, Make or Remake, including refused input. `bind_deployment`
  returns false before mutation once set; unchanged signature. Restored
  alone does not seal it (answer 17).
- **temper's side:** `crates/temper-engine-domain-forge/src/domain.rs`'s
  wrapper `bind_deployment` (base 263–264), and the Deployment arm in
  `crates/temper-engine-domain/src/restart.rs:397`: false calls
  `restart_refuse`. Next repin keeps Deployment before client Restore/Keep/
  Make/Remake; adapt tests to false, never swallow it or rebind after rows.
- `calls::answered` turns Err(Busy) into Err(InvalidAnswer) before dispatch
  and facts: read terminal InvalidAnswer; write goes uncertain and finds.
  This changes the base's Err(error) passthrough (answer 20).
Test shape: retain 84 tests. Move only fixture bodies into fixtures.rs and
  its keep/outbox children, exposing helpers within tests. Keep test names
  and modules; after extraction tests.rs ~570, keep.rs ~580, outbox.rs ~520,
  locate.rs 31; fixtures.rs ~160, fixtures/keep.rs ~220, outbox.rs ~210.
  contracts.rs ~450; recovery.rs in 02.4 ~600; each new test <40 lines.
  Six new contracts tests, each <20 ms, using fixtures/LIMITS; first three
  are section 10's refusal pairs:
- `keep_count_refusal_precedes_restore_busy`: unrestored, five watches
  against resources=4 (count TooLarge vs restore Busy): Kept(TooLarge), no Save.
- `make_unready_precedes_missing_writer`: valid entry, unrestored/no config
  (Busy vs Forbidden): one Outcome::Failed(Busy), no Call or Progress.
- `read_size_refusal_precedes_full_slab`: eight pending calls, then branch
  of 257 bytes against op_bytes=256 (TooLarge vs Busy): Read(TooLarge), no Call.
- `binding_after_identity_use_preserves_the_namespace`: fresh fixture per
  Restore(Live), Restore(Repository), Keep, Make, Remake; bind false and
  namespace kit, including refused input. Extend 02.1's successful binding
  test with empty Restored before binding: true; that edit is not a new test.
- `an_answered_busy_is_an_invalid_read_answer`: one fresh read, Busy answer:
  one Read(InvalidAnswer), Failed fact names that error.
- `an_answered_busy_makes_a_write_uncertain`: clock, write, Busy: Progress,
  Uncertain, next call a find, zero additional writes.
Proof: unit, `cargo nextest run -p skein-forge-client`: 90 passed,
  zero skipped, <0.5 s, each <20 ms. No higher client harness until skein 07.1.
Don't: change refusal order/recovery classes, clear failed restoration,
  add an owner dependency, allocator or world.
Size: about 1,650 lines moved, 90 new; production only.
Removes: relocated production/fixture bodies; no tests removed.

### 02.4 Recovery, restart and echo contracts

Scope: forge-client.md 2.3, 7.2–7.6, 8 and 10; answers 7, 18, 19, 21, 22.
Needs: skein 02.3's splits/identity guard and 02.2's measure.
Files: client `src/{domain,keep,outbox}.rs`, `src/keep/{poll,echo}.rs`,
  `src/outbox/{answer,find,retry}.rs`, `src/tests{.rs,/outbox.rs,/recovery.rs}`,
  `src/tests/fixtures{.rs,/keep.rs,/outbox.rs}`; if already moved, world
  `tests/forge/client/tests/{simulation,fuzzy_simulation}.rs` (key fixtures only).
Shape:
- Echo landing changes the base's hinted-only path: pass Env to
  `keep::echo::made`, call `nudge_resource` to re-arm a resting item at
  now+hinted; an active call retains its terminal and consumes hinted at rest.
  Save echo before release:
  ```rust
  // crates/skein-forge-client/src/keep/echo.rs
  pub(crate) fn made(d: &mut Domain, env: &Env<Limits>, entry: &Entry,
      made: Made, out: &mut Queue<Request>);
  ```
  Add Env after d to `finish`, `finished`, `withdraw`, `branch_created`, `done`,
  `merge_verified`; update callers/domain::step's Withdraw; otherwise unchanged.
- Answer 18 already holds: calls::answered requeues RateLimited without
  spending a failure; calls::send invokes outbox::sent on every send. Keep
  Attempt from now/wall+lifetime+clock_margin, fixed start, Progress before Call.
- Answer 19: listed already needs writer+same_key, no issue provenance;
  comments_found/reviews_found also require Original. All three lack the
  namespace-frame check: add identity::historical on the matching key.
  Keep review's id/head checks; Summary gains no provenance; frame fixtures with kit.
- **temper's side:** next repin frames keys with the configured namespace in
  `tests/engine/forge/client/tests/{simulation,fuzzy_simulation}.rs`'s issue/post
  constructors (base namespace world). If skein 07.1 already copied them,
  repair its copies here with kit frames; no production caller signature changes.
- Answers 7/21/22 preserve base behavior: oversized write answers become
  InvalidAnswer; received Timeout stays Timeout; both enter uncertain recovery.
  Unknown/attempted Withdraw emits nothing. Restore failure never clears;
  only a new Domain can recover. Keep still checks watch count before Busy.
  Exists on a sent write finds under its saved attempt; keep non-write errors.
Test shape: retain 90 tests; 24 new tests in `tests/recovery.rs`, <20 ms each.
  Use fixtures' `Harness`, `at`, `configured`, `comment`, `summary`, `current`,
  saved rows/entries and LIMITS; observe ReadAfreshDone/OutboxDone. Overrides
  isolate each bound; profile provenance/admission in findings.
  Reference (the max_out row below):
  ```rust
  // crates/skein-forge-client/src/tests/recovery.rs
  #[test]
  fn max_out_covers_floor_and_batch() {
      let empty = crate::Limits { resources: 0, repositories: 0, ..super::LIMITS };
      assert_eq!(crate::max_out(&empty), 4);
      assert_eq!(crate::max_out(&super::LIMITS), 15);
  }
  ```

| Test | Events/world and literal outcome |
|---|---|
| `restart_reads_before_settling_the_outbox` | Restore paired rows at mark 100; Restored; Writer; PauseOutbox; Make attempted entry; ReadAfresh. Each repository reads Items since 100/page 1 and each resource reads Fresh; one ReadAfreshDone, no write. SettleOutbox finds each attempted entry once, then one OutboxDone. |
| `remake_releases_a_held_entry` | Uncertain comment through Held at saved deadline; Remake saved entry: one new write, Made once. |
| `restore_refuses_bad_pages` | Fresh Domain per zero or inbox+1 comment/review page: restoration_failed true, valid-sized Keep Busy. |
| `restore_refuses_bad_stamps` | Comments/reviews: inbox+1 or repeated id; failed true, Keep Busy. |
| `restore_refuses_oversized_caches` | Item and Pull byte tails at answer_bytes+1 or nested rows+1; failed true, Keep Busy. |
| `restore_refuses_oversized_echoes` | rows+1 echoes; failed true, Keep Busy. |
| `restore_refuses_unpaired_rows` | Live without repository; repository without live; finish Restored: failed true, Keep Busy. |
| `restore_refuses_capacity_overflow` | resources+1 distinct live rows; repositories+1 distinct repository rows; failed true, Keep Busy. |
| `restore_refuses_invalid_watches` | Empty/257-byte branch, participating Repository/Branch; failed true, Keep Busy. |
| `keep_refuses_invalid_participation` | Restored, participating Repository/Branch: Kept(Refused), no Save or Call. |
| `keep_refuses_invalid_branch_names` | Empty/257-byte branch: Kept(TooLarge), unchanged working set. |
| `keep_is_busy_around_failed_restore` | Valid-sized Keep before Restored -> Busy; bad Restore then corrected rows and the first Restored: failed stays true, Keep Busy; normal Read/answer does not clear it. No repeated Restored or late Restore. |
| `oversized_hints_read_nothing` | Idle issue/branch; Hint branch or key of 257 bytes: one Fact::Refused, unchanged deadline, zero calls. |
| `worst_case_rejects_invalid_limits` | Zeros for each required positive field, calls>pending, reserve>rate, backoff>backoff_max, poll>poll_max: None. Zero backoff_max/poll_max fail their ordered pairs. |
| `worst_case_accepts_permitted_zeroes` | Independently zero op_bytes, answer_bytes, reserve, facts: Some; no constructor with invalid ordered pairs. |
| `max_out_covers_floor_and_batch` | Reference above; never construct with zero resources/repositories. |
| `invalid_reads_make_no_call` | Page 0 for six paged reads (Items, Reviews, Statuses, Remarks, PullFiles, Collaborators); File empty path; File/Job max_bytes 0 or 4097: Read(TooLarge), zero Call. |
| `rate_refused_writes_save_a_new_attempt` | Send at 100; RateLimited(after=2s); both clocks to 102; resend: distinct token, failures 0, same start, Progress before Call; sent/wall 102, deadline/expires 113 (was 111). A subsequent find keeps 113. |
| `attempted_or_unknown_withdrawals_are_noops` | Drain prior output/facts; attempted entry and unknown number separately: zero requests/facts; attempted entry still reaches Made once. |
| `echo_landing_rearms_the_resting_resource` | Participating issue/pull resting until 160; Commented/Reviewed at 100: Save echo, read at 101 (hinted=1), no echo news. |
| `issue_find_requires_a_believed_key_without_provenance` | Timed-out CreateIssue; matching framed kit key/writer 1, no provenance: Created(9), found=true. Separate fresh cases with author 2, wrong key, or matching foreign/unframed key: no Made, held at 111. |
| `comment_and_review_finds_require_believed_original_keys` | Timed-out Post/Review: matching foreign/unframed key with writer 1 and Original yields no Made, Held at 111; kit frames yield Commented(9)/Reviewed(9), found=true. Keep moved Revised/Unknown and review-boundary tests. |
| `oversized_and_lost_write_answers_are_uncertain` | Branches [kit] measures 19: bounds::answer true at answer_bytes=19, false at 18 (normal 4096; rows=4), independently of shape. Fresh Post per case, cap 18: that Ok answer -> Failed fact InvalidAnswer; Err(Timeout) -> fact Timeout. Each saves failures=1, Uncertain, then one find and no second write. |
| `exists_after_a_write_finds_under_its_attempt` | OpenPull sent at 100 answers Exists: next call PullFor; start and deadline 111 unchanged, no second write; matching open Pull 9 -> Created(9), found=true. |

  Valid write-answer shapes measure 0 (forge.md 3.3, 3.7): none can exceed
  a byte cap. The size assertion above isolates bounds, not write dispatch.
  The failed instance stays failed. After02.4, skein02.s2 separates the
  identical generic fresh paired-row success from the21 rejection cases,
  preserving all mapped assertions and adding exact-bound pairs. Every-table
  saturation belongs to skein07.1.
Proof: unit, `cargo nextest run -p skein-forge-client`: 114 passed,
  zero skipped, <0.5 s, each <20 ms; echo read at 101, resend deadline 113.
  The durable-parent world first connects in skein 07.1.
Don't: change the preconditions, reset a failed client/find deadline, add
  issue provenance, retry uncertain creations blindly, or add saturation here.
Size: about 0 lines moved, 160 new; production only.
Removes: hinted-only echo landing and recovery's acceptance of unframed/foreign keys.

## Done when

In skein at `origin/main`; base forms/outputs are in findings.

- `cargo nextest list -p skein-forge-client` lists the post-02.s1 count plus6 after02.s2 (136 at the checked base):
  74 preserved names and62 new names at that base; no purpose tests or missing locate tests.
- `cargo nextest run -p skein-forge-client` prints that same relative count passed after02.s2 (136 at the checked base),
  zero skipped; under 0.5 s and every new unit below 20 ms.
- `cargo nextest run -p skein-forge -E 'test(measure_)'` prints
  12 passed, <0.5 s, every test <20 ms; client total is the post-02.s1 count plus6 after02.s2 (136 at the checked base).
- `cargo clippy -p skein-forge -p skein-forge-client --all-targets -- -D warnings` exits 0.
- `rg -n -i 'temper|jig|EffectPurpose' crates/skein-forge-client` prints nothing (status 1).
- `sh -c 'wc -l crates/skein-forge-client/src/*.rs crates/skein-forge-client/src/*/*.rs crates/skein-forge-client/src/tests/fixtures/*.rs | awk '\''$1 > 800 && $2 != "total"'\'''`
  prints nothing; the vocabulary move and 02.3 split the base's oversized files.
