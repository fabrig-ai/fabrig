# proposal-reach: status

2026-11-18. Where the design lags or departs from [proposal-reach.md](proposal-reach.md).

The design carries this draft. Section 2's client form is jig's `client/domain/contract/documents.md`, 1: a party's pending amendment is whole on the client wire, and a party's retained proposal its `PersonProposal` or, where that record can't carry it, a retained proposal under the party's key; the wire's layout holds both, so its version stays. A proposal's own need is jig's `domain/people/README.md`, 5, *Each source keeps its own needs*. Withdrawing is ctl's `client/ctl/areas.md`, area 2, and the web's `client/pages/needs-you.md`, 1, *By its kind*. Section 3's subjects stay where they are.

- **Design lags:** 2: ctl's `proposal decide` of a party's retained batch, withdrawn or of several goals, exits 1 with `project unavailable`, sending nothing. Its retained form (`ProposalHistory` under the party's key) names no project, and the engine answers a retained decision only under the project its record holds. `proposal show` prints its state. A party's retained amendment takes its target task's project. Decided by the goal's coordinator, 2026-11-18: a project on that form would move the client wire's version.
- **Design departs:** 2: ctl's withdrawal is `proposal withdraw PROPOSAL --project PROJECT`. The engine's `Withdraw` names the proposal's project, and a proposal's written form names none, so a withdrawal sent without reading takes the project as `propose` and `task amend` print it. Decided by the goal's designer, 2026-11-18, keeping the draft's send without a read.
- **Open:** none.
