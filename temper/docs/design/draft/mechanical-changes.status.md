# mechanical-changes: status

2026-11-15. Where the design lags or departs from [mechanical-changes.md](mechanical-changes.md).

- **Design lags:**
  - 3: `../client/ctl/tools.md`, 3 has no `change open` act.
  - 4: `../domain/forge/change/README.md`, 1 makes a change only from a run's `change` call or the forge itself; no request of a client's is expanded into a change for a party, and no parameter names a source branch to take.
  - 5: `../domain/forge/change/README.md`, 1 and 2 have no taking state before opening, and `../domain/forge/branches.md` makes a change's own branch only by its producer's push.
  - 9: `../protocol/store.md`, 2 keeps no source or taking state on a change; the client's wire has no request to open one.
- **Design departs:** none.
- **Open:** none.
