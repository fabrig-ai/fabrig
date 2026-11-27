# pair: status

2026-11-27. Where the design lags or departs from [pair.md](pair.md).

- **Design lags:**
  - 3.1: jig's `domain/tasks/README.md`, section 3: a conversation's reads and inbox need a person requester, and a producer's requester is its change; no participant beside the requester exists, and temper routes any party's words to a change's producer.
  - 3.2: smith's `domain/run.md`, section 1: a run is Finite or Open, and jig derives the mode from the task's ending. jig's `domain/tasks/lifecycle.md`, section 2: a parked result-ended agent with nothing awaited is woken and spends a try.
  - 3.3: no finish request of a person's exists; smith's `domain/run-messages.md`, section 1: Finite ignores `finish_requested`, and a run may `deliver` or `finish` whenever its charter grants them.
  - 3.4: smith's `domain/run-delivery.md`, 1.5: a run saves only when its model calls `save`. `../domain/forge/change/README.status.md`: early starts read branch heads.
  - 3.5: no idle bound for a change exists.
  - 3.6: `../domain/forge/change/repairs.md`, section 1: repairs reuse the producer's charter.
  - 3.7, 4: [../protocol/configuration/stock.md](../protocol/configuration/stock.md) has no `pair` charter; `../client/ctl/tools-acts.md` has no `task finish`.
- **Design departs:** none.
- **Open:**
  - 3.5: the idle bound's default, three days, a figure of the drafter's.
  - 3.3: a finish request lasting through fixes until it lands, the default; or one try per request.
