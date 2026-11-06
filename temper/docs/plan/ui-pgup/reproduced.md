Fails: reading across the call and the turn ending, and focus after pressing “Latest ↓”, at both widths. Fast wheel returns also leave the view behind new words in the cases detailed below.

# PgUp before the fixes

The driver is `temper/docs/plan/ui-pgup/driver/pgup.py`. The measured product is base commit `301458e0f3517dfbbaa64e0e4c562a68f6e79a1c`, also the initial head of `ui-pgup/reproduce`. No product code changed. Chromium reports product `Chrome/147.0.7727.101` through its browser version API; its headless user agent reports `HeadlessChrome/147.0.0.0`. CDP device metrics, not the window size, set 1440 × 900 and 420 × 900.

## Setup and observation (spec steps 1–8)

The native binary, loopback Wasm client and bundler were built separately through `heavy`, with `TEMPER_BUNDLE=none`. Bundling used the non-playground lines of `temper/scripts/bundle.sh`, including loading HTML/CSS, both application stylesheets, files, `/bundle/`, and the loopback client feature. Each width and pace got a fresh `temper init --box` deployment. The driver reserves and substitutes loopback ports in both configuration documents, sets `application.clients.bundle`, consumes the box's printed browser link without retaining it in evidence, and follows Try the playground → Chats → + New chat. It sends six seed messages and waits for their scripted answers before starting the measured replies. No browser story or tier ran.

There is a necessary departure from the proposed setup: this base's deployment parser accepts neither `application.demo.llm.pace` nor pieces. `temper/crates/temper-fake/src/demo.rs::read` uses the deployment turn parser, and `temper/crates/temper-fake/src/turn.rs::line` accepts pieces only with standalone credentials. The driver therefore starts the supported `temper fake llm --document` peer. The box keeps a valid minimal demo configuration; its demo key path names the standalone peer's generated private file, and the agent's demo endpoint names that peer. The driver never reads that key. Other stock endpoint definitions remain present for startup validation; this journey uses the demo model.

The slow scripts have sixteen paragraph pieces before `read {"path":"src/main.rs"}`, `finish: "tool_calls"`, then sixteen more pieces with `finish: "stop"`. Paragraphs have blank lines, and piece 3 includes a two-item list. The first reply is several screens at both widths. Ending cases use a sixteen-piece reply without the call. The scripts exercise both a call row and the next provisional line; they do not substitute a static page.

**Pace:** 300 ms for slow observations; 20 ms for races. Despite the configuration document describing first-piece latency, `temper/crates/temper-fake/src/llm.rs::Document::limits` sets both latency bounds to `reply_ms`, and `skein/crates/skein-fake-llm-domain/src/domain.rs::fire` rearms after every `State::Streaming` delivery. `pieces.rs::fire` emits each intermediate piece; the last piece arrives in the terminal reply containing the joined text. Thus the gap applies between pieces, including the final reply. A released hold also rearms that timer.

All acts use CDP input: paired `rawKeyDown`/`keyUp` with PageUp/33 or PageDown/34; wheel events over the log; pointer press/release at Latest's centre; `Input.insertText` followed by five ArrowLeft presses. Page script observes only. It neither scrolls nor focuses nor clicks. A page binding reports writing-mark removal, and Python sends the boundary PgUp through CDP. JavaScript dialogs are accepted so leaving a page cannot strand the pipe.

The animation-frame sampler records time, scroll geometry, `data-log-away`, pill display, active tag, composer value/caret, writing mark, call summaries, and the reading paragraph's position and presence. It identifies the first visible **prose paragraph**, excluding attribution labels such as “Temper”. It finds that text again after a change; repeated scripted prose is disambiguated by its occurrence among paragraphs with exactly that text, never by its former element. The slow paragraphs are unique; the occurrence is needed in the repeated fast trials. Holds require zero final movement, zero maximum sampled movement, and no frame where that paragraph disappeared.

Each call hold ends only after next-line piece 003, spanning the call row and four pieces of the next line.

The draft is **“Keep these draft words intact.”**, caret **25**. In the tables `D` means that exact value; `TA` means `TEXTAREA`. Sending intentionally consumes the composer, so its expected result is empty, caret 0. Positive moved means the paragraph moved down on the screen, as intended when scrolling up. Movement during an intentional scroll or following is expected; reading holds require 0 px. Heights and offsets are CSS pixels.

Fast trials prepare the draft while the second model piece is held at `race-ready`, then release it and vary the input phase by 0–19 ms in the 20 ms delivery cycle. This prevents draft preparation from using up the stream. Each input path starts a fresh chat with six seed exchanges; between paths CDP navigates to `/chats/new`, waits for its heading, composer and Live link, and accepts any unload dialog. Typing waits for Send to become enabled before sending Enter. Each trial starts a fresh streamed turn; downward trials begin 200 px above the foot so one PgDn can reach it. A capturing input observer verifies that the browser received the input while writing. The sampler then waits for an observed delivery after that input. Exploratory trials that did not ensure this were discarded, rather than reported as successful races.

## 1440 × 900: slow steps and focus

| Step | scrollTop | scrollHeight | minimum height | clientHeight | moved | pill | focus | words | caret | result |
|---|---:|---:|---:|---:|---:|---|---|---|---|---|
| 1. PgUp from following | 2880 → 2174 | 3610 → 3610 | 3610 | 730 | 706 | N→Y | TA→TA | D→D | 25→25 | **holds** |
| 1. Hold across call + next line | 2174 → 3824 | 3610 → 4554 | 1057 | 730 | -1650 | Y→N | TA→TA | D→D | 25→25 | **fails** |
| 1. PgUp again | 3824 → 3118 | 4554 → 4554 | 4554 | 730 | 706 | N→Y | TA→TA | D→D | 25→25 | **holds** |
| 1. PgDn until foot | 3118 → 3992 | 4554 → 4722 | 4554 | 730 | -874 | Y→N | TA→TA | D→D | 25→25 | **holds** |
| 1. Follow incoming words | 3992 → 4496 | 4722 → 5226 | 4722 | 730 | -504 | N→N | TA→TA | D→D | 25→25 | **holds** |
| 1. Pointer Latest after PgUp ×2 | 3084 → 4664 | 5394 → 5394 | 5394 | 730 | -1580 | Y→N | TA→BODY | D→D | 25→25 | **fails (focus)** |
| 1/wheel. Wheel up (−600) | 8495 → 7895 | 9225 → 9225 | 9225 | 730 | 600 | N→Y | TA→TA | D→D | 25→25 | **holds** |
| 1/wheel. Hold across call + next line | 7895 → 9439 | 9225 → 10169 | 6672 | 730 | -1544 | Y→N | TA→TA | D→D | 25→25 | **fails** |
| 1/wheel. Wheel up again | 9439 → 8839 | 10169 → 10169 | 10169 | 730 | 600 | N→Y | TA→TA | D→D | 25→25 | **holds** |
| 1/wheel. PgDn until foot | 8839 → 9607 | 10169 → 10337 | 10169 | 730 | -768 | Y→N | TA→TA | D→D | 25→25 | **holds** |
| 1/wheel. Follow incoming words | 9607 → 10111 | 10337 → 10841 | 10337 | 730 | -504 | N→N | TA→TA | D→D | 25→25 | **holds** |
| 1/wheel. Pointer Latest after wheel ×2 | 8911 → 10279 | 11009 → 11009 | 11009 | 730 | -1368 | Y→N | TA→BODY | D→D | 25→25 | **fails (focus)** |
| 2. PgUp while writing | 13942 → 13236 | 14672 → 14672 | 14672 | 730 | 706 | N→Y | TA→TA | D→D | 25→25 | **holds** |
| 2. Hold across ending | 13236 → 14278 | 14672 → 15008 | 12422 | 730 | -1042 | Y→N | TA→TA | D→D | 25→25 | **fails** |
| 2. Post-end state after preceding snap† | 14278 → 14278 | 15008 → 15008 | 15008 | 730 | 0 | N→N | TA→TA | D→D | 25→25 | **fails** |
| 2. Send a message | 14278 → 14387 | 15008 → 15117 | 15008 | 730 | -109 | N→N | TA→TA | D→empty | 25→0 | **holds** |
| 2. PgUp at mark removal | 17210 → 16504 | 17940 → 17940 | 17940 | 730 | 706 | N→Y | TA→TA | D→D | 25→25 | **holds** |
| 2. PgUp about 200 ms later | 16504 → 15798 | 17940 → 17940 | 17940 | 730 | 706 | Y→Y | TA→TA | D→D | 25→25 | **holds** |

PgUp moved exactly 730 − 24 = 706 px. The Latest row starts immediately before the pointer press, after the two setup scrolls. Both Latest destinations **hold**; their overall rows fail only focus.

- `1/key/hold-call-next-line`: maximum sampled movement 1650 px; paragraph absent in 3/87 frames; 5 sampled tail states. Paragraph: “slow-key paragraph 007.”
- `1/wheel/hold-call-next-line`: maximum sampled movement 1544 px; paragraph absent in 2/84 frames; 5 sampled tail states. Paragraph: “slow-wheel paragraph 008.”
- `2/hold-across-end`: maximum sampled movement 1042 px; paragraph absent in 2/46 frames; 2 sampled tail states. Paragraph: “ending-held paragraph 006.”

The boundary PgUp entered 3.4 ms after observed mark removal; the next PgUp began 202.4 ms after it.

## 420 × 900: slow steps and focus

| Step | scrollTop | scrollHeight | minimum height | clientHeight | moved | pill | focus | words | caret | result |
|---|---:|---:|---:|---:|---:|---|---|---|---|---|
| 1. PgUp from following | 5119 → 4420 | 5842 → 5842 | 5842 | 723 | 699 | N→Y | TA→TA | D→D | 25→25 | **holds** |
| 1. Hold across call + next line | 4420 → 6700 | 5842 → 7423 | 1376 | 723 | -2280 | Y→N | TA→TA | D→D | 25→25 | **fails** |
| 1. PgUp again | 6700 → 6001 | 7423 → 7423 | 7423 | 723 | 699 | N→Y | TA→TA | D→D | 25→25 | **holds** |
| 1. PgDn until foot | 6001 → 6996 | 7423 → 7719 | 7423 | 723 | -995 | Y→N | TA→TA | D→D | 25→25 | **holds** |
| 1. Follow incoming words | 6996 → 7882 | 7719 → 8605 | 7719 | 723 | -886 | N→N | TA→TA | D→D | 25→25 | **holds** |
| 1. Pointer Latest after PgUp ×2 | 6484 → 8178 | 8605 → 8901 | 8605 | 723 | -1694 | Y→N | TA→BODY | D→D | 25→25 | **fails (focus)** |
| 1/wheel. Wheel up (−600) | 14814 → 14214 | 15537 → 15537 | 15537 | 723 | 600 | N→Y | TA→TA | D→D | 25→25 | **holds** |
| 1/wheel. Hold across call + next line | 14214 → 16395 | 15537 → 17118 | 11071 | 723 | -2181 | Y→N | TA→TA | D→D | 25→25 | **fails** |
| 1/wheel. Wheel up again | 16395 → 15795 | 17118 → 17118 | 17118 | 723 | 600 | N→Y | TA→TA | D→D | 25→25 | **holds** |
| 1/wheel. PgDn until foot | 15795 → 16690 | 17118 → 17413 | 17118 | 723 | -895 | Y→N | TA→TA | D→D | 25→25 | **holds** |
| 1/wheel. Follow incoming words | 16690 → 17577 | 17413 → 18300 | 17413 | 723 | -887 | N→N | TA→TA | D→D | 25→25 | **holds** |
| 1/wheel. Pointer Latest after wheel ×2 | 16377 → 17872 | 18595 → 18595 | 18595 | 723 | -1495 | Y→N | TA→BODY | D→D | 25→25 | **fails (focus)** |
| 2. PgUp while writing | 24213 → 23514 | 24936 → 24936 | 24936 | 723 | 699 | N→Y | TA→TA | D→D | 25→25 | **holds** |
| 2. Hold across ending | 23514 → 24804 | 24936 → 25527 | 21028 | 723 | -1290 | Y→N | TA→TA | D→D | 25→25 | **fails** |
| 2. Post-end state after preceding snap† | 24804 → 24804 | 25527 → 25527 | 25527 | 723 | 0 | N→N | TA→TA | D→D | 25→25 | **fails** |
| 2. Send a message | 24804 → 24913 | 25527 → 25636 | 25527 | 723 | -109 | N→N | TA→TA | D→empty | 25→0 | **holds** |
| 2. PgUp at mark removal | 29776 → 29077 | 30499 → 30499 | 30499 | 723 | 699 | N→Y | TA→TA | D→D | 25→25 | **holds** |
| 2. PgUp about 200 ms later | 29077 → 28378 | 30499 → 30499 | 30499 | 723 | 699 | Y→Y | TA→TA | D→D | 25→25 | **holds** |

PgUp moved exactly 723 − 24 = 699 px. The Latest row starts immediately before the pointer press, after the two setup scrolls. Both Latest destinations **hold**; their overall rows fail only focus.

- `1/key/hold-call-next-line`: maximum sampled movement 2280 px; paragraph absent in 2/88 frames; 5 sampled tail states. Paragraph: “slow-key paragraph 011.”
- `1/wheel/hold-call-next-line`: maximum sampled movement 2181 px; paragraph absent in 2/88 frames; 5 sampled tail states. Paragraph: “slow-wheel paragraph 011.”
- `2/hold-across-end`: maximum sampled movement 1290 px; paragraph absent in 2/46 frames; 2 sampled tail states. Paragraph: “ending-held paragraph 010.”

The boundary PgUp entered 3.1 ms after observed mark removal; the next PgUp began 202.5 ms after it.

† The post-end row is a consequence of the preceding lost reading position: it is already at the foot and the pill correctly stays hidden there. It does **not** demonstrate an independent pill-display fault when reading up. Separate fresh-box ending controls held the position and post-end pill at both widths:

| Step | scrollTop | scrollHeight | minimum height | clientHeight | moved | pill | focus | words | caret | result |
|---|---:|---:|---:|---:|---:|---|---|---|---|---|
| 1440 control: 2/hold-across-end | 2006 → 2006 | 3442 → 3778 | 3442 | 730 | 0 | Y→Y | TA→TA | D→D | 25→25 | **holds** |
| 1440 control: 2/ended-stays-up | 2006 → 2006 | 3778 → 3778 | 3778 | 730 | 0 | Y→Y | TA→TA | D→D | 25→25 | **holds** |
| 420 control: 2/hold-across-end | 4124 → 4124 | 5546 → 6137 | 5546 | 723 | 0 | Y→Y | TA→TA | D→D | 25→25 | **holds** |
| 420 control: 2/ended-stays-up | 4124 → 4124 | 6137 → 6137 | 6137 | 723 | 0 | Y→Y | TA→TA | D→D | 25→25 | **holds** |

Those steps **hold** in the controls. A further slow control also held across the call at 0 px for both key widths and for the desktop wheel. The failures above therefore depend on the handoff ordering; a replacement can hold, and a visible missing-text interval can destroy it. These are measured holds, not assumed failures.

## Fast race counts (spec step 11)

Each row is thirty presses at 20 ms per piece. All 240 inputs were observed while writing, all 240 had a subsequent sampled streaming delivery, and all retained focus/D/25. No trial was counted as a successful race merely because the turn had ended.

“Reached” counts the foot as it existed at the browser input, including a later sampled position equal to or beyond that former foot. “Current” is the stricter count where a post-input observation also meets the concurrently growing height. Lost up means the later observation is back at the foot/never away, after at least one observed streaming delivery. This includes an ending that defeats an upward scroll which survived the first delivery. Lost down requires the first subsequent delivery and the later observation to agree: it has reached the former foot and remains away. Native wheel movement can cross the former foot between frames, so the distinction is an explicit inference from positions, not a claim of an unobserved current-foot frame. A press that never reaches even the former foot is reported separately, not counted as a follow success.

| Width | Path | Presses | Reached former foot | Current-foot observation | Missed following / lost up | Never reached former foot | Focus failures | Result |
|---:|---|---:|---:|---:|---:|---:|---:|---|
| 1440 | key-up | 30 | — | — | 0 | — | 0 | **holds** |
| 1440 | key-down | 30 | 30 | 30 | 0 | 0 | 0 | **holds** |
| 1440 | wheel-up | 30 | — | — | 0 | — | 0 | **holds** |
| 1440 | wheel-down | 30 | 29 | 23 | 6 | 1 | 0 | **fails** |
| 420 | key-up | 30 | — | — | 0 | — | 0 | **holds** |
| 420 | key-down | 30 | 30 | 30 | 0 | 0 | 0 | **holds** |
| 420 | wheel-up | 30 | — | — | 1 | — | 0 | **fails** |
| 420 | wheel-down | 30 | 27 | 17 | 10 | 3 | 0 | **fails** |

All zero counts above **hold for the specified lost-press criterion**. They do not assert that a slow committed handoff holds; the slow rows show it failing. The final downward-wheel failures were already away after a streaming delivery. The phone wheel-up failure survived that delivery, then lost its position at the ending; it is included in the lost-up count rather than hidden by a first-delivery-only oracle.

The per-trial JSON includes every input, before/immediate/next-delivery/after geometry, paragraph text and occurrence, moved distance, minimum height, frame samples, and draft/caret. The ranges below summarize those thirty measurements; intentional movement and following are included in moved. An early PgUp can clamp at the head rather than travel a whole screen, and native wheel scrolling can precede its main-thread input callback.

| Width | Path | scrollTop before range | scrollTop after range | scrollHeight before range | minimum height range | clientHeight | moved range | pill after (Y/N) | words/caret |
|---:|---|---:|---:|---:|---:|---:|---:|---|---|
| 1440 | key-up | 462–35306 | 0–34600 | 1192–36036 | 1192–36036 | 730 | 462–706 | 30/0 | D/25 |
| 1440 | key-down | 262–35106 | 1427–36270 | 1192–36036 | 1192–36036 | 730 | -1165–-270 | 0/30 | D/25 |
| 1440 | wheel-up | 462–35306 | 0–34706 | 1192–36036 | 1192–36036 | 730 | 0–600 | 30/0 | D/25 |
| 1440 | wheel-down | 262–35106 | 792–35306 | 1192–36036 | 1192–36036 | 730 | -965–0 | 7/23 | D/25 |
| 420 | key-up | 916–49810 | 217–49111 | 1639–50533 | 1639–50533 | 723 | 699–699 | 30/0 | D/25 |
| 420 | key-down | 716–49610 | 1373–50267 | 1639–50533 | 1639–50533 | 723 | -1521–-245 | 0/30 | D/25 |
| 420 | wheel-up | 916–49810 | 316–49210 | 1639–50533 | 1639–50533 | 723 | -864–600 | 29/1 | D/25 |
| 420 | wheel-down | 716–49610 | 1373–49810 | 1639–50533 | 1639–50533 | 723 | -1521–-70 | 13/17 | D/25 |

For example, the first failed downward-wheel trial at each width:

- 1440, trial 15: input top/height/client = 17083/18013/730, former foot 17283; next delivery 17283/18343, away=true; later 17283/18673, away=true, writing=true. No key or wheel focus loss occurred.
- 420, trial 9: input top/height/client = 14204/15127/723, former foot 14404; next delivery 14404/15172, away=true; later 14404/16448, away=true, writing=false. No key or wheel focus loss occurred.

The key path explicitly writes a current DOM scroll position; the wheel follows Chromium’s native scroll scheduling. The failed wheel returns reach a stale foot or fail to catch the growing one, while the page’s later measurement keeps following off. The traces demonstrate that difference; they do not expose Chromium’s compositor internals. No fast upward-key loss was observed in these thirty trials at either width.


## What the failures show

**The disappearing live text is visible, not just a changed offset.** During each failed handoff the selected paragraph leaves the DOM for sampled frames, the log's height shrinks, `data-log-away` goes, and the pill disappears. The replacement text returns with the log following its foot. The traces retain the intervening frames, not just the before/after readings.

`jig/crates/jig-client/src/thread/model.rs::Thread::run` clears `live` immediately when the attempt changes, including a run state with no attempt, and returns early for no attempt. This path does not wait for the committed activity page. `temper/crates/temper-client-domain/src/pages/chat.rs::Chat::run_changed` feeds run-state deliveries to it. The later `Thread::end_run` is guarded by `self.run.is_none() && self.thread.loaded >= self.thread.committed` in `Chat::told`, but that guard cannot protect text already cleared by `Thread::run`.

The scroll consequence is explained by `skein/crates/skein-ui-web/src/page/web.rs::Mounted::put_view`: it can restore only a retained entry still in the log. During the gap those entries are gone; the browser clamps the position to the shrunken foot. `page.rs::Page::scrolled` treats that foot as following again. A subsequent apply uses `Page::finish_logs` to issue `ScrollEnd`, so the committed page carries the view to its new foot. This explains both the lost paragraph and the pill going away without a person's end request.

The observed DOM order is live text → missing text/shrunken log → committed text/call row → next live line, and at the final ending, live text → missing text → committed text. The writing mark can still be present elsewhere during the gap. The driver does not decode the link's wire payloads, so the exact ordering of a RunState delivery and an ActivityPage answer is **not directly observed**; the early-clear source path is consistent with that DOM order. The report does not claim packet-level proof.

**The key alias is present, but not independently shown to cause a failure.** In `temper/crates/temper-client-view/src/pages/chat.rs::line_key`, provisional and committed turns share `(kind:40, attempt, number)`. `block_key(key, 0)` is that same key, and `committed` uses it for the first block. That cannot preserve an entry across a period when the whole live turn is absent. Controls also held at 0 px through replacement, so these observations do not establish that the alias by itself needs changing.

**Keys and the wheel share the delayed-follow decision.** `skein/crates/skein-ui-web/src/page.rs::Page::key` emits `ScrollScreen` without changing `follow_log`; `page/web.rs::Mounted::scroll_screen` writes `scrollTop` by `clientHeight - lineHeight`. The wheel uses Chromium's native scrolling instead. Both eventually reach `Page::scrolled`; a render before that event can still use the old follow state. The measured key races above must be read as their observed counts, not proof that this source-level timing window is absent. The wheel's native/compositor scroll can also reach a former foot while text grows before a frame records it there; those cases are identified separately above.

**Latest returns to the end and loses focus.** All four slow pointer presses reached the foot and hid the pill; each ended with `document.activeElement.tagName == "BODY"`. The draft and caret remained D/25. `skein/crates/skein-ui-web/src/page.rs::Page::press` handles `State::ToLogEnd` with end requests, log attributes and `ScrollEnd`, then forwards the press. It emits no focus restoration. In `temper/crates/temper-client-domain/src/pages/chat.rs::Chat::act`, `Act::Latest` only calls `request_end`, while `Act::FocusComposer` and `Act::Send` advance `composer_focus`. The pointer can focus the button before its hiding leaves focus on the body. This is why a scripted `element.click()` would have been an inadequate probe. No key or wheel action in the retained trials lost the composer focus, draft or caret.

## Repeating the measurements

Build the same inputs through `heavy`:

```sh
TEMPER_BUNDLE=none heavy cargo build -p temper
TEMPER_BUNDLE=none heavy cargo build --target wasm32-unknown-unknown --features loopback-navigation -p temper-client-shell-web
TEMPER_BUNDLE=none heavy cargo build -p skein-ui-bundle
```

Then run the bundling command's non-Cargo lines from `temper/scripts/bundle.sh` for that loopback Wasm and a new bundle directory, followed by:

```sh
python3 -I -B temper/docs/plan/ui-pgup/driver/pgup.py --bundle BUNDLE --out NEW-OUT
```

`SKEIN_TEST_CHROMIUM` supplies Chromium through the existing `Pipe` loaded with `runpy` from `temper/docs/plan/bundle-loading/driver/load.py`; no copy or websocket package is needed. The default runs every slow and fast step at both widths. `--mode slow`, `--mode fast`, `--mode ending` and `--width` allow focused investigation. Output includes full frame/trial JSON, screenshots, browser version and wall timing. Temporary boxes, profiles and generated secrets are removed. Tracked evidence below is compressed from those JSON observations and contains no sign-in link, credential, machine path or machine address.

## Retained evidence and time

- [1440 slow frames and rows](driver/evidence/1440-slow.json.gz), [420 slow frames and rows](driver/evidence/420-slow.json.gz).
- [1440 fast trials and frames](driver/evidence/1440-fast.json.gz), [420 fast trials and frames](driver/evidence/420-fast.json.gz).
- [Holding controls](driver/evidence/holding-controls.json.gz), [browser version](driver/evidence/browser.json), [manifest](driver/evidence/manifest.json).
- Holding while words arrive: [1440](driver/evidence/1440-holding-control.png), [420](driver/evidence/420-holding-control.png).
- Failed call handoffs: [1440/key](driver/evidence/1440-key-after-handoff.png), [1440/wheel](driver/evidence/1440-wheel-after-handoff.png), [420/key](driver/evidence/420-key-after-handoff.png), [420/wheel](driver/evidence/420-wheel-after-handoff.png).

The successful whole baseline capture took 514.82 s. Its slow rows are retained above. Final fast measurements rechecked every path after correcting fresh-chat navigation, waiting for Send, and disambiguating repeated prose; desktop and phone captures are retained separately. Earlier invalid setup/race attempts are excluded. The final driver has no product dependency changes and adds no browser story or tier test.

Implementation and driver exploration: 17:24:07–18:29:18 UTC, 65.2 minutes, including builds and exploratory captures. Final checking began at 18:29:18 UTC; its duration and repository checks are recorded in the run report. These are phase wall times; report assembly overlapped the browser sampling.
