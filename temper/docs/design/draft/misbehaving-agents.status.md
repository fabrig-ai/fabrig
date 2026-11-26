# misbehaving-agents: status

2026-11-26. Where the design lags or departs from [misbehaving-agents.md](misbehaving-agents.md).

- **Design lags:**
  - 3: skein's programming model makes assertions fail-stop and names invariants (`skein/docs/foundation/programming-model.md`, 10.4), but asks no check at each transfer of a conserved quantity. smith's session checks room at a result (`smith/docs/design/domain/session.md`, 5), not where a call starts.
  - 4: the fake model's random answers copy whole bodies and read none of the conversation's values, and one seed draws its answers, failures and latency (`skein/docs/design/testing/fakes/llm.md`, 1 and 2.6). No part of `Parties` is an agent's (`skein/docs/design/testing/sweeps.md`, 2). Nothing prints a drawn answer as a script.
  - 4.1 and 5.1: smith's session world's menu names no tool's ids and no yield (`smith/docs/design/testing.md`, 4).
  - 5.2: temper's composed workload and jig's one-process world play scripted turns alone (`temper/docs/design/testing/workloads.md`, 5).
  - 6: jig's parks are its half alone (`jig/docs/design/domain/testing-workloads.md`, 4). smith has no family of a session's states at a park (`smith/docs/design/faults.md`, 2), and no world draws their pairs.
  - 7: the checks every component shares are each component's own (`skein/docs/design/testing/inspection.md`, 4). smith's inspection reads waits and tables, not a session's room (`smith/docs/design/testing.md`, 6).
  - 8: no design reads a deployment's activity as shapes.
- **Design departs:** none.
- **Open:**
  - 8: where and how often the shapes reader runs against the reference deployment, and who reads its report: the user's.
