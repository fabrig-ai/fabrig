+++
id = "reading-one-test"
part = "pages"
kind = "feature"
needs = ["stories-named"]
files = ["temper/tests/client-browser/tests/browser_all/browser_pages.rs", "temper/tests/client-browser/tests/browser_all/browser_call_line.rs", "temper/tests/client-browser/tests/browser_all/browser_model_picker.rs", "temper/tests/client-browser/tests/browser_all/main.rs", "temper/tests/client-browser/src/reading.rs"]
effort = "medium"
minutes = 80
+++

## Goal

All of this plan's edits to `browser_pages.rs`. Reading's four tests become one, on one `temper box`, one browser and three chats. The call line's test becomes steps of test 2's, and the model picker's becomes steps of test 4 and 5's. The tier goes from 18 tests to 13, and `welcome-step` takes it to 12.

Acceptance:

- `a_chat_opens_at_its_end_and_reads_back_keeping_its_place` replaces `a_long_chat_opens_at_its_end_and_reads_back_from_the_composer`, `earlier_words_stay_put_without_browser_scroll_anchoring`, `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` and `a_short_chat_opens_whole_without_a_reading_status`.
- `browser_call_line.rs` and `browser_model_picker.rs` are gone, with their `mod` lines. Their steps are in `history_keeps_the_chat_focus_and_filter_across_reload_and_back` and `enter_starts_one_chat_and_the_composer_keeps_the_focus`.
- Every step and measure of those six stays, in its story's order (`notes/common.md`, *Nothing is lost*). The exceptions are what this spec names. The report lists them.
- A failed step or measure of a merged test names its story (`Stories`, and each measure's message).
- `overflow-anchor: none` holds only within story `unanchored`, and two measures show it came and went.
- `heavy cargo check -p temper-client-browser-world --tests` passes. **The browser tests aren't run**; the report says so.
- `heavy cargo nextest run -p temper-client-browser-world --lib` passes, with `reading.rs`'s tests.

## Inputs

- `temper/docs/design/client/browser.md`, 4.3: *The tier's window*, *A chat's history*, *Anchoring within a step*, *A test of several stories*; 4.6: 2, 4, 14 and the first line of its *Not in the tier*.
- `temper/docs/plan/browser-cap/notes/common.md` and `notes/contracts.md`.
- `temper/docs/plan/browser-cap/plan.md`, *What was decided*: why three chats, why the long chat's replies shrink, why the call line goes to test 2.

## Owns and may change

Owns the header's files.

May change, for these and nothing else:
- `temper/tests/client-browser/src/lib.rs`: `Tier::demo_without_anchoring` becomes `Tier::reading`; `Tier::with_replies` may go private or change its argument's type, as it then has one caller; `Tier::with_second_model` is new; `mod reading;` and `pub mod`/`pub use` lines. `start_chat` stays, as `browser_sign_in.rs` uses it. `Tier::accounts_sign_in`, `account_provider` and `src/issuer.rs` are `welcome-step`'s: leave them.
- `temper/tests/client-browser/Cargo.toml` and `Cargo.lock`: `temper-fake.workspace = true` under `[dev-dependencies]`.

Not yours: `browser_public.rs` and `browser_accounts.rs`.

Options, pre-approved:
- If the fake refuses `replies()` at 9,600 code words, lower `LONG_CODE_WORDS` by 100s, keeping the long history at three windows or more, and say so. Never shrink the position history's, and never change the fake's bound.
- A helper's shape inside `browser_pages.rs` is yours. So is a shared helper for what `long` and `unanchored` have in common, if it keeps each one's steps readable in order.

## Steps

### `src/reading.rs` (new)

Move out of `browser_pages.rs` what makes the histories, so a focused test can read it. A module comment cites `temper/docs/design/client/browser.md`, 4.3 and 4.6, 14.

```rust
pub const LONG_MESSAGES: u32 = 8;
pub const POSITION_MESSAGES: u32 = 3;
pub const LATEST_MESSAGES: u32 = 6;
pub const SHORT_START: &str = "A short conversation";
pub const SHORT_END: &str = "The whole answer.";
/// "Reading paragraph 01."
#[must_use] pub fn long_words(number: u32) -> Box<[u8]>;
/// "Position paragraph 01."
#[must_use] pub fn position_words(number: u32) -> Box<[u8]>;
/// "Latest paragraph 1. Keep these words available while reading back. ...", as `words` has it, four times.
#[must_use] pub fn latest_words(number: u32) -> Box<[u8]>;
/// Each history's cues and replies, in `Tier::reading`'s `asked` scripts.
#[must_use] pub fn replies() -> Vec<(String, String)>;
/// The page at the height the private stylesheet turns anchoring off at, and back at the kit's.
#[must_use] pub fn anchoring_off() -> Step;   // Step::Viewport { width: 1280, height: 801 }
#[must_use] pub fn anchoring_on() -> Step;    // Step::Viewport { width: 1280, height: 800 }
```

`replies()` holds, by the code as it stands in `reading_tier`:
- **Long:** cue `long_words(n)`, `LONG_CODE_WORDS = 9_600`, 40 paragraphs but 30 for the last. It keeps `reading_reply` and its "End of reply N." line.
- **Position:** cue `position_words(n)`, `POSITION_CODE_WORDS = 7_000`, unchanged, 10 paragraphs but 30 for the last two.
- **Latest:** one cue, `Latest paragraph`, with today's reply, "Keep these words available while reading back. " four times.
- **Short:** cue `SHORT_START`, reply `SHORT_END`.

Keep the comments that say why ("The last reply makes opening read the preceding one", "A horizontally scrollable code line carries the window's bytes in one node", the small window's), and the one over `READING_WINDOW_BYTES`.

**Its tests**, which replace `reading_tier`'s two runtime assertions:

```rust
#[test]
fn the_long_history_spans_three_windows_and_the_position_history_stays_inside_one() { .. }

/// A startup refusal would otherwise show only in the browser tier.
#[test]
fn the_fake_admits_every_historys_replies() {
    // `{"listen":{"socket":"127.0.0.1:0"},"key":"/tmp/key","tls":null,
    //   "models":[{"name":"qa","window":4096,"output":1024}],"cued":[],"asked":<the JSON `Tier::reading` writes>,
    //   "otherwise":["fallback"]}` through `temper_fake::llm::Document::decode`.
}

/// The fake chooses by the cue that comes first in a request's words.
#[test]
fn each_request_cues_only_its_own_reply() {
    // For every history's words: exactly one cue of `replies()` occurs in them, and it is that history's.
}
```

To keep one form of the JSON, move `with_replies`'s scripts into a function both call. The second test must bite: at `LONG_CODE_WORDS = 10_000` the fake refuses with "scripts past their joint byte bound". The third must bite: with `position_words` reading "Reading paragraph" it fails. The report says so for each.

### `src/lib.rs`

```rust
/// The demo with reading's replies on a private bundle, whose stylesheet turns the browser's
/// anchoring off at `reading::anchoring_off`'s height alone (temper/docs/design/client/browser.md, 4.3).
#[must_use]
pub fn reading(name: &str) -> Self;
```

It is `demo_without_anchoring`'s body, with `with_replies` applied, and the appended rule becoming:

```css
@media (height: 801px) { #app > .temper-session .temper-thread > [role=log] { overflow-anchor: none !important; margin-top: 7px !important; } }
```

```rust
/// Offer a second model with its efforts, for the composer's picker, without reaching an LLM.
#[must_use]
pub fn with_second_model(self) -> Self;
```

It is `browser_model_picker.rs`'s edit of the deployment, through `configuration::private`.

### `browser_pages.rs`: reading

The test's outer shape:

```rust
/// One `temper box`, three chats through ctl, a page load for each story (client/browser.md, 4.6, 14).
#[test]
fn a_chat_opens_at_its_end_and_reads_back_keeping_its_place() {
    let tier = Tier::reading("reading");
    let mut calls = tier.ctl_chat(reading::SHORT_START, &[]);                 // T1
    calls.extend(tier.ctl_next_chat(2, /* position_words(1), 2..=3 */));     // T2
    calls.extend(tier.ctl_next_chat(3, /* long_words(1), 2..=8 */));         // T3
    let mut ctl = tier.ctl_face(calls);
    let mut stories = Stories::of_person(1);
    let mut steps = reader_opens(FIRST_CHAT_ADDRESS);
    stories.begin("short", &steps);
    steps.extend(short_steps());
    stories.begin("latest", &steps);
    steps.extend(latest_steps(&tier));
    stories.begin("unanchored", &steps);
    steps.extend(unanchored_steps(&tier));
    stories.begin("long", &steps);
    steps.extend(long_steps(&tier));
    let mut reader = Script::new(steps.into_boxed_slice());
    let mut harness = Harness::new(tier.held_setup());
    let outcome = harness.run(&mut [&mut ctl, &mut reader]);
    eprintln!("kept boxes: {:?}", harness.kept());
    stories.assert_done(outcome);
    // Each story's measures, in the order its test had them, then the two below.
}
```

Kept boxes are named `{story}/{name}`: `short/log`, `latest/before-incoming`, `long/before-later`. `box_of`, `in_log` and `held` take the story, and their messages begin with it, as `short: start: {y} + {h} outside {top} + {height}`.

**`short`**, on T1, from `a_short_chat_opens_whole_without_a_reading_status` as it stands: `see(Find::text(SHORT_END))`, its three `gone`s, its three `keep_box`es. Measures: `in_log` for `start` and `end`.

**`latest`**, on T1 grown, from `latest_returns_...` as it stands, with its opening replaced:

```rust
Step::Person(PersonStep::Go { address: b"/chats".as_slice().into() }),
see(Find::exactly_named(Role::Link, FIRST_CHAT)),
// tier.ctl_messages(1, 0, /* latest_words(1..=6) */)
Step::Person(PersonStep::Go { address: FIRST_CHAT_ADDRESS.into() }),
// then see(words(6)), see(words(1)), and every step after them as they stand
```

Sending from the chats page keeps those six answers from being delivered to a page that isn't measuring them, and the second `Go` is this story's own page load. The two `task message`s it already holds keep their task and key.

**`unanchored`**, on T2, from `reading_position(false)` and `reading_chat`, in their order, with these additions alone:

```rust
Step::Person(PersonStep::Go { address: b"/chats/2".as_slice().into() }),
// reading_chat's see and three gones, on position_words(3)
keep_box(Find::named(Role::Log, b"Conversation"), "unanchored/log-default"),   // new
reading::anchoring_off(),                                                       // new
// keep_box log, end, head, press the composer, and all that follows, as they stand
keep_box(Find::named(Role::Log, b"Conversation"), "unanchored/log-end"),       // new, after the last Focused
reading::anchoring_on(),                                                        // new
```

`see(reply_end(1))` and `anchor`, message `messages - 1`, stand as they are, on `position_words`.

**`long`**, on T3, from `reading_position(true)` and `reading_chat`, in their order, opening with `Go` to `/chats/3`. Nothing else changes: its 6 PageDowns, `read_to_start`, `read_to_end` and "Latest" steps stand as they are.

**The two new measures**, which are the only way a rule that never matched can't pass unseen:

```rust
assert_eq!(box_of(&harness, "unanchored/log-end").0, box_of(&harness, "unanchored/log-default").0 + 7, "unanchored: the rule holds at its height");
assert_eq!(box_of(&harness, "long/log").0, box_of(&harness, "unanchored/log-default").0, "long: the browser's anchoring is back");
```

`unanchored/log-end` is read at the story's end, after every read, so it says the rule held throughout. The comment says that.

Remove what the merge leaves unused: `READING_*`, `reading_tier`, `reading_words`, `reading_reply`, the four old tests, `demo_without_anchoring`.

### `browser_pages.rs`: the call line, into test 2

Test 2 is already `temper box`, and starts its chat with the words the call line's script is cued on. It changes from `Tier::demo("history-chat-filter-address")` to `Tier::demo_asked("history-chat-filter-address", "Add CSV export to widget", /* the three turns as they stand */)`.

Directly after `start_playground_chat(words)`, before `Step::Address(FIRST_CHAT_ADDRESS)`, its steps as they stand in `browser_call_line.rs`: from `see(Find::text(b"I read the hold and chat pages."))` to the last `gone(body())`. Its `before.value == after.value` measure follows the run, its message beginning `call line:`.

`stories`: `of_person(0)`, `begin("call line", ..)` after `start_playground_chat`; `begin("history", ..)`; `begin("project across a reload", ..)` before `reload_chats`; `begin("history", ..)` after it. `let mut harness` replaces the inline `Harness::new`, and `assert_done` the `assert_eq!`.

Update the file's header comment: it is what only a browser shows, and now holds a run of calls' folded line and the picker's menu. Keep its citation.

### `browser_pages.rs`: the picker, into test 4 and 5

`Tier::new("composer").with_model().with_second_model()`. `model_picker_steps()` becomes `browser_model_picker.rs`'s steps as they stand, from `Step::Viewport { width: 420, height: 900 }` to the last `folded(false)`, with its `keep` helper, followed by a new `Step::Viewport { width: 1280, height: 800 }`, so that the steps after it stand at the size they always had. Those steps hold every step `model_picker_steps()` has today.

The picker's `start_chat(b"Choose the next model")` goes: test 4's chat, already open at this point, stands in for it.

`stories`: `of_person(0)`; `composer`; `model picker`; `composer` again before `chats()`; `panic` before the `Go` to `/test/panic`.

### Then

Remove the two `mod` lines from `main.rs`. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`; the acceptance's two commands; `rg -c '#\[test\]' temper/tests/client-browser/tests/browser_all/` adds to 13. Commit and push.

## Report

As the implementer's text has it. Also:
- Each test the merge retired, with the steps it left in the merged test, and any exception.
- What `LONG_CODE_WORDS` ended at, and the fake's verdict.
- The breaks that showed `reading.rs`'s tests bite.
- That no browser test ran.
- Anything read in the code that makes a merged step look wrong. After-landing proves these, so what you suspect is worth a line.
