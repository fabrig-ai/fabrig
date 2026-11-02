# skein code-quality review, 2026-10-31


Resolved 2026-11-01: the eleven QA binaries are removed. Unique rows run in focused step tests/worlds, fuzzy sweeps or the existing browser stories; the real hand main paths are browser tests. Struck lines below preserve the dated assessment.

**Scope.** The whole `skein/` tree of fabrig at `main` = `4a6683c72` (2026-10-31): 34 crates, 13 test kits, 32 test worlds and 2 examples. That is about 470k hand-written lines; about 12k generated codec lines were left out.

**Rubric.** Cleanliness, organisation, taste, naming, primitives (orthogonality and decoupling), elegance, economy and complexity. It was applied to tests as well as code. Correctness was out of scope; bugs seen in passing are collected in section 6.

**How it was done:**
- One holistic reviewer looked across the tree (A0). It left 15 hypotheses, which then went to 13 area reviewers. Each area reviewer took code and tests together (A1–A13).
- All of them worked statically on a frozen, read-only copy of the snapshot. Nothing was built and no tests were run.
- skein's programming model was the frame. Its constraints were treated as givens, not defects.
- skein has no `tech-debt.md`. Each reviewer instead checked its area against the design docs and the `.status.md` files beside them.
- I spot-checked the headline claims myself (section 9). The figures are the reviewers' own `rg`/`wc` measurements on the snapshot. Line "wins" are estimates, and they overlap between findings.

The full area reports are in `skein-review-2026-10-31-areas.md`, beside this file.

---

## 1. Verdict

**skein is in noticeably better shape than jig.**
- **Layering holds mechanically.** No product crate depends on a test crate, skein-lib depends on nothing, and step crates stand only on lib and their peers.
- **The house rules are explicit and mostly kept.** The tree has 0 `#[allow]`, 1 `#[ignore]` and 0 TODO/FIXME.
- **The protocol cores are among the best code in the repository.** These are the HTTP client and server, the SSE reader and writer, the JSON tokenizer, io's stream and listener, the simulator, skein-ui, the oauth issuer and the codec's step code.
- **Score means are higher than jig's on every dimension but one.** Primitives ties with jig at 2.1 and stays the weakest.
- **The removable share is far smaller.** About 19–23k lines could go, roughly 4–5% of the tree, against about a sixth for jig.

**The debt lies between crates, not inside them.** It comes from four things:
1. **A kit whose catalogue lags its code.** skein-lib, skein-json and skein-http either lack the small twins that callers need, or have them but nothing lists them.
   - So every area writes its own: the JSON whole-buffer pump about 25 times, calendars 4 times, hex about 30 times, LRU tables 6 times, and u32→usize widening 522 times in skein alone.
   - Downstream rewrites what exists: jig and temper call `len_u64` 0 times and write `u64::try_from(..len())` 502 times.
2. **Second dialects built beside the first, not on a neutral root.** GitHub stands on Forgejo's modules, Anthropic on OpenAI's, and the GitHub fake on the Forgejo fake. Each leaves copied Limits, Failures and walkers, mirrors mapped across by identity functions, and runtime dialect tags checked by `unreachable!`.
3. **Departures from programming-model 5.4 in almost every area, almost none recorded.** Lifecycles are held in flags and `Option`s beside a thin tag. Flat 50–85-variant enums are re-matched in every module.
4. **Test infrastructure that was copied, not shared.** The causes:
   - worlds used as kits;
   - `#[path]` reaching across crates;
   - three or four step-test conventions per crate;
~~   - 11 `qa_*` binaries that largely repeat gate tests;~~
   - harnesses copied, notably four HTTP adapters in front of a fake that already has its own server;
   - fakes that share the product's types, which testing-strategy 4 forbids.

**The design and status documents have drifted further than the code.**
- The layout in `testing.md` 6 and the crate graph in `README.md` 3 are both contradicted by the tree.
- Status files miss most departures. Some list as "not yet" things that are built: binary digests, filesystem faults, five fake-llm items, fake-checkout. Others claim as built things that are not: channel 11, "uncapped outputs" in forge-client, io's "checked by `valid`".
- Runtime forge membership is designed only in a temper draft.

**Fixing the kit pays twice.** Most of the primitive gaps that the jig review blamed on jig are real gaps in skein, and skein pays for them itself. Fill them first, in skein, as AGENTS requires; jig and temper then shrink by hundreds of sites more.

## 2. Scorecard

Scores are 1–5 (5 = exemplary), as given by each reviewer. They are not normalised across reviewers.

| Area | Clean | Org | Taste | Naming | Primitives | Elegance | Economy | Complexity |
|---|---|---|---|---|---|---|---|---|
| A1 forge: GitHub dialect + crate root | 4 | 3 | 3 | 3 | 2 | 3 | 3 | 3 |
| A2 forge: Forgejo dialect + conformance tools | 3 | 2 | 3 | 3 | 2 | 3 | 2 | 3 |
| A3 forge-connection + machine/protocol worlds | 3 | 2 | 3 | 3 | 2 | 2 | 2 | 2 |
| A4 forge-setup + forge-client | 3 | 2 | 3 | 3 | 2 | 3 | 2 | 3 |
| A5 fake forge + fake-checkout | 3 | 3 | 3 | 3 | 2 | 3 | 3 | 3 |
| A6 io, tls, channel, heap | 3 | 3 | 3 | 4 | 2 | 3 | 3 | 3 |
| A7 http, json | 3 | 4 | 4 | 3 | 2 | 3 | 3 | 4 |
| A8 codec, codegen | 3 | 3 | 3 | 3 | 2 | 3 | 2 | 3 |
| A9 llm, llm-connection, fake llm | 2 | 2 | 3 | 2 | 2 | 3 | 2 | 3 |
| A10 oauth (client, accounts, issuer, fake) | 3 | 3 | 3 | 3 | 2 | 3 | 2 | 3 |
| A11 ui, ui-web, ui-text, browser kits | 3 | 3 | 3 | 4 | 2 | 3 | 3 | 3 |
| A12 sim, world, conformance, kv | 3 | 3 | 3 | 2 | 3 | 3 | 3 | 3 |
| A13 lib, shell, fake peers, echo | 3 | 3 | 4 | 3 | 2 | 4 | 3 | 4 |
| **Mean (A1–A13)** | **3.0** | **2.8** | **3.2** | **3.0** | **2.1** | **3.0** | **2.5** | **3.1** |
| A0 holistic | 3 | 3 | 4 | 3 | 3 | 3 | 3 | 4 |
| *jig mean, same day, for comparison* | *2.9* | *2.3* | *2.8* | *2.8* | *2.1* | *2.4* | *1.8* | *2.5* |

The pattern is consistent:
- **Primitives are weakest everywhere** (12 of 13 areas give 2), and economy comes next.
- **The low-level protocol kits fare best:** http/json, lib and io.
- **The forge stack and llm fare worst:** forge-connection, forge-setup and llm.

## 3. What is good (keep it, copy it)

- **State machines as the model writes them:**
  - `crates/skein-io/src/stream.rs` and `listener.rs`: data-carrying states, completions decoded into `Happened`, and `mem::replace` then `follow`;
  - `examples/echo/protocol/src/conn.rs` and `layer.rs`: one handler per cell;
  - `crates/skein-ui/src/view.rs:1-9`: the state table in the module doc, with the code following it;
  - `crates/skein-json/src/tokenizer.rs:595` `demand(before, after)`: the demand below is stated in one place after every transition.
- **The rules themselves:**
  - `clippy.toml`, where every ban gives its reason and names the replacement;
  - the `scripts/old-forms.txt` ratchet, where four forms are already at 0 and the component-kit forms fell from 39 to 8.
- **Shared primitives done right:**
  - `skein-lib`'s priced containers (`List::price`, `Intake::price`), `Secret`, `growth.rs` (one growth rule for five containers) and the stream/intake vocabulary;
  - `skein-http/src/body.rs`, one body reader for both sides;
  - `skein-llm/src/filter.rs`, one shared mechanism with each dialect owning only its table;
  - `skein-forge`'s neutral `directory.rs`, `measure.rs`, `marker.rs` and `host.rs`.
- **Design as data:**
  - `skein-forge/src/forgejo/decode/walk.rs` (static field tables, required-field masks, a bounded stack);
  - `forgejo/plans/failure.rs:30` `classify` (forge.md 5.4 row for row);
  - `forgejo/failure.rs`, a refusal table read by both client and fake, so the two cannot drift;
  - `skein-llm/src/derive.rs`, where declared quantities derive every limit and each capture is cited beside it (model 6.3 done right).
- **Contracts stated once and checked:**
  - `skein-io/src/kernel.rs` with `Complete::is_valid`, against which `tests/ring` and `tests/readiness` hold every backend;
  - `skein-sim/src/lib.rs:59-121` "Its choices, where the contract leaves one";
  - `skein-oauth-issuer`: stateless, its checks in the design's numbered order, all credentials `Secret`, all types `Debug`.
- **Tests worth copying:**
  - **Fault coverage.** Fuzzy sweeps that assert every configured fault fell: `tests/sim/.../fuzzy/exchange.rs`, `tests/echo/.../fuzzy/scenarios.rs`, and the http/json sweeps.
  - **Round-trip properties.** `crates/skein-forge/tests/suite/forgejo_items.rs` checks five properties per transcript: it decodes, re-encodes, routes back, answers unchanged, and mutations name their field.
  - **Named sizes.** `tests/ui/tests/suite/limits.rs:32-58` tests named rows from both sides, and `tests/llm/tests/suite/memory.rs` keeps 19 named ceilings.
  - **Golden data.** `tests/kv/tests/suite/golden.rs` holds golden bytes that are never regenerated.
  - **Readable names.** Test names read as behaviours almost everywhere; forge-setup's average 8.9 words.

## 4. Cross-cutting themes, ranked by payoff

### T1. The kit's catalogue: primitives missing, half-built or hidden (highest payoff)

Every area reviewer found the same small mechanisms rewritten. The causes are three:
- **No listing.** `lib.md` and model 10.2's export list lag the code. The component-kit draft's items landed today with no design or status entry: `Lifecycle`, `earliest`, `Tail`, `len_u64`, the hex/decimal/line helpers, `Delegate` and `next_mechanism_deadline`.
- **Single-ended halves.** `len_u64` exists without its u32→usize twin, hex works a nibble at a time, `earliest` covers `Time` but not `Wall`, and `Env` has no methods.
- **Machinery that is `pub(crate)` where callers need it.** skein-http's header predicates and request-head measure, and skein-kv's framing constants.

| Mechanism | Copies measured | Home |
|---|---|---|
| Whole-buffer JSON tokenizer pump | ~25 in 3 idioms (6 hand Fill/Scan, ~16 `Intake::meet`, 3 `Held`); skein-json's own `parse` and `validate` are one loop | `skein_json::tokenizer::Feed` (slice-fed driver), rebuild `parse` and `validate` on it (A7 F1) |
| u32→usize widening | 522 in skein step crates (137 messages); 4,278 in one generated jig file | `skein_lib::index`, emitted by codegen |
| `u64::try_from(len)` vs `len_u64` | skein 72 vs 157; jig 280 vs 0; temper 222 vs 0 | list it (A13 F1) |
| `Env { now, wall, limits: … }` rewrap | 89 skein, 162 jig, 72 temper | `Env::with` |
| Measured concat / `joined` | 27 identical fns (8 skein); 12 `Measure` structs | `bytes::concat`; `Measure` in lib |
| Hex | ~30 slice-level fns; 4 decoders + 5 encoders in skein-forge alone | `lib::hex`, shaped like `base64` |
| UTF-8 | `skein_codec::text_is_valid`, json's private `Utf8`, `from_utf8` under expect (52 such expects across the trees) | `bytes::is_utf8` + streaming `Utf8` in lib; codec and json delegate |
| Calendar | forgejo `time.rs` (RFC 3339 + IMF read, year loop to 2554), http `response.rs` (Hinnant), ui `phrases.rs`, example view `state.rs`, forge machine world `imf()` | civil date on `Wall` in lib; IMF-fixdate read+write in skein-http; RFC 3339 at skein-forge's root |
| LRU table | 4 in skein-forge (drifted), 2 in oauth-accounts | `skein_lib::Lru` (model 10.2: "what is missing goes into lib") |
| `List` removal | 4 hand-rolled in fake forge, 3 via `List<Option<_>>` | `List::remove` |
| Percent coding / URI parse | 3 encoders, 3 decoders, 3 URI parsers in oauth; 3 in forge; ~7 more elsewhere | lib or skein-http percent codec; one parsed redirect URI |
| HTTP head parsing | client and server each have `field`, `persistent`, `content_length`, transfer-coding walk, `Line`/`idle`, `Room`; llm re-implements tchar/value/trim and the head length | shared `header.rs`; export predicates and a head measure |
| CRC tables | 256-line literal CRC-32 in gzip; `const fn` CRC-32C in kv under 4 expects | `skein_lib::crc` |
| Credentials as raw bytes | skein-oauth/accounts/fake: 25 of the tree's 39 `missing_debug_implementations`, a hand clone, 6 push-match workarounds | use `skein_lib::Secret`, which already exists |
| Token namespaces | hand bit ranges in 5+ places, plus the lab browser's | `Token::prefixed`; see T9 for io op tokens |

**What the jig review asked of skein, checked here:**
- Exists but nobody finds it: `skein_json::parse` (jig's `jig-protocol-config/src/document.rs` rewrites it line for line) and the UTF-8 check (in `skein-codec`, not lib).
- Truly missing:
  - a skein-io `owner()` helper, a fixed-capacity name and `skein_ui::date`;
  - skein-kv's exported prices (`frame.rs:12`, `map.rs:15`, `snapshot.rs:16-17` are `pub(crate)`, and jig copies 26 and 9);
  - a Trace gate in skein-world;
  - codegen "mirrors".
- Should not be added: `Lossy<T>`. Most drop-and-count sites are capped fact queues *inside domains*, which model 3 forbids, so those sites are departures to fix first (A13). forge-client is one of them (A4 F10).

**Win:** roughly 1.5–2.5k lines in skein and several thousand call sites across the four trees. One idiom replaces 2–4 in each case. **Effort:** lib/json/http items are small each, about a week together, with the contract first. The migrations are mechanical.

### T2. Second dialects built beside the first

| Where | Shape | Evidence |
|---|---|---|
| skein-forge | GitHub stands on Forgejo modules | 44 non-test lines in `src/github` name `crate::forgejo` (`document::Body`, `id::unsigned` (an alias of `bytes::decimal`), `time`, `path`, `Malformed`); twin `Limits`, `Failure`, `Bounds`, LRU tables and walkers (1,321 and 559 lines). The GitHub fake adds 20 more `forgejo::` uses, and forge-connection translates the twins field by field. forge.md 4.1 and forge-github.md 2 give each dialect its own names, times and ids; nothing records the sharing. |
| skein-fake-forge | GitHub fake on the Forgejo fake | The git route is neutral but typed by Forgejo's `Limits`, so GitHub (and temper) fabricate `protocol::Limits { calls: 1, … }`. `skein-fake-forge-protocol` is in fact the Forgejo fake (A5, A0 F16). |
| skein-llm | Anthropic on `openai/` | `anthropic.rs:8` re-exports 9 OpenAI types. Anthropic's failure classes and rate headers live in `openai/common.rs`. `openai::{Stop, Usage, Failure}` mirror the crate's own, mapped variant for variant. There are four copies of one 8-variant output enum with three identity mappings. `translate.rs` is really the Responses encoder. |
| skein-forge-connection | runtime dialect tags | Eight two-variant enums wrap the dialects, giving 28 `unreachable!`s that name a dialect. Five parallel per-forge lists are indexed by position: 160 `get(at)` and 38 `expect(..).as_mut().expect` pairs. |
| skein-forge-setup | shared vocabulary inside one dialect | `Need`, `Event`, `Reply`, `Failure`, `Stage` and `Why` live in Forgejo's `reconcile.rs`, and `Reply` carries both dialects' bodies (35 `GithubRows(_)` arms, 30 eight-line fallthroughs). GitHub's picker is fused into Forgejo's `Reachable`. |

**Fix:**
- **Root modules for the neutral pieces:**
  - skein-forge: `time`, `path`, `text::Body`, `decode::Malformed`, one `Limits` with `check(l, depth)`, a `plan` module (Failure, Bounds, Order) and the user table;
  - skein-llm: a `wire` module;
  - fake forge: a neutral git route.
- **Dialect state held by enum variants** (`Dialect::Forgejo { plan } | GitHub(Box<GitHub>)`), not by parallel lists.

**Splitting skein-forge into per-dialect crates is refuted.** A1 and A2 agree:
- forge.md 12 settles GitHub as a module of skein-forge;
- forge-connection and forge-setup use both dialects;
- a split would only move about 1k neutral lines into a third crate.

Once the root holds the neutral pieces, a split becomes mechanical, and build time can decide it.

**Win:** about 1.5k lines across A1–A5 and A9, plus two downstream translators. **Effort:** M, mostly moves.

### T3. Flat enums re-matched in every module

| Where | Measure |
|---|---|
| `forgejo/request.rs` `Request` (70 variants) and `server::Answer` (40) | 865 + 276 lines of variant lists in 9 files; 93 `unreachable!`; 7 `too_many_lines` expects citing "exhaustive request selection" |
| fake Forgejo | one 28-variant setup group repeated in 12 exhaustive matches; `missing_scope` and `table` are 75-arm matches |
| GitHub `decode/walk.rs` `Node` (85) and `request.rs` `Request` (53) | `Node::` 480 times; 36 variants repeat `owner, name`; one `Decoder` holds 17 families' state; read `Plan` mirrors `api::Read` with 17 accessor matches |
| forge-setup `Reply`, fake forge `Request` classification, llm `Output` | see T2 |

**Fix:** nest the enums per area or family, matched once by the dispatcher:
- `Request::Setup(setup::Request)`;
- `Node::App(app::Node)`, as `Protection` already is;
- decoders and plans as enums of per-family state.

**Win:** about 1.5–2k lines and about 100 `unreachable!`s. **Effort:** M, mechanical.

### T4. Lifecycles in flags and `Option`s: model 5.4, broken nearly everywhere and recorded nowhere

The model says: "no field is meaningful in some states only… never a lifecycle spread over flags and `Option` fields beside the state". The exemplars that keep it are io's stream/listener, the echo connection, and http/json. The breaches, with no status entry in any area:

| Machine | Shape |
|---|---|
| skein-kv `Store` | 6 bools + 5 `Option`s beside `phase`; `(phase, event)` tuple matches with catch-all arms (`store.rs:842`, `(_, _) =>` at `:1084`); 19 module-wide `#![expect]`, 47 closures, 8 `if let` on own enums |
| skein-io `Pipe` / `Signals` / `Child`; skein-channel machine | 6 bools + 8 `Option`s; 2 + 4; a data-less `Phase` + `Option` + 2 bools; 11 bools + 4 `Option`s |
| skein-io `FileIo` | no `Limits`, no `Env`, `expire` not `fire`, `Vec` buffers under a module-wide expect |
| Forgejo plans | a `Stage` beside 10–22 fields each; 35 `if let`/`let … else` on own enums; tuple scrutinees with `_` |
| forge-connection | `Connection` 33 fields, `Hooked` 21 with an 8-way boolean `pending()`; 15 `deadline =` sites where design 8.1 says one |
| forge-setup / forge-client | 9 + 13 `if let`/`let … else` on own enums; done-markers scattered |
| skein-oauth `Client`, oauth-accounts | `Phase` + `Option<Exchange>`; `State::Record` 5 bools; `Exchange` 19 fields; five parallel per-account lists read 124 times |
| skein-llm `Client`, llm-connection | 39 fields, 8 data-less states, 16 `state ==` tests, no `mem::replace` |
| skein-lib `RequestTable`; echo template | flags beside `state`; `Shutdown` + `stopping` + `aborting` (the template applications copy) |

**Fix:**
1. Record the departures now: a line per crate in its status file.
2. Refactor area by area, the echo template first, since it is what applications copy.
3. The rules checker (programming-model 10.3, still "Not yet" in STATUS.md) could catch the mechanical part: tuple scrutinees, and `if let` on own enums.

**Effort:** L in total, but parallel by area.

### T5. Test infrastructure copied, misplaced or sharing the product's types

**Layout:**
- **testing.md 6 is contradicted by the tree.**
  - Three `crates/` crates have `tests/` directories, among them skein-forge with 6.7k lines and 67 file-level expects that exist only because its tests compile under step lints.
  - 21 test binaries sit outside `tests/suite`.
  - Step tests follow three or four conventions in one crate: `tests.rs` + `src/tests/`, sibling `*_tests.rs` through `#[path]`, and inline modules. skein-oauth, oauth-accounts, skein-shell, fake-person-browser and forge-setup each use three or four.
- **Worlds double as kits.** Seven world crates are dependencies of others, 17 edges in all. There is one dev cycle (echo-shell ↔ echo-world). 10 of 93 `#[path]` reach outside their tree; for example, `skein-codec/src/lib.rs:16` compiles a file from `tests/codec`.

~~**Duplicate tests:** 11 `qa_*` binaries (4,824 lines) run only under `qa-skein`, outside the gate, and largely repeat gate tests one for one:~~
- codec 182 tests;
- oauth-accounts 10 of 12;
~~- browser `qa_look` at least 10 pairs;~~
- the `s12_*` names are pipeline vocabulary.

**Duplicate harnesses:**
- **Four HTTP adapters for one fake.** They put the fake GitHub service behind HTTP, in front of `skein-fake-forge-github::server`, which already does exactly that (A3 F1).
- **Seven io-playing harnesses** in the forge machine/protocol worlds, unified in one place by a `macro_rules!`.
- **tests/io has three world loops**, and none of them runs the `skein_io::stage` that services use.
- **Copies between worlds:**
  - `tests/readiness` copies `tests/ring`'s harness;
  - oauth-accounts has two owner harnesses (211 shared lines);
  - the HTTP/JSON worlds copy `meets`/`delivers`, `fault`/`draw`, `Seen`, `quote` and the transcript loader 2–6 times.
- **Step tests that are worlds in disguise.** 487 of forge-connection's 564 step tests drive the whole component over scripted HTTP.

**Worlds that bypass or fall short of their design:**
- The setup world plays the Forgejo plans without the host component its design says it plays (766 lines that would go).
- The channel world falls short of channel.md 11, although the status file says it is built.
- The kv world has no model, referee or crash sweep (kv.md 9).
- The llm world has no replay check.
- No GitHub decoder is tested against transcripts, mutations or a fuzzer, although Forgejo has a mutation sweep.

**Fakes that share the product's types**, against testing-strategy 4 ("No shared types"):
- skein-fake-oauth imports the client's codecs;
- skein-fake-channel imports the channel's frame codec;
- the fake CDP peer decodes with the browser kit's own decoder;
- the fake LLM reads requests with the product's codecs. fake-llm.md designs this, so it is a conflict between two designs.

**Fixtures:**
- 26-field `Config` and 24-field `Limits` literals are written out a dozen or more times. A `Config::calm(limits)` helper would replace them, as `skein-sim` has.
- Rows carry unlabelled numbers (forge machine memory rows of 15 bare integers).
- Some tests compute their expectation with the code under test (forge-client `budget.rs`).

**Win:** about 6–8k lines, the largest single bucket. **Effort:** mostly S–M moves and deletions.

### T6. Design and status drift

**Contradicted by the tree:**
- `testing.md:371`: "No crate under `crates/` has a `tests/` directory".
- `README.md` 3's crate graph:
  - eight rows disagree with Cargo.toml;
  - it lists a nonexistent `skein-playground`;
  - 14 crates are missing.

  README.status nevertheless says "Built: 1 to 4".

**Status files that list built things as "Not yet":**
- forge.status: 4.3 binary digests (`pin.rs` has all four, tested, and temper uses them). I checked this myself.
- simulator.status: filesystem faults and power loss.
- fake-llm.status: all five "not yet" items.
- fake-checkout.status: "product crate, separate tests".
- ui-web.status: typing into populated fields.
- browser.status: the native-dialog test.
- forge.status: "GitHub's dialect resolves none" (job attempts), although it now resolves them (A1).

**Status files that claim as built things that are not, or are only partly:**
- channel.status: 11 (A6 F12).
- forge-client.status: "uncapped outputs", though facts are capped and dropped inside the step.
- io.status: "limits checked by `valid`", though `Io::new` asserts only `is_usable`.
- forge-github.status: "Built: 2 to 12" includes 4.5, which the design itself calls designed, not built.
- llm.status: "without a client heap sum".

**Departures recorded nowhere:**
- T4, everywhere;
- runtime `Add`/`Remove`/`Abort` in forge-connection and forge-setup, designed only in `temper/docs/design/draft/playground-reset.md` (11 cites);
- `Delegate` (a blanket generic trait whose own comment says it departs from the model) and `next_mechanism_deadline`;
- skein-browser's second, recursive JSON parser;
- `Step<T>` and `Answer<'a>` in the GitHub dialect;
- the fake forge's `impl Iterator`s and hand `Debug`s;
- `json_text` and `string_parts`;
- `"verbosity":"low"` on every Responses request;
- codegen's `skein_bools: [bool; N]` in the generated API (806 downstream uses);
- `context.rs`'s hand `impl Default`.

**Vague or stale status text:**
- 7 "unknown" entries;
- `lib.status.md` cites itself;
- `playground.md` has no status file;
- `testing.md:473` cites STATUS.md sections that don't exist.

**Stale or wrong citations in code:**
- 7 module headers in forge-client;
- about 10 in forge-connection (sections 12, 6.1, 5 for 7.3, 2, 6);
- deliveries cited as forge.md 5 rather than 7 in forgejo.

**Pipeline and history vocabulary:**
~~- `qa_*` and `s12_*`;~~
- "decision 25", "runtime 20/24", "ruling 106", "director", "task C";
- "macOS run 40019066645" and "Extracted from temper's … at commit 246164f".

### T7. Lint configuration copied by hand

- **`[lints]` blocks.** 48 hand blocks come in 14 variants.
  - 9 lack `allow_attributes`, 7 lack `wildcard_enum_match_arm` and 6 lack `unwrap_used`.
  - `tests/browser` has only `unsafe_code = "deny"`.
  - Among `crates/`, skein-ui-web has 9 `_ =>` arms on its own enums.
- **clippy.toml copies.** 25 copies come in 14 variants. Several carry another crate's header, such as "the world harness" in skein-ui-web, codegen and sim-world.
- **Exceptions.** 638 `#[expect]`s carry 541 distinct reasons, and `manual_map`'s one reason ("the subset forbids closures") appears in 18 wordings.

**Fix:**
- Three named profiles (step, ordinary harness, binary), with a gate check that each package matches one.
- The closure-ban lints set once in `[workspace.lints.clippy]`.
- One `#![expect]` per step crate's tests root.
- `testing/clippy.toml`.

**Win:** about 1,000 TOML lines and about 200 attributes, and the gaps close. **Effort:** about a day.

### T8. Naming: one word for several concepts

| Word | Meanings |
|---|---|
| "conformance" | kernel-backend conformance (`testing/skein-conformance`); forge-fact recording (`testing/skein-forge-conformance`, `forgejo-conformance`, `github-conformance`, the last two without `skein-`); `tests/world/.../real_conformance.rs` |
| `Referee` / `Expectations` | swapped between skein-world's root and `domain` frameworks; "Expectation" also names a tree enum; downstream impls named `Judge` collide with `domain::Judge` |
| "owner" | the side above in json's collector/document/sse vs whoever closes the stream below in the tokenizer/client; forge-client's owner is also "top", "parent", "root" |
| `Step`, `Body`, `Refusal`, `Position`, `Held`, `Heard`, `Exchange`, `Phase` | 2–8 types each, sometimes in one module |
| "Codex"/"ChatGPT" | the shared Responses dialect, including errors shown to public-route callers |
| `skein-fake-forge-protocol` | the Forgejo fake |
| "actual" | 79 comment lines (old-forms `comment:actual`; tests/llm 23, oauth-accounts 14, io 11); 358 mentions in all |
| GitHub | `Github*` 124 / `GitHub*` 69 in forge-setup |

**Fix:** rename, don't merge. For skein-world's referee frameworks:
- A0 proposed a merge.
- A12 measured both as load-bearing: the root trait has 85 downstream impls; the domain framework has 67 `Referee::new` and 136 files.
- So rename the low-use root pair.

### T9. Real design gaps found by the review

These need a decision from the designer, not just cleanup:
1. **io operation tokens collide between `Io` and `FileIo` on one kernel.**
   - Io mints `slot<<32 | generation`; FileIo counts from 1. Slot 0 at generation 1 is token 1, which is also FileIo's first token. I checked this myself.
   - io.md 5 gives no way to tell which machine a completion belongs to.
   - Downstream works around it three ways: `skein-world/src/files.rs` sets bit 63, `skein-ui-text/terminal.rs` bit 62, and smith keeps a route table.
   - Proposal: a lane chosen at construction (A6 F1).
2. **Runtime forge membership** (`Add`/`Remove`/`Abort`) needs a section in forge-connection.md and forge-setup.md.
3. **`Delegate` and `next_mechanism_deadline`** are in shell.md 12 or departures.
4. **The shell's harness-only syscalls** (`poll_child`, `close_keeper_fd`, `abandon_binary`, `wait_readable`…) sit in the API services link. Fence them into `skein_shell::keeper` and design them in 7.5.
5. **skein-kv:** a step crate brought to the model, or recorded as ordinary Rust.
6. **Fakes sharing types:** the oauth fake's codecs and fake-llm.md's design conflict with testing-strategy 4.
7. **Channel `MAX_UP`/`MAX_DOWN`** keep the capped-queue reservation that model 2 dropped on 10-28. jig sizes loops from them.
8. **`skein_bools`** in generated code. The fix is configuration (`max-struct-bools`) but changes the generated API.

## 5. Area summaries

- **A1 forge: GitHub dialect.**
  - The plans are small pure `begin`/`heard` machines, the failure map names each fact, and the tests are thorough and in the subset.
  - It is built beside Forgejo instead of on a neutral root (T2), and `Node`/`Request` are flat (T3).
  - Tests sit in three places with redeclared helpers (`bounds()` ×9, `head()` ×9).
  - There is a verification gap against forge-github.md 14 (F15).
  - Est. −1.3–1.6k.
- **A2 forge: Forgejo dialect and tools.**
  - The step code keeps the subset well: no closures, `as` or indexing. The walk is table-driven, with `classify` and refusal tables.
  - Flat dispatch (T3) and plans as a `Stage` plus bags of fields (T4).
  - `plans/reads/size.rs` is a second answer measure that has already drifted.
  - The crate's `tests/` directory breaks the layout.
  - Est. −1.3–1.5k.
- **A3 forge-connection.**
  - The boundary, start transition, JWT cache and limit refusals are careful.
  - Below the boundary: five parallel per-forge lists and runtime dialect tags (T2), states as flags (T4).
  - Tests are three times the code; 487 "step tests" are a machine world.
  - Seven io harnesses, and four HTTP adapters around the fake GitHub service.
  - Forgejo's own connection is drawn from the calls' pool, against design 7.2.
  - Est. −3–4k, the largest per-area win.
- **A4 forge-setup and forge-client.**
  - The client is a faithful child domain (`valid`, a `recovery` table, pinned widths), though it drops facts inside the step.
  - Setup grew by accretion:
    - the admin step written 5 times, `stop` in 6 shapes, two halting styles;
    - the vocabulary inside Forgejo's reconcile;
    - forge-connection's transport stack duplicated;
    - four test conventions;
    - a world that bypasses the host.
  - Est. −1.3k.
- **A5 fake forge.**
  - Credential hygiene is good, and the observation codec and its table test are excellent; fake-checkout is small and sound.
  - The Forgejo layer is as big as the domain it translates (T3).
  - The two servers duplicate about 200 lines of connection machinery.
  - The git route is typed by Forgejo; deliveries are hand-built and then re-parsed.
  - It has the only `impl Iterator`s in skein's crates.
  - Est. −1.3–1.6k.
- **A6 io, tls, channel, heap.**
  - The io core is exemplary.
  - Weak points:
    - the operation-token collision (T9 1);
    - Pipe, Signals, Child and the channel in flags (T4);
    - FileIo out of shape;
    - two public limit checks, of which `Io::new` asserts the weaker;
    - `kernel::Error` spelled out 13 times;
    - tests/io never runs the stage;
    - tests/tls is a kit.
  - Est. −1.1k.
- **A7 http, json.**
  - The best step code in the tree: one `demand` per transition, 6 expects in 18.4k lines.
  - The later additions (collector, `parse`, `validate`, `json_text`) are in a terser second voice with their own vocabulary, and `json_text` departs from json.md 4.
  - Client and server duplicate head parsing; `parse` and `validate` are one loop.
  - Est. −1k here and about as much again tree-wide via `Feed`.
- **A8 codec, codegen.**
  - The step code is good: the two-pass token scan, recursion-free layout and the resumable patch machine.
  - Around it:
    - 10.1–10.3 (matching and comparison) implemented twice, in `plan.rs` and codegen's `diff`;
    - two patch-language readers;
    - a generator that edits its own emitted text;
    - tests/codec generating its fixtures twice, the second time a tautology;
~~    - six `qa_*` binaries repeating step tests.~~
  - The 6-line status file records nothing.
  - Est. −3.5–4k.
  - Note for jig's "mirrors" request: the generator's lossy tree and its text-splicing have to be fixed first (F17).
- **A9 llm.**
  - The derivation, filters, memory ceilings and fake domain are careful.
  - Anthropic sits on `openai/` with a mirror vocabulary (T2), and the client keeps its lifecycle in flags (T4).
  - The withdrawn Codex and Anthropic-bearer routes are the worlds' defaults (337 vs 82 helper uses) and the only routes the live suite covers.
  - Two llm-connection validators disagree.
  - The fake-llm status file is wrong on all five points.
  - Est. −0.6k.
- **A10 oauth.**
  - The issuer is the best code in the area.
  - The client and accounts don't use `Secret` (25 expects), `skein_oauth::Json` duplicates skein-json, and the fake shares the client's codecs.
  - States in flags; percent coding and URI parsing written three times; the fourth hand HTTP-over-TLS stack.
  - Est. −1.3–1.5k.
- **A11 ui and browser kits.**
  - skein-ui, ui-text and ui-web's pure/platform split are excellent.
  - skein-browser has its own recursive JSON parser and `Vec` tables, with module-wide expects in 6 of its 9 modules.
  - Role sets are written as slice `contains` checks (23), so adding a role breaks none of them, and two copies have drifted.
~~  - Four test conventions; `qa_look` duplicates tests that need no Chromium.~~
  - Est. −1.5–1.8k.
- **A12 sim, world, conformance, kv.**
  - The simulator, the conformance suite and skein-scratch are excellent.
  - skein-kv is the step crate furthest from the model (T4), and its admission check is written three times, with an O(queue²) deep copy.
  - Referee and Trace names collide (T8).
  - Hosting, memory choice and the release check are written twice, for the simulated and the real world.
  - Six spawn variants; a `#[path]` web.
  - Est. −1.1–1.4k.
- **A13 lib, shell, fake peers, echo.**
  - lib's containers and streams are careful, the echo is a good template, and all 352 `unsafe` blocks in the shell carry `SAFETY:`.
  - The weakness is lib as a catalogue (T1).
  - `RequestTable` (734 + 1.7k test lines) has one consumer, jig's client domain, so it is a component, not a primitive.
  - The shell's public API carries harness syscalls; fake-peers writes one listener host three times.
  - Est. −0.5–1k here, and the bulk of T1's downstream win.

## 6. Correctness issues seen in passing

Not verified by running anything. Items 1, 2 and 5 I checked in the source myself.

1. **io:** `Io` and `FileIo` operation tokens can collide on one kernel (T9 1). Every service that runs both without a hand tag is exposed.
2. **skein-oauth:** `Client::step` silently drops `SignIn`, `Refresh` and `Revoke` outside `Idle` (`machine.rs:279`: `if self.phase != Phase::Idle { return; }`). No terminal is sent, so a direct caller hangs. The accounts component guards against it.
3. **skein-io:** `file::Request::Store { bytes }` derives `Debug`, so any trace of io requests prints stored oauth token records in full (A10).
4. **skein-oauth:** `issuer_endpoint` admits `https://` alone and `https://a@b`; the issuer's own check refuses both (A10).
5. **`scripts/browser.sh:22`** runs `--profile browser`, but `.config/nextest.toml` defines only `browser-skein`, `-jig` and `-temper`, so the script fails at its start. browser.md and ui-web.md cite the wrong name too.
6. **llm-connection:** `valid()` returns true for limits that `Component::new` refuses (plaintext to non-loopback, TLS/io mismatch) (A9 F10).
7. **forge-connection:**
   - while Closing, a `Call` is dropped with no terminal;
   - an io `Closed` under a live holder may report an earlier written Forgejo step `Unavailable` (risk);
   - a duplicate runtime `Add` panics only in tests, and production's "ignored" path is untested (A3).
8. **forge-setup:**
   - an invalid `Add` is silently dropped;
   - `Remove` fabricates `Stage::Admin`/`"remove"`, which is wrong for GitHub;
   - an unusable io or TLS limit reports `Unusable::Zero` (A4).
9. **forge-client:** `identity.rs:60` admits a `Config` by `size_of::<Writer>()`, so admission depends on the platform (A4).
10. **GitHub walker:** its stack is sized by the constant `DEPTH` (10) whatever `limits.json.depth` says, so a skipped field nested past 10 fails `TooDeep` (risk, not seen) (A1).
11. **Forgejo paths:** capped at 12 segments, so the fake refuses files nested more than about six directories deep as `Malformed` (A2).
12. **skein-json:** the collector pops the caller's `below` queue to take back a demand. That is correct only if `below` was empty on entry, which json.md does not promise (A7 F9).
13. **skein-kv:** each commit deep-copies every queued op, then scans in O(n²), against kv.md 7's bounded loop latency (A12).
14. **fake GitHub:**
    - five RNGs share one seed, so their streams are correlated;
    - the hand constant `ROUTES = 81` is unpinned and panics if a route is added;
    - the fake Forgejo closes the connection on half of the domain's timeouts, where design 4 says 500 (A5).
15. **skein-io:**
    - a `Window` on a gone or closing stream submits `Op::Window` with `Fd(-1)`;
    - `Io::new` panics at `sockets ≥ 2^29` when a caller checked only `is_usable` (A6).
16. **skein-shell:**
    - `process.rs:163` hard-codes Darwin's ENOTSUP as 45;
    - the readiness slot table uses a *wrapping* generation, where lib's `Slab` retires the slot.
    - **skein-lib:** `CryptoRng` derives `Clone`, so a clone repeats the stream's bytes, against lib.md 8 (A13).
17. **codegen:**
    - the CLI writes goldens to `golden/v<N>` while the drift test reads `golden/every-v2`, so two families at one version collide;
    - `patch_check` leaves 7 temp crates behind;
    - `run/heap.rs` prices `Stack` where `Build` holds `List` (A8).
18. **Workspace and build:**
    - root `Cargo.toml` lists 9 skein crates twice in `members` (checked);
    - `skein-browser-world` is a workspace dependency with no user;
    - skein-world cannot build off unix despite its `cfg` fences (A0, A12).
19. **Panics on odd input:**
    - skein-browser's `contains` panics on an empty needle;
    - ui-web's `pixels` panics on a NaN size;
    - the lab browser's origin token ranges and `next += 1` are unchecked (A11);
    - skein-fake-peers uses `panic!` for impossible io cells (A0, A13).

## 7. Recommended order of work

**Phase 0: documents and decisions (hours; no code risk).**
1. **Re-sync the documents.**
   - Rewrite `testing.md` 6, and generate `README.md` 3 from Cargo metadata (or gate it).
   - Fix the wrong status files listed in T6, add `playground.md`'s, and replace each "unknown" with a statement.
   - Record every T4 departure as a status line per crate.
2. **The small fixes:**
   - deduplicate the root `members`;
   - fix `scripts/browser.sh`'s profile name;
   - add `growth_floor` to `old_forms.py`'s `EXCLUDE`;
   - amend programming-model 6.3's "Code written before this model…".
3. **Sweep the words:**
   - stale section citations;
   - history and pipeline words, and "actual" (old-forms `comment:actual` 79 → 0).
4. **Decisions for the designer** (T9): io lanes; runtime forge membership; `Delegate`; the shell's keeper module; kv as a step crate or not; fakes sharing types; channel `MAX_UP`/`MAX_DOWN`; `skein_bools`.

**Phase 1: deletion and configuration (days).**

5. **Delete the doubled and dead pieces:**
   - tests/codec's second generation and `skein-codec/src/lib.rs:11-17` (A8 F1, F2);
~~   - fold or delete the 11 `qa_*` binaries (A0, A8, A10, A11);~~
   - the dead API: llm F11, oauth's refresh-request codec and `RECORD_VERSION`, forge-connection `limits::Forges`, ui-web's pure `Timer`, forgejo `size.rs`, `time::RFC3339_BYTES`, and vestigial `first()`s.
6. **Lint configuration** (T7): three profiles with a gate check, the closure lints at workspace level, `testing/clippy.toml`, and true headers.
7. **Break the world-as-kit edges:** move the shared pieces into `testing/` kits (JSON `generate`/`reference`, LLM fixtures, TLS `pki`/`server`/`drive`/`meter`, the forge machine world's GitHub fixtures, the HTTP `reference`). Then gate "no `-world` dependency" and "no `#[path]` across crates".

**Phase 2: the kit (about a week; skein first, contract first, as AGENTS requires).**

8. **skein-lib:**
   - a complete catalogue in lib.md and model 10.2;
   - `index(u32)`, `Env::with`, `bytes::concat`, `hex`, `is_utf8` plus a streaming `Utf8`, `earliest` for `Wall`;
   - `List::remove`, `Lru`, a `Wall` civil date, `crc`, `Measure`, `Token::prefixed`, `copy_into`.
9. **The protocol kits:**
   - skein-json: `tokenizer::Feed`, with `parse` and `validate` rebuilt on it;
   - skein-http: shared head parsing, exported header predicates and head measure, `date::{read, write}`;
   - skein-kv: `frame_bound` and `row_price`;
   - skein-io: lanes, once decided.
10. **Migrate skein's own call sites** (mechanical). The jig review's phase on skein mechanisms (its item 12) then becomes available downstream.

**Phase 3: structure (weeks; parallel by area).**

11. **The forge stack:**
    - skein-forge's neutral root (T2), and nested `Request`/`Answer`/`Node` (T3);
    - forge-connection: `ForgeEntry` plus a `Dialect` enum;
    - forge-setup: a `plan` vocabulary module, an `admin` sub-machine, and a per-dialect `Reply`;
    - fake forge: classify requests once, a neutral git route, one connection core and structured deliveries.
12. **skein-llm:** a `wire` module, collapsing the mirror vocabulary.
13. **The oauth crates:** `Secret`; `Json` on skein-json; a fake vocabulary of its own.
14. **The browser kit and codegen:**
    - skein-browser onto skein-json and lib containers;
    - codegen keeps one diff and emits structure, not text surgery. That is the prerequisite for jig's "mirrors".

**Phase 4: states per model 5.4 (weeks; the echo template first).**

15. In this order:
    1. the echo;
    2. kv;
    3. io's pipe, signals and child, plus the channel;
    4. the Forgejo plans;
    5. forge-connection;
    6. the oauth client and accounts;
    7. the llm client and connection;
    8. `RequestTable`, after moving it out of lib.

    Consider building the 10.3 rules checker alongside, so the mechanical forms cannot come back.

**Phase 5: tests on the shipping code (weeks; can run alongside 3–4).**

16. **Forge worlds:**
    - one HTTP face for the fake GitHub service, the fake's own `server`;
    - one forge-world io harness;
    - forge-connection's Harness tests moved into the machine world;
    - the setup stories on `HostWorld`;
    - a GitHub transcript and mutation sweep.
17. **The other worlds:**
    - the io worlds on `stage`/`skein_world`;
    - readiness on ring's harness;
    - one oauth-accounts host;
    - the HTTP world kit;
    - the kv crash sweep with a model referee;
    - the channel world per section 11;
    - a replay check in the llm world.
18. **One step-test convention everywhere** (`src/tests.rs` + `src/tests/`).
    - Fixtures from constructors (`Config::calm`), with rows of literals.
    - The worlds' defaults moved to the public LLM route and an Anthropic key, plus a live key test.

## 8. Status-file cross-check (summary)

The individual cross-checks are in each area report. In sum:
- **No status file in skein records a T4 departure.** Most record no departure at all; codec.status is 6 lines.
- **Twelve "Not yet"/"Built" claims are contradicted by the code** (T6).
- **Two departures are recorded only in code comments:** `Delegate`'s, and the Forgejo own-connection pool.
- **What the status files get right is confirmed.** For example: io 5, 6 and 10; tls 3.6, the native face with no consumer (about 3.9k lines carried with no consumer); codec 13; oauth 7; forge-setup 9; lib's `get2_mut`; STATUS.md's rules checker; world 9's `skein-sim-world` in `crates/`.

## 9. What I checked myself, and caveats

- **Re-checked against the snapshot, and each held:**
  - `testing.md:371`'s claim against the three `crates/*/tests` directories;
  - 9 duplicate `members` in the root `Cargo.toml`;
  - `old_forms.py`'s `EXCLUDE` lacks `growth_floor`;
  - jig's `document.rs` rewrites `skein_json::parse`;
  - 41 lines in `skein-forge/src/github` name `crate::forgejo` (reviewers say 44, counting multi-line `use`s);
  - `skein-kv/src/store.rs:1084` `(_, _) => unreachable!`;
  - zero `len_u64` calls in jig and temper;
  - `"verbosity"` written in `skein-llm/src/translate.rs:108`;
  - forge.status's "Not yet: 4.3: binary digests" against `forgejo/pin.rs:26` `digest`;
  - oauth `Client::step`'s early `return` outside Idle;
  - tests/codec's `build.rs` regenerating into `OUT_DIR`;
  - io's `Id::token` layout against FileIo's `next_op` starting at 1;
  - `scripts/browser.sh`'s profile name.
- **Corrected one A0 claim:**
  - A0 said "a step crate, skein-llm, reaches a world" (`translate.rs:879`).
  - The use sits inside a `#[test]` in the crate's unit tests, a dev-dependency, not production code.
  - It is still an edge to a world, and A9 explains it cannot move out of the crate.
- **Where reviewers disagreed, and the resolution:**
  - **skein-world's two referee frameworks.** A0 proposed merging them. A12 measured both as load-bearing (the root trait 85 downstream impls; the domain framework 67 `Referee::new`, 136 files). Rename, don't merge.
  - **Splitting skein-forge.** A0 called it a split candidate. A1 and A2 refute it on the design (forge.md 12) and on its consumers. Move the neutral pieces to the root first.
  - **Owner token namespaces.** A0 hypothesised they collide. A6 found that the listed owner ranges don't overlap. The real collision is io's own operation tokens (T9 1).
  - **`Lossy<T>`.** A0 listed it as "confirmed missing". A13 argues against adding it: most uses are drops inside domains, which the model forbids.
  - **Where the forge conformance tools live.** A2 and A1 say keep them where forge.md 2.1 places them: list `tests/forge/github` in testing.md 6 and fix the package names. A4 would move them to `testing/`. I side with A2 and A1, since the design places them.
- **Everything else is the reviewers' work.** It is static, on a snapshot, without builds or test runs. Line wins are estimates and overlap: T1 and T2, and T3 and T4, share code.
- **Two reviewers wrote a stray scratch file in `/tmp`.** These were `/tmp/dummy` (a list of design file names) and `/tmp/x_int.txt` (test names). I removed both. Nothing in any repository or checkout was changed.
- **One deliberate departure from AGENTS §4.** At the user's request, the primary checkout `~/src/fabrig/main` was moved (it was clean, and stays detached) from `c30cf6a03` to `origin/main` `4a6683c72` before the review.

---

The area reports, unedited, are in `skein-review-2026-10-31-areas.md`.
