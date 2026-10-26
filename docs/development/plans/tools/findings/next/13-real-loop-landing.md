# Session 13: the real loop's landing

Measure the fake git program on the real filesystem (13.1), add the
landing's fake-LLM scripts in their own increment (13.2), then adopt a
repository through the tools and take a chat to a landed change (13.3).
The forge's content marker supplies CI, and its signed webhooks reach
the engine. These were 09.4 to 09.6; temper 09 keeps the tools stories.

Needs at start: **the real-loop world**. Steps: `temper-13.1` through
`temper-13.3`. Only 13.3 additionally needs **the tools' adopt**, plus
this session's measured git host and landing scripts. This session can
run alongside temper 09: neither needs the other's increments. They
share `tests/real-loop/Cargo.toml` and `tests/real-loop/src/lib.rs`;
coordinate their registrations on rebase, without a session-wide fence.

## Read first

- temper at 4e14c2b5: `docs/design/client/tools.md` 5.3, 5.4, 6, 8, 11.3;
  `docs/design/domain/testing.md` 2.4, 4.1 to 4.3, 6.1, 7, 10;
  `docs/design/client/README.md` 4.4 to 4.7 (adoption).
- This plan: `temper/08-real-loop.md` (the crate, `run_forge`,
  `forge::Fixture`, `Seen`, owner checks and deferred LLM construction);
  `temper/07-forge-host.md` (shared `Forge`, `bound`, `deliver_to`);
  `temper/05-ctl.md` (Startup and Invocation);
  `temper/11-tools-forge.md` (the tools' adopt);
  `temper/09-real-loop-stories.md`; `findings/temper-area-1.md` 5 and 6.
- The planner's decisions on real-loop proofs, content-marker CI and
  seams; designer round-2 answers 24, 25, 29 and 30. The specified
  tools contract here is 4e14c2b5, including names in kept results.
- Code: `testing/temper-fake-git/tests/transcripts.rs`, `Disk` and its
  four `hosted_*` cases; `testing/temper-fake-git/src/fixtures.rs`,
  `scenario`, `spawn_of`; `src/program.rs`, `Git::new`;
  `testing/temper-fake-forge-domain/src/api.rs`, `Setup`, `Checks`,
  `Cue`, `Protection`; `tests/one-process/src/stories.rs`, `forge`;
  `src/referee/forge.rs`, `observe`, `green`.
- Code: `crates/temper/tests/support/forgejo.rs`, `landing_scripts`,
  `configuration`; `crates/temper/tests/support/mod.rs`, `EDIT`.
  These are templates, not dependencies on executable test support.
- skein's `docs/foundation/programming-model.md` 2.1, 4.3, 6.2, 6.3,
  10.2; `docs/foundation/testing-strategy.md` 2.8, 4.1, 6, 7, 8, 9, 10;
  `testing/skein-world/src/real.rs`, `World::host_roots`.

## What the design asks

- **client/tools.md 11.3, domain/testing.md 2.4:** adopt, clone, edit, push and land
  through the fake git program and shared in-memory forge; hosted
  smith agents use real pipes and the fake LLM's scripted turns.
- **domain/testing.md 4.3 and 10:** measure fake git on the real ring's files
  before relying on it. The real-git bridge and sh-based CI are later.
- **2.4 and 4.2:** content decides CI; hooked repositories deliver
  signed webhooks, with short configured polling as a backstop.
- **client/tools.md 6:** adopted and successful reads/acts/waits exit 0;
  only engine answers exit 2. Client failures, Held included, exit 1.
  A cut call's status is the OS's, nonzero, with no caught signal.
  Kept lines carry name, operation and outcome, never either key;
  JSON has `name` (or null), `operation`, `outcome` on stderr.
- **5.4, 6.3:** an engine stop ends no watch. A disconnected wait
  fails only past its link deadline, exit 1; it cannot stand in for
  `--until ended`. **domain/testing.md 6.1:** a focused story stays within 1 s.

## Choices this plan makes

- **Use temper 08's world.** `temper-real-loop-world`, `run_forge`,
  `forge::Fixture`, `forge::fixture`, `Ran`, `Seen`, `delegate`, and
  `calls::Step` keep their names and owners. `repository::fixture`
  specializes 08's recipe; it does not start another forge or runner.
- **Use temper 05's startup.** 08's caller supplies a fresh parent
  startup root per invocation. `Invocation` names DIRECTORY's leaf;
  the cli host opens DIRECTORY itself. Neither git nor the landing
  fixture changes this ownership or hands the host a locked open.
- **Git shares the forge family's owner.** 13.1 registers the fake git
  factory against 08's shared `temper_fake_forge_host::Forge`, after
  07's move. All construction, iteration, transport clones and drops
  are charged to that owner; add git's bound to 08's checked sum.
  Snapshots are copied out, then shared handles are released.
- **LLM construction already waits for the forge.** 08's `llm.rs`
  invokes a scripts factory after `Forge::bound`. 13.2 supplies that
  factory; no new late-built peer, fixed endpoint or `llm.rs` change.
- **Land by the tools.** `adopt` supplies the repository, then the chat
  delegates the edit and review. CI passes only when `README.md` holds
  `landing::EDIT`. The ordinary story keeps 08's short polling; a
  second puts the three forge polling paces past the story's patience
  to show webhooks carry the landing. Neither changes client deadlines.

## Rules that bind it

- Keep each seam in its increment. 13.1 is forge/git wiring, 13.2 the
  LLM-accounts hotspot, 13.3 the forge fixture and story. Use existing
  fake APIs; no fake implementation, forge client/component or provider
  edit here. A fake-git defect gets a separate forge-family increment.
- Keep 08's owner, operation and teardown checks, profile builders and
  admitted limits. Overrides name their shipped value and reason beside
  it; recompute the combined bound when adding the git host.
- No fixed ports, readiness sleeps, real git, sh runner, optimized
  build or `[profile.*]` change. Every focused story stays within 1 s,
  within the lower worlds' 4 s guide and the suite's 15 s/60 s budgets.
  Profile a miss and make it cheaper or move it to `fuzzy_*`.
- No draft behaviour or design/STATUS edits. Failures keep trace and
  scratch and are reproduced at the lowest layer that shows them.

## Increments

### 13.1 The fake git program on the real filesystem, measured

Scope: domain/testing.md 2.4, 4.3 and 10's prerequisite measurement.
Needs: nothing beyond the session's start.
Files (forge/git wiring; exact paths): `tests/real-loop/src/git.rs`
(new), `tests/real-loop/src/world.rs`, `tests/real-loop/src/check.rs`,
`tests/real-loop/src/lib.rs`, `tests/real-loop/Cargo.toml`, `Cargo.lock`,
`tests/real-loop/tests/forge.rs` (new). No fake implementation edits.
Shape:
- `git::Disk` adapts the transcripts' Disk to Scratch and `real::World`.
  Its constructor takes a Scenario factory, invoked under the forge
  family's span; lay `scenario.checkout.files()` with `std::fs` under
  the scratch. Helpers write/read paths relative to `scenario.at`.
- `Disk::run` hosts `/usr/bin/git` with `World::host_roots`, constructing
  `temper_fake_git::Git::new(spawn, inherited, transport.clone())`.
  Reuse `fixtures::spawn_of`; replace its root with the caller's owned
  descriptor, keep `dir = scenario.at`, and give git a fresh `worker`
  startup root on that scratch. Drain stdin/stdout/stderr and reap.
- Register that same git adapter in `world.rs` for `run_forge`, using
  08's shared forge and bound-address configuration. Preserve its
  construction order, and account for git in `check.rs`'s forge-family
  bound. The small Disk proof and later landing use one hosted adapter.
Test shape: `tests/forge.rs`, 08's counting allocator/checks and the
fake git's `fixtures::scenario`, CONFIG and spawn shapes. The four
transcript cases become `hosted_clone_creates_only_the_destination`,
`hosted_git_stages_thirteen_files`,
`hosted_checkout_and_clean_replace_and_remove_real_files`,
`hosted_commit_tree_keeps_exact_parents`. Assert their existing exit,
stdout/stderr, filesystem and forge-object expectations; each under
0.5 s. Use the same counted-id conversion as the transcripts. Reference:
```rust
#[test]
fn hosted_checkout_and_clean_replace_and_remove_real_files() {
    let mut disk = git::Disk::new(temper_fake_git::fixtures::scenario);
    disk.write(b"conflict.txt", &vec![b'x'; 5000]);
    disk.write(b"untracked/deep/file", b"remove\n");
    disk.write(b".git/keep", b"metadata\n");
    let topic = invocation::object(&temper_fake_git::id::id(2)).expect("topic id");
    assert_eq!(disk.run(&Command::CheckOut { commit: topic }, b"").exit, 0);
    assert_eq!(disk.run(&Command::Clean, b"").exit, 0);
    assert_eq!(disk.read(b"conflict.txt"), b"topic\n");
    assert_eq!(disk.read(b".git/keep"), b"metadata\n");
    assert!(!disk.exists(b"untracked"), "nested untracked directories removed");
}
```
Proof: world, `cargo nextest run -p temper-real-loop-world --test
forge -E 'test(hosted_checkout_and_clean)'`, under 0.5 s. The kernel
filesystem is the measured boundary; the other three cases are tests.
Report their measured times, the file/forge verdict and owner-bound
sum in the landing report. A failure is raised before landing; 13.3
waits for the separately owned fix and a passing measurement.
Don't: edit `testing/temper-fake-git/src/program.rs` here, import the
transcripts' private Disk, or replace the fake by the installed git.
Size: at most 240 harness lines; no production change.
Removes: nothing.

### 13.2 The landing's fake-LLM scripts

Scope: domain/testing.md 2.4's scripted landing, in its own LLM increment.
Needs: nothing beyond the session's start.
Files (LLM-accounts hotspot; exact paths):
`tests/real-loop/src/landing.rs` (new), `tests/real-loop/src/lib.rs`
(the module/export). No `llm.rs`, `scripts.rs` or provider edits.
Shape: `landing::scripts(forge, repository, readme)` adapts
`forgejo::landing_scripts`, without Throwaway or its extra Hello
script: cues `@forge_chat`, `@coding Small fix`, and
`Review this change at the named head`. Grants name
`["forge", forge.to_string()]`; the resource is the fixture's repository
id. The edit appends `landing::EDIT`, the value of test support's EDIT,
to the given README. 08's factory calls this only after the forge binds.
Test shape: a unit test module in `src/landing.rs`; use 08's script
builders and small shipped profile. Local `call_arguments` decodes a
Script's tool-call JSON and `edit_new` reads the edit's new bytes.
They are test helpers in this module, not another script builder.
Reference (under 20 ms; the address is data, never listened on):
```rust
#[test]
fn the_landing_grants_name_the_forges_bound_authority() {
    let forge: SocketAddr = "127.0.0.1:41234".parse().expect("address");
    let scripts = scripts(forge, 2, b"# app\n");
    let arguments = call_arguments(&scripts[0].turns[0]);
    let grants = arguments["batch"][0]["authority"]["grants"].as_array().expect("grants");
    assert!(!grants.is_empty());
    for grant in grants {
        assert_eq!(grant["segments"], json!(["forge", "127.0.0.1:41234"]));
    }
    assert!(edit_new(&scripts[1]).ends_with(std::str::from_utf8(EDIT).expect("EDIT")));
}
```
Proof: unit, `cargo nextest run -p temper-real-loop-world --lib
-E 'test(the_landing_grants)'`, under 20 ms. No landing consumes the
scripts yet; 13.3 connects this factory to the agent and forge in a
world proof. The unit test makes this increment usable on its own.
Don't: import executable test support, change the fake LLM's crates,
build another peer or take a provider/account setup choice.
Size: at most 130 harness lines; no production change.
Removes: nothing.

### 13.3 A chat to a landed change

Scope: client/tools.md 11.3: adopt, chat, delegate, clone, edit, push, CI, review, land.
Needs: the measured fake git host (13.1); the landing scripts (13.2);
**the tools' adopt**.
Files (forge fixture wiring, behind the existing API):
`tests/real-loop/src/repository.rs` (new), `tests/real-loop/src/lib.rs`,
`tests/real-loop/tests/forge.rs`. No fake forge implementation change.
Shape:
- `repository::fixture() -> forge::Fixture` specializes 08's recipe:
  acme/app, repository id 2, worker write permission, one README.md of
  `repository::README`. Checks: context `check`, zero latency, silent
  and reruns zero, `cue: Some(Cue { path: ..., green: ... })` naming
  README.md and EDIT. Protect main on check, approvals zero;
  `hooked: true`. Build it under 08's forge-family owner.
- `repository::configure` takes 08's document and bound forge address;
  apply the landing's ceiling, policy, projections and engine overrides
  from `forgejo::configuration` through the same shipped builders and
  admission checks. Keep 08's 10/50/100 ms poll/poll_max/slow paces.
- `adopt --project 1 --owner acme --name app --landing main --key
  adopt-app`; then the forge chat, its reply, its tree naming delegate D,
  and `wait task D --until ended`. Completion means D ended, not that
  the service stopped. All these successful calls exit 0.
- Reuse 08's `Seen::merged_green` and `main_file`: drain the shared
  forge's `pop_observation` under its owner. At Merged and the default
  branch's Moved, require every check green as `referee::forge::green`
  does; main's README ends with EDIT. Do not build another referee.
- The second configuration, `repository::webhooks_only`, sets only
  `forge.client.poll_ms`, `poll_max_ms`, `slow_ms` past story patience,
  beside their shipped values and reason; admission must still pass.
  It retains hooks and shows the same landing without a poll fallback.
Test shape: `tests/forge.rs`, the same allocator, runner and profiles as
08 and 13.1; both stories under 1 s including settlement. Reference:
```rust
#[test]
fn a_chat_lands_a_change_through_fake_git_and_the_forges_webhooks() {
    let scripts = |forge| landing::scripts(forge, repository::ID, repository::README);
    let ran = run_forge(repository::fixture(), repository::configure, scripts, &[
        sign_in(D1),
        call(D1, argv!["adopt", "--project", "1", "--owner", "acme", "--name", "app", "--landing", "main", "--key", "adopt-app"]),
        call(D1, argv!["chat", "--project", "1", "@forge_chat Land a small change", "--key", "land"]),
        call(D1, argv!["wait", "task", task(1), "--until", "replied"]),
        call(D1, argv!["task", task(1), "--tree", "--json"]),
        call(D1, argv!["wait", "task", delegate(3), "--until", "ended"]),
    ]);
    assert!(ran.calls.iter().all(|c| c.exit == Exit::Code(0)), "{ran:?}");
    assert!(ran.forge.merged_green(), "every check passed at the merged head");
    assert!(ran.forge.main_file(b"README.md").ends_with(landing::EDIT));
    assert!(ran.killed.is_empty());
}
```
The second, `a_chat_lands_with_polling_past_its_patience`, uses
`webhooks_only` and the same calls/assertions through a helper in this
test file. Fresh directories have no kept requests; require stderr to
contain no kept result. Any future recovery assertion uses 6.1/6.2's
name/operation/outcome only, and never treats a client failure as exit 2.
Proof: world, `cargo nextest run -p temper-real-loop-world --test
forge -E 'test(a_chat_lands_with_polling_past_its_patience)'`, under 1 s.
This joins the real service, worker, smith agent and cli on the kernel;
the process tier's owned Forgejo proves the separate real-git boundary.
Don't: poll the forge from the test, change the forge host, fake, client
or component, edit landing scripts here, or stop a wait by stopping the engine.
Size: at most 180 harness lines; no production change.
Removes: nothing.

## Interface sketches

08 owns the runner, shared forge recipe, scripts factory and observations.
These modules add inputs and an adapter; no replacement for those types.

```rust
// tests/real-loop/src/git.rs: Scenario and Answer are temper_fake_git's.
pub struct Disk { /* scratch, scenario under its owner, shared host adapter */ }
impl Disk {
    pub fn new(make: fn() -> Scenario) -> Self;
    pub fn run(&mut self, command: &Command, input: &[u8]) -> Answer;
    pub fn write(&mut self, name: &[u8], bytes: &[u8]);
    pub fn read(&self, name: &[u8]) -> Vec<u8>;
    pub fn exists(&self, name: &[u8]) -> bool;
}
// tests/real-loop/src/landing.rs
pub const EDIT: &[u8] = b"Changed by temper.\n";
pub fn scripts(forge: SocketAddr, repository: u32, readme: &[u8]) -> Box<[Script]>;
// tests/real-loop/src/repository.rs: ID mirrors api::Setup.id.
pub const ID: u32 = 2;
pub const README: &[u8] = b"# app\n";
pub fn fixture() -> forge::Fixture;
pub fn configure(document: &mut serde_json::Value, forge: SocketAddr);
pub fn webhooks_only(document: &mut serde_json::Value, forge: SocketAddr);
```

## Questions to raise when reached

- **13.1:** fake git differs on real files: report the failing transcript
  case before landing; its fix belongs in a separate forge-family
  increment, with `testing/temper-fake-git/src/program.rs` named as a
  hotspot. 13.3 waits for the passing measurement after that fix.
- **13.3:** a needed delivery is missing, or the landing stalls without
  polling: retain the forge hook log, transport trace and last tools
  document; raise the gap before changing any fake or forge component.
- A combined owner bound fails or a story exceeds 1 s: report the
  bound, profile and times; make it cheaper or move it to `fuzzy_*`,
  retaining the assertions and the unoptimized build.

## Done when

Each in `~/src/rust/temper`, against fetched `origin/main`:

- `git grep -q -F 'temper_fake_git::Git::new' origin/main -- tests/real-loop`
- `git grep -q -F 'Command::Clean' origin/main -- tests/real-loop`
- `git grep -q -F 'Command::CommitTree' origin/main -- tests/real-loop`
- `git grep -q -F 'cue: Some(' origin/main -- tests/real-loop`
- `git grep -q -F 'hooked: true' origin/main -- tests/real-loop`
- `git grep -q -F '@coding Small fix' origin/main -- tests/real-loop`
- `git grep -q -F '"adopt"' origin/main -- tests/real-loop`
- `git grep -q -F 'merged_green()' origin/main -- tests/real-loop`
- `git grep -q -F 'poll_ms' origin/main -- tests/real-loop`
- `test -z "$(git grep -n -E '127\.0\.0\.1:[1-9]' origin/main -- tests/real-loop/tests)"`
