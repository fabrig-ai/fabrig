# wasm32-temper-3: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then `temper/AGENTS.md`. The plan changes no Rust: one line of a list fab's
  gate reads, and status text.
- **The gate's wasm32 clippy:** fab's step `clippy-wasm32` reads `.config/gate/wasm32.txt` and
  runs `cargo clippy --target wasm32-unknown-unknown -p <each listed package the selection
  holds> -- -D warnings`. clippy lints every workspace crate that build compiles, so the step
  lints the listed packages and the workspace crates they depend on on that target.
- **The status file is this plan's to keep.** This departs from the implementer's text ("Not
  yours: any `docs/design/`"), because the request asks for it: a task edits exactly the status
  file its spec lists, in the way it lists, and no other. Design documents (`<name>.md`),
  drafts, `docs/help/` and every other status file stay untouched.
- **A status file's form** is `DESIGN.md`, 2: its date line is today's (`date -u +%F`), and it
  holds only the gaps as they stand after your change, never history. Where an entry names
  several gaps, change only the words for the one closed, and reflow no other line than the
  spec's edit names.
- **Citations** are checked by code (`DESIGN.md`, 4): after committing, run
  `python3 -I -B tools/citations/citations.py --base origin/main` from the repository root; it
  must print no finding your branch adds.
- **The wasm32 target** (`wasm32-unknown-unknown`): if `heavy` reports it missing (`can't find
  crate for \`core\``, "the target may not be installed"), stop: report `Failed: wasm32 target
  not installed`, with the host from heavy's output. Never `rustup target add`: a builder's
  toolchain is the user's.
- **A first error** is the first diagnostic in heavy's log that starts with `error` (not
  `error: could not compile`, which is cargo's summary): its lint as `clippy::<name>` or rustc's
  lint name, or its code (`E0599`) for a compile error; its message's first line; the crate;
  and its `-->` location, as a path from the repository root and a line. A file of a crates.io
  dependency is named `<crate> <version>` and its path inside that crate, never an absolute
  path.
- No `cargo fmt`, `old_forms.py`, `--all-targets` clippy or nextest run: no Rust and nothing
  they read changes. The only builds are the spec's clippy commands, through `heavy`, each once
  (once more only after exit 137). `cargo tree` builds nothing and runs directly. If heavy
  exits 75, stop and report `Infra: disk` (`AGENTS.md`, 5).
- One commit per coherent step, each saying why. Push your branch, also when it stays at its
  base.
