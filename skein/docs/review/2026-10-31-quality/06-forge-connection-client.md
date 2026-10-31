I read the area against your brief and the house rules. The main results:

- **forge-client is in good shape.**
- **forge-connection's core and tests are the problem.** The tests at its own tier exist in two copies.
- **The duplicated mechanism with llm-connection is real.** It is the socket-plus-TLS connection, which is also hand-written in two more crates.

All paths below are relative to `skein (at 3371027)/`.

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