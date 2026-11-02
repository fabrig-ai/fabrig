+++
id = "client-jobs"
part = "client-jobs"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-client-protocol-wire/**",
  "jig/crates/jig-client-domain/**",
  "jig/crates/jig-client-protocol-link/**",
  "jig/crates/jig-client-view/**",
  "jig/testing/jig-client-scripted-engine/**",
  "jig/tests/client-wire/**",
]
effort = "medium"
minutes = 120
+++

# The client knows a job task: `Job { kind }`, its contract, a delegation's job kinds

## Goal

jig's client contract names a job task (`jig/docs/design/client/domain/contract.md, 5.7`): its
executor `Job { kind }`, its executor kind, its chip, its result contract `Job { bytes }` and a
delegation's job kinds, on the client wire (v19 in place) and in the client's domain, view and
every client that matches them; `Create` naming a job is refused as not offered. The engine's
side, which maps the core's job tasks into these values, is `tasks-jobs-surface`'s: no job task
exists yet, so nothing here renders one from a live engine.

Acceptance:

- `client-v19*.schema` and `jig-client-domain`'s contract hold `notes/contracts.md`, section 7,
  exactly; `v19.rs` and the goldens regenerated; the client wire's drift test passes without
  `SKEIN_CODEGEN_WRITE`.
- `jig-client-protocol-link` translates each new arm both ways, exhaustively.
- `Create` naming `Executor::Job` is refused `Refusal::NotOffered` by the client's admission
  (`jig-client-domain/src/contract/admission.rs`, `creation`), and by the engine's Create
  translation (below).
- jig's view shows a job's executor and chip as `job <kind>` and its contract as a job's,
  following the view's words idiom; temper's clients show `job <kind>` too.
- Tests below pass; the packages the compiler names build and their tests pass.

## Inputs

- `temper/docs/plan/reference-m7a/notes/common.md` and `notes/contracts.md`, section 7.
- `jig/docs/design/client/domain/contract.md, 5.7` (job tasks), `5.3` (an application's part:
  a document carries no job bytes) and `3.2` (`Create`).
- The precedent for an appended arm across the client: commit `e1cb0b87c8` (`nothing_awaited`):
  `git show --stat e1cb0b87c8` lists the client wire, link, domain, view, scripted engine and
  temper files such an arm reaches.

## Owns and may change

Owns: `jig/crates/jig-client-protocol-wire/**`, `jig/crates/jig-client-domain/**`,
`jig/crates/jig-client-protocol-link/**`, `jig/crates/jig-client-view/**`,
`jig/testing/jig-client-scripted-engine/**`, `jig/tests/client-wire/**`.

May change, each for the arms the new variants force, as written:

- `jig/crates/jig-protocol-engine-clients/src/translate/operations.rs`: Create's executor from
  the wire, `w::Executor::Job(_)`, is refused as an executor not offered: the function's error
  path the engine gives `Create` for an executor the deployment does not offer (find it from
  `jig-domain-core/src/routing/party/create.rs`'s `executor()` and the `NotOffered` refusal);
  if the translation cannot refuse there, translate nothing and refuse in `create.rs`'s
  `executor()`, and say so.
- `jig/crates/jig-protocol-engine-clients/src/translate/settings/**`: a template's contract
  `w::TaskContractDocument::Job(_)` is `Err(bad())`, as an untranslatable value (no template
  makes a job); a delegation's `w::AuthorityExecutor::Job(_)` from a party is `Err(bad())` until
  the core's authority has job kinds (`tasks-jobs-surface` replaces it). The engine's other
  translation files (core to wire) need no arm here: the core has no job yet.
- `jig/crates/jig-protocol-engine-clients/src/translate/queries.rs`: a query filtering on
  `w::ExecutorKind::Job` is refused as an untranslatable query, the same way (replaced by
  `tasks-jobs-surface`).
- temper's clients, arms only, each showing `job <kind>` where they show an executor or a
  chip, and a job's contract as `job` where they show a contract:
  `temper/crates/temper-client-view/**`, `temper/crates/temper-client-domain/**`,
  `temper/crates/temper-client-tools/**` (owner chips, endings, task fields, the settings
  adapter, `root.rs`, `change.rs`), `temper/crates/temper-settings-form/**`,
  `tools/gallery/src/worlds.rs`. JSON goldens in `temper-client-tools/src/tests/golden/` do not
  change: no fixture holds a job.

Pre-approved options:

- **Words:** if the view's words are phrasebook entries (`jig-client-view/src/words/`), add one
  entry for a job's executor with the kind as its number, and a phrasebook golden regenerated
  by its owning test; if a temper help or golden would change, stop and ask (help is a lead's).
- **A wrapper record's name** the schema's idiom forces differs from the contracts' (for
  instance `ExecutorJob`'s fields): take the idiom's, and say so.
- If a client test enumerates every executor arm (a fuzz or round-trip table), add `job` there.

## Steps

1. Schema arms, appended; regenerate (`notes/common.md`); drift again.
2. Domain variants (`contract/part.rs`, `queries.rs`, `common.rs`, `proposed_task.rs`,
   `authority.rs`); `admission.rs`'s `creation` refuses a job `NotOffered`.
3. Link both ways (`translate/facts_part1.rs`, `facts_part0.rs`, `answers.rs`,
   `operations.rs`, `queries.rs`, `documents_part1.rs`, `settings.rs`), exhaustive.
4. View and temper arms; the scripted engine's mirrors.
5. Tests, each citing `jig/docs/design/client/domain/contract.md, 5.7`:
   - in `jig-client-domain`: `create_naming_a_job_is_refused_as_not_offered`;
   - in `jig-client-protocol-link`: each new arm round-trips wire to domain to wire (an
     executor `Job { kind: 7 }`, a chip, a contract `Job { bytes: 4096 }`, a delegation kind
     `Job(7)`, an executor-kind filter);
   - in `jig-client-view`: a task document whose executor is `Job { kind: 7 }` renders
     `job 7` (a fixture beside its agent's);
   - in `jig-protocol-engine-clients` (its unit tests): a Create naming a job is refused as an
     executor not offered.
6. `cargo fmt`; `heavy cargo clippy` naming every package you changed `--all-targets -- -D
   warnings`; `heavy cargo nextest run -p jig-client-protocol-wire -p jig-client-wire-world -p
   jig-client-domain -p jig-client-protocol-link -p jig-client-view -p
   jig-protocol-engine-clients`; then the tests of every temper package you changed.
7. Commits: wire and domain, then link and view, then the forced arms; push your branch.

## Report

As the implementer's text says, and also: the words the view and temper's clients show; the
three engine-side refusals left for `tasks-jobs-surface` to replace (file and function each).
