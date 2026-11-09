# delivery-checks: status

2026-11-09. Where the design lags or departs from [delivery-checks.md](delivery-checks.md).

- **Design lags:**
  - 3: smith's `smith/docs/design/domain/run.md` (section 8), its
    `domain/tools.md` and `domain/host.md`, jig's
    `jig/docs/design/domain/hosts.md` (section 8) and temper's
    `../domain/agent.md` (4.3 and section 6) say nothing of `save`, the
    owner's choice of 2026-11-09: a tool that pushes the tree to the
    change's own branch at any time with no checks, marked unchecked, used
    before a question, while `deliver` and a `finish` that delivers run the
    full check and only a checked tree is finished and merged into the
    goal's branch.
  - The draft's opening paragraph: `../protocol/configuration.md`, 13.9's
    charter 2 row still has the coding agent run the changed packages'
    lints and focused tests itself. The charters' words are generated
    elsewhere, not in this plan.
- **Design departs:** none.
- **Open:** 3: whether `save` may change smith's charter (a `save` grant)
  and channel (a `Save` ask) formats, the unchecked mark kept in the
  save's commit and receipt rather than in temper's store (asked of the
  owner 2026-11-09, as the draft asks of a protocol change).
