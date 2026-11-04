# macos: status

2026-11-04. Where the design lags or departs from [macos.md](macos.md).

skein carries section 3: the readiness backend on epoll and kqueue (skein's `kernel.md`; `shell.md`, 7.5).

- **Design lags:**
  - 4.1: smith's design says nothing of finding `rg` and `git` rather than taking `/usr/bin/rg` and `/usr/bin/git`, or of its tests' addresses and `/proc` uses.
  - 4.3: `../protocol/setup.md`, 6.6 fetches Linux's releases alone, refuses a redirect, takes a plain binary, and has `mirror` keep the release paths, and has every fetched binary the build the conformance checked. It has neither macOS's bottles, their token, their one redirect, their archive nor their two digests. skein's `shell.md`, section 9 keeps the throwaway Forgejo Linux's, saying macOS has no owned Forgejo. `../protocol/setup.md` has no pipe check without `/proc/self/fd` and no login from `getpwuid`; it says only how a shutdown and a held listener differ on macOS.
  - 4.4: `../protocol/configuration.md` does not say that the XDG directories hold on macOS, or that case-only differences in a checkout's names are refused.
  - 5: no `macos` nextest profile exists, and no design lists smith's or jig's Linux-only tests by name; `../domain/testing.md`, 2.8 has the workflow compile every target and run the one story, what the first stages ask of it.
