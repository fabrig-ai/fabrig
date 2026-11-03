# wasm32-list: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then `skein/AGENTS.md`. The plan changes no Rust: a list fab's gate reads,
  and status text.
- **Status files are this plan's to keep.** This departs from the implementer's text ("Not
  yours: any `docs/design/`"): a task edits exactly the status files and `STATUS.md` its spec
  lists, in the way it lists, and no other. Design documents (`<name>.md`), drafts and
  `docs/help/` stay untouched.
- **A status file's form** is `DESIGN.md`, 2: its date line is today's (`date -u +%F`), and it
  holds only the gaps as they stand after your change, never history. Remove an entry the change
  closes; where an entry names several gaps, remove only the words for those closed. A status
  with no gap left says `No gap.` under its date line (as `skein/docs/design/ui-web.status.md`
  does). The tree's `docs/design/STATUS.md` summary paragraph drops the clause about the gaps
  closed and names the document among those with no gap.
- **Citations** are checked by code (`DESIGN.md`, 4): after committing, run
  `python3 -I -B tools/citations/citations.py --base origin/main` from the repository root; it
  must print no finding your branch adds.
- **The wasm32 target** (`wasm32-unknown-unknown`) is installed on the builders hetzner6, 8, 9
  and 10. If `heavy` reports it missing (`can't find crate for \`core\``, "the target may not be
  installed"), stop: report `Failed: wasm32 target not installed`, with the host from heavy's
  output. Never `rustup target add`: a builder's toolchain is the user's.
- No `cargo fmt`, `old_forms.py` or nextest run: nothing they read changes. The only build is
  the spec's clippy command, through `heavy`, once.
- One commit per coherent step, each saying why. Push your branch.
