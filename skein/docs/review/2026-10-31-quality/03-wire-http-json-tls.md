**Code-quality review: skein-http, skein-json, skein-tls and their test crates**

The HTTP client, the HTTP server and the classic TLS client share one clear machine shape, and it is a good one. Three other machines (the SSE reader, the JSON collector and the TLS native face) do not follow it. The biggest saving is in the tests: the same machine-world skeleton is copied six times, roughly 1,000 lines that could go.

All paths below are relative to `T = skein (at 3371027)`.

## 1. Scope read

**Read closely:**
- **skein-http:** `client.rs`, `server.rs`, `body.rs`, `header.rs`, `client/head.rs`, `server/head.rs`, `client/request.rs`, `server/response.rs`, `sse.rs`, `sse/writer.rs`, the top of `sse/lines.rs`.
- **skein-json:** the top half of `tokenizer.rs`, all of `collector.rs`, the top half of `writer.rs`, the public part of `document.rs`.
- **skein-tls:** the top of `client.rs`, `client/native.rs` (state and entry points).
- **In-crate test harnesses:** `skein-http/src/tests.rs`, `tests/server.rs`, `tests/server/head.rs`.
- **Test crates:**
  - `tests/http/src/lib.rs`, the outline and helpers of all four `*_world.rs`, `tests/suite/client.rs`, the top of `memory.rs`.
  - `tests/json/src/world.rs` (World/Below), `tests/suite/world.rs`.
  - `tests/tls/src/drive.rs`, `native.rs` (its loop), `tests/suite/fake_peer.rs`, `native_output.rs`.
  - `tests/protocol/src/lib.rs`, the top of `wire.rs`, `tests/suite/worlds.rs`.
  - `testing/skein-world/src/stream.rs`.
- **Callers sampled:** `skein-forge` `github/decode/issues.rs` and `github/request.rs`; `skein-oauth/src/json.rs`; `skein-llm/src/openai/json.rs`.

**Skimmed or outline only:** `gzip.rs`, the rest of `sse/lines.rs` and `tokenizer.rs`, `collector/tagged.rs`, the rest of the TLS client modules, `tls/native/output.rs`, the remaining in-crate tests, the test-crate generators, references and transcripts, `tests/protocol` `client_end`/`server_end`/`world`.

## 2. Scores

| Criterion | Score | Why |
|---|---|---|
| Cleanliness | 4 | HTTP and classic TLS are exemplary. JSON and SSE docs carry stacked summary lines; test-crate docs name stale paths. |
| Organization | 3 | Good module trees, but head parsing is duplicated per side, gzip sits inside HTTP, and the world skeleton is copied. |
| Taste | 4 | Client, server and classic TLS read as written with care. The collector, the native face and the SSE reader read like another hand. |
| Naming | 3 | `settle`/`demand` both mean follow-state, `largest_read`/`largest_demand` mean the same thing, Line/Below/Held name one idea, and there are two `Wire` types. |
| Primitives | 3 | `body::Incoming` is the right shared read face. It has no write-face twin; `Line`, `Measure` and JSON navigation are missing. |
| Elegance | 3 | The client and server follow the problem's shape. The server's `Reply` and the collector do not. |
| Economy | 2 | Large near-copies, mostly in tests (about 1,000 lines), the HTTP sides (about 250) and JSON callers (about 400). |
| Complexity | 3 | Mostly right-sized. The collector and the native face are over-wrought; `json_text` validates four times. |

## 3. Strengths

- **`skein-http/src/body.rs`** is one read face for both the client's response body and the server's request body. It keeps the stream contract once and is documented well.
- **`client.rs`, `server.rs` and `skein-tls/src/client.rs` are the house idiom done right:**
  - the `mem::replace` move-out/assign-back transition (programming-model 5.4);
  - `settle()` called after every transition;
  - `waiting()` derived from the state alone;
  - an exhaustive Error→Fault map (`client.rs:976-990`).
  - Learning one of them does teach the other two.
- **The JSON `writer::Encoder` API is pleasant.** Forge's `body_json` (`skein-forge/src/github/request.rs:932-960`) reads like the JSON it writes.
- **`tests/protocol`** joins two real stacks by bytes, uses skein-world's referee, and says why it keeps its own loop (`src/lib.rs:17-23`).

## 4. Findings, ranked by payoff

**1. The machine-world skeleton is copied six times, plus four more stream-below simulators** (Economy, Primitives)
- **Evidence:**
  - Six worlds share the same skeleton (Settings{calm, chaotic, sent}, `limits(rng)`, `fault`, `draw`, Run/Fell, `run`/`check`, World{late, end, fail, below_acts, is_stalled, route}, Below/Life/Above): `tests/http/src/{client,server,sse,writer}_world.rs`, `tests/json/src/world.rs`, `tests/tls/src/world.rs`.
  - `late`/`end`/`fail` are line-for-line the same in `tests/json/src/world.rs:386-420` and `tests/http/src/sse_world.rs:322-353`; the SSE copy has lost its doc comments.
  - `fault` and `draw` exist verbatim in all six (`client_world.rs:172-178`, `server_world.rs:216-222`, `sse_world.rs:116-122`, `writer_world.rs:98-104`, `json/src/world.rs:138-144`, `tls/src/world/scenario.rs:201-212`).
  - `meets` and `delivers` are copied three times (`client_world.rs:441,1068`, `server_world.rs:611,629`, `tls/src/world.rs:235,252`). `delivers` is also production's `body.rs:283 answers`.
  - Stream-below simulators exist separately as `skein_world::stream::Wire`, `tests/tls/src/drive.rs:20-127` (also named `Wire`, same fields and take/answer), `tests/protocol/src/wire.rs` `Bottom`, and the Below in each world.
  - The suites mirror each other test-for-test: compare `tests/json/tests/suite/world.rs` with `tests/http/tests/suite/client.rs`.
- **Why it matters:** each copy re-states the lib stream contract checks. A fix to one does not reach the others, and the copies have already drifted.
- **Fix:** add a `testing/skein-world::machine` module (test code may use generics) holding:
  - `Below` (seeded cut, late, end, fail, grant, contract checks);
  - `fault`, `draw`, `meets`, `delivers`;
  - a `Settings` core that each world extends.
  - Retire `drive::Wire` in favour of `stream::Wire`.
- **Size:** about 1,000 of the roughly 5,400 world lines (estimate).

**2. HTTP head parsing and encoding are duplicated per side** (Economy, Organization)
- **Evidence:**
  - **Same field parser:** `field` at `client/head.rs:99-123` and `server/head.rs:255-283`; only the error type differs.
  - **Byte-identical copies:** `decimal` (`client/head.rs:281-293` / `server/head.rs:380-392`), `persistent` (`194-200` / `397-403`), `Parsed` (`12-17` / `33-38`).
  - **Near-identical:** `content_length` (`250-278` / `355-377`), and the Transfer-Encoding element loops (`211-239` / `319-350`).
  - **Encoding side:**
    - `Measure` and `put` are written three times (`client/request.rs:168-184`, `server/response.rs:452-468`, `sse/writer.rs:456-486`).
    - The field-name/value/reserved checks appear twice (`client/request.rs:131-151` / `server/response.rs:188-209`).
    - The header measure and write loops appear twice.
  - **Machine level:**
    - `Line` + `idle()` are identical (`client.rs:395-400,501-514` / `server.rs:526-531,640-653`).
    - The body-demand `Face` transition is identical (`client.rs:803-824` / `server.rs:983-1004`).
    - `Reuse` is defined twice (`client.rs:168` / `server.rs:210`), and so are `Response::header` and `Call::header` (`client.rs:192-199` / `server.rs:239-246`).
- **Fix:**
  - Move `field` (returning a local `Bad{Malformed, TooMany}` that each side maps), `decimal`, `persistent`, `content_length`, the coding list, `Parsed` and `header::find` into `header.rs`.
  - Put `Measure` in `skein-lib` beside `Writer` (house rule: no hand-rolled structures).
  - Move the Face demand transition into `body::demanded(face, read, max)`.
  - Make `Reuse` crate-level.
- **Size:** about 250 lines out of about 2,900.

**3. The written body face has no shared primitive, and the server's `Reply` triplicates `Room`** (Primitives, Elegance)
- **Evidence:**
  - `server.rs:607-623` is `Length{left, room} | Chunked{room} | UntilEnd{room}`. Every Room cell is therefore written three times: `814-836`, `1056-1070`, `1074-1100`, `1262-1273`, plus the wide or-patterns at `1063`, `1095`, `1144`, `1218`, `1325`.
  - The client's `Upload::Open{left, room}` (`client.rs:740-781`) makes the same Demand/Send/Finish assertions over an identical `Room` enum (`client.rs:448` / `server.rs:627`).
- **Fix:**
  - Use `Reply::Open { framing: Out, room: Room }` (with `Out` = Length(left) / Chunked / UntilEnd) next to Last/Finished/Withdrawn.
  - Add `body::Outgoing` as the mirror of `Incoming`, used by both sides.
- **Size:** about 80 lines, and each Room transition is written once.

**4. Three machines diverge from the shape without reason** (Taste, Naming, Complexity)
- **Evidence:**
  - **SSE reader:**
    - `State` is a Copy enum mutated in place (`sse.rs:298,302,351,363`), with no `mem::replace`.
    - The fields `piece`, `at` and `body_wait` (`sse.rs:174-182`) mean something only in some states, against 5.4.
    - `impl Clone for Event` (`sse.rs:117-133`) is a hand-written trait impl in step code. It exists for one test line, `vec![sse::Event::Ended; n]` (`tests/protocol/src/world.rs:358`).
  - **JSON collector:** a data-less `State` and a mutable `Position` with no per-cell handlers (`collector.rs:189-205`).
  - **TLS native face:**
    - The fields `lower_read`, `withdrawn_read`, `upper_output`, `lower_output` and `next_lower` sit outside `state` (`native.rs:175-184`).
    - `fn open(&mut State) -> Option<&mut Open>` (`native.rs:147-157`) turns an own-enum match into an Option, so 13 sites use `let…else` or `if let`. That gets around the no-`if let`-on-own-enum rule.
  - **Names differ:**
    - Follow-state is `settle` in the client/server/TLS, `demand` in the SSE reader, tokenizer and collector, and missing in the SSE writer. The tokenizer's own `settle` (`tokenizer.rs:668`) means something else.
    - The same edge bound is called `largest_read`, `largest_demand` and `LARGEST_READ`.
    - `valid()` exists for HTTP and TLS. The SSE reader and writer and the tokenizer only assert in `new()`, and `frame_bound`/`tokenizer_bound` return `None` to mean the limits are invalid.
  - These three files also use a terser, vaguer doc voice ("Builds the reusable fields and data intake under stable nonzero chunk limits", `sse.rs:203`).
- **Fix:**
  - Move the SSE reader's and the native face's per-state data into their enums and use the transition idiom.
  - Delete `impl Clone` and build the test vector with `repeat_with`.
  - Name follow-state one way everywhere, add `valid()` to every machine, and use `largest_read` throughout.

**5. JSON's API makes callers big: two of them rebuild the same layer** (Primitives, Economy)
- **Evidence:**
  - `skein-oauth/src/json.rs` (397 lines) and `skein-llm/src/openai/json.rs` (532 lines) each define a `Json` type with `from_bytes`, `span`, `field`, `required`, `optional_at`, `text_ref`, `unsigned`, `value_at` and `validate(…, depth)`.
  - Their `from_bytes` slice drivers (`oauth/json.rs:23-79`, `llm/openai/json.rs:36-89`) are the same loop as `tokenizer::validate` (`tokenizer.rs:127-178`), down to `ticks = len*4+16`.
  - Forge built a 1,250-line walker over tokenizer events (`github/decode/walk.rs`) instead of using the collector (reason unknown).
  - The writer is fine. The read side stops at `Document::token(i)`.
- **Fix:**
  - Add one `skein_json::parse(input, &collector::Limits) -> Result<Document, Error>`, which `validate` can use too.
  - Add value navigation on `Document`: `span(at)`, `field(at, name)`, `unsigned(at)`, `text(at)`.
- **Size:** about 400 lines leave the callers, and about 40 leave `tokenizer.rs`.

**6. The collector is over-complex** (Complexity, Elegance)
- **Evidence:**
  - `active_tag` linearly scans the tag list ten times; `tags.get_mut(i).expect("tag")` appears 22 times; `has_parent` scans the list again (`collector.rs:826-833`).
  - The "defer to parent, else fail" pattern is retyped at `680-683`, `807-814` and `1013-1019`; the `!has_parent && let Some(error)` pattern at `854-857`, `935-939` and `969-972`.
  - `Frame.kind: Kind` (11 variants) forces nine-arm `unreachable!` blocks (`746-758`, `1048-1060`), although a two-variant `Container` exists (`tokenizer.rs:545`; it is also defined a second time at `writer.rs:105`).
  - `up()` passes `below` straight to the tokenizer, then pops the tokenizer's demand back out of it and asserts the withdrawal (`336-363`). That un-sends an output.
- **Fix:**
  - Hold the active tags in a `Stack<u32>`: the top is the active one, and a length above 1 means it has a parent.
  - Add one `defer_or_fail(collector, error) -> Result<(), Error>`.
  - Make `Frame.kind` a `Container`.
  - Route the tokenizer's demands through an internal queue (as is already done for events) and forward them only after the skip check.
- **Size:** about 120 of 1,109 lines.

**7. Tests break section 10 in setup and data** (Economy)
- **`tests/tls/tests/suite/native_output.rs:17-109`:**
  - Seven tests open with the same `for version in [Tls12, Tls13]` and `Config { buffer: 7, ..Config::calm() }` (seven times).
  - Each asserts the same six or seven fields.
  - Seeds `802` and `805` are each reused in a copy-pasted test pair.
  - Fix: a `conversation_both(case)` helper plus an `Outcome` expectation struct.
- **`tests/http/tests/suite/client.rs:73-79,112-118`:** the same Host-carrying `Call` literal is typed twice, and the sizes 400/200 are each typed three times, unnamed.
- **`tests/protocol/tests/suite/worlds.rs`:** eight tests share a four-line opening with bare seed bases (100 to 700). `:62` asserts a precondition.
- **`tests/tls/tests/suite/fake_peer.rs:59-62,346-349`:**
  - The bare size 6359 is typed three times and the setup asserts `body.len() == 6359`.
  - `Time::from_nanos(3_000_000_000)` is written twice.
  - It is an LLM-upload test sitting in the TLS suite.

**8. The in-crate step harness is copied six times** (Economy)
- **Evidence:** `Machine{m, env, above, below}` with `down`/`up`/`take` appears at `skein-http/src/tests.rs:52-89`, `tests/server.rs:32-69`, `tests/writer.rs:22-50`, `tests/sse.rs:374-410`, `skein-tls/src/tests.rs:85-130` and `skein-json/src/tests.rs:48-`.
- **Fix:** one generic `Stepper` in a tiny test-support crate used as a dev-dependency.
- **Size:** about 150 lines.

**9. Answer to the brief: the tls process loop is not a reinvention, but another tls file is** (Organization)
- `fake_peer.rs` uses `skein_world::World`; its `work_pending`, `next_policy_deadline` and `overdue` are implementations of the skein-world `Host` and `Referee` traits. That is fine, apart from about 40 lines of `Process`→`dyn Host` delegation (lines 220-270), a pattern also seen in `tests/world` and `tests/echo`.
- The real reinvention is `tests/tls/src/native.rs:686-728` `conversation`. It is a hand-rolled `Sim` loop (reap, iterate, submit, advance_to, quiescence) that `World` already provides.
- Its `iterate` (`640-684`) re-implements the io resume/up/fire/down hosting dance that `fake_peer.rs:97-183` also writes, in a different style.
- **Fix:** have `Driver` implement `Host` and run it in `World`.
- **Size:** about 80 lines.

**10. Small stream primitives are retyped across machines** (Primitives, Naming)
- **The "stream below while idle" memory** exists five times under three names: `Line` (HTTP ×2, TLS), `Below` (SSE ×2) and `Held` (tokenizer). The semantics have drifted: TLS lets a failure after End overwrite it (`skein-tls/src/client.rs:384-386`), while HTTP keeps End (`client.rs:507-511`) (unsure whether this is deliberate).
  - Fix: `lib::stream::Line` with `Line::after(Up)`.
- **The withdrawal literal** `Down::Demand { read: Read::Nothing, room: 0 }` is written 21 times in this area.
  - Fix: a `stream::WITHDRAW` const.

**11. Noise and small rule breaches** (Cleanliness)
- A stacked "Bounds X." line sits on top of each `*_bound` doc: `tokenizer.rs:94,116`, `writer.rs:59`, `document.rs:194`, `collector.rs:287`, `sse/writer.rs:64`. Delete them.
- Each `Kind` variant's doc ends "…, for the decoder or writer." (ten times; `document.rs:35-55`).
- Test-crate docs name `tests/fuzzy_*.rs`, but the files live in `tests/suite/fuzzy/` (`tests/http/src/lib.rs:37`, `tests/tls/src/lib.rs:23`, `tests/protocol/src/lib.rs:25`, `suite/client.rs:3`).
- `server/response.rs:121` matches on a tuple scrutinee, which 10.3 forbids.
- `longest_answer` (`server/response.rs:346-358`) lists the Rejection variants by hand, so a new variant would silently leave the room set aside wrong.
- `Encoder::json_text` (`writer.rs:236-255`) runs `validate` twice on each pass, four times in all.
  - Fix: run it once with the depth capped to the writer's remaining room.

**12. gzip does not belong in skein-http** (Organization, low priority)
- `gzip.rs` is a 1,132-line pure codec inside a protocol-machine crate, and a matching inflate lives in `skein-fake-llm-protocol`.
- **Fix:** move both into a `skein-gzip` crate.

## 5. Cross-area suspicions (not verified outside this area)

- The JSON slice-driver loop and the `Json` navigation layer are likely repeated in other `skein-llm` providers (an `anthropic/` module exists) and in `skein-forge`'s Forgejo tables.
- The `Process` enum with hand-written `dyn Host` delegation recurs in `tests/world/tests/suite/{writer_pipe,real_conformance,end_to_end}.rs` and `tests/echo/tests/end_to_end.rs`. skein-world could supply it.
- The io hosting dance (resume/up/fire/down, bounded) probably recurs in every `Host` implementation (fake peers, oauth-accounts); `skein-io` or `skein-shell/src/drive.rs` may already hold a version.
- `Measure`, `fault`/`draw` and `delivers` probably exist in other protocol crates and worlds (`tests/io/src/owner.rs:596` has its own `draw`).

## 6. Rule costs (low priority)

- **No closures or fn pointers:**
  - JSON's two-pass encoding makes every caller write measure, map the refusal, then write: about 10 lines at each of 76 sites, roughly 700 lines in all.
  - The comma-list walk over `header::element` is retyped five times (`header.rs:84`, `client/head.rs:220,259`, `server/head.rs:328,362`), because an iterator would need a trait impl.
  - A `lib` helper taking an enum-coded encoder would claw some of the encoding cost back.
- **No `if let` on own enums plus exhaustive matches:** the wide or-patterns over `Reply` and `Token` are forced. Finding 3 removes most of them by reshaping the type rather than the rule.