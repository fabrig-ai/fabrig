+++
id = "evict-keeps"
part = "evict-keeps"
kind = "feature"
needs = []
files = ["temper/crates/temper-worker-domain-checkout/src/cache.rs", "temper/crates/temper-worker-domain-checkout/src/facts.rs", "temper/crates/temper-worker-domain-checkout/src/domain.rs", "temper/crates/temper-worker-domain-checkout/src/tests.rs", "temper/tests/worker/checkout/src/world.rs", "temper/tests/worker/checkout/tests/suite/reuse.rs", "temper/crates/temper-service/src/tests/workspace_reuse.rs"]
effort = "high"
minutes = 60
+++

# With `TEMPER_WORKSPACE_REUSE` set, eviction keeps a clone the new workstream names

## Goal

Unset, a workspace the cache evicts is made again, empty. Set, a new workstream that finds the least recently used idle workspace holding exactly the repositories its spec names keeps the clone, so `hold.rs` goes straight to fetch and check-out and only what changed is touched. This task also carries the variable from the worker's start down to the cache; `restart-keeps` builds the restart on it.

Acceptance:

- Set: when no workspace is free and the least recently used idle one holds exactly the spec's repositories (`holds_exactly`, by directory and remote), and no repository starts saved, its `disk` stays, the prepare tells `Cached::Kept`, and its first operation is a `Fetch`, never a `Make` or a `Clone`.
- Set: anything else evicts as today, telling `Cached::Evicted` and starting with a `Make`: other repositories, other remotes, a repository more or fewer, a repository starting saved, and a workspace whose disk is `Unknown`. Only the least recently used idle workspace is considered: no search for a better one.
- Set or unset: the evicting workspace keeps its `Id`, so the path `workspaces/<token>/<repository>` stays the same across an eviction. A test holds it.
- Unset: nothing differs from today (`notes/common.md`). Every existing test of the crates below passes with its expected values unchanged.
- The stories below pass, each with the variable set and unset.

## Inputs

- `temper/docs/design/domain/worker.status.md`, the **Departs** entry on 5 and 9, which this cut wrote.
- `temper/docs/design/domain/worker.md`, 5.
- `temper/docs/plan/worker-slots/notes/common.md`.
- `temper/docs/plan/worker-slots/notes/contracts.md`: the switch, how a prepare finds its workspace, and the story's world. Its section on the restart is the next task's.
- The request, for what it asks in full: `temper/docs/plan/worker-slots/inputs/request.md`, change 1.

## Owns and may change

Owns: `temper-worker-domain-checkout`'s `cache.rs`, `facts.rs`, `domain.rs` and `tests.rs`; `temper/tests/worker/checkout/src/world.rs`; the new `temper/tests/worker/checkout/tests/suite/reuse.rs`; the new `temper-service/src/tests/workspace_reuse.rs`.

May also change, and how:

- `temper-worker-domain-checkout/src/hold.rs`: the `first` match in `prepare` takes `Cached::Kept` beside `Cached::Reused`, and the module's words on what a prepare finds. Nothing else.
- `temper-worker-domain/src/domain.rs`: the one forwarding method.
- `temper-service/src/system.rs`: `Service::reuse_workspaces`, beside `configure_program_sources`. `temper-service/src/worker.rs`: `Worker::reuse_workspaces`. `temper-service/src/tests.rs`: the `mod workspace_reuse;` line.
- `temper/crates/temper/src/lib.rs`: `workspace_reuse()` beside `private_root`, and the call once the service is built. `temper/crates/temper/src/worker_alone.rs`: the same call. No other change there, and no new test there: the read is one line, as `TEMPER_R8_WAIT_CHECK`'s is.
- `temper/tests/worker/checkout/tests/suite/main.rs`: the `mod reuse;` line.
- `temper/tests/worker/checkout/src/lib.rs`, `noisy.rs`, `merges.rs`, and `tests/suite/memory.rs`: only what the new `Settings` field and `Cached` variant force.
- `assert_told` in `world.rs` counts `told.kept` among the admitted prepares.

Options, each pre-approved:

- If `Cached` needs an arm in a file not named here, add it (compiler-forced); no question.
- If a story can't be drawn from `Settings` with one seed (say, two workstreams that differ), try seeds until the world's `workstream(place)` shows the precondition, and assert the precondition in the test.
- If `assert_told` or another referee in the world assumes a workspace is made after eviction, teach it `Kept` and say so in the report.

## Steps

1. **The cache.** `Cache` gains `reuse: bool`, false from `with_capacity`. Pull the loop that finds a saved start out of the workstream's own branch into one helper, so that both branches read it. In the eviction branch of `hold`:

   ```rust
   let kept = self.reuse && !starts_saved(repositories) && holds_exactly(&workspace.disk, repositories);
   let disk = if kept { mem::replace(&mut workspace.disk, Disk::Unknown) } else { Disk::Unknown };
   *workspace = Workspace { key: copy_of(&key), disk, used: Use::Held };
   // ...
   Ok((id, if kept { Cached::Kept } else { Cached::Evicted }))
   ```

   The module's comment says what is true: with the switch on, an evicted workspace may keep its clone. `facts.rs` gains `Cached::Kept`, as `notes/contracts.md` words it.

2. **The switch**, from the shell down, as `notes/contracts.md` has it. `temper-service/src/tests/workspace_reuse.rs`, on `fixtures::system::one_process()`: off until asked (`service.worker_domain()`, then `.checkout().reuses_workspaces()`), on once `service.reuse_workspaces()` is called. An engine alone has no worker and the call does nothing.

3. **Step tests** in `temper-worker-domain-checkout/src/tests.rs`, with a `Harness::reusing(limits)` beside `Harness::new`, next to `the_least_recently_used_idle_workspace_is_evicted_when_the_cache_is_full`:
   - kept: the op is a `Fetch`, the fact `Cached::Kept`, the token the evicted workspace's, and `h.domain.workstream` lists the new workstream at once;
   - evicted as today, one test per cause: another remote under the same directory, another directory, one repository more, one fewer, a saved start, and a workspace spoiled by a `Broken` operation;
   - the same scenario on `Harness::new` still tells `Cached::Evicted`, with the same token.

4. **The world.** `Settings::reuse`, `Told::{kept, makes, clones}`, `World::advance_at` (scheduling the existing `Delivery::Advance`) and `World::touched`, as `notes/contracts.md` has them. `World::new` calls `domain.reuse_workspaces()` when `settings.reuse`. `touched` is recorded in `perform`, around `translate::perform`, for `Op::CheckOut`: the symmetric difference of `disk.tree(path)` before and after, less `in_git` paths, by client and repository.

5. **The stories**, `tests/suite/reuse.rs`. Each runs a helper twice, `reuse: true` and `reuse: false`, on `Settings { checkout: Limits { workspaces: 1, ..LIMITS }, repositories: 1, workstreams: 2, reuse, ..Settings::calm(seed) }`, both workstreams naming `r0`. A client is `Plan { start: Some(Pick::Branch), pushes: 0, ..Plan::simple(workstream) }`.
   - `a_new_workstream_reuses_the_evicted_clone_and_a_changed_file_is_the_only_one_touched`: client 0 of workstream 0; `world.advance_at(at(100), 0, b"main")`, which adds `THEIRS`; client 1 of workstream 1 at `at(200)`. Set: `(told.new, told.kept, told.evicted) == (1, 1, 0)`, `(told.makes, told.clones) == (1, 1)`, and `world.touched(client1, b"r0")` is exactly `{THEIRS}`. Unset: `(1, 0, 1)`, `(2, 2)`, and `touched` is every file (`README`, `src/lib.rs`, `THEIRS`). Both are `Prepared::Ready`, and both clients' `workspace` tokens are equal: the path stays.
   - `other_remotes_evict_as_today`: `repositories: 2`, a seed where the workstreams differ. Set and unset alike: `told.kept == 0`, `told.evicted == 1`.
   - `a_saved_start_evicts_as_today`: `Plan { save: true, pushes: 0, .. }` for workstream 0, whose edit lands on its saved branch; workstream 1 keeps the clone when set; workstream 0 again with `Pick::Saved` evicts, `told.evicted` rising by one in both. If a save with no edit leaves no saved branch, give the first client a push's edit; say what you chose.
   - `a_noisy_run_holds_the_referees`: the world's own referees hold over a noisy run with `reuse: true` (`noisy` settings, a few seeds, as `tests/suite/simulation.rs` runs them). If a seeded noisy run with faults breaks a referee, that is a finding: read it before changing anything.

6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-worker-domain-checkout -p temper-worker-domain -p temper-worker-checkout-world -p temper-service -p temper --all-targets -- -D warnings`.

7. Tests, one command at a time: `heavy cargo nextest run -p temper-worker-domain-checkout`; `heavy cargo nextest run -p temper-worker-checkout-world`; `heavy cargo nextest run -p temper-worker-domain --lib`; `heavy cargo nextest run -p temper-service --lib -E 'test(~workspace_reuse)'`; `heavy cargo check -p temper --tests`; `heavy cargo check -p temper-worker-domain-world --tests`; `heavy cargo check -p temper-worker-protocol-git --tests`. Break each rule once by hand, as your text says: drop the `!starts_saved` and the `holds_exactly`, and see their tests fail.

8. Commit and push.

## Report

Your text's, and: what each story's numbers came to with the variable set and unset; whether anything in the world assumed a make after eviction; and anything you learned that makes `temper/docs/design/domain/worker.status.md`'s entry untrue.
