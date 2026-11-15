# mechanical-changes: status

2026-11-15. Where the design lags or departs from [mechanical-changes.md](mechanical-changes.md).

- **Design lags:**
  - 3: `../client/ctl/tools.md`, 3 has no `change open` act.
  - 4: `../domain/forge/change/README.md`, 1 makes a change only from a run's `change` call or the forge itself, and names no branch to take as a parameter; nothing says what the forge fills in for a change a party makes in its own batch.
  - 5: `../domain/forge/change/README.md`, 1 and 2 have no producer that takes a pushed branch's head onto the change's own branch; `../domain/forge/branches.md` makes a change's own branch only by its producer's push.
- **Design departs:** none.
- **Open:** none.
