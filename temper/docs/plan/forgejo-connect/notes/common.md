# forgejo-connect: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then `temper/AGENTS.md`. Your task stays in the files its spec names; a reader
  or neighbour file outside them is edited only as the spec says.
- **The design is written** (`inputs/request.md`, answered 2026-11-03): the five open points of a
  connected Forgejo are settled in temper's designs, and `notes/contracts.md` holds the shapes
  that cross tasks. Your task brings the code to the sections its spec cites. Design documents'
  text is the lead's: you never edit it; what the code taught goes in your report.
- **Status files close with the code.** This departs from the implementer's text ("Not yours: any
  `docs/design/`"): a task edits exactly the status entries its spec lists, in the way it lists,
  and no other. A status file's form is `DESIGN.md`, 2: its date line is today's (`date -u +%F`),
  it holds only the gaps as they stand after your change, never history. Remove the words your
  change closes; where an entry names several gaps, leave the rest word for word. A status left
  with no gap says `No gap.` under its date line.
- **Drafts and `docs/help/`** stay untouched unless the spec names a file there and gives its
  words (a reference page is regenerated from its table, never hand-written).
- **No budget, timeout or limit moves** (skein's `docs/foundation/testing-strategy.md`, 8.1).
- **Stored and wire formats change in place,** at their current versions (`temper-client-wire`
  11, `temper-secrets` 1, the store's 11), and only where the spec sketches it. Generated code and
  goldens are regenerated with `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p
  temper-codecs-world --test suite -E 'test(drift::)'`, never edited by hand; read the
  regenerated diff with `git diff --stat` and keep only what your schema change moves.
- **No secret anywhere but the secrets' store:** a token, key or secret never reaches a domain, a
  root, a record of the main store, an event's `Debug`, a log line, a test's assertion message
  or a golden. Tests draw tokens as random bytes, never a real credential (protocol/forge.md, 10).
- **Old-form counts only fall:** `python3 -I -B temper/scripts/old_forms.py`; `--write` only to
  lower a count for what you removed. A rising count is yours to fix before you push.
- **Tests:** after each change the narrowest test (`AGENTS.md`, 6); before the push, the focused
  tests of every package you changed and of its direct dependents (`heavy cargo nextest run -p
  ...`), and `heavy cargo check -p <pkg> --tests` for each reader of a public item you changed.
  Check each run's count: a run that selected nothing proves nothing.
- **Before the push,** when your branch changed any Markdown: `python3 -I -B
  tools/citations/citations.py --base origin/main` from the repository root, and fix what it lists
  in your files.
- One commit per coherent step (the code and its tests; then the status text), each saying why.
  Push your branch.
