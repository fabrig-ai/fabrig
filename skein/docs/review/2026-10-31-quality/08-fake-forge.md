**Fake forge code-quality review**

All paths are relative to the tree root `skein (at 3371027)/`.

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