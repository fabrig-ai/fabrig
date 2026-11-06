+++
id = "composer-focus"
part = "composer-focus"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/pages/task.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chats.rs",
  "temper/crates/temper-client-domain/src/tests/pages/task.rs",
  "temper/crates/temper-client-view/src/pages/goal.rs",
  "temper/crates/temper-client-view/src/pages/agent.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/goal.rs",
  "temper/crates/temper-client-view/src/tests/task.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/tests/client-browser/tests/browser_all/browser_pages.rs",
]
effort = "high"
minutes = 60
+++

# The composer keeps the focus after sending

## Goal

After a person sends, the focus is in the composer, so they can type again with no click: a chat just started by sending opens
with its composer focused, and every `Act::Send` asks for the composer's focus, by Enter or by "Send". A chat opened any other way
keeps its heading's focus.

Acceptance:

- **A start that sends:** a chat opened by the answer to a start has `composer_focus` 1: from `NewChat`, `NewChat { draft: true }`,
  `Configure`'s Send, a starter and a suggested request. A chat opened by address has 0, and so does the same chat opened again after
  leaving it.
- **A send:** each `Act::Send` raises the count of the open chat or task page by one, whether or not its send went out. `Act::Stop`,
  `Act::AddDraft` and `Act::MessageWords` don't.
- **The view:** a goal's and an agent's task's composer ask for the focus as a chat's does: count nonzero, and no rename's form open.
- **A browser story,** `browser_pages::the_composer_keeps_the_focus_after_sending`, red before the code at the start and at the
  press of "Send", then green. The report gives what it saw before the code at each step (where the focus stood), and the
  story's CPU and wall.
- **The phone,** in the report, as Step 6 says.

## Inputs

- `temper/docs/plan/ui-composer-focus/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5, *Where the focus stands on a chat*; 5.3, the closed form's focus; 5.11, the composer;
  section 11, *A press that starts a chat, on a phone*.
- `temper/docs/design/client/browser.md`, 4.1, item 3.
- `skein/docs/design/ui-web.md`, 7: *A focus request* (a node inserted with one, or whose count moved, takes the focus) and *Going
  to another page* (the heading, unless a focus request moved in the same apply).

## Owns and may change

**Owns:** the `files` above.

**Readers, each edited only as said:**

- `temper/crates/temper-client-view/src/tests/cards.rs`, `change_something_focuses_the_composer`, reads `composer_focus`, and
  `tests/frame.rs`, `opening_rename_takes_focus_from_a_chat_composer`, its focus. They open the chat by address, so they should
  stay as they are. If one reaches its chat by a start or sends first, its expected count moves by that, nothing else.
- `temper/tools/gallery/src/samples.rs` steps `Act::StartChat`, and `tools/gallery/src/lib.rs` prints a focus as `autofocus`: a sample
  that reaches a chat through a start now prints its composer `autofocus`. Regenerate that print by its tool.

**Pre-approved options:**

- **If the story puts the tier over `cpu_suite_s`** (85, `tools/tiers/budgets.toml`, `browser-temper`, on the tier's measure,
  `notes/common.md`): first see that the excess is the story's. A story that starts a chat and sends costs about 8 CPU s
  (`latest_returns_a_chat_to_its_end_after_new_words_while_reading_back`), and the tier's last runs were 57.8, 73.8 and 78.6, before
  other plans' stories. Then fold its steps into `enter_starts_one_chat_keeping_a_shift_enter_break`, which already starts a chat, renamed
  to say the focus (`enter_starts_one_chat_and_the_composer_keeps_the_focus`), and give both measures. Still over, report them.
- **If the composer of a started chat isn't drawn until later** (Message not yet offered): that is the design: its insert carries
  the request. The story waits for it with `see`, and nothing is added for it.
- **If `Step::Focused` can't read a focused heading:** check the chat's by `Step::Keep`, with the selector `main h1:focus`, as
  `closing_project_rename_returns_focus_to_the_summary` keeps `main:not(:focus)`.
- **If the focus moving to the composer scrolls the page or a log:** list it under `## Left`. If that turns an existing test red, report
  `Question:`, since the focus op is skein's.

**Not yours:** `skein/` (a kit mechanism for the phone is section 11's, not this task's), `session_composer`'s hint and the bar
(`ui-pause-close`), a refused start's focus, `docs/design/`, `docs/help/`.

## Steps

1. **The domain's failing tests:**
   - `tests/pages/chats.rs`, with its `Page` fixture, as `configure_and_new_chat_start_their_template_and_open_the_chat` and
     `a_suggestion_replaces_new_chat_with_its_conversation` answer a start. `a_chat_started_by_sending_opens_asking_for_its_composer`:
     for `Act::StartChat` at `NewChat { draft: false }`, at `NewChat { draft: true }` and at `Configure`, `Act::Starter` at `Configure`,
     and `pages.suggest`: answer, then `crate::pages::Event::Went { address: Address::Chat { task } }`, then
     `page.pages.chat().expect("chat").composer_focus == 1`.
   - `a_chat_opened_otherwise_asks_for_nothing`: went to the chat directly, 0; started, went there, went to `Chats`, went there again, 0.
   - `tests/pages/chat.rs`, with `f::opened()`: `every_send_asks_for_the_composer`: a send of words, then one with the words empty
     (nothing sent), each raises the count; `Act::Stop`, `Act::AddDraft` and `Act::MessageWords` leave it.
   - `tests/pages/task.rs`, beside its `page_act(.., Act::Send)` tests: `a_send_asks_for_a_task_s_composer`.
2. **The domain:**
   - `pages/chat.rs`, `Chat::act`: `Act::Send` calls `self.send(..)`, then raises `composer_focus` as `Act::FocusComposer` does.
     Not in `send`: `Act::AddDraft` and `Act::Stop` send too.
   - `pages/task.rs`: `pub composer_focus: u64` on `Task`, 0 in `Task::new`, raised in its `Act::Send` arm. `Act::FocusComposer` stays
     in its no-op arm.
   - `pages.rs`, `went`: where `chats.opening` names this address, it also marks the chat started, and the arm that makes the
     `chat::Chat` sets its `composer_focus` to 1. `Chat::new`'s signature stays.
3. **The view's failing tests,** then the view:
   - `tests/chat.rs`: `a_started_or_sent_chat_asks_for_its_composer_and_an_opened_one_does_not`: no node asks on a chat opened by
     address; after `Act::Send`, the `Message` text area has the count.
   - `tests/goal.rs`, beside `send_disabled_while_empty`, and `tests/task.rs`, beside its `Message` text area's test: the same after
     `Act::Send`, in both shells' trees.
   - `tests/frame.rs`, beside `opening_rename_takes_focus_from_a_chat_composer`: a task page's composer gives way to the rename's
     field and has it back.
   - `pages/goal.rs` (its `TextArea`, about line 730) and `pages/agent.rs` (`composer`, its `TextArea`): as `pages/chat.rs`'s
     `session_composer` has it:

     ```rust
     if page.composer_focus != 0 && root.pages().switcher().renaming().is_none() {
         b.focus(page.composer_focus);
     }
     ```
4. **The browser story,** in `browser_pages.rs`, beside `closing_project_rename_returns_focus_to_the_summary`. Write it first and
   run it red: this is the request's "reproduce first". Its shape:

   ```rust
   /// Sending keeps the focus in the composer, by a press or a key (client/README.md, 3.5).
   #[test]
   fn the_composer_keeps_the_focus_after_sending() {
       let tier = Tier::new("composer-focus").with_model();
       let message = || Find::exactly_named(Role::TextArea, b"Message");
       // as enter_starts_one_chat_keeping_a_shift_enter_break: to /chats, connected, "+ New chat", the composer
       // Type "Hello", Key Enter, see first_chat(b"Hello"), see message(), Step::Focused(message()),
       // PersonStep::Type message "By the button", PersonStep::Press Button "Send",
       //   see Paragraph "By the button", Step::Focused(message()),
       // PersonStep::Type message "By the key", Step::Key(Key::Enter),
       //   see Paragraph "By the key", Step::Focused(message()),
       // Press Link "Chats", Press Link FIRST_CHAT, see message(), Step::Focused(first_chat(b"Hello")),
   }
   ```

   A press here is the harness's real mouse (`Input.dispatchMouseEvent`), so the browser moves the focus as a person's press does.
   Keep it at the floor: no reload, no `Step::Back`, none of Configure's or a goal's paths, which the tiers below hold.
5. **Check,** the implementer's text, and because `Act::Send` and a started chat mean something new: `heavy cargo nextest run -p
   temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-world -p
   temper-client-gallery`. Then the story alone and the tier's measure (`notes/common.md`).
6. **The phone,** by a throwaway CDP script (`notes/common.md`) on a fresh `temper box --fake-backends` (`temper help box`), at 420 px
   with `Emulation.setDeviceMetricsOverride` (`mobile: true`) and `Emulation.setTouchEmulationEnabled`. A capturing `focusin`
   listener records its target and `navigator.userActivation.isActive`. Read them for: Send pressed, Enter, a typed start, a starter's
   press on `Configure`, and a chat opened from the list. Headless Chromium draws no keyboard, so this shows where the focus lands and
   whether the browser would count it a person's act, not a keyboard: say so. No width branch and no kit change come of it: the design's
   default stands (`client/README.md`, section 11). If the focus doesn't land in the composer after a send at 420 px, that's a fault
   of this task: fix it.
7. `cargo fmt`, `old_forms.py` for temper, clippy on the changed packages, commit and push.

## Report

The implementer's report, and:

- **The reproduction:** where the focus stood before the code at the start and after the press of "Send".
- **The rule broken** for each new test, and the failure seen.
- **The story's CPU and wall,** and the tier's measure against 85 CPU s.
- **The phone:** Step 6's table and what it does and doesn't show.
