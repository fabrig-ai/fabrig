# temper on macOS

Draft, 2026-10-22, revised 2026-11-04 with the user, who gave it an
owned Forgejo and one story to pass first. What it takes for temper to
build, run a change from a chat to its landing, and pass its suites on
macOS. It records a session with the user on 2026-10-22, which started
from a survey of what stands between temper and macOS (every claim with
its file and line). It crosses skein, smith, jig and temper on purpose;
an owner splits it.

macOS is a person's own machine: development, and a first look at
temper. Linux stays the platform of record: the gate, deployments,
and everything only Linux can give (containment, trees, io_uring).

The user's rule for all of it (2026-11-04): keep it cheap. What matters
is that 2.4's story basically works. Whatever is Linux's may be degraded
or less sophisticated on macOS, and rough edges are fine.

## 1. In one page

- **One story first** (2.4): on GitHub's macOS runners, a box with its
  owned Forgejo and a fake LLM, driven through `temper ctl`: a change is
  asked for in a chat, Forgejo runs its CI, temper lands it. That is the
  quick try `v0.1.0.md` names.
- **That story is the acceptance.** Nothing else here holds it up:
  where macOS differs, it gets the simplest thing that lets the story
  pass, a degraded behaviour or a Linux-only entry, never a port for its
  own sake.
- **Then the rest, as it stays cheap** (section 2): the suites, tree by
  tree (2.1); a person on a real Mac (2.3); the first-run demo in a
  browser, best effort (2.2).
- **The bulk is skein's readiness backend** (skein's `runtime/readiness.md`
  ): one backend for hosts without io_uring, kqueue's on macOS
  and epoll's on Linux. Step code never knows which backend runs; no
  step crate changes (`macos.md`, section 3).
- **The floor is macOS 15, on Apple Silicon** (`aarch64-apple-darwin`):
  the first with `O_RESOLVE_BENEATH`, which an open beneath a root
  needs. The runners are `macos-26`'s, arm64 (3.4).
- **Above skein, a handful of Linux habits:** fixed tool paths in
  smith, `/proc` and `/etc/passwd` in temper, and their tests' `/proc`
  observations (section 4).
- **An owned Forgejo on macOS, Homebrew only as a URL** (4.3): `temper
  forge fetch` takes Forgejo and its runner from Homebrew's bottles, at
  addresses fixed by the pins' digests. No `brew`, nothing under
  `/opt/homebrew`; the rest is temper's, as on Linux.
- **CI is GitHub Actions,** a macOS workflow in the repository, run on
  the newest `main` a few times a day. A red run blocks nothing: the
  gate stays Linux's (section 5).
- **What macOS cannot give stays Linux-only, by name:** each tree
  lists its Linux-only tests in one place, each with its reason. None is
  skipped silently (5.2).

## 2. The stages

### 2.1 Built and green on the runners

- **skein, smith, jig and temper compile for `aarch64-apple-darwin`,**
  every crate, test targets among them (clippy's `--all-targets`).
- **Their gates' suites pass on `macos-26`:** the default and fuzzy
  profiles, and the browser tier where the gate runs one, less each
  tree's Linux-only list (5.2).
- **The conformance suite passes against the readiness backend** on
  both waits: epoll's on our Linux machines, in skein's gate, and
  kqueue's on the runner (3.6).

### 2.2 The demo in a browser

- **temper's browser tier already drives the demo:** `Tier::demo` runs
  `temper box` on a scratch deployment with its own fake LLM, fake
  Forgejo and seed (`tests/client-browser/src/lib.rs`), and the task
  pages' and chat's stories run through it. On the runner it runs as on
  Linux, against Chrome for Testing 147, the Chromium our machines run.
- **One story walks the first run whole** (`archive/first-run.md`, 2.1): the
  link signs in, the demo project's chat takes the suggested request,
  the demo model plays its script, and the change goes through temper's
  procedure on the demo forge. If no test walks it whole yet, temper
  gains one, for both platforms.
- **Best effort, after 2.4 and 2.1:** it shows the person's own way
  through works on macOS. 2.4 holds the mechanisms it relies on.

### 2.3 A real Mac

- **A checklist for a person with a Mac,** in temper's docs: install,
  `temper box`, the first run in their browser, then a real Claude or
  Codex account added from the accounts page.
- **What only a real Mac answers:** Claude's login in the Keychain
  (jig's `domain/providers.md`, 3.3), a browser that is the
  person's own, an APFS volume that ignores case (4.4), and a machine
  whose `git` is Apple's shim until the Command Line Tools are in.

### 2.4 One story on the runner

- **The story:** a box on a scratch deployment, with its owned Forgejo
  and its runner, the model the fake LLM. Through `temper ctl`, a person
  signs in, sets the forge up and asks for a change in a chat; the
  change is pushed, Forgejo's runner runs its CI on the Mac, and temper
  lands it.
- **It is a story temper has:** `temper/bin/temper-box/tests/process.rs`,
  `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart`,
  in the process tier (`../testing/process-tier.md`). macOS runs that
  story, never a copy of it, its agent review and its restart included:
  the restart is a graceful one.
- **What the story's support takes on macOS,** each the simplest that
  works:
  - `parent::Guard`, which is Linux's and which `story_command` builds
    unconditionally: nothing on macOS, or a plain kill of the child;
  - the tools of its CI's steps as macOS's own have them (`base64 -w0`
    is GNU's), within the cleared environments of its children: the
    box's `/usr/bin:/bin`, and the owned runner's configured `PATH`;
  - the story's `left`, Linux's whole-tree proof: under Group (3.5) it
    proves nothing, so on macOS the story holds the box's clean exit
    alone.
- **No containers:** its jobs run on the runner's host, under the
  `temper-host` label, as on Linux. The macOS runners offer no Docker,
  and the story needs none.
- **Its binaries come through `temper forge fetch`** (4.3), on a cache
  of the workflow's own that starts empty, so each run also shows that
  the pinned addresses still answer. `TEMPER_FORGEJO` and
  `TEMPER_FORGEJO_RUNNER` then name what it fetched.
- **It runs alone,** `TEMPER_BUNDLE=none`, with its one selected test
  checked: `cargo nextest run --profile process-temper -p temper --test
  process -E 'test(=a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart)'`.
- **It proves the plumbing, not a person's Mac:** a real account, the
  Keychain and a person's browser stay 2.3's.

## 3. skein: the readiness backend

### 3.1 One backend, two waits

- **It emulates completions** (`skein/docs/design/runtime/readiness.md`): it waits for
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
- **Chosen at startup** (`skein/docs/design/runtime/shell.md`, 4): Linux takes the ring, and the
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
  cancel of a file operation always comes too late. `skein/docs/design/runtime/kernel.md` says so
  beside its cancellable file operations.
- **Processes are groups, as they are today:** a descendant that leaves
  its group escapes the close's kill. A cgroup tree, when there is one
  (`skein/docs/design/runtime/kernel.md`, 6.6), is refused on macOS as unsupported.

### 3.4 The floor

- **macOS 15 (Darwin 24),** the first with `O_RESOLVE_BENEATH`.
  `O_NOFOLLOW_ANY` came earlier. As the kernel's floor on Linux,
  confirmed by a probe at startup that refuses below it; no feature
  checks.
- **Apple Silicon only.** Intel Macs are not a target.
- **The runners are `macos-26`, arm64:** generally available since
  2026-02. `macos-15` is added only if a person's Mac needs it.

### 3.5 The harness

- **The tree keeper's method on macOS is Group:** the binary in a
  process group of its own; settlement is the group empty after its
  kill. It cannot see a descendant that leaves the group, nor give a
  tree's peak memory, so the tests that need those stay Linux's.

### 3.6 The conformance suite

- **It runs against the readiness backend,** each scenario once, on each
  wait (`skein/docs/design/runtime/kernel.md`, section 8).
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
- **An owned Forgejo on macOS.** Neither Forgejo nor its runner
  publishes a darwin build; Homebrew builds both for Apple Silicon and
  keeps them as bottles on `ghcr.io`, each a keg in a tarball. temper
  uses Homebrew as a URL and nothing else: no `brew`, no `/opt/homebrew`.
  The bottles' binaries stand alone (`any_skip_relocation`, no
  dependency beyond macOS's own libraries), and they are Homebrew's
  builds, not signed by Forgejo, pinned by digest as Linux's are.
  Forgejo's listener is as `../protocol/setup/owned.md`, 4 has it on macOS.
- **`temper forge fetch` on macOS** extends `../protocol/setup/owned.md`, 6,
  with each pin's bottle for macOS 15, the floor (3.4), which later
  systems run too; `DIGEST` is the pin's:

  | Binary | From | The archive's member | Into |
  |---|---|---|---|
  | Forgejo | `https://ghcr.io/v2/homebrew/core/forgejo/blobs/sha256:DIGEST` | `forgejo/VERSION/bin/forgejo` | `<cache>/forgejo/VERSION/forgejo` |
  | the runner | `https://ghcr.io/v2/homebrew/core/forgejo-runner/blobs/sha256:DIGEST` | `forgejo-runner/RUNNER_VERSION/bin/forgejo-runner` | `<cache>/forgejo-runner/RUNNER_VERSION/forgejo-runner` |

- **Fixed by the digest, never looked up.** The address is the pin: no
  request to Homebrew's formula API, so what Homebrew moves to changes
  nothing temper fetches.
- **Three things differ from Linux's fetch:**
  - **A token that is no credential:** the first request carries
    `Authorization: Bearer QQ==`, the anonymous value Homebrew itself
    sends, to `ghcr.io` alone.
  - **One redirect:** `ghcr.io` answers with its storage's address. The
    fetch follows one redirect, to `https`, without the token; another
    refuses, naming the URL. Linux's fetch still takes none.
  - **An archive:** the body goes to `<file>.archive`, private, and is
    hashed as it is written, as 6.6 has its part file, against the
    archive's digest, within `setup.fetch_bytes`. Only on a match is
    the one member, by its exact name, taken out by macOS's own
    `/usr/bin/tar` into `<file>.part`, within the same bound, its digest
    checked, made `0o755` and renamed over its name, and the archive
    removed. A wrong digest, a bound passed or a failed extraction
    removes both files and refuses, and an interrupted fetch starts
    again from nothing, as 6.6 has it. A binary already at its name with
    another digest is refused, never overwritten.
- **Two digests per binary on macOS,** in `skein-forge` beside Linux's
  (skein's `forge/README.md`, 4.3): the archive's, which is its address, and
  the binary's, which a file already in the cache is held to. A pin is
  selected by platform, architecture and kind of file: `arm64` alone no
  longer says which.
- **One version on both platforms** (`../protocol/setup/owned.md`, 1): a pin
  moves only to versions with bottles, all its digests together, and
  the version check holds on macOS as on Linux, a bottle's first line
  checked when its pin is taken. The rule that a fetched binary is the
  build the conformance checked, byte for byte, stays Linux's: a bottle
  is the same version and another build. 2.4's story is what checks it,
  and skein's forge conformance (skein's `testing/fakes/forge.md`, section 7)
  joins the workflow with skein's suites.
- **A bottle gone:** Homebrew promises nothing about old blobs. The
  fetch refuses, naming the URL and saying what answers it: a binary
  given by path, or the owned entry's `mirror`, which serves the same
  archive at the same `/v2/homebrew/core/…/blobs/sha256:DIGEST` path,
  under the same digests and the same one redirect, and is sent no
  token. Every workflow run fetches, so a blob's going shows before a
  person meets it (section 7).
- **The rest is as on Linux:** the machine's `git` on the runner's
  `PATH` (6.1's inputs), and an Intel Mac refused, naming its
  architecture. The platform is decided by the binary's shell, never by
  step code.

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

- **One workflow,** `.github/workflows/macos.yml`: on `macos-26`, on
  the newest `main` a few times a day by a schedule, and by hand. Not on
  each push: `main` takes over a hundred landings on a busy day.
- **It grows with the stages** (section 6): every target compiled, then
  2.4's story, then each tree's suites as they pass. When they all do it
  runs what the gate runs: formatting, clippy over every target, the
  default and fuzzy profiles, and the bundle and browser tier where the
  gate has them.
- **What it installs:** Rust 1.95, the workspace's `rust-version`;
  `cargo-nextest`; the `wasm32-unknown-unknown` target and, for the
  first browser suite, Chrome for Testing 147, given by
  `SKEIN_TEST_CHROMIUM`.
- **The runner is small:** 3 cores, 7 GB of memory, 14 GB of disk by
  GitHub's table. Line tables only for debug information, the
  dependencies' build cached by the lock, and each run reports its disk,
  so that what no longer fits shows as a number before it fails.
- **Its own budgets:** a `macos` nextest profile, measured there, since
  a test takes about twice as long as on our machines.
- **A failure reads without a login:** the repository is public and
  each failing check annotates its run, so its failing tests and the
  end of their output come through the public API.
- **A red run blocks nothing.** It is a finding, triaged as a red
  after landing is (`../domain/after-landing.md`, section 7).
- **Before the runner,** Darwin's compile is checked on Linux, as
  skein's `runtime/shell.md`, section 8 has it: clippy for
  `aarch64-apple-darwin`, the members that reach rustls left to the
  runner.

### 5.2 Linux-only, by name

- **One list per tree,** each entry with its reason; skein's is
  `runtime/shell.md`, section 8, and the others follow it. Code Darwin cannot
  compile takes `cfg(target_os = "linux")` on its binary, module or
  support, before the workflow's first run; a test that compiles and
  cannot run may instead go in the tree's nextest configuration.
- **What it holds, known now:** skein's ring and ring-conformance tests,
  the tree keeper's cgroup and subreaper tests; temper's `unshare`
  adversary and its cgroup cluster; and the after-landing tiers, which
  the workflow doesn't run, 2.4's story alone apart.

## 6. Order

1. **The pins:** Forgejo and its runner at versions with bottles, their
   digests for macOS beside Linux's (4.3): today 17.0.1 and 13.2.1,
   from 17.0.0 and 13.2.0.
2. **It compiles:** the workflow, from skein's former one (in this
   repository's history at 44c5be34fa), compiling every target of every
   tree; each tree's first Linux-only entries (5.2) and what above skein
   stops a compile.
3. **The fetch** (4.3).
4. **What the story needs to run,** and no more of section 4 than that:
   the habits it reaches, compiling or not: temper's pipe check without
   `/proc/self/fd`, its login (4.3), smith's tools (4.1), the trust
   roots and the directories (4.4), and its own support (2.4). The pipe
   check may simply be Linux's.
5. **The story** (2.4) green on the runner, the workflow's first check
   that runs anything.
The story green is the acceptance. What follows comes as it stays
cheap, and a test that would take real work goes on its tree's
Linux-only list:

6. **The suites,** tree by tree, each with its Linux-only list (5.2):
   skein's, with the `macos` profile again and the throwaway Forgejo
   off its list; smith's; jig's; temper's (2.1).
7. **A real Mac:** the checklist, with a person (2.3).
8. **The demo** through the browser (2.2).

## 7. Open questions

- **A path held without following its last link** (`PathNoFollow`):
  Linux opens it `O_PATH | O_NOFOLLOW`; macOS has no `O_PATH`. Default:
  the owner's design in `skein/docs/design/runtime/shell.md`, from `O_SYMLINK` and `O_EVTONLY`.
- **A directory listing's position on macOS:** `getdirentries`'s
  cookies or `readdir`'s positions. Default: the owner's call, held to
  the same conformance.
- **Names that differ only in case:** refused everywhere, refused on
  macOS only, or a case-sensitive volume required. Default: refused
  everywhere, since the check is cheap and the same on both.
- **Claude's Keychain and several accounts:** whether
  `CLAUDE_CONFIG_DIR` keeps two logins apart. Default: answered on a
  real Mac (2.3); until then, one Claude account per macOS user.
- **A launchd agent for the box,** as `../protocol/setup/box.md`, 4
  arranges systemd. Default: later.
- **A pinned bottle gone from `ghcr.io`,** while a released temper
  still pins it (4.3). Default: the refusal, `mirror` and a binary given
  by path, and a copy of our own only if it happens: a copy we host is
  GPL code we distribute, its source to keep beside it.
