+++
id = "copy-button"
part = "copy-button"
kind = "feature"
needs = ["copy-part", "printed-words"]
files = ["temper/crates/temper-client-domain/src/copy.rs", "temper/crates/temper-client-domain/src/tests/copy.rs", "temper/crates/temper-client-view/src/pages/welcome_copy.rs", "temper/crates/temper-client-view/src/tests/welcome_copy.rs", "temper/crates/temper-client-shell-web/src/host_tests_copy.rs", "temper/crates/temper-client-shell-web/tests/suite/welcome_copy.rs"]
effort = "high"
minutes = 75
+++

## Goal

The welcome's command block has a quiet "Copy" in its top right corner, in a browser alone. Pressed, it writes the command's exact text and reads "Copied" for 2 s, or "Couldn't copy" for 2 s on plain http or any refusal or failure, the text staying selectable.

Acceptance, each with a test in the file named:

- **The domain,** `tests/copy.rs`: a press asks `Request::Copy` with the words and the next token, in that step; `Done` labels `Copied`, `Refused` and `Failed` `NotCopied`; the label is back to `Copy` once `fire` runs at `now + 2 s`, and not before; `next_deadline` says when; a report for an earlier press's token changes nothing; a press within the 2 s asks again; `shown` moves at each change.
- **The view,** `tests/welcome_copy.rs`, on the Claude panel as `welcome.rs`' tests reach it:
  - browser: a `Role::Button` named "Copy" beside the `Role::Code`, inside a `Role::Slot` with `WELCOME_CMD_BOX`; the button has `FLOW_QUIET` and `WELCOME_COPY`
  - terminal: no such button, no such slot, and the same `Role::Code` as today
  - its press's event is `Action::Copy { words }`, `words` equal to the `Role::Code`'s text, byte for byte, with no line added
  - after `Event::Copied`: "Copied" for `Done`; "Couldn’t copy" for `Refused` and for `Failed`, spelt as the page's other words spell an apostrophe (`temper-client-view`'s `words.rs`); back to "Copy" after 2 s; the code's text unchanged throughout
- **The host,** `host_tests_copy.rs`: the press's own `report` returns `Duties::copy`, with the token and the words, so it's asked in the press's turn; `Report::Copied` changes the tree; `Report::Due` at the deadline puts it back.
- **The stylesheet,** `tests/suite/welcome_copy.rs`, as `draft_section_headings_keep_the_document_line_height` reads `temper.css`: the block's right padding holds the button's inset, plus 13 characters at 12px in a monospace font (7.2px each), plus a gap, and no other rule changes it. Break the padding by hand and see it fail.
- **The recording:** `copy` and `copied` lines write and parse, as `temper/docs/design/client/journeys.md`, 3.2, has them, a `copy`'s words among what a redaction replaces.
- Remove the **Not yet** lines naming *The command's Copy* from `temper/docs/design/client/README.status.md`, and `copy` and `copied` from `temper/docs/design/client/journeys.status.md`. This task may make those edits, against the implementer's text.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, The welcome's Model step, *The command's Copy*.
- `temper/docs/design/client/journeys.md`, 3.2.
- `skein/docs/design/ui.md`, section 13; `skein/docs/design/ui-web.md`, section 11.
- `temper/docs/plan/box-small/notes/contracts.md`, *The clipboard* and *The Copy*; `temper/docs/plan/box-small/notes/common.md`.

## Owns and may change

Owns the files in the header, all new. Everything else is a small hunk, as `notes/common.md` asks, since `ui-walk-3` has priority on these crates and `sign-in-new-tab` is on `boundary.rs`, `domain.rs`, `host.rs`, `lib.rs` and `journey/*`:

- `temper-client-domain`: `lib.rs`, the `mod` line; `boundary.rs`, the three variants; `domain.rs`, the field, `copy_label`, and one line each in `next_deadline`, `fire` and `shown`; `route.rs`, the two arms; `tests.rs`, a `mod` line.
- `temper-client-view`: `look.rs` and `tests/look.rs`, the two rows of `notes/contracts.md`; `pages.rs` and `tests.rs`, `mod` lines; `accounts.rs`, `Meaning::Copy { words: Box<[u8]> }`, its `decode` arm and the arms the compiler asks for; `words.rs`, three `Word` rows beside `WelcomeClaudeNote`; `pages/welcome_model.rs`, in `program`, the one call; `pages/connect_model.rs`, `command` split into the text and the drawing.
- `temper-client-shell-web`: `host.rs`, the report, the duty and its arm in `duties`; `host_tests.rs`, a `#[path]` module; `lib.rs`, as *Steps* has it; `journey/line.rs`, `record.rs`, `replay.rs`, `fields.rs`, `originals.rs`, the two lines' arms, as `navigate`'s and `carried`'s are; `tests/suite/main.rs`, a `mod` line; `temper.css`, the one edit and the rules of *Steps*.
- `temper-client-shell-terminal`: `route.rs` and `client.rs`. It answers `Request::Copy` at once with `Event::Copied { token, written: Refused }` (`skein/docs/design/ui.md`, section 13), though its tree has no such button.
- Other readers of `Request`, `Action` or `Event` (`temper/tools/gallery/src/driver.rs`, `temper/tests/client/src/app.rs`): the arm the compiler asks for, and nothing more.
- If `first-usable-model` is on your base, `printed_command(root, w, name)` is the text and `command` already draws a prompt above it: `program` draws that prompt, then the block. Otherwise split `command` yourself: `command_words(w, name, environment)` gives the text.
- `command` keeps `welcome: bool` only if something still passes `true`; if nothing does, drop it.
- No look request is asked for: this is the whole of the look. `ConnectModel`'s `FLOW_CODE` command has no Copy, and neither has any other code block.
- Journeys hold no `temper-welcome-cmd`, so no recording moves. If a gallery test's list of a sample's parts names the block, edit that literal and say so.

## Steps

The block, in `temper-client-view/src/pages/welcome_copy.rs`:

```rust
//! The welcome's command block and its Copy (temper/docs/design/client/README.md, 3.5).
pub(super) fn block(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, words: &[u8]);
```

In the terminal it draws the `Role::Code` alone, with `WELCOME_CMD`. In a browser (`w.shell == Shell::Browser`), a `Role::Slot` with `WELCOME_CMD_BOX` holding that `Role::Code`, then a `Role::Button` named and reading what `root.copy_label()` says, with `FLOW_QUIET` and `WELCOME_COPY`, bound to `Meaning::Copy`. A `Role::Code` holds text alone, which is why the slot is there.

`temper.css`, beside `.temper-welcome-cmd`, whose padding becomes `12px 116px 12px 14px`:

```css
#app:has(> .temper-welcome) .temper-welcome-cmd-box { position: relative; }
#app:has(> .temper-welcome) .temper-welcome-copy { position: absolute; top: 12px; right: 12px; min-height: 0; padding: 0; border: 0; background: transparent; color: #9c573f; font: 600 12px/1.58 var(--gf-mono); cursor: pointer; }
#app:has(> .temper-welcome) .temper-welcome-copy:hover { text-decoration: underline; }
```

The shell, in `lib.rs`, all under `wasm32`: `Report::Copied(clipboard::Report)`; `Shell` gains `clipboard: clipboard::web::Clipboard<Report>`, made at each of its constructions with `Rc::clone` of the inbox and `Report::Copied`; the turn maps it to `host::Report::Copied`, each `Outcome` to its `Written`; and `carry`'s first lines, before any transport:

```rust
// Keep the person's gesture (skein/docs/design/ui-web.md, section 11).
if let Some((token, words)) = &duties.copy {
    self.clipboard.write(*token, words);
}
```

1. The domain, failing first; then `copy.rs` and its wiring.
2. The view's tests; the block and its words.
3. The host, its tests; `lib.rs`'s shell; the recording's lines.
4. The stylesheet and its test.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal --all-targets -- -D warnings`; `heavy cargo clippy -p temper-client-shell-web --target wasm32-unknown-unknown -- -D warnings`.
6. One command at a time: `heavy cargo nextest run -p temper-client-domain`, `-p temper-client-view`, `-p temper-client-shell-web`, `-p temper-client-shell-terminal`, `-p temper-client-world`, `-p temper-client-gallery`. Citations. Commit and push.

## Report

As the implementer's text asks. Which reserved rows you took, and which of the other drafts' changes were on your base.
