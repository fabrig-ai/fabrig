+++
id = "smith-unused-deps"
part = "smith-items"
kind = "migration"
needs = []
files = ["smith/crates/smith-agent-shell/Cargo.toml", "smith/crates/smith-local-protocol/Cargo.toml", "smith/crates/smith-local-shell/Cargo.toml", "smith/crates/smith-local-service/Cargo.toml", "smith/tests/hosts/Cargo.toml", "smith/tests/protocol-llm/Cargo.toml", "smith/tests/session/Cargo.toml", "Cargo.lock", "smith/docs/development/debt/D014-unused-dependencies.md", "smith/docs/development/debt/README.md"]
effort = "medium"
minutes = 15
+++

# smith: pay D014, the manifests' unused dependencies

## Goal

Pay `smith/docs/development/debt/D014-unused-dependencies.md`: no smith manifest names a dependency
that none of its package's sources uses, unless a comment beside it says why.

The cut ran that search over every `smith/**/Cargo.toml` at the base (each dependency's snake-case
name, against every `.rs` file of the package). It found the item's six and one more:

- `smith-protocol-channel` in `smith-agent-shell`, `smith-local-protocol`, `smith-local-shell`,
  `tests/hosts`, `tests/protocol-llm` and `tests/session`: unused, each line goes.
- `smith-local-service`'s own `[dev-dependencies]` line, `smith-local-service = { path = ".", features
  = ["fixture"] }`: kept for the feature, and it has no comment. It gets one.

Acceptance:

- The same search over `smith/**/Cargo.toml` reports only the self-dependency, and that line has a
  comment.
- `Cargo.lock` matches, and each of the seven packages builds with its tests.
- The item's file and its README row are gone.

No check of manifests against sources is built: the item's **Paid when** does not ask for one. No test
changes, so this task has no hand-break.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `smith/docs/development/debt/D014-unused-dependencies.md`.

## Owns and may change

- Owns: the seven manifests' dependency lines, `Cargo.lock`, the item and its README row.
- Not yours: `smith/docs/design/`. Those documents name `smith-protocol-channel` as a crate, not as a
  dependency of these six: if one states such an edge, stop and ask.
- If a package fails to build without its line, the dependency is kept for a feature: put the line
  back with a comment saying which, and say so in the report.

## Steps

1. Remove the six lines. Add the comment above `smith-local-service`'s line, in the manifest's form:
   `# The crate's tests turn on its own fixture feature.`
2. Update the lock with `cargo metadata --format-version 1 > /dev/null`, and check by `git diff --stat
   Cargo.lock` that only the six packages' dependency lists moved.
3. `heavy cargo check --tests -p smith-agent-shell -p smith-local-protocol -p smith-local-shell
   -p smith-local-service -p smith-hosts-world -p smith-protocol-llm-world -p smith-session-world`.
4. Re-run the search with a throwaway script, untracked, deleted before the push.
5. Delete the item and its row; the citations check. Commit and push.

## Report

The common note's items; the search's result; net lines.
