+++
id = "folds-away"
part = "folds-away"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-ui/src/tree.rs",
  "skein/crates/skein-ui/src/builder.rs",
  "skein/crates/skein-ui-web/src/mapping.rs",
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/page/**",
  "skein/crates/skein-ui-web/src/tests/**",
  "skein/tests/browser/**",
  "skein/docs/design/ui.status.md",
  "skein/docs/design/ui-web.status.md",
]
effort = "high"
minutes = 60
+++

# A disclosure that folds away

## Goal

A panel behind a disclosure closes as a person expects: on a press outside it, or on Escape. No disclosure
does today. temper's byline needs it (`chat-byline`), and a generic mechanism goes into skein first.

Acceptance:

- `skein_ui::State::FoldsAway`: a disclosure's state, set by its view. The web shell maps it to
  `data-folds-away` on the `<details>`.
- In the browser, an open disclosure with it folds on a press outside it, and the press still reaches what
  it lands on. A press inside it leaves it open.
- Escape folds it, after an open menu takes Escape and before a form's Escape, and the focus returns to its
  summary if the focus was inside it. An Escape that folded it presses nothing else.
- A disclosure without the state behaves exactly as today, and a call's fold stays open on a press outside.
- The terminal prints a disclosure with the state as any disclosure.
- The status entries for it in `skein/docs/design/ui.status.md` and `ui-web.status.md` go; with no gap left
  in a file, it says so as `DESIGN.md`, 2 has it.

## Inputs

- `temper/docs/plan/ui-chats/notes/common.md`; its *Your checks* for temper's packages do not apply here.
- `skein/docs/design/ui.md`, 3.2, its states' tables; `skein/docs/design/ui-web.md`, 7, *Menus and
  disclosures are the page part's to fold*.
- The reverse-dependency survey: `State` is matched in `skein-ui-web/src/mapping.rs` (`State::Expanded` at
  about line 83 and 345) and in `skein-ui-text`, and each `match` lists every variant.

## Owns and may change

Owns, as `files` lists them.

May also change: `skein/crates/skein-ui-text/src/**`, and any file of skein that matches on `State`: the arm
the compiler requires, each edited only for that. If `State` is matched outside skein, the arm alone, and say
so in the report.

Options, each pre-approved:

- The state on a node that is no disclosure: follow `builder.rs`' treatment of a state a role can't take (an
  assertion, or ignored), and say which.
- Where the platform half's press and Escape go: `page/web.rs`'s listeners take them, with whatever pure part
  fits `page/fold.rs`. If a click's listener returns early on `[data-component]` before your check, a press in a
  component still counts as outside: move the check before it.
- If a native shell's table in `ui.md` needs a row to stay whole, add it to the design, and say so.

## Steps

1. The state, its mapping, its arms.
2. The press and Escape, in the page part, with no step and no report to the application.
3. **Tests:** `skein/crates/skein-ui-web/src/tests/mapping.rs` for the attribute;
   `skein/crates/skein-ui-web/src/tests/page/events.rs` or `tests/fold.rs` for what is pure; a story in
   `skein/tests/browser/tests/suite/` in real Chromium: open, press outside and see it folded and the outside
   button pressed; open, press inside and see it open; open, Escape and see it folded with the focus on its
   summary; a disclosure without the state stays open through both. Each bites.
4. **Check:** `heavy cargo nextest run -p skein-ui -p skein-ui-web -p skein-ui-text`, the browser story with
   `heavy cargo nextest run --profile browser-skein -p skein-browser-world -E 'test(=NAME)'`, clippy for the
   packages you changed (the wasm-only half as `skein/crates/skein-ui-web`'s `AGENTS.md` or manifest says),
   `python3 -I -B tools/citations/citations.py --base origin/main`, push.

## Report

As `notes/common.md`, *Your report*.
