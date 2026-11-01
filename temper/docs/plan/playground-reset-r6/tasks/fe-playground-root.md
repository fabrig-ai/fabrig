+++
id = "fe-playground-root"
part = "pg"
kind = "feature"
needs = ["fe-kept-root", "pd-vocabulary"]
files = ["temper/crates/temper-engine-domain/**"]
effort = "high"
minutes = 100
+++

## Goal

The root's half of the playground's start (draft
`temper/docs/design/draft/playground-reset.md`, 6.1; `notes/contracts.md` T6, J8;
`notes/fe-surface.md` section 5): on the connector's `PlaygroundStart`, the root makes the
playground's project and its scripted account as the deployment's acts, waits for the fake
forge to connect, adopts `playground/widget` into the project, and answers the start. The
reset's acts are `fe-kept-root`'s; the connector's row and record are
`fe-playground-forge`'s, built in parallel: this task reads the row through
`forge::Domain::playground()` and tells the connector through fe's events.

Acceptance:
- `forge::Request::PlaygroundStart { request }` begins one start at a time (later ones wait
  for the connector's shared answer, as the demo's do), with the admitted start's context
  (sign-in, carried key) the root keeps today for `StartPlayground`
  (`route/applications.rs`, `application_demo`/`demo_contexts`), and the deadline
  `DemoConfig.deadline` from now. Its phases, each skipped when already done:
  - Project, when the row's `project` is `None`: `people::Act::MakeProject { maker, name:
    demo.project_name, policy: demo.project_policy }`, `maker` the starting person (the
    sign-in's person, a deployment owner since the start was admitted at the deployment's
    scope); on `ProjectMade { project, .. }`, `forge::Event::PlaygroundProject { project }`.
  - Account, when the row's `account` is `None`: `people::Act::AddKey { provider:
    demo.provider, join: None, digest: carried.digest, uses:
    Some(Box::new([people::Use::Project(project)])) }` with the carried key beside it; on
    the kept key's answer, `forge::Event::PlaygroundAccount { account }`. No carried key:
    the start fails `StartFailure::Account`. An unused carried value is answered
    `CarriedUnused`.
  - Connecting: until the row's forge reads `Connected`, or the deadline passes
    (`StartFailure::NotConnected`); a group failure of that forge fails the start
    `StartFailure::Group { why }` (cut to 256 bytes), as the demo's `connection` does.
  - Adoption, unless the project already holds `demo.adoption`'s owner/name on the row's
    forge: the adoption through the application route as the demo adopts today
    (`applications::start` with `Ask::Adopt`, the starter's sign-in, the project's scope),
    with `adoption.forge` the row's forge and `adoption.project` the project; adopted, the
    answer is `Running`; refused, `StartFailure::Adoption`.
  - The answer: `forge::Event::PlaygroundAnswered { answer }`.
- The two acts' keys: the first 16 bytes of SHA-256 over `b"temper playground start\0"`,
  the playground forge's two big-endian bytes and the purpose (`b"project"`,
  `b"account"`), so a start resent for the same playground meets jig's retained answer and
  makes nothing twice (see `notes/fe-progress.md`, contract request 2).
- Act answers: `Again(_)` waits and resends at the next `Resume`-driven pass (or fails the
  start `Group { why: "busy" }` past the deadline); `Refused(NotOffered)` fails the start
  with a `DemoNote` line naming the grant word (`make_project`, `add_key`); other refusals
  fail it (`Account` for the key, `Group { why: "project" }` for the project).
- The demo's own start code (`route/demo.rs`'s `start`, `begin`, `progress`) is no longer
  reached once the connector sends `PlaygroundStart`; it stays for `fe-demo-gone`.
- Unit tests below pass; every test of the crate passes.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T4, T6, J8;
  `notes/fe-surface.md` sections 1 (`DemoConfig`) and 5.
- jig's `jig/docs/design/domain/people.md`, 5.3 (the deployment's acts).
- `temper/crates/temper-engine-domain/src/route/demo.rs` (today's start, the model) and
  `src/route/kept.rs` (`fe-kept-root`'s act replies, to extend).

## Owns and may change

- Owns: `temper/crates/temper-engine-domain/`.
- May change: nothing else.
- Pre-approved options:
  - Module: default, a new `src/route/playground.rs`; the act replies dispatch through
    `fe-kept-root`'s `Held::Acted` dispatch with a third map.
  - The starting person: default, the person the core's people child holds for the
    start's sign-in; if the root cannot read it there, keep the person the core names in
    its `Application` admission (`core::Ask::Application { person, .. }`) beside the
    context, and say which.
  - The account's number from the `AddKey` answer: take it from the outcome jig gives for
    a kept key (`Outcome::AccountKeyKept { .. }`, read its fields); if the outcome carries
    no account number, read the newest deployment account of `demo.provider` from the
    core's accounts child after the answer, and say so.

## Steps

1. `src/route/playground.rs`: `start`, `progress` (called where the demo's `progress` is:
   resumes, connection changes), `acted`, `adopted`, `finish`, `start_key`.
2. `forge_route.rs`'s `PlaygroundStart` arm (a no-op since `fe-surface`) calls `start`;
   the connection and adoption hooks the demo uses also call this module.
3. Tests in a new `src/tests/playground.rs`, on the crate's domain fixture, injecting the
   connector's `PlaygroundStart` and the row it would hold:
   `a_start_makes_the_project_then_the_account_serving_it_alone`;
   `a_start_with_project_and_account_kept_only_adopts`;
   `the_start_waits_for_the_fake_forge_until_its_deadline`;
   `a_missing_key_or_grant_fails_the_start_and_names_why`;
   `the_start_acts_carry_keys_derived_from_the_playground_forge`.
4. Run, one at a time: `heavy cargo nextest run -p temper-engine-domain --lib`; `heavy
   cargo check --workspace --all-targets`; `cargo fmt --all`; `heavy cargo clippy -p
   temper-engine-domain --all-targets -- -D warnings`; `python3 -I -B
   scripts/old_forms.py` from `temper/`.
5. Commit, push your branch.

## Report

- The jig outcome fields you read for the project and the account, and how the starting
  person is found.
