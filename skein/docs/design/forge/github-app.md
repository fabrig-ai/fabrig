# GitHub's dialect: the App and setup documents

## 1. The App's documents

### 1.1 The JSON Web Token

- **`github::jwt::signing_input(app, issued, expires) ->
  Option<Box<[u8]>>`**: the base64url, unpadded, of the header
  `{"alg":"RS256","typ":"JWT"}`, a `.`, and the base64url of the claims
  `{"iat":<issued>,"exp":<expires>,"iss":"<app>"}`, the times in whole
  seconds since the epoch, rounded down, and the App's id in decimal.
  None when `expires` is not after `issued` or is more than 600 seconds
  after it, which GitHub refuses.
- **`github::jwt::token(input, signature: &[u8; 256]) -> Secret`**: the
  input, a `.`, and the signature's base64url.
- **`github::jwt::JWT_BYTES`,** 512, bounds a token's length: the
  header's 36 bytes, the claims' at most 96 (two times of 11 digits, an
  id of 20), the signature's 342 and the two dots, with room. The
  component bounds a request's head by it (connection-machines.md,
  section 1).
- **The component signs** (`skein_tls::sign_rs256`, the App's 2048-bit
  key), and chooses the times: `issued` a minute before its clock, and
  `expires` ten minutes after `issued` (connection.md, section 5).

### 1.2 The mint

- **`POST /app/installations/<id>/access_tokens`,** on the JSON Web
  Token, `201`, either `Full`, with no body, or `Contents`, with
  `{"permissions":{"contents":"write"}}`, the git token of a forge's
  `Event::GitToken`.
- **Its decoder** keeps `token`, as a `Secret` within `string_bytes`,
  `expires_at`, and the `contents` and `pull_requests` levels of
  `permissions` (`read`, `write`, absent or another word as none). The
  full mint's levels are the installation's: the connection keeps them
  beside the token, each mint replacing them, and answers the bot's
  `Read::Permission` from them (`github-operations.md`, 1.1). GitHub answers a bot's
  collaborator permission `{"permission":"none","role_name":""}`
  (`bot-permission-is-not-a-role`): an App's access is its
  installation's, never a role. A repository outside a selected
  installation is refused at its read (`404`), before any permission.
- **A token minted for Contents creates issues but cannot edit them:** the
  observed mint permissions are `contents: write`, `metadata: read`, with
  `repository_selection: selected`. Issue creation answers `201`, with
  `x-accepted-github-permissions: issues=write`; a title PATCH with that
  token answers `403` "Resource not accessible by integration", with
  `x-accepted-github-permissions: issues=write; pull_requests=write`
  (`token-narrowed-creates-not-edits`). The dialect mints the same tokens;
  its mint decoder reads token, expiry and the contents and pull_requests
  levels (1.2).
- **So the git token is not a boundary:** for its hour it can open
  issues as the bot, though it is refused an edit (observed 2026-10-20).
  The narrowing stays; a second App would close the gap, and that is the
  user's call (connection.md, section 5.3).

### 1.3 Installations

- **`GET /app/installations?per_page=100&page=<n>`,** on the JSON Web
  Token, every page: each row's `id`, `account.login`, `account.id`,
  `account.type` (`User` or `Organization`), `created_at`, and
  `suspended_at` not null as suspended.
- **An installation's repositories:** `GET
  /installation/repositories?per_page=100&page=<n>`, on that
  installation's token, every page: each `id`, `owner.id`,
  `owner.login`, `name` and `private` (`github::Reachable`), which
  temper's picker shows.
- **The App's uninstall** is `DELETE /app/installations/<id>`, on the
  JSON Web Token, `204` (section 2).

### 1.4 The identity read

`GQ viewer` on an installation's token: `databaseId` the bot user's id,
and `login`. REST's `GET /user` is a `403` on that token
(`user-on-installation-403`). Counted in no call's cost, as Forgejo's.

### 1.5 The rate and validators

- **`api::Rate { limit, remaining, reset_at }`,** from
  `x-ratelimit-limit`, `-remaining` and `-reset` (seconds since the
  epoch), on every answer that has all three; the client's budget takes
  it in place of its own (client.md, section 6). GitHub's core
  allows an installation 5000 an hour (`rate-core-5000`); the App's own
  calls carry none (`jwt-calls-no-rate`).
- **`api::Validator`,** an answer's `ETag`, at most 128 bytes of
  printable ASCII, else none kept. A request sent with it
  (`If-None-Match`) is answered `304`, with no body, as
  `Answered.free`: GitHub does not count it (`etag-304-free`). The
  fact uses an installation token's two reads of an unchanging repository,
  with the first ETag on the second and the same remaining rate. A changing
  delivery log may answer a new body instead. Every read sends the
  validators the component kept for it (connection-calls.md, 1.6), the
  delivery log's first page among them (`github.md`, 9.3).

## 2. The setup's documents

Pure functions and decoders; setup/README.md's plans run them.

- **`github::setup::manifest(values)`,** the manifest's JSON: `name`,
  `url`, `public` true, `redirect_url`, `setup_url`, `setup_on_update`
  true, `hook_attributes { url, active: true }`, `default_permissions`
  and `default_events`. The values are the name, the url, the hook's
  url and the two return addresses; the permissions and events are the
  module's constants, `PERMISSIONS` and `EVENTS` (contents, pull
  requests, issues and merge queues write; checks, statuses, actions, administration and metadata read; the fourteen
  events).
- **The form's target:** `<web>/settings/apps/new?state=<s>` for a
  person or `<web>/organizations/<org>/settings/apps/new?state=<s>` for
  an organization, `<web>` being `https://github.com` or the fake's
  origin. The page whose form posts the manifest there, in its
  `manifest` field, is the owner's (setup/github.md, 1.1): the dialect
  makes no HTML.
- **The install page:** `<web>/apps/<slug>/installations/new?state=<s>`.
- **The conversion:** `POST /app-manifests/<code>/conversions`, with no
  credential, `201`: `id`, `slug`, `owner.login`, `owner.id`, `pem` and
  `webhook_secret` as `Secret`s; the OAuth client's id and secret are
  skipped.
- **The bot user:** `GET /users/<slug>%5Bbot%5D`, no credential: `id`.
- **The App's own reads,** on the JSON Web Token:
  - `GET /app` for `id`, `slug`, `owner`, `permissions` and `events`:
    reading `id` and `slug` back proves a conversion's key, and checks a
    pasted App, which the plan takes for another App when they differ
    (setup/github.md, section 1). `owner` is the App's account, with `id`,
    `login` and `type`, `User` or `Organization`. GitHub's documentation
    lists them as required; the dialect accepts an answer without an
    account. An `owner` that is absent or `null`, lacks one of the three
    or has a `null` for it, has an id of 0, an empty `login` or another
    `type` is no account, and the document still decodes. A non-null field
    of the wrong JSON kind is `InvalidAnswer`, as anywhere. Permissions retain every name and its
    `read`, `write` or `admin` access; another access is `InvalidAnswer`.
    Permission and event names fit `string_bytes`, each list fits `inner`,
    and a name or list past its bound is `TooLarge`. Missing permissions
    or events read as empty lists; id and slug remain required. Unknown
    fields are skipped. `setup::lacks(&App)` names the wanted permissions
    and events absent from these lists, in manifest order; admin covers
    write and read, and write covers read;
  - `GET /app/hook/config` for `url`;
  - `GET /app/installations/<id>`, one installation's row as 1.3's,
    which the plan's accept reads; strays are found from 1.3's
    listing, by `created_at`.
- **The App's writes,** on the JSON Web Token: `PATCH /app/hook/config`
  `{url}` (`200`); `DELETE /app/installations/<id>` (`204`). The
  conformance sends neither live (../testing/fakes/github.md, section 2).
- **The key** is the conversion's PEM; the setup's host converts it to
  the DER that `Credential::App` holds (README.md, section 7).
