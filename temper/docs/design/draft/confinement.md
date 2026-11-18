# Confining agents' runs

Draft, 2026-11-18, with the user. What a run of an agent may reach on
the machine that hosts it (files, secrets, other processes, the network,
the machine's resources), and how temper bounds each, starting small and
growing. It started from the user's rule: an agent sees only its
workspace, the file system first. It records a session that began from a
survey of what confines a run today (section 2, every claim with its
file). It crosses skein, smith, jig and temper on purpose; an owner
splits it.

Linux is the platform of record. macOS keeps the tools' own checks and
no more (section 9), by the user's rule that macOS stays cheap.

## 1. In one page

- **A run reaches what it was given, and nothing else:** its workspace,
  the machine's programs read-only, the caches named for it and a
  scratch of its own. Not other runs, not its host's state or logins,
  not a person's home, and never more privilege than it started with.
- **Every process of the run, not only its tools.** smith's tools are
  confined to the workspace already (2.1); commands are not: smith's
  shell, a delivery's checks, a build's scripts, a program's `Bash`
  (2.2). So the fence is the kernel's, around the run's whole process
  tree, and the tools' checks stay above it.
- **One policy, a run's reach, as data** (section 4): files, secrets,
  processes, network, resources and authority, set by the worker's
  configuration and the run's start, denying by default. The mechanisms
  beneath it grow; the policy's words do not change.
- **Started small: files, by Landlock** (section 6). Unprivileged,
  inherited by every child, irrevocable by the run, and needing no setup
  of the host; with `no_new_privs`, so nothing in the tree gains
  privilege through a set-user-id program. It bounds what a run reads,
  writes and runs; what it leaves (files' modes and times, sockets named
  by a path, other processes' command lines) is said (6.5), and the
  full view closes it.
- **Grown in steps** (section 5): the worker's own account; files;
  the program's login out of reach; the network; a cgroup per run; the
  full view of skein's contained trees; people apart. Each step leaves
  the system working, and each earlier one stays as a layer beneath the
  next.
- **Tested where it is built** (section 8): the mechanism in skein, held
  to the real kernel by its conformance suite; above it, worlds that
  check every spawn asks for the right reach; a few focused end-to-end
  tests; one fuller story after landing. A refusal is told in words: to
  the LLM as the command's own error, to operators in the run's events.
- **Against mistakes, injected instructions and other people's runs**
  (section 3); not against a kernel exploit, which only machines apart
  answer.

## 2. Where it stands

The code is main's on 2026-11-18. What a deployment adds (the account
its worker runs as and what that account holds) is the deployment's,
and section 2.3 says it in general.

### 2.1 Tools: confined by path

- **smith's own tools** resolve every path against the workspace's
  directories and refuse what leads out, writes outside the writable
  ones, and any `.git` part, case-insensitively
  (`smith/crates/smith-domain-tools/src/authority.rs`;
  `smith/crates/smith-protocol-machine/src/files.rs`). Opens go beneath
  the root by the kernel's `RESOLVE_BENEATH`, so a link that leads out
  is refused, not followed (`skein/crates/skein-shell/src/linux/opens.rs`).
  smith's tools tests prove it (`smith/tests/tools/tests/suite/simulation.rs`,
  `paths_that_lead_out_of_the_checkout_are_outside`).
- **A program backend's built-in tools** pass the permission prompt,
  which checks each path by its names alone (smith's
  `smith/docs/design/protocol/programs.md`, 5.4;
  `smith/crates/smith-protocol-program/src/backend/policy.rs`): a link
  within the workspace that leads out is allowed, as that section says,
  and `Bash` is allowed whole once the run has the shell family.

### 2.2 Commands: not confined

- **smith's shell and checks** start `/bin/sh -c` or the check's
  executable with the configured environment, in a process group of
  their own, as the worker's user, with no namespace, Landlock, seccomp
  filter, rlimit, cgroup or `no_new_privs`
  (`smith/crates/smith-protocol-machine/src/process.rs`;
  `skein/crates/skein-shell/src/linux/process.rs`). smith's design says
  so (`smith/docs/design/domain/tools.md`, 5): without contained trees a
  command has the permissions of the agent's user.
- **A program's `Bash`** is the program's child, unparsed and unwrapped
  (`smith/docs/design/protocol/programs.md`, 5.4).
- **What runs on behalf of a run** is a command like any other: a
  delivery's check, `.temper/pre-pr` in this repository, runs Cargo, and
  Cargo runs every dependency's build script with the run's reach
  (`tools/pre-pr/pre_pr.py`).
- **A process group is no tree:** a descendant that leaves it outlives
  the run's stop (`skein/docs/design/runtime/io/processes.md`, 2).

### 2.3 So a run reaches what its worker's user reaches

- **The worker's own:** its configuration, its data (every workspace of
  every run, the git stores, its secrets directory), the program
  sources' logins, and its caches. A run's command can read the login of
  every program source on its host, which smith's design says outright
  (`smith/docs/design/protocol/programs.md`, 7: "A command can read the
  login").
- **Whatever else the account holds:** a worker run as a person's own
  account gives each run that person's home (keys, other services'
  configuration and credentials), and, where the account may use `sudo`
  without a password, the machine.
- **Its other processes:** the same user may signal them, read their
  environment through `/proc`, and, where Yama's `ptrace_scope` is 0,
  attach to them and read their memory. The worker's git carries a
  forge credential in its environment for the operations that need one
  (`temper/crates/temper-worker-protocol-git/src/invocation.rs`,
  `environment_for`).
- **Shared caches and build outputs:** a compile cache, Cargo's
  registry and every build directory the account writes are written by
  one run and read by the next, and by whatever else builds as that
  account.

### 2.4 The host trusts what the run wrote

The worker's git turns off the system's and the global configuration,
hooks, fsmonitor and credential helpers
(`temper/crates/temper-worker-protocol-git/src/invocation.rs`), but reads
the checkout's own `.git/config`, which a command can write. Git runs
programs that a repository's configuration names (filter and merge
drivers among them), and the worker runs some of its git operations with
a credential in their environment. Most of its operations find the
repository from their working directory
(`temper/crates/temper-worker-protocol-git/src/invocation.rs`), so a
`.git` the run rewrote could also name another directory altogether.
This was not tried. Whatever the reach, the host must not run or trust
what a run wrote outside its delivery (6.3).

### 2.5 Credentials are designed well

A run's environment holds no credential: its LLM calls' credentials
come as grants on its channel (jig's
`jig/docs/design/domain/hosts/README.md`, 5.1; smith's
`smith/docs/design/domain/host.md`, 4 and 7), the worker starts its agent
with an empty environment
(`temper/crates/temper-worker-service/src/worker.rs`), and a program's
environment drops every credential variable smith knows
(`smith/crates/smith-protocol-program/src/source.rs`). The gap is what
lies on disk within the tree's reach (2.3), the program's login first.

### 2.6 Designed, not built

- **skein's contained trees** (`skein/docs/design/runtime/io/processes.md`,
  3): a cgroup of their own, a user namespace, a composed view of the
  file system, loopback alone for the network, a pid namespace,
  `no_new_privs` and lowered rlimits. Their status is *Not yet, the whole
  document*; skein-io's `Spawn` holds a program, its arguments, its
  environment, a root, a directory and its pipes, and nothing more
  (`skein/crates/skein-io/src/kernel.rs`), and the helper that would set
  a tree up (`skein/docs/design/runtime/kernel.md`, 6.6) does not exist.
- **What waits on them:** jig's agent processes (jig's
  `jig/docs/design/protocol/README.md`, section 10), smith's commands
  (`smith/docs/design/domain/tools.md`, 5), hearth's build sandboxes
  (`hearth/docs/design/domain/build/sandboxes.md`), and temper, which
  leaves containment for shared hosts until after standalone
  (`temper/docs/design/protocol/README.md`, 11).
- **The kernels deployments run** already offer what the first steps
  need: Debian 13's 6.12 has Landlock's ABI 6, unprivileged user
  namespaces and cgroup v2.

## 3. What it defends against

- **The model's mistakes:** a command that deletes, rewrites or reads
  what is not its run's, another run's checkout among them. The most
  common, and the cheapest to stop.
- **Instructions injected through what a run reads:** a repository's
  files, an issue's text, a web page, a dependency's build script, each
  asking for a login, a key or a token to be read and sent.
- **One person's run reaching another's,** where a host serves runs paid
  for by several people's accounts: their logins, their workspaces, their
  results.

What it protects, in order of harm: credentials (program logins, keys,
forge tokens); other runs' and people's data; the host (its state, its
programs, its other processes); and what later runs and builds read
(caches and build outputs), since integrity matters as much as secrecy.

Not: a kernel exploit, side channels between runs on one machine, or a
malicious operator. Machines apart answer those (5.7), and nothing here
pretends to.

## 4. A run's reach

One vocabulary, from the worker's configuration to the kernel. A run
reaches what its reach names and nothing else.

| Dimension | What the run is given | Everything else |
|---|---|---|
| files | its workspace's directories, each writable or not as its start says; the machine's programs, libraries and certificates, read-only; named caches, each with its way; a scratch of its own, writable | unreachable |
| secrets | none on disk within reach; its credentials as grants, in memory (2.5) | unreachable |
| processes | its own tree | not signalled, traced or read; no privilege gained |
| network | the agent: its LLM endpoints and its host's channel; its commands: none, or named destinations through a relay | refused |
| resources | bounds on memory, processes and CPU; the tree ended as a whole and proved empty | — |
| authority | the tools it is granted, its host's to serve (jig's grants, unchanged) | refused by the host |

- **Who sets it.** The worker's configuration names the machine's
  paths, the caches and the scratch's parent: they are facts of the
  deployment. The run's start names its workspace, as smith's start
  does now (`smith/docs/design/domain/tools.md`, 2). A charter may only
  narrow it, as a run with no network would.
- **Deny by default.** A path, a destination or a resource not named is
  refused; nothing is allowed because no rule mentions it.
- **Enforced by skein, at the spawn.** A spawn carries the reach, and
  each backend enforces what it can and refuses a spawn it cannot
  enforce, never running it with less (6.5). Mechanism is code; nothing
  in a reach is judged by an LLM.
- **Visible.** The run's start event records the reach it was given. A
  refusal is the command's own error (*permission denied*), which the
  LLM reads as any failure, and a spawn refused for its reach names it
  in the worker's words.

## 5. The steps

Each step is a milestone of its own, in order of harm stopped per cost,
and each leaves the system working.

### 5.1 The worker's own account

A worker runs as a service user of its own, never a person's account,
with no `sudo`, under a unit hardened as an engine's can be
(`NoNewPrivileges`, `ProtectSystem=strict`, `ProtectHome`, `PrivateTmp`,
`ReadWritePaths` its own directories). No code: the deployment's setup
says it, and a deployment follows it. The worker's account belongs to no
group that opens a privileged socket (a container daemon's, for one),
since a socket named by a path stays reachable in the first steps (6.5).
`NoNewPrivileges` covers the whole unit until the full view (5.6) needs
the set-user-id `newuidmap` for a run's subordinate ids, which it
defeats (`skein/docs/design/runtime/kernel.md`, 6.6); the helper then
sets it on each run instead.

- **Stops:** privilege, people's homes and keys, other services' state,
  and builds outside the worker sharing its caches.
- **Leaves:** what the worker itself holds, shared by its runs (2.3's
  first point), which the next steps bound.
- **On a person's own machine,** a box runs as its person, whose
  account is theirs to give; the next step is what bounds a run there.

### 5.2 Files, by Landlock

The first step in code, designed in section 6: the run's tree is
restricted to its reach's files, with `no_new_privs`, and scoped so that
it cannot signal or trace a process outside it.

### 5.3 The program's login out of reach

The program must read and refresh its login, so its domain grants its
configuration directory. Two routes reach the login through it: its
`Bash` commands, which are its children, and its built-in file tools,
which run in its own process and follow a link in the workspace that
leads into that directory, since the permission prompt judges a path by
its names (2.1). Resolving the link at the prompt leaves a race with any
command that rewrites it. The ways, ranked by what they protect:

1. **The login held outside the tree:** the program reaches its
   provider through a relay of the agent's, which adds the credential,
   and nothing of the login is on disk within reach. Whether the program
   runs on a relayed login is unknown.
2. **Both routes closed:** each command in a narrower domain, and the
   program's file tools served by smith.
   - **Commands:** the program's own prefix for the commands it runs
     (Claude Code reads `CLAUDE_CODE_SHELL_PREFIX`, set in its
     environment at each launch, never in the source's shared settings)
     is skein's helper. It takes the command as one string, restricts
     itself to the run's reach less the configuration directory, read
     from a document the agent wrote before the launch, closes every
     descriptor but the three standard ones, and runs the command
     through the shell. Landlock's domains nest, and a child's may only
     be narrower. The prefix also starts the program's MCP servers, the
     agent's own among them, which the narrowing must leave working.
   - **File tools:** smith's own tools, which open beneath the root by
     `RESOLVE_BENEATH` (2.1), served on the agent's MCP server as
     `remove` already is (`smith/docs/design/protocol/programs.md`, 5.3),
     in place of the program's `Read`, `Write`, `Edit`, `Glob` and
     `Grep`.
3. **Commands alone narrowed,** the built-in tools' links left to the
   prompt.
4. **The program's own sandbox for `Bash`,** set in the settings the
   agent writes, with the configuration directory denied.
5. **Accepted on a host that serves one person,** as smith's design
   accepts it now (`smith/docs/design/protocol/programs.md`, 7).

The pick is the second (10.1), if the live suite shows that the prefix
wraps every command the program runs. It costs the model its own file
tools. The first follows if the program can run on a relayed login.

### 5.4 The network

- **Commands:** loopback alone, by a network namespace of their own
  (skein's `Network::Loopback`,
  `skein/docs/design/runtime/io/processes.md`, 3), whose loopback is
  not the host's. Named destinations, a crate registry for one, go
  through a relay the host runs: the spawn hands the namespace a Unix
  socket to it, and a forwarder there listens on the namespace's
  loopback and passes each connection on. Nothing else of the host's
  network is reachable.
- **The agent:** its LLM endpoints and its host's channel.
- Landlock's TCP rules bound ports, not addresses, so they are a
  stopgap at most.

### 5.5 A cgroup per run

Each run's tree in a cgroup of its own beneath the worker's delegated
one, from its first instruction (`CLONE_INTO_CGROUP`): bounded in
memory, processes and CPU, ended whole by `cgroup.kill`, and proved
empty. It also ends programs that outlive a forced stop
(`temper/docs/design/protocol/README.md`, 11). It needs the worker's
unit delegated.

### 5.6 The full view

skein's contained trees as designed
(`skein/docs/design/runtime/io/processes.md`, 3): a user namespace, a
mount namespace composing the run's view, a pid namespace, a private
`/tmp` and `/dev/shm`, and a run's ids of its own. What is outside the
reach is then not even visible: what Landlock leaves (6.5) has nothing
to act on. Landlock stays beneath as a second layer.

### 5.7 People apart

A host's runs serve one person's accounts, or each person's runs get a
user, a machine or a virtual machine of their own, by placement (jig's
fleet). This is the answer to section 3's last case at any strength, and
to what this draft does not defend against.

## 6. Files by Landlock

### 6.1 Who applies it

The worker's spawn of the agent carries the run's reach, and skein
applies it in the child before the agent's first instruction, through
the helper `skein/docs/design/runtime/kernel.md`, 6.6 designs, built
first for Landlock and `no_new_privs` alone. So the fence is the host's,
not something the run does to itself, and the same spawn serves smith's
narrower domains for its commands (5.3) and hearth's sandboxes.

The alternative is the agent restricting itself at its start, before it
reads anything a run controls: smaller, and enough while the agent is
trusted to do it (10.3).

### 6.2 What the domain grants

The domain handles every file-system right of ABI 6, the floor, which
brings renaming across directories (ABI 2), truncating (ABI 3), devices'
`ioctl` (ABI 5) and the scopes (ABI 6). Full rights below are every one
of them but `ioctl` on devices; reading is reading files and listing
directories.

| Beneath | Rights |
|---|---|
| the workspace's writable directories; the scratch; a writable cache | full, executing included: Cargo runs build scripts and tests from its output |
| the workspace's read-only directories; a read-only cache | reading and executing |
| the stores whose objects the run's git borrows (6.3) | reading |
| the machine: `/usr` whole, and in `/etc` the certificates, the resolver, the name service, the locale, the time zone, the loader's cache and the alternatives; toolchains and `/opt` only as the worker's configuration names them | reading and executing |
| `/dev/null`, `/dev/zero`, `/dev/full`, `/dev/random`, `/dev/urandom` | reading and writing the file |
| `/proc` | reading |
| the agent's executable; its start document | reading and executing; reading |
| on a program source: the source's configuration directory, and the run's own directory of the program's files, its instructions and MCP configuration among them (`smith/docs/design/protocol/programs.md`, 5.1), not its parent | full but executing, narrowed away for commands (5.3) |

- **The environment points tools at their places:** `HOME` and
  `TMPDIR` name the scratch, and each toolchain and cache the
  configuration names carries its own variables (`RUSTUP_HOME`,
  `CARGO_HOME`, a compile cache's), so a toolchain's install is read and
  its downloads land in a cache. Without them, Rustup and Cargo look
  beneath `HOME` and find nothing.
- **Checked against itself:** the worker refuses a reach that would
  grant a path holding its own configuration, data or secrets, so a
  configured machine path cannot leak them.
- **Scoped:** signals and abstract Unix sockets stay within the domain,
  and the kernel refuses a restricted process every ptrace access to a
  process outside its domain: attaching to it, and reading its memory,
  its environment or its descriptors.

Nothing else is granted: the worker's configuration and data, other
runs' workspaces, other program sources, `/dev/tty` (commands have no
terminal), `/dev/shm`, shared by every process of the account, and
every home.

### 6.3 Two git directories

A run's commands need a git that writes: the delivery check of this
repository adds to an index, writes trees, makes a linked worktree and
keeps a cache in the git directory (`tools/pre-pr/pre_pr.py`). And the
host must trust no git metadata a run can write (2.4). So each checkout
has two:

- **The host's,** beyond the run's reach. The worker's git names it and
  the work tree on every operation, never finding either from its
  working directory, and reads the work tree as plain files. Its
  configuration, hooks and refs are the worker's alone.
- **The run's,** the one the checkout's `.git` names: its index, refs
  and configuration are the run's to write, and its objects are
  borrowed, read-only, from the stores its alternates name. The host
  never reads it.
- **Missing objects:** a store fetched without its blobs
  (`temper/docs/design/domain/worker/checkouts.md`) cannot fetch more
  for a run's commands, which hold no forge credential; the run's git
  reports them missing, and the worker may fetch ahead what a run needs.
- **Checkouts made before** get a host directory made afresh from the
  store, and their old directory becomes the run's own: nothing in it is
  trusted, so nothing in it needs cleaning.

smith's tools still refuse `.git` (2.1).

### 6.4 Caches

A cache one run writes and another reads carries whatever the first one
put there. A cache is named in the configuration with a way: read-only
to runs, written by the host alone; or writable, per workstream, keyed
as the workspace is, made with it and removed with it. A cache shared
writable between workstreams is the configuration's explicit choice,
named in each run's start event as the gap it is (10.2). The scratch is
made at the run's start and removed at its end.

### 6.5 Where it cannot hold

- **What Landlock leaves, at ABI 6.** A file's metadata: its mode, owner,
  times and extended attributes, and whether it exists. A run keeps the
  worker's user, so it may change the mode of a file that user owns
  outside its reach, though not read or write it. Connecting to a Unix
  socket named by a path. Another process's command line and status in
  `/proc`, which need no ptrace check. The full view (5.6) closes each,
  with its own ids, its own view and its own pid namespace; until then
  they are recorded here, and a credential never goes in a command line.
- **No fallback.** A kernel without Landlock, or below the ABI the
  reach needs, fails the spawn naming the reach, as a contained spawn
  fails without user namespaces
  (`skein/docs/design/runtime/io/processes.md`, 3). A worker checks at
  startup, by spawning a restricted child, and refuses the runs that
  rely on the fence when it cannot restrict them: those with commands,
  and those on a program source, whose built-in file tools follow links
  (2.1).
- **Unconfined only by name.** A worker's configuration may say its runs
  are unconfined, for a box on an old kernel, and its runs' start
  events say so.
- **Inside a container,** whether its seccomp profile admits Landlock's
  calls is the startup check's to find.

### 6.6 What the run is told

The run's instructions say what its commands see: its directories, the
machine's programs, its scratch; other paths fail with *permission
denied*. A refusal is the command's error output, which the LLM reads as
any failure.

### 6.7 Who builds what

- **skein:** the spawn's reach, the helper's Landlock and
  `no_new_privs`, the startup check; the fake machine and the simulator
  honouring a reach, and the conformance scenarios that hold them to the
  kernel (8.1).
- **smith:** what the instructions say (6.6); then the program's
  commands narrowed and its file tools served by smith (5.3).
- **jig:** the reach in the workspace's vocabulary: a workspace's
  prepare answers its directories and ways
  (`jig/docs/design/domain/hosts/README.md`, 6), and the host adds the
  machine's paths, caches and scratch.
- **temper:** the worker's configuration of the machine's paths, the
  caches and the scratch, and their making and removal; the two git
  directories (6.3); and the deployment's setup for 5.1.

The reach stays the worker's: nothing new crosses the engine's wire with
workers in this step.

## 7. With the rest

- **smith's tools keep their checks:** they give the better error and
  the path's alias; the kernel is the floor beneath them.
- **The permission prompt stays:** a link it allows leads nowhere
  outside the program's reach, and into the program's own directory
  only until its file tools are smith's (5.3).
- **The engine's inline agents** run no commands and hold no workspace
  (`jig/docs/design/domain/hosts/README.md`, 5.1), so nothing here
  changes them.
- **CI** is the forge's runners'; the only rule here is that they share
  no account with a worker (5.1).

## 8. Testing

By skein's strategy (`skein/docs/foundation/testing-strategy.md`), and
the user's word of 2026-11-18: most of it lives in skein, a few
end-to-end tests are focused, cheap and fast, and a somewhat fuller
story runs after landing. The mechanism is skein's and is tested there;
the trees above test that they ask for the right reach, and never retest
the kernel's enforcement: a service's tiers stand on skein's and do not
retest the kit (`skein/docs/foundation/testing-strategy.md`, 9).

### 8.1 In skein: the mechanism

- **Step tests:** a reach's encoding and its bounds, refused at io's
  entrance past its limits, each limit from both sides, as rows of one
  table.
- **The fake machine honours a reach.** skein's design gives its
  minimal fake machine a program that reports, from inside a tree, its
  `no_new_privs` and whether chosen paths can be read and written
  (`skein/docs/design/testing/README.md`, 4), not yet built. It is built
  with the reach, and also reports what it can execute and whether it
  can signal its parent; the machine refuses what a reach does not
  grant, so io worlds and simulated worlds run restricted spawns over
  their seeds.
- **The conformance suite holds it to the kernel**
  (`skein/docs/design/runtime/kernel.md`, 8): the same scenarios against
  the simulator and the real ring. A restricted tree's probe finds what
  was granted readable or writable as granted, and everything else
  refused: a canary beside the root, the parent directory, a link that
  leads out. Privilege is not gained through a set-user-id program; a
  signal to its parent is refused; a kernel without the ABI the reach
  needs fails the spawn. A child that narrows its domain loses what its
  parent keeps, as the prefix of 5.3 relies on; a descriptor opened
  before a restriction keeps its access, so the helper closes every one
  the spawn does not name, and the probe finds none. A disagreement is a
  conformance bug, fixed in the suite first.
- **Cheap by construction:** one spawn per scenario, the probe answering
  every question in one run, within the conformance suite's and the real
  loop's budgets as they stand.

### 8.2 Above skein: the reach asked for

smith's, jig's and temper's domain and system worlds check that every
spawn of a run carries its reach, and that the reach is the run's: its
directories with their ways, the configured machine paths, its scratch;
and, from the step of 5.3, the program's configuration directory for the
program and not for its commands, with the prefix set at each launch. A
referee fails a world in which a spawn goes out without one. No kernel,
no process. smith's fake program calls the prefix as the real one does,
so the adapter is tested here, and the live suite checks only that the
real program calls it every time (8.5).

### 8.3 A few end-to-end tests, focused

Against fakes, one per path through `main` that the reach adds, each
Linux-only by name (the list `temper/docs/design/draft/macos.md`, 5.2
asks each tree to keep):

- **smith:** the agent, as it ships, runs one command that reads a file
  outside its workspace and is refused, and the run goes on.
- **temper:** a worker refuses a run's spawn it cannot restrict, and
  says why in its log.

### 8.4 After landing, one story

In `process-temper`, the cluster's processes with smith's fake program,
whose `Bash` commands try what section 3 fears: another run's
workspace, the host's git directory, the worker's configuration, a
signal to the worker. Each is refused, and in the workspace a small
Cargo build, a commit and the run's own git still work. The step of 5.3
adds the program's login, through a command and through a link read by
a file tool. Each milestone writes and compiles its part; it runs after
landing, as every slow tier does.

### 8.5 Live, by choice

That Claude Code's command prefix wraps every command it runs (10.1)
is checked in `live-smith`, beside the checks of the program's prompt
(`smith/docs/design/protocol/programs.md`, 8).

Plans use focused and fuzzy only, within the suites' budgets.

## 9. macOS

The tools' own checks and the permission prompt hold; commands are not
confined, and a run's start event says so. Nothing more, unless a later
draft finds it cheap.

## 10. Open questions

- **10.1 The program's login** (5.3): the pick is both routes closed,
  commands narrowed through the program's command prefix and its file
  tools served by smith, if the prefix wraps every command, background
  ones included (the live suite checks it, 8.5). It costs the model its
  own file tools; the relayed login, which costs it nothing, follows if
  the program runs on one.
- **10.2 Caches** (6.4): read-only or per workstream by default, or a
  shared writable cache from the start, as a compile cache is today.
  Default: per workstream; a shared one only by the configuration's
  name, recorded in each run's start event as a gap.
- **10.3 Who applies the domain** (6.1): skein's helper at the spawn, or
  the agent itself at its start. Default: the helper.
- **10.4 A machine that cannot restrict** (6.5): refuse the runs that
  rely on the fence, or run them unconfined when the configuration says
  so.
  Default: refuse, unless the configuration says so.
- **10.5 The order after files:** the network (5.4) before the cgroup
  (5.5), since leaving data out is the larger harm. Default: so.
- **10.6 The worker's own account** (5.1) is a deployment's change: for
  dev and staging it waits for the user's word.
