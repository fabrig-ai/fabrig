# Forgejo on the same footing

This holds Forgejo's client connection and owned mode, split from the [forge connection draft](github.md).

## 1. Connected and managed, from the client

### 1.1 Connected

- **What the person does on Forgejo** is ../protocol/setup/README.md's (`../protocol/setup/README.md`, section 4): a user
  for temper, not a site admin; its API token and its git token, with
  their scopes; its write access to each repository temper will adopt;
  optionally, hooks to temper's listener.
- **"Connect Forgejo"** in the client takes the two tokens, and the
  hooks' secret when the person adds hooks, in one request carrying
  values (`github.md`, section 3).
- **The engine checks before it keeps:** the API token's user (`GET
  /api/v1/user`), which becomes the forge's identity, shown to the
  person ("connected as temper, id 7"); the git token answering as the
  same user, which its `read:user` scope allows (skein's
  `forge/setup/forgejo.md`, 1.3, step 3). A mismatch is refused, naming which.
  `forges[].user` goes from configuration: the identity is the one the
  token answered when the person connected it, kept and checked at each
  start (`github.md`, section 4).
- **Replaced** from the client: a token that must answer as the same
  user (`github.md`, section 4).

### 1.2 Managed

- **"Connect Forgejo as its admin":** the person pastes a site admin's
  token. The engine runs skein's reconcile (skein's `forge/setup/forgejo.md`,
  section 1) with it, as its host, and keeps what reconcile makes, the
  bot's two tokens, its id and the hooks' secret, in the store. The
  forge then connects as 1.1's would.
- **Reconcile's host is in the engine:** its protocol layer runs it at
  each start, and when the person connects, before the component opens
  the forge; its items (skein's `forge/setup/forgejo.md`, 1.1) are the store's
  records. The forge waits, "connecting", meanwhile; the engine serves.
- **Access per repository, at adoption,** in place of setup's `orgs`:
  adopting a repository on a managed forge first grants the bot write on
  it, as a collaborator (`PUT /api/v1/repos/{o}/{r}/collaborators/{bot}`),
  and makes its hook when the deployment has a webhooks' listener; then
  the adoption reads as any other (`../domain/forge/adoption.md`).
  This is a plan of skein's setup beside reconcile, run by the same
  host with the admin's token. Reconcile's org steps narrow: step 5
  makes an owned forge's orgs when missing (`github-forgejo.md`, section 2), with no team,
  and step 6, each org's hook, goes. In their place, at each start, it
  checks and repairs each adopted repository's collaborator and hook.
- **The picker** lists every repository the admin can see: all of the
  forge's, since temper grants itself access when one is chosen
  (`github.md`, 2.2).
- **The admin's token is kept** in the store, since adoption needs it
  at any time, as on an owned forge (section 2). It is read by the
  setup's host alone and never handed to the forge component, which
  makes every request of temper's work. Least power is restated so: the
  component never holds an admin's token; the engine's setup host holds
  one for a managed or owned forge, and uses it for reconcile and for
  access at adoption alone. On a managed forge, "Forget the admin's
  token" in the client makes the forge connected: the bot and its
  tokens stay, and access is the person's to give again.
- **Connected, then managed:** an admin's token given for a forge
  already connected keeps its user, since a forge's writer never changes
  (`github.md`, section 4). Reconcile is given the connected user's login, and takes
  it over when it carries temper's mark, `temper:bot`, in its full name;
  an unmarked user stops it (`Unmarked`, skein's `forge/setup/forgejo.md`, 1.3),
  and the client says to add the mark on Forgejo, so that no login
  written wrong takes over a person's account.

## 2. Owned

- **Preparing and running stay commands,** with their files:
  `temper forge prepare` and `temper forge run` (../protocol/setup/owned.md).
  Forgejo's own secrets (`app.ini`, `state`) and the runner's are
  Forgejo's and the runner's to read, from files, as now; and so is the
  person's first password, `person_password`.
- **The admin's token lives in the store,** as a managed forge's does
  (1.2); only its way in differs, since temper's preparation makes it,
  not a person. The preparation makes it by Forgejo's command line,
  under Forgejo's lock and before the engine runs, so it cannot write
  the store: it leaves the token in `admin_token`, a handover, as today.
  The engine's setup host takes it at its start, through the same
  connect as a pasted admin's token: kept in the store, the file
  removed, and temper's admin's other tokens deleted on Forgejo, by the
  routes reconcile uses for the bot's (skein's `forge/setup/forgejo.md`, 1.3,
  step 3). The file lives from the preparation to the engine's next
  start, and no credential is ever read from a file after.
- **A new token at each preparation:** with the file gone once taken,
  each preparation makes the next, as it makes one now when the file is
  missing (../protocol/setup/owned.md, 3, item 2). A box's every start so renews
  temper's admin's token, and one revoked on Forgejo is replaced at the
  next. `temper serve` started again alone, with no preparation before
  it, takes no file and runs on the store's token.
- **What reconcile makes goes to the store:** the bot's tokens, its id
  and the hooks' secret. The forge's directory loses `api_token`,
  `git_token`, `user` and `webhook_secret`, and configuration's forge
  stops naming them (`github.md`, section 14).
- **The box drops its reconcile steps** (../protocol/setup/box.md): prepare,
  Forgejo, its runner, `temper serve`. A token revoked under a running
  engine is made again by the engine's next reconcile, at its restart.
- **The person's orgs:** `setup.orgs` goes. An owned setup keeps
  `mode.owned.person.orgs`, the orgs reconcile makes for the person when
  missing, owned by them, so that a fresh box has somewhere for
  repositories; temper's access to those repositories is granted at
  adoption, as on a managed forge (1.2).
- **Connected from the start:** an owned forge is connected by its
  preparation and the engine's first reconcile; the client has nothing
  to paste. An owned forge not yet prepared is not connected, and the
  client names `temper forge prepare`.
