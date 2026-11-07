+++
id = "temper-flow-look"
part = "temper-flow"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-shell-web/temper.css"]
effort = "medium"
minutes = 10
+++

# temper's look for a flow

## Goal

Give skein-ui's flow tokens temper's look, the one `skein/docs/design/draft/diagrams/preview.html` draws, in `temper.css` under the chats' marker. The rules touch nothing else, and nothing draws a flow until `temper-flow` lands, so this can land first.

Acceptance:

- The block below stands in `temper.css`, under `/* ==== ui: chats ==== */`, directly after that marker's `model-picker` rules or anywhere within that marker's rules. No other rule changes.
- The tests that read `temper.css` pass: `heavy cargo nextest run -p temper-client-gallery --lib -E 'test(stylesheet) | test(css) | test(sign_out)'`, and, if `rg -l 'temper.css' temper/crates temper/tools --glob '*.rs'` names another test file, that file's tests by name.

## Inputs

- `temper/docs/design/client/browser.md`, 2.5, *A flow*.
- `skein/docs/design/draft/diagrams/preview.html`, the `.skein-ui-flow-*` rules, read and never edited.
- `temper/docs/plan/diagrams/notes/contracts.md`, *The tokens' selector*.
- `temper/docs/plan/diagrams/notes/common.md`.

## Owns and may change

Owns the block, in `temper.css`. Nothing else.

`--gf-*` are set on `:root:has(#app > .temper-rail)` and the other frames, with the values `preview.html` writes out: `--gf-paper` #f7f7f5, `--gf-line-2` #d4d4cf, `--gf-ink` #18181b, `--gf-surface` #ffffff, `--gf-muted` #5b5b63, `--gf-faint` #6b6b73.

A flow's figure is `<figure>` with no class, since a Markdown drawing has no token: `:not([class])` keeps `Role::Figure`'s styled figures in `pages/goal.rs` and `pages/needs.rs` as they are.

## Steps

1. Add, whole:

   ```css
   /* A flow (client/browser.md, 2.5): diagrams/preview.html's look, and no margin, so its first measure is its width. */
   :where(#app) figure:not([class]) { margin: 0; }
   [data-drawn] > svg .skein-ui-flow-box { fill: var(--gf-paper); stroke: var(--gf-line-2); }
   [data-drawn] > svg .skein-ui-flow-box.skein-ui-flow-strong { fill: var(--gf-ink); stroke: var(--gf-ink); }
   [data-drawn] > svg .skein-ui-flow-name { fill: var(--gf-ink); stroke: none; font: 500 13px var(--gf-sans); }
   [data-drawn] > svg .skein-ui-flow-name.skein-ui-flow-strong { fill: var(--gf-surface); }
   [data-drawn] > svg .skein-ui-flow-caption { fill: var(--gf-muted); stroke: none; font: 400 12px var(--gf-sans); }
   [data-drawn] > svg .skein-ui-flow-arrow { fill: none; stroke: var(--gf-faint); stroke-width: 1.5; stroke-linecap: round; stroke-linejoin: round; }
   ```

2. The narrow tests above. No clippy is needed for a stylesheet. Commit and push.

## Report

The implementer's report, with which tests read the stylesheet.
