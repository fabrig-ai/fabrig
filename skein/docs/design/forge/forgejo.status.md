# forgejo: status

2026-11-11. Where the tree's code lags or departs from [forgejo.md](forgejo.md).

- **Not yet:** 1.1 to 1.3: `Reviews`, `PullFiles` and a `Remarks` slice sized for real rows and cut at the answer's bytes, read on with `skip`: each holds as many of its listing's largest rows as `answer_bytes` holds (`skein-forge-connection/src/limits.rs`, `page_size`) [in flight: change 161].
- **Not yet:** 1.2: `Read::Pull`'s `ci`; `PullFor`'s status read (`plans/reads/done.rs`, `pull_status`) [in flight: change 158].
- **Not yet:** 1.2, 1.4: `Settings` read with a row kept; `PullFor`'s and Actions' `404`s drop the row; a write's `401` or `403` keeps it (`plans/reads.rs`, `plans/failure.rs`, `plans/actions.rs`) [in flight: change 159].
- **Not yet:** 1.2: `Read::PullFiles` without `sides`: every page reads both sides (`plans/contents.rs`) [in flight: change 436].

- **Not yet:** 2: no conformance fact records a `style=merge` update of a
  head already holding its base, or what a rebase's push does to a push
  made while its git work is running. The fake answers the first unchanged
  and reads the head at landing for the second; the design assumes the
  rebase can drop the intervening push (client.md, 7.3). These facts
  await real-Forgejo conformance after landing.
- **Not yet:** 1.2: `Item`'s `since`, which neither dialect sends.
 
- **Not yet:** 1.2: `Branches`' names bounded by `rows`
  (`skein-forge/src/forgejo/plans/reads.rs`, and `measure.rs`'s row count).
 
- **Not yet:** 1.2: `Read::Range` and its answer, in
  neither dialect. 2: no fact records the three
  narrowing filters; the fake honours them by design
  (../testing/fakes/forge.md, 4).
- **Not yet:** 1.1, 1.2: `Runs` sent with `workflow=` in place of
  `workflow_id=` (`skein-forge/src/forgejo/client/files.rs`); `Pull`'s CI
  folds every page's rows where a short first page's `state` serves, and
  `Statuses` reads `more` from `total_count`, which is the page's
  (`plans/actions.rs`) [in flight: change 158].

- **Not yet:**  the missing labels and `UpdateRef` remain `README.status.md`, 3.2.  The protection fields remain `README.status.md`, 3.6.  The GitHub routes remain `github.status.md`, 4.5. 2: the pin's conformance is incomplete at Forgejo 17.0.1: the seven `actions-artifact` facts and their ten transcripts hold at 17.0.0 and are unchecked at the pin, because they were recorded by hand and the conformance has no recorder for them: one would be a new tool for facts temper's story never reaches on a green CI, across a patch release, and `macos-cheap` stayed cheap by the user's rule of 2026-11-04. 2: no fact records that a job no runner took has no steps and no log, nor that comments take no page, nor that a comparison's rows say `removed` of a file gone: `skein-forge/src/forgejo/tests/removed_files.rs` holds a captured answer, and no conformance transcript has one, so the pin has not shown it.
- **Departs:**  `skein/docs/foundation/programming-model.md`, 5.4's one handler per cell: the plans under `skein-forge/src/forgejo/plans` hold their states, and keep many cells as arms of one `match`; no reason is recorded.  `skein/docs/foundation/programming-model.md`, 10.3 has no `_` arm on an own enum: the exchange's decoder picks the kind, so under `skein-forge/src/forgejo/plans` a match over a decoded `d::Document` or `d::Row`, or the `Option` of one where `None` is real, ends in `_`, under a named `#[expect]`, on the function that holds it. Three row dispatches, in `actions.rs`, `contents.rs` and `turn.rs`, still list their other kinds: their arms nest a `State` match, which an expect on the function would silence too. A kind a state didn't ask for is an invalid answer whichever it is. A new kind goes silent there and nowhere else: a plan's `State` keeps every arm. This departure was decided under the user's standing word, 2026-11-06, for the user's later review. 1: `skein/docs/foundation/programming-model.md`, 10.2 has `if let` and `let … else` on `Option` and `Result` only: `skein-forge/src/forgejo/client` and `skein-forge/src/forgejo/server` keep 23 on their own `Request`, `Answer` and `Body`, because the request of 2026-11-05 that gave the plans their states named the plans alone (the code's side is `skein/docs/development/debt/D015-flat-forge-enums.md`).

The client world's late copies read
  the head at landing, so its stories for both styles do not establish the
  rebase's concurrent-push behaviour. The connector world checks a dropped
  push by an explicit branch rewrite; the design takes the forced push at
  its worst until the real forge's behaviour is recorded.
