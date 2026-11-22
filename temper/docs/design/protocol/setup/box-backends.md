# The box's fake backends

This document holds the box's fake backends, split from [`box.md`](box.md).

## 1. The welcome's fake backends

temper-box [DIRECTORIES] --fake-backends [--bundle BUNDLE] prepares the retained isolated
sandbox of ../configuration/directories.md, 1. Ordinary children read the effective
temper.json and agent.json through their effective directory flags; peer
commands read absolute effective document paths. Changing only the parent's
parsed bytes is insufficient.

Build the bundle once, then start even with empty configuration:

    scripts/bundle.sh DIR && temper-box --fake-backends --bundle DIR

The flag overrides an init-shaped source's application.clients.bundle;
without it, that configured bundle is used. Missing both refuses with the
bundle command and --bundle. Empty C stays untouched: init writes directly
into the effective sandbox. Box startup performs no build or fetch.

Without the test override below, a sandbox written by init takes the
kernel's ports, a bind of port 0 for each of its five, never the 8367 to
8371 a person's box prefers (`init.md`, 2). So a
fake box leaves those to the box a person starts after it. A kernel-chosen
port lost between bind and listen is closed and chosen again, within 32
attempts; the fake peers' reservations likewise retry address-in-use,
counting excluded ports against the same bound. Other errors refuse at
once. Only a listening socket's port is reported. The ports are written
fixed and held as init's are, and
the printed link names them. A stopped box holds none, so a port the kernel
hands on meanwhile refuses its restart, naming the port. An init-shaped
source's listeners stay its own.

`TEMPER_INTERNAL_FAKE_BOX_PORTS`, the process tier's and QA's places', never a person's
(`../../testing/process-tier.md`; `../../testing/qa/places.md`, 2), names a new sandbox's nine ports, separated
by commas: init's five, clients, owned, forge, llm and webhooks, then the
peers' github, oauth, llm and forgejo. A new sandbox holds each of them or
refuses to start, naming the port and the variable, as it does a value that
isn't nine nonzero ports. A retained sandbox keeps the ports it has,
whatever the variable says.

After the sign-in block (`box-supervision.md`, 1), before any supervised peer starts, the box
prints the effective directories, the fakes' origins and the files a walk
pastes (../configuration/directories.md, 1).
GitHub has one fixed person and repository. Its App 1 is seeded and
installed, with the setup URL a person gives an App of their own (`github.md`, 3),
`http://<listener>/forges/1/github/setup`, the number GitHub's forge
takes when it is the first forge a walk makes, so that the own-App path's
Install comes back to temper; App 1's private key is D'/github-app.pem,
mode 0600. The dummy model key is the key a walk pastes. The elsewhere
Forgejo's admin token is kept in its private file: instructions name only
the path, and no log, report or record contains its contents. The fixed signed-in model bearer uses the first added
credential stream with the same seed as the issuer, while the primary
model key stays the API-key choice. Neither value is a real credential.

The standalone issuer command is:

    temper-box fake oauth --document FILE [--ready PREFIX] [--seed N]

Its bounded document has a loopback listen socket with a nonzero port,
redirect_port (the shipped 1455 by default) and redirect_path
(auth/callback by default). The listen socket determines its HTTP origin
and shipped authorization, token, revocation and keys paths. Unknown,
duplicate, ill-typed and oversized members are refused before any output
or listener: at most 16 KiB for the document and 1024 bytes per URL/path.

temper-fake decodes the document and configures skein's OAuth peer;
temper owns the files, signal handling, readiness and the real loop.
Authorization grants immediately with the shipped client/scopes/resource,
the fixed fake person, OpenID tokens/keys and the fixed bearer accepted
by the model. No login or consent page is added.

The wrapper refills its bounded token-plan queue each pass. Each issuer
launch admits at most 64 sign-ins, since the primitive retains used codes.
Exhaustion, or refresh after revoking the fixed refresh token, requires an
issuer restart. Generated configuration keeps callback port 1455; concurrent
human sign-ins from several local boxes are not promised. Tests name
explicit callback ports. Long-running peers retain no observation stream
unless one is explicitly requested.

The GitHub and model peers are long steps named `fake github` and
`fake model`, before ordinary steps and outside the playground's asked
group. Their arguments are `fake forge --document C'/github.json --seed 7`
and `fake llm --document C'/llm.json --seed 7`, respectively, with no
directory flags. The issuer uses the same seed, so its bearer matches the
model's first added key. A supervised issuer is `fake issuer`, using
`fake oauth --document C'/oauth.json --seed 7`; it requires the primitive's
inherited-listener support. It never substitutes a self-bound startup race
for that contract.

**Scripts given.** `--scripts FILE`, repeatable, gives the fake model
scripts in place of those the box generates, and `--strict` its strict
switch (skein's `testing/fakes/llm.md`, 2.5), at every start, of a new
sandbox or a retained one: QA's (`../../draft/landed-qa.md`, 6) and the
process tier's. Each FILE is a JSON object of the document form's script
members, `cued`, `asked` and `otherwise`, each optional (`playground.md`,
the document form). Before any step starts the box reads them in the
order given and joins them: `cued` and `asked` each in that order, the
first file's first, and `otherwise` the last file's that holds one. A
file that does not read, a member it does not know, a cue repeating one
before it in its group, or a join past the document form's bounds
refuses the start, naming the file. The join is written to
`C'/llm-scripts.json`, 0o600, replacing the last start's, and the `fake
model` step's arguments add `--scripts C'/llm-scripts.json`, and
`--strict` with `--strict`; a start without `--scripts` removes the
file, and `llm.json`'s generated scripts answer again. The retained
`llm.json` never changes: its listener, keys, TLS, models and outputs
stay. So scripts are the start's, the same at every place, and never
change while the box runs.

The box owns the peers' held listeners before spawning them. First
preparation transfers its reservations exactly once; on reuse the ordinary
box listener mechanism binds the retained documents' addresses. Each is
loopback with a nonzero port, distinct from the other peer ports and the
ordinary box listeners. Output paths stay within D'. The retained documents
are revalidated on both paths; edits are not silently replaced.

Held listeners provide the ordinary fake peers' readiness and accept
backlog. Restart, backoff, give-up and reverse-order shutdown use the
ordinary supervisor. The playground's reset ends no peer of these, so a
person's newly connected model or issuer runs on. The fake forges keep
their state across a restart: the `fake github` and `fake forgejo` steps'
arguments add `--state D'/github-forge.state` and `--state
D'/forgejo-forge.state` (`../../testing/fakes.md`, 2), written at each
one's orderly stop, so a stopped box's data holds its Apps,
installations, repositories, pull requests and CI as they stood, and a
copy of that data holds them (`../../draft/landed-qa.md`, 6). A forge
that exits otherwise starts again from what its last orderly stop wrote,
or from its seed when none has. The model and the issuer keep their
memory for one launch: a restarted one uses the same seed, scripts and
output files, and so answers as before. A state a newer build cannot
read refuses that step's start, naming the file, as a store refused by
name does. Effective configuration and temper's data remain retained.
Their CI is a head's own: each seeded repository's `ci` and each
document's `made_ci` are `{"declared": ".fake-forge/ci"}`
(`../../testing/fakes.md`, 2), so a head holding no `.fake-forge/ci`
passes `check`, as every head did, and a person's commit decides a
head's verdict by what it holds, on every repository a walk makes there.

**Sources to copy** (`../../draft/repository-copies.md`, 6): the `fake forgejo`
peer's document names two, and `box: copy source ADDRESS` prints each
(../configuration/directories.md, 1). They serve a Forgejo elsewhere, the peer the
walk connects as its admin: an owned Forgejo is real, and reaches no
`.invalid` host.

Forgejo here remains owned and real. `application.test.github` points GitHub
at its fake (`place.md`, 7), and names no forge: each is made by the walk's
own `SetUp` or `Connect`. Elsewhere is a Forgejo the walk connects as its
admin (`reconcile.md`, 5), at the address preparation prints.
Its separate `fake forgejo` long step uses
`fake forge --document C'/forgejo.json --seed 7`, with its held loopback
listener before ordinary steps and outside the playground's asked group.
The fixed peer prefix order is fake github, fake issuer, fake model,
then fake forgejo. Its retained token output is D'/forgejo-admin.token,
subject to the same private-file validation and seed-output reuse as the
other peers. Admission and reconciliation use the
ordinary configuration checks, without raised budgets or policy limits.
The program way uses a separately provisioned fake adapter
(../configuration/directories.md, 1) and the ordinary program-login mechanism. Only
after validating the adapter does the box print the exact program-login
command for its effective directories; a walker starts the box before using
it. No product path branches on the flag. A conflicting issuer listener is a
refusal, not a reason to bind another port or regenerate the retained
document.

## 2. Open questions

- **Repeated admission during fake-box preparation** (`box-backends.md`, 1): share a
  checked protocol document and typed deployment between preparation,
  step assembly and peer validation, preserving admission of every
  changed retained/source document and the order of refusals. Ordinary
  boxes still discover their environment and generate independent fresh
  provider keys. Default: retain those checks and keys; further startup
  cost work follows `../../testing/README.md`, section 7, with its measured
  causes in `temper/docs/development/limits-heavy.md`.

- **The fake box's retained ports under concurrent stories** (`box-backends.md`, 1):
  prove the story-supplied ports across a stopped box's restart in the
  after-landing tiers (`../../testing/README.md`, 7), beside other test
  processes drawing their own ports. Focused proof holds the choice and
  retention rules; the process and browser walks hold the running box.

- **A fake fetch for a box's rendered walk:** a real fetch lasts seconds
  and its `fetched` lines keep the page moving. Main's journeys print
  the percentage through the world's box peer, on simulator time,
  without a box or fetch process (`../../client/journey-walks.md`, 4.4).

- **The other fake peers' refused connections** (`box-backends.md`, 1): the issuer's log
  takes a line for each connection it turns away at its cap, from skein's
  OAuth peer's count; `fake github`, `fake forgejo` and `fake model` turn
  theirs away silently, and skein's forge and LLM peers count none. Each
  takes the same count and line when a walk meets its cap.
