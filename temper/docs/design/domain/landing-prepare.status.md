# landing-prepare: status

2026-11-27. Where the tree's code lags or departs from [landing-prepare.md](landing-prepare.md).

- **Not yet:** 1.3: the project's
  prepare and key commands, as jobs (`worker/jobs.md`).
- **Not yet:** 1.1 and 1.2, *Whether the base contains the head*: the
  round pages its containment read page by page until it finds the
  head, for the head's trailers, and its request marks no member
  `head_trailers`. [in flight: change 21515]
- **Not yet:** 1.2, *The head's trailers*: the job reads no head's
  message in its checkout; a squash keeps only the trailers the round
  read. [in flight: change 21516]
- **Design departs:** 1.2, from the request "A landing reads a member's
  trailers from its own checkout" of 2026-11-27, its first item, decided
  by its plan's design: the job writes the head's trailers into the
  squash it commits, where the request has it give them in its result.
  The squash is made in the job, before any result, and the job's
  result of 4,096 bytes could not hold five members' trailers of up to
  4,096 bytes each, while a larger result bound would keep the job off
  every worker of an earlier build, whose prepare the request keeps
  landing. Nothing of a person's changes: a squash keeps its head's
  trailers, and no list of commits is read for them.
