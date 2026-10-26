# Session 01: temper on skein's, smith's and jig's main

temper moves its three pins onto jig 01's landing and that landing's own
pins of skein and smith: skein `b4c6241` and smith `a277e922`, their
mains when this plan started (decisions.md, 1 and 2). It then repairs
what they break, at the least change that keeps behaviour: the 17
errors of `findings/base-repin.md`'s groups T1 to T6. skein added a
kernel lock, a file lock, a paged listing and `Error::Busy`. smith's
parity work added jobs, with `Request::Job` and `Turn.jobs`, and
transcript version 5 in place of 3. Jobs stay smith's. Per agent.md,
4.3, `temper-engine-smith`'s `turn` answers a job's request as it
answers every request but a turn, as `Problem::Type`. The findings come
from a rehearsal that only compiled. It put jig at its main of the
time, with that rehearsal's own repairs, rather than at jig 01's
landing. Its counts are measured; whether the tests then pass is not.

## Read first

- temper's `docs/design/domain/agent.md`: 4.3, the bullet "Jobs are
  smith's", and 4.4; `docs/design/STATUS.md`: section 3, "smith's parity
  changes, for the tools plan's repin of smith".
- `findings/base-repin.md`: the sections "temper", "What the skein
  changes are" and "Not measured"; `decisions.md`: 1 and 2.
- skein's `docs/foundation/development.md`: 5.7 and 9.5.
- jig 01's session file, `jig/01-repin.md`, and the bodies of its
  landing commits. They say what jig 01 did to `jig_protocol_transcript::Limits`,
  to `jig_channel_world::fixtures` and to its test configurations' agent
  memory.
- smith's commits `b415f737` (the house style for skein's new variants:
  new arms, with locks kept impossible where nothing asks for one) and
  `99356013` (the shipped memory raised to `2_214_592_512`); smith's
  `docs/design/shell.md`: 6.3, the row `memory`.

## What the design asks

- agent.md, 4.3: "Neither the engine nor the worker routes or answers
  a job. `temper-engine-smith`'s `turn` takes a turn request alone, and
  answers any other request, a job's included, as not a turn
  (`Problem::Type`)."
- STATUS.md, section 3: "temper's pin of smith is at transcript version
  3, which `tests/client` names; the tools plan moves it to smith's
  main, at transcript version 5, with jobs (`Request::Job`)."
- smith's shell.md, 6.3, `memory`: "as large as smith's work needs [...]
  and never smaller than the derived worst case".
- skein `b4c6241`, `crates/skein-io/src/kernel.rs`, `Error::Busy`:
  "`Lock`: another open holds the lock (`EWOULDBLOCK`)".
  `crates/skein-shell/src/ring.rs`, `file_error`, maps `Op::Lock`'s
  `libc::EWOULDBLOCK` to it.

## Choices this plan makes

- **One increment.** The pins move together, because jig 01's landing
  needs skein `b4c6241` and smith `a277e922`. Every repair is needed
  before anything compiles.
- **jig at jig 01's landing, not jig's `origin/main`.** By the time this
  session starts, jig 02 may have moved jig's skein to skein 01's
  landing, and jig 03 may have moved its wire to version 9. Both are
  temper 03.1's, which repins onto them.
- **T3: `Error::Busy => 11`.** `errno` writes Linux's numbers, and
  skein's ring makes `Busy` from `EWOULDBLOCK`, which is 11. The
  rehearsal's 16 (`EBUSY`) is not taken.
- **T6: temper names no transcript version.** `tests/client`'s `codec`
  is built with `jig_protocol_transcript::Limits::new`, which takes
  smith's ceilings from the version jig decodes. The `smith-transcript`
  dependency of `tests/client` then goes. jig's `main` has
  `Limits::new(bytes, turn)` already; a JIG01 without it is a question.
- **Locks stay impossible.** temper takes its first lock in the next
  plan's host (`temper ctl`).
- **One unit test** pins agent.md 4.3's rule for a job (4.4's step
  tests). No other test is added.
- **STATUS.md is not this session's.** The designer keeps it
  (development.md, 3.3). The item quoted above moves with the plan's
  landing and is cleared when the plan ends.
- **What jig 01 may change here.** The rehearsal put jig at `08430b94`,
  with its own probe repairs. T1 to T5 are temper's own matches and
  literals of skein's and smith's types, so jig 01 does not change them.
  jig 01 decides three things:
  - T6's repair: the shape of `jig_protocol_transcript::Limits`;
  - the documents that `crates/temper/tests/support/configuration.rs`,
    `documents`, copies from `jig_channel_world::fixtures`
    (`engine_document`, `WORKER_DOCUMENT`);
  - whether jig's test configurations raise their agent memory, as its
    outline says. temper's agent document follows them (Shape, 7).

  If jig 01 changes a type of `jig_client::contract` that temper matches
  (`Block`, `TurnProblem` in `crates/temper-client-view/src/pages/chat.rs`
  and `pages/task.rs`), the view's arms follow jig-client-view's at that
  landing. Nothing in the outline expects this.

## Rules that bind it

- **Pins** (development.md, 5.7 and 9.5). Every pin is named with
  `cargo update -p <package> --precise <sha>`. After 01.1, `grep -o
  'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' Cargo.lock |
  sort -u` prints three lines:
  - `jig.git#<JIG01>`;
  - `skein.git#b4c6241aefea0e434b6e1128754168d02bae0706`;
  - `smith.git#a277e9226273bf71c845db98b62a59b9ca4c9bfd`.

  A conflict in `Cargo.lock` is never merged by hand: take
  `origin/main`'s, then run Shape step 1 again.
- **A repin's bounds** (5.7): at most about 200 lines besides
  `Cargo.lock`, and no change of design. A downstream test broken by an
  upstream change of behaviour is adapted the way that upstream commit
  adapted its own; the landing's report names the commit. A break that
  needs more than this, or a decision, is a question. In that case the
  step lands nothing, and the gap goes to temper's STATUS.md under "Not
  planned" and to the plan's `later.md`.
- **Users of a changed type follow it, with no other change.** Tests
  keep their names and expected values, except those adapted under the
  rule above.

## Increments

### 01.1 temper on skein `b4c6241`, smith `a277e922` and jig 01's landing

Scope: the three pins; the repairs T1 to T6; the job's arm and its
test; any runtime adaptation that 5.7 allows.
Needs: jig on skein's and smith's main.
Files (the rehearsal's 12 files at +30 −13; `Cargo.lock` is a shared
hotspot):
- `Cargo.lock`, `tests/client/Cargo.toml`;
- T1: `crates/temper-worker-protocol-git/src/files.rs`
  (`Scan::complete`, `page`);
  `crates/temper-worker-protocol-git/src/tests/fixtures.rs`
  (`complete_ok`, `read`, `listing`, `clone_file_kind`);
  `crates/temper-client-shell-terminal/tests/terminal.rs`
  (`refused_directory`); `tests/worker/protocol-git/src/io.rs`
  (`Below::file`); `testing/temper-fake-git/tests/transcripts.rs`
  (`Disk::run`, `Disk::file`);
  `crates/temper-service/src/tests/fixtures/system.rs`
  (`Kernel::answer`);
- T2: `crates/temper-service/src/accounts.rs` (`file_owner`);
- T3: `crates/temper/src/lib.rs` (`errno`);
- T4: `crates/temper-engine-smith/src/finish.rs` (`turn`),
  `crates/temper-engine-smith/src/tests.rs` (`turn` joins its
  `use crate::{…}`);
- T5: `tests/worker/domain/src/protocol.rs` (`session_turn`),
  `crates/temper-worker-domain/src/tests/fixtures.rs` (`turn`);
- T6: `tests/client/src/fixtures.rs` (`codec`), `tests/client/Cargo.toml`
  (its `smith-transcript` line), and so `Cargo.lock`;
- only if step 7 is reached: `crates/temper/tests/support/configuration.rs`
  (`documents`) and `docs/examples/owned/agent.json`;
- if step 7's owned-example check refuses: `docs/examples/owned.json`
  (`engine.memory_bytes` and `workers[0].memory_bytes`, nothing else).
None of these is in the fake forge, the forge client, the forge
component, the fake LLM or the providers. `testing/temper-fake-git`
stays in temper under temper's `draft/forge-kit.md`, section 2.
Shape, in this order:
1. The pins. Find JIG01, the `commit` of the last landed `jig-01.*` row
   in the run's ledger (`plan status` prints it). `git -C ~/src/rust/jig
   merge-base --is-ancestor JIG01 origin/main` must pass, and
   `git -C ~/src/rust/jig show JIG01:Cargo.lock | grep -o
   'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' | sort -u` must
   print the skein and smith lines of the Pins rule; anything else is a
   question. From temper's root, run:
   - `cargo update -p jig-domain-core --precise JIG01`;
   - `cargo update -p skein-lib --precise
     b4c6241aefea0e434b6e1128754168d02bae0706`;
   - `cargo update -p smith-host-protocol --precise
     a277e9226273bf71c845db98b62a59b9ca4c9bfd`.

   If a source still holds two revisions, run its command once more.
   Then check the Pins rule's three lines. A second revision still
   there after that is a question.
2. T1, skein's `kernel::Op::Lock`, 11 errors. At each site above,
   `Op::Lock { .. }` joins the arm for the operations that site never
   submits (its `unreachable!`, `panic!` or `other @` group).
3. T2, skein's `file::Request::{Lock, ListPage}`. `file_owner` adds
   `Lock { owner, .. }` and `ListPage { owner, .. }` to its owner arm,
   as jig's `testing/jig-test-service/src/accounts.rs`, `file_owner`,
   does at JIG01.
4. T3, skein's `kernel::Error::Busy`: `Error::Busy => 11` in `errno`.
5. T4, smith's `Request::Job`: `| smith::Request::Job { .. }` joins the
   arm of `turn` that answers `Err(Problem::Type)`. Nothing routes,
   answers or counts a job.
6. T5 and T6. T5, smith's `record::Turn.jobs`: `jobs: 0` in the two
   literals. T6: `codec` returns
   `jig_protocol_transcript::Limits::new(32768,
   jig_client::contract::TurnLimits { messages: 32, blocks: 32, text: 8192 })`,
   and `smith-transcript` leaves `tests/client/Cargo.toml`. jig's `main`
   has that `new` already; a JIG01 without it is a question.
7. The runtime. Run the focused and fuzzy suites, then the Proof. If
   `temper serve`'s start (the startup world in `crates/temper/tests`),
   or `temper agent`'s start (the process proof), refuses the agent
   document because its worst case passes `memory`, then `documents`'
   `"memory"` becomes `2_214_592_512`, and so does
   `docs/examples/owned/agent.json`'s, which the owned deployment tests
   load (`SHIPPED`, `crates/temper/tests/support/owned.rs`). That is the
   change smith
   `99356013` made to its shipped memory; jig 01 drops its tests'
   declared memory instead (01.3), so it names no value for temper.
   If `serve_prepares_once_on_the_owned_example` (`crates/temper/tests/startup.rs`)
   refuses `engine.memory_bytes` or `workers[0].memory_bytes` of
   `docs/examples/owned.json`, "the worst case passes memory_bytes by N
   bytes", each refused number rises by its N, as forge-setup's
   `temper-14.s2` did (setup.md, 8.1: "Its `memory_bytes` hold its worst
   cases wherever a person lays it"), and the commit body gives each
   refusal before the raise. Any other failure follows the 5.7 rule.
Test shape: one new unit test, `a_job_request_is_not_a_turn`, in
`crates/temper-engine-smith/src/tests.rs`. It asserts that
`turn(smith::Request::Job { owner: Token::new(1), act:
smith::tools::JobAct::Hasten }, Box::default()).err()` is
`Some(Problem::Type)`. Ceiling: under 20 ms. No test's expected value is
known to change: the rehearsal only compiled. A test that fails is
adapted under the 5.7 rule and named in the landing's report, with the
upstream commit it follows.
Proof: unit, `cargo nextest run -p temper-engine-smith -E
'test(=a_job_request_is_not_a_turn)'`, once. It shows a job's request
answered as not a turn. Bound: 1 minute with the build.
Proof: process, the real binaries on the new pins, once through the
build lane: `sh -c 'cargo build -p temper --bin temper &&
NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper --profile process -E
"test(=temper_open_signs_in_prints_the_inbox_and_exits_on_q) |
test(=a_native_chat_reaches_the_model)"'`. The first test shows
`temper serve` printing its link, then `temper open` signing in, printing
`== Inbox == Live` and exiting 0 on `q`. The second shows `temper
agent`, smith's agent shell at `a277e922`, passing its startup checks
on temper's own agent document and answering a chat through the fake
LLM. It is the cheapest of the process tests that launch the agent on
that document. Bound: 10 minutes, the process profile's `global-timeout`. The second
test's throwaway Forgejo dominates, and was not measured at this plan's
base.
Don't:
- pin a jig other than JIG01, or jig's `origin/main` past it;
- route, answer or count a job anywhere;
- take or answer a lock, or give `Busy` any meaning but the errno;
- hand-edit `Cargo.lock` or generated code;
- raise `memory` past `2_214_592_512`, or change any other declared
  quantity than the owned example's two `memory_bytes` (Shape, 7);
- edit `docs/design/STATUS.md`.
Size: at most 200 lines. The rehearsal had 43 changed lines (+30 −13),
and steps 6 and 7 add about 4. Tests and `Cargo.lock` are not counted.
Removes:
- the pins temper's `origin/main` holds at the session's start (at this
  plan's drafting: jig `bbea9408`, skein `8a51e10b`, smith `29fa97c9`);
- `smith_transcript::v3`;
- `tests/client`'s `smith-transcript` dependency.

## Interface sketches

None: the session adds no type and no public item.

## Questions to raise when reached

- A test that fails on the new pins for a reason the 5.7 rule does not
  cover. This means a change of behaviour that no upstream commit
  adapted its own tests for, or a repair past about 200 lines. Raise it
  with the test, its output and the commit it points at. Examples are
  smith's transcript version 5 decoding in `tests/client`, smith's job
  defaults, or skein-sim's 9 changed files (`8a51e10b` to `b4c6241`) under
  temper's worlds.
- JIG01's `Cargo.lock` pinning a skein or smith other than the Pins
  rule's.

## Done when

- 01.1 has landed with its gate.
- `git -C <worktree> show origin/main:Cargo.lock | grep -o
  'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' | sort -u`
  prints three lines, one each for jig, skein and smith. smith's line is
  `smith.git#a277e9226273bf71c845db98b62a59b9ca4c9bfd`. The jig and
  skein lines are not checked here, because temper 03.1 moves them.
- `git -C <worktree> grep -n -F 'smith::Request::Job { .. }' origin/main
  -- crates/temper-engine-smith/src/finish.rs` prints one line.
- `git -C <worktree> grep -n -E 'smith_transcript::v[0-9]+' origin/main
  -- crates tests testing` prints nothing.
- `git -C <worktree> show origin/main:crates/temper/src/lib.rs | grep -n
  -F 'Error::Busy => 11'` prints one line.
- 01.1's two proofs passed once.
