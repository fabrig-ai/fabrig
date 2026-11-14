# Engine component performance work in progress

Task 3006, after all prerequisite changes landed. Base: goal branch at `25fc331de59ed18bd79fdd8820b5adb92abfbe21`. All nine modified Rust files preserve their public results, assertions, wire/schema texts, fingerprints and limit values. No process-wide cache, tier, budget or build-setting change was made.

## Implemented

- `skein-codec/schema.rs` and `schema/tokens.rs`: one character/span pass instead of lexing twice just to count tokens. Token capacity uses the text-length bound already covered by `schema_bound`; character checks still precede grammar. A trial byte-classification constant was removed after profiling; it is not part of the work.
- `schema/limits.rs`: fixed-width fields and earlier declaration references do not allocate/spell a named-limit key that their branch never reads.
- `skein-lib/sha256.rs`: bounded partial-block and length copies use slice copies, padding uses fill; full input blocks still go straight to compression. Matched git diagnostic: 315,642,138 -> 309,906,283 instructions, other changes held fixed.
- `tests/setup_admin.rs`: the ten synthetic access-stop mapping cases share their checked service; all assertions remain.
- `tests/git_tokens.rs`: thirteen stop cases share transport/App records, with fresh identity generations and empty, unannounced token tables per case. All generation, withdrawal, account and renewal assertions remain literal and unchanged. Git profile 2,991,280,215 -> 309,906,283 instructions before the last stock-word improvement. Admin profile 2,602,530,843 -> 338,294,747.
- `tests/fixtures.rs`: expose the completed root/store first start as `completed_store`; `completed_maps` takes its maps. `tests/setup_github.rs::owner_service` takes that store and reads its header rather than starting a disposable service. The profiled bootstrap cost was 208,247,501 of 540,994,636 instructions (38.49%).
- `temper-engine-service-stock/stock/kinds.rs::word`: allocate a final-length repeated-byte box instead of appending each byte through List. In the 54-schedule accounts profile, word-building cost 675,070,451 of 2,269,848,575 instructions (29.74%). Same test after this fix: 1,604,395,362 instructions. Kinds' values and limits are unchanged.

## Measurements

`measurement-notes.json` retains every nextest time, process CPU samples, and 51 individually completed Callgrind profiles with normalized repository-relative frames. No new test or timing assertion was added. The default focused profile selects 579 passing tests and excludes the existing fuzzy-route test.

- Measured first: 23.776121 CPU s (23.328947 user + 0.447174 system), one process, one thread; 450/579 raw serial nextest samples >40 ms; 579 passed.
- The initial cached executable and a dependency rebuild differed in cost despite unchanged SHA compression source. No cause is claimed for that build-artifact difference. To avoid crediting it, a second paired baseline restored all nine source files from the goal tip, rebuilt through the same cargo command, measured, and restored the implementation without moving history.
- Rebuilt original-source paired baseline: 28.461441 CPU s (28.013293 + 0.448148); 455/579 raw nextest samples >40 ms; 579 passed.
- After all fixes: 25.245859 CPU s (24.814101 + 0.431758); 451/579 in that raw nextest sample. A subsequent rebuilt nextest sample passed 579 but had 461 >40 ms. These are reported observations, not proof of formal idle-machine guide medians. No per-test CPU is claimed.
- Clippy passed for skein-lib, skein-codec, temper-engine-service-component and temper-engine-service-stock. Whole component measurements passed after each group; named admin and git tests passed. One compile failure from Map lacking clear was repaired with a fresh same-capacity map. No product assertion failure was observed.

## Remaining work / question

The remaining cold constructor work is dominated by skein-codec parsing, fingerprinting and bridge planning, with no per-process cache allowed. Examples from current individually completed profiles: non-format open refusal rows cost 2,440,294,928 instructions, including 1,967,886,455 in workers construction and 829,813,413 in schema parsing; pre-settings admission costs 2,122,066,068, including 1,743,994,763 in parsing the retained family texts for store admission/held-cycle checks. The independently checked cold rows and actual restarts cannot share mutated services.

A single `skein-codec` per-crate opt-level override is the user's decision, expressly not added here. The requester is being asked whether to seek that approval or leave the guides unmet with complete profiles/reporting. Per-test profiling was stopped between batches for this question: 51 completed profiles are retained; the interrupted admin second-access profile is unobserved, not failed. No implementation may change while a profiling/test command is running.

Next actions after the answer: remeasure after any approved change; complete profiles for remaining >40 ms tests (51 already retained in JSON), explain their costs; run cargo fmt once when done, old_forms for Skein and Temper (and any other touched tree), remove all scratch scripts/data before checked delivery. A permanent measurement appendix may carry the full per-test report if it exceeds the finish transport's 63,738-byte body bound. No design/help document has been touched.

Exact implementation/check minute split is unknown across host resumptions. The current implementation/profiling phase began about 14:25 UTC and reached 15:12 UTC on 2026-11-14; that span includes checks and profiling waits, not implementation minutes alone. Do not invent a split.
