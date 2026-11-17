# config: status

2026-11-16. Where the design lags or departs from [config.md](config.md).

- **Design lags:**
  - 1, 3, 4: `../protocol/configuration/README.md`, sections 1, 2, 3, 6 and 8, give one JSON document, `temper.json`, with jig's three parts at its top level, and every member written out; `../protocol/configuration/engine.md` gives `application`'s members; `../protocol/setup/init.md`, sections 2 and 3, give the documents `init` writes, the box's of 631 lines.
  - 4.5: `../protocol/documentation.md`, section 8, renders `configuration` and `configuration-engine` from the JSON document's decoders, and `limits` and `limits-engine` with `small`'s values.
  - 5: `../protocol/configuration/limits.md` and jig's `protocol/limits.md` give two profiles that a document names, `small` and `standard`, overridden by dotted name in the document; `../protocol/configuration/stock.md` sizes `standard` at five times fabrig's largest figures, below dev.fabrig.ai's in 43 limits.
  - 6.1, 6.2: `../protocol/configuration/settings.md`, section 1, keeps the limits and `engine.llm` in the document, the providers leaving it "by a design of their own, later".
  - 6.3: `../protocol/configuration/README.md`, section 4, has a person write smith's agent configuration, and smith's `protocol/agent.md`, section 4 reads it from a file a worker names.
  - 7: the start's checks of `../protocol/configuration/README.md`, section 2, and `../protocol/configuration/limits.md` refuse a document whose coupled figures disagree, where this draft derives them.
- **Design departs:**
  - 6.7: a new worker takes the lowest number no worker setting holds, chosen by the client that adds it, where a model takes the highest free number the core chooses; the core checks that a number keeps its name, so no number goes to another name, as this draft wants, and a lowered `configured_workers` stays above the numbers held (jig's `domain/engine/workers.md`, 2; goal 5136's design, 2026-11-16).
  - 6.7: until the TOML file (4.2), a box's document keeps its worker part, now named `local`, and its `slots`; the box's worker is the setting `local` all the same (`../protocol/cluster.md`, 1; goal 5136's design, 2026-11-16).
  - 6.4: an owned or fake forge's port is always chosen at set-up and its binaries fetched: no person gives others at set-up (`../protocol/setup/place.md`, 1), because nobody sets them but by editing the document today, and a field in `SetUp` would move the client's wire for it (the user, 2026-11-16). A host's own units give theirs to `forge prepare` once (`../protocol/setup/place.md`, 4).
  - 6.4: `engine.clients.provider` stays the document's: a managed Forgejo's sign-in through temper names it, and section 4's file has no key for it; only the owned forge's client is made with the forge (`../protocol/configuration/moved.md`, section 6; `../protocol/setup/place.md`, 6). Decided by goal 5135's design, 2026-11-16, so that dev.fabrig.ai's sign-in to its Forgejo keeps working.
  - 8: `upgrades.md`, section 10, has each release read the previous release's configuration members. This draft's builds refuse a member that moved, and dev.fabrig.ai moves by its operators' steps, by the user's word until v0.1.0 has users (`notes.md`, section 21).
- **Open:**
  - 11: a raise taken at once by every child; the memory budget's stock; an override's reason; the webhooks' listener. Each has a default in its question.
