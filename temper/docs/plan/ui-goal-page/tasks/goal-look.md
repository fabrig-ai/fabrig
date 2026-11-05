+++
id = "goal-look"
part = "goal-look"
kind = "feature"
needs = ["needs-base"]
files = [
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
]
effort = "medium"
minutes = 50
+++

# A goal's look

## Goal

The tokens and the rules of a goal's page and of the sheet, so that the view's tasks need only name them. Nothing
draws them yet.

Acceptance:

- Each const of `temper/docs/plan/ui-goal-page/notes/contracts.md`, Names and tokens, stands in `look.rs`, with its
  name in `NAMES`, in the table `tests/look.rs` holds (as `GOALS_NAMES`), and passes
  `each_style_or_region_is_named_under_temper` and `the_looks_names_pass_skeins_check`.
- Every rule of `temper/docs/plan/ui-goal-page/inputs/request.md`, section 6, stands in `temper.css` under the marker
  `/* ==== ui: goals ==== */`, its values exact, each selector the token's, written as `#app:has(> .temper-rail)
  .temper-goal-… { … }`, with `var(--gf-*)` for the mock's variables, as `.temper-goals-*` are.
- The bundle's stylesheet check still passes.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, `notes/contracts.md`; the request's section 6.
- `temper/docs/design/client/browser.md`, 2.5, *A goal's look*.
- `temper/docs/design/client/greenfield/mock.css`, by selector, for what the request's list abbreviates.

## Owns and may change

Owns, as `files` lists them. Don't remove `task-pages.css`' rules or any old token: `goal-route` does.

Options, each pre-approved:

- **A rule's selector** follows the table: `.btn.small` is `.temper-goal-btn.temper-goal-small`; `.task-line
  span:last-child` is `.temper-goal-line > :last-child`; `.bar span` is `.temper-goal-bar > *`; `.typing i` is
  `.temper-goal-typing > *`, their `:nth-child`s likewise. `.option:has(input:checked)` keeps `:has`.
- **The mock's colours with no `--gf-*` of their name:** `#b4b4ae`, `#a1a1aa`, `#a8a8a2`, `#fafaf9`, `#000`, `#fff`,
  `#b42318` and the `rgba`s stay literal, as `mock.css` writes them. Any variable missing from `--gf-*` (`--signal`,
  `--signal-pale`, `--signal-line`, `--hint`, `--good`): add it beside the others with the mock's value, or use the one
  `ui-needs` added.
- **At 760 px:** in this marker's own `@media`, as the mock's. `.choices` and its `.btn` rules are the band's: `.temper-goal-choices`.
- **A token's number:** the next free ones. If merging moves them, renumber: `NAMES` is positional.
- `.typing i`'s `animation: typing`: copy the mock's `@keyframes typing`.

## Steps

1. The tokens, their names, the test's table.
2. The rules, copied by selector.
3. `heavy cargo nextest run -p temper-client-view --lib -E 'test(look)'`; the stylesheet's test in
   `temper-client-shell-web`.
4. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report, naming any variable you added.
