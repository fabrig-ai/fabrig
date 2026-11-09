# cli-c2-box-engine: temper-box and temper-engine in temper/bin/

The request: milestone C2 of `temper/docs/design/draft/cli.execution.md`. That's `temper/docs/design/draft/cli.md`, sections 4.2, 4.3, 4.6, 8.4, 8.5, 8.7 and 8.10, as its row says:
- `temper-box` (today's whole-product `temper`) and `temper-engine`, each in a package under `temper/bin/`, its `main.rs` at most 30 lines, its logic in crates;
- the libraries holding their names, renamed;
- their tables, sets, help and `--version`;
- pre-table modes as rows;
- generated documents regenerated;
- their internal variables renamed;
- the dropped twins' tests reworked.

It follows the conventions C1 landed. Remove the checks' exceptions C1 named for C2, and no others.

## Ranked

1. `temper-box` and `temper-engine` move and rename as above, with every command each has today kept, and their help and tables as the conventions give them.
2. Every caller follows: tests, scripts, documents, help pages and recipes naming the old binary, package or variable.
3. The checks C1 landed pass with C2's exceptions removed.

## Out

- `temper` the client (C3), which builds on C2's `temper-box` name.
- The worker and the agent (C6), being built elsewhere now. Their crates are left alone; whichever lands second catches up.
- Any new Cargo feature: the workspace has none, as `cli.md` says.

## Tests

Focused and fuzzy suites only. Each binary's step tests print its help and `--version`, refuse an unknown command, and refuse another binary's document as today. A story in a slower tier that a rename touches is changed and compiled, not run. No walk; `walk = false`.
