# The file, `temper.toml`

Provisional, 2026-11-19, from `../../draft/config.md`, section 4
(`../../draft/config.execution.md`, N6). What a person writes to run
temper: a few lines of TOML, one file per binary, read by the `toml`
crate's parser through one table, which help, the `configuration` page
and `init` are rendered from. The file gives the start the document its
decoders have read since `README.md`, section 2, so every check of a
member stays where it is; a person never writes that document, and
`temper.json` is refused by name. Where `README.md`, `engine.md`,
`moved.md`, `../cluster.md` or a setup document still describes
`temper.json` or one of its members as what a person writes, this
document wins.

## 1. In one page

- **One file per binary,** `temper.toml` in the configuration directory
  (`directories.md`): `temper-box` reads a standalone file, engine and
  worker in one process; `temper-engine` an engine alone's;
  `temper-worker` a worker alone's. Each binary's table is its own, so
  a key of another shape is refused, naming its line and the binary
  whose key it is (section 3).
- **TOML, flat, named by what a person means** (section 2): `listen`,
  `public`, `workers`, `engine`, `owner`; a table only where a group
  means something of its own. Its values are plain: an address a
  string, a path a string typed by its directory.
- **Strict, naming the line:** an unknown key, a wrong type or an
  out-of-bounds value refuses the start as
  `temper.toml:LINE: KEY: WORDS`, as smith's `config.toml` does (smith's
  `shell/settings.md`, 1.3).
- **One table declares every key** (section 3), in
  `temper-config-shell`: its set (public, internal, test), its shapes,
  its form and bound, its default, a line, and the start's members it
  gives (section 4). Help (`temper-box help config`, `temper-engine help
  config`, `temper-worker help config`), the reference page
  `configuration` and `init`'s file are rendered from it (section 7).
- **The start's document** (section 4) is what the file gives, built by
  the code, with temper's constants and derivations: the JSON members of
  `README.md`, section 2, decoded and checked as before. The JSON's
  tagged forms (`{"socket": ...}`, `{"in_process": ...}`) are the
  code's. A problem a start finds at a member a key gave names that key
  and its line.
- **Code reads the file only through the table** (section 8): one crate
  parses TOML, and only it names the file.
- **`temper.json` is refused by name** (section 6), and the
  `configuration` page says where each of its members went.
- **Places are the worker's setting's** (`figures.md`, sections 3 and
  4): no file has `slots`.
- **A box's file is one line,** `listen`, which `temper-box init`
  writes (section 7); an engine's and a worker's a few, written from
  help (section 9 gives dev.fabrig.ai's).

## 2. Its format

- **TOML 1.0, read whole at start,** at most 1 MiB, as UTF-8, by the
  `toml` crate's parser alone, `toml::de`'s spanned tables
  (`toml::de::DeTable`), the workspace's `toml` without its writer or
  its `serde` where the crate allows. Its spans give each refusal its
  line. temper parses TOML nowhere else.
- **Its values are plain.** A socket address is a string,
  `"127.0.0.1:7100"`; a path is a string, typed by its directory as
  `directories.md` types each member's (a data path relative to the
  data directory or absolute); a list is a TOML array; a group is a
  table. No key takes a duration or a size; one that comes to follows
  smith's form, whole seconds with the unit in the key's name (smith's
  `shell/settings.md`, 1.2).
- **Refusals,** one line each on stderr, every one found, then exit 1
  before io starts, as the start's problems are printed today
  (`README.md`, section 2):

  | What | Its line |
  |---|---|
  | TOML that doesn't parse | `temper.toml:LINE: WORDS`, the parser's message |
  | an unknown key | `temper.toml:LINE: KEY: unknown; temper-box help config lists the keys`, the binary's own name |
  | another shape's key | `temper.toml:LINE: KEY: a key of temper-engine's file, not temper-worker's`, with the binaries that read it and this one |
  | a moved name (3.3) | `temper.toml:LINE: KEY: moved: WORDS` |
  | a wrong type, a bound passed, a duplicate | `temper.toml:LINE: KEY: WORDS`, saying the form or bound, as `a socket address, such as "127.0.0.1:7100"` |
  | a required key absent | `temper.toml: KEY: required in temper-engine's file` |
  | a problem the start finds at a member a key gave (4) | `temper.toml:LINE: KEY: WORDS`, WORDS the start's own |
  | one at a member a key's default gave | `temper.toml: KEY (its default): WORDS` |

  KEY is the key's dotted path, an array's entry by its index from 0:
  `sign_in.google.client`, `sign_in.members[1].login`. A problem the
  start finds elsewhere, in a secret's file, the store or the agent's
  configuration, prints as it does today.
- **It holds no secret.** A secret is a file a key names, in the data
  directory, at mode 0600 (`README.md`, section 5).
- **It's written once:** `init` writes the box's (section 7); nothing
  temper does writes it after.
- **A key whose value equals its default** is accepted; `init` writes
  none but `listen` (section 7).

## 3. Its keys

One row each in the table. The shapes: **S** standalone (`temper-box`),
**E** engine alone (`temper-engine`), **W** worker alone
(`temper-worker`). The start's members each gives are section 4's.

### 3.1 Public

Help lists these.

| Key | Shapes | Form and bound | Default | What it says |
|---|---|---|---|---|
| `listen` | S E | a socket address on loopback | `"127.0.0.1:7100"` | the clients' listener |
| `public` | S E | an origin, as `application.clients.public` takes it (`engine.md`) | none: the listener's origin | the address people reach, through a proxy that holds its TLS |
| `webhooks` | S E | a socket address on loopback | `listen`'s address, its port plus 1; port 0 where `listen`'s is 0 | the forge's webhooks' listener |
| `owner` | S E | an address, 3 to 254 visible ASCII bytes with one `@` | the process's user's login at `localhost`, as `init` derives it (`../setup/init.md`, 2) | the first owner, read at the first start alone; later owners are the store's |
| `workers` | E | a socket address on loopback | none: required | the workers' listener |
| `engine` | W | a socket address on loopback | none: required | its engine's workers' listener |
| `name` | W | 1 to `link.name` bytes (32 in the stock) | the host's name, as the kernel gives it | the name its setting has (jig's `domain/engine/workers.md`, 2) |
| `[sign_in.google]` `client`, `secret` | S E | `client` 1 to 256 visible ASCII bytes; `secret` a data path | none: no Google sign-in | Google's sign-in: its client and its secret's file |
| `[[sign_in.members]]` `address`, `login` | S E | as `application.sign_in.members` (`engine.md`), 1 to 64 entries; with `[sign_in.google]` alone | none; required with `[sign_in.google]` | each person who may sign in with Google |
| `[[sign_in.forges]]` `forge`, `client` | S E | as `application.sign_in.forges` (`../openid-provider.md`, 7); with `[sign_in.google]` alone | none | each Forgejo whose sign-in goes through temper, by its forge's number and its client |
| `[openid]` `key`, `addresses` | S E | `key` a data path; `addresses` a list of 1, the printed link's, and 3, Google's | `key` `"provider.key"`; `addresses` `[3]` with `[sign_in.google]`, else `[1]` | temper as an OpenID provider for Forgejo's sign-in (`../openid-provider.md`, 7): its signing key, and the identity providers whose subjects are given as `email` |
| `[[openid.clients]]` `id`, `secret`, `redirect` | S E | as `engine.clients.provider.clients` (jig's `protocol/configuration.md`, section 3): `id` 1 to 255 visible ASCII bytes, `secret` a data path or absent for a public client, `redirect` its redirect URI | none | each client of that provider |
| `[push]` `key`, `subject` | S E | as `application.push` (`engine.md`) | none: no push | the deployment's push key's file and its subject |
| `[programs]` `git`, `claude` | S W | an absolute path to an executable | found on `PATH` at each start, or `claude` fetched, as today (`figures.md`, section 6) | a program that isn't found, by its path |
| `[programs]` `forgejo`, `runner` | S | an absolute path to an executable | fetched at the owned Forgejo's set-up, as today (`../setup/place.md`, 2) | the owned Forgejo's binaries, by their paths |
| `[environment]` | S W | a table of strings, `NAME = "value"`: names unique, not empty, with no `=` or NUL; values with no NUL | the worker's own `PATH` and `HOME`, appended where absent (`README.md`, section 3) | variables its agents' programs get, in the file's order |

- **`[openid]` on a box.** Without the table, a box's start gives its
  provider as `temper-box init --box` made it (`../setup/place.md`, 6):
  `provider.key` in the data directory, no client and `addresses`
  `[1]`, where that file is there, and none where it isn't. An engine
  alone has a provider only with `[openid]`.
- **`owner`'s provider** is Google's, 3, where the file has
  `[sign_in.google]`, and the printed link's, 1, otherwise, as today's
  documents have it (`../openid-provider.md`, 9). With Google's, it is
  one of `[[sign_in.members]]`, as the start checks today.
- **The workers' channel stays as today** (`../cluster.md`, section 3):
  `workers` and `engine` are a plaintext link's on loopback.

### 3.2 Internal and test

Help lists neither; the `configuration` page lists both, each marked
(`skein/docs/foundation/cli.md`, 4).

| Key | Set | Shapes | Form | Default | Gives |
|---|---|---|---|---|---|
| `[internal]` `bundle` | internal | S E | an absolute directory | the binary's own bundle | a client bundle of another build |
| `[test.github]` `api`, `transport`, `remote`, `web`, `git` | test | S E | `api` `"HOST:PORT"`, a socket address or a host name and port; `transport` `"plaintext"`, to a loopback socket alone, or `{ server_name = "...", trust = "..." }`, `trust` a configuration path or absent for the machine's roots; `remote`, `web` URL bases with no final slash; `git` a data path, optional | github.com's | a fake GitHub (`../setup/place.md`, 7) |
| `[test.google]` `issuer`, `authorization`, `token`, `keys`, `transport` | test | S E | URLs within `sign_in.url_bytes`; `transport` as `[test.github]`'s; with `[sign_in.google]` alone | Google's | a fake issuer for Google's sign-in |
| `[test.push]` `services` | test | S E | 1 to 8 origins; with `[push]` alone | Google's, Mozilla's and Apple's | fake push services |

Each carries the bounds and checks its member has today (`engine.md`;
`../setup/place.md`, 7). A test that needs more than the file gives,
such as other limits, takes it as a person would: a setting, the
operator's `limits-held` (`limit-settings.md`, 3), or its world's own
construction of the typed configuration (`moved.md`, section 2).

### 3.3 Moved names

A key named as a member of `temper.json` that has no key of its own is
refused with words saying where it went, each at most 120 bytes:

| Key | Words after `moved: ` |
|---|---|
| `slots` | `a worker's places are its setting's; temper ctl worker places NAME N` |
| `secret` | `a worker's secret is secrets/worker.secret in the data directory` |
| `limits`, `memory_bytes` | `limits are settings; ask temper, or use temper ctl settings apply` |
| `agent` | `temper writes the agent's configuration from the engine's figures` |
| `welcome` | `the welcome is a setting; change it with temper ctl settings` |
| `machine` | `a forge's place is chosen when it's set up, and kept with the forge` |
| `demo`, `llm` | `LLM providers are settings; see them with temper ctl settings show provider` |
| `owners` | `the first owner is owner; later owners are the store's` |
| `clients`, `hosts`, `application` | `temper.toml is flat; temper-box help config lists its keys`, the binary's own name |

## 4. What a file gives

The reader builds the start's document from the file's values and
these constants, then the start reads it as `README.md`, section 2
gives (its steps 2 to 5). Members not named here are absent, and each
constant of `moved.md`, section 2 is passed as there.

| Member | Standalone | Engine alone | Worker alone |
|---|---|---|---|
| `engine.hosts.listener` | `[{"in_process": "workers"}]` | `[{"socket": workers}]` | no `engine` |
| `engine.clients.provider` | `[openid]`'s, else the box's (3.1), else none | `[openid]`'s, else none | — |
| `engine.owners` | one: `owner`'s provider (3.1), its subject `owner` | the same | — |
| `workers` | one: `name` `"local"`, `engine` `{"in_process": "workers"}`, `secret` `"worker.secret"`, `slots` 1 | `[]` | one: `name` `name`, `engine` `{"socket": engine}`, `secret` `"secrets/worker.secret"`, `slots` 1 |
| `application.clients` | `listener` `{"socket": listen}`, `public`, `bundle` `[internal]`'s or `null` | the same | — |
| `application.webhooks` | `{"listener": {"socket": webhooks}}` | the same | — |
| `application.sign_in` | `[sign_in.google]` with provider 3 and endpoints `[test.google]`'s or `null`, its `members` and `forges`; `null` without it | the same | — |
| `application.push` | `[push]` with `services` `[test.push]`'s or `null`; absent without it | the same | — |
| `application.test.github` | `[test.github]`'s, absent without it | the same | — |
| `application.environment` | `[environment]`, in its order | — | the same as standalone |

- **`slots` gives no places.** jig's worker part requires it, and a
  start reads its places from the worker's setting (`figures.md`,
  sections 3 and 4), so the reader fixes it at 1, which every check of
  it passes.
- **The secret's file** is the data directory's: a box's
  `worker.secret`, as `temper-box init` writes it (`../setup/init.md`,
  2), and a worker alone's `secrets/worker.secret`, whose digest its
  setting holds (`../../draft/config.md`, 6.7).
- **`[programs]` gives no member:** the start's lookups take it
  (`figures.md`, section 6; `../setup/place.md`, 2) from the values the
  reader returns beside the document (section 5).
- **The start's document is never a file.** Tests build one in memory
  as before (`moved.md`, section 2); one that runs a binary writes a
  `temper.toml`.

## 5. The table in code

A shell crate, `temper-config-shell` (ordinary Rust, the binaries'
lint profile), holds the table, the reader and the renderers, and
names `toml`. Its surface, which `init`, the binaries' shells, the
reference's test and the test harnesses build on:

```rust
// temper_config_shell
pub use temper_service_core::startup::Shape; // Standalone, EngineAlone, WorkerAlone
pub const FILE: &str = "temper.toml";
pub const FORMER: &str = "temper.json";

/// What a file gives, every key typed; `File::default()` is an empty file.
pub struct File {
    pub listen: Option<std::net::SocketAddr>,
    pub public: Option<String>,
    pub webhooks: Option<std::net::SocketAddr>,
    pub owner: Option<String>,
    pub workers: Option<std::net::SocketAddr>,
    pub engine: Option<std::net::SocketAddr>,
    pub name: Option<String>,
    pub sign_in: Option<SignIn>,      // [sign_in.google], [[sign_in.members]], [[sign_in.forges]]
    pub openid: Option<OpenId>,       // [openid], [[openid.clients]]
    pub push: Option<Push>,
    pub programs: Programs,           // git, claude, forgejo, runner: Option<PathBuf>
    pub environment: Vec<(String, String)>,
    pub internal: Internal,           // bundle: Option<PathBuf>
    pub test: Test,                   // github, google, push_services
}

/// The machine's facts the defaults need: the user's login, the host's name,
/// whether the data directory holds provider.key.
pub struct Facts { /* ... */ }

pub struct Read {
    pub file: File,
    pub document: Box<[u8]>, // the start's document (4)
    pub keys: Keys,          // each key written, its line and the members it gave
}

pub fn read(directories: &temper_shell::directories::Directories, shape: Shape) -> Result<Read, Refusal>;
pub fn parse(text: &[u8], shape: Shape, facts: &Facts) -> Result<Read, Refusal>;
pub fn render(file: &File, shape: Shape) -> Box<[u8]>; // the file's text, each line with its comment
pub fn help(shape: Shape) -> Box<[u8]>;                // `help config`'s page
pub fn reference() -> Box<[u8]>;                       // the `configuration` page
```

- **`SignIn`, `OpenId`, `Push`, `Programs`, `Internal` and `Test`**
  hold their tables' keys by the keys' names (`client`, `secret`,
  `members`, `forges`; `key`, `addresses`, `clients`; ...), each entry
  of an array of tables a record of its own (`Member`, `ForgeSignIn`,
  `OpenIdClient`).
- **`read`** is `parse` over the configuration directory's file, with
  the machine's facts, after `temper.json`'s refusal (section 6).
- **`Keys`** renders a start's problem, a member's path and its words,
  as section 2's line: the key that gave the member, or its nearest
  ancestor, and its line.
- **`render`** writes only the keys `File` holds, in the table's
  order, each with its comment, under the file's two header lines
  (section 7), so that `parse` of `render(file)` gives `file` back, a
  default left absent.
- **One row per key,** each with its key, set, shapes, form and bound,
  default, line, and the start's members it gives, which `reference`
  renders and `Keys` reads. A row's line is at most 120 bytes.

## 6. Reading

Every command that reads the configuration reads it through `read`, at
its binary's shape: `serve` of each binary; `temper-box`'s own run,
`program-login`, `upgrade`, `checkpoint`, `restore`, the fake forge's
check and the fake backends' import; `temper-engine`'s `forge`
commands that read it, `upgrade`, `checkpoint` and `restore`;
`temper-worker program-login`. In order:

1. **`temper.json` refused:** a `temper.json` in the configuration
   directory, whatever else is there, refuses the command:
   `temper.json: moved: the configuration is temper.toml; write it from
   temper-box help config, then remove temper.json`, the binary's own
   name. Its contents are never read.
2. **`temper.toml` absent:** `temper.toml: absent; temper-box init
   writes one` from `temper-box`, and `temper.toml: absent; write it
   from temper-engine help config` from the other two.
3. **Parsed and checked** against the binary's table (section 2).
4. **The start's document** built (section 4), and read by the start as
   `README.md`, section 2 gives, every problem it finds printed through
   `Keys`.

- **A binary reads its own shape alone:** `temper-box` no longer serves
  an engine alone's or a worker alone's configuration; those are
  `temper-engine`'s and `temper-worker`'s
  (`../../draft/deployment-binaries.md`, section 4).
- **`init`'s refusal** names `temper.toml` or `temper.json`, whichever
  the configuration directory holds (`../setup/init.md`, 2).

## 7. Rendered from the table

- **Help:** `temper-box help config`, `temper-engine help config` and
  `temper-worker help config` print the binary's page, `help(shape)`:
  the file's public keys for that shape, each its key, default and
  line, then an example file, the box's of `../../draft/config.md`, 4.2,
  and the engine's and worker's of its 4.3. It is answered from the
  table as `help commands` is (`../../cli/README.md`, 5), and needs no
  deployment.
- **The `configuration` page** (`../documentation.md`, section 8),
  `reference()`: every key of every shape and set, each marked with its
  shapes and set, its form and bound, its default and its line; then
  each member of `temper.json` and where it went: a key, by the rows'
  members, or `moved.md`, section 3's words; then `The merge command`,
  as today. It replaces the pages `configuration` and
  `configuration-engine`, and `configuration-engine` goes.
- **`init`** (`../setup/init.md`, 2 and 3) writes `render` of a `File`
  holding `listen`, the port it chose, and `webhooks` only where the
  next port isn't free. `listen` is the one default `init` writes, since
  its port is the machine's choice. The box's file:

  ```toml
  # temper's configuration. `temper-box help config` says what each line does.
  # Everything else is changed by asking temper.

  listen = "127.0.0.1:7100"   # where the client connects
  ```

## 8. The check

A focused test in `tests/boundary/` holds that code reads the file
only through the table:

1. in temper's tree, `temper-config-shell` alone depends on `toml`, as
   `cargo metadata` gives every kind of dependency;
2. in temper's tracked Rust, the strings `temper.toml` and
   `temper.json` appear in `temper-config-shell` alone: every other
   crate, its tests among them, names `FILE` or `FORMER`.

Beside it, `temper-config-shell`'s tests hold `help` of each shape and
`reference` equal to what the table renders, the checked-in page
`temper/docs/help/reference/configuration.md` among them
(`skein/docs/foundation/cli.md`, 9, check 6).

## 9. Upgrades

The format moves once, as the user chose for this draft
(`../../draft/config.md`, section 8): a `temper.json` is refused by
name, and a person writes `temper.toml` from help. dev.fabrig.ai's
operators take these steps at its next redeploy, before its start, on
the user's yes, each value from today's member in its `temper.json`:

1. **The engine's file,** from `temper-engine help config`:

   ```toml
   # temper's engine. `temper-engine help config` says what each line does.
   # Everything else is changed by asking temper.

   listen  = "LISTEN"    # the clients' listener, behind your proxy
   public  = "PUBLIC"    # where people reach it
   workers = "WORKERS"   # where workers connect, on loopback
   owner   = "OWNER"     # the first owner, read at the first start alone

   [sign_in.google]
   client = "CLIENT"
   secret = "SECRET"     # in the data directory, mode 0600

   [[sign_in.members]]   # one for each member
   address = "ADDRESS"
   login   = "LOGIN"
   ```

   `LISTEN` is `application.clients.listener`'s socket, `PUBLIC`
   `application.clients.public`, `WORKERS` `engine.hosts.listener`'s
   socket, `OWNER` the subject of `engine.owners`' entry at provider 3,
   `CLIENT` and `SECRET` `application.sign_in.google`'s, and each
   member one of `application.sign_in.members`. Where they're there,
   also: `webhooks` from `application.webhooks.listener`'s socket, unless
   it is `LISTEN`'s next port; `[[sign_in.forges]]` from
   `application.sign_in.forges`; `[openid]` from `engine.clients.provider`
   (`key` unless `provider.key`, `addresses` unless `[3]`) and an
   `[[openid.clients]]` for each of its `clients`, `redirect` its
   `redirect_uri`; `[push]` from `application.push`; `[internal]
   bundle` from `application.clients.bundle`.
2. **Each worker's file,** w1 to w3, from `temper-worker help config`:

   ```toml
   # temper's worker. `temper-worker help config` says what each line does.

   engine = "ENGINE"   # where its engine listens for workers
   name   = "w1"       # the name its engine knows it by, if not the host's

   [environment]       # one line for each variable
   NAME = "VALUE"
   ```

   `ENGINE` is `workers[0].engine`'s socket, `name` `workers[0].name`,
   and each variable one entry of `application.environment`. Where
   `workers[0].secret` names another file than `secrets/worker.secret`
   in the data directory, that file moves there: its digest is the
   same, so the worker's setting still admits it.
3. **The four `temper.json` files are removed.** Nothing else moves:
   the settings, the store and the secrets stay.

After landing, a fresh `temper-box init --box` writes its one line and
the box starts.

## 10. Testing

Focused, with fuzzy where it says; the slow tiers' stories that change
are written and compiled, and run after landing.

- **The reader:** each shape's example file (7; `../../draft/config.md`,
  4.3) gives the start's document section 4 gives and starts its shape
  in memory; an unknown key, another shape's key, a moved name, a wrong
  type, a bound passed and a required key absent are each refused with
  section 2's line; a start's problem at a member a key gave names the
  key and its line; `temper.json` is refused by name with or without a
  `temper.toml`.
- **The renderer, fuzzy:** for any `File` of a shape within its bounds,
  `parse` of `render` gives it back.
- **The pages:** `help` of each shape and `reference` equal the
  checked-in renderings (8).
- **`init`:** `temper-box init --box` writes the one line of section 7,
  and a box starts from it.
- **The check** (8).

## 11. Open questions

- **Who may sign in** (`[[sign_in.members]]`) stays the file's, and so
  does the OpenID provider's configuration (`[openid]`): a start checks
  the owner against the members before its store opens. Default: they
  stay until a design makes them settings.
- **The webhooks' listener** (`../../draft/config.md`, section 11): the
  next port, or a path on the clients' listener. Default: the next
  port, as 3.1 has it.
