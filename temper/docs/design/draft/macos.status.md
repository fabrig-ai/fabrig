# macos: status

2026-11-04. Where the design lags or departs from [macos.md](macos.md).

skein carries section 3: the readiness backend on epoll and kqueue (skein's `kernel.md`; `shell.md`, 7.5).

- **Design lags:**
  - 4.1: smith's design says nothing of finding `rg` and `git` rather than taking `/usr/bin/rg` and `/usr/bin/git`, or of its tests' addresses and `/proc` uses.
  - 2.4: `../domain/testing.md`, 2.6 has the process tier on Linux; no design holds its one story on macOS.
  - 4.3: `../protocol/setup.md`, 6.6 fetches Linux's releases alone, refuses a redirect and takes a plain binary, and skein's `forge.md`, 4.3 pins Linux on amd64 and arm64 alone; neither has macOS's bottles, their token, their one redirect, their archive or their two digests. `../protocol/setup.md` has no pipe check without `/proc/self/fd` and no login from `getpwuid`; it says only how a shutdown and a held listener differ on macOS.
  - 4.4: `../protocol/configuration.md` does not say that the XDG directories hold on macOS, or that case-only differences in a checkout's names are refused.
  - 5: no CI workflow runs on macOS runners (the repository has none), no `macos` nextest profile exists, and no design lists the Linux-only tests by name.
