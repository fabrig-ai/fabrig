# Session 06: the agent's area 1 stories in `tests/client`

`temper-client-world` runs the tools' root against temper's engine root,
beside the fake person, as a scripted agent making calls (`client/tools.md`,
11.2). jig's `World` holds one `App` type, and temper's `Temper` is a
client root read by the fake person through temper-client-view's tree.
So 06.1 gives `Temper` a second kind of client: the tools' root
(temper 03), with an empty tree, given calls by the story and keeping
what each call printed. The front and jig's check serve both kinds.
Then come the stories: a chat started, a turn too large for a page read
whole, a repository adopted, calls cut by the world's faults asked again
under their names and made once, a name told `Held`, and last a reply
waited for, a message, the chat closed and a held task released. The
session starts on the tools' root; only 06.5 needs the tools' waits.

## Read first

- temper's `docs/design/client/tools.md`: 4 (the root and its asker per
  call), 5.2 (a turn too large for a page; `turn`), 5.3, 5.4
  (`replied`), 6.1 to 6.3, 7, 8.2 (steps 3 to 6), 11.2, 12;
  `domain/testing.md`: 6.1 (the budgets) and 6.3; `client.md`: 10 (the
  `tests/client` bullet); `STATUS.md`, the rows `domain/testing.md` and
  `client/tools.md`.
- jig's `docs/design/client/domain/client.md`: 6 (keyed requests,
  `Held`), 11 and 20 (one-shot); `client/domain/testing.md`: 5, 6.2
  (jig's check), 7 and 8.
- jig's `tests/client/src/harness/` at temper's pin: `App`,
  `World::{client_with, reload, event, kept}`, `FrontFault`.
- This plan's `temper/02-tools-table.md` (argv to a call; ctl's
  `jig_client` `Limits` preset), `temper/03-tools-root.md` (the root,
  its printed results, 03.1's `Held`) and `temper/04-waits.md`;
  `findings/temper-area-1.md`, section 4.
- skein's `programming-model.md`: 3, 4.5, 6.3, 9; `testing-strategy.md`:
  3, 6, 7, 10 (by number only).

## What the design asks

- **`client/tools.md`, 11.2:** the stories, the profile's turn bound past its
  page bound, and the world's faults with each call made once.
- **4:** the root has no page or view; each call is an asker of its
  own, so several calls may be in flight.
- **7:** a named key is the engine's: the same name from the same party,
  from any DIRECTORY, gets the first answer; a name the client holds is
  answered `Held`, nothing sent, exit 1.
- **8.2:** a call sends what DIRECTORY kept first (`kept:` lines), then
  its own call, prints and exits.
- **5.2, 5.4, 6.1 to 6.3:** the large turn's line and `turn`'s parts;
  `--until replied`; the printed results and exits.
- **`domain/testing.md`, 6.3; `client.md`, 10:** `tests/client` is the joint
  world: jig's faults, each request made once, jig's check. jig's
  `client/domain/testing.md`, 8: stories through the client's surface
  where it has no tree.
- **`domain/testing.md`, 6.1:** a story past its guide is made cheaper or moved
  to fuzzy, never merged.

## Choices this plan makes

1. **One `App`, two arms.** `Temper`'s `Root` and `View` become enums:
   the person's arm is today's, and the agent's arm holds the tools'
   root with an empty `skein_ui::Tree`. The agent's root hands its jig
   requests on as `temper_client_domain::Request::Client { request }`.
   So `duty`, `sent`, `decision`, `received`, the front's `wire` and
   `translate::request` serve it unchanged, the forge family included.
2. **The agent's profile is the one the tools ship.** The agent comes
   from `World::client_with`, with ctl's `jig_client` preset (temper 02)
   as its client limits and the person's forge limits. `Joint` keeps
   each client's limits and translates each client's requests and
   answers under its own.
3. **A process is a root.** A new process on the same DIRECTORY is
   `world.reload(agent)`: a new root from the client's constructor and
   its kept records. Another DIRECTORY is a second agent client of the
   same party. The 30 s link deadline is the host's (temper 05, 08):
   here the driver cuts a call by reloading.
4. **Step 4 is the fixture's.** `fixtures::call` waits until
   `world.kept(agent)` is empty before it gives its call, as temper 05's
   host does, so `kept:` lines come first.
5. **The turn bound passes the page bound.** The page bound is the
   engine's constant `temper_engine_domain::limits::LOAD_BYTES` (64 KiB),
   so 06.2's profile raises the engine's turn bounds (8 KiB in
   `forge_limits()`) past it: the one deviation from the shipped
   profile, named beside it with 11.2's reason.
6. **The proofs are joint.** `tests/client` is the joint layer
   (development.md, 5.5; `domain/testing.md`, 6.3): the outline's "world
   proofs". The real loop's tools stories are temper 08's and 09's;
   nothing here goes to the process tier.
7. **`Held` is met in one root.** Two calls under one name, the second
   given while the first is in flight (client/tools.md, 4). ctl's host gives
   one call, and 8.2's step 4 keeps a kept record from meeting it.
8. **The agent copies acts.** A story runs the act a result printed,
   such as `release H --hold R` from `task G --tree` (5.3), and never
   builds it from the engine's state.

## Rules that bind it

- **jig's check holds in every story** (`client.md`, 10; jig's
  `client/domain/testing.md`, 6.2): the agent's decisions name revisions
  the client delivered; the tree half is the persons'.
- **Limits come from the shipped profile** (development.md, 5.6). The
  agent uses ctl's preset, the person `fixtures::domain_limits()`, and
  the engine `forge_limits()`, except for 06.2's named deviation.
- **No hotspot:** `tests/client/src/translate/forge.rs` is used as it
  stands; the fake forge, forge client, forge component, fake LLM and
  providers are not touched.

## Increments

### 06.1 The agent beside the person

Scope: the driver of Choices 1 to 4, and a thin story. The agent starts
a chat under its key, and the fake person of the same party sees it on
the chat's page (client/tools.md, 4, 5.3, 6.1, 8.2; 11.2).
Needs: nothing beyond the session's start (the tools' root).
Files: `tests/client/Cargo.toml` (`temper-client-tools`);
`tests/client/src/{app.rs, agent.rs (new), lib.rs, front.rs,
fixtures.rs, cards.rs}`; `tests/client/tests/chat.rs` (its one
`world.root(..).pages()` through `Root::person`), and any other user
`rg -n 'Temper::|world.root\(' tests/client` finds;
`tests/client/tests/agent.rs` (new).
Shape:
- `app::Root` and `app::View` with two arms each, and
  `app::Event::Call { call }`; each `App` function matches both arms.
- The agent's `step` takes `domain::Event::Client { event }` into the
  tools' root and ignores `Went` (no address); each variant the world
  gives a person alone panics, named (no wildcard). `Event::Start`
  restores the kept records and signs in as the person's arm does.
- Its `take` keeps the root's printed results on `Agent`, one per call,
  and passes its jig requests on (Choice 1). `act` returns `None`,
  `build` does nothing; `max_out` and `worst_case` give the larger arm.
- `agent::new`, the constructor; `Joint::client_limits(client, limits)`;
  `fixtures::{agent, drain, give, answered, call, done_task}`.
Test shape: `tests/client/tests/agent.rs`, on `fixtures::world`; the
agent from `fixtures::agent` (ctl's preset). The reference story:
```rust
#[test]
fn a_chat_the_agent_starts_is_seen_by_the_person() {
    let mut world = fixtures::world(fixtures::SEED);
    let agent = fixtures::agent(&mut world);
    let answer = fixtures::call(&mut world, agent, &["chat", "--project", "1", "Hello from the agent", "--key", "hello"]);
    assert_eq!(answer.status, 0, "{answer:?}");
    let task = fixtures::done_task(&answer);
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    world.person(person, vec![Step::See { find: Find::text(b"Hello from the agent"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 1);
}
```
Tests (world, each under 0.5 s):
- `a_chat_the_agent_starts_is_seen_by_the_person`;
- `the_agent_reads_a_chat_the_person_started`: `chat N` prints its
  words, exit 0;
- `an_agent_and_a_person_acting_at_once_keep_jigs_check`: a chat each,
  `chats` 2, verdict passed.
Proof: joint, the client's joint world, once: `cargo nextest run
-p temper-client-world -E 'test(a_chat_the_agent_starts_is_seen_by_the_person)'`
(under 0.5 s). It shows the tools' root asking temper's engine root
beside the fake person, under jig's check. Nothing higher: Choice 6.
Don't: give the agent a view, a page or an address; translate the
agent's requests under the person's limits; copy jig-client's or the
root's logic into the world.
Size: at most 280 lines in `tests/client/src`.
Removes: nothing.

### 06.2 A turn too large for a page, read whole

Scope: a chat with a turn past the page bound. `chat N` shows that
turn as the line of 5.2, and the agent reads it whole by following each
`--part K --from B` call that `turn` prints (5.2; 11.2; 12, the tools'
window).
Needs: 06.1 (the read of one turn comes with the tools' root, through
temper 03.1).
Files: `tests/client/src/{front.rs, fixtures.rs, translate.rs}` (a
read's answer decoded under the asking client's codec bounds, where
`fixtures::codec`, 32 KiB with 8 KiB of a turn's text, serves every
client today); `tests/client/Cargo.toml` (`temper-worker-domain-world`);
`tests/client/tests/agent.rs`.
Shape:
- `Joint::with_large_turns(seed, limits)`, beside `with_goal_proposals`,
  and `fixtures::world_with_large_turns(seed)`, build the engine from
  `forge_limits()` with `journal.transcript_bytes` and
  `loads.transcript_bytes` at `fixtures::TURN_BOUND`
  (4 × `LOAD_BYTES`), and any bound the engine's limits check ties to
  them. A comment there names 11.2's reason.
- `fixtures::large_turn(front, task) -> (u64, u32)` commits, through
  `temper_engine_domain_world::protocol::turn`, a body from
  `temper_worker_domain_world::protocol::transcript_turn(1,
  LARGE_TURN, true)`, where `LARGE_TURN` is `LOAD_BYTES` plus 16 KiB.
- `fixtures::read_with(&Answer) -> Vec<String>` copies a printed call;
  `fixtures::read_whole` follows the parts.
Test shape: in `tests/client/tests/agent.rs`, on
`fixtures::world_with_large_turns(seed)`. Tests (world, each under
0.5 s):
- `a_turn_too_large_for_a_page_is_read_whole_by_its_parts` (the
  reference). The parts joined are the turn's text, `LARGE_TURN`
  bytes, and none is past the tools' window.
- `a_chat_page_shows_a_large_turn_by_its_size_and_its_call`. The
  printed `read with:` call names the task and its attempt and turn.
Proof: joint, once: `cargo nextest run -p temper-client-world -E
'test(a_turn_too_large_for_a_page_is_read_whole_by_its_parts)'`
(under 0.5 s). It shows jig's read of one turn through the tools,
whole, from temper's engine. Nothing higher: Choice 6.
Don't: lower the page bound or `LOAD_BYTES`; restart the engine in
these stories (`Joint`'s restart rebuilds from `forge_limits()`); print
a turn's bytes in a page.
Size: at most 120 lines in `tests/client/src`.
Removes: the single codec bound for every client's reads.

### 06.3 A repository adopted

Scope: `adopt`, a forge-facing increment of its own: adopted exits 0
with 4.7's sentence, a repository the forge lacks exits 2 (client/tools.md,
5.3, 6.3; `client.md`, 4.4, 4.7; 11.2).
Needs: 06.1.
Files: `tests/client/tests/agent.rs`. No hotspot: the forge family goes
through `translate::forge::{request, incoming}` as they stand.
Shape: no new type.
Test shape: in `tests/client/tests/agent.rs`, 06.1's kind. The first
checks exit 0, stdout "Adopted acme/shop on forge 1 as owned." and
`front.engine_adopted(fixtures::FORGE, 7)`.
Tests (world, each under 0.5 s):
- `a_repository_the_agent_adopts_is_held_by_the_engine`;
- `adopting_a_repository_the_forge_lacks_exits_2_not_found`: "No
  repository acme/none on forge 1 that temper's forge user can see.";
- `an_adoption_asked_again_under_its_name_gets_the_first_answer`.
Proof: joint, once: `cargo nextest run -p temper-client-world -E
'test(a_repository_the_agent_adopts_is_held_by_the_engine)'` (under
0.5 s). It shows the forge family's request from the tools' root,
adopted by the engine root. Nothing higher: Choice 6.
Don't: change the fake forge, `tests/client/src/translate/forge.rs`,
temper-client-domain's `forge.rs` or the engine's forge component.
Size: at most 20 lines in `tests/client/src`.
Removes: nothing.

### 06.4 Cut calls made once, and a name told `Held`

Scope: 11.2's faults through the tools.
- A chat whose answer is lost (`FrontFault::NoAnswer`: the engine took
  it) is cut and asked again on the same DIRECTORY: a `kept:` line, then
  the first answer.
- A chat cut by an engine restart (`FrontFault::Restart`) is asked
  again from another DIRECTORY and gets the first answer.
- Two calls in one root under one name: the second is told `Held` and
  exits 1, with nothing sent (client/tools.md, 7, 8.2; jig's
  `client/domain/client.md`, 6).
Needs: 06.1.
Files: `tests/client/src/fixtures.rs`; `tests/client/tests/agent.rs`.
Shape: `fixtures::step_until(world, until: impl Fn(&mut Joint) ->
bool)`. A restart is drawn once `fixtures::chats` counts the chat and
before the agent's root has its answer.
Test shape: in `tests/client/tests/agent.rs`. The reference story:
```rust
#[test]
fn a_chat_cut_by_a_lost_answer_is_asked_again_and_made_once() {
    let mut world = fixtures::world(fixtures::SEED);
    let agent = fixtures::agent(&mut world);
    let argv = ["chat", "--project", "1", "Hello", "--key", "hello"];
    world.fault(FrontFault::NoAnswer);
    fixtures::give(&mut world, agent, &argv);
    fixtures::step_until(&mut world, |front| fixtures::chats(&front.engine) == 1);
    world.reload(agent);
    let answer = fixtures::call(&mut world, agent, &argv);
    assert!(answer.err.starts_with(b"kept: hello"), "{answer:?}");
    assert_eq!(fixtures::done_task(&answer), world.front().engine.last_assignment().task);
    assert_eq!(fixtures::chats(&world.front().engine), 1);
}
```
Tests (world, each under 0.5 s):
- `a_chat_cut_by_a_lost_answer_is_asked_again_and_made_once`;
- `a_chat_cut_by_an_engine_restart_is_asked_again_from_another_directory_and_made_once`;
- `a_name_the_root_holds_already_is_told_held_and_nothing_is_sent`
  (exit 1, `chats` 1, the first call exits 0);
- `a_call_asked_again_after_its_answer_gets_the_first_answer`.
Proof: joint, once: `cargo nextest run -p temper-client-world -E
'test(made_once)'` (two stories, each under 0.5 s). It shows a named
call cut by a lost answer or a restart and asked again, made once,
under jig's check. Nothing higher: Choice 6.
Don't: give a cut call another name; drain kept records in the
`Held` story (Choice 7); assert on the text of `Held`'s line beyond
its exit (temper 03's).
Size: at most 60 lines in `tests/client/src`.
Removes: nothing.

### 06.5 The chat followed and closed; a held task released

Scope: `wait task N --until replied` after `chat`, the front answering
runs; `message N TEXT --key K`, then the wait again (5.4's example);
`close N --key K`, then `wait task N --until ended`; a held task
released by the act `task G --tree` printed (client/tools.md, 5.2 to 5.4;
11.2); the STATUS rows.
Needs: 06.1; the tools' waits.
Files: `tests/client/tests/agent.rs`; `tests/client/src/fixtures.rs`
(`printed_act`); `docs/design/STATUS.md`.
Shape:
- `fixtures::printed_act(&Answer, tool: &str) -> Vec<String>`, the
  first act line naming `tool`.
- STATUS: the `domain/testing.md` row's "stand" list gains "the agents' tools'
  area 1 stories in `tests/client` (6.3)", and its clause on the agents'
  tools' stories keeps 2.6 alone, as the plan's landing left it. The `client/tools.md` row says "11.2: area 1's stories in
  `tests/client` stand".
Test shape: in `tests/client/tests/agent.rs`, of 06.1's kind, with
`world.front().answer_runs = true` (as `tests/client/tests/chat.rs`):
`wait task N --until replied` exits 0, then `chat N` prints the run's
"Answered by the run".
Tests (world, each under 0.5 s):
- `the_agent_waits_for_its_chats_reply`;
- `a_message_is_followed_by_its_reply_waited_for`;
- `a_chat_the_agent_closes_ends`;
- `a_held_task_the_agent_releases_from_the_printed_act_runs_again`,
  on `fixtures::goal_with_held_delegate`, with tries reset as in
  `tests/client/tests/task.rs`.
Proof: joint, once: `cargo nextest run -p temper-client-world -E
'test(the_agent_waits_for_its_chats_reply) |
test(a_held_task_the_agent_releases_from_the_printed_act_runs_again)'`
(each under 0.5 s). It shows a wait met by the run's reply, and a
release copied from a printed act that resets the held task's tries,
under jig's check. Nothing higher: Choice 6.
Don't: build the release's revision from the engine's state; wait on
`chat`, `activity` or `turn`, which are pages, not subjects (5.4).
Size: at most 40 lines in `tests/client/src`.
Removes: "6.3" from the `domain/testing.md` row's "not planned" clause.

## Interface sketches

The names of temper 02 and 03 below are placeholders,
`temper_client_tools::{Call, Root}`. The ready check aligns them with
those sessions' sketches. `Agent` is owned by the world and ends with
its client. A reload builds a new one from `agent::new`.

```rust
// tests/client/src/app.rs (06.1)
/// A client's root: the person's, or the scripted agent's.
pub enum Root {
    /// temper's client root, read through the view by the fake person.
    Person(domain::Domain),
    /// The tools' root (client/tools.md, 4), given calls by the story.
    Agent(Box<crate::agent::Agent>),
}
/// A client's view: the person's, or an empty tree no one reads.
pub enum View { Person(view::View), Agent(skein_ui::Tree) }
pub enum Event {
    // `Root(domain::Event)` and `Start { address, kept }` as today.
    /// One call the story gives the agent's root; a person's root panics.
    Call { call: temper_client_tools::Call },
}
impl Root {
    pub fn person(&self) -> &domain::Domain;
    pub fn agent(&self) -> &crate::agent::Agent;
}

// tests/client/src/agent.rs (06.1)
/// The tools' root, and what it printed, call by call.
pub struct Agent { root: temper_client_tools::Root, answers: Vec<Option<Answer>>, kept: Vec<Box<[u8]>> }
/// One call's printed result (client/tools.md, 6): stdout, stderr and the exit.
#[derive(Debug)]
pub struct Answer { pub status: u8, pub out: Box<[u8]>, pub err: Box<[u8]> }
/// The constructor `World::client_with` takes, under ctl's preset.
pub fn new(seed: u64) -> (Root, View);
impl Agent {
    pub fn calls(&self) -> usize;
    pub fn answer(&self, call: usize) -> Option<&Answer>;
}

// tests/client/src/fixtures.rs (06.1); `give`, `answered`, `done_task` beside them
pub fn agent(world: &mut World<Temper, Joint>) -> ClientId;
/// 8.2's step 4: runs until the agent's kept records are answered and erased.
pub fn drain(world: &mut World<Temper, Joint>, agent: ClientId);
/// `drain`, then the call given and its answer awaited, panicking past `CALL_WITHIN`.
pub fn call(world: &mut World<Temper, Joint>, agent: ClientId, argv: &[&str]) -> Answer;
```

## Questions to raise when reached

- If jig's check, recording the agent's release (06.5), refuses it for
  want of a card or a tree, raise it with the recorded verdict. The
  check's tree half is the person's alone (jig's
  `client/domain/testing.md`, 6.2).
- If the agent client's heap, measured under ctl's preset, passes
  `fixtures::world`'s 64 MiB per client, raise it with the measure.
  Never lower the preset to fit.
- If jig 03's read of one turn turns out to be bounded by the page's
  `load_bytes`, no turn is both too large for a page and readable
  whole. Raise it with both bounds as found.
- If the engine's limits check or the hub refuses 06.2's raised turn
  bounds or its turn's body, raise it with the refusal.
- If the engine answers the chat within the step that takes it, no
  restart can fall between the two. Raise it with the trace.
- If the session's stories, measured, put the worlds below the domain
  past 4 s of the default suite (`domain/testing.md`, 6.1), raise it with the
  times.

## Done when

- 06.1 to 06.5 have landed on temper's `origin/main`, each proof passed
  once.
- `git -C <worktree> show origin/main:tests/client/Cargo.toml | grep -n
  temper-client-tools` prints one line.
- `git -C <worktree> grep -n -w temper_client_tools origin/main --
  tests/client/src` prints at least one line.
- `for t in chat message turn adopt release close replied; do git -C
  <worktree> grep -q -F "\"$t\"" origin/main -- tests/client/tests ||
  echo "$t"; done` prints nothing.
- `git -C <worktree> grep -n -F 'LOAD_BYTES' origin/main --
  tests/client/src` prints at least one line.
- `git -C <worktree> show origin/main:docs/design/STATUS.md | grep -n
  -F "the agents' tools' area 1 stories in \`tests/client\` (6.3)"`
  prints one line.
