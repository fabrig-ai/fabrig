+++
id = "bounds-wait"
part = "bounds"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-fleet/src/history.rs",
  "jig/crates/jig-domain-fleet/src/channel.rs",
  "jig/crates/jig-domain-fleet/src/domain.rs",
  "jig/crates/jig-domain-fleet/src/needs.rs",
  "jig/crates/jig-domain-fleet/src/lib.rs",
  "jig/crates/jig-domain-fleet/src/tests/needs.rs",
  "jig/crates/jig-domain-core/src/documents/task_wait.rs",
  "jig/crates/jig-domain-core/src/tests/runs/run_choice.rs",
  "jig/crates/jig-client-domain/src/contract/llm.rs",
  "jig/crates/jig-client-domain/src/contract/deployment.rs",
  "jig/crates/jig-client-protocol-wire/schema/client-v19.schema",
  "jig/crates/jig-client-protocol-wire/src/generated/v19.rs",
  "jig/crates/jig-client-protocol-wire/golden/**",
  "jig/crates/jig-client-protocol-wire/src/tests/**",
  "jig/crates/jig-protocol-engine-clients/src/translate/task_wait.rs",
  "jig/crates/jig-protocol-engine-clients/src/translate/deployment.rs",
  "jig/crates/jig-protocol-engine-clients/src/tests/llm.rs",
  "jig/crates/jig-client-protocol-link/src/translate/task_wait.rs",
  "jig/crates/jig-client-protocol-link/src/translate/documents/capacity.rs",
  "jig/testing/jig-client-scripted-engine/src/codec/values/llm.rs",
  "jig/testing/jig-client-scripted-engine/src/codec/values/deployment.rs",
  "jig/crates/jig-client-view/src/words/words/status.rs",
  "jig/crates/jig-client-view/src/tree/fragments/deployment/hello.rs",
  "jig/docs/design/domain/engine.status.md",
  "jig/docs/design/client/domain/contract.status.md",
  "jig/docs/design/protocol/client.status.md",
]
effort = "high"
minutes = 150
+++

# A run its hosts are all too small for says which bound keeps it out

## Goal

A goal whose charter grew past its box's worker (100,000 turns against 800) waits on "no host has room" for ever: the worker stays admitted but can never take it, so no hello is refused and the standing refusal stays suppressed. Make the engine say so: the fleet answers which bound keeps every admitted host out and by how much, the standing refusal keeps its figures, and both reach the client's task document through the v19 wire. Also carry the charter the wire drops on a turns, time or waiting shortfall. temper's words are the next task's (`wait-words`).

Departures from the implementer's text, each because the design is already written: you remove three status entries (acceptance 6), and you edit temper lines only so that they compile (*Owns and may change*).

Acceptance:

1. `jig_domain_fleet::Domain::bounds_refusal` answers as `notes/contracts.md`, *The fleet*, says: one new table test in `jig/crates/jig-domain-fleet/src/tests/needs.rs` covers its multi-host rules, and every row of the existing figures table there (the one with rows `(Bounds, AgentShortfall, required, offered)`, beside `a_bound_set_after_the_start_refuses_with_its_figures`) also checks that a worker admitted before those needs is named by `bounds_refusal` with the same `why` and `figures`.
2. A standing refusal's record keeps its figures, and the core's `TaskRunWait::Refused(TaskWaitRefused::Hello { .., figures })` carries them: the expected values of `due_and_preparing_documents_refine_only_the_host_wait_from_an_actual_refused_hello` (`jig-domain-core/src/tests/runs/run_choice.rs`) name the real figures.
3. A new core test beside it: a due agent task whose admitted worker fitted, then a `fleet::Event::Needs` raising its charter's turns past the worker's, reads `Refused(Bounds { kind: Worker, why: Some(Turns { charter: 1 }), figures: Some(..) })` with the charter's turns and the worker's; lowering the turns again gives the wait it had before.
4. The wire and client contract are `notes/contracts.md`'s, in v19 in place; `hello_refusals_reuse_the_deployment_context_through_server_bytes_and_client_tasks` (`jig-protocol-engine-clients/src/tests/llm.rs`) keeps every row and gains rows for: turns, time and waiting with their charter; a hello with figures and one without; `Bounds` with a `why` and figures, and with neither. Its round trip goes through server bytes to the client's task, as it does.
5. The deployment document keeps the three charters (`translate/deployment.rs` no longer drops them), seen in an existing row of that document's translation tests if one has a turns shortfall, else in the round-trip test of acceptance 4.
6. The three entries this plan wrote are gone: `jig/docs/design/domain/engine.status.md`'s *Not yet: 8: a standing refusal keeps no figures...*, `jig/docs/design/client/domain/contract.status.md`'s *Not yet: 5.1: `TaskWaitRefused::Hello` has no `figures`...* and `jig/docs/design/protocol/client.status.md`'s *Not yet: 3: `TaskWaitHello` has no `figures`...*.
7. Focused tests pass in `jig-domain-fleet`, `jig-domain-core`, `jig-client-domain`, `jig-client-protocol-wire`, `jig-client-wire-world`, `jig-protocol-engine-clients`, `jig-client-protocol-link`, `jig-client`, `jig-client-view`, `jig-core-world` and `jig-fleet-world`, and in temper's `temper-client-view` and `temper-client-tools`; `heavy cargo check --tests` passes for every other package the compiler names (`jig-client-scripted-engine`'s dependents among them).

## Inputs

- `temper/docs/plan/ui-wait-words/notes/contracts.md`: it binds, every section but *ctl's JSON*.
- `temper/docs/plan/ui-wait-words/notes/common.md`.
- `jig/docs/design/domain/engine.md`, section 8: *Standing hello refusals are volatile facts* and *A run its hosts are all too small for says which bound keeps it out*.
- `jig/docs/design/client/domain/contract.md`, 5.1: the task document's `Task.run_wait`.
- `jig/docs/design/protocol/client.md`, section 3: *A task's run wait carries...*.

## Owns and may change

**Owned:** the `files` above.

**Readers, compile-forced edits only** (each a pattern's `..`, a new field, a literal's value, or the arm below):

- jig: `jig/crates/jig-client/src/tests/facts.rs`, `jig/crates/jig-client-view/src/tests/**`, `jig/tests/core/tests/suite/settings.rs`, `jig/tests/fleet/tests/suite/host_kinds.rs`, `jig/crates/jig-client-protocol-link/src/tests/**`, `jig/testing/jig-client-kit/src/**`, and any other exhaustive match or literal the compiler names in jig.
- temper, so that it compiles, exactly as `notes/contracts.md`, *What the `bounds` part leaves in temper*, says, and nothing more: `temper/crates/temper-client-view/src/choice.rs` and `src/tests/{choice,model_check}.rs`; `temper/crates/temper-client-tools/src/jig/render/document/llm.rs` and `src/tests/jig/reads.rs`. The words, ctl's text and its JSON are `wait-words`'. temper's `temper-service` reads `jig_domain_fleet::AgentShortfall`, which doesn't change: leave it.

**Options, each with its default:**

- **The multi-host table test** (acceptance 1): rows of the hosts' bounds and kinds, the needs and the expected `Option<BoundsRefusal>`. Default rows: one worker at 800 turns against 100,000, named with `Figures { required: 100000, offered: 800 }`; two workers at 800 and 1,000, offered 1,000; one of them fitting, none; a fitting worker draining beside a short one, the short one named; the checks timeout of two workers at 10 and 8 minutes against a 5-minute reserve, offered the 8 minutes; one worker short of turns alone and one of time alone, `why` and `figures` none; no host, none; a worker short beside the charter's engine kind not allowed, the worker. Use the file's `Harness` and fixtures.
- **Admitting before the needs** (acceptance 1's existing rows): start the harness with needs that hold no charter, admit the row's worker, then step `Event::Needs(needs)`. If the harness can't start that way, start it with the file's `needs_window(8192)` and bounds that hold it, and say so.
- **The version the core asks with** is `charter_version(task)`. A task whose `charter_version` is none keeps `Host`.
- **An `option` of an enum in the schema:** if the generator refuses `why: option AgentShortfall`, wrap it as `record TaskWaitBoundsWhy { value: AgentShortfall }` (as `TaskRunWaitRefused` wraps its enum), and say so; the contract's Rust shapes stay.
- **jig's own view** (`jig-client-view`, `host_refused`): `Bounds` gets its words as `Hello` does: two new phrases, "No worker can take this run." and "The engine's own host can't take this run.", numbered as `RefusedWorkerAt` is, then `deployment::hello_refusal`'s agent words when `why` is some. Its `deployment/hello.rs` names the new charters with `{ .. }` or, where it already prints a charter for checks, prints them the same way. A golden of the refused chat header that regenerates differently is not expected: if one does, regenerate it and say why.
- **Journeys:** if the focused journey replay calls a recording stale after the wire change, re-record it as `notes/common.md` says. None is expected: no journey's task waits on a refused host.
- **A scripted engine** (`jig-client-scripted-engine`) mirrors the client contract: its codec writes the new fields and arm as the wire's.

## Steps

1. The fleet. `HelloRefusalRecord.figures` is `refuse`'s `figures`. `bounds_refusal` beside `standing_refusal` in `domain.rs`; the per-version shortfall beside `charter_shortfall` and `fits` in `needs.rs`, so that hello admission, placement and this answer read one list of bounds:

   ```rust
   // needs.rs
   /// The first bound of `version`, then of the deployment, that `bounds` falls short of.
   pub(crate) fn version_shortfall(needs: &Needs, version: Version, bounds: &Bounds) -> Option<AgentShortfall>;
   // domain.rs
   pub fn bounds_refusal(&self, version: crate::Version, kinds: crate::Kinds) -> Option<crate::BoundsRefusal>;
   ```

   For `why` and `figures`, build the most the kind's hosts offer (each bound's largest; the least checks timeout; each endpoint's largest window and output) and ask `version_shortfall` of it; a shortfall of that composite is one every host has. Export `BoundsRefusal` from `lib.rs` beside `HelloRefusalRecord`.
2. The core: `TaskWaitRefused` and `waiting` as the contract says.
3. The client contract, then the schema's lines, then regenerate: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world -E 'test(drift::)'`. `git status` shows which goldens moved: the `agent_shortfall_{turns,time,waiting}` and task-wait ones, and new ones for the new records; any other moved golden is a question.
4. The translations: `jig-protocol-engine-clients` (core to wire, keeping the three charters in `deployment.rs`), `jig-client-protocol-link` (wire to contract), the scripted engine's codec.
5. The compile-forced readers, jig's then temper's, as *Owns and may change* says.
6. Tests: acceptance 1 to 5. For each new row that pins a rule, break the rule once by hand (answer the first host's bounds instead of the most; drop `figures` from the record; drop a charter in the translation) and see the row fail for that reason.
7. Remove the three status entries. Then `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p <each package you changed> --all-targets -- -D warnings`; the focused tests of acceptance 7. Commit, push.

## Report

As the implementer's text says, and: the `bounds_refusal` cases each row covers, the goldens that moved, the temper lines you touched and how, and any compile-forced reader beyond the lists above.
