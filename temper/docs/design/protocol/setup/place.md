# A forge's place

Provisional, 2026-11-16, from `../../draft/config.md`, 6.4
(`../../draft/config.execution.md`, N5). Where an owned or fake forge
listens, and what runs it, are chosen when the forge is set up and kept
with it, never in the document: `application.machine` is refused
(`../configuration/moved.md`, section 3). GitHub's place is github.com's,
and a fake GitHub's is a test's.

## 1. In one page

- **Chosen at set-up** (section 2): an owned Forgejo's `SetUp` and the
  playground's start choose a free port on loopback; the binaries are
  fetched at temper's pinned versions, and the runner is the stock's.
- **Kept twice, each for its readers:** the forge's record keeps its
  API's address and its remotes' base, as it does once made
  (`../forge/engine.md`, 1); the forge's setup file keeps what its
  processes need (section 3).
- **Every forge command reads the setup file:** `fetch`, `prepare`,
  `run`, `source`, `sign-in` and `remove`, and the box's group. A host's
  own units write it once, through `forge prepare`'s flags (section 4).
- **The box opens a group's listener** when the group is first asked, at
  its setup file's port, and holds it from then on (section 5).
- **The owned forge's sign-in through temper** is made with the forge:
  its client is added to the OpenID provider while the engine runs
  (section 6).
- **GitHub** is github.com's; a test gives a fake's (section 7).
- **No person gives a place at set-up:** the port is always chosen and
  the binaries fetched (`../../draft/config.status.md`, *Design
  departs*).

## 2. Chosen at set-up

- **An owned Forgejo's `SetUp`** (`../forge/engine.md`, 2), under the box
  alone and while no owned forge is kept, as today. Once the root admits
  it, the routing chooses the forge's port before the record is made:
  - **The port:** it binds a socket on `127.0.0.1`, port 0, reads the
    port the kernel gave, and closes it. A port that is one of the
    deployment's own listeners' or an LLM sign-in's fixed redirect port
    (`../configuration/README.md`, section 2) is closed and chosen
    again, at most 8 times, then the act is refused *busy*.
  - **The record's place:** the API `127.0.0.1` and that port, in
    plaintext; the remotes' base `http://127.0.0.1:PORT`; no pages of
    its own.
  - **Its person,** derived (`../../draft/config.md`, 6.4): the
    requester's login, and the address `LOGIN@localhost`, as
    `temper-box init --box` derives them today (`init.md`, 2).
  - **Its binaries,** Forgejo's and the runner's, fetched at temper's
    pinned versions into the cache (`owned.md`, 6). A test that must not
    fetch puts the pinned binaries where the fetch finds them present.
  - **Its runner,** the stock's: the binary fetched, the label
    `temper-host:host`, capacity 1, `PATH` `/usr/local/bin:/usr/bin:/bin`.
  - **No public origin,** and **its sign-in client** `forgejo` where the
    deployment has an OpenID provider (section 6), none otherwise.

  The routing then writes the setup file (section 3) and asks the box
  `start N` (`box.md`, 1).
- **The playground's start** (`playground.md`) chooses its fake forge's
  port the same way. Its record's place is that port; its remotes' base
  `http://127.0.0.1:PORT`; its git program `demo/bin/git` in the data
  directory, the link to temper's own binary that `temper-box init
  --box` makes, as today (`init.md`, 2).
- **A forge kept** keeps its place: a later start reads its record and
  its setup file, never chooses again. A forge set up again is a new
  forge, with a new number and a new port.

## 3. The setup file

- **Where:** `forge-N/setup` in the data directory for an owned or
  managed Forgejo that temper's commands run, `playground-N/setup` for
  the fake forge, N the forge's number, beside the forge's own files
  (`box.md`, 1). The directory is made 0o700 as needed, the file 0o600.
  A forge's removal removes it with the directory.
- **What it holds:** a JSON object of temper's, written by temper and
  read by temper alone, never by hand, so its members are the code's:
  `version`, 1; `kind`, `owned` or `fake`; and the members
  `../configuration/engine.md`'s `machine.owned` and `machine.fake`
  had, each with its form and bounds there. For `owned`: `listen`,
  `remote`, `person`, `public`, `forgejo`, `runner` (or `null` for
  none), `mirror` and `sign_in`; for `fake`: `listen`, `remote` and
  `git`. A file of another `version`, or one that doesn't read, refuses
  the command, naming the file. A change of its members moves its
  `version`, and a build reads the versions before its own, as a stored
  format's does.
- **Written once:** by the routing, under the box, before `start N`
  (section 2); or by `forge prepare N` on a host's own units (section 4).
  Nothing writes it again.
- **Read by every command for the forge** (`box.md`, 1 and 4): `forge
  fetch`, `prepare`, `run`, `source`, `sign-in` and `remove`, each
  through the one reader that makes the setup entry `owned.md` and
  `playground.md` take. A forge with none refuses the command, naming
  the file and `forge prepare`'s flags.

## 4. A host's own units

A host's own units run an owned Forgejo with no box
(`box.md`, 4; `temper help server`), and its forge is connected as
managed. Its setup file is written by the first `forge prepare`, from
flags a person gives:

| Flag | Required | Gives |
|---|---|---|
| `--listen ADDR` | yes | `listen`, a socket address on loopback |
| `--remote URL` | no: `http://` and `--listen` | `remote` |
| `--public ORIGIN` | no: none | `public` |
| `--person LOGIN EMAIL` | yes | `person` |
| `--forgejo PATH` | no: fetched | `forgejo`, a cache path or an absolute one |
| `--runner-label LABEL`, repeated | no: no runner | `runner.labels` |
| `--runner-binary PATH` | no: fetched | `runner.binary` |
| `--runner-capacity N` | no: 1 | `runner.capacity` |
| `--runner-path PATH` | no: `/usr/local/bin:/usr/bin:/bin` | `runner.path` |
| `--sign-in CLIENT` | no: none | `sign_in`, a client of the OpenID provider |

- **Once only.** With a setup file present, `forge prepare N` with no
  flags reads it, as every other command does; with flags, each must
  equal the file's, or the command is refused, naming the first that
  differs and both values: `forge N: setup differs at <member>: the
  file has A, the flag B` (the user, 2026-11-16), so that a second run
  can't change a live forge. Other commands take none of these flags.
- **Checked as the members were:** each flag's value is refused as its
  member's was, naming the flag.

## 5. The box's listeners

- **A group's listener** is opened at its setup file's port when its
  group is first asked, with skein's `open_listener`, and held from then
  to the box's end, given to its steps as `box.md`, 1 gives the box's
  listeners. One the box can't open fails the start, `failed N
  listener PORT taken`, and the set-up has failed (`../forge/engine.md`,
  2).
- **At a later start,** the box opens each started group's listener
  from its setup file before any step, as it opened the document's.
  A group with no setup file stays dormant, named on the box's stderr.

## 6. The owned forge's sign-in through temper

- **The box's provider:** `temper-box init --box` makes its signing key,
  `provider.key` in the data directory, as today, and writes
  `engine.clients.provider` with that key, no client, and `addresses`
  `[1]`, the printed link's provider (`../openid-provider.md`, 9). A
  provider may start with no client (jig's `protocol/client/openid.md`).
- **The client is made with the forge:** at an owned forge's `SetUp`
  the routing adds the provider's client `forgejo`, public, its redirect
  URI the forge's browser origin's `/user/oauth2/temper/callback`, and
  the setup file's `sign_in` names it. At each start, a kept owned forge's
  client is added from its record and its setup file as the routing
  hands the forge to the setup host (`../forge/engine.md`, 1); an
  authorization for it before then is refused *unknown client*, and the
  person tries again. A forge removed has its client removed.
- **A managed forge's sign-in** is unchanged: its client is the
  document's (`../openid-provider.md`, 7; `../configuration/moved.md`,
  section 6).

## 7. GitHub

- **A GitHub forge's `Connect`** takes github.com's place: its API at
  `api.github.com`, port 443, TLS with the machine's roots, remotes
  under `https://github.com`, pages at `https://github.com`, and the
  worker's own git.
- **A test gives a fake's,** in `application.test`'s `github`, with the
  members, forms and bounds `machine.github` had
  (`../configuration/engine.md`): the fake backends'
  rewriter (`box.md`, 5), the live tier's relay
  (`../../testing/live-tier.md`) and the process tier's fixtures write
  it. `application.test` holds nothing else, help lists none of it, and
  the TOML file's `[test]` takes it over (`../../draft/config.md`, 4.4).

## 8. Open questions

- **A place given at set-up.** A person may want another port, a public
  origin, or binaries of their own for a box's owned Forgejo. Default:
  none until someone asks; binaries by path come with the TOML file's
  `[programs]` (`../../draft/config.md`, 4.4).
