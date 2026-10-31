I read across the eight paths, sampling the large files. **Overall:** lib and `stream.rs` are high quality. The file layer, `pipe.rs` and the newer tests/io worlds don't follow the house idioms, and most of the duplication is in them.

## 1. Scope read

**Read closely:**
- **lib:** `lib.rs`, `id`, `slab`, `queue`, `list`, `stack`, `map`, `set`, `deadlines`, `buffer`, `intake`, `reader`, `writer`, `stream.rs` with `held` and `output`, `btree`.
- **io:** `lib.rs`, `layer.rs`, most of `file_layer.rs`, `file.rs`, `output.rs`. Parts of `stream.rs` (types, dispatch, demand, the wrappers), `pipe.rs`, `process.rs`, `joined.rs`, `joined/end.rs` and `records.rs`.
- **io step tests:** `tests.rs`, plus the file, store and private rigs.
- **tests/io:** `lib.rs`, `world.rs`, `output.rs`, `renames.rs`, `files.rs`, every Host and Referee impl, and the suite files `files`, `cuts`, `directory`, `remove`, plus the first part of `memory`, `process_memory` and `scenarios`.
- **tests/lib:** `memory.rs`, `search.rs`, `intake.rs`.
- **Others:** fail-stop, scratch.

**Skimmed:** `kernel.rs` (outline), the journal and request-table code and tests, `skein-heap` and its tests, tests/io `owner`, `ledger`, `referee`, `census`.

**Not read:** `sha256`, `base64`, `crypto_rng`, `listener.rs`, `signals.rs`, `limits.rs`, most `stream.rs` handlers, the fuzzy suites.

## 2. Scores

| Criterion | Score | Why |
|---|---|---|
| Cleanliness | 3 | lib and `stream.rs` are clean. There are stale doc paths, accreted mod lists, citation noise and bare numbers in file code. |
| Organization | 3 | lib is well cut. io has two parallel layers with asymmetric module names. tests/io runs two world frameworks side by side. |
| Taste | 3 | `stream.rs` is exemplary. `pipe.rs`, `file_layer.rs` and `renames.rs` break the house idioms. |
| Naming | 3 | Mostly precise. Mixed: `root: Fd` vs `Token`, `up` vs `events`, `fire` vs `expire`, two things called `Referee` and two called `Rig`. |
| Primitives | 3 | Containers are orthogonal. The growth rule is copied five times. Read-meeting semantics have no lib primitive, so they are recomputed in 6+ places. |
| Elegance | 3 | Stream's `State` enum follows the problem. Pipe and FileIo are bags of flags and Options. |
| Economy | 2 | Many near-copies (detail in section 4). |
| Complexity | 3 | Mostly just enough. One layer in `stream.rs` only forwards, and FileIo has flat Option fields. Journal and RequestTable are not speculative: temper and jig use them. |

## 3. Strengths

- **lib containers share one small shape:** `with_capacity`, `price`, `len`, `room`, and a `push` that returns `Result`. Each has a documented `Hash` policy. The step tests (`tests/containers.rs`) are short and literal.
- **`stream.rs` is the model of 5.4:** `State` enum, the `mem::replace` transition, one handler per cell, and `follow`/`tidy` applied once after each transition.
- **The tests/io socket world is a good harness:** a plan-driven owner, a ledger that checks io's contract, a census over the trace, and scenarios that read as stories (`plan`, `expecting`, `serve`, `dial`).
- **Small shared primitives are done well:** io's `output::Reservation`, the `Rig` in `skein-io/src/tests.rs`, and `skein-heap`'s documentation.

## 4. Findings (ranked by payoff)

**1. `file_layer` uses its own idiom instead of the house state machine.** (Economy, Taste, Elegance)
- `Pending` has 29 flat variants, each carrying `owner`. They are listed out about 8 times: 204 `| Pending::` lines, including `owner()` at 155-186 and `can_cancel`, `cancel`, `cancelled`, `stopped`.
- It matches on tuples with wildcards, which 5.4 forbids:
  - `match (pending, complete.result) { … _ => None }` at `file_layer.rs:876-894`.
  - `match (pending, complete.kind, result) { … (_, _, _) => unreachable!` at 1054/1260.
- `events.push(Event::Failed { owner, error, committed: false, residue: None })` is written out 32 times (23 in `file_layer.rs`, the rest in `opens.rs`, `private.rs` and `store.rs`), although `terminal_failure` (904) already exists.
- **Fix:**
  - Make it `struct Pending { owner: Token, step: Step }`.
  - Group the Load\*, Scan\* and List\* variants into sub-machines, as `Store(store::Store)` and `Private(private::Opening)` already are.
  - Match on the step, then on the result inside each arm.
  - Add `Event::failed(owner, error)`, or make `committed`/`residue` a store-only variant.
- **Size:** about 250–350 of 1,262 lines.

**2. Pipe is a bag of flags, and the "face to the side above" is copied three times.** (Elegance, Economy)
- `pipe.rs:15-44` has 22 fields and 6 bools under `#[expect(clippy::struct_excessive_bools)]`.
- `demand`, `granted`, `classic_grant` and `independent` appear in `stream.rs:75-90`, `joined/end.rs:15-20` and `pipe.rs:27-31`.
- Demand validation is written out three times: `stream.rs:1018-1057`, `pipe.rs:136-156`, `joined/end.rs:68-84`.
- **Fix:** add a `Face` type beside `Reservation` with `fn demand(read, room, limits)`, and give Pipe a `State` enum as Stream has.
- **Size:** about 60 lines of copies, and roughly 100 of `pipe.rs`'s 633.

**3. lib is missing the read-semantics primitive.** (Primitives)

"What a `Read` takes from bytes already held", "is this delivery a valid answer" and "does this read fit a cap" are each computed separately in:
- `held.rs:34-76` and the `Intake::meet` asserts (`intake.rs:137-150`);
- two test references, `skein-lib/src/tests/streams.rs:239-266` and `tests/lib/src/intake.rs:16-50`;
- io's three validations (finding 2);
- `tests/io/src/ledger.rs:234 met`.

Outside the area the same logic is in `skein-http/src/body.rs:282 answers` and `skein-json/src/tokenizer.rs:154-168`, which hand-rolls `Held`.

**Fix:** add `Read::fits(cap)`, `Read::takes(&[u8]) -> Option<usize>` (used by `Held`) and `Read::answered_by(&[u8])`. Keep Intake's incremental scan as the independent implementation the tests compare against `takes`. **Size:** about 150 lines across the tree.

**4. tests/io has two world frameworks and five hand-written io stages.** (Organization, Economy)
- The crate's own `world::World`/`Proc` and its `referee::Referee` struct sit beside `skein_world::World` and its `Referee` trait (whose impls are called `Judge`).
- The resume → up → fire → down stage is retyped in `world.rs:72-112`, `output.rs:283-311` (plus its own run loop, 336-366), `processes.rs:213-245`, `usage.rs:147-168` and `append.rs:148-160`.
- Only three of those five check `MAX_OUT`, which testing-strategy section 6 requires at every call.
- **Fix:** add one `io_stage` helper with the room check; `examples/echo/service/src/io_stage.rs` is a ready model and could become a skein-io export. Move `output.rs` onto one of the two worlds.
- **Size:** about 150 lines.

**5. Answer to your question on `work_pending` / `next_policy_deadline` / `overdue`.** (Economy, Complexity)
- **`next_policy_deadline`** returns `None` in 9 of 9 hosts. This is the honest form: `drive.rs:36-38` says "Every host states it explicitly". Keep it.
- **`overdue`** returns `None` in 9 of 9. That follows from referees with no deadline expectations. It is honest; the boilerplate belongs to `skein-world`'s trait (outside the area).
- **`work_pending`** is the real smell. It is a hand-kept copy of `iterate`'s issuing condition. For example, `files.rs:176-179` has `matches!(phase, Store|Load|Close)`, which must stay in sync with the arms at 123-162. It could be derived.
- The six file owners (cuts, files, locks, pages, private, renames) retype the same Host skeleton: 55–93 lines each, plus trivial FileSystem/Judge wrappers. Their event queue is called `up` in three and `events` in the other three.
- **Fix:** one `FileOwner<S: Script>` in tests/io, where `Script` has `next()`, `heard(event)` and `settled()`. Write the Host impl once, with `work_pending = !completions.is_empty() || (io.takes() && script.ready())`.
- **Size:** about 300 of roughly 2,800 lines.

**6. Section-10 breaches in tests/io.** (Cleanliness, Naming)
- **Bare-integer phases:** `renames.rs` uses a `u64` phase from 1 to 11 (36, 139, 152-183), with tokens equal to the phase numbers and an `other => panic!` arm. Every sibling module uses a `Phase` enum.
- **Rows that branch on their label:**
  - `suite/files.rs:94-114` and `117-137` (`match story` over stories with different outcomes);
  - `167-198` (`if matches!(story, Create|Replace)` across 13 stories × 2 configs × 3 seeds);
  - `suite/cuts.rs:44` (`if cut == Cut::Kill {[2,1]} else {[2,2]}`).
- **Bare numbers:**
  - `FileIo::new(3, 16, 4, Duration::from_secs(1))` at 8+ sites;
  - `self.max == 3` as a sentinel meaning "use OLD/NEW" (`files.rs:135`);
  - `0..17`, `step >= 15`, `Error::Other(5)` in `suite/files.rs:10-14`;
  - the seed list `[0, 1, 17]` nine times.
- **Retyped closing lines:** `names() == vec![b"record"]` followed by `finish()` ends about 10 tests.
- **Production root cause:** `FileIo::new` and `with_whole_limit` take 4–5 positional same-typed `u32`s. Add a `FileLimits` struct; tests/kv and `skein-world/files.rs:37` would benefit too.

**7. Test helpers copied across crates.** (Economy)
- `tests/lib/src/search.rs:6-43` is identical to `skein-lib/src/tests/bytes.rs:80-112`.
- io has three file rigs (`tests/file.rs:17-77`, `tests/store.rs:18-106`, `tests/private.rs:22-133`), each with its own `complete` and buffer-filling `read`. Two of them are named `Rig`, the same as the Io `Rig` in `tests.rs`.
- The metering `Driver`s in `suite/memory.rs` and `suite/process_memory.rs` share a skeleton.
- `tests/lib` `memory.rs` writes `metered.start(); …; metered.end(x);` 71 times; a `step(|| …)` helper would do. Its four "grows to its capacity exactly" tests (848-976) are near-copies.
- **Size:** about 250 lines.

**8. lib's growth rule is written five times.** (Economy, Primitives)
- `slab.rs:179`, `queue.rs:84`, `list.rs:73`, `stack.rs:61`, `intake.rs:111`, each with its own `GROWTH_FLOOR = 4`.
- `fn index(u32) -> usize` is defined four times, and the `usize::try_from(…).expect(…)` conversion appears inline 15 more times.
- **Fix:** one crate-private growth function and one `index`. One memory test of growth then covers all five containers.

**9. Gaps in the container API that callers hand-roll.** (Primitives)
- `Queue` has no `clear`. `skein-oauth-accounts/src/listener.rs:414-435` and `component.rs:899-977` write `for _ in 0..q.len() { drop(q.pop()) }` (6 loops).
- `Stack` has no `clear` (`skein-json/src/tokenizer.rs:334`), no `room` and no `iter`.
- `Map` has no `pop_first`, though `Set` does.
- `Writer` counts in `usize` (`writer.rs:23-41`) while every other lib count is `u32`; 26 call sites convert.

**10. The file vocabulary leaks kernel types.** (Primitives, Naming)
- Ten `file::Request` variants take `root: Fd` (`file.rs:50-147`). io.md section 5 says "never by its descriptor". As a result, callers turn tokens back into descriptors (`renames.rs:69`, `pages.rs:114`).
- `Failed` carries the kernel's `Error` (`file.rs:289`), while sockets get io's own `Error`.
- `io.status.md` already tracks this ("remaining roots by token").
- Smaller inconsistencies: `*NoFollow` as separate variants vs a `no_follow` field; `fire` (Io) vs `expire` (FileIo).

**11. io dispatch breaks its own let-else rule.** (Taste)
- `process.rs` has 13 `let Some(Entity::Child(c)) = … else { return }` sites.
- `layer.rs` has `if let Some(Entity::…)` at 410, 539, 665, 732, three bare `unreachable!()` at 514/520/525, and `_ => None` at 768.
- `joined.rs:75-96` already shows the right form: typed accessors with one exhaustive match each. Extend that to Child, Stream and Pipe.
- The four `adopt_*` methods (`layer.rs:196-245`) have the same body.
- **Size:** about 60 lines.

**12. A layer in `stream.rs` that only forwards.** (Complexity) At 1591-1744, `landed`, `resume`, `answer`, `request`, `expired` and `retried` exist only to emit `owed` before calling the matching `state_*` function. Fold `owed` into the state, or emit it once at the dispatch. **Size:** about 100 lines.

**13. Small cleanliness items.**
- **Stale paths:** `tests/io/src/lib.rs:15-19`, `scenarios.rs:7-9` and `suite/scenarios.rs:3` point to `tests/*.rs` and `fuzzy_scenarios.rs`; `tests/lib/src/lib.rs:6` names `tests/memory.rs`.
- **Accreted mod lists:** `tests/io/src/lib.rs:34-40` and `skein-io/src/tests.rs:328`.
- **Citation noise:** `stream/output.rs` has 21 `(lib.md, 7.1)` citations in 68 lines, in telegraphic prose. `tests/io/src/output.rs:46-56` cites `§` on every field.
- `suite/directory.rs` holds the renames and pages tests, with a `use` partway down.
- **Bare 4096 (path limit) and 32/8192 (list buffers):** at `file_layer.rs:379-380, 454, 470`, `store.rs:178`, `private.rs:127`, `directory.rs:12`. `kernel` has `LONGEST_NAME` but no path constant.
- `opens::open_read` also does `OpenWrite`.
- `pipe::fault` (`pipe.rs:457`) is a second error-to-`Fault` map beside `records::stream_fault` (`records.rs:264`), and the two disagree on `TimedOut` and `Unreachable`.

## 5. Cross-area suspicions (unverified)

- **The io stage loop is hand-rolled** in the echo example (`examples/echo/service/src/io_stage.rs`), `skein-ui-text/src/terminal.rs:254`, `tests/tls/src/native.rs:643`, `tests/forge/conformance/src/component/hosted.rs:254`, `tests/forge/setup/src/closing.rs:67` and `tests/world/src/bin/terminal_fixture.rs:81`. It is a good candidate for a skein-io export.
- **Read semantics are recomputed** in `skein-http`, `skein-json`, `skein-oauth/json.rs`, `skein-llm/openai/json.rs` and `skein-forge/.../delivery.rs`.
- **Host/Referee boilerplate** (`next_policy_deadline`/`overdue` returning `None`) recurs in `tests/world`, `tests/tls/.../fake_peer.rs` and `tests/oauth-accounts`. The trait owners (`skein-shell/src/drive.rs`, `skein-world/src/referee.rs`) may want a watch-only referee.
- **`FileIo::new` positional numbers** recur in `tests/kv` (`world.rs:46,124,135`; `real.rs:49,117`).

## 6. Rule costs

The no-`_` rule makes io spell out `kernel::Error`'s 26 variants about 13 times outside `kernel.rs`, and `Done`'s variants about 20 times. For example, `layer.rs:445-472` lists all 26 errors just to say that Usage fails only with `Other`. That is roughly 300–500 lines.

Some of it could be reclaimed within the rules (unsure how much): category predicates written once in `kernel.rs` (for example "a file error" or "a socket error"), since many of the arms repeat the same grouping (`records.rs:277-289`).