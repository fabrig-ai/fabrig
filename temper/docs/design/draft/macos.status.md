# macos: status

2026-11-05. Where the design lags or departs from [macos.md](macos.md).

skein carries section 3: the readiness backend on epoll and kqueue (skein's `runtime/kernel.md`; `skein/docs/design/runtime/readiness.md`, 5). `macos.md`, section 6's steps 1 to 5 stand in the design: skein's `forge/README.md`, 4.3, `../protocol/setup/owned.md`, 1 and 6, and `../testing/README.md`, 2.8 and `../testing/real-loop.md`'s story, this draft's acceptance, passes on the runner.

- **Design lags:**
  - 4.1: smith's design says nothing of finding `rg` and `git` rather than taking `/usr/bin/rg` and `/usr/bin/git`, or of its tests' addresses and `/proc` uses. The code keeps both paths, though section 6's step 4 names smith's tools: the story's scripts reach no `rg` and macOS has `/usr/bin/git`, so the cut of `macos-cheap` left them, a small follow-up if a run shows otherwise.
  - 4.3: skein's `runtime/shell.md`, section 8 keeps the throwaway Forgejo Linux's, saying macOS has no owned Forgejo, until section 6's step 6 takes it off the list.
  - 4.3: no design says that errors print errno's names, and temper's print numbers (`temper/crates/temper-box-shell/src/box_command.rs`). Section 6's first steps leave it out, and the story doesn't need it.
  - 4.4: `../protocol/configuration/README.md` does not say that the XDG directories hold on macOS, or that case-only differences in a checkout's names are refused.
  - 5: no `macos` nextest profile exists, and no design lists smith's or jig's Linux-only tests by name; `../testing/README.md`, 2.8 has the workflow compile every target and run the one story, what the first stages ask of it, and its section 11 defers the rest of 2.1, 2.2 and 2.3.
- **Design departs:**
  - 4.3: `../protocol/setup/box.md`, 2, and `../protocol/setup/init.md`, 2: the pipe check stays Linux's and asks the shell nothing, and macOS's login is what `/usr/bin/id -un` prints, where the draft has the shell's `getpwuid` answer on both. It needs no change to skein and leaves Linux as it is, by the user's rule of 2026-11-04; the cut of `macos-cheap` chose it.
