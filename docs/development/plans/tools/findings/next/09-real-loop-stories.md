# Session 09: the real loop's tools stories

The real-loop world keeps a chat and its reply across a service restart
(09.1), makes a request once after its caller is killed before the answer
(09.2), and serializes two acts on one DIRECTORY, including a killed
holder (09.3). The fake git measurement and landing are temper 13's.

Needs at start: **the real-loop world**. Steps: `temper-09.1` through
`temper-09.3`. Temper 13 can run alongside this session: neither needs
the other's stories. Shared files are `tests/real-loop/Cargo.toml` and
`tests/real-loop/src/lib.rs`; keep their registrations together on rebase.

## Read first

- temper at 4e14c2b5: `docs/design/client/tools.md` 5.4, 6, 7, 8.1 to 8.3,
  11.3 and 11.4; `docs/design/domain/testing.md` 2.4 and 6.1;
  `docs/design/client/README.md` 7 (the first owner's printed sign-in link).
- jig: `docs/design/client/domain/contract.md` 3.1 (answered once
  durable); `docs/design/protocol/store.md` 4.4 (answered keys).
- This plan: `temper/08-real-loop.md` 08.3, 08.4, 08.6 and Interface
  sketches; `temper/05-ctl.md` (Startup, Invocation, recovery and the
  lock); `temper/02-tools-table.md` (`key::engine`, `limits::LOCK_TRY`);
  `temper/03-tools-root.md` (the restored-name readiness gap);
  `temper/13-real-loop-landing.md`; `findings/temper-area-1.md` 3, 6, 8.
- The planner's decisions on real-loop proofs and seams; designer
  round-2 answers 24, 25, 29 and 30. Follow the specified 4e14c2b5
  contract, including the request's name in a kept result.
  Temper 03 must resolve durable original-name metadata before its
  root is ready; this session cannot reconstruct a name from its digest.
- Code: `tests/one-process/src/referee/durable.rs`, `Capture::complete`
  and `is_log`; jig's `crates/jig-protocol-store/src/keys/people.rs`,
  `write`; skein's `testing/skein-world/src/real.rs`, `Running::kill`,
  `settle_children`, `release_killed`; `tests/world/tests/real.rs`,
  `kill_drops_child_and_settles_pending_read_before_parent_wait`.
- skein's `docs/design/kernel.md` 6.1 and 6.2;
  `docs/foundation/programming-model.md` 2.1, 4.3, 6.2, 6.3, 10.2;
  `docs/foundation/testing-strategy.md` 2.8, 6, 7, 8, 9, 10.

## What the design asks

- **client/tools.md 11.3:** the restarted service retains the work; a fresh
  caller on another DIRECTORY, signed in as the same party, gets the
  lost answer under its name, with one chat; concurrent acts serialize.
- **8.1 to 8.3:** DIRECTORY and its private files belong to the host;
  acts recover and erase kept requests before sending the new act.
  Reads and `pending` leave those requests alone.
- **6.1 and 6.2:** a kept result says name, operation and outcome;
  neither client key nor engine key is printed. JSON has exactly
  `name` (or null), `operation`, `outcome` on stderr, one object a line.
- **6.3:** successful calls exit 0; a cut call's status is the OS's,
  never 0. Exit 2 is the engine's answer alone; client answers, Held
  among them, exit 1. **5.4:** stopping the engine ends no watch;
  the client reopens it and eventually fails at the link deadline, 1.
- **domain/testing.md 2.4, 6.1:** real files and descriptors, checked ownership
  and teardown, one second per focused story, retained failure traces.

## Choices this plan makes

- **Extend temper 08's crate and caller.** `temper-real-loop-world`
  already owns `run`, `run_on`, `Scratch`, `calls.rs`, `Step::Start`,
  `Step::Join`, `same_party`, `Called` and `Ran`. Add no second runner.
  `scripts::chat_scripts`, `WORDS`, `REPLY`, `fixtures.rs` and its
  shipped profile documents remain the shared fixtures.
- **Startup stays temper 05's.** Each call gets a fresh parent startup
  root in `Startup.parent`, with DIRECTORY's leaf in `Invocation`.
  The host opens DIRECTORY itself through `OpenPrivate`; the harness
  never passes a shared or duplicated locked descriptor to a cli host.
- **The durable cue** watches completed writes and the following
  successful Sync of `store/log-*`, using `Capture::complete`'s offset
  and partial-write handling. Arm before starting the named calls;
  match their `key::engine(&Name)` bytes across writes, never a print.
  The people store's answer key writes those 16 bytes unchanged.
  `ServiceHost` observes before forwarding completions, as 08 provides.
- **Kill without another I/O turn.** The service runs before the
  caller, and the caller before its children (08.4). On the cue the
  caller submits Kill in that turn, before the child can read the
  answer. A failed Signal, or an answer read before Kill, fails the
  story. Probe the released lock only after Kill and Wait; a probe
  between the cue and Kill could let the answer arrive.
- **Observe locks outside the cli host.** Extend the caller's wrapper
  to record completed Lock and Close operations, with call ids, under
  08's ownership checks. The probe opens DIRECTORY independently;
  `Probe::Taken` retains that open until `Release` closes it.
- **Restart uses a kept scratch and a fresh world.** Re-sign D1 by the
  second run's printed link; its old address may name a closed port.
  Both runs settle separately. This is persistence, not a stopped-watch
  test: 05's virtual-time tests cover exit 1 at the unchanged 30 s
  deadline. No real-loop story waits 30 s or changes that deadline.

## Rules that bind it

- Extend 08's fixtures and owner/operation checks; new observations are
  bounded and released. Keep its admitted profiles and all shipped
  client deadlines, including the 100 ms lock retry and 60 s lock wait.
- No fixed ports or readiness sleeps. Every story drains within 1 s;
  a miss is profiled unoptimized and made cheaper or moved to `fuzzy_*`.
  The lower worlds' 4 s guide and the suite's 15 s/60 s budgets still bind.
- No forge or LLM hotspot changes here. No signal handler, profile
  override, design or STATUS edit. Failures keep their trace and scratch
  for reproduction at the lowest layer that shows them.

## Increments

### 09.1 A service restarted on its directory keeps the work

Scope: client/tools.md 11.3's service restart, using 08's existing `run_on`.
Needs: nothing beyond the session's start.
Files: `tests/real-loop/tests/calls.rs` (new);
`tests/real-loop/Cargo.toml` (the `calls` binary).
Shape: run once on a Scratch, sign in, start a chat and await its reply;
drain. Run again on that Scratch with no LLM scripts, sign D1 in through
the fresh printed link, and read the chat and its already-kept reply.
Test shape: `tests/calls.rs`, 08's counting allocator and fixtures;
`run_on`, `chat_scripts`, `rows` and `holds` are reused. Reference:
```rust
#[test]
fn a_restarted_service_keeps_the_chat_and_its_reply() {
    let scratch = Scratch::new();
    let first = run_on(&scratch, chat_scripts(1), &[
        sign_in(D1),
        call(D1, argv!["chat", "--project", "1", WORDS, "--key", "first"]),
        call(D1, argv!["wait", "task", task(0), "--until", "replied"]),
    ]);
    assert!(first.calls.iter().all(|c| c.exit == Exit::Code(0)), "{first:?}");
    let n = first.number(0);
    let second = run_on(&scratch, Box::new([]), &[
        sign_in(D1),
        call(D1, argv!["tasks", "--project", "1", "--label", "chat", "--json"]),
        call(D1, argv!["chat", &n]),
        call(D1, argv!["wait", "task", &n, "--until", "replied"]),
    ]);
    assert!(second.calls.iter().all(|c| c.exit == Exit::Code(0)), "{second:?}");
    assert_eq!(rows(&second.calls[0].stdout), 1, "one kept chat");
    assert!(holds(&second.calls[1].stdout, REPLY), "the reply was kept");
    assert!(!second.llm.iter().any(|o| matches!(o, Observation::Query { .. })));
}
```
Proof: world, `cargo nextest run -p temper-real-loop-world --test
calls -E 'test(restarted)'`, under 1 s for both runs together. This
kernel persistence proof is already the highest useful layer; both
runs pass 08's drain, memory and descriptor checks.
Don't: implement `run_on` again, write `address` by hand, carry a watch
between worlds or call a stopped engine a watch's terminal outcome.
Size: at most 10 harness lines; no production change.
Removes: nothing.

### 09.2 The lost answer, made once by its name

Scope: client/tools.md 7, 8.2 and 11.3's dropped caller, recovery and lock release.
Needs: nothing beyond the session's start.
Files: `tests/real-loop/src/cue.rs` (new);
`tests/real-loop/src/{calls.rs,service.rs,lib.rs,fixtures.rs}`;
`tests/real-loop/tests/calls.rs`.
Shape: extend Step with Held, Kill, Probe and Release (sketch below).
Wire `cue::Watch` into 08's `ServiceHost` completion observation, with
root/open/close tracking and bounded log bytes. Kill submits
`Op::Signal { pidfd, signal: Signal::Kill, to: Target::Child }`, then
reaps and drains via 08's Join machinery. D2 is `same_party(D2, D1)`:
only settled token/address bytes copied, its request store empty.
Test shape: `tests/calls.rs`, the same fixtures and limits. Add
`fixtures::assert_kept_create(stderr, name, task)` over 08's `kept_lines`:
exactly one text line, the name, jig's `create` operation and the done
outcome naming that task, with neither key. It checks the whole line.
`pending` assertions count rows only; they do not settle question 33's
row vocabulary. Reference (under 1 s):
```rust
#[test]
fn a_call_killed_once_its_request_is_held_is_made_once_by_its_name() {
    let chat = argv!["chat", "--project", "1", WORDS, "--key", "lost-chat"];
    let tasks = argv!["tasks", "--project", "1", "--label", "chat", "--json"];
    let ran = run(chat_scripts(1), &[
        sign_in(D1), same_party(D2, D1),
        Step::Start { at: D1, call: 0, argv: chat.clone() },
        Step::Held { names: &[b"lost-chat"] }, Step::Kill { call: 0 },
        Step::Probe { at: D1, expect: Probe::Taken }, Step::Release { at: D1 },
        call(D2, tasks.clone()), call(D2, chat), call(D1, argv!["pending"]),
        call(D2, tasks),
        call(D1, argv!["close", task(2), "--key", "lost-close"]),
        call(D1, argv!["pending"]),
    ]);
    assert_ne!(ran.calls[0].exit, Exit::Code(0), "the cut status is the OS's");
    assert!(ran.calls[0].stdout.is_empty(), "no answer before the cut");
    assert_eq!(ran.killed.len(), 1, "one hosted program dropped");
    assert!(ran.calls[1..].iter().all(|c| c.exit == Exit::Code(0)), "{ran:?}");
    assert_eq!(numbers(&ran.calls[1].stdout), [ran.number(2)]);
    assert_eq!(rows(&ran.calls[3].stdout), 1, "the request is still kept");
    assert_eq!(rows(&ran.calls[4].stdout), 1, "one chat after the rerun");
    fixtures::assert_kept_create(&ran.calls[5].stderr, "lost-chat", &ran.number(2));
    assert!(ran.calls[6].stdout.is_empty(), "nothing kept after recovery");
}
```
Proof: world, `cargo nextest run -p temper-real-loop-world --test
calls -E 'test(killed_once)'`, under 1 s. The trace orders the durable
cue, Kill and the released lock; no answer was read first. The real
kernel world is this proof's layer; executable signal conversion is 10's.
Don't: catch a signal, require `Exit::Signal(9)`, insert an I/O wait
between cue and Kill, or turn Held into exit 2. No engine or cli seam.
Size: at most 240 harness lines; no production change.
Removes: nothing.

### 09.3 Two acts at once on one DIRECTORY, serialized by the lock

Scope: client/tools.md 8.3 and 11.3, contention and recovery after a killed holder.
Needs: the durable cue and lock observations (09.2).
Files: `tests/real-loop/src/{calls.rs,fixtures.rs}`;
`tests/real-loop/tests/calls.rs`.
Shape: add Contended, KillHeld and JoinOther. For bounded, observed
contention, the caller takes D1's lock by Probe, starts both calls, waits
until both have received Busy, then releases its probe. Both retry on
the shipped 100 ms pace; which succeeds first is the kernel's choice.
KillHeld selects the name at the durable cue and kills immediately;
JoinOther uses 08's Join. No call's lock interval overlaps another's.
Test shape: two stories in `tests/calls.rs`, 08's profiles and scripts:

- `two_acts_on_one_directory_are_serialized_by_the_lock`: both chats
  exit 0, neither emits a kept line, two chats exist, pending is empty.
- `an_act_waiting_on_a_killed_holder_takes_the_lock_and_sends_its_record`:
  the holder exits nonzero, the other recovers its record before its
  own act, exits 0, two chats exist, pending is empty. Use `--json` on
  both started acts; its one stderr object has exactly name, operation,
  outcome; the name is the killed call's, operation `create`, outcome
  the first task. No client or engine key field or value.

The second reference, with fixtures reusing 09.2's decoder:
```rust
#[test]
fn an_act_waiting_on_a_killed_holder_takes_the_lock_and_sends_its_record() {
    let ran = run(chat_scripts(2), &[
        sign_in(D1), Step::Probe { at: D1, expect: Probe::Taken },
        Step::Start { at: D1, call: 0, argv: argv!["chat", "--project", "1", WORDS, "--key", "both-one", "--json"] },
        Step::Start { at: D1, call: 1, argv: argv!["chat", "--project", "1", WORDS, "--key", "both-two", "--json"] },
        Step::Contended { calls: &[0, 1] }, Step::Release { at: D1 },
        Step::Held { names: &[b"both-one", b"both-two"] },
        Step::KillHeld, Step::JoinOther,
        call(D1, argv!["tasks", "--project", "1", "--label", "chat", "--json"]),
        call(D1, argv!["pending"]),
    ]);
    let holder = if ran.calls[0].exit == Exit::Code(0) { 1 } else { 0 };
    let other = 1 - holder;
    assert_ne!(ran.calls[holder].exit, Exit::Code(0));
    assert_eq!(ran.calls[other].exit, Exit::Code(0), "the lock was released");
    let name = ["both-one", "both-two"][holder];
    fixtures::assert_kept_json_create(&ran.calls[other].stderr, name, &ran.calls[2].stdout, &ran.number(other));
    assert!(ran.calls[2..].iter().all(|c| c.exit == Exit::Code(0)), "{ran:?}");
    assert_eq!(rows(&ran.calls[2].stdout), 2);
    assert!(ran.calls[3].stdout.is_empty());
}
```
`assert_kept_json_create` checks the exact field set and the created
task: one of the two listed tasks, different from the surviving call's.
Each story under 1 s; trace checks Busy, retry and exclusive intervals,
not a guessed duration between spawn and exit.
Proof: world, `cargo nextest run -p temper-real-loop-world --test
calls -E 'test(an_act_waiting_on_a_killed_holder)'`, under 1 s. The
kernel's lock and descriptor release are already at the required layer.
Don't: choose which concurrent call wins, shorten a client deadline,
or add signal handling to the tools.
Size: at most 100 harness lines; no production change.
Removes: nothing.

## Interface sketches

Add variants to `calls::Step`; keep 08's existing variants and types.
`Ran.calls` still includes Call and Start in step order, excluding sign-ins.

```text
// tests/real-loop/src/calls.rs: new variants, not another enum.
Held { names: &'static [&'static [u8]] }, // wait for one armed durable name
Probe { at: Dir, expect: Probe },        // independent open and Op::Lock
Release { at: Dir },                    // close that probe's open
Kill { call: u8 },                      // signal, reap, drain
Contended { calls: &'static [u8] },      // each has a completed Busy
KillHeld,                              // kill the name which fired the cue
JoinOther,                             // join the other of two Starts
pub enum Probe { Busy, Taken }
```

```rust
// tests/real-loop/src/cue.rs: bounded observations, no domain access.
pub struct Watch { /* descriptor paths, completed bytes, armed keys, cue */ }
impl Watch {
    pub fn arm(&mut self, keys: &[skein_lib::RequestKey]);
    pub fn root(&mut self, fd: Fd);
    pub fn complete(&mut self, complete: &Complete);
    pub fn fired(&self) -> Option<skein_lib::RequestKey>;
}
```

The new fixture assertions decode only the published text/JSON, never
the client root. Lock observations stay in the caller's existing bounded
trace, not in a second public result structure.

## Questions to raise when reached

- **09.2:** the cue misses the encoded key, or D2 lists no chat before
  the rerun: retain completed log writes and their Sync; report the
  failed durability premise. Do not substitute an unrelated commit.
- **09.2/09.3:** Kill follows a read of the answer, fails, or leaves a
  lock held after Wait: report the ordered operations and descriptors.
- A story or suite exceeds its time guide, or new observations exceed
  an owner bound: report the profile and counts before moving the story
  to `fuzzy_*`; keep the checks and unoptimized build.

## Done when

Each in `~/src/rust/temper`, against fetched `origin/main`:

- `git grep -q -F 'run_on(&scratch' origin/main -- tests/real-loop`
- `git grep -q -F 'Signal::Kill' origin/main -- tests/real-loop`
- `git grep -q -F 'Op::Lock' origin/main -- tests/real-loop`
- `git grep -q -E 'assert_ne!.*exit, Exit::Code\(0\)' origin/main -- tests/real-loop`
- `git grep -q -F 'key::engine' origin/main -- tests/real-loop`
- `git grep -q -F 'Step::Contended' origin/main -- tests/real-loop`
- `git grep -q -F '"operation"' origin/main -- tests/real-loop`
- `test -z "$(git grep -n -E '127\.0\.0\.1:[1-9]' origin/main -- tests/real-loop/tests)"`
