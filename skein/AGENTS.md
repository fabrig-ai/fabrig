# skein

The io and protocol kit, and the rules every crate in fabrig follows. Read the root `AGENTS.md` first, then the design sections your task names.

- **The design:** `skein/docs/design/`, cited as `skein/docs/design/tls.md, 3.6`.
- **The rules:** `skein/docs/foundation/programming-model.md` (step machines, states, what step code may use), `testing-strategy.md` (tests, worlds, suites) and `notes.md` (their reasons and open questions).
- **The layout:** step crates in `crates/*`, their worlds in `tests/*`, shared test harnesses in `testing/*` (`skein/docs/design/testing.md, 6`).
- **Step crates** follow `programming-model.md, 10`: no `std`, no `unsafe` but where a module says why, every `match` lists every variant (no `_`, no `matches!`), no indexing, no unchecked arithmetic, no `as`, no `unwrap`. The workspace's lints and `skein/clippy.toml` hold it: `include_bytes!`, `copy_from_slice` and `Vec` only under a scoped `#[expect]` with its reason.
- **skein-tls** is also checked alone (`heavy cargo check -p skein-tls`), since its features differ standalone.
- **rustls's debug assertions are off** in every dev build (`[profile.dev.package.rustls]`; `skein/docs/design/tls.md, 6`).
- **Markdown read by tests:** the UI fixtures and the GitHub conformance design.
