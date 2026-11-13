# README: status

2026-11-13. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 9: `temper-engine-domain-forge-vocabulary` doesn't exist yet [in flight: change 2193]

Section 9's `temper-agent`, `temper-fetch`, `temper-fetch-shell` and
`temper-launcher` are present; their remaining deployment integration is
kept in `draft/deployment-binaries.status.md`.

The first milestone (section 2), including the stock chat's instructions to
delegate repository changes, stands in the code, and section 9 names the crates
there are.

- **Not yet:** 6: the client's documents by surface. All of them are in
  `client/` itself: `client/web/README.md`, `client/web/journeys.md` with its journeys'
  prints, `mockups/` and `greenfield/` are the browser's, for
  `client/web/`; `client/ctl/README.md` is `temper ctl`'s, for `client/ctl/`;
  `client/README.md` holds the browser's pages' look and the TUI's
  sections, 6.11 to 6.16, beside what is generic. They move in a cleanup
  of their own, which fixes every citation to them.
- **Departs:** 4: skein's `runtime/lib.md`, 6 has `len_u64` and `usize_of` widen a
  length and a `u32` count, so that no step writes `try_from` for either,
  and the user's request of 11-04 asked that temper's hand-written code
  holds no such form. Six stay. Two steps write
  `usize::try_from(..).expect("a u32 fits usize")`:
  `temper/crates/temper-worker-domain/src/jobs.rs`, `refuse`, and
  `temper/crates/temper-worker-domain/src/translate.rs`, `failure`. Four
  tests write `u64::try_from(..len())`:
  `temper/crates/temper-playground/src/tests/stories.rs`, twice,
  `temper/tests/client-browser/tests/browser_all/browser_playground.rs`, and
  `temper/tests/one-process/src/public/load.rs`, the last over a sum, no
  length of a value, and so left as it is. No reason is recorded for the
  other five, and no debt item holds any of the six.
