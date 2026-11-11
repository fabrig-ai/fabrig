# temper's members for the engine

**`machine`:** the machine's places, never a forge: where a box's owned
Forgejo and the playground's fake forge listen, which `temper-box init
--box` chooses (../setup/init.md, 2) and the box holds from its start (../setup/box.md,
1), and where GitHub is. A forge set up or connected takes its place
from here (../forge/engine.md, 2). Each:

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `owned` | an object, or absent | ../setup/init.md's owned members: `listen`, a socket address on loopback, unique among the deployment's listeners; `remote`, its remotes' base, `http://` and its `listen` as `temper-box init` writes it; `forgejo` and `runner.binary`, cache paths or `"fetched"`; `runner`, `person`, and the optional `mirror`, `public` and `sign_in`, a client of `engine.clients.provider` (`../openid-provider.md`, 9) | an owned Forgejo's place, which `SetUp` gives its record (../forge/engine.md, 2), and its setup entry and group's steps (../setup/init.md and ../setup/box.md, section 1); absent, no owned forge is offered |
| `fake` | an object, or absent | `listen`, as `owned.listen`; `remote`, `http://` and its `listen`; `git`, a data path, the fake git's link (../setup/init.md, 2) | the playground's fake forge's place and setup entry, its owner `playground` and its repository `widget` (../setup/playground.md), and its git program, which the engine's routing names in each of its repositories' records (../channel.md, section 4); absent, no playground is offered |
| `github` | an object, or `null` | `null` for github.com's: `api.github.com`, port 443, TLS with the machine's roots, remotes under `https://github.com`, pages at `https://github.com`. Otherwise `{"api", "transport", "remote", "web", "git"}`: `api` a socket address or a host's name and port, never an in-process name; `transport` plaintext to a socket address on loopback alone, or `{"tls": {"server_name", "trust"}}`; `remote` and `web` remotes' bases; `git` an optional data path, the git program of its repositories | a GitHub forge's place and git program, which `Connect` gives its record (../forge/engine.md, 2); a value is given only to point at a fake, as temper's tests and `temper box --fake-backends` do (`directories.md`, 1), or at the live tier's relay to github.com (`../../testing/live-tier.md`) |

- **A remotes' base** (each part's `remote`, GitHub's `web`, and the
  base a person gives a Forgejo connected as its admin) is an `http` or
  `https` URL: a host, an optional port and an optional path; no user,
  query, fragment or final `/`; `http` only to a loopback host. It is
  the base of each repository's remote, `<base>/<owner>/<name>.git`,
  which the engine's routing makes (../channel.md, section 4), and the
  host segment of the forge's resource paths: its host, with `:port`
  when it names a port.
- **No forge here.** A forge's kind, mode, place, git program, bot
  and number are its record's in temper's store (../forge/engine.md, 1), made
  at runtime: an owned Forgejo's from `owned` by `SetUp`, the
  playground's fake forge's from `fake` at its start, a GitHub forge's
  from `github` by `Connect`, and a Forgejo connected as its admin from
  the address, base and git program the person gives (../setup/reconcile.md, 5).
  A first run's document names no forge.
- **No credential is configured:** a forge's API token, git token,
  webhooks' secret, App's key and user are records, kept from the
  client or by the engine's setup host (../credentials.md, section 2). A
  forge's kind and `api` are its record's, checked against its `Origin`
  row at start: rows made for another kind or address leave the forge
  not connected, named, and erase nothing (../credentials.md, 2.1).
- **Git identities are not configured either:** each kept forge has one,
  its number the forge's, its login and author its record's bot
  (../credentials.md, section 4). The largest value an identity can take,
  encoded (../channel.md, section 6), must fit the channel's bound on a
  credential's value, or the start is refused, naming the limit: its
  login and author at their bounds, with a Forgejo forge's token at
  `records.value_bytes`, or a GitHub forge's list at
  `forges.installations` entries, each at its largest (`limits.md`).

**`landing`:** the projects' ways of landing, `projects` alone, each
entry its `project`, `by`, `gates` and the turn's and the tiers' project
settings below. The landing rules are settings in the store, temper's
kinds 1 and 2 (`settings.md`): a `deployment` member, or a project's
`rules`, is refused as `settings.md`, 4 says. What `landing` keeps leaves the
document with kinds 4 and 5, the ways and checks `../../domain/landing.md`,
13.1 moves there (`settings.md`, 5, step 2).

```
"landing": {
  "projects": [ {"project": u32, "by": "queue" | "turn", "gates": [ gates ]} ]
}
```

- **`by` and `gates`,** until kinds 4 and 5
  (`settings.md`) are: how the project lands, `queue` when absent; under
  `turn`, each repository's test gates; and, by either way, its tiers;
  a `gates` entry being
  `{"forge": u16, "repository": u64, "gates": [{"name": string,
  "workflow": string, "ignored": [string], "blocking": boolean}],
  "tiers": [{"name": string, "workflow": string, "ignored": [string],
  "artifact": string, "format": "junit" | "lines", "branch": string,
  "span_ms": u64, "description": string}]}`,
  a repository at most once (`../../domain/landing.md`, 13.1;
  `../../domain/after-landing.md`, section 11). A tier's `branch` is the
  branch it follows, by its name on the forge, the repository's landing
  branch when absent, or `goal`, the goals' own branches, a tier that
  judges no landing (`../../domain/after-landing.md`, section 2). Its
  `span_ms` is the least time between the starts of two of its runs on
  one branch, none when absent or 0.
- **A tier's `description`:** what the tier checks, in words written with
  it ("the slow suites"), which its stop's card says
  (`../../domain/forge/change/branch-path.md`) and `see settings gates` reads
  (`../../domain/agent/ask-temper.md`, 1): 1 to 120 bytes of UTF-8 holding no
  control character, none when absent. One empty, past 120 bytes, not
  UTF-8 or holding a control character refuses the start at its member.
  It is the words saved with a tier (`../../draft/whole-story.execution.md`,
  P3e), kept in the document as the tier is until kind 5 holds tiers
  (`settings.md`, 2), so that whoever writes the tier writes them; Ask temper, which
  proposes no tier, writes none. The engine decides nothing by them.
- **The delivery checks' bound,** beside `by`, by either way:
  `check_timeout_ms`, a u64 of milliseconds from 1,000 to 86,400,000,
  none when absent: the wall bound each delivery check is given in a run
  delegated for one of the project's changes, its producers', repairs' and
  resolutions', a goal's landing into the landing branch's among them, in
  place of the agent's `check_timeout`
  (`../../domain/agent/charter.md`, section 4; smith's `domain/run.md`,
  8.1). A project whose landings are goal-sized sets it above the agent's,
  so a landing's repair has room to check. It need not fit a charter's
  reserve: a check is cut at its run's end. One past its bounds refuses the
  start at its member, `application.landing.projects[i].check_timeout_ms`.
- **The tiers' project settings,** beside `by`, by either way:
  `tier_retries`, a u8, 2 when absent; `verification_bound_ms`, the
  project's verification bound, a u64, 7,200,000 (two hours) when absent
  (`../../domain/after-landing.md`, sections 3 and 6).
- **The merge job's command,** beside `by`, by either way: `merge_job`,
  a relative path of at most 4,096 bytes, without `..`, in each
  repository the merge job merges, none when absent: the project's merge
  command (`../tooling.md`, 4.1), and with it the merge job before each
  resolution of its changes and catch-ups into its goals' branches
  (`../../domain/forge/change/repairs.md` and `../../domain/forge/README.md`, 6). A path past its bounds
  refuses the start at its member.
- **The close job's command,** beside `merge_job`, by either way:
  `close_job`, a relative path of at most 4,096 bytes, without `..`, in
  each repository a goal's branch of the project is in, none when
  absent: the project's close command (`../tooling.md`, 4.2), and with it
  the close step of each goal's closer (`../../domain/forge/branches.md`, 2). It
  is read as `merge_job` is, its errors named at
  `application.landing.projects[i].close_job`; a path past its bounds
  refuses the start at its member.
- **The turn's project settings,** beside `by`:
  `prepare` and `key`, paths in the project's home, or absent for
  temper's own; `home_revision`, the home's commit they run at;
  `prepare_timeout`; `pins`, a list of `{"downstream": REPOSITORY,
  "upstream": REPOSITORY}`, each a repository as `gates` names one;
  `pins_follow_main`, false when absent; `dependencies_join`, true when
  absent (`../../domain/landing.md`, 4.1 and 5.3).

A landing rule, the value of kinds 1 and 2 (`settings.md`, 2), is a `LandingRule`,
member for member:

| Member | Type | Meaning, and bounds |
|---|---|---|
| `connector` | u16 | the forge connector's number, the one `engine.connectors` names |
| `kind` | u16 | the effect kind the rule governs: 4, the forge's merge, the only kind a landing rule takes |
| `pattern` | jig's pattern, as jig's `rules` and `projects` write one: `{"segments": [string], "last": "exact" or "open", "value": string}` | the resources the rule covers, as jig's authority matches them: the exact base segments, then one more segment equal to `value` (`exact`), or beginning with it, with all below it (`open`). The forge's resource path is `forge`, the forge's host (as `git` gives it), the repository's owner and name, then `branch` and the branch's segments |
| `enforce` | boolean | true makes each criterion below a requirement every landing it covers must meet (jig's `domain/authority.md`, section 10; `../../domain/forge/change/gates.md`, 2); false leaves them to the change procedure alone |
| `ci` | boolean | CI must have passed at the change's head |
| `up_to_date` | boolean | the change's head must contain its base's tip |
| `gates` | list of `{"number": u32, "blocking": boolean, "freshness": "exact" or "clean"}` | the gates every change it covers asks for (`../../domain/forge/change/gates.md`, 1): an agent's review, or one of a repository's checks when it has no CI; at most the forge's `change_policy.gates` |
| `approvals` | list of `{"role": u32, "people": u32, "freshness": "exact" or "clean"}` | `people` distinct people holding `role` in the project approve the change, each by a review on the forge (`../../domain/forge/change/gates.md`, 2) |

- **`freshness`,** a gate's or an approval's, says at which later heads
  a verdict still counts: `exact`, the head it was given at; `clean`,
  every head reached from it by updates temper made that merged cleanly
  (`../../domain/forge/change/gates.md`, 1). It is not jig's freshness of an observed
  requirement, the age within which its facts must have been read: CI,
  gates and approvals are guarded, and up to date is observed with a
  freshness of 0 (`../../domain/forge/change/gates.md`, 2).
- **Bounds:** a project appears in `projects` at most once, and must be
  one of `engine.projects`; a rule's bounds are those the forge judges
  it by (`settings.md`, 2).
- **Feeds** `Config.landings`: each project's way, gates, tiers, merge
  job and turn's settings.

**`clients`:** the clients' listener, which `temper-service` owns
(`../../client/engine.md`), and the browser's bundle it serves
(`../../client/web/README.md`, 2.2):

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `listener` | an address, jig's format | a socket address; on loopback (in `127.0.0.0/8`, or `::1`) in standalone, and wherever `engine.clients` enables the printed link or the approval (jig's `protocol/service.md`, section 3); a fixed port, never 0, for a GitHub forge to be connected | the listener on which jig's clients' component is mounted with jig's default HTTP mapping, under `/jig`; the mapping's Host allow-list; the address of the sign-in links `temper-box serve` prints (`../../client/README.md`, section 4); and temper's forge routes beside it, whose addresses a GitHub App's redirect and setup URLs name (`../../client/README.md`, the routes) |
| `bundle` | an absolute directory, or `null` | a bundle of skein's bundler, built apart (`scripts/bundle.sh`): its page and its digest's directory; its files, and the binary's own, within `clients.bundle_files` and `clients.bundle_bytes` (`limits.md`) | the browser's page and bundle, made at start and served on the clients' listener (`../../client/web/README.md`, 2.2): `null` serves the bundle the binary holds (2.6), or, from a binary built without one, the page for no bundle; a directory serves its bundle in place of the binary's. A document naming an old `share/temper/bundle` keeps serving that directory, older than the binary once it is rebuilt: set it to `null` |
| `public` | an origin, or `null` | `https://HOST[:PORT]`: HOST a lower-case DNS name, the port not 443 when written, no user, path, query, fragment or final `/`; `http://` only to a loopback host, for a test; within 253 bytes | the address people's browsers use, behind a TLS proxy (`../sign-in.md`, 2.2): the mapping's Host allow-list holds its host beside the listener's (jig's `Settings.public`, the host with its port where not 443), jig's cookie is `Secure` by it, and every address temper gives out is the origin's: Google's return, a GitHub App's redirect and setup URLs. `null`: the listener's address alone, `public: None` |

- **The printed link and a terminal's approval** are jig's members,
  `engine.clients.link` and `engine.clients.approval` (jig's
  `protocol/configuration.md`, section 3, clients): `temper-box init`'s
  documents, so a box's, and the owned example enable both
  (`../setup/init.md`, 3), the approval being how the TUI signs in from the
  person's browser (`../../client/README.md`, 4.1). Neither runs off
  loopback: a start refuses a clients' listener off loopback in
  standalone (`README.md`, section 2), whichever is enabled, naming
  `application.clients.listener`; the approval absent is the approval
  off, and its ask is refused.
- **A fixed port for a GitHub forge:** a GitHub App's `redirect_url`
  and `setup_url` name the clients' listener's address, and GitHub's API
  cannot change them once the App is made. A GitHub `Connect` on a
  listener at port 0 is refused, naming `application.clients.listener`
  (../forge/engine.md, 1); `temper-box init` writes a fixed port (../setup/init.md, 3).

**`sign_in`:** the people's sign-in with Google (`../sign-in.md`), or
`null` for none, never missing: with `null`, the browser's signed-out
page keeps its printed link's line, and `/sign-in/google` and its return
answer not found.

```
"sign_in": {
  "google": {"provider": 2, "client": "1234-abc.apps.googleusercontent.com",
             "secret": "google-client.secret", "endpoints": null},
  "members": [{"address": "sam@example.com", "login": "sam"}]
}
```

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `google.provider` | u16 | a provider of jig's `engine.providers` of kind application | the identity's provider, in `Mint` and in jig's `owners` (`../sign-in.md`, section 3) |
| `google.client` | a string | Google's client id: 1 to 256 bytes of visible ASCII | the client's `client_id` |
| `google.secret` | a data path | a secrets' file (`README.md`, section 5), its bytes 1 to `sign_in.secret_bytes`, visible ASCII | the client secret, in the token request's body |
| `google.endpoints` | an object, or `null` | `null` for Google's own (`../sign-in.md`, 3.1); otherwise `{"issuer", "authorization", "token", "keys", "transport"}`, `null` in a deployment; a value is for a test, or any issuer that is not Google, and names no deployment: each an `https` URL, or `http` to a loopback host, within `sign_in.url_bytes`; `transport` as a forge's, `"plaintext"` to loopback alone or `{"tls": {"server_name", "trust"}}`. Each host resolved at start; one that does not resolve refuses the start, naming it | the registration's endpoints and the fetcher's origins (`../sign-in.md`, 6.3) |
| `members` | a list | 1 to `sign_in.members` | who may sign in, each `{"address", "login"}` |
| `members[].address` | a string | 3 to 254 bytes of visible ASCII, one `@` with bytes on both sides; unique among members with ASCII letters folded to lower case | the identity's subject, exactly as written; compared with Google's verified `email`, folded |
| `members[].login` | a string | 1 to 40 bytes of ASCII letters, digits, `-`, `_` and `.`, its first and last a letter or a digit; unique, folded | the login in `Mint`, and the name where Google gives none |
| `forges` | a list, or absent for none | each `{"forge", "client"}`: `forge` a forge's number, as the Forges page and the `temper-box forge` commands name it, `client` the id of one of jig's `engine.clients.provider.clients`, in visible ASCII (the page's metas carry it); each forge and each client at most once; with no provider, refused naming `engine.clients.provider`. The forge, a record, is checked once its records load: a Forgejo, whose browser origin's `/user/oauth2/temper/callback` is exactly the client's `redirect_uri` (`../openid-provider.md`, 7) | the forges that sign their people in through temper (`../openid-provider.md`): the page's `temper-forge-sign-in` metas, "Open Forgejo ↗", an owned Forgejo's `app.ini` (../setup/owned.md, 2) and `temper-box forge sign-in` (../setup/owned.md, 8) |

- **jig's owners of Google's provider are members:** each of
  `engine.owners` whose provider is `google.provider` has a member's
  address as its subject, or the start is refused, naming the owner; and
  one of them at least, the first, signs in by the printed link
  (`../sign-in.md`, 2.3).
- **Google's provider is not the link's:** `google.provider` differs
  from `engine.clients`' link provider, or the start is refused, naming
  both.

**`limits`:** temper's profile and overrides (`limits.md`).

**`webhooks`:** the webhooks' listener, the forge component's own
(../forge/README.md, section 4), or `null` for none:

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `listener` | an address, jig's format | a socket address on loopback (in `127.0.0.0/8`, or `::1`), as the clients' listener's in standalone; unique among the deployment's listeners: neither the clients' listener's nor the machine's owned or fake `listen` (`machine`, above) | the forge component's listener for Forgejo's deliveries, plaintext HTTP on skein's HTTP server, one path per Forgejo forge whose records hold a webhooks' secret (../credentials.md, 2.1), `/hooks/<number>` by the forge's `number`; the URL of each hook the setup's plans make (../setup/reconcile.md); an owned Forgejo's `[webhook] ALLOWED_HOST_LIST` (../setup/owned.md, 2) |

- **A delivery is a hint,** never a fact: polling is the same with
  webhooks or without (../forge/README.md, section 5), and finds what a delivery
  lost, refused or never sent told of.
- **A forge on another host polls:** its deliveries would need TLS on
  the listener (../README.md, section 11; ../setup/README.md, section 9).
- **A GitHub forge has no path:** its hints come from its App's
  delivery log, which the forge component reads, with no address GitHub
  must reach (../forge/README.md, section 4).

**`demo`:** the playground's scripted model, or `null` for none, never
missing (`README.md`, section 8): its `llm`, the fake LLM the box runs, with its
port (`listen`), its LLM provider of `engine.llm` (`provider`), its
key's file (`key`) and its scripts (`asked`). Its members, their bounds
and their checks against the document's LLM providers are ../setup/playground.md
's, `temper-fake`'s. `temper-box serve` takes the provider, the key's
path and each script's cue, and reads the key at the playground's start,
which makes the scripted account with it (../setup/playground.md); with `null`,
the playground is not offered.

**What the engine root's `Config` takes,** field by field:

| Field | From |
|---|---|
| `core` | `engine`, by `jig-protocol-config`, with the deployment's id and seed |
| `landings` | `application.landing`'s `projects` |
| `kinds_stock` | not configured: the stock defaults of temper's kinds 1 to 3 (`stock.md`), which the root gives the forge at each start (`settings.md`, 2); kinds 6 and 7 have none |
| `forge.writers` | not configured: each forge's writer comes from its connect, its user as the routing tells the root (../credentials.md, section 5) |
| `forge.namespace` | not configured: the root derives it from the deployment's id |
| `identities` | not configured: each kept forge's `GitIdentity { forge, credential }`, its credential the forge's number, comes with its record (../forge/engine.md, 1) |
| `machine` | `application.machine`'s places, from which the forge connector makes a new forge's record (../forge/engine.md, 2) |
| `playground` | `application.demo`'s provider, with `setup.deadline_ms`; none for `null` (../setup/playground.md) |
| `under_box` | whether `temper-box internal serve` was given `--box` (../setup/box.md, 1) |

Beside the root: the forge component takes each kept forge's number,
kind, address and transport from its record, and its kind's log
origins (../forge/README.md, 2.6), with the access the engine's routing builds
from the forge's credentials (../credentials.md, section 5), and the
webhooks' listener, `webhooks.listener`; the engine's setup host takes
each kept forge's record, with an owned or fake one's setup entry,
and the machine's GitHub place for GitHub's plans (../forge/engine.md, 6); the
identities' table takes each kept forge's identity, its number, login
and author from its record, its value from its forge (../credentials.md,
section 4); the engine's routing takes each forge's `git` base and git
program; the clients' listener takes `clients.listener`, and its forge
routes each GitHub forge's `web`.

`merge_job`, when present, is a string of 1 to 4,096 bytes, with no leading
`/`, NUL or `..` segment. An invalid form refuses the start at
`application.landing.projects[i].merge_job` as a form error; a length past
the ceiling is a bound error. Leaving it out configures no merge command
(2026-11-04).

A tier's name is unique among both the gates and tiers of its repository.
Its artifact name is non-empty, at most 32 bytes, and contains no `/`. The
repository's tier count remains bounded by `forge.tiers.tiers` (2026-11-04).
