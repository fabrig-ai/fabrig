# github: status

2026-11-04. Where the design lags or departs from [github.md](github.md).

The design carries parts A, B and C (`../domain/forge.md`, `../protocol/setup.md`, section 17, `../protocol/credentials.md`, `../client/forge.md`, skein's `forge-github.md`), and `../../help/github.md` walks it.

- **Design lags:**
  - 9: `../domain/forge.md` leaves open which checks are CI on GitHub without required checks (section 20), where this draft takes the repository's Actions workflows' check runs, pending skein's `Status` carrying each row's App.
