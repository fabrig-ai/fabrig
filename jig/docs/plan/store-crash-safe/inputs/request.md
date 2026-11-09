# store-crash-safe: an engine killed mid-write loads its store again

A real deployment's engine ignored SIGTERM twice in one night, and systemd killed it with SIGKILL after 90 seconds each time. The second time, the store it left no longer loaded. The binary that wrote it panicked at start, every time:

```
panicked at jig/crates/jig-domain-core/src/start/links.rs:420:9:
Tasks(Live(199)) links missing People(Person(102))
```

A newer binary refused it as `temper serve: restore forge: store load failed`. Task 199 was a resolution task, and person 102 a person task, both made shortly before. Every goal's state in that store was lost. The store is the deployment's only record of its tasks, so this is data loss. The evidence is outside this repository, so the words here are what you have. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **The failing story first,** at the lowest tier that holds the engine's store and its start: an engine killed at each point of a decision's write, after each record, mid-frame and between families, starts again from what was durable. It loads the last whole decision and never a half of one, with no panic. Rows of the existing restart or crash table, over the simulated disk's crash points. Include a decision that makes a task and the person task it links, as above.
2. **The cause and the fix:** whatever lets a crash leave one family's record without the record it links to, such as a decision written across families without one commit point, a flush order, or a snapshot taken mid-decision. A store that fails a link check at start says which records disagree and stops with a reason, not a panic. Repairing a damaged store isn't asked.
3. **A clean stop:** if the plan `worker-relinks` hasn't landed its SIGTERM item by then, the engine stops within its stop grace on SIGTERM, with the store consistent, so the kill isn't reached. Say which plan landed it.
4. **The design says so:** jig's store design wherever a decision's durability across families and a start's link checks are written.

## Not in this plan

Any figure, and a store repair tool.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test, on the simulated disk; no test reaches a real host. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format change is a question first; formats change in place before v0.1.0. An interface change in skein's `io`, `sim` or `world` goes to hearth/core first. jig never names temper. `walk = false`.
