# Adopting repositories

- **A repository joins a project with a role** (jig's `domain/model.md`,
  6.1): *owned*, where temper pushes its branches, opens pull requests
  and lands; a *fork* of an upstream (`README.md`, section 8); or *context*, read
  only. An owner of the project adopts it in the client, through the
  adoption form (`../../client/forge/family.md`, 4). A repository temper writes
  belongs to one project; others may adopt it as context, and each such
  adoption is kept for its project beside the writer's, so that the
  project's lookups, runs and follows read it (`follows.md`, section 8).

## 1. Repository identity and adoption requests

- **A repository is named by its owner and name, and kept by the forge's
  id for it.** The adoption names the forge, the repository's owner and
  its name; the connector first locates it, a read whose answer is the
  forge's own id for the repository, and names the repository by that id
  from then on (`Repository { forge, repository }`), so a rename or a
  transfer on the forge keeps it the same repository. A repository the
  forge does not have refuses the adoption as not found.
- **A task names a repository by its tag, the pair packed.** jig's
  resource number, opaque to jig, is temper's tag: `(forge << 32) |
  repository`, the forge's number (16 bits) in the high half and the
  forge's id for the repository (32 bits) in the low, packed and read
  back by one pair of functions in the forge's domain; a tag with a bit
  set above the forge's names none. A delegation's resource,
  a workspace's repository, what a run left and a pushed head carry it,
  and a saved resource's path is the pair itself (`../root-translation.md`, section
  1), so two forges' repositories with the same id in one project stay
  two. A tag whose pair the task's project has not adopted names none.
- **The adoption request is the forge's own,** keyed as any party's
  request (`../../client/forge/family.md`, 4): the root takes it from the protocol
  layer typed, and the connector refuses a forge the deployment does not
  keep, and an ended one as not connected (5); then locates the
  repository (`Read::Locate { owner, name }`, answered
  `Answer::Located { repository }`); then the root makes jig's
  adoption request on the person's behalf (jig's `domain/people/acts.md`,
  1), in which the connector reads what temper may do there (below)
  before the core commits the adoption (jig's `domain/connectors.md`,
  12). A forge not connected refuses it first, as such (5); a managed
  or owned Forgejo's access is asked for before the reads (6). Its
  outcome is the adoption's: adopted, refused by the core with jig's
  reasons, or refused by the forge with the connector's. The
  connector keeps it under the people child's token, to give again when
  the request is sent again, until its retention ends or it makes room
  for a newer request (section 13; jig's `domain/connectors-facts.md`, 1.5;
  `../../protocol/store.md`, section 2).

## 2. Forge address and permissions

- **The forge's host** in a repository's resource path, and in what a
  person reads of it, is the forge's as its record has it: the host of
  its `git` base, with `:port` when it names one
  (`../../protocol/forge/engine.md`, 1). The adoption does not ask a person for
  it.
- **What temper may do is read at adoption:** its user's permission on
  the repository; the repository's settings (the merge styles it allows,
  its default branch); the branch protection on the landing branch; and,
  but for context, the branches already under temper's prefix. Write
  permission, or admin, gives every kind of effect on an owned or fork
  repository, merging only where the repository allows a merge style;
  read gives reads alone; no permission refuses the adoption as
  forbidden. Forgejo refuses protection reads to a writer who is not an
  admin (skein's `forge/forgejo.md`, section 2): adoption records
  protection as unknown, never as absent, and its words say why
  (`../../client/forge/family.md`, 6). On GitHub temper's user is
  its App's bot, its permission the installation's on that repository,
  and protection the rules GitHub applies to the branch (8). A write
  the forge refuses narrows what temper may do there, for every later
  decision (jig's `domain/connectors.md`, 12).
- **Its CI workflows are read once it is adopted** (11): the names of
  the workflows on its landing branch, kept on its record and current
  with its landings, which CI's card names (`change/branch-path.md`).
- **The project's ceiling follows** (jig's `domain/authority.md`,
  section 6) from each repository's role and what was read: on an owned
  repository, pushing under temper's prefix, opening pull requests and
  landing into its landing branches; on a fork, pushing into the fork;
  on a context repository, reading. Protection that keeps temper from
  merging makes landing a wait for someone else's merge (`README.md`, section 7).
- **One deployment's prefix.** Adoption refuses a repository where
  branches under temper's prefix exist that the store records no making
  of: they are another deployment's (jig's `domain/connectors.md`, 12).
  A repository this deployment adopts already, as owned, adopted or a
  fork, is not checked again: its prefix has been this deployment's
  since that adoption, and the heads that recorded its branches' making
  are freed once no live work needs them (`keeping-up.md`, 1, *The
  branch heads*). One whose row went (its project deleted, its forge
  removed) is checked anew, and branches its earlier work left under
  the prefix refuse it until they are deleted.
  A prefix whose branches' names would pass the authority's segments
  (`authority.segments`, counting forge, host, owner, repository and
  `branch` before the branch's own parts) is refused as too large.

## 3. People and repository access

- **People the forge knows.** Adoption reads the repository's
  collaborators, each with its permission (read, write or admin), and
  gives the human collaborators to the core as parties known by their forge
  account: provider 0, with the subject of the repository's forge and the
  collaborator's user (`../../protocol/store.md`, section 2). temper's own writer
  is excluded, and so is the user the forge's link names (10): the link's
  person is known there already. The
  project's configuration maps each permission to a role, given to those
  with no role in the project yet (jig's `domain/people/README.md`, section 4):
  temper's usual mapping is admin to maintainer, write to member, read to
  observer (roles 1, 2 and 3 respectively, owner 0 reserved for the project
  owner). Re-adoption seeds people still without a role; it does not replace
  existing project roles. After that, roles are the core's: a permission that changes
  on the forge changes no role.
- **Who a forge's user is: one lookup, by forge and user.** A user's number
  means nothing without its forge: GitHub's user 3 isn't Forgejo's, and anyone
  may post a review on a public repository's pull request. The root's
  `forge_person` answers the link's person when the forge's link names that
  user (10), else the party whose key is provider 0's subject for that
  pair, with their role in the project, else unknown. A party seeded for
  the link's user before the link was made stays, role and all, but is
  never answered on that forge: nothing merges. Every reader of a
  person's forge account goes through it: the reviews counted (`change/gates.md`, 2), the people
  eligible (`change/gates.md`, 3) and the inventory's people (`../agent/agents.md`, 1). Going the other
  way, `person_forge_user` answers the user a party is on a forge only where
  `forge_person` answers that party for that user: the link's user for the
  link's person, else the user their key names. So a seeded party shadowed
  by the link is known on no forge it's shadowed on. A party counts as a
  person on a forge, `forge_human`, when `person_forge_user` answers them a
  user there who isn't that forge's retained writer. The subject's form is
  known to the forge's `user_key` and `key_user` alone.
- **Access is given per repository, at adoption.** On a managed or
  owned Forgejo, the adoption asks for it first: temper's user made a
  collaborator with write, and the repository's hook made when the
  deployment has a webhooks' listener; a refusal refuses the adoption,
  named (6). On a connected Forgejo a person gives temper's user its
  access, as a collaborator or through a team, and adds its hook or
  leaves the repository to poll. On GitHub the App's installation on the
  repository's account grants it, and its hints come from the App's
  delivery log, with no hook per repository (`../../protocol/forge/README.md`,
  4.1). The adoption then reads what it was given as any other.

## 4. Branch, workflow and provisioning choices

- **The adoption names, per repository:** its landing branch, kept on
  its record and read wherever temper lands (12); temper's
  branch prefix, `temper/` by default, which protection can reserve to
  temper's user; whether it is the project's home, where its goals'
  issues are kept (`issues.md`), one per project and never a context
  repository; and whether it runs CI, and if not, the checks its changes
  are gated on (`change/gates.md`, 1). The merge style is the forge's for the repository,
  and the adoption shows it (8).
- **Nothing is required:**
  - no webhook: polling is the backstop, webhooks only cut the latency,
    and the client shows a repository whose hints never arrive;
  - no label: temper writes none, and decides nothing by one, but for a
    project that turns findings on (`../findings.md`, section 3);
  - no protection: temper's rules are its enforcement, protection a second
    net;
  - no CI: a repository without it gates its changes on checks its
    adoption names (`change/gates.md`, 1).
- **Provisioning** (creating a repository, its webhook, its protection)
  is an optional set of effects a person may ask for, never a step of
  adoption (jig's `domain/connectors.md`, 12). The access a managed or
  owned Forgejo's adoption asks for is the setup host's run, not an
  effect of the connector's (6), and so are the labels a project's
  findings use (`../findings.md`, section 2).

## 5. Connected or not, ended and removed

Moved to [`adoption-connection.md`](adoption-connection.md), section 1.

## 6. Access at adoption

- **On a managed or owned Forgejo** the connector asks for access
  before the adoption's reads: `Adopt { forge, repository }`, through
  the root and the routing, which runs it on the setup's host with the
  admin's token (`../../protocol/forge/engine.md`, 6). The repository is the
  path the request carries, since temper's user may not see it yet. It
  makes temper's user a
  collaborator with write on the repository, and its hook when the
  deployment has a webhooks' listener and serves the forge one
  (`../../protocol/setup/reconcile.md`, 5). Done, the adoption reads as any
  other; refused, the adoption is refused, with the run's failure named
  (`../../protocol/setup/reconcile.md`, 4).
- **The connector never holds the admin's token,** and the forge
  component never does: the setup's host alone uses it, for reconcile,
  for access at adoption and for the picker's reads (7)
  (`../../protocol/credentials.md`, least power).
- **At each start** the repositories adopted on a managed or owned
  Forgejo are named to the routing for its reconcile, which checks and
  repairs each one's collaborator and hook (`../../protocol/forge/engine.md`,
  1).
- **Elsewhere there is nothing to ask:** on a connected Forgejo a person
  gives the access, and on GitHub the App's installation does (7). A
  repository this leaves temper's user only able to read is still
  adopted, and its answer says so (`../../client/forge/family.md`, 6).

## 7. What the picker offers

The adoption form lists, per connected forge, the repositories temper
can reach, a page at a time, by number (`../../client/forge/family.md`, 4; the
FORGE family's `Reachable { forge, installation?, page }`, answered
`{ rows, more }`). The request names no project, so the project's own
repositories are among the rows, and adopting one it holds is refused
as any adoption of it is. Each row is a repository's id, its owner's id
and login, its name and whether it is private.

Every kind is read the same way: the connector asks the root, which has
the routing run the setup host's plan `Reachable { forge, installation?,
page }` (skein's `forge/setup/forgejo.md`, 1.8; `../../protocol/forge/engine.md`, 6),
pages counted from 1, at most `reachable` rows (`picker.page`), ending
`Done(Reachable { rows, more })`. The plan writes nothing, so it runs
beside the forge's one writing plan, and no call of the forge client
reads the picker:

- **GitHub:** one accepted installation's repositories (`GET
  /installation/repositories`), `installation` its id, read on a token
  the host mints for the page and drops. An id the forge has not
  accepted is refused. The document is skein's (`forge/github-app.md`, 1.3).
- **A connected Forgejo:** the repositories temper's API token can push
  to, read on that token (skein's `forge/setup/forgejo.md`, 1.6).
- **A managed or owned Forgejo:** every repository the admin can see, by
  id ascending, since adoption gives temper's user its access (6). The
  admin's token stays the setup host's. Forgetting it makes the forge
  connected, and its picker with it.
- **"Not listed":** any repository by owner and name, located as any
  adoption is (`adoption.md`), and, when it cannot be reached, what would
  make it reachable, in words.

What a refusal says:

- **`NotConnected`:** the forge is offered for Connect, not listed.
- **`NotInstalled`** (GitHub: no accepted installation on the owner's
  account): "temper's App is not installed on <owner>", with "Install
  on another account".
- **`Missing`** on a repository whose account has an installation that
  selects repositories: "not granted to temper's App", linked to
  GitHub's installation page; on a connected Forgejo, "make <login> a
  collaborator". Once the forge says so, the repository is listed,
  with no restart.

An installation's repositories changing, or its account renamed
(`Repositories`, `Renamed`; `../../protocol/forge/engine.md`, 4), reaches the
connector with no names: it lists that installation again, shows the
account's login as that listing gives it, and marks each adoption whose
repository left the installation; a later listing that has it unmarks
it. Each `Connected` lists again every accepted installation that holds
an adoption, so a change made while temper was away is found. An
installation deleted marks every adoption on its account, and one
suspended marks them until it is unsuspended. The marks are held in
memory, never stored. A marked repository is shown with what would make
it reachable again (`../../client/forge/README.md`, a repository's health). Its
calls meet `Missing`, `NotInstalled` or `Forbidden`, which the connector
takes there as access lacking, as `NotConnected` is (5): nothing is
narrowed, and its changes wait.

## 8. What adoption says of the branch's rules

- **The rules on the landing branch** are read at adoption and said in
  words, each with what it means for landing, and the one setting that
  would let temper land where a rule keeps it from landing. temper
  never changes a repository's rules.
- **On GitHub,** the rules GitHub applies to the branch, from rulesets
  and classic protection alike, neither's absence taken for the
  other's: required approvals, required checks, code owners, a merge
  queue (`queues.md`, 1), signed commits, linear history, and whether temper's App
  may bypass them, `Never`, `PullsOnly` or `Always` (the App's own,
  never the ruleset's list). skein's `Protection` carries them as
  `queue`, `signatures`, `linear`, `code_owners` and `bypass` (skein's
  `forge/vocabulary.md`, 1.6; `skein/docs/design/forge/github-operations.md`, 2.3).
  Forgejo answers each false, and `Never`.
- **What each means:**
  - a merge queue: temper's changes enter it (`queues.md`, 1);
  - required approvals or code owners: a person reviews, or the App
    bypasses;
  - required checks: CI, as the landing rule's;
  - signed commits: temper's workers do not sign their commits, so
    where the forge refuses temper's merge for it, landing is a wait for
    someone else's merge (`README.md`, section 7; `README.md`, section 11);
  - linear history: a merge style that keeps it;
  - temper's prefix reserved to temper's user: welcome.
- **The merge style** is shown at adoption. On Forgejo it is the
  repository's default (skein's `forge/README.md`, section 10). GitHub keeps
  none: it is the first the repository allows of merge, squash and
  rebase (skein's `forge/github-operations.md`, 2.1). On a branch a merge queue
  guards, the queue merges by its own method, set in the branch's rules,
  and the adoption says so.
- **Commit statuses on GitHub:** temper's App reads statuses and writes
  none: `status` is false in the kinds there from adoption on, and
  nothing is held for it. Every adoption on GitHub says that the gates'
  verdicts (`change/gates.md`, 1) stay temper's own, and that GitHub's rules cannot
  require them. For a project that lands by turn, the App writes
  statuses and dispatches workflows (`../../protocol/setup/github.md`, 1), so
  that its receipts and verdicts are mirrored (`../landing.md`, 10), and
  adoption says, for each landing branch, whether its bypass is
  `Always`, which the turn needs (`../landing-turn.md`, 1).

## 9. A repository made or copied

`MakeRepository` (`../../client/forge/records.md`) asks the connector for a
repository on an owned or managed Forgejo, empty or copied from a
public address, adopted into a project when `into` names one
(`../../draft/repository-copies.md`, 5).

- **Admitted** by key, as any party's request: the root's scope is the
  project `into` names, or the deployment with none, and the connector
  then refuses any role but owner, `NotOwner`. In order, it refuses a
  forge it does not keep or that has ended, `NoSuchForge`; a forge that
  is not an owned or managed Forgejo, `NotOffered`; a name or an address
  that breaks `keeping-up.md`, 4 of the draft, or past `name_bytes`, `Invalid`; a
  `home` in a project that has one already, `HomeHeld`, which spares a
  copy that would be refused at its end; and a new row past
  `forge.makings`, `Full`. Its outcome is the request's, kept as an
  adoption's is (`../../protocol/store.md`, section 2): *kept*, or *refused*
  with its reason.
- **One row per name,** `RepositoryMaking`, the family `Making`
  (`../../protocol/store.md`, section 2). A second request for a name whose
  row is not failed is answered *kept*, as the first, and changes
  nothing. One for a failed row replaces it, keeping `copying` when it
  copies from the same address.
- **A failed row is dismissed** by `DismissMaking` (`../../client/forge/records.md`
  ): erased, its place in `forge.makings` free. No row at the name is
  answered *kept*; a row not failed is refused, `NotFailed`, and goes on.
  Admitted as `MakeRepository` is, from the project the request names; a
  row whose `into` is not that project is refused, `NotOwner`, and goes on.
- **Its state** moves one way: `Waiting`, `Making`, `Made { id, branch }`,
  `Adopting { id, branch }`, then erased; or `Failed { step, reason,
  words }`, the step `Admin`, `Org`, `Repository`, `File` or `Adoption`,
  the reason `Taken`, `Empty`, `TimedOut`, `Refused(status)`, `NotAllowed`
  or `Other`, and the forge's words, cut to 256 bytes.
- **The connector holds the rows; the setup runs them.** Only the
  service sees the forge's writing runs, so it pulls: when no writing
  run is live on a connected forge, it asks the connector for the
  forge's oldest `Waiting` row, starts its plan and tells the connector
  (`../../protocol/setup/owned-repositories.md`, 2). So a row waits for reconcile, a set up,
  an adoption's access or another copy, and restarts need no path of
  their own.
- **`Copying`** (skein's `forge/setup/forgejo.md`, 1.10) is the row's `copying`:
  the plan's write sets it in a decision, and the service answers
  `Written` once that decision is kept. A stop at `Taken` clears it.
- **Made,** with no `into` the row is erased: the repository waits for
  an adoption as any made one does. With `into` it is `Adopting`.
- **The adoption is the connector's own act,** jig's `Act::Adopt` under
  `engine.acts` (jig's `domain/people/acts.md`, 3, as the
  playground's reset acts; `../../protocol/setup/init.md`, 6), not a party's
  request, so that a sign-in that ended during a long copy, or a restart,
  cannot lose it. It names the repository by the id the plan gave, so
  nothing locates it: owned, landing on the branch the plan gave, with
  CI and no gates, the prefix `temper/`, the project's home when `home`.
  The act is keyed from the row's forge, org, name and wall time, so a
  restart joins it or is answered from its saved answer, adopted once.
  jig asks the connector back as it does for a party's adoption, and
  the reads and refusals of this section hold.
- **Refused,** the row is `Failed` at `Adoption`, `NotAllowed` for jig's
  refusals and `Refused` for the forge's, with what the adoption's
  refusal says; the repository stays made, adopted nowhere. A row that
  is asked again runs its plan, which finds its copy finished, and
  adopts it.
- **Adopted,** the row is erased in the decision that hears jig's answer.
- **A forge removed** erases its rows in its removal's decision (5),
  and its live run ends with the forge's runs (skein's `forge/setup/README.md`, 4.3).
- **A project deleted** while a row names it: the act is refused, and the
  row says so.

## 10. A box's person on its forges

Moved to [`adoption-person.md`](adoption-person.md), section 1.

## 11. Its CI workflows

Neither forge's API lists the workflows a repository runs, so the forge
reads the directory each forge runs them from, and keeps on the
repository's record what it found there, which CI's card names (`change/branch-path.md`,
*CI's card*).

- **The read** is the workflows' directory at a tip of the landing
  branch, the record's `landing` (12): its immediate entries,
  one `Read::File` (skein's `forge/vocabulary.md`, 1.2) answered as a directory's,
  `skein_forge::api::Answer::Directory`, within the client's
  `answer_bytes`, as a charter document's read is
  (`../../protocol/configuration/settings.md`, 6). On GitHub it is
  `.github/workflows`; on Forgejo `.forgejo/workflows`, and where that
  is absent `.github/workflows`, as Forgejo itself falls back. Where
  what asks for it names no tip, a `Read::Branch` of the landing branch
  gives it first.
- **A workflow** is each file entry whose name ends `.yml` or `.yaml`,
  named by its file name whole, as "ci.yml", in the forge's order; a
  directory, a submodule and any other file are none. The first
  `WORKFLOWS` (16) are kept, each at most `WORKFLOW_BYTES` (255) bytes,
  a file name's bound; a longer name, or one not UTF-8, is skipped
  whole. A listing the forge cut (`truncated`) gives what it listed.
- **What the record keeps:** `workflows`, the names, saved with the
  record (`../../protocol/store.md`, section 2) only when they differ from
  what it held. No directory there, on Forgejo neither, or a file where
  the directory would be, is no workflow: the list empties. A read
  refused or failed, or answered in another shape, keeps what the record
  held, and the landing branch's next move reads again whatever its
  files (below), by a mark held in memory alone: a restart forgets it,
  and the record keeps its last list.
- **When it is read,** at the landing branch's tip each time:
  - once an adoption saves the repository's record (`adoption.md`; the act
    of 9 alike), and once a refusal's refresh of what temper may do
    there saves it (jig's `domain/connectors.md`, 12), which keeps the
    record's landing (12). It is a read of its own, after the
    adoption's: the adoption's answer waits for none of it, and its
    failure refuses nothing. A context adoption that leaves another
    project's record as it is (`adoption.md`) reads nothing;
  - at the first tip the forge observes of the landing branch, which no
    landing's comparison precedes;
  - at a later move of the landing branch, at the tip it moved to, only
    where that landing's files name a path under `.github/workflows/` or
    `.forgejo/workflows/`, or are not known: the comparison refused, past
    its bound, or not made. The forge reads those files
    for every landing it publishes (`topics.md`, 1.1, the landings on a repository's
    branch), so a landing that touches no workflow costs no read, and a
    move by someone else is a landing as temper's own merge is (`topics.md`, 1.2).

  The landing branch's tip is watched while a live task or a
  subscription names it (`keeping-up.md`, 1); a move made while none does is seen at
  the next observation, from the tip the forge last held, and its files
  read then, as any landing's.
- **One read per repository at a time:** what asks for one while one is
  in flight reads once more when it ends, at the tip then. A repository
  released while its read is in flight keeps nothing of the answer.
- **Every workflow there is kept,** a dispatch-only one among them: CI's
  card leaves out those the repository's test gates and tiers dispatch
  (`change/branch-path.md`), since those settings change apart from the record.

## 12. Its landing branch

A repository has two branches that may differ: the forge's default
branch, its settings' `default_branch`, and the landing branch its
adoption names, where temper lands. One is the other's where the
adoption names the default branch; a deployment that lands into
`staging` on a repository whose default branch is `main` keeps them
apart.

- **The record keeps it.** The repository's record (`Repository`, in
  `temper-engine-domain-forge-vocabulary`'s `boundary.rs`) holds
  `landing: Box<[u8]>`, the adoption's landing branch: the form's
  (`../../client/forge/family.md`, 4), `repository adopt --landing
  BRANCH`'s, or, for a repository made or copied, the branch its plan
  gave (9). It is never empty: the adoption refuses an empty landing as
  past its bounds. `settings.default_branch` stays the forge's own, as
  read.
  - **The protection read** is that branch's (8), at adoption and at a
    refusal's refresh of what temper may do (jig's
    `domain/connectors.md`, 12). The refresh reads the forge's settings
    again and keeps them whole; a default branch it finds moved moves
    no landing.
  - **A re-adoption** names the landing again, as it names the prefix,
    and the record takes it (13).
  - **A context adoption** that leaves another project's record as it
    is (the opening) keeps that record's landing, the writer's; one
    that makes the record keeps its own.
- **Kept with the record** (`../../protocol/store.md`, section 2): the
  `Adopted` record's `landing`, `bytes 4096 = 0x`, its runtime bound
  the forge's `name_bytes`, as the adoption's `landing` is. A record
  stored before the field reads it empty, the default its plan gives;
  the store's decoder then takes the record's `settings.default_branch`
  for it, as the record is read (`../../draft/archive/layouts-both-ways.md`,
  4.2 and 4.4). Since no adoption keeps an empty landing, empty means
  that layout alone, and an existing repository lands where it did. A
  record is written with its landing whole. No version moves, and no
  job runs: a registered family's added field takes a default
  (`AGENTS.md`, 2).
- **Everything that means the branch temper lands into reads
  `landing`:**
  - a change whose landing is left out, otherwise its applicable held
    goal branch (`../agent/tools-change.md`, 1); `"branch": "mine"`'s with
    none; a change a person opens with no `into`
    (`change/opened.md`); and a change's base and its landing news's
    topic where its parameters name none;
  - a goal branch's base, at its making, its making again after a
    restart or a release (`branches.md`, 1), and a task's workspace
    whose change names no base;
  - the closer: whether the landing branch holds the goal branch, the
    goal's landing into it, and the pulls its report quotes as landed
    there (`branches-closer.md`, 1); and a closer-ended goal's skip of the
    landing branch's landings;
  - the landing queue: a train's batch cap and the project's landing
    gates and their proofs, a held goal branch's upstream, and a
    landing request's commit message and combining (`../landing.md`);
  - the CI workflows' read, at the landing branch's tips (11);
  - verification and tiers that follow the landing
    (`../after-landing.md`);
  - a follow whose source or target names no branch (`follows.md`);
  - a milestone's last landing (`../milestones.md`);
  - the changes views: `default_branch` on a landed row and
    `landed_default`, which mark a landing into the landing branch as
    the main one, and the panel's `landing` while nothing is queued
    (`../../client/forge/records.md`); the home's attention `branch`
    (`attention.md`);
  - a landed draft's read (`drafts.md`);
  - the branch path's landing (`change/branch-path.md`).
- **The forge's default branch is read where the forge's own is
  meant:**
  - the forge's settings, as `read_forge what=settings` answers them
    (`../agent/tools.md`): the forge's answer, which the refusals of
    a branch read point to;
  - Ask temper's `file` with no branch (`../agent/ask-temper.md`, 1),
    a read of the repository as the forge shows it;
  - `read_forge`'s `branch`, `protection` and `file` with no branch
    (`../agent/tools.md`, *The forge's reads are three tools*), each
    read of `settings.default_branch`'s name;
  - a repository's store, a bare clone of the forge's default branch,
    git's `HEAD` (`../../protocol/git-operations.md`, 1.5;
    `../worker/checkouts.md`), from which every checkout then fetches
    its own base.
- **Reads that show the record show it:**
  - the adoption's outcome, `landing` beside its role and home: its
    words, "It lands into BRANCH." (`../../client/forge/family.md`, 6),
    `repository adopt`'s text and JSON, and its kept form, the stored
    `ApplicationAdopted`'s `landing`, `bytes 4096 = 0x`, of which an
    outcome kept before the field says nothing;
  - the inventory's repositories, each with its landing branch
    (`../agent/agents.md`, 1).
- **Where the landing branch is the default branch,** every reader
  reads the same name as before, and nothing behaves otherwise.

## 13. Kept answers

The answers to the forge's keyed requests, each kept under the people
child's token and given again when its request is sent again (jig's
`domain/connectors-facts.md`, 1.5): an adoption's (the opening's bullets), an
opening's (`change/opened.md`, 2), a repository's making and its
dismissal (9), and a follow's act (`follows.md`). One table holds them
all.

- **Its places:** at most `forge.applications`, each an answer kept or
  a request pending, admitted and not yet answered, at most
  `forge.adoptions` of those. The figure bounds what the table stores,
  never how many requests may be decided within a retention: no request
  is refused because many answers are kept.
- **Expired answers go first:** each new request first drops every kept
  answer whose retention has ended, all of them at once, and erases each
  one's row (`../../protocol/store.md`, section 2, tag 15).
- **The oldest answered makes room:** where the places are still all
  taken, the kept answer whose retention ends first is dropped, the
  lowest token first among equals, and its row erased in the same
  decision; under one retention it is the oldest answered. The request
  takes the place freed.
- **A pending request is never dropped:** its answer is still to come.
  With every place pending, a request is answered busy, which passes as
  they are answered (jig's `domain/people/acts.md`, 1).
- **Room only for a request that takes a place:** one answered busy for
  its flights, with `forge.adoptions` requests pending or repositories
  being read, is answered so before any answer is dropped.
- **Sent again once its answer went:** jig's people child keeps the key
  and its token for its own retention (jig's
  `client/domain/contract/requests.md`, 3), so the request is not
  decided again: its connector no longer keeping the outcome, it is
  answered unknown, as one past its retention is. An opening sent again
  so opens nothing new.
- **Seen where the store's tables are read:** the store's census
  (jig's `protocol/store/growth.md`, 1.3), in `temper ctl deployment
  census` and the Limits page (`../../client/ctl/results.md`;
  `../../client/pages/settings.md`, 1). Its `temper.applications` row,
  the store's family 15, counts the kept answers as its rows. The
  requests pending are not shown (14).
- **The figures stay:** `forge.applications` 32, `forge.adoptions` 2,
  and the request retention a day (`people.request_retention_ms`), in
  the standard profile (`../../protocol/configuration/limits.md`).

## 14. Open questions

- **A workflow that runs on no pull request** (`adoption.md`, 11; `change/branch-path.md`, *CI's card*):
  CI's card names every workflow file on the landing branch but those
  the repository's test gates and tiers dispatch, so a release's or a
  schedule's workflow is named though it reports nothing at a change's
  head; and it names each by its file, not by the `name` its file gives
  it. Telling them apart reads each file and its triggers, a read per
  workflow at each change of the directory. Default: by file, every one
  but the dispatched.

- **What a refused copy leaves** (`adoption.md`, 9; skein's `forge/setup/README.md`,
  section 6): Forgejo keeps an empty repository at the name of a source
  it could not clone, and only the same address asked again takes it
  away: asked from another, the row holds no `copying` and stops at
  `Taken`. Default: so, and the person deletes it on the forge.

- **A merge style per repository,** named by the project rather than the
  forge's (`adoption.md`, 8). Default: on Forgejo the repository's default (skein's
  `forge/README.md`, section 10); on GitHub the first the repository allows of
  merge, squash and rebase; either shown at adoption.


- **`Forbidden` and `Missing` taken as access lacking** on a repository
  an installation suspended or stopped selecting (`adoption.md`, 7), rather than as
  facts that narrow what temper may do there. Default: access lacking,
  while the installation's events say so; any other refusal narrows, as
  ever.

- **Signed commits required** on a landing branch (`adoption.md`, 8): temper's
  workers do not sign. Default: landing there is a wait for someone
  else's merge, said at adoption; signing is later.


- **Another landing branch after adoption** (`adoption.md`, 12): a
  re-adoption naming another landing branch is taken, as a prefix is,
  and the goal branches made from the former one land into the new one
  at their closers. Changing a live adoption's landing on purpose, and
  more than one landing branch per repository, are not designed.
  Default: so.

- **The pending requests where the store's tables are read** (`adoption.md`,
  13): the census shows `temper.applications`' kept answers as its rows,
  but not the requests pending beside them, which take places in the
  same table. Showing them gives jig's census row a count its owner
  sets as the report is handed on, and moves jig's client wire
  (`../../draft/archive/redeploys.md`, section 8): out of proportion to the
  table's fix, the user judged (2026-11-19). Default: not shown.
