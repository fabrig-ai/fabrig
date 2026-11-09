# agent-binary-closure: contracts

The shapes that cross tasks, binding for every spec that cites this note.

## 1. `temper-launcher` (task `launcher-crate`)

A new package, `temper/crates/temper-launcher`, library `temper_launcher`,
ordinary Rust under the `binary` lint profile
(`temper/docs/design/protocol/setup.md`, 6.6 and section 11;
`temper/docs/design/protocol/README.md`, section 5). Its public items sit at
the crate root and are `temper_fetch_shell::launcher`'s, moved with their
signatures and bodies unchanged:

```rust
pub fn is_launcher(program: &OsStr) -> bool;
pub fn run(name: &str, program: &OsStr, arguments: &[Box<[u8]>]) -> u8;
pub fn place(directory: &Path, cache: &Path) -> Result<(), String>;
pub const IO: skein_io::Limits;
pub fn beside(directory: &Path) -> Option<PathBuf>;
pub fn substituted(bytes: &[u8]) -> Result<Option<Box<[u8]>>, String>;
```

Its normal dependencies: `temper-fetch-shell` (Claude Code's fetch,
`fetch::claude_code`), `temper-fetch`, `smith-agent-document` (the links'
directory checked private, `program::private_directory`), `skein-io`,
`skein-lib`, `skein-json`, `skein-shell`, `skein-forge`.

`temper-fetch-shell` keeps `pub mod fetch` alone: no `launcher` module,
no re-export of `temper-launcher` (which depends on it), and no smith
crate among its dependencies.

## 2. jig's directories (task `shell-without-jig`)

`temper_shell::directories::Directories` loses its method `jig`, and
`temper-shell` its dependency on `jig-protocol-config`. The method's body
moves to `temper-engine-shell`'s crate root:

```rust
/// jig's form of the three directories (temper/docs/design/protocol/README.md, section 5).
#[must_use]
pub fn jig_directories(directories: &Directories) -> jig_protocol_config::Directories {
    jig_protocol_config::Directories::given(&directories.config, &directories.data, &directories.cache)
        .expect("resolved absolute bounded directories")
}
```

The box's `temper::directories` module re-exports it
(`pub use temper_engine_shell::jig_directories;`), beside its re-export of
`program_environment`. Every former `x.jig()` is `jig_directories(&x)`.

## 3. The agent's closure row (tasks `launcher-crate`, `agent-version`)

`tests/boundary/tests/closure.rs`'s `temper-agent` row, as
`temper/docs/design/protocol/README.md`, 4.4 has it once both land:

```rust
temper: &["temper-agent", "temper-shell", "temper-docs", "temper-launcher", "temper-fetch", "temper-fetch-shell", "temper-version"],
trees: &["jig"],
exceptions: &[],
```

`launcher-crate` adds `"temper-launcher"`; `agent-version`, which needs it,
adds `"temper-version"`. Nothing else of the row changes, and
`binaries.rs`'s `temper-engine` row doesn't change at all.
