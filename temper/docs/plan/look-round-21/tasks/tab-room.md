+++
id = "tab-room"
part = "room"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tests/client-browser/src/lib.rs",
  "temper/tests/client-browser/tests/browser_all/browser_pages.rs",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 50
+++

# Ask temper's composer above its three tabs on a phone

## Goal

The request's task 1 (`temper/docs/plan/look-round-21/inputs/request.md`). At 420 px, Ask temper's page as To begin's Start opens it shows the three tabs Home, Ask and Settings at `y=937…1000`, and the composer at `(16, 899.25, 388, 56)`: its foot and the model picker lie under the tabs. The rule meant to leave them room, in `/* ==== ui: phone ==== */` (about line 3076 of `temper/crates/temper-client-shell-web/temper.css`),

```css
#app:has(> .temper-session):has(> .temper-tabbar:not(:has(> a:nth-child(4)))) { padding-bottom: calc(64px + env(safe-area-inset-bottom)); }
```

nests `:has()` inside `:has()`, which Chromium refuses, so the whole rule never applies. Restate it in a form browsers accept, and hold it with a row of the browser tier's history story, written and compiled, not run.

Acceptance, by `temper/docs/design/client/browser.md`, 2.5, *The project's frame* and *A selector a browser accepts*, and 4.6, story 11:

1. **The rule, restated where it stands,** its comment line kept above it and its declaration unchanged:

   ```css
   #app:has(> .temper-session):has(> .temper-tabbar):not(:has(> .temper-tabbar > a:nth-child(4))) { padding-bottom: calc(64px + env(safe-area-inset-bottom)); }
   ```

   `:not(:has())` is accepted (the same file's `scroll-padding-bottom` rule, about line 3094, already uses it); `#app` is `box-sizing: border-box` at `100dvh` on a chat's layout (about line 1825), so the padding lifts the session's grid row, and with it the composer and the picker, wholly above the tab bar's 63 px. No other rule of `temper.css` changes.
2. **No other nesting:** no selector in `temper.css` or `task-pages.css` nests `:has()` inside `:has()` once the rule is restated (this cut found that rule alone; the report says how it was checked, by a throwaway scan deleted before the push).
3. **The row,** a story of the existing test `history_keeps_the_chat_focus_and_filter_across_reload_and_back` (`temper/tests/client-browser/tests/browser_all/browser_pages.rs`), on its `Tier::demo_asked` box, before its first chat: the playground's project ready, its repository in and no way of working yet, so To begin shows, as the render check found it live. No new test function: the tier keeps its twelve.
   - `temper/tests/client-browser/src/lib.rs`: `start_playground_chat(words)`, whose one caller is this test, splits in two, keeping its steps and their order: `pub fn playground_ready() -> Vec<Step>` (from "Try the playground" through `connected()`) and `pub fn playground_chat(words: &[u8]) -> Vec<Step>` (from pressing "Chats" to `see(first_chat(words))`); `start_playground_chat` goes, its doc comment split between the two.
   - The test: `let mut steps = playground_ready();`, then `stories.begin("phone ask", &steps)` and the story's steps, then `steps.extend(playground_chat(words))` under the existing "call line" story's `begin`, as now.
   - The story, sketched:

     ```rust
     Step::Viewport { width: 420, height: 900 },
     // Home, from the tab bar or by its address, then To begin's Start.
     see(Find::exactly_named(Role::Region, b"To begin")),
     press(Role::Button, b"Start"),
     Step::Address(b"/projects/1/ask".as_slice().into()),
     see(Find::exactly_named(Role::TextArea, b"Ask temper, or tell it how you work")),
     see(Find::exactly_named(Role::Navigation, b"Sections")),
     keep_box(Find::exactly_named(Role::TextArea, b"Ask temper, or tell it how you work"), "phone-ask/composer"),
     keep_box(Find::named(Role::Disclosure, b"Model"), "phone-ask/picker"),
     keep_box(Find::exactly_named(Role::Navigation, b"Sections"), "phone-ask/tabs"),
     Step::Viewport { width: 1280, height: 800 },
     // Back where playground_ready left the person, so that playground_chat runs as before.
     ```

     and after the run, in the test's body, with the existing `box_of(&harness, story, name)`: the composer's and the picker's `y + height` at most the tabs' `y`, each with a message naming the boxes.
   - Compiles: `heavy cargo check -p temper-client-browser-world --tests`. It is not run (`notes/common.md`, *Runs*).
4. **Status:** delete the **Not yet** entry naming `tab-room` in `temper/docs/design/client/browser.status.md`. No other design line.
5. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-shell-web` (its `tests/suite` reads `temper.css`) and `-p temper-client-gallery` (its print links it); `cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`.

One commit or more, each with the trailer `Look-request: to-begin-two-steps`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-21/notes/common.md`.
- `temper/docs/plan/look-round-21/inputs/request.md`, task 1.
- `temper/docs/design/client/browser.md`, 2.5, *The project's frame* and *A selector a browser accepts*; 4.6, the table and story 11.
- `temper/docs/design/client/README.md`, 3.5, *Ask temper's page*, and 5.3, *While To begin shows*.
- `temper.css`: the chat layout's phone rules (about lines 1780 and 1822 to 1840), the project's frame's tab bar (about line 1416), `ui: phone` (about lines 3072 to 3099).
- The view's names: the tab bar `Role::Navigation` "Sections" (`temper/crates/temper-client-view/src/frame.rs`, about line 817); To begin's Start (`pages/home.rs`, about line 788); Ask temper's composer (`pages/ask_page.rs`, about line 311); the picker's `Role::Disclosure` "Model" (`choice/picker.rs`, about line 105).
- The browser kit: `Step::KeepBox`, its value `"y height"` (`skein/testing/skein-fake-person-browser/src/harness/script/keep_box.rs`), read back as `browser_pages.rs`'s `box_of` and `a_chat_opens_at_its_end_and_reads_back_keeping_its_place` do.

## Owns and may change

Owns the one rule of acceptance 1 in `temper.css`; `start_playground_chat`'s split in `temper/tests/client-browser/src/lib.rs`; the history test's new story and its measures in `browser_pages.rs`; the status entry. Not the night set or any other rule of `temper.css`; not the view; not another story's steps.

Options, each decided:

- **How the story reaches Home:** press the tab bar's "Home" link, at 420 the only one; if the page `playground_ready` leaves draws no tab bar, `Go` to the project's Home address and await `tier.states().live` and `connected()` first. Say which.
- **The way back:** `Step::Back` to the page `playground_ready` left, checked by `Step::Address`; if Back can't return there (Start replaces the entry), `Go` to that address and await live and `connected()` again. Never change `playground_chat`'s steps to suit the story.
- **The project's number** in Ask temper's address is the playground's project's, as the box makes it (`FIRST_CHAT_ADDRESS`'s neighbours in `lib.rs`); if it isn't 1, use the one the box gives and say so.
- **The picker isn't in Ask temper's page at that state:** keep the composer's and the tabs' boxes alone, and say so in the report.
- **The rule's comment** names the reservation: keep it, "Ask's three tabs keep the composer above them; ...", unchanged.

## Steps

1. Read the request's task 1, the Inputs, and the rule; restate it (acceptance 1); scan both stylesheets (acceptance 2).
2. Split the helper, then write the story and its measures (acceptance 3). The implementer's bite step departs here: the story is not run in a plan, so its bite is the after-landing run's, against the rule as it stood (the render check's numbers above), and the report says so.
3. The status entry (acceptance 4); the runs of acceptance 5; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| The tab bar's room on Ask temper's page while To begin shows, at 420: a selector browsers accept | landed (`ui: phone`, the restated rule) |
| The composer and the model picker wholly above the three tabs | landed by that rule; its story compiled, not run |
| No other `:has()` nested in `:has()` | checked, both stylesheets |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4): compiling and focused tests establish no rendered equality |

## Report

The implementer's report: the rule as restated, the scan and its result, the story's steps and measures as written, the options taken, the runs' counts, and the account as committed.
