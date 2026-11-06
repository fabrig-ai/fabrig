# browser-budget: `browser-temper` back under 85 CPU seconds

After-landing at `668f7cd3e503`, with forge's `chat-reading-c` on main: **`browser-temper`, 17 tests, 129.4 CPU s against a budget of 85**, 70.8 s wall. The user, 2026-11-06: "tests should stay within budget". Raising it is off. This plan is the repair, in the order of `skein/docs/foundation/testing-strategy.md`, 8.1: does each test belong in this tier; can it be made faster; and if it's still over, measurements go to the user. Never move a test to escape a budget. Don't touch any budget, timeout or tier's membership by way of a fix.

| Story, in `temper/tests/client-browser/tests/browser_all/browser_pages.rs` | CPU s | wall s |
|---|---|---|
| `a_long_chat_opens_at_its_end_and_reads_back_from_the_composer` | 35.2 | 21.0 |
| `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` | 12.7 | 5.6 (5.6 to 8 before `chat-reading-c`) |
| `earlier_words_stay_put_without_browser_scroll_anchoring` | 7.9 | 4.3 |
| `a_short_chat_opens_whole_without_a_reading_status` | 4.3 | 2.1 |
| The other 13 stories | 69.3 | |

The other 13 ranged from 45 to 70 on the runs before this, with the load: the same story varies by about a third. So the target has margin: **under 70 on a quiet run**, the long story at about 12 or less, `latest_returns` about 6 or less, and `earlier_words` about 5 or less. These are targets, not limits.

**Every behaviour these stories prove stays proved in this tier:** opening at the end, reading back to the start with the composer focused, the position holding as lines load above, the window dropping the lines furthest from the reader and bringing them back, the newest never lost, and Latest and sending returning to the end. temper's `temper/docs/design/draft/chat-reading.md`, section 5, and `temper/docs/design/client/browser.md`, section 4, say what's in this tier and why.

## The levers, in the order I expect them to pay

1. **The history isn't typed in a browser.** `reading_history` types 16 messages through Chromium, each answered by a reply of about 65 KB, which makes about 1 MB, the window's bound. `latest_returns` types 6. None of that typing is what the stories are about. Chromium renders every delivery and the harness polls a tree that keeps growing. Build the history without the page: through the tier's own service, `temper ctl` or a client connection in the harness, whichever is cheapest and real, and open the page afterwards. Keep a typed message only where the story is about typing: `latest_returns`' "Sent while far back".
2. **A smaller window, same mechanism.** `document_bytes` is a limit. If the tier can set it lower, say 128 KB, with replies scaled to match, the window's behaviour is the same at an eighth of the bytes. The story must still pass the bound by at least two windows' worth, at both ends. forge's long-chat check by hand keeps the real size. Say in the story why it's lower.
3. **A find's cost on a long tree.** An earlier reading: 519 accessibility queries in 23 s on one story, with polls of 10 ms and 50 ms. A patch from agents/loop, unmeasured and never landed, backs off repeated misses of one find (below). Measure it on the long story. Land it, in skein with skein's tests, only if it pays. It must not make a press or a real change slower to notice.
4. **Whether each delivery re-renders or re-measures the whole window.** Read it and measure it. That would be a person's cost too, so report what you find even if it's not fixed here; fix it if it's small.

**Belonging:** say in the report, for these four stories, whether a lower world could show what each proves. `browser.md`, section 4, is the test. If one really can, move it and say why; budget isn't a reason.

## Measuring

This is a repair of a flagged tier, so **this overrides the common block's first bullet for these four stories alone:** run them by name, alone, as needed, with `tools/tiers/cpu-summary` for their CPU. Never run the whole tier; after-landing proves the whole on the next main. Report a before-and-after table for each story, with the machine's load at the time. If the targets aren't met, say what's left and where its CPU goes, with numbers. That's what goes to the user. Don't leave out a measurement because it's unflattering.

## The patch

```diff
diff --git a/skein/testing/skein-fake-person-browser/src/face.rs b/skein/testing/skein-fake-person-browser/src/face.rs
index 767e88ce19..73ed3fa850 100644
--- a/skein/testing/skein-fake-person-browser/src/face.rs
+++ b/skein/testing/skein-fake-person-browser/src/face.rs
@@ -1,7 +1,7 @@
 //! Queries and acts translated without executing script in the page.
 use skein_browser::boundary::{Event, Go, Key, Match, Query, Read, Refusal, Request, Seen};
 use skein_fake_person::{Doing, Face, Find, Found, Target};
-use skein_lib::{Time, Token};
+use skein_lib::{Duration, Time, Token};
 use skein_ui::{Act, NodeId, Role};
 use skein_ui_web::mapping::accessible;
 use std::{os::unix::ffi::OsStrExt, path::PathBuf};
@@ -70,6 +70,7 @@ pub struct BrowserFace<'a> {
     now: Time,
     last_find: Option<(NodeId, Find)>,
     look_again: Option<(Find, Time)>,
+    missing: Option<(Find, Duration)>,
     press: Option<Press>,
     expected_path: Option<Box<[u8]>>,
     failure: Option<String>,
@@ -111,6 +112,7 @@ impl<'a> BrowserFace<'a> {
             now: Time::ZERO,
             last_find: None,
             look_again: None,
+            missing: None,
             press: None,
             expected_path: None,
             failure: None,
@@ -196,7 +198,21 @@ impl<'a> BrowserFace<'a> {
         });
     }
     fn absent(&mut self, find: Find) {
-        let again = self.now.saturating_add(skein_browser::Limits::default().poll);
+        let poll = skein_browser::Limits::default().poll;
+        // Repeated misses walk the accessibility tree less often; a press keeps its poll.
+        let delay = if self.press.is_some() {
+            self.missing = None;
+            poll
+        } else {
+            let delay = self
+                .missing
+                .as_ref()
+                .filter(|(missing, _)| *missing == find)
+                .map_or(poll, |(_, delay)| delay.saturating_mul(2).min(poll.saturating_mul(4)));
+            self.missing = Some((find.clone(), delay));
+            delay
+        };
+        let again = self.now.saturating_add(delay);
         self.look_again = Some((find, again));
         if let Some(press) = &mut self.press {
             press.again = Some(again);
@@ -453,6 +469,7 @@ impl<'a> BrowserFace<'a> {
                 let node = self.remember(seen, role);
                 self.look_again =
                     Some((search.find.clone(), self.now.saturating_add(skein_browser::Limits::default().poll)));
+                self.missing = None;
                 self.last_find = Some((node, search.find));
                 if let Some(press) = &self.press {
                     let at = press.at;
diff --git a/skein/testing/skein-fake-person-browser/src/tests.rs b/skein/testing/skein-fake-person-browser/src/tests.rs
index a80cb1dd92..e546e7d1f9 100644
--- a/skein/testing/skein-fake-person-browser/src/tests.rs
+++ b/skein/testing/skein-fake-person-browser/src/tests.rs
@@ -608,3 +608,49 @@ fn a_match_past_the_last_is_absent() {
     assert_eq!(face.find(&find), Found::Absent);
     assert!(face.failure().is_none());
 }
+
+#[test]
+fn repeated_missing_finds_back_off_without_delaying_a_different_find() {
+    use skein_lib::{Duration, Time};
+    let mut face = face();
+    let find = Find::exactly_named(Role::Button, b"Ready");
+    let mut now = Time::ZERO;
+    assert_eq!(face.find(&find), Found::Pending);
+    for delay in [50, 100, 200, 200] {
+        found(&mut face, None);
+        assert_eq!(face.find(&find), Found::Absent);
+        now = now.saturating_add(Duration::from_millis(delay - 1));
+        face.tick(now);
+        assert_eq!(face.find(&find), Found::Pending);
+        assert_eq!(face.requests().count(), 0, "a repeated miss waits {delay} ms");
+        now = now.saturating_add(Duration::from_millis(1));
+        face.tick(now);
+        assert_eq!(face.find(&find), Found::Pending);
+    }
+    found(&mut face, None);
+    assert_eq!(face.find(&find), Found::Absent);
+    let other = Find::exactly_named(Role::Button, b"Cancel");
+    assert_eq!(face.find(&other), Found::Pending);
+    assert_eq!(query(&mut face).name.as_ref(), b"Cancel");
+}
+
+#[test]
+fn finding_a_target_resets_its_missing_find_backoff() {
+    use skein_lib::{Duration, Time};
+    let mut face = face();
+    let find = Find::exactly_named(Role::Button, b"Ready");
+    face.find(&find);
+    found(&mut face, None);
+    assert_eq!(face.find(&find), Found::Absent);
+    face.tick(Time::ZERO.saturating_add(Duration::from_millis(50)));
+    face.find(&find);
+    found(&mut face, Some(seen(b"")));
+    assert_eq!(face.find(&find), Found::Node(FOUND_TARGET));
+    face.tick(Time::ZERO.saturating_add(Duration::from_millis(100)));
+    face.find(&find);
+    found(&mut face, None);
+    assert_eq!(face.find(&find), Found::Absent);
+    face.tick(Time::ZERO.saturating_add(Duration::from_millis(150)));
+    face.find(&find);
+    assert_eq!(query(&mut face).name.as_ref(), b"Ready", "a new miss waits only the first interval");
+}
```

## Sources

- `temper/tests/client-browser/tests/browser_all/browser_pages.rs`
- `skein/testing/skein-fake-person-browser/src/face.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
