# ui-public-history: public pages say what history they hold, and show what a stock queue landed

The request: `inputs/request.md`. Two tasks, by its limit, released together.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `queue-merges` | `tasks/queue-merges.md` | | Item 1: a stock queue's merges are lines of the forge's public *landings*, and answer its *landing* (`temper/docs/design/client/forge.md`, 11.2) |
| `board-words` | `tasks/board-words.md` | | Item 2: the board's line and the two empty words (`temper/docs/design/client/browser.md`, 8.3), and a test of a merge's form on the page |

Critical path: either task alone; they share no code and meet at `notes/contracts.md`.

## What this plan decided

- **A queue merge's public landing link works, with no new record.** The request leaves this to the cut. A queue merge has no
  landing row, so `/public/projects/<project>/landings/<task>` would answer unknown and a stranger's link would say "Not
  public, or not here." So the forge makes the existing `PublicLanding` from the landing log: phase landed, its change with
  its landed commit, no gates, counts zero. `public-v1` and its golden don't change, and neither does the view's code.
- **The source is the landing log** (`temper/docs/design/domain/after-landing.md`, section 10), the record the `Changes` page
  and the brief already read: it has the commit and the time, and a change row's state has no time past `Landed`. Its commit
  and tasks alone cross; its branch, title and pull request's number don't (`forge.md`, 11.3).
- **What's public is jig's handing, as for everything.** A merge shows while every one of its changes' tasks is handed. So it
  stays through a change's close (jig retains a live goal's ended delegates, then freezes them with the goal) and leaves with
  the index's bounds, a restart's loss or the project made private. The ring and the panel bound 64 cap it.
- **A live goal's merge from before publication can show.** Publication builds the index from live goals' live tasks, and a
  change still live under one is handed, with its merge already readable on its task page. It discloses nothing new, and the
  board's line is about work that *ended*. Left as it is.
- **`way == Queue` tells a queue's merge from a landing by turn.** A change by turn stays read through its landing, so a
  retired landing still leaves *landings* as today. A retired landing's commit could have been shown too, but a landing with
  a member not public is absent and unknown by 11.2, and the log can't tell after its row retires.
- **Landings by turn stand first, then merges, each newest first.** A project lands one way at a time, so the two only meet
  across a configuration change, and landing rows hold no time to merge them by.
- **The browser story accepts either page.** The demo's goal is public and its stock queue merges at its own pace, so
  `browser_public`'s Landings page reads the empty words or the merge. Compiled, not run; after-landing runs it.

## Left

- **`0 stale proofs`** prints on a queue merge's landing page, as it does for a landing by turn with none: the same form, and
  true. A later look request could drop a zero.
- **The other two limits stay** in `temper/docs/design/client/browser.status.md`, as limits the page now states, and loading
  ended goals stays open (`jig/docs/design/domain/people.md`, section 12).
- **No draft's status** names this gap, so none closes here.

## Overlaps

- **`browser_public.rs` with `red-50`:** same file. `board-words` edits only `check_page_fields`' `Page::Landings` arm and
  leaves the geometry story's measuring; `red-50` is taken at catch-up.
- **`temper/docs/design/STATUS.md` with `ui-proposal-card/card-close`:** both edit its summary, in different sentences.
- **`temper-client-view/src/words.rs`:** the drafts beside this one have merged their tasks on it; `board-words` regenerates
  counts rather than hand-merging on a catch-up.
- **The status lines:** each task removes its own `Not yet`, one in `browser.status.md` and one in `forge.status.md`, so the
  two don't meet.
