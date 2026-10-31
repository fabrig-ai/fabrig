# What every task may change without asking

The lead's, for every spec that cites it. It widens each spec's "Owns
and may change" by the edits a task's own change forces; it does not
widen its goal.

- **Forced arms and literals, anywhere:** a variant, field, key or type
  your task adds or renames forces edits in exhaustive matches, struct
  literals, derived or hand-written impls (codec, sizing, ordering,
  coalescing, diagnostics, `RecordKind`/`Need`-style mirrors), fixtures
  and test helpers, in any crate and tree. Make each, doing for the new
  item what the code does for its nearest kin, and list them in the
  report. That includes a file the spec says "nothing else of": its
  forced arms are still yours.
- **Companion items the shape needs:** a key, a coalescing identity, a
  mirror enum or a limit that a new write, record or request cannot
  exist without (for a new root write, its `Key` variant; for a new
  family, its record kind) is yours, named after the item it serves.
- **Exhaustion:** a counter that would pass its type's range refuses the
  act that would need it (*limit*), and never wraps; a sentinel value is
  yours to choose, said in the report.
- **What stays a question:** a change of another part's public
  behavior or of a contract in `notes/contracts.md`, a stored format or
  wire shape beyond what your spec names, a new dependency edge between
  crates or trees, a budget, a test moved or deleted for being slow.
- When the spec and this note are silent, take the option that keeps the
  change smallest and inside your part, and say so in the report.
