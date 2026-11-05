# github: status

2026-11-05. Where the design lags or departs from [github.md](github.md).

The design carries parts A, B and C (`../domain/forge.md`, `../protocol/setup.md`, section 17, `../protocol/credentials.md`, `../client/forge.md`, skein's `forge-github.md`), and `../../help/github.md` walks it.

- **Design lags:**
  - 9: `../domain/forge.md` leaves open which checks are CI on GitHub without required checks (section 20), where this draft takes the repository's Actions workflows' check runs, pending skein's `Status` carrying each row's App.
- **Open:**
  - 12.2: a Forgejo connected as its admin has its hook here, at adoption and at each start's repair, where `../protocol/setup.md`, 5.5, serves it no listener, so it polls. The user's, not yet answered (`../protocol/setup.md`, 16).
- **Design departs:**
  - 11: this section keeps a probe account beside `fabrig-ai`, with `temper-probe-2` that account's spare, not granted to the App. `../domain/testing.md`, 2.7 and the managed sitting answer to `fabrig-ai` alone, whose `temper-probe-2` the App's installation covers, because the user's request of 11-04 took the personal account out of the live probes.
