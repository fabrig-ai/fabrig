# room: status

2026-11-27. Where temper's code lags or departs from [room.md](room.md).

- **Not yet:** 2.2, *A new entry takes one place*, *An entry leaves once, by its own write* and *Its sums are exact*: keeping a new entry asserts its number is new and the outbox had its place, and keeping one again that it is the same task's; an entry's outcome, withdrawal or settling as not made asserts it is held under its write's task; the room's sums are saturated, not checked (`room.rs`, `reserved` and `free`; `domain.rs`, `keep_entry` and `drain`). [in flight: change 19640]
- **Not yet:** 2.2, *A reservation is made once and used by its holder* and *Each holder against its owner*: an ask, a call's and a projection's reservation are removed without asserting they were held, a call with no reservation falls back to the open room, an asked waiter's check reads the entries alone, and an owner's end gives back its place and reservation unchecked (`room.rs`, `ask`, `leave`, `pass`, `check`; `domain.rs`, `enqueue`, `unreserve_call`, the projection events). [in flight: change 19644]
