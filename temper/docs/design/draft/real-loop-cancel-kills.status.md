# real-loop-cancel-kills: status

2026-11-16. Where the design lags or departs from [real-loop-cancel-kills.md](real-loop-cancel-kills.md).

The design carries this draft. Section 1 is `../testing/real-loop.md`'s
*A cancel's kill is asked for*: the world pairs each `Cancelled` kill a
root requests with the attempts that root noted `Cancelling` at the
latest instant at or before it, and accepts it when each of their tasks
is one the story cancelled, or a delegate below one, cancelled at or
before the kill. The worker's `Cancelling` note
(`../protocol/services.md`, 5) is the observation it pairs with: no note
named a cancelled attempt before, and an agent run's cancelled
preparation took none at all. Section 2's order is
`../testing/reference-case.md`'s row for `fakes.md`, 1: ws alone
started, s1 added once the director's run is placed on ws.

- **Design lags:** none.
- **Design departs:**
  - 1: "a task the story cancelled" is one whose cancellation a person
    decided, or a task whose executor is an agent: case_5_4's cancels are
    its scripted agents' `cancel` tools, not steps of its own, and the
    deployment's, a procedure's or a job's cancel is the service's
    decision, not the story's. "After the story's cancel step" is the
    cancellation first seen at or before the kill (the goal's designer,
    2026-11-16).
  - 1: a `Cancelled` kill a hosted child requests, such as an agent's
    kill of its own tool's process when its run is cancelled, is not
    paired and still needs a `Kill` step: its record names the agent as
    requester, nothing the world keeps maps an agent's process to its task
    in both of the world's shapes, and no story has met one (the goal's
    designer, 2026-11-16).
- **Open:** none.
