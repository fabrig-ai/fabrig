# probes-by-test: status

2026-11-15. Where the design lags or departs from [probes-by-test.md](probes-by-test.md).

- **Design lags:** none: `../domain/landing-search-by-test.md` carries
  sections 3 to 10, and `../domain/landing.md` and
  `../domain/landing-search.md` point to it where a search by test
  differs.
- **Design departs:**
  - 3: a failed receipt's ids are kept sorted byte by byte and unique,
    not in the report's order, as a tier's are and as the store's
    `failures` keeps them (`../domain/landing-search-by-test.md`, 2;
    the designer, goal 4045).
  - 4: a selection whose `temper_tests` would pass the room a probe's
    dispatch leaves it within `op_bytes` makes the whole search whole
    from its start, not that probe alone: the room is measured once, at
    the round's longest member branch, and the selection only narrows
    (`../domain/landing-search-by-test.md`, 3; the designer, goal 4045).
  - 8: the store's fields are added in place to the registered family
    `temper-store` with their defaults, with no patch and no golden
    store's line, as `AGENTS.md`, section 2 and
    `../protocol/baseline.md`, 7 say registered families take none
    (`../domain/landing-search-by-test.md`, 9; the designer, goal 4045).
- **Open:** none. Adoption's words for a gate whose workflow does not
  declare `temper_tests` (8, 10) are designed in `../domain/landing.md`,
  13.2 and left unbuilt with that section's other checks under `turn`
  (`../domain/landing.status.md`), by the user's word to goal 4045.
