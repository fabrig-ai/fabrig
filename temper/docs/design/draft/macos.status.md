# macos: status

2026-11-20. Where the design lags or departs from [macos.md](macos.md).

skein carries section 3: the readiness backend on epoll and kqueue (skein's `runtime/kernel.md`; `skein/docs/design/runtime/readiness.md`, 5). `macos.md`, section 6's steps 1 to 5 stand in the design: skein's `forge/dialects.md`, 1.3, `../protocol/setup/owned.md`, 1 and 6, and `../testing/macos.md`, 1 and 2 and `../testing/real-loop.md`'s story, this draft's acceptance, passes on the runner.

- **Design lags:**
  - 4.1: smith's design says nothing of finding `git` rather than taking `/usr/bin/git`, or of its tests' `/proc` uses; its tests' addresses are smith's `testing.md`, 4. The code keeps git's path, though section 6's step 4 names smith's tools: macOS has `/usr/bin/git`, so the cut of `macos-cheap` left it, a small follow-up if a run shows otherwise.
  - 4.3: skein's `runtime/shell.md`, section 8 keeps the throwaway Forgejo Linux's, saying macOS has no owned Forgejo, until section 6's step 6 takes it off the list.
  - 4.3: no design says that errors print errno's names, and temper's print numbers (`temper/crates/temper-box-shell/src/box_command.rs`). Section 6's first steps leave it out, and the story doesn't need it.
  - 4.4: `../protocol/configuration/README.md` does not say that the XDG directories hold on macOS, or that case-only differences in a checkout's names are refused.
  - 5: no design lists smith's or jig's Linux-only tests by name. `../testing/macos.md` has the workflow dispatched after landing for its sets of parts, the suites among them (`macos-tiers.md`); the rest of 2.1, 2.2 and 2.3 comes after landing.
- **Design departs:**
  - 4.1: smith finds `rg` on the commands' `PATH`, but where that environment has no `PATH`, or none of its directories holds `rg`, it still takes `/usr/bin/rg` (smith's `protocol/agent.md`, section 2), where the draft has tools found, not assumed. Many of smith's and temper's configurations, fixtures among them, give their commands no `PATH`, and refusing them would change Linux's runs and its gate. The goal's designer chose it, 2026-11-19; the draft's intent, `rg` found wherever a Mac has it, holds for any configuration that gives its commands a `PATH`.
  - 4.3: `../protocol/setup/box-supervision.md`, 1, and `../protocol/setup/init.md`, 2: the pipe check stays Linux's and asks the shell nothing, and macOS's login is what `/usr/bin/id -un` prints, where the draft has the shell's `getpwuid` answer on both. It needs no change to skein and leaves Linux as it is, by the user's rule of 2026-11-04; the cut of `macos-cheap` chose it.
  - 5.1: `../testing/macos.md`, 6, installs the toolchain the repository's `rust-toolchain.toml` pins, `nightly-2026-10-21` with LLVM, where the draft has Rust 1.95, the workspace's `rust-version`: the later draft `crate-graph.md`, 2, moves the workspace to that nightly, by the user's choice of 2026-11-12.
