# Session 17: the fake git program and the one-process world

temper's simulated world (domain/testing.md, 2.5): `temper-service`'s one
process, started from configuration's bytes as `temper serve` starts it,
on skein's simulator and fake machine, with smith's agent processes over
skein's fake LLM peer, the fake forge as a service, a person through
temper's native client, and, for each git invocation, the fake git
program. The program comes first (17.1): git as the machine runs it
below the git component, a program hosted by skein's world harness on
the fake machine's files, reaching the fake forge through its typed
transport, and checked against real git's transcripts. Then the world
and its first story, a chat answered (17.2); a repository adopted and a
change pushed, and a merge in progress pushed with both parents (17.3);
and a drain, kills and power loss at drawn moments, and the referee's
secrets check (17.4). It is ordinary Rust: worlds, a hosted program and
their referee.

## Read first

- `domain/testing.md`, sections 2.5, 3.2, 4.2, 4.3, 5.2, 6 and 6.1;
  `protocol.md`, section 8; `git.md`, sections 3 to 6 and 9;
  `forge.md`, section 9 ("A commit the fake counts is a 40-digit id");
  `credentials.md`, section 7.
- `decisions.md`, 18; `code-organization.md`, sections 2, 4 and 5.
- `findings/jig-service.md`, section 4; `findings/temper-worker.md`,
  section 5 and Finding 6; `findings/skein-smith.md`, D.
- At the pins: jig's `tests/one-process/{Cargo.toml, src/world.rs,
  src/referee.rs, src/referee/agent.rs, src/fixtures.rs,
  src/world_signals.rs, tests/one_process.rs, tests/fuzzy_one_process.rs}`
  (the world this one copies); skein-world's `src/program.rs`
  (`Inherited`, `HostedProgram`, `StartupRoot`, `Machine`), `src/world.rs:161-330`
  (`new_controlled`, `with_machine`, `host`, `host_roots`,
  `spawn_restartable`, `cut`), `src/writer_pipe.rs` (`PipeHost`);
  skein-shell's `src/drive.rs:12-41` (`Host`, its `exit`);
  skein-fake-machine's `src/fs.rs` (`lay`, `reopen_root`, `crash`,
  `close`); skein's `docs/design/fake-checkout.md`, section 4, and
  `testing/skein-fake-checkout/src/git.rs` (`Remote`, `clone_repository`,
  `fetch`, `create`, `check_out`, `merge`, `commit`, `commit_merging`,
  `push`, `push_expected`).
- temper's `tests/worker/checkout/src/forge.rs` (`Forge`, `:95-110`, and
  its `Remote` over the fake forge's domain, `:276`), the precedent for
  the fake git's transport.
- What this session takes from others: 07.2's
  `tests/worker/protocol-git/src/transcripts.rs` and
  `temper_worker_protocol_git::{invocation, output}`; 10.1's
  `temper_service::startup::{named, startup}`; 12.1's
  `temper_shell_terminal::client::NativeClient`; 14.3's
  `testing/temper-fake-forge-protocol`; 15.1's forge page (17.3); their
  session files.

## What the design asks

- **What runs** (domain/testing.md, 2.5): "`temper-service`'s system module, the
  engine's and one worker's services in one `iterate`, started from
  configuration's bytes as `temper serve` starts them, joined by the
  in-process name `workers`; the store, jig's store component on
  skein-kv, on the simulator's crashing disk; the worker's agents,
  smith's agent processes as `temper agent` runs them, hosted as programs
  of the world, over skein's fake LLM peer; the fake git program for
  every git invocation (4.3); the fake forge as a service, its protocol
  layer over its domain (4.2); people: jig's fake person in front of
  temper's native client".
- **Stories** (2.5): "a person signs in by the link the engine prints,
  and a chat is answered; a repository is adopted through temper's
  client, and a goal's change is prepared, committed and pushed on the
  fake forge through the fake git program; a merge in progress is
  resolved and pushed with both parents; a signal mid-run drains the
  engine, the worker ends, and the next start settles the worker's
  claims at once; the service is killed, or the machine's power lost, at
  a drawn moment, then started again on the same disk, an adoption
  pending among what it finds."
- **Its referee** (2.5): jig's conformance referee through bytes; "the
  engine's referee over the fake forge (5.2): nothing lands on a default
  branch without green CI, and a run's change is the tree its agent
  left"; "no git identity's value or API token in a domain's state, a
  store record or an agent process's environment".
- **The fake git program** (4.3): "a program hosted by skein's world
  harness (its `HostedProgram`, spawned by name), acting on the fake
  machine's files"; it takes "the invocations of git.md, section 4, with
  their settings (git.md, section 3) and their environment (git.md,
  section 5), and nothing else. Any other argument vector exits as git's
  usage error does"; "keeping its git state in each repository's `.git`
  as skein's fake checkout keeps it, and reaches the fake forge through
  its typed transport"; "it prints ... what real git prints for those
  invocations"; "the credential only in `GIT_CONFIG_*`'s
  `http.extraHeader`, for an identity the fake forge knows, a wrong one
  answered as Forgejo answers it; no `PATH`; the settings present; the
  remote a `https` URL, or a loopback `http` one"; "its answers to the
  transcripts of `tools/git-transcripts` match real git's, masked".
- **Budget** (2.5, 6.1): "a story aims at 1 s; sweeps of drawn kills are
  fuzzy tests"; code-organization.md, 5 gives `tests/one-process` "1.0 s
  and 6 s".

## Choices this plan makes

- **The fake git program runs a typed git over the fake machine's
  files.** For one invocation it reads the repository's files beneath
  the worker's root (its startup root `worker`, the spawn's `dir` below
  it) into a `skein_fake_checkout::Checkout`, with its `.git` markers
  (`.git/objects/<id>`, `.git/temper-head`, `.git/MERGE_HEAD`,
  `.git/temper-conflicts/<i>`, fake-checkout.md, 4) and
  `.git/refs/temper/fetched` for the one reference the component writes;
  runs the matching `skein_fake_checkout::git` function against its
  transport; writes back what changed; prints what real git prints on
  its pipes; and exits with git's code (`Host::exit`). A world's
  repositories are a few small files, so reading them per invocation
  costs little.
- **Its arguments are read back through the builders:** `check` parses
  argv into 07.1's `invocation::Command` and refuses it unless
  `invocation::arguments` of that command gives argv back byte for byte;
  the transcripts' argv (17.1's test) hold the builders to real git.
- **Its transport is the world's handle on the fake forge's domain,**
  routed as `tests/worker/checkout/src/forge.rs`'s `Forge` routes a
  working tree's git, by the user the header's token stands for in the
  scenario's token table. `HostedProgram::make` is a `fn` pointer
  (`program.rs:28`), so the world keeps the forge's server in an
  `Rc<RefCell<..>>` that a `thread_local!` cell of the world hands both
  the forge's process and each git's factory; the test sets it before
  `World::run` and takes it after; no thread is spawned.
- **Ids:** a commit the fake counts is 40 hex digits, its count
  big-endian in the first eight bytes, then twelve zero bytes
  (forge.md, 9).
- **The world copies jig's `tests/one-process`:** its `Proc` enum, the
  restartable service with the roots `state` and `worker`, the machine
  wrapper (Kill closes held handles; power loss is `crash(seed)`), the
  conformance referee through bytes; temper's adds `Git` and `Forge`
  processes, and forwards `Host::exit` (jig's `Proc` does not: a git's
  exit code would read 0).
- **The configuration is the world's own JSON** in `src/fixtures.rs`,
  written after 10.4's process support's (`crates/temper/tests/support/mod.rs`),
  with the agent program `/opt/temper/bin/temper agent
  /opt/temper/agent.json`, `git` at `/usr/bin/git`, the forge's API and
  `git` base at the forge process's loopback address over plaintext
  http, and the identities' and the forge's token files among the files
  handed to `startup`.
- **The chat story lands with the world (17.2)** as its proof, the
  sign-in inside it; 17.3 adds the two git stories. Focused tests: six,
  all in `tests/one_process.rs`: four stories (chat, adoption and push,
  merge, drain), each aiming at 0.25 s, and two referee tests (secrets,
  the store's records after a restart) of milliseconds, so the world
  keeps its 1.0 s share; if the six pass 1.0 s, the merge story moves to
  the fuzzy binary first.

## Rules that bind it

- "**The new worlds' share of the suites** ... `tests/one-process`
  1.0 s and 6 s ... A world past its share is made cheaper before more
  stories land in it; past twice it, a question" (code-organization.md,
  5).
- "**No thread in a test,** even for independent seeds: a test that
  would run several seeds in parallel is several tests"
  (code-organization.md, 5).
- "**Secrets in tests:** every world with a protocol layer checks that
  no value its fakes issued (an LLM access token, a git identity's
  token, an API token) appears in a root's events or requests, a fact, a
  store record or a log" (code-organization.md, 5); and "**Transcripts
  are read as files** ... never re-recorded by a test".

## Increments

### 17.1 `testing/temper-fake-git`

Scope: the crate `testing/temper-fake-git` (ordinary Rust: its
`clippy.toml` a copy of `tests/clippy.toml`, as `testing/` has none
above it; its `[lints]` copied from `tests/worker/domain/Cargo.toml`):
`check.rs` (`check(spawn) -> Result<Checked, Refusal>`: argv read back
through 07.1's builders; git.md, 3's settings present, in order; the
environment holding the names git.md, 5 lists and no other, no `PATH`;
the credential only in `GIT_CONFIG_VALUE_0`, `Authorization: Basic` and
the base64 of `<login>:<token>`; the remote `https`, or `http` on
loopback), `answer.rs` (`answer(checked, checkout, transport) ->
Answer`: each command of git.md, 4 on a `Checkout`, its stdout as git
2.53 prints it (porcelain lines, NUL lists, ids), its exit code, and,
for the forge's refusals and faults, the stderr line git.md, 6's table
reads: a missing repository, branch or commit, a wrong token (Forgejo's
401), a protected branch (`[remote rejected] … declined`), the forge out
of reach; argv it does not take exits 129 with git's usage line),
`transport.rs` (`Transport`, `Remote` over the fake forge's domain by
the token's user), `program.rs` (`Git`, a `skein_shell::Host`: reads the
repository's files with kernel operations beneath its startup root,
answers, writes back, writes its pipes, exits with the code), `id.rs`
(40-digit ids of counts).
Needs: 07.2's builders, parsers and transcripts world; 14.3's fake forge
protocol (its domain's handle, the scenario's token table).
Files: `testing/temper-fake-git/{Cargo.toml, clippy.toml, src/{lib.rs,
check.rs, answer.rs, transport.rs, program.rs, id.rs}, tests/check.rs,
tests/transcripts.rs}`; the workspace `Cargo.toml`. Shared hotspot: the
workspace `Cargo.toml`.
Shape: as the sketch below.
Test shape: `tests/check.rs`:
`an_argument_vector_the_builders_did_not_make_exits_129`,
`a_token_outside_the_header_or_a_path_variable_is_refused`,
`a_remote_over_plaintext_off_loopback_is_refused`; `tests/transcripts.rs`
(reading 07.2's `transcripts::read`):
`each_transcript_s_exit_and_masked_stdout_match_the_fake_s`,
`each_failure_transcript_s_stderr_classifies_as_the_fake_s`,
`a_wrong_token_is_answered_as_forgejo_answers_it`. Fixtures in
`src/fixtures.rs`, a public module 17.2 uses too: a forge with one
repository `acme/app` of two commits, its token table, the equivalent
`Checkout` for each case, `answer_case(case, fault)`, `spawn_of(command)`
(the spawn the component would make for `command`, its identity
`fixtures::IDENTITY`). The rows name 05.4's case directories; 05.4's
landed directory names win. Under 20 ms each. Reference tests, in full,
one per test file:

```rust
// testing/temper-fake-git/tests/check.rs
#[test]
fn an_argument_vector_the_builders_did_not_make_exits_129() {
    let mut spawn = fixtures::spawn_of(&invocation::Command::WriteTree);
    let mut args = invocation::arguments(&invocation::Command::WriteTree).into_vec();
    args.push(Box::from(&b"--missing-ok"[..]));
    spawn.args = args.into_boxed_slice();
    assert_eq!(temper_fake_git::check(&spawn).err(), Some(Refusal::Usage));
}
```

```rust
// testing/temper-fake-git/tests/transcripts.rs
#[test]
fn each_failure_transcript_s_stderr_classifies_as_the_fake_s() {
    let rows = [
        ("clone-missing-repository", fixtures::Fault::MissingRepository, git::Kind::Clone),
        ("fetch-missing-branch", fixtures::Fault::MissingBranch, git::Kind::Fetch),
        ("fetch-wrong-token", fixtures::Fault::WrongToken, git::Kind::Fetch),
        ("push-protected", fixtures::Fault::ProtectedBranch, git::Kind::Push),
        ("clone-refused-connection", fixtures::Fault::Unreachable, git::Kind::Clone),
    ];
    for (case, fault, kind) in rows {
        let real = transcripts::read(case);
        let fake = fixtures::answer_case(case, fault);
        assert_eq!(fake.exit, real.exit, "{case}: exit");
        assert_eq!(output::classify(&fake.stderr, kind), output::classify(&real.stderr, kind), "{case}: stderr");
    }
}
```

Proof: world, `cargo nextest run -p temper-fake-git`, run once:
the fake refuses what the builders did not make, and answers each
transcript's case with real git's exit, masked stdout and a stderr that
classifies alike (the six tests above); under 0.2 s.
Don't: run a program of the machine's own; answer an invocation git.md
does not list; re-record a transcript; let a token reach the fake's
files or its output.
Size: at most 600 lines (test support, ordinary Rust); its allowance
2000, its parts summed (decisions.md, ruling 142).
Removes: nothing.

### 17.2 The world, and a chat answered

Scope: the world `tests/one-process` (`temper-one-process-world`,
ordinary Rust, its `[lints]` copied from `tests/worker/domain/Cargo.toml`,
`tests/clippy.toml` applying by its directory):
`src/world.rs` (jig's, copied: `Proc { Service, Agent, Git, Peer, Forge,
Client }`, each a `skein_shell::Host`, `exit` forwarded; the service from
10.1's `startup` with the world's document and files, restartable with
the roots `state` and `worker`; smith's agent hosted as
`/opt/temper/bin/temper`; the fake git hosted as `/usr/bin/git` with the
startup root `worker`; the fake LLM peer and the fake forge's server as
processes on loopback; the person: jig's fake person in front of 12.1's
`NativeClient`, signing in by the link the service prints (jig's
referee pops `Service::links`)), `src/machine.rs`, `src/forge.rs` (the
shared forge handle and its `thread_local!` cell), `src/referee.rs`
(jig's conformance referee through bytes, `Service::fmt_roots` for the
roots), `src/fixtures.rs` (the document, the files, the seeds, `calm`;
the document's `engine.limits.overrides` name `documents.rows` 2 and
`documents.text_bytes` 128, as configuration.md, 7 says: "The engine's
documents fit temper's client").
Needs: at the session's start: 14.3's fake forge protocol, 12.1's
native client, 10.1's startup, 07.2's transcripts; 17.1.
Files: `tests/one-process/{Cargo.toml, src/{lib.rs,
world.rs, machine.rs, forge.rs, referee.rs, fixtures.rs},
tests/one_process.rs}`; the workspace `Cargo.toml`. It adds
`skein-fake-peers`, `smith-agent-process-world` and `smith-agent-shell`
where `Cargo.lock` does not hold them yet: code-organization.md, 3's
three `cargo update --precise` commands then run, and the check prints
the three pins.
Shape: as the sketch below (`world::{run, Outcome, Story}`).
Test shape: `tests/one_process.rs`:
`a_person_signs_in_by_the_printed_link_and_a_chat_is_answered`, aiming
at 0.25 s, at most 1 s. Reference test, in full:

```rust
#[test]
fn a_person_signs_in_by_the_printed_link_and_a_chat_is_answered() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), world::Story::Chat { words: b"Hello" });
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.signed_in, 1);
    assert_eq!(outcome.chats_answered, 1);
}
```

Proof: joint, `cargo nextest run -p temper-one-process-world`, run
once: the real service, started from configuration's bytes, prints its
link, the person signs in by it, and a chat is answered by the
in-process worker's agent over the fake LLM (the test above); under 1 s.
Don't: start the service other than through 10.1's `startup`; spawn a
thread; give the person a link other than the printed one.
Size: world code; no production line.
Removes: nothing.

### 17.3 An adoption and a pushed change; a merge pushed with both parents

Scope: two stories (domain/testing.md, 2.5): a repository adopted through
temper's client (the person on 15.1's forge page) and a goal's change
prepared, committed and pushed on the fake forge through the fake git
program; and a merge in progress (the base moved under the branch,
conflicting in one file) resolved by the agent and pushed with both
parents. The referee gains the engine's checks over the fake forge's
observations (domain/testing.md, 5.2): nothing lands on a default branch
without green CI, and a pushed commit's tree is the tree the agent left.
The fake LLM's scripts write their edits and resolutions in the shell
subset (domain/testing.md, 4.3).
Needs: 15.1's forge page and form.
Files: `tests/one-process/{src/world.rs, src/referee.rs,
src/fixtures.rs, src/stories.rs (new), tests/one_process.rs}`.
Shape: `world::Story::{Adopt, Merge}`; `Outcome` gains `adopted`,
`pushed: Vec<Pushed { branch, commit, parents }>`.
Test shape: `tests/one_process.rs` gains
`an_adopted_repository_gets_its_change_pushed_through_git` and
`a_merge_in_progress_is_pushed_with_both_parents`, each aiming at
0.25 s, at most 1 s; the focused tests at most 1.0 s summed, else
the merge story moves to `tests/fuzzy_one_process.rs`. Reference test,
in full:

```rust
#[test]
fn a_merge_in_progress_is_pushed_with_both_parents() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), world::Story::Merge);
    assert_eq!(outcome.verdict, Verdict::Passed);
    let [pushed] = outcome.pushed.as_slice() else { panic!("one push: {:?}", outcome.pushed) };
    assert_eq!(pushed.parents, [fixtures::BRANCH_HEAD, fixtures::MOVED_BASE]);
}
```

Proof: joint, `cargo nextest run -p temper-one-process-world`, run
once: an adopted repository's change reaches the fake forge through the
fake git program, and a resolved merge lands with both parents (the two
tests above, each under 1 s); the process tier's real git is 15.3's.
Don't: route git around the fake git program; let the fake forge accept
a push the transport did not carry.
Size: world code; no production line.
Removes: nothing.

### 17.s1 The shipped profile admits a merge's conflicts

A step not in the plan (decisions.md, ruling 153), before 17.4.
Scope: the profile `small`'s checkout limits
(`crates/temper-service/src/limits.rs:170-173`) take `conflicts` 4 and
`path_bytes` 255 in place of 0 and 0, each doc line naming what it
bounds and where its number comes from (the checkout merges world;
`largest_output` within `stdout_bytes` 1,024); 17.3's two overrides,
`checkout.conflicts` and `checkout.path_bytes`, leave the world's
document.
Files: `crates/temper-service/src/limits.rs`,
`tests/one-process/src/fixtures.rs`.
Test shape: no new test; every test's name and expected value kept.
Proof: joint, `cargo nextest run -p temper-one-process-world
--ignore-default-filter`, once: the merge story passes with the two
overrides gone; the gate.
Don't: change `stdout_bytes`, another profile constant or the
configuration document's defaults; a startup or worst-case check that
now refuses the profile is a question naming the check and its numbers.
Size: at most 10 production lines.

### 17.s2 The referee and a two-turn chat

A step not in the plan (decisions.md, ruling 154), after 17.s1, before
ruling 152's measure and 17.4.
Scope: name the cause of `Violation { promise: Commit, why: "outside
delivery preceded its durable prerequisite: Turn(1, 1, 1)" }`
(`src/referee.rs:420`) that 15.3's probe meets: the chat with one
`read` before its text and wait (`~/src/rust/tmp/temper/protocol/logs/temper-15/151-world-two-turn-{world,fixtures}.rs`,
`151-world-two-turn.log`, 0.150 s). A misreading by the referee is
fixed in the referee; the probe then lands as a story in
`tests/fuzzy_one_process.rs`, named for the behaviour, its reply
asserted.
Files: 17's own (ruling 108), `tests/fuzzy_one_process.rs`.
Test shape: one story, at most 1 s; every existing assertion kept.
Proof: joint, `cargo nextest run -p temper-one-process-world
--ignore-default-filter`, once; the gate.
Don't: loosen a promise of the referee, or make the story pass by
finishing the chat without its wait; edit production code: a turn the
service delivers before it is durable is a question naming its site.
Runs: at most five world runs, as ruling 148 says; past them, a
question with their logs.
Size: world code, at most 150 lines.

### 17.s3 Adopt is disabled until a party is confirmed

A step not in the plan (decisions.md, ruling 157), from `origin/main`,
landed before 17.4.
Scope: `forge::Form` (`crates/temper-client-domain/src/forge.rs`)
gains `confirmed: bool`, true while the side's `party` is some; the
forge page (`crates/temper-client-view/src/forge/page.rs:75-81`)
disables Adopt while it is false, beside the pending and out-of-date
cases.
Files: those two files, their users (ruling 123), the view's
`src/tests/forge_page.rs` and the domain's fixtures.
Test shape: one view test,
`adopt_is_disabled_until_a_party_is_confirmed`, under 20 ms; every
existing test's name and expected value kept, a fixture's form gaining
`confirmed: true` where its test presses Adopt.
Proof: unit, that test; the gate.
Don't: queue or park a press made before confirmation; change ruling
124's scope or the request table.
Size: at most 30 production lines.

### 17.4 A drain, kills and power loss, and the secrets check

Scope: the drain story: a signal mid-run drains the engine, the worker
ends, and the next start on the same disk settles the worker's claims at
once; the fuzzy sweeps: the service killed, or the machine's power
lost, before a drawn submission, then started again on the same disk,
an adoption pending among what it finds (the person's keyed request,
sent again, answered once); the referee's secrets check: no value the
fakes issued (the LLM access token, the git identity's token, the forge's
API token) in `Service::fmt_roots`'s output, a store record the world
observed, or a hosted agent's environment (`Spawn::env` as the agent's
factory saw it).
Needs: 02.4's durable outcome (`Stored::Application`: an adoption sent
again after a kill answered once).
Files: `tests/one-process/{src/world.rs, src/referee.rs,
src/fixtures.rs, tests/one_process.rs, tests/fuzzy_one_process.rs}`.
Shape: `world::{drain_mid_run, crash}` (jig's names);
`Outcome::{drained, restarts, claims_settled_at_restart}`.
Test shape: `tests/one_process.rs` gains
`a_signal_mid_run_drains_and_the_next_start_settles_the_claims`, aiming
at 0.25 s, at most 1 s. `tests/fuzzy_one_process.rs`:
`drawn_kills_keep_the_conformance_rules_and_answer_adoptions_once` and
`drawn_power_losses_keep_the_conformance_rules`, seeds one after
another, each at most 3 s, 6 s summed. The secrets check is the
referee's, failing any story that breaks it; its own test, in `tests/one_process.rs`,
`a_secret_in_a_root_fails_the_referee_naming_it`, feeds the check a
formatted root holding the fixture's token. The store's check (store/README.md,
6: "after each restart the forge's records are those of the last
acknowledged commit, or of that and one whole uncertain one, never part
of one"): at each restart the referee reads the forge's records the
restarted service loads (the world's store observer, as jig's world
captures commits) and compares them with the last commit the store
acknowledged before the cut and the one uncertain after it; any other
set fails, naming the record. Its own test, in `tests/one_process.rs`,
`a_restart_showing_part_of_a_commit_fails_the_referee`, feeds the check
a restart holding one of an uncertain commit's two forge records. Both
referee tests run no world and take milliseconds. Reference test, in
full:

```rust
#[test]
fn drawn_kills_keep_the_conformance_rules_and_answer_adoptions_once() {
    for seed in 1..=4 {
        let mut draw = skein_lib::Rng::new(seed);
        let before = 75 + u32::try_from(draw.below(59)).expect("a bounded draw");
        let outcome = world::crash(fixtures::calm(seed), skein_world::Cut::Kill, before);
        assert_eq!(outcome.verdict, Verdict::Passed, "seed {seed}, before {before}");
        assert_eq!(outcome.restarts, 1, "seed {seed}, before {before}");
        assert!(outcome.adoptions_answered <= 1, "seed {seed}, before {before}: one answer per key");
    }
}
```

Proof: joint, `cargo nextest run -p temper-one-process-world` and
`cargo nextest run -p temper-one-process-world --profile fuzzy`,
each run once: a drain ends the worker and a restart settles its claims,
drawn kills and power losses keep jig's conformance rules with each
adoption answered once and each restart's forge records whole commits,
and no issued secret shows in a root, a record or an agent's
environment (the three focused tests, the story under 1 s; the fuzzy
tests at most 6 s summed).
Don't: run seeds in threads; loosen the referee to pass a seed.
Size: world code; no production line.
Removes: nothing.

## Interface sketches

Ordinary Rust, not compiled (the scratch crate is step code): the fake
git program's surface and the world's. The world owns its processes and
the forge's handle; the fake git owns one invocation, from its spawn to
its exit; the service owns everything between.

```rust
// testing/temper-fake-git (ordinary Rust, not compiled)

use skein_fake_checkout::{Checkout, git::Remote};
use temper_worker_protocol_git::invocation::Command;

/// One invocation, as the fake takes it from a spawn.
#[derive(Debug)]
pub struct Checked {
    pub command: Command,
    /// The spawn's directory beneath the worker's root.
    pub at: Box<[u8]>,
    pub home: Box<[u8]>,
    /// `GIT_COMMITTER_*`, and `GIT_AUTHOR_*` for a commit.
    pub committer: (Box<[u8]>, Box<[u8]>),
    pub author: Option<(Box<[u8]>, Box<[u8]>)>,
    /// `<login>` and `<token>` from `GIT_CONFIG_VALUE_0`'s header.
    pub credential: Option<(Box<[u8]>, Box<[u8]>)>,
}

/// Why the fake will not run a spawn.
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Argv the builders did not make: git's usage error, exit 129.
    Usage,
    /// A rule of git.md, 3 or 5 broken (a token outside the header, `PATH`,
    /// a setting missing, a plaintext remote off loopback): the test fails,
    /// naming it.
    Broken(&'static str),
}

pub fn check(spawn: &skein_io::kernel::Spawn) -> Result<Checked, Refusal>;

/// What git prints and how it exits.
#[derive(Debug, PartialEq, Eq)]
pub struct Answer {
    pub exit: u8,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// One invocation on a repository's working tree and git state.
pub fn answer(checked: &Checked, checkout: &mut Checkout, transport: &mut Transport) -> Answer;

/// The fake forge as git meets it: the world's handle on the fake forge's
/// domain, by the user the header's token stands for.
#[derive(Debug, Clone)]
pub struct Transport { /* Rc<RefCell<..>> to the forge's server, its token table */ }
impl Remote for Transport { /* heads, fetch, create, push, parent, merge_parent, tree, message, store */ }

/// The hosted program: reads the repository's files, answers, writes back,
/// writes its pipes, exits with `Answer::exit`.
#[derive(Debug)]
pub struct Git { /* elided */ }
impl Git {
    pub fn new(spawn: &skein_io::kernel::Spawn, inherited: &skein_world::Inherited, transport: Transport) -> Git;
}
impl skein_shell::Host for Git { /* ...; fn exit(&self) -> Option<Exit> */ }

/// A counted commit's id: the count big-endian in the first eight bytes,
/// twelve zero bytes after them, as 40 hex digits (forge.md, 9).
pub fn id(count: u64) -> [u8; 40];
```

```rust
// tests/one-process (ordinary Rust, not compiled)

pub enum Proc {
    Service(Box<ServiceHost>),
    Agent(Box<skein_world::PipeHost<smith_agent_shell::Agent>>),
    Git(Box<temper_fake_git::Git>),
    Peer(Box<SignalHost<skein_fake_peers::llm::Peer>>),
    Forge(Box<ForgeHost>),
    Client(Box<SignalHost<temper_shell_terminal::client::NativeClient>>),
}
impl skein_shell::Host for Proc { /* each method delegated, `exit` included */ }

/// What a run of the world is asked to show.
pub enum Story { Chat { words: &'static [u8] }, Adopt, Merge }

pub struct Outcome {
    pub verdict: skein_world::domain::Verdict,
    pub signed_in: u32,
    pub chats_answered: u32,
    pub adopted: u32,
    pub adoptions_answered: u32,
    pub pushed: Vec<Pushed>,
    pub drained: bool,
    pub restarts: u32,
    pub claims_settled_at_restart: u32,
}

pub struct Pushed { pub branch: Box<[u8]>, pub commit: u64, pub parents: Vec<u64> }

pub fn run(settings: Settings, story: Story) -> Outcome;
pub fn drain_mid_run(settings: Settings, restart: bool) -> Outcome;
pub fn crash(settings: Settings, cut: skein_world::Cut, before: u32) -> Outcome;
```

## Questions to raise when reached

- **The forge's handle across processes.** 14.3's server owns the fake
  forge's domain, and the fake git program must reach the same domain
  "through its typed transport". Default: 14.3's server gives its domain
  by `&mut` (a `domain_mut`), and the world shares the server through
  its `thread_local!` cell; if 14.3 has no such access, the question is
  raised with 14.3's file and line.
- **The native client's host.** 12.1's `NativeClient` is assumed to be a
  `skein_shell::Host` that jig's fake person drives, as 12.2's process
  story drives it. Default: as 12.2 uses it; if it is not, the world
  wraps it as jig's world wraps jig's (`world_signals.rs`'s
  `SignalHost`).
- **`startup`'s signature.** 10.1's `startup` is assumed to take the
  document, the files `named` lists, the agent's profile, the seeds and
  the worker's runtime, as jig's reference's does. Default: the landed
  signature, with the world handing what it asks.
- **The world's share.** Four focused stories at 0.25 s each, and two
  referee tests, fill the 1.0 s share. Default: the merge story moves to the fuzzy binary first;
  past 2.0 s, a question (code-organization.md, 5).

## Done when

- The fake git program and the world pass: `cargo nextest run -p
  temper-fake-git -p temper-one-process-world` and `cargo nextest
  run -p temper-one-process-world --profile fuzzy` pass.
- The world keeps its share: `cargo nextest run --workspace
  --profile measure -j 1 -E 'package(temper-one-process-world) & not
  binary(/^fuzzy_/)'`, then `awk -F'[][]' '/PASS \[/ {s += $2} END
  {printf "%.1f\n", s}' <its log>`, prints at most 1.0; with
  `binary(/^fuzzy_/)` in place of `not binary(/^fuzzy_/)` and
  `--ignore-default-filter` added, at most 6.0.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/temper log -1 --grep='^Step: temper-17\.4$'
  --format='%(trailers:key=Gate-Result,valueonly)' origin/main` names
  `scripts/check.sh passed`.
