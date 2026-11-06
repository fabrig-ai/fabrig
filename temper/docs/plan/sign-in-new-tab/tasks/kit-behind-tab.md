+++
id = "kit-behind-tab"
part = "web-tab"
kind = "feature"
needs = ["kit-tabs"]
files = ["skein/testing/skein-browser/src/**", "skein/testing/skein-fake-person-browser/src/**", "skein/tests/browser/**", "skein/docs/design/browser.md"]
effort = "high"
minutes = 60
+++

## Goal

A person's page answers the kit once a tab of its own has opened beside it: a story sees, presses and captures that page as it did before the press. A page that stays while a tab opens is what `temper/docs/design/draft/sign-in-new-tab.md`, 3 draws, and `browser-story` cannot hold without it.

What three runs saw, all on Chromium 147.0.7727.101 under the kit's `--headless`, the cause established by none:

- After a press's `window.open`, a fresh `Find` (await, status, button, heading by name or by text) answers none, while `Accessibility.getFullAXTree` for the failure's snapshot holds that node.
- `Page.captureScreenshot` on that page is refused `Timeout`; a fresh find has timed out too.
- A press on a node found before the opening works, and so does a `DOM` attribute read: `kit-tabs` and `web-tab-story` went round the fault that way, and both say so in their reports.

The likely reading, read and not run: the new tab takes the front, and Chromium stops rendering the page behind it, which the accessibility query and the capture wait on or read stale. Establish it, or say what it is instead.

Acceptance:

- A real Chromium test in `skein-browser-world` (`skein/tests/browser/tests/browser_real.rs`, the `Scenario::Tabs` rig or one beside it): once a press has opened a tab, a fresh find by role and name sees a heading the press's listener wrote after the opening, a fresh find sees a button, a press lands on that button, and a screenshot answers. It fails before the fix, for the reason the report names.
- The opening's tab goes on loading behind the page: a test holds that the tab reaches an address it was pointed at, and the facts `chromium_lists_a_pressed_tab_and_its_cut_opener_but_blocks_an_unpressed_opening` holds stay as they are.
- `skein/docs/design/browser.md`, section 6, says in its table's words what the kit does for a page with a tab beside it.

## Inputs

- `skein/docs/design/browser.md`, sections 1 (*A test sees what a person sees*) and 6.
- `skein/docs/design/ui-web.md`, section 10, *A tab of the page's own* and *A hidden tab lets its stream go*; 14.2, *Its tabs*.
- `temper/docs/plan/sign-in-new-tab/notes/common.md`.
- The asking run's partial story, as a downstream check to read and not to change: `temper/tests/client-browser/tests/browser_all/browser_accounts.rs` on the branch `sign-in-new-tab/browser-story`.

## Owns and may change

- Owns the files in the header. Change nothing in temper; `browser-story` takes its stories on from this.
- The fix is the kit's, or its command's, and holds these:
  - No script in the page but the `document` expression, no read of page state, and no attaching to the tab (`browser.md`, sections 1 and 6).
  - The cheapest mechanism that makes the test pass and that the report can explain. Try in this order: a page brought to the front as a person does before looking (`Page.bringToFront` on its session, where the kit looks, so that a page already in front costs no more than the polling allows); then focus emulation (`Emulation.setFocusEmulationEnabled`) at the page's set-up; then a flag on `command`. Say which were tried and what each did.
  - A page's hidden-tab rule (`ui-web.md`, section 10) still has a meaning: the report says what `document.visibilityState` a page has, seen by the test's page itself and not by a script of the kit's, under the fix, and `browser.md` says when a story's page counts as hidden.
- `web-tab-story`'s story stays as it is: its held nodes still pass.
- If no mechanism inside these limits holds, or the fix moves a test's expectation elsewhere in skein: stop and ask, with what each try did.

## Steps

1. The failing real test first, reading the reply that comes back: `heavy cargo nextest run --profile browser-skein -p skein-browser-world --test browser_real -E 'test(=NAME)'`, with `chromium` on the builder's `PATH` or `SKEIN_TEST_CHROMIUM`. If the builder has no Chromium, say so plainly and don't count it as run.
2. The fix, then the test beside the machine's own, a fake-peer test holding the commands the kit sends if the fix adds one.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-browser -p skein-fake-person-browser -p skein-browser-world --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-browser -p skein-fake-person-browser -p skein-browser-world`. Commit and push.

## Report

As the implementer's text asks. Give the cause as observed, Chromium's version, the mechanisms tried, and the `browser-skein` tier's wall time for what was run. Firefox and Safari are not the kit's: it drives Chromium alone.
