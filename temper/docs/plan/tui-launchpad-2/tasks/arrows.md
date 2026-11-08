+++
id = "arrows"
part = "left-off"
kind = "feature"
needs = ["seam", "left-off"]
files = ["temper/crates/temper-client-shell-tui/src/host.rs", "temper/crates/temper-client-shell-tui/src/tests.rs"]
effort = "high"
minutes = 45
+++

## Goal

The host's half of the plan: Up and Down reach the view's `arrow`
before skein's key reader, so that the launchpad's left-off chats are
reached by the arrows (request item 1); words typed on the launchpad
clear the selection; and the start's two facts (task `seam`) reach what
draws by them: the theme's band (item 3) and the TUI's `boxed` (the
look request `tui-chat-title-below`).

Acceptance (`temper/docs/design/client/README.md`, 6.13, the keys'
paragraph and the events table; 6.14, *Where you left off*):

- `Report::Key(Key::Up | Key::Down)`: the host asks
  `tui::arrow(&view, Arrow::Up | Arrow::Down)` first; with `Some(event)`
  it gives the event and skein's key reader is not asked; with `None`,
  as today. `Host::event_of` gives the same event for those keys.
- `Event::Typed` on the launchpad with a project (`Act::ChatWords`) also
  tells the TUI's state `Told::LeftOff { task: None }`.
- The theme drawn after each report is `tui::theme(scheme, background)`,
  `background` the start's; `Told::Boxed` is given to the TUI's state at
  `Start` when `boxed`. The `#[expect(dead_code)]` task `seam` put on the
  host's two fields, if any, is gone.
- Rows of the host's existing tables in `src/tests.rs`, on a launchpad
  in a project with two live chats:
  - Down: `event_of` is `LeftOff { task: Some(NEWEST) }`; given, the
    state's `left_off()` is it, the frame's row for that chat starts
    ` ❯ `; Down again selects the other; Up returns.
  - Enter then: the root's address is `Chat { task: NEWEST }` and the
    chat's reads are among the duties, as for a chat taken from
    `/chats`.
  - Esc with a chat selected clears it; a typed `a` clears it and Down
    then makes no event (`event_of` is `None`).
  - With `/` typed, Down moves the commands' active item as before (the
    frame's `❯` on the second command) and makes no `LeftOff`.
  - `Start` with `background: Some(#191816)`: a chat's person's words'
    cells have the background `#2d2c2a`; with `None`, those cells have
    no background and the words are dim.
  - `Start` with `boxed: true`: `state().boxed()` is true; false
    otherwise.
- The joint world's TUI stories still pass (`heavy cargo nextest run -p
  temper-client-world -E 'test(/tui/)'`, read the count).

## Inputs

- `temper/docs/design/client/README.md`, 6.13 (*A key*'s paragraph, the
  events table, *The frame*, *The look*), 6.14 (*Where you left off*)
  and 6.15 (*The band*).
- `temper/docs/plan/tui-launchpad-2/notes/common.md` and
  `notes/contracts.md`, sections 1 and 2.

## Owns and may change

- Owns the files in `files`; in `host.rs`, the key path, `event_of`, the
  `Typed` handling, the theme's call and the start's handing on.
- Pre-approved: if the view's `arrow` or Enter's `Act::Go` don't behave
  as task `left-off`'s acceptance says, that is a red of `left-off`'s:
  fix it in `temper/crates/temper-client-tui/src/view.rs` with the
  smallest change and a row in `tests/launchpad.rs`, and say so.
- Recorded journeys: none walks an arrow; if a replay goes stale, stop
  and report it (`Failed:`), since nothing here should change a duty.

## Steps

1. `host.rs`: the arrows before `command`, in `report` and `event_of`;
   the clearing on `Typed`; `theme(scheme, background)`; `Told::Boxed`.
2. Rows as above; break each once (ask `command` first, drop the
   clearing, pass `None` as the background), see its row fail, restore.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-shell-tui --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-shell-tui -p
   temper-client-journeys`; the joint world's TUI stories above.
4. Commit and push.

## Report

As the implementer's text says.
