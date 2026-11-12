# version-rebuilds: status

2026-11-11. Where the design lags or departs from [version-rebuilds.md](version-rebuilds.md).

The design carries every item, none dropped. Item 1: each binary's
package depends on `temper-version` and hands its identity to its
crate's `main`, whose table names none, and the client's package splits
as the agent's did, into `temper/bin/temper` and its crate
`temper-client-shell`, which takes `line`'s line (`../release.md`, 3;
`../cli/README.md`, 2 and 3; `../protocol/components.md`). Item 2 is a
row of the existing dependency rule, the boundary test's direction
rule, `tests/boundary/tests/direction.rs`, which reads every member's
manifest in every table (`../release.md`, 3). Item 3: `--version`'s
line is unchanged, and the tests that read the identity move to the
binaries' suites (`../cli/README.md`, 3 and 10). Item 4: `../release.md`,
3, and `../cli/README.md`, 8, which names where `TEMPER_INTERNAL_COMMIT`
is read and where its `Variable` now lives, `temper-shell`'s
`environment`.

- **Design departs:** item 1's "rebuilds no ... test binary": a
  binary's package's own tests, its `tests/` suites, recompile at a new
  commit, as `temper-version`'s own tests do, because Cargo builds a
  package's tests against every dependency of the package and gives no
  dependency to a binary target alone; nothing they link rebuilds, where
  before every one of them rebuilt with the libraries below it
  (`../release.md`, 3). The goal's designer, 2026-11-11: no option
  within stable Cargo avoids it, and moving the suites out of their
  packages would lose `CARGO_BIN_EXE_` (`../cli/README.md`, 2).
