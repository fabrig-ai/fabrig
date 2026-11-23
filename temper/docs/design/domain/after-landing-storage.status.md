# after-landing-storage: status

2026-11-22. Where the tree's code lags or departs from [after-landing-storage.md](after-landing-storage.md).

`after-landing-storage.md`, section 1's public queue merges read the log's commits and tasks alone;
there is no known gap in that projection. Older public history is deferred
in `after-landing.md`, section 14. The log's other gaps and the retained change rows remain below.

- **Not yet:** 1: no `trailers` or `trailer_bytes` limit;
  a push from outside temper and trailers stay out of the log
  until `after-landing.md`, sections 8 and 9 need them.
- **Departs:** 1: a tier task holds the project's whole ceiling, not
  kinds 6, 7, 9, 16 and 17 alone, as a landing does (`landing.md`,
  section 11), because jig gives a procedure the deployment starts its
  project's ceiling (`jig-domain-core/src/effects/connector.rs`,
  `procedure_seed`).
- **Departs:** 1: a change's close does not take its row, as this section
  and the request of 2026-11-05 say it does: no step erases a `ChangeRow`,
  because the answer of 2026-11-05 kept it for a closed change's panel. The log
  holds its titles without it: `a_landed_change_keeps_its_title_after_its_close`
  reads them after a restoration of the log and its repository alone.
