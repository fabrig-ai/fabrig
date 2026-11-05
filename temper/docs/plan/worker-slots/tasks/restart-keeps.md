+++
id = "restart-keeps"
part = "restart-keeps"
kind = "feature"
needs = ["evict-keeps"]
files = ["temper/crates/temper-worker-domain-checkout/src/**", "temper/crates/temper-worker-protocol-git/src/**", "temper/crates/temper-fake-git/src/**", "temper/crates/temper-fake-git-hosted/tests/suite/**", "temper/tests/worker/checkout/**", "temper/tests/worker/protocol-git/**", "temper/tools/git-transcripts/**"]
effort = "high"
minutes = 120
+++

# With `TEMPER_WORKSPACE_REUSE` set, a restart keeps the workspaces

## Goal

The cache is in memory, and a start moves `workspaces` into `trash`, so a restart loses every clone twice over. Set, a start leaves `workspaces`, and a workspace's first hold looks at what its directory holds: where each repository the spec names is there, a git repository, from that remote, it goes straight to fetch and check-out; otherwise it is made, as today. A clone a killed git left damaged heals as any spoiled workspace does. A workspace's path stays the same across a restart.

Acceptance:

- Set: a start moves `home` into `trash`, keeps `workspaces` with what it holds, and makes `workspaces` only if it is absent. A `workspaces` reached through a link still refuses the start.
- Set: a workspace's first hold in this worker's life, with no repository starting saved, tells `Cached::Looking` and runs `Op::Look`, one repository at a time, in the spec's order. All hold: the workspace counts as cloned (`Cache::cloned`) and the next step is `Fetch { repository: 0 }`. One does not: the next step is `Make`, then clones as today.
- Set: a saved start tells `Cached::New` and makes, as today. A look that fails (`Broken`, `TimedOut`, `Cancelled`) spoils the workspace and fails the prepare `Transient`, and an abort or a release during a look behaves as during any prepare step.
- `Op::Look` runs exactly the invocation of `temper/docs/design/protocol/git.status.md`, from the worker's root (`Op::Make`'s spawn directory), with no identity, and ends as that entry says, its deadline `local_timeout`.
- A stale `.git/index.lock` heals: the look holds, the check-out breaks, the prepare fails `Transient`, and the next prepare tells `Cached::Rebuilt`, makes it again and is ready, the lock gone.
- `<root>/workspaces/<token>/<repository>` stays the same across a restart: tests hold the first workspace's token at 0 and the second's at `1 << 32`, whichever workstream comes first, and `invocation::place`'s bytes for both.
- Unset: nothing differs from today (`notes/common.md`). In particular `the_start_moves_workspaces_and_home_into_trash_and_makes_them_afresh` and every existing transcript stay as they are.
- The stories below pass, each with the variable set and unset.

## Inputs

- `temper/docs/design/domain/worker.status.md`, the **Departs** entry on 5 and 9.
- `temper/docs/design/protocol/git.status.md`, the **Departs** entry on 2 and 4.
- `temper/docs/design/protocol/git.md`, 2, 3, 4.1, 4.2 and 9. `temper/docs/design/domain/worker.md`, 5 and 9.
- `temper/docs/plan/worker-slots/notes/common.md`.
- `temper/docs/plan/worker-slots/notes/contracts.md`, all of it.
- `temper/tools/git-transcripts/README.md`.
- The request: `temper/docs/plan/worker-slots/inputs/request.md`, changes 2 and 3.

What the lead found with real git 2.47.3, so you need not settle it again: `git clone` writes `remote.origin.url` as the URL it was given, and no credential travels in that URL. `--git-dir=<dir>/.git config --local --get remote.origin.url` exits 0 with the URL and a newline; exits 1 for a repository with no such key; and exits 128 for a directory that is missing, is plain, or holds a `.git` with only a config. `checkout --force --detach` with a stale `index.lock` exits 128, `fatal: Unable to create '<path>/.git/index.lock': File exists.`, and `output::classify` already makes a check-out's failure `Broken`. A fetch doesn't take that lock, so it still succeeds.

## Owns and may change

Owns: `temper-worker-domain-checkout` (`git.rs`, `facts.rs`, `cache.rs`, `hold.rs`, `tests.rs`); `temper-worker-protocol-git`; `temper-fake-git`; `temper-fake-git-hosted`'s tests; `temper/tests/worker/checkout`; `temper/tests/worker/protocol-git`; `temper/tools/git-transcripts`.

May also change, and how:

- `temper/tests/worker/checkout/tests/suite/reuse.rs` and `Harness::reusing` in `temper-worker-domain-checkout/src/tests.rs`, written by `evict-keeps`: with the variable set a first hold now tells `Looking`, not `New`, and starts with a look. Change the set column's `told.new` to `told.looking`, and let the harness answer a look `Looked { holds: false }`. The numbers of makes, clones, kept and evicted stay.
- `temper-service/src/worker.rs`: `Worker::reuse_workspaces` also calls `self.git.component.keep_workspaces()`. `temper-service/src/worker_git.rs`: the identity arms, with `Look` beside `Make`. `temper-service/src/tests/workspace_reuse.rs`: the component's flag too.
- Compiler-forced arms only, `Look` beside `IsAncestor`: `temper/tests/system/src/composed.rs`, `temper/tests/worker/domain/src/runtime.rs`, `temper/tests/worker/domain/src/whole/jobs.rs`, `temper/tests/worker/domain/src/whole/orders.rs`, `temper/crates/temper-worker-domain/src/tests/jobs.rs`, `temper/crates/temper/tests/suite/fake.rs`.

Options, each pre-approved:

- **Capturing the transcripts.** If `TEMPER_FORGEJO` names an executable where you run, run the tool (step 6). Keep the four new case directories, the new `facts.txt` line, and nothing else: restore any other transcript the run changed with `git checkout`, and say which. If it doesn't, don't look for a Forgejo: run the case's exact argument vector with the machine's git in a scratch fixture, under `invocation::SETTINGS` and the tool's environment, write the four files by hand with the README's masks (`/scratch`, port `3000`, ids by the map), leave `facts.txt` alone, and say so in the report. The next recapture records them through `cases.rs` either way.
- **The fixture's URL.** The hosted fixture's remote isn't the transcript's (`fixture/specimen`), so `origin-matches` doesn't go in `CASES`: its own test compares the fake's stdout with `scenario.remote` and a newline, and its exit with the transcript's.
- **A referee or fixture in `temper/tests/worker/protocol-git`** that lists operations takes `Look` as `standalone-ancestry` does, with `identity = false`.

## Steps

1. **Vocabulary**, as `notes/contracts.md` has it. In `git.rs`: `Op::Look`, `Done::Looked`, `Kind::Look`; `is_remote` false. In `facts.rs`: `Ending::Looked`, and `Cached::Looking`, worded: a new one, with the switch on and no repository starting saved; what its directory holds, perhaps from before a restart, is looked at first, and it is made again, empty, unless that is the spec's repositories. Let the compiler list the arms, using the files named above.

2. **The domain.** `Cache::hold`'s room branch tells `Cached::Looking` when `self.reuse && !starts_saved(repositories)`, and `Cached::New` otherwise. `hold.rs`: `PrepareStep::Look { repository }`; `Cached::Looking => PrepareStep::Look { repository: 0 }` in `prepare`'s `first`; in `prepared`:

   ```rust
   PrepareStep::Look { repository } => match done {
       Done::Looked { holds: true } if repository < last => Next::Step(PrepareStep::Look { repository: after(repository) }),
       Done::Looked { holds: true } => {
           cache.cloned(holding.workspace, &holding.repositories);
           Next::Step(PrepareStep::Fetch { repository: 0 })
       }
       Done::Looked { holds: false } => Next::Step(PrepareStep::Make),
       Done::Failed { fault: _ } | Done::FailedWithOutput { fault: _, .. } => Next::Failed(Failure::Transient),
       // every other terminal: unreachable!("io ends a look with its own terminals")
   },
   ```

   In `prepare_step`: `Op::Look { at: place(workspace, repository), remote: copy_of(&repository.remote) }`. The module's table and words say what is true now. Step tests: each acceptance line above, in `tests.rs`, on `Harness::reusing` and `Harness::new`.

3. **The component.**
   - `invocation.rs`: `Command::Origin { directory }`, with the argument vector of `notes/contracts.md`. `environment`: `Kind::Look` beside `Kind::IsAncestor`, no identity. `output::classify`: the same.
   - `sequence.rs`: `first` gives `Command::Origin { directory: invocation::place(at) }`. `after`: for `Command::Origin` under `git::Op::Look { remote, .. }`, code 0 ends `Looked { holds: stdout.strip_suffix(b"\n") == Some(remote) }`, codes 1 and 128 end `Looked { holds: false }`, and anything else falls through to the failure below. With no git on the machine the synthetic exit 128 ends `holds: false`, so the make and the clone go on to fail as they do today.
   - `operation.rs`: `Op::Look`'s spawn directory is `b""`, as `Op::Make`'s.
   - `files.rs`: `Directories` gains `keep: bool`. In `layout`, `Layout::OpenTrash` goes to `(Layout::Home, rename home)` when `dirs.keep`, and `Layout::NewWorkspaces` allows `Err(Error::Exists)` only when `dirs.keep`.
   - `component.rs`: `keep_workspaces`, asserting `self.start == Start::Idle`, and `keeps_workspaces`.
   - Tests: `tests/invocation.rs` holds the exact vector and an environment with no header, and `place`'s bytes for tokens 0 and `1 << 32`; `tests/sequence.rs` holds each exit; `tests/files.rs` holds the kept start's file operations, with `Exists` answered to `workspaces`, and the unset start refusing `Exists`.

4. **The fake git.** `check.rs`: `parse` takes `[git_dir, b"config", b"--local", b"--get", b"remote.origin.url"]`, stripping `--git-dir=` and `/.git`. `answer.rs`: `Command::Origin` reads `<directory>/.git/temper-remote`, which a clone writes: there, exit 0 with its bytes and a newline; `.git` there without it, exit 1, empty; no `.git`, exit 128, `fatal: --local can only be used inside a git repository`. `Command::CheckOut` first checks `.git/index.lock`: there, exit 128, `fatal: Unable to create '<path>/.git/index.lock': File exists.`, nothing changed. `program.rs` and the rest: arms. `hosted.rs`'s `spawn_of` gives `Command::Origin` the kind `Look` and the directory `b""`.

5. **The transcripts' readers.** `temper-fake-git-hosted/tests/suite/transcripts.rs`: `origin-no-repository`, `origin-no-remote` and `checkout-index-locked` in `CASES`, with `answer_case` arms in `hosted.rs`, and `origin-matches` per the option above. `temper/tests/worker/protocol-git/tests/suite/transcripts.rs`: `origin-matches`' stdout is the URL in `clone`'s `argv` and a newline, which is the proof of what a clone writes; `sequence::after` on each transcript gives its `Looked`, another remote giving `holds: false`; `checkout-index-locked` exits 128 and classifies `Broken`. `tests/worker/protocol-git/src/fixtures.rs`: a `look` scenario beside `standalone-ancestry`, so that the deadline and the cancel run at its invocation.

6. **The capture.** `temper/tools/git-transcripts/src/cases.rs`: a `looks(&f, c)` after `pushes`, before the rename, so that the masks of what exists keep their numbers:
   - `origin-matches`: from `<scratch>/worker`, `--git-dir=workspaces/1/specimen/.git config --local --get remote.origin.url`; the tool errs unless stdout is `f.remote("specimen")` and a newline;
   - `origin-no-repository`: the same, `workspaces/1/absent`;
   - `origin-no-remote`: the same, on a `git init`'d `workspaces/1/bare`, the init unrecorded;
   - `checkout-index-locked`: an empty `.git/index.lock` in `f.repo`, `checkout --force --detach <base>` from `f.repo`, the lock removed after.

   Add `clone-writes-the-url-as-given` to `facts`, as the other two are. The README's count goes from 42 to 46. Run, from the repository's root: `heavy cargo run -p git-transcripts -- --git "$(command -v git)" --scratch target/git-transcripts --out temper/tools/git-transcripts`. Then `git status` shows what it changed.

7. **The world.** `translate.rs`: `Op::Clone` writes `.git/temper-remote` as `temper-fake-git/src/answer.rs` does; `Op::Look` compares it with the remote, `identity` and `remote` giving `None`; `Op::CheckOut` fails `Fault::Broken` if `.git/index.lock` is there, before `git::check_out`. `world.rs`: `Told::looking` in `tell` and `assert_told`; `World::restart` and `World::write_disk`, as `notes/contracts.md` has them. `restart` asserts that no client holds anything, and keeps `told`, `stats` and `touched`.

8. **The stories**, in `tests/suite/reuse.rs`, each run set and unset, as `evict-keeps`' are. `run` to the end, `restart`, then `submit` at `world.now()` and later.
   - `a_restart_reuses_the_clone_and_a_changed_file_is_the_only_one_touched`: one workspace, `r0`; client 0; restart; `advance_at`; client 1 of the same workstream. Set: `told.looking == 2`, `(makes, clones) == (1, 1)`, `touched` exactly `{THEIRS}`. Unset: `told.new == 2`, `(2, 2)`, `touched` every file. Both ready, on the same token.
   - `a_restart_with_another_remote_makes_it_again`: a client of a workstream naming another repository, after the restart. Set: looked, then made: `makes` and `clones` each rise by one. Unset: made.
   - `a_damaged_clone_heals`: after the restart, `write_disk` of `ws/0/r0/.git/index.lock`, then two clients of the workstream. Set: the first `Prepared::Failed { failure: Failure::Transient }`, the second tells `rebuilt`, is ready, and the lock is gone. Unset: the first is ready, `ws` having gone.
   - `what_an_earlier_worker_left_is_made_again`: `leftovers: 1000`, set: a look that doesn't hold, `made_over == 1`, ready.
   - `a_restart_keeps_the_paths`: two workspaces, `r0` in both; two workstreams; restart; the same two in the other order. Tokens 0 and `1 << 32` in the order made, set and unset, and set, `told.kept == 0` and no make after the restart.
   - `a_noisy_run_across_restarts_holds_the_referees`: a few seeds of `noisy`, with `reuse: true`, a restart between two runs. A seed that breaks a referee is a finding: read it before changing anything.

9. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-worker-domain-checkout -p temper-worker-protocol-git -p temper-fake-git -p temper-fake-git-hosted -p temper-worker-checkout-world -p temper-worker-protocol-git-world -p git-transcripts -p temper-service --all-targets -- -D warnings`.

10. Tests, one command at a time, each package in turn: `heavy cargo nextest run -p` each of `temper-worker-domain-checkout`, `temper-worker-protocol-git`, `temper-fake-git`, `temper-fake-git-hosted`, `temper-worker-checkout-world`, `temper-worker-protocol-git-world`, `git-transcripts`, `temper-worker-domain`, `temper-worker-domain-world`; `heavy cargo nextest run -p temper-service --lib -E 'test(~workspace_reuse) or test(~worker_git)'`; `heavy cargo check -p temper --tests`; `heavy cargo check -p temper-system-world --tests`. Break each rule once by hand, as your text says: the look's remote comparison, the `!starts_saved`, the kept layout, the lock check in the world.

11. Commit and push.

## Report

Your text's, and: how the transcripts were captured and which option you took; each story's numbers set and unset; what `git status` showed after the capture; and anything you learned that makes either status entry untrue.
