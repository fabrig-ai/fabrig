# macos: status

2026-11-02. Where the design lags or departs from [macos.md](macos.md).

skein carries section 3: the readiness backend on epoll and kqueue (skein's `kernel.md`; `shell.md`, 7.5).

- **Design lags:**
  - 4.1: smith's design says nothing of finding `rg` and `git` rather than taking `/usr/bin/rg` and `/usr/bin/git`, or of its tests' addresses and `/proc` uses.
  - 4.3: `../protocol/setup.md` has no owned Forgejo refused by platform, no pipe check without `/proc/self/fd`, and no login from `getpwuid`; it says only how a shutdown and a held listener differ on macOS.
  - 4.4: `../protocol/configuration.md` does not say that the XDG directories hold on macOS, or that case-only differences in a checkout's names are refused.
  - 5: no CI workflow runs on macOS runners (the repository has none), and no design lists the Linux-only tests by name.
