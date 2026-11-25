# Credentials

Provisional, 2026-10-03, revised 2026-11-02. temper's credentials: which
secrets temper's processes hold, where each comes from, who may use it,
and how it reaches the process that does. LLM accounts, workers' secrets
and people's sign-ins are jig's (jig's `protocol/README.md`, section 8;
jig's `domain/accounts.md`; jig's `protocol/llm.md`, sections 5 and 6),
and temper adds `temper-box program-login` (`credentials-llm-accounts.md`, section 1); temper's own are its
forges' credentials, records in jig's secrets' store under temper's tag
(jig's `protocol/store/secrets.md`), and the git identities made from
them. A forge's credentials come from the client, in a request carrying
a value (jig's `protocol/client/README.md`, section 3), or from the engine's
setup host, never from configuration (configuration/README.md, section 5); a
git identity's value on the channel is channel.md's (`channel.md`, section 6); git's
use of it is git.md's (`git.md`, section 5); the forge's setup's own secrets are
setup/README.md's (`setup/README.md`, section 5). What is open is listed in `credentials.md`, section 8.

## 1. In one page

- **Names through the domains, values beside them.** A domain names a
  credential by its number and a generation, and never holds its value,
  so no trace, snapshot, record or log of a domain can leak one. Values
  live in the protocol layer's tables, filled in as a record is encoded
  and taken out as it is decoded.
- **LLM accounts are jig's,** made from the client: a sign-in with the
  LLM provider, a key, or a program's login. temper writes no OAuth
  exchange and configures no account, and adds `temper-box program-login`,
  which signs a program's login in on a host (`credentials-llm-accounts.md`, section 1).
- **Every forge credential is a record in jig's secrets' store,** under
  temper's tag, one row per forge and item: the forge's tokens, a GitHub
  App's id, slug and key, the bot's user, a Forgejo forge's webhooks'
  secret, the accepted installations (section 2.1). A connect from the
  client makes them, from the values pasted in its request or through
  the engine's setup host; nothing configures them, and `temper-box serve`
  reads no forge credential from a file but an owned Forgejo's handover,
  once. The main store keeps each forge's writer, which is not secret.
- **Kept before used, read once at start.** A new value is durable in the
  store before the forge component or a worker gets it; one load at start
  builds every forge's access before the forge component starts.
- **A pasted value never reaches a domain.** jig keeps a request's
  carried value under temper's tag before the core sees the request, and
  the core gets its number and its digest only (section 2.2).
- **A forge's access is the forge component's:** Forgejo's API token and
  its webhooks' secret, or a GitHub App's id and key and the
  installations' tokens it mints, with the forge's user; never granted,
  never sent to another forge or host (section 5). GitHub has no
  webhooks' secret: its changes are read from the App's delivery log.
- **Git identities are temper's application credentials,** one per
  forge: Forgejo's kept git token, or a GitHub App's installations'
  tokens, minted about hourly and never kept; each with a login and an
  author. Each change of value is a new generation, granted with each
  assignment that needs one (`credentials-git-identities.md`, section 1).
- **Git gets its credentials per invocation,** in that process's
  environment only, never in an argument, a URL or a file.
- **Least power.** The forge component never holds an admin's token: the
  engine's setup host holds a managed or owned Forgejo's only while a
  plan runs, for reconcile, for access at an adoption and for the
  picker's reads (a `Reachable` plan), and Forget erases it, making the
  forge connected. The picker's read of any other forge takes its API
  token, or on GitHub a token minted for the page, held while the plan
  runs and dropped. A worker gets the git identity alone, never the
  forge's access.
- **A forge's credential failing stops that forge, never the engine.** A
  credential the forge refuses, or records that do not fit the forge's
  record, leave that forge stopped or not connected, named, with
  Replace or Connect offered; other forges and the engine run on
  (section 5).
- **A forge removed takes its credentials with it:** its rows are
  erased at once, its access dropped and its git identity withdrawn
  (2.1; forge/engine.md, 3).
- **Workers keep nothing between runs.** A worker's configuration holds
  no credential but its own secret; every other comes with assignments.

## 2. The credentials

| Credential | Owner | Kind | Source | Held by | Used by | Changes |
|---|---|---|---|---|---|---|
| an LLM account's source | jig | a signed-in record (refresh token, access token, expiry), a key, or a program's login | a sign-in or a key from the client; a program's login, `temper-box program-login` on each host (`credentials-llm-accounts.md`, section 1) | jig's secrets' store, lent by skein's accounts driver, the access tokens and keys in jig's accounts' table; a program's login, the program, in the source's configuration directory | the worker's agent processes, through grants | the driver refreshes a signed-in record; a key's holder replaces it from the client; the program renews its own login |
| a git identity | temper | a token with its login, and an author: on Forgejo the `GitToken` record; on GitHub the accepted installations' tokens | the forge's records and tokens, and on Forgejo its record's bot (`credentials-git-identities.md`, section 1) | the identities' table on the engine, and on the worker from grants | the git component's invocations | a new generation at each change of value (`credentials-git-identities.md`, section 1) |
| a forge's API token | temper | a Forgejo token, the `ApiToken` record | pasted from the client at a connect, or made by the setup host's reconcile when the forge is managed or owned (setup/reconcile.md) | jig's secrets' store; the forge component, in the forge's access | that forge's API requests | replaced from the client (`credentials.md`, section 5), or made again by reconcile when it does not answer as the bot |
| a GitHub App's key | temper | an RSA private key, PKCS#1 DER, 2048 bits, the `AppKey` record beside `AppId` | the manifest's conversion; or pasted from the client as GitHub gives it, PEM, which the setup host decodes to DER | jig's secrets' store; the forge component, in the forge's access; the setup host, while a plan runs | signing the App's own requests, and minting its installations' tokens | replaced from the client with a key of the same App (section 5) |
| an installation's token | temper | a GitHub token, expiring within the hour | minted by the forge component with the App's key, one per accepted installation (section 5); by the setup host, for a page of the picker | the forge component; the forge's git identity; the setup host, while a page is read; never kept | that installation's API requests; git's, through the identity; the picker's page | minted again `renew_before` ahead of its expiry; a picker's page's, dropped with the page |
| a worker's secret | jig | a shared secret | the worker's secret file; the engine's digest file | the worker; the engine, its digest | the link's opening | an operator replaces both |
| a sign-in's secret | jig | a bearer token | the printed link, then the sign-in | `jig-client` in `temper-box open`, which keeps it in its directory; the engine, its digest | each of the client's exchanges and its stream | the sign-in ends or expires |
| a deployment's VAPID key | temper | a P-256 private key, PKCS#8 PEM | the operator, a data path `application.push.key` names (configuration/engine.md) | the engine, read at start | each push's token, signed by skein's `sign_es256`; its public half the clients' (`../domain/forge/attention.md`, 7.3) | never: a new key leaves every device subscribed to the old one, each turned on again by its person |
| a deployment's mail password | temper | a password, designed and not built (`../domain/forge/attention.md`, 7.4) | the operator, a data path `application.mail.password` names | the engine, read at start | the SMTP server's sign-in | the operator replaces it and restarts |
| a Forgejo forge's webhooks' secret | temper | a shared secret, the `WebhookSecret` record; a GitHub forge has none | pasted at a connected Forgejo's connect; or drawn by the setup host and set on the hooks of a managed or owned Forgejo served the listener (setup/reconcile.md, 3), by the plan that makes them | jig's secrets' store; the forge component's webhooks' listener, on loopback; the forge's hooks, which sign each delivery with it | checking each delivery's signature (forge/README.md, section 4) | replaced from the client with the forge's tokens; reconcile sets a managed or owned Forgejo's hooks' secret to the record's |
| a site admin's token | temper | a Forgejo token, the `AdminToken` record | pasted from the client at a managed connect; an owned Forgejo's handover file, taken once and removed (setup/README.md, section 5) | jig's secrets' store; the setup host, only while a plan runs | reconcile, access at an adoption, and the picker's reads on a managed or owned Forgejo (a `Reachable` plan) | erased by Forget, after which the forge is connected |

- **Every forge credential the engine uses is in jig's secrets' store,**
  under temper's tag (`SecretOwner::Application`), beside jig's own:
  sign-ins' and services' digests, LLM accounts' records and keys (jig's
  `protocol/store/secrets.md`). No forge credential is a file
  `temper-box serve` reads, but an owned Forgejo's handover admin's token,
  read once, kept as a record, and removed.
- **A GitHub forge has no webhooks' secret.** The setup host points the
  App's hook at `https://<slug>.invalid/`, a name that never resolves; the
  conversion's webhook secret is never kept, and a pasted App's connect
  draws none. The forge component reads changes from the App's delivery
  log (forge/README.md, a GitHub App's delivery log).
- **The forge's setup's own secrets** (the bot's and temper's admin's
  passwords, the person's first password on an owned Forgejo, the
  runner's secret, Forgejo's own secrets) are setup/README.md's (`setup/README.md`, section 5):
  kept in the forge's directory or nowhere, never in jig's secrets'
  store.

### 2.1 The records

- **Where:** jig's secrets' store, under temper's tag,
  `SecretOwner::Application`, one row per forge and item, written and
  read by the engine's protocol layer alone, through `Request::Secrets`
  (`Keep`, `Erase`, `Get`, `Load`). The items that are not secret (ids,
  the App's slug, the bot's user, the accepted installations, the
  origin) sit beside the secret ones, so that one load builds a forge's
  access, one durability answer (`Kept`) covers each of the setup host's
  writes, and one erase forgets a forge. The main store's application
  rows are the root's, and hold none of them.
- **The number** (`u64`): `family << 48 | forge << 16 | item`. Family 1 is
  a forge's records; other families, up to `0x7FFF`, are free for
  temper's later records. temper's own numbers keep the top bit clear,
  below 2^63, where jig numbers carried values under temper's tag itself
  (section 2.2). `forge` is the forge's number (`u16`), given when the
  forge's record is made and never given again (forge/engine.md, 1): adding
  or removing forges never moves one forge's records to another, and a
  forge set up again is a new forge, with a new number and no rows.
- **The items** (`item`, `u16`):

  | Code | Item | Value | Forges |
  |---|---|---|---|
  | 1 | `AdminToken` | a site admin's token | Forgejo, managed or owned |
  | 2 | `ApiToken` | the bot's API token | Forgejo |
  | 3 | `GitToken` | the bot's git token | Forgejo |
  | 4 | `User` | the bot's user id: on Forgejo the bot's, on GitHub the App's bot user | both |
  | 5 | `WebhookSecret` | the hooks' shared secret | Forgejo alone; optional on a connected Forgejo |
  | 6 | `AppId` | the App's id | GitHub |
  | 7 | `AppSlug` | the App's slug | GitHub |
  | 8 | `AppKey` | the App's private key, PKCS#1 DER, 2048 bits | GitHub |
  | 9 | `Accepted` | the accepted installations' ids, at most `forges.installations` | GitHub |
  | 10 | `Origin` | the forge's kind and `api` address the records were made for | both |
  | 11 | `Pasted` | a pasted App key's PEM, in a pasted bundle alone (2.2), never a record: a row under 11 is unreadable | GitHub |
  | 12 | `AppMade` | the id of the App the manifest flow made, written first by `Convert` (setup/github.md, 2) and kept with `AppId`, since no write spans this store and the forge's record. A pasted App never writes it | GitHub, when a manifest made the App |

  Code 0 is never written, and a code never changes its meaning. temper
  maps skein's setup items to these codes; on GitHub, `User` is the
  App's bot user. `Origin` is temper's own, written at each connect.
- **The value:** a versioned `Stored` record of the family
  `temper-secrets` (`temper-protocol-records`,
  `schema/secrets-v1.schema`; README.md, section 7), its item one of the
  table's:
  - the tokens (`AdminToken`, `ApiToken`, `GitToken`, `WebhookSecret`):
    a secret, at most `token_bytes`;
  - `User`, `AppId` and `AppMade`: a `u64`, never 0;
  - `AppSlug`: text, at most `name_bytes`;
  - `AppKey`: a secret holding PKCS#1 DER, at most `key_bytes`;
  - `Accepted`: the installations' ids, a list of `u64`s, ascending,
    distinct, at most `installations` of them, empty for none;
  - `Origin`, temper's: the forge's kind, Forgejo or GitHub, and the
    `api` address as text, `<host>:<port>`: a socket address as Rust
    writes it (`[<ipv6>]:<port>` for IPv6), a host's name as the
    forge's record holds it, `api.github.com:443` for github.com's.
    Equal origins are the same forge; the text is shown as is.

  A record of another version is an unreadable row (below); a change of
  an item's meaning is a new item, under a new code.
  temper keeps the values opaque, hands them back unchanged on the setup
  host's read, and decodes only what builds an access. A value is at
  most `records.value_bytes` (configuration/limits.md), checked
  against jig's secrets' store's bound at start.
- **A PEM is never a record.** skein's setup plan decodes the
  conversion's key and a pasted key to DER before it writes; a pasted PEM
  sits only in its carried row, which jig numbered, until its run ends.
- **`Accepted`** is one row per GitHub forge, the whole list, rewritten
  at each installation accepted or deleted.
- **Replacement:** `Keep` under the same number replaces; the new value is
  durable before the forge component or a worker gets it.
- **Read at start, whole:** one `Load`, paged, grouped by forge, builds
  each kept forge's access before the forge component starts (forge/engine.md,
  1).
- **Records that do not fit the forge's record** (an `Origin` naming
  another kind or `api` address, an `AppKey` under a Forgejo forge, an
  `ApiToken` under a GitHub forge): that forge is not connected, named on
  the deployment's page, "its records were made for <kind> at <address>";
  nothing is erased, and connecting again overwrites them, its user
  aside (below).
- **A connected Forgejo's user stays:** connecting it again overwrites
  the rows its paste and its run write and keeps the others, but its
  kept `User` row binds: its `Check` compares the user the new tokens
  answer as with it, and tokens of another user stop the run
  `OtherUser`, the forge as it was (setup/README.md, section 4). A forge's
  writer is its user (section 5), which skein's client never changes
  under a writer (skein's `forge/client-effects.md`, 1.1): another user's
  tokens are another forge, connected anew once this one is removed.
- **An unreadable row** (an unknown version, bytes skein's decoder
  refuses): that forge is not connected, named, "its records are
  unreadable"; other forges run on; nothing is erased.
- **An incomplete set** (a crash inside a conversion: an `AppId` kept, no
  `AppKey`): the forge is not connected; the next Connect overwrites it.
- **Erased with the forge, at once:** its removal erases each of its
  rows, `AdminToken` through `Origin`, then `AppMade`, one after another, each durable
  before the next, an absent row among them, since a run may have had a
  write in flight; a removal a restart cut erases again at the next
  start (forge/engine.md, 3). The setup host forgets the forge first, and
  each erase follows any write its run had in flight.
- **Rows of a forge with no record** are left as they are, and named
  once in the log at start (section 8).
- **A mode is no row of this store.** It is the forge's record's, in
  temper's main store (forge/engine.md, 1): owned, fake, managed or
  connected, as the act that made the forge gave it. A GitHub App made
  from the manifest and one pasted end in the same rows.
- **Never kept:** installations' tokens; a GitHub forge's git tokens; the
  manifest's states and pages; the conversion's PEM; the App's OAuth
  client and its secret; any generation number.
- **The main store** keeps no credential. It keeps each forge's
  record, its kind, mode, place and bot (forge/engine.md, 1), and its writer,
  neither secret: the writer given at runtime (`credentials.md`, section 5), kept by
  skein's forge client as its own record (skein's `forge/client-effects.md`,
  1.1), `Stored::Writer` under `Key::Writer(forge)`, temper's small
  store's tag 16. It is restored at start, so effects
  restored with it never meet `Forbidden`.

### 2.2 Pasted values

- **One value per request.** A client request may carry one value (jig's
  `protocol/client/README.md`, section 3). jig's clients' component takes it out
  as it decodes the request, keeps it in the secrets' store under
  temper's tag before the core sees the request, and hands the core its
  number and its digest. The value never reaches the core, a root, a
  document or a log.
- **The requests that carry one** (`../client/README.md`, the FORGE
  requests), each bundle holding the items its request takes and no
  other: `Connect` with `ForgejoTokens`, `ApiToken` and `GitToken`, and
  `WebhookSecret` when the person gives one; with `ForgejoAdmin`,
  `AdminToken`; with `GitHubApp`, `Pasted`, the App's key as GitHub
  gives it, PEM (the id rides in the request); and `Replace`, on a
  connected Forgejo `ApiToken`, `GitToken` or both, and `WebhookSecret`
  when its hooks' secret changes, on GitHub `Pasted`, the App's new key.
  A bundle that lacks an item its request needs, or holds one it does
  not take, stops the run before any plan, in its way's words: "the
  pasted value is not a key" (an App's), "the pasted value is not a
  token" (an admin's), or "the pasted values are not a Forgejo's
  tokens" (`ForgejoTokens`, and a Forgejo's `Replace`); its carried
  value is erased as at any run's end. A
  GitHub App has no webhooks' secret: a pasted App's connect writes
  `AppId`, `AppSlug`, `AppKey` and `User`, and sets the App's hook to
  `https://<slug>.invalid/` (section 2).
- **The number is jig's.** jig numbers each carried value itself, under
  temper's tag from 2^63 up, from a next number it keeps durably in a
  counter row of its own, which is not one of temper's rows and never
  reaches temper's `Load`, and never reuses. The request names no
  number and the client draws none; the core gets the number and the
  digest. A number already holding a row is refused at decode, never
  overwritten. A forge's records lie below 2^63 (section 2.1), so no
  request, refused or not, overwrites or erases them.
- **A carried value left unused** is named once: when the core leaves
  one unused, its request refused, replayed, joined to another or cut
  by a stop, it emits one output naming the number, and temper's root
  forwards it to jig's clients' component, which erases the row.
- **Only the core's decisions** replace or erase a kept value: a carried
  row's erase when it is left unused, at its run's end, and at start, is
  a decision of the core's; the routing carries out those at a run's end
  and at start.
- **The value:** temper's bundle, the `Pasted` record of the family
  `temper-secrets` (`temper-protocol-records`,
  `schema/secrets-v1.schema`), versioned as `Stored` is: a list of
  items in the codec's own form (skein's `data/codec/README.md`), at most three,
  each one of 2.1's items or `Pasted` (code 11, a pasted App key's PEM)
  and each at most once, in any order; at most 4 KiB in all, jig's
  bound on a carried value, and at most `records.value_bytes`. A pasted
  App's PEM key, under 2 KiB for GitHub's 2048-bit keys, fits, and so
  do a Forgejo's two tokens and its hooks' secret. A list with no item
  or an item twice, an item empty or past its bound, or bytes the codec
  refuses stop the run as a bundle of the wrong items does (above); a
  bundle of another version, "pasted with another version of temper:
  paste it again".
  It lives until its run ends. The codec's own form, not a layout of
  temper's own bytes, so that the bundle has a schema's bounds, built-value
  round trips and version check, its items the records' own (decided 2026-11-03).
- **The core** keeps the number and the digest in the connect's own
  record, no secret, and checks the person's role, the forge's kind and
  mode, and that no run holds the forge. Refused, it answers so, and
  names the value left unused (above).
- **The run:** accepted, the root asks the routing to run the connect's
  plan with that number. The routing gets the row, checks its digest is
  the request's (else the run is refused, "the pasted value changed"),
  and serves the setup host's read of each pasted item from it before the
  forge's rows. When the run ends, done or failed, the core decides the
  row's erase, which the routing carries out; on done, the items the run
  wrote, and each pasted item but `Pasted` (the tokens and the secret
  the plan was given, which it keeps: skein's `forge/setup/forgejo.md`, 1.6),
  are kept under the forge's family-1 numbers first, after the run's
  writes.
- **A restart** drops runs, whose states are in memory: the core decides
  the erase of each carried row under temper's tag found at start
  (loaded from 2^63), and the forge of a connect that was running reads
  as its records say: not connected when its paste wrote nothing, and
  nothing tells the person the connect was cut. Saying "interrupted;
  paste again" waits for a stored connect (section 8).

One pasted item alone holds at most 4,085 value bytes, the carried value's
4,096 bytes less the codec's version, one-item list count, variant and
length framing. A file for one pasted credential whose value is outside 1 to
4,085 bytes is refused with `credential file must contain 1 to 4085 bytes`;
a multi-item tokens bundle must also fit the aggregate carried-value bound
(decided 2026-11-03).

## 3. LLM accounts

Moved to [`credentials-llm-accounts.md`](credentials-llm-accounts.md), section 1.

## 4. Git identities

Moved to [`credentials-git-identities.md`](credentials-git-identities.md), section 1.

## 5. The forges' access

- **One access per forge,** built at start from its records (section
  2.1) and held by the forge component alone:
  - **Forgejo:** the `ApiToken`, in the `Authorization: token` header of
    each request to that forge's API (forge/README.md, 2.1); the `User`
    record's id; the `WebhookSecret`, when there is one.
  - **GitHub:** the App's id and key (`AppId`, `AppKey`), which sign the
    App's own requests, and by which the component mints a token for
    each installation the `Accepted` record names, expiring within the
    hour and minted again `renew_before` ahead of its expiry (skein's
    `forge/connection.md`); the `User` record's id, the App's bot's; no
    webhooks' secret. The App's hook points at `https://<slug>.invalid/`,
    and the component reads changes from the App's delivery log (forge/README.md,
    a GitHub App's delivery log).

  Each request goes over TLS unless the forge's record names plaintext
  to loopback (forge/engine.md, 1).
- **Never granted, never stored elsewhere:** an access's values are in no
  grant, root, record of the main store, fact or log, and are never sent
  to another forge or another host. An installation's token is never
  kept: a restart mints new ones, and the one the setup host mints for a
  page of the picker is dropped with the page.
- **Its user, checked.** A connect's run finds the user its credentials
  answer as, and keeps its id (`User`) with the `Origin` the records
  were made for. A connected Forgejo's two pasted tokens must
  answer as one user, not a site admin; a replacement, and a connected
  Forgejo connected again, must answer as the forge's kept user, and on
  GitHub be a key of the same App (setup/README.md, the setup host's plans).
  A connected Forgejo's login is its record's bot's, kept by its
  connector at each `Check` (forge/engine.md, 2). The forge component's first exchange with each forge
  checks its access answers as that user (skein's `forge/connection.md`,
  section 3): until it passes, the forge's calls wait in the component,
  and a check that goes unanswered is made again (skein's
  `forge/connection.md`, 3.3).
- **A forge that refuses its access stops that forge, never the
  engine.** A `401`, a `403` for the credential, or an answer as another
  user, ends in `Stopped { forge, why }` from the forge component; the
  routing hands it to the root, the deployment's page names it, and the
  client offers Replace. That forge's calls fail as forge/README.md classifies a
  stopped forge; its git identity entries are withdrawn at once. Other
  forges, the engine and its runs go on. A wrong
  credential would make temper act as someone else, so the stop holds
  until a replacement answers as the forge's user: never a degraded mode.
  A failure of jig's secrets' store itself stops the engine, as the main
  store's does (jig's `protocol/store/secrets.md`). A restored secret
  of the wrong kind stops it too (jig's `protocol/service-parts.md`, 1.1): the
  store holds what the account's row says it should not.
- **The same user is the forge writer** in the domain (`Writer { forge,
  author: user }`), by which the forge connector tells temper's own
  writes from others': a key's provenance, and its own writes' echoes
  (`../domain/forge/keeping-up.md`, 2; forge/README.md, section 3). It comes from the
  forge's connect, never from configuration: on `Connected` the root is
  told the user (id and login), and the connector takes the writer.
  skein's forge client keeps it in the main store, not secret (`credentials.md`, section 2.1
  ).
- **The admin's token is the setup host's alone,** never the forge
  component's. The setup host holds a managed or owned Forgejo's while a
  plan runs (reconcile at each start and at a managed connect; access at
  an adoption; the picker's reads, a `Reachable` plan, skein's
  `forge/setup/forgejo.md`, 1.8), from its record, and drops it when the plan ends (setup/README.md, the
  setup host in the engine; forge/README.md, the setup's host). The picker's
  page holds repositories only, never the token (`../domain/forge/README.md`,
  the picker). `Forget` erases the record: the forge is then connected,
  the bot's tokens its whole power.
- **Replaced from the client:** `Replace { forge }` carries the new
  tokens or key as its value (section 2.2); the setup host checks they
  answer as the forge's user, each new record is kept over the old, and
  the forge component is given the new access. On a managed or owned Forgejo, reconcile makes
  the bot's tokens again when they do not answer as the bot (setup/reconcile.md
  ).

## 6. Workers' secrets and sign-ins

- **A worker's secret is jig's** (jig's `protocol/channel/README.md`, 4.1). In
  standalone the link is in process, and still opens with the worker's
  name and secret; the files are jig's worker and engine parts'
  (configuration/README.md, section 5).
- **People's credentials are jig's** (jig's `protocol/client/sign-in.md`
  ): jig mints, verifies, expires and revokes them. Its printed
  link signs in the deployment's first owner: `temper-box serve` prints the
  link at start, or `temper box` before it starts serve (`setup/box-supervision.md`, 1),
  `temper-box open` signs in by it; its client holds the
  bearer token it is given and sends it with every exchange, and the
  terminal keeps it in its private directory, to sign in with it again
  (`../client/sign-in.md`, section 1). Every other way of signing in is open
  (`credentials.md`, section 8).
- **No TLS key** in standalone: nothing temper serves leaves the machine.

## 7. Testing

- **No value across a domain boundary:** the one-process world and the
  forge's protocol world search every root's events and requests, every
  fact and every record of the main store for the values their fakes
  issued (LLM access tokens and keys, forges' tokens, Apps' keys,
  installations' tokens, admins' tokens, webhooks' secrets), and fail on
  any (`../testing/tiers.md`, 1.5). After each real-loop story, no
  secret is in a root, a fact, a record but the secrets' store's, a
  request or an answer.
- **No live forge:** every forge these tests reach is skein's fake, and
  every credential a placeholder its fake issued; no test calls
  github.com or uses a real credential. Live calls are skein's opt-in
  conformance's alone.
- **The codecs,** with records drift, built-value round
  trips and fuzzing (skein's `data/codec/golden.md`, 1): the
  records' numbers and each item's value, the pasted bundle, and the git
  identity's value of each kind; an unknown version refused, and each
  item's bytes that skein's decoder refuses (an id of 0, `Accepted` out
  of order, repeated or past its bound) leaving the forge not connected.
- **The identities' table:** step tests of its filling at start, a value
  filled into a grant and taken out on the worker, two generations, a
  push across a change ending on the value it began with, and on GitHub
  the entry picked by the owner's account id, a push with none refused.
- **Git's environment:** step tests of its builder: the header's bytes,
  the author's variables, and no credential in an argument or a URL
  (git.md, section 9).
- **The routing,** in `temper-service`'s own tests (forge/README.md, section
  9): start's load, where a
  forge's unreadable row, records made for another forge, or an
  incomplete set leave it not connected, named, and the other forges
  running; a connect whose carried value is read by its run and erased
  at its end; carried rows erased at a restart; installations' tokens
  renewed, each a new generation; a credential of another user, or one
  refused, stopping that forge alone, named, while the engine and the
  other forges run on; calls waiting until the user check is answered,
  and an unanswered check made again; a removal erasing each of the
  forge's rows in order, each durable before the next, its identity's
  values gone from the table and its grant lapsed, and a removal cut by
  a restart erasing again.
- **Startup:** a secret file readable by another user, or reached
  through a link, refuses the start, naming it.
- **`temper-box program-login`,** step tests: each refusal (a name out of
  its bounds, no program configured, a directory not private or reached
  through a link), each before anything is made; the directory made
  `0o700`; and the exec's program, arguments and environment: the
  caller's kept, `CLAUDE_CONFIG_DIR` the source's directory, the
  program's credential variables removed. The process tier execs smith's
  fake `claude` (smith's `protocol/programs.md`, section 8) and finds its
  login in the source's directory (`../testing/process-tier.md`).
- **LLM accounts' stories** in the real loop (`../testing/real-loop.md`): a
  key added, a sign-in against skein's fake issuer with a redirect the
  test drives, and a program's source over smith's fake `claude`; after
  each, no value in temper's store, a fact, a request, an answer or the
  client's small store.
- **The live tier** runs on a program's login on its machine, which
  temper never reads, or on a sign-in or a key added per run
  (`../testing/live-tier.md`).

## 8. Open questions

- **A connect cut by a restart.** Default: nothing kept; the carried
  row is erased at start and the forge reads as its records say, so a
  person whose connect a restart cut sees the forge not connected, with
  no line, and pastes again. Saying "interrupted; paste again" would
  take the connect's own record (its forge, number and digest) in the
  small store, a stored format (the store's version), read at start
  and erased once said; a carried row alone names no forge.
- **Rows of a forge with no record.** Default: left as they are and
  named once in the log at start; a kept forge's are erased by its
  removal (2.1).
- **Backups of the secrets' store.** Default: none; a deployment moved
  elsewhere takes its secrets' directory with its store, since its
  forges' records are there, or sets up and connects its forges
  again.
- **The forge as people's sign-in provider,** for a deployment with more
  than one person: routes of temper's on its clients' listener run the
  forge's OAuth flow, read who the person is, and ask jig's clients'
  component to mint a credential for that identity, with a `Mint` from
  temper's server code alone (jig's `protocol/client/sign-in.md`;
  jig's `protocol/service.md`, section 2; `../client/README.md`, section 5).
  The forge's OAuth client and its secret are then temper's records, in
  jig's secrets' store. Default: the printed link signs in the
  deployment's one owner.
