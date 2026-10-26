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
- **What the machine must have** is a short list the guide gives and
  `temper box` checks at its start, naming what is missing: git 2.41 or
  later, ripgrep, a POSIX shell and the system's CA certificates;
  Forgejo and its runner are fetched by the box, pinned; Claude Code
  only for Claude (section 4).
- **A release is made by a script and published by a person:**
  `scripts/release.sh VERSION` builds the file at the tag, in a
  build root of the glibc floor, writes the checksum and prints the
  command that publishes both; the publishing is the user's (section 5).
- **Proved on a fresh machine:** rung A walked from the downloaded file,
  by the guide alone, on a machine that has only what section 4 lists
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
  2.39 before it is published.
- **Not static, for now:** skein's shell uses glibc's extensions, so a
  musl build needs a spawn of its own first (section 7).
- **Its checksum:** SHA-256, in the form `sha256sum` checks, one line
  naming the file.

## 3. Its identity

- **`temper --version`** (and `temper version`) prints one line,
  `temper VERSION (COMMIT)`, before any configuration is read, exit 0.
- **VERSION** is the workspace's version in `Cargo.toml`, `0.1.0` at
  v0.1.0's tag and `0.0.0` between releases; **COMMIT** the commit it
  was built from, twelve hex digits, taken at build time, with `-dirty`
  when the tree had changes; `unknown` when built outside git.
- **The client and `temper ctl`** show the same line where they show
  the deployment, so that a report names what ran.

## 4. What the machine must have

- **Programs:** git 2.41 or later (`protocol/configuration.md`, 4 and
  8; `protocol/git.md`, 3); ripgrep and a POSIX shell, which the
  agents' tools run (smith's machine tools); the system's CA
  certificates, or `SSL_CERT_FILE`, for every HTTPS call, since no roots
  are embedded (`protocol/setup.md`, 6.6); `claude`, Claude Code, only
  for Claude (`protocol/credentials.md`, 3).
- **Fetched by the box:** Forgejo and its runner, at the versions and
  digests skein pins, from their releases (`protocol/setup.md`, 6.1 and
  6.6), or given by absolute path.
- **Checked at the start:** `temper box` checks each program it will
  need before it starts anything, and a missing or too old one stops it
  with one line naming the program, the version found and the one
  needed. Nothing is installed by temper.
- **A browser** for a person's sign-ins, on the machine or through a
  forwarded port; a held record or a key needs none
  (`../guide/box.md`, 5).

## 5. A release

- **`scripts/release.sh VERSION`,** run at the tag `vVERSION`, on a
  clean tree whose `Cargo.toml` says VERSION: builds the file in a build
  root of the glibc floor's distribution (Ubuntu 24.04's), so that its
  symbols need no newer glibc; checks them; runs `--version` and
  compares; writes the checksum; and prints the command that publishes
  the two files. It publishes nothing itself.
- **Publishing is the user's:** the files go among the releases of
  temper's public repository, at the tag, by the user or by an agent
  the user has asked, with the user's credential. The site links them.
- **Reproducible enough to check:** the same tag and build root give
  the same checksum; a release whose rebuild differs is a finding.

## 6. Proved on a fresh machine

- **The walk is rung A's** (`draft/v0.1.0.md`, 2.1), its brief's
  scenario from the guide's first step, with the downloaded file in
  place of a build: downloaded, its checksum checked, then the guide
  followed as written.
- **A fresh machine** is a root of the floor's distribution with only
  section 4's programs installed, no Rust, no checkout, a user with an
  empty home, run on the machine's kernel; a container or a new
  virtual machine. Anything the walk must add beyond section 4 is a
  finding against the guide or this document.
- **Before the tag,** the file the release script built stands in for
  the download; at the tag, the published one, fetched by its address.

## 7. Open questions

- **A static build:** a musl target, once skein's shell spawns without
  glibc's extensions. Default: glibc, its floor said.
- **arm64 Linux:** Forgejo's pins cover it; a second file per release.
  Default: x86-64 alone for v0.1.0.
- **Signing:** a signature beside the checksum. Default: the checksum
  alone, from the release's page.
- **Where the releases live:** temper's public repository's releases on
  GitHub, or git.fabrig.ai's (`draft/first-release.md`, 9). Default: the
  user's, at the tag.
