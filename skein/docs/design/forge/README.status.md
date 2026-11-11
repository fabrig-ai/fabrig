# forge: status

2026-11-11. Where skein's code lags or departs from [README.md](README.md).

`forgejo.md`, section 1.2 has no known gap in Forgejo's `Read::Commits`: its plan keeps
the oldest commits the answer's rows and bytes hold, giving up the newest
as older ones come, counts every row Forgejo gives against
`total_commits`, and answers a page those hold whole with `more`, a page
they don't, or a range whose oldest commit alone passes the answer,
`TooLarge` (`skein-forge/src/forgejo/plans/commits.rs`;
`a_range_past_the_answer_keeps_its_oldest_commits`). The pin's `compare`
transcript lists its three commits newest first, as the plan reads them.

- **Not yet:** 3.3, by plan `forge-polls-quiet`: Forgejo's dialect
  sends no validator, and GitHub's sends one on the delivery log's first
  page alone [in flight: change 155].
- **Not yet:** 3.2: `Pull`'s `ci`: every `Pull` and `PullFor` reads its
  head's CI [in flight: change 158].
- **Not yet:** 3.2, 3.3: `skip` on `Reviews`, `Remarks` and `PullFiles`,
  and `cut` on their answers [in flight: change 161].
- **Not yet:** 3.2: `Item`'s `since`, which neither dialect sends.
- **Not yet:** 3.2: `PullFiles`' `sides`, which neither dialect reads:
  every page reads both sides [in flight: change 436].
 
- **Not yet:** 3.7: `Branches`' names bounded by `rows`
  (`skein-forge/src/forgejo/plans/reads.rs`, and `measure.rs`'s row count).
 
- **Not yet:** 3.2, 3.3, 3.7: `Read::Range` and its answer, in
  neither dialect. No fact records the three narrowing filters; the fake
  honours them by design (../testing/fakes/forge.md, 4).
- **Not yet:** 3.2: `Runs` sent with `workflow=` in place of
  `workflow_id=` (`skein-forge/src/forgejo/client/files.rs`); `Pull`'s
  CI folds every page's rows where a short first page's `state` serves,
  and `Statuses` reads `more` from `total_count`, which is the page's
  (`plans/actions.rs`) [in flight: change 158].

- **Not yet:** 3.2: `Labels`, `CreateLabel` and `UpdateRef`, and `CreateIssue`'s labels, in neither dialect (`skein-forge/src/api.rs`'s `Read` and `Write`). 3.6: `Protection`'s `pulls_required` and `pulls_bypass`. 3.7, 8: GitHub's dialect answers the Actions calls, `Artifact` among them, `Tree`, `Commits`, a fast-forward-only merge and an update by rebase `Refused` (github.md, 4.5). 4.3: the pin's conformance is incomplete at Forgejo 17.0.1: the seven `actions-artifact` facts and their ten transcripts hold at 17.0.0 and are unchecked at the pin, because they were recorded by hand and the conformance has no recorder for them: one would be a new tool for facts temper's story never reaches on a green CI, across a patch release, and `macos-cheap` stayed cheap by the user's rule of 2026-11-04.  no fact records that a job no runner took has no steps and no log, nor that comments take no page, nor that a comparison's rows say `removed` of a file gone: `skein-forge/src/forgejo/tests/removed_files.rs` holds a captured answer, and no conformance transcript has one, so the pin has not shown it.
- **Departs:** 4.2: `skein/docs/foundation/programming-model.md`, 5.4's one handler per cell: the plans under `skein-forge/src/forgejo/plans` hold their states, and keep many cells as arms of one `match`; no reason is recorded. 4.2: `skein/docs/foundation/programming-model.md`, 10.3 has no `_` arm on an own enum: the exchange's decoder picks the kind, so under `skein-forge/src/forgejo/plans` a match over a decoded `d::Document` or `d::Row`, or the `Option` of one where `None` is real, ends in `_`, under a named `#[expect]`, on the function that holds it. Three row dispatches, in `actions.rs`, `contents.rs` and `turn.rs`, still list their other kinds: their arms nest a `State` match, which an expect on the function would silence too. A kind a state didn't ask for is an invalid answer whichever it is. A new kind goes silent there and nowhere else: a plan's `State` keeps every arm. This departure was decided under the user's standing word, 2026-11-06, for the user's later review.  `skein/docs/foundation/programming-model.md`, 10.2 has `if let` and `let … else` on `Option` and `Result` only: `skein-forge/src/forgejo/client` and `skein-forge/src/forgejo/server` keep 23 on their own `Request`, `Answer` and `Body`, because the request of 2026-11-05 that gave the plans their states named the plans alone (the code's side is `skein/docs/development/debt/D015-flat-forge-enums.md`).
