# Configuration at a start

This document holds the configuration checks at a start, split from [`README.md`](README.md).

## 1. First start and every start

- **Every command that reads the document checks it,** once its three
  directories are resolved and checked (`directories.md`). `temper-box serve`
  checks it whole, as `README.md`, section 2 says. `temper-box program-login`, every
  `temper-box forge` command and `temper box` check its structure, its
  members, the limits and `machine`, and none of the checks that need
  a private file read, since a set-up forge's files do not exist before
  its commands make them (../setup/init.md). A `temper-box forge`
  command finds its forge's kind in the box's group file, `fetch`,
  `prepare` and `run` taking an owned forge with none (../setup/box.md, 4),
  and its setup entry in `machine` (../binaries.md; ../setup/box.md, 1).
  Each reads
  the private files its own part needs alone: a `temper-box forge` command
  its forge's, by ../setup/owned.md; `temper box` and `temper-box
  program-login` none, the latter reading the agent's configuration
  besides (`worker.md`, section 2); `temper-box serve` every one the document names, and
  of a forge's directory an owned or fake forge's handover alone
  (`README.md`, section 5).
- **A deployment's first start** is jig's (jig's
  `protocol/configuration.md`, section 7): the first with an empty store,
  whose owners and projects the core seeds from `engine`. temper adds:
  - **periods from the smallest source,** as jig's reference service
    gives them (jig's `protocol/configuration.md`, section 5): before the
    core admits a party's request, the engine's routing gives the root
    one period for each configured project, its budget the project's
    spend per period, with no next start;
  - **each project's landing rules** join its policy as it is seeded;
  - **a project made after the first start** (jig's `MakeProject`, its
    `jig/docs/design/domain/model.md`, 6.1) has the deployment's landing rules, which
    every start applies (`../../domain/root-translation.md`, section 1), and none of
    `landing.projects`, which names seeded projects alone; its first
    period is made with it (jig's `domain/people/acts.md`, 2). The default
    document seeds no project, its first owners the deployment's alone,
    and offers policy 1, which every project made starts from; the
    green-CI landing rule is in `landing.deployment` for every project
    (../setup/init.md, 3). A deployment whose first start seeded a project
    keeps it: no later start removes or renames it.
- **Every start:**
  - jig's parts apply as jig says (jig's `protocol/configuration.md`,
    section 7);
  - temper's members apply as configured: the machine's places, the
    deployment's landing rules, the forge's judges for every rule, the
    charter, the limits and the workers. The forges are their records,
    restored with the forge connector's (../forge/engine.md, 1);
  - every limit is checked again, temper's and jig's together, with the
    channel's start checks on temper's parts (`limits.md`; ../channel.md,
    section 8), so a `git` base or a limit changed since the last start
    is refused before io starts when a remote or a part could pass its
    bound;
  - the root's configuration is held to the root's limits before the
    root is made (`../../domain/root-translation.md`, section 1): a start that breaks
    one is refused, naming it;
  - each provider's fixed `sources.sign_in.redirect_port` differs from
    the clients', webhooks', the machine's owned and fake and `demo`'s
    fake LLM's listener port, even between IPv4 and IPv6. An absent or
    ephemeral listener reserves no fixed port. A connected forge's
    remote API is no local listener;
  - once io starts, the forge connector restores every forge's record,
    and an ended one's removal resumes (../forge/engine.md, 3); the engine's
    routing loads every kept forge's credentials from jig's secrets'
    store before the forge component opens a forge. A forge whose
    records are missing, incomplete, unreadable, or made for another
    kind or `api` (their `Origin`) is not connected, named, and the
    other forges run on; nothing is erased. Rows of a forge with no
    record are left, and named once in the log (../credentials.md, 2.1);
  - each forge's writer is restored from the main store, where skein's
    forge client keeps it (`Key::Writer(forge)`; ../credentials.md, 2.1);
  - a GitHub forge's delivery log is read from its newest delivery on:
    temper keeps no `guid` across starts, and skein's forge client
    reads each adopted repository afresh (../forge/README.md, a GitHub App's
    delivery log); `forge_component.log_catch_up_ms` bounds only a later
    start of a forge within the same process (`limits.md`);
  - carried rows that a run left at a stop are erased, a decision of
    the core that the routing carries out; a connect they belonged to
    ends interrupted, and the client asks to paste again
    (../credentials.md, 2.2);
  - an owned forge's handover is taken, when its file exists (`README.md`, section 5
    ), at the start, and at the box's `started` for a forge set up
    from the client, while the engine serves (../setup/owned.md, 5, and ../setup/box.md, 1);
    each managed and owned forge is reconciled by the engine's setup
    host, and shows "connecting" until reconcile is done, while the
    engine serves (../setup/README.md, the setup host in the engine); an owned
    forge with neither a handover nor a record of one is not set up
    until the client asks the box for it (../setup/owned.md, 5);
  - under the box, `temper-box internal serve` takes the listeners the box
    gives it, from fd 3, `LISTEN_FDS` of them, its clients' and webhooks'
    listeners claiming them by their addresses, and `--box` adopts the
    box's pipe, the two fds after them, refusing its start, naming the
    fd, when either is not an open pipe (../setup/box.md, 1); a start without
    the flag has no pipe, and leaves those fds alone;
  - every kept forge's identity that has a value is announced at
    generation 1 before the root's restart; a GitHub forge's, once its first tokens
    are minted (../credentials-git-identities.md, section 1). Every connected forge's user
    is checked by the forge component's first exchange (../credentials.md,
    section 5);
  - the branch namespace is derived from the deployment's id, the same
    at every start;
  - a later start gives no period from the smallest source, but the
    calendar's (`../../domain/routing.md`, 7) where a project's next start
    has passed: the store keeps each project's
    current one (jig's `domain/engine/README.md`, 5.4).
- **Repositories are never configured.** A person adopts each at runtime
  through the client, by the forge's number, the owner and the name, and
  the store keeps it (jig's `protocol/configuration.md`, section 3;
  `../../client/forge/family.md`).
- **What a change reaches,** after its restart: `machine`'s places,
  the forges made after it; a kept forge keeps the place and git
  program its record holds (../forge/engine.md, 1), while its setup entry is
  `machine`'s part of its kind (../setup/init.md), so a forge moved
  to another place is one removed and set up or connected again
  (../forge/engine.md, 3); the webhooks' listener, from the first delivery on; a worker's
  `git`, every invocation, checked at the start.
- **A forge's credentials are no change of configuration:** they are
  connected, replaced and forgotten from the client, with no restart
  (../credentials.md, section 5).

## 2. Open questions

None.
