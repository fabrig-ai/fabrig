# macos-box-start: what every task here holds to

- **Nothing here runs on a Mac before landing.** No builder runs macOS. What decides Darwin's path
  is written so that a Linux test reaches it, and what still can't be is said in the report.
- **Linux stays green and means what it means.** On Linux every errno from these calls is a refusal
  as before, and the existing refusals keep their words.
- **Darwin's compile.** After the checks your implementer text names, also run
  `heavy cargo clippy -p PKG --all-targets --target aarch64-apple-darwin -- -D warnings` for each
  package you changed (`skein/docs/design/shell.md`, section 9). If the target or a dependency
  won't build there on the builder, say so in the report and go on: the macOS workflow compiles
  every target, and this does not stop your acceptance.
- **The rule above all of it** (`temper/docs/design/draft/macos.md`): keep it cheap. No port, no new
  Darwin call, no new workflow step.
- **Not touched:** `.github/workflows/macos.yml`, the story in `temper/crates/temper/tests/process.rs`,
  and `temper/docs/design/draft/`.
