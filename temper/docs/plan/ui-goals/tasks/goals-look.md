+++
id = "goals-look"
part = "goals-look"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/files/fonts/ibm-plex-mono-latin-400-normal.woff2",
  "temper/crates/temper-client-shell-web/files/fonts/ibm-plex-mono-latin-500-normal.woff2",
  "temper/crates/temper-client-shell-web/files/fonts/README.txt",
]
effort = "medium"
minutes = 50
+++

# Goals' rules and tokens

## Goal

The stylesheet can draw Goals as `mock.css` does, and the mono face reaches these pages. Nothing draws them yet.

Acceptance:

- Every token of `temper/docs/plan/ui-goals/notes/contracts.md`, The look, in `look.rs` and `NAMES`, each with its
  doc comment citing `client/README.md`, 3.5.
- `temper.css` holds the request's rules (`temper/docs/plan/ui-goals/inputs/request.md`, 4), values and all, under the
  marker `/* ==== ui: goals ==== */`, each scoped under `#app:has(> .temper-rail)`, with `--gf-*` for the mock's variables.
- IBM Plex Mono 400 and 500 are bundled as the sans is: the files, `@font-face`, a `README.txt` row each.
- `--gf-mono` is `'IBM Plex Mono', 'JetBrains Mono', ui-monospace, monospace`; a row's top is `font: 12px var(--gf-mono)`,
  colour faint. `#app:has(> .temper-rail)`'s `--temper-mono` is left as it is, as other pages read it: Goals' rule uses `--gf-mono`.

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-goals/inputs/request.md`, 4; `temper/docs/design/client/browser.md`, 2.5, *Goals' look*.
- `temper/docs/design/client/greenfield/mock.css`, its variables.

## Owns and may change

Owns, as `files` lists them. May also change `temper/crates/temper-client-shell-web/files/fonts/LICENSE-ibm-plex.txt`
only if it doesn't cover Mono (it's one family's licence; say so).

Mapping the rules: `.col` is `.temper-goals`, `.intro` `.temper-goals-intro`, and so on by the contract's names;
`.goal-row .top` is `.temper-goals-top` and `.mono`'s declarations join it; `.goal-row.ended .line` is
`.temper-goals-row-ended .temper-goals-line`; `.state-failed` isn't a rule since `failed` takes `state-stopped`;
`.section-title.signal` is `.temper-goals-section-title.temper-goals-signal`; `.main`'s padding at 760 px and under is
Home's, already there, so it isn't copied.

Options, each pre-approved:

- A variable the frame lacks as `--gf-*` takes its value from `mock.css`'s `:root`, added beside its neighbours.
- `.filters button` selects by `button` inside `.temper-goals-filters`; `aria-pressed` is what `State::Pressed` draws in the
  browser: check on a button, and if it draws otherwise, select by what it does.
- Fonts come as the sans came: `@fontsource/ibm-plex-mono` at 5.0.8 on `cdn.jsdelivr.net`, into a new empty directory, then
  copied; each row states source and SHA-256. A failed fetch after ten minutes' retries is `Infra: fetch`.
- A bundle size test that fails by these files: raise nothing; report it as `Question:` with its measurements.

## Steps

1. **`look.rs`:** tokens from the next free number, `NAMES` in step (fillers as the list has them).
2. **`temper.css`,** then a render of a throwaway page of these classes beside the mock's `goals.html` if you doubt a rule.
3. **Tests,** `tests/look.rs`: `GOALS_NAMES` as `HOME_NAMES`, chained in its loop.
4. **Check:** `heavy cargo nextest run -p temper-client-view --lib -E 'test(look)'`, clippy on that package, then push.

## Report

As `notes/common.md`, Your report, with each variable you added and the fonts' sizes.
