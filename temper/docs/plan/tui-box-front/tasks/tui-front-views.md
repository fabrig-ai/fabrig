+++
id = "tui-front-views"
part = "tui-front"
kind = "feature"
needs = ["tui-surface"]
files = [
  "temper/crates/temper-client-tui/src/view/front.rs",
  "temper/crates/temper-client-tui/src/view/launchpad.rs",
  "temper/crates/temper-client-tui/src/view/foot.rs",
  "temper/crates/temper-client-tui/src/view/foot/lists.rs",
  "temper/crates/temper-client-tui/src/view/foot/notes.rs",
  "temper/crates/temper-client-tui/src/tests/front.rs",
  "temper/crates/temper-client-tui/src/tests/foot.rs",
  "temper/crates/temper-client-tui/src/tests/launchpad.rs",
  "temper/crates/temper-client-tui/src/tests/commands.rs",
  "temper/crates/temper-client-tui/src/tests/golden/front-playground-80x24-dark.txt",
  "temper/crates/temper-client-tui/src/tests/golden/front-quit-80x24-dark.txt",
  "temper/crates/temper-client-tui/src/tests/golden/front-log-80x24-dark.txt",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 130
+++

# The playground's header and quiet line, the quit question, the box's state, the give up and `/log`

## Goal

The views of the request's frames 03, 04 and 05 (`temper/docs/plan/tui-box-front/inputs/box-front-03.txt` to `-05.txt`; `temper/docs/design/client/README.md`, 6.17, *The playground*, *Quitting*, *The box's state and `/log`*), on `tui-surface`'s state and words: "playground" in the launchpad's header and the banner's words in it, "playground" on the quiet line; the quit question as a list under the box; the box's state on the quiet line; the give up's note; `/log` offered in front, with its note. Out of front nothing drawn today changes.

Acceptance, by 6.17 and `notes/contracts.md`, section 7:

1. **`view/front.rs`,** the readings the other views call (each `pub(crate)`):
   - `playground(root, project: Option<u32>) -> bool`: `root.forge().playground_running()` and the read's `project` is `project`, `Some`.
   - `box_state(tui, words) -> Option<(Box<[u8]>, Style)>`: `BoxState::Restarting { step }` → `Word::Restarting` (`look::SIGNAL`), `GaveUp { step }` → `Word::GaveUp` (`look::DOWN`), `Running` → none; STEP made UTF-8 by `temper_client_domain::forge::words::lossy` and cut to `limits::STEP` bytes at a UTF-8 boundary (`skein_lib::bytes::utf8_prefix`).
   - `quit(root, tui, words) -> Option<Options>` while `tui.quitting()`: head `Word::QuitOne` with `super::work::running(root).title` when its `count` is 1, `Word::QuitMany` with the count above 1, `Word::QuitNone` at 0; items `Word::Numbered` 1 + `Word::StopBox` (`Event::Stop`) and 2 + `Word::DontQuit` (`Event::Stay`), the first active as skein makes it; `escape: Event::Stay`; drawn with a head, so in `tui-option`.
   - `log_note(tui, words) -> Option<LogNote>` while `tui.log_shown()` and `tui.front()`: the head `Word::LogHead`, the lines of `tui.box_log()` each made UTF-8 by `lossy` and cut to `limits::LOG_LINE` bytes, or `Word::LogEmpty` alone, and its end request `tui.revision()`.
   - `gave_up_note(tui, words) -> Option<Box<[u8]>>`: `Word::GaveUpNote` with the step (as above) once given up.
2. **The launchpad's header** (`view/launchpad.rs`): `Launchpad` gains `playground: bool`, `front::playground(root, number)`; while it holds, `home()` keeps its sections (left off, needs) but pushes no brief or headline, and `read` pushes `Word::PlaygroundBanner` (no style) in their place; `draw` lays, before the address in the heading, `Word::Playground` (`look::PLAYGROUND`) and `Word::Separator` (`look::END_FAINT`), so the border's end reads "playground · 127.0.0.1:8367". Connecting, out of date and not kept are as today.
3. **The quiet line** (`view/foot.rs`): `Foot` gains `box_state: Option<(Box<[u8]>, Style)>` (`front::box_state`) and `playground: bool` (`front::playground(root, project)`, the foot's project); `quiet()` lays the left half as hint, then `Word::Separator` and the box's state in its style, then `Word::Separator` and `Word::Playground` (`look::HINT`), then the needs' count as today, each left out when absent, and counts their widths in `left_width`, so the closed line goes first where the window is narrow.
4. **The lists** (`view/foot/lists.rs`): `read` takes `front::quit(..)` first, before the confirmation, the listings, the commands and the chat's question. The commands' list in front holds `/log` (name `tui::LOG_COMMAND`, words `Word::LogCommand`, `Event::Log`) after `/help` and before `/exit` when `LOG_COMMAND` begins with the words; `/exit`'s words are `Word::ExitFront` in front. `description` takes whether the TUI is in front; its reader in `notes.rs` passes it.
5. **The notes** (`view/foot/notes.rs`): `read_notes(root, tui, words) -> Notes` and `draw_notes(b, &Notes, words)` keep their signatures (`view/chat.rs`, `view/launchpad.rs` and `tui-welcome`'s `view/welcome.rs` call them). `Notes` gains `gave_up: Option<Box<[u8]>>` and `log: Option<LogNote>`. `/help`'s note in front: `/log` among the commands (the padding counting it), `/exit` as `ExitFront`, and `Word::KeyCloseFront` in `KeyClose`'s place. `draw_notes` draws the root's and the TUI's notes as today (`NOTE`), then the give up's (`REFUSED`), then `/log`'s: `Word::LogHead` (`look::HEAD`) and a log (`Role::Log`, `end_request` the note's end) holding its lines two columns in, each in `NOTE`. The notes' list capacity grows by `BOX_LOG + 2`.
6. **Tests,** a failing test first for each (`notes/common.md`, *Tests stay cheap*):
   - `tests/launchpad.rs`' `the_launchpad_says_each_state`: rows "the playground running in the project" (the banner, no headline, "playground" and the separator before the address, the sections kept), "the playground in another project" and "the playground starting" (as today).
   - `tests/foot.rs`' `quiet_states`: rows restarting, gave up, playground, the three with needs (in that order), narrow (the closed line left out first); `question_states` (or `read_states`, where lists are read): the quit question with one, two and no working chats (its head), standing over the commands' list and a chat's question, its escape `Stay`; `note_states`: the give up's note, `/log`'s note with three lines, with none ("Nothing yet."), a 600-byte line cut at 512, a line with invalid UTF-8, out of front no note.
   - `tests/commands.rs`' `the_tui_offers_its_commands`: rows in front (`/log` before `/exit`, `/exit`'s words "Close temper and stop the box"; `/l` lists `/log` alone) and out of front (no `/log`).
   - `tests/front.rs`, one golden test, `the_front_views_are_their_goldens`, rows `front-playground-80x24-dark` (frame 03: Demo, the banner, "playground · 127.0.0.1:8367", "/ for commands · playground", "Scripted model"), `front-quit-80x24-dark` (frame 04: Hello's launchpad, csv-export working, the question), `front-log-80x24-dark` (frame 05: `/log`'s note with three lines, "/ for commands · forgejo-3 restarting"), written by hand from the frames (`tests/goldens.rs`' `check`); and, in the same file, a tree with `/log`'s note of 50 lines of 512 bytes, the question and the give up's note fits `limits()` (`super::limits::fits`).
7. **Status:** delete the **Not yet** bullet naming `tui-front-views` in `temper/docs/design/client/README.status.md`. No other design line.
8. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p temper-client-tui`; `cargo check -p temper-client-shell-tui --tests`. Every existing golden stays as it is: out of front nothing changes.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`, section 7; the frames `inputs/box-front-03.txt` to `-05.txt`.
- `temper/docs/design/client/README.md`, 6.17 (*The playground*, *Quitting*, *The box's state and `/log`*, *The place*, the tokens' table); 6.14 (*The launchpad*'s header, *The foot*, *The quiet line*, *The commands*, `/help`'s note); 6.15.
- The code: `view/launchpad.rs` (`Launchpad` 24, `home` 99-139, `read` 140-187, `draw` 224-242); `view/foot.rs` (`Foot` 25, `read` 81-145, `quiet` 155-198, `draw` 200-244); `view/foot/lists.rs` (`read` 16-74, `commands` 77-108, `description` 109, `question_list` 113, `draw` 163); `view/foot/notes.rs` (`help` 25, `read_notes` 43, `draw_notes` 63); `view/chat.rs:48,193` (notes' readers); `view/work.rs` (`tui-surface`'s `running`); `temper-client-domain/src/forge.rs` (`playground`, `playground_running`); tests named above, and the goldens `foot-question-80x24-dark.txt` (04's shape) and `launchpad-80x24-dark.txt`.
- `tui-surface`'s items: the words, `PLAYGROUND`, `END_FAINT`, `Tui::{front, quitting, log_shown, box_log, box_state}`, `BoxState`, `Event::{Stop, Stay, Log}`, `LOG_COMMAND`, `limits::{STEP, LOG_LINE, BOX_LOG}`, the stubs `view/front.rs` and `tests/front.rs`.

## Owns and may change

Owns the files listed. In the same wave `tui-welcome` changes `view.rs`, `view/welcome.rs`, `tests/welcome.rs` and its goldens: not this task's. `read_notes` and `draw_notes` keep their signatures for `tui-welcome`. Not `temper-client-shell-tui` (`tui-seam`'s), not `view.rs`' `offered` (the root's commands; `/log` is added in the list and the help alone).

Options, each decided:

- **The place:** if `temper_client_tui::place` is on your base, the header's place is `place(address, tui.front())` (or with `tui.boxed()` where that is on your base too) and the goldens say `box`; else the address as today, and the goldens say `127.0.0.1:8367`; say which.
- **`/log`'s log in the launchpad's sections:** if skein's body doesn't follow a log inside the region `tui-sections` (the golden at 80 by 24 with 20 lines shows its oldest rows), draw `/log`'s note after the region, in the body unplaced, as a chat's conversation is, and say so.
- **The quiet line's width:** reuse `width` and the existing order of leaving out; if the box's state must be cut, skein cuts the left half's end as today.

## Steps

1. The failing rows and the golden test, seen failing; then `view/front.rs`, the header, the quiet line, the lists, the notes; the narrowest test after each, e.g. `heavy cargo nextest run -p temper-client-tui -E 'test(/foot::quiet_states/)'`.
2. The goldens by hand from the frames, then the test green.
3. The status edit; the runs of acceptance 8; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tui --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit (why: in front of a box the TUI says it is the playground, asks before it stops the box, and shows the box's state and log), with `Look-request: tui-box-front` directly above the other trailers; push.

## Report

As the implementer's text says, with each row and test added and the rule broken by hand for it (for instance the question below the commands' list, or `/log` offered out of front), the place drawn, where `/log`'s log is laid, and each difference from the frames.
