+++
id = "phone-walk"
part = "phone-walk"
kind = "feature"
needs = ["phone-sweep-gallery"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
]
effort = "high"
minutes = 60
+++

# One box walk at phone size

## Goal

A person on a phone can get from nothing to a chat and back. `temper box --fake-backends`, in Chromium by CDP at 390 by 844: the welcome, Forgejo and a model, Create, Home, "Start a chat", a message sent, back by "‹" to Chats.

Acceptance:

- **The walk runs whole** on the real client over a fresh fake box. Each press is a real one: `Input.dispatchMouseEvent` at the control's centre, after a hit test says that the control is what's there. `element.click()` doesn't count: it presses what a person couldn't.
- **The served page's head** holds `viewport-fit=cover`: its `index.html`, read from the box.
- **A screenshot at each step,** in `~/src/fabrig/ui-phone/walk/` on your builder, never committed.
- **What stopped the walk at this size is fixed,** if it's the phone's: a covered control, a control off the screen, a page that doesn't scroll. Anything else stopping it goes under `## Left`, with the step and its screenshot.

## Inputs

- `temper/docs/help/box.md`, *Fake backends*.
- `temper/tools/journeys/src/main.rs` (the bundle, `scripts/bundle.sh --loopback --record`, and what it builds) and `src/again.rs` (how it starts a box); `temper/docs/design/protocol/configuration.md`, 12.1.
- `temper/crates/temper/tests/fake_backends/walk.rs`, for what a fresh box asks.
- `temper/docs/plan/ui-phone/notes/common.md`, `temper/docs/plan/ui-phone/notes/sweep.md`, The driver.

## Owns and may change

Owns the `files` above.

- **The build:** `heavy cargo build -p temper -p smith-fake-claude`, then the bundle by `temper/scripts/bundle.sh --loopback ~/src/fabrig/ui-phone/bundle` (`docs/help/box.md`), through heavy. If heavy takes cargo commands alone, run the script's own cargo steps each by `heavy cargo`, then its bundler as the script does. The box and the browser run on your builder, outside any worktree: their data in `~/src/fabrig/ui-phone/walk/data`, and stopped before you end.
- **The driver** is a throwaway outside the tree, beside the sweep. A pending request's `onbeforeunload`: handle `Page.javascriptDialogOpening`.
- **Reader files, a line or two each,** where the sheet alone can't: a page's view in `temper/crates/temper-client-view/src/pages/` and `look.rs`, with what follows regenerated.
- **A wait** is on what the page shows, never a fixed sleep; the box answers in its own time.
- **Nothing in the tree** records this walk: no journey, no test. A fix's test, if it needs one, goes where the page's others are.

## Steps

1. **Build,** start the box, drive the steps, screenshot each.
2. **Fix** what stopped it, and walk again from a fresh box.
3. **Check:** the implementer's text; `cargo fmt`; commit and push. If nothing stopped it, push nothing and say so in the first line.

## Report

As `notes/common.md`, Your report: each step, what it showed, its screenshot's path, and how many walks it took.
