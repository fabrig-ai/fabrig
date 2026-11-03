# status-lines: standing rules

Every task of this plan names this note. The plan is docs only: it writes status-file lines for
the design gaps the debt backfill found (`temper/docs/plan/status-lines/inputs/request.md`), and
corrects one stale design sentence. No code, schema, golden, test or debt file changes.

- Read `AGENTS.md`, then the `AGENTS.md` of your task's tree, then `DESIGN.md` 2 (a status
  file's form). Your task stays in its tree, but for the files its spec names in another one.
- **This departs from the implementer's text** ("Not yours: any `docs/design/`"), because the
  task is to write design status: a task edits exactly the status files, the design sentence and
  the `STATUS.md` its spec lists, in the way it lists, and no other file. Drafts, `docs/help/`,
  `docs/development/` (the debt items among them) and every design document's own text stay
  untouched unless the spec names them.
- **Check the code first.** The request's facts were taken on 2026-11-03 near main `a6a8155d35`.
  Before writing a line, read the code it names (`rg` for the item) and the design section it is
  against. Write the line only if the gap is still there, naming the code as it stands now. A gap
  the code or the design has closed gets no line: list it in your report under `## Dropped` with
  what you found (the file and item, or the commit).
- **One entry per gap,** in the status file's own form (`DESIGN.md`, 2):
  - **Departs** where the code is built otherwise than the design says, or does what the design
    has no text on; **Not yet** where the design says it and the code has not built it.
  - What the design says (its section, or that it has no text on it), what the code does (the
    crate-relative file and the item, as the file's other entries name them:
    `jig-domain-core/src/admission/proposals.rs`, `holder`), and the reason: a reason the code
    or a commit writes, or "no reason is recorded".
  - Then the code's side, in the form `jig/docs/design/examples.status.md` already uses:
    `(the code's side is \`<tree>/docs/development/debt/DNNN-<slug>.md\`)`, with the item the
    spec names; an entry whose spec names no debt item has none.
  - Where the spec says the designer decides, the entry says so in its words ("whether this
    departs is the designer's decision"), and is still written.
  - Name items, not line numbers: line numbers drift.
- **Where an entry goes:** at the end of the file's **Departs:** (or **Not yet:**) bullet, as a
  new sentence; where the file's labels hold sub-bullets (smith's `testing.status.md` and
  `benchmarks.status.md`), as a new sub-bullet in section order. A file with no bullet of that
  label gets a new `- **Departs:**` (or `- **Not yet:**`) bullet after its last bullet, unless the
  spec places it elsewhere. A file that says `No gap.` has that line replaced by the bullet.
  Wrap as the file wraps (about 72 columns, or one long line per bullet); leave every other line
  of the file as it is, word for word.
- **Labels are checked against the file's own document** (`tools/citations/README.md`): a
  bullet's first `N.N:` and every sentence that begins `N.N:` after a full stop must be a
  numbered heading of the status file's own document. Label each entry with that document's
  section the gap is in (read its headings: `grep -n '^#' <doc>.md`). Cite another document by
  its path from the repository root and its section, never as a label:
  `` `skein/docs/foundation/programming-model.md`, 5.4 ``. Where the document has no section on
  the subject, label the entry with the section that describes the crate or component the code
  belongs to, and say in the report which you chose.
- **The date line** of every status file you change is today's (`date -u +%F`); a file already
  dated today keeps its line as it is.
- **`STATUS.md`:** a tree's `docs/design/STATUS.md` changes only where its spec says (where a
  "have no gap" list names a document this plan gives a gap). Its date line follows the rule
  above.
- **Citations:** commit, then run `python3 -I -B tools/citations/citations.py --base origin/main`
  from the repository root; it must report nothing new. Fix what it lists in your own lines.
- **No build:** status files are read by no test, but smith's design tree is read by
  `smith-bench`'s guard checks (`.config/gate/read-by-tests.txt`; `smith/benchmarks/src/check.rs`,
  `check_guard`, which reads design documents' headings): the smith task runs that package's
  tests, as its spec says. Before pushing, check that no test reads a file you changed:
  `rg -n '<file name>' --glob '*.rs' <tree>`; a hit in a test you have not been told about is a
  test to run, reported.
- One commit per task, saying why (the backfill found the gaps, and no status file held them);
  push your branch. Your report lists each entry written (file, label, first words), each
  dropped gap and why, and the citation check's output.
