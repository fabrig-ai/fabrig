# README: status

2026-11-11. Where temper's code lags or departs from [README.md](README.md).

`temper` takes its table in `../draft/cli.execution.md`'s C3; until then
the client's table is `temper-shell`'s `commands` rows, and its help is
today's (`../protocol/binaries.status.md`).

- **Not yet:** 2, 3, 8 and 10: the four tables' packages hand `temper-version`'s identity to their crates' `main`: today each crate links `temper-version`, its table holds `VERSION` and `COMMIT`, `TEMPER_INTERNAL_COMMIT`'s `Variable` is `temper_version::INTERNAL_COMMIT`, and the agent's crate's unit tests read `temper-version`.
- **Not yet:** 2 and 3: the client's package at `temper/bin/temper-client` and its crate `temper-client-shell`, with `main(line)`: today one package, `temper/crates/temper-client`, holds the library and the binary, whose `main.rs` makes the capture.
