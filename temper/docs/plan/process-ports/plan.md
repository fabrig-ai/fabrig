# process-ports: tests can't take each other's ports

The way to `process-temper` at three threads (`temper/docs/design/domain/testing.md`, 2.6). This plan changes where a test's ports come from. It doesn't raise `test-threads`, and it runs nothing in the slow tiers: after-landing proves it.

## Parts and tasks

- **`ports`**: one task, `chosen-ports`, as the request sizes it. Needs nothing. Effort high, about 50 minutes.

The critical path is that task.

## What the survey found

- **The trial's port:** `a_story_panicking_after_its_box_stopped_leaves_the_held_record_at_its_path` gets its ports from `restart_deployment`, then `support::own_listener_files`, then `support::free_addresses`: a bind of port 0, released at once. So 45949 was a kernel draw that nobody held while the box was stopped.
- **Who took it isn't established.** Nothing in the trial's logs says. The likely reading stands: the ephemeral range here reads `32768 60999`, so 45949 was free for any process's outgoing socket or port-0 bind. The change closes that whole class, whichever process it was.
- **The sites:** most binds of port 0 under `temper/crates/temper/tests/` and `temper/tests/` hold their listener from bind to last use, which is safe on any port. The ones that let go and listen again are `support::free_addresses` at its 15 call sites, `suite/fake.rs`' `prepared_forge`, client-browser's `reserved` and demo sockets, real-loop's `clients_listener` and the fake box's nine. The task lists both sets.
- **One deployment had no chosen ports at all:** `forge_land::live_deployment` keeps init's, which fall back to a kernel draw when another box holds 8367, and the live story's sittings restart on them. It takes `own_listener_files`, as its process-tier sibling does.
- **The fake box is why the test must choose.** A stopped box holds nothing and is another process, so only the test process can keep its ports across a stop. Its fresh path also takes the kernel's ports in production code (`init`'s `[0; 5]`, `reserve`), and walks start it from empty configuration on purpose, so a source document can't carry them. Hence one process-tier variable, `TEMPER_FAKE_BOX_PORTS`, as `TEMPER_R8_*` already are. Unset, as for a person, nothing changes.

## What was decided

**A block per process, claimed by a listener on the block's first port,** below the ephemeral range. The request's two obvious shapes each stop short of "holds between tiers on one builder":

- **`NEXTEST_TEST_GLOBAL_SLOT`** is unique within one nextest run only: two runs on a builder both have slot 0.
- **A lock file under `target/`** is unique within one checkout only: each builder slot has its own `target/`. A machine-wide directory would need a path that every run shares, and a test's temporary directory isn't promised to be one.

A listening socket has the same scope as the ports it protects, the network namespace: it holds across runs, tiers and checkouts, the kernel frees it when the process ends however it ends, and it needs no file and no cleanup. It is also no more code than the trial bind beside it. `std`'s listeners are close-on-exec, so a box the test starts doesn't inherit it.

- **Where it lives:** `temper_fixtures::ports`. `temper-real-loop-world` already depends on that crate, `temper` and `temper-client-browser-world` take one dependency line, and a branch stays in its tree. Jig's browser tier has the same `reserved` hook and could want the mechanism in skein later: left alone here, the request being temper's.
- **Public interface:** `TEMPER_FAKE_BOX_PORTS` is new, and `prepare` takes an argument. Neither touches a person's box.
- **macOS** has no `/proc`: 49152, its default, is used where the file is absent, since its one story runs `free_addresses` (`testing.md`, 2.8).

## Design written

`temper/docs/design/domain/testing.md`, 2.4 and 2.6; `temper/docs/design/protocol/setup.md`, 7.5; a **Not yet** entry in each status file, which the task removes.

## Overlaps

None early that the order could avoid, with one task:

- **`landing-fixes-2/port-reservation`** (on its trunk, not on main) changes `reserve`'s body and init's retry in `fake_backends.rs` and `init.rs`. The task leaves `reserve`'s body and `init.rs` alone and adds beside them.
- **`ui-repo-link/box-proof`** adds a `mod` line to `temper/crates/temper/tests/support/mod.rs`, and `ui-repo-link/box-sign-in` changes `temper/crates/temper/src/box_command.rs`: not the lines this task changes.
- **`bundle-loading/gzip-serve`** touched `suite/startup.rs`: this task leaves it as it is.
- **`model-catalogue-1/stock-seven`** touches `support/first_repository.rs`, which this task leaves as it is, and `support/forge_land.rs`, where this task adds one line to `live_deployment`.
