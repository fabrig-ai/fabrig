# github: status

2026-11-02. Where the design lags or departs from [github.md](github.md).

The design carries parts A and B and most of C (`../domain/forge.md`, `../protocol/setup.md`, section 17, `../protocol/credentials.md`, `../client/forge.md`, skein's `forge-github.md`), and `../../help/github.md` walks it.

- **Design lags:**
  - 9: `../domain/forge.md` leaves open which checks are CI on GitHub without required checks (section 20), where this draft takes the repository's Actions workflows' check runs, pending skein's `Status` carrying each row's App.
  - 12.2 and 16's step 6: `../protocol/setup.md` keeps the standalone `temper forge reconcile` and its files (6.5), and a managed forge's `{"managed": {"admin_token": PATH}}` until it goes, where part C moves the managed host into the engine and removes the configuration's members.
