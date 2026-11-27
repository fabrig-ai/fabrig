# whose-red: status

2026-11-27. Where the design lags or departs from [whose-red.md](whose-red.md), once its milestone's five goals have landed (`whose-red.execution.md`, 1).

- **Design lags:**
  - 3.3: the after-landing tiers keep a flake of their own: a red tier runs its failed tests again and calls those that pass a flake, kept on the tier's record (`../domain/after-landing.md`, 5.2), whatever a project's `flakes` says (`../domain/forge/judgement.md`, 4).
  - 4.1: `../protocol/wires.md`, 8 still says the checked close warns of main's counts missing and leaves them to the gate, where `tools/pre-pr/design.md`, 4.1 has the close break.
  - 7.4 and 8: `../domain/forge/catching-up.md`, 4 and `../protocol/store.md`, 2 still describe the head beside as a held branch's, where the write record now keeps it as a compare entry (`../domain/forge/writes.md`, 7).
  - 10: `../testing/engine-checks.md`, 5.1 still lists P37's scenario without saying it runs, beside P58's and P67's, which say so.
- **Design departs:**
  - 3.3: a CI runner's checkout is the project's own: temper prepares none, so the project's command writes the rerun marker its setting states where it finds none (`../domain/forge/judgement.md`, 4), rather than temper giving it to every checkout it dispatches. Decided by the second goal's design, 2026-11-26.
  - 4.1: the engine checks a refusal against the head its job's checkout started at, not the tree it judged, which the engine never sees (`../domain/forge/change/repairs-releases.md`, 1, *A close job ends*). Decided by the first goal's design, 2026-11-25.
  - 4.2: main's tip stands for its published counts: a released refusal checks again when the goal's branch or main moves, not when counts alone are published (`../domain/forge/change/repairs-releases.md`, 1, *A refusal where nothing it judged has moved*). Decided by the first goal's designer, 2026-11-24, and kept by its requester, 2026-11-25, over a forge read of the counts' ref.
  - 6.3: a fresh measurement is taken by whoever may release the held task or decide its escalation (`../domain/forge/change/repairs-releases.md`, 4), not by the escalation's decider alone: a measurement is a release asking one check more, so jig's standing for a release holds for it. Decided by the fifth goal's design, 2026-11-27.
  - 7.3: a worker's push that finds its branch's record full is refused at once, not kept waiting, and pushes nothing (`../domain/forge/writes.md`, 6); a write through the outbox waits as the draft says. Decided by the fourth goal's design, 2026-11-26.
  - 8: a writer the previous build kept as live converts to an entry sent by its attempt, settled when that attempt answers or is lost, and a lost one to an uncertain entry (`../domain/forge/writes.md`, 7), rather than both to uncertain entries. Decided by the fourth goal's design, 2026-11-26.
- **Open:** none.
