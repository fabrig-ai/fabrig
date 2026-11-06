+++
id = "reproduce"
part = "reproduce"
kind = "docs"
needs = []
files = [
  "temper/docs/plan/ui-pgup/driver/**",
  "temper/docs/plan/ui-pgup/reproduced.md",
]
effort = "high"
minutes = 75
+++

# What each step does today

## Goal

Before anything changes, find out what PgUp, PgDn, a wheel and "Latest ↓" do on a real page in real Chromium while a fake model writes, and as its turn ends. You change no product code. You write a driver and report what it saw, step by step. The fix tasks read your report, and `proof` runs your driver again on the fixed code.

Acceptance:

- `temper/docs/plan/ui-pgup/driver/pgup.py` runs every step below, whole, at 1440 and at 420, on this branch's base.
- `temper/docs/plan/ui-pgup/reproduced.md` says **holds** or **fails** for every step at both widths, with the scroll offsets, and, for each failure, its cause, by file and function, as far as you can show it.
- A step that holds is said to hold. Nothing is left "not checked" without its reason.
- No file outside `temper/docs/plan/ui-pgup/` changes.

## Inputs

- `temper/docs/plan/ui-pgup/notes/common.md`, then `temper/docs/plan/ui-pgup/inputs/request.md`: its *What a person does, and must see* is what you drive.
- `temper/docs/design/draft/chat-reading.md`, section 5.
- `skein/docs/design/ui-web.md`, section 7: *A log*, and *Reading a log*.
- `temper/docs/design/client/browser.md`, *Reading the conversation*.
- `jig/docs/design/client/domain/client.md`, section 10, *Live, then committed*.

## Owns and may change

- **Owns:** `temper/docs/plan/ui-pgup/driver/**` and `temper/docs/plan/ui-pgup/reproduced.md`.
- **Reads, changes nothing in:** everything else.
- **Options:**
  - The driver is Python's standard library, run with `python3 -I -B`. Python here has no websocket module: load `Pipe` from `temper/docs/plan/bundle-loading/driver/load.py` with `runpy`, as `render.py` beside it does. It speaks CDP over `--remote-debugging-pipe` and takes Chromium from `SKEIN_TEST_CHROMIUM`. If `Pipe` doesn't fit, copy what you need into your driver and say so.
  - If the fake box can't be scripted as the steps need, use the nearest script that still has pieces arriving, a call and a turn's end, and say what differs.
  - If a step can't be made to fail or hold cleanly after three tries on a fresh box, report what you saw, with its counts, as **unsettled**. Don't keep rerunning it.

## Steps

### The box

1. Build through heavy: `heavy cargo build -p temper`, and the bundle by the cargo lines of `temper/scripts/bundle.sh` with `--loopback`'s feature, each through heavy, then its bundling lines. `temper/docs/design/protocol/setup.md`, 7.5, says what the bundle is for.
2. Start a fresh box the way `temper/tests/client-browser/src/configuration.rs`, `demo`, does: `temper init --box` with its flags, loopback ports you reserve, and `application.clients.bundle` set to your bundle. Open a chat the way `temper/tests/client-browser/src/lib.rs`, `start_playground_chat`, does.
3. Script the model in `temper.json` under `application.demo.llm`, as `with_replies` and `configuration::demo_asked` beside it do:
   - **Pieces:** a line of `{"pieces":[{"text":"...","hold":null}, ...]}`, each piece its own delivery. Write paragraphs and at least one list, separated by blank lines, long enough that one reply is several screens.
   - **A call in the writing:** a reply of a turn with pieces and a call (`{"call":"read","arguments":{"path":"src/main.rs"}}`, `"finish":"tool_calls"`), then a turn with more pieces (`"finish":"stop"`), so the view meets the next call row and the next line.
   - **Pace:** `application.demo.llm.pace.reply_ms`, the gap between pieces. Read `skein/crates/skein-fake-llm-domain/src/pieces.rs` to confirm that gap, and say in your report what you found.
4. Give the chat enough words to scroll before the reply starts: send several messages first, as `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` does in `temper/tests/client-browser/tests/browser_all/browser_pages.rs`. Read that story for the shape only: it is a tier story, and you don't run it.

### The driver

5. Set each width with CDP's `Emulation.setDeviceMetricsOverride`: 1440 × 900 and 420 × 900. `--window-size` won't go below 500. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs the driver.
6. Act as a person does, through the browser's input, never through page script:
   - **Keys:** `Input.dispatchKeyEvent`, `rawKeyDown` then `keyUp`, with `key` and `code` `PageUp` or `PageDown`, and `windowsVirtualKeyCode` 33 or 34.
   - **Wheel:** `Input.dispatchMouseEvent`, type `mouseWheel`, over the log.
   - **"Latest ↓":** `Input.dispatchMouseEvent`, a pointer's press and release at the pill's centre. Not `element.click()`, which moves no focus, and focus is what step 3 checks.
   - **Typing:** `Input.insertText` in the composer, then a caret set back a few characters.
7. Install a per-frame sampler in the page (`requestAnimationFrame`), storing the time, the log's `scrollTop`, `scrollHeight` and `clientHeight`, whether it has `data-log-away`, whether the pill is displayed, `document.activeElement`'s tag, and the composer's value and `selectionStart`. One delivery's collapse lasts only until a read answers, so a value taken at each step alone won't see it: report each step's smallest `scrollHeight` too.
8. Measure **the words being read:** at a press, take the first paragraph in the log whose top is inside the view, and its text and `getBoundingClientRect().top`. Afterwards find the paragraph with that text again, by its text, since its element may have been made anew, and report how far it moved in pixels. Moved is the number that decides **holds**.

### The steps, at each width, focus in the composer with typed words throughout

9. **1. While the agent writes,** at a pace slow enough to watch (about 300 ms a piece):
   - From the foot, following: PgUp. Report the offset's change against a screen less a line.
   - Hold across at least three deliveries, one of them a call row and one the next line. Moved stays 0, and "Latest ↓" shows.
   - PgUp again: further up.
   - PgDn until the foot: the view follows the live words, and "Latest ↓" goes.
   - PgUp twice, then press "Latest ↓": to the end, following.
10. **1, with a wheel** in place of PgUp: the same holds, and PgDn or more wheel back to the foot follows again. Say where the key path and the wheel path differ.
11. **The race, fast.** The scroll event comes up to a frame after a key's scroll, so 300 ms a piece hides what a real model's stream shows. At `reply_ms` of 20 or less, from the foot, press PgUp at least 30 times, returning to the foot each time, and report how many presses were lost: the view back at the foot, or never away, after the next delivery. The same for PgDn reaching the foot: how many times it reached the foot and did not follow. The same for the wheel, for the comparison. Report the counts at each width.
12. **2. After the turn ends:**
   - PgUp while it writes, then let the turn end. Moved stays 0, across the ending. Report the sampler's smallest `scrollHeight` across it, and whether the words left the page at any frame.
   - PgUp at the moment the turn ends: as the writing mark (`▍`) goes. Then PgUp just after, about 200 ms on.
   - After the turn has ended: "Latest ↓" still shows where the view is up, and the view stays up. Send a message: the view goes to the foot, as the design says.
13. **3. Focus:** after every action above, the focus is in the composer, with the typed words and the caret where they were. Report any action that took it elsewhere, and where it went. The press on "Latest ↓" is one of those actions.

### The report

14. Write `temper/docs/plan/ui-pgup/reproduced.md`: a table per width, a row per step above, with `scrollTop` before and after, `scrollHeight`, `clientHeight`, moved, pill, focus, words and caret, and **holds**, **fails** or **unsettled**. Then the race's counts. Then, for each failure, the cause you can show. Read these first:
    - `skein/crates/skein-ui-web/src/page.rs`, `key` and `scrolled`, and `page/web.rs`, `scroll_screen`: a key's scroll changes `follow_log` only when the browser's scroll event arrives, so a delivery before it can restore the foot.
    - `jig/crates/jig-client/src/thread/model.rs`, `Thread::run`: a run state with no attempt clears the live turns at once, before the committed turn's page has been read, while `Thread::end_run` waits for it.
    - `temper/crates/temper-client-view/src/pages/chat.rs`, `line_key` and `block_key`: a provisional line and its committed first block share a key.
    - `skein/crates/skein-ui-web/src/page.rs`, `press`: the pill takes the focus on a pointer's press and hides, and nothing puts it back.

    These are guesses from reading. Report what the page does, whichever way it goes. Say the order the run's end and the committed page arrived in, if you can see it.
15. Check: `python3 -I -B tools/citations/citations.py --base origin/main`. `tests/boundary/tests/outward.rs` doesn't read a plan's directory, so read your driver and report yourself: no path or address of a machine, and nothing private. Commit and push.

## Report

The first line says which steps fail. Then the file you wrote, what you ran it on (the base commit), the machine's Chromium version, the pace you found, and your minutes.
