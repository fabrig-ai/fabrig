+++
id = "reading-place"
part = "reading-place"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-domain/src/pages/chat.rs", "temper/crates/temper-client-domain/src/tests/pages/chat.rs", "temper/crates/temper-client-view/src/pages/chat.rs", "temper/crates/temper-client-view/src/tests/chat.rs", "temper/docs/plan/red-87-reading/plan.md"]
effort = "high"
minutes = 150
+++

## Goal

Since `red-82-chat-opening` (merge `cbfe498098`) stopped a chat's opening
reads at a screen and a half, the browser story
`temper-client-browser-world::browser_all`
`browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place`
has failed in two new ways (the request). Find each failure's cause by
running the story by name, alone. If the person's place moves as older
rows arrive at open, fix it in the product, with a focused test row. If
the story's fixture or steps are the cause, say exactly what the story
needs and change nothing: the story is temper/tiers'.

The two failures, as the request records them:

- **`long`, a flake at `0599201e87`:** `browser_pages.rs:655 long: (519,
  19) -> (558, 19): words moved`. Line 655 is `held`'s first assertion,
  and the story calls `held` for `long` twice: `reading_measures`
  (`before-earlier`, `after-earlier`) and then `before-later`,
  `after-later`.
- **`unanchored`, its run and its rerun at `23261749d4`:**
  `browser_pages.rs:383 unanchored: person 1, step 76: missing box …
  Paragraph 'Position paragraph 02.'`. Line 383 is `stories.assert_done`;
  step 76 is a `KeepBox` of `reading_back_steps`' anchor,
  `paragraph(position_words(2))`: its `head`, `before-earlier` or
  `after-earlier`. Step numbers count from 0 over the whole script
  (`temper/tests/client-browser/src/story.rs`).

Acceptance:

1. **Each cause is recorded.** For each failure that reproduces, the report
   gives the step, the kept box or failing check, and the failing run's
   lines that show it, read with `grep` and `tail`, never whole. It also
   says what the chat held at open: the rows of its first activity page,
   their weight by `row.weight()`, whether the domain's opening stopped at
   `OPENING_WEIGHT`, whether the log's head was then near (skein's
   `ui-web.md`, 7, *Near*), and which reads followed and when. For `words
   moved`, it says which pair moved and what was inserted or removed above
   the words. It says how many runs it took to fail, and under what load.
2. **The repair follows the cause** (Owns and may change, the options):
   - **The place moves:** a focused test row that fails for that reason
     first, then the fix in the domain's or the view's chat page. The story
     then passes at least 10 runs by name, alone, under the conditions that
     reproduced it, every one passing; at least 20 if it failed less than
     one run in five before the fix.
   - **The story is the cause** (its fixture's sizes, or a step that reads
     before the opening has settled): no product or story change. The
     report and `plan.md`'s *Found* say exactly what the story needs (the
     window, each reply's code-line words and paragraphs, or the step that
     waits, and for what), the measurements that set each number, and why.
     Each number keeps `temper/docs/design/client/browser.md`, 4.3: the
     position chat stays inside one window, and the long chat passes three.
   - **Not reproduced:** change nothing and report `Done: not reproduced`,
     with every run's time and outcome, the machine and what you ruled out.
     After landing, the story runs again, and a red comes back as a repair.
3. **No check is weakened:** `git diff origin/main --
   temper/tests/client-browser` is empty.
4. **finding-465's short-chat case and red-82's long-chat case pass:**
   Steps, 4's focused runs. Only a product fix needs them.
5. **`plan.md`'s *Found*** says which case each failure was (Steps, 5).

## Inputs

- The request: `temper/docs/plan/red-87-reading/inputs/request.md`.
  `temper/docs/plan/red-87-reading/notes/common.md` holds its rules for
  this plan.
- `temper/docs/design/client/README.md`, 5.15: *The log is bound*,
  *Opening history* (the rule red-82 built), *Each line's outer node is
  keyed* and *A chat's log head holds nothing that stays while lines
  arrive below it*.
- skein's `skein/docs/design/ui-web.md`, 7, *Reading a log*: *Near* (a
  page opened at its end reads earlier lines until its head is one and a
  half of its screens off) and *The view stays* (across an apply, a log not
  following keeps the words in view where they were).
- `temper/docs/design/client/browser.md`, 4.3 (*Reading's replies are
  sized to the window*, *A chat's history*) and 4.6, item 14: the story's
  design.
- jig's `jig/docs/design/client/domain/client.md`, section 10: the
  thread's window, its pages and a line's weight.
- The story: `temper/tests/client-browser/tests/browser_all/browser_pages.rs`,
  `a_chat_opens_at_its_end_and_reads_back_keeping_its_place` and its
  helpers (`reading_chat`, `reading_back_steps`, `reading_measures`,
  `held`). Its fixture: `temper/tests/client-browser/src/reading.rs`
  (`POSITION_MESSAGES`, `LONG_MESSAGES`, `READING_WINDOW_BYTES`,
  `POSITION_CODE_WORDS`, `LONG_CODE_WORDS` and `replies`' paragraph counts).
- The opening's code: `git show fcf1a1b56b` (red-82: `OPENING_WEIGHT`,
  the stop in `Chat::accept_activity`, the domain's table test
  `a_chat_opens_to_its_opening_or_a_screen_and_a_half`) and `git show
  1e6e2388ee -- temper/crates/temper-client-domain` (the opening's follow
  before it).

What the lead's reading found. Nothing below was run; it narrows the
search and settles nothing.

- **Before red-82, the opening followed the older cursor to the window's
  bound,** and the position chat, inside one window, opened whole. Now
  the domain stops once the held lines weigh 2,880 bytes, and each
  position reply alone weighs several kilobytes (its code line has 1,520
  words). So what the position chat holds at open is its first page,
  plus whatever the browser's log reads because its head is near. Whether
  `Position paragraph 02.` is held when the story keeps it depends on which
  rows that page holds, the log's height against the replies' paragraphs,
  and whether a `Near` read is still out (or not yet asked) when the story
  passes `reading_chat`'s `gone(Loading earlier messages)` and keeps the
  box. `KeepBox` reads once and doesn't wait.
- **Between `0599201e87` (the flake) and `23261749d4` (failed twice),**
  main merged `ui-p2-home` (`b92e9d0e15`) and four retires. In the
  conversation's code, that range added only two `Act` arms and a lint
  expectation to the domain's `pages/chat.rs`; its `temper.css` changes are
  Home's and Needs' rules. From `23261749d4` to the trunk, the reading code
  (both chat pages, jig's thread, skein's `log.rs`) is unchanged. So load
  and timing may tell the two tips apart.
- **The place is the page part's to keep** (`ui-web.md`, 7, *The view
  stays*), on keys the view gives (`README.md`, 5.15, *keyed*). A read in
  above can still move the words if it rewrites or replaces an entry in
  view rather than inserting above it, or if what it changes above is no
  entry (a status, the context line, a day's label). The view's
  `an_earlier_page_only_inserts` (`temper/crates/temper-client-view/src/tests/chat.rs`)
  holds that for a read the person asks for, not for one at open.

Departures from the implementer's text, and why:

- **You run one browser story, by name and alone, as often as you need.**
  The lab's rules forbid a plan's runs the slow tiers. They allow a red's
  repair to run the flagged test by name, alone, to reproduce it and to
  prove the fix, and the request grants exactly this run. Run only this
  story, never the browser tier or another story. The command is in
  Steps, 1.
- **You may make a load, and add temporary instrumentation** (Steps, 1
  and 2), to see the cause. Neither is committed. Neither drives the
  product as a walk or a render check does: they are throwaway code, not a
  throwaway driver.
- **You write one section of the plan's `plan.md`** (Steps, 5).

## Owns and may change

Owns, for the product case only:

- `temper/crates/temper-client-domain/src/pages/chat.rs` and
  `src/tests/pages/chat.rs`: the opening's reads, if the domain is what
  moves the place.
- `temper/crates/temper-client-view/src/pages/chat.rs` and
  `src/tests/chat.rs`: the conversation's tree, if the view is what moves
  the place (a key, or a node at the log's head that changes while older
  rows arrive).

Owns in every case: `temper/docs/plan/red-87-reading/plan.md`, its
section *Found* alone.

Change only the files the cause and its fix need, and name them in the
report. Name each file outside this list that you change, with the reason.

Reader and neighbour files:

- `temper/crates/temper-client-view/src/pages/chat_line.rs` and
  `temper/crates/temper-client-shell-web/src/**`: only if the cause sits
  there (a line's key, the shell's handling of the log's acts). The test
  row then goes in the table nearest the change (the shell's
  `src/host_tests.rs`).
- `temper/crates/temper-client-view/src/tests/**` and
  `temper/crates/temper-client-domain/src/tests/**`: assertions a product
  fix makes stale, kept checking what they check for everything else.
- Prints and goldens a product fix makes stale (the view's, the terminal
  shell's, the gallery's), regenerated by their own tool; a journey
  recording, by `temper-client-journeys again`.

Not yours: `temper/tests/client-browser/**` (the story and its fixture:
temporary edits only, Steps, 2), `temper/docs/design/**`,
`temper/docs/help/**`.

Pre-approved options:

- **The domain moves the place** (a read at open replaces the window, or
  reads its tail again, where it should page in above): fix it in
  `Chat::accept_activity` or beside it, keeping every stop
  `a_chat_opens_to_its_opening_or_a_screen_and_a_half` and the tests from
  `1e6e2388ee` pin. The test row goes in that table, or in the domain's
  test of what moved (`a_full_first_page_waits_for_the_persons_next_read`,
  `an_empty_opening_page_keeps_an_announced_tail_read`) as a row.
- **The view moves the place** (an entry in view rewritten, removed or
  re-keyed as older rows arrive at open, or a node at the head that the
  page part takes for the entry it holds): make it insert above, as a read
  the person asks for does. The test row goes in
  `an_earlier_page_only_inserts`: turn its `retained_day` loop into rows
  if the new case needs another field (for instance, the thread's state at
  open: `Reading::Older` from the opening's follow, the status at the head,
  the context line). Its row fails before the fix for the cause's reason.
- **skein's page part loses the place** (`skein/crates/skein-ui-web`,
  `log.rs` and `log/`; `ui-web.md`, 7, *The view stays*), with the patches
  the view sends being inserts above: stop. That is skein's fault, to be
  reproduced in skein (`AGENTS.md`, 5). Report `Question:` with the
  evidence, the patches, and the fix you'd make in skein. Commit and push
  your *Found* first.
- **The story reads before the opening has settled,** or its fixture sits
  where the opening's stop changes what it opens on: that's the story's.
  Change nothing in the product. Say exactly what it needs:
  - a step that waits: which step, for what (a `see` of the anchor, a
    `gone` that follows a read the page part asks for), and why the
    current `gone(Loading earlier messages)` passes before it;
  - or sizes: the window, each history's code-line words and paragraph
    counts, computed from what you measured (the first page's rows and
    weight, the log's height, a reply's height in pixels).
  A temporary, uncommitted edit of the story or its fixture may show the
  need passing a run or two. Say so, and remove it.
- **Each failure is its own case:** the `long` flake and the `unanchored`
  failure may have different causes. Settle each by these options. One
  that doesn't reproduce is reported as not reproduced, with what you ruled
  out, beside the other's repair.
- **If the fix changes what a shared item means** (what a chat holds at
  open, the order of its reads, a key): also run the focused tests of
  every package that reads it (Steps, 4).

## Steps

1. **Reproduce.** Run the story by name, alone, through heavy, from the
   repository's root:

   ```sh
   TIERS_WRAPPER=heavy tools/tiers/run temper browser -- -E 'test(=browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place)'
   ```

   The runner builds the tier's bundle (with `small-window`,
   `tools/tiers/bundles.toml`), the playground's and the `temper` binary.
   It then runs nextest's `browser-temper` profile with the filter
   (`tools/tiers/design.md`, 1 and 3). Check that each run selected exactly
   one test, and record `date -u` before each. It reads and compares no CPU
   figure. Read a failure's log around its step with `grep` and `tail`,
   never whole, before anything reruns.

   If it won't fail in 10 runs, run it under a load you make on the
   machine the story runs on: CPU busy loops or `stress-ng --cpu N` beside
   it, stopped when the run ends. Never stop heavy's units or touch its
   state. If it still won't fail in 10 loaded runs, check out
   `23261749d4` detached in your slot, where it failed twice, and run it
   there. Then `git switch` back to your branch before any edit you commit.
   Say in the report what you used.

2. **See what the chat holds at open.** The story prints `kept boxes:`
   after every run, passing or failing; read it with `grep`. nextest shows
   a passing test's output only when asked: append `--success-output
   immediate` to step 1's nextest arguments, after the filter. To see more,
   you may add temporary, uncommitted steps or prints: a `KeepBox` of
   `paragraph(position_words(2))` and of `reply_end(2)` right after
   `reading_chat`'s `gone(...)` steps, or an `eprintln!` of the rows and
   weight held where `Chat::accept_activity` stops the opening, and of each
   `Act::Older` after it. Map step 76 to its `KeepBox` by counting the
   script. Remove every such edit before you commit.

3. **Repair it at its lowest place,** by acceptance 2 and the options. For
   a product fix, write the focused test row first and see it fail for the
   cause's reason. Then fix it and see it pass: the implementer's text's "a
   new test must bite". Run the narrowest test first:
   `heavy cargo nextest run -p PKG --lib -E 'test(=NAME)'`.

4. **Check.**
   - For a product fix: the story's runs of acceptance 2, with step 1's
     command, under the conditions that reproduced it.
   - For a product fix, the focused tests that hold finding-465's and
     red-82's cases, and the readers of what a chat holds at open:
     - `heavy cargo nextest run -p temper-client-domain --lib -E 'test(/pages::chat::/)'`
     - `heavy cargo nextest run -p temper-client-view --lib -E 'test(/tests::chat::/)'`
     - `heavy cargo nextest run -p temper-client-tui --lib -E 'test(/chat/)'`
     - `heavy cargo nextest run -p temper-client-world --test suite -E 'test(/^reading::/) | test(/^tui::/) | test(/^open_chat::/) | test(/^chat::/)'`

     `tui::a_tuis_fast_completed_chats_keep_the_opening_said_row` and
     `pages::chat::a_chat_opens_to_its_opening_or_a_screen_and_a_half`
     must pass. Then the focused tests of every package you changed, and
     of their direct dependents. Check each run's count: one that selected
     nothing proves nothing.
   - `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
     clippy -p <each package you changed> --all-targets -- -D warnings`.
   - `git diff origin/main -- temper/tests/client-browser` is empty.
   - Commit, then `python3 -I -B tools/citations/citations.py --base
     origin/main` (it reads `HEAD`).

5. **Write `plan.md`'s *Found*.** Replace the section's one placeholder
   line with three to eight lines. For each failure, say its case (the
   product's, the story's, skein's, or not reproduced) and what the chat
   held at open. For a product fix, give its file and how many runs passed
   after it, under which load. For the story's case, give the sizes or the
   step it needs and why, in numbers temper/tiers can apply as they stand.
   Change nothing else in `plan.md`.

6. **Commit and push.** Name the cause in the message: `red-87-reading:
   <the cause, in a few words>`. Its body says why the story began failing
   after red-82, and where the repair sits or what the story needs.

## Report

Follow the implementer's text. Also give:

- each failure's step, what the chat held at open, and the failing lines
  that show it;
- each run of the story: its `date -u` time, its outcome, how many tests it
  selected, the commit and the load;
- for a product fix: the test row you added, the rule you broke by hand,
  and the failure you saw;
- for the story's case: exactly what it needs (the window, each reply's
  code-line words and paragraphs, or the step that waits), the measurement
  behind each number, and why;
- every file you changed outside `files`, and why;
- what you ruled out.
