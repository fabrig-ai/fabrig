# The download

Provisional, 2026-10-26. How temper reaches a person who did not build
it: the one Linux binary of a release, its identity and checksum, what
the machine must have, how a release is made, and the walk that proves
it on a fresh machine (`draft/v0.1.0.md`, 5 and 9). The site that links
it is `draft/first-release.md`'s, the user's; macOS is
`draft/macos.md`'s. What is still open is section 7.

## 1. In one page

- **One file a release:** `temper-VERSION-x86_64-linux`, the whole
  product, its browser bundle embedded, with
  `temper-VERSION-x86_64-linux.sha256` beside it (section 2).
- **It says what it is:** `temper --version` prints the version and the
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

## 2. The file

- **Its target:** `x86_64-unknown-linux-gnu`, built with
  `TEMPER_BUNDLE=embed` and the release profile, stripped; one file,
  nothing beside it at run time but section 4's programs.
- **Its floors:** Linux 6.12, skein's (skein's `shell.md`, sections 4
  and 7); glibc 2.39, the oldest that has the process calls skein's
  shell uses (`pidfd_spawn`, `pidfd_getpid`,
  `posix_spawnattr_setcgroup_np`). Built against a newer glibc, it
  would require that one: so the release builds in a root of the floor's
  own glibc (section 5), and its versioned symbols are checked against
  2.39 before it is published. That 2.39 is the newest of the
  `GLIBC_` symbol versions those calls carry, as the first release
  build's own symbols show; a newer one found there moves the floor
  here.
- **Not static, for now:** skein's shell uses glibc's extensions, so a
  musl build needs a spawn of its own first (section 7).
- **Its checksum:** SHA-256, in the form `sha256sum` checks, one line
  naming the file.

## 3. Its identity

- **`temper --version`** (and `temper version`) prints one line,
  `temper VERSION (COMMIT)`, before any configuration is read, exit 0.
- **VERSION** is the `temper` crate's version, the workspace's in
  `Cargo.toml`, `0.1.0` at v0.1.0's tag and `0.0.0` between releases;
  **COMMIT** the commit it was built from, twelve hex digits, with
  `-dirty` when the tree had changes, `unknown` when built outside git:
  set by the `temper` crate's build script from git at build time,
  which reruns when `HEAD` or the index moves, and held in one place of
  the crate's own that every command reads.

## 4. What the machine must have

- **Programs:** git 2.41 or later (`protocol/configuration.md`, 4 and
  8; `protocol/git.md`, 3); `/usr/bin/rg` and `/bin/sh`, at those paths,
  which the agents' machine tools run; `claude`, Claude Code, only for
  Claude (`protocol/credentials.md`, 3).
- **CA certificates:** the system's, or the file `SSL_CERT_FILE` names,
  for every HTTPS client of temper's alike, the fetch's and the
  providers' and the forges', since no roots are embedded
  (`protocol/setup.md`, 6.6).
- **Fetched by the box:** Forgejo and its runner, at the versions and
  digests skein pins, as `protocol/setup.md` 6.6 fetches them (an
  answer of 200 alone, no redirect followed, or a mirror's address
  given), or given by absolute path.
- **Checked at the start, as the configuration needs them:** `temper
  box` checks what the configuration it runs needs now, before it
  starts anything: git, `/usr/bin/rg` and `/bin/sh` where a worker runs
  agents. A missing or too old one stops it with one line naming the
  program, the version found and the one needed; where
  `protocol/configuration.md` 4 and 8 let a deployment run with a
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
     `TEMPER_BUNDLE=embed cargo build --release --locked -p temper`,
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

## 6. Proved on a fresh machine

- **The walk is rung A's** (`draft/v0.1.0.md`, 2.1), its brief's
  scenario from the help's install step, with the downloaded file in
  place of a build: downloaded, its checksum checked, installed as the
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
- **The client and `temper ctl` showing the version** of the
  deployment they reach, which may differ from their own binary's: a
  field of the deployment's document. Default: `--version` alone.
