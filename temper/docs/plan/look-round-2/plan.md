# look-round-2: the plan

From `inputs/request.md`: land the five look requests on `origin/look-requests` at `7bbb7ec2d7`,
none held, by the procedure of `temper/docs/design/draft/look-requests.md`, sections 3 to 5. One
task per request, oldest first, each needing the one before: they build on one another's pages and
share the stylesheets. The trunk is `look-round-2/integration`, so it never meets the person's
branch.

No design is cut ahead of a landing, as the request says: each landing brings the design in line
(section 3, steps 2 and 8). So each spec departs from the implementer's text on `docs/design/`, and
`notes/common.md` states the departure.

## Parts and tasks

| # | Part and task | Commit | Effort, minutes | Lands | Deferred |
|---|---|---|---|---|---|
| 1 | `welcome-model-forge-pane` | `87cddc4453` | high, ~150 | whole: the region, the stacked line, every state, the narrow width, the terminal; three gallery states | nothing |
| 2 | `ready-account-email` | `0a94e5f1fb` | medium, ~45 | the gallery's email-like display | the email itself, to jig's account-facts draft, tracked in `temper/docs/design/client/README.status.md` |
| 3 | `welcome-forge-busy-bar` | `95532806d8` | high, ~90 | whole: the bar, the swap, the token, the keyframes | nothing |
| 4 | `welcome-fetch-percentage` | `218a9b9080` | medium, ~50 | the meter's token and rule, ready for a value | the percentage to `temper/docs/design/draft/fetch-progress.md`, the fill to a skein draft, both tracked in its status |
| 5 | `welcome-claude-card-words` | `7bbb7ec2d7` | medium, ~45 | whole: the card's two sentences | the card missing from a real box, to `temper/docs/design/draft/claude-provider.md` |

## Critical path

The whole chain, 1 to 5, ~380 implementer-minutes, then the trunk's end check. Nothing runs beside
anything else, as the request asks.

## What the survey found, and the specs settle

- **The terminal is the trap.** `skein-ui-text` joins a paragraph's runs with no separator, and
  prints a named progress node as `NAME: TEXT`. So the stacked line would read
  "Forgejo on this machineSetting up: fetchingwork/hello", and a textless bar
  `Setting up: fetching: `. No test prints the Model step today. The view reads `w.shell`: the
  terminal's line stays one line of the name's words with its "✓ ", and the bar is the browser's
  alone. Tests print through `skein_ui_text`.
- **A rule the message doesn't name would hurt the done state.** `temper.css`, 978, colours the
  line's `::first-letter`: with the ✓ gone from the text, it would turn the forge's name's "F" green.
  No journey reaches done, so only the render check would see it. The spec removes it, and also 1387,
  whose margin would beat the overlay's at the narrow width.
- **States in words need gallery states.** No state draws the Model step's forge line. Task 1 adds
  `welcome-model-forge-under-way`, `-done` and `-failed`, from `FirstRun` steps 6, 7 and 9.
- **GitHub's waits are `underway` but run nothing.** The bar leaves them out, and the account says
  so.
- **"The fill rule waits" and "land the rule ready for a value"** are one thing: the rule lands with
  its fill at `var(--temper-flow-meter-fill, 0%)`, and no node carries the token.
- **The gallery's value** is `sam@example.com`, the first-run world's person, as `browser.md`, 7.2,
  has the worlds' values, not the request's hand-written `you@example.com`.
- **A draft not on `main` can't be cited by path:** the citation check refuses it. Task 2 names jig's
  account-facts draft in words.

## Left outside

- **The draft's own design document** (`look-requests.md`, its head and status): the request wants
  no new design of its own, so the draft's **Design lags** stand.
- **The render-check command:** that draft's section 8, Later.

## Overlaps

- **`serve-without-box`, not yet landed:** `welcome-inbox` changes `pages/welcome.rs` at the head of
  `main`, not the Model arm, and adds tests at the head of `tests/welcome.rs`; `no-project-page`
  changes `words.rs` and the terminal's `tests/render.rs`; its trunk changes `forge/connect.rs` and
  `temper/tools/gallery/src/worlds.rs`, not at the lines these tasks touch. New tests go in a new
  file, `tests/model_forge.rs`, for this reason. The edits to `tests/welcome.rs`'s helpers and to
  `README.md`, 3.5, and `README.status.md` could not be kept off those files.
- **`temper/docs/design/draft/claude-provider.md`:** left to its own change.
