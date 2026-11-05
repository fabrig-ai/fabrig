# ui-phone: the sweep

The request's section 3 is the plan's check, kept as a script for the plan's life, not in the tree: never tracked, never in a worktree, in `~/src/fabrig/ui-phone/sweep/` on your builder. Builders don't share a disk: if it isn't on yours, write it again from this note, and say so in your report. Its form (Python, Node, the language is yours) is yours; what follows binds what it checks.

## The driver

- **The gallery's prints:** `heavy cargo run -p temper-client-gallery -- ~/src/fabrig/ui-phone/gallery`, printed afresh from your own branch before each run. It prints one page per state in `temper/tools/gallery/names.txt` (229 when this was cut). They hold no WebAssembly: open states are set by the script, `details.open = true`.
- **Chromium,** `--headless=new`, by CDP, each width by `Emulation.setDeviceMetricsOverride` with `mobile: true` and a height of 844, with `Emulation.setTouchEmulationEnabled`, so that `(hover: none)` holds. Handle `Page.javascriptDialogOpening`. Never by `--window-size`.
- **The mock:** `temper/docs/design/client/greenfield/*.html` from the checkout, by `file://`. Its scenarios live in `localStorage`, set by its "Mockup" panel (`app.js`).
- **Results** go in `~/src/fabrig/ui-phone/sweep/out/<your task>/`: one line per failure, `state width check element measure`, and a count per check at the end. Your report quotes the counts.

## What counts as shown

An element is shown if `element.checkVisibility({ visibilityProperty: true })` holds, which is false inside a closed `details` (`content-visibility: hidden`) and under `display: none`, and if its box is over 1 px each way and it isn't clipped away (`clip-path: inset(50%)`, the visually hidden labels). Only what is shown is measured.

## The checks at 360, 390 and 420, every gallery state

1. **Past the edge:** `document.scrollingElement.scrollWidth` is at most the viewport's width, and no shown element's right edge passes it by over half a pixel, unless an ancestor scrolls that way (`overflow-x` of `auto` or `scroll`).
2. **The last control:** the last shown, enabled `a`, `button`, `input`, `select`, `textarea` or `summary` inside the main: `scrollIntoView({ block: 'end' })`, then `document.elementFromPoint` at its centre answers it or a descendant, and its bottom is at or over the top of the tab bar if one shows. In a chat, the composer's send counts as that.
3. **Tall enough:** each `.temper-tabbar a`, and each shown `.temper-flow-primary`, `.temper-goal-primary`, `.temper-home-primary`, `.temper-settings-primary`, and the primary button of a sheet or a dialog, is at least 44 px tall. Where the mock draws that same control under 44 at 390, the mock wins and your report names it.
4. **Open states:** the switcher (`.temper-projects`) and the menu (`.temper-menu`) opened, and any shown sheet or `dialog`: the panel's box lies wholly within the viewport.

A page that scrolls inside a panel of its own passes check 4 when the panel's box does.

## What a failure is

Group failures by cause, not by state. Fix the cause in that page's own place (`notes/common.md`, `temper.css`). A failure on a page `ui-chat-draft` or `ui-away` owns is named, not fixed. What takes more than a rule, or a small view change, goes under your report's `## Left` with the states it covers.
