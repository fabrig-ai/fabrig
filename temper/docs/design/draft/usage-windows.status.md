# usage-windows: status

2026-11-19. Where the design lags or departs from [usage-windows.md](usage-windows.md).

- **Design departs:**
  - 2 and 5: a report equal to the fact held writes nothing, but may keep a reading, and so move its window's rate, which marks the accounts documents at most once every 5 minutes for a window (jig's `domain/accounts.md`, 3.7, *A window's rate*). Section 2 says such a report marks nothing, and section 5 that the rate moves on as readings come: a rate kept from changed shares alone would hold a stopped window's pace. Decided by the goal's design, 2026-11-19, keeping section 2's intent that a busy account doesn't churn its readers.
  - 3: a usage read gives each window's share, and not its reset: `/usage` words a reset in the host's local time, so the reset a run reported stands (smith's `protocol/programs.md`, 5.11). Decided by the goal's design, 2026-11-19, until a recording of `/usage` says otherwise.
- **Open:** 7, each question with its default; and how Claude Code's `/usage` answers without its screen (smith's `protocol/programs.md`, 9, *The usage command*).
