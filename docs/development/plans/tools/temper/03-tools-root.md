# Session 03: the tools' root

temper repins onto jig's named send, turn read and deadline spans (03.1), then builds `temper-client-tools`' root,
runner and rendering: `me` and `task N` (03.2), other reads (03.3), and jig's acts under named or drawn keys (03.4).
It provides **the tools' root**. Needs at start: **the tools' table**. The forge's words, adopter and parts in reads
are temper 11's, **the tools' adopt**. temper 04 adds waits; the next plan hosts the root, as `temper ctl`, and connects it to the client world.

## Read first

- temper `docs/design/client/tools.md`: 4, 5.2, 5.3, 6.1 to 6.4, 7, 8.2 (steps 3 to 6), 11.1, 12, 13; `docs/design/client/README.md`: 3.1, 4.6.
- jig `docs/design/client/domain/client.md`: 4, 6, 7, 8.1, 19, 20; `docs/design/client/domain/root.md`: 3, 5, 15; `docs/design/client/domain/contract.md`: 3.1, 5.2, 5.3, 6.
- This plan's `temper/{02-tools-table,04-waits,11-tools-forge,12-tools-help}.md`;
  `jig/{02-named-send,03-turn-read,05-deadline-spans}.md` and their landing bodies; `findings/temper-area-1.md` 3 and 4, `findings/jig-section-13.md` 2 and 3.
- `decisions.md`: design questions 21 to 34 whole (jig `62bcf71e`, temper `fe0ec316`; 29's correction supersedes its first answer) and the plan's own 2 to 4, 10 and 13.
- Until jig 05's file exists, use jig `62bcf71e`: `client/domain/contract.md` 3.1, 3.2, 3.6 and `client/domain/client.md` 19.
- Code: `crates/temper-client-domain/src/pages/chats.rs`, `Chats::start`; jig `crates/jig-client/src/shapes.rs`, `chat`, `fill`, `close_chat`;
  `crates/jig-client-domain/src/boundary.rs`, `Key`, `Held`, `Told::Kept`.
- Owners: `crates/temper-client-protocol-wire/src/lib.rs`, `Owner`, `number`, `owner`;
  `crates/temper-client-domain/src/owner.rs`, `Owner::{asker,of}`, and `src/route.rs`.
- skein foundation `development.md`: 5.7; `programming-model.md`: 3, 5.1, 6.3, 9, 10; `testing-strategy.md`: 2.1, 8, 10.

## What the design asks

- **client/tools.md 4, client.md 4.6:** a root holding `jig_client::Client`, routing by asker to runner (`Owner::Runner`, 3) and forge;
  one asker per owner, call numbers in keys' and handles' high 32 bits; no page/view or temper item under `src/jig/`.
- **5.2:** every read; provisional text only where supplied; large turns by size and a `turn N A.T` call; parts cut with `--part K --from B`.
  A turn past either read bound prints its size and “past the limit; report it”, exit 0.
- **5.3:** one keyed act, no confirmation; a draft within its bounds sent, between them proposed where allowed, past them refused without sending.
- **6.1, 6.2:** header, fields, offered acts as calls; proposable acts say to whom, refused acts why without a call; rows then cursor. One snake-case
  JSON result, integers only; one JSON object per kept result on stderr.
- **6.3, 7, 13:** named or drawn engine keys; build a named request from the call and `me`, a deadline span kept as a span.
  Exit 2 is the engine's answer alone; client-own answers exit 1, except the sized turn's 0. Impossible client answers say “tools' bug”.
- **8.2:** me live first; its party's kept requests answered and erased before the host's new call; other parties stay parked. **12:** 32 rows (jig's client wire ceiling, decisions.md, K), 32 KiB text, 16 KiB turn parts,
  1 MiB document; paths and authorities whole. Forge parts are temper 11's.

## Choices this plan makes

- **One runner asker:** `Owner::Runner.asker()`, number 3 beside Forge 1 and Pages 2, never reused. The shared wire enum owns
  the numbers; the domain's existing routing enum maps to it. Reserve call zero for the root's me watch; each call gets
  one prefix, in every key's and handle's high 32 bits, local numbers in the low 32. At Start, inspect all kept keys;
  start one past their highest call prefix (one for an empty store), then increase without reuse. Exhaustion exits 1.
- **Forge seam:** reserve `Owner::Forge.asker()` for temper 11's adopter, never hold page state in `forge::Side`. `jig::render::Parts` prints
  paths joined by `/` until temper 11 replaces that fallback with forge words.
- **Borrow documents by handle**, never across steps. Watch `Subject::Me` from Start, tell Ready when live, resolve project names there. An act
  reads its task/proposal/question/escalation and uses `facts::offer`; chat uses me's template. Supplied revisions stay the call's revisions.
- **Stable named requests:** build from the call and `me` alone; use jig 05's `shapes::chat(me, project, words)`, with no local copy or clock input.
  The engine resolves a span at Create admission or proposal acceptance; a proposal keeps its span until accepted.
  A changed template or bound in `me` can make a rerun another ask, answered as an engine key conflict.
- **Close and proposals:** `--reason TEXT`, empty by default; `tN/K` for a task proposal, `pN/K` for a party's, everywhere.
- **Kept results:** `kept: OPERATION <outcome>`; JSON has `operation`, `outcome` as the act's, no `name` field.
  Use jig's operation, or the forge's `stop`/`adopt`; neither key nor name prints, and no runner records keep names. temper 11 renders forge outcomes.
- **Client answers:** unoffered/over-bound drafts, Limit, Held, NotKept and Unknown past retention exit 1.
  Unconfirmed and a client-side key conflict exit 1 saying “tools' bug; report it as a finding” (temper 12's guide).
  Check key ownership before Send; keep jig's conflict assertion. Only an engine `Refusal::KeyConflict` exits 2.
  Held is by engine key, table-wide, including another party's parked request; exit 1, nothing kept or sent.
- **Read exits:** engine refusals exit 2; client admission, decoding and transport failures exit 1, including
  read-table/document limits. Busy, Unreachable, SignedOut and NotServed are failures; a sized too-large turn exits 0.
- **Output:** `jig::Printed` (`runner::Printed`, re-exported by `src/jig.rs`) holds status/stdout/stderr. The next plan's host, `temper ctl`, writes bytes, prefixes failures `temper ctl: ` and exits.
  Use jig's phrases through `skein-ui`; decimal KB rounded up, text times UTC HH:MM, dated off the step's day.
  JSON depends on delivery and limits alone, for temper 04's clock-independent digest.
- **Fixtures:** one module, temper 02's `src/tests/fixtures.rs`, extended here with the root harness `signed_in`, `submit`, `serve`,
  `deliver`, `done`, `answer` and `golden(name)` (reads `src/tests/golden/NAME.txt`), and by temper 04; all take temper 02's shipped limits.
  `src/jig/` holds no test (temper 02's boundary scan): the tests that print through it are in `src/tests/jig/`.

## Rules that bind it

- **Boundary/subset:** `src/jig/` names neither temper nor a `crate::`
  item outside `crate::jig`, and holds no test; no clock reads or std.
- **Seams:** forge-facing edits are mechanical repin/owner arms in domain `forge.rs` and service `family.rs`;
  adoption and words remain temper 11's, behind the existing family interface. No design, STATUS or profile edits.
- **Repin/proofs:** precise pins, at most 200 production lines besides
  Cargo.lock; unit tests under 20 ms, unoptimized, goldens byte for byte.
  This plan builds no tools world: its proofs are unit and joint, and the next plan's stories bring the tools' world.

## Increments

### 03.1 temper on jig's named send, turn read and spans
Scope: temper's jig pin moves to a landing containing jig 02, jig 03 and jig 05; its skein pin stays where it descends from that landing's own (skein 01's), and moves to it where not; its smith packages share one revision (ruling 119).
The repairs follow (development.md 5.7); step `temper-03.1`.
Needs: jig's send under a named key, landed; jig's read of one turn, landed; jig's deadline spans, landed.
Files: `Cargo.lock` is shared; `crates/temper-client-domain/src/forge.rs` is the forge hotspot, one arm:
- `Outcome::Held`: `crates/temper-client-domain/src/forge.rs` (`told`), `crates/temper-client-view/src/confirm.rs`, `crates/temper-client-view/src/pages/chats.rs`;
- `Send`'s `name`: `crates/temper-client-domain/src/{acts,pages,route}.rs`, `src/pages/{chat,chats}.rs`;
- deadline signatures: `crates/temper-client-domain/src/pages/chats.rs` (`Chats::start`), `src/tests/pages/chats.rs`;
- jig 05's callers: `crates/temper-service/src/tests/fixtures.rs` (`people_member`), `crates/temper-client-view/src/cards/words.rs`,
  `tests/{conformance/src/scenarios,engine/domain/tests/root_routes}.rs` (`tasks::New`);
- the turn read:
  - `crates/temper-client-view/src/pages/chat.rs` (`TurnProblem`);
  - `ReadValue`'s matches in `crates/temper-client-domain/src/acts.rs`, `crates/temper-client-view/src/cards/proposal.rs` and `src/pages/inbox.rs`;
  - `read_value` in `crates/temper-client-shell-terminal/src/tests/fixtures.rs` (ruling 119);
- the owned example's sums: `docs/examples/owned.json` (`engine.memory_bytes`, ruling 119);
- wire version 9:
  - `crates/temper-service/src/family.rs`, `src/family/refusals.rs`;
  - `crates/temper-service/src/tests/{clients,clients_unwatch,family}.rs`, `src/tests/fixtures/http.rs`;
  - `crates/temper-client-domain/src/tests/fixtures/{acts,chat}.rs`, `crates/temper-client-view/src/tests/fixtures/task.rs`;
  - `tests/one-process/src/referee/client.rs`, `src/referee/client/cache.rs`.
Shape:
1. **The pins.**
   - JIG_TOOLS is the ledger's landing commit containing every `jig-02.*`, `jig-03.*` and `jig-05.*` step and proof; the three final landings are its ancestors.
   - `git -C ~/src/rust/jig show JIG_TOOLS:Cargo.lock` names one skein, SKEIN01.
   -  `cargo update -p jig-domain-core --precise JIG_TOOLS`.
   - skein (ruling 119): temper's own pin at the step's start stays where it descends from SKEIN01 (at temper e10aa299f,
     `378fd8cfdeb2af90261b8710b60fc91d70131792`, 33 packages, where forge-kit's temper-05.1 moved it from `0db7b1fc`; a
     descendant of SKEIN01, jig 872bfb7c's `93052c6b`): where the jig update moved it,
     `cargo update -p skein-lib --precise <that full hash>`. Where it does not descend, `--precise SKEIN01`.
   - smith (ruling 119): one revision, the newest temper's `Cargo.lock` holds at the step's start (at temper e10aa299f,
     `538ef49a574bbaf02d57a85cc2b02d0abf1d4e3b`, 19 packages, beside `a277e922`'s 9; the two differ in nothing under
     `crates/`): `cargo update -p smith-domain --precise <that full hash>`; where cargo calls the name ambiguous, use the
     package spec its error lists.
   - Check `Cargo.lock` with `grep -o 'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' Cargo.lock | sort -u`;
     one source revision per repository, each the selected one; `cargo check --workspace --all-targets`.
2. **`Outcome::Held`.** Nothing in temper open sends a name, so it is impossible there. It joins `NotKept`'s arm in
   `confirm.rs` and `pages/chats.rs`, and `Answer::Unknown`'s in `told`.
3. **The name.** Each `jig_client::Event::Send` literal gives `name: None`.
4. **The turn read.**
   - `TurnProblem::TooLarge` takes `{ .. }`.
   - `ReadValue::Turn(..)` joins the arms for "not this document"; no match of `ReadNone` in temper needs an arm.
     The terminal fixtures' `read_value` gains `Read::Turn { .. } => None` (ruling 119).
   - `jig_client_protocol_wire::v8` becomes `v9`.
5. **Deadline spans and store families.** Drop `env.wall` from `shapes::chat` in `Chats::start` and its test. Follow jig 05's landed
   request/authority signature repairs, preserving spans; no client conversion to a wall. Its landing body names the other users.
   temper's store families are at v4 on `main` already: nothing names `people_v3` or `tasks_v3` (ruling 119).
6. **The owned example's sums** (ruling 119). `docs/examples/owned.json`'s `engine.memory_bytes` rises by needs minus has,
   as `temper::startup`'s `serve_prepares_once_on_the_owned_example` reports it at JIG_TOOLS (at jig 590015b4, +370824, to
   78669037184614); the commit body names it.
7. **Any other break** follows 5.7: adapted as jig's landing adapted its own, named in the report, or raised as a question.
Test shape: no new test. Existing tests keep their names and expected values (5.7).
Proof: joint, the gate (`scripts/check.sh`), whose `tests/one-process` referee speaks wire version 9.
Don't: give `Held` words or behaviour; send a name; read a turn; hand-edit `Cargo.lock`.
Size: at most 200 lines besides `Cargo.lock`.
Removes: temper's jig pin at the step's start, its skein pin where SKEIN01 replaces it, and its older smith revision;
`jig_client_protocol_wire::v8`.

### 03.2 The root and the runner, with `me` and `task N`
Scope: root Start/sign-in/me live; runner call numbers and handles; text and JSON through
`skein_json::writer::Encoder` (measure then write), cut at 32 KiB with what remains;
`me`, `task N` with acts as calls, and read exits (client/tools.md 4, 5.2, 6); step `temper-03.2`.
Needs: temper on jig's named send, turn read and spans.
Files: `crates/temper-client-tools/src/root.rs` (new), `src/lib.rs`, `Cargo.toml` (`skein-ui`);
`src/jig/{runner,render}.rs`, `src/jig/render/{document,acts}.rs` (new), `src/jig.rs` (`pub use runner::Printed`);
`src/tests.rs`, `src/tests/{root,jig}.rs`, `src/tests/jig/reads.rs` (new), `src/tests/fixtures.rs`, goldens in `src/tests/golden/`.
Also `crates/temper-client-protocol-wire/src/{lib,tests}.rs`, `crates/temper-client-domain/src/{owner,route}.rs`,
`crates/temper-service/src/family.rs` (exhaustive owner routing only).
Shape:
- **Owner.** Add `Runner` to the shared wire `Owner`, `number` = 3, `owner(3)` = Runner; extend the domain's
  `Owner::{asker,of}` through that mapping. The person's root reports Runner as UnknownAsker; service `family::to_root`
  rejects it as it does Pages: neither has family records. No new owner enum, family version or per-call asker.
- **Routing.** The root steps `jig_client::Client` and routes each `Told` by its key's or handle's asker, exhaustively:
  - `Owner::Runner.asker()` to the runner (`RUNNER` is that typed owner);
  - `Owner::Forge.asker()` to the adopter, activated by temper 11;
  - any other asker to no child: a fact (root.md 5).
- **Calls.** Queue until `me` is live; use `RUNNER.asker()` for every runner call, partitioning numbers as Choices says.
  Root allocates the internal call prefix above the restored high-water mark; Event::Call.number is only the caller's correlation ID.
- **`task N`** reads `Read::Task` and prints 6.1's text: the header line, the fields, then `acts:`, one call per offered
  act in the document's order. Each act prints as its admission says:
  - Allowed: its call. A keyed tool's call ends ` --key NAME`.
  - Proposed: the call, then ` (proposed to <recipient>)`.
  - Refused: the act's name and why, in jig's words, with no call.
  - No tool in client/tools.md, 5.3 (decisions.md, M): its name in the tools' words, then ` (no tool)`, with no call and no
    admission. JSON prints the document's offers as this Shape says.
  - Bounded (ruling 119; jig's `Admission::Bounded { within: Bounds, beyond: Beyond }`): its call, then the bounds a draft is
    compared with (`within`, printed whole, as client/tools.md, 6.4, prints an authority), then beyond them ` (proposed to <recipient>)` for
    `Beyond::Proposed`, or `refused beyond them: <why>`, in jig's words, for `Beyond::Refused`. client/tools.md, 4: "each act made,
    proposable (to whom) or refused (why), with the bounds a draft is compared with".
- **Exits.** `Readout`/`ReadNone` follow Choices; expose `client` and `kept_json` for the next plan's host before Start.
- **JSON** prints the document's fields by jig's names, in snake case. A cursor prints as lower-case hex (temper 02).
Test shape:
- In `src/tests/jig/reads.rs`, on the root harness. The reference test:
  ```rust
  #[test]
  fn a_task_read_prints_its_header_fields_and_acts_as_calls() {
      let mut root = fixtures::signed_in(fixtures::PARTY);
      let call = root.submit(&["task", "41"]);
      root.serve(call, ReadResult::Value(ReadValue::Task(Box::new(fixtures::goal(41, fixtures::waiting_on_me())))));
      let answer = root.answer(call).expect("answered once the read is");
      assert_eq!((answer.status, &*answer.stdout), (0, fixtures::golden("task_goal_waiting")));
  }
  ```
- Tests, each under 20 ms:
  - `a_task_read_prints_its_header_fields_and_acts_as_calls`, `a_proposable_act_says_to_whom_and_a_refused_one_why`,
    `a_keyed_acts_call_ends_with_key_name`, `an_act_with_no_tool_prints_its_name_and_no_call`, `me_prints_the_live_frame_in_text_and_json`,
    `no_call_runs_before_me_is_live`, `each_read_none_exits_as_the_table_says`,
    `a_result_past_the_text_limit_says_how_much_is_left`;
  - in `src/tests/root.rs`, with the same shipped limits: `the_root_routes_by_asker_and_tells_no_child_of_an_unknown_one`,
    `two_calls_in_flight_are_answered_each_under_its_own_number`, `new_calls_start_above_all_kept_call_numbers`,
    `every_call_key_and_handle_uses_its_prefix_and_one_owner`, `call_number_exhaustion_exits_1_without_sending`.
- Extend the wire's existing owner-number round-trip test with `(Owner::Runner, 3)`; no new test file.
- Extend `crates/temper-service/src/tests/family.rs`'s `a_request_under_the_pages_owner_is_answered_undecodable` to
  `Owner::Runner` beside `Pages`, as one case per owner of the same setup, renamed for both (ruling 119).
- Reference for `src/tests/root.rs` (the same harness):
  ```rust
  #[test]
  fn two_calls_in_flight_are_answered_each_under_its_own_number() {
      let mut root = fixtures::signed_in(fixtures::PARTY);
      let a = root.submit(&["task", "41"]);
      let b = root.submit(&["task", "42"]);
      root.serve_task(b, fixtures::goal(42));
      assert!(root.answer(a).is_none());
      assert_eq!(root.answer(b).expect("second answered").status, 0);
  }
  ```
Proof: unit, `cargo nextest run -p temper-client-tools -E 'test(=a_task_read_prints_its_header_fields_and_acts_as_calls)'`
(under 1 s): a read through `jig_client::Client`, printed as 6.1. The next plan's stories bring the tools' world.
Don't: write `temper`, in any case, anywhere under `src/jig/`, comments and strings included (`src/tests/boundary.rs`
lowercases each file); keep a document across steps; print a float; send anything for a read but its read.
Size: at most 300 lines.
Removes: nothing.

### 03.3 The other reads
Scope: `inbox`, `tasks`, `activity N`, `run N`, `chat N`, `turn N A.T`; `task N --tree --activity K --run` in one answer;
turns by their size; cuts with their calls (client/tools.md 5.2, 6.4, 12); step `temper-03.3`.
Needs: temper on jig's named send, turn read and spans.
Files: under `crates/temper-client-tools/`, `src/jig/render/{lists,activity,turn}.rs` (new), `src/jig/runner.rs`,
`src/jig/rows.rs` (`--ended`'s help, ruling 119), with any help golden that prints it,
`src/tests/jig.rs`, `src/tests/jig/{lists,turn}.rs` (new), `src/tests/fixtures.rs`, goldens.
Shape:
- **Lists.**
  - `inbox` reads needs first (decisions.md, For the user, O, decided , temper 0ecb23bf6): while needs has
    more, a page is needs alone; once needs ends on a page, updates' first page follows it when both fit in 32 rows, and
    when they do not, the call that reads on starts updates (group byte 1, no jig cursor). `tasks` reads `Read::Tasks`, one group per call
    (decisions.md, For the user, L, decided , temper 09cfaca18): without `--ended`, the live group, `--after C` its `live`;
    with it, the ended group alone, as jig's view reads it (`query.phase = Some(PhaseClass::Ended)`, `live: None`,
    `--after C` its `ended`), the call that reads on keeping `--ended`; `rows.rs`'s `--ended` help reads
    "Only ended tasks". An agent that wants both makes two calls; the printed cursor stays jig's, with no group byte.
  - `inbox`'s printed cursor (decisions.md, For the user, N, decided , temper ad1b1f80f) is one group byte
    (0 needs, 1 updates), then jig's cursor, whole. The call that reads on stays in needs while needs has more, then
    moves to updates; `--after C` reads C's group, and with `--kind K` a C of the other group (`InboxReadGroup::of`) is
    a usage error, exit 1.
  - Each prints at most 32 rows, one a line, number first, then the call that reads on, or `no more`.
- **A task's parts.**
  - `--tree` watches `Subject::Tree { task, window: 32 }`; `--run` watches `Subject::Run { task, turn: None, offset: 0 }`.
    Each is unwatched at its first delivery.
  - `--activity K` reads `Read::Activity` and prints its K latest lines (K at most 32); one that cuts a page prints
    `read with: activity N` (decisions.md, For the user, P, decided , temper 37c08b325). `run N` is `--run`
    alone.
- **The conversation.**
  - `chat N` pages through `jig_client::thread::Thread` (`older`, `accept`, `parts`).
  - `activity N` reads `Read::Activity`, `--limit K` at most 32. A page that `--limit K` cuts reads on with the same read
    without `--limit` (P): the page whole, its K rows again, then jig's `before`; the printed cursor stays jig's.
  - Provisional text prints only where `client.text` gives it.
- **Turns.**
  - A line whose turn is `TurnProblem::TooLarge` prints `turn A.T · S KB · read with: turn N A.T`.
  - `turn` sends `Read::Turn { task: N, attempt: A, turn: T }`; `ReadValue::Turn(Box<ArchivedTurn>)` supplies text, calls and results.
  - Each part is cut at 16 KiB, with the bytes left and `read on with: turn N A.T --part K --from B`.
  - `ReadNone::TooLarge { bytes }` from either bound prints `turn A.T · S KB · past the limit; report it`, exit 0.
    `Readout::Limit` is full read-table admission, has no size and exits 1; other document limits also exit 1.
- **Paths and authorities** print whole. Page and single read share `load_bytes`: the next plan's fixture must use the row-cost overlap jig 03 measures.
Test shape: in `src/tests/jig/{lists,turn}.rs`, on the root harness, each under 20 ms:
`a_list_prints_a_row_a_line_then_the_call_that_reads_on`, `inbox_prints_what_needs_the_party_then_updates`,
`an_inbox_reads_on_in_needs_while_needs_has_more`, `an_inbox_reads_on_in_updates_once_needs_ends`,
`an_inbox_page_whose_updates_do_not_fit_reads_on_at_the_start_of_updates`,
`an_inbox_cursor_of_another_group_than_its_kind_exits_1`,
`task_with_tree_activity_and_run_answers_once_and_unwatches`,
`a_turn_too_large_for_a_page_prints_by_its_size_and_its_call`,
`a_limited_activity_reads_on_with_its_page_whole`,
`a_turns_part_past_16_kib_is_cut_with_the_call_that_reads_on`, `a_turn_past_the_read_bound_exits_0_past_the_limit`,
`read_table_and_document_limits_exit_1`, `chat_prints_no_provisional_text_where_the_host_gives_none`, `a_path_is_never_cut`.
Reference tests, one per new file (`lists.rs`, then `turn.rs`):
```rust
#[test]
fn a_list_prints_a_row_a_line_then_the_call_that_reads_on() {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let call = root.submit(&["tasks", "--project", "1"]);
    root.serve(call, fixtures::task_page_with_cursor());
    assert_eq!(&*root.answer(call).expect("page answered").stdout, fixtures::golden("tasks_next"));
}
#[test]
fn a_turn_past_the_read_bound_exits_0_past_the_limit() {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let call = root.submit(&["turn", "41", "3.7"]);
    root.serve(call, fixtures::turn_too_large(1_048_577));
    let printed = root.answer(call).expect("sized answer");
    assert_eq!((printed.status, &*printed.stdout), (0, fixtures::golden("turn_limit")));
}
```
Proof: unit, `cargo nextest run -p temper-client-tools -E 'test(/turn/)'` (under 1 s). It shows 5.2's three turn
forms. The next plan's story "a turn too large for a page read whole" connects it.
Don't: keep a watch past its first delivery; print more than 32 rows; cut a path.
Size: at most 380 lines, with the inbox's cursor and first page and a limited activity's reads-on (decisions.md, N, O and P).
Removes: nothing.

### 03.4 jig's acts under their keys
Scope: `chat`, `message`, `close`, `answer`, both `decide` forms, `release`, `stop`, `cancel`; named/drawn keys,
drafts against bounds, outcomes, kept lines and exits (client/tools.md 5.3, 6, 7); step `temper-03.4`.
Needs: temper on jig's named send, turn read and spans.
Files: under `crates/temper-client-tools/`, `src/jig/runner.rs`, `src/jig/render/outcome.rs` (new), `src/root.rs`, `src/tests/jig.rs`, `src/tests/jig/acts.rs` (new), `src/tests/fixtures.rs`, goldens.
Shape:
- **Sending.** Read the offered act, then send `Event::Send` with the application's key and
  `name: Some(jig::key::named(salt, name))` for a named call; without a name, jig draws the engine key.
- **Stable shapes:** use jig 05's clockless `shapes::chat(me, project, words)`; send a template's deadline span
  unchanged. Compare encoded asks for the same call and me under different clocks/seeds, excluding envelope keys.
  Audit every act for process-local fields; temper 11 checks adopt. Changed me inputs may produce an engine conflict.
- **Close:** use `jig_client::shapes::close_chat`, moving each `--move D`, cancelling the other live delegates;
  pass `--reason` or empty text. Parse and print `tN/K` and `pN/K` through the table's proposal codec.
- **Outcomes:**
  - `Done(Made)` prints `done: task 41, chat`.
  - `Done(Proposed)` prints `proposed: t3/17, to <recipient>` (or `p3/17`). A kept line of a restored proposal prints its
    identity alone, `kept: chat, proposed: t3/17`, with no `to` (decisions.md, For the user, Q, decided ,
    temper 37849f0b0).
  - `Done(DecidedFirst)` prints `decided first: by <party>, <decision>, HH:MM`.
  - `Done(Changed)` and `Done(Decided)` print `done: task N, <act>`.
  - `Refused` exits 2 with the refusal in jig's words, then its alternatives: `read again: <call>` where a revision
    moved, and `propose instead: <call>` where proposable. An engine key conflict also exits 2.
  - Client answers follow Choices: exit 1; Unconfirmed and a detected client key collision name the tools' bug.
    jig's client panics on a key it holds for another request (`jig-client-domain`'s `requests.rs`, "request key
    conflict"), so the runner asks `Client::pending(key)` before each `Event::Send`: a key held is the tools' bug, exit 1,
    nothing sent. An identical named re-send is told nothing, so the test sends another ask under the same key (ruling 119).
- **Kept lines.** Track restored operations by application key in memory; print Choices' text/JSON once answered as `Out::Kept`.
  `Root::kept()` counts told-but-unanswered requests for live `me`'s party only, using `Client::pending(key).party`.
  Report other parties' parked counts once `me` is live and `Told::Dropped { count }` without inspecting unread records.
  Use Interface sketches' lines; the next plan's host awaits this party's answers and erases' Stored completions, writes the lines, then admits Call.
Test shape:
- In `src/tests/jig/acts.rs`, on the root harness. The reference test:
  ```rust
  #[test]
  fn a_message_under_a_name_sends_it_as_the_engine_key_and_prints_done() {
      let mut root = fixtures::signed_in(fixtures::PARTY);
      let call = root.submit(&["message", "41", "Use port 8080", "--key", "port-answer"]);
      root.serve(call, ReadResult::Value(ReadValue::Task(Box::new(fixtures::chat(41, &[ActKind::Message])))));
      assert_eq!(root.sent_name(call), Some(fixtures::named("port-answer")));
      root.done(call, Outcome::Changed);
      let answer = root.answer(call).expect("answered once durable");
      assert_eq!((answer.status, &*answer.stdout), (0, &b"done: task 41, message\n"[..]));
  }
  ```
- Tests, each under 20 ms: `a_message_under_a_name_sends_it_as_the_engine_key_and_prints_done`,
  `an_act_without_key_draws_one`, `chat_prints_the_task_it_made`, `a_proposable_act_prints_proposed_and_to_whom`,
  `decide_sends_its_two_forms`, `decided_first_prints_by_whom_how_and_when`,
  `a_refusal_exits_2_with_its_alternatives_in_text_and_json`, `an_engine_key_conflict_exits_2`,
  `held_exits_1_and_nothing_is_sent`, `another_partys_parked_name_is_held_and_exits_1` (nothing kept or sent),
  `an_act_not_offered_exits_1_with_nothing_sent`, `each_kept_request_prints_operation_and_outcome_without_name_or_keys` (named/unnamed, text/JSON; no `name` field),
  `a_restored_proposals_kept_line_prints_its_identity_without_a_recipient` (Q),
  `dropped_and_other_party_kept_lines_use_text_or_json` (counts per party, only live me's party in `Root::kept()`, no unread payload decoded),
  `a_named_chat_has_the_same_request_bytes_under_two_clocks` (including a Span template),
  `a_named_act_has_no_process_local_fields`, `close_passes_its_reason_or_empty`,
  `client_refusals_exit_1`, `unconfirmed_and_client_key_conflicts_say_tools_bug_and_exit_1`,
  `changed_me_inputs_can_change_a_named_ask`, `proposal_names_preserve_task_or_party`.
Proof: unit, `cargo nextest run -p temper-client-tools -E 'test(/held|key|kept/)'` (under 1 s). It shows a name sent
as the engine key, `Held` exiting 1, and the kept lines. The next plan's stories connect it: "asked again under its named key, made once"
and "a named key told Held".
Don't: confirm before sending; send what the offer refuses; print a kept name or key; keep runner records to recover names.
Size: at most 310 lines, with a restored proposal's kept line (decisions.md, Q).
Removes: nothing.

## Interface sketches

temper 03 owns these interfaces; 04 extends Runner, the next plan's host hosts Root and adapts Printed to Answer, 11 implements Parts.
03.2 supplies `client()` and pre-Start `kept_json(bool)`, as that host cites; the latter selects all three recovery forms.
03.4 renders through `Out::Kept { line }`, forwarded as `Request::Kept { line }`; the host owns the call sequence and writes
these bytes on stderr without a prefix: answered operations; `dropped: N kept records these tools cannot read`
(`{"dropped": N}`); `kept: N for party P, unsent while it is not signed in here` (`{"party": P, "unsent": N}`).
The host awaits only me's party's answers and erases before Call, never parked work.

```rust
// crates/temper-client-tools/src/root.rs (03.2)
/// The shared owner; pass RUNNER.asker() to Runner, never allocate an asker per call.
pub const RUNNER: temper_client_domain::owner::Owner = temper_client_domain::owner::Owner::Runner;
/// Client and runner, ending with the host.
pub struct Root { .. }
/// temper 02's presets, shared with the host; the runner gets `Env { now, wall, limits: tools }`, built by the root.
pub struct Limits { pub domain: temper_client_domain::Limits, pub tools: jig::Limits }
/// Host inputs; each Call ends with Printed.
pub enum Event {
    /// Transport, store or sign-in completion.
    Client { event: jig_client::Event },
    /// Parsed call with the host's correlation number; Root allocates its key prefix.
    Call { number: u32, call: jig::call::Call, json: bool },
}
/// Host outputs, drained each step.
pub enum Request {
    /// I/O for Client; answered as Event::Client.
    Client { request: jig_client::Request },
    /// me live; wait for its party's kept requests and erases.
    Ready,
    /// Recovery line on stderr: an outcome, dropped count or another party's unsent count.
    Kept { line: Box<[u8]> },
    /// One call's terminal result, emitted once.
    Printed { number: u32, printed: jig::Printed },
}
impl Root {
    pub fn new(limits: &Limits, seed: u64) -> Result<Self, jig_client::Unusable>;
    /// After Ready, kept but unanswered for me's party; the host separately awaits Stored.
    pub fn kept(&self) -> u32;
    /// Read-only link state for the next plan's host deadlines.
    #[must_use]
    pub fn client(&self) -> &jig_client::Client;
    /// Set before Start, since kept results precede the new Call.
    pub fn kept_json(&mut self, json: bool);
}
pub fn step(root: &mut Root, env: &Env<Limits>, event: Event, out: &mut Queue<Request>);
pub fn fire(root: &mut Root, env: &Env<Limits>, out: &mut Queue<Request>);
#[must_use]
pub fn next_deadline(root: &Root) -> Option<Time>;
// crates/temper-client-tools/src/jig/runner.rs (03.2); src/jig.rs re-exports `Printed` as `jig::Printed`
/// Terminal bytes and exit, consumed by the host.
pub struct Printed { pub status: u8, pub stdout: Box<[u8]>, pub stderr: Box<[u8]> }
/// Calls sharing one asker, each with its own high bits.
pub struct Runner { .. }
/// Effects for Root; Ready, Kept and Printed mirror Request field for field.
pub enum Out {
    /// Client input, consumed by Root rather than the host.
    Client { event: jig_client::Event },
    /// First live me.
    Ready,
    /// Recovery line, as Request::Kept.
    Kept { line: Box<[u8]> },
    /// Call's terminal result.
    Printed { number: u32, printed: Printed },
}
impl Runner { #[must_use] pub fn new(asker: Asker, salt: &'static [u8]) -> Self; }
/// One call as Root hands it over: the host's number, Root's call prefix, the call and its form (ruling 119).
pub struct Called { pub number: u32, pub call_number: u32, pub call: Call, pub json: bool }
pub fn call(runner: &mut Runner, client: &Client, env: &Env<crate::jig::Limits>, called: Called, out: &mut Queue<Out>);
pub fn told(runner: &mut Runner, client: &Client, env: &Env<crate::jig::Limits>, told: Told, out: &mut Queue<Out>);
// crates/temper-client-tools/src/jig/render.rs (03.2)
/// Application words for owner parts; temper 11 supplies the forge's.
pub trait Parts { fn owner_part(&self, part: &OwnerPart, out: &mut Text); }
```

## Questions to raise when reached

- **03.1:** a repository still at two revisions after Shape 1's updates, or repairs exceed about 200 lines: report commits and compiler errors.
- **03.3:** the landed turn interface loses the size at either bound: report the answer and its bytes; never substitute an unknown size.
- **03.4:** unequal encoded asks for the same call and me across clocks: report both requests and the first differing field.

## Done when

Each command runs in `~/src/rust/temper` against fetched `origin/main`;
proofs run once at their increment, not again here.

- `git show origin/main:Cargo.lock | grep -o 'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' | sort -u`
  shows one selected revision each of jig, skein and smith.
- `for p in crates/temper-client-domain/src/forge.rs crates/temper-client-view/src/confirm.rs crates/temper-client-view/src/pages/chats.rs; do git show origin/main:$p | grep -q 'Outcome::Held' || exit 1; done`
- `test -z "$(git grep -n -e 'jig_client_protocol_wire::v8' -e 'people_v3' origin/main -- crates tests testing)"`
- `git grep -q 'jig_client_protocol_wire::v9' origin/main -- crates/temper-service/src`
- `git show origin/main:crates/temper-client-tools/src/root.rs | grep -q 'jig_client::Client'`
- `git show origin/main:crates/temper-client-protocol-wire/src/lib.rs | grep -q 'Owner::Runner => 3'`
- `git show origin/main:crates/temper-client-domain/src/owner.rs | grep -q 'Self::Runner'`
- `git show origin/main:crates/temper-client-tools/src/root.rs | grep -q 'pub fn client('`
- `git show origin/main:crates/temper-client-tools/src/root.rs | grep -q 'pub fn kept_json('`
- `test -z "$(git grep -n -i temper origin/main -- crates/temper-client-tools/src/jig)"`
- `! git show origin/main:crates/temper-client-tools/Cargo.toml | grep -q 'temper-client-view'`
- `git grep -q -F 'read with: turn' origin/main -- crates/temper-client-tools/src/jig`
- `git grep -q -F 'past the limit; report it' origin/main -- crates/temper-client-tools/src/jig`
- `git grep -q -F 'decided first: by' origin/main -- crates/temper-client-tools/src/jig`
- `git grep -q -F 'kept: ' origin/main -- crates/temper-client-tools/src`
- `git grep -q 'Outcome::Held' origin/main -- crates/temper-client-tools/src`
