# The deployment's binaries

Draft, 2026-11-08, from the user's words that day (`notes.md`, section 11).

temper has one download today, the whole product in one binary, which a person runs as `temper box` to try it. This draft adds a second set, the binaries of a real deployment, one per process. It covers:

- what each binary holds, and what the user's four left without a home;
- the names, with one choice left to the user;
- the version in the path;
- macOS;
- how the site's release job builds and publishes them, with no new CI.

It departs from "one binary" (`../README.md`, section 1; `../protocol/binaries.md`) for a deployment only. The box keeps one binary.

No earlier story split the binaries. What exists already:
- the cluster's engine-alone and worker-alone shapes of one binary's `serve` (`../protocol/cluster.md`, 1);
- the client, worker and engine crates' boundaries;
- smith's own `smith-agent` executable.

## 1. In one page

- **Two sets on fabrig.ai:**
  - **the box,** as today: one whole-product binary at `fabrig.ai/temper/<os>-<arch>`, stripped and gzipped, built from main;
  - **the deployment,** new: four binaries under `fabrig.ai/temper/v{MAJOR}/`, gzipped and not stripped, each build in a directory of its own commit (section 2).
- **The user's four cover a deployment's processes.** Seven of today's commands had no home among them, and section 3 places each:
  - `ctl`;
  - `forge`;
  - `program-login`;
  - `versions`;
  - `upgrade`;
  - `init`;
  - help.

  `ctl` joins `temper`, so the client is the TUI and `ctl`, not the TUI alone.
- **The four** (section 4):
  - **`temper`:** the TUI and `temper ctl`, with no engine, worker or agent code;
  - **`temper-engine`:** the engine and its web client, with the owned forge's commands, `versions` and `upgrade`;
  - **`temper-worker`:** a worker, with `program-login`;
  - **`temper-agent`:** smith's agent, one process per run, with its helper modes, the fetched Claude Code launcher among them.
- **What the deployer brings:** Forgejo and its runner, the programs runs call, the proxy and the service units. Also the configuration documents, written by hand as the server's help says, since a cluster's `init` isn't designed yet (section 3).
- **The name clash is settled:** B, the whole product as `temper-box` and the client as `temper` (section 5; `cli.md`, section 5).
- **`v{MAJOR}`** is the workspace version's major, `v0` now. Before the first release, versions don't move, so binaries of different commits aren't refused. The four of one commit are the deployer's to run together (section 6).
- **macOS gets the client alone,** for a person on a Mac working with a deployment on Linux (section 7).
- **The site's release job builds them,** when a release build is asked for, as it builds the box binary. Development and CI stay as they are, and a build that breaks is caught at that build (section 8).

## 2. The two sets

| | The box | The deployment |
|---|---|---|
| For | a person trying temper on their own machine | an engine, its workers and their agents, on hosts a deployer runs |
| Binaries | `temper`, the client, which fetches `temper-box`, the whole product, at its first `temper box` (`cli.md`, section 5) | four, one per process (section 4) |
| Where | `fabrig.ai/temper/<os>-<arch>` for `temper`, and `temper-box` in each commit's directory of the deployment set | `fabrig.ai/temper/v{MAJOR}/<COMMIT>/<os>-<arch>/<name>`, and `fabrig.ai/temper/v{MAJOR}/<os>-<arch>/<name>` for the latest |
| Prepared | stripped, then gzipped | gzipped, not stripped, so a crash names its frames |
| Built from | main's latest, for now | one commit for the four |
| Beside them | as the site serves it | `SHA256SUMS`, one line per served file, and `examples.tar.gz`, the commit's `temper/docs/examples/`, in each commit's directory |

- **`<os>-<arch>`** is `$(uname -s)-$(uname -m)`, as the box's (`../release.md`, 5.1): `Linux-x86_64` and `Darwin-arm64`.
- **The starting documents:** the server's help has a deployer copy the example documents. A downloaded set has no checkout, so each commit's directory carries them, as `examples.tar.gz`, from the same commit as the binaries.
- **Gzipped with no `.gz` suffix,** as the box's file is served. A download pipes through `gunzip`, and the help says so beside each URL.
- **Each build in its own directory,** named by its commit's twelve hex digits. A build writes its directory whole, then moves `v{MAJOR}`'s latest link to it in one step. A reader never sees two commits mixed, and a deployer can pin a commit by its directory. The latest link is the draft's reading of "always pointing to the latest release (main for now)", which the user said of the box set.

## 3. What the four leave without a home

The four cover the processes. These commands of today's binary needed a home, and some things stay the deployer's:

- **Clients beyond the TUI: `temper ctl`.** It's how a person's scripts, automations and QA drive a deployment (`../client/ctl/README.md`, sections 1 and 6). Like the TUI, it's a client of the engine's listener, with no engine code, so it goes in `temper` beside the TUI. That's a departure from "just the TUI". `temper open`, the older line client the tests use, stays in the box binary alone.
- **The engine host's commands:** `forge fetch`, `prepare`, `run`, `sign-in`, `source` and `remove`, the owned Forgejo's setup and run (`../protocol/setup/owned.md` and `../protocol/setup/box.md`, section 4). Also `versions` and `upgrade` (`../protocol/binaries.md`).
- **The worker host's command:** `program-login`, signing a program's source in where the worker's agents run it (`../protocol/setup/owned.md`, 6).
- **Help in each binary,** of its own commands. The `commands` reference page names each command's binary.
- **Not in the set: `init`.** Writing a cluster's documents isn't designed: `../protocol/README.md`, section 11, leaves it open, and `init` writes a standalone deployment (`serve-without-box.md`, section 1). A deployment's engine and worker documents are written by hand, as `../../help/server.md` does, until a cluster's `init` is designed. That includes the worker's secret and its digest on the engine, `hosts.workers` and the shared limits. The design then gives the engine's and the worker's binaries their `init`.
- **Also not in the set,** staying in the box binary:
  - the fakes (`temper fake …`);
  - `temper box`;
  - `internal serve --box`;
  - `temper open`.
- **What temper doesn't ship:**
  - Forgejo and its runner, fetched by `temper-engine forge fetch` at skein's pins, or brought by the deployer;
  - the programs a run calls: git, `/usr/bin/rg`, `/bin/sh` and Claude Code (`../release.md`, section 4);
  - each project's own build tools;
  - a reverse proxy with TLS, the network between hosts (`first-release.md`, 5.1) and the service units.

  A runner on a host without `temper-engine` runs Forgejo's runner directly, as its own unit.

## 4. What each binary holds

| Binary | Commands | Runs on |
|---|---|---|
| `temper` | `temper [ADDRESS]`, the TUI; `temper ctl …`; `temper help` | a person's machine, or an automation's |
| `temper-engine` | `serve` (an engine-alone document); `forge …`; `versions`; `upgrade`; `help`; `internal forge-child` | the engine's host |
| `temper-worker` | `serve` (a worker-alone document); `program-login`; `help` | each worker's host |
| `temper-agent` | `temper-agent AGENT.json`, one run; its list mode; `--mcp-relay`; `--program-child`; run as `claude`, the fetched Claude Code launcher | each worker's host, started by the worker |

- **A binary refuses a document that isn't its own.** `temper-engine serve` refuses one with a worker member, and `temper-worker serve` refuses one with an engine member. Each refusal names the binary that would run it. The standalone shape, engine and worker in one process, stays the box's (`../protocol/cluster.md`, 1).
- **The worker starts `temper-agent`:**
  - The worker's document names `temper-agent` as the agent's program, in place of today's own executable with `internal agent` (`../protocol/setup/init.md`, 2).
  - The model-list launch runs the same program in its list mode.
  - The fetched Claude Code launcher's `claude` link points at the agent's program, not at the binary that places it. Today the worker's start and `program-login` place it.
- **The web client is the engine's.** Only `temper-engine` links the browser bundle, embedded.
- **What each may link, as the aim:**
  - the client: the client crates, except the web shell and the bundle, jig's client crates and skein's;
  - the engine: no worker domain and no smith agent;
  - the worker: no engine domain and no web bundle;
  - the agent: smith's agent and temper's thin agent glue alone.

  A boundary test holds each binary's dependency closure, as `cargo tree` gives it for that binary's package (section 10). Code shared between binaries goes in a crate each links, never through another binary.
- **The known couplings the plans cut** (section 9), from the code as it stands:
  - the worker's route calls `temper-engine-smith`'s `fit_brief`;
  - the worker reads the agent's document through `smith-agent-shell`, which links smith's agent service;
  - the engine's limits read the worker's domain and checkout crates;
  - `temper-service` composes engine and worker together;
  - `temper-protocol-records`' `forge` feature is on wherever the service is built;
  - the fetched Claude Code launcher and its fetch live in `temper-forge-setup`, which the worker's `program-login` and the agent's `claude` mode both need.

  A cut that costs more than it gives is named in the boundary test as that binary's exception, with its reason.

## 5. The names

Today the box's file is saved and run as `temper`, and the site's first step is `./temper box`. The user named the deployment's client `temper` too. The user settled it on 2026-11-08 with B (`cli.md`, sections 4 and 5). The options were:

- **A. `temper` in both sets,** each meaning its own set's thing. The URLs keep them apart, and there's one name for a person to learn. But:
  - `temper box` fails on a deployment's `temper`, while `temper ctl` works on both, so the help and the refusals must say which set a command is in;
  - one PATH can't hold both files as `temper`.
- **B. The box's binary becomes `temper-box`,** and `temper` is the client everywhere. Every `temper` a person meets is the same client. But:
  - the site's first step, the box's help, every printed `temper …` command in the box's pages and units, and the tests' commands change to `temper-box …`;
  - a person with the box types `temper-box` for everything the box does, including the client.
- **C. The deployment's client becomes `temper-client`** (or `temper-tui`), and `temper` stays the box's whole product. The box and today's help are unchanged. But a deployment's people type a longer name for their everyday command.
- **D. The deployment's `temper` is the box's whole-product file.** No clash, as `temper` is one thing everywhere, and the deployment set is then the box's `temper` plus the other three. But it departs from "just the TUI": the client on a person's machine carries the engine, worker and agent's code, at about the box's size.

Whichever is chosen:
- under B, chosen, the Cargo targets are `temper` for the client and `temper-box` for the box, each a package of its own (`cli.md`, 3.1);
- the release job names the published files as the choice says;
- with B chosen, the client's file is published as `temper`, and `temper-box` joins each commit's directory, which `temper box` downloads from (`cli.md`, section 5).

## 6. Versions

- **`v{MAJOR}`** is the major of the workspace's version, `0` now, v0.1.0 included. `--version` prints it beside the commit, as `../release.md`, section 3, designs. That isn't built yet, and the plans build it for the four (section 9).
- **One commit a set:** the four are built and published together from one commit, which names their directory (section 2). Each binary's `--version` prints its name, the version and the commit.
- **Mixing isn't refused before the first release.** temper's versions keep their numbers until the first release (`../protocol/README.md`, section 7), so an engine, a worker or a client of another commit isn't refused while their versions match. Running the four of one commit together is the deployer's to keep: the set's directory and its pinned link are how. At an upgrade, jig's rule stops the engine and its workers together. Whether a client shows the version of the deployment it reaches stays `../release.md`'s open question (section 7).
- **A tagged release** builds its set as `../release.md`, section 5, builds its file, and the site's job publishes it under `v{MAJOR}` in its commit's directory, like any build. Placing a release's files among the tag's releases, named by version, stays `release.md`'s, and is later here (section 11).

## 7. macOS

macOS is a person's own machine, and deployments stay Linux's (`macos.md`: "Linux stays the platform of record: the gate, deployments"). So the deployment set for `Darwin-arm64` is the client alone, for a person on a Mac working with a deployment on Linux. The engine, worker and agent aren't built for macOS in this set. The box binary for `Darwin-arm64` stays as today, the whole product.

## 8. Building and publishing

The site's release job builds the box binary today, when a release build is asked for. It builds Linux on a Linux build machine, and macOS through the repository's dispatch-only workflow. It builds the deployment set in the same run:

- **Linux, one cargo invocation per binary:** `cargo build --release --locked` with that binary's package and `--bin`, at the run's commit, in the root of `../release.md`'s glibc floor, with its symbol check (sections 2 and 5). Only the engine's build embeds the bundle. Separate invocations kept one binary's features from reaching another's, as section 4's rule needs; once `cli.md`'s section 6 has taken the features out, they're kept for the release job's builds alone.
- **macOS:** the existing workflow (`.github/workflows/release-macos.yml`) also builds the client's target and uploads it as a second artifact. It's dispatched at a ref pinned to the run's commit, and its commit is checked against the run's before publishing.
- **Prepared and published:**
  - each file checked: it runs `--version` and prints the run's commit;
  - each file gzipped with `gzip -9 -n`, not stripped;
  - `examples.tar.gz` written from the commit's `temper/docs/examples/`;
  - `SHA256SUMS` written;
  - the commit's directory written whole, then the latest link moved (section 2).
- **No new CI:**
  - Development runs as it does, and the gate checks the crates each binary links, as it checks them now.
  - A binary's own build, of its package and `--bin`, runs only in the release job. A break shows there, and is repaired as a red of whoever's change broke it.
  - That's the user's word: "build-level breakages would be caught when I ask for the one-off release build".

## 9. What changes in the code

For the plans, each in its area (`deployment-binaries.execution.md`):

1. **The client:**
   - a package of its own, `temper-client`, with a `temper-client` binary target holding the TUI's start and `ctl`'s entry, and their help, which `cli.md` renames `temper` and moves to `temper/bin/temper/`;
   - the TUI's address and link-file helpers move out of the `temper` library into a client crate;
   - its records without the `forge` feature, which its own invocation keeps (section 8);
   - `--version`.
   - Under option D of section 5, none of this: the client is the box's file.
2. **The engine and the worker:**
   - `temper-service` splits so that an engine-alone service and a worker-alone service each compose without the other's crates, cutting section 4's couplings:
     - `fit_brief` moves into a crate the engine and the worker both link;
     - the engine's limits read no worker crate;
   - the standalone service that composes both stays for the box and the tests;
   - `temper-engine` and `temper-worker` packages, each with its binary, its commands' rows, its help and `--version`;
   - each refuses the other's document;
   - the worker names the agent's program for the model-list launch and the `claude` link.
3. **The agent:**
   - a `temper-agent` package on smith's agent shell and temper's thin agent glue, holding its modes, with `--version`;
   - the fetched Claude Code launcher and its fetch move out of `temper-forge-setup` into a crate the worker and the agent both link;
   - smith's agent-document reader and program helpers move into a smith crate without the agent service, so the worker reads the document without linking the agent. That's a change of smith's.
4. **The box's `temper`** links all of the above and keeps every command it has, so the box and the tests are unchanged.
5. **The dependency rule's test** (section 4), in the repository's boundary tests.
6. **The help:** `../../help/server.md` names the deployment's binaries and starts from a commit's `examples.tar.gz`, and the `commands` page names each command's binary.
7. **The release job** (section 8), outside this repository.

## 10. Acceptance

Plans use the focused and fuzzy suites alone.

- **Dependency closure:** for each binary's package, a boundary test reads `cargo tree` and finds none of the crates section 4 excludes, apart from its named exceptions.
- **Each binary,** in step tests of its package:
  - it prints `--version` and its help;
  - it refuses an unknown command;
  - the engine and the worker each refuse the other's document, naming the right binary.
- **The cluster on separate binaries:**
  - The process tier's cluster story (`../protocol/cluster.md`, 6) gets a sibling that starts `temper-engine`, two `temper-worker`s and `temper-agent`, in place of `temper serve` and `temper internal agent`, and runs a task on each worker.
  - Another story signs `temper ctl` from the client, `temper` (`cli.md`, section 4), in to a `temper-engine` and reads `me`.
  - Both are written and compiled in their plans, and run after landing.
- **The release:** after landing, a one-off release build publishes both sets. A fresh Linux machine then runs an engine, a worker and its agent from one commit's directory, by the help alone. That's a walk, after landing.

## 11. Later

- A tagged release's files named by version, beside its commit's directory.
- A cluster's `init`, giving the engine's and the worker's binaries their `init` (section 3).
- A deployment on macOS.
- `temper open` in the deployment set, if anyone but the tests needs it.
