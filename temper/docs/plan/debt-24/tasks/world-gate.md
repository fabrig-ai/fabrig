+++
id = "world-gate"
part = "skein-kits"
kind = "feature"
needs = ["json-kit", "http-kit", "llm-kit", "tls-kit", "forge-kit", "echo-edge", "path-fixtures", "skein-world-readers"]
files = ["tests/boundary/tests/skein_worlds.rs", "tests/boundary/Cargo.toml", "skein/docs/development/debt/D028-worlds-as-kits.md", "skein/docs/design/testing.status.md", "Cargo.lock"]
effort = "medium"
minutes = 25
+++

# The gate holds skein's worlds apart, and D028 goes

## Goal

Skein D028's **Paid when** and the check it asks for: no package depends on a package under `skein/tests/`, in any tree, and no `#[path]` under `skein/` reaches outside its package.

Acceptance:

- `tests/boundary/tests/skein_worlds.rs`, in `tests/boundary/tests/fail_stop.rs`'s metadata pattern and `jig/tests/boundary/src/tests/kits.rs`' shape: pure functions over packages and files, each with fixture rows, then one test on the repository.

  ```rust
  fn world_edges(packages: &[Package]) -> Vec<String>;      // any dependency kind, any tree, whose path is under skein/tests/
  fn path_crossings(root: &Path) -> Vec<String>;            // each #[path = "..."] under skein/, resolved against its file, outside its nearest Cargo.toml's directory
  ```

- **Both must bite:** add `skein-json-world` to one manifest's dev-dependencies, and add one `#[path]` across a package, by hand; watch each fail for that reason; restore.
- `rg -c '^skein-(echo|forge-client|forge-machine|http|json|llm|tls)-world\b' skein -g Cargo.toml | awk -F: '{s+=$2} END{print s}'` reads nothing or 0.
- `skein/docs/development/debt/D028-worlds-as-kits.md` is deleted. `skein/docs/design/testing.status.md`'s **Not yet** on 6 goes. `python3 -I -B tools/citations/citations.py --all` reads 0, and `python3 -I -B tools/debt/debt.py | rg 'D028'` shows no skein row.
- `heavy cargo nextest run -p fabrig-boundary` passes.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `skein/docs/development/debt/D028-worlds-as-kits.md`
- `skein/docs/design/testing.md`, 6

## Owns and may change

- **Owns:** the files in the header.
- **`#[path]` inside a package and manifests' `path =`** aren't breaches, as `path-fixtures` says.
- **`skein/tests/*` depending on one another's kits is fine.** A `dev-dependency` of a world on itself isn't an edge.
- **If `world_edges` finds an edge a task missed,** fix it by the note's step 4, if it's a line or two; otherwise stop and ask, with the edges.

## Steps

The test; the breaks; the deletion and the status; the checks. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p fabrig-boundary --all-targets -- -D warnings`. Commit and push.

## Report

The common note's items; each break's failure; the Measure.
