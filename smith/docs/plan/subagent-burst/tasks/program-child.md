+++
id = "program-child"
part = "lifetime"
kind = "feature"
needs = []
files = ["smith/crates/smith-agent-shell/src/program_child.rs", "smith/crates/smith-agent-shell/src/program_child/**", "smith/crates/smith-agent-shell/src/lib.rs", "smith/crates/smith-agent-shell/src/agent_shell.rs", "smith/crates/smith-agent-shell/src/config/programs.rs", "smith/src/bin/smith-agent.rs", "smith/crates/smith-protocol-program/src/lib.rs", "smith/crates/smith-protocol-program/src/backend.rs", "smith/crates/smith-protocol-program/src/backend/start.rs", "smith/crates/smith-protocol-program/src/backend/fixtures.rs", "smith/crates/smith-protocol-program/src/backend/diagnostics.rs", "smith/crates/smith-protocol-program/src/backend/launch_tests.rs", "smith/crates/smith-agent-service/src/service/program.rs", "smith/tests/end-to-end/program_lifetime.rs", "smith/tests/end-to-end/end_to_end.rs", "smith/tests/end-to-end/support/lifetime.rs", "smith/tests/end-to-end/support/mod.rs", "smith/tests/end-to-end/credentials.rs", "smith/docs/design/protocol/programs.status.md", "smith/docs/design/shell.status.md", "smith/docs/design/STATUS.md"]
effort = "high"
minutes = 180
+++

# A program dies with its agent

## Goal

An agent process that aborts left its `claude` child and that child's
relay running, still spending on their login: the program is spawned in
a group of its own with no parent-death signal, and an abort closes
nothing. The agent now starts every program process through its own
binary's `--program-child` mode, which execs the program through skein's
`exec_dying_with_parent` with Kill, so the kernel kills the program when
the agent dies; the relay, the program's child, ends at its input's end.

Acceptance:

- `smith-agent --program-child <parent> <program> [arguments…]` execs
  `<program>` as `notes/contracts.md`, section 3 says; a malformed argv,
  a changed parent or a failed exec prints `smith-agent: program child:
  <why>` and exits 1.
- On Linux the agent's every program process starts as
  `<relay> --program-child <pid> <claude> <arguments>…`: the status check
  and each print run (main's, a summary session's, each child's). With
  `parent: None` the launches are as today, byte for byte.
- **A unit table** in `smith-protocol-program` pins both launches: for
  `parent: None`, the program is `claude` and the arguments today's; for
  `parent: Some(4242)`, the program is the relay, the arguments begin
  `--program-child`, `4242`, the claude path, and go on with the direct
  launch's arguments, environment, directory, root and pipes unchanged.
- **A unit table** in `smith-agent-shell` for the mode's argv refusals:
  no parent, a parent that is not plain decimal (`-1`, `012`, `x`), no
  program; each gives the one line and exit 1, nothing exec'd.
- **The abort row**, in the shipped product's tests (package `smith`,
  binary `end_to_end`, a new module `program_lifetime.rs`): a `smith
  exec` run on the fake program (`signed_in`'s settings in
  `smith/tests/end-to-end/program_children.rs`) whose first turn is
  `Step::Hang`; once its print run and its relay are up, the test kills
  the agent (the product's child `smith-agent`) with SIGKILL, as an
  abort ends it, without a core file; the product exits 1, its failure
  naming the agent's end; and within ten seconds neither the fake's print
  process nor its relay remains. Seen failing on the base, where the fake
  outlives the agent; any process the row finds left is killed before
  the test ends, red or green, so a red run leaves nothing behind.
- The **Not yet** entries `2, 5.1: each program process started to die
  with its agent …` (`smith/docs/design/protocol/programs.status.md`)
  and `2.2: an agent's programs dying with it …`
  (`smith/docs/design/shell.status.md`) are gone, and the phrase `an
  agent's programs dying with it (`protocol/programs.md`, `shell.md`),`
  from `smith/docs/design/STATUS.md`'s summary.
- Clippy clean, fmt clean, the focused tests below green; `jig-channel-kit`
  checks.

## Inputs

- `smith/docs/plan/subagent-burst/notes/common.md` and `notes/contracts.md`,
  section 3, which binds the flag, the argv, the fields and the mode.
- `smith/docs/design/protocol/programs.md`, sections 2 (*The child is
  inside the run's containment*) and 5.1 (*Started to die with its
  agent*), as this cut wrote them; 5.3 (the relay), 5.6 (a program that
  cannot start), 5.10 (children start as 5.1 says).
- `smith/docs/design/shell.md`, 2.2 (*The end*).
- `skein/docs/design/shell.md`, section 3 (*A child-side exec*) and 13
  (*A crash closes nothing*).
- The precedent: temper's `internal forge-child`
  (`temper/docs/design/protocol/setup.md`, 7.1's *Stays Forgejo's parent*,
  and `temper/crates/temper-forge-setup-shell/src/process.rs`, `child`).

Departures from the implementer's text, and why:

- **Status files:** you delete the three pieces named above, since this
  task closes them; touch no other design text.
- **Cross-tree literal:** `jig/testing/jig-channel-kit/src/program_runs.rs`'s
  `ProgramBackend` literal gains `parent: None`, a required field's line
  (the producer's branch carries it).

## Owns and may change

Owns:

- `smith-agent-shell`: a new module `program_child` (`src/program_child.rs`,
  submodules beside it if wanted) with `pub fn main() -> ExitCode` and its
  argv parsing as a function the unit table calls; the `mod` and export
  lines in `src/lib.rs`; `src/agent_shell.rs`, where it fills
  `program.relay`, fills `program.parent` (`Some(std::process::id())` on
  `target_os = "linux"`, `None` elsewhere); `src/config/programs.rs`'s
  literal, `parent: None`.
- `smith/src/bin/smith-agent.rs`: dispatches `PROGRAM_CHILD_FLAG` as it
  dispatches the relay's flag, before the configuration path.
- `smith-protocol-program`: `PROGRAM_CHILD_FLAG` at the crate root;
  `Settings::parent` (`src/backend.rs`) and its `Debug`
  (`src/backend/diagnostics.rs`); one place in `src/backend/start.rs`
  that turns a direct launch into the trampolined one, used by the status
  check's `Launch` and by `launch_spec`; `fixtures.rs`, `parent: None`;
  the unit table in a new `src/backend/launch_tests.rs` (with its `mod`
  line) or beside `launch_wraps_the_unchanged_contract_within_its_bound`.
- `smith-agent-service/src/service/program.rs`: `ProgramBackend::parent`
  and `settings()` passing it to `backend::Settings`.
- `smith/tests/end-to-end/program_lifetime.rs`, its `mod` line in
  `end_to_end.rs`, and a helper in `support/lifetime.rs` (with its `mod`
  line in `support/mod.rs`) that runs the product, finds the processes
  and kills the agent.
- The status deletions above.

Reader and neighbour files (edits allowed as stated):

- Every other literal of `ProgramBackend` or `backend::Settings` the
  compiler names: `parent: None`, nothing else.
  `smith-agent-service/src/service/program/tests.rs`,
  `program/validity_tests.rs`,
  `smith/tests/agent-process/src/program_world.rs` (the literal at
  `config.program = Some(…)`: add `parent: None,` after
  `effective_user: 1000,`, and change no other line of that file: task
  `burst` edits it beside you), and
  `jig/testing/jig-channel-kit/src/program_runs.rs`.
- A test that pins a launch's program or arguments with a `Settings`
  whose `parent` you set: it keeps `None`, and its pins hold unchanged.

Pre-approved options:

- **Where the trampoline is built:** if the status check and the print
  run share no launch builder, add one function in `start.rs` both call;
  if `Launch` and `kernel::Spawn` differ in shape, two small adapters
  over one argument list. Never a second copy of the flag's text.
- **`std::process::id` or `exec_dying_with_parent` refused by a lint** in
  `smith-agent-shell`: a scoped `#[expect(clippy::<lint>, reason =
  "protocol/programs.md, 5.1: the program dies with its agent")]` on the
  one item.
- **Finding the processes:** prefer the real world's own observers
  (`smith/tests/real`, `skein/testing/skein-world/src/tree.rs`, whose
  keeper kills leftovers and counts them) if one reports a leftover
  descendant; otherwise walk `/proc/<pid>/stat` (parent and start time)
  and `cmdline` from the product's pid: the agent is its child
  `smith-agent`, the print run the agent's child with `-p` in its argv,
  the relay the print run's child with `--mcp-relay`. Read no `environ`.
  A pid is the same process only while its start time matches.
- **Killing:** the `smith` package forbids `unsafe` (`[lints.rust]`), so
  no `libc::kill`: a safe signaller the real world's crates already offer
  (`skein-world`, `smith-real-world`) if one signals a pid; otherwise
  `/bin/kill -KILL <pid>` through `std::process::Command`, its exit
  checked.
- **The relay outlives the fake** in the abort row: read why (a pipe end
  held elsewhere, the relay's connection to a dead agent not ending). If
  a relay change in `smith-agent-shell/src/relay.rs` or
  `smith-protocol-program/src/relay.rs` ends it at its input's end as
  programs.md 5.1 says, make it and name it; if only a new skein-shell
  function would (a parent-death signal set in a running process), that
  is an interface change in skein's shell: stop with a `Question:`, your
  work pushed, the row's relay check left failing locally and
  described.
- **The row costs more than two CPU seconds:** say so with the
  measurement; never move it to another tier.

## Steps

1. Read the inputs; `smith-protocol-program/src/backend/start.rs` (the
   status check around `STATUS`, `launch_spec`), `backend.rs`'s
   `Settings`, `smith-agent-service/src/service/program.rs`
   (`ProgramBackend`, `settings`), `smith/src/bin/smith-agent.rs`,
   `smith-agent-shell/src/relay.rs`'s `main`, and the end-to-end
   program tests (`program_children.rs`, `support/exec.rs`'s
   `run_fixture`).
2. Write the end-to-end abort row first and see it fail on the base for
   the fake left running (`heavy cargo nextest run -p smith --test
   end_to_end -E 'test(/program_lifetime::/)'`); write the two unit
   tables and see the launch table fail for the missing trampoline.
3. Build the flag, the fields, the launch, the mode and the shell's
   `parent`; run the three narrow tests green (check the counts). For
   each new check, break its rule once by hand (the launch left direct;
   a refusal accepted; the mode exec'ing without the parent-death call)
   and see it fail.
4. Fix the compiler-named literals; delete the status pieces.
5. Once, at the end: `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`;
   `heavy cargo clippy -p smith -p smith-agent-shell
   -p smith-protocol-program -p smith-agent-service
   -p smith-agent-process-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p smith-protocol-program -p smith-agent-shell
   -p smith-agent-service -p smith-agent-process-world`;
   `heavy cargo nextest run -p smith --test end_to_end`;
   `heavy cargo check -p jig-channel-kit --tests`;
   `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit in steps that each say why (the mode; the launch through it;
   the row), then push.

## Report

As the implementer's text says, and: the processes the abort row found
and how it found them, the CPU and wall time it took, whether the relay
ended at its input's end, the rules broken by hand and the failures
seen, and your implementation and check minutes apart.

## Answers

2026-11-08. *May this task repair the inherited `Secret` compile errors
in `smith/tests/end-to-end/credentials.rs`?* Yes, narrowly: the
`end_to_end` binary holds this task's abort row and does not compile
without it (main carries the same break since `6c401e7d17` made
`SavedToken`'s tokens `Secret`). Change only the conversions the
compiler names (lines 47, 51, 52, 81, 82): build each `Secret` with
`Secret::new(..)` from the same bytes, and replace `clone_from(&held)`
with an assignment of a `Secret` of `held`'s bytes. The fixture's bytes,
the records it writes and every assertion stay as they are; no other
test of the file changes. Commit it alone, saying it repairs the
fixture for the `Secret` record fields, before the row's commits; then
resume from `480d12d44e`.

