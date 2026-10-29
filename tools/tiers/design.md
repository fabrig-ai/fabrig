# Slow tier runner

Provisional, 2026-10-29. Run one tree's slow tier with its prerequisites.

## 1. In one page

- **One entry point:** from a fabrig checkout root on a builder,
  `tools/tiers/run TREE TIER [-- NEXTEST_ARGS...]` builds prerequisites and
  runs `heavy cargo nextest run -p PKG ... --profile TIER-TREE --no-fail-fast`,
  using the pair's sorted package names from `tools/tiers/packages.toml`,
  followed by the extra arguments unchanged. The map is generated from the
  packages with selected tests in each profile's workspace JSON listing.
- **The config owns the pairs:** the root `.config/nextest.toml` profile
  tables whose names contain a hyphen supply the pairs, split at the last
  hyphen into tier and tree. Unknown pairs and pairs missing a valid package
  set fail before any command runs.
- **Builds are sequential:** every build and nextest command uses `heavy`;
  the built bundle tool runs directly. Cargo metadata builds nothing and
  runs directly to discover the target directory.
- **Disk before each heavy command:** wait for at least 80,000,000,000 bytes
  available on `/`, checking once per minute and printing each wait.
- **Visible evidence:** print each step's command; heavy prints its log path
  and nextest's summary. Prerequisite failures name their step.

## 2. Prerequisites

Paths to source assets are relative to the checkout root. Build artifacts
and bundle output directories are beneath `cargo metadata`'s
`target_directory`, including when Cargo config relocates it. All builds
use Cargo's default dev profile, as the trees' browser scripts do.

| Pair | Prerequisites in order | Bundle environment |
| --- | --- | --- |
| browser-skein | wasm `skein-ui-example-shell-web`; `skein-ui-bundle`; run the tool; `skein-ui-example-server` | `SKEIN_UI_EXAMPLE_BUNDLE`, target's `ui-example-bundle` |
| browser-jig | wasm `jig-client-test-shell-web`; `skein-ui-bundle`; run the tool; `jig-test` | `JIG_TEST_BUNDLE`, target's `jig-test-bundle` |
| browser-temper, qa-temper | wasm `temper-client-shell-web` with `loopback-navigation`; `skein-ui-bundle`; run the tool; `temper` | `TEMPER_TEST_BUNDLE`, target's `temper-bundle` |

The wasm target is `wasm32-unknown-unknown`. Each tool invocation uses the
module's underscored package name, `/bundle/` prefix, and the old browser
script's stylesheets, pictures, files and noscript text. Temper matches
`temper/scripts/bundle.sh --loopback`, without embedding the bundle in the
native binary. Existing bundle outputs are rebuilt on every invocation.

All other configured pairs run nextest alone. Tier tests also use binaries
from their own packages (`skein-lab-browser`, `harness_fixture`, `jig-test`,
`smith`, `temper`); Cargo/nextest builds those integration-test targets.
The browser servers cross package boundaries and require the explicit builds
above. No additional cross-package binary or bundle requirement was found.

## 3. Environment and output

Every child inherits the caller's environment with `NEXTEST_DOUBLE_SPAWN=0`
(skein/docs/foundation/testing-strategy.md, 8). The chosen bundle variable
is set to the absolute output directory for nextest, replacing any inherited
value. `TEMPER_BUNDLE=none` stays as the builder sets it: temper's native
binary must not embed a person's browser bundle. Chromium, fonts and live
sign-ins remain the caller's environment prerequisites; this tool installs
nothing and performs no sign-in.

The runner sets umask 077, closes child stdin, and leaves child output visible.
Heavy supplies its own log paths and command summaries. A direct bundle tool
has no heavy log. Commands remain sequential, and their inputs must stay
still until completion. A low disk check sleeps 60 seconds, checks again,
and prints another line only if disk remains low; there is no disk timeout.

## 4. Exit codes

Return nextest's exit code, including a zero-selection or test-failure code.
Verification returns 0 for identical selections and 1 for differences,
naming every missing or added test by package, binary and test name.
Return 2 for a refused pair, invalid invocation/configuration, failed metadata,
failed disk check, command launch failure, or failed prerequisite; report the
failed step and do not start later steps. The runner does not retry failures.

## 5. Offline checks

`python3 -I -B tools/tiers/test_run.py` uses temporary fixture checkouts
inside this checkout, fake commands on PATH, and a mocked disk sleep.
It checks configuration-derived pairs, prerequisite order, arguments and
environment, map refusal and sorted package arguments, verification with fake
JSON listings, disk admission and refusal/failure exit codes without builds,
network access, Chromium or credentials. The suite's limit is 60 seconds.

## 6. Package-map verification

Run `tools/tiers/run --verify-all` with every green baseline. It checks all
configured pairs, including live profiles and process-temper, without running
their tests or building bundles. `tools/tiers/run --verify TREE TIER` checks
one pair. Neither form accepts extra nextest arguments.

For each pair, run `cargo nextest list --message-format json --profile TIER-TREE`
through heavy, first over `--workspace`, then over the map's `-p` arguments.
Compare the selected package/binary/test identities, ignoring filtered tests
and build artifact paths. Print both counts for every pair, and continue
checking the remaining pairs after a difference. Heavy's complete listing
output is captured with `HEAVY_TAIL` set to retain all lines; its evidence
around the JSON stays visible. A listing failure names its step and stops.

Verification follows the same unchanged 80,000,000,000-byte admission rule
before each heavy command. Workspace listings build the workspace test
executables; ordinary tier runs build only their mapped packages' tests and
the prerequisites in section 2. When profiles gain or lose packages, regenerate
the sorted map from their workspace listings and verify it before the baseline.

## 7. Later

The after-landing runner calls this entry point. The trees' old
`scripts/browser.sh` scripts retire in their own trees' branches.
