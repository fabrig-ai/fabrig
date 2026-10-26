# Session 03: the look

temper's view has no look at the base: it sets no style token, its composers are text areas that only take words, and
its signed-out main says the terminal's "Signed out." in every shell. `client/browser.md` 2.4 gives the browser its own
signed-out words, taken by the view at construction; 2.5 makes the chats' and a chat's composers submit on Enter with
line breaks kept, and puts temper's style tokens on the frame, the cards, the turns and the composers, drawn by
`temper.css`. 03.1 gives `Words` the shell's words. 03.2 makes the two composers submit on Enter, and pins that a line
break is kept within the bound. 03.3 sets the tokens, names them in a `look` the shell passes to its page, and writes
their rules after temper 02.3's palette. 03.1 and 03.2 need nothing of temper 02 and run beside it; 03.3 needs 02.3's
`temper.css` (decision 3). The session changes `temper-client-view`, and plans against `main` after
`tools/temper-repin` (landed, `8c02b3b9e`).

## Read first

- **temper:** `docs/design/client/browser.md` 2.4 ("Signed out"), 2.5, 4.1 (story 3), 6 (the first milestone);
  `docs/design/client/README.md` 5.1 ("`act`", "The look"), 5.4, 6.6; `docs/design/client/mockups/style.css` (`:root`,
  `.topbar`, `.page`, `.card`, `.composer textarea`) and `chats.html`; `docs/design/domain/testing.md` 6.3;
  `crates/temper-client-view/src/words.rs` (`Words::english`, the private `english`), `src/frame.rs` (`owner`,
  `link_region`, `banner`, `navigation`, `Bindings::decode`), `src/root.rs` (`build`, `adoption_main`, `act`),
  `src/pages/chats.rs` (`main`, `row`), `src/pages/chat.rs` (`main`, `composer`, `conversation`), `src/confirm.rs`
  (`card`), `src/limits.rs`, `src/tests/fixtures.rs`; `crates/temper-client-domain/src/owner.rs` (`Owner::asker`);
  `tests/client/tests/stories.rs` (`a_double_press_of_start_makes_one_chat`).
- **jig:** `crates/jig-client-shell-web/src/look.rs` (`NAMES`) and `src/tests.rs`
  (`the_look_names_the_view_s_styles_and_regions`); `crates/jig-client-view/src/owner.rs` (`Owner::opaque`'s widening).
- **skein:** `docs/design/ui.md` 8; `docs/design/ui-web.md` 3.2, 7, 12; `docs/design/ui-text.md` 5, 6;
  `crates/skein-ui/src/look.rs` (`Names`, `check`); `crates/skein-ui-web/src/mapping.rs` (`core_element`: a token is a
  class on the node's own element, a text area's on its `textarea`).
- **skein foundation:** `programming-model.md` 4.2 (bindings are echoed tokens), 6.3 (the worst case), 10.3;
  `testing-strategy.md` 2.1, 2.2, 10.
- **Plan:** the README's "Rules of this plan" (the Chat page, browser tests, the hotspot `pages/chat.rs`);
  `code-organization.md` 2, 3, 5 and 6; decisions.md 3, 4, 9 and 11; `temper/02-shell.md` 02.2 (`limits::view`, the
  `Look`) and 02.3 (`temper.css`).

## What the design asks

- **browser.md 2.4, "Signed out":** the browser's main says "Signed out. To sign in, open a sign-in link `temper serve`
  printed."; the view takes its shell's words at construction; the terminal's stay (README 6.6). 03.1.
- **browser.md 2.5, the composers:** the chats' and a chat's text areas carry `SubmitsOnEnter`; Shift+Enter is a line
  break the view allows, counted within the bound (README 5.1, 5.4). 03.2. Their growth is `temper.css`'s alone. 03.3.
- **browser.md 2.5, the tokens:** under `temper-`, on the banner, the navigation, the main, the cards, the turns and the
  composers; the terminal prints none. 03.3.
- **browser.md 2.5, the stylesheet:** the compact top navigation and quiet borders from the mockups, no icon, nothing a
  person must know said by style alone (skein's `ui.md` 8). 03.3, after 02.3's palette and font.
- **browser.md 6, the first milestone:** the frame's, the chats' and a chat's look; the inbox's and a task's are the
  second's (`later.md`).
- **decisions.md 4 and 9:** the composers and the words are proved on the host, in the view's tests; one browser story,
  temper 04.4's, proves Enter and Shift+Enter.

## Choices this plan makes

- **The words name their shell.** `words::Shell { Terminal, Browser }` is a parameter of `Words::english`, which picks
  `SignedOutMain`'s literal by it. `View::new` is unchanged: it takes the words it is given. Every other word is the
  same in both shells.
- **The browser's words drop the backticks:** "Signed out. To sign in, open a sign-in link temper serve printed." A
  phrase's literal is a plain run, and the browser would show the backticks as typed.
- **Enter on a composer is its button.** `Bindings::decode` maps `skein_ui::Act::Submit` on the chats' composer to
  `Act::StartChat`, and on a chat's to `Act::Send`, the acts the "Start" and "Send" buttons bind. skein-ui refuses a
  Submit on a text area without `SubmitsOnEnter` (`Tree::act`), so no other area can send it.
- **`confirm.rs`'s text areas are not composers.** "Answer words" and "Reason" keep Enter as a line break; a Submit on
  them decodes to nothing.
- **Line breaks need no code.** The domain keeps a composer's bytes whole within `limits.client.domain.text`
  (`temper-client-domain`'s `src/pages/chat.rs` `Chat::act`, `src/pages/chats.rs` `Chats::write`), and jig's `Text::new`
  checks length and UTF-8 alone. 03.2 pins it with tests.
- **The text shell honours the state too.** skein-ui-text prints `(enter sends)` beside the composers and submits after
  `n WORDS` (ui-text.md 5, 6). The terminal's goldens print synthetic trees (`src/tests/fixtures.rs` `chats_page`,
  `src/tests/fixtures/chat.rs` `page`), so none changes.
- **Six tokens, one a node.** `temper_client_view::look::{BANNER, NAVIGATION, MAIN, CARD, TURN, COMPOSER}`, tokens 0
  to 5, and `look::NAMES`, as jig's `look.rs`. `styles_per_node` stays 2. `NAMES` also names `frame::link_region` as
  `temper-link`, so that the look names every region the view sets (ui-web.md 3.2); `temper.css` gives it no rule.
- **The owner is a constant.** `NAMES` and the tokens are `const`, so that the shell's `&[NAMES]` is `'static` for
  `Page::new`. `frame::owner` and the domain's `Owner::asker` become `const fn`, widening with jig's
  `Owner::opaque` bytes, since `u32::from` is not `const`.
- **The cards are the chats' rows and the chat's card and departure items.** The inbox's and a task's cards get the
  token with milestone 2. `MAIN` goes on every page's main: it is the frame's.
- **A turn's token goes on the `Region` each line already opens** in `conversation`: no node, key or order changes.
- **`temper.css`'s rules,** after 02.3's, from the mockups: the banner and navigation in one compact row with a quiet
  bottom border (`.topbar`); the main's width (`.page`); the cards (`.card`); `white-space: pre-wrap` on a turn, since
  `skein-ui.css` sets none and a line break must show; the composer `field-sizing: content` from one line to 120px
  (`.composer textarea`), and under `@supports not (field-sizing: content)` a fixed 120px that scrolls.

## Rules that bind it

- **The Chat page stays open to live text:** this session changes the Chat page's composer and its tokens only, never
  the turns' list (README, Rules; code-organization.md 5). `pages/chat.rs` is a hotspot.
- **The style tokens are constants of `temper-client-view`, each a `temper-` name; `temper.css` names no class the view
  does not set** (code-organization.md 2), and has no web font, icon or external origin (code-organization.md 5).
- **Browser tests sparingly:** this session adds none. The words and the composers are proved in the view's tests and
  the client world; temper 04.2 and 04.4 read them in a browser (README, Rules; decision 4).

## Increments

### 03.1 The browser's signed-out words
Scope: browser.md 2.4, "Signed out": the browser's signed-out main, the terminal's unchanged.
Needs: nothing.
Files: `crates/temper-client-view/src/words.rs`. `Words::english`'s users, each passing `Shell::Terminal`:
`crates/temper-client-shell-terminal/src/limits.rs` (`view`), `crates/temper-client-shell-terminal/src/tests/fixtures.rs`
(`page_words`), `crates/temper-client-view/src/tests/fixtures.rs` (`words`), `tests/client/src/fixtures.rs`. temper
02.2's `crates/temper-client-shell-web/src/limits.rs` (`view`) passes `Shell::Browser`: whichever of 02.2 and 03.1
lands second writes it (02-shell.md 02.2). Tests: `src/tests/frame.rs`.
Shape:
- `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Shell { Terminal, Browser }` in `words.rs`, with a doc line
  citing browser.md 2.4.
- `Words::english(connector, text, shell)` passes `shell` to the private `english(word, shell)`, whose
  `Word::SignedOutMain` arm matches it; every other arm ignores it.
- The fixture `browser_page_view(root) -> crate::View`: `page_view`'s, with `Words::english(FORGE, 64, Shell::Browser)`.
Test shape: unit, in `crates/temper-client-view/src/tests/frame.rs`, beside
`signed_out_the_main_says_so_and_nothing_else`, which stays the terminal's. The reference test:

```rust
#[test]
fn signed_out_the_browser_main_says_how_to_sign_in() {
    let view = fixtures::browser_page_view(&fixtures::signed_out());
    assert_eq!(
        &*fixtures::main_text(view.tree()),
        b"Signed out. To sign in, open a sign-in link temper serve printed.",
    );
}
```

Beside it:
- `out_of_date_the_browser_main_keeps_its_words`: `out_of_date()` in the browser's words says the terminal's sentence.
- `the_browser_words_fit_the_views_limits`: `crate::Limits::of(&fixtures::domain_limits(), &browser_words)` is `Some`
  (programming-model.md 6.3).

Ceilings: each under 20 ms.
Proof: unit: these three, focused, in the gate. Nothing above them on the host: the shell passing `Shell::Browser`
builds for wasm32 alone, and the gate's wasm32 clippy compiles its call. Later, world: the browser suite, `--profile
browser`, temper 04.2's `a_spent_link_shows_the_signed_out_words`.
Don't: change `View::new`; carry the shell in the domain; change the terminal's words (README 6.6); add a `Word`.
Size: at most 30 lines (`words.rs`, the terminal's `limits.rs`, the shell's `limits.rs`).
Removes: nothing.

### 03.2 The composers submit on Enter and keep line breaks
Scope: browser.md 2.5, "The composers": `SubmitsOnEnter` on the chats' and a chat's composers, Enter as their button,
a line break kept within the bound. The view alone: no CSS.
Needs: nothing.
Files: `crates/temper-client-view/src/pages/chats.rs` (`main`), `src/pages/chat.rs` (`composer` only: the turns'
list is untouched; a hotspot, README), `src/frame.rs` (`Bindings::decode`). Tests: `src/tests/fixtures.rs`,
`src/tests/chats.rs`, `src/tests/chat.rs`; `tests/client/tests/stories.rs`. `confirm.rs` is unchanged.
Shape:
- `b.state(State::SubmitsOnEnter)` on the "Start a chat" and the "Message" text areas, after `b.name`.
- `Bindings::decode`: `(Act::ChatWords { .. }, skein_ui::Act::Submit) => Act::StartChat` and
  `(Act::MessageWords { .. }, skein_ui::Act::Submit) => Act::Send`, before the `Press` arm.
- Fixtures, extended (code-organization.md 4): `chats_root()`, `root_at(Address::Chats, &domain_limits()).0`;
  `chat::messaging_root()`, `chat_root`'s with a `c::Offered { act: c::ActKind::Message, admission:
  c::Admission::Allowed, effect: None }` beside its close offer, the two sharing one private builder;
  `type_into(root, name, words) -> Box<[u8]>`, which acts `Input` on the named text area, steps the root with the
  action, rebuilds and returns the area's value.
Test shape: unit, in `crates/temper-client-view/src/tests/chats.rs` and `src/tests/chat.rs`. The reference test:

```rust
#[test]
fn enter_in_the_start_composer_starts_a_chat() {
    let mut view = fixtures::page_view(&fixtures::chats_root());
    let composer = fixtures::find(view.tree(), Role::TextArea, b"Start a chat").expect("composer");
    assert_eq!(
        crate::act(&mut view, composer, skein_ui::Act::Submit),
        Some(Action::Pages(pages::Act::StartChat)),
    );
}
```

Beside it:
- `a_line_break_in_the_start_composer_is_kept`: `type_into` "Hello\nthere" gives it back whole.
- `enter_in_the_message_composer_sends`: on `messaging_root()`, Submit on "Message" is `Act::Send`.
- `a_line_break_counts_within_the_message_bound`: words of `domain.text` bytes holding a line break are kept; one byte
  more leaves the value as it was.
- `enter_in_a_cards_reason_does_nothing`: on `proposal_root()`, Submit on "Reason" is `None`.

Joint, in `tests/client/tests/stories.rs`, the client world joined to the engine (`domain/testing.md` 6.3). It reads
the break through the title: a started chat's title is its words' first line, cut at the first `\n` (jig
`crates/jig-domain-core/src/documents/small.rs` 49-50, `chip_from_row`), which the Chat page shows as a Paragraph named
by it (`src/pages/chat.rs` 35); the first words are on no conversation line. Once the Chat page shows its "Message"
area (as `tests/client/tests/chat.rs`'s `a_task_with_the_chat_label_opens_as_a_chat`), a paragraph named exactly
"Hello" holds; a dropped break gives "Hellothere".

```rust
#[test]
fn enter_in_the_start_composer_starts_one_chat_titled_by_its_first_line() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::chats_page());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"Start a chat"), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"Start a chat"), words: Box::from(*b"Hello\nthere") },
            Step::Send { find: Find::named(Role::TextArea, b"Start a chat") },
            Step::Send { find: Find::named(Role::TextArea, b"Start a chat") },
            Step::See { find: Find::named(Role::TextArea, b"Message"), within: fixtures::WITHIN },
            Step::See { find: Find::exactly_named(Role::Paragraph, b"Hello"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 1, "one chat task for the two Enters");
}
```

Ceilings: each unit test under 20 ms; the story under 0.5 s, as `a_double_press_of_start_makes_one_chat`.
Proof: joint: the story, above the units: the fake person's `Send` is a Submit, which skein-ui refuses before this
increment; the engine counts one chat, whose title is the line before the break. Later, world: the browser suite,
`--profile browser`, temper 04.4's `enter_starts_one_chat_keeping_a_shift_enter_break`, the key listener and the area's
own line break.
Don't: set `SubmitsOnEnter` on `confirm.rs`'s areas; filter, translate or count line breaks apart; touch
`conversation` or the turns; add CSS (03.3's); add a browser test.
Size: at most 15 lines (`chats.rs`, `chat.rs`, `frame.rs`).
Removes: nothing.

### 03.3 The look's tokens and their rules
Scope: browser.md 2.5, "temper's style tokens" and "The bundle's stylesheets": the six tokens on the frame, the chats'
and the chat's nodes; `look::NAMES` in the shell's `Look`; their rules in `temper.css`, with the composers' growth.
Needs: temper 02.3 (`temper.css`, and 02.2's `Look` before it).
Files:
- `crates/temper-client-domain/src/owner.rs` (`Owner::asker`). Its users (`rg -l '\.asker\(\)'`: the domain's
  `acts.rs`, `route.rs`, `pages.rs`, `pages/*.rs`, `forge/me.rs`; the view's `frame.rs`; `temper-client-tools`'
  `root.rs`, `forge.rs`; their tests) are unchanged: a `const fn` is called as before.
- `crates/temper-client-view/src/look.rs` (new), `src/lib.rs` (`pub mod look;`), `src/frame.rs` (`owner`, `banner`,
  `navigation`), `src/root.rs` (both mains), `src/pages/inbox.rs`, `src/pages/task.rs` (their mains),
  `src/pages/chats.rs` (`main`, `row`), `src/pages/chat.rs` (`main`'s items, `composer`, `conversation`'s regions).
- `src/limits.rs`'s `styles_per_node: 2` and its copies (`src/tests/fixtures.rs` `tree_limits`; the terminal's
  `src/tests/fixtures.rs` and `tests/support/mod.rs`) are unchanged.
- `crates/temper-client-shell-web/src/lib.rs` (the `Look`), `crates/temper-client-shell-web/temper.css`.
- Tests: `src/tests.rs` (`mod look;`), `src/tests/look.rs` (new), `src/tests/fixtures.rs`, `src/tests/chats.rs`,
  `src/tests/chat.rs`.
Shape:
- **`owner.rs`:** `pub const fn asker`, each arm `jig_client::Asker(wide(number(…)))`, with a private `const fn
  wide(number: u16) -> u32` copied from jig's `Owner::opaque` (`to_le_bytes`, `u32::from_le_bytes([low, high, 0,
  0])`). `frame::owner` becomes `pub const fn`.
- **`look.rs`:** the six `pub const` `Style`s, each with a doc line naming its nodes, and `NAMES`, as the sketches say;
  the module's doc cites browser.md 2.5 and jig's `crates/jig-client-shell-web/src/look.rs`.
- **Set, by `b.style(…)` after the node's `open`:** `BANNER` in `banner`; `NAVIGATION` in `navigation`; `MAIN` at
  each `Role::Main` (`root.rs` twice, `inbox.rs`, `task.rs`, `chats.rs`, `chat.rs`); `CARD` on `chats::row`'s item and
  `chat::main`'s card and departure items; `TURN` on `conversation`'s `Role::Region`; `COMPOSER` on both composers.
- **The shell:** `Look { names: &[temper_client_view::look::NAMES], … }`, the rest as 02.2 wrote it.
- **`temper.css`,** after 02.3's rules, each citing its mockup rule in a comment:
  - `.temper-banner, .temper-navigation`: a flex row with a gap and `border-bottom: 1px solid var(--temper-line)`.
  - `.temper-main`: `.page`'s width, centred.
  - `.temper-card`: `var(--temper-paper)`, `1px solid var(--temper-line)`, a radius, `var(--temper-shadow)`.
  - `.temper-turn { white-space: pre-wrap; }`.
  - `.temper-composer`: `field-sizing: content; min-height: 1lh; max-height: 120px; resize: none; overflow-y: auto;`,
    and `@supports not (field-sizing: content) { .temper-composer { height: 120px; } }`.
  - No `@import`, `@font-face` or `url(`.
Test shape: unit, in `crates/temper-client-view/src/tests/look.rs` (new), with the fixture `styles(tree, role) ->
Box<[Box<[skein_ui::Style]>]>`, the styles of each node of that role in order. The reference test:

```rust
use super::fixtures;
use crate::look::{BANNER, MAIN, NAVIGATION};
use skein_ui::Role;
#[test]
fn the_frame_carries_its_tokens() {
    let view = fixtures::page_view(&fixtures::inbox_root());
    for (role, token) in [(Role::Banner, BANNER), (Role::Navigation, NAVIGATION), (Role::Main, MAIN)] {
        assert_eq!(fixtures::styles(view.tree(), role).as_ref(), [Box::from([token])], "{role:?}");
    }
}
```

Beside it, in `look.rs`:
- `the_looks_names_pass_skeins_check`: `skein_ui::look::check(&[NAMES])` is `None`, as jig's
  `the_look_names_the_view_s_styles_and_regions`.
- `each_style_or_region_is_named_under_temper`: `NAMES.style` of each constant is its `temper-` name; `NAMES.region`
  of `frame::link_region()` is `temper-link`.
- `each_pages_main_carries_the_main_token`: `signed_out()`, `adoption_root()`, `task::held_root()`, `chats_root()`,
  `chat::chat_root()`.
- `the_composers_carry_the_composer_token`: `chats_root()` and `chat::messaging_root()`, each one `[COMPOSER]`.

In `chats.rs`, `a_chats_row_carries_the_card_token` (its `tree(&line())`); in `chat.rs`,
`a_chats_card_carries_the_card_token` (`proposal_root()`) and `each_turn_carries_the_turn_token`
(`tree(&provisional())`: every region `[TURN]`, as many regions as before).
Ceilings: each under 20 ms.
Proof: unit: these eight, focused, in the gate. No golden changes: the terminal prints no token (README 5.1), its
goldens print synthetic trees, and the view has none. Nothing on the host reads a class; later, world: the browser
suite, `--profile browser`, loads `temper.css` in every story from temper 04.1 and fails on a policy violation, such
as an external origin (decisions.md, For the user 1). No story reads a style (skein's `ui.md` 8); the growth and its
fallback are a person's to see (browser.md 2.5).
Don't: add, key, move or rebuild a node in `conversation`; give a node two tokens or raise `styles_per_node`; set
`CARD` on the inbox's or a task's cards (milestone 2); name a class in `temper.css` that `NAMES` lacks; add a web
font, an icon, `url(` or `!important`; ship `mockups/style.css`.
Size: at most 130 lines (`owner.rs`, `look.rs`, `lib.rs`, `frame.rs`, `root.rs`, the four pages, the shell's `lib.rs`,
`temper.css`).
Removes: 02.2's empty `names` in the shell's `Look`.

## Interface sketches

The planner compiles the first three on the host against `temper-client-domain` and `temper-client-view`, with stub
bodies; the shell's `Look` line compiles in the gate's wasm32 clippy.

```rust
// crates/temper-client-view/src/words.rs (03.1)
/// The shell whose words the view says (client/browser.md, 2.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shell {
    Terminal,
    Browser,
}
impl Words {
    pub fn english(connector: u16, text: u32, shell: Shell) -> Result<Words, Invalid>;
}

// crates/temper-client-domain/src/owner.rs (03.3)
impl Owner {
    #[must_use]
    pub const fn asker(self) -> jig_client::Asker;
}

// crates/temper-client-view/src/frame.rs (03.3)
#[must_use]
pub const fn owner() -> skein_ui::Owner;

// crates/temper-client-view/src/look.rs (03.3)
// skein_ui::Style { owner: Owner, token: u16 } crosses into the shell; every field is set.
pub const BANNER: skein_ui::Style = skein_ui::Style { owner: crate::frame::owner(), token: 0 };
pub const NAVIGATION: skein_ui::Style = skein_ui::Style { owner: crate::frame::owner(), token: 1 };
pub const MAIN: skein_ui::Style = skein_ui::Style { owner: crate::frame::owner(), token: 2 };
pub const CARD: skein_ui::Style = skein_ui::Style { owner: crate::frame::owner(), token: 3 };
pub const TURN: skein_ui::Style = skein_ui::Style { owner: crate::frame::owner(), token: 4 };
pub const COMPOSER: skein_ui::Style = skein_ui::Style { owner: crate::frame::owner(), token: 5 };
/// The look's names (skein's ui.md, 8): styles by token, regions by number.
pub const NAMES: skein_ui::look::Names<'static> = skein_ui::look::Names {
    owner: crate::frame::owner(),
    prefix: b"temper",
    styles: &[
        b"temper-banner", b"temper-navigation", b"temper-main", b"temper-card", b"temper-turn", b"temper-composer",
    ],
    regions: &[b"temper-link"],
};

// crates/temper-client-shell-web/src/lib.rs (03.3, wasm32)
// Look { names: &[temper_client_view::look::NAMES], pictures: &[], draw: |_, _, _, _, _| false, components: &[], elements: &[] }
```

## Questions to raise when reached

- **03.3:** if a golden of the terminal's or another crate's test changes when the tokens are set, report which and ask:
  the terminal prints none (README 5.1).

## Done when

Each in a worktree of temper at `origin/main`, after `git fetch`:
- `git show origin/main:crates/temper-client-view/src/words.rs | grep -q -F 'To sign in, open a sign-in link temper serve printed.'`
- `git show origin/main:crates/temper-client-shell-terminal/src/limits.rs | grep -q -F 'Shell::Terminal'`
- `git show origin/main:crates/temper-client-shell-web/src/limits.rs | grep -q -F 'Shell::Browser'`
- `git show origin/main:crates/temper-client-view/src/pages/chats.rs | grep -q -F 'State::SubmitsOnEnter'`, and the
  same for `crates/temper-client-view/src/pages/chat.rs`
- `git show origin/main:crates/temper-client-view/src/frame.rs | grep -q -F 'skein_ui::Act::Submit'`
- `git show origin/main:tests/client/tests/stories.rs | grep -c -F 'starts_one_chat_titled_by_its_first_line()'`
  prints 1
- `git show origin/main:crates/temper-client-domain/src/owner.rs | grep -q -F 'pub const fn asker'`
- `git show origin/main:crates/temper-client-view/src/look.rs | grep -q -F 'b"temper-composer"'`
- `git show origin/main:crates/temper-client-shell-web/src/lib.rs | grep -q -F 'temper_client_view::look::NAMES'`
- `git show origin/main:crates/temper-client-shell-web/temper.css | awk '/\.temper-composer/{c=1} /field-sizing:
  content/{f=1} /@import|@font-face|url\(/{bad=1} END{exit !(c && f && !bad)}'`
- `sh -c 'names=$(git show origin/main:crates/temper-client-shell-web/temper.css | grep -o -E "\.temper-[a-z-]+" | sort
  -u); test -n "$names" || exit 1; for n in $names; do git show origin/main:crates/temper-client-view/src/look.rs |
  grep -q -F "b\"${n#.}\"" || exit 1; done'`: every class `temper.css` names is one the view sets.
- `cargo nextest run -p temper-client-view -p temper-client-world` exits 0.
