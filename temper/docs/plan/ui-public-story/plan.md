# ui-public-story: the plan

A red's repair: `browser_public::public_board_and_task_lines_are_readable_at_desktop_and_phone_widths` fails in about half of after-landing's runs on `visible text has a box` (`inputs/request.md`). The work is done on `watch-room/red-50`, whose plan has retired, so this plan gives it a trunk.

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| public-story | `public-boxes` | | merges `d3ca9d872e`, resolves the status conflict, runs the story by name at most five times, and fixes what that shows |

## Critical path

`public-boxes`, alone.

## Decisions

The request's own answers stand: `Seen` stays as it is, the first-quad reading stays the limit `browser.status.md` records, and the sign-out 401 is fixed only if it recurs. What the cut decided beyond them:

- **The design comes with the merge.** The two `browser.md` hunks and the status entry were written on the branch and read at this cut. The merge is the task's first step, so the code and its design arrive in one landing. Writing them on the trunk first would have needed a passing **Not yet** entry for nothing.
- **The status's old 8.4 entry goes.** It says the story's check awaits the browser tier after landing, and after-landing has been running it. `temper/docs/design/STATUS.md` says the same, and the merge doesn't reach it, so the task changes that sentence too.
- **The first-quad entry stays word for word,** as the request asks. Its "needs a browser testing interface decision" still holds for whoever takes the fragments on.
- **The request's likely cause is carried as unproved, with a reading against it.** `rect` is `None` only on an answered `DOM.getContentQuads` that `parse_rect` can't read: no `quads`, empty `quads`, or an exponent-form coordinate. An error reply for a node that has left the document becomes a refusal, which the face answers as `Found::Absent`. So a page reading again on a timer would show as `long title stays readable:` with empty text, not as `has no box`. This was read from the code; nothing ran. It also means batching narrows the window of a re-render during a query and doesn't close it. The spec has the implementer tell the two apart, with a pre-approved fix for each.
- **The story's run names the tier's profile,** where a spec otherwise names no slow-tier run: this is the repair's exception, in the request and in `AGENTS.md`, 5, and the spec states it as a departure. By name it needs the bundle, so the spec lists the tier's own steps by hand, not through `tools/tiers/run`.
- **Proving a fix gets its own limit:** the request's "at most five" is read as the limit to show a flake, and a fix may take at most five more.
- **No notes:** one task, and nothing crosses a part.

## Overlaps with drafts beside this one

None in the code: `browser_public.rs` and `skein-fake-person-browser` are in no other draft's list. Two design files can't be avoided, each by a small hunk:

- `temper/docs/design/client/browser.md`, 8.4 alone, beside `look-round-13/look-proof` and `model-catalogue-2/models-proof`.
- `temper/docs/design/STATUS.md`, one sentence, beside the same two tasks.

A task that needs a fix past the spec's options would add files; the options name the ones they open.
