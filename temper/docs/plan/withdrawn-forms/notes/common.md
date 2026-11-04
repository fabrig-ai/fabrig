# withdrawn-forms: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of your task's tree. Your task stays in its tree, but
  for the files its spec names in another one, each edited only as the spec says.
- **The decision** is `temper/docs/plan/withdrawn-forms/inputs/request.md`, option (a): the
  withdrawn compatibility forms leave every tree, as skein's `skein/docs/design/llm.md` (4.6,
  4.7) and smith's designs say, in five tasks, each landing before the next one that reads it:
  temper's configs and literals, jig's fixtures and dialect rule, smith's product, smith's agent,
  protocol and channel at v6, skein's routes.
- **The status files are the spec.** Each task closes the gaps its spec names, and updates beside
  the code the design text and status entries the change makes true. This departs from the
  implementer's text ("Not yours: any `docs/design/`"): a task edits exactly the design
  documents, status files and `STATUS.md` its spec lists, in the way it lists, and no other.
  Drafts and `docs/help/` stay untouched unless the spec names them.
- **A status file's form** is `DESIGN.md`, 2: its date line is today's (`date -u +%F`), it holds
  only the gaps as they stand after your change, never history. Remove an entry the change
  closes; where an entry names several gaps, remove only the words for those closed. A status
  with no gap left says `No gap.` under its date line. The tree's `docs/design/STATUS.md` summary
  paragraph drops the sentence or clause about the gaps closed.
- **A removed section leaves its number unused:** never renumber the sections after it. Whoever
  removes a section or a document fixes every citation to it outside `docs/plan/` and dated
  reports in `docs/development/`; before pushing, run
  `python3 -I -B tools/citations/citations.py --base origin/main` from the repository root and
  fix what it lists in the files you own.
- **Design text says what is true now,** in the design's words: no "no longer", "was",
  "removed", "formerly", "withdrawn" for something gone. A sentence about something removed is
  deleted, not reworded into its history.
- **Stored data starts afresh: no migration.** A stored charter or transcript at dialect 1 and a
  replay at skein-llm's Codex tag are refused by the decoder that reads them, as any value it
  does not know; no compatibility reader, no converter, no new error kind unless the spec
  sketches one.
- **No probe of a real service.** A recording of a real provider is never captured again; a
  test whose only input was a removed route's recording goes with it, and a synthetic case that
  replaces coverage is labelled synthetic, as skein's `llm.md`, 5 asks.
- **Two names that stay:** smith's Claude Code *program* (`claude_code`,
  `smith/docs/design/protocol/programs.md`) is not the withdrawn Claude Code *identity profile*
  (an endpoint's `identity = "claude-code"`, skein's `anthropic::identity`); Codex the coding
  agent (smith's benchmarks' Codex adapter, `~/.codex`, the implementers) is not the withdrawn
  Codex *route* (`Endpoint::codex()`, `Provider::OpenAiCodex`, dialect 1). Only the second of
  each pair goes.
- **Dialect numbers are fixed for good** (`smith/docs/design/protocol/agent.md`, 4): `openai` 3,
  OpenAI's public Responses dialect; `anthropic` 2, Anthropic's Messages dialect. 1 was the
  Codex route's and is never given to another dialect; 0 and 4 name none.
- **Old-form counts only fall.** If your removal lowers a count of your tree's
  `scripts/old-forms.txt`, run `python3 -I -B scripts/old_forms.py --write` from the tree's root
  and commit the lowered file; never raise a count. If a count would rise, stop and ask.
- **Goldens and generated code** are regenerated with `SKEIN_CODEGEN_WRITE=1` on the test that
  owns them, never edited by hand; read the regenerated diff with `git diff --stat` and keep
  only what your change moves. A write mode that leaves stale files is followed by `git rm` of
  exactly those.
- **Removing, not replacing:** no new public items, crates or dependencies unless the spec
  sketches them. A test that exercised only the removed form is deleted with it; a test that
  exercised something else beside it keeps its other assertions, moved to a form that stays.
- **Cross-tree readers:** where your branch removes a public item that another tree names, it
  carries that tree's lines, each edited only to drop or rename that item (an arm, a literal's
  field, an import, a path); your spec lists those it found. After the removal, run
  `heavy cargo check -p <pkg> --tests` for each package in another tree that named it.
- **smith's design Markdown is read by tests** (`.config/gate/read-by-tests.txt`:
  `smith/docs/design/**`): `smith-local-domain`'s `parent_tests.rs` includes `shell.md` (its
  `> Chat ` quotes), and smith's benchmarks check each task's design guards (`<file>, section
  <n>`, `smith/benchmarks/src/check.rs`, `check_guard`) against `smith/docs/design`. A branch
  that edits smith design text runs those (its spec names them).
- One commit per coherent step (code and its tests; then the design and status text), each
  saying why. Push your branch.
