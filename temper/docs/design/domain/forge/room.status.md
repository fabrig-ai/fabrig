# room: status

2026-11-16. Where temper's code lags or departs from [room.md](room.md).

- **Not yet:** 2, 3, 4 and 6: free room with an asked waiter's reservation; the line, with a change's effects and a goal branch's making in it; a change's place on its record; the line across a restart and a cancel; a write over `forge.client.op_bytes` refused before room. A change's effect that finds the outbox full is refused and holds its task (`temper-engine-domain-forge/src/domain.rs`, `emit_change_effect`).
- **Not yet:** 2 and 5: an agent call's and a goal projection's reservations from check to entry, and every writer's check counting them; a call's check counts only the root's staged calls (`temper-engine-domain/src/route/forge_route.rs`, `describe_agent`).
- **Not yet:** 3: release cleanup in the line; a full outbox fails its release (`enqueue_release`).
- **Not yet:** 3: a landing's writes in the line; a full outbox refuses a round's write (`temper-engine-domain-forge/src/landing.rs`, `refuse_entry`).
- **Not yet:** 7: a change's panel carrying its wait, on the client's wire and in its words.
