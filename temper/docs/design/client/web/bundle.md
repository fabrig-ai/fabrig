# The browser's bundle in the binary

The browser's embedded bundle is split from [the browser overview](README.md).

## 1. The bundle in the binary

The `temper-box` binary holds its bundle, so that `temper box` and `temper-box
serve` serve the browser with nothing beside the binary, and `cargo
build` in a checkout makes that binary. The engine's binary,
`temper-engine`, holds the same bundle through the same crate, the only
one of a deployment's binaries that does (`../../protocol/binaries.md`,
2). It is a person's convenience:
the machine's builds keep building the way that serves their work,
without it.

- **One file.** The binary holds the bundle: the shell's module, its
  glue, and the stylesheets and files skein's bundler puts beside them
  (`serving.md`, 1, `look.md`), and their compressed forms (skein's `ui/web/bundle.md`, 2).
  Running it is the whole install: no directory beside it,
  no `scripts/bundle.sh` step.
- **A plain `cargo build` makes it.** `cargo build`, then
  `./target/debug/temper box`, serves the browser; so do `cargo build
  --release` and `cargo install --git` of temper's repository.
- **A crate of its own embeds it,** `temper-client-bundle`, ordinary
  Rust on the `binary` lint profile (`tools/lints/design.md`, 2.3), which
  `temper-box-shell` depends on. Its library is data: the bundle's files, each its path
  from the bundle's root (`index.html`, then `<digest>/…`) and its
  bytes by `include_bytes!`, in a directory read's order, and the
  digest, both written by its build script into `OUT_DIR`. A build
  without the embed has no files and no digest. The include sits in a
  small crate, not in `temper-box-shell`'s, because a build script that runs
  makes its package build again, and `temper-box-shell`'s is the slowest to
  build. `main.rs` and the integration tests live in the binary's
  package, `bin/temper-box` (`../../cli/README.md`, 2).
- **Its build script,** unless `TEMPER_INTERNAL_BUNDLE` is `none`, takes
  `scripts/bundle.sh`'s steps with the cargo running it (`CARGO`), from
  the workspace's root, into the nested build's directory (below):
  1. `build --locked --target wasm32-unknown-unknown -p
     temper-client-web-page`, the shipped page's entry (`shell.md`, 1), with the
     module's profile (below);
  2. `build --locked --release -p skein-ui-bundle`: the bundler,
     always optimized, as skein's `ui/web/bundle.md`, 1 asks of a tool;
  3. the bundler, with `scripts/bundle.sh`'s inputs (the name, the
     title, the prefix `/bundle/`, `temper.css`, `task-pages.css`, the
     placeholder's `loading.html` and `loading.css`, the shell's `files/`
     and the noscript words), into its `OUT_DIR`;
  4. the list of the bundle's files, written into `OUT_DIR` for the
     library's include.
  It has no build dependencies: the bundler is built by the nested
  cargo and run, never compiled into the script, so a build without the
  embed compiles a script of `std` alone, and nothing of the bundler is
  ever linked into `temper-box` (skein's `ui/README.md`: "never linked into a
  service or a module"). One `Cargo.lock` still locks the bundler's
  `wasm-bindgen-cli-support` with the module's macros (skein's
  `ui/web/bundle.md`, 1). The script's logic (the dep-info's read, the
  nested commands, the bundle's directory read into its list) is plain
  functions, which the crate's tests take on samples.
- **The nested build's directory** is `<target>/bundle/`, `<target>`
  the outer build's, found as the nearest ancestor of `OUT_DIR` holding
  cargo's `CACHEDIR.TAG`. The outer cargo holds its own directory's
  lock while build scripts run, so a nested build there would wait on
  it forever. It is one directory, not one under each `OUT_DIR`, so
  that a check, a clippy run, a build and a test build of `temper-box`
  reuse one module and one bundler, and a build after a change is
  incremental. Under `cargo clippy`, cargo builds a member's build
  script apart, so the first clippy run and the first build after a
  change each bundle once; the second finds the module built. A build
  without the embed never makes the directory.
- **None of the outer build's flags or wrappers:** the nested cargo
  runs without `CARGO_ENCODED_RUSTFLAGS` and `RUSTFLAGS` (the host's
  flags, not wasm32's), `RUSTC_WORKSPACE_WRAPPER` and `CLIPPY_ARGS`
  (which `cargo clippy` sets, and which would make a module of another
  fingerprint, built again at each switch between clippy and build),
  with `CARGO_TARGET_DIR` set anew. It keeps the jobserver
  (`CARGO_MAKEFLAGS`), so that its jobs count within the outer build's.
- **Its own selection:** both nested cargo commands run with
  `CARGO_RESOLVER_FEATURE_UNIFICATION=selected`, against the workspace's
  unification of every member (`skein/docs/foundation/cli.md`, 3). The
  module's features come from the packages it builds alone, so a union
  resolved for `wasm32-unknown-unknown` over members that never build
  there doesn't reach it.
  Its stdout goes to the script's stderr, since cargo reads the
  script's stdout.
- **Rebuilt only when what it is made from changes,** each by
  `cargo:rerun-if-changed`: each source file the nested build's dep-info
  names (the `.d` cargo writes beside the module: every path package
  the module is built from, temper's client crates and, when a local
  jig or skein is patched in, theirs); each of those packages'
  `Cargo.toml`, the workspace's `Cargo.toml` and `Cargo.lock`, which
  pins the git dependencies; the shell's stylesheets, its placeholder and
  its `files/`;
  and `TEMPER_INTERNAL_BUNDLE`, by `cargo:rerun-if-env-changed`. The first run
  has no dep-info yet, and runs regardless. An edit elsewhere never
  runs the script, and a build that does not include `temper-box` (a
  domain's tests, the client crates' own) never runs it.
- **The module's profile:** a dev build embeds a dev module, as
  `scripts/bundle.sh` makes; a release build embeds one built with
  `web`, the profile skein's `ui/web/bundle.md`, 1 names for an
  application's release module, which temper's workspace defines as
  skein's does. The script maps cargo's `PROFILE`, `debug` or
  `release`, to `dev` and `web`.
- **`TEMPER_INTERNAL_BUNDLE`:** unset or `embed`, the embed; `none`, no nested
  build and no bundle; any other value fails the build, naming the two.
  `embed` is for a build whose environment says `none` and wants the
  browser this once. The machine's builds set `none`: the gate's and
  development builds, a machine without the
  target, an editor's own (rust-analyzer runs build scripts at its
  start). Such a binary serves the page for no bundle (`serving.md`, 1).
- **The embed needs the wasm32 target.** Before any nested build, the
  script asks rustc for the target's library directory
  (`--print target-libdir`); without it, the build fails with one
  message: `temper-client-bundle: the browser's bundle needs the
  wasm32-unknown-unknown target: add it (rustup target add
  wasm32-unknown-unknown), or build without the browser
  (TEMPER_INTERNAL_BUNDLE=none)`. Never a binary without its browser in
  silence. The repository's `rust-toolchain.toml` pins the channel and
  names the target (`skein/docs/foundation/programming-model.md`,
  10.1), so rustup
  installs it with the toolchain; the message is for a toolchain
  installed otherwise, or rustup told not to install.
- **Offline:** the nested build may download crates only wasm32 uses,
  as any first build of the target does; an offline build runs `cargo
  fetch` first, which fetches for every target.
- **Served as a directory's is.** At start, `application.clients.bundle`
  `null` takes the binary's bundle: its files, already in the binary,
  are made into `temper-service`'s `Bundle` (its page, its digest, its
  files by path with their types from the closed list, its `ETag`) by
  the same code as a directory's read, given files in memory in place
  of a directory, so the same checks hold: the closed list, each path's
  bound, one digest directory, and `clients.bundle_files` and
  `clients.bundle_bytes`, a start past them refused naming the limit
  (`application.clients.bundle: the binary's bundle: <path>: <why>`).
  The copy is held as a read one is, within what the profile's worst
  case already counts for the bundle, so the memory account does not
  change. `temper-service` adopts a bundle, wherever the binary got it.
  A directory configured is read as before, and the binary's is unused.
- **The fake-backend bundle is provisioned separately.** Build it with
  scripts/bundle.sh DIR, then start temper-box --fake-backends
  --bundle DIR even with an empty configuration (../../protocol/configuration/directories.md,
  1). An explicit --bundle overrides source application.clients.bundle;
  otherwise that configured directory is used.
  The embedded bundle is not used for that box. Missing provision is a startup refusal explaining the bundle
  command and --bundle. Startup never builds or downloads the bundle.
  The second-embed prototype added 8,974,112 bytes in the measured dev build,
  exceeding its 5 MB limit; the provisioned path keeps that cost outside
  ordinary builds. TEMPER_INTERNAL_BUNDLE=none runs no nested browser build.
- **`scripts/bundle.sh` stays,** for a bundle built apart: the browser
  tier's and the journeys tool's, whose pages load with their internal
  settings (`shell.md`, 1; `testing.md`, 3), and a developer's. It builds the
  shipped page as the binary's build does, with no switch of its own
  but `--release` and `--playground`. `scripts/install.sh PREFIX`
  installs the binary alone, built with `--release --locked`, and
  prints its path.
- **Tested in both builds.** Every suite passes with and without the
  embed, and the gate runs them under `none`. The build script's logic
  is tested on samples in its crate; the in-memory read on fixture
  files beside the directory's read, the same `Bundle` and the same
  refusals; and a start with `bundle` `null` expects what its build
  holds, which `temper-box-shell` says (`temper_box_shell::bundle::OWN`, the
  binary's digest or none): the page for no bundle under `none`,
  otherwise the bundle's page at `/` and its files under
  `/bundle/<digest>/` (`../../testing/process-tier.md`). The page for no
  bundle stays proved where it is built, in `temper-service`. The embed
  itself is a person's build, which fails loudly when its steps do; a
  change to it checks it with `TEMPER_INTERNAL_BUNDLE=embed cargo nextest
  run -p temper-box`, which the gate does not run.

## 2. Open questions

- **A bundle built apart, embedded:** `TEMPER_INTERNAL_BUNDLE=DIR`, a bundle's
  directory embedded in place of building one, so that one wasm32
  build serves binaries for several hosts and a machine without the
  target can still make a whole binary. Default: none; it waits for
  the release pipeline (`bundle.md`, 1).
- **Served from the binary's bytes,** with no copy at start: a `Bundle`
  that borrows. Default: copied (`bundle.md`, 1), until a bundle whose copy
  matters.
- **The embed's steps, skein's for every application:** the nested
  build, its directory, its environment and what reruns it are nothing
  of temper's. Default: temper's crate, until a second application
  embeds its bundle (jig's reference application, or skein's example),
  when they move to a small crate of skein's that a build script
  depends on (skein's `ui/web/README.md`, section 12).
- **A binary to download:** where release binaries are built, for
  which targets and C library (a static musl build, or a floor of
  glibc), and where they are published. Default: none; temper's
  binary is whole (`bundle.md`, 1), not shipped.
