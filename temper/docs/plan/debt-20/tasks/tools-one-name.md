+++
id = "tools-one-name"
part = "tools-namespace"
kind = "feature"
needs = ["tools-world-names"]
files = ["smith/crates/smith-domain-tools/src/**", "smith/crates/smith-domain/src/translate.rs", "smith/docs/development/debt/D007-tools-namespace.md"]
effort = "high"
minutes = 50
+++

# smith: a directory's mount is one name

## Goal

Pay `smith/docs/development/debt/D007-tools-namespace.md`. `Directory.mount` is a `Name` and `Authority.cwd` an `Option<Name>`, and the code for other shapes goes. Every test is on one-name mounts already (`tools-world-names`), so its assertions don't change here.

```rust
pub struct Authority { pub cwd: Option<Name>, pub repos: Box<[Directory]>, pub grants: Grants }
pub struct Directory { pub host: Box<[Name]>, pub mount: Name, pub root: Token, pub writable: bool }
```

Acceptance:

- `rg -n 'Box<\[Name\]>' smith/crates/smith-domain-tools/src/authority.rs` lists only `Directory.host`.
- `rg -n 'fn first_name|mounted inside|Nested mounts|longest mount' smith/crates/smith-domain-tools/src` finds nothing, and `choose` compares no lengths: a name holds one directory, since `admit` refuses two at one mount. `locate`'s `longest` for host aliases stays.
- `beneath` has no empty-mount arm and `resolved` no `is_empty` arm or first-component split: a place's mount is its name, the working name's `Resolved::Working` and any other's `Resolved::Other`.
- `heavy cargo nextest run -p smith-domain-tools -p smith-tools-world -p smith-session-world -p smith-domain -p smith-domain-session -p smith-agent-service` passes, with no assertion changed.
- The branch's net production lines fall (estimate: -50). The item is deleted.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `smith/docs/development/debt/D007-tools-namespace.md`
- `smith/docs/design/domain/tools.md`, 2

## Owns and may change

- **Owns:** the files in `files`.
- **Reader files, the literal's field only:** `smith/tests/session/**`, `smith/tests/tools/**`, `smith/crates/smith-domain-session/src/tests.rs`, `smith/crates/smith-agent-service/src/tests/line.rs`, and `jig/tests/channel/tests/suite/source_carry_over.rs` (`cwd: Box::new([])` is `cwd: None`). The producer's branch carries that one jig line. Others the compiler finds, likewise.
- **`admit`'s checks** on a mount's length, against `limits.path_bytes`, stay; the count of its names goes, since it has one.
- **`Namespace` and `namespace_bytes`** answer as they do: `cwd` is the first and only name. A `None` `cwd` with two directories answers `Namespace::One`; rewrite the comment beside it to what's true now.
- **`path::normalise`** keeps its slice parameter: pass `working.cwd.as_slice()`.
- **`smith/benchmarks/tasks/**` stays:** that grader names old types and isn't built in the workspace.
- **Not to change:** any wire or stored form. `Resolved`, `Place` and `Namespace` are what cross; if a generated codec or a schema turns out to read `Directory` or `Authority`, stop and ask.

## Steps

1. The types, `admit`, `locate`, `choose`, `beneath`, `resolved`, `namespace`, `namespace_bytes`, `kit.rs`'s `directories` and `first_name`.
2. `translate.rs`: `mount: name`; `cwd` is the first directory's name, cloned.
3. The readers' literals.
4. Delete the item. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p smith-domain-tools -p smith-domain -p smith-tools-world -p smith-session-world -p smith-domain-session -p smith-agent-service --all-targets -- -D warnings`; the tests above; `heavy cargo check -p jig-channel-world --tests` (the package of `jig/tests/channel`; read its manifest for the name if the compiler says otherwise). The citations check. Commit and push.

## Report

The common note's items.
