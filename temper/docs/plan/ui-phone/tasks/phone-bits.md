+++
id = "phone-bits"
part = "phone-bits"
kind = "feature"
needs = ["viewport-bundle"]
files = [
  "temper/crates/temper-service/src/setup/routes.rs",
  "temper/crates/temper-client-shell-web/playground/index.html",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/lib.rs",
  "temper/tools/journeys/src/print.rs",
  "temper/docs/design/client/journeys/*/*.html",
  "temper/docs/design/client/browser.status.md",
]
effort = "medium"
minutes = 45
+++

# The phone's own bits

## Goal

Every other page head temper prints takes `viewport-fit=cover`, what sits at the foot keeps off the home bar, and the closed menu on a page of no project's is measured and answered. The request's section 2.

Acceptance:

- **Heads:** `rg 'initial-scale=1\\?"' temper/crates temper/tools temper/docs/design/client/journeys` finds nothing. The mock's words: `width=device-width, initial-scale=1, viewport-fit=cover`.
- **The foot:** at 760 px and under, `.temper-toast` has `bottom: calc(88px + env(safe-area-inset-bottom))` and `.temper-session-foot` has `padding: 8px 16px calc(14px + env(safe-area-inset-bottom))`, each edited where it stands. With no inset they measure as they did. The tab bar is left as it is: it already has its inset.
- **The closed menu:** at 420 with the menu closed, `signed-out`, `google-sign-in` and `out-of-date` are answered by what shows (`notes/sweep.md`, What counts as shown) and a screenshot. If `.temper-corner` shows past the edge, keep it within the page under `/* ==== ui: phone ==== */`. If it's hidden, leave it and say so in your report.
- `temper/docs/design/client/browser.status.md` loses its 2.5 entry on `viewport-fit=cover`.

## Inputs

- `temper/docs/design/client/browser.md`, 2.5 (*The page covers the display*).
- `temper/docs/design/client/journeys.md`, 5.1 and 5.2, for the reprint.
- `temper/docs/plan/ui-phone/notes/common.md`; `temper/docs/plan/ui-phone/notes/sweep.md`, What counts as shown.
- `temper/docs/plan/ui-phone/inputs/request.md`, section 2.

## Owns and may change

Owns the `files` above.

- **Where the heads are:** `routes.rs`, `HEAD`; `playground/index.html`; `gallery/src/lib.rs`, `page` and `index`; `journeys/src/print.rs`. The served page is `viewport-bundle`'s, already on your base.
- **`routes.rs`:** `HEAD` isn't the pinned style, so `PAGE_STYLE_HASH` and `MANIFEST_STYLE_HASH` hold. If a hash test goes red, you touched `STYLE`: undo that.
- **The journeys' prints** are regenerated, never edited: `heavy cargo run -p temper-client-journeys`. Only the viewport line of each of the 11 prints should move. A stale journey on your base: record it again (`journeys.md`, 8.1), a mark that catches a moving page retried up to three times on a fresh fake box, and if all fail, widen the mark's wait in the recording driver and say so.
- **The menu:** reading `temper-client-view/src/frame.rs`, `banner`, the corner sits inside the closed `MENU` disclosure, so hidden is the likely answer. That's for the screenshot to settle. A fix stays in the sheet; the view's tree doesn't move.
- **Option, safe-area proof:** try CDP's `Emulation.setSafeAreaInsetsOverride` with a bottom of 34 on a printed `goal-toast` and `chat-working`: the tab bar, the toast and a chat's foot rise by 34. If the method is missing on the builder's Chromium, read the computed styles instead and say so.

## Steps

1. **The heads,** then `heavy cargo run -p temper-client-gallery -- ~/src/fabrig/ui-phone/gallery` and the reprint.
2. **The foot:** the two rules, at their places in `temper.css` (under `ui: needs` and `ui: settings`).
3. **The menu:** measure, screenshot, answer.
4. **Check:** the implementer's text: `heavy cargo nextest run -p temper-client-gallery -p temper-client-journeys`, `heavy cargo nextest run -p temper-service --lib -E 'test(setup_lifecycle)'`, clippy on what you changed, `cargo fmt`, the citations check. Commit and push.

## Report

As `notes/common.md`, Your report, with the closed menu's answer for each of the three states and its screenshot's path.
