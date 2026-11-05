+++
id = "viewport-bundle"
part = "viewport-bundle"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui-bundle/src/page.rs",
  "skein/crates/skein-ui-bundle/src/tests/golden/index.html",
  "skein/docs/design/ui-web.status.md",
  "skein/docs/design/STATUS.md",
]
effort = "medium"
minutes = 15
+++

# The bundler's page covers the display

## Goal

temper's served page is the bundler's (`temper/crates/temper-client-bundle/build/steps.rs` passes no `--page`), and its viewport lacks `viewport-fit=cover`, so a notched phone's `env(safe-area-inset-bottom)` is always 0. The bundler's page prints the viewport the design now says.

Acceptance:

- `skein-ui-bundle`'s page prints `<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">`, and only that line of its page moves.
- `the_page_matches_its_golden` passes on the new line.
- `skein/docs/design/ui-web.status.md` says `No gap.` again, and `skein/docs/design/STATUS.md` lists `ui-web.md` among those with no gap as before.

## Inputs

- `skein/docs/design/ui-web.md`, 4.3.
- `temper/docs/plan/ui-phone/notes/common.md`.

## Owns and may change

Owns the `files` above.

- **Departure from your text:** you close the two status lines above, as `notes/common.md` says.
- **The golden** has no tool: the test reads it by `include_bytes!`, so its one line is edited by hand. If any other line of it would move, that is a red: stop and read why.
- An application's own page (`--page`) stays copied as bytes; no new option on the bundler.

## Steps

1. **The line,** in `skein/crates/skein-ui-bundle/src/page.rs`, `render`, and in `src/tests/golden/index.html`.
2. **Status:** `ui-web.status.md` back to its `No gap.`; `STATUS.md`, the sentence reading "`forge-client.md`, `oauth-issuer.md`, `ui-text.md`, `ui-web.md` and `browser.md` have no gap."
3. **Check:** `heavy cargo nextest run -p skein-ui-bundle`, `heavy cargo nextest run -p skein-ui-web-tests`, `heavy cargo clippy -p skein-ui-bundle --all-targets -- -D warnings`, `cargo fmt`, the citations check (`python3 -I -B tools/citations/citations.py --base origin/main`). No browser world: nothing there reads the meta (`rg initial-scale skein temper/tests jig/tests` finds only this page and its golden). Commit and push.

## Report

As `notes/common.md`, Your report.
