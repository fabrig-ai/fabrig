# macos-cheap: what every task holds

- **The user's rule governs every choice:** "make sure that anything we do is relatively cheap". Whatever is Linux's may be degraded or less sophisticated on macOS; rough edges are fine. Never a port for its own sake: the simplest thing that lets the story pass (`temper/docs/design/draft/macos.md`, 1), or a Linux-only entry.
- **The acceptance is on GitHub's runner, after landing.** No builder runs macOS. A task's acceptance is that Linux stays green and that its macOS code compiles for the target where it can. Nothing here is proved on a Mac by a task.
- **The Darwin check, where it can reach.** The builders have the `aarch64-apple-darwin` target. `heavy cargo clippy --target aarch64-apple-darwin -p PKG [-p PKG ...] --all-targets -- -D warnings` reaches a package that does not reach rustls: its `ring` crate compiles C against Apple's headers, which no builder has, so a package that reaches it fails in `ring`'s build script, and that failure is no finding. Nothing links. Never `--workspace`.
- **The platform is decided by a binary's shell, never by step code.** A step crate holds no `cfg(target_os)`; the machine is given the platform as data. Test crates and shell crates may hold it.
- **skein and jig are shared with hearth:** keep a change there small. hearth is not touched.
- **Budgets and timeouts are the user's.** `.config/nextest.toml`'s budgets, `global-timeout`s and profiles are not touched, and a test's deadline is not raised for macOS.
- **No test reaches a real service.** Fakes and loopback, as the worlds already have them. The named `curl` of a spec's own steps is the only exception, and checks every download by its digest.
- **`docs/design/` stays out of a task,** but where its spec says otherwise; what the code taught goes in the report.
- **The workflow names no tool, machine or session of whoever runs this development** (`AGENTS.md`, 1; `tests/boundary/tests/outward.rs`).
