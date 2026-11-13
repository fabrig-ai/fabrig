# The download

Provisional, 2026-10-26. How temper reaches a person who did not build
it: the one Linux binary of a release, its identity and checksum, what
the machine must have, how a release is made, and the walk that proves
it on a fresh machine (`draft/v0.1.0.md`, 5 and 9), and Apple Silicon's
file the site serves beside it (5.1). The site that links them is
`draft/first-release.md`'s, the user's; the rest of macOS is
`draft/macos.md`'s. What is still open is section 7.

## 1. In one page

- **One file a release:** `temper-VERSION-x86_64-linux`, the whole
  product, its browser bundle embedded, with
  `temper-VERSION-x86_64-linux.sha256` beside it (section 2).
- **It says what it is:** `temper-box --version` prints the version and the
  commit it was built from, and the release's file name carries the
  same version (section 3).
- **Its floors are said, not guessed:** Linux 6.12 or later, glibc
  2.39 or later, x86-64; the binary is built against the oldest glibc
  it promises, so that Debian 13 and Ubuntu 24.04 with a 6.12 kernel
  both run it (section 2).
- **What the machine must have** is a short list the help gives and
  `temper box` checks at its start, naming what is missing: git 2.41 or
  later, `/usr/bin/rg`, `/bin/sh` and the system's CA certificates;
  Forgejo and its runner are fetched by the box, pinned; Claude Code
  only for Claude (section 4).
- **A release is made by a script:** `scripts/release.sh VERSION`
  builds the file at the tag, in a root of the glibc floor's
  distribution entered with bubblewrap, checks its symbols, and writes
  the checksum; the files go among temper's releases at the tag
  (section 5).
- **Proved on a fresh machine:** rung A walked from the downloaded file,
  by the help alone, on a machine that has only what section 4 lists
  (section 6).
- **Apple Silicon's file, for the site:**
  `.github/workflows/release-macos.yml`, started by a dispatch alone on
  GitHub's `macos-26`, builds `temper-box` for release at the commit it is
  started on, strips it and signs it again ad hoc, checks that it runs,
  and leaves it with that commit as the run's artifact
  `temper-Darwin-arm64`, which the site's host fetches; no tag, no
  release, no signature but an ad hoc one (5.1).

## 2. The file

- **Its target:** `x86_64-unknown-linux-gnu`, built with
  `TEMPER_INTERNAL_BUNDLE=embed` and the release profile, stripped; one file,
  nothing beside it at run time but section 4's programs.
- **Its floors:** Linux 6.12, skein's (skein's `runtime/shell.md`, section 4 and `runtime/readiness.md`
  ); glibc 2.39, the oldest that has the process calls skein's
  shell uses (`pidfd_spawn`, `pidfd_getpid`,
  `posix_spawnattr_setcgroup_np`). Built against a newer glibc, it
  would require that one: so the release builds in a root of the floor's
  own glibc (`release.md`, section 5), and its versioned symbols are checked against
  2.39 before it is published. That 2.39 is the newest of the
  `GLIBC_` symbol versions those calls carry, as the first release
  build's own symbols show; a newer one found there moves the floor
  here.
- **Not static, for now:** skein's shell uses glibc's extensions, so a
  musl build needs a spawn of its own first (section 7).
- **Its checksum:** SHA-256, in the form `sha256sum` checks, one line
  naming the file.

## 3. Its identity

- **`--version`** prints one line, `NAME VERSION (COMMIT)`, NAME the
  binary's, before any configuration is read, exit 0: `temper-box
  VERSION (COMMIT)`, `temper-engine VERSION (COMMIT)`, each of a
  deployment's binaries the same way (`protocol/binaries.md`, 1). It is
  a flag, with no `version` command beside it
  (`skein/docs/foundation/cli.md`, 7).
- **The commit's build name,** `TEMPER_INTERNAL_COMMIT`, is the one
  `temper-version`'s build script hands its library (`cli/README.md`,
  8); every binary's line reads it there.
- **VERSION** is the workspace's version in `Cargo.toml`, `0.1.0` at
  v0.1.0's tag and `0.0.0` between releases; **COMMIT** the commit it
  was built from, twelve hex digits, with `-dirty` when the tree had
  changes to tracked files, `unknown` when built outside git.
- **Both are `temper-version`'s,** held in one place every binary reads:
  its build script reads git at build time, the commit `HEAD` names and
  whether tracked files differ from it, and reruns when `HEAD`, the
  branch it names, the packed refs or the index moves, read where `git
  rev-parse` puts them, a worktree's included; its library holds
  `VERSION` and `COMMIT`, and `line(NAME)`, the line above. Only a
  binary's package links it, so a commit rebuilds it and relinks those
  binaries, and nothing they link.
- **Each binary hands its identity in:** its package's `main.rs` passes
  `temper-version`'s `VERSION` and `COMMIT` to its crate's `main`, which
  prints them with its table (`cli/README.md`, 2 and 3; a tool's,
  `testing/tools.md`, 3 and 4); the client's
  passes `line`'s line, which its crate prints as it is. No library
  names a build identity of its own, and no package outside
  `temper/bin/` depends on `temper-version` in any of its tables: a row
  of the boundary test's direction rule fails one that does
  (`tests/boundary/tests/direction.rs`). A binary's package's own
  tests, which Cargo builds against every dependency of their package,
  recompile with it, as `temper-version`'s own do; nothing they link
  does.

## 4. What the machine must have

- **Programs:** git 2.41 or later (`protocol/configuration/README.md`, 3 and
  6; `protocol/git.md`, 3); `/usr/bin/rg` and `/bin/sh`, at those paths,
  which the agents' machine tools run; `claude`, Claude Code, only for
  Claude (`protocol/credentials.md`, 3).
- **CA certificates:** the system's, or the file `SSL_CERT_FILE` names,
  for every HTTPS client of temper's alike, the fetch's and the
  providers' and the forges', since no roots are embedded
  (`protocol/setup/owned.md`, 6).
- **Fetched by the box:** Forgejo and its runner, at the versions and
  digests skein pins, as `protocol/setup/owned.md` 6 fetches them (an
  answer of 200 alone, no redirect followed, or a mirror's address
  given), or given by absolute path.
- **Checked at the start, as the configuration needs them:** `temper
  box` checks what the configuration it runs needs now, before it
  starts anything: git, `/usr/bin/rg` and `/bin/sh` where a worker runs
  agents. A missing or too old one stops it with one line naming the
  program, the version found and the one needed; where
  `protocol/configuration/README.md` 3 and 6 let a deployment run with a
  warning, the warning stands. Forgejo's binaries are checked when a
  set up fetches them, and `claude` when a program's source is declared.
  Nothing is installed by temper.
- **A browser** for a person's sign-ins, on the machine or through a
  forwarded port; a held record or a key needs none
  (`../help/box.md`, 5).

## 5. A release

- **`scripts/release.sh VERSION`,** run at the tag `vVERSION`, on a
  clean tree whose `Cargo.toml` says VERSION:
  1. **The build root:** Ubuntu 24.04's minimal root file system, its
     published image pinned by digest in the script, unpacked into the
     script's scratch directory, with the build's packages (a C
     compiler for `ring`, git) installed into it from the
     distribution's archive, and the Rust toolchain the repository's
     `rust-toolchain.toml` pins, with its `wasm32-unknown-unknown`
     target, installed into it by rustup.
  2. **The build:** entered with bubblewrap alone (`bwrap`, user
     namespaces, the network shared for the archives and crates), the
     checkout bound read-only, a target directory of the root's own;
     `TEMPER_INTERNAL_BUNDLE=embed cargo build --release --locked -p temper`,
     stripped.
  3. **The checks:** no `GLIBC_` symbol version newer than 2.39
     (`objdump -T`); `--version` printing VERSION and the tag's commit.
  4. **The files:** the binary and its checksum, written into the
     script's output directory. It publishes nothing.
- **Published at the tag** among temper's releases, where
  `draft/first-release.md` 9 puts them: git.fabrig.ai's, mirrored to
  GitHub. The site links them.
- **Reproducible enough to check:** the same tag, root digest and
  toolchain give the same checksum; a rebuild that differs is a
  finding.

### 5.1 Apple Silicon's file

The site's install line downloads
`https://fabrig.ai/temper/$(uname -s)-$(uname -m)`, for x86-64 Linux and
Apple Silicon. Apple Silicon's, `Darwin-arm64`, needs a Mac to build:
GitHub's hosted `macos-26` runners, free on the public repository, the
ones `testing/README.md`, 2.8 runs on. It is the site's file, not a
release's: no tag names it, and the workflow writes no checksum.

- **One workflow,** `.github/workflows/release-macos.yml`, beside
  `macos.yml`: started by a dispatch alone (`workflow_dispatch`; no
  schedule, no push, no pull request), at the commit it is started on.
  One run at a time; its only permission is `contents: read`, and it
  uses no secret and writes nothing to the repository.
- **Its toolchain:** the one the repository's `rust-toolchain.toml`
  pins (`skein/docs/foundation/programming-model.md`, 10.1), installed
  as `macos.yml` installs it: rustup reads the file in the checkout,
  and no version is written in the workflow. The file names the
  `wasm32-unknown-unknown` target the browser's bundle needs
  (`client/web/README.md`, 2.6). Its actions
  are pinned by full commit, the version in a comment, as `macos.yml`'s
  are.
- **Its cache:** the registry may be restored by `macos.yml`'s keys,
  and nothing is saved; the release's target is never cached, so each
  file is built from the commit and the registry's sources alone.
- **The build:** `TEMPER_INTERNAL_BUNDLE=embed cargo build --release --locked -p
  temper --bin temper`, for the runner's own target,
  `aarch64-apple-darwin`, under the release profile exactly as the
  workspace's `Cargo.toml` has it: the workflow sets no profile, flag
  or feature of its own, and builds nothing else.
- **Stripped on the runner:** after the build, Xcode's `strip` takes
  the symbols out of `target/release/temper`, in place. Stripping
  invalidates the linker's ad hoc signature, without which Apple
  Silicon runs no file, so `codesign --force --sign -` signs it again,
  ad hoc, and `codesign --verify` checks the new signature; a strip,
  signature or verification that fails uploads nothing. Only Apple's
  tools remake the signature, so the file is stripped here, where it
  is built, and not on the host that publishes it. The build before
  it keeps the release profile as it is, which sets no `strip`.
- **Its signature is ad hoc,** the one `codesign` makes after the
  strip, which is all Apple Silicon needs to run a file. No Developer
  ID, no notarization: curl marks nothing it downloads as quarantined,
  so Gatekeeper never asks (section 7).
- **Its checks, before the upload,** on the stripped file: `file`
  names it a Mach-O 64-bit arm64 executable, and `temper --help` exits
  0 with its usage line first (`usage: temper COMMAND [ARGUMENT...]`),
  which also shows that Apple Silicon runs it under its new signature.
  A failed check annotates the run with what it found and uploads
  nothing.
- **What it hands over:** one artifact of the run,
  `temper-Darwin-arm64`, kept 7 days, holding two files at its root:
  `temper`, the stripped and signed binary, and `commit`, the full SHA
  of the commit built, one line. GitHub's artifact is a zip, which
  keeps no file mode. `commit` names the build beside the file, as
  section 3's `--version` does within it.
- **Where it goes is the site's:** the host that serves fabrig.ai
  fetches the artifact and serves `temper` at `/temper/Darwin-arm64`,
  gzipped and with its checksum, both made there, and starts the
  workflow when it wants a newer file; all of it happens outside the
  repository.

## 6. Proved on a fresh machine

- **The walk is rung A's** (`draft/v0.1.0.md`, 2.1), its story's steps
  (`../development/acceptance.md`, section 5) from the help's install
  step, with the downloaded file in place of a build: downloaded, its checksum checked, installed as the
  help's download path says, then the help followed as written. The
  walk `box`'s install step gains that path beside the checkout's once
  a release's download exists (`protocol/documentation.md`).
- **A fresh machine** is section 5's root of the floor's distribution
  with only section 4's programs installed (no Rust, no checkout),
  entered with bubblewrap as a user with an empty home, on the
  machine's kernel, its network shared; the held login's resting
  directory is bound in at its own path, so that the walk's import and
  export use the path the login rests at. Anything the walk must add
  beyond section 4 is a finding against the help or this document.
- **The file walked** is the one the release script built, until the
  tag; at the tag, the published one, fetched by its address.

## 7. Open questions

- **A static build:** a musl target, once skein's shell spawns without
  glibc's extensions. Default: glibc, its floor said.
- **arm64 Linux:** Forgejo's pins cover it; a second file per release.
  Default: x86-64 alone for v0.1.0.
- **Signing:** a signature beside the checksum. Default: the checksum
  alone, from the release's page.
- **Apple Silicon at a release:** 5.1's file among a tag's files, with
  its checksum, and a Developer ID signature and notarization for a
  file a browser downloads. Default: the site's file alone, signed ad
  hoc on the runner after its strip.
- **The client and `temper ctl` showing the version** of the
  deployment they reach, which may differ from their own binary's: a
  field of the deployment's document. Default: `--version` alone.
