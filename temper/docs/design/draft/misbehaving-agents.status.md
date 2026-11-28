# misbehaving-agents: status

2026-11-28. Where the design lags or departs from [misbehaving-agents.md](misbehaving-agents.md).

- **Design lags:**
  - 8: the worlds count each shape's run fields as `unknown`: no world counts a run's end, its next begin or its wake yet (`smith/docs/design/shapes.md`, 2 and 7). So a deployment's run fields are compared against `unknown` alone (`tools/shapes/design.md`, 4).
- **Design departs:**
  - 4.1: the noisy sweep reaches full cells with one to four cells drawn for each seed, and the stories keep four held jobs and a fifth yielding shell (`smith/docs/design/domain/session.md`, 10). At the gate's seeds four cells never fill, and a drawn bound reaches the same refusal with no seed added. A1's design.
  - 4.2: an answer is sized first with each slot at its never-issued value; one past the fake's bound once filled is made again with every slot at that value, drawing nothing more, not drawn again without the slot (`skein/docs/design/testing/fakes/drawn-agent.md`, 2). A4's design.
  - 4.3: a fake given one seed draws from it as before, and only a world on streams gives it three (`skein/docs/design/testing/fakes/drawn-agent.md`, 3 and 7). Split for every fake, the faults' and the timing's draws would change in every world with a chance or a latency range. A4's design.
  - 5.1: no world sets up `busy`: a world that faked a refusal the product can't give would draw a shape no deployment's model meets (`smith/docs/design/drawn-agent.md`, 4 and 9). A5's design.
  - 5.2: jig's chat story reaches no answered question, since no party there asks the chat one (`jig/docs/design/protocol/service-testing.md`, 1). A6's design.
  - 7: temper's one-process world checks no relation: its components run inside its service's processes, which no quiet point borrows (`temper/docs/design/testing/checks.md`, 6.1). The composed system world checks them. A7's design.
  - 8: the run fields' words are the tags a deployment's activity prints, not 6's cells' names, since no activity row names a cell (`smith/docs/design/shapes.md`, 3 and 6). A deployment's refusal by jig's or temper's tools reads `unknown`, since the archived turn holds those answers as text (`tools/shapes/design.md`, 3.2 and 8). A8's design.
- **Open:**
  - 8: where and how often the shapes reader runs against the reference deployment, and who reads its report: the user's.
