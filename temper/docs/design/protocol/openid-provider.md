# temper as Forgejo's OpenID provider

A Forgejo of the deployment's signs its people in through temper: its
authentication source `temper` is an OpenID Connect client of jig's
provider (jig's `protocol/client/openid.md`), whose codes and
access tokens jig holds and whose documents and tokens skein's issuer
writes (skein's `oauth/issuer.md`). temper serves the routes, builds
the issuer at start, and draws the authorization's page.

## 1. What a person sees

- **From temper:** on the Forges page, a Forgejo that signs in through
  temper has "Open Forgejo ↗" (`../client/forge/deployment.md`), to
  Forgejo's own route that starts a sign-in with the temper source,
  `ROOT/user/oauth2/temper`, in a new tab. ROOT is the forge's browser
  origin: its owned setup's `public` when present, else its record's
  `web` when it has one, else its `git` URL
  (`forge/engine.md`, 1; `setup/owned.md`, 2). Forgejo sends the
  browser to temper's `/oauth/authorize`, temper answers at once, and
  Forgejo opens signed in. No click. A repository's link does the same with
  Forgejo's `redirect_to`, so it opens on the repository's page
  (`../client/forge/deployment.md`, *A repository's link*).
- **At Forgejo first:** Forgejo's sign-in page offers "Sign in with
  temper". Signed in to temper, one click; signed out, temper's sign-in
  page says "to continue to" the forge, Google follows, then Forgejo,
  signed in.
- **The first time,** Forgejo makes the member's account, named by
  their login (4); an account already there by that name or address
  is linked once, by its password. On a box, the person's account is linked
  with no password (9).

## 2. The routes

On the clients' listener, after what jig's mapping claims, beside
Google's (`sign-in.md`, 4.1). The issuer is the public origin (`sign-in.md`, 2.2), or
`http://<listener>` with none; each URL below is the issuer's.

| Route | What it is | Answered by |
|---|---|---|
| `GET /.well-known/openid-configuration` | the discovery document: the issuer, the endpoints below, the code flow alone, PKCE S256, RS256, the scopes and the claims | the issuer's bytes, taken once at start |
| `GET /oauth/keys` | the key set, its `kid` the key's | the issuer's bytes, taken once at start |
| `GET /oauth/authorize` | the authorization: temper's page, as every other `GET` (3) | the bundle |
| `POST /oauth/token` | the code's exchange, from Forgejo's server: the client id, a confidential client's secret by Basic or in the body, and the verifier; a public client's password is ignored (9) | jig's `Redeem` with the request's `Authorization`, `Content-Type` and body, answered `Redeemed` |
| `GET /oauth/userinfo` | the claims, for the access token as Bearer | jig's `Claims` with the request's `Authorization`, answered `Claimed` |

- **Each answer jig gives is sent as it is:** its status, its body,
  `Content-Type: application/json`, `Cache-Control: no-store`, and
  `WWW-Authenticate` from its challenge, when there is one. The discovery
  document and the key set are `application/json`, cacheable.
- **The token's body** is read within the issuer's request bound (jig's
  `provider_limits`); past it, `413` and nothing sent to jig.
- **Any other method** on the four is `405`. **With no provider**
  (`engine.clients.provider` `null`), the four answer not found, and
  `/oauth/authorize` is the page, whose `Authorize` jig refuses
  `no_provider`.
- **The issuer is built at start** by jig's
  `jig_protocol_config::clients::provider_issuer`, from jig's
  `engine.clients.provider`, its key's and its clients' secrets' files
  read privately by temper's startup (`configuration/README.md`, section 5),
  and the URLs above; it goes to jig's clients' component in its
  settings. A key or a client the helper refuses refuses the start at
  its path.
- **A moved port moves the issuer,** at the next start, and Forgejo keeps
  the address its source was given. A box's preparation gives it the
  current one (9); on any other arrangement the source is its owner's
  to change, as its owner added it.
- **What Forgejo 17's client does,** and the routes meet (its goth
  v1.82.0): it always sends a PKCE challenge, S256, and its verifier at
  the exchange; it authenticates at the token endpoint by Basic first,
  and by its secret in the body when that fails; it calls the userinfo
  endpoint by `GET` with the access token as Bearer, and requires its
  `sub` to be the ID token's. It verifies no signature of the ID token:
  it takes the token straight from the token endpoint. A confidential
  client authenticates with its secret; the box's public client has none,
  and relies on S256 and the exact redirect (9).
- **The signing algorithm is RS256 alone,** by the key jig's
  configuration names (7): `skein_tls::sign_rs256` signs it, its
  signatures replay byte for byte in the worlds, and skein's client, the
  issuer's peer in its world, verifies RS256.

## 3. The authorization

jig's cookie is `SameSite=Strict` and its `Path` is jig's prefix (jig's
`protocol/client/default-http.md`, 3), so a request to `/oauth/authorize` never
carries it, and a navigation that started on another site (Forgejo's
redirect) would not carry it anywhere. So the authorization is decided
by the client, not by a route:

1. `GET /oauth/authorize?QUERY` is answered with temper's page. The
   client root's routing reads the address as a place of its own,
   `Authorize { query }`, `QUERY` as the address carried it
   (`../client/view.md`, 1; `../client/web/README.md`, 2.3).
2. The client signs in by `Transport`: its exchanges are same-origin,
   started by temper's page, so the cookie goes with them whatever site
   the navigation came from.
3. Once the link is live and local transport credit is available, it
   sends jig's `Authorize { query }` (`POST /jig/v1/authorize`), and
   draws the answer. Cancelled watch exchanges and cleanup calls retain
   credit until their HTTP reply or failure (jig's `protocol/client/README.md`,
   section 10); the first authorization waits for that credit:

| jig's answer | The page |
|---|---|
| *go*, an address | "Continuing to Forgejo…", with "Signed in as PERSON · ADDRESS. FORGE opens in a moment, signed in as LOGIN."; the page replaces its address with jig's, so the back button never returns to the authorization |
| *signed out* | the sign-in page (`sign-in.md`, section 7), under its "Sign in" the line "to continue to **FORGE**", its `then` the authorization's address, so Google's return lands back at step 1 |
| refused *unknown client* or *unknown redirect* | "This sign-in can’t continue": "The site that sent you here asked temper to send you back to an address temper doesn’t know. Nothing was sent anywhere.", and "Back to sign in"; no address is followed |
| refused *malformed*, *no provider* or *not a person* | the same heading, its paragraph open (`sign-in.md`, section 9) |
| *not ready*, *busy* or a locally *lost* exchange | "temper couldn’t sign you in just now": "It is starting or stopping, or signing many people in at once. Nothing was kept: try again in a moment." |

- **FORGE** is the forge as the client names it (`../client/forge/deployment.md`
  ): "Forgejo at HOST", or "Forgejo on this machine" for an owned
  one, of the forge whose client is the query's `client_id`; a
  `client_id` that names no forge's is "Forgejo". PERSON, ADDRESS and
  LOGIN are the signer's, as the client knows them.
- **The page's facts:** temper-service inserts, beside the
  `temper-sign-in` meta (`sign-in.md`, section 7), one `<meta
  name="temper-forge-sign-in" content="NUMBER CLIENT ROOT MODE">` for
  each entry of `sign_in.forges` that serves (7): its forge's number,
  its client's id, its
  browser origin (1) and `owned` or `other`, space-separated,
  attribute-escaped.
  The web shell gives them to the view with the ways; the signed-out
  note and "to continue to" read them, as "Open Forgejo ↗" reads the
  forge's row.
- **The address's bound:** signed out, an authorization address
  (`/oauth/authorize?` and its query) past the Google start's `then`
  bound, 1,024 bytes (`sign-in.md`, 4.2), cannot ride the sign-in, and is refused
  *malformed* before anything is sent. Signed in, the query goes to jig
  as it came, within jig's own bound on it.
- **No ADDRESS yet:** jig's party gives the client its name and login,
  not its identity's subject, so the signed-in line drops " · ADDRESS"
  until it does (`sign-in.md`, section 9).
- **No consent page:** the clients are the deployment's, configured by
  its owners.

## 4. The claims, and Forgejo's account

- **`sub`:** the identity, Google's provider number, `:`, and the
  member's address as configured, `2:sam@example.com`. It outlives the
  store: a deployment started afresh gives Forgejo the same `sub` for the
  same member, never a party's number, which a fresh store would give to
  someone else.
- **`preferred_username`:** the member's login. **`email`:** the
  member's address, verified, for Google's provider is one of jig's
  `addresses`. **`name`:** the member's name as minted. No `nickname`.
- **Forgejo's settings,** in its `app.ini` (`setup/owned.md`, 2 and 8):
  - `[oauth2_client] ENABLE_AUTO_REGISTRATION = true` and `USERNAME =
    preferred_username`, which Forgejo 17 takes (a value it does not
    take is logged and read as `nickname`): a member's first sign-in
    makes their account, named by their login;
  - `ACCOUNT_LINKING = login`, Forgejo's default: an account that
    already exists with that name or address is linked once, by its
    password. Not `auto`, which links by a matching name or address
    alone, but on a box's owned Forgejo (9);
  - `[service] ALLOW_ONLY_EXTERNAL_REGISTRATION = true`, beside
    `DISABLE_REGISTRATION`, so that no one registers on Forgejo's own
    form and the temper source registers its members. `ENABLE_INTERNAL_SIGNIN
    = false`, once the source works, makes it Forgejo's only sign-in on
    the web; its admin keeps Forgejo's command line, and temper's bot its
    tokens.

## 5. The source on Forgejo

Forgejo's API has no route for authentication sources (Forgejo 17), so
the source is added on Forgejo's host, by its command line:

```
forgejo admin auth add-oauth --name temper --provider openidConnect \
  --key forgejo --secret "$(cat CLIENT_SECRET_FILE)" \
  --auto-discover-url https://temper.example.com/.well-known/openid-configuration
```

`temper-box forge sign-in` prints it, for a managed Forgejo and `temper-box serve`'s
owned one alike (`setup/owned.md`, 8); a box runs its own (9), with no secret.
The name `temper` is the callback's path
segment, so it is fixed. `add-oauth` fetches the discovery document at
once and adds no source when it does not answer, so temper serves
before it runs.

## 6. Signing out

- **Signing out of temper ends temper's sign-in only.** Forgejo keeps
  its own session, and Google its own. While a forge signs its people in
  through temper, the signed-out page says so, under its "Sign in": "You
  signed out. FORGE keeps its own sign-in: sign out there too.", one
  line for each such forge.
- **Signing out of Forgejo** ends Forgejo's session; its next sign-in
  bounces through temper, and a member still signed in to temper is
  signed in again at once.
- **No single sign-out:** Forgejo 17 takes no signal from its provider:
  no back-channel, front-channel or session-management logout, and no
  RP-initiated one (it parses the discovery document's
  `end_session_endpoint` and never uses it); its own sign-out is local.

## 7. The configuration

jig's `engine.clients.provider` (jig's `protocol/configuration.md`,
section 3), and temper's `application.sign_in.forges`:

```
"engine": {"clients": {"provider": {
  "key": "provider.key",
  "clients": [{"id": "forgejo", "secret": "forgejo-client.secret",
               "redirect_uri": "https://git.example.com/user/oauth2/temper/callback"}],
  "addresses": [2]}}},
"application": {"sign_in": {..., "forges": [{"forge": 1, "client": "forgejo"}]}}
```

- **The key:** an RSA key of 3072 bits, PKCS#8 PEM, mode 0600, the
  service's own user, made on its host by `temper-box forge sign-in`
  (`setup/owned.md`, 8) with `openssl genpkey`, never copied, printed or
  logged; jig's start refuses any other. A key replaced is a file
  replaced; its key set's `kid` names it anew.
- **`sign_in.forges`:** a list, absent for none (`configuration/engine.md`
  ): each `{"forge", "client"}`, `forge` a forge's number, as
  the Forges page and `temper-box forge` name it, `client` one of the
  provider's clients. Each forge and each client at most once, and each
  client one of the provider's, or the start is refused at its path. A
  member whose login, folded, is the bot's (`temper`) refuses the start at
  its path, whatever the forges: temper never signs a person in as the bot.
- **Checked once the forge is known.** A forge is a record in temper's
  store, made at runtime (`forge/engine.md`, 1), so the start cannot check
  it, and an entry refuses no start, whichever forges are kept. Once
  the forge's records load, and when a forge is made or removed, an
  entry serves only when its forge is a kept Forgejo whose browser
  origin's `/user/oauth2/temper/callback` is exactly its client's
  `redirect_uri`: its owned setup's `public` when present, else its
  record's `web` when it has one, else its `git` URL (1); and when
  its meta has room within `clients.bundle_bytes`. An entry that does
  not serve is named once in the log, for that reason, and its forge
  has no meta, no "Open Forgejo ↗" and no line on the signed-out page;
  every other entry serves. Server-to-server API calls keep the
  record's `api` and transport; git uses `git`.
- **A browser origin apart from `git`,** by `web`: a Forgejo connected
  as its admin (`setup/reconcile.md`, 5) gives it where temper and its workers
  reach it at another address than people do, as a Forgejo a host's own
  units run beside temper is on loopback to temper and on its public
  name to a browser (`setup/box.md`, 4).
- **The page follows.** The metas are made anew when an entry's serving
  changes, and the `ETag` is the page's so made, so a browser's next
  revalidation takes them. A reply on its way ends with the page it
  began.
  With a provider and no `sign_in.forges`, nothing of temper's names
  the provider's clients.

## 8. Testing

- **An attacker review of the box's sign-in** (9): another process on
  the machine and another browser tab, account linking by name and address,
  excluded identities, public-client redemption and PKCE, the printed link,
  cookies, key files, source failure and restored source facts. Every finding
  is fixed or recorded with its remaining exposure.
- **A box's owned Forgejo** (9): the step tests of its claims, its refused
  logins, its address's encoding and the issuer's refusals; the box world, its
  source step failing with the box running and the set-up done; the process
  tier, its one click on a real Forgejo against temper's own issuer.
- **The sign-in world** (`sign-in.md`, section 8) gains the provider: Google's
  sign-in, then the page's `Authorize`, the token by Basic and by the
  body, and the userinfo, with the same `sub` and `preferred_username`;
  the discovery document and the key set at their routes; `405`; not
  found with no provider; a token's body past its bound. jig's own
  refusals are its world's, not repeated.
- **The view's tests:** the `Authorize` place's pages for each answer,
  "to continue to", the signed-out note, and "Open Forgejo ↗" with its
  line. **The client's domain:** the address read as `Authorize`, each
  of jig's answers.
- **Step tests:** `sign_in.forges`' checks, at start and once the
  forge's records load, an entry for a forge not kept, not a Forgejo,
  of another origin or past the bundle's room named and left out; the
  metas' bytes;
  `temper-box forge sign-in`'s printed bytes and the files it makes.
- **One process-tier story** (`../testing/process-tier.md`): an owned
  Forgejo, the temper source added by the printed command, a scripted
  client with a cookie jar that signs in to temper by the printed link
  and then to Forgejo through temper, and Forgejo's API naming the
  member's account by their login, their address and the `sub` it
  keeps. It proves the claims and
  Forgejo's settings, which no fake can. The printed link, not the fake
  Google: with `sign_in` configured it signs in the member of Google's
  provider (`sign-in.md`, 2.3), so Forgejo gets the same claims; Google's leg is the
  service world's; and skein's fake issuer peer cannot yet give its
  account a verified address.
- **The browser tier, one story** (`../client/web/testing.md`):
  jig's strict cookie absent from a navigation that started on another
  site and present on the page's own exchanges, so the `Authorize` place
  answers *go*.

## 9. A box's owned Forgejo

A box's person presses a repository's link and lands on its page, signed in
(`../client/forge/deployment.md`, *A repository's link*). A box has no provider
without this, and nobody to run a printed command, so its owned Forgejo sets
itself up, with no command for the person. A managed Forgejo and `temper-box
serve`'s owned one stay as 4 to 7 say. This is an authentication change, so
its reasons are kept here.

**What the box's configuration holds,** as `temper-box init --box` writes it
(`setup/init.md`, 2):

```
"engine": {"owners": [{"provider": 1, "subject": "ada@localhost"}],
           "providers": [{"number": 1, "kind": "link"}, ...],
           "clients": {"provider": {
             "key": "provider.key",
             "clients": [{"id": "forgejo", "secret": null,
                          "redirect_uri": "http://127.0.0.1:8370/user/oauth2/temper/callback"}],
             "addresses": [1]}}},
"application": {"sign_in": null,
                "machine": {"owned": {"listen": ..., "person": {"login": "ada", "email": "ada@localhost"},
                                      "sign_in": "forgejo", ...}}}
```

- **`application.sign_in` stays `null`:** it holds Google's sign-in and its
  members, and a box has neither. `sign_in.forges` names a forge by its number,
  which is known only once the forge is made (7), so a box's entry names a
  kind instead: **`machine.owned.sign_in`,** a client of the provider, or `null`.
  It is the entry for the kept forge of mode owned, under 7's check, whatever
  its number. A client it names that the provider lacks refuses the start at its
  path.
- **The redirect is known at init,** since the owned `listen` socket is written
  there: 7's check holds it against the forge's browser origin as ever.

**The claims** (4), which Forgejo's `auto` needs both of:

- **`preferred_username` is `machine.owned.person.login`.** The printed link's
  login on such a box is that login (`sign-in.md`, 2.3), so the party's login is the Forgejo
  account's. Its name stays `Owner`. With `machine.owned.sign_in` `null`, nothing
  of this applies and the login stays `owner`.
- **`email` is `machine.owned.person.email`:** the owner's subject is that
  address and provider 1 is in `addresses`. `sub` is `1:ada@localhost`, which
  outlives the store as 4 wants.

**`ACCOUNT_LINKING = auto`, on a box's owned Forgejo alone.** Measured on a
throwaway Forgejo 17.0.1, with a stand-in provider, against a site-admin account
with a password, as the set-up makes:

| Setting and claims | Where it lands |
|---|---|
| `login` | Link account, asking for the password: what breaks one click |
| `auto`, the name matching, the address differing | the repository, signed in, three redirects, no second account |
| `auto`, no `email` claim | Link account, asking for the password |
| a second visit in one browser | through the provider again, every click passing `/oauth/authorize` |

So every press passes "Continuing to Forgejo…" (3), which replaces its address
as soon as jig answers, with nothing to hold it longer. `auto` links by the name,
then by the address, with no password. That is safe here because temper made every
account on this Forgejo and the printed link signs in the box's one person (below).
A box trusts every program listening on this machine's loopback, since browser
cookies ignore ports. So Forgejo's session lasts an hour (`setup/owned.md`, 2): every
press signs in again, and one click needs no more. `login` stays for a
managed Forgejo, whose accounts are not temper's, and for `temper-box serve`'s owned
one, until the same reasoning has been checked for several members.

**What a name match could take,** and how a box rules each out:

| Forgejo holds | Its name | How a box rules it out |
|---|---|---|
| the person's own account, a site admin | `machine.owned.person.login` | the one it is meant to take: the same person, who holds the machine and may read its password |
| temper's bot | `temper` | a person so named refuses the start (below); **temper never signs a person in as the bot** |
| the set-up's admin | `temper-admin`, `temper-admin@example.invalid` | refused likewise, the name and the address |
| the first repository's organization: `auto` looks in the table that holds users and organizations alike | `work` | refused likewise; any other organization's name can't be the person's, since the person's account is made first and a name is taken once |
| an account by the person's address, if the name finds none | `ada@localhost` | the bot's address (`temper@temper.invalid`) as the person's refuses the start; the address is otherwise the person's alone |

- **Refused, at the start, at its path:** `machine.owned.person.login`, folded,
  being `temper`, `temper-admin` or `work`, or its `email`, folded, being the bot's
  or the set-up's admin's, when `machine.owned.sign_in` is not `null`. `temper-box init --box` already falls back
  from these logins; a hand-written document is refused. A login reaches jig only
  from these or from `sign_in.members` (7), so a start-time refusal is whole.
- **Refused too, at the start, at `machine.owned.sign_in`'s path:** that member
  beside `machine.owned.public`. Behind a proxy the forge's browser origin is
  `public`, where the redirect init writes is the owned `listen`'s, so 7's check
  would serve nothing, with the source added and no off fact. No walk has proved a
  box behind a proxy (`sign-in.md`, section 9).
- **Refused too, at the start, at `machine.owned.sign_in`'s path:** that member
  beside an `application.sign_in` that isn't `null`, in the words "a box's sign-in
  takes application.sign_in null". A box is its one person's: a member Google signs
  in could land on the person's Forgejo account by name, and temper's link to that
  account (`../draft/forge-identity.md`, 9) would count them as the person.
- **Nobody else signs in:** a box has no member and no Google; the printed link
  alone signs anyone in, always its owner, so the link provider's one owner is the
  one person who can reach `/oauth/authorize` signed in. An agent or a coding run
  is refused *not a person* (jig's `protocol/client/openid.md`).

**The provider's key:** an RSA key of 3072 bits, as 7 says, made at the first run
by `temper-box init --box`, before the document is written, in `provider.key` in the data
directory, 0600 and the service's user's, under `temper-box forge sign-in`'s rule
(`setup/owned.md`, 8): `openssl genpkey`, since skein has no key generation (ring signs
and does not generate); a key already there is left as it is. With no `openssl`, or if it fails, the document has
`clients.provider` and `machine.owned.sign_in` `null`, the box goes on and the links
stay plain, and `box.log` says so.

**The client is public** (skein's `oauth/issuer.md`, section 2): `secret` `null`.
`add-oauth` takes `--secret` only as an argument, which a command line shows to other
local users, and a fixed value is no secret. So the box's client has none, and
`add-oauth` is given none. S256 and the exact redirect stand in for it. The clients'
other rules are skein's, per client, so no other client is weakened.

**The source's step:** the last of the owned group's, after Forgejo answers
(`setup/box.md`, 1): `temper-box forge source`, which makes the source serve. It reads
temper's discovery document first, `ISSUER/.well-known/openid-configuration`, within
the set-up's deadline; with no answer it ends failed, `temper's discovery did not
answer`. Then, unless Forgejo lists the source, it adds it, by `add-oauth --name temper
--provider openidConnect --key forgejo --auto-discover-url
ISSUER/.well-known/openid-configuration` with `--work-path` and `--config`, which works
at once while Forgejo runs. With the source listed, it asks Forgejo for the source's
sign-in, `GET /user/oauth2/temper` at the owned `listen`, following no redirect: a
redirect to temper's `/oauth/authorize` says Forgejo serves it, and the step ends
added. Forgejo loads a source's discovery when it starts, and a source whose discovery
did not answer then is listed but not served; on that sign-in Forgejo, finding no
provider for it, loads its sources again, so the same request makes it serve. Any
other answer ends the step failed, `Forgejo does not serve the source`. A step that
ends failed runs again while its group stands (`setup/box.md`, 2), so the source serves
once the engine answers, whichever started first, and Forgejo is never restarted for
it. temper is serving by the step's first run, since it asked for the group. On a restart, serve
starts before this group too: a later preparation's source commands and Forgejo, loading its
source, fetch discovery. So serve's read at its start comes before preparation's new handover,
which it reads again once its first reconcile has waited for Forgejo, taking it before the
connect (`setup/owned.md`, 5).
A restored group reports started only after its final source outcome, as on its first start.
Its source fact is accepted for the kept box-owned Forgejo even with no setup ask in flight;
the restored forge needs no ask. An unknown or non-owned forge cannot set this fact.

**A moved port.** The discovery address names temper's own listener, and a port a box
finds taken is one its person may edit (`setup/box.md`, 1). Forgejo reads a source's
address when it loads the source, at its start, and a source already listed is not
added again. So a later preparation, before Forgejo runs, lists the sources and, with
`temper` among them, runs `forgejo admin auth update-oauth --id ID --use-custom-urls false
--auto-discover-url ISSUER/.well-known/openid-configuration`, with `--work-path`
and `--config` (`setup/owned.md`, 3). Each of the two commands has the set-up's deadline, since Forgejo may fetch the
address it is given and serve answers it only once it is up.

- **It never fails the set-up.** Any ending of the step leaves the set-up succeeding:
  one line in `box.log`, the forge's page fact saying so, and the links plain until a
  later run of the step makes the source serve, which turns the fact and the links back.
  Never a welcome failure (`setup/box.md`, 2). Any ending of the preparation's two commands
  leaves the preparation going, with one line in its own log and the source as it was:
  the entry still serves, so a press then ends at Forgejo's own error, on the old
  address, until a later start succeeds. Forgejo's own login still works.
- **Until the step says the source is there, the entry does not serve:** 7's
  "does not serve" with the reason, so no address leads to a route Forgejo lacks.
  Then the metas are made anew as 7 says, and "Open Forgejo ↗" and the repository's
  link appear on a box too.
- **The page's other fact:** while a box's owned forge is kept and its entry can't serve
  for want of a key or a source, one `<meta name="temper-forge-sign-in-off"
  content="NUMBER">`; the Forges row says so (`../client/forge/deployment.md`).

Forgejo's own login with the password keeps working: `ENABLE_INTERNAL_SIGNIN` stays
as it is. `temper ctl` and a coding run's credentials are untouched: they are the
bot's and the person's tokens, and neither passes this provider.
