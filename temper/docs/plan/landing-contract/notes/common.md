# landing-contract: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`. The work is at the repository's root (`.config/`, `tests/boundary/`): no
  tree's `AGENTS.md` governs it.
- **Wording and documentation only** (`temper/docs/plan/landing-contract/inputs/request.md`,
  Bounds): no check, list entry, profile, filter, budget or script changes what it does. A
  comment line stays a comment line; `.config/nextest.toml` parses to the same TOML before and
  after; `.config/gate/wasm32.txt` keeps its package lines as they are; tests pass unchanged.
- **No runner is named.** fabrig's text says "a landing check", "the gate", "the after-landing
  tier" and "a caller", never who runs them: no "fab", "the queue", "the merge queue", "the
  pipeline", "ops", "the after-landing runner", "the director", "a shelf", "lends", and no
  machine, host, session or outside repository. `tests/boundary/tests/outward.rs` fails a file
  that adds one of the lab's names it counts; the other words are yours to keep out.
- **The design is already written**, by this plan's lead, on your base: `tools/tiers/design.md`
  (7.1, 7.5, 8 and the new 10), `smith/docs/design/testing.md` (1, 2.3, 9),
  `temper/docs/design/domain/testing.md` (2.7) and the status files
  `skein/docs/design/testing.status.md`, `smith/docs/design/shell.status.md` and
  `temper/docs/design/domain/testing.status.md`. You cite them; you never edit them, nor any
  other `docs/design/`, draft or status file.
- **Not this plan's:** the root `AGENTS.md` and `DESIGN.md`, `tools/old_forms/README.md` and the
  trees' `scripts/old-forms.txt` (a parallel step owns them); `tools/citations/` (the
  outward-citations draft); `temper/docs/design/client/browser.status.md` and
  `temper/docs/design/playground.status.md` (the playground-browser draft rewrites the lines
  that name a runner there).
- **No old-forms run:** the branch touches no tree (`.config/` and `tests/` are no tree's), so
  no tree's `scripts/old_forms.py` counts anything it changes. This departs from the
  implementer's text, which runs it for each tree the branch touches: there is none.
- **No clippy or fmt run:** the branch changes no Rust. This departs from the implementer's
  text too.
- **Citations:** after committing, `python3 -I -B tools/citations/citations.py --all` from the
  repository root prints `citations: 0 broken in HEAD`. A broken citation your branch did not
  add goes under `## Left`.
- Every build and test through `heavy`, one command at a time (`AGENTS.md`, 5 and 6); the
  Python checks above run directly, as the tools' own READMEs say.
- One commit per coherent step, each saying why. Push your branch.
