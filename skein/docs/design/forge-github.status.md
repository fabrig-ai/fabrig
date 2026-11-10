# forge-github: status

2026-11-10. Where skein's code lags or departs from [forge-github.md](forge-github.md).

- **Not yet:** 8.5, by plan `forge-polls-quiet`: the dialect sends a
  validator on the delivery log's first page alone.
- **Not yet:** 4.1: `Item`'s `since`, which the comments' pages do not send.
 
- **Not yet:** 4.1: `Branches`' names bounded by `rows`
  (`skein-forge/src/github/plans/reads.rs`).
- **Not yet:** 4.1: `Read::Range`.

- **Not yet:** 13: no live result establishes `app-owner-id-login-type` on
  `GET /app`. The case is present in
  `skein/tests/forge/github/src/cases.rs`; its live observation follows
  landing, as the request limits this plan to focused and fuzzy runs.
  The by-slug read is not evidence for this route.
- **Not yet:** 4.5: the GitHub routes of `Runs`, `Run`, `Artifact`, `Dispatch`, `Rerun`, `Tree`, a fast-forward-only `Merge` and a rebase `Update`, which the dialect answers `Refused` (`skein-forge/src/github/tests.rs`, `turn_calls_are_refused_without_a_request`); of `UpdateRef` and the labels, which no dialect has (forge.md, 3.2); `Protection`'s `pulls_required` and `pulls_bypass`; and the stories of 4.5 on the fake GitHub.
