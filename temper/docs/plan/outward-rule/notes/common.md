# outward-rule: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`. The work is at the repository's root, in `tests/boundary/` (package
  `fabrig-boundary`): no tree's `AGENTS.md` governs it; it is ordinary Rust held to
  `tests/clippy.toml` and to the lints in `tests/boundary/Cargo.toml`, like
  `tests/boundary/tests/direction.rs`, the model for its shape.
- **Nothing is reworded** (`temper/docs/plan/outward-rule/inputs/request.md`, Bounds): no file
  that holds one of the lab's names changes in this plan, however easy the fix looks. The
  inventory records them; later steps remove them.
- **No new dependency:** `std` and `skein-scratch` (already a dev-dependency) only. `Cargo.lock`
  does not change.
- No design document, draft, status file, `docs/help/` page or plan file changes.
- **No old-forms run:** the branch touches no tree (`tests/` and the root `Cargo.toml` are no
  tree's), so no tree's `scripts/old_forms.py` counts anything it changes. This departs from the
  implementer's text, which runs it for each tree the branch touches: there is none.
- **Citations:** after committing, `python3 -I -B tools/citations/citations.py --all` from the
  repository root prints `citations: 0 broken in HEAD` (it was 0 at this plan's cut). A broken
  citation your branch did not add goes under `## Left`.
- Every build and test through `heavy`, one command at a time (`AGENTS.md`, 5 and 6). The
  measured time the spec asks for is nextest's own figure for the test, as heavy's log shows it.
- One commit per coherent step, each saying why. Push your branch.
