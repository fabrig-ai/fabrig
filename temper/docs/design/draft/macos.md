# temper on macOS

Draft, 2026-10-22. What it takes for temper to build, pass its suites
and run its first-run demo on macOS. It records a session with the user
on 2026-10-22, which started from a survey of what stands between temper
and macOS (`~/src/rust/tmp/macos-survey-2026-10-22.md`, every claim
with its file and line). It crosses skein, smith, jig and temper on
purpose; an owner splits it.

macOS is a person's own machine: development, and a first look at
temper. Linux stays the platform of record: `land`'s gate, deployments,
and everything only Linux can give (containment, trees, io_uring).

## 1. In one page

- **Three stages** (section 2). First, temper and the kits it builds on
  compile on GitHub's macOS runners and their suites pass there. Then,
  best effort: `temper box`'s first-run demo, with its fake LLM and fake
  Forgejo, driven in a browser on the same runners. Then a person tries
  it on a real Mac.
- **The bulk is skein's readiness backend** (skein's `shell.md`,
  section 7): one backend for hosts without io_uring, kqueue's on macOS
  and epoll's on Linux. Step code never knows which backend runs; no
  step crate changes (section 3).
- **The floor is macOS 15, on Apple Silicon** (`aarch64-apple-darwin`):
  the first with `O_RESOLVE_BENEATH`, which an open beneath a root
  needs. The runners are `macos-26`'s, arm64 (3.4).
- **Above skein, a handful of Linux habits:** fixed tool paths in
  smith, `/proc` and `/etc/passwd` in temper, and their tests' `/proc`
  observations (section 4).
- **No owned Forgejo on macOS.** `temper init` writes none into the
  default document there, and `temper forge fetch` refuses (4.3).
- **CI is GitHub Actions,** a macOS workflow in each of the four public
  repositories, run after each push to `main`. A red run blocks nothing:
  `land`'s gate stays Linux's (section 5).
- **What macOS cannot give stays Linux-only, by name:** each repository
  lists its Linux-only tests in one place, each with its reason. None is
  skipped silently (5.2).

## 2. The stages

### 2.1 Built and green on the runners

- **skein, smith, jig and temper compile for `aarch64-apple-darwin`,**
  every crate, test targets among them (clippy's `--all-targets`).
- **Their gates' suites pass on `macos-26`:** the default and fuzzy
  profiles, and the browser tier where the gate runs one, less each
  repository's Linux-only list (5.2).
- **The conformance suite passes against the readiness backend** on
  both waits: epoll's on our Linux machines, in skein's gate, and
  kqueue's on the runner (3.6).

### 2.2 The demo in a browser

- **temper's browser tier already drives the demo:** `Tier::demo` runs
  `temper box` on a scratch deployment with its own fake LLM, fake
  Forgejo and seed (`tests/client-browser/src/lib.rs`), and the task
  pages' and chat's stories run through it. On the runner it runs as on
  Linux, against Chrome for Testing 147, the Chromium our machines run.
- **One story walks the first run whole** (`first-run.md`, 2.1): the
  link signs in, the demo project's chat takes the suggested request,
  the demo model plays its script, and the change goes through temper's
  procedure on the demo forge. If no test walks it whole yet, temper
  gains one, for both platforms.
- **Best effort:** it shows the basic story works on macOS. It does not
  hold macOS to the process and containment tiers.

### 2.3 A real Mac

- **A checklist for a person with a Mac,** in temper's docs: install,
  `temper box`, the first run in their browser, then a real Claude or
  Codex account added from the accounts page.
- **What only a real Mac answers:** Claude's login in the Keychain
  (jig's `domain/providers.md`, 3.3), a browser that is the
  person's own, and an APFS volume that ignores case (4.4).

## 3. skein: the readiness backend

### 3.1 One backend, two waits

- **It emulates completions** (`shell.md`, section 7): it waits for
  readiness, makes the call without blocking, and completes the record.
  Regular files are always ready: their operations run at submit, as
  the ring's synchronous ones do.
- **kqueue's on macOS, epoll's on Linux.** The waits differ only in the
  wait call. Each operating system has its own parts beside it (3.2).
- **Why epoll too.** None of our machines is a Mac, and a runner is
  minutes away. With epoll, the backend's shared part (its pending
  table, retries, partial transfers, cancels, closes) is built and held
  to the conformance suite on our machines, in skein's gate; the runner
  then proves only what is Darwin's. It also serves the Linux hosts
  section 7 names: Docker's default seccomp profile, and `io_uring`
  disabled.
- **Chosen at startup** (`shell.md`, 4): Linux takes the ring, and the
  readiness backend when the probe finds no ring; macOS always takes the
  readiness backend. A setting forces it on Linux, for its tests.

### 3.2 What each operating system does its own way

| Part | Linux | macOS |
|---|---|---|
| The wait | `epoll_wait` | `kevent` |
| Spawn | `pidfd_spawn` | `posix_spawn`: the same file actions, process group, and session for a terminal |
| A child's exit | its pidfd, readable | `EVFILT_PROC` and `waitid(WNOWAIT)` |
| Signals to the service | `signalfd` | `EVFILT_SIGNAL` |
| Signalling a child | `pidfd_send_signal` | `kill`, `killpg` |
| Open beneath a root | `openat2`, `RESOLVE_BENEATH` | `openat`, `O_RESOLVE_BENEATH`, `O_NOFOLLOW_ANY` |
| Listing a directory | `getdents64` | the owner's call (section 7) |
| Rename, never replacing | `renameat2`, `RENAME_NOREPLACE` | `renameatx_np`, `RENAME_EXCL` |
| Stat | `statx` | `fstat` |
| The seed | `getrandom` | `getentropy` |
| Usage's resident size | KiB | bytes |

- **A child stays unreaped until io closes it,** on macOS as with
  pidfds, so its pid is never reused while io can still signal it.
- **Only the shell's own direct children are watched:** there is no
  subreaper on macOS.

### 3.3 What is weaker without the ring

- **A stalled filesystem holds the loop.** File operations run at
  submit, so an NFS or FUSE mount that stalls stalls the service, and a
  cancel of a file operation always comes too late. `kernel.md` says so
  beside its cancellable file operations.
- **Processes are groups, as they are today:** a descendant that leaves
  its group escapes the close's kill. A cgroup tree, when there is one
  (`kernel.md`, 6.6), is refused on macOS as unsupported.

### 3.4 The floor

- **macOS 15 (Darwin 24),** the first with `O_RESOLVE_BENEATH`.
  `O_NOFOLLOW_ANY` came earlier. As the kernel's floor on Linux,
  confirmed by a probe at startup that refuses below it; no feature
  checks.
- **Apple Silicon only.** Intel Macs are not a target.
- **The runners are `macos-26`, arm64:** generally available since
  2026-02. `macos-15` is added only if a person's Mac needs it.

### 3.5 The harness

- **The simulator's worlds build without the real machine.**
  `skein-world` today includes the real loop, the ring and the tree
  keeper unconditionally, so on macOS not even a simulated world
  compiles. The owner splits it.
- **The tree keeper gains a method on macOS, Group:** the binary in a
  process group of its own; settlement is the group empty after its
  kill. It cannot see a descendant that leaves the group, nor give a
  tree's peak memory, so the tests that need those stay Linux's.

### 3.6 The conformance suite

- **It runs against the readiness backend,** each scenario once, on each
  wait (`kernel.md`, section 8).
- **What is Linux's is said so.** Some scenarios encode Linux's
  behaviour: a full listen queue, the order of refusals, errno values.
  Each says what every backend must do and what is the operating
  system's; none is loosened silently.

## 4. Above skein

### 4.1 smith

- **Tools are found, not assumed:** `/usr/bin/rg` and `/usr/bin/git` are
  fixed paths today (`smith-protocol-machine/src/process.rs`,
  `smith-local-protocol/src/git_child.rs`). rg is never there on a Mac.
- **Its tests' addresses:** the agent-process fixtures use `127.1.*`,
  which macOS doesn't route without an alias; they take `127.0.0.1` and
  their own ports.
- **Its tests' `/proc` observations** (threads, children) become the
  shell's, or join the Linux-only list.

### 4.2 jig

- **No production change.** Its tests' `/proc` uses follow smith's.

### 4.3 temper

- **`serve --box` checks its pipes** through `/proc/self/fd`; it asks
  the shell instead.
- **The login comes from `/etc/passwd`,** where macOS's users are not;
  the shell's `getpwuid` answers on both.
- **Errors print Linux's errno numbers;** they print names.
- **No owned Forgejo on macOS.** `temper forge fetch` fetches Forgejo's
  and its runner's pinned Linux releases; on macOS they would come from
  Homebrew's builds, if at all. There, `temper init` writes no owned
  forge into the default document, so the client offers no Set up a
  Forgejo here (`first-run.md`, 3.8); a document that names one, and
  `temper forge fetch`, refuse naming the platform. The platform is
  decided by the binary's shell, at init, never by step code.

### 4.4 Every platform

- **XDG's directories on macOS too:** `~/.config`, `~/.local/share`,
  `~/.cache`. One story on both; no `~/Library`.
- **Trust roots from the bundle file:** macOS has `/etc/ssl/cert.pem`.
  Roots added to the Keychain are not read.
- **Names that differ only in case** are one directory on a default
  APFS volume. Default: refused on every platform where a checkout's
  name is checked (section 7).

## 5. Tests and CI

### 5.1 The workflows

- **One per repository,** `.github/workflows/macos.yml`: on `macos-26`,
  on each push to `main` and by hand. It runs what the gate runs:
  formatting, clippy over every target, the default and fuzzy profiles,
  and the bundle and browser tier where the gate has them.
- **What it installs:** Rust 1.95, the workspace's `rust-version`;
  `cargo-nextest`; the `wasm32-unknown-unknown` target; Chrome for
  Testing 147, given by `SKEIN_TEST_CHROMIUM`.
- **Upstream by git:** the git dependencies are public, so a fetch needs
  no token. The `fabrig-dev` addresses reach `fabrig-ai` through
  GitHub's redirect, or an `insteadOf` in the workflow.
- **A red run blocks nothing.** It goes to the director, as an
  after-landing failure does (`owners.md`, 4.5), who gives it to an
  owner.
- **Before the runner,** an implementer checks that Darwin compiles
  (`cargo check --target aarch64-apple-darwin`) where rustup can add the
  target.

### 5.2 Linux-only, by name

- **One list per repository,** in its nextest configuration or as
  `cfg(target_os = "linux")` on whole test binaries (the owner's call),
  each entry with its reason.
- **What it holds, known now:** skein's ring and ring-conformance tests,
  the tree keeper's cgroup and subreaper tests; temper's `unshare`
  adversary and its cgroup cluster; the Python lab (bwrap, systemd), and
  the after-landing tiers, which the workflows don't run.

## 6. Order

1. **skein:** the readiness backend on epoll, under the conformance
   suite on our machines; the shell and the harness split by platform;
   then kqueue's and Darwin's parts, with skein's workflow, until
   skein's suites pass on the runner (stage 2.1, for skein).
2. **smith and jig,** on that skein: their habits (4.1, 4.2), their
   workflows, their Linux-only lists.
3. **temper:** its habits and the owned forge off (4.3), its workflow,
   the demo's story through the browser (stage 2.2).
4. **A real Mac:** the checklist, with a person (stage 2.3).

## 7. Open questions

- **A path held without following its last link** (`PathNoFollow`):
  Linux opens it `O_PATH | O_NOFOLLOW`; macOS has no `O_PATH`. Default:
  the owner's design in `shell.md`, from `O_SYMLINK` and `O_EVTONLY`.
- **A directory listing's position on macOS:** `getdirentries`'s
  cookies or `readdir`'s positions. Default: the owner's call, held to
  the same conformance.
- **Names that differ only in case:** refused everywhere, refused on
  macOS only, or a case-sensitive volume required. Default: refused
  everywhere, since the check is cheap and the same on both.
- **Claude's Keychain and several accounts:** whether
  `CLAUDE_CONFIG_DIR` keeps two logins apart. Default: answered on a
  real Mac (2.3); until then, one Claude account per macOS user.
- **A launchd agent for the box,** as `protocol/setup.md`, 7.4
  arranges systemd. Default: later.
- **An owned Forgejo on macOS:** Homebrew's builds, or a Linux VM.
  Default: not planned.
