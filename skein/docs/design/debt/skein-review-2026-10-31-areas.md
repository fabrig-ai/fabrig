# skein code-quality review, 2026-10-31: area reports


Resolved 2026-11-01: the eleven QA binaries are removed. Unique rows run in focused step tests/worlds, fuzzy sweeps or the existing browser stories; the real hand main paths are browser tests. Struck lines below preserve the dated assessment.

Companion to `skein-review-2026-10-31.md`. These are the reviewers' own reports, unedited apart from this header and the section dividers. A0 is the coordinator's condensation of the holistic pass, made before the area reviewers started; it is the text they were given. Snapshot: origin/main `4a6683c72`. Paths are relative to `skein/` unless they name another tree. Line numbers refer to the snapshot.

---

# A0-holistic

# A0 holistic (saved verbatim-condensed by the coordinator)

Scores: Cleanliness 3, Organisation 3, Taste 4, Naming 3, Primitives 3, Elegance 3, Economy 3, Complexity 4

Verdict: The crate graph points the right way (no crates/* normal dependency on testing/* or tests/*, skein-lib depends on nothing, step crates stand only on lib and peers), rules are explicit (clippy.toml, model, old-forms ratchet), 0 #[allow], 1 #[ignore], 0 TODO. Weaknesses are drift and duplication between crates: layout docs (testing.md 6, README.md 3) contradicted by the tree; worlds double as kits (7 world crates imported by 14 others); lint configuration hand-copied (48 [lints] in 14 variants, 25 clippy.toml in 14 variants, some silently dropping step denies); one concept in several shapes (three Limits-check shapes, three test-module conventions, two referee frameworks); ~10 primitives rewritten per crate (JSON pump, UTF-8, calendar, token namespaces, Env rewraps, widening, concatenation, limits recipes). Most jig asks are truly missing from skein and skein pays the same cost; two exist but are hidden (skein_json::parse; UTF-8 in skein-codec).

Good: clippy.toml (every ban gives reason + replacement); crates/skein-lib/src/lib.rs (priced containers List::price, Intake::price; Secret with length-only Debug; Decimal/bytes::hex_digit); ratchet hygiene (old-forms max_out/JournalRoom/takes/worst_case 0; component kit onto skein_io::stage 39 -> 8 sites, 31 files use stage::); 32/33 design docs have short status files; 32/36 tests/* use tests/suite + tests/suite/fuzzy.

~~Top: (1) re-sync layout docs + gate layout check (no -world dep, no cross-crate #[path]) ~1 day; (2) three named lint profiles (step, harness, binary) + gate check, testing/clippy.toml; ~1,000 TOML lines; (3) lib/json primitives: bytes::is_utf8 (+prefix), Env::with, u32->usize widen, bytes::concat, slice-fed tokenizer driver, token-namespace helper; ~800-1,200 lines, ~700 sites; ~1 week; (4) fold 11 qa_* (4,824 lines) and 10 other top-level test bins into tests/suite; rename qa-skein; hours; (5) closure-ban lints once in [workspace.lints.clippy] + one #![expect] per tests root; ~200 of 638 expects; hours; (6) merge skein-world's two referee/trace frameworks; move kit pieces from tests/* to testing/; ~300 lines; days.~~

Findings:
~~F1 testing.md 6 layout not the tree's: testing.md:371 "No crate under crates/ has a tests/ directory" vs crates/skein-forge/tests (32 files, 6.7k), skein-fake-forge-protocol/tests, skein-fail-stop/tests; 21 test bins outside tests/suite (11 qa_*, browser_* x7, end_to_end, exhaustive_tls, live); three step-test conventions (tests.rs 71 + src/tests/ 229 files; sibling *_tests.rs 28 files 7.8k mostly oauth/oauth-accounts/browser; inline #[cfg(test)] mod 80 in 76 files); layout omits testing/skein-forge-conformance, skein-lab-browser, tests/oauth-issuer, oauth-accounts, codec-patches, forge/github.~~
F2 Worlds as kits: 17 edges (json-world <- skein-llm dev, tests/http, tests/protocol; llm-world <- fake-peers, llm-connection, oauth, tls; tls-world <- forge/machine, forge/setup, llm-connection, oauth-accounts; forge-machine-world <- forge/protocol, forge/setup; forge-client-world <- forge/protocol; http-world <- tests/llm; echo-world <- echo-shell dev). skein-llm/src/translate.rs:879 uses skein_json_world::generate. Cycle: examples/echo/shell dev-dep skein-echo-world -> skein-echo-shell; shell compiles ../../../tests/echo/tests/end_to_end.rs via [[test]] path. 10/93 #[path] reach outside module tree (skein-codec/src/lib.rs:16 compiles tests/codec/src/generated/v1.rs; tests/fake-peers/tests/suite/github_memory.rs:2; skein-forge/tests/common/mod.rs:9; fake-forge-protocol/tests/suite/memory.rs:4; tests/browser compiles src/pages.rs twice).
F3 README.md 3 crate graph stale: 8 rows disagree with Cargo.toml (codegen, oauth, fake-oauth, forge-setup :93, ui-bundle, fake-machine :107, fake-person-browser, throwaway-forgejo); skein-playground (:68, :109) doesn't exist; playground.md no status file, absent from STATUS.md; 14 crates absent; README.status says "Built: 1 to 4".
F4 [lints] hand-copied: 42 workspace=true, 48 own blocks in 14 variants; 9 lack allow_attributes, 7 lack wildcard_enum_match_arm, 6 lack unwrap_used; tests/browser only unsafe_code=deny, tests/ui-web only forbid; crates/skein-ui-web 16 `_ =>` arms (3 in src/page/web.rs).
F5 clippy.toml 25 copies, 14 variants; testing/skein-world bans Arc/Mutex/RwLock, tests/clippy.toml doesn't but bans skein_browser::boundary::Look; std::env::var different reasons.
F6 638 #[expect], 541 distinct reasons; disallowed_macros 96, too_many_lines 92, disallowed_types 86, disallowed_methods 48, struct_excessive_bools 40, missing_debug_implementations 39, manual_map 33, wildcard_enum_match_arm 22, too_many_arguments 20; manual_map 18 wordings; in crates/, 85/89 disallowed_macros, 44/71 disallowed_types, 27/38 disallowed_methods in step-test files.
F7 Limits validity three shapes: model :759 valid->bool (~22 crates); check()->Result in forge forgejo/limits.rs, github/limits.rs, forge-setup limits.rs + github/limits.rs, forge-connection limits.rs, examples/echo/service; Limits::is_usable io, kv, echo protocol (io has both). Step entry varies (free step(state,&Env<L>,ev,out); methods self.step in oauth/fake-oauth; 12 forge-setup machines step(m, ev, &mut Queue<Need>) no Env; UI view step(&mut self, report, now, wall, duties); channel up/down no Env); Env<Config> 76 vs Env<Limits> 517. No status file records it.
F8 JSON whole-buffer pump ~15x though skein_json::parse (parse.rs:26): oauth/src/json.rs:28; fake-forge-protocol observations/read.rs:140; forge github/server/{app.rs:70, writes.rs:339, queries.rs:120}; github/decode/delivery.rs:282; fake-forge-github {app.rs:256, manifest.rs:294, git.rs:265}; lab-browser json.rs:66; tests/forge/setup decode.rs:51; tests/forge/github runner.rs:1053; ~6 fixtures; json's own parse.rs:37 and tokenizer.rs:138 validate. Missing slice-fed driver; parse takes two Limits. Fix tokenizer::over(input,&limits) next()->Option<Result<Token,Error>>; ~500 lines.
F9 Calendar x3: forge forgejo/time.rs:30,85,125 (RFC3339 r/w, HTTP-date parse); http server/response.rs:255-310 (HTTP-date write); ui phrases.rs:594 (civil). GitHub dialect + forge-connection import forgejo::time (github/header.rs:150, connection.rs:702); 44 lines in github/ name crate::forgejo::{document, id::unsigned, time, path, decode}. Fix skein_lib::time::{rfc3339, civil}, skein_http::date.
F10 UTF-8 three ways: skein_codec::text_is_valid (lib.rs:90, 675 downstream uses via generated codecs); skein-json utf8.rs:9 private Utf8; core::str::from_utf8 under expect (forge github/request.rs:487, browser wire encode.rs:88 decode.rs:483, oauth redirect.rs:106,118, oauth-issuer x2, forge-connection, 3 fakes). Fix skein_lib::bytes::{is_utf8, utf8_prefix}.
F11 Lib gaps: Env rewraps 83 (66 in crates/), no Env::with; usize::try_from(..).expect 557 non-test, 137 messages ("u32 fits usize" x169, "u32 fits" x140); u64::try_from(x.len()) 64 vs len_u64 154; measured concat then Writer in 17 files (fake-forge-protocol delivery.rs:140 and fake-forge-github delivery.rs:398 near-identical POST heads); hand namespaces (oauth-accounts route.rs:14 slot*3, forge-connection hooks.rs:19 1<<59, ui-text terminal.rs:12 1<<62, fake-peers forge/shared.rs:8 7<<60, kv store.rs:25 0x4b56); 88 local fn limits(), 62 literal io::Limits { in 45 files, 59 tokenizer::Limits { in 36.
F12 skein-world two referee frameworks, swapped names: referee.rs:45 trait Referee<P>, :95 struct Expectations<P,X>; domain/referee.rs:22 trait Expectations, :55 struct Referee<X>. Traces: stream.rs:21 pub trace Vec<String>, domain/trace.rs:14 Trace always formats, no off; worlds keep private traces (tests/llm-connection world.rs:283, tests/forge/setup world.rs:26). testing-strategy 7 one referee shape; only 5/36 worlds have referee.rs.
F13 skein cites temper draft playground-reset.md 11x (forge-connection boundary.rs:60 Add/Remove, component/runtime.rs:1, tests/runtime.rs:1, connection.rs:115; forge-setup host.rs:2,174, host/{boundary.rs:110, settings.rs:2, states.rs:93, runtime_tests.rs:1}); forge-connection.md 3.5 designs connect only; no status record. History comments: fake-checkout lib.rs:3 ("Extracted from temper's ... at commit 246164f"), testing/skein-world/src/domain.rs:2.
~~F14 Pipeline vocabulary: 11 qa_* bins (4,824 lines) (tests/codec x5, codec-patches, oauth, oauth-accounts, browser, lab-browser x2); tests/codec/tests/qa_diff.rs:12 expect(dead_code, "The QA oracle ..."); "valid QA schema", "family qa"; skein-forge/src/tests.rs:337 "decision 25"; testing.md:444 qa-skein profile; testing.md 7 history ("before the user's doubling of 10-22", "macOS run 40019066645").~~
F15 old-forms: comment:actual 79 (tests/llm 23, tests/oauth-accounts 14, tests/io 11; then skein-lib 5, skein-llm/fake-peers 4); 358 "actual" mentions incl 7 test names; growth_floor 1 counts lib's own GROWTH_FLOOR (growth.rs:7) as EXCLUDE lacks it, can never fall; earliest_helper 1 is oauth documents.rs:184 fn earliest(tokens), different meaning; host_forwarder 4 (tests/io x2, oauth-accounts x2) waits on component-kit.md 6.1; io_resume 8; programming-model 6.3 still says "Code written before this model still sums a worst case" though all such forms count 0.
F16 Naming: "conformance" two unrelated things (kernel-backend conformance testing/skein-conformance, tests/conformance/{sim,real} vs forge-fact recording testing/skein-forge-conformance, packages forgejo-conformance, github-conformance lacking skein- prefix); skein-fake-forge-protocol is Forgejo's layer, fake-forge-github imports its routes and git; skein-forge/tests/common/generated.rs is hand-written seeded values; "Held" names 8 structs. Fix skein-fake-forge-forgejo; skein-forge-facts; generated.rs -> drawn.rs.
F17 tests/codec generates schemas twice: 6 schemas checked in 8.3k lines (lints/v1.rs 3,778 lines 534KB) and build.rs regenerates all six into OUT_DIR only for tests/suite/names.rs; build.rs:25-29 asserts old worst_case_* absent (finished rename); schema/kept/ 5 files byte-identical to schema/. Other generated (oauth v3, ui view v1) single version, needed.
F18 Size: 96 files >=800, 10 >=1,600 (testing/skein-browser/src/browser.rs 2,289, fake-forge-domain/src/tests.rs 2,235, skein-llm/src/client.rs 1,975); prod fns 474 >60 lines, 99 >100, 10 >200 (fake-forge-protocol observations/read.rs observation 368, observations.rs render 337, skein-browser step_reply 323, skein-io file_layer.rs up_inner 308); skein-forge 51k = 1.7k neutral + 18.6k Forgejo + 24.3k GitHub + 6.7k tests; split candidate.

Downstream asks: skein_json parse pump REFUTED as missing (exists; jig-protocol-config/src/document.rs:20 rewrites it; skein invites copy via own ~15 pumps, two Limits, no non-Document driver). skein-io owner() CONFIRMED missing. kv overheads CONFIRMED missing (frame.rs:12 HEADER, map.rs:15 ENTRY, snapshot.rs:16-17 pub(crate)). skein_ui::date CONFIRMED missing (3 private calendars). Fixed-capacity name CONFIRMED missing (only skein-io/src/addr.rs:27 Name, private). UTF-8 predicate REFINED (skein_codec::text_is_valid exists, not in lib, no prefix cut). Lossy<T> CONFIRMED missing (model 3 names drop-and-count sinks, 7 hand counters). Trace gate REFINED (skein-sim has Trace::Off; skein_world::domain::Trace no gate). codegen mirrors CONFIRMED missing. Env::with, List::from_boxed, widening CONFIRMED missing.

Status cross-check: confirmed lagging world.status 9 (sim-world in crates/), testing.status 6 (fakes' tests/ dirs), STATUS 10.3 rules checker. Missed: three Limits shapes, forge runtime membership, tests/ dirs under skein-forge & fail-stop, playground.md no status. Contradicted: README.status "Built: 1 to 4", testing.md:371. 7 "unknown" entries (STATUS, browser, examples, fake-checkout, forge-client "Not yet: unknown", forge-connection "details unknown", simulator); lib.status.md repeats Built/Not-yet and cites itself (:6); testing.md:473 cites STATUS.md "section 2 and section 3" (don't exist); STATUS dated 10-29, two indexed files 10-30.

Incidental: root Cargo.toml lists 9 skein crates twice in members (fake-checkout, fake-forge-*, fake-llm-*, fake-machine, fake-oauth, fake-peers); skein-browser-world in [workspace.dependencies] unused; skein-fake-peers (custom lints, no clippy::panic) panics on impossible io events (llm.rs:299, forge.rs:298,421, oauth.rs:286, transport/native.rs:101); skein-kv store.rs:1084 (_, _) => unreachable! tuple wildcard in step code; examples/ui/shell-web tree.rs:96,110 _ => unreachable!; skein-fake-checkout uses std::time::Duration and is the only crate with missing_docs=deny; skein-forge/tests/common/forgejo.rs:48,447,544 reads tests/forge/conformance/transcripts by tree path.

---

# A1-forge-github

# A1 forge-github: skein-forge (minus src/forgejo) and tests/forge/github

Scores: Cleanliness 4, Organisation 3, Taste 3, Naming 3, Primitives 2, Elegance 3, Economy 3, Complexity 3

Verdict: The GitHub dialect does what forge-github.md asks. Its plans are small pure `begin`/`heard` machines, its failure map names the fact behind each message, and its tests are thorough and written in the step subset. Two things hold it back. First, it is built beside Forgejo's dialect, not on shared ground. It borrows neutral pieces from `crate::forgejo` (44 lines) and copies mechanisms both dialects need: Limits, Failure, LRU tables, the field walker, the tokenizer pump, hex, and percent-encoding and query parsing. Second, its central types are flat. The decoder and the read plan each hold every family's state at once, and the flat `Node` and `Request` enums force the same variant lists to be written again and again. About 1.3-1.6k of ~26k lines could go with no loss. Two departures from the programming model (a generic `Step<T>`, and `Answer<'a>` types) are not recorded.

## Good

- `src/github/{jwt,redirect,guid}.rs` and `src/marker.rs`: small, documented, and matching the design word for word.
- `src/github/failure.rs:6-39`: each message constant names its conformance fact.
- `src/directory.rs`, `measure.rs`, `host.rs`: neutral root pieces shared by both dialects. This is the pattern F1-F4 should follow.
- `decode/protection.rs` with `decode/walk/protection.rs`: a family owns its node enum and field table. This is the model for F6.
- `github/server.rs:31-81` `route` re-encodes the parsed request and compares it with the target, so one encoder defines both sides.
- `src/github/tests/` is written in the subset, with 0 file-level `#![expect]` (`crates/skein-forge/tests/` has 65).
- `tests/forge/github` checks its facts against `forge-github.md` (`src/tests.rs:340`), and `mask.rs` builds on the shared Mask.

## Top recommendations

1. Move the neutral pieces to the skein-forge root: `time`, `path` (with `segment_length`, unescape, query), `Body`, `Malformed`, `Limits` plus `check(limits, depth)`, plan `Failure`, and the user table. This removes 44 github→forgejo lines and 20 more in skein-fake-forge-github, and deletes two downstream translators. About -250 lines, half a day, mechanical. (F1-F3)
2. Add an LRU map to skein-lib, as programming-model 10.2 requires. It replaces 4 eviction loops here and 2 in oauth-accounts. About -120 lines, small. (F4)
3. Nest `Node` and `Request` by family, as Protection already is, and make the decoder and the read plan enums of per-family state. About -600 lines, removes 17 exhaustive `Read` accessor matches, medium effort. (F6, F8, F10)
4. One slice-fed tokenizer driver for the 9 whole-buffer pumps in this area. About -200 lines, small, to be coordinated with A7. (F11)
5. One shared plan-test driver, with `tests/suite/github_*` folded into `src/github/tests`. About -450 lines, medium. (F14)
6. Record or remove `Step<T>` and `Answer<'a>`, and fix both status files. Trivial. (F5, status cross-check)

## Findings

**F1. GitHub's dialect is built on Forgejo's modules, which the design does not plan.**
- 44 non-test lines in `src/github` name `crate::forgejo` (`rg -c forgejo src/github` with the tests excluded; 56 with them).
- The items borrowed: `document::Body` 25, `id::unsigned` 13, `time::{read,write,http_date}`, `path::{segment,file}`, and `decode::Malformed`, which `github/decode.rs:7` re-exports as GitHub's own.
- The coupling leaks downstream. `skein-fake-forge-github` names `skein_forge::forgejo` 20 times: unsigned 8, Body 8, time 3, limits 1. `tests/suite/github_items.rs:4` imports `forgejo::document::Body as Description`.
- `forgejo::id::unsigned` (`forgejo/id.rs:15-19`) is just an alias for `skein_lib::bytes::decimal`. `github/redirect.rs` calls `decimal` directly, so one module uses two spellings.
- forge-github.md 2 lists "Names, times and ids" as the module's own, and forge.md 4.1 lists them under Forgejo's dialect. The status files are silent.
- Fix: root modules `crate::{time, path, text::Body, decode::Malformed}`, and call `bytes::decimal` directly.

**F2. Limits are written twice.**
- `github/limits.rs:5-43` and `forgejo/limits.rs:3-35` have the same six fields.
- The `Problem` enums differ only in naming (`String` against `TokenizerString`). The `check` functions differ only in depth (10 against 8) and one `json.string == 0` test that Forgejo lacks.
- `skein-forge-connection/src/limits.rs:575-586` (`github_documents`) copies one into the other field by field.
- Fix: one `skein_forge::Limits` and `check(&l, depth)`, with `forgejo::DEPTH` and `github::DEPTH`.

**F3. The plan-protocol types are copied.**
- `Failure { Unseen, Seen }` is defined at `github/plans.rs:27` and `forgejo/plans.rs:37`, and the component has a third copy that it translates by hand (`skein-forge-connection/src/connection.rs:905-913`).
- `Bounds` is the same in both dialects except for GitHub's `inner` (`github/plans/plan.rs:35`, `forgejo/plans/plan.rs:102`).
- Fix: put them in a root `plan` module.

**F4. The LRU tables are hand-rolled four times.**
- `github/plans/tables.rs:55-110` has two eviction loops and `forgejo/plans/tables.rs:47-90` two more.
- The copies have drifted: GitHub's tick saturates where Forgejo's uses `expect`, and only GitHub guards a capacity of 0.
- Each copy has its own tests (`src/tests.rs:71-104`, `github/plans/tests.rs`).
- Programming-model 10.2: "Application code does not hand-roll data structures: what is missing goes into lib." The same loop appears in `skein-oauth-accounts/src/component.rs:537` and `sign_in.rs:206`.
- Fix: `skein_lib::Lru<K, V>`. The user table, (forge, id) → login, is identical in both dialects and moves to the root.

**F5. Two departures from the programming model, not recorded.**
- `github/plans/start.rs:34,38` declares `pub enum Step<T>` and `impl<T>`, used 6 times. Generic items are out (10.3), and its doc comment argues the case: "Only setup outcomes are generic".
- `github/server/app.rs:108` and `server/issues.rs:11` declare `enum Answer<'a>`, holding references in fields.
- Fix: one concrete `start::Step` with an `Outcome` enum, or one step enum per plan. The encoders can take each document directly. Otherwise record both in forge-github.status.md.

**F6. `Node` is flat for most families and nested for two.**
- `decode/walk.rs:17-104` has 85 variants. Only `Protection(P)` and `PullGraphql(PullNode)` are nested.
- So `decode.rs` `movement` (line 217) and `scalar` (line 437) each list the same family or-patterns: app 20 nodes, pulls 10, issues 9, files 9, reviews 6. `walk.rs`'s `element` lists every node a third time.
- `decode.rs` names `Node::` 187 times and `walk.rs` 293 times.
- Fix: nest each family (`Node::App(app::Node)`, ...), each with its field table in its own module as protection does. Each dispatcher then needs one arm per family. About -200 lines.

**F7. The walker knows about documents, and there are three walkers.**
- The generic walk code special-cases particular nodes:
  - `walk.rs:734-785` (`Want::Element`): GraphqlErrors with an `enqueue: bool` flag, Contexts, AppEvents, LogRows and JobSteps against `inner`, and Errors strings;
  - `walk.rs:821-840` (`demand`) and `862` (`key`): AppPermissions read as a map.
- Forgejo's walker does the same (`forgejo/decode/walk.rs:282-305`).
- The two walkers are independent copies of one mechanism: static field tables, touched/seen bitmasks and a bounded `Stack`. They are 1,321 and 559 lines.
- A third field parser, `github/server/writes.rs:114-330`, reads request bodies and special-cases the keys `query`, `variables` and `reviewers`.
- json.md 5 says its collector is "generic: an LLM dialect, a forge client or a webhook uses it the same way".
- Fix: make the tables carry these cases (`Read::Strings{field}`, `Read::Map`, a row bound) and keep one root walker for both dialects and the server bodies. Otherwise record why the collector cannot serve (it does not stream rows).

**F8. One value holds every family's state.**
- `decode.rs:34-58`: one `Decoder` carries 17 per-family states for a single document kind.
- `plans/reads.rs:85-110`: one read `Plan` carries the state of every read.
- `plans/reads.rs:27-49` mirrors `api::Read` minus three variants and maps it 1:1 at lines 114-137. Forgejo keeps `api::Read` (`forgejo/plans/reads.rs:45`).
- The mirror needs 17 exhaustive accessor matches over `Read` (`rg -c 'Read::Locate \{ \.\. \}'` over `plans/reads*`), about 340 lines, plus 105 `Stage::` references.
- The model's rule is "A state and what it holds are one value".
- Fix: `enum Plan { Checks(checks::Plan), ... }` over `api::Read`, and the same shape for `Decoder`.

**F9. The exchange-order check is written three times.**
- `plans/reads.rs:297-315`, `plans/writes.rs:242-259` and `plans/start.rs:107-120` each check the Head → Row/Done | Failed order and turn an error into `Done`.
- Each has its own `Response` enum (`reads.rs:78`, `writes.rs:33`, `start.rs:77`).
- Fix: one `plans::Order` with `admit(&Heard)` and `settle`.

**F10. `Request` is flat.**
- It has 53 variants, and 36 of them repeat `owner: Box<[u8]>, name: Box<[u8]>`.
- `Request::X { .. }` occurs 321 times in `request.rs`. `encode`'s Accept match lists 52 variants to single out `Raw`, and `target()` regroups the variants by hand into `app_target` and `items_target`.
- `Target::text` (`request.rs:441-460`) measures percent-encoding again although `forgejo::path::segment_length` exists.
- `request::unescape` (line 778) together with the server's `split` and `Query` (`server.rs:217-260`) duplicates `forgejo::path::read`.
- Fix: `Request::Repo { repo, call: RepoCall }` and `Request::App(AppCall)`, with path and query handling from the root `path` module. About -250 lines.

**F11. Nine whole-buffer tokenizer pumps.**
- Four are in step code, in two styles:
  - `decode/delivery.rs:282` slices Fill/Scan reads by hand, with `expect` and `unreachable!`;
  - `server/app.rs:70`, `server/queries.rs:120` and `server/writes.rs:339` use `Intake::meet` and return `None`.
- Five are in tests and tools: `github/tests/fixtures.rs:71,168`, `tests/suite/github_start_plans.rs:89`, `tests/common/forgejo.rs:196`, `tests/forge/github/src/runner.rs:1053`.
- `server/queries.rs:113-149` collects every token into a `List`, which is what `skein_json::parse` already does.
- `server/app.rs:58-61` recognises the viewer query by stripping the literal `{"query":"`.
- Fix: one slice-fed `Pump::next(demand) -> Option<Event>`. That is about 8 lines per site instead of about 30.

**F12. Hex and bounded-printable bytes are hand-coded.**
- Lowercase-hex decode loops appear 4 times: `github/delivery.rs:16-32`, `github/id.rs:6-20`, `forgejo/id.rs:22-40`, `marker.rs:47-63`.
- Hex encoders appear 5 times: `delivery::sign`, `id::write_commit`, `forgejo::id::write`, `marker::encode`, `guid::write`.
- There are three bounded-printable newtypes: `header::Cursor`, `header::Etag` and `api::Validator`. They have two printable checks and two copies of the 128 limit (a literal at `header.rs:54`, and `api::VALIDATOR_BYTES`). `Etag` exists only to become a `Validator` (`header.rs:141`).
- Fix: `skein_lib::bytes` hex helpers, and read the ETag straight into `api::Validator`.

**F13. One word, several concepts.**
- `Body` in github means both Forgejo's kept-or-long string and `plans::Body`: `plans/reading.rs:2` imports the first beside the second at `plans.rs:18`.
- `Refusal` means GitHub's error body (`document.rs:122`) and also input that cannot be encoded (`request.rs:321`).
- `plans/reads.rs` uses four different `Repository`s.
- `Step` names four types in github alone.
- `api.rs` says "adapter" 6 times; forge.md never does.
- The tool packages `github-conformance` and `forgejo-conformance` lack the `skein-` prefix that every other `tests/forge/*` package has.
- `tests/forge/github/src/fake.rs:24` has `bounds()` return a `Limits`.

**F14. GitHub's step tests live in three places, with no rule for which goes where.**
- The places are `src/github/tests/` (9.8k), `src/github/plans/tests.rs`, and `tests/suite/github_*.rs` (1.9k, 56 tests).
- Topics are split between them. Item decoding is in `src/github/tests/items.rs`, item plans in `tests/suite/github_items.rs`. Branch plans are in both `reads.rs:1118-1185` and `github_plans.rs:190-620`.
- Each file redeclares its helpers:

  | Helper | Copies |
  |---|---|
  | `struct Call` | 5 |
  | `bounds()` | 9 |
  | `head()` | 9 |
  | `KEY` | 7 |
  | `repository()` | 6 |
  | `begin()` | 6 |
  | `limits()` | 4 |
  | `ended` / `failed` | 7, under 5 different `unnecessary_wraps` reasons |

- The `limits()` at `github_plans.rs:28` and `github_start_plans.rs:36` are byte-identical to `fixtures.rs:23`.
- Fix: one `fixtures::Call`, and fold `tests/suite/github_*` into `src/github/tests`.

**F15. The verification gap against forge-github.md 14.**
- No GitHub decoder is tested against conformance transcripts. None are checked in, and `crates/skein-forge/tests` reads only `tests/forge/conformance`.
- None is tested against seeded or mutated answers (no `Rng` in the GitHub tests), and none is fuzzed.
- Forgejo has a mutated-transcript sweep (`tests/suite/fuzzy/forgejo.rs`).
- Fix: check in the fake run's masked transcripts, and reuse `common::mutated` for a GitHub sweep. At the least, list the gap in the status file.

**F16. Leftovers.**
- `api.rs:105-179` has 20 placeholder docs ("Read item.", "Write create issue.").
- `github/plans/tables.rs:1` cites forge.md 3.5, which is the hint; the tables are 4.2.
- `github/credential.rs:9` says "runtime 24" and `src/tests.rs:337` says "decision 25": numbering from outside the design.
- `decode/delivery.rs:73,77` has a vestigial `first(_decoder)` and a free `step(.., _limits, ..)` beside the method at line 92. Only test fixtures call them.
- `lib.rs:19` declares `pub mod marker;` after the test module.
- Two `manual_map` expects in tests (`tests/writes.rs:301`, `tests/writes/merge.rs:496`). Tests are ordinary Rust, so the expects are not needed.
- Six expect reasons in `tests/` carry history ("copied fixture imports", "copied bounded sweeps retain their original counts").
- There are 23 `u64::try_from(x.len())` and no `skein_lib::len_u64`, some with impossible-failure paths that become refusals (`.ok().ok_or(Refusal::TooLong)?`, `request.rs:1193`).
- `server.rs:67-77` splices a literal `&since=1970-01-01T00%3A00%3A00Z` into the re-encoded target.

**F17. The conformance tool carries its own HTTP client.**
- `tests/forge/github/src/wire.rs` hand-writes an HTTP/1.1 encoder and a parser with chunked decoding (about 200 lines).
- `live.rs:346-470` pumps skein-tls over a blocking `TcpStream`. skein-http is a dependency, but it is used only for the fake server's Limits.
- `runner.rs:43-79`: `Runner` has 30 public fields, including 5 `cfg(test)` fault hooks.
- `runner.rs:1030` is another percent encoder.
- Fix: if this independence from the code under test is deliberate, say so in fake-forge.md 11. Otherwise use skein-http's codec.

## Hypotheses

**1. Confirmed and refined.**
- The measurements are as in F1: 44 non-test lines, plus 20 in the GitHub fake and the Limits and Failure translators in forge-connection.
- The design does not plan it. Its neutral pieces belong at the root, beside `directory.rs`, `measure.rs`, `marker.rs` and `host.rs`.
- A per-dialect split is not warranted now:
  - forge.md 12 settles "a module of skein-forge";
  - forge-connection (77 forgejo / 70 github references) and forge-setup (88 / 36) use both dialects;
  - only the two fakes use a single dialect.
- Sizes: root 1.2k src + 0.5k tests; Forgejo 18.6k; GitHub 14.5k src + 9.8k unit tests + 1.9k suite.
- Once the root holds the neutral pieces, a split becomes mechanical and can be decided on build time alone.

**7. Refined, for A1.**
- GitHub calls `forgejo::time` 23 times (read 8, write 13, `http_date` 2 in `header.rs` for `Date` and `Retry-After`); the GitHub fake adds 3 more.
- The cost: every GitHub time and both header dates go through a module documented as "the pinned Forgejo's documents" (`forgejo.rs:1`), and a change made for Forgejo would silently change GitHub.
- RFC 3339 belongs at skein-forge's root.
- `http_date` belongs with skein-http's writer (A7's call); until then, at the root.

**15. The paths in my area.**
- `tests/forge/github` appears in forge.md 2.1 and fake-forge.md 11 but not in testing.md 6.
- It is a tool with a focused-suite test (29 tests) like `tests/forge/conformance`. Keep it there, list it in testing.md 6, and rename it `skein-forge-github-conformance`.
- `crates/skein-forge/tests` reads only Forgejo's transcripts by tree path (`tests/common/forgejo.rs:48,447,544`). These are step tests, not worlds, so they stay in the crate.
- The GitHub third has no reason to sit outside `src/`. Fold it in, as in F14.

## Tree-wide findings in this area

- **T-layout:** skein-forge has four test placements: `src/tests.rs`, `src/github/tests{.rs,/}`, `src/github/plans/tests.rs`, and `tests/{suite,common}`. It has two `#[path]` reaches: `tests/suite/main.rs:3`, and `tests/common/mod.rs:9` into `src/forgejo/tests/fixtures/admitted.rs`.
- **T-lints:**
  - the crate takes workspace lints;
  - `src/github` has 34 `#[expect]`, 13 of them `too_many_lines`;
  - `tests/` has 65 file-level expects (`disallowed_macros` 13, `disallowed_types` 12, `disallowed_methods` 12, `must_use_candidate` 10);
  - `manual_map` appears 3 times;
  - `tests/forge/github` has a hand-written `[lints]` block and a `clippy.toml` copy.
- **T-limits-shape:** both dialects use `check() -> Result<(), Problem>` and agree with each other. The delivery decoder has two step entry points (F16).
- **T-primitives:**
  - 9 tokenizer pumps;
  - UTF-8 checked through `core::str::from_utf8` under an expect (`request.rs:486`);
  - `u64::try_from(len)` 23 times against `len_u64` 0;
  - `usize::try_from(..).expect` 12 times, 11 of them "u32 fits usize";
  - percent-encoding written 3 times here, and 7 times elsewhere;
  - hex (F12).
- **T-naming:** the tools' package names (F13).
- **T-temper-draft:** forge-github.md cites temper's draft 7 times (PullFor's fork rule, batched reads, which checks count as CI, the events subscribed). The code in my area names no draft.

## Status-file cross-check

**forge-github.status.md (2026-10-29):**
- Confirmed lagging: "Not yet: 4.3, 15", batched GraphQL and conditional reads beyond the log. Only `LogPage` carries a validator.
- Contradicted: "Built: 2 to 12" includes 4.5, which the design itself calls "Designed, not built", and `plan.rs:53-77` refuses Runs, Run, Tree, Dispatch, Rerun, FastForwardOnly and rebase Update.
- Missed:
  - 13 and 16 are built in the tool (the facts, the organization selection);
  - the verification gaps of 14 (F15);
  - the github→forgejo dependency (F1);
  - the `Step<T>` and `Answer<'a>` departures (F5).

**forge.status.md:**
- "GitHub's dialect resolves none" (job attempts) is contradicted. `github/plans/reads/checks.rs:201` sets `status.job = Some(api::JobAttempt{..})` after `GET R/actions/jobs/<id>`, as forge-github.md 4.1 asks.

## Incidental

- `decode/walk.rs:723` sizes the walker's stack with the constant `DEPTH` (10), and `unknown()` (`walk.rs:1098`) refuses past 10, whatever `limits.json.depth` is. forge-connection passes the owner's depth. An answer whose skipped field nests past 10 fails as `TooDeep` even when 32 is configured. No GitHub document seen reaches 10 (a logged `workflow_run` reaches about 8), so this is a risk, not a seen failure.
- `header.rs`: `ETag` is kept untrimmed (`Etag::new(raw_value)`) while every other field is trimmed.
- Process note: I accidentally wrote one scratch file, `/tmp/x_int.txt` (a list of test names), outside the scratch directory. Nothing else was written apart from this report.

---

# A2-forge-forgejo

Scores: Cleanliness 3, Organisation 2, Taste 3, Naming 3, Primitives 2, Elegance 3, Economy 2, Complexity 3

Verdict: The Forgejo dialect keeps the subset well. Its step code has no closures, `as`, indexing or `matches!`. Its core ideas are good: a table-driven JSON walk with required-field masks, one `classify` table for 5.4, and transcript round-trips in both directions. The weak point is shape. A flat 70-variant `Request` and a 40-variant `server::Answer` are matched once in a dispatcher, then matched again in every area module, each time listing the other variants. That accounts for 1,141 lines of variant lists, 93 `unreachable!`s and 7 `too_many_lines` expects. The plans are a `Stage` plus a bag of 10-22 fields, a departure from programming-model 5.4 that no status file records. Several mechanisms exist twice: the answer measure, target measuring, JSON pumps, the event list, `Limits`. GitHub stands on four Forgejo modules that are really neutral. The tests are good, but they sit where testing.md 6 says they must not. About 1.3-1.5k of the area's 28k lines could go without losing behaviour.

Good:
- `crates/skein-forge/src/forgejo/decode/walk.rs`: static field tables, `required` bitmasks, a bounded `Stack`, unknown fields skipped as they pass. This is forge.md 5.3 written as data.
- `forgejo/plans/failure.rs:30` `classify`: one function maps (asked, status, refusal, retry) to `Error` + `Drop`, row for row with forge.md 5.4.
- `forgejo/failure.rs`: refusal messages as one (status, message) table. The client reads it (`refusal`) and the fake answers from it (`answer`, used by skein-fake-forge-protocol/src/errors.rs:14), so the two sides cannot drift.
- `forgejo/pin.rs` and `tests/pin.rs`: exactly forge.md 4.3, every architecture tested.
- `crates/skein-forge/tests/suite/forgejo_items.rs`: per case, the transcript decodes; the encoder reproduces what the tool sent; the server routes back to the same request; the server's answer decodes unchanged with a shape that is a subset of the transcript's; and mutations name their field. This set is worth copying.
- `tests/forge/conformance`: secrets registered before masking, scanned before every write, leaks named by kind.

Top recommendations:
1. **Make dispatch single-level (F1).** Nest `Request` into per-area enums (`Request::Setup(setup::Request)`, …), or call per-variant functions from the one dispatcher. Do the same for `server::Answer`. Win: about 800 lines, 93 `unreachable!`, 7 expects. Effort: M, mechanical.
2. **Make the shared pieces neutral (H1).** Move `time`, `path::{segment,file}`, `document::Body` and `decode::Malformed` to skein-forge's root, delete `id::unsigned` for `skein_lib::bytes::decimal`, and share `Limits` with a depth argument. Win: one dependency direction, 44 cross-dialect lines. Effort: S-M.
3. **Replace `plans/reads/size.rs` with `crate::measure::answer` plus a row check (F5).** Win: about 130 lines and one accounting. Effort: S.
4. **Move `crates/skein-forge/tests` to `tests/forge/dialect` and use one pump (F14, F15).** Win: 67 expects, the `#[path]` into `src/`, about 150 lines. Effort: S-M.
5. **Add walk flags, parse the target once, and keep one encoder skeleton (F4, F6, F7).** Win: about 150 lines and 11 parses per request reduced to 1. Effort: M.
6. **Let plan states hold their data (F2), or record the departure in forge.status.md.** Win: complexity, plus the 35 `if let` sites below. Effort: L.

Findings:

F1. One flat request enum, re-matched per area. `forgejo/request.rs:45` has 70 variants. `client.rs:41-113` dispatches them to 8 modules, and each module re-matches the whole enum, ending in an `unreachable!` arm that lists the rest. The per-area unreachable arms have rough names (`"start encoder dispatch"` at client/start.rs:104 and 219). Measured with `rg '^\s*\| Request::' --glob '!tests*' | wc -l`: 865 lines in 9 files, of which client.rs's real dispatch is 78. `server.rs:161-250` repeats the pattern for `Answer`: each module's `write` matches again (276 `| Answer::` lines; `server/hooks.rs:80-90` does it with `if let … else unreachable!`). Altogether 93 `unreachable!` in non-test forgejo code. 11 of the 12 non-test `#[expect]`s are `too_many_lines`, 7 of them citing exhaustive request selection (for example client/files.rs:97, client/start.rs:117). Fix: per-area enums, matched once.

F2. The plans are not state machines in the model's sense. The struct field counts are reads.rs:43-66 (22), actions.rs:28-45 (16), contents.rs:30-46 (15), writes.rs:32-46 (13) and turn.rs:26-37 (10). Each holds a `stage` beside `Option` sub-plans and counters that mean something in only some stages. turn.rs keeps three `List`s, two of them always of capacity 0 (turn.rs:41-43), and re-checks `mode` with `let Mode::Runs { .. } = self.mode else` (turn.rs:128, 148, 224, 234). This contradicts programming-model 5.4 ("no field is meaningful in some states only", "never a lifecycle spread over flags and `Option` fields"), and forge.status.md does not record it. Fix: `Stage` variants carry their own data, with `Mode` folded into them.

F3. Matches that fall out of the lint's sight. The forgejo step code has 22 `if let` and 13 `let … else` on its own enums (`rg '(if|while) let ' | grep -v Some/Ok/Err`), for example client/setup.rs:231-287 and server/writes.rs:377-574. It has 3 tuple scrutinees with a `_` arm (client/setup.rs:86, client/hooks.rs:120, client/repositories.rs:43) and 4 `Move::End(_)` nested wildcards (client/hooks.rs:139, client/repositories.rs:51, client/writes.rs:368, client/create_file.rs:93). `classify` builds `read` from `==` comparisons on `Asked` (plans/failure.rs:31). All of these break 5.4 / 10.3. Most disappear with F1 and F2.

F4. The table-driven walk has exceptions hard-coded by kind and by key:
- arrays of strings are found by comparing five kinds (walk.rs:319-336);
- null is tolerated for the keys `statuses`, `creator` and `tree` by name (walk.rs:450-452);
- the body cap is chosen by kind (walk.rs:412);
- dispatch inputs (walk.rs:396) and skipped comments (walk.rs:392, 475) are special-cased.
`server/writes.rs:360, 387, 410` repeat the Logins/Scopes/Events test three times. `Value::Input(_) => unreachable!("dispatch input table")` appears 12 times in client decoders, because one `Kind`/`Value` serves answers, request bodies and deliveries. Fix: field flags (`Read::Texts`, `nullable`, `Read::Message`) in the tables, and a server-only input move.

F5. A second answer measure. `plans/reads/size.rs` (167 lines) re-implements `crate::measure::answer` (measure.rs:151) with the same constants. Its only addition is a per-list row limit, and it delegates to `measure` for Runs, Run and Tree anyway (size.rs:110-121). It has already drifted: it omits `pull.dequeued` (measure.rs:260) and refuses `Directory`, which measure counts. GitHub's plans use `measure` directly (github/plans/reads/done.rs:204). Fix: `measure::answer` plus a `rows_within(answer, limit)` check.

F6. Three ways to size a target:
- `Target`, a measure/write pair defined in client/items.rs:12-66 but used by 7 modules, each repeating the measure → check → write skeleton (8 copies of `measure.length > usize::try_from(limits.target_bytes)`);
- hand-counted literal lengths in client/start.rs:29-35 (`User => 12`, `ApiSettings => 20`, `21 +`, `15 +`, `25 +`) that must match the strings written in `target()`;
- `path::segment_length` (path.rs:49), which repeats `segment`'s unreserved predicate (path.rs:23).
Fix: one `encode` in client.rs over one `target(request, &mut Target)` dispatch. A counting `skein_lib::Writer` would serve every measured concat (T-primitives).

F7. The router re-parses the target. `server.rs:43-79` is two nested `match … Err(NotFound) =>` ladders over 11 route functions, each calling `path::read` again (11 calls). Fix: parse once and pass `&Target` down a flat sequence.

F8. Module boundaries do not follow the concepts:
- client/files.rs:113-172 encodes Actions requests although client/actions.rs exists;
- client/reviews.rs:88-123 encodes Branch, Protection, Collaborators, Permission and Pulls although client/branches.rs exists;
- `decode/walk.rs` imports kinds from client/, server/ and delivery while client/ imports walk, a cycle;
- the item field tables live in walk.rs:82-145, while every other kind's tables are in client/*.rs.
The Forgejo dialect also splits its sides differently from GitHub's (`client/` versus GitHub's `request.rs` + `decode/`). Fix: GitHub's layout.

F9. GitHub depends on Forgejo modules. 44 lines in 26 non-test GitHub files name `crate::forgejo`. `github/decode.rs:7` re-exports `forgejo::decode::Malformed` as GitHub's own. 13 files use `forgejo::time` and `request.rs:455-457` uses `forgejo::path`. `forgejo/id.rs:17` `unsigned` is only `bytes::decimal` "under the name the forge readers call" (about 15 Forgejo, 50 GitHub and 15 downstream calls). `forgejo/limits.rs` and `github/limits.rs` have identical structs and differ only in depth. None of this is designed: forge.md 4.1 and forge-github.md 2 give each dialect its own "names, times and ids".

F10. Naming:
- `forgejo::client` ("the engine's side", client.rs:1; "for the engine", delivery.rs:19) collides with skein-forge-client, and "engine" is not the design's word ("component").
- `plans/turn.rs` is temper's word for Actions runs and trees, and forge.md 9 calls the same group "`landings` facts".
- Within forgejo there are 5 `Step` types (decode, plan, start, delivery, and a job's `document::Step`), 2 `Body` (document.rs:6 and plans.rs:23), 2 `Refusal`, 2 `Target` and 2 `Read`.
- `pub enum Drop` (plans/failure.rs:20) shadows the prelude trait.
- The package names `forgejo-conformance` / `github-conformance` break "named by its path" (siblings are `skein-forge-*-world`).
- "conformance" names three things: testing/skein-conformance, testing/skein-forge-conformance, and the tools.

F11. Vestigial code:
- `time::RFC3339_BYTES` (time.rs:3) is never used;
- `Body::read` (document.rs:12) is called only by tests;
- `client::first`, `client::delivery::first` and `server::body_first` (client.rs:226, client/delivery.rs:137, server/writes.rs:343) ignore their argument and return `Request::Next`;
- `reads::Plan::begin` takes `_word` unused (reads.rs:95);
- the tool's shims re-export the shared crate (conformance/src/json.rs: "the Forgejo tool retains this module path"; mask.rs:1; facts.rs:7), and write.rs wraps `Secrets` in a newtype with two `Deref` impls only to add `from_instance`;
- conformance/src/lib.rs:50 writes facts.txt, then lines 51-54 overwrite it.

F12. Stale references and history words:
- deliveries cited as "forge.md, section 5" (delivery.rs:1, 54, 68, 97; plans/hint.rs:1), but they are section 7;
- "3.4, Unavailable is provable…" (plans.rs:34), which is 5.4;
- tables as "3.5" (plans/tables.rs:1), which is 4.2;
- forgejo.rs:1 cites "sections 2, 3 and 9", but the dialect is 4 and 5;
- "(runtime 20)" (credential.rs:10) points at nothing in skein's docs;
- "(protocol/forge.md, 10)" (conformance/src/lib.rs:1) names no existing file;
- the README says `VERSION` lives in `skein_throwaway_forgejo` (only `RUNNER_VERSION` is re-exported, lib.rs:136) and uses "after the receipt succeeds" (README:23, an undefined term);
- 10 history phrases, for example "copied fixture imports", "moved family tables" (tests/common/generated.rs:1), "preserved comment stories" (forgejo/tests/fixtures.rs:18) and "copied bounded sweeps".

F13. Literal lists copied. The 22 subscribed events are written out 5 more times beyond `delivery::SUBSCRIBED` and the deliberate contract test (plans/tests/edges.rs:88): conformance deliveries.rs:26, hooks.rs:14 and repo_setup.rs:24, and tests/common/forgejo.rs:655 and tests/suite/fixtures.rs:108. conformance hooks.rs:38-62 and repo_setup.rs:47-71 are the same hook-body builder. `const fn field` is defined 9 times in forgejo (client/*.rs ×7, server/writes.rs:49, walk.rs:61). Fix: import `SUBSCRIBED`; one builder; `Field::required` / `Field::optional` in walk.

F14. The tests sit where testing.md 6 forbids. `crates/skein-forge/tests` holds 6.65k lines, 4.7k of them Forgejo's. They read `tests/forge/conformance/transcripts` by tree path (tests/common/forgejo.rs:48, 447, 544) and reach into `src` with `#[path = "../../src/forgejo/tests/fixtures/admitted.rs"]` (tests/common/mod.rs:9), which forces `#[expect(dead_code)]` at src/forgejo/tests/fixtures.rs:15. 67 of the directory's 70 `#[expect]`s exist only because these files compile under step lints, and 19 files carry a redundant `#![cfg(test)]`. Fix: a `tests/forge/dialect` crate named by its path.

F15. Hand-written JSON pumps. forgejo/tests/fixtures.rs pumps the tokenizer three times (`capped` :26, `Pump` :105, `request` :147), and tests/common/forgejo.rs:196 a fourth time. testing/skein-forge-conformance/src/json.rs:15-45 and skein-throwaway-forgejo/src/instance.rs:318-350 hand-drive the collector for whole documents, where `skein_json::parse` (parse.rs:25) does the job. Fix: one test `Pump`; `skein_json::parse` in both testing crates.

F16. Undesigned or hand-made pieces. testing/skein-forge-conformance is named in no design: not in forge.md 2.1, not in testing.md 6, not in fake-forge.md. The tool's `--admin-scopes` and `--create-repository` modes, and the README's "merge those named facts into facts.txt" by hand, are not in fake-forge.md 7. facts.txt:106-107 ("declared; recording requires the director's run") is a line no code writes, so the committed record cannot be reproduced. "director" is pipeline vocabulary.

F17. Test seams in the tool's own code. conformance/src/deliveries.rs:73-120 puts `#[cfg(test)]` fields and methods (`worker`, `start_test`, `next_test`, `finish`) into the production `Listener`. The tool also uses three test conventions: src/tests.rs, an inline `mod tests` (repo_setup.rs:264) and inline `#[cfg(test)] #[test]` (facts.rs:611).

Hypotheses:
1. Confirmed, and refined. The dependency is real (F9) and undesigned: forge.md 4.1 and forge-github.md 2 each give their dialect its own names, times and ids, and neither status file records the sharing. The neutral pieces should go to skein-forge's root, joining `marker`, `measure`, `directory` and `host`, which are already neutral there:
   - `time` (RFC 3339), `path::{segment,file}`, `document::Body`, `decode::Malformed`, and one `Limits`;
   - `id::unsigned` deleted in favour of lib's `decimal`.
   A per-dialect crate split is refuted. forge.md 12 settles GitHub as a module of skein-forge, the component links both dialects, and a split would only move about 1k neutral lines into a third crate.
2. A1 territory. In this area it shows up only as T-limits-shape: both dialects use `check(&Limits) -> Result<(), Problem>` with variants named differently (`TokenizerString` vs `String`), against the model's `valid -> bool`.
7. Confirmed. `http_date` (forgejo/time.rs:125) parses the IMF-fixdate that skein-http writes (server/response.rs:255). GitHub's `Date` and `Retry-After` go through it (github/header.rs:150, 156), so it belongs in skein-http beside its writer. The calendar arithmetic itself exists twice, with different algorithms: a year loop up to 2554 (time.rs:89) and Hinnant's `civil_from_days` (response.rs). A `Wall` ↔ civil-date helper in lib would serve both, plus ui's phrases.rs. Keeping RFC 3339 in a Forgejo module costs GitHub 13 files importing `crate::forgejo::time`.
15. Refined. tests/forge/conformance as "a tool outside the suites" is designed (forge.md 2.1 and 11, fake-forge.md 7, testing.md 6), so its place is no departure, and testing/ is for shared libraries, which it is not. Its package name breaks the path rule (F10). The real departure is the crate's own `tests/` directory (F14). The dialect's integration tests should become a crate under `tests/forge/` that reads its sibling's transcripts.

Tree-wide findings in this area:
- **T-layout:** the skein-forge crate has a `tests/` directory. skein-throwaway-forgejo has `tests/suite`, plus `tests/support` holding a `[[bin]]` (Cargo.toml:33). The step tests use `tests.rs` + `tests/` (forgejo/tests.rs, plans/tests.rs), while the throwaway uses `src/tests.rs` + `src/http/host/tests.rs`.
- **T-lints:** the three tool/testing crates carry byte-identical hand `[lints]` blocks (md5 equal). Their clippy.toml files say "the harnesses under tests/" even in testing/. testing.md 6 says testing/ crates take the step lints.
- **T-primitives:** the pumps (F15), the calendar (H7), the measured concat (F6), and 9 copies of `usize::try_from(limits.target_bytes).expect("u32 fits")`.
- **T-naming:** "conformance" (F10), and pipeline words ("director", "receipt").

Status-file cross-check:
- Contradicted: forge.status "Not yet: 4.3: binary digests". pin.rs:6-36 has all four digests and `digest`, tested in tests/pin.rs and used by temper (temper-forge-setup/src/fetch.rs:66).
- Missed:
  - the 5.4 departure (F2, F3);
  - forge.md 11's "a fuzz target per decoder": none exists, and seeded mutation sweeps (tests/suite/fuzzy/forgejo.rs) stand in;
  - forge.md 9 still lists "to record: a dispatch's inputs reaching the run; a job's rerun answering 204, and 400", while facts.txt:277 and 285-287 record them as `holds`;
  - the GitHub → Forgejo module dependency;
  - testing/skein-forge-conformance and the two extra tool modes (F16);
  - the skein-forge `tests/` directory against testing.md 6.
- Confirmed lagging: "9: fact for an untaken job". forge.md 9 still lists it under "What no run records yet".

Incidental:
- path.rs:94 caps a target at 12 segments (`List::with_capacity` is a cap, skein-lib list.rs:23). So the fake's contents and raw routes (server/files.rs:55) refuse, as `Malformed`, any file nested more than six directories deep.
- size.rs and measure.rs disagree on `Directory` and `dequeued` (F5); this is harmless today.
- client/start.rs:29-35: hand-counted lengths can drift from the literals written in `target()`.
- conformance write.rs `from_instance` guesses a secret's kind from its length (40 → ApiToken, 64 → Password), so a leak can be reported under the wrong kind.
- throwaway binary.rs reads the whole Forgejo binary (about 122 MB) into memory to hash it.
- server/writes.rs:382 uses `Request::User` as the placeholder for `mem::replace`. A decoder reused after `Done` would hand out `User`; only the `terminal` flag stops that.

---

# A3-forge-connection

# A3 forge-connection: code-quality review

Area: `crates/skein-forge-connection` (about 7.2k lines of step code; 12.8k of step tests plus 343 inline), `tests/forge/machine` (8.4k), `tests/forge/protocol` (3.0k). Static review of snapshot `4a6683c72`. Scratch: two fixture excerpts in `scratch/A3/` for a diff.

## Scores

Cleanliness 3, Organisation 2, Taste 3, Naming 3, Primitives 2, Elegance 2, Economy 2, Complexity 2.

## Verdict

The boundary (`boundary.rs`), the start's pure transition (`start.rs`), the JWT cache (`app.rs`) and the limit refusals are careful and well documented, and test names read as behaviours. Below the boundary the component is built from parallel per-forge lists indexed by position, with dialects told apart by two-variant enums and `unreachable!` at about 22 sites. State lives in flags and `Option`s beside thin tags, which departs from programming-model 5.4 and from the design's tables in 8.1 and 8.4. The tests are larger than the code three times over and are spread across tiers. 487 of the 564 "step tests" drive the whole component over scripted HTTP through a harness of their own, so they are really a machine world. The area holds seven separate io-playing harnesses and four adapters that turn the fake GitHub service into HTTP, although `skein-fake-forge-github` already ships a step-machine HTTP server. Runtime `Add`/`Remove` is designed nowhere in skein. I estimate 3 to 4k lines could go with no loss of behaviour or confidence.

## Good

- `src/start.rs:23`: `heard` is a pure transition with a total match, and `src/tests.rs:33` tests it as a table whose rows carry data. Copy this.
- `src/app.rs`: a small JWT cache with a reuse rule tied to `deadline`, and eight focused tests (`app/tests.rs`), including Debug redaction.
- `src/boundary.rs`: each variant's doc states its contract (one terminal, at most one Connected/Stopped per generation).
- `tests/forge/machine/src/pipe.rs` and `tests/forge/protocol/src/referee.rs`: costs are checked against first-byte writes counted independently on the stream. The credential-needle check covers the Debug and escaped forms (`referee.rs:85-99`).
- `tests/forge/machine/src/tls.rs`: the fault sweep cuts at boundaries in the gate and at every byte in the exhaustive suite, as design 10 records.

## Top recommendations

1. **One HTTP face for the fake GitHub service.** Point the four adapters at `skein-fake-forge-github::server` and move the fixture App, accounts and repositories into the fake. Win: about 1.2 to 1.5k lines, and a single place for the Outcome loop. Effort: medium.
2. **One `ForgeEntry` per forge.** Hold settings, start, dialect state, own token and own plan in one struct instead of five parallel lists. Win: about 160 `.get(at).expect` chains and 38 double expects, 39 `at: u32` parameters, and `remove` shrinking from 74 lines to about 25. Effort: medium.
3. **Move the Harness-driven step tests into the machine world, with one io harness.** Win: about 2k lines and one tier per behaviour. Effort: high.
4. **States per model 5.4.** Make the connection phase and the webhook connection hold their data, and set deadlines in one `follow` function. Win: 15 deadline sites become one, and about 30 flags go. Effort: medium-high.
5. **Design runtime membership in forge-connection.md and record the departures in the status file** (F11, F12). Effort: low.
6. **Neutral primitives.** Make one `Failure`, add an `is_raw` on skein-forge's requests, use lib's `Tail` and `Buffer`, and fold `Unusable`'s mirror of `Value`. Win: about 250 lines and 3 concepts. Effort: low-medium.

## Findings

**F1. Four HTTP adapters for one fake, beside the fake's own server.** The fake service's `Outcome::{Reply,Admit,Call}` loop is written four times:
- `src/tests/github_files/fake.rs:200` (396 lines; it hand-encodes the response head at `:212-259` with measured concatenation and three `.unwrap()`);
- `tests/forge/machine/src/github.rs:238` (605 lines; it hand-parses requests at `:348` and hand-writes heads at `:233-290` and `:384`);
- `tests/forge/machine/src/joint/github.rs:152` (375 lines);
- `tests/forge/protocol/src/github.rs:357` (550 lines).

Meanwhile `crates/skein-fake-forge-github/src/server.rs` (792 lines) is that face as a step machine. Fixture App, accounts and repositories are built in three places: `machine/src/github.rs:321-345,408,457`, `protocol/src/github_fixture.rs`, and the step-test fake. `github_fixture.rs:30` admits the copy: "The machine world's service and domain are private". Testing-strategy 1 says a fake is the same at every tier and only its face changes.
Fix: drive `server::Server` in every world, and keep scenarios in the fake's own terms (testing-strategy 7).

**F2. Five parallel per-forge lists indexed by position.** `Component` holds `settings.forges`, `starts`, `github: List<Option<Box<GitHub>>>`, `own_tokens` and `own_plans` (`component.rs:105-120`). `rg -o` counts 160 `get(at)` accesses (35, 26, 55, 14, 30), 38 `expect("forge slot").as_mut().expect` pairs (`component/github.rs` 30, `puller.rs` 8), 39 methods taking `at: u32`, and 9 hand loops that look a forge's position up by number. `runtime.rs:87-115` swap-removes across all five lists in 30 lines.
Fix: use `List<ForgeEntry>` with `enum Dialect { Forgejo { plan }, GitHub(Box<GitHub>) }`, and pass `&mut ForgeEntry`.

**F3. The dialect is a runtime tag checked by `unreachable!`.** Eight two-variant enums wrap the dialects: `CallPlan`, `OwnPlan`, and in `connection.rs` `Exchange`, `Heard`, `Refusal`, `Request`, `Decoder`, `Body`. The `Start` enum mixes Forgejo's Checking and Settings with GitHub's Listing, Minting and Identity. That gives 28 `unreachable!` sites whose messages name a dialect; for example `calls.rs:218,282,306,348,385,446` and `component/github.rs:107,162,246,477,507`.
Fix: with F2, put each dialect's plans in its `Dialect` variant, and split `Start` into a public `Start` (Unconnected, Starting, Ready, Stopped) plus dialect start enums.

**F4. Three `Failure` enums and two dialect lookup tables.** `connection::Failure` (`connection.rs:56`) mirrors `forgejo::plans::Failure` and `github::plans::Failure`, which are identical, and `connection.rs:897-915` maps between them. `raw_request` (77 lines, `:1027`) and `github_raw_request` (55 lines, `:1104`) are exhaustive tables of a property of skein-forge's request types.
Fix: make one neutral `plans::Failure` in skein-forge, and give both dialects `Request::is_raw()` there (or have the decoder say so). This touches H1: the neutral pieces belong at skein-forge's root.

**F5. Hand containers duplicate lib.** `grow::Tail` (`grow.rs:63-104`) re-implements `skein_lib::Tail` (`skein-lib/src/tail.rs`) and adds only `dropped`. `grow::Grow` is a capped, doubling byte list beside lib's `Buffer`. The module is `pub` with no user outside the crate (`rg` finds nothing in jig, smith or temper). Model 10.2 says "what is missing goes into lib."
Fix: use lib's `Tail` plus a `dropped` count, and add growth to `Buffer` or `List` in lib if it is needed.

**F6. Step code holds state in flags beside thin tags (model 5.4; design 8.1 and 8.4).**
- `Connection` has 33 fields: `Phase` is a `Copy` tag, while `request`, `receive`, `done`, `upload`, `deadline`, `written`, `closing`, `draining`, `graceful` and `membership` sit beside it (`connection.rs:122-156`).
- `Hooked` has 21 fields, among them `malformed`, `ended`, `done`, `hint_state`, a `forge: 0` sentinel and three `Option`s (`hooks/connection.rs:43-63`). Its `pending()` is an eight-way boolean (`:242-251`), where design 8.4 has Head, Body, Answering and Closing.
- `installations::GitHub` has 24 fields under `#[expect(clippy::struct_excessive_bools)]` with 7 bools (`installations.rs:351-378`).
- Design 8.1 says the deadline is "set in one place after each transition". `rg` finds 15 `deadline =` sites across five files and 12 `phase =` sites in `connection.rs`.

Fix: give each state a variant that holds its data, and add one `follow(conn)` that sets demand and deadline.

**F7. "Step tests" are mostly a machine world, in three layouts.**
- 487 of 564 crate tests sit in the 21 files that use `tests/fixtures.rs::Harness` (`:116`), which plays io over `skein_world::stream::Wire` and answers raw HTTP. Testing-strategy 2.1 defines step tests as one step fed events.
- Three conventions live in one crate: `src/tests.rs` with `src/tests/` (20 modules), sibling `foo/tests.rs` for app, component, connection, installations and limits, and an inline `mod tests` in `component/github.rs:1005-1347`.
- Topics are split. Limits are tested in `tests.rs` (99 references to `Unusable`, admission or check) and again in `limits/tests.rs`. Installations are tested in four places, and file names repeat across tiers (`github_renewal`, `github_installation_loss`, `github_unknown_repository`, `github_start`, `github_calls`).
- Heap ceilings run in 5 step-test files under a global `Counting` allocator (`tests.rs:28`). Testing.md 6 puts memory in `tests/suite/memory.rs`.

Fix: keep only pure-function tests in the crate, in one layout, and move the rest into the machine world.

**F8. Seven io-playing harnesses.** Each writes its own `fn io(&mut self, skein_io::Request)`:
- `machine/src/machine.rs:602`, a `Machine` of about 45 fields;
- `machine/src/joint.rs:433`;
- `machine/src/joint/github.rs:198`;
- `machine/src/hooks.rs:222`;
- `machine/src/memory.rs:439`;
- `protocol/src/github.rs:441`;
- the step `Harness`.

The protocol world unites two of them through a `macro_rules! dispatch` over identically named fields (`protocol/src/driver.rs:12-183`). Also, `joint.rs` runs the component against the real fake protocol layer, which is the protocol tier (2.5), inside the machine world.
Fix: one forge-world io (sockets, listener, pipe per connection) parameterised by a peer.

**F9. Duplicated fixtures and bare sizes.** `machine/src/fixtures.rs:6-114` repeats `src/tests/fixtures.rs:23-115` (limits, shared, settings) almost exactly, but writes `body_bytes: 1042` and `op_bytes: 512` bare where the crate derives `BODY_ROOM = 2 * OP_BYTES + 3 + 15`. `machine/tests/suite/memory.rs:8,14,20` carry rows of 15 unlabeled numbers (`[2, 2, 2, 2, 8, 4, 0, 0, 2, 8, 8, 0, 0, 65408, 4]`) and branch on row data (`if forges.github > 0`). `tests/runtime.rs:34` asserts `Some(12)` bare. This breaches testing-strategy 10 on named sizes and on rows carrying data.
Fix: name the saturation fields as a struct, and derive sizes from the code's constants.

**F10. Dead and leftover code.**
- `limits::Forges` (`limits.rs:85-104`) is computed at `:270` and passed as `_forges`, unused, to `capacity_bound` (`:282`). Its only real consumer is the machine world's memory scenarios, so move it there.
- Six `forges > 0` branches are always true because `nonzero` refuses 0 (`component.rs:158-163`, `limits.rs:287,290`).
- Unused parameters: `Connection::cut(_env)` (`connection.rs:989`) and `github_error(_error)` (`component/github.rs:414`).
- `GitHub::new` sets `said_origins` to capacity 0, and both callers overwrite it (`component.rs:143`, `runtime.rs:33`).
- `use` items appear mid-file (`limits.rs:194`, `connection.rs:24`, `tests.rs:75`).
- `machine/Cargo.toml` has an empty `[dev-dependencies]`.
- Out of the subset (model 10.3): `if let` or `let … else` on own enums at `component.rs:240,709,867`, `connection.rs:661` and `settings.rs:92`, and a hand `impl Debug for Connection` (`connection.rs:1004`).

**F11. Runtime membership is undesigned, and the test build behaves differently.** `Request::Add` and `Request::Remove` (`boundary.rs:60-64`) and `component/runtime.rs` cite temper's draft. That draft designs only temper's act (`RemoveForge`, sections 3 and 4), and its section 10 amends nothing in skein. On a duplicate number, `runtime.rs:12-13` calls a `#[cfg(test)]` `debug_assert_ne!` under `#[expect(disallowed_macros)]`. So tests panic (`tests/runtime.rs:218`, `a_duplicate_number_asserts_only_in_tests`), while production does what the documentation says, "ignored", and that path is untested. A refused `Add` is silent, and `remove` rewrites the owner's `up` queue (`:53-67`).
Fix: write 3.6 in forge-connection.md (add, remove, refusal event, queued-event rule), drop the test-only assertion, and test the production path.

**F12. Forgejo's own connection is drawn from the calls' pool.** Design 7.2 says "Each forge has one more, its own". In the code, `drive_forgejo` opens the own connection only while `call_connections() < connections` (`component.rs:647-653`), and `call_connections` reserves only GitHub's own (`:763-770`). The test at `component/tests.rs:28-53` pins this ("Three Forgejo starts contend for the two call slots"). The reserved-slot loop is also written twice (`component.rs:764`, `calls.rs:156`).
Fix: either reserve a slot per forge or record the departure.

**F13. Naming.**
- `Waiting` names the call entity (`component.rs:46`; the design says "call") and also `Start::Waiting`, `Tokens::Waiting` and `Work::Waiting`.
- Four `Heard` enums and three `Exchange` types are in play, with `Exchange as Pending` (`component/github.rs:16`) clashing with the fake's `Pending`.
- One struct mixes two vocabularies: `Row` has `api`, `api_expires` and `git_expires` beside `full` and `contents` clocks (`installations.rs:11-25`), while the design says "API token" and "git token".
- `Hooks` and `Hooked` stand for the design's "listener" and "webhook connection".
- `limits/tests.rs:337-399` names tests `valid_*` for `check`.

**F14. Two shapes for the same thing in the limit checks.** `Unusable` has 30 variants and 20 `Unusable::Zero` sites, whereas 7.4 says "each refusal names both limits". `Credential`, `Token`, `Key`, `User` and `Secret` `{ forge }` mirror `Value` through a 7-line match (`limits.rs:387-393`). `check` mixes `valid() -> bool` (`:211,271-273`) with `check() -> Result` (`:237,304`), which is T-limits-shape. The request-head sum is written twice (`:417-436`, `:473-482`).
Fix: `Unusable::Access { forge, value }`, plus one `head_bound` function.

**F15. Stale citations and pipeline words.**
- `limits.rs:68` cites section "12, Shared limits" (it is 7.3).
- `limits.rs:132` cites temper's `protocol/configuration.md`.
- `limits.rs:30,32,164` refer to "06's".
- `hooks.rs:1` and `hooks/connection.rs:1` cite section 5 for webhooks (it is 6).
- `component/tests.rs:4` cites 6.1 for the token prefix (it is 2).
- Four module headers carry the same boilerplate cite "sections 2, 3.5, 7 and 8".
- "landed" appears three times: `protocol/src/github.rs:1`, `tests/github_logs.rs:309`, `tests/github_installations.rs:1`.

## Hypotheses

**H2: confirmed.** forge-connection.md has no Add or Remove: neither the 2.1 table, nor 3.1 to 3.5, nor 7.4 or 9 mentions them, and 9 says the values come at start. `rg 'playground-reset'` finds 5 cites in this area (`boundary.rs:60,63`, `runtime.rs:1`, `connection.rs:115`, `tests/runtime.rs:1`). Neither forge-connection.status.md nor forge-setup.status.md records it. The temper draft (sections 3 and 4) designs temper's act, not the component's semantics: ignore on a duplicate or overflow, `Unavailable` for live calls, a `Membership::Removed` abort, and queued events dropped. Temper depends on it (`temper-service/src/forges.rs:165,203`, `lib.rs:265`, `forge/gone.rs:27`).

**H1 (as seen here):** the component pays for the dialect split. See F3 and F4: three identical `Failure` enums, parallel `Heard` and `Body`, and `forgejo::time::http_date` used by `connection.rs:702`.

**H15 (as seen here):** the machine world reads `../conformance/transcripts` by tree path (`machine/src/machine.rs:847`), against testing.md 6, which puts transcripts "beside its tests".

## How the tree-wide findings show up here

- **T-layout:** three step-test conventions in one crate (F7). `machine/tests/exhaustive_tls.rs` sits outside `tests/suite`; design 10 records it. The protocol lib has no `[lib] test = false`, so `referee.rs:168` builds a lib test binary, and that test branches on its row index (`match sink`).
- **T-worlds-as-kits:** the machine world is a kit for the protocol world (`fixtures`, `joint::Joint`, `pipe::Pipe`) and for `tests/forge/setup` (`github::Fake`, at `setup/src/host.rs:4`). It in turn depends on `skein-tls-world`.
- **T-lints:** hand `[lints]` blocks in two variants (machine, protocol). There are 21 `#[expect]`s, and `manual_map` appears 6 times in 4 wordings.
- **T-primitives:**
  - 13 `Env` rewraps, 10 of them in step code;
  - 45 `usize::try_from(..).expect`;
  - `u64::try_from(len)` 6 times against `len_u64` 12 times;
  - 4 `skein_io::Limits {` literals and 3 `fn limits()`;
  - a fourth calendar, `imf()` (Hinnant's algorithm), at `machine/src/github.rs:438`;
  - token namespaces asserted in two files (`component.rs:787`, `hooks.rs:19,157`), with world listeners at `1<<57` (`joint.rs:13`) and `Token(90)` (`machine/src/hooks.rs`);
  - `Answered` copied field for field from the client's `Event::Answered` (`boundary.rs:39`).
- **T-naming:** one "actual" (`boundary.rs:107`) and an `observed` variable (`referee.rs:67`).
- **T-temper-draft:** see H2.

## Status-file cross-check

**Confirmed:** the listener is plaintext and loopback only (`limits.rs:438-442`). Heap checks and the machine and protocol stories exist.

**Missed:**
- runtime Add/Remove (F11);
- `Request::Abort`, which is absent from forge-connection.md's 2.1 and 8.5. It ends calls "as at its deadline" (`boundary.rs:87-91`), while model 5.2 says "as cancelled";
- Forgejo's own connection sharing the pool (F12);
- 8.1's single deadline site and 8.4's states (F6);
- heap tests in step tests, and transcripts read from `../conformance`.

**Vague:** "10: remaining GitHub cases, details unknown" cannot be checked. The machine suite has a file for each GitHub case design 10 names (renewal, installations, unknown repository, delivery log, job log), so the status should name what is missing.

## Incidental

- `component.rs:513-523`: an io `Closed` under a live holder ends the call `Timeout` or `Unavailable` by the connection's `written` and skips `plan.error`. `expire` (`calls.rs:18-23`) uses the call's `cost` and `plan.error`. If this path is reachable for a multi-step Forgejo write, an earlier written step could be reported `Unavailable` (forge.md 5.4). This is a risk, not confirmed.
- `component.rs:237-246`: while Closing, a `Call` is dropped with no terminal. Model 5.2 says to refuse new work "with a refusal that says it is closed".
- Long bodies are counted in `Diagnostics` for Forgejo calls only (`calls.rs:391`). The GitHub path counts only malformed answers (`github_calls.rs:398`). No consumer downstream reads `diagnostics()`.
- `drive_calls` allocates a `List` of every call on each entry point (`calls.rs:51`), and `Request::Offer` rescans `held` once per answer (`component.rs:294-308`).

---

# A4-forge-setup-client

# A4: forge-setup and forge-client

Paths are relative to `skein/`. Line counts come from `find … | xargs cat | wc -l`. Non-test means excluding `src/tests/`, `tests.rs` and `*_tests.rs`.

| Unit | Non-test | Test |
|---|---|---|
| skein-forge-setup | 7,142 | 10,855 (516 `#[test]`) |
| skein-forge-client | 5,168 | 5,551 (246) |
| tests/forge/setup | 2,742 | 3,193 |
| tests/forge/client | 1,808 | 1,679 |

## Scores

| # | Criterion | Score | Why |
|---|---|---|---|
| 1 | Cleanliness | 3 | Few lint exceptions in step code (4 in setup, 6 in client). Against that: 7 stale section citations in the client, 6 citations of temper's draft, a redundant `#[path]`, and a halting trick. |
| 2 | Organisation | 2 | The setup crate is lopsided (Forgejo plans at the root, GitHub's in `github/`, the shared vocabulary inside Forgejo's `reconcile.rs`) and uses four test conventions. The client alone would earn a 4. |
| 3 | Taste | 3 | |
| 4 | Naming | 3 | |
| 5 | Primitives | 2 | The admin step is written 5 times, `stop` has 6 shapes, the HTTP/TLS/JSON stack and `Transport` duplicate forge-connection's, there are two identical `Progress` types and 7 hand-written reply maps. |
| 6 | Elegance | 3 | |
| 7 | Economy | 2 | About 430 lines of setup step code, about 760 lines of world harness and about 100 test lines could go. |
| 8 | Complexity | 3 | |

## Verdict

**skein-forge-client is the better half.** It is a faithful child domain: `valid(&Limits) -> bool` exactly as programming-model.md 6.3 asks, a `recovery` table whose arms carry their reasons, pinned measure widths, one test convention, and a world with a durable parent and a referee. Its debts are small:
- stale section citations;
- facts that are dropped inside the step;
- done-markers scattered through `domain.rs`;
- a few `let … else` forms that are outside the subset.

**skein-forge-setup works, but grew by accretion.**
- Twelve plans each re-derive the same pieces: the admin check, the page-cap read, the stop and the halt.
- The neutral vocabulary sits in Forgejo's `reconcile.rs`.
- GitHub's picker is fused into Forgejo's `Reachable`.
- The host re-implements forge-connection's transport stack.
- Its tests are mounted four ways, and include heap ceilings that the design places in the world.
- Its world runs the Forgejo stories through a test-side exchange rather than the host component that the design says it plays.

Three departures are not in the status file:
- runtime `Add`/`Remove` and `Abort` (designed only in temper's draft);
- "at most 4 events" (design 6.3), which the code does not hold;
- the bypassed host in the world's Forgejo stories.

## Good

- **`crates/skein-forge-client/src/limits.rs:45-80`** is the model's `valid` shape: zero and order checks, then `capacity_bound` with checked arithmetic.
- **`crates/skein-forge-client/src/measure.rs:6-18`** pins stored widths as named constants, with `const` asserts that each covers its type, so prices never move with a type's size.
- **`crates/skein-forge-client/src/boundary.rs:46-81`**: `recovery` puts design 7.3's table into one exhaustive match, with each arm's reason beside it.
- **Module docs that say what each module never knows** (`outbox.rs:1-9`, `keep.rs:1-7`, `facts.rs:1-4`).
- **`crates/skein-forge-setup/src/access.rs`** is the right shape: one sub-machine (4.5 steps 2 to 4) shared by reconcile, adopt and create. It is the model for the missing admin sub-machine.
- **`crates/skein-forge-setup/src/item.rs`** has one total decoder per item, mirroring 6.4's table, plus `fits`.
- **`given.rs`'s `Place`/`Refused`** names the place, never the value.
- **`Secret` carries every credential**, and the tests check that `Debug` redacts it.
- **Test names** average 8.9 words across 485 setup tests and name a behaviour.

## Top recommendations

1. **One vocabulary module and a `forgejo/` directory for setup** (F1).
   - Win: names `plan::{Need, Event, Reply, Outcome, Failure, Stage, Why}`; removes 8 type-name collisions; puts GitHub's picker in `github/`.
   - Effort: medium, mostly moves. Downstream renames touch 7 temper files.
2. **Extract the plans' shared pieces** (F2, F3): an `admin` sub-machine, the page-cap read, the paged token delete, one `stop`, one `halt`, one `Progress`, and a per-dialect `Reply`.
   - Win: about 300 lines and 2 mechanisms.
   - Effort: medium.
3. **Share the transport with forge-connection** (F4): one `Transport` and `host()`, one setup `Connection` constructor, and one document-limits builder (this needs skein-forge's twin `Limits` merged, which is A1's).
   - Win: about 100 setup lines, plus temper's field-by-field translation.
   - Effort: medium.
4. **One test convention in setup** (F5, F6, F14): everything under `src/tests/`, the heap ceilings moved into the world, and a redaction helper.
   - Win: 6 `#[path]` attributes, 39 `#[expect]`s become 1, about 100 lines.
   - Effort: low.
5. **Record or close the design departures** (F7, F8, F9, F10).
   - Effort: low for the documentation; high to move the Forgejo stories onto `HostWorld`, which would delete about 760 lines.
6. **Client hygiene** (F11, F12, F15, F16): re-cite 7 module headers, one end-of-decision done-mark, literal rows in the budget tests, and one word for the owner.
   - Win: about 30 lines.
   - Effort: low.

## Findings

### F1. Setup's shared vocabulary lives in Forgejo's reconcile module, and the crate is asymmetric

**Evidence.**
- `reconcile.rs:21-118` defines `Need`, `Outcome` (including GitHub's `Manifest`, `Lacking` and `Install`), `Event`, `Reply`, `Failure`, `Stage` and `Why`.
- The crate-wide `reply`/`unexpected` helpers sit in `reconcile/tokens.rs:218-233`.
- 10 of the 14 `github/*.rs` files import `crate::reconcile`, and the host names `reconcile::Need` 47 times (`rg -c`).
- Forgejo's plans sit at the crate root (`given`, `limits`, `access`, `adopt`, `check`, `create`, `take_over`, `reachable`), while GitHub's sit in `github/`. The result is pairs of same-named types: `given::{Given, Place, Refused}` against `github::given::{…}`, and `limits::Limits` against `github::limits::Limits` against `host::Limits`.
- `reachable.rs:30-43` is two machines in one type: six `Github*` states, two constructors (`:67`, `:86`), and dummy fields for GitHub (`admin_name: Box::new([])`, `listing: Token::Admin`).
- `given::check_create` (`given.rs:174-180`) takes a `host::Plan`, so the plan layer depends on the host's boundary.

**Fix.**
- Create `plan.rs` holding the vocabulary, `reply`, `unexpected` and a single `stop`.
- Move the Forgejo plans into `forgejo/`.
- Give GitHub its own `github/reachable.rs`, sharing `Reached`.
- Make `check_create` take the fields, not the `Plan`.

### F2. The admin step is written 5 times, `stop` has 6 shapes, and plans halt two ways

**Admin step.** Begin, read `AdminToken`, `GET /user`, `is_admin`: `reconcile.rs:207-279`, `adopt.rs:73-104`, `create.rs:164-187`, `take_over.rs:80-103`, `reachable.rs:116-160`.

**Page-cap read.** `max_response_items > 0` is checked 4 times (`access.rs:175`, `reachable.rs:365`, `take_over.rs:125`, `reconcile/tokens.rs:120`).

**Paged token list and delete.** Written twice, in `tokens.rs:62-202` and `take_over.rs:77-180`.

**`stop` signatures.** Six: `(step, why)`, `(subject, why)`, `(why)`, `(step, subject, why)`, `(forge)`, and the free function `app::stop(forge, stage, subject, why)`.

**Halting.**
- Five plans have a bespoke `halt`.
- The other seven are halted by `Machine::halt_begun` (`host/run.rs:119-134`), which feeds them a second `Begin` to provoke an `Unexpected` stop and then overwrites `why`.
- `reconcile::halt` (`reconcile.rs:449-470`) does the same, choosing `Written` or `Begin`.
- `stop_named` (`reconcile.rs:415-419`) and `tokens::advance` (`tokens.rs:234-251`) build a stop and then patch its subject through `let … else unreachable!` or a match with an `unreachable!` arm.

**Child-machine results.** Two identical `Progress` enums (`github/hook.rs:22`, `github/installations.rs:41`) sit beside `access::Move`/`Fail` and `write::advance`'s `Result<Option<Need>, Why>`.

**Fix.**
- An `admin` sub-machine shaped like `Access`.
- One `plan::stop(forge, stage, subject, why)`.
- Every plan exposes `halt` naming its current subject, so `halt_begun` and the provoked events go.
- One `plan::Progress`.
- Win: about 200 lines.

### F3. `Reply` mixes both dialects, so every arm pays for the other

**Evidence.**
- `Reply` (`reconcile.rs:66-77`) carries `GithubDocument` and `GithubRows` beside Forgejo's bodies.
- Step code has 35 `Reply::GithubRows(_)` lines and 30 eight-line `other @ (Reply::… )` fallthroughs.
- Seven hand-written Reply-to-`Why` maps duplicate `unexpected()`: `github/app.rs:44`, `github/installations.rs:179,188`, `reconcile.rs:341`, `github/reconcile.rs:171`, `github/hook.rs:61`, `tokens.rs:227`.
- `account()` lists all 27 `Document` variants (`reconcile.rs:308-347`).

**Fix.** Split the reply per dialect (or `Reply::Body(Body)`), and route every fallthrough through one `unexpected`. Win: about 100 lines.

### F4. The host duplicates forge-connection's transport stack and settings

**Evidence.**
- `host/connection.rs` (763 lines) and `skein-forge-connection/src/connection.rs` (1,174 lines) each hand-build the same stack: an HTTP client, an optional TLS client, a JSON tokenizer, three up/down queue pairs, `json_next`, and `upload`/`uploaded`.
- `Transport` and `host()` are duplicated (`host/settings.rs:79-103` against `forge-connection/src/settings.rs:66-90`). Temper copies one into the other field by field (`temper-service/src/setup/runtime.rs:63-66`), although design 6.3 says the settings are "as forge-connection.md's do".
- Inside setup, `Connection::new` and `::github` (`connection.rs:73-203`) repeat a 22-field struct literal and a 5-arm `Method` map.
- `limits::documents` and `limits::github` (`host/limits.rs:42-75`) are byte-identical, because skein-forge has two identical `Limits` types (`forgejo/limits.rs:4`, `github/limits.rs:5`).

**Fix.**
- One `Transport` and `host()` in skein-forge.
- One "exchange" machine shared by both components, or at least one constructor in setup.
- One documents-limits type (A1).

### F5. Four test conventions and 6 `#[path]` attributes in the setup crate

**Evidence.** `rg '#\[path'` finds:
- `src/tests/` mounted from `tests.rs`;
- `src/tests/github_host.rs` and `github_states.rs` mounted from production `host.rs:6,12`;
- `src/tests/github_memory.rs` mounted from `host/states.rs:113`;
- `host/tests.rs` mounted through `host.rs:523`, which is redundant (it is the default path);
- the sibling `host/connection_tests.rs` (`connection.rs:762`);
- `host/runtime_tests.rs` mounted from another test file (`src/tests/host.rs:1432`);
- an inline `mod tests` in `github/key.rs:67`.

`host/tests.rs` also reaches `crate::tests::host::limits`. The worlds add 3 more `#[path]` attributes (`tests/forge/setup/src/host.rs:55,57`, `tests/forge/client/tests/suite/main.rs:15`). forge-client uses a single convention.

**Fix.** Put every test under `src/tests/`, declared from `tests.rs`, as the client does.

### F6. Heap ceilings in setup's step tests rely on a hidden global allocator

**Evidence.**
- `src/tests/github_memory.rs:25-26` declares `#[global_allocator]` for the whole unit-test binary.
- `create_bounds.rs:125-160` checks `CREATION_PEAK_CEILING = 8_500` and depends silently on that declaration, made in a file mounted from `host/states.rs`.
- `github_memory.rs:303` sets `GITHUB_PLANS_PEAK_CEILING`.
- Design 7 puts the scenario ceiling in the world ("the pure plans keep their edge and derived-capacity checks"). The status file is silent.

**Fix.** Move both checks into `tests/forge/setup/tests/suite/heap.rs`, which already declares the allocator (`main.rs:1-2`), or record the departure.

### F7. The setup world plays the Forgejo plans without the host

**Evidence.**
- `World` (`world.rs`) steps `create`, `adopt`, `reconcile`, `check`, `reachable` and `take_over` directly, at `world.rs:106,250,254,258,263,269`.
- It runs them over its own `exchange.rs` (321 lines) and `decode.rs` (83 lines, a whole-buffer tokenizer pump).
- `exchange.rs:276-321` re-derives the host's Document-to-Reply mapping (`host/connection.rs:411-420`).
- The Forgejo suites (`check.rs`, `reconcile.rs`, `reachable.rs`, `take_over.rs`, `create.rs`) use `World`. Design 7 says the world "plays the host component", so deadlines, retries, slots and connections go unexercised for those stories.
- There is no referee (testing-strategy.md 7).

**Fix.** Run the stories on `HostWorld`, then delete `World`, `exchange.rs` and `decode.rs` (766 lines). Otherwise, record the departure.

### F8. Runtime membership and `Abort` are designed only in temper's draft

**Evidence.**
- `host/boundary.rs:110-133` defines `Add`, `Remove` and `Abort`.
- Six comments cite `temper/docs/design/draft/playground-reset.md`: `host.rs:2,174`, `host/settings.rs:2`, `boundary.rs:110`, `states.rs:93`, `runtime_tests.rs:1`.
- forge-setup.md 6.3's table lists only `Run`, `Answer` and `Close`. forge-connection.md 3.5 covers access at runtime, not membership.
- `Add` silently ignores an invalid forge (`host.rs:70-77`).

**Fix.** Write the three rows into 6.3 and the status file, and cite skein's own design.

### F9. Design 6.3's "at most 4 events up and 4 requests to io" is not held

**Evidence.**
- `drive` runs `for _ in 0_u32..256` (`host.rs:294`), a bound that is neither named nor derived from the limits.
- `Abort` pushes one `Stop` per run.
- `ready()` (`host.rs:353-366`) restates `drive`'s two selection predicates (`:296-309`).

**Fix.**
- Drop the "at most 4" from the design, since outputs are uncapped under the model.
- Derive the pass bound from the limits.
- Share one predicate between `drive` and `ready`.

### F10. Client facts are dropped inside the step

**Evidence.**
- `domain.rs:96` creates `Queue::with_capacity(l.facts)`, and `domain.rs:149-153` does `try_push` with a `lost` counter.
- programming-model.md 3 and notes.md ("Why facts are never lost inside") forbid loss inside the process.
- forge-client.md 2.2 caps `facts` without calling it a departure.
- The status file claims "uncapped outputs".

**Fix.** Emit facts into an uncapped output queue and let the owner's sink drop and count, or record the departure.

### F11. Seven client module headers cite the wrong section

| Location | Cites | Should cite |
|---|---|---|
| `limits.rs:1` | 5 | 2.2 |
| `keep.rs:1` | 5 | 3 and 4 |
| `identity.rs:1` | 6 | 7.1 |
| `facts.rs:1` | 5 | 2 |
| `domain.rs:1` | 5–6 | 2 |
| `calls.rs:1` | 5.4 | 6 (5.4 does not exist) |
| `domain.rs:126` | "the keep" | 3 |

`resume`'s doc (`domain.rs:252-254`) has two summary lines run together.

### F12. Client done-markers are scattered and named as queries but mutate

**Evidence.**
- `domain.rs` pushes `OutboxDone` behind `settle_done` at 6 sites, and `ReadAfreshDone` behind `fresh_done` at 3.
- Both functions clear a flag (`outbox.rs:236-246`, `keep/restore.rs:96-111`).

**Fix.** One `end_passes(d, out)` at the end of `step`, `resume` and `fire`; rename the two functions `end_settle` and `end_fresh`.

### F13. Subset departures held only in review

**Setup: 9 `if let`/`let … else` on own enums or tuples, plus 2 tuple matches.**
- `create.rs:339,350`
- `reconcile.rs:416`
- `host.rs:130`
- `orgs.rs:46`
- `access.rs:70,257,261`
- `given.rs:176`
- tuple matches at `host.rs:447` and `reachable.rs:420`

**Client: 13, plus 1 tuple match.**
- `outbox.rs:325`
- `keep/answer.rs:267,501`
- `keep/poll.rs:151,187`
- `api.rs:122,138,144,224`
- `outbox/retry.rs:219`
- `outbox/find.rs:213,224`
- `domain.rs:220`
- tuple match at `domain.rs:97`

**Two related style problems.**
- `api.rs` uses both styles in one file: 4 `let … else` beside 9 helpers that each list all 28 `Answer` variants, and those variants are already excluded by `shape()`.
- `orgs.rs:44-55` is a one-arm match whose body branches on `self.phase == Phase::Org`.

**Fix.** Use exhaustive matches; use `skein_lib::earliest` at `domain.rs:97`.

### F14. Debug-redaction tests repeat a lint exception 39 times

**Evidence.**
- 39 `#[expect(clippy::disallowed_macros)]` with 36 distinct reasons, covering 55 `format!("{:?}")` calls in setup's tests.
- Two different scan idioms: 7 `.windows()` loops and `bytes::find`.

**Fix.** One fixture, `assert_redacted(&dyn Debug, &[&[u8]])`, carrying a single `expect`.

### F15. Client budget tests compute their expectation with the code under test

**Evidence.** `tests/budget.rs:17-52` compares against a twin `Harness` and branches on row values (`if cost == 4 && free == 1`). This breaks testing-strategy.md 10, "One behaviour per test" and "Rows carry data".

**Fix.** Rows of literals: `(cost, free, left, first)`.

### F16. Naming drift

- **The client's owner** is called "owner" (58), "top" or "connector top" (14), "parent" (12) and "root" (6); the design says only "owner".
- **`Priority`** (59 uses) is the design's "class" (used 14 times; "priority" appears 0 times), and it lives in `facts.rs`.
- **Setup spells GitHub two ways:** `Github*` (124, for example `Reply::GithubDocument`) and `GitHub*` (69, for example `Call::GitHub`).
- **Setup's `Place` limit names differ by dialect:** `OrgsLimit` and `ReachableLimit` against GitHub's `Reachable` and `KeyBytes`.
- **`reachable::Token`** collides with lib's `Token`.

### F17. Limits shapes in one place, with duplicates

**Evidence.**
- Setup has three `check -> Result` functions: `limits.rs:20`, `github/limits.rs:24` and `host/limits.rs:142`.
- `host::check` calls `is_usable` (io), `valid` (tls, http) and `check` (docs, dialect) side by side (`host/limits.rs:165,171,215,252`).
- `Unusable::Zero` stands for four distinct failures (`:162,166,248`).
- The `deadline <= backoff <= backoff_max` triple and its check are duplicated (`limits.rs:15-41`, `github/limits.rs:15-47`), and copied again into `Times` (`host/settings.rs:56-70`).

**Fix.**
- One `Wait { deadline, backoff, backoff_max }` type with one check.
- Distinct `Unusable` variants.
- Reconcile design 6.1's `check` with the model's `valid` in the status file.

## Hypotheses

- **H2: confirmed for setup.**
  - `Add`, `Remove` and `Abort` exist only through temper's draft (F8).
  - Neither forge-setup.md 6.3 nor 6.5 has them, and neither does the status file.
  - forge-connection.md 3.5 covers access at runtime, not forge membership.
- **H15: refined.**
  - `tests/forge/conformance` is listed in testing.md 6 (line 351) as "the conformance tool, outside the suites".
  - `tests/forge/github` is missing from that list; only forge-github.md:904 names its README.
  - Both are binaries with their own `clippy.toml` and hand-written `[lints]` blocks.
  - `crates/skein-forge/tests/common/forgejo.rs:48,447,544` reads `tests/forge/conformance/{transcripts,deliveries}` by tree path.
  - Recommendation: move both tools beside `testing/skein-forge-conformance`, which they already depend on, as `testing/*` tools. Keep the transcripts as data beside their consumer in skein-forge. List `tests/forge/github` in testing.md 6 until it moves.
- **Tree-wide findings in this area:**
  - **T-layout:** 4 test conventions and 9 `#[path]` attributes (F5).
  - **T-worlds-as-kits:**
    - `tests/forge/protocol` depends on the client world for `LIMITS` and the 5-line `translate::commit`.
    - The setup world borrows `skein-forge-machine-world`'s fixtures, `pipe` and its GitHub fake (10 references), plus `skein-tls-world` through a path (`Cargo.toml`, although the workspace declares it at `../Cargo.toml:360`).
  - **T-lints:** two identical hand-written `[lints]` blocks (the setup and client worlds). The setup world lacks the client world's `[lib] test = false`. 39 redundant `expect`s (F14).
  - **T-limits-shape:** F17.
  - **T-primitives:**
    - two whole-buffer tokenizer pumps: `src/tests/github_manifest.rs:91-127` and `tests/forge/setup/src/decode.rs:40-81`;
    - 7 `Env` rewraps in `host/connection.rs`;
    - 8 `u64::try_from(len)` against 2 `len_u64`;
    - 24 `usize::try_from(..).expect`;
    - `translate.rs:25-73` hand-rolls `skein_forge::marker::{encode, decode}`.
  - **T-temper-draft:** 6 citations (F8).
  - **T-naming:** "actual" is negligible here (1).

## Status-file cross-check

- **forge-setup.status.md**
  - "Not yet: 9" is correct, since its items are open questions with "later" defaults.
  - It misses F6, F7, F8, F9, the duplicated `Transport` (F4), the missing referee, and `check` against `valid` (F17).
  - "Measured peaks under scenario ceilings" holds, but two of the peaks sit in step tests, not the world.
- **forge-client.status.md**
  - "uncapped outputs" is contradicted by the facts cap and its loss (F10).
  - "Not yet: unknown" is a placeholder, not a statement.
  - It misses the stale citations (F11).

## Incidental

1. **`identity.rs:60`** admits a `Config` by `size_of::<Writer>()`, so admission depends on the platform (16 bytes on x86_64, 12 on i686) and moves with the type. `measure.rs` pins its widths instead.
2. **`host.rs:70-77`**: an invalid `Add` is silently dropped. The owner only learns at a later `Run`, as `Stop(Admin, plan name, Unexpected)`.
3. **`host.rs:196`**: `Remove` makes up the failure `Stage::Admin` / `"remove"` / `Why::Down`, which is wrong for a GitHub forge.
4. **`host/limits.rs:165`**: an unusable io or TLS limit reports `Unusable::Zero`, which misdirects the owner.
5. **`reconcile.rs:219`**: the guard `if !token.is_empty()` is dead after `normal()`.

---

# A5-fake-forge

Scores: Cleanliness 3, Organisation 3, Taste 3, Naming 3, Primitives 2, Elegance 3, Economy 3, Complexity 3

Verdict: The fake forge does what fake-forge.md asks, its credential hygiene is careful, and its observation codec and tests are among the best in skein. It is bigger than its fidelity needs, though. The Forgejo layer is 8.5k src lines, as large as the domain it translates. That size comes from the same 28-variant setup group repeated in 12 exhaustive matches over Forgejo's ~75-variant `Request`. The GitHub layer (8.2k src) leans on that layer and on skein-forge's `forgejo` modules for things that are neutral: the git route, its `Limits`, `Route`, error statuses, decimal parsing and RFC 3339. The two `Server`s duplicate about 200 lines of connection machinery. Four JSON tokenizer pumps re-implement `skein_json::parse`. The step code also breaks the model in ways the status file does not record: the only two `impl Iterator` in skein's crates, a generic fn, `loop`/`while`, and 8 hand `Debug` impls. Removing these departures and duplicates would save about 1.3–1.6k lines with no loss of behaviour. skein-fake-checkout is small and sound; its status file is wrong.

Good:
- `crates/skein-fake-forge-protocol/src/observations.rs:12-37`: measure, then write exactly, with skein-json's `Encoder`, and no hand JSON.
- `crates/skein-fake-forge-protocol/src/tests/observations.rs:28`: one table test for every observation variant, each with its exact JSON line (the shape testing-strategy 8 and 10 ask for).
- `crates/skein-fake-forge-domain/src/object.rs`: the SHA-1 spelling lives in one place, shared by both faces and the queue.
- Credential hygiene: heads are snapshotted with `Authorization` emptied (protocol `server.rs:515`); `logs::Record` keeps header presence only; `GitCall` keeps a mint number, never a token.
- `crates/skein-fake-checkout` plus `tests/fake-checkout`: 1.2k lines, no dependencies, a narrow synchronous `git::Remote` boundary, and a suite plus fuzzy replay sweep that share a `src/` library, as testing.md 6 says.

Top recommendations:
1. **Classify Forgejo requests once** (setup / table / domain), ideally by nesting the setup routes in `skein_forge::forgejo::request`. Win: ~400–500 lines and 3 `too_many_lines` expects (F1). Effort M.
2. **Make the neutral pieces Forgejo-free.** Give `git` its own `Limits` and status function; move the inverse of `object_bytes` into the domain; keep `Hosted`, `Faults` and `observations` out of the Forgejo face, as a module set or a small `skein-fake-forge-wire` crate. Win: GitHub stops building fake `protocol::Limits`, and so does temper (F4). Effort M.
3. **Structured deliveries.** Make `Post` carry `{target, host, headers, body}`. This deletes both hand-built heads and the peer's re-parse; move Forgejo's `sign` into skein-forge (F5). Win: ~90 lines. Effort S.
4. **One connection core for both servers**, with `Option<Hosted>` entry points, or a skein-http "respond with a whole body" request (5 hand reply streamers tree-wide) (F6). Win: ~150–200 lines and the peer's `entry!` macro. Effort M.
5. **Lib/json primitives:** `List::remove`, `skein_json::parse` for the pumps, `bytes::decimal`, `bytes::count`, a concat helper, and `Secret`-typed fields so `Debug` can be derived (F2, F3, F7, F8). Win: ~300–400 lines and all 10 hand trait impls. Effort S each.
6. **Test layout.** Give `tests/fake-forge` a `src/lib.rs` of fixtures; move the protocol crate's `tests/suite` there; add a domain `Config::calm(limits)`; measure the two hand-summed ceilings (F9, F10). Win: ~250 lines, 3 `#[path]`, 1 `dead_code` expect. Effort S–M.

Findings:

F1. **The 28-variant setup group is repeated in 12 exhaustive matches over `forgejo::request::Request`.** `rg -n "Request::DeleteRepoHook" crates/skein-fake-forge-protocol/src` finds `server.rs:673,827`, `tally.rs:183`, `setup.rs:189,254`, `response.rs:89,177`, `routes.rs:84,199,406` and `answers.rs:276,354,598`. The same Tokens…UserSearch arm list opens `routes::repository` (`routes.rs:61-88`) and `routes::op` (`routes.rs:179-206`). The expects at `routes.rs:171,303` give the cost as their reason: "exhaustive domain request selection also names the protocol-only setup variants". `missing_scope` (`server.rs:658`) and `table` (`server.rs:753`) are each a 75-arm match. Fix: in `decoded` (`server.rs:584`), split into `Setup` / `Table` / `Domain` once; each later function then matches its own sub-enum. Estimate: −400 lines.

F2. **Step code departs from programming-model 10.3, and the status file records none of it.**
- `impl<'a> Iterator for RepositoryHooks` (`setup/repo_hooks.rs:117`) and `impl<'a> Iterator for Subscriptions` (`setup/hooks.rs:112`), each with a `loop` (lines 120 and 115). These are the only `impl Iterator` in skein's crates.
- A generic `fn remove<T>` (`setup.rs:433`).
- `-> impl Iterator` (`installations.rs:294`).
- `while` loops at `setup/repos.rs:252` and `ci.rs:350,361`.
- 8 hand `impl core::fmt::Debug`: `rates.rs:32`, `app.rs:35,73`, `logs.rs:22`, `service.rs:145,192`, `pages.rs:28,50`.
- Types with lifetime parameters: `Hosted<'a>`, `Request<'a>`, `Page<'a>`, `Body<'a>`, `Route<'a>`, `Detail<'a>`, `RepositoryHooks<'a>`, `Subscriptions<'a>`.

Fix:
- Replace the iterators with a `for` over the slice at the caller.
- Add `List::remove` to lib (F8).
- Hold the redacted fields (a target that carries a code, a key) as `Secret` and derive `Debug`.
- `template_bound` becomes `len + 14 * bytes::count(bytes, b"{pull}", ..)`, since lib already has `count`.
- Record whatever stays.

F3. **The GitHub layer uses Forgejo modules for neutral helpers.** 21 src lines name `forgejo::` (`rg -n "forgejo::" crates/skein-fake-forge-github/src | grep -v /tests`):
- `id::unsigned` ×10 (e.g. `routes.rs:65-66`, `comment_edit.rs:34`, `reads.rs:517-518`). It is a one-line alias of `skein_lib::bytes::decimal` (`skein-forge/src/forgejo/id.rs:17-19`).
- `time::write` ×2. `rates.rs:209-214` slices the RFC 3339 string to build GitHub's `YYYY-MM-DD HH:MM:SS UTC` expiry header.
- `document::Body` ×8.
- `limits::Limits` ×1 (`git.rs:250`).

Fix: call `bytes::decimal`; take time formats from a neutral module (hypothesis 7).

F4. **The private git route is neutral by design but typed by the Forgejo face.** The `git` module reads only `documents.*` and `http.body` (`rg -o "limits\.[a-z_]+(\.[a-z_]+)?" git.rs git/ server/git.rs` gives 35 hits on `documents.*` and 1 on `http.body`), yet it takes `crate::Limits` (with `calls`, `connections`, `raw_bytes`). It also re-exports `forgejo::server::Unrouted` (`git.rs:12`). As a result:
- GitHub fabricates `protocol::Limits { calls: 1, connections: 1, … }` (`github/src/git.rs:244-262`).
- It gets its statuses through `protocol::errors::answer(why, protocol::Route::Git(name), Time::ZERO)` (`git.rs:238`).
- Downstream, `temper/tests/one-process/src/lower/github.rs:50` builds Forgejo protocol limits for a GitHub world.

Fix: `git::Limits {json, writer, string_bytes, body_bytes, inner}` and `git::status(Error)` inside `git` (see hypothesis 3 for where it should live).

F5. **Delivery heads are hand-built twice, and one of them is parsed back.** `protocol/src/delivery.rs:139-169` and `github/src/delivery.rs:390-423` use the same parts→measure→write algorithm. `skein-fake-peers/src/forge/github_delivery.rs:155-166` splits the GitHub head back into lines to rebuild headers for `skein_http::client`, which encodes them again (dropping `Content-Length`/`Connection`). Two more duplicates: `address()` (`protocol delivery.rs:188`, `github delivery.rs:459`, each with its own `disallowed_methods` reason); and Forgejo's `sign` lives in the fake (`delivery.rs:172`) while GitHub's lives in the dialect (`skein-forge/src/github/delivery.rs:35`). Fix: `Post { target, host, headers: Box<[Header]>, body }`. The GitHub sender passes the headers straight through. The Forgejo raw writer either moves onto the client (it then gains the status) or uses an exposed `skein_http::client::request::write` (now `pub(super)`, `client/request.rs:70`). Move `forgejo::delivery::sign` next to `EVENT_HEADER`.

F6. **The two `Server`s duplicate their connection machinery.** Duplicated parts:
- an identical 8-state `State` (both `server.rs:17`);
- head redaction (protocol `515-521`, GitHub `370-376`);
- `Up::Room` reply streaming (protocol `475-489`, GitHub `410-424`);
- `room`/`reply_room` (`881` and `645`);
- `start`, `next`, `http_down` and `close`/`closed`;
- the paired entry points `up`/`up_with_domain` and `resume`/`resume_with_domain`.

The domain is lent in three shapes: `Hosted`; `&Domain` in GitHub's `down`; nothing in Forgejo's `down`. The peer needs a `macro_rules! entry` to dispatch over both faces (`skein-fake-peers/src/forge/layer.rs:146`). Fix: a shared connection core, or a skein-http request that streams a whole body (the same loop appears in oauth-accounts `listener.rs:315`, fake-llm `provider.rs:928` and fake-peers `oauth.rs:319`). Use one entry per event, with `Option<Hosted>`. Estimate: −150–200 lines.

F7. **Four whole-buffer tokenizer pumps in step code:**
- `observations/read.rs:140`: ticks ×6+32, `Held`.
- `github/src/app.rs:251`: ticks ×4+16, with a hand cursor over `Read::Fill`/`Scan`; it copies `skein_json::parse` (`parse.rs:26-60`).
- `manifest.rs:291`: ticks ×8+64, `Intake`.
- `github/src/git.rs:263`: drives the git decoder.

Three of them collect a `List<Token>` and walk it with ad hoc helpers (`read.rs` `object`/`find`/`at`, `manifest.rs` `take`/`validate`). Fix: `skein_json::parse` plus `Document::member`; a slice driver for the decoder (hypothesis 6). Estimate: −120–150 lines.

F8. **`List` removal is hand-rolled four times:** `setup.rs:433` (generic, rebuilds the list); `github delivery.rs:96-106`; `installations.rs:167-180`; `installations.rs:382-390`. The last three wrap rows in `Option` only so they can shift by `take()` (`List<Option<Entry>>` at `delivery.rs:62`, `installations.rs:82-83`), which forces `expect("packed log")`. Fix: `List::remove(at)` in skein-lib, then drop the `Option` wrappers.

F9. **The 26-field domain `Config` and 24-field `Limits` literals are spelled out by hand.** In this area: `fake-forge-domain/src/tests.rs:23-77`, `tests/fake-forge/tests/suite/memory.rs:17-70`, `protocol src/tests/fixtures.rs:505-555`, `github src/tests/fixtures.rs:512-562` and `tests/fake-forge/tests/github/fixtures.rs:124-172`. Outside it: 5 more in skein (`rg -l "hooks_lost:"`) and 7 downstream files. Fix: `Config::calm(limits)` (no faults, fixed latencies), as `skein-sim/src/config.rs:153` does. Estimate: −150 lines here.

F10. **The fake forge's memory tests live in five places under two layouts.**
- `crates/skein-fake-forge-protocol/tests/suite/`, against testing.md 6. Its `memory.rs:4-6` pulls in `src/tests/fixtures.rs` by `#[path]` under `#[expect(dead_code)]`, and `main.rs:3-6` re-imports 14 names so the fixture's `crate::` paths resolve.
- `tests/fake-forge`, which has no `src/lib.rs`. Its fixtures are pulled in by `#[path]`, and `tests/fake-peers/tests/suite/github_memory.rs:2` reaches into them too.
- Two GitHub unit tests (`delivery.rs:567-611`, `src/tests/memory.rs`) add up `size_of_val` and lengths by hand instead of measuring with the counting allocator. They can miss `Map` nodes.
- Inline test modules in `delivery.rs:564`, `rates.rs:255` and `server.rs:654,714`.

Fix: shared fixtures in `tests/fake-forge/src/`; move the protocol suite there; measure the delivery and installation stores in `github_memory`.

F11. **Naming: one word per concept fails in several places.**
- Forgejo's `Error { Limits }` (`service.rs:7`) names every bad scenario table, including a duplicate token at `service.rs:53`. GitHub has `Problem { Limits, Config }`.
- `Hosted.config` holds a domain `Config` that callers take from `fake_env.limits` (`tests/fixtures.rs:179`). This is because the domain's `Env<Config>` makes `env.limits` a `Config`: `env.limits.limits.` appears 18× in domain src.
- The crate is aliased `forgejo` ×4, `fake` ×6 and `protocol` ×2.
- temper's "engine"/"worker" vocabulary appears in the domain docs (`lib.rs` ×6, `api.rs:3`, `reads.rs:1`, `scenario.rs:175`).
- fake-checkout's `forge: &impl Remote` beside `remote: &[u8]` (the repository's name) (`git.rs:156-160`).

F12. **Leftovers:**
- `autotests = false` (`github Cargo.toml:2`), though the crate has no `tests/`.
- A doc comment on the wrong fn (`github server.rs:81`).
- Task vocabulary: `routes.rs:1` ("The task's route subset"); `reads.rs:423` ("belong to the rates task; no rate simulation here", stale since rates are built per 10.1); `tests/queries.rs:41`.
- A history note in `fake-checkout lib.rs:3-4` ("Extracted from temper's … at commit `246164f` … behavior preserved").
- `fake-checkout/clippy.toml:1` is headed "What the test crates under tests/ may not use".
- 119 citations of "(fake-checkout.md, section N)" in 1,245 lines, many on tuple fields documented only because of `missing_docs` (`git.rs:84-90`).
- `protocol lib.rs` interleaves `use` and `mod` with no blank lines.

F13. **The tally counts, then corrects itself.** The entrance counts every request as unrouted (`server.rs:528`). `decoded` then decrements `unrouted` and re-implements `Tally::count`'s map increment (`server.rs:593-595`). `count_store` saves and restores `unauthorized` (`tally.rs:261-265`). `ROUTES = 81` is a hand constant (`tally.rs:3`) against 80 keys, and the design says "up to 74". Fix: count once after routing; pin `ROUTES` with a test.

F14. **Validity checks are duplicated.**
- Protocol `Server::new` (`server.rs:69-79`) restates `service::valid` (`service.rs:219-229`).
- GitHub's `service::valid` calls `server::valid`, then repeats `response >= 128` and `inner.checked_mul(4)` (`service.rs:903,906` vs `server.rs:103,105`).
- `Service::new` repeats the rate checks (`:275-276` vs `:893-894`).
- The shared `Faults` type is bounds-checked by two different pieces of code (protocol `service.rs:34-36`, GitHub `valid_faults` `:968`).
- There are five origin/URL validators: `check_base` (`service.rs:196`), `destination` (`delivery.rs:193`), `valid_origin` (`github service.rs:914`), `manifest::absolute` (`:206`) and GitHub `delivery::valid` (`:427`).

Fix: `Faults::valid`, one origin parser (in skein-http), and constructors that call their crate's `valid`.

F15. **fake-checkout economy.** The both-parent walk is written three times (`git.rs:163-171,192-201,391-401`), and `[at, b"/.git/temper-head"].concat()` four times (`:219,246,281,369`). Fix: one `reach()` and path constants. Estimate: ~25 lines.

Hypotheses:
- **3: confirmed and refined.**
  - The heads are built by the same algorithm (F5). A helper would be the wrong fix: the GitHub head exists only to be parsed back, so the layers should return structured headers and skein-http's client should encode them.
  - `routes::{object, commit}` are neutral wrappers around the domain's `object_bytes`. GitHub uses only `.bytes` (`reads.rs:291-308`). The inverse belongs in the domain, next to `object_bytes`.
  - `git` is semantically neutral (both faces serve it, and temper's fake git uses its client), but it is typed by Forgejo (F4). The domain cannot hold it (it has no JSON), so it needs Forgejo-free types, in a neutral module or crate along with `Hosted`, `Faults` and `observations` (the last also has nothing Forgejo in it).
- **Tree-wide findings as they appear here:**
  - T-layout: the protocol crate's `tests/` (F10); the GitHub crate uses three step-test conventions (`src/tests` via `tests.rs`, 4 inline modules, 1 `#[path]`).
  - T-worlds-as-kits: 3 `#[path]` in the area, plus `tests/fake-peers` reaching into `tests/fake-forge/tests/github`.
  - T-lints: 3 hand `[lints]` blocks (`fake-checkout`, `tests/fake-forge`, `tests/fake-checkout`); 1 `clippy.toml` copy; 37 `#[expect]`; `manual_map` with 3 wordings over 6 sites.
  - T-limits-shape: `valid(&Limits)` in the domain; `valid(&Limits, &Config)` plus an inline restatement in protocol; GitHub has four `valid`s and calls skein-forge's `check()->Result`. Domain entry points `step`/`step_admitted`/`fire`.
  - T-primitives: F7; measured concat at 16 `for part in` sites plus `joined` (24 calls); `usize::try_from(..).expect` ×69 (26 messages); `len_u64` 16 vs `u64::try_from(len)` 5; Env rewraps 11; hand hex in `queue.rs:49-53` and `delivery.rs:176-180`.
  - T-naming: F11.
  - T-temper-draft: none here, but temper vocabulary does appear (F11).

Status-file cross-check:
- **fake-forge.status.md.** Confirmed as lagging: GH013 and the push refusal (`required_signatures` is display only), organization hooks route nothing, no real-git bridge. Not recorded:
  - Design 4's `MAX_UP = 3, MAX_DOWN = 18` do not exist; the server's queues are `Queue::uncapped()` (`server.rs:84-87`).
  - Design 2.2 says "Twenty-one limits"; `Limits` has 24.
  - Design 4 says "up to 74" routes; there are 80 keys and `ROUTES = 81`.
  - Design 2.1 names `step` and `fire`; the code adds `step_admitted` and `rate` (`domain.rs:478`, `lib.rs:78`).
  - An undocumented fault (Incidental 1).
  - The GitHub git route answers a malformed body `400` (`github git.rs:123`), where design 4 and 10.1 imply Forgejo's `422`.
  - The model departures (F2) and layout departures (F10).

  "Shared git-route work" is listed as not yet, but 10.1 and the code show the shared route built on both faces.
- **fake-checkout.status.md.**
  - "Not yet: product crate, separate tests": the crate is in `crates/` and ships (temper-fake-checkout wraps it), and `tests/fake-checkout` exists (767 lines, suite plus fuzzy).
  - "Fake git over the forge's git route" is temper's to build by design 1 (it exists at `temper/crates/temper-fake-git/src/route.rs:124`).
  - "The rest unknown": sections 2–5 are visibly built (`lib.rs:251-618`, `git.rs:156-389`, `tests/suite/fuzzy/git.rs`).

  The file needs a rewrite.

Incidental:
1. `protocol server.rs:281` closes the connection on half of the domain's timeouts (`rng.chance(500)`). Design 4 says a timeout is `500`. It also adds an RNG draw.
2. The GitHub service seeds five RNGs with the same `seed` (`service.rs:343`, `logs.rs:47`, `delivery.rs:72`, `pages.rs:76`, `installations.rs:98`), so their streams are correlated. The blob HMAC key's first word and the delivery-id offset come from the same first value. Derive sub-seeds from one master RNG.
3. `ROUTES` is unpinned. Adding a route without bumping it panics at "route capacity" (`server.rs:595`).
4. Some fixtures in skein's own tests carry temper's names: `temper.test` (`github tests/pages_install.rs:152`) and `temper/` branches (`protocol tests/http.rs:487`).

---

# A6-io

# A6 io stack: skein-io, skein-tls, skein-channel, fake-channel, skein-heap and their worlds

Scores: Cleanliness 3, Organisation 3, Taste 3, Naming 4, Primitives 2, Elegance 3, Economy 3, Complexity 3

Verdict: The core of io is good. `stream.rs`, `listener.rs` and the joined ends follow the model's state-machine idiom closely, `kernel.rs` states the backend contract once and checks it with `Complete::is_valid`, and `stage.rs` gives every loop one shared stage. The weak points sit around that core. Three io entities, and the channel machine, hold their lifecycle in flags. `FileIo` does not have the step-machine shape. Io and FileIo share one kernel but mint operation tokens from spaces that overlap, and downstream code patches this in three different ways. The demand check is written three times, and the channel's control sizes four times. The tests carry most of the excess: tests/io has three loops and five hand-written io drivers, and none of them runs the stage that services run. tests/readiness copies tests/ring's harness. tests/tls is a kit as well as a world. The channel world does less than channel.md 11 asks, although the status file says section 11 is built. My estimate is ~1.1k lines removable (~450 src, ~650 tests) with no loss.

Good:
- `crates/skein-io/src/stream.rs`, `listener.rs`: states are enums that carry their data, each completion is decoded into `Happened` and then stepped, and every change goes through `mem::replace` followed by `follow`. This is programming-model 5.4 as written.
- `crates/skein-io/src/kernel.rs`: the contract docs and `Complete::is_valid`. tests/ring (`lib.rs:89`) and tests/readiness (`readiness.rs:66`) hold every backend to the same contract through it.
- `crates/skein-io/src/stage.rs` and `limits.rs`: a most-out bound for each entry point, one stage shared by ~25 callers.
- `crates/skein-tls/src/held.rs` explains why it does not use lib's Intake. `client/limits.rs` names its formulas the way 6.3 asks.
- `testing/skein-heap/src/lib.rs` documents exactly what is counted, including the runtime's own allocations.
- `tests/io/src/ledger.rs` checks both halves of io's contract as the world runs.

Top recommendations:
1. **Give each machine its own lane of kernel operation tokens** (F1). Pass a lane when Io or FileIo is made, and route completions by `kernel::Lane::of(&Complete)`. This removes three hand-made workarounds and a silent collision. Medium effort.
2. **Move the per-call MAX_OUT assert into `skein_io::stage`** (F4, F5). The tests/io drivers can then call the stage, and the socket and output worlds can move onto `skein_world::World`. About −350 test lines, and the io world then tests the order services actually run. Medium effort.
3. **Turn the flags into state enums and share one demand check** (F2, F3). Pipe, Signals and Child get state enums, and stream, pipe and joined end share one demand check. About −60 lines, and one wording replaces three. Medium effort for Pipe, low for the shared check.
4. **Keep one public validity check for io limits** (F6). Make `is_usable` crate-private and have `Io::new` assert `valid`. Low effort, and it fixes downstream checks that test only the weaker condition.
5. **Clean up the channel** (F9–F11). Use one `largest_control` with named constants, use let-else and drop four expects, and give the fake its own frame codec. About −60 src lines. Low effort.
6. **Trim the tests** (F13, F15, F18). Readiness reuses ring's harness (−110), the native_output header/body pairs become loops (−150), the 9 stale path references go, and "worst case" becomes "ceiling". Low effort.

Findings:

F1. **Io and FileIo operation tokens collide on the one kernel.** Io names each operation `Id<Flight>::token()`, which is slot<<32 | generation (`crates/skein-lib/src/id.rs:41`). Generations start at 0 (`slab.rs:103`), so slot 0 at generation 1 is token 1. FileIo counts `next_op` up from 1 (`file_layer.rs:279`, `:344-349`). io.md 5 says a service "hands each completion back to the machine whose operation it was", but gives no way to tell which machine that was. Three workarounds exist:
- `testing/skein-world/src/files.rs:14` sets bit 63 (`TAG 1<<63`) and rewrites both `op` and `Op::Cancel{target}` (`:125-145`);
- `crates/skein-ui-text/src/terminal.rs:12` does the same with bit 62 (`:237-239`, `:312-315`);
- smith's `host_io.rs:35` keeps a `Map<Token, Operation>` route table.

Fix: a lane chosen at construction, held in the top bits of the token, plus one `Lane::of` (or a `kernel::Tag` that rewrites `op` and the cancel target both ways). Once a stage accepts a lane, all three workarounds can go.

F2. **Lifecycles are held in flags and Options beside the state** (programming-model 5.4: "never a lifecycle spread over flags and Option fields"):
- `pipe.rs:21-43`: 6 bools and 8 Options, under an `#[expect(struct_excessive_bools)]` at `:18`; the flags are touched 36 times (`rg -c 'pipe\.(closing|closed|eof|ended|finish|append)\b'`);
- `signals.rs:24-32`: the doc lists five states (`:8-15`), but the code holds 2 bools and 4 Options;
- `process.rs:40-48`: a `Phase` enum with no data, beside `pidfd: Option` and two bools;
- `crates/skein-channel/src/machine.rs:117-142`: 11 bools and 4 Options beside `phase` and `read`.

Stream, Listener and joined End are proper enums. Neither status file records this. Fix: one state enum per entity, with `follow`.

F3. **The demand check is written three times**: `stream.rs:1018-1059`, `pipe.rs:135-160` and `joined/end.rs:68-86`. The three copies use different messages ("fill within intake" against "a fill past the intake's cap could never be met (Limits::largest_read)") and different rules for an outstanding demand. Fix: one `stream::admit_demand` used by all three, saving ~45 lines. In the same vein, `layer.rs:196-241` repeats the insert, mark and token steps in four `adopt_*` functions, ~25 lines.

F4. **No io world runs the stage services use.** tests/io drives io by hand in `src/world.rs:76-118`, `processes.rs:216-245` and `output.rs:283-311`, and their order differs from `stage::up`:
- `world.rs:89` fires deadlines while completions are still waiting, where `stage.rs:51` returns;
- `processes.rs:240` calls `down` without checking `takes()`;
- `processes.rs:59` asserts there is room, where the stage waits for it.

`tests/suite/stage.rs` (112 lines) tests the stage only on its own. The only reason for the hand loops is the per-call MAX_OUT check. Fix: put that assert in the stage itself (fail-stop on io's own contract), and the three drivers collapse to stage calls, ~100 lines. Also, `stage.rs:54` works out the timer count again instead of calling `limits::timers` (`limits.rs:107`).

F5. **tests/io has three world loops:**
- `world.rs`, a 288-line loop of its own over `Sim`;
- `output.rs:330`, another loop of its own;
- `skein_world::World`, used by 9 modules (append, cuts, files, locks, pages, private, processes, renames, usage).

The 9 `impl Host` blocks also repeat the same four forwarders (completions, submissions, two deadlines), ~100 lines. Fix: move the socket and output worlds onto `skein_world` once F4 is done, and add an `IoHost` helper in skein-world that holds io and its queues. About −250 lines.

F6. **Two public limit checks, and callers pick the weaker one.** io.md 2 designs both `Limits::is_usable` and `valid`. But `Io::new` asserts only `is_usable` (`layer.rs:139`), and then calls `expect` on the capacities (`:140`). Downstream, `crates/skein-forge-setup/src/host/limits.rs:165` and `crates/skein-fake-peers/src/face.rs:113` check only `is_usable`. smith's `host_io.rs:110-113` (`native_io_valid`) rebuilds `valid` by hand. Fix: make `is_usable` crate-private and assert `valid` in `Io::new`. This leaves the one public check that model 6.3 names.

F7. **FileIo does not have the step-machine shape** (model 3 and 7):
- it has no `Limits`; it is built from positional arguments, `new(files, max_read, max_entries, timeout)` (`file_layer.rs:263`), plus `with_whole_limit`, `with_effective_user` and `seed_randomness`;
- its entry points take no `Env` (`down(io, now, …)` at `:388`);
- its deadline entry is called `expire` (`:604`) where Io's is `fire`;
- its pending states grow `Vec<u8>` and `Vec<Entry>` buffers (`:63`, `:86-142`) under a whole-module `#![expect(disallowed_types)]` (`:14`); the expect's reason cites "FileIo's limits", which do not exist as a type;
- `Event::Failed { owner, error, committed: false, residue: None }` is written out 27 times (`rg -c`) next to the `terminal_failure` helper (`:903`).

io.status does not record any of this. Fix: add `file_layer::Limits` and `valid`, make the entry points `down`/`up`/`fire(io, env, …)`, and use `List` and `Writer` for the buffers.

F8. **`kernel::Error` is spelled out in full 13 times outside kernel.rs** (`rg -c 'Error::NotAFile'`): records.rs ×3, layer/results.rs ×2, store.rs ×2, private.rs ×2, and once each in layer.rs, listener.rs, pipe.rs and process.rs. Each copy is ~25 lines, mostly marking the file errors unreachable for sockets. Fix: one exhaustive `Error::family()` that sorts errors into socket, file, cancel and other. Each mapper then matches its own subset plus one unreachable arm, saving ~200 lines.

F9. **Channel control sizes appear as literals in four tables**:
- `read.rs:21-41` (`3 ⇒ ≤262`, `5 ⇒ 0`, `6 ⇒ 2`);
- `machine.rs:768-784` (`header_allowed`, kinds 1 to 6);
- `limits.rs:14-19` (3, 6, 4, 14, 262, 8);
- `testing/skein-fake-channel/src/lib.rs:186-205`.

The 8-byte header length is also a literal (frame.rs ×3, and `7` at machine.rs:668). Fix: named constants and one `frame::largest_control(kind, &Limits) -> Option<u32>`.

F10. **Channel lint reasons contradict the model, and its output bounds are an old form.**
- Four `manual_let_else` expects give the reason "step code uses exhaustive matches instead of let-else" (`machine.rs:652`, `:752`, `:787`; `opening.rs:56`). Model 10.2 allows `let … else` on Option and Result, and io and tls use it.
- `machine.rs:790` gives `too_many_lines` the reason "until the machine migration completes", which is a plan, not a reason.
- `MAX_UP`/`MAX_DOWN` (`boundary.rs:9-12`; `lib.rs:9` "the owner reserves") keep the capped-queue reservation that model 2 (revised 10-28) dropped. io has already dropped it: `MaxOut` has only `submissions`. jig sizes loops from these constants (`jig-protocol-worker-link/src/component.rs:296`, `:384`, `:418`).

channel.md 9 ("Bounds") still describes the reservation, so either the design lags the model or the departure needs recording.

F11. **The fake channel shares the channel's types.** It imports `Control`, `Frame`, `Header`, `Schema`, `Term`, `control_frame`, `decode_control` and `parse_header` (`lib.rs:8-10`). testing-strategy 4 says "No shared types", because a misreading in `frame.rs` then passes on both sides. The fake also has a `_` arm on a tuple (`lib.rs:239`), and its lints block (`Cargo.toml:14-21`) lacks `wildcard_enum_match_arm` and `allow_attributes`. Fix: give the fake its own ~60-line header and body codec, taking the schema only as data.

F12. **The channel worlds fall short of channel.md 11.**
- In `generic_protocol.rs:30` and `:39` the mode makes no difference: `Mode::Socket | Mode::Pipes` share one arm and the same queues.
- Room is always granted at once (`:89-95`).
- Every Fill is met exactly, so bytes are never cut.
- No test has a slow reader or backpressure.
- There is no machine world that closes in every state.
- The "fuzz target" is a 512-seed loop that discards its results with `let _ =` (`fuzzy/channel.rs:30-37`).
- `generic_protocol.rs` is compiled twice through `#[path]` (`fuzzy/channel.rs:7`), needing a `duplicate_mod` expect (`main.rs:3`), although `seeded_protocol` is already `pub`.

Fix: call `crate::generic_protocol` directly, and record these gaps in channel.status.

F13. **Test harnesses are copied.** The `World` in `tests/readiness/tests/suite/readiness.rs:31-140` repeats `tests/ring/src/lib.rs:24-150` (start, reap, turn, wait, check_deadline, cancel_and_drain, run, close, socket), ~110 lines. Fix: `skein_ring_tests::World::on(Backend, ops)`. The area also has 6 hand-written `[lints]` blocks in tests/* (5 distinct by md5) and 2 copies of clippy.toml.

F14. **tests/tls is both a kit and a home for other crates' tests.**
- Four other worlds (llm-connection, oauth-accounts, forge/machine, forge/setup) import its `pki`, `server`, `drive` and `meter` modules, ~555 lines.
- `fake_peer.rs` (339 lines) tests the fake LLM's TLS face, `stack.rs` (358) tests HTTP routing over TLS, and `startup.rs` tests the shell's trust roots.
- This pulls skein-llm, llm-world, fake-llm-*, fake-peers, http and shell into its dev-dependencies.

Fix: move the shared modules into `testing/skein-fake-tls`, and move `fake_peer.rs` and `startup.rs` to the worlds of the crates they test.

F15. **Old forms and stale text.**
- skein-heap's `Meter::check` takes "bound, a worst case" and panics with "more than the worst case of" (`lib.rs:424`, `:438`; also `Cargo.toml:3` and `lib.rs:3`). Five `should_panic` strings in `tests/heap/tests/suite/meter.rs` pin the wording, while every caller passes a constant named `CEILING`.
- "actual" appears 38 times: tests/io 31, skein-io 6, tests/channel 1.
- Nine references point at test paths that no longer exist (`tests/*.rs`, `fuzzy_*.rs`): `tests/io/src/lib.rs:15-18`, `scenarios.rs:7-9`, `tests/suite/scenarios.rs:3`, `tests/tls/src/lib.rs:23-24`, `tests/ring/src/lib.rs:7`.
- `records.rs:50` and `:133` say "go here when a user pulls them" about Spawn, Signal, Exited and Shutdown, which are already defined just above.
- History notes in comments: "macOS run 40010966195" (`skein-heap/src/lib.rs:56`) and "task C" (`readiness.rs:20`).

F16. **Step code has wildcards and bare `unreachable!()` that the lints cannot see:**
- `layer.rs:768` has `_ => None` on `Option<&Entity>`;
- `signals.rs:85` has `Ok(_) => unreachable!` over `Done`;
- `file_layer.rs:874-893` matches on a tuple with a `_` arm;
- `layer.rs:514`, `:520` and `:525` use `unreachable!()` with no reason.

Fix: list the variants, and give each `unreachable!` its reason.

F17. **Five hand byte-copy loops** (`iter_mut().zip`) in step code: `addr.rs:46`, `machine.rs:673` and `:691`, `held.rs:67` and `session.rs:174`. They get around the scoped-expect rule for `copy_from_slice` instead of following it. Fix: add a lib `bytes::copy_into(target, src) -> Result<(), Overflow>`.

F18. **The TLS native face has no consumer but ~3.9k lines** (status confirms "3.6: native-face consumer" is not yet built): ~1.5k in src including its unit tests, ~2.4k in worlds. In `native_output.rs:284-500`, five header/body pairs differ only in `queued_ciphertext(false|true)`, and `:527-632` has four more such pairs. Fix: fold each pair into a loop over both cases, ~150 lines.

Hypotheses:
- **H4: refined.** The owner-token namespaces listed in the hypothesis do not overlap:
  - forge-connection `1<<59` (with next `<1<<60`);
  - fake-peers `[7<<60, 8<<60)`;
  - kv `0x4b56`;
  - oauth-accounts packs `slot*3+tag`.

  Each also sits over a machine of its own, so I found no live collision. Only convention keeps them apart. The certain collision is io's own operation tokens (F1). `Event::owner()` already exists (`records.rs:151`, and `file::Event::owner`) and does not help, because the clash is in kernel operation tokens, not in owners. What would fix it: io lanes (F1), and a lib `Token::prefixed(prefix, n)` for owners.
- **H5: refined.**
  - io.md 2 does record both `is_usable` and `valid`, but having two invites the weaker check (F6).
  - Of the eight resume sites:
    - `memory.rs:169` and `process_memory.rs:159` and `:389` meter each call, so they should stay;
    - `world.rs:78`, `processes.rs:218` and `output.rs:285` can move onto the stage once it asserts MAX_OUT (F4);
    - `ui-text/terminal.rs:254` cannot move without a lane, because it rewrites tokens between calls;
    - `throwaway-forgejo/http/host.rs:81` steps one transition at a time on purpose (its comment cites io.md 3.3), so keep it.
  - The `world.rs:102` loop should use the stage.
  - Refuted: `processes.rs:29` `Lifecycle {Running, Exited, Closed}` tracks the child's observed life, not lib's component `Lifecycle` (Live, Closing, Aborting, Closed), so it should not move to lib. Rename it so it does not shadow lib's name.
  - The Host forwarders are the shape of skein-shell's trait. They cannot go onto the stage, which is step code, but a skein-world `IoHost` could hold them (F5).

Tree-wide findings in this area:
- **T-layout:** three step-test conventions:
  - `src/tests.rs` plus `src/tests/`, with `mod directory` tacked on at `tests.rs:329`;
  - an inline `#[cfg(test)]` module in `digest.rs`;
  - `client/native/tests.rs` as a sibling file.
- **T-worlds-as-kits:** tls-world (F14), and the channel's `#[path]` (F12).
- **T-lints:** 6 plus 1 hand-written `[lints]` blocks, 2 clippy.toml copies, 48 `#[expect]`.
- **T-limits-shape:**
  - io has `valid` and `is_usable`; tls has `valid` and `native::valid`; channel has `valid(schema, role, limits)` and `Schema::check() -> Result`; FileIo has no Limits at all.
  - Entry points: io and tls are free functions taking `&Env`; FileIo takes `now`; channel uses methods with stored limits plus a `poll` that channel.md 9 does not list.
- **T-naming:**
  - "actual" appears 38 times in this area;
  - `Measured` names two things (`skein_io::Measured` for process usage, `skein_heap::Measured` for a step's heap);
  - `Limits::sockets` and `Io::sockets()` count every entity, not just sockets.

Status-file cross-check:
- **io.status:**
  - Confirmed: sections 5, 6 and 10 are not yet built.
  - Contradicted: "limits checked by `valid`". `Io::new` checks only `is_usable` (F6).
  - Missing: F1, F2, F4 and F7.
- **kernel.status:** consistent with the code. The harness duplication (F13) is not a design matter.
- **tls.status:** confirmed: the native face has no consumer. Missing: nothing on the design side; F14 is about test layout.
- **channel.status:** claims section 11 is built, which F12 contradicts. Missing: F2, F10 and F11.

Incidental:
- `layer.rs:585-586`: a `Window` request on a stream that is gone or closing submits `Op::Window` with `Fd(-1)` to the kernel, instead of telling `Window::Gone` from the ready list.
- `skein-ui-text/terminal.rs:235-254` hands io its completions before calling resume, against the order io.md 2 sets.
- Callers that check only `is_usable` (F6) let `Io::new` panic at `layer.rs:140` once `sockets ≥ 2^29`, when the flight count overflows u32.
- `processes.rs:240` calls `down` without checking `takes()`.
- `tests/io/tests/suite/memory.rs` and `process_memory.rs` each `assert!(peak <= CEILING)` after `Meter::check` has already asserted it, 8 times in all.

---

# A7-http-json

# A7: http and json (skein-http, skein-json, tests/http, tests/json, tests/protocol)

Scores: Cleanliness 3, Organisation 4, Taste 4, Naming 3, Primitives 2, Elegance 3, Economy 3, Complexity 4.

Verdict: The protocol machines are among the best code in the tree. The HTTP client, server, SSE reader and writer, and the JSON tokenizer follow the model closely: one `demand(before, after)` per transition, exhaustive matches, sized writes, and only 6 `#[expect]`s across 18.4k step lines, each with a reason. Their docs speak the design's words. The weak points are around that core. The same mechanism is written several times: the slice-fed tokenizer pump, head-field parsing on both HTTP sides, `Measure`, the calendar, CRC tables, and test-world neighbours and sweep helpers. The later additions (`parse`, `validate`, `json_text`, the collector and its tagged module, the collector world) are in a second, terser voice with their own vocabulary ("owner"), and some of them depart from json.md without the status file saying so. About 1,000 lines could go in the area, and as many again tree-wide once the JSON pump is a primitive.

Good:
- `crates/skein-http/src/body.rs`: one body reader for both sides (http.md 7). Reads below take the demand's own shape, `ends_partway` completes a split delimiter a byte at a time, and every comment says why.
- `crates/skein-json/src/tokenizer.rs:595` `demand(before, after, ..)`: the demand below is stated in one place after every transition (programming-model 5.4). Copy this.
- `crates/skein-http/src/client.rs:1-50`, `server.rs` module docs and `Limits` docs: each limit says what passes it and what it bounds; `largest_read`, `largest_room` and `valid` are named per the model (6.3).
- `crates/skein-json/src/utf8.rs`: a 50-line table-driven UTF-8 checker that carries state across cuts, as json.md 7 decides.
- Tests: step-test names read as sentences (`src/tests/exchange.rs`). Transcripts have expectations and print a draft when one is missing; their counts (47, 36, 39) match the design. Each sweep asserts that what it injected fell.

Top recommendations:
1. **A slice driver for the tokenizer** (`skein_json::tokenizer::Feed`: borrowed input, lib `stream::Held` semantics, `fn answer(&mut self, Request) -> Event`). Rebuild `parse` and `validate` on it, then migrate the pumps. Win: about 60 lines in skein-json and about 500 tree-wide; one idiom instead of three. Effort M.
2. **Share HTTP head parsing in `header.rs`**: `field`, `content_length`, the transfer-coding walk, `persistent`, header lookup, plus `Line`/`idle`/`Room` and `Measure`. Win: about 150 lines and 7 duplicated concepts. Effort S-M.
3. **Cleanliness and naming pass**: 9 stale doc sites, 6 doubled `Bounds` lines, delete the hand `impl Clone`, "owner" to "side above" (34 uses), rename two of the three `Position`s. Effort S; the docs become true again.
4. **Calendar to lib, IMF-fixdate to skein-http, RFC 3339 to skein-forge's root** (H7). Win: about 100 lines, and 15 GitHub→`forgejo::time` edges become neutral. Effort M.
5. **Test kit**: `meets`/`delivers`, `fault`/`draw`, `Seen`/`assert_fell`, `quote`/`unquote`, the transcript loader, and one stream-below neighbour (or `skein_world::stream::Wire`). Win: 300-400 lines. Effort M.
6. **Lib CRC** (const-fn tables for CRC-32 and CRC-32C). Win: about 240 lines in gzip.rs, and 4 expects gone from skein-kv. Effort S.

Findings:

F1. `parse` and `validate` are one loop, and both re-implement lib's `stream::Held::answer`. `diff` of `tokenizer.rs:127-178` against `parse.rs:26-84` (errors normalised) differs only in the first request (`Skip` vs `Next`), one event arm, the queue constructor, and `string: 0`. Each hand-codes Fill/Scan meeting from a slice (`remaining.get(..remaining.len().min(max))`), which `skein_lib::stream::Held::answer` (`crates/skein-lib/src/stream/held.rs`) already does for a box. skein-http even re-exports it (`sse.rs:59`). Fix: add a borrowed sibling of `Held` and a `Feed` that owns the tokenizer and two one-entry queues and answers one request per call. `parse` and `validate` become about 12 lines each.

F2. `writer::json_text` departs from json.md 4 and is unrecorded. `writer.rs:236-253`: json.md 4 lists the calls without `json_text` or `string_parts`, and says that what the data gets wrong "is a `Refusal` of the measuring pass". `json_text` instead returns `tokenizer::Error` directly, with only `TooDeep` folded into `Refusal`. It also validates twice per call (caller limits, then the remaining depth), so a document costs four full tokenizer runs, each allocating, across its two passes. `tokenizer::validate` (pub) and `Collector::counts`/`Counts` (test-only users) are also outside the design. Fix: validate once in the measuring pass with `depth: min(limits.depth, room)`, record failures as `Refusal::Json(Error)`, and skip validation when writing. Record the three calls in json.md 4.

F3. HTTP client and server duplicate head parsing and small machinery.
- `field()`: `client/head.rs:99-122` vs `server/head.rs:255-282`, identical apart from the error type.
- `persistent()`: `client/head.rs:194-200` = `server/head.rs:382-388`.
- `content_length()`: 250-278 vs 355-379.
- The transfer-coding walk: 211-240 vs 319-352.
- `Response::header` (`client.rs:192`) = `Call::header` (`server.rs:239`).
- `enum Line` + `fn idle`: `client.rs:395,501` = `server.rs:526,640`.
- `enum Room`: `client.rs:448` = `server.rs:627`.
- `struct Measure(Option<u32>)` + `put`, three byte-identical copies: `client/request.rs:169`, `server/response.rs:453`, `sse/writer.rs:457` (12 tree-wide).
- The list-walk loop `for _ in 0..=header.value.len() { let Some((element, next)) = header::element(..) ...}` appears 5 times.

Fix: move these to `header.rs` (an `Elements` cursor; `field` returning `Bad|Full`, which each side maps) and a small shared `line.rs`; `Measure` goes to lib. The rejection→status table is also written twice (`server.rs:321` `Rejection::status`, `response.rs:231` `status_line`).

F4. Two calendars and a Forgejo-owned time module (H7). `server/response.rs:250-345` writes IMF-fixdate by Hinnant's `civil_from_days` (about 95 lines with tables). `skein-forge/src/forgejo/time.rs:125` reads IMF-fixdate by rewriting it into RFC 3339 for `read`, and `write` (85-121) walks years 1970..=2554. Fix: move the civil date both ways to `skein_lib::Wall`, put `date::{write, read}` in skein-http, and move RFC 3339 to skein-forge's root.

F5. Hand-written trait impl in step code. `sse.rs:117-134` `impl Clone for Event` exists because lib's `stream::Up` (`stream.rs:71`) derives no Clone. Its only consumer is `vec![sse::Event::Ended; requests.len()]` at `tests/protocol/src/world.rs:358`. Programming-model 10.3 rules out hand-written trait impls, and no status file records this one. Fix: derive `Clone` on `Up` (it holds a `Box<[u8]>` and a `Copy` fault; it is not entity state), then derive on `Event`; or build the test vector by repetition.

F6. Stale doc comments (9 sites).
- `skein-http/src/lib.rs:15-17` says "An event's data goes up whole, and `sse::Data` reads it", against http.md 7's "An event's data goes up as a stream".
- `tests/http/tests/suite/stack.rs:9` and `tests/protocol/src/client_end.rs:4` say "through `sse::Data`". Both route the reader's `Event::Data` face and never construct `Held`.
- `tests/{http,json,protocol}/src/lib.rs` say "`tests/*.rs` … `tests/fuzzy_*.rs`", but the files are `tests/suite/` and `tests/suite/fuzzy/`.
- `tests/{http,json}/Cargo.toml` say "tests/memory.rs only", but the files are `tests/suite/memory.rs` (and `memory_server.rs`).
- `skein-json/src/lib.rs` and its Cargo description name only the tokenizer and the writer, not the collector, document or `parse`.
- `tests/http/src/transcript.rs:400` documents `quote` as "printable ASCII as it is", but the code keeps all non-control UTF-8; the identical copy in json documents it right.

F7. Doubled summary line on every `_bound` doc: `tokenizer.rs:94-95, 116-117`, `writer.rs:59-60`, `document.rs:263-264`, `collector.rs:287-288`, `sse/writer.rs:64-65` (6 in the area; `rg -c '^/// Bounds '` gives 26 tree-wide). Each opens "Bounds X." and then the real first sentence. Fix: keep one.

F8. Naming.
- "owner" means the side above in `collector.rs` (20 uses), `document.rs` (5) and `sse.rs` (8). The design and the other machines say "side above" (125+ uses), and in `tokenizer.rs:28` and `client.rs` "owner" means whoever closes the stream below. That is one word for two concepts.
- skein-json has three `Position`s: `tokenizer::Position` (pub, json.md 3.6), the collector's walk state (`collector.rs:198`), and the writer's grammar state (`writer.rs:113`). The last is the concept the tokenizer calls `Expect`.
- `sse::Data` is lib's `Held` under a second name.
- `gzip::bound` should be `encoded_bound` by model 6.3.
- Fix: rename the collector's to `At` and the writer's to `Expect`, and say "side above" throughout.

F9. The collector edits its caller's output queue. `collector.rs:345-358` pops `below`, which holds the oldest item (`queue.rs:94`), to take back the demand the tokenizer just pushed when the skip budget passes, then asserts that the next pop is the withdrawal. This holds only if `below` was empty on entry, and json.md 3.3/5.1 promise no such thing. Fix: give the tokenizer a private one-entry `Queue<Down>`, as `collector.events` already is for events, and forward from it; or check the budget before forwarding.

F10. `tagged::describe` is iterative deepening, run three times. `collector/tagged.rs:478-534`: for depth 1..=65,536 it makes two passes, restarts whenever the walk stack overflows, and shares one 65,536-step assert across all of them. `Collector::new` runs it twice (`collector.rs:221-222`, via `build` and again directly) and `collector_bound` once more (`collector.rs:299`). Fix: one pass with a stack sized by a static filter-depth constant, and keep the `Plan` in the collector. About -40 lines.

F11. The CRC-32 table is 256 lines (`gzip.rs:875-1131`, one entry a line), while `skein-kv/src/crc.rs` builds CRC-32C in a `const fn` under 4 file-wide expects. Fix: `skein_lib::crc` (lib may use `while` and generics), with both tables computed.

F12. Test-world helpers written several times.
- `meets`/`delivers`: `client_world.rs:441,1068` vs `server_world.rs:611,629`, empty `diff`; `delivers` is also `body.rs`'s private `answers`.
- `fault()`/`draw()`: 5 worlds.
- `struct Seen(BTreeMap<String,u32>)` + `note` + `assert_fell`: 6 sweeps (`fuzzy/{client,events,server,writer}.rs`, json `fuzzy/documents.rs`, protocol `fuzzy/worlds.rs`).
- `quote()`: byte-identical in `tests/http/src/transcript.rs:402` and `tests/json/src/transcript.rs:203`; `unquote` has two implementations.
- The transcript directory scan: 3 copies.
- Four hand stream-below neighbours (json `world.rs:342`, `sse_world.rs:291`, `client_world.rs:472`, `server_world.rs`) plus `tests/protocol/src/wire.rs` `Bottom` (272 lines), beside `skein_world::stream::Wire`, which oauth-accounts, forge machine and llm-connection use.

Fix: move the HTTP pair into `tests/http/src/lib.rs`; move the sweep tally and transcript loader to `testing/`; make one neighbour the kit.

F13. `stack.rs` overlaps the protocol world's client end. `tests/http/tests/suite/stack.rs:229-300` and `tests/protocol/src/client_end.rs:399-480` route client→reader→a tokenizer per event in nearly the same code. http.md 6 calls stack.rs "a small step towards the protocol worlds", and those worlds now exist. Fix: drive `client_end`'s router over a transcript-fed `Bottom`. About -200 lines.

F14. Two voices in the area. These docs differ in voice and terms from the tokenizer, client and server prose:
- `collector.rs`, `document.rs`, `parse.rs`, `sse.rs` ("Receives a stream event.", "Builds the reusable fields and data intake under stable nonzero chunk limits.", "A grammar/input refusal or a retained-count refusal");
- the `Kind` docs, "for the decoder or writer" ×10 (`document.rs:35-54`);
- the citation form `skein/docs/design/json.md, 5.x` (5 uses) against `json.md, N` (25).

The collector world (`tests/json/src/world.rs:675-796`) leaves `let _ = at;` (line 768) and a `use` inside a function, and its sweep tallies a `BTreeSet<String>` instead of `Seen`. Fix: one restyle pass.

F15. Limits validity takes four shapes in one crate pair (T-limits-shape):
- `client::valid` and `server::valid` → bool, per the model;
- the tokenizer and the SSE writer use `*_bound()` returning `None` (plus an assert in `new`);
- `sse::Reader` only asserts (`sse.rs:206`, "A chunk must be positive", http.md 4.2);
- the JSON encoder has none.

Fix: `valid(&Limits) -> bool` for each.

F16. Three ways into the document builder. `document.rs:282-349,418-427`: `push(token)`, `push_compact` and `push_long` each repeat the room check and the kind match, and `push` is `push_compact` over `token_kind`/`token_text`. About -30 lines.

Hypotheses:
- **H6, confirmed and refined.** About 25 whole-buffer pumps tree-wide (`Tokenizer::new` sites with a tick loop over one input), in three idioms for one kernel:
  - 6 hand-roll Fill/Scan: `skein-json` `parse.rs` and `tokenizer.rs` (validate), `skein-forge/src/github/decode/delivery.rs:282`, `skein-fake-forge-github/src/app.rs:256`, `skein-oauth/src/json.rs:28`, `testing/skein-lab-browser/src/json.rs:66`;
  - about 16 copy into an `Intake` and `meet` (`skein-forge/src/github/server/{app,queries,writes}.rs`, `skein-fake-forge-github/src/manifest.rs`, and fixtures and tests in forge, forge-setup, fake-forge-github, fake-peers, tests/forge);
  - 3 use lib `Held::answer` (`skein-fake-forge-protocol/src/observations/read.rs:142`, `tests/suite/memory.rs:109`, `skein-fake-forge-github/src/git.rs:265`).

  Those that collect every token (`app.rs:256`, `observations/read.rs:142`, oauth) can become `skein_json::parse` plus a `Document` walk. Those that step a streaming decoder need the closure-free `Feed::answer` of F1, since no closure or trait can carry the per-site step. `validate` and `parse` can share one loop (F1).
- **H7, confirmed.** See F4. skein-forge depends on lib and json only, so either it gains skein-http (it already parses HTTP header values by hand) or the calendar goes to lib and each format stays with its users. Cost to GitHub: 15 references to `forgejo::time` across 13 files under `crates/skein-forge/src/github/` (`header.rs:5`, `request.rs:667`, `server/{reviews,pulls,log,app,issues}.rs`, `decode/{reviews,app,checks,issues,pulls}.rs`, `server.rs:7`, `tests/fixtures.rs:57`), plus `skein-forge-connection/src/connection.rs:702`. The dialect cannot be split from Forgejo while RFC 3339 and HTTP-date live in a Forgejo module.

Tree-wide findings in the area:
- **T-worlds-as-kits.**
  - skein-json-world is a dependency of skein-http-world (dev), skein-protocol-world (`reference::parse`, `write`) and skein-llm (dev, `generate`).
  - skein-http-world is a dev-dependency of skein-llm-world (`tests/llm/tests/suite/fake.rs` ×6, `reference::{response,events,request}`).
  - All three are plain `{ path = ".." }` dependencies, not workspace entries.
  - testing.md 6 places "a reference reader of each" in tests/http, but not their use by others.
- **T-lints.** Three byte-identical 17-line `[lints]` blocks in `tests/{http,json,protocol}/Cargo.toml` (empty `diff`). The step crates take the workspace lints.
~~- **T-layout.** Conforms: `src/tests.rs` + `src/tests/`, `tests/suite/` + `fuzzy/`, no `qa_*`.~~
- **T-primitives.**
  - The JSON pump (H6).
  - The calendar (F4).
  - `Measure` ×3 (F3).
  - 110 u32→usize `expect`s in 4+ wordings ("u32 fits usize", "a u32 fits a usize", "u32 fits a usize", "fits a usize").
  - `writer.rs:517` uses `u64::try_from(len).unwrap_or(u64::MAX)` instead of `len_u64`.
  - `collector.rs:401` re-wraps `Env` per entry.
  - 5 `Env { now: Time::ZERO, wall: Wall::EPOCH, .. }` in skein-json and 16 in the worlds.
- **T-naming.** "actual": 0 in the area.

Status-file cross-check:
- **http.status.md.** Confirmed:
  - "6: selective collector": `client_end.rs:410,447` stacks a `Tokenizer`, not the collector http.md 6 names.
  - "metered protocol worlds": there is no memory suite in tests/protocol.
  - "fuzz targets".

  Misses: the hand `Clone` (F5), `sse::Data` being lib's `Held`, and the reader having no validity check despite "Built: 4.2 edge validity". "Trailers: not yet" is vague: both sides read and drop trailer sections (`body.rs` `Rest::Trailer`), so only keeping or writing them is missing.
- **json.status.md.**
  - Confirms the fuzz target is missing.
  - Misses `tokenizer::validate`, `Encoder::json_text` and `string_parts`, and `Collector::counts`, all outside json.md.
  - Misses `json_text`'s error channel departing from json.md 4 (F2).
  - Does not say the protocol world reads through the tokenizer rather than the collector (http.md 6), which the HTTP status file does record.

Incidental:
- `collector.rs:347`: with a non-empty `below` on entry, `below.pop()` removes the caller's item. The assert at 355 then fires, or a foreign demand is lost (F9).
- `tagged.rs:430`: `FILTER_STEPS` is spent across all of describe's depth retries, so a large but legitimate static filter could trip the startup assert. Low risk, since filters are static and tested.
- `writer.rs:236`: `json_text` validates, and can return `Err`, even on a measuring pass that was already refused.

---

# A8-codec

Scores: Cleanliness 3, Organisation 3, Taste 3, Naming 3, Primitives 2, Elegance 3, Economy 2, Complexity 3

~~Verdict: skein-codec's step code is the best part of this area. It has one bounded grammar, a two-pass token scan that allocates once, layout checks with no recursion, a small fingerprint encoder, and a patch machine that stops and resumes with no callbacks. That code reads well inside the model. The weight is around it. The design's mechanisms are written twice: 10.1/10.3 matching and comparison exist once in skein-codec's `plan` and again in skein-codegen's `diff`; the patch language has a strict reader and a lenient one; the schema exists as step tables and as a generator tree. The generator writes source as strings and then edits its own output to stay under the lints. tests/codec generates its fixtures twice and runs generated tests up to three times. Six `qa_*` binaries (182 tests, 2,941 lines) repeat step tests outside the gate. I estimate about 3.5–4k of the ~26k hand-written lines can go with no loss of confidence. The status file (6 lines) records none of the departures below.~~

Good:
- `crates/skein-codec/src/schema/tokens.rs:12`: `scan(text, Option<&mut List<Token>>)` counts on the first pass and fills on the second, so the table is allocated once at its exact size. Worth copying wherever a parser sizes a table.
- `crates/skein-codec/src/schema/layout.rs:85-145`: a field's type chain is walked forward and then priced backward in two loops, with no recursion and with checked sums. This is 7.1's "no recursion" done plainly.
- `crates/skein-codec/src/patch/run.rs:16-80`: `Step`/`Halt`, and `Old` and `Build` borrowed apart. A stopped machine in the subset, with no function pointers.
- `crates/skein-codec/src/schema/fingerprint.rs`: the 9.1 encoding in 160 lines, starting with its version byte.
- `testing/skein-codegen/src/emit_tests.rs:341`: the emitted table helpers are `include_str!("test_helpers.rs")`, one source for both. `tests/codec-patches/story/01..05` and `tests/codec/diff/*`: history and diffs kept as reviewed text.

Top recommendations:
1. **Delete tests/codec's second generation:** `build.rs`, `tests/suite/names.rs`, `[build-dependencies]`, and `crates/skein-codec/src/lib.rs:11-17`. Win: about 80 lines, 56 test processes, an 8.3k-line second compile and a build-script compile of codegen. Effort: under an hour (F1, F2).
~~2. **Fold the six `qa_*` binaries into the step tests and `tests/suite` as table rows, and delete the one-for-one duplicates.** Win: about 2k lines, 182 processes and one tier. Effort: about a day (F3).~~
3. **Give the tree shared helpers.** A `skein_codegen::drift(schema, code, goldens)` controlled by one write variable; `skein_scratch::Scratch` in the five hand-rolled scratch dirs; one copy each of `rust_type`, `snake`, `pascal` and the keyword list. Win: about 250 lines in skein, and about 300 more downstream (11+ hand drift tests). Effort: low (F8, F9, F10).
4. **Tidy tests/codec's fixtures.** Fold `scalars` and `secrets` into `every`. Move the patch story to its own owner crate, so the plain fixtures need no kept texts. Key the goldens by family. Move `tests/support` into `src/`. Win: about 700 generated lines, about 420 lines of `patches/mod.rs`, 10 `#[path]`s and four kept copies. Effort: low to medium (F11, F12).
5. **Keep one diff.** Expose plan's matching, comparison and change list (`patch/plan.rs:105-770`) and have codegen's `diff` only spell the result and propose renames. Win: about 480 lines in codegen, and one definition of 10.1–10.3. Effort: medium to high (F4).
6. **Make the generator emit structure, not text surgery.** Build statements as `Vec`s and chunk them; use `writeln!`; keep `held` in the tree. Win: about 100 lines and 6 module-wide expects. This is also the prerequisite for jig's "mirrors" request. Effort: medium (F6, F7, F17).

Findings:

F1. **tests/codec generates its fixtures twice, and the second generation is a tautology (H8).** `build.rs:8-35` regenerates all six schemas into `OUT_DIR`. `tests/suite/names.rs:5` includes the result. `build.rs:53,59` then compares each type's `encoded_bound`/`decoded_bound` with the committed module's. But `suite/drift.rs:19` already requires the committed text to equal `generate(schema)`, so this compiles the same text twice and compares the copy with itself. Cost: 36 per-declaration tests and 6 family tests. Because the included modules carry their own `#[cfg(test)] mod golden_tests`, all 12 generated tests run a second time as well: 54 extra processes (≈0.43 s of suite CPU at testing-strategy 8's 8 ms each). It also costs a second compile of 8,297 generated lines. One test per declaration also breaks 8's table rule. `build.rs:25-29` guards the old names `worst_case_bytes`/`worst_case_heap`, a past rename, which `src/check.rs:93-96` and the test `every_bound_name_is_reserved` already cover. Fix: delete `build.rs`, `names.rs` and the build dependency.

F2. **A step crate's unit tests compile a test crate's file.** `crates/skein-codec/src/lib.rs:14-17` has `#[path = "../../../tests/codec/src/generated/v1.rs"] pub mod generated_tests` and needs `extern crate self as skein_codec` (line 12) to make it work. The sample's two generated tests then run three times: in skein-codec, in skein-codec-tests' lib, and in the suite through F1. The dependency points the wrong way (crates/ reaching into tests/). Fix: delete lines 11-17.

~~F3. **The `qa_*` binaries repeat step tests outside the gate.** There are six: `tests/codec/tests/qa_{diff,fingerprint,patch,schema,value}.rs` and `tests/codec-patches/tests/qa_patch.rs`, with 182 `#[test]`s in 2,941 lines (`grep -c '#\[test\]'`, `wc -l`). They run only under `qa-skein` with `test-threads = 1`, and there is one case per test. Sampled names match step tests one for one:~~
~~   - `qa_schema` `held_words_before_colons_are_field_names` = `schema/tests.rs` `holds_and_pieces_can_name_the_next_field`;~~
   - `held_clauses_keep_external_names…` = `held_records_and_pieces_keep_their_names`;
   - `characters_are_checked_before_grammar…` = `character_pass_precedes_grammar…`;
~~   - codec `qa_patch` `header_reads_levels_marks_without_reading_block_grammar` = `patch/text/tests.rs:176`;~~
   - codec-patches `s12_1_baseline_guard…`, `s12_3_first_patch_requires_owner…`, `s12_3_rederive_keeps_values…`, `s12_3_no_value_types_get_field_sites…`, `s12_2_cancelled_patch_removes…` = `testing/skein-codegen/tests/suite/patch_run.rs:68,468,346,211,387`;
   - `s12_3_skeleton_appends_missing_function…` = `patch/skeleton.rs:87`.
   The `s12_3_` prefixes are pipeline vocabulary (T-naming). Fix: keep each case that no step test has, as a row in the nearest step table or in `tests/suite`, and delete the rest along with the tier.

F4. **10.1–10.3 are implemented twice.** `crates/skein-codec/src/patch/plan.rs:105-356` (matching), `357-646` (`payload`, `held`, `width`, `leaf`, `compare`) and `647-770` (`changes`) compute in step code what `testing/skein-codegen/src/diff/{matching.rs (161), compare.rs:1-145, walk.rs (177)}` compute again in ordinary Rust, over the same `skein_codec::schema` tables. Even the small helpers are duplicated: `same_shape` (plan.rs:136 and matching.rs:50), `parts` (plan.rs:148 and matching.rs:30), integer `width` and `integer_width`, and two private `Comparison` enums. Two definitions can disagree; the only thing that catches it is the check's `Unplanned` finding. Fix: make plan's matching (with renames as input), `compare` and `changes` `pub`, and have the diff spell `Op` lines and run proposals over them.

F5. **The patch language has two readers.** `testing/skein-codegen/src/patch/text.rs` (158 lines, "leniently") splits on whitespace and has its own comment stripper (`uncommented`, :111). `crates/skein-codec/src/patch/text.rs` (646 lines) is the grammar. The design's reason for having one grammar (7: "so there is one language") applies here too. Fix: expose skein-codec's `Parsed` (spans, line kinds, site and refusal numbering), since it already reads `?`, and drop the lenient copy.

F6. **The generator edits its own output.** `src/emit_functions.rs` (169 lines) re-reads emitted text line by line and splits functions by name, matching `"check" | "measure" | "encode" | "decode_from"` (:81). It renames locals with `tail.replace("decoded_{name}{suffix}", …)` (:146) and uses four unrelated thresholds (100 at :23, 99 at :85, 90 at :63, 70 at :117). Other edit sites: `emit_codec.rs:267` replaces `fn decode_from(limits:` with `_limits:`; `lib.rs:162` splits the whole module; `patch/module.rs:122` rewrites `pub mod refusal {\n}\n`. There are 96 `push_str(&format!(…))` in `emit_*` under 6 identical module-wide `#![expect(clippy::format_push_string)]`, while `patch/*` uses `writeln!`. Fix: emit each method as a `Vec` of statements, chunk before printing, choose `_limits` before writing, and use `writeln!` everywhere.

F7. **The generator keeps a second, lossy tree.** `src/lib.rs:74-88` `Type::Bytes(u32)`/`Secret(u32)` drops `held`, which `src/parse.rs:98-99` discards. `src/analysis.rs:75-84` `min_bytes`, and its `minimum` map, re-implement `Schema::fewest_bytes` (`crates/skein-codec/src/schema.rs:366`), which 7.3 calls "the generator's". `patch/golden.rs:93-143` then counts nodes over this tree. The tree is designed (7), but carrying it costs the duplication and loses the facts that typed piece helpers would need. Fix: let `analysis::Context` hold the `skein_codec::schema::Schema` and read `held`, `fewest_bytes` and `most_bytes` from it.

F8. **Small helpers are copied.**
   - `emit_bounds.rs:7-22` `rust_type` is byte-identical to `emit_types.rs:7-22`.
   - The keyword list appears at `check.rs:11-67` and `patch/module.rs:48`.
   - `snake` appears three times: `emit_limits.rs:7`, `patch/module.rs:34` and `tests/codec/tests/support/generic.rs:104`. The copies differ on `.` and `-`, although 12.3 says the skeleton's names are "as in the generated code's names".
   - `pascal` appears twice: `emit_limits.rs:22` and `support/generic.rs:11`, whose comment says "emit_limits::pascal".
   - `spelled`/`add` appear at `crates/skein-codec/src/schema.rs:420,426` and `patch.rs:428,431`.
   - The `CONTEXT` text appears three times: `patch/skeleton.rs:11`, again in full at `:82` (a test that compares the constant with itself), and in `tests/suite/patch_run.rs:268`.
   Fix: one `names` module in codegen exported as `#[doc(hidden)] pub`, and one `spelled` in `schema`.

F9. **There is no drift primitive, and there are three write switches.** Generated-code drift is written by hand in `tests/codec/tests/suite/drift.rs`, `tests/oauth/tests/suite/drift.rs` and `examples/ui/view/src/tests/drift.rs`, plus 8 downstream `drift.rs` files using `SKEIN_CODEGEN_WRITE`. Patch drift has `patch::current`/`goldens`. Three variables carry one concept: `SKEIN_CODEGEN_WRITE`, `SKEIN_DIFF_WRITE` (`suite/diff.rs:30`) and `SKEIN_PATCH_WRITE` (`patch/golden.rs:36`). All five reads use `std::env::var_os`, which avoids the `disallowed-methods` ban on `std::env::var` ("a run's settings come from its code"). Fix: `skein_codegen::drift(schema, code, goldens) -> Result<(), Error>` reading one variable, and record that variable as the exception in clippy.toml.

~~F10. **Five scratch directories are hand-rolled although `skein_scratch::Scratch` exists.** They are in `testing/skein-codegen/tests/suite/patch_run.rs:23`, `patch_check.rs:16` and `patch_generated.rs:19`, and in `tests/codec-patches/tests/suite/story.rs:36` and `tests/qa_patch.rs:21`, the last with an `AtomicU64` and its own `Drop`. Fix: use `Scratch::new(label)`. See also Incidental 1.~~

F11. **tests/codec's fixtures are one crate doing four jobs.**
   - **Redundant families.** `scalars` (Empty, Flag, Scalars) and `secrets` (Credential) are subsets of `every`, which by its own comment holds "every type of the language" (Blank, Light, Numbers, `Strings.key: secret 5`). They are used only in `suite/wire.rs:8,14,25`, `worst.rs:29-32` and `fuzzy/decode.rs:34,42`.
   - **Kept texts for families that never move.** Because the patch story makes the crate an owner, all six families get kept texts. Five of the six `schema/kept/*` files are byte-identical to `schema/*` (`cmp`); the sixth has identical content under another name. Constants for the four families that never move fill about 420 of `patches/mod.rs`'s 754 lines (`kept::lint_fixture`, `sample`, `scalars`, `secrets`).
   - **Shared golden directory.** `golden/v1/` holds 84 files from five families, so a type name shared across families would make two drift tests overwrite each other's file. Design 6 says "a file beside the code".
   - **One family, several names.** `lints-v1.schema` holds family `lint-fixture`, its module is `lints`, and its kept constants are `lint_fixture`. The sample family's module is `v1` (`src/lib.rs:11-12`), while the others are named by family, and `every`'s second version is `every_v2`.
   Fix: fold `scalars` and `secrets` into `every`; put the `every` story in an owner of its own (or in `tests/codec-patches`); use `golden/<family>-v<N>/`; name each module and file after its family.

~~F12. **tests/codec's shared harness lives in `tests/`, reached by `#[path]`.** `tests/support/{generated,generic,reprint}.rs` (464 lines) is wired in four different ways: `suite/main.rs:8`, `suite/support.rs:3,5`, `support/mod.rs` for `qa_patch.rs:15`, and `#[path]` in `qa_value.rs:5,7` and `qa_diff.rs:10,13`. Two of those need `#[expect(dead_code)]` (`support/mod.rs:4`, `qa_diff.rs:12`). testing.md 6 says "what its test binaries share is its library, in `src/`". Fix: move it to `tests/codec/src/support/`, which removes 8 `#[path]`s and both expects.~~

F13. **A lint workaround changes the generated API.** A record with more than three `bool` fields (`emit_types.rs:55`) gets `pub skein_bools: [bool; N]` in its `Parts` (`:148`), so callers write `skein_bools: [true, false, true, false]` (`tests/codec/tests/suite/wire.rs:196`). There are 806 occurrences downstream (`rg -c skein_bools` over jig, smith and temper). codec.md 4 ("hands its fields back") does not describe this. Fix: set `max-struct-bools` in clippy.toml, which is configuration rather than an allowance, generate named fields, and record the change in codec.md 4.

F14. **`context.rs` departs from the design without a record.** codec.md:1588 says "written once, empty, deriving `Default`". The command writes a hand `impl Default` under `#[expect(clippy::derivable_impls)]` (`patch/skeleton.rs:11`; `tests/codec-patches/patches/context.rs:10`), because root clippy.toml:92 refuses the derive. A hand trait impl is also out of the subset (programming-model 10.3). Meanwhile `tests/codec/patches/context.rs:3` simply derives it. Fix: decide the rule in codec.md (12.1 and the 11.5 `goldens<C: Default>`), for example a `Context::new()` and a `fn() -> C` parameter, and record it in the status file.

F15. **`plan.rs` is 1,736 lines with six concerns.** They are matching, comparison, the change list, line checks, value encoding (`:1048-1248`) and reference flags. Three of its four expects are `too_many_lines` (`:413`, `:1089`, `:1443`). Fix: `plan/{matching,compare,values,check}.rs`. Splitting out matching and compare is also the seam F4 needs.

F16. **Several descriptions, names and tests are stale.**
   - The `crates/skein-codec/Cargo.toml` description is "Bounded reads and decoding reasons", about 2% of the crate.
   - The codegen lib doc says "sections 3-6, no running machine state", but the crate also holds the diff and the command.
   - `tests/codec/src/lib.rs:1-3` says "sample family … and, later, wire entry points".
   - The tests/codec Cargo description and testing.md:356 say "its worlds", but there are no worlds (11.8: "no world").
   - The test `a_family_of_400_bounded_declarations_generates_in_half_a_second` (`src/tests.rs:119`) measures no time.
   - `crates/skein-codec/src/tests.rs:39` tests derived `PartialEq`.
   - `testing/skein-codegen/clippy.toml:1-6` is the world harness's text ("The world harness is ordinary Rust… a world's settings come from its scenario").
   - Six path dependencies (`testing/skein-codegen/Cargo.toml:19`, `tests/codec/Cargo.toml:11,16,20`, `tests/codec-patches/Cargo.toml:12,15`) ignore the workspace's `skein-codec`/`skein-codegen` entries.

F17. **jig's "mirrors" request would not fit the generator as it stands.** A mirrors declaration (domain types, both translations, typed piece helpers, a limits mapping) would meet three obstacles:
   - the tree drops `held` (F7), so typed piece helpers have nothing to read;
   - the text splitter only knows four method names (F6), so every new emitted method must be registered with it;
   - a grammar word (like `boxed`, outside the layout) touches the step parser, the fingerprint (9.1), the diff and the plan, twice because of F4.
   Good seams: `analysis::Context`, and `boxed` as the precedent for a word that changes Rust, not bytes. A per-record `encoded(&self) -> Box<[u8]>` would remove the measure-and-encode idiom: 112 times in tests/codec's generated tests, about 16 hand-written times in skein, and the 86 jig counts.

Hypotheses:
- **H8: confirmed and refined.**
  - The double generation is not needed (F1).
  - The `build.rs:25` guard is not needed (F1). The `worst_case_*` reservation itself is in codec.md 4 and could be dropped there.
  - The byte-identical kept copies are needed by the mechanism, but only because the patch story shares an owner with fixtures that never move (F11). Four of them, and about 420 lines of `mod.rs`, go if the story moves.
  - `skein-codec/src/lib.rs:16` is not needed (F2).
~~  - The six `qa_*` binaries should fold into the step tests and `tests/suite`, with duplicates deleted (F3).~~
- **How the tree-wide findings show up here:**
~~  - **T-layout:** `testing/skein-codegen/tests/suite` (2,572 lines) is a `testing/` crate with a `tests/` dir; T-layout's crate list missed `testing/`, which also has skein-lab-browser and skein-throwaway-forgejo. 6 of the 11 `qa_*` binaries are here. `tests/codec-patches` is missing from testing.md 6. skein-codec mixes per-module `tests.rs` (7 files) with `src/tests/piece.rs`; codegen uses `src/tests.rs`, inline modules in 5 files and `tests/suite`.~~
  - **T-worlds-as-kits:** `crates/skein-codec/src/lib.rs:16` is a `#[path]` outside its tree. tests/codec has 8 more internal `#[path]`s (F12).
  - **T-lints:** codegen's clippy.toml is a copied variant. There are 26 `#[expect]`s, 6 of them one module-wide wording. `plan.rs:182` is one of the 18 `manual_map` wordings. The `var_os` reads go around the env ban (F9).
  - **T-limits-shape:** the generated `check_limits(&Limits) -> Result<(), Problem>` is a fourth shape. `patch::Capacity` has no check, only `workspace_bound -> Option`.
  - **T-primitives:** `skein_codec::text_is_valid` (`lib.rs:88-145`) is the UTF-8 check lib should own. skein-codec has 18 `u32::try_from(x.len())`, and lib has no `len_u32`.
~~  - **T-naming:** the `qa_`/`s12_` names; `check` names two modules (`src/check.rs` and `src/patch/check.rs`); `Op` names two enums (`diff::Op` and `patch::Op`).~~

Status-file cross-check:
- **Lagging, confirmed:** 13 is not built (no maps, sets or variable-width integers).
- **What it misses:**
  - context.rs (F14), `skein_bools` (F13) and golden directories per version rather than "a file" (F11).
  - The 7.3 interface still shows `Secret(u32)`; the code has `Secret { bound, held }` (`schema.rs:175`), which is the "built" 3 the status lists.
  - codec.md:558 still promises "the old parser as an oracle", which is gone.
  - 7:321-323 and 7.2 carry history ("as before the parser moved").
  - The 11.5 example imports `pNNNN::{old, new}` so that "written code names no version", but the skeleton writes no such import, and `tests/codec/patches/p0001.rs:4` names `v1`/`v2`.
- **What it claims that the code contradicts:** nothing it claims is contradicted. It is 6 lines and records no departures at all.

Incidental:
1. `testing/skein-codegen/tests/suite/patch_check.rs` creates 7 temp crates and never removes them. `patch_run.rs:53` removes its crate only on success. With a reused PID, `create_dir(..).expect("unique scratch crate")` fails spuriously.
2. `crates/skein-codec/src/patch/run/heap.rs:14-15,27-28` prices `Stack::<Frame>`/`Stack::<Open>`, but `Build` holds `List<Frame>`/`List<Open>` (`run.rs:61-62`). The formulas are equal today; if one changes, the bound silently becomes wrong.
3. `testing/skein-codegen/src/main.rs:144` writes goldens to `<crate>/golden/v<N>`, but tests/codec's drift test uses `golden/every-v2`. The CLI and the drift test disagree, and two families at one version collide.
4. `testing/skein-codegen/src/parse.rs:33-34` hard-codes "1048576" and "4096" instead of `schema::MAX_TEXT`/`MAX_DECLARATIONS`.

---

# A9-llm

# A9 llm: skein-llm, skein-llm-connection, skein-fake-llm-{domain,protocol}, tests/llm, tests/llm-connection

Scores: Cleanliness 2, Organisation 2, Taste 3, Naming 2, Primitives 2, Elegance 3, Economy 2, Complexity 3

Verdict: The design around this code is careful: declared quantities derive every limit, decoding keeps only selected fields, memory has named ceilings, and the fake domain is a step crate that shares no types with the client. The code underneath carries the shape it had before it supported two dialects. The neutral layer lives in `openai/`, Anthropic imports from it, and the boundary is crossed by three copies of the output and terminal vocabulary with mappings that change nothing. The two dialects share what they should (filters, the reasoning-run rules, the error document and its classes), but through the wrong module. They also duplicate what they should share (budget charging, measure refusals, JSON member lookup) and organise the same jobs differently (peer codecs, request oracles). The withdrawn Codex and Anthropic-bearer routes are still the default in the worlds and the only routes the live suite covers. The client and connection state machines keep their lifecycle in flags and `Option`s, which programming-model 5.4 rules out, and no status file records this. Roughly 350 production lines and 250 test lines could go with no loss.

Good:
- `crates/skein-llm/src/derive.rs` + `derive/arithmetic.rs`: declared to derived limits, `Violation`/`Rule` naming each relationship, with each constant's capture provenance written beside it (derive.rs:104-152). This is programming-model 6.3 done right.
- `crates/skein-llm/src/filter.rs`: shared named caps and collector limits. Each dialect owns only its static filter (`openai/filter.rs`, `anthropic/filter.rs`). This is the right way to share a mechanism.
- `crates/skein-fake-llm-domain`: depends on lib only, has its own vocabulary (`api.rs`), and its lib.rs doc states the boundary. The independence is deliberate and stated: the fake's inflater shares nothing with the gzip encoder (`provider/inflate.rs`), and `documents.rs:642` re-parses the UUID independently.
- `tests/llm/tests/suite/memory.rs`: each ceiling is a named constant beside its one test (19 ceilings across the three memory files).
- `crates/skein-llm-connection/src/endpoint.rs:48-78`: `EndpointError` names each demand/cap pair.

Top recommendations:
1. **Make a neutral `wire` module and collapse the duplicate vocabulary (F1, F2, F3, F12).** Move `openai::{common, json, response::reasoning}` to the crate root. Delete `openai::{Stop, Usage, Failure}`, both dialect `Output` enums, `codex_outputs`/`anthropic_outputs` and `translate::{stop, usage, failure}`. Move each dialect's failure classes into that dialect. Embed the native limits in `client::Limits`. Win: about 300 lines and 6 types. Effort: M.
2. **Make the public route and an Anthropic key the world defaults (F6).** Today 337 helper uses ride Codex. Add a live test for a key. This removes risk from the planned Codex removal. Effort: M.
3. **Export skein-http's header grammar and a head measure (F8).** About 80 lines, and one drift-prone parallel spelling of the HTTP head goes. Effort: S.
4. **Sweep dead and undesigned API (F11).** About 110 lines and 6 names. Effort: S.
5. **One validator in llm-connection, and test limits from the world (F10, F14).** About 115 lines. Effort: S.
6. **Bring the status files and docs up to date (F7, F9, status section).** Record the 5.4 departure (F5) and plan the state refactor; the refactor itself is effort L. The doc fixes are effort S.

Findings:

F1. **Dialect-neutral code lives under `openai/`.**
Evidence:
- Anthropic re-exports the OpenAI module's types: `anthropic.rs:8` re-exports `openai::{DecodeError, Failure, Json, Limits, ProviderError, RateLimit, Stop, Usage, classify}`. 44 lines under `anthropic/` name `openai::` (`rg -c 'openai::' anthropic anthropic.rs`).
- Anthropic code reaches into the OpenAI response module: `anthropic/response.rs:286,288` call `openai::response::reasoning::{charge, append}`; `anthropic/request.rs:147,438` call `replay_item` and `write_replay`; `anthropic/response.rs:725` `decode_error` only forwards to `openai::decode_error`.
- Anthropic's failure classes live in the OpenAI module: `openai/common.rs:322-372` (`anthropic_failure`, `unified_reset`), and `RateLimit::observe` parses the `anthropic-ratelimit-unified-*` headers (common.rs:212-219).
- `client::Limits::native()` returns `openai::Limits` for every dialect (client.rs:73).
- `translate.rs` (1,317 lines) sounds neutral but is the Responses encoder ("Bridge between the provider-neutral vocabulary and Codex documents", :1). Anthropic's equivalent is `anthropic/request.rs`.

llm.md 4 names `openai::common::*` bounds but never says Anthropic depends on OpenAI. This is the same shape as H1 (GitHub on Forgejo).
Fix: a root `wire` module (Limits, DocumentError, ProviderError, RateLimit, json view helpers, reasoning runs, clip/measured/limit_detail, error document). Each dialect's classes go to its own module, and `translate.rs` becomes `openai/encode.rs`.

F2. **A mirror vocabulary crossed by identity mappings.**
Evidence:
- `openai::{Stop, Usage, Failure}` (common.rs:147-193) duplicate `crate::{Stop, Usage, Failure}` (types.rs:138-331); the crate's `Failure` is a superset.
- `translate.rs:795-827` maps them variant for variant, and every `classify` call is wrapped in that mapping (dialect.rs:156, client.rs:1430,1468).
- `openai::Output` (response.rs:123) and `anthropic::Output` (response.rs:74) are the same 8-variant enum. `dialect::codex_outputs` (:176) and `anthropic_outputs` (:207) are the same 28-line loop through a temporary queue into a third copy, `dialect::Output` (:13), which `client::output` (client.rs:1276) maps a fourth time.
- Downstream, the fake's `anthropic_stop` and `openai_stop` (documents.rs:614-630) are identical, because `anthropic::Stop` *is* `openai::Stop`.

Fix: dialects emit the crate's `Stop`/`Usage`/`Failure` and one shared output enum, converting `Part` to `Block` where they emit. Win: about 180 lines, 5 types, and 2 queue hops per event.

F3. **The dialects duplicate helpers they should share.**
Evidence:
- `anthropic/request.rs:215-262` (`charge`/`text`/`json`/`measured`) re-implements `translate.rs:438-484` and `openai/common.rs:397`. The mapping from `writer::Refusal` to a cap is spelled 7 times (`rg 'Refusal::TooLong'`, production).
- The member lookup `json::value_at(v, json::required(v, name)?)` is spelled out 74 times. It already exists twice as a private helper: `anthropic/peer.rs:264` `required` and `anthropic/response.rs:744` `field`.
- `openai/identity.rs:14-26` defines its own `Header`, while `anthropic/identity.rs:9` uses `skein_http::Header`.

Fix: one `charge`/`measured` in `wire`, a `json::member`, and `skein_http::Header` everywhere. About 70 lines.

F4. **The same peer-side job is organised two ways.**
Evidence:
- Anthropic has `peer.rs` (`decode_request` returning the neutral `Prompt` plus marks, and `encode_event`).
- OpenAI's `decode_request` (request.rs:213) returns the native `openai::Request`, and its peer encoders sit inside `openai/response.rs:971-1340` (about 370 lines: `encode_error`, `encode_failed`, `Echo`, `encode_peer_event`, `encode_completion`, attribution).
- OpenAI also keeps a second, owned request writer (`openai/request.rs:98-212`), used only as a test oracle against the borrowed one ("The old owned translator is a test oracle only", translate.rs:851). Its `#[cfg(test)]` helpers (about 145 lines) sit in the production file. Anthropic has no such oracle.

Fix: an `openai/peer.rs` that mirrors Anthropic's. Then either move the oracle into tests, or retire it once captured bytes pin the encoder (about 260 lines).

F5. **States are spread over flags and `Option`s (programming-model 5.4); unrecorded.**
Evidence:
- `client.rs:245`: `State` has 8 variants and no data. `Client` has 39 fields (:275-314), including `outcome`, `asked`, `close_sent` and `sse_closed`, plus `decoder`/`content`/`call`/`upload` options that mean something in some states only.
- Transitions are 16 `client.state == State::X` tests and 7 bare assignments, never the `mem::replace` idiom.
- `skein-llm-connection/src/call.rs:50`: `Connection` has a `phase` with Option `call`/`socket`/`tls`/`idle_at` and 3 bools beside it.

Model 5.4: "no field is meaningful in some states only… never a lifecycle spread over flags and `Option` fields". Neither status file records this.
Fix: record it now, then move to data-carrying variants (`Streaming { decoder, content }`, `ErrorBody { error }`).

F6. **The worlds default to the withdrawn routes.**
Evidence:
- `tests/llm/src/lib.rs:31-62`: `limits()` and `call()` default to `Endpoint::codex()` with an account id. `anthropic_call` (:78) uses `Credential::anthropic`, the withdrawn bearer.
- Uses in `tests/llm/tests/suite`: `call(`/`limits()` 337, against 82 for `public_call`/`anthropic_call`/`limits_for`. `OpenAiCodex` appears 104 times in tests/llm plus llm-connection; `Credential::key`/`Scheme::Key` 11 times.
- `tests/llm/tests/live.rs:58,88` covers only Codex and the Anthropic bearer.
- `derive.rs:152` builds the public route's constants as `OPENAI = { .., ..CODEX }`.

llm.md 4.6 and 4.7 say these routes take no new caller.
Fix: make the defaults the public route and a key, put the Codex cases in one file, and add a live test with a key.

F7. **Naming drift.**
Evidence:
- Docs say "subscription"/"ChatGPT"/"Codex" for the shared dialect: lib.rs:4 ("Supports ChatGPT's Codex Responses and Anthropic Messages subscription routes", which omits the public route), openai.rs:1, anthropic.rs:1, translate.rs:1.
- Names say "codex" for code that serves both OpenAI routes: `codex_outputs`, `validate_codex` (translate.rs:42), and `Exchange::new_with_codex_echo` (fake.rs:267), although the echo works on both routes (fake-llm.md 3.1).
- Public-route callers receive "malformed ChatGPT stream" and "incomplete ChatGPT stream" (openai/response.rs:239,316).
- `Provider` means a route in skein-llm (types.rs:10 calls it "a concrete wire dialect", yet it holds both `OpenAi` and `OpenAiCodex`) but a dialect in the fake (`documents.rs:18`).
- One error type has three paths and two names: `DocumentError`, `openai::DecodeError`, `anthropic::DecodeError` (lib.rs:27). `Json` also has three paths.
- `Delta::ToolArguments` and `Output::ArgumentsDelta` name the same thing.
- llm-connection sees three `Phase` types: `llm::Phase`, `call.rs:29` and `deadlines.rs:12`.

Fix: use "Responses" for the dialect and "route" for `Provider` (rename it or fix its doc), and give each type one path.

F8. **skein-http's header grammar is re-implemented.**
Evidence:
- `client.rs:1817` (tchar set), :1829 (value check), :1919 `trim` and :1874 media-type parse.
- `skein-fake-llm-protocol/src/provider.rs:180-188` repeats the tchar and value checks.
- skein-http has `is_tchar`, `is_field_byte` and `trim` as `pub(crate)` (header.rs:29,36,47).
- `client.rs:1694-1766` `head_length` rebuilds the HTTP/1.1 head's length from string literals (`"POST "`, `" HTTP/1.1\r\n"`, `"Content-Length: \r\n"`). This runs parallel to skein-http's own measure in `client/request.rs:68` and to `headers()` (:1629): three spellings of one head.

Fix: export the header predicates and a borrowed head-length measure from skein-http, build the field list once, and measure it.

F9. **Undesigned wire behaviour.**
Evidence:
- Every Responses request sends `"text":{"verbosity":"low"}` (translate.rs:105-110, openai/request.rs:138-142). The value comes from `openai::identity::VERBOSITY`, a module llm.md 4.6 calls opt-in data "never installed by default". llm.md 4.8 does not list the field.
- `dialect.rs:73` cites "llm.md 4.8: response.done terminates only the Codex route", but llm.md has no `response.done` (`rg` count: 0).

Fix: design both or drop them, and record the choice in llm.status.

F10. **llm-connection has two validators, and they disagree.**
Evidence:
- `limits.rs:23` `valid` and `component.rs:70-149` `Component::new` repeat the zero/`connections<calls`/`per_endpoint`/`request_head`/reservation checks. `new` then calls `valid` (:147), which recomputes `request_head`.
- `valid` omits the plaintext-loopback and TLS/io demand checks (:113-143). So `valid()` can return true for limits that `new` refuses.

Fix: one `check() -> Result<(), EndpointError>`, with `valid = check().is_ok()`. About 30 lines.

F11. **Dead or undesigned public API.**
Evidence:
- `Component::admit_with_reasoning_drop` (component.rs:175) has no caller anywhere in `~/src/fabrig/main`.
- `Component::admit` (:164) is called only from crate tests and is not in llm-connection.md 4's interface.
- `provider::head_bound` (fake protocol provider.rs:984) has no caller.
- `openai::encode_completion` (response.rs:1258) is called only from a crate test.
- `openai::identity::{PROVENANCE, ACCOUNT_HEADER, SESSION_HEADER}` are unused. `SESSION_HEADER = "session_id"` also contradicts the client's `session-id`.
- `client::Limits.drop_reasoning` (client.rs:62) is a default that only `Client::prepare` and `admit` read; the component always passes `Start`'s value (component.rs:646).
- `Exchange` has 6 constructors (fake.rs:259-340), and `prepared_with_codex_echo` is only an internal hop.

Fix: delete these, and reduce `Exchange` to `{new, prepared}(…, Options)`. About 110 lines.

F12. **`client::Limits` re-lists `openai::Limits`.**
Evidence: `native()` copies 16 fields one by one (client.rs:71-95).
Fix: an embedded `native: wire::Limits` field. About 25 lines.

F13. **Test layout and fixtures.**
Evidence:
- Four step-test conventions in the area: sibling `tests.rs` (7 files), `*_tests.rs` (`anthropic/request_tests.rs`), inline modules (14 in 13 files; translate.rs's inline module is 489 lines), and test-only child files (`respond/failures.rs`, `respond/played.rs`).
- Fixtures sit under `src/` and are read five ways: `CARGO_MANIFEST_DIR` (openai/tests.rs:101), `include_bytes!` (derive/tests.rs:29), `paths::package().join("../../crates/skein-llm/src/…")` (tests/llm archives.rs:9, derive.rs:74), `include_str!("../../../../crates/…")` (selective.rs:17), and `fs::read` (translate.rs:933).
- The archived-head headroom walk is written twice: crate `derive/tests.rs:26-62` and `tests/llm/tests/suite/derive.rs:69-115`.
- `#[path = "pieces.rs"]` (fake protocol provider.rs:14) puts a child of `provider` at the crate root.

Fix: one fixture loader, drop the crate's copy of the headroom test, and move the file to `provider/pieces.rs`.

F14. **The connection world's limits are repeated literally.**
Evidence: literal `Limits { io: skein_io::Limits {…} }` blocks in `src/world.rs:85`, `simulated.rs:21` and `tests/suite/{protocol.rs:13, fake_protocol.rs:18, failures.rs:15, memory.rs:19}`. The three in protocol, fake_protocol and failures equal `world::limits(1)` except for a 10 s idle keep. `src/plaintext.rs` is a 3-line re-export.
Fix: call `world::limits(1)` and override the keep. About 85 lines.

F15. **The skein-llm world has no replay check.**
Evidence: tests/llm does not depend on skein-world, and its harness has no trace or digest check (`rg 'trace|digest' tests/llm/src`: 0). tests/llm-connection uses `skein_world` 44 times, including `assert_replays`. Testing-strategy 6 requires that a seed replays to the same trace.
Fix: run the World/Exchange stories under `assert_replays`, or record the gap.

F16. **Heap sums kept as edge bounds.**
Evidence: `openai/common.rs:82-125` `decoded_bound` sums with ×2/×4/×8 factors and `size_of::<Compact>()`, and `anthropic.rs:26` adds to it. The only production caller is the fake (provider.rs:1013), which only tests `.is_some()`. llm.status claims "without a client heap sum".
Fix: replace these with representability checks.

Hypotheses:
- **H9: refined.**
  - "actual": confirmed at exactly 23 comment lines in tests/llm. The word also fills 142 string-literal lines and 45 identifier lines (228 occurrences). It comes from the design's own text (fake-llm.md 11 uses, llm.md 5), so fix the design and the code together.
  - Size: tests/llm has 9,781 test lines over 1,600 src lines. About 3,190 of those test the fake crates (`fake*.rs` and `bounded_peer_memory.rs`), which have no world of their own. fake-llm.md 5 designs `skein-llm-world::fake::Exchange` as a kit, used in 31 downstream files.
  - `generate`: skein-llm uses `skein_json_world::generate` in a single test (translate.rs:877-925). That test compares against the crate-private owned oracle, so it cannot move out of the crate. `generate.rs` (264 lines, lib plus json only) belongs in a `testing/` kit that json-world re-exports.
- **Tree-wide findings in this area:**
  - T-layout: four step-test conventions (F13). One binary sits outside `tests/suite`, `tests/llm/tests/live.rs`, as designed.
  - T-worlds-as-kits: `llm-world` is depended on by tls, oauth, fake-peers and llm-connection; `#[path]` once (F13).
  - T-lints: the two world crates have different hand lint sets (12 and 6 entries); llm-connection does not deny `disallowed_types`/`disallowed_methods`/`allow_attributes`. The area has 45 `#[expect]`, 5 `manual_map` with two wordings.
  - T-limits-shape: `valid` versus `Component::new` (F10); `projection_bound() -> Option<()>` is a check named as a bound (filter.rs:42).
  - T-primitives: 69 `usize::try_from(..).expect` (53 say "u32 fits usize"), 13 `u64::try_from(..).expect` against 28 `len_u64`, and 24 `Env { now, wall, limits }` rewraps (call.rs 12, client.rs 7, provider.rs 5).

Status-file cross-check:
- **llm.status:** confirmed: "withdrawn routes still present" (242 Codex/ChatGPT mentions in skein-llm `src`). It misses F5, F9 (verbosity, `response.done`), F15, a live suite that covers only withdrawn routes, and the README's stale Anthropic section. The README presents the bearer and `anthropic-beta` as the way, and carries the history note "before the user's doubling of 10-22". The claim "without a client heap sum" is contradicted by F16.
- **llm-connection.status:** misses F5, F10 and the undesigned `admit` API.
- **fake-llm.status:** it says "Not yet: request-cued scripts, fallback answers, record-free peer and exported test root", and "2.2: standalone byte peer". The code contradicts all five:
  - request-cued scripts and fallback: `api.rs:316-324` `Scripts { cued, asked, otherwise }`, and `domain.rs:344,352` `played`/`played_by_input`;
  - record-free peer: `skein-fake-peers/src/llm.rs:159` `without_records`;
  - exported test root: `transport.rs:16` `ROOT`;
  - standalone byte peer: `llm.rs:185` `Peer::release`.
- **Not recorded anywhere:** the fake reads requests with the product's own codecs (`openai::decode_request`, and `anthropic::decode_request`, which returns `skein_llm::Prompt`). fake-llm.md 1 and 4 design this, but it conflicts with testing-strategy 1 ("A fake shares no types with what it stands in for").

Incidental:
- The request's header count is checked against the response cap and refused as `Cap::ResponseFields` (client.rs:150-157, 1604-1621). The refusal names the wrong bound.
- Public-route `Protocol` failures carry "ChatGPT stream" in their detail (F7).
- `valid()` passes plaintext to a non-loopback address and TLS/io mismatches that `new` refuses (F10). A startup check that relies on `valid` is fooled.
- In `head_length`, arithmetic overflow becomes `Error::Invalid` rather than a named limit (client.rs:1703-1764).

---

# A10-oauth

# A10 oauth: skein-oauth, skein-oauth-accounts, skein-oauth-issuer, skein-fake-oauth, tests/oauth*

Scores: Cleanliness 3, Organisation 3, Taste 3, Naming 3, Primitives 2, Elegance 3, Economy 2, Complexity 3

Verdict: The crate boundaries follow the design: client, driver, fake and stateless issuer. The comments are precise and cite the design. `skein-oauth-issuer` is the best code in the area. The two older crates have drifted from the model, though:
- The client's state is split between `Phase` and `Option<Exchange>`, and the accounts component keeps its lifecycles in flags and `Option`s beside the state.
- `skein-oauth` keeps its own copy of skein-json's document type.
- None of the crates holds credentials as `skein_lib::Secret`, which already exists. That gap causes most of the area's `#[expect]`s and several workarounds.
- The fake shares the client's types and codecs, against testing-strategy 4.
~~- The tests repeat themselves: two qa_* binaries copy suite tests, tests/oauth-accounts has two owner harnesses, and the test files use three layouts.~~

About 1,300–1,500 lines could go with no loss of behaviour or test confidence. oauth-accounts has no design doc of its own: oauth.md section 6 ("The driver") covers it.

Good:
- `crates/skein-oauth-issuer` (962 code lines):
  - Stateless, with its checks in the design's numbered order (requests.rs:14-63).
  - Credentials are `Secret`, every type derives `Debug`, and there is no `missing_debug_implementations` expect.
  - Every output is measured, then written.
  - Step tests are per area under `src/tests/`.
- `Client::admits` and `Redirected` share one check (`check_redirect`, machine.rs:389), as oauth.md 3 asks ("One check serves both, so they cannot differ").
- The fake's `openid::Fault` (skein-fake-oauth/src/openid.rs:24) has one scripted fault per ID-token check of oauth.md 2.2.
- The kept record is a schema (`schema/oauth-v3.schema`), with goldens and a drift test.
- The accounts component uses lib's `Lifecycle` and `earliest`, and every variant in `boundary.rs` documents its terminal.
- tests/oauth-issuer runs two independent readings of the protocol against each other (oauth-issuer.md 1).

Top recommendations:
1. **Hold credentials as `skein_lib::Secret`** in skein-oauth's types (SavedToken, RefreshState, TokenResponse, Registration.client_secret, Redirect, HttpRequest/HttpResponse bodies), then derive Debug, and Clone on Registration.
   - Win: about 20 of the area's 25 missing-debug expects, `copy_registration`, 6 push-match workarounds, and record.rs's `secret`/`exposed` helpers (about 80 lines).
   - Effort: medium (about 31 field uses in 16 downstream files become `.expose()`).
2. **Rebuild `skein_oauth::Json` on `skein_json::parse` and `Document`**, as skein-llm's openai/json.rs does. Win: about 250 lines and one concept. Effort: medium.
~~3. **Delete `tests/oauth-accounts/tests/qa_paste.rs` and fold `tests/oauth/tests/qa_paste.rs` into `tests/suite`.** Win: about 350 lines and 2 binaries. Effort: small.~~
4. **One test layout:** `src/tests/<area>.rs` in skein-oauth and accounts, and the oauth-issuer world harness in `src/lib.rs`. Win: removes 10 `#[path]` and one copy of the fixture keys. Effort: small.
5. **Give the fake its own vocabulary and writer**, then delete skein-oauth's server-only codecs and its dead refresh-request codec. Win: about 120 lines, and conformance with testing-strategy 4. Effort: medium.
6. **Make each state one value** (model 5.4): client `Phase` variants own their exchange, the accounts `Purpose::SignIn` owns the sign-in-only fields, and `State::Record`'s 5 bools become enums. Win: about 10 impossible-state fallbacks and 5 parallel lists. Effort: large.

Findings:

**F1. Credentials are raw `Box<[u8]>`, although `skein_lib::Secret` exists for this (lib.md 6: "a record that holds one prints, logs and traces without it").**
- Evidence:
  - 25 of the tree's 39 `missing_debug_implementations` expects are in this area (`rg -c`): skein-oauth 12, accounts 7, fake 6.
  - All 9 "credential values must never occur in traces" reasons are here (jwt.rs:8, record.rs:20,33, documents.rs:7,13, boundary.rs:10,47,198, component.rs:122).
  - The generated codec already holds `Secret`. record.rs:184-231 converts it to raw bytes and back with `secret()`/`exposed()`, under 2 manual_map expects.
  - Registration has no derives, so accounts.rs:61-74 hand-clones it (`copy_registration`, 4 callers).
  - `List::push`'s `Err(T)` cannot `.expect` without `T: Debug`, hence 6 matches like `Ok(()) => {} Err(State::Empty | …) => unreachable!` (component.rs:159-166, component/accounts.rs:177-183).
  - The issuer shows the fix works: lib.rs:19-37 derives Debug over `Secret`.
- Fix: as recommendation 1. The component still holds `CryptoRng`, which has no Debug, so give it a redacting Debug in lib.

**F2. `skein-oauth/src/json.rs` (397 lines) duplicates skein-json.**
- Evidence:
  - `Json::from_bytes` (json.rs:23-77) is a hand whole-buffer tokenizer pump, the same loop as `skein_json::parse` (parse.rs:26).
  - `Json { tokens: Box<[Token]> }` with `field`/`span`/`value_at` (json.rs:263-376) re-does `Document::member` and `value_end` (document.rs:179,215).
  - `validate` and its `Frame` enum (json.rs:13,207) are copied in skein-llm/src/openai/json.rs:27.
  - `admit` (json.rs:103) re-validates and re-measures a document that `from_bytes` already checked, once per decode.
  - `Collector` is exported and used nowhere outside the crate (`rg skein_oauth::Collector|oauth::Collector`: 0).
- Fix: `Json { document: Document }` over `skein_json::parse`; drop Collector.

**F3. The fake shares the client's types and codecs** (testing-strategy 4: "No shared types. A fake's vocabulary is its own"; 4.1: "two sides written together can share one misreading").
- Evidence:
  - skein-fake-oauth/src/lib.rs:9-12 imports HttpEvidence, HttpRequest, HttpResponse, Json, Limits, OAuthError, TokenResponse, challenge, encode_error and encode_response.
  - lib.rs:42-55 copies record.rs:156-168's `v3::Limits` mapping.
  - skein-oauth/src/lib.rs:3 says "Its server codecs serve the fake peer": `encode_response` and `encode_error` have no other users (`rg`).
  - The departure is visible only as an edge in docs/design/README.md:88, not in oauth.status.md. The issuer was built to share nothing (oauth-issuer.md 1).
- Fix: the fake writes its own documents (as the issuer does in documents.rs) and tests/oauth translates. Then delete skein-oauth's server codecs, or record the exception in the status file.

**F4. Dead production code.**
- Evidence:
  - `RefreshRequest`, `encode_request`, `write_request`, `validate_request` and `decode_request` (documents.rs:6-62) are used only by src/tests.rs. The machine builds its own body (machine.rs:734-800).
  - `RECORD_VERSION` (record.rs:10) has no user in skein, jig, smith or temper.
- Fix: delete them (about 55 code lines and 30 test lines).

**F5. The client departs from model 3 and 5.4, unrecorded.**
- Evidence:
  - `Phase` (machine.rs:189) and `active: Option<Exchange>` (:232-236) are two values for one state.
  - Exchange's `fetch`, `visit`, `keys`, `verifier` and `nonce` (:217-229) hold only in some phases. `exchange.fetch.is_some()` stands in for the phase (:628, :669, :683).
  - The impossible empty-`active` case is answered with a peer-facing `Failure::Malformed` (:393-394, :474-475, :655-656, fetch.rs:108).
  - `Client::step(&mut self, event, out)` (:263) takes no `Env`: `now` travels in 6 events and in HttpResponse, and the limits live in the machine. The fake does the same (lib.rs:280).
- Fix: phase variants own their data, transitions use `mem::replace`, and the signature becomes `step(&mut self, env, event, out)`.

**F6. The accounts lifecycles are spread over flags and `Option`s (5.4: "never a lifecycle spread over flags and Option fields beside the state").**
- Evidence:
  - `State::Record` (component.rs:100-121) has 5 bools and an `Option<u64>`.
  - `Exchange` (exchange.rs:41-64) has 19 fields. 7 of them are sign-in-only (`sign_in_deadline`, `visit`, `state_refusals`, `sign_in`, `return_location`, `subject`, `listener`), and `lock_candidate` copies `Stage::Keeping`'s pair.
  - keeper.rs:24-42 uses `cancel`+`cancelled` and `next`/`pending`/`waiting_grant`/`closed`.
  - The Component keeps five parallel per-account Lists (component.rs:124-133), read by index 124 times (`rg '\.X\.get(_mut)?\('`: accounts 33, names 10, sources 24, bindings 35, keepers 22).
- Fix: one `List<Slot>`, `Purpose::SignIn { … }`, and enums for the record's hold and refresh state.

**F7. Three step-test layouts plus `#[path]` (H10).**
- Evidence:
  - skein-oauth uses tests.rs, a crate-root `machine_tests.rs` with `machine_tests/` (6 files, whose `mod` lines are scattered mid-file at :733-738 and :813-815), and an inline module in pkce.rs:23.
  - accounts' tests.rs pulls in 7 `src/*_tests.rs` files by `#[path]` (:393-608).
  - component.rs:26 has a `#[path]` to `component/nonce_tests.rs`, the default location anyway.
  - exchange.rs:570 has an inline module.
  - The issuer uses `src/tests/<area>.rs`, which testing.md 6 reserves for lib, io, JSON, HTTP and TLS.
- Fix: use `src/tests/<area>.rs` everywhere (the module names already match) and amend testing.md 6.

~~**F8. The qa_* binaries copy suite tests.**~~
- Evidence:
~~  - 10 of the 12 tests in tests/oauth-accounts/tests/qa_paste.rs (159 lines) run the same `Story` (Paste … Restart) as suite/sign_in.rs:244-365. The suite copies assert exact fact lists; the qa copies only count.~~
~~  - tests/oauth/tests/qa_paste.rs (269 lines) re-tests `admits` and overlaps machine_tests/admits.rs.~~
- Fix: delete the accounts copy, after moving `RequiredScopes` and the faulted sweep into suite and suite/fuzzy. Fold the oauth copy into suite.

**F9. tests/oauth-accounts has two copies of its owner harness.**
- Evidence:
  - The `Client` and `impl Host` in world.rs:277-788 and sign_in.rs:286-519 share 211 distinct lines over 40 characters (`comm` of the sorted lines). This includes the `iterate` loop with its comment, verbatim (world.rs:668-700 and sign_in.rs:415-445).
  - Both files define Contracts, Fact, Process, Judge, assert_replay and protocol.
  - sign_in.rs:519 says "The two actual processes" over 3 variants (a stale copy).
  - world.rs:278-292 bolts per-scenario hooks onto one host (`stop_at_store: Option<bool>`, `retry_load: Option<()>`).
- Fix: one host with a scenario enum (about −250 lines).

**F10. Tests break testing-strategy 10.**
- Evidence:
  - Table bodies branch on the row: suite/world.rs:27-44 (`match story`) and four `for protocol in [false, true] { if protocol …` in suite/sign_in.rs:76-160.
  - A test sits in tests/oauth-issuer/tests/support/mod.rs:631-664: it edits a form body with `str::replace` and branches on `expired`.
  - `run()` asserts each story's outcome inside the harness (:599-613).
  - The 11 one-line `valid_refuses_*` tests (machine_tests.rs:828-884) are one behaviour; it belongs in a table.
  - Harness calls take positional bools: `sign_in::run(seed, story, faulted, large, memory)`.
- Fix: as the rules say.

**F11. Percent coding and loopback-URI parsing are written several times.**
- Evidence:
  - Encoders: skein-oauth redirect.rs:273-327 and issuer form.rs:117-152. `plain` and `unreserved` are identical, and both keep a local `HEX` table.
  - Decoders: accounts redirect.rs:83-105, issuer form.rs:71-115 and fake lib.rs:666-694.
  - URI parsing: skein-oauth redirect.rs:89-188, issuer settings.rs:124-186, and accounts redirect.rs:146-192. The accounts `parts`/`colon`/`with_port` re-parse what `parse_redirect` parsed, and `with_port` repeats `sign_in_values`' port rewrite (redirect.rs:363-390).
  - accounts redirect.rs:180 `varying` re-implements the private `varying_port` (redirect.rs:329).
- Fix: a lib or skein-http percent codec (measure, then write), and a parsed redirect-URI type exported by skein-oauth (about −200 lines). The issuer can still read its own forms.

**F12. skein-oauth's module names don't predict their contents.**
- Evidence:
  - redirect.rs (502 lines) holds the Terms/Parameter/Sent/OpenId types, registration validation, form encoding, the authorization URL and scopes.
  - machine/fetch.rs holds the whole `sign_in` start (:37-89) and the ID-token `identity` check (:180-207).
  - `refresh` (machine.rs:341-380) and `start_revocation` (revoke.rs:37-75) share about 25 lines, and the `Exchange` literal is repeated 3 times.
- Fix: split into terms.rs, registration.rs, form.rs and machine/sign_in.rs, with one refresh-token start.

**F13. Naming.**
- Evidence:
  - `skein_oauth::Limits` is the document/record limits, while `ClientLimits` is the client's (the fake imports it `as DocumentLimits`, lib.rs:10).
  - `client_valid` (machine.rs:45) breaks model 6.3's `valid(limits)`, which accounts, the issuer and the fake follow.
  - Accounts' `account` means the owner's number before `named()` (component/names.rs:19-62) and the slot index after it, translated back by `number(&self.names, …)`.
  - The aliases `Lower` and `LowerEvent` (component.rs:52-55) name skein_io's Request and Event.
  - There are two `Issuer`s, two `Terms` and two `OpenId`s across skein-oauth and the fake.
- Fix: `DocumentLimits`/`Limits` + `valid`, and a `Place` newtype for the internal index.

**F14. Redundant API shapes.**
- Evidence:
  - `down`, `up` and `filed` take `_files: &mut Queue<FileLower>` and ignore it (component.rs:210, 482; private.rs:147).
  - `keys_request` takes `_now` (fetch.rs:92).
~~  - `Event::Redirected` repeats `Redirect`'s five fields (machine.rs:153 vs 119), and `Event::SignIn` repeats `SignInValues`' ten (:140 vs :198). `step` destructures and rebuilds both, and tests/oauth/tests/qa_paste.rs:11-44 converts them back and forth.~~
- Fix: `Redirected { redirect, now }`, `SignIn { registration, values }`, and drop the unused parameters.

**F15. Subset rules held only by review are broken.**
- Evidence:
  - Tuple scrutinees: machine.rs:526, :690; fetch.rs:50; redirect.rs:357, :363.
  - Types with lifetime parameters: jwt.rs:79 `Expected<'a>` and fake openid.rs:208 `Part<'a>`.
  - `loop`/`while` in the fake's step code: lib.rs:631, 646, 672, 682.
  - Module-wide `#![expect(single_match, manual_let_else)]`: component.rs:6, exchange.rs:6, listener.rs:5. AGENTS.md asks for scoped expects.
- Fix: per site.

**F16. tests/oauth-issuer keeps its fixtures twice and its harness outside `src/`.**
- Evidence:
  - The `.der` keys and their README in crates/skein-oauth-issuer/src/tests/fixtures and tests/oauth-issuer/tests/fixtures are byte-identical (md5).
  - There are two `test_key` functions (issuer src/tests.rs:33 and support/mod.rs:50), each comment calling itself "this crate's only key seam", against oauth-issuer.md 9's "one test function".
  - The 664-line harness is `tests/support/mod.rs`, pulled in by `#[path]` (suite/main.rs:4). src/lib.rs is a single doc line (testing.md 6: "what its test binaries share is its library, in src/").
- Fix: move the harness into src/lib.rs and keep one fixture directory.

**F17. The accounts `Web` is the fourth hand-built HTTP-over-TLS socket stack.**
- Evidence: exchange.rs:174-569 (about 395 lines: socket `Option`, tls `Option`, four queues, `close_sent`, upload offset) sits beside skein-llm-connection/src/call.rs:50, skein-forge-connection/src/connection.rs:122 and skein-forge-setup/src/host/connection.rs:28.
- Fix: one shared "request over a socket" component (cross-area).

Hypotheses:
- **H10 confirmed, with refinements.**
  - The area holds 25 of the 39 missing-debug expects, including all 9 "credential values" reasons.
  - `skein_lib::Secret` (crates/skein-lib/src/secret.rs:5) already redacts its Debug, and the issuer proves the pattern.
  - What remains after converting: Component (it holds `CryptoRng`, which has no Debug) and FileLower. skein_io's `file::Request::Store { bytes }` already derives Debug over raw bytes, so FileLower's expect hides nothing.
  - Layout: 9 `*_tests.rs` files plus 6 under `machine_tests/` make 15, and 16 with exchange.rs's inline module. This contradicts testing.md 6 (F7).

Tree-wide findings in this area:
- **T-layout:**
~~  - 2 qa_* binaries (F8).~~
  - skein-oauth and skein-oauth-accounts each use all three step-test conventions (F7).
  - testing.md 6 does not list crates/skein-oauth-accounts, crates/skein-oauth-issuer, tests/oauth-accounts or tests/oauth-issuer.
- **T-worlds-as-kits:**
  - tests/oauth dev-depends on skein-llm-world (issuer_tokens.rs:3).
  - tests/oauth-accounts depends on skein-tls-world for its PKI (world.rs:424, memory.rs:51).
  - The area has 10 `#[path]`: generated/v3, 8 accounts test files, and oauth-issuer's support.
- **T-lints:**
  - 3 hand `[lints]` blocks (tests/oauth, tests/oauth-accounts, tests/oauth-issuer), byte-identical (one variant).
  - 45 `#[expect]` in the area.
  - The closure-subset reason has 4 wordings (fake ×7, issuer settings.rs:103, requests.rs:214, oauth json.rs:353).
- **T-limits-shape:**
  - `client_valid` vs `valid`.
  - Step shapes: Client and fake `step` without Env, the accounts component with Env, and the issuer stateless.
- **T-primitives:**
  - The json.rs:23 pump (F2).
  - UTF-8 is checked through a JSON writer measure (common.rs:143-149, 8 callers) and through `from_utf8` under expect (issuer documents.rs:177, settings.rs:187, oauth redirect.rs:88).
  - 101 `usize::try_from(..).expect` with 15 messages; 49 of them are `len() > usize::try_from(cap)` (skein-oauth 29, fake 16).
  - `len_u64` 13 vs `u64::try_from(len)` 1.
  - 40 Env rewraps.
  - A hand token namespace: route.rs:14 packs slot×3+tag.
  - 11 local limits fixtures.
- **T-naming:** 16 "actual": 14 filler uses in tests/oauth-accounts, plus an `actual` identifier in issuer src/tests/documents.rs:6.

Status-file cross-check:
- **oauth.status.md confirmed:** section 7 is not built. `localhost` binds 127.0.0.1 only (redirect.rs:177-183).
- **oauth.status.md misses:**
  - Section 3's `resume` and section 5's exported maximum output per step are not built: `step` asserts an empty queue instead (machine.rs:264).
  - The F3 (fake shares types), F5, F6 and F15 departures.
  - 6.3 lists `add` as a request, but the code has a `Component::add` method (component/accounts.rs:141).
  - Stale docs: the crate README.md:14-15 ("SKOT magic and version 1"), and docs/design/README.md:86-88's graph, which omits skein-oauth's codec and tls dependencies and the fake's tls.
- **oauth.status.md claims the code contradicts:** "Built: 5" includes "a version-2 record read, refreshed and kept as version 3" (oauth.md 5:426-428; section 1:113-121 says it reads v2). 6.4 and the code make v2 unreadable (src/tests.rs:428). The design contradicts itself here and the status file does not say so.
- **oauth-issuer.status.md:** "Built 2 to 9" holds, except section 9's single key function and fixture set (F16).

Incidental:
1. **`Client::step` silently drops** `SignIn`, `Refresh` and `Revoke` outside Idle (machine.rs:279, 300, 307). There is no terminal and no assertion, against "every request gets exactly one terminal". The accounts component guards against it; a direct caller would hang.
2. **Token records print in full through skein_io's Debug**: `file::Request::Store { bytes }` derives Debug (skein-io/src/file.rs:30,112-116), so any trace of io requests prints the stored record.
3. **`issuer_endpoint` admits an empty host or userinfo**: it accepts any `https://` prefix (redirect.rs:89-93), including `https://` alone and `https://a@b`. The issuer's `address` refuses both (settings.rs:143-146). `jwks_uri` from metadata goes through the same check (keys.rs:21-27).
4. **The fake's key can drift**: key.rs:31 hard-codes `MODULUS` beside `PKCS8`, instead of deriving it with `skein_tls::rs256_public` as the issuer does (settings.rs:93-97).
5. **A test rewrites another crate's source**: under `SKEIN_CODEGEN_WRITE`, tests/oauth/tests/suite/drift.rs:36-41 rewrites crates/skein-oauth/src/lib.rs by text splice (`split_once`).

---

# A11-ui-browser

Scores: Cleanliness 3, Organisation 3, Taste 3, Naming 4, Primitives 2, Elegance 3, Economy 3, Complexity 3

~~Verdict: The UI kit itself is well made. `skein-ui` is a model step crate: no `Vec`, lib containers throughout, exhaustive role matches, a state table on the view, budgets checked before each allocation. `skein-ui-text`'s renderer and `skein-ui-web`'s pure/platform split, with a host-side DOM model that golden HTML can check, are in the same class. Quality falls off toward the browser end of the area. `skein-browser` is held to the step lints but works around them: a second JSON parser of its own (recursive, `loop`, a hand-written `Iterator`), entity tables built on `Vec`, closures, and `disallowed_types` silenced for the whole module in 6 of its 9 source modules. The test harnesses around it repeat helpers across crates (owner routing, the Chromium locator, the loopback HTTP loop), test layout follows four conventions, and the `qa_*` binaries largely repeat tests that already exist. Role classifications are written out several times, so the compiler cannot point to every place a new role must reach, which ui.md 3 promises. About 1.5-1.8k of the area's ~45k lines could go with no loss.~~

Good:
- `crates/skein-ui/src/view.rs:1-9`: the module doc carries the state table (programming-model 5.4), and the code follows it.
- `crates/skein-ui/src/builder.rs`: every capacity is asserted before it allocates, and `splice` re-checks nodes, slots, bindings and bytes as a whole. `binding.rs:38` `takes` is an exhaustive table.
- `crates/skein-ui-web/src/page.rs` + `dom.rs`: the pure half emits `Op`s over a DOM model, so the mapping is tested on the host against `src/tests/golden/*.html` without a browser.
- `crates/skein-ui-text/src/render.rs:26` `render_bound`, and `Terminal::operations`: room comes from a formula, not a cap.
- `tests/ui/tests/suite/limits.rs:32-58`: named rows that test each bound from both sides ("last scene styles" / "first scene style overflow").
- `tests/ui-web/tests/suite/serve.rs`: "the server consumes a bundle without linking its producer", with a test that the server's types match the bundler's.

Top recommendations:
1. Move `skein-browser` onto skein-json and lib containers (F1, F2). Win: about 450 lines, 6 module-wide expects gone, one JSON reader in the tree. Effort: M-L.
2. One ordinary-Rust lint profile and one `clippy.toml` per kind for the area's ordinary crates (F3, F6). Win: 10 hand-written `[lints]` blocks in 7 variants become one pattern, 4 mislabelled `clippy.toml` copies are fixed, and 9 own-enum `_` arms come to light. Effort: S.
~~3. Fold `qa_look` into `tests/suite`; fold or design `qa_program` and `qa_hand` (F7). Win: about 300 lines, plus helper duplication gone. Effort: S-M.~~
4. Role primitives in `skein-ui`: `Role::ALL`, one `acts(role)`, named exhaustive role sets (F4, F5). Win: about 150 lines, and the compiler reaches every role set. Effort: S-M.
5. Shared browser-tier helpers (F12): `io::Event::owner`, one Chromium locator, one loopback HTTP host. Win: about 200 lines, 3 copies become 1. Effort: M.
6. Small deletions: the reference shell's `/test/tree/` fixture moved out, the dead pure `Timer` removed, the boilerplate docs removed (F10, F11, F13). Win: about 200 lines out of the reference shell and the core. Effort: S.

Findings:

F1. `skein-browser` has a second JSON parser that breaks the subset. `testing/skein-browser/src/wire/decode.rs` (527 lines) is recursive (`value` → `object`/`array` → `value`, :267-320), has 3 `loop` and 7 `while` (rg `^\s*(loop \{|while )`), and a hand-written `impl Iterator for Array` (:216). It holds a module-wide `#![expect(clippy::disallowed_types)]` (:6). Each `Value::get` (:70-95) re-parses the object and boxes every key it compares. The crate already depends on skein-json (`params.rs:7` uses its writer), whose `parse.rs:26` returns a bounded `Document`. browser.md 4 asks only for "a bounded JSON document view", and browser.status.md records none of this. Fix: decode replies through `skein_json::parse` under the kit's limits, or add a borrowed view to skein-json. Then delete decode.rs. About -350 lines.

F2. `skein-browser` hand-rolls containers and contradicts its own stated reasons. `Vec<Option<Person|Page|Operation>>`, three `insert_*` functions and six lookups do linear scans (browser.rs:293-395, about 100 lines). Queues are `VecDeque`. `disallowed_types` is expected module-wide in browser.rs:3, command.rs:3, browser/look.rs:2, wire/frame.rs:3, encode.rs:3 and decode.rs:6. browser.rs:7 expects `manual_find, manual_map` because "the step subset excludes closure-based combinators", yet the same file has `retain(|…|)` closures at :911, :1228, :1756 and :1760. `contains` (:1811) re-implements `skein_lib::bytes::find`. `Browser::new` repeats `usize::try_from(limits.x).expect("u32 fits usize")` 7 times (:200-221). Fix: use `skein_lib::Map<Token, T>`, `Queue` and `List`, as `skein-ui` does with zero `Vec`. Drop the module-wide expects. About -100 lines.

F3. The lint gaps (hypothesis 11). `skein-ui-web` and `skein-ui-bundle` declare `[lints.clippy]` with pedantic plus 3 allows. `shell-web` has the same with 2 allows. None has `wildcard_enum_match_arm`, `match_wildcard_for_single_variants`, `unwrap_used`, `allow_attributes`, `allow_attributes_without_reason` or `missing_debug_implementations`. `tests/browser/Cargo.toml` has only `unsafe_code = "deny"` (every other crate here forbids), and `tests/ui-web` has only `unsafe_code = "forbid"`. Both lack the `tests/` profile that `tests/ui` and `tests/ui-text` carry. ui-web also avoids `as` in two different ways: `pixels`/`coordinate` round-trip through `to_string().parse()` (page/web.rs:771, :904), while platform/web.rs:53-59 uses `as` under `#[expect]`. Fix: give the ordinary crates the profile that `testing/skein-fake-person-browser/Cargo.toml` uses. Pick one cast idiom.

F4. Role sets written as slices escape exhaustiveness. There are 23 `[Role::…].contains(&role)` checks (rg -U, non-test): builder.rs 6, ui-text render.rs 3, ui-web mapping.rs 5, page.rs 9. They are `matches!` under another name: adding a role breaks none of them. Two near-copies have already drifted. mapping.rs:227-238 (no tabindex needed) and page.rs:526-539 (`focusable`, the fallback after a removal) share 9 roles but differ in Main and Disclosure, and neither name says which question it answers. Fix: exhaustive `fn`s with names that say which question they answer, for shared sets in `skein-ui`. builder.rs:569 `acted_on` already shows the shape.

F5. The same role tables appear more than once. `next_role` (51 lines) exists twice: ui-web mapping.rs:449 and fake-person tree.rs:233 (test-only). Which act a role takes is answered three times, each as a 50-line exhaustive match: binding.rs:38 `takes`, builder.rs:569 `acted_on` (exactly the roles `takes` admits), and ui-text command.rs:74 `first`. Fix: a `Role::ALL` const in skein-ui, and one `acts(role)` that all three read. About -130 lines.

F6. `clippy.toml` copies carry other crates' headers. `crates/skein-ui-web/clippy.toml` and `crates/skein-ui-bundle/clippy.toml` are byte-identical to `testing/skein-world/clippy.toml` ("The world harness is ordinary Rust…"). `examples/ui/server/clippy.toml` is identical to `examples/echo/shell/clippy.toml` ("What the echo's main may not use"). `testing/skein-lab-browser` and `skein-fake-person-browser` copy `tests/clippy.toml` ("the test crates under tests/"). The lab's copy rightly drops the `Look` ban. `examples/ui/server/Cargo.toml` restates 24 workspace lints by hand. The area has 10 hand-written `[lints]` blocks in 7 variants. Fix: per-kind shared files with headers that are true.

~~F7. The `qa_*` binaries largely repeat the suites. `tests/browser/tests/qa_look.rs` (493 lines) drives the same in-memory `World` as `src/tests/look.rs` (616), with its own `opened`/`answer`/`properties` helpers. At least 10 tests are pairs, for example `tree_entry_bound_counts_visible_entries_without_cutting` / `tree_past_entries_is_refused_whole`, `screenshot_bound_applies_to_decoded_png_bytes` / `shot_at_screenshot_is_kept_and_past_screenshot_refused`, and `every_look_command_times_out…` / `looks_have_command_deadlines…`. It needs no Chromium, yet it runs only under `qa-skein`, outside the gate. `testing/skein-lab-browser/tests/qa_program.rs` (452) and `qa_hand.rs` (510) go beyond lab-browser.md 6 and 8.9, which ask for "one story" per form, and they rebuild the process helpers. All four lab binaries share code through `#[path = "../src/json.rs"]` and need 6 `#[expect(dead_code)]` to do it. One reason reads "the existing Reader…". Fix: merge `qa_look` into `tests/suite/look_world`. Fold the lab `qa_*` tests into the stories, or design them. Expose `json` through a lib target.~~

F8. Test layout uses four conventions and 7 out-of-tree `#[path]`s. `skein-fake-person-browser` alone uses `src/tests.rs`+`src/tests/`, `harness/X/tests.rs` (×4), sibling `harness/X_tests.rs` (×2) and inline `mod tests {` (×3). `skein-browser` mixes sibling `*_tests.rs` through `#[path]` (browser.rs:2280, attribute.rs:130, look.rs:680) with 5 inline modules. These `#[path]`s reach outside their module tree: tests/browser support/real.rs:16 and suite/page_server.rs:4 include `src/pages.rs`, which the lib does not compile; 4 lab test binaries include `../src/json.rs`; ui-web page/fold.rs:20 includes `../tests/fold.rs`. The lab's runtime.rs:2 and hand.rs:2 build a fake module tree with `#[path]`. `tests/browser/src/tests*` holds 1.7k lines of `#[cfg(test)]` unit tests inside a test crate's library, beside its `tests/suite`. `skein-lab-browser` is missing from testing.md 6's layout.

F9. The fake CDP peer decodes with the kit's own decoder. `tests/browser/src/world.rs:4,164,199`, `src/tests.rs:2` and `src/tests/{attribute,navigation,look}.rs` parse the kit's commands with `skein_browser::wire::decode::Document`. testing-strategy 4 says there are no shared types, and 4.1 warns that "two sides written together can share one misreading". Fix: the fake parses with skein-json. With F1 done, this happens anyway.

F10. The reference shell carries an undesigned test fixture. `examples/ui/shell-web/src/tree.rs` (129 lines, `#[cfg(debug_assertions)]`) is served at `/test/tree/*` (lib.rs:70-74) and driven by `tests/ui-web/tests/support/live_tree.rs` (243). ui-web.md 14.3 and examples.md 9.4 list only the panic address and the small-store address. The shell is described as "about 200 lines… to be copied", but comes to 424. jig already keeps its equivalent in `jig/testing/jig-client-test-shell-web`. Fix: move it to a test shell crate, or record it.

F11. The pure `Timer` half is dead. `crates/skein-ui-web/src/timer.rs` (22 lines) is used only by `tests/history.rs:21-26`. `timer/web.rs` keeps its own deadline, and `hidden/web.rs:74-98` re-implements setTimeout handling. Fix: delete the pure half, or have both web timers wrap it.

F12. The browser-tier harness helpers are duplicated.
- The io event owner match is byte-identical in `skein-fake-person-browser/src/harness.rs:1274` and `skein-lab-browser/src/runtime.rs:887`.
- `chromium()` exists 3 times: `tests/browser/tests/support/real.rs:63`, `tests/ui-web/src/lib.rs:28` and `skein-lab-browser/tests/support/mod.rs:35`.
- The loopback HTTP connection loop exists twice: `examples/ui/server/src/main.rs` (about 157-283) and `skein-fake-person-browser/src/origins.rs:136-283`. Both do accept, `Bind`, `Next`, drain `below` into `Stream`, and chunk the reply on `Up::Room` by `limits.send`.
- Hand-made token namespaces in the lab's single io: hand 1<<38 (hand.rs:25), proxy 1<<39 (proxy.rs:8), origins 1<<40+n·1_000_000 (origins.rs:13,68), `u64::MAX-k` (runtime.rs:28-30; harness.rs:21-24), each advanced by an unchecked `next += 1`.

Fix: `skein_io::Event::owner()`, the locator in skein-fake-person-browser, and a small io host for skein-http's server.

F13. Leftovers in the core.
- tree.rs:23-147 has 42 boilerplate doc lines ("A item node.", "A alert node.", "The node is submitsonenter.").
- `Node::vacant` and `Node::new` repeat a 25-field literal (tree.rs:309, :339). `Attribute::of` copies 18 of those fields (diff.rs:43). A `Node` that held an `Attribute` would drop both.
- builder.rs repeats `match &self.node().X { Some(b) => length(b), None => 0 }` 6 times.
- ui-text render.rs:23-24 has two doc lines for one function.
- render.rs:7 gives the reason "ruling 106", which nothing in skein cites.
- Three wordings of one reason: tree.rs:427, phrases.rs:427, browser.rs:7.

F14. Subset departures not recorded in any status file:
- `impl core::fmt::Display for Scene` (scene.rs:370, about 95 lines of `format_args!`). The golden scenes could use `pub fn print(&self, out: &mut List<u8>)` instead.
- `look::Names<'a>` (look.rs:7) holds references in fields.
- `skein-browser` `impl Default for Limits` (limits.rs:29).
- The example view's SSE pump uses `loop`/`while let` (examples/ui/view/src/state/events.rs:44-87), in step code that the reference teaches applications to copy.

F15. Calendar code in this area: `skein-ui/src/phrases.rs:594` repeats Hinnant's civil-from-days (as does skein-http response.rs:253), and `examples/ui/view/src/state.rs:313-339` writes its own days-in-month and leap-year rule. Fix: a `Wall` civil date in skein-lib.

Hypotheses:
- 11, refined. The pure halves are ordinary Rust by design, not step code: ui.md 2 says so in its table, and ui-web.md 1 and 5.1 say "each is ordinary Rust" with `Inbox<R>` generic and closures. So workspace lints would be a redesign, not a fix. ui.md 3 and 4.3 still list "each shell's mapping" as a place an addition must break, so they need the ordinary profile (F3). The 16 `_ =>` arms in skein-ui-web split 9 / 7. Nine are on own enums: page/web.rs:301 (`Op`), tests/page/events.rs:343 and :391, tests/mapping.rs:342 (`Property`), and tests/page.rs:225, :254, :588, :796 and :828. Seven are on `u8`, `&[u8]`, `&str`, `char` or `(Option, Option)` and are fine: mapping.rs:666, svg.rs:139, page.rs:801, dom.rs:65, page/web.rs:767 and :873, tests/page.rs:660. The larger hole is the 14 role-slice checks in ui-web (F4). shell-web's 2 arms are on `&str`/`u64` and are fine. tests/browser's own-enum wildcards include src/tests/look.rs:376 and :416 and src/tests.rs:299.

Tree-wide findings in this area:
~~- T-layout: see F8. There are 10 binaries outside `tests/suite` (lab 4, tests/browser 3, tests/ui-web 3), 3 of them `qa_*`. Name-based nextest profiles select them.~~
- T-worlds-as-kits: no area world crate is anyone's dependency.
- T-lints: see F3 and F6. There are 115 `reason =` strings with 68 distinct.
- T-limits-shape: consistent with the model here. `skein_ui::valid`, `phrases::valid` and `skein_browser::valid` all return bool. ui-text returns `Option`.
- T-primitives: hand JSON pumps at lab json.rs:60 and the whole of browser decode.rs. Calendar in F15. 28 `Env {` rewraps (ui-text terminal 4, view 6, browser 5). 117 `usize::try_from(..).expect`. `u64::try_from(len)` 5 times against `len_u64` 11. Token namespaces: ui-text terminal.rs:12 (1<<62, kernel ops, the same space as skein-world files.rs:14's 1<<63), plus the lab's in F12.
~~- T-naming: `qa_*` and "ruling 106".~~
- T-temper-draft: lab-browser.md cites temper's `qa-web.md` 5 times (:175, :288, :336, :505, :553).

Status-file cross-check:
- Confirmed lags: ui "builder's named-control assertion" (`close`, builder.rs:111, asserts nothing). ui-text's open items. lab-browser "dark scheme".
- Contradicted by code:
  - ui-web lists "typing into populated fields" as Not yet, but face.rs:483 and :556-559 select all and replace (ui-web.md 949-950).
  - browser lists "native-dialog test" as Not yet, but `tests/browser/tests/browser_real.rs:97` (`chromium_dismisses_a_confirm_as_trouble`) and `support/real/dialogs.rs` exist. Its "kit split" names nothing in browser.md.
- Missed:
  - F1, F2 and F14 (departures).
  - F10 (`/test/tree/`).
~~  - `qa_look` and the lab's `qa_*` tests (F7).~~
  - lab-browser's absence from testing.md 6.
  - examples.status says "Built: unknown" and says nothing about section 9.
  - browser.md:579 and ui-web.md:894 name `--profile browser`. The profile is `browser-skein` (testing.md:439).

Incidental:
- `scripts/browser.sh:22` runs `cargo nextest run --profile browser`, but `.config/nextest.toml` has only `browser-skein`, `-jig` and `-temper`, so the script fails at its start.
- `skein-browser` browser.rs:1811 `contains` calls `windows(needle.len())`, which panics on an empty needle. Every current caller passes a literal.
- origins.rs:68 gives each origin `number * 1_000_000` tokens, unchecked. Past a million connections an origin's tokens run into the next origin's range. `next += 1` is unchecked in origins.rs:154, hand.rs:77 and proxy.rs:82.
- ui-web page/web.rs:771 `pixels`: a NaN size becomes the string "NaN", whose parse fails, so it panics with "whole CSS pixels fit u16".

---

# A12-sim-world

# A12: simulator, world and conformance harness, kv

Scope: crates/skein-sim (5,119 lines), crates/skein-sim-world (1,498), testing/skein-world (7,084), testing/skein-conformance (4,498), testing/skein-fake-person (1,546), testing/skein-scratch (211), crates/skein-fail-stop (145), crates/skein-fake-machine (2,027), crates/skein-kv (2,053), tests/sim (3,216), tests/world (5,944), tests/conformance (3,324), tests/kv (1,003). About 37.7k lines in all.

## Scores

Cleanliness 3, Organisation 3, Taste 3, Naming 2, Primitives 3, Elegance 3, Economy 3, Complexity 3.

## Verdict

The simulator and its tests are the best code in this area. skein-sim documents every choice the contract leaves open, checks its client, and its fuzzy sweeps assert that each fault actually fell. The conformance suite and skein-scratch are equally well made. Three things pull the area down. First, skein-kv is a step crate that departs from programming-model 5.4 and 10.3 more than any sibling, and neither kv.md nor its status file records the departures. Second, the harness's public names collide: there are two referee frameworks whose struct and trait names are swapped, four "Expectation(s)" concepts, two `Script`/`Act` pairs and two `Killed` types. Third, the hosting registry, the memory-mode choice and the release-on-drop check are each written twice, once for the simulated world and once for the real one. A layer of stale prose also survives the move of the fake machine and the simulated world to `crates/`. About 1.1k to 1.4k lines could go, and the bigger gain is concepts.

## Good

- `crates/skein-sim/src/lib.rs:59-121` ("Its choices, where the contract leaves one"). Each modelled choice is stated once, next to the code that makes it. Copy this shape.
- `tests/sim/tests/suite/fuzzy/exchange.rs:59-74` and `fuzzy/files.rs:176-186`. Every configured fault is asserted to have fallen (testing-strategy 3).
- `tests/world/tests/suite/steps.rs:11-41`. The test names quote world.md 8 word for word.
- `testing/skein-scratch/src/paths.rs`. Small, explains why it exists, one failure helper.
- `testing/skein-conformance/src/lib.rs:62-140`. The `Backend` docs, and `Check` naming the rule behind each assertion.
- `testing/skein-world/src/domain/{schedule,ledger}.rs`. Small single-purpose primitives, used in 136 downstream files.
- `tests/kv/tests/suite/golden.rs`. Golden bytes that are never regenerated, as kv.md 9 asks.

## Top recommendations

1. **Bring skein-kv's `store.rs` into the 5.4 idiom (F1, F2).** Split `up` into one handler per cell: match on the phase first, then on the event. Fold the six bools and the `Option`s into phase or lifecycle enums. Share one commit-admission check. Expected win: −150 to −250 lines, five `#![expect]`s gone, and the O(queue²) clone removed. Effort: 2 to 3 days.
2. **Publish kv's edge formulas (F3).** Add `skein_kv::frame_bound(ops)` (or `commit_price`) and `row_price(key, value)`, and use them in `Limits::is_usable`. jig then stops hard-coding 26 and 9 in production code. Expected win: 3 magic numbers in kv, about 5 in jig. Effort: half a day.
3. **Un-swap the referee names (F4).** Rename the root polling helpers `Expectations<P,X>` and `Expectation<P>` (6 files in all) and the `end_to_end::Expectation` enum. Make `assert_replays` generic over the trace type and re-export it at the crate root (F5). Expected win: one meaning per name. Effort: half a day.
4. **One hosting registry and one memory choice for both worlds (F6).** Move `host`/`host_roots`/`host_startup` and the release-on-drop check into skein-sim-world. Have `real::World` take `Memory` (9 downstream sites). Expected win: −120 lines and 4 constructors. Effort: 1 day.
5. **Remove the `#[path]` web in tests/conformance/real and tests/world (F8).** Move `tests/support/*` into `tests/conformance/real/src/`. Give the shared process fixture one home. Expected win: 9 `#[path]`s and 1 cross-package `[[bin]]` path gone. Effort: half a day.
6. **Sweep the stale prose (F10)** and fix the status files. Effort: 1 hour.

## Findings

**F1. skein-kv breaks the step-machine idiom, unrecorded.**
- `store.rs:637` matches the tuple `(phase, event)` with a catch-all binding arm at `:842`.
- `store.rs:1040` matches `(action, event)` with `(_, _) =>` at `:1084`.
- Both break 5.4's rule "never on a tuple of both: a `_` inside a tuple pattern escapes clippy's wildcard lint". The `too_many_lines` reason at `:9` says "one exhaustive state match", which the catch-all contradicts.
- `Store` keeps a lifecycle in six bools (`recovered`, `has_snapshot`, `has_temp`, `repairing`, `repair_required`, `recovering_after_failure`) plus `snapshot`, `snapshot_start`, `open_owner`, `close_owner` and `tail_name` beside `phase` (`:111-133`, with `struct_excessive_bools` expected). That breaks 5.4's "never a lifecycle spread over flags and Option fields beside the state".
- `if let` on its own enums appears 8 times (`store.rs:474`, which even has an empty then-branch, `:584`, `:873`, `:1018`, `frame.rs:24`, `map.rs:70,87,115`).
- Measured with `rg '\|[a-z_&, ]*\| '` and `rg '^\s*(loop|while)'`: 47 closures (25 per kloc, against io 8, http 3, json 2) and 10 `loop`/`while`.
- It has 19 module-wide `#![expect]` and no item-scoped one (io has 6 module-wide and 10 item-scoped). `map.rs:5` lifts `arithmetic_side_effects` for the whole module.
- Fix: per-cell handlers as in 5.4, a `Recovery`/`Snapshotting` enum per lifecycle, and item-scoped expects. Alternatively, record kv as ordinary Rust in kv.md 1 and kv.status.

**F2. Commit admission is written three times, and one copy clones the whole queue.**
- Key and value limits are checked in `down` (`store.rs:507-519`), again in `commit_memory` (`:601-609`, in a different order from the ops-count check), and a third time in `KvMap::check` (`map.rs:66-73`).
- `all_ops` (`store.rs:235-245`) deep-clones every op of every active and queued commit on each admission (`:526`).
- `KvMap::check` and `apply` then scan "is there a later op on this key" in O(n²), three times (`map.rs:76,106,112`).
- Fix: one `admit(ops, limits) -> Result<u64, Refusal>`. Keep a running `used` with a per-key pending charge (a `Map` of key to last charge) instead of re-scanning the copies.

**F3. kv's framing and prices are private literals that downstream copies.**
- `frame.rs:12` `HEADER = 22`, `map.rs:15` `ENTRY = 128`, `snapshot.rs:16-17` `HEADER = 18`, `TRAILER_LEN = 28`.
- `limits.rs:29` says `commit >= 26` (that is `HEADER + 4`) and `:39` says `+ 20` (chunk overhead plus row lengths). Both are bare numbers.
~~- jig production code repeats them: `jig/crates/jig-protocol-store/src/sizes.rs:166` (`key + 9`, `+ 26`), and so do jig's tests (`qa_check.rs:273,287,395`, `patching.rs:121`).~~
- Programming-model 6.3 names exactly these: a frame's formula `_bound`, and a stored row's `_price`.
- `queued <= 32` (`limits.rs:30`) has no reason anywhere.
- Fix: public `frame_bound(&[Op]) -> Option<usize>` (today's `encoded_len`), `row_price(key, value)`, and named `PUT_OVERHEAD`/`ERASE_OVERHEAD`. Build `is_usable` from them.

**F4. The two referee frameworks swap their names, and "Expectation" means four things.**
- Root: `trait Referee<P>` (`referee.rs:45`), `trait Expectation<P>` (`:85`), `struct Expectations<P,X>` (`:95`).
- Domain: `trait Expectations` (`domain/referee.rs:22`), `struct Referee<X>` (`:55`), `struct Judge` (`:174`).
- Separately, `end_to_end::Expectation` is an enum of what happens to a binary's descendants (`tree.rs:90`).
- Downstream, 85 impls of the root trait are mostly named `Judge`, which collides with the domain's `Judge`.
- Load measured: the root trait has 85 downstream impls and 36 in skein. The domain framework has 21 `impl Expectations`, 67 `Referee::new` and 136 files. The root `Expectations`/`Expectation` pair is used by only 3 skein and 3 smith files.
- Fix: keep both frameworks, since they serve different tiers (a pull over `&[P]` against a pushed `Seen`). Rename the low-use root pair (for example `Pending<P,X>`/`Condition<P>`) and the tree enum (`Leftovers`).

**F5. Replay has one helper, and it is typed for domain worlds only.**
- `domain::assert_replays` takes `Vec<String>` (`domain/trace.rs:36`).
- Simulated worlds therefore Debug-format `skein_sim::Entry`s to call it (`tests/world/tests/suite/digests.rs:28`, `tests/echo/tests/suite/scenarios.rs:216`).
- "Trace" means four things: `skein_sim::Trace` (a config enum, Whole or Off), `Outcome.trace` (entries), `domain::Trace` (lines) and `stream::Wire.trace`. In addition, `Digests::{Traced, Untraced}` sits next to `Trace::{Whole, Off}`.
- Fix: `assert_replays<R: PartialEq + Debug, T>` at the root. It stays source-compatible for the 25 downstream callers. Rename the config enum `Tracing`.

**F6. Hosting and memory are duplicated between the simulated and the real world.**
- `host`/`host_roots`/`host_startup`, with the same two asserts, appear in `crates/skein-sim-world/src/world.rs:203-219` and again in `testing/skein-world/src/real.rs:261-277`. Their doc comments have drifted apart.
- skein-world's `World` forwards 11 methods to its inner world (`world.rs:62-131`).
- Memory is chosen by `Memory` for the simulated world, but by five constructors and `CheckedWorld`/`CheckedOutcome` Deref wrappers for the real one (`real.rs:77-131,184-240`).
- The exact-release check on drop is written twice: `outcome.rs:50-75` (`crate::fail`) and `real.rs:87-104` (`assert_eq!`).
- There are two `Killed` types, keyed by `pid` (`skein-sim-world/src/world.rs:94`) and by `host` (`real.rs:147`).
- The teardown check is also written twice: `world.rs:239` and `real.rs:779,1189`.
- Fix: a `Registry<P>` in skein-sim-world, `real::World::new(referee, Memory)`, one `release_check(procs, held)`, and one `policy_after_last_word`.

**F7. The world's spawn API is combinatorial.**
- The simulated world has 6 spawn variants: `spawn`, `spawn_root`, `spawn_append`, `spawn_signals`, `spawn_restartable`, `spawn_restartable_startup`. The real world has 4.
- Downstream use: `spawn_append` 0, `spawn_restartable_startup` 0, `spawn_signalfd` 0. Skein uses them 2, 4 and 1 times.
- Fix: one `spawn(Startup { roots, appends, signals }, make)`, with `spawn` kept as the zero-startup shortcut.

**F8. Tests reach across files and packages with `#[path]`.**
- `tests/conformance/real/src/bin/process_fixture.rs:3` and `signal_sender_fixture.rs:4` compile `tests/support/*`.
- `tests/suite/given_listener.rs:3,6` and `conformance.rs:32,43` do the same within the test tree.
- `tests/world/Cargo.toml` builds `../conformance/real/src/bin/process_fixture.rs` as its own binary, which `#[path]`s into conformance's `tests/support/`.
- `tests/world/tests/steps/support.rs` is a lone directory pulled in by `suite/main.rs:4`.
- `crates/skein-sim/src/sim/processes.rs:1087,1091` `#[path]` into `src/tests/`. The sim keeps about 690 lines of contract tests in `src/`, although simulator.md 2 places "its own tests" in `tests/sim`. One of them, `src/tests/given_listener.rs`, drives io over the simulator.
- Fix: move the support files into `src/` (testing.md 6 says "what its test binaries share is its library"), give the process fixture one package, and move the sim's io-level tests to `tests/sim`.

**F9. tests/world has two `Script`/`Act` pairs.**
- `src/lib.rs:26,47` (sockets) and `src/hosted.rs:11,54` (pipes, files, signals).
- Each has its own `hog`/`held`/`leak`/`pause`/`acts`/`fds` fields, its own `Host` impl and its own `Judge` (`lib.rs:364`, `hosted.rs:337`).
- Separately, 8 trivial "observe nothing, pass" referees exist in skein (for example `tests/world/tests/suite/real_conformance.rs:20` and `testing/skein-world/src/real.rs:1302`) and 3 downstream.
- Fix: one `Script` with one `Act` enum (about −150 lines) and a `skein_world::Quiet` referee.

**F10. Stale prose after the moves to `crates/`.**
- skein-world: `Cargo.toml:3` ("heap against the worst cases") and `lib.rs:16` ("against its own worst case") contradict programming-model 6.3. `domain.rs:2,7-8` has history ("Extracted from temper's…", "whose API is unchanged"). `domain/stage.rs:31` cites "draft dynamic-memory 4" although testing.md 2 is the design.
- Fake machine: `lib.rs:21` cites a `README.md` that does not exist, and `lib.rs:23` says "it runs in tests only", which testing.md 4 and world.md 1 contradict ("a playground ships it").
- `crates/skein-sim-world/clippy.toml:1-6` is a byte-identical copy of skein-world's ("The world harness… none of it runs in a service"), although world.md 1 calls the crate "a product, not a test kit".
- `testing/skein-fake-person/clippy.toml:1` is headed "What the test crates under tests/ may not use".
- `tests/conformance/sim/src/lib.rs:10-12` names `tests/conformance.rs` and `tests/fuzzy_conformance.rs`, which do not exist.
- `tree.rs:11` cites "benchmarks.md, section 10", which is smith's document, not skein's.
- `real.rs:110` says "the existing unchecked world".

**F11. Unused or decorative surface.**
- `skein_kv::Limits::deadline` is read only by `is_usable` (`limits.rs:38`); the store never uses it. kv.md 12 says the limits supply "a timeout for each file request".
- `fire`'s `_above` parameter is unused (`store.rs:881`).
- skein-world fences 6 modules behind `#[cfg(unix)]` (`lib.rs`), but `referee.rs:24,182` names `crate::Starved` and `crate::real::Sched` unconditionally, so the crate cannot build off unix and the fences do nothing.
- Fix: drop the field (the owner configures its `FileIo` stall), drop the parameter, and drop the fences or gate `Controls::sched_start`.

**F12. kv's world is a hand loop, with no referee and no crash sweep over a model.**
- kv.md 9 asks for "commits, loads and snapshots against a model map… crashing at every operation and every sync boundary… Its referee: …".
- `tests/kv/src/world.rs:70-115` is a bespoke drive loop (`for _ in 0..10_000 … panic!`).
- The crash tests cut one fixed two-commit scenario (`store.rs:87-117`, cuts `0..8`) and one snapshot (`:120`, `0..25`).
- The only fuzzy test runs `Store::memory`, never the disk path, and silently accepts `Refused` (`fuzzy/map.rs:37,58`).
- tests/kv also hand-rolls an xorshift (`fuzzy/map.rs:25`) although `skein_lib::Rng` exists, re-implements CRC-32C and frame encoding (`store.rs:31-57`), and keeps 6 local `Limits` literals and 106 bare `Token::new(n)` (`store.rs`).
- Fix: a fuzzy crash sweep over the sim world with a referee holding the model, `Rng`, and one `limits()` fixture.

**F13. fake-person escapes the subset that testing.md 6 sets for it.**
- testing.md 6 says crates under `testing/` are "held to the step crates' lints", and that a step-machine fake "has no clippy.toml of its own".
- skein-fake-person is `no_std`, shaped as a step machine (ui.md 63), yet it has a hand `[lints]` block without `indexing_slicing`, `arithmetic_side_effects`, `as_conversions` or `panic`, plus its own clippy.toml.
- Its `src/tree.rs:29,50,58,62,73,101` index slices.
- Fix: `[lints] workspace = true` and delete its clippy.toml, or record the exception in testing.md 6.

**F14. Real-clock waits outside the one mechanism.**
- `tests/conformance/real/tests/support/given_listener.rs:26,44` arms `clock.now() + 2 s` and asserts against it, which testing-strategy 2.10 forbids ("no harness arms a deadline on the real clock beside it").
- `support/deadline.rs:12-47` (`Patience`) is a pure forwarder over `Deadline`, except for `watch_spawn`.
- Fix: use `Deadline` directly, and add `watch_spawn` to `Deadline`.

**F15. The sim's own fault sweep misses one fault, and the list is hand-kept.**
- `tests/sim/tests/suite/fuzzy/exchange.rs:59-69` omits `Fault::LateReset`, which `Config::CHAOS` turns on (`late_reset: 300`). Only tests/io and tests/echo check that it fell.
- Fix: derive the expected set from an exhaustive `match` over `Fault`, so a new variant must be placed.

**F16. Small duplicate primitives.**
- `usize_of` is defined three times (`skein-sim/src/sim.rs:1699`, `sim/files.rs:456`, `skein-conformance/src/run.rs:1003`).
- There are 75 `usize::try_from(..).expect` in the area; skein-kv alone has 16, mostly `usize::try_from(env.limits.key).expect("key limit fits usize")`.
- `skein_world::files::Driver` (`files.rs:14`, token tag bit 63) has one user, tests/oauth-accounts. testing-strategy 7 keeps such a harness with its first user until a second needs it.

## Hypotheses

**12. Confirmed and refined.**
- Both frameworks are load-bearing, so merging them would cost about 230 downstream sites for no gain: the root is a pull over `&[P]` with `act`/`starved`/`last_word`, the domain a push of `Seen` with `Judge`/`Verdict`. Fix the names instead (F4).
- The traces should not merge, but `assert_replays` should go generic (F5).
- On referees per tier, measured by `impl … Referee<` or a domain `Expectations` impl per tests crate: 10 of 36 crates use skein_world's referee. Six have a `referee.rs`, and the one in `tests/forge/protocol` is a hand-rolled `Accounting`, a third shape.
- Machine worlds (http, json, channel, llm) and tests/kv have none. testing-strategy 7 ("at every tier") and kv.md 9 both say they should, and testing.md does not record the exception.

**13. Confirmed.**
- Constants and jig's copies: see F3. `ENTRY = 128` is not copied downstream, but programming-model 6.3 wants it to be `row_price`.
- On `is_usable` against the model's `valid(limits) -> bool`: kv.md 7, io.md 53 and kv.status all name `is_usable`, so it is recorded in the designs. The divergence from programming-model 6.3's naming is not reconciled anywhere, and `is_usable` is built from literals rather than "from the edge bounds alone".

**How the tree-wide findings show up here**
- **T-layout:**
  - `crates/skein-fail-stop/tests/suite` contradicts testing.md 6. The crate also ships its fixture as a `[[bin]]` of a product crate and reads `CARGO_BIN_EXE_*` by hand rather than through `skein_scratch::paths`.
  - One binary sits outside `tests/suite`: `tests/world/tests/system_alloc`, which is justified.
  - All three step-test conventions appear: `src/tests.rs` (kv, fake-machine, fake-person); `src/tests/` via `#[path]` (sim); inline `mod tests` (scratch, fake-machine ×2, sim ×2, and skein-world ×6, together 1,383 lines inside `src/`).
- **T-worlds-as-kits:** 9 `#[path]`s plus the cross-package `[[bin]]` path (F8).
- **T-lints:**
  - 6 hand `[lints]` blocks. The one in `tests/kv/Cargo.toml` is the thinnest: no `wildcard_enum_match_arm`, `unwrap_used`, `allow_attributes` or `disallowed_*`.
  - 8 clippy.toml files in 7 variants.
  - 49 `#[expect]`s, of which kv has 19, all module-wide.
- **T-limits-shape:** kv follows io (`is_usable`, free `down`/`up`/`fire` with `&Env`).
- **T-primitives:**
  - 20 `Env { now: …, wall: … }` rewraps, 6 `fn limits()`, 23 literal `Limits {`.
  - Hand token namespaces: `crates/skein-kv/src/store.rs:25` (a fixed owner, so a store needs its own `FileIo`, which kv.md 12 records) and `testing/skein-world/src/files.rs:14`.
- **T-naming:** "conformance" also names `tests/world/tests/suite/real_conformance.rs`, which belongs to neither suite.

## Status-file cross-check

- **simulator.status.md:**
  - "Built: unknown (partly built)" says nothing.
  - "Not yet: … filesystem faults and power-loss semantics" is contradicted by the code. The filesystem faults (`config.rs` `no_space`, `read_only`, `io_error`, `hung`; checked in `tests/sim/…/fuzzy/files.rs:176-186`) and power-loss cuts (`Cut::PowerLoss`; the fake machine's crash model, `tests.rs:298-453`) are built.
  - Tree-metadata calls and 3.4 cgroups are indeed absent: `rg -i cgroup crates/skein-sim` finds nothing.
  - The file misses that the sim's own contract tests sit partly in `src/` (F8), and the LateReset gap (F15).
- **world.status.md:**
  - It lists the crate's name as "Not yet", but that is an open question, not a lag.
  - It misses that the simulated world's clippy.toml and the fake machine's docs call themselves test-only (F10), and the duplicated registry (F6).
- **kv.status.md:**
  - It misses the 5.4 and 10.3 departures (F1).
  - It misses that kv.md 9's world (model, referee, crash sweep) is only partly there (F12).
  - It misses that kv.md 12's per-request timeout is an unused field (F11).
- **testing.status.md:**
  - It does not record skein-fail-stop's `tests/` directory, or that fail-stop is missing from testing.md 6's list.
  - It does not record fake-person's lints (F13).
  - It does not record the `tests/conformance/real/tests/{cases,support}` and `tests/world/tests/steps` directories, which testing.md 6 does not list.

## Incidental

- `skein-kv` admission cost: each `Commit` deep-copies all queued ops (`store.rs:235,526`), then runs an O(n²) scan. With jig's large `queued_bytes` profiles this works against kv.md 7's "loop latency stays bounded".
- `skein-world` does not compile off unix despite its fences (F11).
- `crates/skein-sim-world/src/writer_pipe.rs:40` computes `capacity.checked_mul(2)` only to `expect` it, and discards the value.
- My own side effect: an early command wrote `/tmp/dummy` (a list of design file names). Nothing in the snapshot or any repository was touched.

---

# A13-lib-shell

# A13: lib, shell, fake peers, echo

Scores: Cleanliness 3, Organisation 3, Taste 4, Naming 3, Primitives 2, Elegance 4, Economy 3, Complexity 4

**Verdict:** The containers, cursors, stream vocabulary and intake in skein-lib are careful, small, well tested step code. The echo is a good template: one test convention per crate, 2 `#[expect]`s, a fault census that asserts every fault fell. The shell has `SAFETY:` on all 352 of its `unsafe` blocks. The weakness is lib as a *kit*. Its design catalogue (lib.md, and the "Everything lib exports" list in programming-model 10.2) lags the code. The component-kit draft of 2026-10-31 landed `Lifecycle`, `earliest`, `Tail`, `len_u64`, the hex, decimal and line helpers, `Delegate` and `next_mechanism_deadline`, but no design or status file records them. Its primitives are also lopsided: `len_u64` exists but no u32-to-usize helper, hex comes only one nibble at a time, `Env` has no methods, and there is no concat. So downstream re-implements them: jig and temper never call `len_u64` once and write 502 `u64::try_from(..len())` by hand. In the area itself, skein-fake-peers triplicates its listener host, and the shell exports about 10 undesigned test-keeper syscalls in the API that services link.

## Good

- `crates/skein-lib/src/{intake,stream,stream/held,writer,reader}.rs`: one demand vocabulary, met two ways. Each module's doc restates its contract in the design's words.
- `crates/skein-lib/src/growth.rs`: one growth rule and one floor, shared by five containers.
- `examples/echo/protocol/src/conn.rs` and `layer.rs`: one small handler per cell, `follow` applied in one place, a `CLOSED` placeholder. This is model 5.4 as written.
- `tests/echo/tests/suite/fuzzy/scenarios.rs`: the chaotic sweep asserts that every fault fell and every outcome appeared.
- `crates/skein-shell/src/drive.rs`: `drive` is about 25 lines and exactly shell.md 12.

## Top recommendations

1. **Complete lib's kit and its catalogue** (F1, F2). Add:
   - `lib::index(u32) -> usize`, the twin of `len_u64`;
   - `Env::with(limits)`;
   - `bytes::concat(&[&[u8]])`;
   - a `hex` module shaped like `base64` (`encoded_len`, `encode`, a bounded `decode`);
   - `bytes::is_utf8`, with a streaming `Utf8` check;
   - `earliest` for `Wall`.

   List every export in lib.md and in model 10.2.

   Win: about 600 hand conversions downstream, 27 concat copies, about 30 hex helpers, about 330 `Env` rewraps; 3 UTF-8 checkers become 1. Effort: lib S, migrations M (mechanical).
2. **Record the landed component-kit draft** in lib.md, shell.md and the two status files (F3). Effort: S.
3. **Fold skein-fake-peers' three hosts onto `Face`**, and merge its parallel dialect enums (F7). Win: about 200 lines and 12 `unreachable!` arms. Effort: M.
4. **Fence the shell's harness-only calls** into `skein_shell::keeper` and design them in shell.md 7.5 (F6). Win: about 10 items leave the service-facing API. Effort: S.
5. **Delete the step-test copies** of the naive search and the intake reference; run the short form in `tests/lib` (F9). Win: about 140 lines. Effort: S.
6. **Make the echo model lib** (F8):
   - use `earliest` and `Lifecycle`;
   - hold shutdown in one enum;
   - fix 6 stale comments;
   - forward `Proc` through `Delegate`.

   Win: about 60 lines, and the template stops teaching hand-rolled forms. Effort: S.

## Findings

**F1. lib's undocumented exports go unused downstream, which re-rolls them.**
- Documented nowhere in lib.md or model 10.2:
  - `len_u64` (`src/length.rs`);
  - `earliest` (`src/time.rs`);
  - `Tail`;
  - `Lifecycle`;
  - `bytes::{line_end, is_line_end, hex_digit, hex_value, decimal}` (`src/bytes.rs:63-145`).
- Model 10.2's list also omits `Buffer`, `Secret`, `base64`, `sha256`, `StateHasher`, `CryptoRng`, `Journal`, `RequestTable` and `Held`.
- Measured with `rg -o 'u64::try_from\([a-z_.()]*len\(\)\)'` against `rg -o 'len_u64\('`, excluding generated code:

  | Tree | `u64::try_from(..len())` | `len_u64` |
  |---|---|---|
  | jig | 280 | 0 |
  | temper | 222 | 0 |
  | skein | 72 | 157 |
  | smith | 89 | 120 |

- Private `earliest`/`earlier`: 12 in jig, 3 in temper, 2 in smith. `jig-domain-core/src/acts/bounds.rs:203` re-rolls it for `Wall`.
- Nibble hex helpers that duplicate `hex_digit`/`hex_value`: `temper-service/src/clients/google.rs:99`, `setup/routes.rs:47`, `start_refusal.rs:69`, `temper-client-view/src/forge_sign_in.rs:127`.

Fix: list every export in lib.md (sections 6, 8, 9 and a lifecycle section) and in model 10.2. Add `earliest` for `Wall`.

**F2. lib's primitives are lopsided, and each missing twin is written by hand.**
- **Widening.** `len_u64` covers usize to u64, but nothing covers u32 to usize:
  - lib itself keeps three private copies (`src/intake.rs:238`, `src/slab.rs:239`, `src/stream/held.rs:79`) and 10 inline ones (`buffer.rs:24,57,72,73,87`, `list.rs:76,104`, `queue.rs:87`, `stack.rs:64`, `tail.rs:73`);
  - skein's step crates have 522 single-line `usize::try_from(..).expect(` (`rg -o` over `crates examples`, tests excluded);
  - codec-generated code emits them by the thousand: 4,278 in `jig-client-protocol-wire/src/generated/v19.rs` alone.
- **Concat.** 27 identical `fn joined/join(parts: &[&[u8]]) -> Box<[u8]>`: 8 in skein (e.g. `skein-fake-forge-protocol/src/setup.rs:336`), 15 in temper, 3 in jig, 1 in smith.
- **Hex.** lib has nibbles only, unlike `base64`. About 30 slice-level hex functions sit in step crates (e.g. `skein-forge-setup/src/reconcile.rs:379`, `temper-client-tools/src/landing.rs:315`).
- **Env.** `Env` has no methods. `rg -U 'Env \{\s*now: …\.now,\s*wall: …\.wall'` counts 89 sub-env rewraps in skein, 162 in jig, 72 in temper; the top shapes are `limits.tls` (14) and `limits.http` (11).

Fix: as in recommendation 1. Have the codec generator emit `skein_lib::index`.

**F3. The component-kit draft landed in code but in no design or status file.**
- `docs/design/draft/component-kit.md` 3.1 and 3.5 propose `Lifecycle`, an earliest helper, hex and decimal parsing, a provided `Host::next_deadline` and a forwarding `Host`. All are in code.
- shell.md 12 lists `next_deadline()` and `next_policy_deadline()`, but the code requires a third method, `next_mechanism_deadline()` (`crates/skein-shell/src/drive.rs:44`), with 122 implementations across the trees.
- `Delegate` (`drive.rs:72-83`, a trait with a blanket generic impl, implemented 27 times) says in its own comment that it "departs from programming-model.md, 4 and 10" (`drive.rs:67`). That departure appears in neither shell.md nor shell.status.md, and draft 6.1 left its form to the user.

Fix: amend lib.md and shell.md 12 and 14, or record each item in the status files.

**F4. Lib's stream-output types use different words from the stream they extend.**
- lib.md 7 and `src/stream.rs` say "side below" and "side above" (13+14 and 12+7 uses).
- lib.md 7.1 and `src/stream/output.rs` say "lower" (5 and 4 uses), "classic" (9 and 5), "native", "affine", "obligation" and "right", and use "actual" 5 times (`output.rs:50,59,61,63,65`).
- `output.rs` cites "(lib.md, 7.1)" 21 times in 68 lines.
- `journal::Released::{Some, None}` and the echo's `conn::Stop::None` shadow `Option`'s variants.

Fix: rewrite 7.1 and `output.rs` in section 7's words, cite once in the module doc, and rename the variants (`Moved`/`Waiting`; `Stop::Running`).

**F5. `RequestTable` is a component with one consumer, not a primitive.**
- It is 734 lines, plus about 1.7k lines of tests: `src/tests/requests.rs` (638), `src/tests/requests/admissions.rs` (153), `tests/lib/src/request_table.rs` (750), and part of `memory.rs`.
- Its vocabulary is one client's delivery policy: `SignedOut`, scope confirmation, retention, link up or down.
- `rg -l RequestTable` finds a single consumer, `jig/crates/jig-client-domain/src/domain.rs`. Compare `Journal`, which 42 files in jig, temper and smith use.
- Inside it:
  - 10 full scans of `order` (`src/requests.rs`);
  - `reclaim` is quadratic: `for _ in 0..limits.requests`, rescanning from the start (`requests.rs:487`);
  - the eligibility expression is written three times (`:608`, `:637`, `:663`);
  - `save`, `changed`, `restored` and `anchor` are lifecycles held as flags beside `state`, against model 5.4.

Fix: move it to a sibling crate (the "component kit" role of draft 6.2) or to jig. Give it an attempt index and a single `eligible()`.

**F6. The shell's public, service-linked API carries undesigned test-harness syscalls.**
- `crates/skein-shell/src/process.rs:69-181` exports `poll_child`, `signal_kept_child`, `close_keeper_fd` ("an exclusively owned test keeper descriptor"), `abandon_binary` ("abandoned by a failing test"), `open_child_pidfd`, `start_binary` and the subreaper and cgroup calls.
- `linux.rs:93` and `darwin.rs:205` export `wait_readable`, a blocking `poll` outside the kernel.
- shell.md 7.5 names the cgroup, subreaper and pidfd helpers and the tree keeper's four calls. It does not name `poll_child`, `signal_kept_child`, `close_keeper_fd`, `abandon_binary`, `open_child_pidfd` or `wait_readable`.
- Every caller is a test or tool: `tests/conformance`, `skein-world`, `temper/crates/temper/tests/support/parent.rs` and others.

Fix: move them into `pub mod keeper`, design the module in 7.5, and say "for harnesses alone" once.

**F7. skein-fake-peers writes one listener host three times.**
- **Event routing.** `llm.rs:214-300` and `oauth.rs:212-288` are near-identical: Listening; Accepted with reject-when-full and token allocation; Output and Stream found by owner; Closed; Failed; the 8 impossible variants. `forge.rs:326-421` repeats it.
- **Observations.** The recording cap is triplicated: `llm.rs:159-212`, `oauth.rs:170-210`, `forge/observe.rs:9`.
- **`Listening`.** Its 3 impls are identical (`lib.rs:45-82`).
- **Dialect enums.** `forge/layer.rs` keeps three parallel enums, `Limits`, `Service` and `Server`, each Forgejo or GitHub. They must match, so the code carries 4 `unreachable!("paired layer")` (`layer.rs:89,98,164,212`) and 6 `let … else { unreachable! }` (`forge/github_delivery.rs:36,51,57,82,147,265`).
- **Layout.** `layer.rs` declares an enum before its `use` lines.

Fix: let `Face` route listener and connection events and own a generic `Observations<T>` (generics are allowed in ordinary Rust). Use one `Dialect { Forgejo{..}, Github{..} }`. About 200 lines.

**F8. The echo template hand-rolls what lib and shell provide.**
- `earliest` is unused; the min of two deadlines is hand-matched at `service/src/lib.rs:226-235` and `protocol/src/layer.rs:157-170` (and in shell's `drive.rs:34-40`).
- The shutdown lifecycle is spread over an enum and two flags: `Shutdown` + `stopping` + `aborting` (`layer.rs:29-30,88`), and `signals: Option` + `signals_closing` (`service/src/lib.rs:145-146`). Model 5.4 says "never a lifecycle spread over flags and `Option` fields", and lib exports `Lifecycle`.
- `tables.active: Set<Id<Conn>>` (`layer.rs:42`) mirrors the slab's live entries.
- `tests/echo/src/proc.rs:51-120` forwards `Host` by hand, which is what `Delegate` was written for.
- The limits come in two shapes, `is_usable` (protocol) and `check() -> Result` (service), and neither is the model's `valid` (T-limits-shape).

Fix: use `earliest`, `Lifecycle` and `Delegate`, and record the shape choice in examples.md.

**F9. The step tests copy the fuzzy suite's references verbatim.**
- `crates/skein-lib/src/tests/bytes.rs:79-111` matches `tests/lib/src/search.rs:7-43`: `naive`, `naive_count`, `up_to`, `fill`, and the same test body with `ROUNDS` in place of 20,000.
- `src/tests/streams.rs:237-270` matches `tests/lib/src/intake.rs:15-48`.
- The copies have drifted: `.unwrap()` against `.expect`, a `for` loop against `position`.

Fix: run the 300-round form in `tests/lib`'s focused suite, delete the copies, and amend lib.md 10.

**F10. Stale comments.**
- `examples/echo/domain/src/lib.rs:50-51` says io's `Shutdown` "is not built", but `service/src/lib.rs:183` adopts it.
- These name paths that do not exist: `tests/echo/src/lib.rs:15-17` (`tests/scenarios.rs`, `tests/fuzzy_scenarios.rs`, `tests/real.rs`), `tests/echo/Cargo.toml:22`, `tests/echo/tests/suite/scenarios.rs:6`, `tests/lib/src/lib.rs:6` and `tests/lib/Cargo.toml:13` (`tests/memory.rs`).
- `tests/lib/Cargo.toml:27` cites `skein_io::kernel::Error`, but the crate does not depend on io.
- `crates/skein-fake-peers/clippy.toml:1` says "the test crates under tests/", but the crate lives in `crates/`.
- `Journal::failed` has a dead branch: both paths set `stopped = true` (`src/journal.rs:193-197`).

**F11. Tree-wide findings as they appear here.**
- **T-layout:**
  - skein-shell mixes four test conventions: inline `mod tests` in 10 files, `src/tests.rs`, sibling `*_tests.rs` (`listener_tests.rs`, `startup/lower_tests.rs`), and `linux/ring/tests/`;
  - lib mixes two (`src/tests/`, plus inline in `tail.rs` and `length.rs`), against lib.md 10's "`src/tests/`";
  - the echo shell owns `tests/echo`'s `end_to_end.rs` through `[[test]] path = "../../../tests/echo/…"`.
- **T-worlds-as-kits:**
  - `tests/fake-peers/tests/suite/github_memory.rs:2` includes another crate's tests (`../../../fake-forge/tests/github/fixtures.rs`);
  - 5 more `#[path]` pull `tests/{forge,llm}/*.rs` into the suite;
  - `listener_calls.rs` is compiled into both system modules by `#[path]` (`linux.rs:29`, `darwin.rs:20`);
  - echo-world and echo-shell depend on each other.
- **T-lints:** the 6 non-workspace `[lints]` blocks here come in 6 variants. fake-peers lacks `allow_attributes`, `allow_attributes_without_reason`, `elided_lifetimes_in_paths` and `panic`.
- **T-naming:** "actual" occurs 22 times here (F4; the module docs of `tests/fake-peers/tests/suite/{llm,oauth}.rs:1`).

## Hypotheses

**14: refined.** In order:
- **`Env::with`:** confirmed. 89 sites in skein and 332 across the four trees; it belongs in lib, since `Env` is lib's and a generic method there is allowed.
- **Widening:** confirmed and larger than stated (F2). It belongs in lib beside `len_u64`, and in the codec generator.
- **`len_u64`:** not a gap but a catalogue failure. It exists, and jig and temper never found it (F1).
- **Concat:** 27 copies, not 17 files; it belongs in `lib::bytes`.
- **`is_utf8`:** three implementations exist: codec's `text_is_valid` (25 skein call sites), json's private streaming `Utf8` (`skein-json/src/utf8.rs`, kept because from_utf8 is banned), and `from_utf8` under `#[expect]` (`skein-forge/src/github/request.rs:487`, and 52 such expects across the four trees by their reason text). Put a streaming check plus `is_utf8` in lib; codec and json build on it.
- **`Lossy<T>`:** no such type exists. Drop-and-count is hand-written 38 times in 24 files, many of them capped fact queues *inside domains* (`skein-forge-client/src/domain.rs:150`, `jig-domain-fleet/src/facts.rs:67`, `temper-engine-domain/src/link.rs:368`). Model section 3 says a fact is never lost inside. Do not add it to lib: remove those departures first; a sink-side form, if any, belongs with io's streams.
- **fake-peers lints:** not workspace lints. The crate is std code using rustls, `Rc<RefCell>` and `Vec`, so `std_instead_of_core` and its kin would fire. It should take the ordinary-Rust test block plus `panic` and `missing_debug_implementations`.
- **The 5 `panic!`s:** confirmed (`llm.rs:299`, `forge.rs:421`, `oauth.rs:286`, `transport/native.rs:101` are impossible cells; use `unreachable!`). `forge.rs:298` is a caller precondition on `setup_mut`: use `unreachable!` or return `Option`.

**Does lib's API shape invite the duplication?** Yes, in three ways:
- the catalogue is incomplete (F1);
- primitives come in single-ended halves (`len_u64` without its twin, hex nibbles without hex text, `earliest` for `Time` only);
- `Env` is behaviourless (F2).

## Status-file cross-check

- **lib.status.md:**
  - its `get2_mut` gap is confirmed: there is no `get2_mut` in `src/slab.rs`;
  - it states that gap twice and cites itself ("(see lib.status.md)");
  - it misses F1's undocumented exports, F3's landed draft, and `stream/output.rs`'s state against 7.1;
  - lib.md 10's list of test modules is stale: there are 13 modules plus 2 inline.
- **shell.status.md:**
  - it misses `Delegate` and its recorded-in-code departure, `next_mechanism_deadline`, and F6's undesigned exports;
  - "Built: 14, the portable face": the face also exports `Delegate`, which 14 does not list.
- **examples.status.md:**
  - "Built: unknown (partly built)" is a placeholder;
  - the 6 gap (referee reads service state) is confirmed at `tests/echo/src/referee.rs:249`;
  - the 7 gap (no domain worlds) is confirmed;
  - it misses the connection's owner transitions (Drain and Abort, `protocol/src/conn.rs:13-22`), which examples.md 3.4 does not table.

## Incidental

- `crates/skein-shell/src/process.rs:163`: `Err(Error::Other(45))` hard-codes Darwin's ENOTSUP; use `libc::ENOTSUP`.
- `RequestTable::reclaim` is O(requests²) per call (`requests.rs:486-513`). At large limits this is a per-iteration cost risk.
- `CryptoRng` derives `Clone` (`src/crypto_rng.rs:7`): a clone repeats the stream's bytes, defeating lib.md 8's "never yield the same bytes". The only uses found are tests (`skein-forge-setup/src/tests/github_host.rs:506`); prefer a test-only constructor.
- `crates/skein-fake-peers/src/forge/shared.rs:8,42`: the unnamed constants `7 << 60` (the external reply range) and `7 << 28` partition the shared domain's token space with no name or check against other callers (hypothesis 4).
- The readiness backend's slot table (`crates/skein-shell/src/readiness.rs:14-31,128`) re-implements an index-plus-generation slab with a *wrapping* generation, where lib's `Slab` retires a slot whose generation would wrap.
