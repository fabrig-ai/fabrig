+++
id = "engine-serve"
part = "engine-shell"
kind = "migration"
needs = ["engine-forge"]
files = ["temper/crates/temper-engine-shell/**", "temper/crates/temper/src/lib.rs", "temper/crates/temper/src/host.rs", "temper/crates/temper/src/restart_fault.rs", "temper/crates/temper/src/bundle.rs", "temper/crates/temper/src/github_logs.rs", "temper/crates/temper/src/google_runtime.rs", "temper/crates/temper/src/accounts_runtime.rs", "temper/crates/temper/src/printed.rs", "temper/crates/temper/src/printed/**", "temper/crates/temper/src/worker_alone.rs", "temper/crates/temper/src/worker_half.rs", "temper/crates/temper/src/directories.rs", "temper/crates/temper/src/tests.rs", "temper/crates/temper/src/tests/program_environment.rs", "temper/crates/temper/Cargo.toml", "temper/docs/design/protocol/README.status.md"]
effort = "high"
minutes = 180
+++

## Goal

`serve`'s host and start move out of the box's `temper` library into
`temper-engine-shell`'s `serve` module, where an engine alone's document
starts with no worker code. The box's `temper` gives the worker's half
of a standalone or worker alone's start through a trait,
`serve::WorkerHalf` (`temper/docs/design/protocol/README.md`, section 5's
`crates/temper` and `crates/temper-engine-shell` rows, and 4.2, "Its
lines name it"). The box keeps every command, word and test.

Acceptance:

- **The move** (`notes/contracts.md`, 6, `serve`): `temper/src/lib.rs`'s
  `serve`, `prepare`, `prepare_with_store`, `prepare_with_limits`,
  `close`, `operations_in`, `boxed_operations_in`,
  `operations_in_mode`, `operations_in_with`, `prepare_in`,
  `prepare_in_with` and the helpers only they use (`given_secret`,
  `read_files*`, `trust`, `server_name`, `accounts`, `github_runtime`,
  `held_refusal`, `listener_words`, `box_refusal`, and the like); and
  `host.rs`, `restart_fault.rs`, `bundle.rs`, `github_logs.rs`,
  `google_runtime.rs`, `accounts_runtime.rs`, `printed.rs` with its
  tests, and `program_environment` (from `temper::directories`), into
  `temper-engine-shell`, unchanged but for paths, the binary's name and
  the seam below.
- **The seam.** `serve::WorkerHalf` and `serve::WorkerStart` as the
  contract sketches them. `prepare_with_limits`'s worker inputs come
  from it: a worker alone's document is `WorkerHalf::alone`'s whole; a
  standalone one's `agent`, `runtime` and `program_login` are
  `WorkerHalf::start`'s, and after startup `WorkerHalf::configure` sets
  the program sources, places the launcher and applies workspace reuse
  and the LLM dump. The program environment (`printed::running_word`,
  `program_environment`) is the engine's, computed in the shell. Given
  no `WorkerHalf`, a document whose shape is a worker alone, or whose
  named worker is present, is refused before startup: `{binary} serve:
  temper.json names a worker, and {binary} runs none`.
- **The box's half:** a new `temper/src/worker_half.rs` implements
  `WorkerHalf` from what stays in `temper`: `agent::preflight` and the
  worker's root (today's `prepare_with_limits` lines), `program_login`'s
  line, `configure_sources` and `claude_code::place`,
  `workspace_reuse`, `dump_llm`, and `worker_alone::prepare` for
  `alone`. `temper`'s `run` serves with `serve::run("temper", …,
  Some(&its half))`.
- **The binary's name:** serve's lines (`host.rs`'s `temper serve: …`,
  `problems`, the refusals) take `binary` where they say `temper`
  (`Host` holds it; `note_line` and its kin take it), the box passing
  `"temper"`, so its lines are unchanged. One row in an existing table
  test of the host's lines, with `binary` `temper-engine`, reads
  `temper-engine serve: …`. Break it once, see it fail, restore.
- **Every path stays** (`notes/contracts.md`, 6): `temper::prepare_in`,
  `prepare_in_with`, `operations_in`, `boxed_operations_in`,
  `operations_in_with` keep their signatures as wrappers passing the
  box's `WorkerHalf`; `temper::Host`, `service_ending`, `bundle`,
  `github_logs`, `printed` and `temper::directories::program_environment`
  are re-exports. No file outside `temper/src`, `temper/Cargo.toml` and
  the new crate changes: `temper/tests/**`, `temper/tests/real-loop` and
  the worlds build and pass as they are. A test of `temper`'s that
  reaches a moved private item by `crate::…` (`tests/help.rs`'s
  `crate::host::note_line`) reaches it through a `pub(crate) use`, its
  call given `"temper"` if the item now takes the name.
- **The engine's half names no worker:** the moved code calls only the
  names `notes/contracts.md`, 5 says exist both ways. If `temper-service`
  has the `worker` feature on your base (task `worker-feature`), `heavy
  cargo check -p temper-engine-shell` passes (it builds the service
  without it) and `cargo tree -p temper-engine-shell -e normal --locked
  --offline` holds no `temper-worker-*`, `smith-host-*` or
  `smith-agent-*`; if not yet, `worker-feature` checks it when it runs.
- Every test of `temper` passes unchanged; the tests of moved code that
  reach no box item move with it.
- The status entry closes: in `temper/docs/design/protocol/README.status.md`,
  the **Not yet** 4.1, 5 entry "`temper-engine-shell`, serve: …", its
  lines alone. This departs from the implementer's text, which closes
  `docs/design/` to you: the entry is this task's.

## Inputs

- `temper/docs/design/protocol/README.md`, section 4 (`temper serve`'s
  bullet), 4.1, 4.2 and section 5's `crates/temper`,
  `crates/temper-engine-shell` and `crates/temper-service` rows; 15.1.
- `temper/docs/design/client/browser.md`, 2.6 (the bundle).
- `temper/docs/plan/ui-engine-split/notes/contracts.md`, 5 and 6.
- `temper/docs/plan/ui-engine-split/notes/common.md`.

## Owns and may change

- Owns the files in `files`; in `temper/src/lib.rs`, everything above
  and the `mod`/`use` lines of the moved modules; `link_file`'s use in
  serve is `temper-shell`'s already.
- `temper/src/box_command.rs`, `fake_backends.rs` and `program_login.rs`:
  their paths to `printed`, `prepare_in*` or the host alone, if a
  re-export can't keep them; name each.
- Pre-approved:
  - The argument lists of `WorkerHalf`'s methods, `run` and the
    `prepare*`/`operations*` functions may change to what the moved code
    needs; the names and roles bind. Say what changed.
  - A helper both halves call (`path`, `errno`, `private_root`,
    `dump_llm`, `workspace_reuse`): under ten lines, the crate takes a
    private copy; longer, it moves to the crate, `pub`, and `temper`
    calls it there.
  - If `bundle`'s embed needs `temper-client-bundle` in the crate, the
    crate takes it and `temper` keeps its own line too.
- Overlaps: `tui-box-front` is changing `lib.rs`; keep its `Command`,
  `configure_with`, `configure_row` (but for `serve`'s arm, if it must
  change) and the client's arms as they stand.
- Not yours: `agent.rs`, `claude_code.rs`, `program_login.rs`'s logic and
  `main.rs` (`agent-binary` is changing the agent's modes); `forge` (done
  by `engine-forge`); `temper-service`.

## Steps

1. The host's line row first, against a `binary` it doesn't take yet;
   see it fail.
2. The move, the seam and `worker_half.rs`; `heavy cargo check -p
   temper-engine-shell -p temper --tests` while errors last.
3. `heavy cargo nextest run -p temper-engine-shell`; `heavy cargo
   nextest run -p temper -E 'test(/startup|serve|box|configuration|program_environment|help|forge/)'`;
   read the counts.
4. The status entry.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I
   -B tools/lints/lints.py`; `heavy cargo clippy -p temper-engine-shell -p
   temper --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-shell -p temper`; `heavy cargo check -p
   temper-real-loop-world -p temper-playground -p temper-client-world
   -p temper-one-process-world --tests`.
6. `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit and push.

## Report

As the implementer's text says, with the seam's final signatures, the
helpers copied or moved, and the tests that moved or stayed.
