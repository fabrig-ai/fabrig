+++
id = "workspace"
part = "declare"
kind = "feature"
needs = ["declare"]
files = ["temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/status_route.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/tests/engine/domain/tests/suite/forge_routes/status.rs", "temper/docs/design/domain/agent.status.md", "temper/docs/design/domain/root.status.md"]
effort = "medium"
minutes = 60
+++

# ui-ask-m1: an asking charter's runs have no workspace

## Goal

A run whose charter is an asking one (`notes/contracts.md`, section 5's test,
`temper_engine_domain::asking`) is given no repository, whatever its project's home and
context repositories, as the status agent's and a trial's runs are: no checkout, and
smith starts it with no workspace. Every other run's repositories are unchanged.

Acceptance:

- `run_workspace` (`route/forge_route.rs`) answers `status_route::workspace(context.task)`,
  the empty workspace, for an agent whose charter, read as the routine above it reads
  `charter.policy.modify` (`domain.core.charter(charter)`), is an asking one, before it
  resolves the home. A charter it cannot read keeps today's path.
- One test, in the engine's domain world beside its siblings: in
  `temper/tests/engine/domain/tests/suite/forge_routes.rs`, next to
  `a_chat_made_before_its_home_was_adopted_reads_the_home_and_says_why`, or as a row of
  `forge_routes/status.rs`' `the_run_holds_no_tools_no_grants_and_no_workspace` if
  that test's world can take a second charter: a project with a home and a context
  repository, a chat under the world's own charter with tools 3,072 (asking and
  settings), and its assignment's repositories empty; a row beside it, the same project
  and a charter with `RunPolicy::EVERY_FAMILY`, keeps its home. Use the world's own
  settings for the charter (stock charter 11 is the task `stock`'s).
- Status (a departure from the implementer's text, which closes `docs/design/`: the
  status must say what is built): delete from `temper/docs/design/domain/agent.status.md`
  the **Not yet** "4.2: a run whose charter names the asking family is given the
  project's home repository ..." bullet, and from `temper/docs/design/domain/root.status.md`
  the **Not yet** "7.1: a run whose charter names the asking family is given the
  project's home and context repositories ..." bullet, and nothing else.

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`; `notes/contracts.md`, section 5 (the
  asking test).
- `temper/docs/design/domain/root.md`, 7.1 (*The repositories*);
  `temper/docs/design/domain/agent.md`, 4.2; `temper/docs/design/domain/forge.md`, 5.7
  (the status agent's run, which selects no repository).

## Owns and may change

- Owns the files in the header.
- A change of meaning for asking charters alone, of which none exists before the task
  `stock`; run `heavy cargo nextest run -p temper-engine-domain-world -E
  'test(/forge_routes/)'` and the status tests, so that every other run's repositories
  are seen unchanged.
- Pre-approved: if `status_route::workspace` is private to its module, make it
  `pub(super)` or move the empty workspace's builder beside `run_workspace`, and say so.

## Steps

1. The test and its row; run them; see the asking row fail (the home is given).
2. The early return in `run_workspace`.
3. `heavy cargo nextest run -p temper-engine-domain --lib`, the world's tests above;
   `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`.
4. Commit and push.

## Report

As the implementer's text says.
