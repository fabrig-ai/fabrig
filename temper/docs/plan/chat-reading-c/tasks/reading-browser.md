+++
id = "reading-browser"
part = "browser"
kind = "feature"
needs = ["chat-page", "page-reading", "log-busy", "beside-store"]
files = [
  "temper/tests/client-browser/tests/browser_all/browser_pages.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/tests/task.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tests/client-browser/src/lib.rs",
  "skein/testing/skein-fake-person-browser/src/harness/script.rs",
  "skein/testing/skein-fake-person-browser/src/face.rs",
]
effort = "high"
minutes = 50
+++

# A long chat is read by scrolling in a real browser

## Goal

The whole thing, in real Chromium on a real temper with fake backends, by what the browser tier runs. The tier runs Chromium alone: the position staying put is the one thing browsers differ on, and your report says plainly what's run and what's only read.

**Your base must hold `look-round-6`:** this plan stands on it, and `fabrig/ui` has priority. Check first: `rg -n 'State::ToLogEnd' temper/crates/temper-client-view/src/pages/chat.rs`. If that finds nothing, change nothing and report `Question: look-round-6 has not reached my base; wait for the catch-up?`, recommending yes.

Acceptance, in `temper/tests/client-browser/tests/browser_all/browser_pages.rs`, beside `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back`, as few page loads as prove them:

- **Opening:** a long chat opens with its end in view, and the log's head at least one and a half screens above it, with no "Older" on the page. A short one shows whole, with its start and no status.
- **Reading back:** PgUp from the composer, with the focus held there, scrolls the conversation, and earlier lines load as the head nears. "Loading earlier messages…" shows only while a read is out.
- **The position stays:** the words in view don't move on screen when lines load above, within 1 px: `getBoundingClientRect` on a chosen line before and after a page reads in. A run with `overflow-anchor: none` injected on the log, which is what Safari gives, and one without.
- **The end comes back:** past the window's bound, the pill shows, the foot of the window reads later lines as it nears, and they stay put in the same way; a press of "Latest" from far back goes to the end.
- **Sending from far back** goes to the end and shows the words.
- **New words while reading back** don't move the view: this story exists from `look-round-6`. It stays, and it holds.

If `temper.css` needs a hunk for what a state now does, it's the least, under `/* ==== ui: chats ==== */`, and it keeps `#app > .temper-session .temper-thread > [role=log]`'s `position: relative`.

## Inputs

- `temper/docs/design/client/README.md`, 5.15.
- `skein/docs/design/ui-web.md`, section 7, *Reading a log*.
- `temper/docs/plan/chat-reading-c/notes/common.md`.
- `skein/docs/foundation/testing-strategy.md`, 8.1: the browser tier's budget is the user's.

## Owns and may change

Owns the files in the header. `skein/testing/skein-fake-person-browser/src/harness/script/growth.rs` shows the shape of the one step this task adds there; a module beside it is this task's too. No page, shell or browser-kit code gains a hook for this.

Options, each decided:

- **Fake backends:** a long chat comes from the fake's agent speaking in long pieces, or from a fixture's activity if it has one. Take what `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` takes, longer.
- **A fault below** is reproduced and fixed in the lowest tier that shows it (`skein/tests/browser` for the page part) in a commit that says so, and named in your report. If it needs another task's interface changed, it's a `Question:`.
- **Over budget** is red: does a story belong here; can it be made faster; otherwise your report's measurements go to the user.

Not this task: the journeys' prints, design text.

## Steps

0. In the harness, failing unit test first, beside `growth.rs`'s: `Step::KeepBox { find: Find, name: Box<[u8]> }`, `name` keeping `<y> <height>` in whole CSS pixels from the find's box (`BrowserFace::find_boxed`). In temper's world, `Tier::demo_without_anchoring(name)`.
1. The stories, one at a time, with `heavy cargo nextest run --profile browser-temper -p temper-client-browser-world --test browser_all -E 'test(=NAME)'`.
2. Break the rule once for the position story: with the page part's `PutView` off, it fails with `overflow-anchor: none`; restore.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`; one whole `browser_pages` run. Commit and push.

## Report

As the implementer's text asks, with: wall and CPU against the tier; and, plainly, for each of Chromium, Firefox and Safari, what's run and what's only read.

## Answers

2026-11-06. **May this task add test-only fixture hooks outside its two owned files, or should a prerequisite extend the browser harness?** Neither: no hook in temper's shell, no new request in the kit, and no prerequisite. The kit already gives a node's box, and temper's world already serves a private copy of the bundle. The acceptance stands whole; the header above now lists the files this needs.

- **Measuring, no script run:** `skein_browser::boundary::Query.boxes` returns `Seen.rect`, the box in viewport coordinates, `y` signed and whole pixels, so a line above the view is negative (`skein/docs/design/browser.md`, section 5). `Step::Taller` reads it through `BrowserFace::find_boxed` and `height`. Add `Step::KeepBox { find, name }` to the harness the same way: it fails the story on an absent find or a find with no box, and keeps `<y> <height>` for `Harness::kept`, as `Keep` does. `Request`, `Event` and the rule that `document` is the one expression stay as they are.
- **The position:** a numbered paragraph as the chosen line. `KeepBox` it, read in (`See` a line that wasn't on the page, so the read has landed), `KeepBox` it again, and compare with skein's `held`, within 1 px. A rounded `y` is what the 1 px allows.
- **The head at opening:** `KeepBox` the log (`Role::Log`) and a numbered line that the story's sizes put past one and a half screens: the line's `y` is at or above the log's `y` less one and a half times the log's height, and the end's line is in the log's box.
- **`overflow-anchor: none`, injected by the bundle:** `Tier::demo_without_anchoring(name)` is `Tier::demo` on a private bundle, made as `Tier::accounts` makes its own (`copy_bundle`, `application.clients.bundle`), with `#app > .temper-session .temper-thread > [role=log] { overflow-anchor: none !important; }` appended to the digest directory's `temper.css`. Nothing checks a bundle file against its digest. Step 2's break, `PutView` off and this story red, is what shows the rule took.
- **Budget:** the second run is one more page load of the position story alone, not of every story. The tier's 85 s a suite and 60 s a test stand.

2026-11-06. **Should the owning task repair the inherited `Act::Near` matches and give a catch-up, or may this task repair them outside its files?** Neither is left to do: catch up. Merge `origin/chat-reading-c/integration` into this branch first, and keep the `KeepBox` commit (`9eb575dc58`).

- **The two matches:** `journey/fields.rs` and `journey/originals.rs` already hold their `Act::Near` arms on the trunk (`80c3200e26`).
- **A third, found checking the trunk:** `browser_all` still failed on `PersonStep::Near` at `temper/tests/client-browser/tests/browser_all/browser_playground.rs:68`. This answer adds its one pass-through arm on the trunk, and `browser_all` compiles there.
- **If another inherited match stops `browser_all` compiling** after the catch-up, add the missing arm yourself, whatever file it is in, in a commit of its own that says so, and name it in your report. That is a compiler repair, not a change of scope: do not park on it. Anything more than an arm is still a `Question:`.

2026-11-06. **May chat-page move the unchanging context heading outside the bound log, or should page-reading gain a generic boundary identifying conversation entries?** Neither. The context stays in the log and draws only at the chat's start, and the day's label stops staying at the window's head. `chat.rs` and `tests/task.rs` are this task's now, for these two hunks alone. No page part change, no new interface, no look change, so nothing here waits on fabrig/ui.

- **Why:** the context is at the thread's head (`temper/docs/design/client/README.md`, 5.6), so it belongs where the chat starts, not at whichever line the window has reached. While a page remains above, an entry drawn above the lines is one the lines arrive below, and the first entry in view is what holds the view (`skein/docs/design/ui-web.md`, section 7). Moving it out of the log would pin a line above every long chat, which is a look nobody asked for.
- **The context:** in `conversation_head`, call `context` only when `chat.thread.lines.edges.reaches(End::After)`. A long chat opens with its head 1.5 screens up, so the context was out of view there already. It arrives with the chat's start, an insert above the words, which the page part holds.
- **The day's label has the same fault:** the context is only the first of two. `turn_time` keys the label by its date alone, so a same-day page read in above leaves it first in the log with the lines arriving below it, and with the context gone it is what the page part notes. Key it by the date and the first line under it: `Key { a: date as today, b: at.as_nanos(), ..fixed_key(46) }`. A page read in above then replaces the label at the head, and the page part goes on to the next noted entry that is still there. A label further down keeps its key, since its first line doesn't change.
- **Failing first, in the lowest tier:** in `temper-client-view`'s `tests/task.rs`, beside `latest_binds_where_the_window_let_its_end_go`: with an earlier page remaining the log draws no context, and with the chat's start held it draws it; and the day's label changes key when an earlier line of its day arrives, and keeps it when an earlier day's does. Then the position story, red today at 1,365 px, is the proof, as the acceptance says.
- **If it still moves:** another entry that stays at the head while lines arrive below it gets the same treatment in `chat.rs`, named in your report. That is no new question. `322c826760`'s status skip stays.
- **The rest stands:** the acceptance is unchanged, with everything your report lists as pending. The status clause in `skein/docs/design/ui-web.status.md`, 7, *The view stays*, is yours to remove once your skein commit lands with it. A journey that moves, a long playground chat losing its context until its start is read, is named for fabrig/ui.

2026-11-06. **Schedule a prerequisite repair of the tier and its startup under the 85 s budget, or take a new budget and a repair's scope to the user?** Neither yet, and no prerequisite: this task repairs both, in the files it owns. The figures are out of date and the budget's second question is still open, so nothing goes to the user until the steps below have run. The tier's 85 s a suite and 60 s a test stand, and the acceptance is unchanged. Everything below comes from reading the code, not from a run.

- **Catch up first:** merge `origin/chat-reading-c/integration`. It holds `browser-trim` (`9c1da9d243`, `970302c050`), which your branch does not. The run's 83.9 CPU s for the stories it did not change is from before that trim. `temper/docs/plan/browser-trim/figures.md`, *After*, has the trimmed tier at 78.6, 73.8 and 57.8 CPU s on a loaded machine, or 70.2, 65.4 and 51.5 without the Latest story. Adding the 60.7 CPU s your report gives the four chat stories makes about 112 to 131, before `browser_call_line`, which those runs did not measure. So the tier is still red, by 27 to 46 CPU s, but the room for the chat stories is 15 to 33 CPU s, not 1.
- **The startup failure:** `start_playground_chat` (`temper/tests/client-browser/src/lib.rs`) presses "Try the playground" and at once does `PersonStep::Go` to `/chats/new`, a full navigation. The press has already asked to start the playground and shown that address (`temper-client-domain`'s `route.rs`, `playground`), so the navigation can cancel an ask still out. The page then has no playground and nothing starting, which is what your report saw: "No model yet" and the account banner. That is a race, which fits it passing in the full run and failing on a loaded machine. Remove the `Go` step and let the next `see` wait for the region. The page's own move to `/chats/new` is what the later `Step::Address` reads. A race can't be run red on demand, so say that in the commit. Read the failing log's page first. If the failure comes back without the `Go`, it is a `Question:` with its logs.
- **The long story's cost, the pace:** the demo's model waits `application.demo.llm.pace.reply_ms` (1200) before each reply, so sixteen sends wait about 19 s of the story's 24.5 s wall, with the harness polling the whole page's accessibility tree through a growing log. The story is about reading, not pace: set it to 0 in the reading stories' tier, in `lib.rs` beside `with_replies`, without touching the demo's or other stories' pace.
- **The long story's cost, the nodes and keys:** every find walks the tree, and thirty paragraphs in each of sixteen replies is 480 nodes. Keep the code line that carries the bytes in every reply, and give only the replies the story reads as many paragraphs as the view needs, which also cuts the 164 page keys. Your link-target experiment was dropped only because this startup failure stopped it, so nothing has yet shown that this can't help. Measure each repair alone and keep what lowers CPU.
- **Fewer page loads:** the short story and the unanchored story are one page load each. Keep them separate unless one can prove both, since they expect different things.
- **Don't cut:** the bound crossing, the foot's read staying put and Latest from far back stay in this story. `document_bytes` is compiled into the web shell, so no test can lower it without a hook, and the spec forbids a hook. Moving them to another tier to escape the budget is raising it.
- **The verdict:** three consecutive whole-tier runs, as `browser-trim` measured, each `heavy python3 -I -B tools/tiers/run --cpu DIR temper browser` then `tools/tiers/cpu-summary DIR --budgets tools/tiers/budgets.toml --pair browser-temper`, each in a fresh private `DIR` under `target/`. All three within 85 CPU s and no test past 60 is `Done`. Otherwise it is the third question: report `Question:` with the same per-test table against `figures.md`, *After*, what each repair above saved, and the options, which are the user's. Do not reopen the repairs on that report.

2026-11-06. **Four runs without landing: go on, or stop and take it to the user?** Go on: one more run, on the answer above, which no run has tried yet. It was written after run 4 ended, and the branch's head (`13ea8595d4`) is from before it. The four runs asked four different questions, each settled and none asked again, so this is not a loop. The acceptance, the 85 s a suite and the 60 s a test stand, and this run's report ends the budget question either way. Four things the answer above left out, checked by reading and by a trial merge; nothing has been run:

- **The figures:** `figures.md` left the tree when the `browser-trim` plan left main, so after the catch-up it isn't there. Read it with `git show 9c1da9d243:temper/docs/plan/browser-trim/figures.md`.
- **The catch-up:** a trial merge has one conflict, in `browser_pages.rs`, where both sides added at the same place. Keep both: this branch's `Together` and `Stage`, then the trunk's doc comment `/// Scrolling with the browser's keys, and the view staying put.` on the item it already precedes. The trim removed code from `browser_all`, so read what the merge leaves of the stories this branch relies on, and name anything lost in your report.
- **The pace's key:** in `with_replies`, `value["application"]["demo"]["llm"]["pace"]["reply_ms"] = 0.into();`. The fake takes 0 (`temper/tests/one-process/src/review_delivery.rs` does the same). If `demo_without_anchoring` doesn't go through `with_replies`, set it there too.
- **The wall:** the `browser-temper` profile's `global-timeout` is 120 s, and that is the limit run 4 hit, not `wall_hard_s`. It is the user's and stays. A run it cancels is starved: a warning, not red, but the tests it didn't run are unobserved, and its CPU figure is not the whole tier's. So a run counts toward the verdict only if every test ran and passed, with no starvation. The trimmed tier's wall was 43 to 51 s, so the chat stories have about 70 s of wall. If a run is starved, it is the third question too, with the wall in the table.

**The verdict stands as written:** three whole-tier runs that count, each within 85 CPU s with no test past 60, is `Done`. Anything else is `Question:` with the measurements. A startup failure that comes back without the `Go` is a `Question:` with its logs.
