# Session 02: temper on `skein-forge-client`

Put temper on skein's bounded client and its measure, give effects' purposes
to temper's forge connector, and delete temper's copy of the client. One
increment: the purposes' move is about 90 new lines (their 88-line encoding
moves from the client), under the 300 that would split it into a 02.2, so
the repin, the purposes, the client's users and the deletion land together,
in four commits.

**Needs at start:** **skein 02.2** on skein's `origin/main` (README): the
client with `effect_key` (02.1) and the measure (02.2, decision 24); and
**temper 01.3**, the component on skein's call plans. Skein 02 needs temper 01.1. Skein 02.3 and 02.4 may or may not have landed; the repin
repairs what they break (02.1, Shape).

## Read first

- skein `docs/design/forge-client.md` 2 and 7, `forge.md` 2.3 and 3.7;
  temper `docs/design/domain/forge.md` 5 and 6 (the effects and their
  purposes).
- README (sessions, Rules, Done when), `code-organization.md` 1-3 and 5-8,
  `decisions.md` 2-5, 13 and 24, Design questions 14 and 17.
- `skein/02-client.md` (02.1's `effect_key`, 02.2's “temper's side”, 02.3's
  `bind_deployment`, 02.4's key fixtures); findings `survey-02-client.md`,
  `trace-client.md`, `base-runs.md`, `base-temper-02.md`.

## What the design asks

- forge-client 7.1: the client frames opaque identity bytes into a key
  (version 1, a two-byte namespace length, the namespace, the identity); it
  never inspects the identity. The owner supplies both.
- forge 2.3 and domain/forge 6: the purposes and their tags stay with the
  connector. No key at rest changes: the keys of tags 1 to 5 keep their bytes.
- forge 3.7: one fixed-width measure, the dialect's and the client's.

## Choices this plan makes

- Counts are **at the base** (temper dd12b8e). After temper 01.1 the files
  that name only the vocabulary name `skein_forge`; this session moves the
  lines that name the client's state. At the base 716 lines in 103 files
  name the client (193 by the crate's name, 523 through the files' alias);
  561 in 48 files reach its state and change: `findings/base-temper-02.md`.
- No alias: no `as` on the kit's crate, no `mod`, no re-export. Every
  `use` and path names `skein_forge_client`; rustfmt reflows the longer name.
- `EffectPurpose` is the connector's: `temper_engine_domain_forge::purpose`.
  Temper hands the kit no new value here: the namespace stays the
  deployment's 16 bytes (`bind_deployment`), the writers stay `Config`'s.

## Rules that bind it

- The repin is the first commit: `cargo update -p skein-lib`, then
  `cargo check --workspace --all-targets`. A Cargo.lock conflict takes
  `origin/main`'s and runs the update again, never a hand merge.
- No behaviour change, no raised bound, no loosened assertion. The sole
  allowance exception is the derived owned-example `engine.memory_bytes`
  correction below (README, Derived owned-example memory); operational
  admission caps and limits stay unchanged. Moved tests keep their names.
  New units under 20 ms, worlds under 0.5 s, unoptimized.

## Increments

### 02.1 The repin, the purposes, the client's users, the deletion

Scope: forge-client 7.1 and forge 2.3, 3.7; domain/forge 5 and 6; domain/root 6
  (a landed change's words).
Needs: skein 02.2 and temper 01.3 on `origin/main` (README); for its naming
  proof and its landing, jig01.s1's boundary check repair on jig's `origin/main`
  (README, Rules: the boundary check's jig repair). Skein 02.3's `bind_deployment`
  guard and 02.4's frame checks may be on it; if they are, the repin commit
  repairs the two places below.
Files: Cargo.toml (root: delete the member `crates/temper-engine-domain-forge-client`
  and its `path` entry, add `skein-forge-client = { git = "https://github.com/fabrig-dev/skein.git" }`
  beside the other skein entries) and Cargo.lock; the 15 manifests naming
  `temper-engine-domain-forge-client.workspace = true` at the base:
  `crates/temper-{engine-domain,engine-domain-forge,engine-protocol-forge,engine-smith,protocol-store,service}`,
  `tests/{client,codecs,conformance,system,engine/domain,engine/forge,engine/forge/client,engine/protocol-forge}`,
  `tools/forgejo-conformance` (the line becomes `skein-forge-client.workspace = true`;
  `tests/client`, whose `front.rs` uses only `api`, drops it).
  Changed Rust: the 48 files with client state at the base (561 lines; by crate in findings).
  New: `crates/temper-engine-domain-forge/src/{purpose.rs,tests/purpose.rs}`,
  with `pub mod purpose;` in `lib.rs` and `mod purpose;` in `tests.rs`.
  Deleted: `crates/temper-engine-domain-forge-client/`.
Repin compatibility Files (README): `crates/temper-service/src/{worker_route.rs,
  tests/worker.rs}`, `crates/temper-worker-domain/src/tests/fixtures.rs`,
  `tests/worker/domain/src/protocol.rs`. Set the three Start literals to
  `live: None`; in worker_route::{process,channel}, explicitly ignore only
  `OpenEvent::Live` beside existing ignored variants; refresh codec-derived values.
Derived fixture Files: `docs/examples/owned.json`; existing regression in
  `crates/temper/tests/startup.rs::serve_prepares_once_on_the_owned_example`,
  whose body stays unchanged. Its consumers are `crates/temper/tests/support/owned.rs`
  (`EXAMPLE`), `tests/{startup,configuration,process}.rs` under that crate,
  and the copy command in `docs/guide/box.md`; preserve them and their assertions.
HTTP room Files (commit 7): `crates/temper-service/src/{clients.rs,tests/server.rs}`,
  already named in its Shape below; the shared `unclaimed` branch covers both
  the existing 404 and the new 503.
Shape: seven commits, each builds and passes its crates' tests.
  1. **The repin**: update and check; repair there what skein's steps broke
     since the last repin (at the base the check fails on `Op::Lock` and jig's
     `Event::Locked`, which the tools plan's first repin, landed before this
     plan's first, repairs: findings).
     It also takes jig's boundary-check repair (README, Rules); a commit 1
     made before that repair landed is amended to take it.
     For this first compatible repin, also await jig01.s2 and take its
     landed compatible jig source, retaining a newer compatible revision.
     README's Settled smith adapters rule assigns this repin the three
     Start live:None literals and explicit ignored OpenEvent::Live arms;
     refresh the derived encode_start proof with the landed codec. This
     follows live-text's decision and next-repin assignment.
     Refresh reached copies/derived values and name them in the commit
     body; keep all bounds, assertions and existing proofs, subject only to
     the derived owned-example allowance correction in commit 3 below.
     If skein 02.3 is on `origin/main`, `bind_deployment`
     answers false once a Restore, Keep, Make or Remake was taken: the one
     caller, `restart.rs`'s deployment row (via `domain.rs`'s wrapper), already
     calls `restart_refuse` on false; a test restoring a forge row before its
     deployment row would take a restart's order (at the base none:
     `rg -n bind_deployment` finds only `restart.rs:397` and the wrapper). If 02.4
     is on it, the key fixtures of `tests/engine/forge/client/tests/{simulation,
     fuzzy_simulation}.rs` (`deployment:goal`, `[entry]`, the world's namespace
     `world`) take the frame `\x01\x00\x05world` + identity, as 02.4 says.
     The plans now measure with skein 02.2's widths, so this commit also puts
     `crates/temper-engine-protocol-forge/src/tests/{reads.rs:489,contents.rs:259}`
     on `skein_forge::measure::answer` (the crate already depends on skein-forge)
     and changes the two `contents.rs` values (Test shape).
  2. **The purposes**, with the root `skein-forge-client` entry and the
     connector's manifest line it needs: `purpose.rs` (sketch, compiled with a
     stand-in for the kit's `effect_key`; clippy clean). The tag, then each field big-endian,
     is the identity; the kit frames it. Sizes: 26, 25, 11, 18, 11 + resource.
     ```rust
     // crates/temper-engine-domain-forge/src/purpose.rs
     #[derive(Clone, PartialEq, Eq, Hash, Debug)]
     pub enum EffectPurpose {
         ProjectionHistory { goal: u64, task: u64, position: u64, family: u8 }, // tag 5
         Call { task: u64, attempt: u64, completion: u32, position: u32 },      // tag 1
         Step { task: u64, purpose: u16 },                                      // tag 2
         Projection { goal: u64, part: u8, number: u64 },                       // tag 3
         Release { task: u64, resource: Box<[u8]> },                            // tag 4
     }
     impl EffectPurpose {
         /// Tag, then fields big-endian; `ProjectionHistory`: tag, family, goal, task, position.
         #[must_use] pub fn identity(&self) -> Option<Box<[u8]>>;
         /// `skein_forge_client::effect_key(namespace, &self.identity()?, max_bytes)`.
         #[must_use] pub fn key(&self, namespace: &[u8], max_bytes: u32) -> Option<Box<[u8]>>;
     }
     ```
     Built with `skein_lib::List` (no `Vec`: the lints ban it). Callers:
     `domain.rs`'s `projection_key`, three calls, become
     `EffectPurpose::ProjectionHistory { goal, task, position, family: 1 }.key(namespace, l.client.op_bytes)`
     (family 2 with `position: revision`) and
     `EffectPurpose::Projection { goal, part, number }.key(..)`; `forge_route.rs`'s
     `effect_key(domain, key)` builds `Call { task, attempt, completion, position }`
     and `.key(&domain.core.counters.deployment().id, 44)`, `expect("fixed call key fits its frame")`.
  3. **The users**, in the other 14 manifests and the 48 files: every `temper_engine_domain_forge_client` and
     `client::`/`forge_client::`/`d::` path to client state becomes `skein_forge_client::`;
     vocabulary stays `skein_forge`. Every alias of the crate goes (`as client`,
     `as forge_client`, `as d`, `{self as client, ..}`: `rg -n
     'temper_engine_domain_forge_client( as |::\{self as)' crates tests tools` finds 43 at the base). The nine measure callers change too:
     `effect_bytes` (domain-forge `lib.rs`, three) -> `skein_forge_client::measure::effect`;
     `stored_bytes` (`lib.rs`) -> `skein_forge_client::measure::stored`;
     `answer_bytes_unbounded` (`lib.rs`; `temper-engine-domain`'s `store.rs`
     `call_answer_bytes`) -> `skein_forge::measure::answer`; `domain.rs`'s
     `keep_named_answer`: `skein_forge_client::measure::answer_rows(value,
     limits.client.rows)` and `skein_forge::measure::answer(value)` compared with
     `limits.client.answer_bytes`: admitted only if `answer_rows` is true and the
     measure is `Some(b)` with `b <= answer_bytes`, else `Err(answer)` as at the base
     (an `Err` or a non-`Read` payload stays admitted). If an earlier repin already
     repaired these (`rg` for the old names finds none), only the users move.
     The complete users cutover also refreshes the owned example's derived
     `engine.memory_bytes` from `system::startup_worst_case`, with all inputs
     and operational limits unchanged (README, Derived owned-example memory).
     Skein02.3's `Domain::identity_used` grows the inline client from 592 to 600 bytes,
     carried once by `system::memory`'s `size_of::<Service>()`: complete sum
     78,675,187,547,016 becomes 78,675,187,547,024 on the checked tree. Record
     both complete sums and this cause; after a rebase rederive the whole sum,
     rather than applying a stale delta. Refresh any further reached copy or
     derived value and list every user in the commit body.
  4. **The deletion**: the crate's directory, its member and its path entry.
  5. **The landing's words** (design `domain/root.md`, 6: "its words the
     merge commit's id as text: in lowercase hexadecimal, as a delegate's
     commits are (below), shortened to its first digits, as a short id is,
     when the task's contract bounds its words under the id's 40 or 64"), the
     repair the diagnostic exception below returned for, made on the
     preserved four-commit checkpoint. In
     `crates/temper-engine-domain/src/route/forge_route.rs`, `change_decision`'s
     `Decision::Finish { merge }` result takes `words: landing_words(&merge,
     bound)` in place of `Box::from(&merge[..])` (the padded commit's 32
     raw bytes, which jig's `documents/small.rs::text` asserts are UTF-8);
     `bound` is the `words` of the task's own `tasks::Contract::Change`,
     read as `link.rs:26` reads a task's contract
     (`domain.core.tasks.task(task)`), 0 for any other contract. The new
     `fn landing_words(merge: &skein_forge::api::Commit, bound: u32) -> Box<[u8]>`
     beside `commit_text` keeps `commit_text(merge)`'s first
     `min(bound, its digits)` digits. `commit_text`, its parameter callers,
     every contract and bound, and jig stay as they are.
  6. **The stories' endings** (design `domain/testing.md`, 2.6, "Its
     endings": every end of `temper serve` but the restart's crash "is a
     deployment's healthy stop (skein's `shell.md`, 13): the test sends
     `Terminate`, … and the binary exits 0 with none of its tree
     running"; skein's `examples.md`, 6, "A crash on purpose"), the
     repair the process-cleanup diagnosis returned for, on the
     five-commit checkpoint. In `crates/temper/tests/support/mod.rs`,
     `run_story` takes an `Ending`, `Stop` or `Crash`: `run` stops, and
     `run_until_landed`, whose one caller is `process.rs`'s `landing`
     before its restart, crashes. A stopped story's binary starts as now
     (`serve`: `Binary::start`, `EndsWithBinary`); `ChatJudge::act` sends
     it `Signal::Terminate` where it sent `Signal::Kill` (line 396), and
     `observe` asserts that once it has exited it exited `Exit::Code(0)`.
     A crashed story's binary starts with `Binary::start_with_tree(…,
     Mode::Pipes, 65_536, Expectation::Supervise, false)`, as
     `support/boxed.rs` starts the box; `act` sends it
     `signal_tree(Signal::Kill)`, `observe` asserts
     `!stderr_overflowed()` (an overflow fails under `EndsWithBinary`),
     and `run_story` asserts its `counts()` present before it returns, so
     `landing`'s second `serve` starts only once nothing of the first
     runs. `support/sign_in.rs`'s `Judge::act` (line 257) sends the engine
     `Signal::Terminate` in place of `Signal::Kill`, and its `observe`
     asserts the engine's exit, once it has one, `Exit::Code(0)`. No test
     is added, renamed or loosened.
  7. **The mount's room** (jig's `protocol/client.md`, 17.4, "The engine
     end's room": "The application asks for its room before it hands the
     engine end a call, and answers a call under the prefix that finds
     none itself, as it answers a call no route claims: `503`, with no
     body, closing the connection"; design `client/README.md`, 1), the
     repair the HTTP call-table diagnosis returned for, on the six-commit
     checkpoint, once jig01.s3 has landed (README, Rules). It repins jig
     onto that landing; then `crates/temper-service/src/clients.rs`'s
     `server_requests` hands a call `http::claims` to `mount.down` only
     while `mount.room()` is not 0. One that finds none goes into
     `unclaimed`, as a path no one claims does, answered
     `Respond(Response { status: 503, headers: [], body: Body::None,
     close: true })`. In that shared branch, both the existing 404 and
     the new 503 send `Request::Discard` before `Respond`, including for
     an empty body: skein's `http.md`, 5.3, says a response without reading
     the body discards it, or `Done` never comes. Keep existing 404 headers
     and both statuses, empty response bodies and `close: true`. Remove
     `unclaimed` and call `server.done` only on `Done`, `Failed` or `Closed`,
     as now; neither responding nor discarding retires the entry itself.
     One test in `crates/temper-service/src/tests/server.rs`,
     `a_call_past_the_mounts_room_is_answered_busy`: with the mount's
     `calls` (read from the limits, not a literal) each holding a read
     head whose body is withheld, the next call under `/jig` is answered
     503 with nothing reaching the root; once one of them is closed and
     the mount has handled its end, a further call is mounted. Observe
     the busy response's terminal and retirement in this same test; keep
     its other assertions and add no test. Refresh further reached copies
     within this step and list users in the commit body. No sum is
     re-derived: `clients::worst_case` already counts `unclaimed` at the
     server's connections, and a 503 with no header is within the
     response head's bound, so `system::memory`, the startup worst case
     and the owned example's memory stay.
  The HTTP call-table diagnosis and its artifacts stay as they are.
  Diagnostic exception for the pinned Jig UTF-8 panic: investigate the retained
  failed retry in run `logs/temper-02/process-attempt2/process`, comparing the
  failing four-commit tree and its actual pins with the last passing source.
  Scope is scratch-only provenance tracing into
  `jig-domain-core/src/documents/small.rs::text` and the reached callers and
  application translations; check each discovered name in the frozen sources.
  Budget: 20 minutes and three executable diagnostic attempts through pinned
  explicit bash script files; compile/setup failures consume attempts,
  and queue waits grant none. Use retained artifacts and deterministic lower-layer
  probes; no further full process run, recording or gate is authorized by this
  exception. Preserve the original artifacts and implementation checkpoint.
  Return at most 15 lines naming full source/pin hashes, the invalid value's
  provenance, smallest reproducer and proposed repair owner/location, or the
  remaining uncertainty. No repair lands here: return the report for an upstream
  scope ruling or design question. Keep UTF-8 validation, payload bytes, errors,
  assertions, admission limits and existing proof requirements unchanged. At
  cutoff identify any active task/source/target owner and retain its target until
  it ends; refresh any further reached copy in the diagnosis map and list it.
  Test shape: **2 tests move, 74 leave with the crate.** `cargo nextest list -p
  temper-engine-domain-forge-client` shows 76 at the base (0.084 s); the two
  purpose tests keep their names and move to `tests/purpose.rs`, adapted to
  `purpose.key(&[1; 16], N)`: `effect_keys_are_stable_by_purpose_and_separate_deployments`
  (44-byte cap, 43 refuses) and `projection_history_keys_keep_task_position_and_history_family`
  (45 bytes, 44 refuses). Skein 02.1 holds the other 74. Three new units, <20 ms:
  `identity_bytes_pin_tags_one_to_five` (one row per tag: `Call{7,2,3,4}` ->
  `[1, 0,0,0,0,0,0,0,7, 0,0,0,0,0,0,0,2, 0,0,0,3, 0,0,0,4]`; `Step{7,9}` ->
  `[2, 0*7,7, 0,9]` (`0*7` is seven zero bytes); `Projection{1,2,3}` -> `[3, 0*7,1, 2, 0*7,3]`; `Release{7,"ab"}`
  -> `[4, 0*7,7, 0,2, b'a', b'b']`; `ProjectionHistory{1,2,3,family 1}` -> `[5, 1,
  0*7,1, 0*7,2, 0*7,3]`), `a_call_key_is_the_framed_forty_four_bytes` and
  `a_history_key_is_the_framed_forty_five_bytes`, each asserting the whole key
  against a literal and `None` one byte under the cap. Namespace `[7; 16]`:
  ```rust
  #[rustfmt::skip]
  const CALL_KEY: [u8; 44] = [ 1, 0, 16, 7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
      1, 0,0,0,0,0,0,0,7, 0,0,0,0,0,0,0,2, 0,0,0,3, 0,0,0,4 ];
  #[rustfmt::skip]
  const HISTORY_KEY: [u8; 45] = [ 1, 0, 16, 7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
      5, 1, 0,0,0,0,0,0,0,1, 0,0,0,0,0,0,0,2, 0,0,0,0,0,0,0,3 ];
  ```
  The vectors are the base client's bytes (checked in the scratch against
  the base `effect_key`, all five tags). Commit 5 adds one unit, <20 ms, in
  `crates/temper-engine-domain/src/route/forge_route/tests.rs` (1 at the base),
  `landing_words_are_the_merge_id_cut_to_the_contract_bound`, on that file's
  `SHA1` and `SHA256`: (`SHA1`, 64) and (`SHA1`, 40) give its 40 digits;
  (`SHA256`, 512) and (`SHA256`, 64) its 64; (`SHA256`, 32) its first 32;
  (`SHA1`, 7) `abcdef0`; (`SHA1`, 0) nothing. It also tightens one assertion:
  `tests/engine/domain/tests/forge_routes.rs`'s
  `a_worker_frozen_past_its_grace_resumes_with_a_push_in_hand_and_lands_nothing_twice`,
  whose delegate's contract bounds words at 32 (`change_world_in`, line 834),
  asserts that its one Done result's words are the first 32 lowercase
  hexadecimal digits of the stored change's `State::Landed { merge }`, besides
  "one result after recovery". No other test changes name or value but the three below.
  skein 02.2 puts the plans' `reads::size::answer` on the measure's widths
  (decision 24), and its "temper's side" names the three component tests
  that follow: `the_running_answer_size_agrees_with_the_clients_measure`
  (`tests/reads.rs`) passes unchanged; `contents_sizes_agree_with_the_clients_measure`
  (`tests/contents.rs`) expects pull files 68 and comparison 100 (52, 84), and
  `raw_sides_at_the_answer_bound_fit_and_one_byte_more_is_too_large` sides of
  4031 and 4032 (4047, 4048). Those are the only component-test values that change. If a side
  disagrees otherwise, stop and return the question; never loosen an
  assertion, drop a row or raise a bound.
Proof: unit; `cargo test --locked --offline -p temper --test startup
  serve_prepares_once_on_the_owned_example -- --exact --nocapture`: the same
  existing regression passes with only the derived allowance correction,
  no new test or assertion change. Complete the required reverse dependents
  and all remaining joint/process/gate proofs; the scratch diagnostic is not
  a substitute for them.
Proof: unit; `cargo nextest run -p temper-engine-domain-forge tests::purpose`:
  5 passed, each <20 ms (0.002 to 0.003 s in the scratch; the two moved
  ones took 0.004 s and 0.006 s at the base).
Proof: unit; `cargo nextest run -p temper-engine-domain forge_route::tests`:
  its count at the step's start plus 1 (2 from the base's 1), each <20 ms.
Proof: unit; `cargo nextest run -p temper-engine-domain-forge`: origin/main's
  count at the step's start plus 5 (27 from the base's 22, 0.015 s), <1 s.
Proof: joint; `cargo nextest run -p temper-engine-domain-forge
  -p temper-engine-domain -p temper-engine-smith -p temper-protocol-store
  -p temper-service -p temper-engine-forge-client-world -p temper-engine-forge-world
  -p temper-engine-protocol-forge -p temper-engine-protocol-forge-world
  -p temper-engine-domain-world -p temper-conformance-world -p temper-codecs-world
  -p temper-client-world -p temper-system-world -p forgejo-conformance`: the
  set's count at the step's start plus 7 (928 from the base's 921, 4.823 s; the
  client's 76 are not in it): the five purpose tests, commit 5's unit and
  commit 7's test are the only change; <15 s. A failure of a profile, limits or
  size test after the measure is a design question: return it, change no bound,
  except for the fully traced derived owned-example allowance correction
  explicitly authorized above. An unexplained difference still returns.
Proof: joint; the same command with `--profile fuzzy`: its count at the step's
  start, unchanged (39 at the base, 20.410 s, its slowest `fuzzy_walking` 20.4 s); <60 s.
Proof: process; `TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  scripts/proof-one-process.sh` (README, Rules): its count at the step's
  start passed (19 at the base), <5 minutes (145.694 s in `base-runs.md`, 149.083 s alone here). Once: the keys
  at rest and the engine's calls go through the new client. With commit 6,
  each story's `temper serve` stops and exits 0 with none of its tree
  left (`EndsWithBinary`), but the landing's first, which crashes whole
  (`Supervise`) and settles before the restart. A process left at a
  stopped binary's exit ("binary exited while tree processes still ran")
  is temper's shutdown's (`domain/root.md`, 9.2, and io's group close),
  not a load failure to rerun: return it as a finding, with each PID's
  command. One left by the crash's kill is skein's kit's: return it as a
  question. With commit 7, a call the mount has no room for is answered
  503 and the client sends it again; a story stalled on 503s until its
  deadline is a call the mount never lets go, jig01.s3's finding, never
  a bound to raise.
Proof: joint; `scripts/check.sh`: origin/main's focused count at the
  step's start less 69 (the client's 76 out, 7 in: 1810 from the base's 1879) and
  its fuzzy count unchanged (50 at the base; 3.754 s and 16.803 s), clean
  clippy and format, focused <15 s, fuzzy <60 s.
Don't: add an alias, a re-export or a `mod client`; change
  `tests/client/tests/names.rs` or exempt a crate there; change a tag, a field's
  order or width, or any key at rest; change an admission bound, a limit or a
  test's value to pass, beyond skein 02.2's three and the derived owned-example
  allowance correction above; write `temper` into skein; move `EffectPurpose` into
  skein or give the kit a purpose; touch jig's own, different
  `jig_domain_core::EffectPurpose` (`forge_route.rs` names it as an effect's
  `origin`); give a landing's words more digits than its contract's bound,
  raise a contract's words bound (the tests' 32 stay) or refuse a bound
  under the id's digits; change `limits::server`'s connections or jig's
  `clients_http`, loosen the mount's insertion assertion, count mounted
  calls in temper (the room is the mount's), or add a `Retry-After`; relax jig's UTF-8 assertion, or rewrite, migrate or
  delete the run's retained failed artifacts (`logs/temper-02/`): their
  stores keep the raw words, test artifacts that no fixed tree restores;
  end a story's healthy binary with `Binary::signal(Signal::Kill)`, give a
  stopped story's binary an expectation other than `EndsWithBinary`, crash
  the leader alone (`signal` or `signal_group` on the crashed binary), or
  add a backstop kill or a longer deadline for a tree to settle.
Size: about 680 lines moved (changed callers counted as moved: 561 lines
  of client state in 48 files, the callers among them; 88 of the purposes' encoding, the client's
  `identity.rs`; 31 of the two moved tests), 90 new (the three new tests and
  their vectors about 72, `key` and the module's lines about 15, the manifest
  and root entries about 3; commit 5 about 35: `landing_words` and its use
  about 15, its unit about 15, the tightened assertion about 5; commit 6
  about 40: the ending and the crash's start about 25, the two judges'
  signals and assertions about 15; commit 7 about 45: the room check
  and the busy answer about 10, its test about 35). Under 300: no 02.2.
Removes: `crates/temper-engine-domain-forge-client/`, 7,209 lines at the base
  (4,913 source, 2,283 test, 13 manifest; about 6,757 after 01.1 took the
  vocabulary and `locate.rs`): its 74 other tests live in skein; its
  `EffectPurpose` and `effect_key` live on in `purpose.rs`.

## Done when

- `cargo check --workspace --all-targets` prints `Finished`, exits 0.
  The gate (`scripts/check.sh`) is required before merging.
- `rg -n 'temper_engine_domain_forge_client|temper-engine-domain-forge-client' Cargo.toml Cargo.lock crates testing tests tools -g '*.rs' -g Cargo.toml -g Cargo.lock`
  prints nothing (exit 1); at the base it prints the crate and its users.
- `rg -n 'skein_forge_client( as |::\{.* as )' crates testing tests tools -g '*.rs'`
  prints nothing (exit 1): no alias of the kit's crate.
- `rg -n 'answer_bytes_unbounded|effect_bytes|client::(answer_bytes|stored_bytes)\b' crates testing tests tools -g '*.rs'`
  prints nothing (exit 1): the old counters are gone.
- `rg -l 'enum EffectPurpose' crates testing tests tools -g '*.rs'` prints
  one path, under `crates/temper-engine-domain-forge/src/`, and nothing else.
- `python3 -c 'from pathlib import Path; print("client_copy_absent=" + str(not Path("crates/temper-engine-domain-forge-client").exists()))'`
  prints `client_copy_absent=True`. At the base it prints False.
- `cargo nextest run -p temper-engine-domain-forge tests::purpose`
  shows 5 passed.
- `rg -n '\.signal\(Signal::Kill\)' crates/temper/tests` prints
  nothing (exit 1): no story ends its binary with a leader's kill. At the
  base it prints `crates/temper/tests/support/mod.rs:396` and
  `crates/temper/tests/support/sign_in.rs:257`.
