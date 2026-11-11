# agent-docs: status

2026-11-11. Where the design lags or departs from [agent-docs.md](agent-docs.md).

`../protocol/documentation.md`, `../client/ctl/README.md`, `../client/ctl/tools.md` and `../protocol/binaries.md` build on this draft. They state its readers, map, kinds, placeholders, voice, command line, checks and layout, with the departures below. `../protocol/documentation.md`, 9 still cites section 3 for errors at the point of use; its 2.2 still takes the reference pages' generation from section 8.1.

- **Design departs:**
  - 2, 10: `../domain/agent/agents.md`, 1 and `../protocol/documentation.md`, 2.2 have Ask temper read temper's pages by `see help`, its instructions naming no list of pages, where this draft has the configuring agent read them with `read_docs` and its pages listed: `ask-temper.md`, section 9, the user, 2026-11-07.
  - 5: `../protocol/documentation.md`, 2.1 states the small placeholder set for written pages, but reference pages print their tables' argument names. `../client/ctl/tools.md`, 1 keeps ctl's larger vocabulary of `shown` names, rather than renaming it to the index's set, to preserve the design's explicit vocabulary (the requester, 2026-11-11).
  - 7.1: `../protocol/binaries.md` sorts each binary's commands by reach, across the deployment's binaries, where this draft has one binary in documentation, deployment, clients, spawned and test-doubles groups. Its table-row and internal-command rules are already stated there; the reach labels and several binaries stay as designed (the requester, 2026-11-11).

The help lags the design too: of the map's pages (section 4), the concepts `finding` and `qa`, the walk `upgrade` and the recipes "review before landing" and "a tester that cannot land" are not written (`../protocol/documentation.status.md`).
