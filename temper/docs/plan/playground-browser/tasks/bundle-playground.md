+++
id = "bundle-playground"
part = "served"
kind = "feature"
needs = ["page-feature", "playground-worker", "page-policy"]
files = [
  "temper/scripts/bundle.sh",
  "Cargo.toml",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/playground.status.md",
]
effort = "high"
minutes = 75
+++

# `bundle.sh --playground OUT` and the `playground` profile

## Goal

"Built by `scripts/bundle.sh --playground OUT`: temper's web shell with `playground`, in the `web`
profile, and `temper-playground` for wasm32 in a profile of its own, `playground`, which inherits
`web` and optimizes for size ... Both bundled by skein's bundler with the worker's module ...
two builds at one commit are identical" (`temper/docs/design/playground.md`, 7). Add the option,
the profile and the `headers` file beside the files (`notes/contracts.md`, C6 and C8).

Acceptance:

- `temper/scripts/bundle.sh --playground OUT` builds `temper-client-shell-web` for wasm32 with
  `--profile web --features playground` and `temper-playground` for wasm32 with `--profile
  playground`, runs skein's bundler with `--module` (the shell), `--name temper_client_shell_web`,
  `--worker temper_playground=<the world's .wasm>`, `--page
  crates/temper-client-shell-web/playground/index.html`, `--prefix /bundle/`, the shell's
  stylesheets (`temper.css`, `task-pages.css`, `playground/playground.css`) and `--files`, then
  writes `OUT/headers` from `crates/temper-client-shell-web/playground/headers` with `{bundle}`
  replaced by `/bundle/<digest>` (the digest directory the bundler printed). Its usage line names
  the option; the other options are unchanged and `--playground` refuses being combined with them.
- The root `Cargo.toml` has `[profile.playground]`, `inherits = "web"`, `opt-level = "z"`, right
  after `[profile.web]`.
- `OUT` holds `index.html` (the playground's page, its `{bundle}` markers replaced), `headers`,
  and under its digest directory `boot.js`, `boot-worker.js`, the two modules and their glue,
  the stylesheets and `files/`.
- Two runs at one commit into two directories are identical (`diff -r`).
- The shipped `bundle.sh OUT` (no option) and `--loopback`, `--record` write what they wrote
  before; the gallery's test of the script's stylesheet lines
  (`temper/tools/gallery/src/tests.rs`, ~893) passes.
- `temper/docs/design/playground.status.md`, **Not yet**, 7 removed in place: delete "7: the
  served files and their `headers`," and "`scripts/bundle.sh --playground` and the `playground`
  profile. ", never reflowing; date line today's.

## Inputs

- `temper/docs/plan/playground-browser/notes/common.md`;
  `temper/docs/plan/playground-browser/notes/contracts.md`, C4, C5, C6 and C8.
- `temper/docs/design/playground.md`, 7 and 8 (the download's bar, 6 MB gzipped, is a figure to
  report, not a check here); `skein/docs/design/playground.md`, 7 (the worker's files);
  `skein/docs/design/ui-web.md`, 4.2 to 4.4 (the bundle, `--page`).
- `temper/scripts/bundle.sh` as it is; skein's bundler's CLI
  (`skein/crates/skein-ui-bundle/src/main.rs`: `--worker NAME=PATH`, `--page PATH`).

## Owns and may change

- The files in `files`. In the root `Cargo.toml`, only the new table, appended right after
  `[profile.web]`; no member or dependency line.
- `temper/tools/gallery/src/tests.rs`: only the test that reads `bundle.sh`'s `--stylesheet`
  lines, and only if the playground's invocation makes it read lines that are not the shipped
  bundle's: teach it to read the shipped invocation's lines alone (for example, stop at the
  playground's branch), keeping what it asserts.
- Pre-approved options:
  - If `--page` makes the bundler ignore `--stylesheet` (the page links its own), pass the
    stylesheets in whatever way the bundler copies them into the digest directory, and say how.
  - Run the script through `heavy` if heavy runs a script; if not, run its two `cargo build`
    commands through `heavy cargo build ...` first and then the script, whose builds are then
    fresh, and say which you did.
  - If `opt-level = "z"` fails to build or makes the world's module larger than `"s"`, keep
    `"s"` and report both sizes.

## Steps

1. The profile; the two wasm32 builds through heavy; note the two `.wasm` sizes.
2. The script's option, the bundler's call, `headers`; the usage line.
3. Build twice, `diff -r`; check `OUT`'s files against the acceptance; the gallery's test
   (`heavy cargo nextest run -p temper-client-gallery --lib`, the test that reads the script), and the shipped `bundle.sh OUT` once.
4. The status edit; the citations check; commit; push.

## Report

As the implementer's text says, and also: each file of `OUT` with its size, raw and gzipped
(`gzip -9 -c FILE | wc -c`), and their totals; the two `.wasm` sizes at `"s"` and `"z"` if you
measured both; `headers` quoted whole.
