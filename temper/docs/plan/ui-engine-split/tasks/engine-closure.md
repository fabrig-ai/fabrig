+++
id = "engine-closure"
part = "engine-closure"
kind = "feature"
needs = ["engine-binary", "worker-feature"]
files = ["tests/boundary/tests/binaries.rs", "tests/boundary/Cargo.toml", "temper/docs/design/protocol/README.status.md"]
effort = "high"
minutes = 60
+++

## Goal

The engine's half of the deployment's boundary test: `temper-engine`'s
dependency closure, as `cargo tree` gives it for its package, holds no
worker domain and no smith agent, apart from its named exceptions (the
request's third item; `temper/docs/design/protocol/README.md`, 4.1,
"What it links is held by a test", and 4.2, "What it links"). Later
binaries add their rows to the same table.

Acceptance:

- `tests/boundary/tests/binaries.rs`, a test binary of `fabrig-boundary`,
  ordinary Rust under the boundary's lints, its module doc citing
  `temper/docs/design/protocol/README.md`, 4.1. Its table is
  `notes/contracts.md`, 8's `BINARIES`, with one row, `temper-engine`:
  - `excluded`, as 4.2 lists them: `temper-worker-`,
    `jig-protocol-worker-agents`, `jig-protocol-worker-link`,
    `smith-host-`, `smith-agent-`, `smith-local-`, `smith-inline-agent`,
    `smith-domain`, `smith-domain-`, `smith-protocol-channel`,
    `smith-protocol-events`, `smith-protocol-llm`,
    `smith-protocol-machine`, `smith-protocol-program`, `smith-events`;
  - `exceptions`: `smith-domain`, `smith-domain-run`,
    `smith-domain-session`, `smith-domain-tools`, each with 4.2's reason
    in a sentence.
- **The check:** for each row, `cargo tree -p PACKAGE -e normal --prefix
  none -f '{p}' --locked --offline`, run at the workspace's root (as
  `skein_worlds.rs` runs `cargo metadata`), each line's first word a
  crate's name. The test fails, in one message, naming each excluded
  crate found that is no exception, with `cargo tree -p PACKAGE -e normal
  -i CRATE --locked --offline`'s output for it, and each exception no
  longer found ("an exception the closure no longer holds: drop it").
  `cargo tree` that fails or prints nothing is a failure, never a pass.
- **A table test of the matching,** in the same file, over fixed names
  and no `cargo`: `(excluded, exceptions, closure) → findings` rows: a
  prefix entry matching `smith-agent-shell` and not `smith-agents`; an
  exact entry matching itself and not a longer name; an excepted crate
  found (no finding); an exception not found (a finding); a clean
  closure.
- **It bites:** on your branch alone, give `temper-engine`'s
  `temper-service` dependency `features = ["worker"]`, run the test, see
  it name `temper-worker-domain` and the others, and restore; remove an
  exception, see it named, restore. Confirm too that a dev-dependency
  asking the feature (add one to `temper-engine`'s `[dev-dependencies]`
  for the trial) leaves the closure clean, as `-e normal` resolves
  without dev-dependencies; restore. Report each.
- If `binaries.rs` exists on your base (the plan `agent-binary` may
  write it first, with the agent's row), add the engine's row to its
  table in its shape, keeping its mechanism, and say so; the matching
  test is then the one there, given the rows above if it lacks them.
- The status entry closes: in `temper/docs/design/protocol/README.status.md`
  the **Not yet** 4.1, 4.2 entry "`tests/boundary/tests/binaries.rs`
  doesn't exist: …", its lines alone. This departs from the implementer's
  text, which closes `docs/design/` to you: the entry is this task's.

## Inputs

- `temper/docs/design/protocol/README.md`, 4.1 and 4.2.
- `tests/boundary/tests/skein_worlds.rs` (running cargo from a test) and
  `tests/boundary/tests/direction.rs` (the boundary tests' style).
- `temper/docs/plan/ui-engine-split/notes/contracts.md`, 8.
- The report of task `worker-feature`, for any worker crate it kept.
- `temper/docs/plan/ui-engine-split/notes/common.md`.

## Owns and may change

- Owns the files in `files`; `tests/boundary/Cargo.toml` only if a
  `[[test]]` line is needed (the directory's tests are found without
  one today).
- Pre-approved: if the closure holds an excluded crate that
  `worker-feature`'s or `engine-binary`'s report names as kept, with the
  reason, add it to the row's exceptions with that reason, add it to
  4.2's exception list in `temper/docs/design/protocol/README.md` (its
  line alone), and name it in the report. Any other excluded crate found
  is a red of the plan's: report it, with `cargo tree -i`'s path, as
  `Failed:`, and push the test as written.
- Not yours: any manifest of `temper/` (but the trials above, restored).

## Steps

1. The matching's table test; see it fail against a stub; then the
   matching.
2. The check; `heavy cargo nextest run -p fabrig-boundary --test
   binaries`; read the count.
3. The trials above, each restored.
4. The status entry.
5. `cargo fmt`; `heavy cargo clippy -p fabrig-boundary --all-targets --
   -D warnings`; `heavy cargo nextest run -p fabrig-boundary`.
6. `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit and push.

## Report

As the implementer's text says, with the closure's size, each trial's
failure, and the exceptions as written.
