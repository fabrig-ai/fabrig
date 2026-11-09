# cli-c8-tools: temper's tools and fixtures in temper/bin/, as internal binaries

The request: milestone C8 of `temper/docs/design/draft/cli.execution.md`. That's `temper/docs/design/draft/cli.md`, section 3.1: temper's tools and fixtures, each an internal binary in a package of its own under `temper/bin/`, its `main.rs` at most 30 lines and its logic in crates, following the conventions C1 landed (`skein/docs/foundation/`'s command-line page, skein-shell's table, help's format and the checks). Remove the checks' exceptions C1 named for C8, and no others.

## Ranked

1. Each tool and fixture moves to `temper/bin/<name>/`, keeping what it does and its arguments, with its table, help and `--version` as the conventions give them.
2. Every caller follows: scripts, documents, tests and recipes naming the old paths or binary names.
3. The checks C1 landed pass with C8's exceptions removed.

## Out

- The products' binaries (C2, C3, C6), features (C5), the test set's names (C4), and the site (C7).
- Any new Cargo feature: the workspace has none, as `cli.md` says.

## Tests

Focused and fuzzy suites only. Each moved binary's step test prints its help and `--version`, and refuses an unknown command, as rows of one table test where it fits. No walk; `walk = false`.
