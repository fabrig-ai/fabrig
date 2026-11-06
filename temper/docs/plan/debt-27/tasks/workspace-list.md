+++
id = "workspace-list"
part = "workspace-list"
kind = "feature"
needs = []
files = ["Cargo.toml", "skein/docs/development/debt/D041-workspace-duplicates.md"]
effort = "medium"
minutes = 10
+++

# Each skein crate once in the workspace's members; no unused skein dependency

## Goal

Pay skein's D041. The root `Cargo.toml`'s `[workspace] members` lists ten skein crates twice now, and its `[workspace.dependencies]` names five skein worlds no manifest takes. Delete the second entries and those five lines.

Acceptance:

- The item's measure reads 0: `python3 -I -c "import tomllib,collections;m=tomllib.load(open('Cargo.toml','rb'))['workspace']['members'];print(sum(v-1 for v in collections.Counter(m).values()))"`.
- `[workspace.dependencies]` no longer names `skein-forge-machine-world`, `skein-forge-client-world`, `skein-browser-world`, `skein-tls-world` or `skein-llm-world`. No other skein package it names goes unused.
- `Cargo.lock` doesn't change. `git diff --stat` shows `Cargo.toml` and the item file alone.
- `skein/docs/development/debt/D041-workspace-duplicates.md` is deleted.

## Inputs

- `skein/docs/development/debt/D041-workspace-duplicates.md`, whole.
- `skein/docs/design/testing.md`, 6.
- `temper/docs/plan/debt-27/notes/common.md`.

## Owns and may change

Owns the `members` list's lines and the `[workspace.dependencies]` lines above, and the item file.

The cut measured ten duplicates: the item's nine and `skein/crates/skein-fake-forge-http`. Keep each crate's first entry, in lines 18 to 27 of `members`. The second entries are the runs at lines 64 and 74 to 82 as the cut saw them.

Other drafts change the root `Cargo.toml`'s profiles (`hearth-speed`): keep your diff to the two lists above.

Left as they are: `smith-run-world` and the jig worlds in `[workspace.dependencies]` are as unused, and aren't skein's. List them in your report under `## Left`.

Options, each pre-approved:

- The readers of the lists are `tests/boundary/tests/direction.rs` and `tests/boundary/tests/skein_worlds.rs`, through the manifest and `cargo metadata`, and `tools/lints/lints.py`; `tools/tiers/packages.toml` names `skein-browser-world` as a package, not a dependency, and stays. If one of them needs a removed line, keep that line and say so in the report.

## Steps

Edit, then:

- the measure above; a script of your own, untracked and deleted before the push, that lists `[workspace.dependencies]` path entries under `skein/` whose key no other manifest names.
- `heavy cargo nextest run -p PKG --test direction` and `--test skein_worlds`, where PKG is the name in `tests/boundary/Cargo.toml`. Each run selects tests.
- `python3 -I -B tools/citations/citations.py --all` and `python3 -I -B tools/debt/debt.py`.

Commit, push.

## Report

As your text says: the measure's reading, the lines removed, and what you list under `## Left`.
