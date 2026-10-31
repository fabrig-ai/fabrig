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