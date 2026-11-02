# playground-reset: status

2026-11-02. Where the design lags or departs from [playground-reset.md](playground-reset.md).

The draft is adopted (the user, 2026-11-02). The code is built ahead of the design (`temper-protocol-store/src/records/forges.rs`, `temper-engine-domain-forge/src/removal.rs`, `temper-service/src/forge/gone.rs`, and about 120 files that cite this draft). Each design document below records the gap as **Departs**, because forges are store records and the playground is reset. The draft's close-out carries it into these documents and then removes it. That close-out comes first after this pass, before the reference drafts execute.

- **Design lags, by draft section:**
  - 1, 3: forges are still described as part of the configuration document, not as records in the store.
    - `../protocol/configuration.md`, section 3, names `forges` and each identity's `forge`.
    - `../protocol/store.md` keeps forges' records only in jig's secrets' store, with no family of temper's.
    - `../protocol/setup.md`, 8.3: `temper init` writes forges 1 to 3 into the default document.
    - `../protocol/setup.md`, 7.1: the box builds its groups from the document's forges, not by kind.
  - 4: no design has a forge's removal, the kept acts that follow it, or its group stopped and its directory removed.
    - `../domain/forge.md`, 4.1 ends the playground's forge alone, and nothing designs `RemoveForge`.
    - `../protocol/setup.md`, 7.1: the pipe has no `remove` for a group.
    - `../client/forge.md` has no Remove on a forge's row.
    - skein's `forge-connection.md` and `forge-setup.md` design no forge added or removed while the component runs.
  - 5: jig's `domain/people.md` has no `RemoveProject`. jig's `domain/tasks.md` erases an ended task only by its retention, never at once with its project.
  - 6.1, 6.2: the design still has the demo's start and its end at the first connected forge, where the playground is made when chosen (`StartPlayground`) and reset by `EndPlayground`.
    - `../protocol/setup.md`, 8.4.
    - `../client/README.md`, 4.8, with `StartDemo` and its words.
    - `../playground.md`.
  - 6.3: `../client/README.md`, 3.5 does not say that the playground is offered only while no real forge is kept. The welcome's Forgejo card opens its Details ("Set up a Forgejo here") before the set up, which no design document draws.
  - 6.4: `../protocol/setup.md`, 6.7 holds a set up's intent in memory, so a restart loses it. The draft instead removes an unfinished forge and sets it up again.
- **Design lags, by design document and section,** each recorded there as Departs:
  - `../protocol/setup.md` 3, 5, 5.5, 6.4, 6.5, 6.7, 7.1, 7.3, 7.4, 8, 8.1, 8.3, 8.4 and 13.4;
  - `../protocol/configuration.md` 2, 3, 4, 6 and 8;
  - `../protocol/forge.md` 2, 2.1, 2.6, 2.8, 7 and 8.1;
  - `../protocol/store.md` 2 and 4;
  - `../protocol/credentials.md` 2.1, 4 and 5;
  - `../protocol/channel.md` 4, 6, 7 and 8;
  - `../protocol/sign-in.md` 9.7;
  - `../protocol/README.md` 4;
  - `../domain/forge.md` 4.1;
  - `../playground.md` 1, 2, 4 and 5.1;
  - `../client/README.md` 1, 3.3, 3.5, 4.6, 4.8, 5.1, 5.5, 5.6, 8 and 11;
  - `../client/forge.md` 1, 5.4, 9 and 10;
  - `../client/tools.md` 5.5 (area 6) and 10;
  - `../client/browser.md` 7.2;
  - `../client/journeys.md` 9;
  - jig's `domain/people.md` and `domain/tasks.md`;
  - skein's `forge-connection.md` and `forge-setup.md`.
