+++
id = "many-slots"
part = "many-slots"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/worker_settings.rs",
  "temper/crates/temper-service/src/startup/checks.rs",
  "temper/crates/temper-service/src/startup/problems.rs",
  "temper/crates/temper-service/src/startup/reference.rs",
  "temper/crates/temper-service/src/limits/names.rs",
  "temper/crates/temper-service/src/tests/worker_slots.rs",
  "temper/crates/temper-worker-domain/src/limits.rs",
  "temper/crates/temper-worker-domain/src/tests/**",
  "temper/crates/temper-playground/src/documents.rs",
  "temper/crates/temper/tests/suite/startup.rs",
  "temper/docs/help/server.md",
  "temper/docs/help/reference/**",
  "temper/docs/design/domain/worker.status.md",
  "jig/crates/jig-protocol-config/src/problem.rs",
  "jig/crates/jig-protocol-config/src/worker.rs",
  "jig/crates/jig-protocol-config/src/members.rs",
  "jig/crates/jig-protocol-config/src/tests/worker.rs",
]
effort = "high"
minutes = 120
+++

# A worker can start with more than one slot

## Goal

A real worker refuses `workers[0].slots` above 1 on every profile: `workers[0].slots: must agree with the agent's agents`. smith's `host_limits` sizes its host for one agent (`agents: run.runs`, and `runs: 1`), and temper takes that number as it is. Make the number of agents the host holds at once follow the worker's slots, where temper composes them, and make each start's refusal about slots name what to raise.

This is urgent: a comparison waits on it. Take the fewest lines that are right.

Acceptance:

- A failing test first (T1 below), committed on its own; the report quotes its failure on the code before the fix.
- T1 to T4 pass, and each bites (implementer's text).
- The owned example's documents, with the raises in T1 and nothing else changed, start at 8 slots and 10 workspaces, `engine.memory_bytes` at the example's 1 GiB and `agent.json` as it is.
- `smith-domain-run`'s `runs: 1` and all of `smith/` are unchanged. `Unfit::Workspaces` stays. No protocol changes: if the work seems to need one, stop with `Question:`.
- `temper help server` and `temper help configuration` say what must rise, within their byte bounds.
- `temper/docs/design/domain/worker.status.md` loses the `6:` entry under **Not yet**, once it's built.
- The report has the figures under `## Memory` below, and the list of limits that scale, under `## Report`.

## Inputs

Read:

- `temper/docs/design/domain/worker.md`, 6: the contract for this task.
- `temper/docs/design/protocol/README.md`, 15.6 and 15.7: the cluster's workers keep one slot from their fixtures; don't change them.
- `temper/docs/design/protocol/configuration.md`, 7: "The engine's byte budget".
- `jig/docs/design/domain/engine.md`, 8: the fleet uses a hello's slots up to `fleet.slots`.
- `skein/docs/foundation/testing-strategy.md`, 8.1, if a budget is at issue. Budgets and timeouts are the user's.

Departures from the implementer's text:

- **`docs/help/` and one status file are yours here:** `temper/docs/help/server.md`, the generated `temper/docs/help/reference/**`, and `temper/docs/design/domain/worker.status.md`. Other `docs/design/` files stay closed.
- **This branch crosses trees,** by the request's one task: a small `jig-protocol-config` change and its consumers in temper. Nothing in `smith/`.
- **fabrig/ui has priority:** leave `temper-client-*`, `temper/tools/gallery`, `temper/tools/journeys` and `temper/tests/client*` alone.

## Owns and may change

Owned: the `files` above.

Readers and neighbours, and the edits allowed there:

- Any test document in `temper/` or `jig/` that a new refusal breaks, with `workers[..].slots` above its engine's `fleet.slots` or its worker's `hub.slots`: raise that limit in the document, nothing else. List each in the report.
- `temper/crates/temper-service/src/tests/*` and `temper/crates/temper-service/src/tests/fixtures/*`: a `mod worker_slots;` line, and a helper made `pub(crate)` for T3.
- `temper/crates/temper-service/src/tests/startup.rs`, `checks.rs`: only if an expected refusal's words changed. The line `application.limits.overrides.git_component.operations: must agree with checkout.workspaces` does not change.
- `temper/crates/temper-service/Cargo.toml`, `temper/crates/temper/Cargo.toml`: a dev-dependency line T1 or T3 needs.

Options, each pre-approved:

- **The engine's `hub.slots`:** the owned example's `engine.limits.overrides` also has `hub.slots` 2. Leave it at 2 in T1. If a start refuses it, raise it in T1, add it to T2's table and to the help's words, and say so in the report.
- **Sockets:** `checks::sockets` counts `slots * 4 + git.operations * 4` against the constant `limits::SOCKETS`, 128, and names `engine.limits.io.sockets`, which `process_io` overwrites, so no document can raise it. By reading, 8 and 10 come to 88 plus the webhooks and sign-in terms. If T1 hits it, raise `SOCKETS` to the next power of two and say so; a refusal that names a limit no one can raise is a fault of this task.
- **Any other anonymous refusal at 8 and 10** (`workers[0].limits: past its bound`, say, from `worker_settings::prepare`, `link::valid`, or `worker_cap`'s or `engine_cap`'s checked arithmetic): this is the fault the request warns of. Find the limit, and name it in the refusal's words if that takes a line or two; otherwise report it as `Question:`, with its figures.
- **T3's route:** by default the lib test below. If driving a second agent through `crate::Worker` needs more than a helper made `pub(crate)`, use `temper/tests/one-process` instead, whose `temper_playground::documents::agent()` derives the real `host_limits` and, with `host_limits.agents = AGENTS` gone, reaches the composition. Say which you used and why, with the tier's measured time.

## Steps

### The composition

`temper/crates/temper-service/src/worker_settings.rs`, `root_limits`:

```rust
agent: smith_host_domain::Limits { agents: worker.jig.slots, ..agent.host_limits },
```

`temper/crates/temper/src/agent.rs` and smith's `host_limits` stay as they are. `settings::Agent.host_limits` keeps smith's number; only the composed `root::Limits.agent` is read for capacity (`temper-worker-domain/src/domain.rs`, `Domain::new`).

`temper/crates/temper-playground/src/documents.rs`: remove `host_limits.agents = AGENTS;` and the `mut`. Its `workers[0].slots = AGENTS` now carries it.

### The rule

`temper/crates/temper-worker-domain/src/limits.rs`:

- Remove `Unfit::Agents`, its row, and lower `UNFIT` to 19. A start can no longer break the rule, and its words named nothing a person can raise.
- Hold the domain's invariant in `valid`, beside the same idiom for the tooling's jobs, which also follow the slots: `&& limits.agent.agents >= limits.host.slots`. Worlds that build `Limits` by hand still answer to it.

`temper/crates/temper-service/src/startup/checks.rs`: remove `Unfit::Agents` from `worker_payload`'s list and `worker`'s table.

### The refusals

Each names what to raise. Exact lines, under `temper serve: `, for T2:

| Left short | Line |
|---|---|
| `workers[0].limits.overrides` `hub.slots` | `workers[0].slots: past this worker's limits.overrides.hub.slots, which must be at least its slots` |
| `engine.limits.overrides` `fleet.slots` | `workers[0].slots: must agree with engine.limits.overrides.fleet.slots, at least the slots` |
| `application.limits.overrides` `checkout.workspaces` | `workers[0].slots: must agree with application.limits.overrides.checkout.workspaces, at least the slots` |
| `application.limits.overrides` `git_component.operations` | `application.limits.overrides.git_component.operations: must agree with checkout.workspaces` (unchanged) |

- **`hub.slots`:** `jig/crates/jig-protocol-config/src/worker.rs` pushes `What::Bound`, and reads `past its bound`. Add a variant to `jig/crates/jig-protocol-config/src/problem.rs`, `What::SlotsPastHub`, with a one-line doc comment, and push it there; `temper-service/src/startup/problems.rs`, `jig_words`, gets the words above. jig's words table, if another matcher lists every variant, takes its own short arm. Update `jig/crates/jig-protocol-config/src/tests/worker.rs`, line 123.
- **`checkout.workspaces`:** the `other` of `Unfit::Workspaces` in `checks::worker`.
- **`fleet.slots`, new:** today no start compares them, and `jig-domain-fleet/src/channel.rs` silently uses a hello's slots up to `fleet.slots`, so 8 slots against 2 starts and runs 2. In `checks::standalone`'s `for (index, worker)` loop, past its `alone` return:

  ```rust
  if worker.slots > engine.limits.core.fleet.slots {
      problems.push(path.member(b"slots"), What::Against { other: b"engine.limits.overrides.fleet.slots, at least the slots" });
  }
  ```

  An engine alone can't see a worker's slots and stays as jig has it (`temper/docs/design/domain/worker.md`, 6).

### The words for a person

- **`temper/crates/temper-service/src/startup/reference.rs`, `configuration_reference`:** add a section, rendered as `merge_command` is, after it, with these words; then regenerate:

  ```markdown
  ## More slots

  `workers[0].slots` bounds the runs a worker hosts at once, an agent process each. The host's agents follow it, and nothing in `agent.json` raises them. Each of these must be at least the slots, and a start names the one that's short:

  - `hub.slots`, in `workers[0].limits.overrides`.
  - `fleet.slots`, in `engine.limits.overrides`. An engine alone can't see a worker's slots: it uses them up to this, and turns away a hello listing more runs.
  - `checkout.workspaces`, in `application.limits.overrides`, and `git_component.operations` there, at least the workspaces.
  ```

- **`jig/crates/jig-protocol-config/src/members.rs`, the worker's `slots`:** keep its line unless it misleads; jig may name `hub.slots` and `fleet.slots`, and must not name temper's limits.
- **`temper/crates/temper-service/src/limits/names.rs`:** add to the lines of `checkout.workspaces` and `git_component.operations` that each is at least the slots, or the workspaces. Keep each a line.
- **`temper/docs/help/server.md`:** it is a walk, whose bound is 12,288 bytes, and it is at 12,288 now, so what you add you must take back: condense its sentences, losing no fact. After the `engine.memory_bytes` paragraph:

  > To run more at once, raise `workers[0].slots` with the worker's `hub.slots`, the engine's `fleet.slots`, and `checkout.workspaces` and `git_component.operations`: a start names any that is short (`temper help configuration`). Each slot is an agent process, up to `agent.json`'s `declared.memory`.

- Regenerate the reference pages with `SKEIN_CODEGEN_WRITE=1` and the tests in `temper/crates/temper/src/tests/configuration.rs` and `limits.rs`, never by hand. A page past its bound is condensed, losing no fact.
- `temper/docs/help/**` is read by tests (`.config/gate/read-by-tests.txt`): run `temper-docs`' tests and `temper`'s `help` lib tests.

### Tests

**T1, first and failing,** in `temper/crates/temper/tests/suite/startup.rs`, by `serve_prepares_once_on_the_owned_example`:

```rust
/// The owned example, raised by what a worker of 8 slots and 10 workspaces needs.
fn eight_slots(case: &str, leave: Option<&str>) -> support::Scratch { /* owned_example, then edit temper.json */ }

#[test]
fn serve_prepares_eight_slots_and_ten_workspaces_on_the_owned_example() {
    let scratch = eight_slots("eight-slots", None);
    let prepared = temper::prepare_in(&support::configuration::directories(scratch.path()));
    assert!(prepared.is_ok(), "{}", prepared.err().unwrap_or_default());
    scratch.remove();
}
```

Its raises: `workers[0].slots` 8; `workers[0].limits.overrides["hub.slots"]` 8; `engine.limits.overrides["fleet.slots"]` 8; `application.limits.overrides["checkout.workspaces"]` 10; `application.limits.overrides["git_component.operations"]` 10. Before the fix it fails with `workers[0].slots: must agree with the agent's agents`.

**T2,** same file: `serve_names_what_to_raise_for_eight_slots`: for each row of the refusals' table, `eight_slots` leaving that one member at the example's, and `prepare_in`'s error equals that line exactly. Row by row, so a stray second problem fails it. To bite, restore one refusal's old words.

**T3, two agents at once through the real host,** the lowest tier reading the real `host_limits`: a new `temper/crates/temper-service/src/tests/worker_slots.rs`, `two_agents_run_at_once_under_the_real_host_limits`:

- The agent is `fixtures::parsed_agent(fixtures::stock_configuration(10_662, 8192).expect("stock agent"))`, so its `host_limits` is smith's own and `agents` is 1. Assert that, so the test can't pass on a fixture's number.
- The worker has `slots` 2 and `hub.slots` 2, `own.checkout.workspaces` 2, and is otherwise sized to fit that agent (hub turns, calls, held, inbox, as `unfit` asks; `fixtures/startup.rs`, `worker_peers.rs` and `root_checks.rs`' `complete_profile_startup` show the shapes).
- Give `crate::Worker` two assignments with distinct `run` tokens, as `worker_peers.rs`' `assigned_agent` does (`fixtures::chat_assignment`, through `root::step`, decoding each `Request::Decode`).
- The oracle: two distinct `smith_host_domain::process::Request::Spawn` owners, neither answered as busy, and `worker.root`'s agent child holding two agents at once.
- To bite, restore `agent: agent.host_limits` in `root_limits`: `prepare` refuses, or the second run is refused busy.

**T4,** `temper/crates/temper-worker-domain/src/tests/`, with its limits' tests or a new row: `valid` is false for `agent.agents` below `host.slots`, and true at equal.

### Finishing

The implementer's text holds: `cargo fmt`, `old_forms.py` for `temper` and `jig`, clippy on every package changed, focused tests. **This changes what a shared item means**, so also run the readers' tests: `temper-service`, `temper-worker-domain`, `temper-playground`, `temper`'s `--test suite`, `temper-worker-domain-world` (`temper/tests/worker/domain`; check the package's name in its manifest), `temper/tests/one-process`'s package, `jig-protocol-config`. `heavy cargo check -p <dependent> --tests` for `jig-protocol-config`'s direct dependents. Then `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

The implementer's report, and these, under their own headings:

### Memory

Say what a start's memory comes to at 8 slots under `standard`, with the owned example's overrides, and whether `engine.memory_bytes` refuses. Print the real figures once from throwaway code (`sizes::worker_cap`, `sizes::engine_cap`, the composed limits), and confirm or correct this reading:

- `engine.memory_bytes` refuses only `0` (`temper-engine-domain/src/limits.rs`, `valid`; `temper/docs/design/protocol/configuration.md`, 7), so 1 GiB refuses by nothing. It paces: at or past it, due tasks wait.
- What it paces: at 8 slots, each holding `hub.transcript_bytes` 67,215,250 and `hub.turn_queue_bytes` 134,430,480, comes to 1,613,165,840 bytes, 539,424,016 above the 1,073,741,824 budget; with each attempt's stopping reserve of another 134,430,480, 2,688,609,680, or 1,614,867,856 above.
- The worker's machine: `agent.json`'s `declared.memory` 2,147,483,648 times 8 is 17,179,869,184 bytes of agent processes (`standard()`'s own 2,281,701,376 comes to 18,253,611,008), carried with no start's fit check (`smith/docs/design/protocol/limits.md`, 5).
- The links' output caps, `worker_cap` and `engine_cap`: about 1.08 GB and 0.55 GB at 8, by reading. They are `u32`, so they overflow at about 31 slots.

### What scales

The list of smith's and temper's limits that scale with the slots, each confirmed. By reading, and to be checked:

- **Follow the composed number or the slots:** smith's host `agents` slab and `alarms` (`agents * 3`), the worker root's `agents`, `gone`, and `calls` (`agents * calls`) maps; `temper-service/src/worker.rs`'s `processes`, `owners`, `entities` (`slots * 4`), `retired`, `jobs`; `limits::tooling(slots)`; the link's `relays` and `busy`, and `sizes.slots` = `worker.slots`.
- **Per agent, so they don't scale and needn't:** smith's host `outbox`, `calls`, `turns`, `long_open`; `smith-domain-run`'s `alarms` under `runs: 1`.
- **Fixed, and only pacing:** `temper_worker_domain::WATERMARK`, 10,198, against a step's output at 8 slots; its comment's sum at one slot is 5,099, and the replay term grows by 7 times 17 to 5,218. Say whether anything waits on it for good.
- **Engine side, by `fleet.slots`:** `sizes.slots`, so `hello_hosting` and `engine_cap`; the fleet's `hosts` set; `fleet.workers * fleet.slots` in attempts.
