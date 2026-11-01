+++
id = "bx-doc"
part = "fs"
kind = "migration"
needs = ["bx-compose", "sv-startup-machine"]
files = ["temper/crates/temper-fake/src/demo.rs", "temper/crates/temper-fake/src/steps.rs", "temper/crates/temper-fake/src/tests/demo.rs", "temper/crates/temper-fake/src/tests/steps.rs", "temper/crates/temper-forge-setup/src/check.rs", "temper/crates/temper-forge-setup/src/problem.rs", "temper/crates/temper-forge-setup/src/tests/members.rs", "temper/crates/temper/src/setup.rs", "temper/crates/temper/src/box_command.rs", "temper/crates/temper/src/box_command/**", "temper/crates/temper/tests/suite/startup.rs", "temper/crates/temper/tests/suite/box_listeners.rs"]
effort = "high"
minutes = 100
+++

## Goal

The box and the fakes read T4's document: `application.machine.owned` and the new
`application.demo` (its `forge` an entry body, its `project` `{name, policy}`), with no
`application.setup`. The box holds the machine's ports from its start, before any group
names them, so a group loaded later is given its listeners. The old members still read as
today (they are refused in a later task).

Acceptance:
- `temper_fake::demo::read` takes both shapes of `application.demo`: today's (`forge` a
  number naming a fake setup entry, `project` a number) and T4's (`forge` a fake entry body
  read by `temper_forge_setup::setup::Entry::body`, `project` an object `{"name", "policy"}`,
  name 1 to 64 bytes, policy a u32 from 1). `Demo` keeps `llm()` and `provider()`;
  `forge()` and `project()` become `Option` (today's shape only); new `fake() ->
  &temper_forge_setup::setup::Fake` (either shape: the setup entry's fake, or the body's).
- `temper_fake::steps` builds the demo's dormant group only for today's shape (a demo whose
  forge is a number); for T4's shape no demo group is built at construction (the
  playground's group is loaded when serve asks).
- `temper_forge_setup::check::checked` keeps its signature (`at: u32`); a new
  `temper_forge_setup::check::checked_at(.., at: Option<u32>)` beside it, `None` (no
  `application.setup` member) reading as an empty list, and `checked` calling it with
  `Some(at)`. The binary calls `checked_at`. No temper-service file changes (the lead's
  answer to sv's request 6; `sv-startup-refuse` moves serve's call later).
- The binary's `setup::check_with` reads `application.setup` only when present, and
  `application.machine.owned` (null or an entry body) with `Entry::body` (forge 0: only its
  mode is used). `Deployment` gains `ports: Box<[SocketAddr]>`: the machine's ports, the
  owned Forgejo's listener (when `machine.owned` is not null), the demo's fake forge
  listener and the demo LLM's listener (when `demo` is not null), each once.
- `temper box` holds every port of `Deployment.ports` from its start, whether or not a step
  of the construction names it (on macOS the owned Forgejo's port is not held, as today:
  `forgejo_listens` false); an inherited listener is accepted when a step or a machine port
  names it. The box's loader takes its demo LLM listener from `Demo::llm()` in both shapes.
- On a document of T4's shape (no forges, identities or setup; `application.machine`; the
  new demo), `temper box` reaches serve's link with nothing running but serve; the old
  documents behave as today.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.1 (the box's listeners), 8 and 8.4.
- Draft `temper/docs/design/draft/playground-reset.md`, section 3 ("The document keeps the
  machine's").
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T3 and T4.
- What `sv-startup-machine` landed: its report, and the members it parses in
  `temper/crates/temper-service/src/startup/application.rs` (read them; do not parse what
  `temper_service::startup::Named` already gives, if it gives the machine's entries:
  then take them from `Named` and say so).
- bx-steps' readers (`Entry::body`) and bx-compose's `box_command/groups.rs`.

## Owns and may change

- Owns: the files in `files`.
- May change, call sites only: the binary's callers of `check::checked`, moved to
  `checked_at` (`temper/crates/temper/tests/support/owned.rs`, the binary's `src/`); never a
  temper-service or one-process file; callers of `Demo::forge()`/
  `project()` in `temper/crates/temper/src/init/tests.rs` and `src/fake_backends.rs`
  (`.expect` on today's shape, or match). Nothing else.
- Options, pre-approved:
  - If `Named` already holds the parsed machine and demo bodies after `sv-startup-machine`,
    take the ports from it instead of parsing again (default: parse with `Entry::body` in
    `setup.rs`, the cheaper path to keep apart from sv's code).
  - The demo's checks against the setup list (`unique_listen`) apply to T4's shape against
    `Deployment.ports` instead: the LLM's listener differs from the fake forge's, the owned
    one's and the serve listeners.

## Steps

1. `temper-fake/src/demo.rs`: the two shapes as above; refusals rooted at
   `application.demo` as today (`forge`: `names no fake setup entry` for a number,
   the entry body's own problems for an object, `must be a fake forge` for a body of another
   mode). Tests in `src/tests/demo.rs`: `a_demo_of_t4s_shape_reads_its_fake_and_project`,
   `a_demo_forge_body_of_another_mode_is_refused`, `todays_demo_reads_as_before` (the
   existing tests stay).
2. `temper-fake/src/steps.rs`: the demo group for today's shape only. Test in
   `src/tests/steps.rs`: `a_demo_of_t4s_shape_builds_no_group_at_the_start`.
3. `temper-forge-setup/src/check.rs`: `at: Option<u32>`. Test in `src/tests/members.rs`:
   `no_setup_member_checks_as_an_empty_list`.
4. `temper/src/setup.rs` and `box_command.rs`: `Deployment.ports`; `open_listeners` takes the
   machine's ports beside the steps' (each address held once).
5. Suite tests (`tests/suite/startup.rs`), on a fresh `temper init` whose document the test
   rewrites into T4's shape (forges, identities and setup removed; `application.machine`
   `{"owned": <init's forge 3 entry without forge, directory and start>, "github": {"api":
   null, "transport": null, "git": null, "web": null}}`; `application.demo.forge` init's
   fake entry body; `application.demo.project` `{"name":"Playground","policy":1}`; any
   further member `sv-startup-machine` requires, as its report says):
   - `a_box_on_a_t4_document_holds_the_machines_ports_and_reaches_serves_link`: the link is
     printed; while the box runs, a TCP connect to the owned and the fake forge ports
     succeeds (the box holds them); exit 0, nothing left.
   - `a_t4_document_runs_a_loaded_playground_group` (as bx-compose's
     `a_box_runs_a_recorded_group_from_its_file_before_serve`, on this document: the group's
     `fake forge 7` adopts the held fake forge listener).
   `tests/suite/box_listeners.rs`: `an_inherited_machine_port_is_accepted_with_no_step_naming_it`.
   Run each: `heavy cargo nextest run -p temper --test suite -E 'test(=startup::NAME)'`.
6. `heavy cargo nextest run -p temper-fake`, then `-p temper-forge-setup`, then `-p temper`
   (default profile), then `heavy cargo check -p temper-service --tests` (unchanged caller); `cargo fmt`;
   `heavy cargo clippy -p temper-fake -p temper-forge-setup -p temper --all-targets`. Old forms
   never raised. Commit and push.

## Report

The T4 members the binary reads and where (file, function), and every reader left of
today's shapes (`Demo::forge()`, `checked.setup`), for the task that drops them.

## Answers

2026-11-01. Q: may bx-doc add a structured read of `Problems` in
`temper-forge-setup/src/problem.rs`, so `demo::read` keeps `Entry::body`'s own paths and
reasons? A: yes (option A); `problem.rs` is added to `files` for this alone. Add
`Problems::first(&self) -> Option<(&Path, What)>` (the first kept entry, its `What` copied),
read-only, documented, with a test in `src/tests/members.rs`; no other change there. In
`temper-fake/src/demo.rs` convert it to `Refusal`: `What::Reason(words)` gives
`Refusal { path, words }`; `Own(_)` and `Against(_)` (which `Entry::body` does not produce)
and `None` give `invalid form` at the entry's own path (`application.demo.forge`). Never
parse `lines()` back. Everything else in the task stands as written, including the
`Named.machine` findings of run 1: take the owned record from `Named` where it holds it,
and give the T4 fixtures `git_program: null`.
