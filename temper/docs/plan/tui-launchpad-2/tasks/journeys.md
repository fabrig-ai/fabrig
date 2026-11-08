+++
id = "journeys"
part = "journeys"
kind = "docs"
needs = ["arrows", "chat-rule"]
files = ["temper/docs/design/client/tui/journeys/launchpad-chat/recording", "temper/docs/design/client/tui/journeys/launchpad-chat/walk", "temper/docs/design/client/tui/journeys/**/*.txt", "temper/docs/design/client/tui/journeys/**/*.html", "temper/docs/design/client/tui/journeys/*.css", "temper/docs/design/client/tui/journeys/*.js", "skein/docs/design/ui-screen.status.md", "temper/docs/design/client/README.status.md", "temper/docs/design/client/tui/journeys.status.md"]
effort = "high"
minutes = 60
+++

## Goal

Close the plan: record the TUI journey `launchpad-chat` again on the
landed code (generation, as the request asks), check its frames against
the request, close the plan's **Not yet** status entries, and land the
look request `tui-chat-title-below`'s account in the last commit
(`temper/docs/design/draft/look-requests.md`, section 5).

**Departures from the implementer's text, on purpose:** this task runs
`temper-client-journeys again`, which builds and starts a fake box,
headless Chromium and the TUI in a pseudo-terminal (generation, not a
tier); and it edits three status files under `docs/design/`, removing
the entries this plan's code has closed.

Acceptance:

- `heavy cargo run -p temper-client-journeys -- again launchpad-chat`
  has recorded and added the journey: its `recording` and prints
  replaced, its README, walk and setup unchanged. A run that stops on a
  moving screen is tried again, at most three times in all, each on a
  fresh fake box; if all fail, widen the wait before the step it names
  by adding a `see` step, in the walk, for the text the screen was
  moving to, record again and say so.
- The new `launchpad-chat/04.txt` starts with the person's question on
  its first row and has its rule above the box as
  `── Hello / What can you tell… T2 on 127.0.0.1:PORT ──` aligned to the
  end, PORT the recording's; `02.txt` lists `/effort` after `/model`;
  the person's words in `04.html` are `f-dim` after an `f-accent` `❯`,
  with no band. If the chat's number is not T2 in the new recording,
  that is the fake box's, not a red.
- `heavy cargo run -p temper-client-journeys` then changes nothing, and
  `heavy cargo nextest run -p temper-client-journeys` passes;
  `sign-in-approved` replays as it stands.
- The status files lose the plan's **Not yet** entries, each only where
  its code has landed (read the trunk to check each part): the 5.2 entry
  of `skein/docs/design/ui-screen.status.md` (and its "Otherwise" before
  the paragraph that follows); the 6.12-to-6.16 entry of
  `temper/docs/design/client/README.status.md`; both entries of
  `temper/docs/design/client/tui/journeys.status.md`, whose 9 entry the
  new recording closes. Dates stay 2026-11-08 unless the day has moved
  (`date -u`). `python3 -I -B tools/citations/citations.py --base
  origin/main` reports nothing new.
- The last commit's message carries the account, one row each, as below,
  each row's result checked against the landed code and frames:

  | Item | Result |
  |---|---|
  | The rule above the box carries the project, " / ", the title, its number, " on " and the place, two dashes from the right edge | landed (`rule-title`, `chat-rule`) |
  | "✦" goes with the top line | landed: `tui-star` and `tui-chat-top` removed |
  | The title cut to at most 20 characters at a word, ending "…" | landed (`chat-rule`'s cut and its table) |
  | The place: "box" at 8367, "box:PORT" at another port, HOST:PORT otherwise | landed (`place`, the link file read at start, `Start.boxed`); `launchpad-chat/04` differs: `127.0.0.1:PORT`, because `again` runs the TUI apart from the box's link file so that it walks the approval (`temper/docs/design/client/tui/journeys.md`, 8.2); the view's rows hold "box:40691" |
  | The conversation starts on the first row | landed |
  | A narrow terminal: the place goes first, then the project | landed (`chat-rule`'s rows) |
  | A list open under the box: the rule keeps the title | landed (`chat-rule`'s row) |
  | The light theme, by the same tokens | landed (the same tokens in both themes) |
  | Replaces the chat's top line (client README 6.14) | landed |
  | The launchpad's framed header stays | landed (its goldens and `launchpad-chat/01`) |
  | Not the look's: telling a box from another engine | landed by the engine's link file (client README 6.12, *On a box*); from the box starting it is `tui-box-front`'s |
  | The page change, `launchpad-chat/04.txt` | landed but the place, as above |
  | The render check | follows landing: temper/tui's after-landing check (client README, section 11; `tui/journeys.md`, 11) |

  Adjust a row's result only where the landed code says otherwise, and
  then say why in the row.
- Every commit carries the trailer `Look-request: tui-chat-title-below`.

## Inputs

- `temper/docs/design/client/tui/journeys.md`, 8 (the walk, `again`,
  its environment) and 9 (the journeys).
- `temper/docs/design/draft/look-requests.md`, sections 3 and 5.
- The request's message and frame:
  `temper/docs/plan/tui-launchpad-2/inputs/request-commit-chat-title-below.txt`
  and `temper/docs/plan/tui-launchpad-2/inputs/chat-title-below-04.diff`.
- `temper/docs/plan/tui-launchpad-2/notes/common.md`.

## Owns and may change

- Owns the files in `files`: the recording and prints `again` and the
  reprint write, and the three status files' entries above.
- Not yours: the journeys' READMEs, walks, setups and `overlay.css`
  (sources), but a `see` step added to `launchpad-chat/walk` as
  pre-approved above; any code. A red
  in code found by the recording is a `Failed:` naming the task whose
  code it is, with the log's lines.

## Steps

1. `heavy cargo run -p temper-client-journeys -- again launchpad-chat`;
   read its counts.
2. Read the new frames against the acceptance; `heavy cargo run -p
   temper-client-journeys`; `heavy cargo nextest run -p
   temper-client-journeys`.
3. The status entries; the citations check.
4. Commit the recording and prints, then the status files with the
   account in the last commit's message (each with the trailer), and
   push.

## Report

As the implementer's text says; the again run's attempts and counts,
and how `04.txt` compares with the request's frame.

## Answers

2026-11-08. *`launchpad-chat/04` lacks the person's opening question
(finding-465): wait for post-landing-probes' repair, or land without it?*
Land without it. The missing question is finding-465's: its repair
(`post-landing-probes/finding-465`, still unlanded on main) owns the code,
and this task owns none. The request (item 3, *Journeys*) asks the journey
recorded again on this plan's code, which run `journeys-2` did; it doesn't
fix the question's presence. So:

- Keep run `journeys-2`'s recording and prints (`a08df03746`) as they
  stand; don't record again unless main or the trunk has gained the
  finding-465 repair by the next run (then record once more, as step 1
  says, and the acceptance holds as written).
- The acceptance's "starts with the person's question on its first row"
  and "the person's words in `04.html` are `f-dim` after an `f-accent`
  `❯`" are met by `04.txt` starting with the conversation (the answer)
  on its first row, no band in `04.html`, and the dim words held by the
  view's tables. The rest of the acceptance stands.
- Close the plan's status entries as the acceptance says. In
  `temper/docs/design/client/tui/journeys.status.md`, replace the 9
  entry with one **Not yet** line: `launchpad-chat/04` lacks the
  person's opening question, a fast chat's fault (finding-465,
  `post-landing-probes`), so its frame doesn't show the dim person's
  words of `../README.md`, 6.15; it is recorded again with that repair.
- The account's rows "The conversation starts on the first row" and
  "The page change" read: landed (the rule, title and first row), the
  opening question absent by finding-465, outside this plan; its dim
  words are checked by the view's rows and in the journey once that
  repair records it again.
- List the journey's re-record under `## Left` for finding-465's repair.
