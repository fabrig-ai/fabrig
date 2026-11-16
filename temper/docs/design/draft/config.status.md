# config: status

2026-11-16. Where the design lags or departs from [config.md](config.md).

- **Design lags:**
  - 1, 3, 4: `../protocol/configuration/README.md`, sections 1, 2, 3, 6 and 8, give one JSON document, `temper.json`, with jig's three parts at its top level, and every member written out; `../protocol/configuration/engine.md` gives `application`'s members; `../protocol/setup/init.md`, sections 2 and 3, give the documents `init` writes, the box's of 631 lines.
  - 4.5: `../protocol/documentation.md`, section 8, renders `configuration` and `configuration-engine` from the JSON document's decoders, and `limits` and `limits-engine` with `small`'s values.
  - 5: `../protocol/configuration/limits.md` and jig's `protocol/limits.md` give two profiles that a document names, `small` and `standard`, overridden by dotted name in the document; `../protocol/configuration/stock.md` sizes `standard` at five times fabrig's largest figures, below dev.fabrig.ai's in 43 limits.
  - 6.1, 6.2: `../protocol/configuration/settings.md`, section 1, keeps the limits and `engine.llm` in the document, the providers leaving it "by a design of their own, later".
  - 6.3: `../protocol/configuration/figures.md`, section 2, sends as places the fleet's per-worker `slots`, one figure for every worker, until workers are settings (6.7); a worker hosts the smaller of those and its document's `slots` (section 3), and takes its workspaces and git operations from the figures' limits until they're derived from its places (7).
  - 6.4: `../protocol/configuration/engine.md` and `../protocol/forge/engine.md`, section 1, take an owned or fake forge's place from the document's `machine`.
  - 6.7: `../protocol/cluster.md`, section 2, and jig's `domain/engine/fleet.md` make each worker a configured position of the engine's document, admitted by a digest file, so that adding one is a new document and the cluster's restart.
  - 7: the start's checks of `../protocol/configuration/README.md`, section 2, and `../protocol/configuration/limits.md` refuse a document whose coupled figures disagree, where this draft derives them.
- **Design departs:**
  - 6.3: the box keeps the `agents` setting's value in its data directory, `agents.setting`, and its start builds its worker from it (`../protocol/configuration/figures.md`, section 4), since that start builds its worker before its store opens. A change still applies at the box's next start, as at every worker's (N3's designer, 2026-11-16).
  - 8: `upgrades.md`, section 10, has each release read the previous release's configuration members. This draft's builds refuse a member that moved, and dev.fabrig.ai moves by its operators' steps, by the user's word until v0.1.0 has users (`notes.md`, section 21).
- **Open:**
  - 11: a raise taken at once by every child; the memory budget's stock; an override's reason; the webhooks' listener. Each has a default in its question.
