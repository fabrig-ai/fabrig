# playground-temper: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of each tree your task touches. Your task stays in the
  trees and files its spec names; a reader file outside them is edited only as the spec says.
- **The design is written:** `temper/docs/design/playground.md` and the sections your spec cites.
  Your task brings the code to them. Design documents' text is the designer's: you never edit it,
  but for a line your spec names and says how.
- **Status files close with the code.** This departs from the implementer's text ("Not yours: any
  `docs/design/`"): a task edits exactly the status entries its spec lists, in the way it lists,
  and no other. A status file's form is `DESIGN.md`, 2: its date line is today's (`date -u +%F`),
  it holds only the gaps as they stand after your change, never history. Where an entry names
  several gaps, remove only the words for those your change closes and leave the rest word for
  word. Remove them in place and never reflow the paragraph: several tasks of this plan edit one
  entry of `temper/docs/design/playground.status.md`, and unreflowed edits merge.
- **Drafts and `docs/help/`** stay untouched.
- **No budget, timeout or limit of a test tier moves** (skein's
  `docs/foundation/testing-strategy.md`, 8.1). A world's story aims at 2 s
  (`temper/docs/design/domain/testing.md`, 6.1); over budget is red, and the repair is the
  testing strategy's three questions, in order.
- **Behaviour is kept:** a task that moves code changes no meaning. The one-process world's
  stories (`temper/tests/one-process/tests/suite/`) pass after every task, with the same counts
  and verdicts; a test you cannot keep as it was is a question.
- **Product crates name no test kit.** A crate under `*/crates/*` depends on no crate under
  `*/tests/*` or `*/testing/*`, in `[dependencies]`; its dev-dependencies may. Its lints are its
  tree's `clippy.toml`, not `tests/clippy.toml`.
- **Old-form counts only fall:** `python3 -I -B <tree>/scripts/old_forms.py` for each tree you
  touch; `--write` only to lower a count for what you removed. A rising count is yours to fix.
- **Tests:** after each change the narrowest test (`AGENTS.md`, 6); before the push, the focused
  tests of every package you changed and of its direct dependents (`heavy cargo nextest run -p
  ...`), and `heavy cargo check -p <pkg> --tests` for each cross-tree reader of a public item you
  changed. Check each run's count: a run that selected nothing proves nothing.
- **Before the push,** when your branch changed any Markdown or moved a path a document cites:
  `python3 -I -B tools/citations/citations.py --base origin/main` from the repository root, and fix
  what it lists in your files.
- One commit per coherent step, each saying why. Push your branch.
