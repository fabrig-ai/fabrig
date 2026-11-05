# look-round-4: how a request's account is written

The tasks `account-*`, and the account part of `more-accounts` and `drops-connected`, follow
`temper/docs/design/draft/look-requests.md`, sections 4 and 5, for one request. Read
`git show --stat --format=%B 230d52a0a7`: the last round's account, the shape to follow, and
`temper/docs/plan/look-round-3/tasks/account.md`.

## The render check

- **What is compared:** each page the spec names, from the request's own commit with `overlay.css` as
  it stood there, against the landed page: its gallery state, or its reprint, with the landed
  stylesheets and an empty overlay. Fetch the request's files to a scratch directory outside the
  repository's tracked paths; a request's pages are never committed.
- **How:** headless Chromium (`SKEIN_TEST_CHROMIUM`) at 1440, 1000 and 420 pixels wide, with each
  disclosure and menu forced open, comparing each element's computed style and box, then the
  screenshots for what styles miss. The gallery's states are `temper/docs/design/client/browser.md`,
  section 7.
- **The states in words** of the request's message render too, through a gallery state or a
  hand-made page in the scratch directory, beside the message's words.
- **A difference** is fixed in the view or the stylesheet, under the request's trailer, or it has a
  reason in the account. A fix stays in what this request landed: another task may be writing the
  next request's lines beside you, so keep it small.
- **The script** is throwaway: untracked, and gone before the push.

## The screenshots

`~/src/fabrig/look-round-4/<request name>/` on your builder, outside any worktree, never committed:
each page's `-request.png` beside its `-landed.png` at each width, closed and with its open states,
and each state in words. Your report names the directory and the builder (`hostname`).

## The account

- **The last commit of the task** carries the table in its message, above its trailers: one row per
  overlay rule, per page change and per state the message names, each landed, differing and why, or
  deferred, to whom and where it is tracked. No row is left open. The render check's result stands
  above the table: identical, or each difference with its reason.
- **Your report repeats the table,** so that it reaches the user.
- **A deferred row** is carried by the status or the question the spec names, never by the request.
- **Bring the design text in line,** where the spec names it: `README.status.md`, `browser.md`, 7.2,
  `journeys.md`, 9, the journeys' `README.md` files, `mockups/STATUS.md`. Only these, and only what
  this request closed.
- **Check the trailers:** `git log --format='%h %(trailers:key=Look-request,valueonly)' <base>..HEAD`.
