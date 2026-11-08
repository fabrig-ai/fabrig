+++
id = "welcome-end"
part = "welcome"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/outside.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/journeys/welcome-set-up/**",
  "temper/docs/design/client/journeys/welcome-playground/**",
  "temper/docs/design/client/journeys/project-landing/**",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 60
+++

# The welcome's top bar ends as every outside page's does

## Goal

The request's task 1 (`temper/docs/plan/look-round-22/inputs/request.md`). Signed in, the welcome's top bar shows no person's name and no "Sign out": `outside::top` (`temper/crates/temper-client-view/src/outside.rs`, about lines 72 to 83) returns after temper box's line and the steps, before the top bar's end. It draws the end there as on every other page outside a project, and the welcome's top bar keeps the steps and the end at its right.

Acceptance, by `temper/docs/design/client/README.md`, 3.5, The welcome, *Its frame* and *Its steps*, and 5.1, *the top bar*; `temper/docs/design/client/browser.md`, 2.5, *The welcome's look*:

1. **The view:** on `Address::Welcome`, `top` draws the brand, then (the deployment read and `welcome` true, the link available, as now) temper box's line and the steps, then the same `TOP_END` slot every other outside page draws: the link region with its status, "Behind" where it holds, the person's name once `me` is read, the sign-out-lost alert while it holds, "Sign out" once `me` is read or while that alert shows. The early `b.close(); return;` goes; nothing else of `top` changes, in either shell. The welcome's main is unchanged.
2. **The stylesheet,** the welcome's top bar rules where they stand in `temper/crates/temper-client-shell-web/temper.css`:
   - about line 1108, `#app:has(> .temper-welcome) > .temper-top`: `justify-content: space-between` becomes `justify-content: flex-end`;
   - about line 1112, `#app:has(> .temper-welcome) .temper-flow-steps`: `margin: 0` becomes `margin: 0 0 0 auto`;
   - about line 1121, `#app:has(> .temper-welcome) .temper-welcome-box-line`: `margin: 0 auto 0 24px` becomes `margin: 0 0 0 24px` (its `@media (max-width: 1100px)` rule, `margin: 0`, stays);
   - one new rule beside them: `#app:has(> .temper-welcome) > .temper-top > .temper-brand + .temper-top-end { margin-left: auto; }`, with a comment saying the end takes the free space when no steps show yet.

   So the steps and the end keep to the right, temper box's line beside the brand at its 24 px; an end its row cannot hold wraps to the right of the next row (the rows pack to their end); before the deployment answers, the brand stays left and the end right. No other rule changes.
3. **The row:** the existing test `the_welcome_frame_is_the_brand_the_box_line_and_the_steps` (`temper/crates/temper-client-view/src/tests/welcome.rs`, about line 476), renamed `the_welcome_frame_is_the_brand_the_box_line_the_steps_and_the_end`, asserts in place of its "no Sign out", "no frame statuses" and "no `TOP_END`" lines: the banner's children in order are the brand, the box line's slot, the Steps list and the `TOP_END` slot, which finishes the banner and holds the link region, the person's name and "Sign out" (as `tests/frame.rs`'s `assert_top_end` checks them: make that helper `pub(super)` and call it, or move it to `tests/fixtures.rs`; say which); its region count becomes 3 (the link region). The rest of what it checks stays. No new test function.
4. **Prints:** the journeys whose pages show the welcome signed in are reprinted (`notes/common.md`, *Prints*): `welcome-set-up` and `welcome-playground`, and `project-landing` if its prints change; `heavy cargo nextest run -p temper-client-journeys` passes.
5. **Status:** delete the **Not yet** entries naming `welcome-end` in `temper/docs/design/client/README.status.md` and `temper/docs/design/client/browser.status.md`. No other design line.
6. Runs through `heavy`, counts in the report: `cargo nextest run` for `-p temper-client-view`, `-p temper-client-shell-web`, `-p temper-client-shell-terminal`, `-p temper-client-gallery`, `-p temper-client-world` and `-p temper-client-journeys`; `cargo check --tests` for `-p temper-client-browser-world` and `-p temper-one-process-world`; `cargo clippy -p temper-client-view --all-targets -- -D warnings`.

One commit or more, each with the trailer `Look-request: outside-pages`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-22/notes/common.md`.
- `temper/docs/plan/look-round-22/inputs/request.md`, task 1.
- `temper/docs/design/client/README.md`, 3.5, The welcome, *Its frame* and *Its steps*; 5.1, *the top bar* (its right, and "On `Welcome`").
- `temper/docs/design/client/browser.md`, 2.5, *The welcome's look* (the top bar's row) and *Outside a project* (the top bar's end).
- The request's top bar signed in, from its own commit: `git show 43a18a5042:temper/docs/design/client/journeys/outside-pages/03.html`.
- The code: `outside.rs`, `top`; `temper.css`, the welcome's top bar rules (about lines 1107 to 1127) and `ui: outside`'s `.temper-top-end` rules (about lines 2877 to 2881).

## Owns and may change

Owns `top`'s welcome branch, the four rules of acceptance 2, the welcome test's assertions and name, the reprinted journeys and the status entries. Not `root.rs` (`me-first` changes it next), `public.rs`, `ui: outside`'s or `ui: night`'s rules (`outside-ground`'s), the welcome's main, or the TUI's crates.

May change, keeping what each checks for everything else:

- `temper/crates/temper-client-view/src/tests/frame.rs` and `tests/fixtures.rs`: `assert_top_end`'s visibility or place (acceptance 3) alone.
- A test elsewhere that reads the welcome's top bar as ending at its steps (no `TOP_END`, no "Sign out", no link region, a count of regions or statuses), found by `rg -n "Welcome|welcome" ` with `TOP_END`, `Sign out` or `frame_statuses` in `temper/crates/temper-client-view/src/tests`, `temper/crates/temper-client-shell-terminal/src/tests`, `temper/tools/gallery/src/tests.rs` and `temper/tests/client`: change its expectation to the end, and name it.
- A slow-tier story or helper (`temper/tests/client-browser`, `temper/tests/one-process`) that reads the welcome's top bar as ending at its steps: change it alike, compiled not run, and name it.

Options, each decided:

- **The welcome fixture's root has no `me`** (`f::welcome_root`): the row asserts the end with its link region alone and "Sign out" absent, and a second root of the same test, with `me` delivered (`fixtures::deliver_me`), asserts the name and "Sign out"; say which.
- **A recorded journey reports stale** after the reprint: record it again with `again NAME` (`notes/common.md`, *Prints*), and say so.
- **The terminal's print of a welcome page changes** (its first line now carries the link's state and the person): regenerate it by its own test's tool, and name it.

## Steps

1. Read the request's task 1 and the Inputs; change `top` (acceptance 1).
2. The stylesheet's rules (acceptance 2).
3. The test's row (acceptance 3); bite it: restore the early return, see the renamed test fail on the end, and restore.
4. Reprint the journeys (acceptance 4); the status entries (acceptance 5); the runs of acceptance 6; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`.
5. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| The welcome's top bar, signed in: the person's name and "Sign out" at its right | landed (`outside.rs`, `top`: the end after the steps) |
| The welcome's top bar keeps its steps and end at its right, temper box's line beside the brand, at 1440, 1000 and 420 | landed (`temper.css`, the welcome's top bar rules) |
| The journeys showing the welcome, reprinted | landed (name each) |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4): compiling, prints and focused tests establish no rendered equality |

## Report

The implementer's report: `top`'s change, the rules as written, the tests changed and how, the bite and its failure, the journeys reprinted or recorded again, the runs' counts, and the account as committed.
