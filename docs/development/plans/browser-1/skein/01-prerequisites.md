# Session 01: the browser shell's prerequisites in skein

temper's browser tier (browser-1's temper 04) needs four things of skein that it lacks at `8644b70`. Its fake person
cannot read the page's address, which temper's `client/browser.md` 4.1, stories 1 and 2, check: since `9fbda6b`, skein's
design has the kit read it, the history's current entry (browser.md 5 and 6), and the face read it (ui-web.md 14.2). Its
`back_and_forward` story goes to its first address at once, where ui-web.md 14.2, since `15868de21`, has it first await
a page that holds its server's state. Its kit's `Key` request has no `shift`, which browser.md 5 and 6 give it since
`9fbda6b`, and which the composer's story presses as Shift+Enter (ui-web.md 14.2, the face's acts). And a click on the
words inside a button or a menu item reaches `skein-ui-web` as a click on a text node, which presses nothing, where
ui-web.md 6 and ui.md 3 have it press the control. 01.1 adds the address read to the kit and a step to the fake person.
01.2 gives `back_and_forward` its `loaded` find. 01.3 gives the kit's key its `shift`, and the fake person a key with
Shift held. 01.4 moves the click's target into the page model, and gives skein's example a button that holds its words.
The four increments are in decision 1's order, need nothing but skein's `main`, and each lands alone. No repository
downstream repins onto them in this plan (decision 5).

## Read first

- **skein:** `AGENTS.md`; `scripts/check.sh` (it builds the example's bundle, then runs `--profile browser`);
  `.config/nextest.toml`, `[profile.browser]`; `docs/design/browser.md` 5 (the vocabulary; "A key may be pressed with
  Shift held", "The current address", "Every operation has one terminal event"), 6 (the operations' table, its `key` and
  `address` rows; "A find waits out a navigation"), 8 (limits); `docs/design/ui-web.md` 6 ("One listener per kind of
  event"), 7 (Enter and Shift+Enter in a field that submits on Enter), 14.2 (the face's acts, "It reads the current
  address", the harness's steps, the stories' table), 14.3 (the example); `docs/design/ui.md` 3 ("A button, a tab or a
  menu item takes its words one way"); skein's commits `15868de21` and `9fbda6b`.
- **skein foundation:** `programming-model.md` 3 (step functions), 5.2 (lifecycle: one terminal per operation), 6.3 (the
  worst case), 10 (the Rust subset: exhaustive matches); `testing-strategy.md` 2.1 (step tests), 2.4 (machine worlds), 7
  (scenarios and the referee), 10 (a test's shape).
- **Plan:** the README's "Rules of this plan", "One test of a suite that needs a built artifact" and "Repin";
  `code-organization.md` 1 to 3, 5 (Don't) and 6 (the size command, skein's); decisions.md 1, 4, 5, and Design questions
  2 and 3.
- **Code:** `testing/skein-browser/src/boundary.rs` (`Request`, `Event`, `Key`); `testing/skein-browser/src/browser.rs`
  (`request`, `OpKind`, `Step`, `issue_step`, `queue_page`, `step_reply`'s `Step::History`, `Step::KeyDown` and
  `Step::Snapshot` arms, `finish_refused`, `key_info`, `key_state`, `issue_key`, `issue_character`);
  `testing/skein-browser/src/params.rs` (`Params::Key`); `testing/skein-fake-person-browser/src/face.rs` (`BrowserFace`:
  `key`, `perform`, `choice_key`, `history`, `event`, `fail`); `testing/skein-fake-person-browser/src/harness/script.rs`
  (`Step`, `Play for Script`'s `next`, its `Step::Key` arm) and `script/tests.rs`;
  `testing/skein-fake-person-browser/src/harness.rs` (`Runtime::event`); `testing/skein-fake-person-browser/src/stories.rs`
  (`back_and_forward`, `write_refused`, `Shown`); `tests/browser/src/world.rs` (`World`, `reply`, `fault`),
  `tests/browser/src/tests.rs` (`key_commands`, `PART_KEYS`), `tests/browser/src/tests/navigation.rs` (`navigating`,
  `loaded`, `methods`), `tests/browser/src/referee.rs`; `crates/skein-ui-web/src/page.rs` (`Held`, `click`, `close_menu`);
  `crates/skein-ui-web/src/page/web.rs` (the listener, `Mounted::nearest`, `read`);
  `crates/skein-ui-web/src/tests/page/events.rs` and `src/tests/page.rs` (`page`, `main`, `end`);
  `examples/ui/view/src/view.rs` (`form`); `tests/ui-web/tests/browser_example.rs`, `browser_stories.rs`.

## What the design asks

- **browser.md 5 and 6:** a request with its operation and one terminal event, each operation stepping reply by reply; a
  find's commands held while a navigation is in flight. `Address { page, op }`, answered by the terminal
  `Address { op, url }`, the history's current entry by `Page.getNavigationHistory`: 01.1 builds it. `Key`'s `shift`,
  Shift's modifier on the key's down and up: 01.3 builds it.
- **ui-web.md 14.2, "It reads the current address":** the face reads the kit's `Address`, and a story reads it once what
  the page shows says the page has put it there. 01.1 builds the read and its step.
- **ui-web.md 14.2, the stories' table, "back and forward":** the story awaits a page holding its server's state before its
  first move. 01.2 builds it.
- **ui-web.md 14.2, the face's acts, and 7:** a key with Shift held, Shift+Enter a new line in a text area that sends on
  Enter. The page model holds that already (`Event::Enter`'s `shift`). 01.3 builds the face's act and its step.
- **ui-web.md 6, "One listener per kind of event", and ui.md 3:** a click finds the nearest link, button, tab or menu item,
  else its nearest `data-node`; a press on a node a control holds is a press on the control. 01.4 builds it, and skein's
  example gets the form ui.md 3 allows, an unnamed button holding its words.

## Choices this plan makes

- **The current entry is the one at `currentIndex`** of `Page.getNavigationHistory`'s answer (browser.md 6), read as the
  kit's back and forward read it already (`Step::History`). It follows `history.replaceState` and `pushState`, which the
  shell uses.
- **The read waits out a navigation,** as a find's commands do (browser.md 6). `queue_page` marks the address's command as
  a page read, by its step, so that it reads the entry the navigation commits. The history read of `Go::Back` and
  `Go::Forward` stays unheld.
- **The URL is bounded by `Limits::text`,** not cut: a URL past it is refused `Limit`. A cut address would compare wrongly.
  `worst_case` is unchanged: one event's URL is below the snapshot the per-event report already holds (programming-model.md
  6.3).
- **The fake person compares the path,** the URL from the first `/` after its `://`, query and fragment kept. A test names a
  path, as `Step::Visit` and the person's `Go` do. The origin is the harness's, and a port changes from run to run. The face
  compares when the kit answers, as `key` acts, so the step needs no field of the script's.
- **`shift` reaches CDP as `modifiers` 8,** Shift's bit, on the key's down and up. `OpKind::Key` keeps `shift` for the
  up. `Params::Key` gains `modifiers`, written only when it is not 0, so that every command the kit sends today is
  unchanged.
- **The key with Shift held is a step of its own,** `Step::KeyWithShift(Key)`, beside `Step::Key(Key)`, whose form and
  users stay. Both go through the face's one act for a script's key, `BrowserFace::key`, which takes `shift`. The
  person's `Submit` and the face's choice of an option send `shift: false`.
- **The click's target moves into the pure page model,** `Page::click`, which walks `Held.parent` to the nearest button,
  link, tab or menu item. Host unit tests then reach it, and the wasm32 listener loses its link-only lookup. The menu's
  button keeps `Mounted::nearest`'s `aria-controls` lookup, which comes first.
- **skein's example presses inside a button through its Clear button,** which holds its words as a text node instead of
  its name. `escape_in_the_dialog_keeps_the_draft` presses it already, so 01.4 adds no browser test.

## Rules that bind it

- **Browser tests sparingly:** no browser test beyond those at the base. 01.1 extends one browser story, 01.4 changes one
  example node under another, and 01.2 changes a story's call. What else is proved, is proved on the host: the kit in its
  world (`tests/browser`), the face and the page model in unit tests (README, Rules).
- **Exhaustive matches:** every match over `Request`, `Event`, `Key`, `OpKind`, `Step` or `Role` names each variant, with no
  wildcard arm (programming-model.md 10), so a new variant is a build error at each list. Each list is named in Files.
- **Skein's own users only:** `back_and_forward`'s one caller, and every exhaustive list over the kit's types, are skein's.
  No downstream repository repins onto this session in the plan (decision 5). jig's `testing/jig-client-fake-person-browser`
  and `tests/client-browser` use the fake person, but build no `Request::Key`, hold no exhaustive match over `Event`, `Key`
  or the script's `Step`, and do not call `back_and_forward` or `Page::act` (`rg` at jig `8cc7e79f`). smith, hearth and
  temper use neither crate.

## Increments

### 01.1 The fake person reads the page's address
Scope: browser.md 5 and 6: the kit reads its page's address, the history's current entry, with one terminal event.
ui-web.md 14.2, "It reads the current address": the face reads it, and a story's step checks it against a path.
Needs: nothing (skein's `main`).
Files: `testing/skein-browser/src/boundary.rs`; `testing/skein-browser/src/browser.rs` (the `request` arm; `OpKind` and
`Step` gain `Address`; `queue_page`; `step_reply`; the exhaustive lists in `fn dialog`, the poll's `is_find` and
`fn mouse`); `testing/skein-fake-person-browser/src/face.rs` (`read_address`, its field, the `event` arm);
`testing/skein-fake-person-browser/src/harness/script.rs` (`Step::Address`, `next`'s arm);
`testing/skein-fake-person-browser/src/harness.rs` (`Runtime::event`'s list gains `Event::Address`);
`testing/skein-fake-person-browser/src/harness/script/tests.rs`; `tests/browser/src/world.rs` (`reply`);
`tests/browser/src/tests/navigation.rs`; `tests/browser/src/referee.rs` (its terminals);
`tests/ui-web/tests/browser_example.rs`.
The users of `Event` downstream, jig's two crates, match it with a fallback arm and compile unchanged. No hotspot.
Shape: `Request::Address { page, op }` and `Event::Address { op, url }`, as sketched. `request` admits it as
`OpKind::Address`, as `Request::Snapshot`, and issues `Page.getNavigationHistory` with `Params::Empty` and
`Step::Address`. `queue_page` marks a `PendingKind::Step(_, Step::Address)` command `read`, beside the DOM and accessibility
methods. `step_reply`'s `Step::Address` reads `currentIndex`, `entries` and that entry's `url`, as `Step::History` reads
them. It refuses `Protocol` when one is missing, and `Limit` past `env.limits.text`. Otherwise it removes the op and pushes
`Event::Address`. `BrowserFace::read_address(&mut self, path: &[u8])` takes a token, sets `acting`, keeps `path` in a new
field `expected_path: Option<Box<[u8]>>`, and pushes `Request::Address`. `event`'s `Event::Address` arm, for the acting op,
clears `acting`, takes `expected_path`, and sets `failure` to `the page's address was {url}, expected the path {path}` unless
the URL's path (`fn url_path(url: &[u8]) -> &[u8]`, private) equals it. `Event::Refused` already fails the acting op.
`Step::Address(path)` calls `face.read_address(&path)`, then `self.at += 1`, as `Step::Key` does.
Test shape: the kit in `tests/browser/src/tests/navigation.rs`, through `World`, whose `reply` gains
`b"Page.getNavigationHistory"`: three entries, `https://example.test/`, `https://example.test/notes/1` and
`https://example.test/notes/2`, with `currentIndex` 1, so that the read is shown to take the current entry, not the last.
Limits are the world's `Limits`, the kit's shipped ones. The reference test:

```rust
#[test]
fn an_address_read_gives_the_current_entrys_url() {
    let mut world = World::new();
    world.start();
    world.open();
    world.sent.clear();
    world.ask(Request::Address { page: PAGE, op: READ });
    assert_eq!(methods(&world), vec![Box::from(b"Page.getNavigationHistory".as_slice())]);
    match world.above.pop() {
        Some(Event::Address { op, url }) => assert_eq!((op, &*url), (READ, b"https://example.test/notes/1".as_slice())),
        other => panic!("expected Address, got {other:?}"),
    }
}
```

Beside it: `an_address_read_waits_out_a_navigation` (`navigating`: nothing sent; after `loaded`, the one command);
`an_address_past_the_text_limit_is_refused` (`World::with_limits` with `text` 16, below the current entry's 28 bytes, as
`tests.rs`'s limit tests build it: `Refused` with `Refusal::Limit`). The face, in `script/tests.rs`, with
`crate::tests::face` (page 7, its first op `7_000_001`): `a_failed_address_step_names_both_addresses` (the `Request::Address` sent; `Event::Address` with
`http://127.0.0.1:9000/notes/2` against `/notes/1`: not pending, and the failure's words exactly);
`an_address_step_passes_on_its_path` (`http://127.0.0.1:9000/notes/1`: no failure, `script.at()` 1);
`an_address_step_compares_the_query_too` (`http://127.0.0.1:9000/?secret=1` against `/` fails, as a sign-in link left in
the address would); `an_address_answer_to_another_op_is_ignored` (still pending). Ceilings: each unit test under 20 ms; each
world test under 0.5 s.
Proof: world: the browser suite, `--profile browser`, `a_person_opens_a_note_and_comes_back_by_its_link`, which gains
`harness::Step::Address` of `/notes/1` once the note's heading shows, and of `/` once "Notes" shows again. Chromium's own
history then backs the face's check, through the example's pushes. It runs by the README's two commands (the second with
this test's name), in a worktree of skein, and in the gate. Above it, the world tests show the kit's one terminal per
read under the referee.
Don't: add a browser test; add `Address` to `tests/browser/tests/fuzzy_world.rs` (its one test is past 40 lines at the base);
hold `Go::Back`'s or `Go::Forward`'s history read; change `worst_case` or `Limits`; compare the origin.
Size: at most 110 lines (`boundary.rs`, `browser.rs`, `face.rs`, `script.rs`, `harness.rs`).
Removes: nothing.

### 01.2 back_and_forward awaits a loaded page
Scope: ui-web.md 14.2, the stories' table, "back and forward": the story awaits a page holding its server's state before its
first move (`15868de21`).
Needs: nothing (skein's `main`).
Files: `testing/skein-fake-person-browser/src/stories.rs` (`back_and_forward` and its doc);
`tests/ui-web/tests/browser_stories.rs` (`back_and_forward_move_between_the_list_and_a_note`, its only caller: `rg -n
'back_and_forward\('` at skein `31bbf58`, jig `8cc7e79f` and temper `475136369` finds no other; jig's
`back_and_forward_keep_history_and_supersede_pending_shows` is another function).
Shape: `back_and_forward(setup, first, second, loaded: Find)`, `loaded` last, as `write_refused` and `typing_while_patched`
take it. Its script starts with `see(loaded)`, before `go(first.address)`. The doc says the story awaits `loaded` before
its first move. The caller passes `Find::text(b"Live")`, as the file's other stories do.
Test shape: no new test. The caller is the test, and the face's steps are unit-tested already.
Proof: world: the browser suite, `--profile browser`, `back_and_forward_move_between_the_list_and_a_note`, by the README's
two commands as written, and in the gate. It cannot fail first: the change removes a race, which its run does not
show on demand.
Don't: change `Shown` or the story's moves; add an argument to other stories.
Size: at most 10 lines.
Removes: the story's moves before its page holds its server's state.

### 01.3 A key with Shift held
Scope: browser.md 5 and 6: the kit's `Key` request takes `shift`, Shift's modifier on the key's down and up. ui-web.md
14.2, the face's acts: a key with Shift held, Shift+Enter a new line in a text area that sends on Enter (ui-web.md 7).
Needs: nothing (skein's `main`).
Files: `testing/skein-browser/src/boundary.rs` (`Request::Key`'s `shift`); `testing/skein-browser/src/browser.rs`
(`request`'s `Request::Key` arm; `OpKind::Key` and its two exhaustive lists, `fire`'s `is_find` and `fn dialog`'s;
`key_state`; `issue_key`; `step_reply`'s `Step::KeyDown`; `issue_character`); `testing/skein-browser/src/params.rs`
(`Params::Key`, its `write`); `testing/skein-fake-person-browser/src/face.rs` (`key`; `perform`'s `Act::Submit` and
`choice_key`, each a `Request::Key`); `testing/skein-fake-person-browser/src/harness/script.rs` (`Step::KeyWithShift`,
`next`'s `Step::Key` arm); `testing/skein-fake-person-browser/src/harness/script/tests.rs`
(`a_key_step_sends_the_kits_key`'s `Request::Key`); `tests/browser/tests/support/real.rs` (its three `Request::Key`);
`tests/browser/src/tests.rs`. With `face.rs`'s three, these are every constructor of `Request::Key` (`rg -n
'Request::Key'` in skein, jig and temper), each passing `shift: false` save `BrowserFace::key`'s.
`testing/skein-fake-person-browser/src/tests.rs` matches `Request::Key` with `..`, and `Step::Key`'s users,
`script/tests.rs` and `tests/ui-web/tests/browser_example.rs`, compile unchanged. Downstream, no repository names
`Request::Key` or `Step::Key` (`rg` at jig `8cc7e79f`; temper's manifests name neither crate).
Shape: `Request::Key { page, op, key, shift: bool }`. `request` admits it as `OpKind::Key { key, shift }`; the two lists
match `OpKind::Key { .. }`; `key_state` returns `Option<(Key, bool)>`. `issue_key(env, op, key, shift, down)` passes
`modifiers` 8 with `shift`, else 0, to the key's down and, from `Step::KeyDown`'s arm, its up. `Params::Key` gains
`modifiers: u64`, which `write` puts last as `"modifiers"` when it is not 0; `issue_character` passes 0. `key_info` is
unchanged. `BrowserFace::key(&mut self, key: Key, shift: bool)` puts `shift` in its request; `Act::Submit` and
`choice_key` pass `shift: false`. `Step::KeyWithShift(Key)` joins `Step::Key`'s arm: `Step::Key(key) |
Step::KeyWithShift(key)` calls `face.key(key, matches!(step, Step::KeyWithShift(_)))`, as `Step::Back | Step::Forward`
calls `face.history`.
Test shape: the kit in `tests/browser/src/tests.rs`, with `key_commands` and `World`, as `keyed` builds one. The reference
test:

```rust
fn enter_sent(shift: bool) -> World {
    let mut world = World::new();
    world.start();
    world.open();
    world.sent.clear();
    world.ask(Request::Key { page: Token::new(3), op: Token::new(4), key: Key::Enter, shift });
    world
}
#[test]
fn a_key_with_shift_held_sends_shifts_modifier_on_both_commands() {
    let shifted: Vec<String> = key_commands(&enter_sent(false))
        .iter()
        .map(|command| format!("{}{}", &command[..command.len() - 1], r#","modifiers":8}"#))
        .collect();
    let mut world = enter_sent(true);
    assert_eq!(key_commands(&world), shifted);
    match world.above.pop() {
        Some(Event::Done { op }) => assert_eq!(op, Token::new(4)),
        other => panic!("expected Done, got {other:?}"),
    }
}
```

Beside it: `a_key_without_shift_sends_no_modifiers` (Enter's two commands, its down and its up, name no `modifiers`).
`keys_type_each_character_as_its_key_down_and_up` and `PART_KEYS` stay unchanged, and show `issue_character` unchanged.
The face, in `script/tests.rs`, with `crate::tests::face`: `a_key_with_shift_step_sends_the_key_with_shift_held`
(`Step::KeyWithShift(Key::Enter)` sends `Request::Key` of page 7, op `7_000_001`, `Key::Enter`, `shift: true`);
`a_key_step_sends_the_kits_key` gains `shift: false` and stays otherwise. Ceilings: each unit test under 20 ms; each
world test under 0.5 s.
Proof: world: `a_key_with_shift_held_sends_shifts_modifier_on_both_commands` (tests/browser, focused), in the gate.
It shows the kit's commands for a key with Shift held are the key's with CDP's Shift modifier, down and up, and its one
terminal. No browser story in skein: skein's example has no text area that submits on Enter. Its Body text area does not,
and its Title, which does, is a text field. So no page of skein's shows what Shift adds. temper's
`enter_starts_one_chat_keeping_a_shift_enter_break` (temper 04.4) is its browser proof, through
`harness::Step::KeyWithShift(Key::Enter)`.
Don't: add other modifiers, or `shift` to another request; change `Params::Key`'s output for a key without Shift; change
`Step::Key`'s form or `key_info`; add a browser test.
Size: at most 40 lines (`boundary.rs`, `browser.rs`, `params.rs`, `face.rs`, `script.rs`).
Removes: nothing.

### 01.4 A click inside a button or a menu item finds its target
Scope: ui-web.md 6, "One listener per kind of event": a click finds the nearest link, button, tab or menu item, else its
nearest `data-node`. ui.md 3: a press on a node a button, a tab or a menu item holds is a press on it.
Needs: nothing (skein's `main`).
Files: `crates/skein-ui-web/src/page.rs` (`click`, a private `pressed`; 777 lines at the base, under 800 after);
`crates/skein-ui-web/src/page/web.rs` (the listener); `crates/skein-ui-web/src/tests/page/events.rs`;
`examples/ui/view/src/view.rs` (`form`); its users: `tests/ui-web/golden/list.html`, read by `tests/ui-web/tests/pages.rs`'s
`the_list_page_maps_as_its_golden_html` (Clear gains a span, and each later `data-node` and `skein-ui-label-N` one more);
`examples/ui/view/src/tests/golden/empty.txt` and `list.txt`, read by `examples/ui/view/src/tests/text.rs`, as the text
shell prints a button that holds its words; `examples/ui/view/src/tests.rs` (`the_views_worst_case_fits_its_limits`: its
tree's length 486 becomes 487, under `examples/ui/view/src/limits.rs`'s 600 nodes). `Event::Click`'s other users are in
`events.rs` and pass a control's own node, which `pressed` keeps. Downstream, nothing calls `Page::act` or builds
`Event::Click`: jig's test shell reports acts through `skein-ui-web`'s own listener (`rg` at jig `8cc7e79f`), and takes
this at its next repin (later.md).
Shape: `fn pressed(&self, node: NodeId) -> NodeId`: from `node` up `Held.parent` to `NodeId(0)`, the first whose role is
`Button`, `Link`, `Tab` or `MenuItem`, else `node`. It is a `while` over the parents, as the page's other ancestor walks.
`click` begins `let node = self.pressed(node);`; the rest is today's, so a disabled control, a modified or external link,
and an unbound one still press nothing. In `web.rs`, a click's node is `target.nearest(&element)`, as every other kind's:
the `closest("a[data-node]")` lookup goes. In `form`, Clear opens `Role::Button`, binds `CLEAR`, and holds
`builder.text(Run::Plain, &phrase(state, 15))` instead of its name. Add keeps its name.
Test shape: in `crates/skein-ui-web/src/tests/page/events.rs`, with `page`, `main`, `end` and `fixtures::patches`. A new
fixture `held` holds, under main: a bound button holding a text, a disabled bound button holding a text, a link holding a
text, a named menu with a bound menu item holding a text, and a loose text. Its node constants are named by a preorder
comment, as `events`' are. The reference test:

```rust
#[test]
fn a_click_on_the_words_a_button_holds_presses_the_button() {
    let (mut page, mut view) = (page(), View::new(&LIMITS));
    page.apply(&fixtures::patches(&mut view, 0, held), &mut Vec::new());
    assert_eq!(
        page.act(Event::Click { node: HELD_BUTTON_WORDS, modifier: false }, &mut Vec::new()),
        Some((HELD_BUTTON, Act::Press))
    );
}
```

Beside it: `a_click_on_the_words_a_disabled_button_holds_does_nothing`;
`a_click_on_the_words_a_link_holds_presses_it_unless_modified` (`modifier: true` gives `None`); `a_click_on_the_words_a_menu_item_holds_presses_it` (after a click on
the menu; the item's press, and its ops end with `Op::Focus` on the menu, as `close_menu` pushes);
`a_click_on_loose_words_does_nothing`. `each_event_gives_its_act` stays as it is. The goldens are regenerated by their tests'
output and compared by eye: the Clear button and the shifted ids only. Ceilings: each unit test under 20 ms.
Proof: world: the browser suite, `--profile browser`, `escape_in_the_dialog_keeps_the_draft`, by the README's two commands
with this test's name, and in the gate. Its press on "Clear" lands on the button's centre, on the span that holds its
words. Before `pressed` the click presses nothing, and the "Discard draft?" dialog never shows; after it, the dialog
shows. The author runs it once with the example's change alone, to see it fail.
Don't: change `Mounted::nearest`'s menu-button lookup; add options, checks or other roles to the walk (ui-web.md 6 names
four); add the builder's assertion of ui.md 3 (STATUS.md: not planned); add a browser test or an example node beyond
Clear's words.
Size: at most 25 lines.
Removes: `web.rs`'s link-only lookup for a click (`closest("a[data-node]")`).

## Interface sketches

The kit owns its operations: a request opens one, and its one terminal event ends it (browser.md 5). `Address` ends with
`Address` or `Refused`; `Key`, with `shift` or without, with `Done` or `Refused`. The fake person's face owns the
comparison: the script asks, the face reads the kit's answer and records the failure, which the harness reports as for
every act. The sketch compiles under skein's workspace lints, `clippy.toml` and `rustfmt.toml`, with
`clippy --all-targets -D warnings`, in a scratch crate with stand-ins for `Token`, `Key`, `Find`, `Setup`, `Shown` and
`Outcome` and stub bodies.

```rust
// testing/skein-browser/src/boundary.rs

/// What the test asks a browser to do.
pub enum Request {
    // … Person to Keys, unchanged.
    Key { page: Token, op: Token, key: Key, shift: bool },
    /// The page's address, its history's current entry; terminal: `Event::Address` or `Refused`.
    Address { page: Token, op: Token },
    // … Snapshot, Screenshot and Close, unchanged.
}

/// What the browser reports to the test.
pub enum Event {
    // … Ready to Screenshot, unchanged.
    /// The URL an `Address` request read, whole, at most `Limits::text` bytes; terminal.
    Address { op: Token, url: Box<[u8]> },
    // … Trouble and Closed, unchanged.
}
```

```rust
// testing/skein-fake-person-browser/src/harness/script.rs

pub enum Step {
    // … the script's other steps, unchanged; `Key(Key)` among them.
    /// A key with Shift held (ui-web.md, 14.2): Shift+Enter, a new line in a text area that sends on Enter.
    KeyWithShift(Key), // `skein_browser::boundary::Key`, as `Step::Key` names it
    /// Read the page's address; the story fails unless its path, query kept, is this one.
    Address(Box<[u8]>),
}

// testing/skein-fake-person-browser/src/stories.rs

/// Back and forward between two addresses, once the page shows `loaded` (ui-web.md, 14.2).
#[must_use]
pub fn back_and_forward(setup: &Setup<'_>, first: Shown<'_>, second: Shown<'_>, loaded: Find) -> Outcome;
```

01.1 lands `Request::Address`, `Event::Address` and `Step::Address`; 01.2 `back_and_forward`; 01.3 `Request::Key`'s
`shift` and `Step::KeyWithShift`.

## Questions to raise when reached

- If `escape_in_the_dialog_keeps_the_draft` passes with 01.4's example change alone, the kit's press does not land on the
  span. Report the hit node (`DOM.getNodeForLocation`'s answer, from the run's log) and ask whether a press at a point,
  `harness::Step::PressAt`, should stand as the proof.

## Done when

Each in `~/src/rust/skein`, after `git fetch`:
- `git show origin/main:testing/skein-browser/src/boundary.rs | grep -q -F 'Address { page: Token, op: Token },'`
- `git show origin/main:testing/skein-browser/src/boundary.rs | sed -n '/^pub enum Event/,/^}/p' | grep -q -E '^ +Address \{'`
- `git show origin/main:testing/skein-browser/src/boundary.rs | grep -q -F 'shift: bool'`
- `git show origin/main:testing/skein-browser/src/browser.rs | grep -q -F 'Event::Address'`
- `git show origin/main:testing/skein-fake-person-browser/src/harness/script.rs | grep -q -x -E ' +Address\(Box<\[u8\]>\),'`
- `git show origin/main:testing/skein-fake-person-browser/src/harness/script.rs | grep -q -E '^ +KeyWithShift\('`
- `git show origin/main:testing/skein-fake-person-browser/src/stories.rs | grep -q -E 'pub fn back_and_forward\(.*, loaded: Find\) -> Outcome'`
- `git show origin/main:testing/skein-browser/src/params.rs | grep -q -F 'modifiers'`
- `git show origin/main:crates/skein-ui-web/src/page.rs | grep -q -E 'fn pressed\('`
