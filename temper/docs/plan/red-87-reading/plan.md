# red-87-reading: the plan

An after-landing red (`inputs/request.md`): since `red-82-chat-opening`
(merge `cbfe498098`) stopped a chat's opening reads at a screen and a
half, the browser story
`browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place`
has failed in two new ways: `long`'s words moved across a read (a flake at
`0599201e87`), and `unanchored`'s anchor, `Position paragraph 02.`, missing
when the story keeps its box (run and rerun at `23261749d4`). The cause is
not known yet: the one task finds it by running that story alone, then
repairs the product or says what the story needs.

## Decided

- **One task, diagnosis first** (`reading-place`). The request asks for
  the cause before the repair, and the repair's place depends on it: the
  domain's or the view's chat page if the place moves, nothing here if the
  story's fixture or steps are the cause (the story is temper/tiers'), and
  a `Question:` if skein's page part loses the place, since a skein fault
  is repaired in skein. The task's options pre-approve each.
- **No design is written.** `temper/docs/design/client/README.md`, 5.15
  (*Opening history*, *keyed*) and skein's `ui-web.md`, 7 (*Near*, *The
  view stays*) already state what holds: older rows arriving at open are
  inserts above, and the words in view stay. A product fix brings the code
  back to them; a story fix is temper/tiers'.
- **The story runs by name, alone,** through `tools/tiers/run` with
  `TIERS_WRAPPER=heavy`, the command `browser-reading` and `ui-history-red`
  used for their flagged stories: the red's repair exception (`AGENTS.md`,
  5). It may run under a load the implementer makes, and at `23261749d4`,
  where it failed twice, if the trunk won't fail. Nothing else of the
  browser tier runs.
- **The proof of a product fix is the story itself:** at least 10 passing
  runs (20 if it failed less than one in five), plus a focused test row
  that fails for the cause's reason, in the domain's
  `a_chat_opens_to_its_opening_or_a_screen_and_a_half` or the view's
  `an_earlier_page_only_inserts`. finding-465's and red-82's cases stay in
  the focused runs.
- **What the task found lands in this file's *Found*,** so the retire and
  temper/tiers read it on main.

## Found

Both failures were **not reproduced** in 30 named, single-story runs: ten
without added load and ten with 16 CPU busy loops at `2a399129fe`, then ten
with the same load at `23261749d4`.
`unanchored` held `Position paragraph 02.` at its opening measurement in
every run; its earlier-read pair stayed at y=151, height=24.
`long` held `Reading paragraph 07.` and reply 8's end at its opening;
its earlier pair stayed at (144,24), its later pair at (557,19) → (558,19).
Product and fixture are unchanged; after-landing reproduction remains.

## Parts and tasks

| Part | Task | Crates (files) | Needs | Minutes |
|---|---|---|---|---|
| `reading-place` | `reading-place` | `temper-client-domain` (`pages/chat.rs`, `tests/pages/chat.rs`) and `temper-client-view` (`pages/chat.rs`, `tests/chat.rs`), for a product fix only; this file's *Found* | | 150 |

Slice 1 is the part; `walk = false`.

## Critical path

`reading-place` alone, released with the first push.

## Overlaps with drafts beside this one

None of the drafts beside this one changes the task's files. Its product
files are the domain's and the view's chat pages and their tests; the
nearest any sibling comes is `temper-client-view`'s `words.rs`, `look.rs`
and other pages, which the task doesn't touch. A product fix that turns
out to need `temper-client-shell-web` (a reader file) would meet
`temper.css`, which `ui-reaches`, `ui-p4a-team` and `ui-p5a-phone` changed;
the task names only its `src/**` there.
