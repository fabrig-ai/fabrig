# process-threads: `process-temper` at three threads

Request: `inputs/request.md`. Design: `temper/docs/design/domain/testing.md`, 2.6, **Three at a time**, which already says three and needs no edit. The cut runs nothing, and neither does the plan in the slow tiers: the next after-landing run on main proves it.

## Parts and tasks

- **`threads`**: one task, `three-threads`, as the request sizes it. Needs nothing. Effort medium, about 15 minutes.

The critical path is that task.

## What the reading found

The request's "the comment above the profile" doesn't exist on this base: nothing in `.config/nextest.toml` says the tier is serial but the `test-threads = 1` itself. `testing.md` doesn't either. What does say it is `testing.status.md`, 2.6's first **Not yet**, and `temper/docs/design/STATUS.md`'s summary of it. Both are in the task. The task also adds one comment line citing 2.6, since every other tier profile reads `test-threads = 1`.

**One story relies on running alone, and the fix is small.** `fake_backends::walk::status::a_plans_project_has_its_brief_on_a_return_past_the_sitting` runs a ChatGPT `account connect` on the fake box, whose registration fixes the callback at 1455. A fixed binding has no fallback (`skein-oauth-accounts`, `listener.rs`, `Binding::Fixed`). The `temper-sign-in-callback` group exists for exactly this, but its filter names only `fake_backends::walk::sign_in::`. So this walk could fail against a sign-in walk beside it. The window is short, from the connect to its callback, which would explain why the one trial at three didn't meet it. The task names the walk in the filter. This is from reading: no run shows it.

Read and found safe, each across `process.rs`, `support/`, `fake_backends/`, `suite/box_kinds.rs` and `suite/startover.rs`:

- **Scratch directories:** `Scratch::at` removes any earlier directory of the same case name, so two stories sharing a name would delete each other's. The 60 literal names at its entry points are all different, and the three built ones carry the process id.
- **The other `account connect` stories:** they go through `support::sign_in::patch_configuration`, which removes `redirect_port`, so they take a varying port.
- **Init's preferred ports (8367 to 8371):** every story's own `temper init` goes on to `own_listener_files` (`process-ports`). A story's first-run `temper box`, such as `a_killed_serve_keeps_its_port_and_prints_a_fresh_link`, keeps the sockets its init chose and takes another port if one is taken, so it lets nothing go for another story to take (`temper/docs/design/protocol/setup.md`, 7.1).
- **`/proc` and signals:** every scan and signal is within the story's own child or tree (`observing_test`, `tree_status`, `signal_tree`); none looks across the machine.
- **`flock`:** the locks are in the story's own store.
- **Forgejo and its runner:** the pinned binaries are read, or copied into the story's own cache. `HOME` is the story's own.

Not reliance on running alone, but what after-landing may flag first:

- **The hung worker's margin:** `a_worker_hung_past_the_grace_is_fenced_and_links_again` has its re-hello reach the engine inside a 3000 ms fleet grace, called a margin in its comment, and more stories at once narrow it. It wasn't among the trial's two failures. Left as it is: widening it now would be a guess with no run behind it.
- **The CPU budget:** four of the seven whole runs on record already read over the tier's 360 s, and the one sample at three read 12 tests over 25% more CPU than at one thread. Over budget is red, and it's the user's figure. The plan doesn't touch it, as the request says.

## What was decided

- **The filter names one walk, not `walk::status::`:** the rest of that module doesn't sign in, and serializing it would give back wall time for nothing.
- **The task edits the status files**, a stated departure from the implementer's text, with the words given in its spec. The entry is true until the change lands, so it moves with it (`process-ports` did the same).
- **The status keeps one 2.6 line:** no whole run at three has passed until after-landing's. `testing.status.md` records it in place of `test-speed`'s three whole runs and a loaded one, which this request replaces. It stops being true at the first passing run, so whoever reads that run takes it out.
- **The filter is held by listing:** a `nextest list` of the binary, which runs no story, shows four names. A wrong name in `test(=...)` would otherwise select nothing, silently.

## Overlaps

- `temper/docs/design/STATUS.md`: `ui-proposal-card/card-close` and `model-check/trial-close` also change it. The order can't avoid this with one task. The edit is one sentence, and the spec says what to do if a merge has moved it.
- `.config/nextest.toml` and `testing.status.md`: no draft beside this one lists them.
