# macos: status

2026-11-04. Where the design lags or departs from [macos.md](macos.md).

skein carries section 3: the readiness backend on epoll and kqueue (skein's `kernel.md`; `shell.md`, 7.5).

- **Design lags:**
  - 4.1: smith's design says nothing of finding `rg` and `git` rather than taking `/usr/bin/rg` and `/usr/bin/git`, or of its tests' addresses and `/proc` uses.
  - 4.3: skein's `shell.md`, section 9 keeps the throwaway Forgejo Linux's, saying macOS has no owned Forgejo.
  - 4.4: `../protocol/configuration.md` does not say that the XDG directories hold on macOS, or that case-only differences in a checkout's names are refused.
  - 5: no `macos` nextest profile exists, and no design lists smith's or jig's Linux-only tests by name; `../domain/testing.md`, 2.8 has the workflow compile every target and run the one story, what the first stages ask of it.
- **Design departs:**
  - 4.3: `../protocol/setup.md`, 7.2 and 8.2: the pipe check stays Linux's and asks the shell nothing, and macOS's login is what `/usr/bin/id -un` prints, where the draft has the shell's `getpwuid` answer on both. It needs no change to skein and leaves Linux as it is, by the user's rule of 2026-11-04; the cut of `macos-cheap` chose it.
  - 2.4: `../domain/testing.md`, 2.8: the story's `left` needs no code: under Group it reads empty, and the test passes as written.
