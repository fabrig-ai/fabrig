# look-ask-ways: plan

Step 4 of `temper/docs/design/draft/stock-ways.md`, section 7: the look
request `ask-temper-ways` (`98322b3565` on `origin/look-requests`),
landed as `temper/docs/design/draft/look-requests.md`, sections 3 and 5
ask, every state its message names, nothing drawn by hand
(`inputs/request.md`). The engine's side is on main (`ui-ask-m3a`): the
`ways` read, the paths and their cards; the client's domain already
watches `ways` and sends "Use this" and "Keep it". What is left is the
client's: the page's state, the drawing in the request's look, the
opening itself, and the removal of what it replaces.

The cut wrote the design first: `temper/docs/design/client/README.md`,
5.13 (*A way's acts*: the page's state and the toast) and 5.16 (*Drawn*,
*Its layout*, *Pressed*), with their **Not yet** entries in
`README.status.md`.

## Parts and tasks (first batch)

- **state:** `opening-state`, high, 110 minutes, no needs: the tab shown
  and the open stop on `StockWays`, the acts `WayTab` and `WayStop`, a
  way's act disabled while out, and the toast naming the revision
  (`notes/contracts.md`, section 1).
- **drawing:** `path-look`, high, 200, no needs: the path across and
  down the page, its bars as ends and goal branches, two tones of line,
  pressed stops with their halo and caret, and its rules
  (`notes/contracts.md`, section 2).

## Critical path (as it stands)

`path-look` (200), then the opening (next batch), then the stories and
the account.

## Overlaps

With drafts beside this one: `look-outside-pages` (integration, not yet
landed) changes `temper-client-view`'s `look.rs` and `words.rs`,
`temper.css` and the gallery. This plan appends its tokens after the
last (that plan reuses freed numbers below them), numbers its words from
4300 (that plan reuses freed ones), puts its rules under their markers,
and regenerates the gallery's files on a catch-up.
