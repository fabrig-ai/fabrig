# outbox-room-wait: status

2026-11-16. Where the design lags or departs from [outbox-room-wait.md](outbox-room-wait.md).

- **Design lags:** none.
- **Design departs:** 1: an agent's forge call (`effect_forge`) does not wait in the line. One that finds no room, or a waiter ahead of it, is answered `Unavailable` at once, with no hold, and the room its check finds is reserved until its entry is kept, so it never meets a full outbox (`../../domain/forge/room.md`, 5). Decided by the user, 2026-11-16: a call waiting durably would need jig to give a waiting effect a new number, a change to its stored call records, for a write its agent can ask again. The draft's "an agent's reopen" does not exist: agents create issues, post comments and set statuses; a change's reopen is the change's and waits (`../../domain/forge/change/README.md`, 2).
- **Design departs:** 1: a goal branch's making keeps its place in the line in memory, as it keeps the making: a restart puts it behind the waiters restored with their records (`../../domain/forge/room.md`, 4). Decided by the goal's designer, 2026-11-16: a making has no record of its own to keep a place on, and waits a goal's start alone.
- **Open:** none.
