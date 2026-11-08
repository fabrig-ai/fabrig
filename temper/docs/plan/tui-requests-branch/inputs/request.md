# tui-requests-branch: the TUI's look requests stack on a branch of their own, `tui-requests`

The user's word, 2026-11-08: requests for the TUI's look stack on their own branch, `tui-requests`, apart from `look-requests`, where they work on the web in parallel. The commit format and the `Look-request:` trailer are the same, with names starting `tui-` (`temper/docs/design/draft/look-requests.md`, 2). The first such request is on `origin/tui-requests` now.

## The change

- **`temper/scripts/look-refresh.sh` takes either branch:** checked out on `look-requests` or `tui-requests`, it does for that branch what it does today for `look-requests`: its lease is that branch's remote-tracking ref, its requests are that branch's commits over `origin/main`, and it pushes that branch. Any other branch is still refused, and the refusal names both. Nothing else in its behaviour changes.
- **`temper/docs/design/client/tui/journeys.md`, 10** says the TUI's requests stack on `tui-requests`, named `tui-…`, kept on `main` by the same script. Where `temper/docs/design/client/journeys.md`, 7 describes the script, one line says it takes `tui-requests` too. Leave the rest of the browser's section alone.
- **Tests:** if the script has tests, a row for `tui-requests` beside `look-requests`, and one showing that another branch is refused. If it has none, add none: this is a branch name.

## Size

One task, direct. `tui-link-colours` is in flight and edits other sections of `client/tui/journeys.md` (3.2, 4.1, 5, 6.4, 8.2): keep to section 10's lines.

## Walk

`walk = false`.

## For every task of this plan

- **A plan is complete when it lands.** No walk, render check or throwaway driver in it.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, or a tier's CPU figure.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A small edit just outside the file list is made and named in the report. A `Question:` is only for what truly blocks the task.
