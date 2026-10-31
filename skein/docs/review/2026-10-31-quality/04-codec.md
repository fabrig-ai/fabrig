# Code-quality review: codec area (skein-codec, skein-codegen, tests/codec, tests/codec-patches)

The code is careful line by line. The problem is structural: the diff and the patch-text reader are written twice, behaviours are tested in three or four places, and the generator rewrites its own output as text. Codec is a binary-only format, so skein-forge, skein-llm and fake-forge-github decoding JSON by hand is correct by design today.

Tree: `skein (at 3371027)`. All paths below are relative to it.

## 1. Scope read

- **Read closely:**
  - skein-codec: `lib.rs`, `piece.rs`, `schema.rs`, `schema/{grammar,layout,tokens}.rs`, `value.rs`, `patch.rs`, `patch/plan.rs:89–770`, the start of `patch/run.rs` and the tokenizer in `patch/text.rs`.
  - skein-codegen: `lib.rs`, `parse.rs`, `diff.rs`, `diff/compare.rs`, `emit_codec.rs`, `emit_functions.rs`, `patch/{skeleton,text}.rs`, `test_helpers.rs`.
  - Generated output: `every/v1.rs`, sampled.
  - Tests: `build.rs`, `drift.rs`, `bounds.rs`, `suite/{generic,patch}.rs`, `fuzzy/generic.rs`, `qa_schema.rs`, parts of `qa_value.rs` and `qa_patch.rs`, `schema/tests.rs`, `story.rs`, and the scratch helpers in every test crate.
- **Skimmed:** fingerprint, `run/{build,walk}.rs`, the rest of `plan.rs`, codegen `patch/{derive,golden,module,renumber,check,files}.rs`, the other `emit_*` files, the codegen test suites, `qa_diff.rs`, `qa_fingerprint.rs`, `memory.rs`. For `lints/v1.rs` I only took statistics.

**tests/codec breakdown:**
- Generated: 9,051 lines (`src/*/v*.rs` 8,297 + `patches/mod.rs` 754).
- Hand-written Rust: about 4,970 lines (`qa_*` 2,183, `suite/` 2,195, `support/` 464, the rest about 130), plus about 700 lines of schema and diff fixtures.
- The hand-written part is not economical; see findings 2 and 5.

## 2. Scores

| Criterion | Score | Why |
|---|---|---|
| Cleanliness | 3 | The step code is clean. Generated lines reach 8,340 characters, and checks for names removed in an old rename linger. |
| Organization | 3 | Module trees are sound, but `plan.rs` is 1,736 lines covering five concerns, and the `qa_*` test binaries sit beside `suite/`. |
| Taste | 3 | The step code is careful. The generator edits its own output as text, and a lint workaround leaks into the public API. |
| Naming | 4 | Follows the design's words (kept/current, held, pieces, site, Halt, Step). Test names like `s12_3_*` break the pattern. |
| Primitives | 3 | Schema tables, Value and the Old/Build split are good. The diff is implemented twice and the patch text is parsed twice. |
| Elegance | 3 | Text → tables → index fits the problem. The generator's "emit, then split" does not. |
| Economy | 2 | One algorithm written twice, four test tiers covering the same behaviours, seven scratch harnesses, duplicated fixtures. |
| Complexity | 3 | Patching is inherently complex. On top of that, `build.rs` generates every fixture a second time and golden tests run two or three times. |

## 3. Strengths

- The step code holds the subset well: every loop is bounded with its reason stated (`grammar.rs:183`, `value.rs:563`), every `expect` names the invariant it relies on, and refusals carry a line and a span (`SchemaError`, `PatchError`).
- One grammar serves everyone: `schema::parse` is used by the run time, the generator and the diff, and the diff works directly on the codec's tables.
- The generated patch module (`codec-patches/patches/mod.rs`) and the skeleton the command writes are small and clear, and plain to review.
- Table tests carry their data in rows with named helpers (`run/tests.rs` `DERIVED`, codegen `tests/suite/diff.rs` `Row`).

## 4. Findings, ranked by payoff

**1. The diff (codec.md 10.1–10.3) is implemented twice. (Primitives / Economy)**
- The step version is `crates/skein-codec/src/patch/plan.rs:89–770` (about 680 lines). The ordinary-Rust version is `testing/skein-codegen/src/diff/{matching,compare,walk}.rs` (538 lines).
- It is the same algorithm, function for function:
  - `compare` (`compare.rs:55`) and `compare` (`plan.rs:554`);
  - `held` (`compare.rs:30`) and `held` (`plan.rs:366`);
  - `integer_width` (`compare.rs:145`) and `width` (`plan.rs:396`);
  - `same_shape` (`matching.rs:50`) and `same_shape` (`plan.rs:136`);
  - `walk::diff` and `plan::changes` (`plan.rs:672`, which emits the same `Add/Reshape/Versioned/Widen/Wrap/Retype/Payload/Order/Drop` list).
- The tests repeat the split: codegen `tests/suite/diff.rs` (883), `tests/codec/tests/qa_diff.rs` (504) and `plan/tests.rs` (840) each test the 10.3 table.
- **Why it matters:** two copies of one rule set can drift apart, and nothing checks that they agree.
- **Fix:** skein-codec exports matching, comparison and the change list (renames given as name pairs). Codegen's `diff` keeps only spelling, proposals and display.
- **Size:** about −300 lines in codegen `diff/`, and about −300 lines of duplicate test tables.

**2. Behaviours are tested in three or four places. (Economy)**
- **Schema:**
  - `qa_schema.rs` duplicates `schema/tests.rs` pair by pair: `:256/:261` ↔ `:263`, `:236/:243` ↔ `:275`, `:82/:99` ↔ `:158/:175`, `:60/:71` ↔ `:192/:204`, `:315` ↔ `:316`, `:23` ↔ `:330`.
  - It also duplicates codegen `tests/suite/parse.rs:118/:127/:137` (via `qa_schema.rs:289/:298/:28`).
- **Value:** the generic-versus-generated oracle exists three times: `qa_value.rs:341–398`, `suite/generic.rs`, and `fuzzy/generic.rs` (whose `Oracle::compare` repeats the comparison).
- **Patch goldens:** the patched goldens are checked three times: `qa_patch.rs:522`, `suite/patch.rs:330–391` (with its own `SKEIN_PATCH_WRITE` writer), and the generated `p0001_goldens` in `patches/mod.rs`.
- **The command:** `codec-patches/tests/qa_patch.rs` (33 tests) repeats codegen `patch_run.rs`/`patch_check.rs` almost name for name (baseline guard, frozen rename waits, re-derive keeps values, two branch patches, conflict markers, renumbering).
- **Within files:** `bytes`/`secret` near-copy pairs should be rows of one table.
- **Fix:** keep one tier per behaviour. Fold `qa_*` into `suite/`, which also removes the `#[path]` includes and the `#[expect(dead_code)]` at `support/mod.rs:4` and `qa_diff.rs:12`.
- **Size:** about −1,000 of the 2,941 `qa_*` lines, and about −100 in the unit-test pairs.

**3. The generator edits its own output as text. (Elegance / Taste)**
- `emit_functions.rs` (169 lines) re-reads emitted source to split long functions:
  - it finds `pub fn encoded_bound` by prefix and the function's end by its closing line (`:10–20`);
  - it decides chunk boundaries by counting braces (`:115`);
  - it renames variables with `tail.replace("decoded_{f},", …)` (`:139`);
  - it pulls header lines out with `body.remove(0)`, keyed on the text `"let version ="` (`:107`).
- `emit_codec.rs:257–262` swaps `"fn decode_from(limits:"` for `"_limits"` after the fact.
- `emit_record` tests `fields.is_empty() && !versioned` four times (`:204–229`).
- **Fix:** chunk the fields while emitting, since the generator knows the field boundaries. Decide `_limits` from `uses_limits` before writing the signature. Move the empty-record case into its own emitter.
- **Size:** `emit_functions.rs` drops to about 30 lines, and the output's helpers become structural.

**4. The generated code is hard to read, though codec.md §1 says it is "reviewed like any other code". (Cleanliness)**
- Line length: `lints/v1.rs` has a longest line of 8,340 characters and 363 lines over 200; `every/v1.rs` has 81 lines over 200.
- Indentation: loop bodies are not indented (`every/v1.rs:888–893`). `add_size` hard-codes eight spaces (`emit_codec.rs:15`) even though `depth` is passed down.
- Noise: `size.checked_add(0)`, `checked_mul(0_u64)`, `checked_mul(1)`, and `check_limits` wrapping a single `check_limits_0`.
- Retyping: `Problem { path: Path::X, reason: skein_codec::Reason::Short }` is written out in full (103 uses of `skein_codec::Reason::` in `every/v1.rs`).
- **Fix:**
  - indent from `depth`;
  - emit one statement per line;
  - add `use skein_codec::Reason;` and a generated `fn problem(path, reason)`;
  - drop terms that are zero or one.

**5. Generated code is compiled two or three times. (Complexity)**
- `tests/codec/build.rs` regenerates all six families into `OUT_DIR`, and `suite/names.rs` compiles them into the suite again (about 8,300 lines).
- The only purpose is to assert that each bound equals the committed copy's. That is already implied by `drift.rs`, which checks the committed code byte for byte.
- `crates/skein-codec/src/lib.rs:14–17` `#[path]`-includes `tests/codec/src/generated/v1.rs`, so the sample family's golden tables run a third time.
- Leftovers from an old rename still assert that `worst_case_*` is absent: `build.rs:24–28`, codegen `src/tests.rs:16–18`, and `every_bound_name_is_reserved`.
- **Fix:** delete `build.rs`, `names.rs`, the `[build-dependencies]` entry, the path include, and the old-name checks.
- **Size:** about −90 lines, and a much smaller suite binary.

**6. The scratch directory is reinvented seven times. (Economy)**
- `skein_scratch::Scratch` is already a dependency of both crates, yet these roll their own:
  - `codec-patches/tests/suite/story.rs:36–49`;
  - `codec-patches/tests/qa_patch.rs:21–35`;
  - codegen `tests/suite/patch_run.rs:23`, `patch_check.rs:16`, `patch_generated.rs:19`;
  - codegen `src/patch/golden.rs:357`, `src/patch/module.rs:281`.
- About 20 tests end with a manual `fs::remove_dir_all`, so a failing test leaves its directory behind.
- File-writing helpers (`put`, `write`, `write_files`, `author`) appear six times, and the tree snapshot twice (`story.rs:76`, `qa_patch.rs:63`).
- **Fix:** use `Scratch`, plus one shared fixture module (write `&[(&str, &str)]`, snapshot a tree).
- **Size:** about −150 lines.

**7. The codec-patches story: what it is and whether it pays.**
- **The mechanism:** `story.rs` is one 260-line test. It replays an owner's history through `patch::run`: baseline, asked value, frozen rename waiting, written site with skeleton, then renumbering on a rebase across two scratch trees. Each step's inputs and expected outputs are files under `story/NN-*/`. The test ends by checking that the replayed tree equals the committed `patches/` and `schema/`.
- **Verdict:** worth keeping. It is the only end-to-end test of 12.2–12.6.
- **The waste:** it pins the whole `patches/mod.rs` at six steps (68–157 lines each). `02-field/mod.rs` and `03-renames/mod.rs` differ in only two lines. Meanwhile `clean()` already calls `patch::current(root)`, which proves the module matches the pinned inputs.
- **Fix:** drop the per-step `text(…, "patches/mod.rs", …)` pins and the `mod.rs` fixtures. The final tree comparison still pins the module once.
- **Size:** about −580 fixture lines, and a change to the module format no longer touches six files.

**8. Smaller items**
- **Organization:** `plan.rs` (1,736 lines) holds matching, comparison, changes, line checks, literal encoding and assembly. Split it the way codegen's `diff/` is split.
- **Economy:** codegen re-parses patch text "leniently" (`patch/text.rs`, 158 lines) because skein-codec's `Parsed` is `pub(crate)`. Expose a read-only view instead.
- **Taste:**
  - The `CONTEXT` literal is retyped in `skeleton.rs:11`, `:82` (as a tautological `assert_eq!(CONTEXT, <same text>)`) and `patch_run.rs:268`. Use a raw string or `include_str!`, and delete the tautology.
  - More than three bools become `pub skein_bools: [bool; N]` in the public `Parts` struct (`emit_types.rs:55, 148`), losing field names to avoid one lint allowance.
- **Primitives:** `Value` carries the patcher's growth mode (`value.rs:17, 124–158`: `growing`, `reserve`, `use_reserved`).
- **Cleanliness:** `spelled`/`add` are copied in `schema.rs:420`, `patch.rs:428–435` and `value.rs:628`. Span slicing is retyped in tests (`qa_patch.rs:86, 118`). A public `Span::of(text)` would cover all of them.
- **Tests:**
  - `bounds.rs` uses bare sizes (`8_u32`, `22_u32`, `report_bytes(9, 0)`) where `CEILINGS.*` belongs.
  - `codec-patches` test names carry `s12_3_*` section prefixes.
- **Stale text:** the Cargo description and the doc comment in skein-codec's `lib.rs` describe "bounded reads". About 90% of the crate is now schema, value and patch.

## 5. Cross-area questions and suspicions

**Is codec a binary codec, or a schema-to-codec tool that could have generated the JSON decoders?** It is a binary codec, so not adopting it for JSON is correct by design today.
- codec.md §3 fixes the format: fixed-width big-endian integers, `u32` length prefixes, byte tags. §1 says one value has one encoding. Codecs carry "the bodies of a framed channel … and records kept in files or stores".
- The schema language has no key names, no absent-versus-null, no skipping of unknown keys, and no signed integers or floats. The emitters write only `to_be_bytes()`, and readers use `reader.u32()` and the like.
- Both docs leave JSON open:
  - codec.md §13: "JSON from schemas: the same generator writing JSON codecs over skein-json".
  - json.md §5: "Decoding is the application's … small state machines".
  - json.md §8: "verbose without serde or traits. If that hurts, the candidate is codec.md's generator".
- Generating JSON decoders would mean extending the language first.

**Where codec likely should have been used (unverified beyond my area):**
- `crates/skein-llm/src/replay.rs:34–80` hand-rolls a versioned binary envelope: a `u16` version, a `u8` provider tag, then `u32` length plus bytes. That is exactly a `versioned record Replay { provider: Provider, data: bytes N }`.
- `crates/skein-forge-client/src/identity.rs:25, 129` may be similar (a length-prefixed namespace).

**Patterns likely to recur elsewhere:**
- The test harness reinvents `Scratch`; check other `testing/*` crates.
- `qa_*.rs` binaries duplicate focused suites; also present in `tests/oauth`, `tests/oauth-accounts`, `tests/browser`, `testing/skein-lab-browser`.
- Generated output from other users of the generator, `crates/skein-oauth/src/generated/v3.rs` and `examples/ui/view`, will have the problems of findings 3 and 4.

## 6. Rule costs (low priority)

- **No `_` arm and no `matches!`:** the step `leaf` plus `compare` (`plan.rs:413–645`, about 230 lines) does the same work as codegen's `compare.rs:55–143` (90 lines), about 2.5×. Value's nine accessors each spell eight dead arms (about 70 lines).
- **No `From` with `?`:** `Old` forwards 12 accessors only to map `Misread` to `Halt` (`run.rs`, 97 lines).
- **No generics:** six index newtypes with identical `new`/`index` (about 90 lines).
- **No lint allowances in generated code:** the grouped bools (item 8) and the text-surgery splitter (finding 3) exist to satisfy lints without an allowance.