# jig-domain-api: execution

2026-11-17. How [jig-domain-api.md](jig-domain-api.md) gets carried out: the engine first as a pass-through every root adopts, then port by port, then the clean-up (the draft's section 15). It designs nothing: each milestone writes its own design from the draft into jig's documents and their status as it runs, and lands whole. Plans use the focused and fuzzy suites alone. A milestone's figure is the draft's section 16 counts it lowers, read on main once it lands.

## 1. The milestones

Each is a goal, one change of one concern. A milestone that grows past a day's change splits, and this file says how.

| Milestone | Draft parts | Stream | Needs | Its figure after landing | State |
|---|---|---|---|---|---|
| J1, the engine as a pass-through: `Engine` in `jig-domain-core` over `Core`, its `step`, `fire`, `next_deadline` and `idle` forwarding, `In::Legacy` and `Out::Legacy` carrying today's `Event` and `Request` and nothing else, and a temporary accessor for the methods roots call; the counts of section 16 measured, those a pattern finds added to jig's and temper's old-forms ratchets (`jig/scripts/old-forms.txt`, `temper/scripts/old-forms.txt`), the rest recorded in this file | 4.1, 15, 16 | engine | none | the counts' baseline | not started |
| J2, jig's roots on the engine: `ops`'s root and the test root hold `Engine` in place of `Core`, and the kits and worlds that drive the core (`jig-core-kit`, `jig-conformance`, `jig-test-service`) drive it; no behaviour moves | 15 | roots | J1 | counts 1 and 3 | not started |
| J3, temper's root on the engine: `temper-engine-domain` and the engine service's crates that drive the core hold `Engine`; no behaviour moves | 15 | roots | J1 | counts 1 and 3 | not started |
| J4, the store's loads: one `Load` and its one `Loaded`, paced by the engine; the engine's restart through them, and each connector loading its own records; the ten named loads, their `busy` flags, roots' `restart.rs` and the test root's queue of store answers go | 4.4, 5.1, 18 | engine | J2, J3; the user's word on one order for every root (the draft's section 18), or its default | counts 1, 2, 3 and 8 | not started |
| J5, the contract's crate: `jig-domain-contract`, with the operations' values and the connector contract's types moved down from the core's children and vocabulary and re-exported along their old paths; programming-model 4.5's and 4.6's wording of the draft's 14.1 | 6.1, 14.1 | contract | J1 | none of its own: moves | not started |
| J6, the connector contract: the `Declaration` given at construction and checked, roots' installing of hold kinds gone; one `Ask` of twelve families, one `Up`, `Released`; each root's asks routed with one arm per connector; `ops`'s connectors and the test connector speaking it, temper's as the draft's section 18 says | 6.2 to 6.5, 6.7, 14.1, 18 | engine | J4, J5 | counts 1, 2, 5 and 8 | not started |
| J7, parts and run preparation: `PartRef`; what a run needs in one answer per connector; run preparation inside the engine, its activations paced by it; `assemble` one function in each root | 4.4, 6.6, 10 | engine | J6 | counts 1, 2, 3 and 8 | not started |
| J8, hosts: `host::Down` and `host::Up` in `jig-domain-link`, spoken by the fleet and the hub; the workers' channel encoding them to the records it has today, its wire unchanged; the engine's own slots without translation, and `jig-domain-hub-link` gone | 5.2, 9 | hosts | J7 | counts 1, 2 and 7 | not started |
| J9, decoded at the edge: calls, turns and answers decoded by the engine's protocol layer as they arrive; a connector's call in two parts; fencing after decoding; charters' sizes checked at configuration; a settled call's answer as the draft's section 18 says; the hand-outs and payload tables gone | 8, 14.2, 18 | hosts | J8 | counts 1, 2 and 8 | not started |
| J10, the operations: `Act`, `Actor`, `Op`, one `Read` and one refusal's shape, made by the clients' component, by a run's decoded call and by a procedure's step; `people::Ask`, `NamedAction` and `ProcedureStep` gone; a stored answer whose shape moves brings its version and patch | 7 | engine | J5, J9 | counts 1, 2 and 3 | not started |
| J11, the remaining ports: parties' (5.3), accounts' (5.4), the application's (5.5) and `Resume` (5.6); `Legacy` empty | 5.3 to 5.6 | engine | J10 | count 2 at 0 | not started |
| J12, one class per input: `class` and `idle`; `ops`'s connectors' protocol layers delivering a settlement and its news apart, news compared with what was last reported; `Class::Mixed`, the split and the baselines gone | 4.4, 14.4 | engine | J11 | count 1 | not started |
| J13, the engine is the API: `Legacy` and the accessor gone, the core emitting the ports itself, the old vocabulary deleted | 4.5, 15 | engine | J12 | counts 2 and 3 at 0 | not started |
| J14, private children: `jig-boundary`'s check that no crate but the core, its vocabulary and the children names a child's crate; every other dependent through the core's vocabulary; `jig/docs/design/README.md`, 4's escape hatch gone | 10, 14.3 | inside | J13 | counts 4, 5 and 6 at 0 | not started |
| J15, the core inside: one typed continuation (D002), `Core`'s state grouped by concept (D003), and the re-narrowing arms that go with them (D017) | 10 | inside | J14 | count 10's `Core` | not started |
| J16, a task's record: its common fields and one executor part (D014), a stored format, with its version and patch | 10 | inside | J14 | count 10's `TaskRecord` | not started |
| J17, the conformance kit: it answers jig's ports itself, and an application's adapter is its step, its release, its two functions to and from jig's ports and its systems' fakes (D030) | 12 | kit | J13 | count 9 | not started |
| J18, the reference root: `ops`'s root copied again to the draft's section 11; `jig/docs/design/domain/root.md`, `jig/docs/design/domain/engine/README.md` and `jig/docs/design/domain/connectors.md` read whole against the new edge; D004 closed | 11, 17 | roots | J15, J16, J17 | count 8 | not started |

## 2. Order

- **Each milestone is a goal,** started once the user approves it.
- **No priority mark:** jig's domain API isn't one of v0.1.0's critical pieces. Its plans start when places are free and marked work isn't about to arrive. J3 and every milestone after it change temper's root, so each waits for a quiet time in it.
- **One at a time within the engine, hosts and inside streams,** each from main once the one before it has landed: every port moves the same files of the three roots, so no two branches change them at once.
- **Side by side:** J2 and J3, which touch different trees; J5 beside J4, since its moves keep every old path; J15, J16 and J17 once their needs have landed.
- **The user's word on one order for every root** (the draft's section 18) comes before J4 starts. Without it, J4 takes the default and says so in its design.
- **Each milestone lowers its counts** and none raises one; a count the ratchet holds fails the gate if it rises.
