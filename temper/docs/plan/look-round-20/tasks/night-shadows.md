+++
id = "night-shadows"
part = "shadows"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-web/temper.css",
]
effort = "medium"
minutes = 30
+++

# The night's shadows keep the day's colours

## Goal

The request's task 4 (`temper/docs/plan/look-round-20/inputs/request.md`): the night set `look-round-19` landed (`html:root[data-night]:has(#app)`, about lines 148 to 283 of `temper.css`) gives twelve shadow colours `rgba(0, 0, 0, .4)`, where the look request `night-mode` (`3c5333485a`) sets no shadow's colour and so keeps each one's own. Remove them from the night set, so that each shadow takes its day token at night, through the tokens; the rings the request draws stay.

Acceptance, by `temper/docs/design/client/browser.md`, 2.5, *Day and night*, *Shadows keep their colour*:

1. **The night set names no shadow's colour:** its twelve lines `--temper-menu-shadow`, `--temper-projects-shadow`, `--temper-entry-shadow`, `--temper-dialog-shadow`, `--gf-shadow-soft`, `--gf-dialog-shadow`, `--gf-shadow`, `--gf-menu-shadow`, `--gf-selection-shadow`, `--gf-latest-shadow`, `--gf-shadow-faint` and `--gf-starter-shadow` (each `rgba(0, 0, 0, .4)` when cut) are deleted, so that at night each holds its day value. The request's five, checked against the day set: the project picker and person menu `0 12px 32px rgba(24, 24, 27, .12)` (`--gf-shadow`; the person menu's rule, about line 2676, read to confirm it), the model menu `.14` (`--gf-menu-shadow`), the starter and scale cards `0 1px 2px rgba(24, 24, 27, .04)` (`--gf-shadow-faint`), the selected Split `0 1px 2px rgba(0, 0, 0, .08)` (`--gf-selection-shadow`), Latest `0 2px 8px rgba(24, 24, 27, .1)` (`--gf-latest-shadow`).
2. **The rings stay,** as the overlay draws them: `--gf-shadow-edge` and `--gf-home-edge` (`var(--gf-line)`), `--gf-home-begin-shadow` (`0 0 0 1px var(--gf-line)`), `--gf-effort-shadow` (`none`), `--gf-composer-focus-shadow` (`0 0 0 3px var(--gf-hover)`), unchanged.
3. **No other shadow at night:** `grep` the file and `task-pages.css` for `box-shadow` under `:root[data-night]` or `html:root[data-night]` and for a night value of a shadow token outside the set; any that gives a shadow a night colour the request doesn't is named in the report and, where it is one of acceptance 1's, removed. No rule's day value changes.
4. **Status:** delete the **Not yet** entry naming `night-shadows` in `temper/docs/design/client/browser.status.md`. No other design line.
5. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-shell-web` (its `tests/suite` read `temper.css`). No clippy: no Rust changes.

One commit or more, each `Look-request: night-mode`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-20/notes/common.md`.
- `temper/docs/plan/look-round-20/inputs/request.md`, task 4.
- `temper/docs/design/client/browser.md`, 2.5, *Day and night*, *Shadows keep their colour*.
- The request's overlay: `git show 3c5333485a:temper/docs/design/client/journeys/overlay.css`, read for what it sets on a shadow (the three rings alone).
- `temper/crates/temper-client-shell-web/temper.css`: the day `:root` set (lines 1 to 146), the night set (about 148 to 283).

## Owns and may change

Owns the night set's shadow lines in `temper.css`. Not the switch's, the tab bar's or the chat layout's rules (`tabs-and-switch`'s), and no day value. Reader files, an oracle only: a test of `temper-client-shell-web/tests/suite/*.rs` that read a night shadow keeps what else it checks, and is named.

Options, each decided:

- **A shadow token has no day value in `:root`** (defined only in a scoped day set, or read with a fallback): deleting its night line leaves it the day's value or fallback as by day, which is the request's; say which.

## Steps

1. Read the overlay and the two sets.
2. Acceptances 1 to 3; the status entry (acceptance 4); the run of acceptance 5; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`.
3. Commit with `Look-request: night-mode` and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| The project picker and person menu: `0 12px 32px rgba(24,24,27,.12)` | landed: the night takes `--gf-shadow`'s day value |
| The model menu: `.14` | landed: `--gf-menu-shadow`'s day value |
| Starter and scale cards: `0 1px 2px rgba(24,24,27,.04)` | landed: `--gf-shadow-faint`'s day value |
| The selected Split: `0 1px 2px rgba(0,0,0,.08)` | landed: `--gf-selection-shadow`'s day value |
| Latest: `0 2px 8px rgba(24,24,27,.1)` | landed: `--gf-latest-shadow`'s day value |
| The other seven shadow colours | the day's at night, as the request leaves them |
| The rings: To begin's card, a card's edge, the effort, the composer's focus | stand as landed, the overlay's |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4): focused tests establish no rendered equality |

## Report

The implementer's report: the lines deleted, the grep of acceptance 3, the options taken, the run's count, and the account as committed.
