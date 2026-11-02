+++
id = "tooling-capability"
part = "worker-jobs"
kind = "feature"
needs = ["hub-jobs"]
files = [
  "temper/crates/temper-worker-domain-tooling/**",
  "Cargo.toml",
  "Cargo.lock",
]
effort = "high"
minutes = 150
+++

# `temper-worker-domain-tooling`: the worker's job capability, sans-io

## Goal

A new step crate, `temper-worker-domain-tooling`, the worker's job capability
(`temper/docs/design/domain/worker.md, 4.4`): it runs a job of one of its kinds as the
project's command in the job's directory (`temper/docs/design/protocol/tooling.md, 2, 3 and
6`), bounded by its deadline, and answers how it ended and when every process of it is gone.
It is a child domain like `temper-worker-domain-checkout`: sans-io, its operations asked of
the protocol layer as data. Composing it into the worker's root, the hello, the delivery's
push and the protocol layer's process and file operations are `worker-jobs`'s; this task
builds the crate alone, and nothing composes it yet.

Acceptance:

- The crate exists, a member of the workspace, with `notes/contracts.md`, section 8's
  `Kind`, `Spec`, `Domain` (`new`, `bounds`), `step`, `Event`, `Exit`, `Request`, `End`, and its
  own `Limits` and `Invalid`, all exactly as written there; it depends on `skein-lib` and
  `jig-domain-link` only.
- Its unit tests below pass; clippy is clean; `fabrig-boundary`'s direction test passes.

## Inputs

- `temper/docs/plan/reference-m7a/notes/common.md` and `notes/contracts.md`, sections 1 and 8.
- `temper/docs/design/domain/worker.md, 4.4` (the kinds' entries, one spec and one result, a
  command as a process, delivery, the worker's limits), `section 9` (below the domain).
- `temper/docs/design/protocol/tooling.md, 2` (argument list, environment, time), `3` (the
  directory: `request.json`, `home/`, `repos/<name>/`, `result.json`), `6` (bounds) and `1`
  (exit 0 and 3 finish, any other fails). Sections 4 and 5, `prepare` and `key`, are later
  plans': no kind's entry is written here.
- The sibling to copy the shape of: `temper/crates/temper-worker-domain-checkout` (`lib.rs`'s
  module doc, `limits.rs`'s `Limits` and `valid`, `domain.rs`'s `step(domain, env, event,
  out)`, `boundary.rs`, `tests.rs`), and its `Cargo.toml`.

## Owns and may change

Owns: `temper/crates/temper-worker-domain-tooling/**` (new); the root `Cargo.toml`'s
`members` and `[workspace.dependencies]` lines for it, beside
`temper-worker-domain-checkout`'s; `Cargo.lock` as cargo writes it.

Nothing else: no root composes the crate in this task, and `temper/docs/design/README.md`'s
list of the worker's crates is the design's, not yours (say in the report that it needs the
new crate).

Pre-approved options:

- **`Limits`:** `file_bytes` (the request and result files' bound: `tooling_file_bytes`),
  `tail_bytes` (`tooling_log_bytes`), `path_bytes` (the longest command path and directory),
  `jobs` (in flight at once), and `path` (the `PATH` value given to commands, bytes). `valid`
  refuses zeros. If another bound proves needed, add it and say so.
- **`Invalid`** (from `Domain::new`): kinds not ascending or repeated, a verb empty or with a
  NUL or `/`, a bound's spec or result past `file_bytes`, more kinds than a `u16` counts.
- **The job's state:** a `skein_lib::Slab` of jobs by owner, or the sibling's idiom; each
  owner's states: refused, writing, spawning, running, reading, ending, gone. Your choice.
- **A `Run` for an owner already in flight** is refused with `StartRefused` and changes
  nothing of the live one.
- **`Exit::Signal` without `Stop`** fails the job `Failed` with the tail as detail.

## Steps

1. The crate: `Cargo.toml` as the sibling's (description "The worker's job capability: a job's
   command in its directory, sans-io"); `#![cfg_attr(not(test), no_std)]`; a module doc citing
   `temper/docs/design/domain/worker.md, 4.4` and `temper/docs/design/protocol/tooling.md, 2`.
2. `Run`: check the kind is an entry, the request within `file_bytes` and the kind's
   `bound.spec`, the deadline within the kind's `bound.deadline`, the command a relative path
   of no `..` segment, no NUL, within `path_bytes`; else `StartRefused` naming which, and no
   `Gone` follows (nothing ran). Then `Write { path: <directory>/request.json, bytes: request }`;
   on `Written { written: true }`, `Spawn` with program `<directory>/home/<command>`, args
   `[program, verb, <directory>/request.json]`, `dir` the directory, `env` exactly
   `PATH`=limits' `path`, `HOME`=`TEMPER_JOB`=directory, `LANG`=`C.UTF-8`, and `deadline`
   `env.now()` plus the job's deadline; `Written { written: false }` ends `Failed` and `Gone`.
   `Spawned` asks `Started`.
3. `Exited`: `Code(0)` or `Code(3)`: `Read { path: <directory>/result.json, most: min(file_bytes,
   bound.result) }`; `Read { bytes: Some(r) }` ends `Finished { result: r, deliver: code == 0
   && kind.delivers }`; `None` ends `Failed` with the tail. Any other code or `Signal`: `Failed`
   with the tail. `Deadline`: `End::Deadline`. Then, on `Gone`, the root's `Gone` with the
   tail as detail: exactly one per owner that started, last.
4. `Stop`: before the spawn, `Gone` at once; after, `Kill`, then `Gone` when the protocol
   layer's `Gone` comes; an `Exited` after a `Stop` ends nothing (a stopped job may end with
   nothing said, contracts, section 2).
5. Unit tests (`src/tests.rs`, a `Harness` as the checkout's), each citing worker.md, 4.4:
   - `a_job_runs_its_command_in_its_directory_and_finishes_with_its_result_file`: the exact
     `Write`, then `Spawn`'s program, args, env (all four, nothing else), dir and deadline; exit
     0, the result file read, `Ended(Finished { result, deliver: false })` for a kind that does
     not deliver and `deliver: true` for one that does; `Gone` last.
   - `exit_three_finishes_and_never_delivers`.
   - `any_other_exit_fails_with_the_end_of_its_error_output`.
   - `a_missing_or_oversized_result_file_fails_the_job`.
   - `its_deadline_kills_it_and_ends_it_deadline`.
   - `a_stop_kills_it_and_says_gone_once_every_process_is_gone`, and a stop before the spawn.
   - `a_run_of_a_kind_not_entered_or_past_its_bounds_is_refused`, and a command with `..`.
   - `bounds_list_the_kinds_ascending` and `new_refuses_kinds_out_of_order`.
6. `cargo fmt`; `heavy cargo clippy -p temper-worker-domain-tooling --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-worker-domain-tooling`; `heavy cargo nextest
   run -p fabrig-boundary`.
7. One commit; push your branch.

## Report

As the implementer's text says, and also: the `Limits` fields chosen and their meaning; that
`temper/docs/design/README.md`'s crate list needs the new crate; the job states you chose.

## Answers

2026-11-02. **Q:** `JobKind` and `JobBound` are missing from `jig-domain-link` at the base; wait
for `hub-jobs` or bring them in here? **A:** Wait. `hub-jobs` owns them and `jig-domain-link`;
this task now needs `hub-jobs`, and its next run starts from a trunk that has them. Do not
define substitute types or touch `jig-domain-link`. Report `temper/docs/design/README.md`'s
crate list under `## Left`, as the spec says.
