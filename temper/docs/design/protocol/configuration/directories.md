# The three directories

Every command that reads the document takes three directories, by
skein's arguments' rule (skein's `runtime/shell.md`, 6.5): a flag, else its
variable, else its default.

| Directory | Flag, variable | Default | Holds | Kept |
|---|---|---|---|---|
| configuration | `--config DIR`, `TEMPER_CONFIG` | `$XDG_CONFIG_HOME/temper` | the document, `temper.json`; smith's agent configuration, `agent.json`; the charters' instructions, until `settings.md` moves them into the binary; trusted roots' files | as a person's own files are: copied, synced or published, so it holds no secret |
| data | `--data DIR`, `TEMPER_DATA` | `$XDG_DATA_HOME/temper` | the deployment's directory, with the store, transcripts and jig's secrets' store; the worker's secret and its digest; the worker's root; each set-up forge's directory, named by its number (../setup/README.md, section 3); the scripted model's key and the fake git's link, in `demo/` (../setup/playground.md); the box's groups, in `box/groups/`, and its records of the groups it has started, in `box/started/` (../setup/box.md, 1); the box's logs of its last two runs, in `box/logs/` and `box/logs.previous/` (../setup/box.md, 2); the programs directory | the deployment: backed up whole |
| cache | `--cache DIR`, `TEMPER_CACHE` | `$XDG_CACHE_HOME/temper` | binaries fetched again at will: an owned forge's Forgejo and runner, `temper-box forge fetch`'s in `forgejo/VERSION/` and `forgejo-runner/RUNNER_VERSION/` (../setup/owned.md, 6) | never backed up |

- **The document is `temper.json` in the configuration directory,**
  always: no command takes a document's path. A command that finds none
  there is refused, naming the path, and `temper-box init`, which writes one
  (../setup/init.md, 2); `temper box` alone, on an empty configuration
  directory, runs `temper-box init --box` first (../setup/box.md, 1).
- **Resolved once, at start,** before the document is read: each
  directory, from its flag, variable or default, is made absolute
  against the working directory, by joining names, and every parser is
  given the three (`README.md`, section 2; jig's `protocol/configuration.md`, section
  9). Nothing later is resolved against the working directory.
- **Checked at start:** the data and cache directories, and their
  missing parents, are made private (0o700) when absent; an existing one
  must be the process's user's, private, and not reached through a link
  at its name, or the start is refused, naming it (skein's `runtime/io/files.md`,
  3). A linked parent is accepted, as the programs directory's is
  (`README.md`, section 4). The configuration directory must hold the document; only
  `temper-box init` makes it.
- **Each path member's kind:**
  - *configuration paths:* jig's charters' instructions and trusted
    roots' files, and the machine's GitHub `transport.tls.trust`
    (`engine.md`); the
    worker's agent's `configuration` is one in jig's parts, which
    standalone requires absolute (below);
  - *data paths:* jig's `engine.deployment.directory`, each of
    `engine.hosts.workers`' `digest` and the worker's `secret`; each of
    `application.workers`' `root` (`README.md`, section 3); `application.machine`'s
    `fake.git` and `github.git` (`engine.md`); and `application.demo`'s
    `llm.key` (../setup/playground.md);
  - *cache paths:* `application.machine.owned`'s `forgejo` and
    `runner.binary`, or `"fetched"` for `temper-box forge fetch`'s
    (../setup/init.md and ../setup/owned.md, section 6);
  - *absolute paths,* refused when relative: each of
    `application.workers`' `git` but a null one,
    `application.clients.bundle`, and the agent's `program`,
    `configuration` and `arguments` (`README.md`, section 4), which are installed
    programs and files, or the spawn's own; every path in smith's agent
    configuration, which is smith's format.
- **No secret in the configuration directory:** a worker's secret and
  its digest, an owned or fake forge's handover, every set-up forge's
  file and the scripted model's key are under data, and jig's secrets'
  store is in the deployment's directory.
- **The box passes its three on** to each step, as flags, each absolute
  (../setup/box.md, 1).
- **Each of the cluster's processes has its own three** (`../cluster.md`, 1):
  an engine alone's data holds the deployment's directory and each
  worker's digest; a worker alone's, its secret and its root. No two
  processes share a data or cache directory.
- **Tests give all three,** each a directory of the test's own: no test
  resolves a default under `$HOME` (`../../testing/README.md`).

## 1. The fake-backend sandbox

temper box --fake-backends prepares effective configuration for the
welcome's local peers (../setup/box.md, 5). The flag belongs to configuration
preparation and box supervision alone; the client, service and engine use
their ordinary configuration and paths.

For caller directories C, D and K, its private sandbox is
E = D/box/fake-backends, with configuration C' = E/config, data
D' = E/data and cache K. All child commands receive C', D' and K,
including the worker's agent configuration and its argument. No child is
given the original deployment's data. Original configuration and records
are never overwritten; no credential, account, worker secret or program
login is imported.

- **First preparation:** absent or empty C runs ordinary init in
  C'/D'/K', retaining init's listeners. Give --bundle DIR to name the
  separately built bundle; C stays untouched. A nonempty C is
  accepted only for the current init-shaped standalone configuration,
  and its public agent document; it reads and copies no charters, which
  are the binary's stock (`stock.md`). --bundle overrides
  its application.clients.bundle; otherwise that configured directory is
  used. Unsupported shapes are refused with the member named before any
  child starts. Fresh worker/deployment resources are created in D'.
  Source paths resolve against C before public assets are copied; data
  paths are rebased to D', never followed into the source store.
- **Lifetime:** the box holds an exclusive nonblocking file lock on
  E/lock. It publishes E/prepared last, after effective documents and
  resources pass the ordinary deployment and agent checks. The marker
  identifies source directory names and public-document digests. Partial
  preparation, incompatible source identity or changed source documents
  is refused with instructions to reset the sandbox.
- **Restart:** effective configuration and data survive box shutdown.
  Reuse keeps edits to C' and the state in D', so an ordinary managed
  Forgejo configuration/restart uses the same mechanism as elsewhere.
  An explicit --bundle changes only the retained effective bundle selection,
  after validating it; other effective edits and data remain. Without that
  override, the retained bundle is authoritative. The box prints these
  effective directories. Compatible v1 peer documents are revalidated and
  retained; the seeded output files follow ../setup/playground.md. Peer in-memory
  state lasts one launch, independently of retained configuration and data.
  A later required document or adapter schema that cannot safely reuse an
  older sandbox is refused by name with explicit reset instructions;
  startup never migrates or deletes E or silently omits a required peer.
  Removing E resets this scratch deployment; a
  normal start never reads it as the real one.
- **Roots:** the machine's GitHub part names one plaintext loopback
  fake, its API, its remotes' base and its pages, with the fake git
  adapter as its `git` (`engine.md`). Provider 1 keeps its shipped client, scopes,
  must-grant, resource and callback; its five sign-in origins name the
  local issuer. The openai endpoint in both effective documents names
  the same plaintext fake model, with stale TLS members removed. The
  pure transformation is temper-fake's, with bounded bytes and explicit
  paths/sockets as inputs; files and listeners belong to temper's shell.
- **Browser:** --bundle DIR, or source application.clients.bundle,
  names a provisioned bundle (../../client/web/README.md,
  2.6), whose page takes its loopback navigation from its address
  (../../client/web/README.md, 2.1). Build it once with scripts/bundle.sh DIR, then run
  temper box --fake-backends --bundle DIR. Refuse before sandbox mutation
  only when no bundle is known or its shape/types/size are invalid. The
  refusal names that bundle command and --bundle; no hand-edited setting
  is required. The ordinary embedded bundle cannot replace it. Startup
  neither builds a bundle nor accesses a remote service.

After the sign-in block (`../setup/box.md`, 2), before starting its supervised
processes, the box prints these lines on stderr, for the person and for the
process harness, which captures them by prefix:

```text
box: effective configuration C'
box: effective data D'
box: effective cache K
box: fake github http://ADDR
box: fake issuer http://ADDR
box: fake model http://ADDR
box: fake forgejo http://ADDR
box: copy source ADDRESS
box: copy source ADDRESS
box: model key FILE
box: Forgejo elsewhere admin token file FILE
box: program login: COMMAND
box: GitHub own App id 1, key file FILE
```

A FILE is a path, never its contents. COMMAND is `temper-box program-login`
with the effective directories, ending in the placeholder `NAME (the name
the Model step shows)` for the source name the Model step declares.

The effective configuration names no forge, as every document does
(`engine.md`): the person makes each from the welcome. The playground's
fake forge and the owned Forgejo take the machine's fake and owned
places as init wrote them, and GitHub the fake above. Forgejo elsewhere
is an ordinary managed Forgejo, connected as its admin (../setup/reconcile.md, 5)
at a separate loopback fake, its API and remotes' base that fake's and
its pages none, as for ordinary Forgejo; readiness prints the fake
peer's origin separately. GitHub and Forgejo elsewhere run the fake
git adapter as their git program (../forge/engine.md, 1), since skein's fake
forges speak a fake git protocol; the owned Forgejo keeps the worker's
git. Forgejo elsewhere's retained peer document is C'/forgejo.json, and
its dummy admin token is D'/forgejo-admin.token, private with mode
0600. Readiness names only that file's path; its contents belong in no
log, report or record. The sandbox's own overrides admit four forges'
git identities (`accounts.credentials`, 4); these explicit local
overrides are checked through ordinary admission and edge-bound
checks. Repository defaults, policy bounds and
time budgets remain the ordinary ones. Forgejo elsewhere follows the
ordinary admin-connect contract, not an automatic account login or a
product fake branch. A retained sandbox
missing this required document refuses by name with reset instructions;
its marker stays v1 and no file is silently generated on reuse.

Forgejo here keeps the ordinary owned steps and cache (../setup/owned.md, 6);
an empty cache may need its ordinary one-time pinned download, while a
disconnected walk uses a populated cache. Forgejo elsewhere is an
ordinary managed Forgejo backed by its separate fake. Program login is the
ordinary command with its configured fake adapter, not an automatic login.
The fake program way uses the pinned `fake-claude` executable provisioned
beside `temper`, resolved to that exact absolute sibling path, and its
private adapter directory D'/programs. Preparation supplies that path as
init's program fact; source import takes an existing adapter's `"fetched"`
(../setup/init.md, 2) or its bounded absolute shape, without looking for Claude on
PATH. The pure rewrite writes the exact sibling and private directory
whichever the source names.
Init writes Anthropic's provider, number 2 (../setup/init.md, 3), so the rewrite
finds it written and swaps in its own: the provider's one model, the model
and the endpoint, each found by Anthropic's name and replaced where it stands,
not doubled, so that one model, `fake-claude`, the fake model serves through
an Anthropic-dialect endpoint in both effective documents, ahead of the
scripted model as init's stands. An init-shaped document without them is
refused by name. It keeps the configured `documents.lines`
capacity, raising it to at least 3 so that the Model step offers OpenAI,
demo and Anthropic without shrinking creating's executor lists. If the override
is absent, it retains the named profile's capacity, raised to at least 3,
alongside the credential capacity in the same object. No
startup path discovers or falls back to a real `claude` from PATH.

Reuse requires agent.json/programs/claude_code/path to equal that sibling.
A different retained path refuses with that member named and explicit reset
instructions; no credential or login state is imported. A correct path with
a missing or nonexecutable sidecar instead refuses with the executable path
and the provisioning command `cargo build -p temper-fake-claude`, without
asking for a reset. Startup builds or fetches nothing. A walk that needs a
fake program story provisions it outside startup, then uses ordinary
program-login; the product links no fake-adapter story machinery.

The required retained documents are temper.json, agent.json, github.json,
oauth.json, llm.json and forgejo.json. One validation path checks every
required file before any child starts. A missing required file refuses by
name with reset instructions, without generating a replacement on reuse.

## 2. Open questions

- **A configuration directory moved or copied.** The agent's paths are
  absolute (`README.md`, section 4), so they still name the old place: the agent's
  configuration is read there, or the start is refused, naming the
  member, when it is gone. Default: the person edits the two members;
  standalone could derive them from its own binary and its
  configuration directory, as no check can tell temper's binary from
  another program today (`README.md`, section 2).
