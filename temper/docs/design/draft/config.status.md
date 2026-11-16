# config: status

2026-11-16. Where the design lags or departs from [config.md](config.md).

- **Design lags:**
  - 1, 3, 4: `../protocol/configuration/README.md`, sections 1, 2, 3, 6 and 8, give one JSON document, `temper.json`, with jig's three parts at its top level, and every member written out; `../protocol/configuration/engine.md` gives `application`'s members; `../protocol/setup/init.md`, sections 2 and 3, give the documents `init` writes, the box's of 631 lines.
  - 4.5: `../protocol/documentation.md`, section 8, renders `configuration` and `configuration-engine` from the JSON document's decoders, and `limits` and `limits-engine` with `small`'s values.
  - 5: `../protocol/configuration/limits.md` and jig's `protocol/limits.md` give one stock, but a document still overrides it by dotted name in its `limits`, and the counts are constants, not formulas of `live_goals`; `memory_bytes` is still the document's (5.3).
  - 6.1, 6.2: `../protocol/configuration/settings.md`, section 1, keeps the limits and `engine.llm` in the document, the providers leaving it "by a design of their own, later".
  - 6.3: `../protocol/configuration/README.md`, section 4, has a person write smith's agent configuration, and smith's `protocol/agent.md`, section 4 reads it from a file a worker names.
  - 6.4: `../protocol/configuration/engine.md` and `../protocol/forge/engine.md`, section 1, take an owned or fake forge's place from the document's `machine`.
  - 6.7: `../protocol/cluster.md`, section 2, and jig's `domain/engine/fleet.md` make each worker a configured position of the engine's document, admitted by a digest file, so that adding one is a new document and the cluster's restart.
  - 7: `../protocol/configuration/derived.md` derives the turn's, the payload's and the clients' carriers and a worker's places, but not the graces, and not the engine's limits on workers from the workers' settings.
- **Design departs:**
  - 5.2: the stock agent's tool payload is 63,740, dev's, not 64 KiB, since skein's rule, (payload + 260) × 6 ≤ output × 8, admits no more at the stock's output of 48,000 (`../protocol/configuration/derived.md`, section 3). Decided by N1's design, 2026-11-16.
  - 5.2: the stock's `fleet.slots` is below twice dev.fabrig.ai's 36 (80): the engine's link to a worker caps its bytes at a u32, slots × (a slot's assignment + message frame × inbox), near 61 MB a slot at jig's stock (39,671,967 + 263,201 × 82), so 4,294,967,295 holds about 70 slots, and fewer at temper's derived figures. The ceiling is the largest round figure at most 80 the cap holds, never below 36 (`../protocol/configuration/derived.md`, 5.3). Widening skein's channel cap to u64 is not N1's. Decided by the user, 2026-11-16. The cap holds 64 at jig's stock alone and 48 at the figures temper's engine starts with (init's documents' `hub.transcript_bytes` of 67,215,250: 4,231,467,716 bytes at 48, past the u32 at 50), so the stock's is 48, keeping the generated deployment startable on the goal's branch (2026-11-16).
  - 7: a formula's figure passes today's check at the stock, and at twice it only where the wires and the store take twice: jig's worker wire caps a turn's body at 33,554,432, below twice the stock's T (36,863,684), and the store's transcript ceiling likewise. There the test checks the arithmetic at twice, today's check at the largest T the ceiling takes, and the start's refusal at twice (`../protocol/configuration/derived.md`, section 6). Raising a ceiling moves its wire, which isn't N1's. Decided by N1's goal, 2026-11-16.
  - 8: `upgrades.md`, section 10, has each release read the previous release's configuration members. This draft's builds refuse a member that moved, and dev.fabrig.ai moves by its operators' steps, by the user's word until v0.1.0 has users (`notes.md`, section 21).
- **Open:**
  - 11: a raise taken at once by every child; the memory budget's stock; an override's reason; the webhooks' listener. Each has a default in its question.
