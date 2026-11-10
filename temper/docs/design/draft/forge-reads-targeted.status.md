# forge-reads-targeted: status

2026-11-09. Where the design lags or departs from [forge-reads-targeted.md](forge-reads-targeted.md).

The design carries containment and emptiness as skein's `Read::Range`
(skein's `forge.md`, 3.2 and 5.2; `forge-client.md`, 4 and 7.4;
`../domain/forge.md`, 5.3), and the narrow lists of *What's wanted*, 3:
`PullFor`'s `head` and `base` filters, `Item`'s `since`, branch names
bounded by bytes with a refusal naming the bound (skein's `forge.md`, 3.2,
3.7, 5.2 and 9; `forge-github.md`, 4.1; `fake-forge.md`, 4 and 10.1;
`../domain/agent.md`, 4.3), and of *What's wanted*, 2, runs found by
`workflow_id` and resolved once per call, and `Pull`'s verdict from a
short first page's combined `state` (skein's `forge.md`, 5.1 and 5.2).

- **Design lags:** *What's wanted*, 1: skein's `forge.md`, 3.2 and 5.2
  still have `Compare` read both whole sides of every changed file and
  list its commits, and `../domain/forge.md`, 7.3 and 8.4 and
  `../domain/agent.md`, 4.3 still read names, the brief's comparison and
  overlap through it or through `PullFiles`.
- **Design lags:** *What's wanted*, 2: skein's `forge.md`, 3.2 and 5.2
  have `Pull` read its head's statuses unasked, and `../domain/forge.md`,
  5.3 has the change's gather read the head's CI in `Pull` and again in
  `Checks`, both with job reads.
- **Design lags:** *What's wanted*, 4: skein's `forge.md`, 5.1 still
  sizes each listing's page by its largest row.
- **Design lags:** *What's wanted*, 5: skein's `forge.md`, 3.4 keeps a
  bare `TooLarge`; `../domain/agent.md`, 4.3 says "nine files, nine
  commits" pass the stock's bounds, where `forge.client.rows` is 8.
- **Open:** whether this plan may change temper's stored format, asked of
  the requester on 2026-11-09: *What's wanted*, 1's comparison answer, 2's
  pull answer with its CI asked, and 5's `TooLarge` are kept in
  `temper-protocol-store`'s schema for named-call replay.
