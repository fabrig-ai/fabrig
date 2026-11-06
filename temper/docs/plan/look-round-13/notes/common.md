# look-round-13: what every task keeps

The look request `connect-flows`, procedure `temper/docs/design/draft/look-requests.md`, sections 3 to 5.

## The request

- It is commit `97ad67393b` on `origin/look-requests`, the one whose message has the trailer
  `Look-request: connect-flows`. The branch moves: `git fetch origin look-requests`, and if the hash is gone find it with
  `git log origin/look-requests --grep='Look-request: connect-flows' --format=%H -n1`.
- Read its pages without adding them to the tree: `git archive <commit> temper/docs/design/client/journeys/connect-flows | tar -x -C <a scratch directory outside the checkout>`.
  Nothing of the request's own lands: not its pages, not `connect.css`, not `adopt.js`. The `README.md` there says what
  each page is, what goes and what is not mocked; read it first.
- **Make the real pages look exactly like these.** A page's roles, names, words and look tokens are the request's; a
  state the pages don't show (empty, refused, busy, failed, the narrow width, the terminal) keeps today's words and
  meaning unless the request says otherwise.
- **Every commit carries the trailer** `Look-request: connect-flows`, directly above any other trailers.
- A journey's or the gallery's print is not hand-edited.

## Runs

- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories or a
  tier's CPU figure. A browser story the change walks is edited and compiled (`heavy cargo check -p temper-client-browser --tests`), not run, and the report says so. This overrides anything else in a spec.
- A throwaway render script to look at a page is allowed: untracked and gone before the push. Chromium 147's headless won't go below 500 px by `--window-size`; use CDP's `Emulation.setDeviceMetricsOverride` for 420. Handle `Page.javascriptDialogOpening`.
- Before calling a focused or fuzzy failure "outside the change", run it once on the pristine base. If main passes and your branch fails, it is yours.

## Deciding

- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely, with no compatibility kept and no `Question:`; say so in the report. A small edit a task needs just outside its file list (a re-export, a `mod` line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Reach for the request's pages and then `temper/docs/design/client/greenfield/`, never the old page.** Where old tests walk a part the request removes, change them with it, keeping what they check for everything that stays.
- A test's oracle or helper that the correct behaviour trips is yours to repair in the same task, keeping what it checks for everything else, and named in the report.

## Shared files

- **`temper.css`:** rules go under the marker `/* ==== ui: settings ==== */` (or `welcome`, `phone`), never at the end of the file. Remove replaced rules where they stand.
- **Words:** a model page's words go in `temper-client-view/src/accounts/words.rs`, a forge page's in `temper-client-view/src/words.rs`, the sheet's in `temper-client-domain/src/forge/words.rs`. Add beside the words they replace, not at the end, so parallel tasks merge; a word nothing says any more is removed.
- **Goldens, prints, gallery `names.txt`/`tests.rs`:** on a catch-up regenerate rather than hand-merge; keep `names.txt` sorted. Files your change makes stale (`docs/help/reference/*`, prints, goldens, counted baselines) are yours in the same task. A `docs/help/` page your change grows past its bound is shortened, losing no fact, and the bound never raised.
- **Journeys:** a view or look change is carried by a reprint, `heavy cargo run -p temper-client-journeys`, from the journeys' recordings (`temper/docs/design/client/journeys.md`, 8), run in the task whose change makes `the_committed_journeys_are_their_print` fail. A walk that no longer holds (a control renamed, a step gone) is `again NAME` on a fresh fake box (8.1). If `again` stops on "a key mark caught a moving page", retry up to three times, each on a fresh box; then widen the mark's wait in the recording driver and say so.

## Not yours

`docs/design/` is the lead's and `look-proof`'s: what the code taught goes in your report.
