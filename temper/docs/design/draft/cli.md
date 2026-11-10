# temper's command line

Draft, 2026-11-08, from the user's words that day (`notes.md`, section 13). How temper's binaries meet the people and the programs that run them: their environment variables, their arguments, and how they're built. The conventions are the repository's, for every tree (section 3). This round applies them to temper's binaries alone (sections 4 to 8).

It builds on `deployment-binaries.md`, which splits a deployment into four binaries, and settles that draft's section 5: `temper` is the client, and the whole product is `temper-box` (section 5).

## 1. In one page

- **One layout:** each binary is a package of its own at `<tree>/bin/<binary>/`. Its `main.rs`, at most 30 lines, calls into a crate, and its logic lives under `<tree>/crates/` (3.1).
- **No Cargo features:** `cargo build` at the root builds everything, and a binary links exactly its package's dependencies, whatever is built beside it. What a feature chose becomes a crate, a runtime setting or a fixtures crate (3.2, section 6). Crates leave a binary linking what it did; the recorder and a few browser switches join every build.
- **Three sets:** public (stable, advertised by help), internal (hidden, may change without notice) and test (read by tests alone). A variable's name says its set: `TEMPER_DATA`, `TEMPER_INTERNAL_DUMP_LLM`, `TEMPER_TEST_BUNDLE`. A command's or flag's set is in its binary's table, and its name doesn't change with it (3.3, 3.4).
- **One table per binary** declares every command, flag and environment variable it reads. Help and the reference documents are rendered from it, and code reads the process's arguments and environment only through it (3.5).
- **One help:** `-h`, `--help` and `help` print the same page of public commands, flags and variables, in the same format for every binary (3.6).
- **The reference:** `<tree>/docs/design/cli/<binary>-args.md` and `<binary>-env.md`, with `<tree>/docs/design/cli/common-env.md` for what two or more binaries read and `<tree>/docs/design/cli/test-env.md` for the test set, generated from the tables and checked against them (3.7).
- **Checked by code** (3.8): the layout, `main.rs`'s size, no features, environment access, declared names, the generated documents and help.
- **temper's five binaries** (section 4): `temper`, the TUI; `temper-box`, the whole product, for trying temper and for one person's development on one machine; `temper-engine`, `temper-worker` and `temper-agent`, for a deployment. The public set is small: the TUI, the box, `init`, `serve`, `program-login`, `versions`, the three directories. `ctl`, `forge`, the fakes, `open` and the spawned commands are internal.
- **`temper box` puts the TUI in front of `temper-box`** (section 5): the TUI starts at once, finds a `temper-box` of its own build beside it or in its cache, or downloads it once for its own commit and checks it, then runs it as its child, signing in by a local link of its own. Quitting the TUI stops the box, as today. `temper-box` alone is the box without a TUI. The site's line, `curl … | gunzip > temper && chmod +x temper && ./temper box`, still starts the box.
- **In v0.1.0,** a part beside the ladder, carried out by temper/ui, temper/tui, temper/migration and temper/tiers (`cli.execution.md`).

## 2. What there is

From a survey of main at `ef2c22f0e9`, 2026-11-08.

- **Binaries:** three products:
  - `temper`, the whole product, from `temper/crates/temper`, with a 42-line `main.rs`;
  - `temper-client`, the TUI and `ctl`, 15 lines;
  - `temper-engine`, 122 lines holding its whole dispatch.

  Six tools and fixtures have `main.rs` files of 21 to 342 lines. `temper-worker` and `temper-agent` don't exist yet (`deployment-binaries.execution.md`, M3 and M4).
- **Before its table, `temper` is four other programs:**
  - fake git when run as `git`;
  - the Claude Code launcher when run as `claude`;
  - smith's MCP relay on `--mcp-relay`;
  - smith's program wrapper on `--program-child`.

  `--list` comes before `internal agent-list` as well. None of these is in `--help`.
- **Commands** are rows of `temper-shell`'s table, grouped by who runs them: documentation, deployment, clients, spawned, test doubles. Flags are `skein_shell::Settings`, which gives every flag an environment twin (`--seed` and `TEMPER_SEED`).
- **Help:** `help` prints the index of the 62 compiled pages, not the commands. Every binary's `help commands` prints the whole product's table. `temper` has no `--version`.
- **Environment:**
  - 18 `TEMPER_*` names are read by code the products link, and `SMITH_MCP_SECRET` by the relay. Tests and tools alone read 29 more.
  - No table declares them, and most are documented nowhere.
  - Clippy's ordinary and ring profiles ban `std::env::var`, not `var_os` or `vars_os`.
  - An empty value means set for some switches and unset for others.
- **Features:**
  - **temper's:** five of its crates have them. `forge` and `worker`, on by default, keep the client and the engine lean when each is built alone. `record` (TUI and web), `small-window`, `playground` and `loopback-navigation` shape the recorder and the browser's bundle.
  - **Other trees':** smith has 21 `fixture` features and an `end-to-end`, and skein has a `seam` and a `loopback-navigation`.
  - **A plain root build unifies them:** the root is a virtual workspace of 350 members with no `default-members`, so `cargo build` builds them all and unifies their features. That compiles smith's fixtures and skein's fake GitHub seam into the products, which the release job's per-package builds leave out.
  - **`TEMPER_BUNDLE=none`** skips the browser's bundle, and development builds set it (`AGENTS.md`, section 1).
- **Checks there are:**
  - `tests/boundary/` holds the binaries' dependency closures (`binaries.rs`), dependency direction (`direction.rs`) and fail-stop in every binary (`fail_stop.rs`).
  - `tools/lints/` copies clippy profiles to every crate.
  - Generated pages are compared byte for byte, and `SKEIN_CODEGEN_WRITE=1` rewrites them.

## 3. The conventions

This section is written to become the command line's page in `skein/docs/foundation/`, the repository's convention beside the programming model, and the first milestone lands it. Every tree follows it. Where a tree doesn't yet, a check's exception says so, and its exceptions only shrink.

### 3.1 Binaries

- **A binary is a package of its own** at `<tree>/bin/<binary>/`:
  - a `Cargo.toml` naming the package as the binary;
  - one binary target, at `main.rs` beside it;
  - no library.

  A library crate holding the name is renamed.
- **`main.rs` is plumbing, at most 30 lines:** it installs fail-stop and calls one function of one crate under `<tree>/crates/`, which holds the binary's table and everything it does. With fail-stop, argument capture and error mapping in that crate, a `main.rs` is under 10 lines. 30 leaves room for a module comment and imports, not for logic.
- **Every binary follows this,** a product, a tool or a fixture. A tool's or a fixture's commands and variables are internal or test.
- **A binary's package's dependencies decide what it links,** and nothing else does (3.2).

### 3.2 No Cargo features

- **No workspace crate declares `[features]`.** No dependency on a workspace crate is `optional`, or sets `default-features` or `features`. An external crate's features are chosen once, in `[workspace.dependencies]`.
- **`cargo build` at the root builds every member,** and once no tree has features left, each binary is the same, built alone or with the rest.
- **What a feature did goes to one of three places:**
  - **a crate,** when it chose code a binary links or doesn't: the binary that needs it depends on it;
  - **a runtime setting, internal,** when it switched code every build has;
  - **a fixtures crate beside the crate,** when it exposed fixtures to other crates' tests, which depend on it.
- **No other switch chooses what's built:** no custom `--cfg` and no `RUSTFLAGS` in the repository. The one exception is development speed: a declared internal variable may leave out a step of a build that nothing else needs, as `TEMPER_INTERNAL_BUNDLE=none` leaves out the browser's bundle. A build without it builds everything.
- **Size:** a feature that becomes a crate leaves a binary linking what it linked, so most change nothing. One that becomes a runtime setting compiles its code into every build: for temper, the TUI's recorder in the client, and the browser's recorder and test switches in the bundle. That should be small beside the box's 24 MB, gzipped, but it's unmeasured: the milestone that removes a feature reports each binary's size before and after.

### 3.3 Three sets

| Set | For | Help | Reference | Changes |
|---|---|---|---|---|
| Public | people and their scripts | listed | its binary's documents, first | as the design changes: compatible once releases are, from v0.1.0 |
| Internal | the binaries' own processes, tests and developers | not listed, though an internal command answers its own `--help`, headed "internal" | the same documents, after "Internal" | any time, with its readers and setters in the same change |
| Test | tests, harnesses and tools | none | the test set's document (3.7) | any time |

- **The host's variables** that a binary reads, like `HOME`, `PATH` and `XDG_*`, aren't ours to name. The common document (3.7) lists each and what it does: they're public in effect.
- **Variables a binary sets for the programs it runs:**
  - those with the tree's prefix are declared in that binary's table, in their set: `TEMPER_JOB` for a job's scripts, public;
  - others' names, like `GIT_*`, `CLAUDE_CONFIG_DIR` or `LC_ALL`, stay with the protocol that sets them (`../protocol/git.md`).

### 3.4 Names

- **Environment variables:** `<TREE>_<NAME>` is public, `<TREE>_INTERNAL_<NAME>` internal and `<TREE>_TEST_<NAME>` test. `<TREE>` is the tree's name in capitals: `TEMPER`, `SKEIN`, `SMITH`, `JIG`, `HEARTH`. A variable's name says its set wherever a person meets it: in a shell, a unit or a script.
- **Commands and flags** keep their names when their set changes, since the table holds the set.
  - Words are lower case, with hyphens.
  - Flags are long, `--name VALUE` or `--name`, and `-h` is the only short one.
- **A flag has an environment twin only where the table declares one,** named by the flag's set: `--data` and `TEMPER_DATA`.
  - Twins are for what holds across a host or a session, like the three directories. An invocation's choice stays a flag.
  - A flag wins over its twin.
- **Empty is unset,** for every variable, switches included.
- **Exit codes:** 0, done; 1, failed; 2, refused its arguments, after saying what was wrong and how to get help.
- **A delegated grammar stays its program's:** a binary acting as another program, as fake git, the Claude Code launcher or smith's program wrapper, takes that program's arguments and exit codes, as it does today. These conventions are for a binary's own grammar.

### 3.5 One table per binary

- **The binary's crate holds one table:**
  - its commands, with their positional arguments and flags;
  - the environment variables it reads, or sets under its tree's prefix.

  Each entry has its name, its set, its form, one line, what holds when it's absent, and notes.
- **The table's types and their rendering are skein's,** in `skein-shell` beside `Settings`, so every tree's binaries share one format. A tree's rows are its own: `temper-shell`'s for temper.
- **Code reads the process's arguments and environment once,** in the binary's crate, through the table.
  - `std::env`'s `args`, `args_os`, `var`, `var_os`, `vars`, `vars_os`, `set_var` and `remove_var` are disallowed everywhere else.
  - A lookup in the captured environment names a declared entry, not a string.
  - A helper another tree gives the binary, such as smith's relay or program wrapper, takes the captured arguments and environment rather than reading the process's.
- **Two other places read the environment directly,** each read under a scoped `expect`, naming declared variables:
  - a build script, reading Cargo's variables and its own build-time ones;
  - a test harness, reading the test set through its tree's test table (3.7).

  A name a build script hands the code at compile time (`env!`) is declared like any other, as internal.

### 3.6 Help

- **`<binary> -h`, `--help` and `help` print the same page,** in this order:
  1. one line: what the binary is;
  2. usage: its public forms;
  3. commands: each public command's form and one line;
  4. flags: the public flags, then `-h, --help` and `--version`;
  5. environment: the public variables it reads, each with its twin;
  6. one line on where to read more.
- **`<binary> COMMAND -h` or `--help`** prints that command: its forms, arguments, flags with their defaults, its variables and notes.
- **Pages:**
  - `<binary> help NAME` prints a page;
  - `help pages` prints their index, which is what bare `help` prints today;
  - `help --search WORDS...` searches, as today;
  - `help commands` prints the binary's own table, not another's.
- **`--version`** prints the binary's name, version and commit, on every binary. It has no `version` command beside it.
- **The page:** lines fit 80 columns. Help goes to stdout, exits 0, and reads no directory and no configuration.

For example, `temper-box --help` would print:

```
temper-box: temper on this machine, as one box.

usage:
  temper-box [--config DIR] [--data DIR] [--cache DIR] [--echo none|steps]
  temper-box COMMAND [ARGUMENT...]

commands:
  init [DIRECTORIES]                 Write a deployment that starts with temper's defaults.
  serve [DIRECTORIES]                Run a deployment in one process, until stopped.
  program-login [DIRECTORIES] NAME   Sign a program source in on this host.
  help [NAME | pages | --search WORDS...]
                                     Print this, a page, the pages, or a search.

flags:
  --config DIR, --data DIR, --cache DIR
                                     The deployment's three directories.
  -h, --help                         Print this.
  --version                          Print the name, version and commit.

environment:
  TEMPER_CONFIG, TEMPER_DATA, TEMPER_CACHE
                                     As --config, --data and --cache.

temper-box COMMAND --help prints one command; temper-box help pages lists the pages.
```

### 3.7 The reference documents

- **Under `<tree>/docs/design/cli/`:**
  - for each binary, `<binary>-args.md` (its commands, arguments and flags) and `<binary>-env.md` (the variables it alone reads or sets);
  - `<tree>/docs/design/cli/common-env.md`, for the variables two or more of the tree's binaries read, the host's among them;
  - `<tree>/docs/design/cli/test-env.md`, for the test set, from the tree's test table: one table in its test-support crate, through which tests read them.
- **Each is generated from the tables:** public entries first, then "Internal", each with its set, form, default, line and notes. A test checks each byte for byte, and the tree's rewrite switch rewrites them, as it does the generated help pages. A change to an entry changes its document in the same commit.
- **They're the contract** for what they list. Design documents cite them rather than restate the names.
- **The help pages for agents** (`temper help`) stay, with `commands` generated from the same table.

### 3.8 The checks

Each runs in the focused suite, in `tests/boundary/` unless it says otherwise.

1. **Layout:** from `cargo metadata`, every binary target is `<tree>/bin/<name>/main.rs`, in a package named `<name>` with no library.
2. **`main.rs`:** at most 30 lines, beside `fail_stop.rs`'s check of the same files.
3. **No features:** every workspace manifest, read as `direction.rs` reads them. No `[features]` section, and no `optional`, `default-features` or `features` on a workspace dependency.
4. **Environment access:** clippy's `disallowed-methods` in every `tools/lints/` profile lists 3.5's functions. The one crate per binary that reads them, the build scripts and the test harnesses say so with a scoped `expect`. smith's own clippy configurations, outside `tools/lints/`, take the rule in smith's round.
5. **Declared names:** every token `<TREE>_[A-Z0-9_]+` in the tracked files outside `docs/` (code, scripts, manifests and CI) is declared in a table or in the test set's document, in the set its name says. Every declared variable is read or set somewhere. Made-up names in tests' data take no tree's prefix, so the check needs no exclusion for them.
6. **Documents:** each binary's generated documents match its table.
7. **Help,** in each binary's step tests:
   - `-h`, `--help` and `help` print the same page, listing exactly the public entries;
   - an internal command's `--help` says it's internal;
   - `--version` prints the name, version and commit.

Checks 1, 2, 3 and 5 carry exceptions, each with its reason:

- the other trees' binaries, features and names, until their own rounds;
- temper's own, until the milestone that brings each into line lands and removes its exception (`cli.execution.md`).

A check's exceptions only shrink.

## 4. temper's binaries

Their sets after this draft, under section 5's names. The forms are today's; each binary's table words them as 3.6 says.

### 4.1 `temper`, the TUI

| Command | Set |
|---|---|
| `temper [ADDRESS]`: the TUI on an engine | public |
| `temper box [ARGUMENT...]`: the TUI in front of a `temper-box` child, or `temper-box` itself without a terminal (section 5) | public |
| `help`, `-h`, `--help`, `--version` | public |
| `ctl …` | internal, for now (the user, 11-08) |

### 4.2 `temper-box`, the whole product

For trying temper, and for one person's development on one machine (the user, 11-08). A deployment at scale runs the other four.

| Command | Set |
|---|---|
| `temper-box [DIRECTORIES] [--echo none\|steps]`: run the box without a TUI, today's `temper box --no-tui` | public |
| `init [DIRECTORIES]` | public |
| `serve [DIRECTORIES]`: a deployment in one process (`serve-without-box.md`, section 1) | public |
| `program-login [DIRECTORIES] NAME` | public |
| `help`, `-h`, `--help`, `--version` | public |
| `--fake-backends [--bundle BUNDLE]`; `init`'s `--box`; `--front`, for `temper box` (section 5) | internal |
| `ctl …`, `open …` | internal: the tests' |
| `forge …` | internal (the user, 11-08) |
| `fake …`, `internal …`, but for `internal tui`, which goes | internal |
| run as `git` or `claude`; `--mcp-relay`; `--program-child`; `--list` | internal, as rows of its table that match before the others |

### 4.3 `temper-engine`

| Command | Set |
|---|---|
| `serve [DIRECTORIES]`, `versions`, `upgrade` once it's built | public |
| `help`, `-h`, `--help`, `--version` | public |
| `forge …`, `internal forge-child …` | internal |

### 4.4 `temper-worker`

`serve [DIRECTORIES]`, `program-login [DIRECTORIES] NAME`, `help`, `-h`, `--help` and `--version`, all public (`deployment-binaries.md`, section 4).

### 4.5 `temper-agent`

Every mode internal: one run, the list mode, `--mcp-relay`, `--program-child`, and running as `claude`.

### 4.6 The environment

**Public,** in the common document (3.7):

| Variable | What |
|---|---|
| `TEMPER_CONFIG`, `TEMPER_DATA`, `TEMPER_CACHE` | twins of `--config`, `--data` and `--cache`, read by every binary that takes the directories |
| `TEMPER_JOB` | set by the worker for a job's scripts: the job's directory |
| the host's: `HOME`, `PATH`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME`, `XDG_STATE_HOME`, `XDG_RUNTIME_DIR`, `TERM`, `COLORTERM`, `COLORFGBG`, `DISPLAY`, `WAYLAND_DISPLAY`, `SSH_CONNECTION`, `SSH_TTY`, `SSL_CERT_FILE`, `LISTEN_FDS` | read as each one's entry says |

**Internal,** renamed:

| Today | After | Read by | Set by |
|---|---|---|---|
| `TEMPER_BUNDLE` | `TEMPER_INTERNAL_BUNDLE` | the bundle's build script | development builds, CI |
| `TEMPER_COMMIT`, `TEMPER_CLIENT_COMMIT` | `TEMPER_INTERNAL_COMMIT`, from one build script | the version crate | its build script |
| `TEMPER_DUMP_LLM` | `TEMPER_INTERNAL_DUMP_LLM` | the service and its agents | a developer |
| `TEMPER_FAKE_BOX_PORTS` | `TEMPER_INTERNAL_FAKE_BOX_PORTS` | the box with fake backends | tests |
| `TEMPER_FORGE_NO_REGISTRATION` | `TEMPER_INTERNAL_FORGE_NO_REGISTRATION` | the engine's forge sign-in | our own deployment (`dogfood-hacks.md`) |
| `TEMPER_R8_CUT_AT_LISTEN`, `TEMPER_R8_WAIT_CHECK` | `TEMPER_INTERNAL_CUT_AT_LISTEN`, `TEMPER_INTERNAL_WAIT_CHECK` | the engine's serve | the process tier |
| `TEMPER_WORKSPACE_REUSE` | `TEMPER_INTERNAL_WORKSPACE_REUSE` | the worker | a developer |
| `SMITH_MCP_SECRET` | `SMITH_INTERNAL_MCP_SECRET` | the agent's relay | the program backend, for its child |
| none | `TEMPER_INTERNAL_RECORD` | the TUI's recorder (section 6) | the journeys tool |

**Twins dropped:** `TEMPER_ECHO`, `TEMPER_SEED`, `TEMPER_READY`, `TEMPER_OBSERVATIONS` and `TEMPER_DOCUMENT`. Only tests set them: the fakes' tests and the echo's precedence and refusal tests. Those pass the flags instead, or go where they test the twin itself.

**Test,** in the test set's document (3.7). The 29 that tests and tools alone read become `TEMPER_TEST_<NAME>`, with their setters in scripts, `tools/tiers/` and CI changed with them:

- `TEMPER_E4_*`, `TEMPER_LIVE_*` and `TEMPER_MANAGED_*`;
- `TEMPER_BOX_PARENT_TEST` and `TEMPER_FORGE_PARENT_HELPER`;
- `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER`;
- `TEMPER_RATE`, `TEMPER_RECORD_RELEASE`, `TEMPER_SITTING_WAIT_SECS` and `TEMPER_WRAPPER`.

`TEMPER_TEST_BUNDLE` and `TEMPER_TEST_PLAYGROUND` keep their names. `TEMPER_TEST_CLAUDE` goes: the tiers and CI set it, and nothing reads it. `TEMPER_BENIGN`, a made-up name in a test's data, takes one outside the prefixes.

## 5. `temper box` and the download

- **`temper box [ARGUMENT...]`, in a terminal, runs the TUI in front of a box,** as `temper box` does today (the user, 11-08):
  1. The TUI starts at once.
  2. It finds `temper-box` (5.1), downloading it if it must (5.2), with its progress on the TUI's first screen.
  3. **It starts `temper-box --front` as its child,** with the arguments, and hands it two pipes, as the box hands its front two today (`../protocol/setup/box.md`, 6):
     - **On the first,** the box writes the local link, then closes it. That's a one-time link of its own, drawn beside the printed one, and it never goes in an argument, the environment, a log or a file.
     - **On the second,** the box writes its lines. The TUI shows the box's steps as they come, and the lines go to `DATA/box/logs/box.log`, as today.
     - **The box writes nothing on the terminal,** which stays the TUI's.

     The TUI signs in by the local link, and the printed link stays the browser's.
  4. **Quitting the TUI stops the box:** `temper` asks it to stop, shows it stopping, and exits when it has. If `temper` dies, the box stops too, as it does today when its terminal closes.
- **Without a terminal,** `temper box` finds `temper-box` the same way and runs it in its own place (exec), for scripts and units.
- **`temper-box` alone is the box without a TUI:** it prints the sign-in link on stdout and its lines on stderr, as `temper box --no-tui` does today. `--no-tui` and `internal tui` go, since the TUI is `temper`'s alone. `--front` is internal, for `temper box` only.

### 5.1 Finding `temper-box`

- **Its identity:** a `temper-box` is used only when its build identity, its version and commit word as `--version` prints them (`../release.md`, section 3), is `temper`'s own, read from it before it runs.
- **Where, in order:**
  1. **Beside `temper`'s own executable,** as a checkout's `target/` and a deployment's directory have it. One there of another identity is passed over.
  2. **In the cache directory,** at `bin/<COMMIT>/<os>-<arch>/temper-box`, and checked (5.2). The cache directory resolves as the three directories do: `--cache`, `TEMPER_CACHE`, `$XDG_CACHE_HOME/temper`, else `~/.cache/temper`. `bin/` stands apart from the forge's and Claude Code's caches beside it.
  3. **Downloaded (5.2),** when `temper`'s identity names a clean commit. A `-dirty` or `unknown` identity never downloads: it's refused, saying which binary is missing and how to build it.
- **No PATH:** a `temper-box` on PATH may be another commit's, so `temper` doesn't look there. Run as `temper-box`, it's whatever the shell finds.

### 5.2 The download

- **What's trusted:** fabrig.ai over HTTPS, with the machine's roots or `SSL_CERT_FILE`, as temper's other downloads use them (`../release.md`, section 4). The sums come from the same site, so they catch a damaged file, not a site that lies. Writing the checksum into `temper` at release is Later (section 10).
- **What's fetched:** `https://fabrig.ai/temper/v{MAJOR}/<COMMIT>/<os>-<arch>/temper-box`, gzipped, and `SHA256SUMS` beside it (`deployment-binaries.md`, section 2).
  - Redirects that leave fabrig.ai are refused.
  - The sums are bounded at 64 KiB, and the file at 128 MiB as served and 512 MiB unpacked.
  - Exactly one line of the sums names the file. A missing, doubled or malformed line refuses.
- **Into the cache,** as the forge's fetch does it (`../protocol/setup/owned.md`, 6):
  - one fetch of a file at a time, under a lock beside it;
  - a private staging file, checked against its sum before it's unpacked, within the bound;
  - made executable only then, synced, and renamed into place, with its unpacked SHA-256 written beside it as `temper-box.sha256`;
  - an interrupted fetch leaves only staging files, which the next one removes.
- **Reusing it:** a cached `temper-box` runs when its SHA-256 matches the one beside it. Otherwise it's removed and downloaded again, as the forge's fetch treats a wrong file.
- **Offline,** the download fails, saying so on the TUI's screen, and `temper` exits 1, printing the `curl` line that fetches the file by hand.

### 5.3 The download line

`fabrig.ai/temper/<os>-<arch>` serves `temper`, gzipped. The line is

```
curl -fL https://fabrig.ai/temper/$(uname -s)-$(uname -m) | gunzip > temper && chmod +x temper && ./temper box
```

- Its first `temper box` downloads `temper-box` once, and later ones start at once.
- The bytes are about today's: the box's are downloaded either way, and the client's are added. That's about 1 to 3 seconds for the box on fiber or cable, and up to a minute or more on a weak mobile link.
- Someone who wants only the TUI, to reach a deployment elsewhere, downloads the client alone.
- **The site's sets:** `temper-box` joins each commit's directory of `deployment-binaries.md`'s deployment set, and the short link serves the latest `temper`.

## 6. Features out of temper

| Feature | What it does today | After |
|---|---|---|
| `temper-protocol-records`, `forge`, on by default | the records' forge part, which the client leaves out | a crate of its own, which the engine, the worker and the box link |
| `temper-service`, `worker`, on by default | the service's worker half, which the engine leaves out | crates: the engine's service, the worker's, and the standalone service composing both for the box and the tests (`deployment-binaries.md`, section 9) |
| `temper` and `temper-client-shell-tui`, `record` | the TUI's recorder, for the journeys tool, which brings in the web shell for the journey and redaction code it shares with the browser's | that shared code moves out of `temper-client-shell-web` into a crate both link, so the client links no web shell (`deployment-binaries.md`, section 4); then the recorder is built always, and on with `TEMPER_INTERNAL_RECORD`, which the journeys tool sets |
| `temper-client-shell-web`, `record` | the browser's recorder | built into the bundle always, and on by an internal setting of the page (below) |
| `temper-client-shell-web`, `small-window` and `loopback-navigation`, with `skein-ui-web`'s `loopback-navigation` | the browser tier's small window and loopback navigation | internal settings of the page (below), which the browser tier sets; skein's loopback navigation becomes one in `skein-ui-web` |
| `temper-client-shell-web`, `playground` | the playground's build of the web shell | a crate of its own, the playground's entry, with its own bundle |

- **The page's internal settings** are query parameters of the page's address, read once when it loads: `internal-record`, `internal-small-window` and `internal-loopback-navigation`. Each is off by default. They're honoured only on a loopback origin, as the browser tier and the journeys tool serve the page, and ignored elsewhere.

smith's features and skein's `seam` stay, as check 3's exceptions, until their trees' rounds: each becomes a fixtures crate (3.2). Until then, a plain root build's products compile them in, and the release job's per-package builds don't.

## 7. With `deployment-binaries.md`

- **Its section 5 is settled:** option B. `temper` is the client and the whole product is `temper-box`.
- **Its plans go on as designed:**
  - M3 and M4 land where they would, and this draft's milestones move them into `temper/bin/`. A milestone not yet started when this draft's conventions land starts there.
  - M6 publishes the client's file as `temper`, and `temper-box` in each commit's directory (section 5).
- **Its closure test** (`tests/boundary/tests/binaries.rs`) reads each binary's package's dependency graph alone, with `cargo tree`; building the binaries stays the release job's. Once no tree has features left, a root build gives the same binaries as the release job's per-package builds.

## 8. What changes in the code

For the plans, each in its area (`cli.execution.md`):

1. The command line's page in `skein/docs/foundation/`, from section 3.
2. `skein-shell`: the table's types and sets, their help and their reference documents, and `Settings`' twins declared rather than derived.
3. The checks of 3.8, with their exceptions.
4. **The products move to `temper/bin/`:**
   - `temper-box` from `temper/crates/temper`;
   - `temper` from `temper-client`;
   - `temper-engine`, with its dispatch moved into its crate.

   The libraries holding their names are renamed: `temper`, the box's machine `temper-box`, and `temper-client`.
5. **Each binary's table,** with its sets, help, `--version`, pre-table modes as rows, and generated documents.
6. **`temper box`** (section 5): the TUI's first screen showing the download and the box's steps, `temper-box --front` as its child with the two pipes of `../protocol/setup/box.md`, 6, reversed, the identity check, the cache and the download, and the box's stop. `temper-box` loses `--no-tui` and `internal tui`, and the box stops when its front dies.
7. **The environment's renames** (4.6), with their setters in scripts, `tools/tiers/`, CI and the docs.
8. **The features out** (section 6), with the recorder's shared journey and redaction code first moved into a crate of its own.
9. **The site:** `temper` at the short link, and `temper-box` in each commit's directory.
10. **The help pages, the site and the design documents:**
    - `temper box` stays as it is, and `temper box --no-tui` becomes `temper-box`;
    - `temper init`, `temper serve` and `temper program-login` become `temper-box …`;
    - `temper ctl` stays.

## 9. Acceptance

Plans use the focused and fuzzy suites alone.

- **The checks of 3.8 pass,** with only their named exceptions.
- **Each binary's step tests,** as check 7 says.
- **`temper box`'s choices, in step tests,** with no process or network:
  - which `temper-box` it takes: one beside it of its identity, one beside it of another passed over, a cached one that matches or doesn't, a download, a dirty or unknown identity refused;
  - the sums file's lines: one, none, two, malformed;
  - the bounds;
  - the screen's states, from the download through the sign-in to the stop.
- **Its mechanics, in the process tier,** written and compiled in their plans and run after landing:
  - against a fake site, a loopback server holding a file and its sums: the download, its staging, lock and publication, and an interrupted fetch;
  - the local link on its pipe;
  - quitting the TUI stopping the box, and `temper`'s death stopping it;
  - the exec without a terminal.
- **After landing,** a walk: on a fresh machine, the download line starts the box.

## 10. Later

- The other trees' binaries, features and names, in their own rounds.
- `ctl` public, at the user's word.
- `temper-box`'s checksum written into `temper` by the release build, rather than read from the site.
- Shell completions, and `NO_COLOR`.

## 11. Open questions

None now; `cli.status.md` keeps what's open.
