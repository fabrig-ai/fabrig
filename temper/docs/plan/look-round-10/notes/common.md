# look-round-10: the rules every task names

This round lands one look request, `close-chat`, by `temper/docs/design/draft/look-requests.md`, sections 3
to 5. Read this note, then your spec. Where this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design (`temper/docs/design/client/README.md`, 3.6, *The sheet* and
  *Closed ends on one line*). A task may edit the `docs/design/` files its `Owns and may change` lists. A small
  edit just outside a task's files (a re-export, a `mod` line, a design line, a golden) is made and named in
  the report.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a
  public interface may change freely, with no compatibility and no `Question:`; say so in the report. A
  `Question:` is only for what truly blocks the task.
- **A throwaway render script is allowed** in either task, untracked and gone before the push; the
  render-check command is `temper/docs/design/draft/look-requests.md`, 8, Later: don't build it.

## The request

- `git fetch origin refs/heads/look-requests` gives you the commit: `c52e34bd67`; should the branch have moved,
  it is the one whose trailer is `Look-request: close-chat`. Read `git show --format=%B -s c52e34bd67`, whole:
  its message is the request. Its own diff is `git diff c52e34bd67^ c52e34bd67 --
  temper/docs/design/client/journeys/`, the source of every rule, selector, word and node order. Read a page
  with `git show c52e34bd67:temper/docs/design/client/journeys/close-chat/02.html`.
- **Not to land:** the requests above and below it on the branch, which are other plans'. The overlay at this
  commit holds their rules too: this request's rules are its own diff's, under the comment
  `/* close-chat: ... */` and `/* A closed chat ends on one quiet line ... */`.
- **Nothing of a request is landed:** never cherry-pick or merge its commit, and never commit its pages or its
  `overlay.css`. `close-chat/` is hand-made and stays on the person's branch. The journeys' tests fail while
  `overlay.css` holds a byte, so it stays empty.
- **The branch is the person's.** Never push `look-requests`, or anything named after it.
- **`.temper-look-backdrop` is a drawing aid:** the hand-made pages hold a static `dialog` in that `div`. skein
  opens a page's dialog with `show_modal` (`skein-ui-web/src/page/web.rs`), so the real backdrop is
  `::backdrop`. The rule isn't landed; the account says so.

## Beside it

`look-round-6` (`chat-work`) changes how a chat reads, in `pages/chat.rs`, `look.rs`, `words.rs` and
`temper.css`; `ui-walk-2/needs-press` changes `pages/inbox.rs`, `pages/needs.rs`, `frame.rs` and `words.rs`.
Expect catch-ups. Keep this round's lines in `confirm.rs`, the small functions your spec names in `pages/chat.rs`,
and `acts.rs`. On a catch-up keep their side of a conflict in their functions, and renumber a look token or word
id that collides rather than theirs.

## Commits

**Every commit** ends with a last paragraph of trailers: `Look-request: close-chat`, and directly under it
`Co-Authored-By: ...` where your commit carries such a line. No other line comes between them, and nothing
follows the paragraph. A fix, a reprint and a repair of a red carry it too. Check with `git log
--format='%h %(trailers:key=Look-request,valueonly)' <base>..HEAD`.

## `temper.css`

- New rules go under the marker `/* ==== ui: chats ==== */`, never at the end of the file, so that parallel
  plans merge cleanly. Replaced rules go from where they stand.
- Rules move whole: selectors and declarations as the overlay has them.

## Goldens, prints, the gallery

They conflict across plans: on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it
is. Generated files your change makes stale (`docs/help/reference/*`, prints, goldens, counted baselines) are
yours to regenerate in the same task. `docs/help/` pages a change grows past their bound are yours to shorten,
losing no facts, never raising the bound. Where old tests walk a part the request removes, change them with it.
A re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake
box; if all fail, widen the mark's wait in the recording driver and say so.

## Render proofs

Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's
`Emulation.setDeviceMetricsOverride`. Handle `Page.javascriptDialogOpening`, or a pending request's
`onbeforeunload` hangs a CDP driver. Screenshots go in `~/src/fabrig/look-round-10/` on your builder, outside
any worktree, never committed; the report says which builder.
