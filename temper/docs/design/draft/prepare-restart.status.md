# prepare-restart: status

2026-11-17. Where the design lags or departs from [prepare-restart.md](prepare-restart.md).

- **Design lags:** none.
- **Design departs:** 1 (*Wanted*, its last item): the round's preparing
  and its recovery are their own document,
  `../domain/landing-preparing.md`, which `../domain/landing.md`, 5.2
  (*Lost or partial*) and 15 point to, since `landing.md` stands at the
  design documents' size limit (`DESIGN.md`, 5). Decided by the goal's
  design, 2026-11-17.
- **Design departs:** 3, story 1 (and 1, *Wanted*, a restart "after the
  round's intent is saved but before the prepare is admitted"): the
  marks and the prepare child's admission are one root decision, so no
  restart falls between them (`../domain/landing-preparing.md`, 5). The
  story restarts right after that decision, before any worker claims the
  child's attempt; proven absence is reached where the child ended with
  no answer, which story 4 builds while the landing is stopped (7).
  Decided by the goal, 2026-11-17, on change 6063's finding.
- **Open:** none.
