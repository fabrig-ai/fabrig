# macOS dispatch helper

The contract is [`macos.md`](../../temper/docs/design/testing/macos.md), sections
1–8. `.github/workflows/macos.yml` checks out this helper at the workflow commit,
beside the commit it tests.

Run the offline fixtures with `python3 -I -B tools/macos/test_macos.py` (no network
or builds, temporary directories inside the checkout, under 60 seconds).

`python3 -I -B tools/macos/macos.py --help` lists the commands:

- `plan`: read `SET`, `TESTS` and the tested `packages.toml`; write part flags,
  newline-separated ids, `given` and the JSON list `parts` to `GITHUB_OUTPUT`.
  Unknown-to-the-set ids get notices, not refusals. `e2e` holds `process`,
  `browser` (jig and skein) and `browser-temper` (the playground); `all` adds
  the four trees' suites. Each part has its own build probe, `build.PART`.
- `filterset`: read `IDS`, omitting build probes.
- `build-case`, `merge`, `check-one-process`, `check-nextest`, `report`: construct
  or validate JUnit using explicit paths. `report` reads the JSON `PARTS` list,
  accepts flat files or `gh run download`'s `part-PART/PART.xml` directories, and
  copies the full part files beside the failure-only report. A fault exits 2.
- `wasm-compiler`: check Homebrew's newest LLVM targets wasm32 and export its target-only C tools.
- `install-ripgrep`: download smith's workflow-pinned ripgrep release, check its
  SHA-256 before unpacking into `RUNNER_TEMP/ripgrep`, print its version through
  `RUN`, and append its directory to `GITHUB_PATH`.
- `setup-runner`, `annotate`: keep each check's log and annotate failures; the
  runner preserves the command's exit status. With `MACOS_STARTED` and `PART`,
  checks stop 15 minutes before the part's timeout (150 minutes for e2e parts,
  300 for trees, 60 for live). Each command has its own process group; deadline,
  SIGINT and SIGTERM stops send SIGTERM, then SIGKILL after a 60-second grace,
  and raise a fault (exit 2), never a compiler red. Without `MACOS_STARTED`,
  checks have no deadline. The live job's single shell step does not forward
  cancellation signals to the helper (temper/docs/design/testing/macos.md, 10).
- `install`, `build`, `fetch-forge`, `run`, `verdict`, `prune`: the jobs' common
  operations, using `RUN`, `RUNNER_TEMP`, `IDS`, `GIVEN` and the tested tree's
  metadata and tier tools. `build` writes `build_red` and `run_tests` outputs,
  gives each command a notice counting compiled dependencies and workspace
  crates, and appends `MACOS_BUILT=true` to `GITHUB_ENV` only after every command
  succeeds. Browser pair runs warn on any compiled crates without changing
  their verdict. `verdict` fails on a failed case, after its file is uploaded.
  `fetch-forge`
  exports `TEMPER_TEST_FORGEJO` and `TEMPER_TEST_FORGEJO_RUNNER` after the build
  for `process`, `live` and `temper`. Only `temper` restores and saves its fetch
  cache between runs; configuration and data start empty, and `process` and
  `live` start with an empty cache too.
  `prune` keeps each profile's `deps`, `build` and `.fingerprint`, less workspace
  package/target entries matched by whole name, optional sixteen-hex cargo hash
  and extension, in either spelling. Only sourced packages in the tested
  `Cargo.lock` count as dependencies; shared names are kept with a warning.
  Everything outside profile ancestors is removed except `target/.rustc_info.json`
  and `target/CACHEDIR.TAG`. Its logged, no-deps metadata query uses the save
  reserve even after the deadline. If the query fails, prune warns, keeps the
  three dependency areas whole, and removes other outputs; pruning failures
  warn without failing the save step. Cache saves use the complete key after a
  successful build, otherwise a run/attempt-specific partial key; restore tries that lock's
  newest partial before older locks of the same toolchain.

Nextest completeness uses its JSON selection listing, with the run's profile,
package and filter arguments and ignored-test policy, rather than relying on a
version-specific global-timeout marker. One-process completeness reads both
`run.json` and `results.json`. Neither mode accepts a partial report.
