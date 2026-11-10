# forge-reads-targeted: status

2026-11-09. Where the design lags or departs from [forge-reads-targeted.md](forge-reads-targeted.md).

The design carries containment and emptiness as skein's `Read::Range`
(skein's `forge/README.md`, 3.2, and `forge/forgejo.md`, 1.2; `skein/docs/design/forge/client.md`, 4 and 7.4;
`../domain/forge/keeping-up.md`, 3), and the narrow lists of *What's wanted*, 3:
`PullFor`'s `head` and `base` filters, `Item`'s `since`, branch names
bounded by bytes with a refusal naming the bound (skein's `forge/README.md`, 3.2 and 3.7, and `forge/forgejo.md`, 1.2 and 2
; `skein/docs/design/forge/github.md`, 4.1; `skein/docs/design/testing/fakes/forge.md`, 4, and `skein/docs/design/testing/fakes/github.md`, 1.1;
`../domain/agent/tools.md`), and of *What's wanted*, 2, runs found by
`workflow_id` and resolved once per call, and `Pull`'s verdict from a
short first page's combined `state` (skein's `forge/forgejo.md`, 1.1 and 1.2).

- **Design lags:** *What's wanted*, 1: skein's `forge/README.md`, 3.2, and `forge/forgejo.md`, 1.2
  still have `Compare` read both whole sides of every changed file and
  list its commits, and `../domain/forge/README.md`, 5.3 and `../domain/forge/change/repairs.md` and
  `../domain/agent/tools.md` still read names, the brief's comparison and
  overlap through it or through `PullFiles`.
- **Design lags:** *What's wanted*, 2: skein's `forge/README.md`, 3.2, and `forge/forgejo.md`, 1.2
  have `Pull` read its head's statuses unasked, and `../domain/forge/keeping-up.md`,
  3 has the change's gather read the head's CI in `Pull` and again in
  `Checks`, both with job reads.
- **Design lags:** *What's wanted*, 4: skein's `forge/forgejo.md`, 1.1 still
  sizes each listing's page by its largest row.
- **Design lags:** *What's wanted*, 5: skein's `forge/README.md`, 3.4 keeps a
  bare `TooLarge`; `../domain/agent/tools.md` says "nine files, nine
  commits" pass the stock's bounds, where `forge.client.rows` is 8.
- **Open:** whether this plan may change temper's stored format, asked of
  the requester on 2026-11-09: *What's wanted*, 1's comparison answer, 2's
  pull answer with its CI asked, and 5's `TooLarge` are kept in
  `temper-protocol-store`'s schema for named-call replay.
