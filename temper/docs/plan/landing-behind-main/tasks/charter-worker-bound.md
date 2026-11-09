+++
id = "charter-worker-bound"
part = "charter-bound"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/assignment.rs", "temper/crates/temper-service/src/settings.rs", "temper/crates/temper-service/src/startup.rs", "temper/crates/temper-service/src/workers.rs", "temper/crates/temper-service/src/tests/worker_parts.rs", "temper/crates/temper-service/src/tests/fixtures.rs", "temper/crates/temper-service/src/tests/welcome_policy.rs", "temper/docs/design/domain/agent.status.md", "temper/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 90
+++

## Goal

The engine never sends a run a charter its worker would refuse. It fits a
charter to the smaller of `hub.charter_bytes` and its worker's charter
allowance: the local agent's where the process has one, else smith's floor,
65,536. Today it fits to `hub.charter_bytes` alone, 262,144 under the
`standard` profile `temper init` selects, and a worker on the stock agent
refuses past 65,536 at decoding (`jig-protocol-worker-agents/src/route.rs`,
`decode`: `smith::Charter::new(charter, host_limits.charter_bytes)`), so a
resolution with a large brief was refused on every attempt
(`temper/docs/design/domain/agent.md`, 4.3).

Acceptance:

- `temper-service/src/assignment.rs` holds `WORKER_CHARTER_BYTES` and
  `charter_bound` as `notes/contracts.md`, B, *The charter bound*, has them;
  `settings::Engine` holds `agent_charter_bytes`, set from the local agent
  at startup; `Workers::configured` builds `Encoding.charter_bytes` with
  `charter_bound`, and the test fixture's `encoding` does the same. The
  link's own sizes (`workers.rs`'s `limits`, `worker_settings.rs`) keep
  `hub.charter_bytes`.
- One table story in `src/tests/worker_parts.rs` (Steps, 2), its rows
  written and seen failing first, runs both real sides: the engine's
  `assignment::down` encodes a resolution whose pull section passes the
  worker's allowance, and the worker's `worker_route::decode` starts it,
  its brief cut with its line.
- The **Not yet** entries this cut wrote are removed, their lines alone
  (`notes/common.md`, *Your status entry*): 4.3 at the end of
  `temper/docs/design/domain/agent.status.md` ("the engine fits a charter
  to `hub.charter_bytes` alone, …") and 13.8 at the end of
  `temper/docs/design/protocol/configuration.status.md`.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3, the bullet *The brief fits
  the room its charter leaves*; section 10, *A charter where the process
  has no local agent*.
- `temper/docs/design/protocol/configuration.md`, 13.8, the bullet *A
  brief*.
- smith's `smith/docs/design/protocol/limits.md`, 3.10: `run_bytes`, the
  larger of 65,536 and twice `tool_payload`, which a host's `charter_bytes`
  takes.
- `temper/docs/plan/landing-behind-main/notes/contracts.md`, B.
- `temper/docs/plan/landing-behind-main/notes/common.md`.
- As they stand: `assignment.rs`'s `down` and `fitted_charter`, and its
  test `encoded_framing_fits_beside_the_fixed_charter`; `worker_parts.rs`'s
  `the_start_passes_each_ask_and_omits_an_empty_one`, which builds a
  `host::Start` from a `LinkDown::Assign` and decodes it at the worker;
  `startup.rs`, where `settings::Engine` takes `host_reply_bytes` from the
  local agent.

## Owns and may change

- Owns the files in `files`. In `workers.rs`, one line alone: the
  `charter_bytes` member of the `Encoding` built in `Workers::configured`.
  The task `worker-relinks/worker-reads` is changing other lines of that
  file; touch no other.
- `settings::Engine` literals elsewhere in `temper-service` take the new
  field as `None` (`tests/fixtures.rs`, `tests/welcome_policy.rs`; any
  other the compiler names).
- Pre-approved: if the story needs a resolution's assignment the fixtures
  don't make, add a fixture function to `tests/fixtures.rs` beside
  `assignment_with`, building one with a brief section titled
  `Pull request` of the given length.
- Pre-approved: if `temper-service`'s tests already derive the stock
  agent's smith limits (from `temper-init`'s `agent.json`), add a row or an
  assertion that `WORKER_CHARTER_BYTES` is its `host_limits.charter_bytes`;
  otherwise the constant's comment and citation carry it.
- Pre-approved: if the fixture's charter without its brief doesn't fit
  32,768 (`down` answers none), give `hub_below_its_agent` the smallest
  `hub.charter_bytes` it fits, below 65,536, and say so in the report.
- Not yours: the link's sizes, `hub.charter_bytes`'s value or any profile's
  figure, the worker's decoding (`jig-protocol-worker-agents`), the
  worker's own fit (`worker_route::fit_start`), the brief's gathering
  (the task `conflict-files-named`), `tests/worker.rs`, `tests/workers.rs`,
  `tests/worker_admission.rs` and `tests/fixtures/worker_peers.rs`.

## Steps

1. **The surface, unchanged in meaning.** Add `agent_charter_bytes` to
   `settings::Engine` beside `host_reply_bytes` (its doc: the local agent's
   charter allowance, smith's `run_bytes`; none when no local agent is
   configured), set at startup from `agent.host_limits.charter_bytes` as
   `host_reply_bytes` is set; add `WORKER_CHARTER_BYTES` and
   `charter_bound(hub, agent)` returning `hub` for now; build
   `Encoding.charter_bytes` with it in `Workers::configured` and in
   `tests/fixtures.rs`'s `encoding`.

2. **The story, failing first,** in `src/tests/worker_parts.rs`:

   ```rust
   #[test]
   fn a_resolution_past_its_workers_charter_allowance_starts_cut() {
       // rows: (name, hub.charter_bytes, local agent's allowance, the worker's)
   }
   ```

   Each row sets `settings.jig.limits.hub.charter_bytes` and
   `settings.agent_charter_bytes` on `fixtures::engine()`, takes
   `fixtures::encoding(&settings)`, and encodes with `assignment::down` a
   resolution's assignment whose `Pull request` section is 200,000 bytes of
   text (both sides of a large generated file). It builds the `host::Start`
   from the `LinkDown::Assign` as `the_start_passes_each_ask_and_omits_an_empty_one`
   does, and decodes it with `worker_route::decode` at `fixtures::agent()`
   whose `host_limits.charter_bytes` is the row's worker allowance:

   | row | `hub.charter_bytes` | local agent | the worker takes | then |
   |---|---|---|---|---|
   | `engine_alone_under_standard` | 262,144 | none | 65,536 | starts; the encoded charter at most 65,536 bytes |
   | `local_agent_takes_more` | 262,144 | 127,480 | 127,480 | starts; at most 127,480 |
   | `hub_below_its_agent` | 32,768 | 127,480 | 127,480 | starts; at most 32,768 |

   Each that starts holds its `Pull request` section's title and ends its
   body with ` bytes; whole: read_pull/read_pull_files]\n`. Run it: the
   first two rows fail with `Undecoded::Charter`; note it for the report.

3. **The rule:** `charter_bound` as `notes/contracts.md`, B, has it, with
   the constant's and the function's comments citing `agent.md`, 4.3. Run
   the story: every row passes. Break the rule once by hand (return `hub`;
   then ignore `agent`), see `engine_alone_under_standard` and then
   `local_agent_takes_more` fail for that reason, restore, and name both in
   the report.

4. **Narrow runs, then the readers.** This changes how large a charter the
   engine sends, so it runs the tests that read the encoding:
   - `heavy cargo nextest run -p temper-service --lib -E 'test(/worker_parts::/) | test(/assignment::/)'`
   - `heavy cargo nextest run -p temper-service --lib -E 'test(/stock_brief::/) | test(/ask_bound::/) | test(/startup/) | test(/worker/)'`
   - `heavy cargo check -p temper --tests` (the binary builds `settings::Engine` through startup)
   Check each run's count; one that selected nothing proves nothing. A
   test that pinned a charter fitted to `hub.charter_bytes` past the
   fixture worker's 65,536 changes its expectation to the bound and is
   named in the report.

5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-service --all-targets -- -D warnings`; remove the two
   status entries; commit saying why (a worker refuses a charter past its
   allowance before its own fit, so the engine sends none past it); push.

## Report

As the implementer's text has it, and: each row's failure before the
rule; the rule broken for each bite and the failure seen; each other test
whose expectation moved, and why; whether the constant is held to the
stock agent's derived figure or by citation alone; the counts of the runs
in step 4.
