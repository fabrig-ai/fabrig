+++
id = "everyday-look"
part = "everyday-look"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
]
effort = "medium"
minutes = 35
+++

# The everyday Home's rules and tokens

## Goal

Every overlay rule of `project-home-everyday` goes into `temper.css`, and the classes `home-everyday`
will draw get their tokens. No view code changes. Every commit carries `Look-request:
project-home-everyday`.

Acceptance:

- Each rule of `git diff 0197eaea31 8bbc1bc964 -- temper/docs/design/client/journeys/overlay.css` is
  in `temper.css`, selectors and declarations as the overlay has them, with their comments and
  `@media` rules, but the one below. Check by a list, selector by selector; the list goes in your
  report.
- **What differs:** the overlay's `.temper-count` is the view's `temper-nav-count`
  (`look::NAV_COUNT`). The rule reads `#app > .temper-rail > .temper-rail-nav > a >
  .temper-nav-count, #app > .temper-tabbar > a > .temper-nav-count { color: var(--gf-signal);
  font-weight: 600; }`, and the rule it replaces, `#app > .temper-rail .temper-nav-count { color:
  var(--gf-muted); }`, goes.
- **`#app > .temper-home > section > h2`** is replaced by the overlay's two-selector rule, as the
  overlay has it, not added beside it. `--gf-signal-pale` and `--gf-mono` join `:root:has(#app >
  .temper-rail)`.
- **One rule of the design, not the overlay:** "Answer in a row" is the wide page's alone
  (`temper/docs/design/client/README.md`, 3.5): `.temper-home-in-a-row { display: none; }` inside the
  `@media (max-width: 760px)` block of Home's column, selected as its neighbours are.
- `look.rs` holds, in the reserved slots from 214 up and named as `HOME_*` / `ASK_*` beside their
  neighbours, with doc comments like theirs citing `client/README.md, 3.5`: `temper-home-needs`,
  `temper-home-head`, `temper-home-signal`, `temper-home-in-a-row`, `temper-home-rows`,
  `temper-home-when`, `temper-home-empty`, `temper-home-chats`, `temper-home-cta`,
  `temper-home-cta-go`, `temper-ask`, `temper-ask-meta`, `temper-ask-question`,
  `temper-ask-choices`, `temper-ask-quiet`.
- Rules for what nothing draws yet (`temper-home-byline`, `temper-home-facts`,
  `temper-ask-context`, `temper-ask-recommended`, `temper-ask-rec`) land in `temper.css` and take no
  token (`notes/contracts.md`, Tokens).

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/contracts.md` (Tokens).
- `git show --format=%B -s 8bbc1bc964`: the message.

## Owns and may change

Owns, as `files` lists them. Nothing else.

Options, each pre-approved:

- If `tests/look.rs` pins a token list, add yours to it.
- Where `fetch-progress-1` has moved a neighbouring line of `temper.css`, leave it as you find it.

## Steps

1. The rules, in Home's column's block of `temper.css`, in the overlay's order.
2. The tokens.
3. **Check,** `notes/common.md`: `heavy cargo nextest run -p temper-client-view -E
   'test(look)'`, then `-p temper-client-gallery -p temper-client-journeys`, which read the
   stylesheet; clippy on `temper-client-view`; push. Nothing reprints: `temper.css` is linked, not
   copied.

## Report

As `notes/common.md`, Your report: the rule list, one line each, and the tokens' numbers.
