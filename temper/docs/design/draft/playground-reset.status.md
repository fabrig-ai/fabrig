# playground-reset: status

2026-11-02. Where the design lags or departs from [playground-reset.md](playground-reset.md).

The draft is adopted (the user, 2026-11-02), and the code is built ahead of the design (`temper-protocol-store/src/records/forges.rs`, `temper-engine-domain-forge/src/removal.rs`, `temper-service/src/forge/gone.rs`, and about 120 files citing this draft). Its close-out carries it into the documents below and removes it; it comes first after this pass, before the reference drafts execute.

- **Design lags:**
  - 1, 3: `../protocol/configuration.md` names the forges in the document (section 3: `forges`, and each identity's `forge`), where they are records in the store, made by the acts that set them up or connect them; `../protocol/store.md` keeps forges' records only in jig's secrets' store, with no family of temper's for them; `../protocol/setup.md` 8.3 has `temper init` write forges 1 to 3 into the default document, and 7.1 builds the box's groups from the document's forges, not by kind.
  - 4: `../domain/forge.md` 4.1 ends the playground's forge alone, and no design has `RemoveForge`, a removed forge's kept acts, or its group stopped and its directory removed (`../protocol/setup.md` 7.1 has no `remove` on the pipe for a group); `../client/forge.md` has no Remove on a forge's row. skein's `forge-connection.md` and `forge-setup.md` design no forge added or removed while the component runs.
  - 5: jig's `domain/people.md` has no `RemoveProject`, and jig's `domain/tasks.md` erases an ended task by its retention alone, never at once with its project.
  - 6.1, 6.2: `../protocol/setup.md` 8.4 and `../client/README.md` (4.8, the `StartDemo` request and its words) keep the demo's start and its end at the first connected forge, where the playground is made when chosen (`StartPlayground`) and reset by `EndPlayground`.
  - 6.3: `../client/README.md` 3.5 does not say that the playground is offered only while no real forge is kept.
  - 6.4: `../protocol/setup.md` (6.7) holds a set up's intent in memory, lost at a restart, where an unfinished forge is removed and set up again.
