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