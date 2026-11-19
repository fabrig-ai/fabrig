# macOS dispatch helper

The contract is [`macos.md`](../../temper/docs/design/testing/macos.md), sections
1–8. `.github/workflows/macos.yml` checks out this helper at the workflow commit,
beside the commit it tests.

Run the offline fixtures with `python3 -I -B tools/macos/test_macos.py` (no network
or builds, temporary directories inside the checkout, under 60 seconds).

`python3 -I -B tools/macos/macos.py --help` lists the commands:

- `plan`: read `SET`, `TESTS` and the tested `packages.toml`; write part flags,
  newline-separated ids, `given` and the JSON list `parts` to `GITHUB_OUTPUT`.
  Unknown-to-the-set ids get notices, not refusals.
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
  runner preserves the command's exit status.
- `install`, `build`, `fetch-forge`, `run`, `verdict`, `prune`: the jobs' common
  operations, using `RUN`, `RUNNER_TEMP`, `IDS`, `GIVEN` and the tested tree's
  metadata and tier tools. `build` writes `build_red` and `run_tests` outputs;
  `verdict` fails on a failed case, after its file is uploaded. `fetch-forge`
  exports `TEMPER_TEST_FORGEJO` and `TEMPER_TEST_FORGEJO_RUNNER` after the build
  for `process`, `live` and `temper`. Only `temper` restores and saves its fetch
  cache between runs; configuration and data start empty, and `process` and
  `live` start with an empty cache too.

Nextest completeness uses its JSON selection listing, with the run's profile,
package and filter arguments and ignored-test policy, rather than relying on a
version-specific global-timeout marker. One-process completeness reads both
`run.json` and `results.json`. Neither mode accepts a partial report.
