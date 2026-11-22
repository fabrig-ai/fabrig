# The setup's crates

This document holds the setup's crate layout, split from [`README.md`](README.md).

## 1. The crates

```
skein-forge                  + the setup's documents: admin users and their tokens, orgs, repositories'
│                              collaborators and hooks, GitHub's App documents; both sides, as its
│                              others; the pins, the versions and the fetched binaries' digests
│                              (skein's forge/README.md, section 4, and skein/docs/design/forge/forgejo.md, section 1)
skein-forge-setup            the setup's plans, their types and checks, the access they end with, and the
│                              host component a service runs them on (skein's forge/setup/README.md, section 4)
├── skein-forge
└── skein-lib, skein-json
temper-fetch                 step code: a pinned binary's fetch: its machine, its pins' fields and
│                              their origin, Forgejo's and Claude Code's alike; Claude Code's pin and
│                              the word "fetched" (6.6). Names no host
├── skein-forge
└── skein-lib
temper-forge-setup           step code: the setup's types and checks; the machine's part, read from the
│                              configuration, and a forge's setup entry by its kind; the setup mapped
│                              onto the plans' given values, and the kit's check of them; Forgejo's
│                              pins, on temper-fetch's machine, and the preparation's machine;
│                              app.ini and the runner's files; the processes' specs; the kinds' steps
│                              and the group's file. Names no host
├── temper-fetch, skein-forge-setup, skein-forge
└── jig-protocol-config, skein-lib, skein-json
temper-fetch-shell           ordinary Rust: the fetch's host, its requests over TLS, on skein-io;
│                              Claude Code's fetch, under its lock (6.6); no smith crate
├── temper-fetch
└── skein-io, skein-http, skein-tls, skein-lib, skein-shell
temper-launcher              ordinary Rust: Claude Code's launcher, the placing of its links in a
│                              private directory and the substitution of "fetched" in an agent's
│                              document (6.6); for the box's binary, a worker's and the agent's
├── temper-fetch-shell, temper-fetch, smith-agent-document
└── skein-io, skein-lib, skein-json, skein-shell, skein-forge
temper-forge-setup-shell     ordinary Rust: the hosts of fetch, prepare and run: commands, files, the
│                              forge's locks, the exec; Forgejo's fetch on temper-fetch-shell's host,
│                              with its bottle's extraction
├── temper-forge-setup, temper-fetch-shell
└── skein-io, skein-http, skein-tls, skein-lib, skein-json, skein-shell
temper-box-machine           the box's machine (step code), its kinds' groups, their files and records
│                              and its pipe to temper-box serve among it, and its host (ordinary Rust) on
│                              skein-io's processes; nothing of setup but its steps and its kinds
│                              (7.3); one crate, under the subset's
│                              lints, its host written within the subset
├── temper-forge-setup
└── skein-lib, skein-io, skein-shell
temper-init                  step code: the default document and the files temper-box init writes, from
│                              the facts the binary finds (8.2, 8.3); reads, writes and binds nothing
├── temper-fetch
└── skein-lib, skein-json
temper-fake                  step code: the playground's member, application.demo, its types and
│                              checks; the fakes' configurations and seeds; the fake kind's group (8.4).
│                              Names no host
├── temper-forge-setup
├── skein-fake-forge-domain, skein-fake-forge-forgejo, skein-fake-forge-http, skein-fake-llm-domain, skein-fake-peers
└── jig-protocol-config, skein-lib, skein-json
temper-fake-git              ordinary Rust: the fake git program's core, temper-box fake git, its replay and
│                              its two remotes (../../testing/fakes.md, 3)
├── skein-fake-checkout, skein-fake-forge-domain, skein-fake-forge-forgejo, skein-fake-forge-http, skein-fake-peers
└── skein-lib, skein-io, skein-http, skein-json
temper-engine-shell          ordinary Rust: temper-box forge fetch|prepare|run|sign-in|source|remove and
│                              internal forge-child, their rows and the checks they make, for both
│                              binaries below (../binaries.md, 1)
├── temper-forge-setup-shell, temper-forge-setup, temper-box-machine, temper-fake (the demo member's check)
└── temper-service, temper-shell
temper-agent                 the deployment's agent: its table, its modes, its help and --version,
│                              temper-version's (../binaries.md, 3); Claude Code's launcher by its name
├── temper-shell, temper-version, smith-agent-shell, temper-launcher
└── skein-shell, skein-io, skein-fail-stop
temper-box                   the box's binary: the forge's commands through temper-engine-shell,
                               the bare form, temper-box init, temper-box fake llm|forge|git; Claude Code's
                               launcher by its name, temper-launcher's (6.6)
temper-engine                the engine's binary: the forge's commands through temper-engine-shell,
                               on an engine alone's document (../binaries.md, 2)
```

- **The engine runs the setup host** in its protocol layer (`reconcile.md`, 1):
  temper-service maps each forge's setup onto the plans' given values
  through `temper-forge-setup` (`reconcile.md`), and depends on no
  `temper-forge-setup-shell`, and on `temper-box-machine` only for the `box.`
  limits' reference page (`temper-service/src/limits/reference.rs`).
- **The agent's binary links no forge setup:** `temper-agent` reaches
  Claude Code's fetch through `temper-launcher`, `temper-fetch-shell`
  and `temper-fetch` alone, so neither `temper-forge-setup` nor its
  shell is in its closure (`../binaries.md`, 3).
- **The forge's setup links no launcher:** `temper-forge-setup-shell`
  links `temper-fetch-shell`, the fetch's host, which links no smith
  crate, so `temper-engine`'s closure holds none of smith's agent
  document through it (`../binaries.md`, 1 and 2).
- **The binary checks the machine's part and its limits** for every
  command, `temper-box serve`'s included, the box's and the engine's
  through `temper-engine-shell`: `temper-forge-setup` checks
  `application.machine` and the `setup.` limits, and `temper-box-machine` the `box.` limits, beside
  temper-service's startup.
- **Machines, not scripts.** The plans (skein's) and the preparation
  decide from what they read, and say each request or command they need;
  a host runs it and hands back its outcome. Step tests drive them with
  answers; the plans' world, skein's, drives them against the fake forge
  (`testing.md`, 2).
- **Not the throwaway's code.** `testing/skein-throwaway-forgejo` is a
  test crate (skein's `testing/fakes/throwaway.md`, section 1). The production
  machines are written on skein, under the subset, and the process tier
  provisions its Forgejo with them (`testing.md`, 4).
- **The fake forge serves the setup's documents** too
  (`skein-fake-forge-forgejo`, skein's `testing/fakes/forgejo.md`, section 1),
  from the server's side of `skein-forge`, as it serves the engine's.
- **The fakes are product,** `crates/skein-fake-*` (skein's
  `testing/fakes/forge.md`, section 1), since `temper-box fake`
  ships them; the worlds and the real loop use them as before.

## 2. Open questions

None.
