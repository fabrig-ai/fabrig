# skein code-quality review, 2026-10-31 (the first): area reports and rubric

Companion to `2026-10-31-skein-first.md`. The shared rubric (`review-brief.md`) and the 13 reviewers' reports (`reviews/01..13-*.md`), joined, unedited apart from this header, the dividers and each report's title line. Snapshot: github `origin/main` at `337102750`. Paths are relative to `skein/` in the review's clone.

---

# Code-quality review of skein: shared brief

Tree: `<clone>/skein`
(a fresh clone of github origin/main at 337102750). Read only; do not build or run tests.
Work from this clone only; never touch another checkout.

## What is being asked

Not a bug hunt. The user wants an assessment of **code quality** of production code
**and tests (tests are code and are judged the same way)** on these criteria:

1. **Cleanliness**: readable, no dead code, no noise, comments that earn their place.
2. **Organization**: module/file layout, where things live, file sizes, cohesion.
3. **Taste**: idiom, judgement, does it look like it was written by someone who cares.
4. **Naming**: precise, consistent across the area, matches the design's vocabulary.
5. **Primitives**: orthogonality and decoupling. Are the abstractions the right ones,
   each doing one thing, composable, with narrow interfaces? Leaky or overlapping concepts?
6. **Elegance**: does the shape of the code follow the shape of the problem?
7. **Economy**: could the code be smaller? Duplication, near-copies, boilerplate,
   over-long tests, fixtures retyped, parallel hierarchies. Estimate the lines that
   could go (e.g. "~600 of 2,100 lines in tests/x.rs").
8. **Complexity**: is it just complex enough for what it needs, and no more?
   Speculative generality, layers that only forward, state that could be derived.

## Calibrate to the house rules (important)

skein deliberately writes a small Rust: read `docs/foundation/programming-model.md`
sections 1, 3, 4, 5.4 and 10 (no std in step crates, no closures/traits/generics/dyn,
exhaustive matches with no `_`, no indexing, no `as`, checked arithmetic, one handler
per state×event cell, the `mem::replace` transition idiom, free functions over fields,
`Writer`/`Reader`, slabs and `Id<T>`) and `docs/foundation/testing-strategy.md`
section 10 (a test's shape: one behaviour per test, rows carry data, the setup the
assertion reads, a setup written once, named fixtures and sizes, a bound's two sides).

- Do **not** flag verbosity that the rules force (e.g. a long exhaustive match) as
  uneconomical. Judge economy *within* the rules: given these rules, is this the
  smallest clear form?
- Do flag where code breaks its own rules in ways that hurt quality (e.g. tests that
  retype setups, bare numbers, branching on row labels).
- If a rule itself costs a lot of code in your area, note it once, separately, with a
  concrete example and rough cost. Low priority.
- Comments in this repo use a particular plain prose style; don't flag the style itself.
- Test crates (`tests/*`, `testing/*`, `src/tests*`) are ordinary Rust (may unwrap,
  index, use closures) — judge them as ordinary Rust held to section 10.

## How to work

- You cannot read everything. Map the area first (module trees, file sizes, public
  surface, how tests are organized), then read closely where quality matters most:
  the core types and state machines, the biggest files, the test support/fixtures,
  and a sample of tests. Say what you read closely and what you only skimmed.
- Prefer **patterns** (a thing done 20 times) over one-off nits. A finding is worth
  reporting if fixing it would make the code noticeably smaller, simpler or clearer.
- Verify every finding against the source; quote file:line. No speculation reported
  as fact; mark uncertain items "(unsure)".
- Look across crate boundaries inside your area: duplicated helpers, overlapping
  types, a test harness reinvented per test crate.

## Report (your final message; ≤ ~1,800 words)

1. **Scope read**: closely / skimmed.
2. **Scores**: 1–5 for each of the 8 criteria, one line of justification each.
3. **Strengths**: 2–4 bullets (what to keep doing; be specific).
4. **Findings**, ranked by payoff, numbered. Each: title; criterion; evidence (file:line,
   several sites for a pattern); why it matters; concrete fix; estimated size effect.
5. **Cross-area suspicions**: things you noticed that likely recur outside your area
   or involve crates outside it (name them), unverified beyond your area.
6. **Rule costs** (optional, short).

---

# 01: base-lib-io

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
---

# 02: runtime-shell-sim-kv

## skein review: sim, shell, kv, channel and their tests

The weakest code in this area is in two step crates. `skein-kv` and `skein-channel` break the step-code rules they claim to follow, in ways that cost clarity. The second big cost is duplication: the POSIX code in shell's two backends, the test harnesses, and the types that join sim, fake-machine and sim-world.

### 1. Scope read

**Read closely:**
- **shell:** `lib.rs`, `kernel.rs`, and the facade/backend pairs, diffed side by side: `terminal`, `group`, the `process` helpers, private-file reads, `make_root`, `linux.rs` against `darwin.rs`.
- **sim:** `lib.rs`, the `sim.rs` types and `check`, `fds_full` and `assert_no_open_fds`; the `processes.rs` types.
- **fake-machine:** `lib`, `face`, and the types in `fs`.
- **sim-world:** `lib`, `program`, `meter`, and in `world.rs` the struct, builders, `turn` and `drop_killed`.
- **kv:** `store.rs` (`up`, `fire`, `snapshot_up`), `frame`, `key`.
- **channel:** `machine.rs` (state, close paths, read path).
- **conformance:** `lib.rs`, samples of `scenarios.rs` and `files.rs`, both backends, the case lists.
- **test harnesses:** `tests/sim/src/support.rs`; `tests/kv` world and store; the `tests/ring` harness against `tests/readiness`; the channel test harnesses.

**Skimmed:** `linux/ring.rs`, `readiness.rs`, shell settings and drive, sim trace and config, fake-machine `fs` internals, `skein-fake-channel`, the real-kernel suite files, and the `tests/sim` suites other than `files.rs`.

### 2. Scores

| Criterion | Score | Why |
|---|---|---|
| Cleanliness | 4 | Comments earn their place; little dead code. Noise comes from kv's module-wide `#![expect]`s and sim-world's 20 `.expect("a status per process")` calls. |
| Organization | 3 | The linux/darwin trees don't match file for file. Large files mix concerns (`processes.rs` 1,520 lines with 430 lines of tests inside it; kv `store.rs` 1,092). |
| Taste | 3 | sim and conformance are careful. kv and channel lapse into flag soup and copied state-machine cells. |
| Naming | 3 | `Machine` is both a trait and a struct; `Handle`/`Opened`/an internal `Handle` name one thing; `fds` means sockets only; `sync.rs` holds every synchronous call. |
| Primitives | 3 | The conformance `Backend` trait is exactly right. Elsewhere: six descriptor tables, a duplicated `Item`, two handle types. |
| Elegance | 3 | kv's `up`/`fire` and channel's close paths don't follow the shape of their state machines; `World::turn` does five jobs. |
| Economy | 3 | Roughly 1,300 to 1,500 lines (the sum of the estimates below) could go, without touching rule-forced verbosity. |
| Complexity | 3 | Mostly in proportion. State spread across bools and options adds complexity the enums would remove. |

### 3. Strengths

- **The conformance suite is written once.** Each scenario is generic over `Backend`, returns a plain "seen" struct, and is judged by a `Check`. The sim and real crates only implement `Backend`. The Darwin differences are factored into `check_common` plus `check_darwin`, with XNU line citations (`scenarios.rs:176-205`).
- **`tests/sim/src/support.rs` is a model harness.** It is small and well named (`call`, `listener`, `pair`, `settled`), with `ROOM` named.
- **`tests/kv/tests/suite/golden.rs:16-29` carries a derivation beside every limit**, which is what section 10 asks for.
- **The shell already has a tool for sharing POSIX code.** `listener_calls.rs` is compiled into both unsafe boundaries via `#[path]` (`linux.rs:29`, `darwin.rs:20`), and SAFETY comments are kept per module.

### 4. Findings, ranked by payoff

**1. kv's store is not the 5.4 step machine its design describes** (elegance, primitives, complexity)
- **Evidence:**
  - kv.md section 1 says "ordinary `no_std` step machine". But `store.rs:3-10` opts out of `disallowed_types`/`methods`, `struct_excessive_bools`, `too_many_lines` and `too_many_arguments` for the whole module. There are 10 `loop`/`while` sites (`store.rs:259,301,402,436`, `key.rs:89`, `frame.rs:92`, `snapshot.rs:79`, …).
  - `up` matches on a tuple with a catch-all (`store.rs:637`, `:844 (phase, event) =>`), and `snapshot_up` ends in `(_, _) => unreachable!` (`:1086`).
  - Snapshot progress lives in five places: `Phase::SnapIo{action}`, two parallel enums `SnapAction`/`SnapStage` (`:59-86`), `Snapshot{file: Option<Token>, stage}`, `snapshot_start`, and `has_snapshot`/`has_temp`. As a result `snap.file.expect("snapshot file opened")` appears 6 times.
  - `Store` carries 6 bools (`:109-130`).
  - Copied cells: the read loops for `ReadingSnapshot` and `ReadingLog` (`:675-693` and `:711-733`); "start the next batch or close" (`:758-764` and `:784-790`); trailer encoding (`:942-948` and `:962-966`).
  - `b"snapshot.tmp"` is written as a literal 5 times. The bare sizes `12`/`8_usize` in chunk sizing (`:921-923`) restate `snapshot.rs`'s layout.
- **Why it matters:** this is a durable store. Its recovery logic is the hardest code in the area to review, and the subset's guarantees are switched off exactly there.
- **Fix:**
  - One enum per machine where each variant holds its own data, e.g. `Phase::Snapshot { log, offset, step: SnapStep }` with `SnapStep::Writing { file, at, cursor, rows, payload }` and so on. Drop `SnapStage`: the in-flight request *is* the stage.
  - One shared `Reading { file, len, at, bytes, then }` sub-state for log and snapshot reads.
  - One handler per cell, using the `mem::replace` idiom.
  - Constants for the file names, and `snapshot::ROW_OVERHEAD`.
- **Size:** about −200 to −250 of 1,092 lines, and most of the `#![expect]`s gone.

**2. kv re-implements lib's `Reader` and `Writer`** (economy, primitives)
- **Evidence:** `frame.rs:122-135` defines `u16_at`/`u32_at`/`u64_at`. `KeyReader::u64` (`key.rs:73-78`) duplicates `skein_lib::Reader::u64`. `frame::encode` works out the exact length first (`encoded_len`, `:20-29`) but then builds two `Vec`s with `extend_from_slice` (`:36-62`); `snapshot.rs:20-60` does the same. Channel's `frame.rs` uses lib's `Writer` and `Reader`.
- **Fix:** use `Writer::new(total)` and `put` for the encoders and `Reader` for the decoders. `KeyWriter` keeps only its escaping.
- **Size:** about −60 to −80 lines, and the `Vec` escape hatches in `frame.rs`, `snapshot.rs` and `key.rs` go.

**3. shell: POSIX code copied between `linux/` and `darwin/`** (economy, organization)
- **Evidence:**
  - `linux/terminal.rs` and `darwin/terminal.rs` are byte-identical apart from line 1 (90 lines).
  - In `group.rs`, `negative_group`, `signal_group`, `group_live`, `process_live` and the four tests are identical; only `child_id` and `group_has_other_members` differ (46 diff lines out of about 130 per side).
  - `read_private_file`, `check_private`, `read_file`, `read_opened` and `kernel_error` (`linux/startup.rs:20-159` against `darwin/files.rs:255-395`) differ only in the no-follow `open` and two casts.
  - `make_root` (`linux/roots.rs:40-69`, `darwin/files.rs:418-456`) differs only in `fsync` versus `F_FULLFSYNC`.
  - In `process.rs`, `cstring`, `action`, `block_sigpipe`, `usage`, `cpu` and `resources` are identical.
  - The same function also sits in differently named files per OS (`lock`: `linux/sync.rs` against `darwin/files.rs`; `last_errno`: `linux/sync.rs` against `darwin.rs`), so the trees can't be compared file by file.
- **Answer to your question:** sockets (`sin_len`, `SOCK_NONBLOCK` against `fcntl`), spawning (`pidfd_spawn` against fork), readiness (epoll against kqueue) and listing really are platform-specific. Terminal, group signalling, private reads and roots are POSIX.
- **Fix:** put the POSIX parts in shared files (for example `posix_terminal.rs`, `posix_group.rs`, `posix_startup.rs`) included the way `listener_calls.rs` is. Keep small per-OS functions: `open_nofollow`, `full_sync`, `pty_name`, `group_has_other_members`, `child_id`. Give matching files matching names.
- **Size:** about −350 to −400 of about 5,300 lines.

**4. channel's `Machine` keeps its output and terminal state in flags** (primitives, elegance)
- **Evidence:**
  - 11 bools plus several options, under `#[expect(struct_excessive_bools)]` (`machine.rs:116-144`).
  - The output right's lifecycle is spread over `output_right`, `output_granted`, `cancel_requested` and `cancel_sent` (`:129-132`). Combinations such as "granted with no right" are representable.
  - `cancel_requested = self.output_right.is_some()` appears 5 times (`:334,426,631,645,839`).
  - The "go closed" sequence (Closed, `read=Idle`, `clear_output`, cancel, `terminal_why`) is written about 5 times (`:321-325,420-426,627-632,641-646,835-840`).
  - `terminal_why` only means something in `Phase::Closed`.
  - `ping_queued` and `unsupported_queued` shadow the `QueueTag` of items already in the queue.
  - Bare numbers: `bytes.len() != 7` (`:666`), `reason == 3` / `auto_refuse(3)` (`:903,909`), `256` (`:327`).
- **Fix:** `enum Output { Idle, Asked{right}, Granted{right}, Cancelling{right, sent} }`, `Phase::Closed { why }`, one `fn shut(&mut self, why)`, and named constants.
- **Size:** about −60 to −80 lines; the larger gain is that impossible states go away.

**5. sim: six descriptor tables per process** (primitives, naming)
- **Evidence:** `Process` (`sim.rs:62-92`) holds `fds` (sockets only), `files`, `pipe_fds`, `inherited_pipes` (with a `process_open` flag), `pidfds` and `signal_fds`. Each cross-cutting question walks all six by hand: `check` (`:631-646`), `fds_full` (`:1507-1521`), `assert_no_open_fds` (`:507-527`).
- **Fix:** `descriptors: BTreeMap<Fd, Descriptor>` with `enum Descriptor { Socket(SocketId), File(File), Pipe(PipeEnd), Inherited(..), Pidfd(u64), Signal(SignalSource) }`, keeping inherited pipe objects outside the descriptor table. `fds_full` then becomes a `len()`, and `check` a single lookup.
- **Size:** about −60 lines, and `check` reads much more simply.

**6. sim / fake-machine / sim-world: two names for one handle, and a name clash** (primitives, naming)
- **Evidence:**
  - `skein_sim::Handle` (`machine.rs:28`) and `skein_fake_machine::Opened` (`fs.rs:50`) are the same `u64` newtype with the same `new`/`raw`. fake-machine also has an internal `struct Handle` (`fs.rs:246`). I counted 86 `Handle::new(x.raw())` / `Opened::new(x.raw())` conversions across skein and smith (for example `tests/kv/src/world.rs:42`, `tests/conformance/sim/src/lib.rs:160`, `face.rs`).
  - sim-world's `trait Machine` (`program.rs:91`) collides with fake-machine's `struct Machine`. So test crates alias it (`Machine as Files`) and each writes a wrapper `impl Machine for FileSystem { fn step(..) { skein_fake_machine::step(&mut self.files, ..) } }`: at least 12 of them (`tests/io` ×8, `tests/world`, `oauth-accounts`).
- **Answer to your question:** the dependency direction is clean (sim → fake-machine, sim → sim-world). The overlap is in these joining types. Program names are also known in two places: fake-machine's `program_of` (`face.rs:77-106`) and sim-world's `HostedProgram` registry.
- **Fix:** fake-machine uses `skein_sim::Handle` as its public type, and the internal struct is renamed `OpenFile`. Rename the trait (for example `Seam`) and ship one ready implementation for the fake machine.

**7. sim-world `World`: parallel vectors and builders that retype every field** (elegance, economy)
- **Evidence:**
  - `pids`, `hosted`, `finished`, `restart` and `submissions` sit parallel to `procs` (`world.rs:59-73`). They are pushed together (`:275-280`), removed together (`:528-533`), and read through 20 `.expect("a status per process")`-style calls. `programs` and `startup` are parallel too.
  - `with_machine` and `with_meter` (`:140-196`) each retype all 22 fields to change one type parameter.
  - `turn` (`:406-505`) mixes reaping, admission of hosted children, metering, digests, cut planning, submission and hosted exit.
- **Fix:** keep `procs: Vec<P>` (because `turned` takes `&[P]`) and put the rest in `Vec<HostState>`. Move the fields that don't depend on the type parameters into an inner `Core`, so each builder becomes 3 lines. Pull `admit_spawned` and `submit_or_cut` out of `turn`.
- **Size:** about −80 lines.

**8. Test harnesses and fixtures retyped** (economy, section 10)
- **Evidence:**
  - `tests/readiness/tests/suite/readiness.rs:29-163` is a near-copy of `tests/ring/src/lib.rs:24-240`. Only the `Backend` and the field names (`submits`/`submissions`) differ.
  - The given-listener scenario exists twice: `crates/skein-sim/src/tests/given_listener.rs` and `tests/conformance/real/tests/support/given_listener.rs`, with an identical `io::Limits` literal (`:26-38` against `:7-21`).
  - kv's `Limits` literal is written 5 times (`tests/kv` `store.rs:10`, `real.rs:12`, `memory.rs:7`, `fuzzy/map.rs:8`, `src/tests.rs:15`), and only `golden.rs` gives derivations. `tests/kv/src/world.rs:117-130` retypes the constructor (`FileIo::new(8,…,1024,…)`, six `Queue::with_capacity(128)`). The `Request::Open {owner: Token::new(1), …}` prologue appears 15 times in `store.rs`.
  - Channel: `schema()` and `limits()` are written 4 times; the `Peer::step` driver 3 times (`src/tests.rs:120-155`, `generic_protocol.rs:84-120`, and `memory.rs::feed` with nine parameters). There are 12 step loops with bare bounds from 4 to 80 (`src/tests.rs:169,237,…`). `tests/channel/src/lib.rs` is a one-line comment, so nothing is shared.
- **Fix:** give `tests/ring`'s `World` a `Backend` parameter and drop its cfg; add shared kv and channel fixtures in each test crate's `src`; add a `run_until_quiet` helper.
- **Size:** about −350 lines.

**9. Conformance call sites** (economy)
- **Evidence:** the 5-line block `let seen = …; #[cfg(linux)] seen.check(); #[cfg(macos)] seen.check_darwin();` appears 9 times in `real/tests/cases/sockets.rs`. `racing_recv` and `racing_accept` (`sim/src/lib.rs:229-244`) exist only because `each_seed` takes `fn` pointers.
- **Fix:** add a `HostCheck` trait and an `on_host(BACKEND, scenario)` helper; make `each_seed` take `impl Fn(&mut Simulated) -> S`.
- **Size:** about −65 lines.
- **Answer to your question:** scenarios are written once. The per-backend test lists are thin and diverge legitimately (seed sweeps, faults the ring can't show). The exceptions are the io-level listener scenarios in finding 8.

**10. Conformance `Item` duplicates fake-machine's `Item`** (primitives)
- **Evidence:** `skein_conformance::{Item, Made}` (`lib.rs:122-188`) has the same constructors and default modes as `skein_fake_machine::{Item, Made}` (`fs.rs:144-213`). They are translated variant by variant in `tests/conformance/sim/src/lib.rs:149-160`.
- **Fix:** fake-machine's `lay` accepts the conformance `Item` (both crates are test-only), keeping `owner` and `Device` as extensions of its own.
- **Size:** about −80 lines.

**11. Minor**
- `sim/processes.rs:6-10` opts out of `indexing_slicing` and `wildcard_enum_match_arm` for the whole module (12 `_ =>` arms), while `sim.rs` and `files.rs` comply.
- `Child` (`processes.rs:36-50`) is another set of flags: `exit`, `stopped`, `held_exit`, `wait_state`, `descendant_running`, `pidfd_open`.

### 5. Cross-area suspicions (not checked outside this area)

- The fake-machine wrapper and the `Handle`/`Opened` conversions recur in `tests/io`, `tests/world`, `tests/oauth-accounts` and smith's `tests/*` and `smith-local-service`.
- Module-level `#![expect(clippy::disallowed_types)]` in a step crate suggests the workspace clippy setup doesn't hold the subset uniformly. Grep `skein-io`, `skein-http` and `skein-oauth*` for the same.
- lib's `Queue` has no `clear`, so channel hand-writes `for _ in 0..capacity { pop }` (`machine.rs:435-441`). This probably recurs.
- (unsure) The ring's `Table` (`ring.rs:104-135`) and readiness's slots (`readiness.rs:14-31`) each hand-roll a generational slot table with a free list and a token map. lib's `Slab`/`Id` may cover this, if its storage addresses stay stable enough for the ring's `UnsafeCell`s.

### 6. Rule costs

- **Exhaustive matches for derived predicates.** Channel's `Phase::may_ping`, `may_end_input` and `refusing` (`machine.rs:43-80`) take 37 lines for three booleans, and the cost grows with each new phase. That is acceptable.
- **"One behaviour per test" against conformance scenarios.** Each scenario returns 9 to 12 observations (for example `Removes`, `files.rs:456-537`), so the first failing assert hides the rest. One real-kernel process per scenario justifies bundling them, but section 10 would be clearer with an explicit exception for conformance scenarios.
---

# 03: wire-http-json-tls

**Code-quality review: skein-http, skein-json, skein-tls and their test crates**

The HTTP client, the HTTP server and the classic TLS client share one clear machine shape, and it is a good one. Three other machines (the SSE reader, the JSON collector and the TLS native face) do not follow it. The biggest saving is in the tests: the same machine-world skeleton is copied six times, roughly 1,000 lines that could go.

All paths below are relative to `T = <clone>/skein`.

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
---

# 04: codec

# Code-quality review: codec area (skein-codec, skein-codegen, tests/codec, tests/codec-patches)

The code is careful line by line. The problem is structural: the diff and the patch-text reader are written twice, behaviours are tested in three or four places, and the generator rewrites its own output as text. Codec is a binary-only format, so skein-forge, skein-llm and fake-forge-github decoding JSON by hand is correct by design today.

Tree: `<clone>/skein`. All paths below are relative to it.

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
---

# 05: forge

**Code-quality review: skein-forge (crates/skein-forge and tests/forge/github)**

The crate's size is mostly earned, but about 10% of production code (2.5–3.5k lines) and about 1k test lines could go. Most of it is copy-and-adapt between the two dialects and read plans that don't follow the house state-machine rule. I made no edits and ran no builds or tests.

Paths: `S` = `<clone>/skein/crates/skein-forge/src`, `T` = `<clone>/skein/tests/forge/github/src`.

## 1. Scope read
- **Closely:**
  - `api.rs`, `measure.rs`, both `decode/walk.rs`, `github/decode.rs` and `decode/issues.rs`, `forgejo/client.rs`.
  - `forgejo/server.rs`, `github/server.rs`, `server/{people,issues,writes}.rs`.
  - In both dialects: `plans/{plan,tables,failure,reading}.rs`, `plans/reads.rs`, plus `github/plans/reads/done.rs` and `forgejo/plans/reads/size.rs`.
  - `github/header.rs`, both `id.rs` and both `credential.rs`.
  - Tests: `github/tests/{fixtures,reads,pulls,writes,job}.rs`, `forgejo/tests/fixtures.rs`, `forgejo/plans/tests{,/edges}.rs`, `src/tests.rs`, `tests/common`, `tests/suite/github_plans.rs` (top part).
  - In `T`: `cases.rs` (part), `runner.rs` (`plan_as`, `decode`), the top of `tests.rs`.
- **Skimmed:** both write plans, deliveries, setup documents (orgs, hooks), forgejo `turn`/`actions`/`contents`, github app/log/protection decoders, the remaining tests.

## 2. Scores
1. **Cleanliness 3**: little dead code. Noise from 39 placeholder docs, about 220 lines of lint waivers in tests, and derived state.
2. **Organization 2**: the two dialects are not laid out in parallel, and GitHub imports Forgejo as its shared library.
3. **Taste 3**: careful, bounded and exhaustive; but decoders branch on `==` chains and decide by elimination.
4. **Naming 3**: mostly follows the design, with a few swaps against it (`Entry`/`TreeEntry`, `url`/`link`, `client`, `Drop`).
5. **Primitives 2**: four JSON walkers, the LRU hand-rolled four times, `Failure`/`Body`/`Heard` defined per dialect and again in the connection.
6. **Elegance 2**: read plans are bags of fields with a stage tag, against programming-model §5.4.
7. **Economy 2**: about 3k production and about 1k test lines are recoverable.
8. **Complexity 3**: complexity is earned by the breadth; still, some layers only forward, and some state could be derived.

## 3. Strengths
- Forgejo's test harness is the model to copy. `forgejo/tests/fixtures.rs:254-374` has named wire fixtures (`fixture`/`specimen`), sizes with their derivations and one `Run` driver. `edges.rs` job_failures rows carry only data.
- Forgejo shows the compact shape the rules allow: one `server::Answer` enum with one `encode` (`forgejo/server.rs:83-160`), and one `DecoderState` enum (`forgejo/client.rs:121-135`).
- Neutral shared modules exist where they were made: `directory.rs`, `marker.rs`, and `measure.rs` with compile-time width asserts.
- `github/failure.rs:5-40` names each refusal text and cites the conformance case that checks it.

## Area questions, in brief
- **Is the size earned?** Mostly. Production is 29.5k lines:
  - the wire layer is about 17.3k (59%), of which the fake's server side is 5.7k, as the design intends;
  - plans are about 8.9k (30%).
- **Per-field decode boilerplate.** Each field appears in 4–5 places: a table row, a setter arm, a zero-value initialiser, a document struct field, and the fake's writer (`key` plus value). There are about 440 table rows and about 430 writer `key()` calls. Total is about 2.8–3.2k lines, roughly a fifth of the wire layer and 10% of production. By hand that cannot shrink much. A generator would, but skein-codegen targets binary codecs (unsure whether it could cover JSON).
- **Is the vocabulary neutral?** Mostly; finding 8 has the leaks.
- **Module granularity:** inconsistent (finding 6).
- **In-crate tests:** Forgejo's are good, GitHub's are not (finding 9).

## 4. Findings (ranked by payoff)

**1. Read plans are field bags with a stage tag that keep re-matching `api::Read`.** Elegance, Primitives. Severity: style, highest payoff.
- **Evidence, GitHub:**
  - `github/plans/reads.rs:27-48` is a private copy of `api::Read`.
  - `:49-76` is a flat 26-variant `Stage`.
  - `:85-110` holds 25 fields (issues, pulls, checks, collaborators, remarks, reviews, directory, files, protection…), each meaningful for some reads only.
  - `reads/done.rs:26-160` re-matches all 19 variants of `self.read`. One list appears twice, the second time under a guard.
- **Evidence, Forgejo:** `forgejo/plans/reads.rs:50-52` holds three mutually exclusive `Option` sub-plans (`turn`, `contents`, `actions`), probed one after another at `:262-296`.
- **Evidence, across both dialects' plans:**
  - 81 `self.stage =` and 138 `self.stage ==`/`!=`, but no `mem::replace` transitions.
  - 29 exhaustive `api::Read` lists.
  - About 67 `if let`/`let…else` on the crate's own enums, e.g. `reads/people.rs:6,23,47` and `reads/job.rs:8-52`. Rules §10.2/10.3 exclude these.
- **Why it matters:** §5.4 says "no field is meaningful in some states only". Adding a read touches about a dozen sites. `let Read::X = self.read else { InvalidAnswer }` turns a plan bug into what looks like a peer error.
- **Fix:**
  - GitHub: `enum Plan { Issues(issues::Plan), Pulls(..), … }`, each with a `Stage` enum that holds its own rows.
  - Forgejo: `enum Read { Json(..), Contents(..), Actions(..), Turn(..) }`.
  - Repository resolution becomes one shared prefix state.
- **Size:** about −800 to −1,200 lines (the upper figure is uncertain).

**2. Four hand-written JSON field walkers.** Primitives, Economy. Severity: style.
- **Evidence:**
  - `forgejo/decode/walk.rs` (559 lines) and `github/decode/walk.rs` (1,250 lines; its header says "independent of Forgejo's tables") have the same parts: `Field{key,read,required}`, `fields`/`element` tables, `Frame{touched,seen}` masks, a `Want` machine, and the required-field check (`:473` vs `:1113`).
  - A third walker: `github/server/writes.rs:113-375` (`Parser`).
  - A fourth: `server/queries.rs:115-149` tokenizes the whole body into a `List<Token>`.
  - Per-node rules are hard-coded in the walkers instead of in the tables: `forgejo walk.rs:318-338` chooses key names by kind, `:450-454` treats null as seen for "statuses"/"creator"/"tree", and `github walk.rs:739-790` hard-codes AppEvents, Contexts, GraphqlErrors and LogRows bounds.
  - skein-json already has a static-path walker, `crates/skein-json/src/collector.rs:25-90`, which skein-llm uses.
- **Fix:** one crate-level walker over a union `Node { Forgejo(..), Github(..), GithubBody(..) }`. Move the per-node rules into the table (`Read::Strings{cap}`, `null_seen`). Alternatively, evaluate `collector` (unsure whether it handles streamed rows).
- **Size:** about −600 to −900 lines.

**3. GitHub uses Forgejo as its common library, and copies the rest.** Organization. Severity: style.
- **Evidence:**
  - 44 references in 26 GitHub files to `forgejo::{id::unsigned, time, path, document::Body, decode::Malformed}`, e.g. `github/decode.rs:7` and `github/request.rs:455`.
  - Copied instead of shared:
    - hex digit helpers: `github/id.rs:15-24` = `forgejo/id.rs:28-34,62-68`;
    - body/revision/provenance: `github/plans/reading.rs:6-54` ≈ `forgejo/plans/reading.rs:63-116`;
    - `Failure` plus `transport()`: `forgejo/plans.rs:39-47` = `github/plans.rs:25-31`;
    - the forwarding `Plan{Read,Write}`: `forgejo/plans/plan.rs:27-98` vs `github/plans/plan.rs:42-142`;
    - the credential prefix (the GitHub copy uses a bare `7usize`).
- **Fix:** move these to neutral crate-level modules beside `directory.rs`.
- **Size:** about −200 lines, and github no longer imports forgejo.

**4. The LRU table is hand-rolled four times, and the copies have drifted.** Primitives. Severity: risk.
- **Evidence:**
  - `forgejo/plans/tables.rs:47-89` and `github/plans/tables.rs:57-106` hold four copies of the eviction loop.
  - Forgejo's clock `expect`s at `u64::MAX` (`:37`) and `expect`s room even at capacity 0. GitHub saturates (`:47`) and returns early at capacity 0.
  - §10.2: "what is missing goes into lib".
- **Fix:** `skein_lib::Lru<K,V>`.
- **Size:** about −100 lines, and one behaviour.

**5. Two measures of the same answer.** Economy. Severity: risk.
- **Evidence:**
  - `measure.rs:151-254` and `forgejo/plans/reads/size.rs:28-144` both size `api::Answer`, and `tests.rs:100-103` asserts that they agree.
  - `size.rs:107-121` already calls `measure::answer` after a `rows(...)?` whose value it discards.
  - It also leaves out `dequeued`, which `measure::pull` counts.
- **Fix:** `measure::answer` plus one check of the longest list against the limit.
- **Size:** about −130 lines.

**6. The two dialects took opposite shapes for the same jobs.** Organization, Naming. Severity: style.
- **The engine side:** Forgejo calls it `forgejo/client/` (15 files), which collides with the skein-forge-client crate. GitHub calls it `request.rs` plus `decode/`.
- **The decoder:**
  - Forgejo has a `DecoderState` enum.
  - GitHub has a struct holding every sub-state at once (`github/decode.rs:35-58`), although its doc says "Retains only the selected document".
- **The fake's answers:**
  - Forgejo has one `encode`. GitHub repeats the 9-line measure-then-write in 19 places (`github/server.rs:263,302,326`).
  - `server/people.rs:15-25` merges two unrelated documents behind an `Option` just to share that code.
  - Six types carry lifetime parameters (`Answer<'a>` in `issues.rs:11`, `pulls.rs:8`, `app.rs:108`); §10.3 excludes these.
- **Same name, different job:**
  - `reads/size.rs` is branch admission in GitHub but the answer measure in Forgejo.
  - Each dialect has both `failure.rs` and `plans/failure.rs`, with `classify` on opposite sides.
  - The hint mapping is in `forgejo/plans/hint.rs` but in `github/delivery.rs:121`.
  - Two enums are named `Drop`, which shadows the prelude trait.
- **Granularity:** `github/pin.rs` (7 lines), `decode/people.rs` (17), `writes/merge.rs` (23) sit beside `request.rs` (1,319) and `walk.rs` (1,250).
- **Design:** forge-github.md §2 says GitHub "holds the same" list as forge.md §4.1.
- **Fix:** one module map for both dialects, Forgejo's enum shapes for the decoder and the server, and files under 50 lines folded into their callers.
- **Size:** about −150 lines.

**7. The GitHub decoder routes each node twice and parses numbers twice.** Taste, Economy. Severity: style.
- **Evidence:**
  - `github/decode.rs:225-298` and `:437-525` both list the same five node groups (54 nodes). `Protection` and `PullGraphql` are already nested enums; the other groups are not.
  - The walker passes the raw `Token` on, so setters call `unsigned()` about 34 times with `expect("walker checked integer")`.
  - `decode/issues.rs:96-125` decides by elimination: any unmatched string is a time, and any `True` sets `merged`.
  - About 100 `node ==`/`kind ==` comparisons on own enums across the decoders.
- **Fix:** nest every node group, emit a typed `Value` as Forgejo's walker does, and match exhaustively.
- **Size:** about −120 lines; a new field then fails the build where it is not handled.

**8. Leaks and naming slips in the neutral vocabulary.** Naming. Severity: style.
- `api.rs:463` documents `JobAttempt` as "A Forgejo Actions job attempt".
- `Entry`/`EntryKind` (`:197-215`, a directory entry) and `TreeEntry`/`TreeEntryKind` (`:590-604`) are swapped relative to forge.md §3.6, where `Entry` is the tree's entry.
- `Status.url` (`:457`) vs `Write::Status.link` (`:149`); the design says "link".
- The `Commit` alias is used for tree object ids (`:130`, `:595`).
- 39 placeholder docs that restate the name ("/// Read item.", "/// Return done.").
- In `github/header.rs`:
  - `RateFields` duplicates `api::Rate`.
  - `Head.rate` is derived from three other fields (`:200-203`), so `remaining()`/`reset()` at `:120-133` branch to the same value either way.
  - `Cursor` and `Etag` duplicate `api::Validator`, with a bare 128.

**9. GitHub in-crate tests retype the harness, branch on rows, and test several behaviours at once.** Tests. Severity: style.
- **Retyped harness:**
  - `struct Call{plan,tables,bounds}` and its helpers appear in `github/tests/{reads.rs:40,job.rs:39,pulls.rs:78,checks.rs:54}` and `tests/suite/github_items.rs:80`.
  - `const KEY` is retyped 6 times and `bounds()` 11 times.
  - `tests/suite/github_plans.rs:23-40` retypes the limits in `fixtures.rs:13-34`.
- **Whole-body decode loop written 9 times:**
  - twice in one file (`fixtures.rs:60-104` and `:145-185`);
  - the production copy at `github/decode/delivery.rs:274-322` hand-rolls `Intake::meet`.
- **Row branching:**
  - 40 loops over `[false, true]`, many branching on the flag (`reads.rs:112-124`, `pulls.rs:643-654,688-708`, `writes.rs:924-960`).
  - `pulls.rs:645` computes the expected value with the code under test.
- **Several behaviours per test:** `github/tests.rs:167` holds five table tests. `src/tests.rs:443-485` uses bare sizes 408/201/90.
- **Names:**
  - 190 of 224 GitHub tests are prefixed `github_`.
  - `forgejo/plans/tests.rs:37` `github_rate_and_validator_are_none_in_forgejo_plans` asserts neither.
  - `reads.rs:1` refers to "Task 18".
- **Lint waivers:** 65 `#![expect]` in 20 files under `crates/skein-forge/tests`; one set in `suite/main.rs` (the crate root) would do.
- **Size:** about −600 to −900 lines.

**10. The GitHub conformance retypes what it should be checking.** Tests. Severity: risk.
- `T/cases.rs:439,443,451,499,566,1084,1089` retype about 13 refusal texts that `github/failure.rs` already names. If a constant drifts, the fact still passes. Assert against the constants.
- `T/tests.rs:25-45` reinvents `skein_scratch::Scratch` under a hard-coded `/tmp`, although skein-scratch is already a dev-dependency.
- `T/runner.rs:681-700` puts test-only fault injection, keyed on strings, inside the plan driver.

## 5. Cross-area suspicions (unverified outside my area)
- `crates/skein-forge-connection/src/connection.rs:55-80,815-840` wraps every per-dialect type (`Request`, `Decoder`, `Body`, `Heard`, a third `Failure`, `JsonStep`). That is a direct cost of findings 3 and 6.
- The loop that drives a plan over HTTP exists in the connection and in `T/runner.rs:668`; probably also in tests/forge/machine.
- The whole-body tokenizer loop also appears in `tests/forge/setup/src/decode.rs`, `machine/src/{memory,pipe}.rs`. A shared helper could live in skein-json.
- The same `[lints]` block is copied into each `tests/forge/*/Cargo.toml`.
- skein-llm decodes JSON with skein-json's `collector`; the forge kit does not.
- skein-http has `fn decimal` twice (`server/head.rs:380`, `client/head.rs:281`); forge's `id::unsigned` is a third.

## 6. Rule costs
- **No closures:** the measure-then-write pass must be written once per document. Forgejo's single-enum shape shows the minimum is one site per dialect; GitHub has 19.
- **No generics:** a shared walker needs a crate-wide `Node` union, about 30 lines of dispatch.
- **Exhaustive matches:** the 29 `api::Read` lists (about 580 lines) look forced by the rule, but they come from the plan design (finding 1), not the rule.
---

# 06: forge-connection-client

I read the area against your brief and the house rules. The main results:

- **forge-client is in good shape.**
- **forge-connection's core and tests are the problem.** The tests at its own tier exist in two copies.
- **The duplicated mechanism with llm-connection is real.** It is the socket-plus-TLS connection, which is also hand-written in two more crates.

All paths below are relative to `<clone>/skein/`.

## 1. Scope read

**Read closely:**
- forge-connection: `connection.rs`, `component.rs`, `component/{calls,github,github_calls}.rs`, `installations.rs` (the `Row` and `GitHub` structs), `hooks/connection.rs`, `puller.rs`, `grow.rs`.
- forge-connection tests: `tests/fixtures.rs`, part of `tests.rs` and `tests/github_installation_loss.rs`, and samples of `reads.rs` and `writes.rs`.
- forge-client: `outbox.rs`, `outbox/{stop,queue,answer}.rs`, `domain.rs` (`step`, `resume`, `fire`), and all of `tests/fixtures*`.
- llm-connection: `call.rs` and `deadlines.rs`.
- Test worlds: machine `machine.rs`, `fixtures.rs` and `peer.rs`; client world `world.rs` and `recovery.rs`; protocol world `restart`.

**Skimmed:** `limits.rs`, `settings.rs`, `hooks.rs`, the puller and logs modules, `keep/*`, `calls.rs`, `api.rs`, the other in-crate test files (names and setups only), `joint*.rs`, `memory.rs` and `translate.rs`.

## 2. Scores

1. **Cleanliness: 3.** There is little dead code. But there are test-only hooks into internal state in production code (`component.rs:188-217`), and the 85-arm `raw_request` carries comments on arms that never happen.
2. **Organization: 2.** The component's tests live in five places, as two copies of one tier, and Forgejo-plan unit tests sit in the wrong crate.
3. **Taste: 3.** forge-client reads like the house style. forge-connection's core reads as accreted: five 64-turn loops and 38 copies of one accessor chain.
4. **Naming: 3.** Names are precise at the boundary. Inside, "settle" means three things in forge-client, `closed` means closing, and `Waiting` names every call.
5. **Primitives: 2.** The design's state tables (8.1, 8.4, 8.6) are not state enums. Per-forge state sits in four parallel lists. There is no shared transport, and `Grow` redoes lib's `Intake`.
6. **Elegance: 2.** A connection's phase and deadline are set at 27 sites instead of once, as a function of the phase. forge-client alone would score 4.
7. **Economy: 2.** Harnesses, fixtures, GitHub HTTP fakes, limits literals and restart sequences each exist in two or three copies.
8. **Complexity: 3.** The GitHub installation state is far more intricate than its five-row design table. There is little speculative layering, though `Recovery::Keyed` is never produced.

## 3. Strengths

- **The forge-client outbox.** `State::{Due,Busy,Waiting}(Phase)` (`outbox.rs:42-60`) keeps the lane machine small, handlers are free functions over `d`, and `outbox/stop.rs` covers 7.8 in 70 lines.
- **`puller.rs:8-13`.** Its `State` enum maps design 8.7 one to one.
- **`start::heard`.** It is a pure transition tested with a table of data rows (`src/tests.rs:31-66`). This is the shape the rest should copy.
- **The machine-world suite** (`tests/forge/machine/tests/suite/machine.rs`). Its tests are short, named for a behaviour, and built from recorded cases.

## 4. Findings, ranked by payoff

**1. Two harnesses of the same tier for one component (Organization, Economy).**
- **What is duplicated:**
  - The in-crate `fixtures::Harness` (`crates/skein-forge-connection/src/tests/fixtures.rs:116-465`: fake io, one `Wire` per socket, collectors for every event) is the same tier as `Machine` (`tests/forge/machine/src/machine.rs:85-860`).
  - `tests/forge/machine/src/fixtures.rs:1-90` is a verbatim copy of in-crate `fixtures.rs:1-96`: the same doc line, the same 38-field `limits()`, `shared()`, and the FORGE, REPO, TOKEN and WRITER fixtures.
  - Two GitHub fakes hand-format `HTTP/1.1` around `skein_fake_forge_github::service::Service`: `src/tests/github_files/fake.rs` (396 lines) and machine `src/github.rs` (605). The fake crate already has its own `server.rs`, and `joint/github.rs` uses `skein_http::server`, a third way.
- **Same behaviour tested in both:**
  - `start_backoff_doubles_to_its_maximum` (`src/tests/connection.rs:543`) and `a_start_doubles_its_backoff_to_the_cap`.
  - `a_refused_connect_waits_its_backoff…` (`:129`) and `an_unreachable_forge_is_checked_again…`.
  - `a_call_waiting_for_its_forge_is_unavailable…` (`src/tests.rs:115`) and `a_call_waiting_past_its_deadline…`.
- **Why the in-crate copy exists:** to poke internals. There are 160 `test_github()`/`test_github_mut()` calls, and tests set `gh.held` and `gh.read` directly (`github_installation_loss.rs:93-100,124-130`).
- **Fix:** keep in-crate only the step tests of private pure functions (start, installations, limits, grow). Move the byte-level stories to the machine world, asserting on events and wire bytes rather than fields. Keep one GitHub HTTP face, the fake's own `server.rs`.
- **Size:** about −1,500 to −2,500 lines. It also unblocks finding 3.

**2. The socket+TLS(+HTTP) stack is hand-written again (Primitives; your question 1).**
- **What forge needs that llm doesn't:**
  - The GitHub App: about 2,800 production lines (`installations` 682, `component/github` 999, `github_calls` 631, puller 389, app and logs 120).
  - The webhooks listener: 486 lines.
  - Two dialects with resolution tables.
  - Start checks.
  - Limits and settings: about 800 lines against llm's 135.
- **What is duplicated with llm-connection:**
  - `connection.rs:116-146,351-383,397-450,499-523` (socket token, `Option<tls::Client>`, `tls_up`/`tls_down`, the handshake, routing between cipher and plain text) repeats `call.rs:50-112,180-200,371-505`.
  - Connection pooling repeats llm's `reusable_connection` and `start`: token issuance, the connection cap, idle reuse per endpoint and evicting another forge's idle connection (`component.rs:694-731`, `calls.rs:130-200`).
- **Where the composition goes:** llm gets HTTP and framing from the composition crate skein-llm. forge composes HTTP→JSON→dialect decoder inline, and `crates/skein-forge-setup/src/host/connection.rs` (760 lines) is a near-copy of that composition, with the same fields.
- **Shared shape:**
  - (a) A `Link`: socket token + optional TLS + its two queues + a phase {Connecting, Handshaking, Open, Closing}. It takes and gives plain-text `Down`/`Up`. All four connection crates would use it (oauth-accounts' `exchange.rs` too).
  - (b) A forge exchange composition in skein-forge, as skein-llm is for LLM calls, used by both forge-connection and forge-setup.
  - (c) Optionally, a pool per endpoint with its idle deadline.
- **Size (estimate):** −300 in `connection.rs`, −400 in forge-setup, −100 to −150 each in llm-connection and oauth-accounts.

**3. The design's state tables are written as flag structs (Primitives, Elegance; rule 5.4).**
- **Connection (8.1):** `Connection` (`connection.rs:116-146`) has 31 fields, many meaningful only in some phases (`request`, `upload`, `done`, `receive`, `raw_range`, `closing`, `draining`, `written`). `Phase::Sending{written}` duplicates `self.written` (both read at `component/github.rs:859`).
- **Phase and deadline:** the phase is assigned at 12 sites and the deadline at 15 sites across five files. Design 8.1 says the deadline is "a function of the phase, set in one place".
- **Webhook connection (8.4):** `Hooked` (`hooks/connection.rs:32-52`) covers four design states with three bools, `HintState`, `Closing`, four Options and a `forge: 0` sentinel.
- **Installation tokens (8.6):** `installations::GitHub` (`installations.rs:355-379`) has 23 public fields and six bools, and `mint` duplicates `selected` and `Row.state`. `github_wait` resets eight fields by hand (`github.rs:113-160`).
- **Start:** the public `Start` is shadowed by a private `Work`, so `github_state` re-synthesises it (`github.rs:75-95`).
- **Calls:** the call struct `Waiting` has 14 fields, and `op`/`plan` are mutually exclusive.
- **Fix:** one enum per design table, the transition idiom, and one function that sets demand and deadline after every transition. llm-connection's `Table`, armed from the phase in one call (`call.rs:159-170`), is the model.
- **Size:** about −250 lines, with a large gain in clarity.

**4. Four copies of the loop that drives a connection's holder (Economy).**
- **The copies:** `drive_forgejo` (`component.rs:648`), `pump_call` (`calls.rs:242`), `pump_github_call` (`github_calls.rs:277`) and `drive_github` (`github.rs:890`).
- **What each does:** 64 turns of deadline→timeout, Idle→send, pump, charge the cost on the first write, heard→plan step, Closed→close, ready→wake. Each applies its own deadline rules.
- **Fix:** one `drive_holder` that matches on `Holder{Call(id), Own(at)}`.
- **Size:** −150 to −250.

**5. Per-forge state is held in parallel lists (Primitives).**
- **The lists:** `settings.forges`, `starts`, `github`, `own_tokens` and `own_plans` (`component.rs:103-121`), all indexed by `at`.
- **The repeats it causes:**
  - 38 copies of `.get_mut(at).expect(..).as_mut().expect("GitHub state")`.
  - 22 copies of `own_plans.get_mut(at)`.
  - 10 linear lookups of a forge by number.
  - The scan for a GitHub forge's reserved connection, written twice (`component.rs:734`, `calls.rs:153`).
- **Fix:** `List<Forge{settings, start, own: Option<Own>, github: Option<Box<GitHub>>}>`, with one lookup by number and one GitHub accessor.
- **Size:** about −120.

**6. The two dialects run as six parallel enums (Economy; the root is in skein-forge).**
- **The enums:** `Exchange`, `Heard`, `Refusal`, `Request`, `Decoder` and `Body` (`connection.rs:37-95`) each split into `Forgejo|GitHub`. They wrap `forgejo::plans::{Body,Heard,Failure}` and `github::plans::{…}`, which have the same shapes (`skein-forge/src/forgejo/plans.rs:23-53`, `github/plans.rs:18-44`).
- **What that costs:** `closes`, `raw`, `empty`, `malformed_body` and `failed` each re-dispatch on the dialect. `raw_request` (85 arms, `:1008`) and `github_raw_request` (52 arms, `:1085`) decide "raw or discard" outside the crate that defines the requests.
- **Fix:** dialect-neutral types in skein-forge whose only dialect-specific parts are the document, row and head types, with each dialect's decoder returning `Json|Raw|Discard`.
- **Size:** about −200 here and −100 in skein-forge.

**7. `Grow` re-implements `Intake` (Primitives).** `grow.rs:11-50` refuses a piece whole and doubles its buffer, copying byte by byte on each growth. That is `skein_lib::Intake::append` (`intake.rs:108`). **Fix:** use Intake. About −50 lines.

**8. forge-client checks its done flags at every exit (Elegance).**
- `domain.rs` emits `OutboxDone` at six sites (208, 238, 258, 264, 271, 277) and `ReadAfreshDone` at three. `resume` has four near-identical branches mainly to append it.
- **Fix:** one function that emits both, called at the end of `step`, `resume` and `fire` (5.4: applied in one place). About −30.
- Also, `echo` (`outbox.rs:487-512`) re-enumerates `Write` to get what `key()` (`:448`) already returns.

**9. Naming.**
- **"Settle" means three things in forge-client:**
  - The 7.8 stop: `Settle`, `Settled`, `stop::Settling`, `settling_ready`.
  - The restart pass: `SettleOutbox`, `outbox.settling`, `settle_done`.
  - Charging a call's cost to the budget: `calls::settle`.
  The design shares the clash (forge-client.md 7.8 against 8.5). Rename the restart pass "find pass" and `calls::settle` to `charge`.
- **Other forge-client names:** `api.rs` holds answer admission and shadows `skein_forge::api`. `calls.rs:1` cites a section 5.4 that does not exist. `Recovery` is used once and `Keyed` is never produced, while the outbox branches on `is_set`, `key` and `is_merge` instead.
- **forge-connection:** `Component.closed` means closing and `ended` means closed, against 5.2's words (`component.rs:113,122`).

**10. forge-client's test harnesses (Economy; testing-strategy 10, "a setup written once").**
- Three harnesses (`tests/fixtures.rs:19`, `fixtures/outbox.rs:11`, `fixtures/keep.rs:14`) each route all 13 `Request` variants and have their own `send` and `one`. The keep harness counts `Settled` as `outbox_done` (`keep.rs:94`).
- The five-line "lost comment" opening is typed out eight times in `tests/outbox.rs` (34, 48, 98, 115, 138, 273, 395, 433). `recovery.rs:577` already holds it as `lost_turn`.
- The eight-step restart sequence is written twice: `tests/forge/client/src/world/recovery.rs:4-45` and `tests/forge/protocol/src/world.rs:484-511`. The protocol world already depends on the client world.
- **Size:** −200 to −300.

**11. In-crate test layout and test shape (Organization).**
- **Layout:**
  - **Five locations:** an inline `mod tests` (`component/github.rs:1000-1343`), co-located `component/tests.rs` and `connection/tests.rs`, the `src/tests.rs` root (limits boundaries also tested in `limits/tests.rs`), and 18 files under `src/tests/`.
  - **Cross-imports:** test files import fixtures from each other (`github_start.rs` exports 11 items).
  - **Wrong crate:** pure Forgejo-plan tests (`reads.rs` `reading::*`, `writes.rs:182-231` `classify`) live here, while GitHub's equivalents live in skein-forge.
  - **Re-invented helpers:** `contains` twice and request counting three times, though `skein_lib::bytes::count` exists.
- **Shape breaches:**
  - Assertions along the way: `src/tests.rs:115-133` asserts at six points of its timeline.
  - Bare sizes: `Start::Ready{cap: 100}` (`github_installation_loss.rs:147`), `requests(&h), 7` (`:174, :290`), `1041/1042` (`tests.rs:88`).
  - A no-op fixture: `hook_limits()` (machine `fixtures.rs:188-197`) sets five fields to the values `limits()` already has.

**12. Small rule breaches in step code.**
- Tuple scrutinees: `connection.rs:160` (with a guard), `connection.rs:254`, `limits.rs:489`, client `domain.rs:97`.
- `if let` on own enums: `keep/answer.rs:267,501`, `keep/poll.rs:151,187`.
- `next_deadline` (`component.rs:544-590`) inlines the "earlier of two deadlines" check six times.

## 5. Cross-area suspicions (unverified beyond my area)

- **The socket+TLS connection is hand-written in four crates:** `skein-llm-connection/src/call.rs`, `skein-oauth-accounts/src/exchange.rs`, `skein-forge-setup/src/host/connection.rs` and forge-connection.
- **`fn earlier` exists in four crates:** llm-connection `deadlines.rs:159`, oauth-accounts `component.rs:1239`, forge-setup `host.rs:376`, and a private one in lib `requests.rs:718`. It belongs on lib's `Time`.
- **A fake io socket table is rewritten for each world.** There are io routers at:
  - `tests/forge/machine/src/{machine.rs:618, joint.rs:440, joint/github.rs:200, memory.rs:322, hooks.rs:272}`
  - `tests/forge/protocol/src/github.rs:443`
  - `tests/forge/setup/src/{host.rs:511, memory.rs:109}`
  - forge-connection `src/tests/fixtures.rs:376`

  skein-world provides `stream::Wire` but no socket table.
- **skein-forge's plan types are parallel** across Forgejo and GitHub (finding 6).
- **`skein-fake-forge-github`'s `server.rs` is bypassed** by two hand-written HTTP faces (finding 1).

## 6. Rule costs

- **In-crate tests inherit the step crates' clippy config** (no `Vec`). Fixtures therefore use `List::with_capacity(32)` plus `.expect("…fit")`, 36 times in `src/tests`. Moving stories to `tests/forge/*` removes most of it.
- **`wildcard_enum_match_arm` applies in tests.** There are 21 full `Event` enumerations just to pick out one variant (e.g. `component/github.rs:1086-1098`). A helper that collects one kind of event absorbs them.
- **The 15-variant `Write` is enumerated 20 times in forge-client production code,** often to read one field already validated in `make` (`queue.rs:43-58`). Carrying that field in the `Phase` (for example `Queued{head}`) saves about 150 lines.
---

# 07: forge-setup

The setup crate is sound in design but uneven in execution. It is not bloated at 17.6k, though about 1.5k lines could go, mostly in tests. Its host also re-implements the TLS→HTTP→JSON connection that skein-forge-connection already has.

## 1. Scope read

Tree: `<clone>/skein`

- **Read closely:** in `crates/skein-forge-setup`, `reconcile.rs` with `reconcile/tokens.rs` and `orgs.rs`, `access.rs`, `call.rs`, `limits.rs`, `github/{app,write,convert,paste,reconcile,hook,limits,given}.rs`, `host/{run,settings,connection}.rs` (first 420 lines and the helpers), `host.rs` (first 200 lines), the head of `reachable.rs`, `src/tests/{fixtures,github_fixtures}.rs`, and the helper headers of every `src/tests/github_*.rs`. In the setup world, `src/{world,exchange,fixtures}.rs` and `tests/suite/reconcile.rs`. In the conformance tool, `lib.rs`, `main.rs`, `cases.rs`, `tokens.rs`, `write.rs`, `mask.rs` and `json.rs`. Also `skein-forge-conformance/src/{lib,facts,json}.rs` and the throwaway's `lib.rs`, `http.rs` and `instance.rs`. To compare, I read parts of skein-forge-connection's `connection.rs`, `settings.rs` and `limits.rs`.
- **Skimmed:** `create.rs`, `item.rs`, `host/limits.rs`, `boundary.rs`, `src/tests/{host,github_host,steps}.rs`, the world's `HostWorld`, the conformance's `facts.rs`, `deliveries.rs` and `tests.rs`.
- **Not read:** the tests for manifest, memory and states, the throwaway's CLI and its tests, the conformance's component, runner and turn modules.

**Split:** about 7.0k lines of production code against 10.6k of tests in the crate (60% tests). Production is roughly proportionate for 12 plans plus a host. Roughly 0.7k of production and 1k+ of tests could go.

## 2. Scores

1. **Cleanliness: 3.** It reads clean, but has dead guards, refusal arms the host can never reach, redundant `#[path]` attributes and explicit `drop()` calls.
2. **Organization: 2.** The shared plan vocabulary lives in Forgejo's `reconcile` module, Reachable mixes both dialects, and tests are mounted from three parent modules.
3. **Taste: 3.** The GitHub plans are crafted well. Halting works by feeding a plan a wrong event, and `closing: Option<bool>` stands in for a state.
4. **Naming: 3.** Test names are excellent. Type names collide (`Need`, `Event`, `Limits`, `Given`, `Refused`, `Place`), and failure subjects drift from the design.
5. **Primitives: 2.** `Reply` mixes the two dialects, the connection pump is duplicated, `Plan` does not encode its dialect, and an item is validated in three layers.
6. **Elegance: 3.** The plans follow the design's steps, but Forgejo's plans and GitHub's are written in two different state-machine styles.
7. **Economy: 2.** Test helpers are retyped in every file, the setup world has two harnesses, and fixtures are retyped across tiers.
8. **Complexity: 3.** It is mostly justified. The host keeps its state in flags, and dialect checks happen twice.

## 3. Strengths

- `github/paste.rs`, `github/reconcile.rs`, `github/write.rs` and `github/hook.rs` follow programming-model 5.4 exactly: each state variant holds its own data, they use the `mem::replace` idiom, secrets are dropped at terminal states, and none of them needs an `expect`.
- Test names are design sentences, for example `a_token_file_holding_another_users_token_is_made_again`. World tests assert on the fake's store, not on request logs.
- `item.rs` puts one total decoder per item in one file. Limit refusals name their `Place` pair as design 6.1 says.
- `skein-forge-conformance` already factors facts, masking, the leak scan and JSON across the Forgejo and GitHub tools.

## 4. Findings

Answers to the area questions:
- **Is 17.6k proportionate?** Mostly, yes. There is no per-step template, and the cost is the duplication described below.
- **Re-implemented connection mechanics?** Yes; see finding 1.
- **Conformance cases written once or twice?** Once, and run only against the throwaway Forgejo; the fake is never held to them (fake-forge.md section 9 makes that the default).
- **Fixtures:** see findings 3 and 12.

**1. The host's connection re-implements skein-forge-connection's transport** (Primitives, Economy)
- `host/connection.rs:28-50` has the same TLS→HTTP→tokenizer pump as `skein-forge-connection/src/connection.rs:126-140`: the seven queues `tls_up`/`tls_down`/`http_up`/`http_down`/`json_up`/`json_down`/`json_next`, and the same `connect`, `stream`, `http_down` and `json_down` functions.
- `host/settings.rs:87-102` `host()` is byte-identical to forge-connection's `settings.rs:75-90`. The `Transport` enum and `field()` (`connection.rs:655`) are copies too.
- The transport and address checks in `host/limits.rs:185-203` mirror forge-connection's `limits.rs:381-410`, down to the same `Unusable` variant names.
- The method mapping appears five times: setup `connection.rs:96` and `:168`, forge-connection `connection.rs:242` and `:275`, and `tests/forge/setup/src/exchange.rs:84`.
- **Fix:** move one transport primitive (the pump, `Transport`, `host`, the method mapping and the transport checks) into skein-forge or a small crate. The setup keeps only its one-request-per-connection policy and its decoding into `Reply`.
- **Size:** about 400 of the 760 lines in `connection.rs`, plus about 60 in `settings.rs` and `limits.rs`. This is unsure until forge-connection's keep-alive and redirect handling is weighed.

**2. The setup world has two harnesses** (Economy, Primitives)
- `World` (`world.rs`, with `exchange.rs` and `decode.rs`, 766 lines) drives the plans directly. `exchange.rs` builds its own HTTP client calls with its own `Host` and `Authorization` headers and its own method mapping, and panics on anything but a Forgejo token.
- `HostWorld` drives the real host and already serves Forgejo (the `hooks` and `host` suites use it).
- Design section 7 says the world "plays the host component". As things stand, the Forgejo stories in `reconcile`, `check`, `take_over`, `reachable`, `access` and `fuzzy/reconcile` never exercise the host's connection.
- **Fix:** port those stories to `HostWorld` and delete `World`. About 500 lines go.

**3. Unit-test helpers are retyped in every file** (Economy, testing-strategy section 10: write a setup once)
- `fn go` appears in 11 files, `silent` in 9, `failure` in 11, `send` in 8, `at` in 11 and `diagnostic` in 4 (`grep` over `src/tests`).
- `const APP: u64 = 19` is declared in 7 files, while `github_fixtures.rs:25` already hard-codes `id: 19`.
- `fixtures.rs:24,280,301,402` has four `Run` wrappers, each with the same `step` body.
- There are three host harnesses with the same fields: `tests/host.rs:104`, and `github_host.rs:112` and `:292`.
- `host::run::Machine` (`run.rs:84-118`) already dispatches `step` and `halt` for all 12 plans. The setup world's `World::drive` already takes a closure, which shows the shared helper is easy.
- **Fix:** one `go(&mut Machine, Event) -> Need` and one `silent(&mut Machine)`. Put `APP`, `BOT`, `URL` and `diagnostic` in `github_fixtures`, and keep one host harness. About 400 lines go.

**4. Forgejo's plans keep state data in `Option` and sentinel fields** (Primitives, rule 5.4)
- `Reconcile` (`reconcile.rs:139-154`: `admin`, `tokens`, `orgs`, `access`), `Create` (`create.rs:35-50`: `admin_login: []`, `branch: []`, `id: None`), `Access` and `Convert` (`convert.rs:34-43`, where `id: 0` and `slug: []` mean "not yet") hold per-state data in fields. About 17 `expect("... before ...")` calls exist only to unwrap them.
- The host does the same with flags: `connection.rs:28-50` (`closing: Option<bool>`, `closed`, `close_sent`, `answered`) and `run.rs:158-175` (`retry`, `retry_at`, `answered`). forge-connection uses a `Phase` enum instead.
- **Fix:** move the data into the state variants, as `paste.rs` does. This is about size-neutral and removes the `expect`s.

**5. Halting works by provoking an unexpected event** (Taste, Complexity)
- `run.rs:119-136` (`halt_begun`) feeds `Event::Begin` to a running plan, relies on it stopping with `Unexpected`, and then rewrites the `why`.
- `reconcile.rs:450-471` feeds `Written` or `Begin` for the same effect.
- The subject is also patched after the stop is built: `reconcile.rs:416-420` (a `let-else` ending in `unreachable!`) and `tokens.rs:239-246`.
- This is fragile: any state that accepted `Begin` would silently advance instead of halting.
- **Fix:** give every plan a `halt` that calls its own `stop(step, subject, why)`, as `create.rs:349-354` and `paste.rs` already do. About 50 lines go, along with two `unreachable!` calls.

**6. The shared plan vocabulary lives in Forgejo's reconcile module, and `Reply` mixes dialects** (Organization, Primitives)
- `Need`, `Event`, `Reply`, `Failure`, `Stage`, `Why` and `Outcome` are defined in `reconcile.rs`. 22 files import them, every GitHub plan among them. The `reply` and `unexpected` helpers live in `reconcile/tokens.rs:218-233`, the token step.
- `Reply` carries `Document`, `Rows`, `GithubDocument`, `GithubRows` and `Unsigned` together. So each of the 34 "reject the rest" arms lists the other dialect's variants; see `access.rs:134-208`.
- **Fix:** add a `plan.rs` module for the vocabulary and split the answer type per dialect. This saves about 70 lines and makes cross-dialect replies impossible to express.

**7. GitHub's plans contain near-copies** (Economy)
- `paste.rs:56-73` and `github/reconcile.rs:57-73` are the same child-runner function.
- The App check (id, then slug, then `lacks`) is written three ways: `convert.rs:140-150`, `paste.rs:84-111`, and `github/reconcile.rs:92-106` with its helper at `:166`.
- Two identical `Progress` enums exist (`hook.rs:22`, `installations.rs:41`), and two `Checked` structs.
- Children answer through a `Queue::uncapped()` that the parent then pops. `Progress::Pending(Need::Done)` is accepted by the type and then rejected at runtime.
- **Fix:** one `app::proved(...) -> Result<App, Why>`, and one `Progress` returned directly with no `Done` in it. About 80 lines go.

**8. `Plan` does not encode its dialect** (Primitives)
- `Dialect` and `supports` (`boundary.rs:93-109`, `settings.rs:47-53`) refuse a mismatch at `host.rs:73`. `run.rs:37-43` and `:249-251` refuse it again, and those arms are unreachable through the host.
- GitHub's `Reconcile { adopted }` is refused at runtime when `adopted` is non-empty (`run.rs:241-244`). `Reachable { installation: Option }` is checked in both directions (`run.rs:228-235`).
- `reachable.rs:29-44` mixes `Github*` states into one enum. Its GitHub constructor (`:86-101`) fills `admin_name: []` and `listing: Token::Admin`, which mean nothing for GitHub. Every other GitHub plan lives under `github/`.
- **Fix:** split `Plan` into Forgejo and GitHub plans, and move GitHub's picker to `github::reachable` (about 180 lines move). About 60 lines go.

**9. Given values and limits are parallel per dialect** (Economy)
- `limits.rs:35-40` and `github/limits.rs:83-88` hold the same deadline and backoff relation check.
- `settings.rs:55-68` maps both kinds into `Times` with identical arms.
- `Place` and `Refused` are each defined twice.
- **Fix:** one `Times { deadline, backoff, backoff_max }` with one check, embedded in both `Limits`. About 40 lines go.

**10. An item is validated in three layers** (Complexity)
- The host turns any read that does not fit its item into `None` (`run.rs:298-306`).
- Five plans call `reconcile::normal` in their `step` anyway (`reachable.rs:470`, `reconcile.rs:350`, `check.rs:130`, `create.rs:342`, `adopt.rs:129`).
- The plans then check again: `reconcile.rs:219` tests `!token.is_empty()`, which is dead after `normal`; `paste.rs:127` and `github/reconcile.rs` re-decode `app_id` and `app_key`.
- **Fix:** give the check one owner.

**11. Naming collisions and test layout** (Naming, Organization)
- Colliding names: `Need` and `Event` in both `reconcile` and `host::boundary`; `Limits` ×3; `Given`, `Refused` and `Place` ×2; `Reconcile` ×2; and a local `enum Token` in `reachable.rs:48` beside `skein_lib::Token`.
- They force aliases such as `reconcile as github_reconcile` (`run.rs:17`) and `tests::host as fixtures` (`github_host.rs:8`), next to a real `tests::fixtures`.
- Failure subjects `b"key"` (21 sites), `b"id"` (13) and `b"installations"` (12) are not in the design's subject list (forge-setup.md:315-319).
- Tests under `src/tests/` are mounted from three parents via `#[path]` (`host.rs:5,11`, `states.rs:113`), and `host.rs:428` carries a `#[path]` that changes nothing. Both `src/tests/host.rs` and `src/host/tests.rs` exist.

**12. Fixtures are retyped across tiers** (Economy)
- `tests/forge/setup/src/fixtures.rs:200-275` repeats the crate's `src/tests/host.rs:48-103` and `github_fixtures.rs:59-84`. That is 13 identical limit fields, the same GitHub given values, the same `crypto` seed, and the same `TAKEN` constant.
- **Fix:** a small `testing/` fixtures crate. About 120 lines go.

**13. Rule slips, in files that otherwise follow the long form** (Cleanliness)
- `let-else` or `if let` on the crate's own enums: `access.rs:70` (on a tuple), `:257`, `:261`; `orgs.rs:46`; `create.rs:339`, `:350`; `host.rs:69` (on a tuple); `reconcile.rs:417`.
- `call.rs:11-20` hand-writes `From` impls, which section 10.3 rules out.

**14. Conformance and throwaway** (Economy, Taste)
- The throwaway's `instance.rs:318-389` re-implements the JSON parse and two-pass write that `skein-forge-conformance/src/json.rs:15-46,226-242` already provides.
- Shim modules: the conformance's `json.rs` is a bare re-export, and `write.rs` wraps `Secrets` in a `Deref` newtype and classifies a secret by its length (40 or 64) at `:15-24`.
- `cases.rs:248-249` branches on row labels (`starts_with("branch-protection")`, `== "raw-file-range")`); the rows should carry the token and range instead.
- About 104 fact checks re-read the masked transcript they just wrote, by its string name (for example `tokens.rs:34` and `:101`), even though the exchange was in hand. This may be deliberate, so the facts judge the committed output (unsure).

## 5. Cross-area suspicions

- **skein-forge-connection:** the shared transport of finding 1. Its `settings.rs` and `limits.rs` likely carry the rest of the duplication.
- **skein-forge:** each dialect's `request::Method` should probably map to `skein_http::Method` once, there. Its token scopes are spelled out separately in `reconcile/tokens.rs:79-85`, the world tests and the conformance's `tokens.rs:12-14`; the dialect could name them.
- **tests/forge/github:** it likely repeats the conformance shims (it has its own `mask.rs`). Unverified.
- **Fake-forge step tests:** they likely restate the Forgejo facts on their own, since no check ties the fake to `facts.txt`.

## 6. Rule costs

The exhaustive "reject the rest" arms cost about 450 production lines, roughly 6%:
- about 53 `Event` arms, such as `Event::Begin | Event::Read(_) | … => Unexpected`;
- 34 `Reply` lists of 5 to 8 variants;
- the 27-variant `Document` match in `account()` (`reconcile.rs:308-347`) and three 20-variant `Row` lists.

Splitting `Reply` per dialect (finding 6) recovers part of this within the rules.
---

# 08: fake-forge

**Fake forge code-quality review**

All paths are relative to the tree root `<clone>/skein/`.

## 1. Scope read

- **Read closely:**
  - Design docs: fake-forge.md §1–4, 9, 10.1–10.2; forge.md §2.1, 4.1, 11; testing-strategy §4, 10; programming-model §1, 5.4, 10.
  - Domain: `lib.rs`, `boundary.rs`, `domain.rs` (head and entry points), `dispatch.rs`, `ci.rs` report, `next.rs`, `faults.rs`, the type list in `api.rs`, the `tests.rs` harness and about 25 of its tests, the head of `tests/promises.rs`.
  - Protocol: `lib.rs`, `server.rs` (most of it), `server/git.rs`, `routes.rs`, `tally.rs`, `service.rs` head, signing in `delivery.rs`, `tests/observations.rs`.
  - GitHub face: `routes.rs`, `reads.rs` head, `writes.rs`, `git.rs`, `server.rs`, `service.rs` (request, finish, valid), `answers.rs`, the hook part of `delivery.rs`, `logs.rs` signing, the fixtures harness, `write_pulls.rs`, `auth.rs`, the memory tests.
  - `tests/fake-forge`: the memory suite and the GitHub fixtures.
  - I also checked transcript use in skein-forge's tests.
- **Skimmed (structure and signatures only):** `setup/*`, `response.rs`, `answers.rs`, `pages`, `manifest`, `installations`, `rates`, `queue`, the body of `promises.rs`, `pulls`, `issues`, `store`, and most test files.

## 2. Scores

1. **Cleanliness 3.** Comments mostly earn their place. There is noise: unordered `mod`/`use` blocks, a misplaced doc comment, chronological names ("task's", "next").
2. **Organization 3.** The domain's module layout is good. In the faces, Forgejo routing lives inside `server.rs`, the protocol crate doubles as the shared kit, and memory tests sit in three places.
3. **Taste 3.** The code is careful and bounded. But the two faces follow the house rules inconsistently, there are dummy template requests, and some JSON is patched as text.
4. **Naming 3.** The domain matches the design's vocabulary. Across the faces `api`, `a`, `fake` and `domain` swap meanings from file to file, and `mutations` and `next` are misnamed.
5. **Primitives 2.** The domain boundary is clean. But `Answer` is one flat enum, `Hook` is a bag of `Option`s, connection state lives in `Option` fields, the faces route replies two different ways, and route enums mirror the dialect's.
6. **Elegance 3.** The domain follows the problem. The GitHub face's Service/Outcome shape is good; the Forgejo face does not share it.
7. **Economy 2.** About 2.5–3.5k production lines and about 1.5k test lines are recoverable (estimates below).
8. **Complexity 3.** Mostly just enough. Some layers only forward, and some state could be derived.

## 3. Strengths

- **The domain is a real step crate.** It depends only on skein-lib, has its own vocabulary, and states its contract in `boundary.rs`. Module docs cite design sections, arithmetic is checked, and matches are exhaustive.
- **The GitHub face's `Service::request` returns `Outcome::{Reply, Admit, Call}`** (`service.rs:207`). The service stays pure and testable without HTTP, and `Server` only frames bytes. This is the shape to converge on.
- **`domain/src/tests/promises.rs` is a model of testing-strategy §10.** It has named fixtures (`CLIENT`, `REPOSITORY`, `OBJECTS`), one small builder per observation, and one behaviour per test.
- **The Forgejo face does its own reading of Forgejo's semantics** rather than borrowing the client's: paging (`routes.rs:46 page`), quirks and scopes. skein-forge checks the server-side encoders against real transcripts (`crates/skein-forge/tests/suite/forgejo_items.rs:35`, shape-subset tests).

## 4. Findings

**Answers to the area questions, briefly:**
- **Is 45k proportionate?** Production is about 8.3k / 8.6k / 8.5k lines (domain / Forgejo / GitHub), tests about 5.3k / 7.5k / 7.5k. For roughly 125 routes, setup, the git route, the observation codec, the App/JWT/installations, the merge queue and CI, that is defensible. Findings 2–9 recover about 12% of production code.
- **Is the split clean?** The domain is clean, apart from `RateKey::Installation` (`faults.rs:19`) and the queue's `gh-readonly-queue` spelling, which the design states.
- **Does the fake's domain duplicate skein-forge's api?** Yes, about 25 type names (Op, Read, Write, Answer, Error, Pull, Review, Status, Verdict, Check, Entry, ...). This is deliberate (testing-strategy §4) and differs where it matters (u64 commit counts, item filters, wiki, scenario setup). Keep it. The real cost is the naming confusion in the faces (Finding 9).
- **In-crate tests:** see Findings 10 and 11.

### 1. Coupling to skein-forge: a deliberate exception that is not written down, and tighter on the GitHub side (Primitives)

- **What it is.** Most of the ~40 Forgejo sites use the dialect's server half: `request::Request`, `document`, `server::Answer`, `limits`. forge.md §4.1 ("the server's side of each document, for the fake forge") and fake-forge.md §4 and §10 ("decoded by the server side of skein-forge") make this deliberate. But testing-strategy §4 still says "No shared types" with no exception noted.
- **How far it is policed.** At the document level it is mitigated: real Forgejo requests are decoded by `decode_route` (`crates/skein-forge/tests/common/forgejo.rs:432`), and server encoder shapes must be a subset of the transcript shapes. Facts beyond documents are not checked on the fake: fake-forge.md §9 says Forgejo's conformance does not run against it.
- **Harmless parts.** `id::unsigned`, `time::write` and `document::Body` are generic helpers filed under `forgejo::`. GitHub code uses them 21 times.
- **Agreement by construction (the GitHub face).**
  - It builds its `Link: next` with the client's `request::encode` (`github/src/delivery.rs:268`).
  - It signs deliveries with `github::delivery::sign`, which skein-forge itself labels "The fake's header value" (`github/src/delivery.rs:425`; `crates/skein-forge/src/github/delivery.rs:35`).
  - Its routes carry the client's call-plan type `plans::tables::Method` (`writes.rs:26`).
  - Its tests drive the client's plans (`tests/write_support.rs:44`).
  - These are exactly testing-strategy §4.1's "two sides written together".
- **Fix:**
  - Record the exception in testing-strategy §4, scoped to documents.
  - Give the GitHub face its own signer and Link builder (about 30 lines).
  - Move `unsigned` / `time` / `Body` into lib or a dialect-neutral module.

### 2. Two faces, two connection machines (Organization, Economy)

- `protocol/src/server.rs` and `github/src/server.rs` repeat the same skeleton: the identical `State` enum (8 variants); `start`, `up`, `up_with_domain`, `up_on`/`up_inner`, `resume`/`resume_with_domain` (with the same bare `0..8u32` loop), `close`, `closed`, `next`, `http_down`; head capture; and `respond` + `room`/`reply_room` (`protocol:852-899` vs `github:585-655`).
- They then diverge on reply routing. Forgejo keeps a `Service::calls` slab plus `target`; GitHub uses `ReplyTo::new(server.owner)` and `Server::pending`.
- **Fix:** a shared `connection` module in the protocol crate, which GitHub already depends on. Move Forgejo's routing out of `server.rs` into a `Service::request -> Outcome`, as GitHub has.
- **Saving:** about 250 lines now, about 400 with the convergence.

### 3. Connection state lives in `Option` fields, against §5.4 (Primitives, Complexity)

- Protocol `Server` (`server.rs:42-66`) has 8 `Option`s that only mean something in some states (`decoder`, `git_decoder`, `request`, `call`, `listing`, `reply`, ...), beside a `State` enum whose variants carry no data.
- GitHub (`server.rs:35-51`) does the same with `pending`, `admission`, `reply`, `call`.
- As a result `close` clears fields by hand (`protocol:368-395`, `github:297-321`).
- **Fix:** `State::Body { decoder }`, `Waiting { call, request }`, `Streaming { reply, offset }`, with the `mem::replace` idiom. Size neutral, but whole classes of stale-field states go away.

### 4. The GitHub face mirrors the dialect's `Request` (Economy)

- `routes.rs:126-196` translates `Request::X {owner, name, ..}` into `reads::Read::X` / `writes::Route::X` with the same fields (`reads.rs:32-56`, `writes.rs:19-104`).
- Then `writes::Route::request()` (`writes.rs:105-128`) rebuilds the original `Request`, and `path()` / `row()` re-match about 20 variants just to get `(owner, name)`.
- **Fix:** carry `Request` itself, plus `fn path(&Request)` / `fn contents(&Request)`. Better still, give the dialect's GitHub request a `repo` field, as Forgejo's `at: Repo` already is.
- **Saving:** about 250 lines.

### 5. The Forgejo face dispatches its 70-variant `Request` about 10 times (Economy, Primitives)

- Exhaustive catch-all lists:
  - `routes::repository`, `routes::op`, `missing_scope`, `table` (`server.rs:670-843`);
  - `response::select` / `follow`;
  - three in `answers`;
  - `setup` twice.
- That is 668 lines of `| Request::X { .. }`.
- **Fix:** classify once at the entrance, e.g. `enum Served { Table(..), Setup(..), Listing(Page), Domain { repository, op, select, scope } }`.
- **Saving:** about 400 lines.

### 6. The domain's `Answer` is flat, so the two faces each handle it differently (Primitives, Taste)

- `Answer` has 34 variants for three `Op` families. Every consumer expecting one variant must dispose of the other 33.
- The Forgejo face lists them all: 8 lists, about 230 lines (`server/git.rs:177-212`, `response.rs:59`, ...).
- The GitHub face uses `let a::Answer::X = answer else` about 50 times (`writes.rs:365-434`, `merge.rs`, `logs.rs:129`), which §10.2 forbids.
- **Fix:** make `Answer` follow `Op`: `Answer::Git(GitAnswer) | Write(WriteAnswer) | Read(ReadAnswer)`. A git consumer then matches 3 arms, not 34.
- **Saving:** about 200 lines, and the rule breach goes.

### 7. The GitHub face breaks other house rules the Forgejo face keeps (Taste)

- Lifetime-parameter types: `service::Request<'a>`, `pages::Page<'a>`, `pages::Route<'a>`, `answers::Body<'a>`, `Detail<'a>`.
- Eight hand-written `Debug` impls (`service.rs:145,192`, `rates.rs:32`, `app.rs:35,73`, ...). Forgejo instead uses `#[expect(missing_debug_implementations)]`.
- **Fix:** either state the exception for faces or align. Pick one, deliberately.

### 8. The domain's `Request::Hook` is a tag plus a bag of `Option`s (Primitives)

- `boundary.rs:40-50`: `change: Change` beside `number`, `branch`, `commit`, `by`, `body`, `ended` and `queue`, all `Option`.
- Consumers re-prove which combinations are possible: 14 `ok_or(Problem::Config)?` in `github/src/delivery.rs:469-541`, and the same 8-tuple unpacking in `protocol/src/delivery.rs:77`.
- **Fix:** put each change's data in its variant (`Change::Push { branch, commit: Option<u64>, by }`, `Change::Job(Ended)`, ...).
- **Saving:** about 60 lines, and the impossible states disappear.

### 9. Helpers reimplemented across the area (Economy)

- **Lowercase hex, 4 copies:** `protocol/delivery.rs:176`, `github/logs.rs:173` (a byte copy of protocol's `sign`, though GitHub depends on that crate), `github/reads.rs:588`, `domain/queue.rs:47`.
- **Decimal parse, 3 copies:** `github/app.rs:251`, `github/pages.rs:454`, `forgejo::id::unsigned`.
- **Byte concatenation, 6 copies:** `promises.rs:883`, `actions.rs:421`, `pages.rs:550`, `service.rs:880`, `setup.rs:336`, `routes.rs:52`.
- **Loopback address parse, 2 copies:** `protocol/delivery.rs:189`, `github/delivery.rs:459`.
- **Git route Call→Op translation and the 33-line answer list, 2 copies:** `protocol/server/git.rs:94-207`, `github/git.rs:123-230`.
- **Cue judgment, 2 copies:** `ci.rs:111-135`, `dispatch.rs:144-166`.
- **Fix:** add `hex`, `Decimal::parse` and `bytes::concat` to lib; give the protocol crate's `git` module `op(call)` and `answer(result)`, and make its `Limits` parameter narrow, so GitHub no longer fakes `calls: 1, connections: 1` (`github/git.rs:244`).
- **Saving:** about 200 lines.

### 10. Naming (Naming)

- **Aliases change meaning by file.** In `github/reads.rs:10-13`, `a` is the fake's api and `api` is skein-forge's. In `github/git.rs:7` and `protocol/server.rs:5`, `api` is the fake's. The domain crate is `fake` in 8 files and `domain` in 3. With the same type names on both sides (`Verdict`, `Commit`, `Check`, `Permission`), a reader cannot trust `api::Verdict`.
- **`mutations.rs` holds only rulesets.** Name it `rulesets`.
- **`next.rs` is a grab-bag named by history.** Its header reads "Reads added for the connector (protocol/forge.md…)", a doc path that does not exist. It also holds a write (`update`, `rebase`). Split it into `reads`, `actions` and `pulls`.
- **`skein-fake-forge-protocol` is the Forgejo face but also the shared kit.** Name it for Forgejo, or move the shared parts into named modules.

### 11. Tests retype fixtures and tell stories (Economy, testing-strategy §10)

- **Full domain `Config` literals (26 fields):** 5 in this area (`domain/tests.rs:54`, `tests/fake-forge/tests/suite/memory.rs:45`, protocol `fixtures.rs:505`, GitHub `fixtures.rs:512`, `tests/fake-forge/tests/github/fixtures.rs:124`), about 12 repo-wide. `Limits` is retyped likewise, and the `Protection` literal about 20 times. **Fix:** a `pub const` default in the domain, used as `..CALM`.
- **`observation_lines_round_trip`** (`tests/observations.rs:28-1285`) repeats `forward: None` 88 times and `boxed(b"org/repo")` 89 times. An `observed(..)` helper and a `REPO` constant save about 170 lines. `assert_record` exists only because `Recorded` lacks `PartialEq` (`promises.rs:50`).
- **GitHub tests:** `b"acme/widgets"` appears 73 times, `Token::new(77)` 25 times, and users are bare numbers (`1`, `31`, `32`, `if id == 2`, `fixtures.rs:177`). The `Harness` has five telescoping constructors (`fixtures.rs:126-200`). The pull-validation block is pasted twice (`write_pulls.rs:44-55` and `70-81`).
- **Domain tests are many-behaviour stories:** `a_push_fast_forwards_or_creates_and_is_otherwise_rejected` (`tests.rs:907`, 13 behaviours), `what_is_missing_is_refused` (`tests.rs:864`, 9). These are probably older code ("copies already landed stay").
- **Saving:** about 1.2k lines.

### 12. Memory tests use three mechanisms (Organization)

- `tests/fake-forge` (domain and GitHub, counting allocator).
- `crates/skein-fake-forge-protocol/tests/suite/memory.rs` (pulls in-crate fixtures via `#[path]` and `#[expect(dead_code)]`).
- `github/src/tests/memory.rs` (hand-summed `size_of_val`, and it ends with unrelated removal and id-sequence asserts).
- `tests/fake-forge/tests/github/fixtures.rs` (311 lines) retypes the GitHub fixtures with slightly different numbers.
- **Fix:** one place and one mechanism.

### 13. Cleanliness nits

- `protocol/src/lib.rs:6-7` has `use` lines between `mod` lines, and `mod tests` sits mid-file.
- Both faces' `tests.rs` `mod` lists grew by appending, with stray blank lines (`protocol/src/tests.rs`, `github/src/tests.rs:42-74`).
- `github/server.rs:81` puts the doc for `take_head` on `head_pending`.
- `tally.rs:3` has `ROUTES = 81` with no derivation (the design says 74).
- `git.rs:216` `store_answer` only forwards.
- `answers.rs:80-86` patches the encoder's JSON with `strip_suffix(b"}")`.
- `service.rs:588` builds a dummy `Request::Enqueue { id: [], head: [0; 32] }` as a template.
- `routes.rs:1` says "The task's route subset".

## 5. Cross-area suspicions (unverified beyond my area)

- **skein-forge:**
  - `github::delivery::sign` exists for the fake.
  - The server-side GitHub `Request::Merge` carries the client's `plans::tables::Method`.
  - Forgejo documents embed vocabulary types (`document.rs:104,554` use `api::EntryKind` / `TreeEntryKind`).
  - Generic helpers live under `forgejo::`.
  - GitHub's `Request` uses flat `owner, name`, while Forgejo has `at: Repo`.
- **skein-lib:** it lacks hex, decimal parse and concat. `marker::hex` in skein-forge suggests copies elsewhere.
- **Fake domain `Config` and `Limits` literals** are retyped in `tests/forge/{machine,github,client}`, `tests/fake-peers`, and the `skein-forge-connection` tests.
- **skein-fake-peers** (`src/forge.rs`, `forge/layer.rs`) probably hosts the two faces through their different reply-routing schemes.
- **`let Enum::X = .. else`** on own enums may recur in other face crates.

## 6. Rule costs

- **No `_` arm:** about 900 lines of variant lists across the faces. Findings 5 and 6 would cut that to about 300.
- **No closures:** the GitHub face's 50-variant `answers::Body` (about 100 lines) exists only to hand a measured `Writer` to `server::x`. The measure-then-write pattern also calls each field writer twice (`answers.rs:114-123`, `158-170`).
---

# 09: llm

I read the main types, the client and both dialect paths, llm-connection's component and connection, the fake crates' state machines and the test harnesses closely, and sampled the test suites. The two largest costs are two OpenAI request encoders kept in lockstep, and output and outcome types copied once per dialect and again at crate level. About 1,000 lines of production code and 800–1,200 lines of tests could go without breaking any house rule.

All paths below are under `<clone>/skein`.

## 1. Scope read

**Read closely.**
- skein-llm: `types.rs`, `lib.rs`, `dialect.rs`, the production part of `translate.rs`, `openai/common.rs`, `openai/request.rs`, `client.rs` (state, fields, entry points), `anthropic/request.rs` (validate and charge), the receive/fail/ready parts of both `response.rs` files, `derive.rs`, `document.rs`, `replay.rs`, `README.md`.
- skein-llm-connection: `call.rs` (state and `route`), `component.rs` (state, `down`, `has_work`, `next_deadline`), `deadlines.rs`, `endpoint.rs`, `boundary.rs`.
- skein-fake-llm-domain: `domain.rs`, `lib.rs`, parts of `api.rs`.
- skein-fake-llm-protocol: the start of `documents.rs`, `Service`/`Server` in `provider.rs`, `credentials.rs`.
- tests/llm: `src/lib.rs`, the `Exchange` in `src/fake.rs`; samples of `suite/client.rs`, `suite/archives.rs`, `openai/tests.rs`, `anthropic/request_tests.rs`, `client/tests.rs`.
- tests/llm-connection: `world.rs`, `limits` in `simulated.rs`, the start of `suite/failures.rs`.

**Skimmed.** The bodies of the decoders and encoders, `peer.rs`, `json.rs`, `filter.rs`, `respond.rs`, `cache.rs`, `pieces.rs`, `inflate.rs`, most suites, the fixtures.

## 2. Scores

| Criterion | Score | Why |
|---|---|---|
| Cleanliness | 3 | The code is tidy. Against that: about 340 lines of "Contract:" boilerplate in the fake crates, stale route docs, doubled doc summaries, unused parameters. |
| Organization | 3 | Shared dialect code lives under `openai::`. `translate.rs` sits at the crate root but only serves OpenAI. Code that only the fake uses ships in the production crate. |
| Taste | 3 | The bounded arithmetic and derived limits are careful. The main state machines are flags plus fields, and several types have chains of forwarding constructors. |
| Naming | 3 | `Cap`, `Phase`, `Evidence` and `Report` are precise. But "codex"/"ChatGPT" names sit on code that serves the public route, and `openai::Limits` is the neutral limits type. |
| Primitives | 3 | `Cap`/bound, the `Replay` envelope and `Declared → Limits` are good. Against that: three `Output` enums, two of `Stop`/`Usage`/`Failure`, two OpenAI request types, and two error types mapped onto each other by hand 32 times. |
| Elegance | 3 | The derivation and selective decoding follow the problem. The client's and the connection's state does not follow the shape in 5.4. |
| Economy | 2 | About 1,000 removable lines in production; test setups typed out again and again. |
| Complexity | 3 | The withdrawn Codex route is still threaded through everything (261 mentions in the area), there's a test-only second encoder, and constructor chains. |

## 3. Strengths

- `crates/skein-llm/src/derive.rs:111-148`: per-dialect constants, each with a comment deriving it from the archived fixtures, plus a `Violation` that names the relationship that broke. This is the model for "named sizes".
- The failure vocabulary is typed: `Cap` with its `bound`, `Evidence`, `Report`, `Phase`. Tests assert limits by name (for example `tests/llm/tests/suite/client.rs:868`).
- Transcripts are kept as records, with provenance in `README.md` and `meta.json` files. They are driven through the codec (`openai/tests.rs:515`) and through the whole client (`suite/archives.rs:35`).
- skein-llm-connection reuses skein-llm's vocabulary instead of wrapping it. `EndpointError` (`endpoint.rs:49-78`) names each capacity relationship. `deadlines.rs` is a small table with total functions over it.

## 4. Findings, ranked by payoff

**F1. Two OpenAI request encoders and an owned request type, kept for the fake and as a test oracle.** *Economy, Primitives.*
- `translate.rs:49-199` writes a `Prompt`. `openai/request.rs:98-212` writes an `openai::Request` with the same fields in the same order.
- `translate.rs:249-302`, `571-628` and `685-712` are a `#[cfg(test)]` converter from `Prompt` to `openai::Request`, used only to compare the two encoders (`client/tests.rs:572-575`, `translate.rs:858`, `900`, `941`).
- Anthropic already decodes into the neutral `Prompt` (`anthropic/peer.rs:13-17`). OpenAI decodes into the owned type, and the fake's `documents.rs` consumes `openai::Input`.
- **Fix:** have `openai::decode_request` return `Prompt` plus the peer's extras, as Anthropic does. Delete `openai::{Request, Input, Tool, Role, encode_request, measure_request}` and the test-only converter. The archived bodies already check the encoder (`translate.rs:930`).
- **Size:** about 350 production lines and about 100 test lines.

**F2. The output vocabulary is copied three times, and `Stop`/`Usage`/`Failure` twice.** *Primitives, Economy.*
- `openai::Output` (`openai/response.rs:123`) and `anthropic::Output` (`anthropic/response.rs:74`) are identical apart from `Part`; `dialect::Output` (`dialect.rs:15`) is a third copy.
- `openai::{Stop, Usage, Failure}` (`common.rs:146-193`) duplicate `types.rs:137-158` and `304-331`, doc comments included. The identity mappings are at `translate.rs:795-827`.
- `codex_outputs` and `anthropic_outputs` (`dialect.rs:176-232`) are near copies, as are the decoders' `received` and `fail`.
- Anthropic re-exports OpenAI's `Stop`/`Usage`/`Failure` (`anthropic.rs:8`), so these are not separate per-dialect vocabularies anyway.
- **Fix:** have the codecs push `dialect::Output` with `Block` already built, emit the crate's `Stop` and `Usage`, and have `classify` return the crate's `Failure`. Share one detail helper for failures.
- **Size:** about 200 lines.

**F3. Request admission is written once per dialect, with hand-written error remapping.** *Economy.*
- `validate` at `translate.rs:304-353` and at `anthropic/request.rs:37-99` share the same opening checks: empty model, tool count, history count, `validate_choice`, charging the names, model, instructions and effort.
- `charge`/`charge_text`/`charge_json` (`translate.rs:438-484`) are mirrored by `charge`/`text`/`json` (`anthropic/request.rs:215-253`).
- The mapping from `writer::Refusal` to a cap is written 7 times: `translate.rs:446`, `481`, `678`, `756`; `common.rs:414`; `json.rs:124`; `anthropic/request.rs:259`.
- `append` exists twice (`common.rs:386`, `anthropic/response.rs:542`).
- The four-line `Err(error) => return Err(decode(error))` appears at 32 sites.
- **Fix:** one admission module with a per-dialect hook for block checks. Write the inner helpers in `DecodeError` so `?` works, and convert once at `measure_request`. No `From` impl is needed.
- **Size:** about 200 lines.

**F4. The client and the connection hold per-state data in fields next to a state enum that carries none.** *Primitives, Elegance; breaks programming-model 5.4.* This is the largest structural change.
- `client.rs:245` `State` carries no data. The `Client` (`client.rs:275-313`) holds `call: Option<…>`, `upload`/`upload_offset`/`upload_end`, `decoder: Option`, `content: Option`, `error`, `status`, `asked`, `close_sent`, `sse_closed`, plus `Outcome` and `EventData`.
- There are 16 `state == State::…` checks (for example `:682`, `:693`, `:752`, `:768`), no `mem::replace` on `state`, and `expect` calls standing in for invariants the type could hold (`:683`, `:769`).
- The same shape appears in `skein-llm-connection/src/call.rs:28-70` (`Phase`, two `Option<Token>`s, three flags, `Control`) and in `skein-fake-llm-protocol/src/provider.rs:272-300`.
- By contrast, `skein-fake-llm-domain/src/domain.rs:288` and `:432` follow the idiom.
- **Fix:** move `error` into `ErrorBody`, `call` into `Prepared`, `upload*` into `Head`, and `decoder`/`content`/`asked` into `Streaming`. Apply one `follow_state` after each transition.
- **Size:** roughly neutral, but much clearer.

**F5. Every harness re-wires the fake peer by hand.** *Organization, Economy (tests); testing-strategy §4, "one fake, every tier".*
- `provider::Service`, `Server` and the fake `Domain`, plus the routing from domain to protocol, are assembled by hand in:
  - `tests/llm/src/fake.rs:181-216` and `546-760`
  - `tests/llm-connection/src/world.rs:110-278`
  - `tests/llm-connection/tests/suite/fake_protocol.rs:66-78`
  - `tests/llm/tests/suite/fake.rs:993-1008`
  - `crates/skein-fake-peers/src/llm.rs:49-110` and `402`
- **Fix:** add a composed `Peer` step machine to skein-fake-llm-protocol. Harnesses keep only their byte pump and their observations.
- **Size:** about 300–400 test lines.

**F6. "Contract:" boilerplate in the fake crates.** *Cleanliness.*
- There are 171 `/// Contract: …` lines, 156 of them the identical "fake-llm.md, sections 2–5; programming-model.md, section 4.4", each with a blank `///` line before it (168 of those).
- They sit on every enum variant and trivial accessor: `api.rs:220-240`, `domain.rs:380-400`. `api.rs` alone has 65.
- **Fix:** keep the module-level contract, which `lib.rs:22-24` already has, and delete the rest.
- **Size:** about 340 lines.

**F7. Test fixtures default to the withdrawn Codex route.** *Complexity.*
- `tests/llm/src/lib.rs:32-65`: `limits()` and `call()` use `Endpoint::codex()` with an account id, and `anthropic_call` uses the withdrawn bearer.
- `call(N)` has 131 uses against 23 for `public_call`. The area mentions Codex 261 times, while `types.rs:15`, `:391` and `:418` say "removed by the second part".
- **Fix:** make the public route and an Anthropic key the defaults now, and gather the Codex stories into one file, so removing Codex later is mostly deleting that file.

**F8. Dialect-neutral code is placed and named as OpenAI's.** *Naming, Organization.*
- Anthropic's code takes `&openai::Limits` everywhere.
- `openai::classify` handles Anthropic (`common.rs:262`, `327-357`), and `RateLimit::observe` reads `anthropic-ratelimit-*` headers (`common.rs:217-221`).
- `lib.rs:27-28` exports `openai::DecodeError as DocumentError`.
- `translate.rs` sits at the crate root ("Bridge … Codex documents") but serves only OpenAI.
- `codex_outputs` (`dialect.rs:176`) and `validate_codex` (`translate.rs:42`) also serve the public route, and the decoder's failure details say "ChatGPT stream" (`openai/response.rs:234-316`).
- **Fix:** move `common.rs` and `json.rs` into a `native` module, move `translate.rs` to `openai/prompt.rs`, and rename `codex_*` to `responses_*`.

**F9. Stale and doubled docs.** *Cleanliness.*
- `lib.rs:3` says the crate "Supports ChatGPT's Codex … subscription routes"; both are withdrawn.
- `README.md:19-25` and `50-56` present Codex as the main route and Anthropic's default as Bearer with `oauth-2025-04-20`.
- A "`/// Bounds X.`" line is stacked on top of an existing summary at `common.rs:58`, `78`; `anthropic.rs:22`; `client.rs:1571`; `document.rs:27`; `replay.rs:20`; `openai/response.rs:1241`; `anthropic/response.rs:756`.
- `skein-fake-llm-domain/src/lib.rs:16-18` has two crate-doc blocks with a blank line between them.
- **Size:** about 40 lines.

**F10. Tests break the test-shape rules (testing-strategy §10).**
- **Rows branch on labels:**
  - `suite/client.rs:285-296` uses `match field {0 => …}` inside the loop.
  - `suite/client.rs:842-872` uses `match which` and `if bound == edge … else …`, two behaviours in one body.
  - `openai/tests.rs:569`, `573`, `586` branch on `if scenario == "…"`.
  - `client/tests.rs:572` branches on the provider.
- **Compound names:** 75 of 145 skein-llm tests and 94 of 216 tests/llm tests contain "_and_" (for example `openai/tests.rs:661`).
- **Retyped setups:**
  - A roughly 30-line llm-connection `Limits` literal appears in 7 files: `crates/skein-llm-connection/src/tests.rs:8`, `tests/llm-connection/src/world.rs:85`, `src/simulated.rs:21`, `suite/failures.rs:15`, `fake_protocol.rs`, `memory.rs`, `protocol.rs`. Some copies use 19_000 and others 32_768, with no derivation.
  - Five separate fixture loaders: `openai/tests.rs:100`, `archives.rs:7`, `derive.rs:69`, `translate.rs:932`, `selective.rs:17`.
- **Bare sizes:**
  - `TINY_DECL` (`lib.rs:9-18`): 171, 342, 174, 684, with no derivation.
  - `(Cap::Answer, 101)` and `document + 8` (`client.rs:840-846`).
- **JSON edited as text with no check that it changed:** `anthropic/request_tests.rs:498-507`.
- **Fix:**
  - Rows carry a setter closure.
  - Split the bound and one-past-it cases into separate tests.
  - One fixture loader in skein-llm-world.
  - Reuse `world::limits(calls)` with struct update.
- **Size:** about 250 lines.

**F11. Chains of forwarding constructors.** *Complexity.*
- The fake `Domain` has new → scripted → try_scripted → configured → played → played_by_input (`domain.rs:304-372`), each adding an empty default.
- `Exchange` has six constructors (`tests/llm/src/fake.rs:263-405`).
- The same pattern, on a smaller scale: `Client::prepare_with_reasoning_drop` (`client.rs:467`) and `Component::admit_with_reasoning_drop` (`component.rs:182`).
- **Fix:** one constructor that takes a setup struct with an `EMPTY` constant.
- **Size:** about 100 lines.

**F12. Small rule breaks and dead bits.** About 60 lines in total.
- `impl From<Error> for Failure` in a step crate (`skein-fake-llm-domain/src/api.rs:478`). Hand-written trait impls are out; use `Failure::of(error)`.
- Lifetime-parameterised structs in a step crate: `credentials.rs:52` (`Fields<'a>`) and `inflate.rs:134` (`Input<'a>`).
- Unused parameters: `respond.rs:507` (`_query`) and `call.rs:361` (`_owner`).
- No-op stubs `has_ready`/`ready` in `anthropic/response.rs:166-173`.
- `component.rs:584-619` reimplements `deadlines.rs:159` `earlier` twice.
- `call.rs:376-420` repeats the same terminal tail three times.

## Area questions

- **Per-provider shape.** The core is shared: one `Client`, an enum dispatch in `dialect::Decoder`, shared JSON, limits and `classify`. The two stream decoders differ for good reason: Responses reorders items by index, while Anthropic emits one block at a time. The edges are copy-and-adapt: admission (F3), outputs and outcomes (F2), failure details, and the duplicated OpenAI request (F1).
- **Fake coupling.** Importing `Credential` and `Provider` is minor, but it costs something:
  - The fake keeps two provider enums, its own `documents::Provider` and `skein_llm::Provider` as its route, and `route`/`authorize` match on `OpenAiCodex`, so withdrawing Codex will ripple into the fake.
  - The real coupling is the codec. The fake's request decoders and event encoders are skein-llm's. That is roughly 1,100 lines that only the fake uses (`anthropic/peer.rs`, `openai/response.rs:971-1346`, OpenAI request decoding and the owned encoder), and they ship in every service.
  - fake-llm.md §1 and §4 sanction this, and the archived transcripts reduce the risk of client and fake sharing one misreading. Still, it conflicts with testing-strategy §4 ("no shared types"). Moving the peer-only codecs into skein-fake-llm-protocol, with the transcripts as the shared referee, would fit the rules better.

## 5. Cross-area

- **What makes llm-connection small.**
  - The client already owns HTTP, SSE and JSON, measurement, failure classes, evidence and connection reuse (`next_call`, `Reusable`). skein-tls owns TLS and skein-io owns sockets.
  - Deadlines are a flat table of `Option<Time>` per phase.
  - There is no retry, no DNS and no credential logic; the credential passes straight through.
  - Its boundary reuses skein-llm's `Prompt`, `Block`, `Failure` and `Evidence` unchanged.
- **Its shape.**
  - `Component` holds the pool: a FIFO for memory admission, a waiting list, a slot list beside the slab, and a round-robin cursor.
  - Each `Connection` holds a socket token, an optional TLS client, the LLM client, a deadline table and four routing queues.
  - A bounded `route` pass moves work between the client, TLS and io.
  - Weak points: the flag-and-field state (F4), and 12 nested `for slot in &self.slots` / `match` loops that `if let` chains could flatten.
- **Unverified suspicions:**
  - skein-forge-connection (`connection.rs:9` `Phase`) and skein-oauth-accounts (`component.rs:100` `State`) may share F4's flag-and-field state shape.
  - Fake peers are hosted by hand for the forge too (`crates/skein-fake-peers/src/forge.rs`, `tests/forge/machine/src/joint.rs:334`), which would be the F5 pattern again.
  - skein-oauth and skein-fake-oauth may share codecs the same way.
  - The "Contract:" boilerplate does not appear in the forge or oauth fakes.

## 6. Rule costs

- `?` without `From` produces the 32 hand-written remaps; F3's fix avoids them within the rules.
- Having no `Option::map` (no closures) forces `match` blocks, marked with `#[expect(clippy::manual_map)]` (for example `deadlines.rs:151`, `translate.rs:250`). This costs about 5 lines per site, about 20 sites.
---

# 10: oauth

## Code-quality review: OAuth area (skein-oauth, -oauth-accounts, -oauth-issuer, -fake-oauth, tests/oauth*)

The weakest code is oauth-accounts' component state and the tests/oauth-accounts harness. skein-oauth and skein-oauth-issuer are in better shape.

### 1. Scope read

- **Read closely:** in oauth-accounts, all of `component.rs`, `exchange.rs`, `component/sign_in.rs`, `names.rs`, `keeper.rs`, `route.rs` and `accounts.rs`, plus about half of `lock.rs` and `private.rs`. Also `skein-oauth/src/record.rs` and `patches.rs`. In tests/oauth-accounts: `world.rs` (about 1,100 of 1,451 lines), `sign_in.rs` 280–620, `private_cuts.rs`, `sign_in/cuts.rs`, `expected.rs` and `owner.rs`. The crate's unit-test files `tests.rs`, `private_tests.rs` (first 260 lines) and `ports_tests.rs` (first 110 lines).
- **Skimmed:** `skein-oauth` `machine.rs`, `json.rs` and the head of `machine_tests`; the fake-oauth files `lib.rs`, `openid.rs` and `key.rs`; the issuer files `lib.rs`, `requests.rs`, `form.rs` and `tests/bounds.rs`; `listener.rs`; skein-llm-connection's `call.rs`; skein-llm's `openai/json.rs`; tests/oauth `world.rs` and both `qa_paste.rs`; the head of tests/oauth-accounts `memory.rs`; the suite `sign_in.rs`; tests/oauth-issuer `support`.

### 2. Scores (1–5)

1. **Cleanliness 3:** comments earn their place. Against that: test hooks sit in production files (component.rs:34-49), parameters go unused (`_files` at component.rs:217 and 489, private.rs:147), and `match … None => {}` (33 times) is mixed with `if let Some` (36 times) in the same files.
2. **Organization 2:** five parallel per-account lists, three unit-test layouts, and the shared test fixture lives in a file named for one feature.
3. **Taste 3:** bounds and refusal ordering are handled with care. The component's state, though, is a bag of flags.
4. **Naming 3:** follows the design's words (keep, lend, visit, taken). But `account: u32` means both the owner's number and the internal slot, and the names `Web` and `above` are vague.
5. **Primitives 2:** a whole `oauth::Event` is stored as state, and the socket stack, the JSON value type and the percent codecs are each written more than once.
6. **Elegance 2:** `fire` is a priority scan over flags, not the design's state table.
7. **Economy 2:** fixtures and harnesses are retyped across five test crates.
8. **Complexity 2:** state that could be derived is stored, and a story-switch harness hides each test's assertion.

Per crate: skein-oauth would score about 3.5, skein-oauth-issuer about 4, oauth-accounts and its world about 2.

### 3. Strengths

- `skein-oauth/src/record.rs:156-289`: the codec use is clean. One schema (`schema/oauth-v3.schema`), the runtime limits mapped onto the schema's ceilings in one function, codec problems mapped to `DecodeError` in one `match`, and an honest empty `patches.rs`.
- Refusal order and bounds follow oauth.md 6.5 closely. For example `redirected` (sign_in.rs:241-339) refuses in exactly the order the design lists.
- The issuer (`requests.rs`) is small, linear and reads straight against oauth-issuer.md.
- Memory tests have named ceilings, each with its derivation beside it (tests/oauth-accounts `memory.rs:9`, 42, 124, 300).

### 4. Findings, ranked by payoff

**1. The world harness is copied per story family** (Economy, Organization). *Answers the work_pending / next_policy_deadline / overdue question.*
- **Evidence:** those methods are `Host` and `Referee` trait impls, not boilerplate per subcomponent. But the impls are near-copies:
  - Two owner `Client` adapters, world.rs:277-813 and sign_in.rs:286-537, with the same `iterate` loop (~70 lines), `private_io` (~55), `work_pending` and `is_empty`.
  - Two `Process` enums whose `Host` impls only forward (world.rs:815-866, sign_in.rs:548-608).
  - Two restart-signal wrappers (private_cuts.rs:17-80, sign_in/cuts.rs:39-135).
  - Two hand-driven protocol loops, world.rs:1110-1386 and sign_in/protocol.rs:13-537, each under `#[expect(clippy::too_many_lines)]`.
  - Story-specific limits copied verbatim (sign_in.rs:325 and protocol.rs:34).
  - `world::Client` also carries three optional sub-owners and dispatches story hooks to them (world.rs:278-280, 363-375, 446-458).
- **Fix:** one owner adapter parameterised by a small `Script` value. One restart wrapper. One protocol driver. Forward through `fn host(&mut self) -> &mut dyn Host`, as cuts.rs:17-29 already does.
- **Size:** about 900 of 8,257 lines.

**2. The story-switch harness hides each test's assertion** (Complexity; testing-strategy §10 "the setup the assertion reads", "rows carry data").
- **Evidence:**
  - Suite tests are `drop(sign_in::run(43, Story::Public, false, false, Memory::Checked))` (tests/suite/sign_in.rs:4-36). The expected outcome lives in `Judge` match arms keyed on `Story` (sign_in/expected.rs:24-108, world.rs:921-963), and arms share code through `if self.story == Story::OpenIdAgain` (expected.rs:66).
  - About 108 `story ==` / `matches!(story` branches across src.
  - Row bodies branch: `if protocol { … } else { … }` (suite/sign_in.rs:76-94).
  - Positional bools (`faulted`, `large`) carry no names.
- **Fix:** each test builds its script and asserts its own fact list; the referee keeps only the invariants every world checks. Split rows that branch into separate tests. Replace the bools with an options struct.
- **Size:** about 400 lines, mostly exhaustive `Story` arms.

**3. The account state is a bag of flags, and one field can be derived** (Primitives, Complexity; programming-model 5.4).
- **Evidence:**
  - `State::Record` has 10 fields, 6 of them flags (component.rs:102-113). `rejection: Option<u64>` is set only alongside `rejected = true`, and only for the record's own generation (250-255). Nothing mutates the generation in place, so `rejection == Some(gen)` always equals `rejected`: two fields hold one bit.
  - `Exchange` has 19 fields (exchange.rs:41-61), including `Option<oauth::Event>` (`sign_in`). That stored event is unpacked with 6 `unreachable!` arms twice (sign_in.rs:166-184, 403-414).
  - The return location lives in both `Exchange` and `Listener` and is handed between them (sign_in.rs:49-50, 193).
- **Fix:** drop `rejection`. Fold held/due/attempted/announced into a refresh-status enum. Store a `SignInValues` struct, not an `Event`; if skein-oauth made `Event::SignIn` carry a pub struct, that would also remove its own repacking at machine.rs:264-293. Keep the return location in one place.
- **Size:** about 100 lines, and a large gain in clarity.

**4. Five parallel per-account lists, plus refusal boilerplate** (Organization, Economy).
- **Evidence:**
  - `accounts`, `names`, `sources`, `bindings` and `keepers` are separate `List`s indexed alike (component.rs:132-141), with about 85 `.expect("account …")` lookups.
  - `number(&self.names, account)` appears 49 times. About 30 `up.push(Event::Refused { account: number(…), asked, why }); return;` blocks.
  - `grant(slot, self.sources.get(account).expect(..), env.now, number(..), up)` appears 5 times.
  - `fail_exchange` and `end_exchange` share a keeper-cancel block (886-897 ≡ 953-964) and a drain–cancel–drain block (899-905 ≡ 971-977).
- **Fix:** one `List<Slot>`, where a slot holds name, state, source, binding and keeper. A `refuse(place, asked, why, up)` helper. A shared `cancel_exchange` tail. A `Place` newtype (allowed by 10.2) so a slot index is never confused with an owner number.
- **Size:** about 250 of 3,400 production lines.

**5. A hand-rolled linked list over the slab, walked with `while`** (Primitives; rules §3 "Bounded" and 10.2 "does not hand-roll data structures").
- **Evidence:** `first_exchange` plus `Exchange.next`. Inserting walks to the tail and retiring walks the chain (component.rs:996-1030). Nine `while let` walks (component.rs:544, 1002, 1020, 1034, 1136, 1171; sign_in.rs:102, 206, 217). skein-llm-connection solves the same problem with `slots: List<Option<Id<Connection>>>` and `for` (llm-connection component.rs:67, 256).
- **Fix:** do the same here, or add slab iteration to lib.
- **Size:** about 60 lines, and 9 rule breaches gone.

**6. Each test crate retypes the same fixtures** (Economy; §10 "a setup written once").
- **Evidence:**
  - The ~30-line `ClientLimits` literal appears 9 times. The document `oauth::Limits` literal appears 13 times.
  - Literals of 11-field structs: `SavedToken` 23 times, `Event::SignIn` 24 times, `Registration` 12 times, `Account::SignIn` 13 times.
  - `boxed`, `time` and `wall` are redefined in tests/oauth/src/lib.rs:7-17, tests/oauth-issuer/tests/support/mod.rs:36-47 and skein-oauth machine_tests.rs:6-14.
  - In oauth-accounts `tests.rs`, `component.down(&env, R, &mut out, &mut Queue::uncapped(), &mut Queue::uncapped())` appears 26 times, although `private::Owner::ask` already exists.
- **Fix:** one fixture module, in tests/oauth's lib, shared by the three oauth test crates. Use struct-update overrides and an `Owner` in every unit test.
- **Size:** about 800 lines across the area.

**7. The socket stack is written twice** (Primitives). *Answers the llm-connection question.*
- **Evidence:** `Web` (exchange.rs:171-555) and llm-connection's `Connection` (call.rs:50-…) each own a socket token, an optional `tls::Client`, the same four queues (events, plain, tls_events, cipher), `close_sent`/abort handling, and connect → handshake → call phases. Both route TLS and HTTP by hand and both repeat `Env { now, wall, limits: env.limits.tls }`.
- **Fix:** one byte-stream transport over an optional TLS session (socket, TLS, cipher and plain queues, close and abort), for both crates to compose.
- **Size:** about 150–200 lines (unsure: llm's keep-alive and deadline table may limit how much is shared).

**8. Two bounded JSON value types; codec serves only the binary record** (Primitives). *Answers the codec question.*
- **Evidence:**
  - skein-oauth uses skein-codec only for the binary kept record, and cleanly (see Strengths). That use says nothing about JSON.
  - Its JSON is hand-written: `skein-oauth/src/json.rs` (397 lines) has the same accessor set as `skein-llm/src/openai/json.rs` (532 lines): `field`, `required`, `optional_at`, `text_ref`, `unsigned`, `value_at`, `span`, `validate`, and a `from_bytes` tokenizer loop.
  - Smaller duplicates: the record-limits mapping is re-derived in fake-oauth `lib.rs:40-56` (it duplicates `record.rs:156-167`), and the four-field loop repeats between `record.rs:143` and `record.rs:278`.
- **Fix:** move the value type into skein-json, where skein-llm's newer collector-based form is the one to keep. codec.md's planned "JSON from schemas" is the lasting answer for both crates. Export `record_limits` from skein-oauth.
- **Size:** about 350 lines.

**9. Four copies of the percent/form codec** (Primitives, Economy).
- **Evidence:** encoding in `skein-oauth/redirect.rs:304-327`; encoding and decoding in `skein-oauth-issuer/form.rs:72-143`; decoding in `skein-oauth-accounts/redirect.rs:83-116`; decoding in `skein-fake-oauth/lib.rs:642-711`. Each has its own `hex`. The fake's copy uses `loop` and `while` in a no_std step crate (lib.rs:633, 648, 674, 684).
- **Fix:** one `form` module in lib or skein-http.
- **Size:** about 150 lines, plus 4 rule breaches gone.

**10. Three unit-test layouts, and messy test files** (Organization). *Answers the layout question.*
- **Evidence:**
  - Three layouts in four crates: sibling `*_tests.rs` files under `#[path]` (oauth-accounts), `machine_tests.rs` plus `machine_tests/` (skein-oauth), and `src/tests/` (oauth-issuer, which matches about 20 other crates).
  - oauth-accounts' `tests.rs` scatters its `mod` declarations among helpers and tests (lines 393, 458, 508, 511, 586, 604, 607).
  - The `Owner` fixture shared by every test file lives in `private_tests.rs`.
  - Odd ones out: an inline `mod tests` in `exchange.rs:581`, and a test-only `impl` in `names.rs:99`.
  - The test crates differ too: tests/oauth-issuer puts support under `tests/support` with an empty lib, while the other two use `src/`. `qa_paste.rs` is a separate test binary in two crates.
- **Fix:** use `src/tests.rs`, `src/tests/<topic>.rs` and `src/tests/support.rs`, and fold `qa_paste` into the suite.

**11. Table tests break the row rules both ways** (§10 "rows carry data", "one behaviour per test").
- **Evidence:**
  - Cases of one behaviour as separate tests: 26 one-line `valid_refuses_*` tests (tests.rs:611-767) should be rows of one table.
  - Rows that expect different outcomes, or bodies that branch on a row: `ports_tests.rs:73` (true and false rows), `private_tests.rs:243` (`if unreadable`) and `tests.rs:561` (`scope_grant(refresh)`).
  - Worst: oauth-issuer `tests/bounds.rs:3-60` switches on the row index three times and slices with a bare `8`.
- **Fix:** turn the `valid_refuses_*` tests into one table; split the branching rows into separate tests; in the bounds test, give each row its own setter function.
- **Size:** about 120 lines.

**12. A derive(Clone) written by hand** (Taste). `Registration` does not derive `Clone`, so `copy_registration` (accounts.rs:61) writes the clone out by hand for 8 call sites. 10.2 allows `Clone` on configuration. About 15 lines.

### 5. Cross-area suspicions (not verified outside this area)

- skein-forge-connection may compose its own TLS-over-socket stack too (my grep found `tls::Request::Handshake` only in oauth-accounts and llm-connection). Worth checking whether it reuses one or hand-routes another.
- `hex` helpers are repeated in skein-forge (`github/id.rs`), skein-forge-setup (3 copies) and skein-json; lib could export one.
- `Process` enums with forwarding `Host` impls recur in tests/world, tests/tls, tests/llm-connection, tests/io and tests/echo. Restart-signal wrappers recur in tests/io `cuts.rs` and `locks.rs`. skein-world could provide both.
- skein-lib's `Slab` has no iteration, so each component invents its own way to walk live entities.

### 6. Rule costs

- **No closures:** `#[expect(clippy::manual_map)]` plus a hand-written `Option` map, 5 times in record.rs. Small.
- **`wildcard_enum_match_arm = "deny"` in the test crates' Cargo.toml:** this is self-imposed; test code is ordinary Rust. It forces 28-variant `Story` lists (sign_in/owner.rs:5-46, expected.rs) and 12-variant `Event` lists in `refusal_of` and `failure_of` (tests.rs:461-500). About 250 lines; relaxing the lint for tests would remove them.
---

# 11: ui-browser

I read the shared brief and reviewed against it.

Tree root (all paths below are relative to it): `<clone>/skein`

## 1. Scope read

**Read closely:**
- skein-ui: `tree.rs`, `builder.rs`, `diff.rs`, `view.rs`, `binding.rs`, `scene.rs`, `outline.rs`, `look.rs`, `limits.rs`.
- skein-ui tests: `tests/fixtures.rs`, `tests/diff.rs`, part of `tests/markdown.rs`, the head of `tests/tree.rs`.
- ui-text: `render.rs` (lines 1–320), the head of `terminal.rs`, `tests/fixtures.rs`.
- ui-web: `mapping.rs`, `svg.rs`, `page.rs` (structure, `apply`, `insert`, `remove`), `tests/fixtures.rs`, the head of `tests/page.rs`.
- fake-person: `lib.rs`, `tree.rs`, `random.rs`. fake-person-browser: `face.rs`.
- skein-browser: `find.rs`, the type section of `browser.rs`.
- lab-browser: the `runtime.rs` pump and spawn, the `hand.rs` Hand.
- The spawn and pump code in the fake-person-browser `harness.rs`, and `tests/browser/tests/support/real.rs`.
- examples/ui: `view/{view,state,limits}.rs`, `shell-web/src/lib.rs`, the view tests and fixtures.

**Skimmed:** `phrases.rs`, `markdown.rs`, the rest of ui-text, `page/web.rs`, ui-bundle, `tests/ui`, the tests in `tests/browser`, lab-browser's `calls`/`config`/`outline`, the server example, and the design docs (by section).

## 2. Scores

1. **Cleanliness 3.** The core is tidy. Against that: generated doc comments, many bare numbers, and lint waivers covering whole files in a crate that is meant to be step code.
2. **Organization 3.** The crate split matches ui.md §2. But `browser.rs` is 2,289 lines and `harness.rs` 1,294, and test support is copied between crates.
3. **Taste 3.** skein-ui reads as written with care. The testing crates drift into bags of flags and re-matching.
4. **Naming 4.** Mostly the design's own words. Slips: `Made`, the singular `Attribute`, `title_written`, `find_boxed`, `types::of`.
5. **Primitives 3.** Node, role, act, patch and scene are the right concepts. But Node and Attribute are parallel structs, Tree has no navigation methods, `takes` leaks a state, and role sets are re-listed at each site.
6. **Elegance 3.** The diff's phases and the builder follow the problem. `Person::next` and `BrowserFace` do not.
7. **Economy 2.** Three copies of the Chromium host loop, role tables written twice, fixtures retyped in many crates.
8. **Complexity 3.** Mostly just enough. `BrowserFace` and the host loops carry state that could be an enum or a single shared host.

## 3. Strengths

- **skein-ui's builder/diff/view.** It is small and bounded, every `expect` says why it holds, and `view.rs:6-9` puts the state table in the header. The diff is a clean match → remove → move → insert → attributes pipeline (`diff.rs:361-379`).
- **Reuse at the boundaries.** `BrowserFace` serves both the harness and the lab's hand. ui-web's `mapping::accessible`/`person` gives the face and the lab outline one role vocabulary. lab-browser reuses `role_name` (`outline.rs:4-6`).
- **ui-web page tests are the model to copy.** One behaviour each, golden HTML, and named ids with their derivation beside them: `const MAIN: NodeId = NodeId(2); // page, main` (`tests/page.rs:12`).
- **Exhaustive role matches where the design wants them** (`mapping.rs` `tag`/`name`/`value`, `role_name`): adding a role is a build error in every shell.

## 4. Findings (ranked by payoff)

**1. Three copies of the Chromium host loop** *(Economy, Organization)*
- `browser::command::Command` → `io::Spawn` is converted three times: `testing/skein-lab-browser/src/runtime.rs:289-317`, `testing/skein-fake-person-browser/src/harness.rs:279-307`, `tests/browser/tests/support/real.rs:92-120`.
- Event-owner extraction is copied verbatim in `runtime.rs:895-910` and `harness.rs:1279-1294`.
- Routing `Down` to the three pipes appears three times: `runtime.rs:441-447`, `harness.rs:1117-1124`, `real.rs:1014`.
- The pending/until fold and the ring reap/deliver/register/enter steps are copied too: `runtime.rs:399-487` vs `harness.rs:1138-1182`.
- `chromium()` is written three times (`tests/ui-web/src/lib.rs:28`, `real.rs:63`, `testing/skein-lab-browser/tests/support/mod.rs:35`) and `fonts()` twice, with different fallback rules.
- **Fix:** one Chromium host in skein-fake-person-browser (both other users already depend on it) providing `spawn(command, root)`, `route(down)`, `chromium()` and `fonts()`. Event-owner extraction belongs on `io::Event` itself.
- **Size:** about 200 lines now, more if the pump is shared.

**2. One `Find`, two matching rules** *(Primitives)*
- The tree face folds case: `testing/skein-fake-person/src/tree.rs:37-45` compares `to_lowercase()` forms.
- The kit keeps case: `testing/skein-browser/src/find.rs:8-19`, browser.md:258, and its test asserts that `"Failing"` does not match `"3 failing"`.
- So a script that passes on the tree can fail in Chromium, although ui.md §17.1 says the browser face "answers the same queries".
- There are also four hand-written substring searches beside `skein_lib::bytes::find`: `find.rs:9`, `browser.rs:1811`, `tree.rs:37`, `tests/browser/tests/qa_look.rs:340`.
- **Fix:** pick one rule (browser.md's: case kept) and use it on both faces through `bytes::find`.
- **Size:** about 30 lines, plus a real consistency gain.

**3. `Node` and `Attribute` are parallel structs** *(Primitives, Economy)*
- `tree.rs:278-306` (Node, 25 fields) restates 17 of the 19 fields of `diff.rs:19-40`.
- Field-by-field code is written out in `Node::vacant`/`Node::new` (`tree.rs:309-367`, which differ only in capacities), `Attribute::of` (`diff.rs:43-65`) and the 19-term `||` comparison (`diff.rs:273-292`).
- Adding one property means editing six places.
- **Fix:** give `Node` a field holding the shell-visible attributes, with tokens (binding, link, key, slot) beside it. The comparison becomes `old.shown != new.shown || …`. Have `vacant()` call `new()`.
- **Size:** about 70 lines in skein-ui.

**4. Role vocabulary written twice, and role sets that escape exhaustiveness** *(Primitives, Economy)*
- `next_role` is a 50-line exhaustive successor function, copied verbatim in `crates/skein-ui-web/src/mapping.rs:449` and `testing/skein-fake-person/src/tree.rs:233`.
- `role_name`/`role_named` (`tree.rs:118-230`) are two hand-kept inverse tables.
- `acted_on` (`builder.rs:569-622`) restates exactly the set of roles that `takes` accepts (`binding.rs:143-203`).
- About 20 role sets are written as `[Role::…].contains(..)` arrays, which a new role passes silently, against ui.md §3's "closed, so an addition breaks the build":
  - landmarks: `builder.rs:263-277`, again as an `||` chain in `outline.rs:121-129`
  - cell roles: `render.rs:262`, `builder.rs:225`
  - focusable roles, two lists that differ: `ui-web page.rs:526` and `mapping.rs:226-238`
  - native-disabled roles, the same list twice in one expression: `mapping.rs:~306-318`
- **Fix:** `Role::ALL` in skein-ui. Named exhaustive predicates there (`landmark`, `field`, `collection`, `cell`) used by every shell. `acted_on` derived from `takes`. One `(Role, &str)` table for person names.
- **Size:** about 150 lines.

**5. `Tree` gives only an index and a size, so each consumer re-derives navigation** *(Primitives, Elegance)*
- The subtree end (`index + size`) is recomputed about 25 times across ui-text `render.rs` (43-66, 87, 107, 163, 262, 325, 339, 435, 578), fake-person `tree.rs:29,57,72,96`, and skein-ui itself (`diff.rs:115` parents, `builder.rs:530` check_depth, `tree.rs:461-482` reverse ancestor scan).
- Two shells consume two different interfaces:
  - ui-text reads the full `Tree`/`Node`, including the private tokens.
  - ui-web reads `Patch`/`Made` and rebuilds its own parent/children mirror (`page.rs:48-57`, `293-327`).
- `takes(role, submits_on_enter, act)` makes every caller re-read `SubmitsOnEnter` (`tree.rs:452`, fake-person `tree.rs:112`, `random.rs:53,79`).
- `TreeFace::allows` (`tree.rs:103-113`) re-implements `Tree::act` (`tree.rs:449-457`).
- **Fix:** `Tree::end(i)`, `Tree::parent(i)` (the list from `diff::parents`, made public), `Tree::index(id)` and `Tree::takes(id, &Act)`.
- **Size:** about 80 lines, and consumers become clearer.

**6. Test fixtures retyped across crates** *(Economy, §10 "a setup written once")*
- The 12-field `Limits` literal is written out about 14 times. It is retyped even inside skein-ui: `tests/slots.rs:5`, `tests/diff.rs:13`, while `tests/tree.rs` and `tests/markdown.rs` already use `..LIMITS`.
- `tests/ui-text/src/lib.rs:1-44` is a verbatim copy of `crates/skein-ui-text/src/tests/fixtures.rs:1-49`. Getting ids requires calling `diff` against an empty tree (fixtures line 37).
- ui-web `tests/page.rs` and `tests/page/events.rs` contain 76 calls of the form `page.apply(&fixtures::patches(&mut view, N, |b| { main(b); …; end(b); }), &mut Vec::new())`.
- `tests/browser/src/tests.rs` opens 20 tests with `World::new(); start(); open();` and writes `Token::new(3)` 21 times. `src/tests/look.rs:6-14` in the same crate already has `PAGE`, `READ` and `world()`.
- **Fix:**
  - `Limits { … , ..LIMITS }` everywhere.
  - One ui test-support crate holding the page/banner/main builder and `shown(build) -> Tree`.
  - A `Shown { page, view, shown }` fixture with `show(build) -> Vec<Op>` in the ui-web tests.
  - Reuse look.rs's helpers in tests.rs.
- **Size:** about 400 lines.

**7. Omnibus tests in skein-ui** *(§10)*
- One test carries three behaviours: `fields_keep_typing_and_binding_changes_are_private_while_drawings_change_whole` (`tests/diff.rs:211`).
- Two behaviours appear only in one test's body: `every_patch_kind_preserves_ids…` also asserts staleness along the way (`diff.rs:153-154`).
- Bodies branch on a row's value: `if shown == 2` and `if shown == 3` (`diff.rs:226,229`), `if shown == 1` (`markdown.rs:84`).
- Bare indices: `get(8)`, `get(2)`, `kinds[0..5]` (`diff.rs:155,186-191,207`).
- **Fix:** split along the behaviours already named, and name the indices.

**8. examples/ui is not yet an exemplar** *(Naming, §10 "named fixtures and sizes")*
- `view/src/view.rs` refers to phrases by bare index: 18 `phrase(state, N)` calls, plus `zip(4..7)`, `zip(22..26)`, `(DISCARD, 20)`, and `Some(16)` in `state.rs:355`.
- The label→phrase mapping is written three times (`view.rs:157-161`, `253-257`, `273-281`).
- `title_written` is the write count for the whole form (it is also used for body, due and minutes: `view.rs:185,195`).
- `State::start(…, _offset, _languages)` has unused parameters (`state.rs:76`).
- The reference shell retypes the view's owner and style tokens (`shell-web/src/lib.rs:24,93,95`; token `2` appears in both `view.rs:183` and `lib.rs:93`).
- The shell converts the view's `Record`/`Exchange`/`StreamDuty` (`state.rs:8-37`) field by field into ui-web's types (`lib.rs:82-86,169-190`). Every application that copies the shell will copy these roughly 40 lines.
- The view tests hand-scan `view.tree().nodes()` at 8 sites with a bare `Queue::with_capacity(1200)` at 14 sites, instead of using `TreeFace` + `Find` and `limits().patches`, although `tests/person.rs` shows how.
- **Fix:**
  - A `Word` enum or named constants, and one function from label to word.
  - Export `OWNER`, the style tokens and `NAMES` from the view.
  - Rename `title_written` to `draft_written`.
  - Move the duty vocabulary to one shared no_std place (design-level; unsure where it best lives).

**9. `BrowserFace` is a bag of 25 Options and flags** *(Complexity, Elegance)*
- The struct is at `face.rs:53-78`. Whether it is pending is derived from four of those fields (`pending`).
- `title`, `find_boxed` and `Face::find` (`face.rs:117`, `248`, `637`) are near-copies that differ only in two flags.
- The `token(); acting = Some(op); requests.push` sequence is written 10 times, and `seen.get(usize::try_from(node.0)…)` 7 times.
- **Fix:** an enum for the one operation in flight (Search, Act, Keep, Command, Choice), plus one `query(find, title, boxes)`.
- **Size:** about 80 lines.

**10. `Person::next` matches on the step again inside its own arms** *(Elegance)*
- `testing/skein-fake-person/src/lib.rs:214-229`, `240-249` and `258-267` each re-match `step` with `unreachable!` arms.
- **Fix:** give `See`/`Gone` and each act step its own arm, calling shared `expect(…)`/`act_on(…)` helpers.
- **Size:** about 25 lines, three `unreachable!` gone.

**11. skein-browser does not follow the rules it is supposed to** *(Cleanliness, Primitives)*
- browser.md §2 says the crate is "step code under the Rust subset". Yet:
  - six modules waive `disallowed_types` for the whole file (`browser.rs:3`, `browser/look.rs:2`, `command.rs:3`, `wire/{frame,encode,decode}.rs`);
  - entities are kept in hand-made `Vec<Option<T>>` slabs (`browser.rs:178-180`) instead of `Slab`/`Id`;
  - lifecycles are booleans (`Person{ready,closing,closed}`, `Page{navigating,opening,closed}`, `browser.rs:35-66`) instead of a state enum;
  - `browser.rs` is 2,289 lines.

**12. Small items**
- **Scene encoding decoded twice.** The byte encoding is decoded in both `scene.rs:419-485` and `svg.rs:83-151`. skein-ui should export the decoder.
- **Splice room repeated.** `if limits.slots > 0 { limits.nodes } else { 0 }` appears five times (`builder.rs:65,72,77`; `view.rs:35,59`). It should be a single `Limits` method, and `Builder::new` should call `lend`.
- **Dead `Option`.** `Print::new` always returns `Some` (`render.rs:15-21`).
- **`while` in step code.** ui-text's renderer is step code but uses `while` (`render.rs:47`, `207`).
- **Doc-comment noise.** Generated comments such as `/// A item node.` and `/// The node is submitsonenter.` (`tree.rs:49,147`).

## 5. Cross-area suspicions (not checked beyond my area)

- `skein-browser/src/wire/decode.rs` (527 lines) is a second JSON reader beside `crates/skein-json`, which the crate already uses for writing.
- Extracting an `io::Event`'s owner recurs at `tests/forge/machine/src/machine.rs:434` and `crates/skein-oauth-accounts/src/component.rs:503`. It probably belongs on `io::Event` in skein-io.
- The ring-loop boilerplate (reap/deliver/register/enter, the pending/until fold) likely recurs in `examples/ui/server/src/main.rs` and `testing/skein-world`. A shared helper in skein-shell or skein-world may be due.
- The full `Limits` literal is probably retyped in other test crates as well.

## 6. Rule costs (short)

- **Exhaustive role matches for true/false answers.** Over 49 roles these cost about 300 lines in my area:
  - `acted_on`: 54 lines
  - `takes`: 60 lines
  - ui-web `base`, `name` and `value`: about 45 each, even though `value` only tells `TextArea` apart
- With `Role::ALL` and a few named predicates kept in skein-ui, the rule would still hold, at roughly a third of the size.
---

# 12: world-integration

I reviewed the whole area read-only. The biggest problem is a missing primitive: every host hand-writes skein-io's up and down stage. The echo template copies it, the fake clients and fake peers copy it again, and the same ordering test is written twice by `#[path]`-including other crates' private files.

## 1. Scope read

**Read closely:**
- `skein-shell` `Host` (drive.rs).
- skein-world: `lib.rs`, `world.rs`, `referee.rs`, `outcome.rs`, `domain.rs`, the head of `domain/referee.rs`, and `real.rs` (public surface, `Running`, the run loop).
- The echo example: `service/src/{lib,io_stage}.rs`, `shell/src/{lib,main}.rs`, `domain/src/lib.rs`, and the state and shutdown parts of `protocol/src/layer.rs`.
- `skein-echo-client/src/lib.rs`.
- skein-fake-peers: `lib.rs`, `face.rs`, `transport.rs`, `llm.rs`, and the forge and oauth `Host` impls.
- `skein-fake-checkout`, in part.
- tests/echo: `proc.rs`, `referee.rs`, part of `scenarios.rs`, `suite/{real,scenarios}.rs`.
- tests/world: `src/lib.rs`, the head of `src/hosted.rs`, `suite/{stop_continue,interrupt,world}.rs`, and the head and outline of `end_to_end.rs`.
- tests/fake-peers: `src/lib.rs`, `suite/{llm,oauth,stages}.rs`, and the pass/settle code in the forge fixtures.
- The head of tests/fake-checkout `files.rs`.

**Skimmed:** skein-world `tree.rs`, `deadline.rs`, `end_to_end.rs`, `ring/files/stream/heap.rs`; fake-peers `forge/*`; the echo `conn.rs` files; fuzzy tests; census.

## 2. Scores

| Criterion | Score | Why |
|---|---|---|
| Cleanliness | 3 | Mostly tidy, but the template has stale comments and `llm.rs` has dead code. |
| Organization | 3 | Good crate split; three `#[path]` reach-ins to other crates' sources; two near-twin `Script` hosts. |
| Taste | 3 | Careful prose and exhaustive matches; but redundant parameters, Debug-string equality checks, telescoping constructors. |
| Naming | 3 | `Referee`/`Expectations` mean opposite things at the crate root and in `domain`; "worst case" contradicts the design; `temper-*` file names in a skein kit. |
| Primitives | 2 | The io stage is not a primitive; `Host` makes each implementer merge two deadlines by hand. |
| Elegance | 3 | Echo's `iterate` follows programming-model §2 almost line for line; the harness spawn matrix does not follow the problem. |
| Economy | 2 | Forwarding `Host` impls, no-op referee methods, and fake-peers tests that hand-roll their own worlds. |
| Complexity | 3 | `real::World` has 5 constructors plus Deref wrappers; `Running` has 20 fields; elsewhere about right. |

## 3. Strengths

- **Echo service:** `Limits::check` returns a typed `Unusable` reason, and `main` numbers its startup steps after shell.md §6.
- **The teardown invariant is checked once,** in skein-world (`world.rs:173-189`). No service writes its own.
- **tests/world tests are well shaped:** raw-record `Script` hosts keep the harness's tests independent of any service, and `suite/world.rs` is one behaviour per test with derived ceilings ("measured peak 1,021 B plus 25%").
- **Fake peers share one `Face`,** and fake-checkout's `Remote` boundary is narrow and clean.

## 4. Findings, by payoff

**1. The io stage is copied into every host** (primitives, economy)
- **Copies in the area:**
  - `examples/echo/service/src/io_stage.rs:9-48`, plus `lib.rs:403-417`.
  - `testing/skein-echo-client/src/lib.rs:359-392` and `487-501`.
  - `crates/skein-fake-peers/src/face.rs:64-111`, which uses a bare `room() < 2` where the others use `io::MAX_OUT_*`.
  - `tests/world/src/bin/terminal_fixture.rs:81`.
- **Tree-wide:** 39 `io::resume` call sites outside skein-io.
- **The same test, twice:** "ready before completions before timers at a near-full kernel queue" appears in `tests/echo/tests/suite/scenarios.rs:249-317` and `tests/fake-peers/tests/suite/stages.rs:4-97`. Each `#[path]`-includes the other crate's private source.
- **Redundant `now`:** `io_stage::up`, `io_up` and `protocol_up` take `now` beside `env.now` (`service/src/lib.rs:291-329`; client `348-359`). In the copied test (`scenarios.rs:300-302`) the two already disagree.
- **Fix:** add `skein_io::stage::{up, down}` as free functions in step code (no traits). They take bounds from io's limits and room from `MAX_OUT_*`. Test them once in tests/io.
- **Size:** about 330 lines in the area, and likely over 1,000 tree-wide (unsure).

**2. The loop surface (your question 1)** (primitives, economy)

Of 47 `Host` impls tree-wide, about 13 only forward. The rest compute their own answers, which is an honest cost. Two parts are not:

- **(a) `next_deadline` is a hand merge.** In every real impl it is the earliest of a mechanism deadline (io's close or retry, a poll) and `next_policy_deadline`:
  - echo `lib.rs:228-243`, client `292-308`;
  - `llm.rs:468-474`, `oauth.rs:610-616`, `forge.rs:638-646`;
  - `Binary` (end_to_end.rs:631-637), the tests/world `Script`s.
  - **Proposal:** `Host` requires `next_mechanism_deadline()` and `next_policy_deadline()`, and `next_deadline` becomes a provided method. That removes about 47 merges, and a host can no longer arm a policy deadline the loop never wakes for.
  - **Helper:** add `skein_lib::earliest(Option<Time>, Option<Time>)`. Private copies exist in skein-lib `requests.rs:718`, `skein-llm-connection/src/deadlines.rs:159`, `skein-oauth-accounts/src/component.rs:1239` and `skein-forge-setup/src/host.rs:376`, plus nested matches in echo `lib.rs:228`, client `292`, `tests/echo/src/referee.rs:291` and skein-world `world.rs:105`.
- **(b) Enum dispatchers repeat the whole surface.**
  - `tests/echo/src/proc.rs:51-120` matches per method (70 lines).
  - Four near-identical `&dyn Host` forwarders of about 45 lines each: `tests/world/tests/suite/end_to_end.rs:32-79`, `writer_pipe.rs:64-115`, `real_conformance.rs:174-221`, `tests/echo/tests/end_to_end.rs:20-64`.
  - So echo's surface is written three times: `Service`'s inherent methods, then `Echo`'s `Host` impl, then `Proc`'s.
  - **The copies drift:** `exit` is forwarded in 2 of the 4 `dyn` forwarders, `state_digest` only in `proc.rs`. `end_to_end.rs`'s `Process` wraps `hosted::Script`, which overrides `exit` (`src/hosted.rs:315`), but does not forward it (harmless for a root today).
  - **Proposal:** in skein-shell, `trait Delegate { fn host(&self) -> &dyn Host; fn host_mut(&mut self) -> &mut dyn Host; }` with a blanket `impl<T: Delegate> Host for T`. Each test enum then writes about 12 lines, and every method is forwarded. (Unsure: coherence with other concrete impls; the fallback is a test-only `macro_rules!` in skein-world.)
  - **Also:** `Proc`'s `limits` fields (`proc.rs:15-16`, `116-117`) exist only to recompute `operations()`, which `Echo` already answers.
- **The single-struct idea:** a "struct of answers" (one `wake()` method) would only shrink the forwarders; (a) plus (b) remove more.

**3. `Referee` lacks defaults for its no-op methods** (economy)
- **Counts:** in the area, `overdue → None` 16×, an empty `act` 9×, `next_deadline → None` 13×. Tree-wide: 25, 15 and 21.
- **Example:** `tests/world/src/lib.rs:425-447`, where `Later` is about 20 lines and only the `passed`/`last_word` bodies say anything.
- **Fix:** give `act`, `next_deadline` and `overdue` defaults, as `starved` and `last_word` already have (`referee.rs:68-77`). About 120 lines in the area.

**4. skein-world's API: two tiers, two vocabularies, a spawn matrix** (primitives, complexity)
- **Two memory forms:**
  - Simulated: `new(seed, config, referee, Memory)`, and its `Outcome` checks release on drop (`outcome.rs:56-79`).
  - Real: five constructors, including `new_controlled_checked`, which is reached only through `new_checked`. They return `CheckedWorld`/`CheckedOutcome` Deref wrappers (`real.rs:77-131`, `186-239`).
- **Spawn variants:**
  - Simulated: 6 spawn plus 3 host methods, 11 of them pure forwards to `skein_sim_world::World` (`world.rs:56-123`).
  - Real: 4 spawn plus 3 host (`real.rs:242-295`).
  - Factories take `()`, `Fd` or `&Inherited`, depending on the variant.
- **Odd edges:** `run_with_faults` and `run_with_inputs` cannot be combined, although `run_inner` takes both (`world.rs:142-163`). `real::run` has no caller in this tree.
- **Proposal:**
  - One constructor per tier taking `Memory`.
  - A `world.controls()` accessor, with the referee given to `run`; that removes every `new_controlled*`.
  - `spawn(Startup, make: FnOnce(&Inherited) -> P)` with `Startup { roots, appends, signals, restartable }`, and `host(program, Startup)`.
- **Size:** about 150 lines, and one form for a world author to learn.

**5. fake-peers' tests rebuild the world loop six times** (economy, your question 4)
- **Copies:**
  - `pass`: `oauth.rs:183`, `llm/request.rs:8`, `forge/fixtures.rs:245`, `forge/github.rs:251`, `github_memory.rs:82`, `forge/git.rs:15`.
  - `settle` 4×.
  - HTTP request builders 5×.
  - The `Limits` literal 4× (`llm.rs:18`, `github_memory.rs:22`, `fixtures.rs:30`, `stages.rs:20`).
- **Against the model:** the oauth and llm passes call `sim.advance()` unconditionally (`oauth.rs:187`, `request.rs:12`). That moves time while a peer has work, which `time_does_not_move_while_a_process_has_deferred_work` exists to catch.
- **Reinvented:** `github_memory.rs:82-95` re-implements the per-iteration meter that `Memory::Within` provides.
- **Bare numbers:** `0..1000`, `0..100`, `128 + 2 * (…)`.
- **Telescoping constructors:** `llm.rs:42-118` has five rungs, from `make` to `make_domain_with_limits`.
- **Fix:** a `Rig` in `tests/fake-peers/src/lib.rs` with a work-gated `pass(&mut [&mut dyn Host])`, `open/send/receive/fetch`, `settle`, one `limits()`, one `request()` and one fixture struct. About 400 of 4.3k lines.
- **Inside the crate:**
  - Observation bookkeeping is written three times: `observations/observed_bytes/recording`, `without_records`, `observe`, `reserve_observation` in `llm.rs:159-213`, `oauth.rs:170-210`, `forge.rs:250-257` and `forge/observe.rs:9-25`. Move it into `Face`, about 60 lines.
  - `Listening` (`lib.rs:28-82`) has three identical impls. The inherent `address() -> Option` shadows the trait's `address()`, forcing `skein_fake_peers::Listening::address(&peer)` in the tests; make `address` a provided method.
  - **Dead code:** `llm.rs:98-104` moves `face.requests` out, pushes it back and resets `events`. `Face::new` already makes both queues uncapped (`face.rs:47-58`), so this changes nothing.
  - The crate description and lib docs omit the forge peer (`Cargo.toml:3`, `lib.rs:1-2`).

**6. The echo template (your question 3)** (cleanliness, taste)

Its shape is exemplary: a 417-line service and a 109-line `main`. These fixes would make it a clean template:
- **Stale comments:**
  - `domain/src/lib.rs:50-51` says io's `Shutdown` "is not built", but it is built and handled (`service/src/lib.rs:354`).
  - `service/src/lib.rs:380` ("while its replies fit") and `:391` ("while what it asks of io fits") describe checks that do not exist; those queues are uncapped.
- **Three-flag lifecycle:** `protocol/src/layer.rs:27-30` keeps `shutdown: Shutdown` beside `stopping: bool` and `aborting: bool`, which are read together at `:118-136` and `:495-496`. Programming-model §5.4 wants one enum.
- **Repeated destructure:** `Proc::echo` followed by a destructure to call `adopt_signals` appears 3× (`tests/echo/src/scenarios.rs:165-170`, `:424-429`, `suite/real.rs:46-51`). Add `Proc::echo_with_signals`.
- **Opaque flag:** `Echo::drain`'s `short` flag (`shell/src/lib.rs:62-79`) needs a name and a comment.
- **Wrong message:** `main.rs:65` maps an error to "the default address parses", the opposite of what happened; use `expect`.

**7. Named sizes broken in tests/world** (rules)
- **Ignored constant:** `WORLD_PATIENCE` is defined (`end_to_end.rs:10`), yet `Duration::from_secs(2)` appears bare 30× across the suite.
- **Copies:** `WALKS` is copied verbatim 3× (`stop_continue.rs:53`, `interrupt.rs:65`, `end_to_end.rs:159`).
- **Numeric state:** `stop_continue.rs:9-31` steps a bare `stage: u8`.
- **Bare limit:** `1024` in `interrupt.rs:79`, where the sibling test names it `STDERR_LIMIT`.
- **Repeated expiry handling:** the `if deadline.passed() { match deadline.expired() … }` block (about 8 lines) appears 9× in the area and about 48× tree-wide. A `Deadline::conclude(owned, what)` would replace it.

**8. Naming** (naming)
- **Crossed names:** at the root, `Referee` is a trait and `Expectations` a struct (`referee.rs:45`, `86`). In `domain`, the reverse (`domain/referee.rs:20`, `58`). Both are exported from one crate.
- **"Worst case":** `Cargo.toml:3` and `lib.rs:16-18` speak of worst cases; testing.md §5 says the host declares no ceiling or worst case.
- **temper names:** fake-checkout writes `.git/temper-head` and `.git/temper-conflicts` through 7 concatenated literals (`git.rs:219-370`).
- **`Wire`:** the name means two different things, in skein-world (`stream.rs:16`) and in fake-peers.

**9. fake-checkout tests** (rules, low)
- **Debug-string equality:** `format!("{checkout:?}")` stands in for state equality 14× (`files.rs`). Derive `Clone` and `PartialEq`.
- **Many behaviours per test:** `files.rs:9-28` checks about 8 things.
- **Computed expectations:** `first + 1`.

**10. `real.rs` `Running`** (`:421-444`, low, unsure)
It holds 20 fields. `ids` and `positions` are inverse indexes kept in step by hand (`:998-999`, `:1084-1089`). Separately, tests/world's two `Script` hosts (`src/lib.rs:43` and `src/hosted.rs:53`) share about half their state and their submit/proceed machinery.

## 5. Cross-area suspicions (not verified outside the area)

- **io stage copies:** tests/io (10+), oauth-accounts, llm-connection, tls, browser, lab-browser, fake-person-browser, examples/ui.
- **Match-per-method dispatchers:** `tests/oauth-accounts/src/world.rs:815`, `sign_in.rs:548`, `tests/llm-connection/src/simulated.rs:285`.
- **Possible teardown miss:** in `tests/llm-connection/src/simulated.rs:316-329`, the peer's `next_deadline` includes a release time that `next_policy_deadline` leaves out. Check whether the teardown check misses it.
- **`ms()` written 3×** (tests/world, tests/io, tests/echo): `skein_lib` has no `Time::from_millis`.
- skein-sim-world carries the same spawn matrix that skein-world forwards.

## 6. Rule costs

- **No traits in step code:** this forces a shell adapter that forwards `Service`'s 7 loop methods 1:1 (`shell/src/lib.rs:26-53`, about 30 lines per service). Small and acceptable.
- **No closures in step code:** an io stage helper must take its queues explicitly (6 parameters). Still clearly worth it.
---

# 13: cross-cutting

**Code-quality review of skein, the whole tree**

All paths below are relative to `<clone>/skein`.

## 1. Scope read

**Read closely:**
- `docs/design/README.md` §3 and `testing.md` §6.
- `programming-model.md` §1, §2, §5.2, §6.3 and §10, and `testing-strategy.md` §4, §7 and §10.
- Every `Cargo.toml`, compared against the documented crate graph.
- The connection modules side by side:
  - `skein-forge-connection/src/connection.rs`
  - `skein-forge-setup/src/host/connection.rs`
  - `skein-llm-connection/src/call.rs`
  - `skein-oauth-accounts/src/exchange.rs`
  - the settings, limits, boundary and `next_deadline` code of all four components.
- The `Host` trait and `Referee` trait, plus 12 of the 45 `Host` impls.
- `skein-fake-peers/src/face.rs`, `skein-world/src/{stream,referee}.rs`, and the top-level layout of 12 test crates.

**Skimmed:** two or three test files each in forge/machine, oauth-accounts and io, and what the fake crates import.

## 2. Scores (tree as a whole)

| Criterion | Score | Why |
|---|---|---|
| Cleanliness | 4 | Little dead code. Noise is local: about 160 copies of one doc line, and constants retyped per file. |
| Organization | 3 | The layout rules are documented, but each is applied 2–4 ways. Shared harness parts live inside test crates and are reached through `#[path]` and `include_bytes!` across crates. |
| Taste | 4 | Careful and consistent idiom. The step machines read like the design. |
| Naming | 3 | The same concept has several names across components (has-work, validity, lifecycle, errors, queues). |
| Primitives | 3 | lib, io, http and tls are strong. The component layer is missing its shared pieces. |
| Elegance | 4 | The machines follow the problem's shape, and the component boundaries match the docs. |
| Economy | 3 | About 2.5–3.5k lines of cross-crate near-copies (findings 1–4), small next to 481k but concentrated. |
| Complexity | 4 | Mostly just enough. Close/abort is encoded four ways, including an `Option<bool>` used as a three-state value. |

## 3. Strengths

- The component boundaries are uniform where it matters: `new → Result`, then `down`/`up`/`fire`/`next_deadline`/`reclaim`, with `Close` answered by `Closed`.
- The step crates hold the subset everywhere sampled: exhaustive matches, the `Env` threading, `expect` messages that name the invariant.
- The world harness is real infrastructure that tests reuse instead of mocking: `skein-world`'s `Expectations` and `stream::Wire`, and the `Host` trait.
- The test names are behaviour sentences, e.g. `a_refused_github_key_stops_the_forge`.

## 4. Findings, ranked by payoff

### 1. Four hand-rolled "socket + optional TLS + HTTP" stacks (Primitives, Economy)

Each component carries its own copy of the same machinery: socket, `Option<tls::Client>`, the TLS and plaintext queues, routing down to io or through TLS, handshake, chunked upload, and close/abort with a `close_sent` flag.

**Evidence:**
- **forge-connection** (`skein-forge-connection/src/connection.rs`): struct at 116–148, `connect` 352, `stream` 370, `http_down`/`json_down` 385–396, `pump` 397–449, `send` 500, `tls_event` 509, `upload` 779–811, `close` 920–936, `ready` 943.
- **forge-setup** (`skein-forge-setup/src/host/connection.rs`): struct 28–50, `connect` 207, `stream` 239, `pump` 286–355, `upload` 477–505.
- **oauth-accounts** (`skein-oauth-accounts/src/exchange.rs`): `Web` at 171–556, with `up` 260, `progress` 349, and `close`/`close_progress`/`close_socket` 475–536.
- **llm-connection** (`skein-llm-connection/src/call.rs`): `connected` 177, `cipher_up` 250, `close_socket` 305, `close_transport` 322.

**The copies have already drifted:**
- Only oauth-accounts (`exchange.rs:494`) and llm-connection (`call.rs:326`) send TLS `close_notify`. The two forge components close the socket directly (possibly intended; unsure).
- The queues have three naming schemes: `tls_up/tls_down/http_up/http_down`, `tls_events/cipher_down/llm_events/plain_down`, and `events/requests/plain/tls_events/cipher`.
- Close/abort is encoded four ways: `closing: bool` + `Phase`; `closing: Option<bool>` + `close_sent`; a `Control` enum; and `abort: bool` + `Phase` + `close_sent`.

**`Transport` is defined four times:**
- `skein-llm-connection/src/endpoint.rs:11` is identical to `skein-oauth-accounts/src/accounts.rs:38`.
- `skein-forge-connection/src/settings.rs:66` is identical to `skein-forge-setup/src/host/settings.rs:77`, and `host()` is byte-identical in both (`settings.rs:75` and `host/settings.rs:87`).

**Forge-specific copies:**
- Method mapping appears four times (`connection.rs:239`, `:272`; `host/connection.rs:99`, `:170`).
- JWT minting appears twice (`skein-forge-connection/src/app.rs:24` and `host/connection.rs:734`).
- forge-setup repeats its 22-field struct literal (`host/connection.rs:106` and `:177`).

**Fix:**
- Add one concrete transport struct in skein-tls, or a small crate over io and tls. It holds the socket, the optional TLS client, the cipher queues and the close state, with `connected`, `up`, `down`, `close(abort)`, `pump(io)` and `has_work`.
- Add a shared `Endpoint { address, transport }` with `host()`.
- Add a `skein_forge::github::jwt::window(wall)` helper.

**Size:** about 750 lines of plumbing become about 200 shared; net about −550. Abort-after-close would then have one implementation.

### 2. The hosted-process boilerplate is reinvented in test crates (Economy)

**The io pass:** `Face::up`/`down`/`work_pending`/`is_empty` in `skein-fake-peers/src/face.rs:64–133` is `pub(crate)`. It is retyped in:
- `tests/oauth-accounts/src/world.rs:667–745`
- `tests/llm-connection/src/simulated.rs:149–220`
- `tests/tls/tests/suite/fake_peer.rs:94–200`
- `tests/ui-web/src/harness_peer.rs:128–170`

`tests/fake-peers/tests/suite/stages.rs:4` even compiles `face.rs` a second time through `#[path]`. `const ROOM: u32 = 64` appears in 8 files.

**Delegating `enum Process` Host impls:**
- Five go through `host()`/`host_mut() -> &dyn Host` (tls `fake_peer.rs:225`, `echo/tests/end_to_end.rs:20`, and three in `tests/world/tests/suite`).
- At least five write an explicit match in every method, 44–68 lines each: oauth-accounts `world.rs:815`, `sign_in.rs:548`, `sign_in/cuts.rs:69`, `private_cuts.rs:37`, and llm-connection `simulated.rs:285`.

**Trivial referees:** about 21 have an empty `act`, 22 have `overdue → None`, and 17 have `next_deadline → None` (e.g. `tests/io/src/append.rs:278–292`).

**Fix:**
- Make `Face` a public io-pass struct in skein-world.
- Add `impl<H: Host + ?Sized> Host for Box<H>` in skein-shell, so every delegating enum shrinks to a single `host_mut`.
- Give `Referee::act`, `next_deadline` and `overdue` defaults, as `starved` and `last_word` already have.

**Size:** about −900 to −1,200 test lines.

### 3. Small missing primitives, each written privately 4–9 times (Primitives)

- **Earliest-deadline fold:**
  - Four private copies: `skein-lib/src/requests.rs:718`, `skein-oauth-accounts/src/component.rs:1239`, `skein-llm-connection/src/deadlines.rs:159`, `skein-forge-setup/src/host.rs:376`.
  - Seven inline `earliest.is_none() || Some(t) < earliest` checks in `skein-forge-connection/src/component.rs:545–587` and `hooks.rs:116`.
  - Nested-match versions in `skein-llm-connection/src/component.rs:584–619`, even though that crate has its own `earlier`.
- **Header construction:** `fn header/field(name, value) -> Header` has 9 definitions (`skein-llm/src/{openai,anthropic}/identity.rs`, forge-connection `connection.rs:981`, forge-setup `connection.rs:655`, and 5 test crates). There are also about 35 inline `Header { name: bytes::copy_of(..), value: bytes::copy_of(..) }` literals.
- **Forge `Method`:** two identical enums (`skein-forge/src/forgejo/request.rs:5`, `github/request.rs:8`), mapped to `skein_http::Method` four times in production and back three times in tests.

**Fix:** `skein_lib::time::earliest(&mut Option<Time>, Time)`, `skein_http::Header::new`, and one `skein_forge::Method`.

**Size:** about −200 lines, and one name for each concept.

### 4. Shared harness parts live in `tests/` and spread by path tricks (Organization)

**Test crates used as libraries by other test crates** (all depended on by relative path):
- `skein-tls-world::pki` is used by forge/machine, forge/setup, llm-connection and oauth-accounts.
- `skein-llm-world` is used by 6 crates.
- `skein-forge-machine-world` is used by forge/setup and forge/protocol.
- The step crate `skein-llm` takes `skein-json-world` as a dev-dependency.

**Includes that cross crate boundaries:**
- `skein-fake-peers/src/transport.rs:16` and `transport/native.rs:277–279` use `include_bytes!` on `tests/tls/fixtures`.
- `#[path]` reaches across crates in:
  - `tests/fake-peers/tests/suite/github_memory.rs:2`: another test crate's fixtures, behind `#[expect(dead_code)]`.
  - `tests/echo/tests/suite/scenarios.rs:249`: the example's `io_stage.rs`.
  - `skein-codec/src/lib.rs:16`: `tests/codec/src/generated/v1.rs`.

**Why it matters:** `testing-strategy.md` §7 says a harness moves into skein once a second user needs it. `testing.md` §6 says `testing/` holds what more than one crate shares.

**Stream endpoints are reimplemented too:** `skein-world::stream::Wire` is 120 lines with 8 users. Yet `tests/forge/machine/src/pipe.rs` (`End`, 247 lines), `tests/protocol/src/wire.rs`, `tests/tls/src/drive.rs`, `tests/http/src/client_world.rs:479` and `tests/llm/src/fake.rs:206` each build their own. About 20 files hold an `Option<(Read, u32)>` demand. Some of them model withdrawn demands, so they may not all merge (unsure).

**Fix:**
- Move the PKI to `testing/skein-test-pki`.
- Move the LLM fixtures to a `testing/` kit.
- Extend `Wire` with cuts, and retire `pipe.rs`.

### 5. The layout rules are written once and applied four ways (Organization)

**In-crate tests take four shapes:**
- `src/tests.rs`
- a `src/tests/` directory
- about 25 `src/<mod>/tests.rs` files
- 13 sibling `*_tests.rs` files mounted by `#[path]` (8 in oauth-accounts)

**`tests/` inside `crates/`:** three crates have one, against `testing.md` §6 ("No crate under `crates/` has a `tests/` directory"): `skein-forge` (6,654 lines, with its own `common/`), `skein-fake-forge-protocol` and `skein-fail-stop`.

**Test-crate support code** sits in `src/` (the rule), in `tests/support/` (codec, browser, ui-web, oauth-issuer, conformance/real), or in `tests/suite/support.rs`.

**Test-crate paths and names:**
- `tests/forge/*` nests by tier (machine, protocol, client, setup, github, conformance), while `tests/oauth*` and `tests/llm*` are flat.
- Two package names lack the `skein-` prefix: `forgejo-conformance` (in `tests/forge/conformance`) and `github-conformance` (in `tests/forge/github`).

**Test-crate lint policy** is hand-copied into about 36 manifests, with about 20 distinct variants:
- `tests/kv` denies neither `wildcard_enum_match_arm` nor `unwrap_used`.
- `tests/forge/*` deny 9 lints.
- 9 crates take the full step-crate workspace lints.

**Fix:**
- Pick one form for each kind.
- Move `crates/skein-forge/tests` under `tests/forge/`.
- Rename the `*_tests.rs` files to `<mod>/tests.rs`.
- Generate the test-crate lint blocks from one template; the assembly already generates the root manifest.

### 6. The documented crate graph and parts table have drifted (Organization)

Compared with the manifests, `README.md` §3 is wrong in these ways:

**Missing dependencies:**
- skein-oauth also depends on codec and tls.
- skein-fake-oauth also depends on tls.
- skein-forge-setup also depends on tls.
- skein-throwaway-forgejo also depends on fail-stop and world.
- skein-fake-person-browser also depends on http, shell, scratch, ui, fake-peers and world.

**Wrong "nothing from skein":** ui-bundle actually depends on fail-stop and lib, and codegen on fail-stop and codec.

**Absent crates:** 14 crates are not in the graph at all (fake-forge-github, fake-llm-*, fake-peers, fake-checkout, fail-stop, heap, scratch, world, conformance, echo-client, fake-channel, forge-conformance, lab-browser).

**Stale entries:**
- `skein-playground` is listed, but no such crate exists and the entry is not marked unbuilt.
- §2 says "Forgejo's built, GitHub's later", but `skein-forge/src/github` is 14.5k lines.

The root `Cargo.toml` also lists 9 fake crates twice.

**Fix:** generate §3 from `cargo metadata` in the gate.

### 7. Same concept, different names (Naming)

- **"Has a step to take":** `has_work` (7 crates), `ready` (forge-connection, forge-setup, tls and others), `is_ready` (io, kv, forge-client), and `work_pending` (`Host`).
- **Limits validity:** `programming-model.md` §6.3 prescribes `valid(limits) -> bool`. The forge family uses `check -> Result<(), Problem | Unusable | Refused>` (`skein-forge` ×2, `forge-connection/src/limits.rs:205`, `forge-setup/src/limits.rs:20`).
- **Constructor errors:** `Unusable` in three components, `EndpointError` in llm-connection.
- **Component lifecycle:**
  - llm-connection (`component.rs:24`) and oauth-accounts (`component.rs:122`) both define an identical `enum Lifecycle {Live, Closing, Aborting, Closed}`.
  - forge-connection and forge-setup use bools instead.
  - Neither forge component has `Request::Abort` (`forge-connection/src/boundary.rs:81`, `forge-setup/src/host/boundary.rs:114`), although §5.2 says a component "takes an abort as well".
- **`Host` means two things:** forge-setup's component is called `Host` (`host.rs:24`), the same name as the `skein_shell::Host` trait.
- **Limits fields:** `idle_keep` vs `idle`; `start_backoff`/`start_backoff_max` vs `backoff`/`backoff_max`.

**Fix:**
- Use `has_work` everywhere.
- Choose `valid` or `check`, and amend §6.3 to match.
- Add a `skein_lib::Lifecycle` with `close()` and `abort()` transitions.
- Rename forge-setup's `Host`.

### 8. The fakes reuse the real codecs (Primitives; breadth only)

**What the fakes import from the crates they stand in for:**
- fake-forge-protocol: 20 of 29 production files import `skein_forge`, including `forgejo::client::encode`.
- fake-forge-github: 24 of 28 (`api::Answer` ×12, `github::request::encode` ×7).
- fake-llm-protocol decodes requests with the client's own parsers (`documents.rs:145–147`, `:208`) and then translates.
- fake-oauth answers with `skein_oauth::encode_response` and `encode_error` (`lib.rs:9–12`, `:234`, `:445`). `oauth.md` documents this choice.

**Clean:** fake-llm-domain, fake-forge-domain, fake-checkout and fake-machine.

**The pattern:** the domains are independent, but the protocol faces share the client's codec, so a codec bug cancels out on both sides of the wire.

**Fix:** either amend `testing-strategy.md` §4 to allow faces to share codecs, or give the faces their own decoders.

### 9. Noise (Cleanliness)

- `/// Contract: docs/design/fake-llm.md, sections 2–5; programming-model.md, section 4.4.` appears about 160 times, mostly across `skein-fake-llm-domain/src/{api,domain}.rs` and `skein-fake-llm-protocol/src/{provider,documents}.rs`. With the blank `///` line before each, that is about 320 lines that could be one module doc.
- `const SECOND`/`SEED = 7` is retyped in 12 files of `tests/forge/machine/tests/suite`, although `src/fixtures.rs` exists.

## 5. Cross-area suspicions (unverified beyond this pass)

- **Forge dialects:** the GitHub dialect imports neutral helpers from the Forgejo one: `forgejo::document::Body` ×14, `id::unsigned` ×13, `time`, `path`, `decode::Malformed`. These belong at the root of `skein_forge`.
- **Label-driven harnesses (oauth-accounts, io, world):** there are 13 `Story` enums, and the harness branches on the label, e.g. `tests/oauth-accounts/src/world.rs:512` and `:729`, with 63 `Story::` references in `sign_in.rs`. Calls use positional bools, e.g. `sign_in::run(43, Story::Public, false, false, …)`. This goes against §10 "Rows carry data".
- **forge-setup:** `Speech` reimplements forge-connection's per-dialect `Request`/`Decoder`/`Body` enums in a different shape.

## 6. Rule costs

- **Exhaustive matches in tests:** test crates deny `wildcard_enum_match_arm`, so pulling one event out of a list costs about 12 lines of `| Variant { .. } => panic!` (`tests/forge/machine/tests/suite/github_start.rs:43–57`). A `let … else` would do, and the lint does not forbid it. Low priority.