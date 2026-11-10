# Copies of public repositories

Draft, 2026-11-04, from a session with the user that day. A person
copies public git repositories into a Forgejo temper runs, and has them
adopted into a project: from the welcome's Forgejo step, as the
greenfield mockup's "Copies of public repositories" offers, and later
from a project's repositories in Settings and from `temper ctl`. All
three ask the same request of the engine, built on one plan of skein's
setup host; the views over it come last, once their look is settled
(section 10).

## 1. In one page

- **"Copy", in the person's words** (section 2): the source stays where
  it is, and nothing is sent back.
- **A plan of skein's, `Copy`,** beside `Create` (skein's
  `forge/setup/forgejo.md`, 1.9): the org when missing, the repository copied
  from a public address by Forgejo's own migration, the bot's access
  and hook, and the starter workflow; read first at every step, so a
  run cut short, run again, finishes it (`repository-copies.md`, section 3).
- **Forgejo fetches it,** not temper: no bytes of the source pass
  through the engine, and no git credential is minted for it
  (section 4).
- **One request of the engine, `MakeRepository`,** for an empty
  repository or a copy, adopted into a project when one is named, the
  project's home when asked: the welcome, Settings and `temper ctl`
  send it alike (section 5).
- **Kept as it goes:** a new family of temper's store holds each
  repository being made, so a restart finishes it and every client
  shows it live (section 5).
- **Tested without a network:** the fake forge copies from a catalogue
  of sources its owner seeds; the conformance records Forgejo's
  migration on the throwaway Forgejo, from a source on loopback
  (section 6).
- **The welcome** keeps the addresses with the set up's intent, and
  Ready copies them into the project it makes, the first the project's
  home and its name (section 7).
- **One sheet, "Add repositories",** with three sources: one the
  forge has already, a new empty one, a copy of a public one
  (section 9).
- **Changed in place:** until v0.1.0 there is no supported deployment,
  so the store's families, the FORGE records, `CLIENT_VERSION` and the
  set up's intent change in place, with no version kept for an older
  side (the user, 2026-11-04).

## 2. The word

- **"Copy",** as the mockup says it: "Copies of public repositories",
  "Copy a public one". A copy is the person's to change; its source
  is untouched and hears nothing of it.
- **Not "import",** which reads as moving a project from one host to
  another, issues and all; **not "mirror",** which promises to keep up
  with the source (section 11). Forgejo's "migrate" stays the API's
  word, inside skein's plan.

## 3. The plan: `Copy`

`Copy { forge, org, repository, from, files }` copies a public
repository into a managed or owned Forgejo, ready for the service, run
by the admin's token after reconcile, as `Create` is (skein's
`forge/setup/forgejo.md`, 1.9). Its steps, each reading first and writing only
what is missing:

1. **The admin's token,** as `Create`'s step 1.
2. **The org,** as `Create`'s step 2: made private, owned by the orgs'
   owner, when missing; left as it is when present. A name that is a
   user's on the forge, not an org's, stops at `Org`, the subject the
   name.
3. **The repository,** `GET /api/v1/repos/{org}/{name}`:
   - absent, it is copied: `POST /api/v1/repos/migrate`, the clone
     address `from`, the owner `org`, the name, private, the service
     plain git, no mirror, and nothing but git: no issues, pull
     requests, releases, wiki or LFS objects (section 11);
   - present and holding a branch, and named by the owner's record of
     this copy (5.2) as being copied, it is the copy, finished;
   - present and empty, so named, it is what a copy cut short left
     (section 4): it is deleted and copied again;
   - present and not so named, the name is taken: the run stops at
     `Taken`, the subject `org/name`, and writes nothing.
   Either way the plan holds the repository's id and its default
   branch, the source's.
4. **The bot's access,** as 4.5's steps 3 and 4.
5. **The starter workflow,** as `Create`'s files: the owner's files,
   put on the default branch where absent. temper gives the starter
   workflow (`../protocol/setup/owned.md`, 7), so that the copy's checks
   run on the owned runner from its first change (`repository-copies.md`, section 8).
6. **The outcome,** `Done(Copied { id, branch })`.

- **`from`** is an address the owner checked (5.4): `https://`, a host,
  an owner and a name, at most `string_bytes`. The plan sends it as it
  is; whether the host is reachable is Forgejo's to find.
- **Its deadline is its own,** longer than the other plans': a copy's
  time is the source's size over the network's speed. A run past it
  stops at `Repository`, `TimedOut`; the copy Forgejo may still be
  making is found at the next run (step 3).
- **A writing run,** one per forge at once (skein's `forge/setup/README.md`,
  4.3): copies on one forge run one after another.
- **Forgejo's refusals,** each a stop at `Repository` with Forgejo's
  status and message, cut to the host's bounds: a source missing or
  private, a host refused by Forgejo's migration settings, a source
  that is not a git repository.

## 4. Forgejo fetches it

The copy is Forgejo's migration, not a clone by temper's git program
and a push (the user, 2026-11-04):

- **No bytes through the engine.** A copy may be large; Forgejo fetches
  it into its own repositories' directory, as it would fetch from its
  web page. temper's engine keeps its memory and its disk for its own
  work.
- **No credential to mint.** A push would need a git token of the
  admin's or the bot's on the engine's side for the copy's life; the
  migration runs under the admin's token the setup host holds for the
  run, and drops (skein's `forge/setup/README.md`, 4.3).
- **The same plan for owned and managed.** A managed Forgejo is not on
  temper's machine; its migration is still its own.
- **Its settings are Forgejo's defaults:** the owned Forgejo's
  `app.ini` (`../protocol/setup/owned.md`, 2) writes nothing of migrations,
  so copies from public hosts are allowed and addresses on local
  networks refused. `OFFLINE_MODE` concerns the pages' assets, not
  migrations: a fact the conformance records (`repository-copies.md`, section 6).
- **What a copy cut short leaves** (a restart of the box or of
  Forgejo, a deadline) is Forgejo's, recorded by the conformance: the
  plan, run again, reads it first (section 3, step 3).
- **Not for a connected Forgejo or GitHub:** temper holds no admin's
  token there to make a repository with (section 11).

## 5. The request: `MakeRepository`

### 5.1 What it asks

A FORGE request, keyed as any party's (`../client/forge/records.md`):

```
MakeRepository {
    forge,                      // an owned or managed Forgejo
    org, name,                  // where it lands on that forge
    from: Empty | Copy { address },
    into: Option<Into { project, home: bool }>,
}
```

- **`Empty`** runs `Create`, as a set up's first repository does
  (`../protocol/setup/owned.md`, 7), with the starter workflow; **`Copy`**
  runs `Copy`.
- **`into`** adopts it once made, as `temper ctl forge setup`'s
  `--project` does today, in the engine instead of the call: owned,
  landing on its default branch, with CI, the project's home when
  `home` (`../domain/forge/adoption.md`). The adoption is the forge
  connector's own, judged as any; refused, the repository stays made,
  adopted nowhere, and its row says why.
- **Admitted** from an owner of the project `into` names, or, with
  none, from an owner of the deployment, as `SetUp` is.

### 5.2 Kept as it goes

- **A family of temper's,** `Making`, at the store's next free byte
  (`../protocol/store.md`, section 2), keyed by the forge, the org and
  the name: one making per name at once, a second request for it
  answered as the first while it runs.
- **Its row:** what was asked (`from`, `into`), the party, the wall
  time asked, and its state: waiting for the forge's writing run,
  making, made (the id and the branch), adopting, adopted, or failed
  at its step with the forge's words.
- **Written** at admission and at each state; **erased** once adopted,
  or when its forge is removed; a failed row stays, for the person to
  read, until the same name is asked again or the row is dismissed.
  Bounded by `forge.makings`, the rows kept at most.
- **At a restart,** a row waiting or making runs its plan again,
  which reads first (section 3); one made or adopting goes on to its
  adoption.
- **The client reads them** with each forge's row, on the `forges`
  watch (`../client/forge/family.md`, 4), so every client shows "Copying
  ORG/NAME from HOST…" while it lasts, from the step's start to its
  end, with no percentage: Forgejo's migration says none.

### 5.3 Where a copy lands

- **The source's owner and name:** `github.com/acme/widget` becomes
  `acme/widget` on the forge, as the mockup has it, so the copy says
  where it came from. A project's repositories may span orgs, so
  copies from several owners sit well in one project (the user,
  2026-11-04).
- **Another place is the person's:** the sheet and `temper ctl` take
  an org and a name of their own (section 8), for a name taken or a
  user's.

### 5.4 Addresses and limits

- **An address** is `HOST/OWNER/NAME`, with or without `https://`
  before it and `.git` after it: HOST a DNS name with an optional port,
  OWNER and NAME 1 to 100 bytes of `[A-Za-z0-9._-]`, neither `.` nor
  `..`. `http://`, `ssh://`, `git@` and paths of more segments are
  refused with the form to write (section 11 for more).
- **Checked twice,** by the client before it asks and by the forge
  domain at admission, as the first repository's names are.
- **`forge.makings`,** the rows kept at once, deployment-wide; a
  request past it refused as full.

## 6. Tested without a network

- **The fake forge copies from a catalogue** (skein's `testing/fakes/forge.md`,
  section 2): sources its owner seeds, each an address and a
  repository in the fake's own git store, under hosts of the reserved
  `.invalid` domain. Its migration route copies one into the forge,
  answers a missing or private one as Forgejo does, and takes the
  domain's timing faults (skein's `testing/fakes/forge.md`, 2.3), so a copy can
  be slow or cut short.
- **`temper box --fake-backends`** seeds two, and prints their
  addresses with the files a walk pastes (`../protocol/setup/box.md`,
  5): a journey copies them in the welcome.
- **The conformance** (skein's `testing/fakes/forge.md`, section 7) records each
  fact `Copy` relies on, on the throwaway Forgejo, from a source served
  by a second throwaway on loopback, its migration settings allowing
  local networks in the throwaway alone: the migration's answer, a
  missing source's, the repository's state after a migration cut
  short, the default branch kept, `OFFLINE_MODE` leaving migrations
  on, and which workflow directory Actions reads when both are there
  (section 8). They join skein's `forge/setup/forgejo.md`, section 2.
- **No test reaches a real host.**

## 7. The welcome

- **The Forgejo step's choice,** "What it starts with": an empty
  repository (the default, today's `work/hello`), copies of public
  repositories (their addresses, one per line), or nothing for now
  (`first-run-projects.md`, section 4).
- **The set up's intent keeps it:** `SetUp`'s first repository becomes
  one of `Empty { org, name }`, `Copies { addresses }` or `Nothing`,
  kept in the forge's record as today's org and name are
  (`../protocol/setup/owned.md`, 7). With copies, the set up makes no
  repository: it connects the forge, and the copies wait for a
  project to go into.
- **Ready copies them in:** its last act makes the project, then asks
  `MakeRepository` for each address, in the order given, `into` the
  new project, the first its home. The project is named after the
  first copy's name, its first letter in capital ("Widget" for
  `github.com/acme/widget`), as the first repository names it today
  (`first-run-projects.md`, section 4). Its lead says so before it is
  pressed: "Create your first project, with acme/widget copied into
  it", and "and N more" for more.
- **Not waited for:** the project opens once made; the copies run on,
  one after another, and Home's "To begin" says each while it lasts
  (`to-begin.md`, section 3): its repository step a line per copy,
  "Copying acme/widget from github.com…" for the one running and
  "Waiting to copy" for the next, then "acme/widget is in Widget",
  checked. A copy that fails says the forge's words, with "Copy
  again". How a copy under way looks is a look pass of its own
  (the user, 2026-11-04).
- **At Ready,** never while the set up connects: one way to ask and
  adopt a copy, the welcome's and Settings' alike (the user,
  2026-11-04).
- **A refresh or a restart in between:** the intent still names the
  addresses, and the forge's row shows them not yet asked, so the
  project's page offers them again, filled, as an adoption not
  finished is offered (`first-run-projects.md`, section 4).

## 8. A copy's checks

- **The starter workflow is added** (section 3, step 5; the user,
  2026-11-04): a copy from
  GitHub carries `.github/workflows` written for GitHub's runners,
  which the owned runner's label does not take, so its checks would
  wait for ever. With `.forgejo/workflows/ci.yml` present, Forgejo's
  Actions reads that directory alone (a fact the conformance records),
  and the copy lands its first change through temper's merge queue,
  checked by the starter (`../domain/forge/change/README.md`, 2).
- **Its own checks are the person's to bring:** a chat can turn
  `.github/workflows` into the owned runner's, as any change.
- **Why a copy needs one:** the queue lands a change once the
  repository's CI passed at its head, and again after it is brought up
  to date with main (`../domain/forge/change/README.md`, 2, and `../domain/forge/change/gates.md`, 2); CI pending
  is a wait. An implementer's own tests run before its change is
  pushed, and the queue trusts none of them. A repository adopted
  without CI names checks that stand for it (`../domain/forge/change/gates.md`,
  1).

## 9. Adding a repository: one sheet

- **Three ways a repository joins a project,** which a person sees as
  one question, "Where does it come from?": one the forge has already
  (the picker, then the adoption form, `../client/forge/family.md`, 4), a
  new empty one, or a copy of a public one.
- **One sheet, "Add repositories",** a dialog over the page (a sheet
  from the bottom on a phone), with the three as its sources. The forge
  is chosen inside the first, where there are several: a repository is
  only made or copied on a Forgejo, so GitHub chosen above them would
  leave two sources with nothing to do. Those two say which forge they
  land on, and take a Forge field of their own only where there are
  several Forgejos (the user, 2026-11-06). The adoption's
  own fields (role, home, landing branch, CI, checks) under "More",
  their defaults those of section 5.1. From Settings' project
  repositories, from a project with no repository (`first-run-projects.md`,
  section 5) and from "To begin"'s second step (`to-begin.md`, section 3).
- **The welcome shows two of them,** empty and copies, since its forge
  is new and has nothing to adopt; with "Nothing for now" beside them.
- **They meet:** a copy whose name is taken offers "Adopt it" for the
  one already there, filled; the picker's rows and the copies being
  made are listed together, by name, so a person sees one list.
- **Plural,** since a person may add several at once: copies, one
  address a line, as the welcome takes them; rows of the picker
  (the user, 2026-11-04).
- **Light for now:** the sheet's look is tuned on the person's
  `look-requests` branch (`../client/web/journeys.md`, 7) once the
  foundation is in.

## 10. Its order

1. **The foundation, now:** skein's `Copy` plan, with its conformance
   facts and the fake forge's migration and catalogue (sections 3 and
   6); temper's `MakeRepository`, the `Making` family, its restart and
   limits, and `temper ctl repository copy` and `repository make`,
   which send it (section 5, and below), so that a copy can be made and
   adopted from the command line on the fake backends and on a real
   box.
2. **The welcome** (section 7): the set up's intent, the Forgejo
   step's choice, Ready's copies.
3. **The sheet** (section 9) and Home's words for a copy under way.

`temper ctl`'s two commands, beside `repository adopt`
(`../client/ctl/tools.md`, 3): `repository make --project PROJECT
[--forge FORGE] ORG/NAME [--no-home]`, and `repository copy --project
PROJECT [--forge FORGE] ADDRESS [--as ORG/NAME] [--no-home]`, each
answering once the request is kept, and `--until adopted` waiting on
the row.

## 11. Later

- **Home's repository sheet and entry drawings** (9): temper/ui's
  repository-copies follow-up unifies Home's empty-or-copy sheet with
  Add repositories and draws the entry from a project with no repository
  and To begin's second step. Settings uses the three-source sheet;
  `connect-flows` leaves those two entry paths unmocked.

- **Keeping a copy up to date** from its source (Forgejo's pull
  mirror), which temper's landings on it would fight.
- **Issues, pull requests, releases, the wiki and LFS objects,** which
  Forgejo's migration can bring from some hosts, with a token for
  most.
- **A private source,** with a token of the person's for its host,
  kept as a forge's credentials are.
- **Paths of more segments,** GitLab's subgroups among them.
- **Copies into GitHub or a connected Forgejo,** where temper holds no
  admin's token.

## 12. Open questions

None: the user settled the six this draft asked on 2026-11-04:
Forgejo's migration (section 4); `MakeRepository`, the `Making` family
and the set up's intent as written (sections 5 and 7); a copy at its
source's owner and name (5.3); the starter workflow in every copy
(section 8); one sheet, "Add repositories" (section 9); the welcome's
copies at Ready (section 7).
