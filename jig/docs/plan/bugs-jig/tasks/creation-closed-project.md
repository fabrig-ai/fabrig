+++
id = "creation-closed-project"
part = "closed-project"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/routing/party/holdings.rs", "jig/crates/jig-domain-core/src/tests/holdings.rs"]
effort = "medium"
minutes = 40
+++

# A creation in flight is refused once its project is closed

## Goal

A party's `Create` asks every connector for its holdings and finishes when they answer.
`created_holdings` then calls `finish_creation` with no second look, so a project deleted between
the ask and the last answer still gets a period and a pool opened, and its task made. Closed at once
means closed for this too (`jig/docs/design/domain/people.md`, 5.2). This is the one path that gets
past core's gate for a closed project.

Depending on when the answers land, the old code makes a task in a deleted project, leaves ledger
rows for it that nothing closes, or aborts on `close_project`'s assert
(`jig/crates/jig-domain-tasks/src/close_project.rs`).

Acceptance:

- A creation whose project is no longer open when its last connector answers is refused
  `people::Refusal::Unknown`: nothing is made, no period or pool opens, and the propose kind takes
  the same path.
- The new test fails on the old code, and passes. Every test of `jig-domain-core` stays green.

## Inputs

- `jig/docs/plan/bugs-jig/notes/common.md`.
- `jig/docs/design/domain/people.md`, 5.2 ("Closed at once").
- The code: `jig/crates/jig-domain-core/src/routing/party/holdings.rs` (`created_holdings`,
  `finish_creation`); the guard's form in `deleted`,
  `jig/crates/jig-domain-core/src/routing/settings/restore.rs`; `open_project` in
  `src/admission/deletion.rs`; `pool_refused` in `src/routing/refusals.rs`.
- The tests: `refused_person`, `party_core`, `holdings_request`, `person_refusal` in
  `src/tests/holdings.rs`; `begin` in `src/tests/admission/deletion.rs`.

## Owns and may change

- `jig/crates/jig-domain-core/src/routing/party/holdings.rs`: `created_holdings` only.
- `jig/crates/jig-domain-core/src/tests/holdings.rs`: one new test, and a small helper if it needs
  one.
- `jig-domain-tasks` gains no check of its own: it keeps no set of closed projects, and the rule is
  the core's decision (`people.md`, 5.2).
- Options: if a test elsewhere in `jig-domain-core` relied on this gap, that is a surprise: stop and
  ask.

## Steps

1. The test, `a_project_deleted_while_its_connectors_answer_refuses_the_batch`:
   - `party_core()`, then `holdings_request(ask(&mut core, &env, goal()))`;
   - `crate::admission::deletion::begin(&mut core, &env, &mut work, &mut out, Token::new(90), 1, None)`
     for the goal's project (`tests/admission/deletion.rs`, `begin`, shows the call). If the fixture
     lists no project 1, add it as that file's `core` does;
   - `Event::Holdings` from both connectors, each `holdings: Some(Box::new([]))`, `refused: None`;
   - assert the second answer's outcome reaches the person as `Refused(Refusal::Unknown)`, modelled
     on `person_refusal`, `core.tasks.task(OWNER).is_none()`, and no pool funding for the project
     (`core.tasks.funding(tasks::Funder::Pool { .. })`, as `holdings.rs` builds that funder).
   See it fail, then, in `created_holdings`, right after `take_creation` and before the `refused`
   checks:

   ```rust
   let project = creating.members.first().expect("nonempty batch").project;
   if project != people::DEPLOYMENT && !core.open_project(project) {
       pool_refused(&mut routed.work, request, people::Refusal::Unknown);
       return;
   }
   ```

2. `heavy cargo nextest run -p jig-domain-core --lib -E 'test(=tests::holdings::a_project_deleted_while_its_connectors_answer_refuses_the_batch)'`,
   then `heavy cargo nextest run -p jig-domain-core -p jig-core-world`.
3. `cargo fmt`; old forms; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`;
   commit and push.

## Report

As the implementer's text asks, with how the test failed on the old code.
