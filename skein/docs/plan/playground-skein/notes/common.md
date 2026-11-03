# playground-skein: standing rules

Every task of this plan names this note, and `notes/contracts.md` beside it, which holds the
shapes that cross tasks: build to it exactly.

- **The plan:** skein's playground kit, `skein/docs/design/playground.md`, sections 2 to 8:
  the bridge's messages, the session log and the worker's platform in `skein-ui-web`
  (`skein/docs/design/ui-web.md`, 18); the worker module in `skein-ui-bundle`; and the new
  crate `skein-playground` (the visitor, the slices, the bridge in memory, the replay, the
  worker's parts on wasm32) with its world `skein/tests/playground`. skein names nothing of
  smith, jig or temper (`AGENTS.md`, 1): no story, document or name of temper's goes in
  skein's code or tests. A second plan, `playground-temper`, builds temper's composition on
  this kit; nothing here waits for it.
- **The tree:** skein, but for the lines a spec names in another tree, each edited only as it
  says. Read `AGENTS.md`, then `skein/AGENTS.md`.
- **The design is binding:** the sections each spec cites, as they stand on your base. Never
  edit `docs/design/`, drafts or `skein/docs/plan/`; what the code taught goes in the report.
  The status files are the plan's last task's.
- **The code's rules:** `skein/docs/foundation/programming-model.md`, section 1, and section
  10 for step code; `skein-ui-web`, `skein-ui-bundle` and `skein-playground` are ordinary
  Rust (`programming-model.md`, 10.2), under their crate's `clippy.toml` and the workspace's
  lints (no float arithmetic, no `as`, no indexing, no unchecked arithmetic, no `unwrap`, no
  `todo!`, every `match` naming every variant). `skein-ui-web` forbids `unsafe`; so does
  `skein-playground`.
- **Two halves:** what a module does on wasm32 alone (`web_sys`, `js_sys`, `wasm_bindgen`)
  sits in a `web` submodule under `#[cfg(target_arch = "wasm32")]`, as `skein-ui-web`'s
  `fetch/web.rs` does; its pure half builds natively and is tested there. A wasm32 half is
  checked by `heavy cargo clippy -p PKG --target wasm32-unknown-unknown -- -D warnings`.
- **Tests:** a crate's step tests in its `src/tests/` (as `skein-ui-web`'s are); what needs a
  world in `skein/tests/playground` (`skein-playground-world`), under `skein/tests/clippy.toml`:
  one thread, one loop, a seed replays. Each world names its seed in every failure.
- **One writer per file:** two tasks of this plan may work in one crate at once only on the
  files their headers list; the module declarations they need are laid down by the crate's
  first task. A change to a file or a public item another task owns is a question.
- **No public item** beyond your spec's and the contracts'; private helpers are yours. A
  compiler-forced arm or field in your tree is always allowed, and listed in the report.
- **Old-form counts only fall:** no "actual", "genuine", "originally" in comments; run
  `python3 -I -B skein/scripts/old_forms.py`.
- **Every build and test through `heavy`,** one at a time, the narrowest first; check each
  run's count (nextest exits 4 when it selects nothing). Before pushing: `cargo fmt`, then
  `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` naming every package you
  changed, the wasm32 clippy of each crate your spec names, their focused tests and their
  direct dependents' in every tree (`AGENTS.md`, 6), and
  `python3 -I -B tools/citations/citations.py --base origin/main`.
- **Other plans beside this one** touch none of this plan's files but the lines a spec names
  in jig; keep your hunks small and local, and never reformat code your task does not change.
