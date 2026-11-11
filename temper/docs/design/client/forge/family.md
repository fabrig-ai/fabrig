# The forge's client family

The forge's client side is the client root's child for the forge
connector, an owner numbered as its connector (jig's
`client/domain/root.md`, sections 2 and 15). It shows the forge's parts
that need no document, reads the documents of the others (`README.md`),
holds one form, the adoption, with its picker, sends the forges'
page's requests (`../engine.md`), and reads, starts and resets the
playground (7).

## 1. Words that need no document

A connector, a resource or a kind needs no document: its words come from
its number or its path.

- **The connector:** "forge".
- **A resource,** by its path (`../../domain/forge/README.md`, section 2):

  | Path | Words |
  |---|---|
  | `forge` HOST OWNER NAME | OWNER/NAME on HOST |
  | … `branch` S1 … Sn | branch S1/…/Sn of OWNER/NAME on HOST |
  | … `pull` N | pull request #N of OWNER/NAME on HOST |
  | … `issue` N | issue #N of OWNER/NAME on HOST |
  | two segments: a forge's number (2 bytes) and a repository's id (4 bytes), big-endian, as an adoption names it | repository ID on forge F |
  | three segments: a forge's number (2 bytes) and a repository's id (4 bytes), big-endian, then a branch's name, as the hub names a delegate's branch (`temper-engine-domain`'s forge route) | branch B of repository ID on forge F |
  | anything else | its segments joined by `/`, as they are |

  jig's documents name an adopted repository by its forge's number and
  its id (`../../domain/forge/adoption.md`), and the client shows those;
  the forge's documents name it by owner and name (`README.md`, 2.5).
- **A resource pattern** (a grant's): the words of its segments as
  above, its last segment as it is; an open end is its prefix followed
  by `…` ("branch temper/7/… of acme/shop on forge.example").
- **A kind,** by the forge's numbering (`temper-engine-domain-forge`'s
  `effect_shape`; `../../domain/forge/README.md`, section 4):

  | Kind | Words | Kind | Words |
  |---|---|---|---|
  | 1 | read | 9 | create a branch |
  | 2 | push | 10 | edit a pull request |
  | 3 | open a pull request | 11 | update a pull request from its base |
  | 4 | merge a pull request | 12 | close or reopen a pull request |
  | 5 | post a review | 13 | edit, close or reopen an issue |
  | 6 | set a commit status | 14 | set reviewers |
  | 7 | post a comment | 15 | delete a branch |
  | 8 | create an issue | any other N | kind N |

  These are the forge client's writes in words (its `Write`
  vocabulary), with reading and pushing. A new kind is a new row here
  in the same change that numbers it.

## 2. The forge's other parts

Every other part of the forge's that jig's documents name (a panel, a
result, an effect, an outbox entry, a hold, a project's part, the
deployment's part: jig's `client/domain/root.md`, section 6) needs a
document, which the forge's side reads or watches through the family's
own reads and subjects (`served.md`). Wherever a page would
show a part the forge's side does not read, temper's view shows one
paragraph:

> Shown on the forge.

Such a part never shows loading, and never an error. A change's
procedure part on its task's page is not such a part: the forge's side
watches its panel (`served.md`, 2), and the page draws it, or says it
is loading, unknown or cannot be shown (`../pages/task.md`, 3).

## 3. Where the form is

`Adopt { project }` draws the “Add repositories” dialog over Settings’
Repositories (4). Settings’ button and links to adoption open it; Cancel
or Escape returns to Repositories. A project with no repository retains
its Chats band’s entry, which reaches this same address. Home’s second
step can adopt the offered first repository directly, open its own
empty-or-copy sheet, or link to adoption (`../pages/home.md`, 1). Home’s
own sheet stays Home’s, beside this one (the user, 2026-11-11; the draft
status's departure, `../../draft/repository-copies.status.md`). The
two entry paths into this dialog that `connect-flows` left undrawn are the
gallery's (`../web/gallery.md`, 2): `sheet-from-home`, To begin's
repository step's "Adopt another…" pressed, and `sheet-from-chats`, the
Chats band's "Adopt a repository" pressed, each the dialog as that link
opens it for a project with no repository.

## 4. The sheet "Add repositories" and its adoption form

`Adopt { project }` is Settings' Repositories with a dialog over it, a
sheet from the bottom on a phone (`../../draft/repository-copies.md`, section
9; the request `connect-flows`). Its button on that tab, and every link
that said "Adopt a repository", go to it; its Cancel and its Escape go to
`Repositories`. A reload draws it again, empty.

- **Its head:** "Add repositories to PROJECT" (`project N` until `me` names
  it), and the sources, one choice of three: "Adopt", "Make empty" and "Copy
  public". The head stays where it is as the source changes; only the foot
  moves. With no Forgejo that takes them (owned or managed), the last two go,
  and the control with them.
- **Adopt:** a Forge field, where there are several (never "Forge number");
  an Account field, GitHub's, one per installation, where there are several,
  "LOGIN · selected repositories" or "LOGIN · suspended" where they are; then
  the picker of `repositories.md`, 1; "Not listed", folded; "More", folded, its
  summary saying what it's set to ("More · Owned, it runs CI"), holding the fields
  below: "What temper does with it" and its three roles, "Branch prefix",
  "Landing branch", "Checks: gate numbers, separated by spaces or commas",
  "It runs CI" and "Make it the project's home". A chosen new forge or account
  clears the search, the rows and the picks.
- **Make empty:** "Organization" (`work`), "Name", then "On FORGE, with a
  README and a starter CI check.". The button says "Make ORG/NAME", or "Make
  it" until both are typed.
- **Copy public:** "Their addresses, one per line", the browser's caption over
  a field four lines high; the terminal prints the field's name once.
  Read as typed, a line at a time, each with its own words: "Not an address:
  LINE. Write it as github.com/owner/name."; "HOST isn’t a host. Did you mean
  github.com/OWNER/NAME?", with "Use it"; and, for a name the target forge
  has already, among the rows read of it and its makings, but for one that a
  failed copy of the same address left there—an empty repository that copying
  again replaces—"OWNER/NAME is already on this forge.", with "Adopt it", which
  moves to "Adopt" with its row picked. Then "Copied into FORGE from GitHub or
  another git host. Each
  keeps its owner and name, and nothing is sent back.". The button says "Copy
  OWNER/NAME", "Copy N repositories" or "Copy", and is disabled while a line
  has words.
- **Forge:** the last two sources take a Forge field of their own only where
  there are several Forgejos.
- **Its foot:** "Cancel", quiet, and the primary, disabled until it can send.
- **Sent:** adoptions one at a time, each a keyed `Adopt`; `MakeRepository`
  for each line or the one name, `into` this project, `home` the first where
  the project has no home. All kept or adopted, the sheet closes to
  `Repositories`. A refusal keeps the sheet open, with what is left picked or
  written, and its words above the foot; those already done are off it.

The adoption form, under "More" and the picker. The engine
judges the adoption, and a refusal at admission (the party's role in
the project) is shown in jig's words (`records.md`). Its fields are
`temper-engine-domain-forge`'s `Adoption`'s but the repository's id,
which the forge connector locates by owner and name
(`../../domain/forge/adoption.md`), and the forge's host, which temper's
server fills from the forge's record (`../engine.md`). A row chosen in
the picker, below the forge's field, fills the owner and the name (the
picker, below). In order:

| Field | Node, named | Default | Rule |
|---|---|---|---|
| forge | the Forge field, by name | the lowest listed, or a first repository's forge (below) | one the store does not keep is refused as no such forge |
| owner | text field "Owner" | empty, or a first repository's organization (below) | required |
| name | text field "Name" | empty, or a first repository's name (below) | required |
| role | choice group "Role": "Owned: temper pushes, opens pull requests and lands", "Fork: temper contributes through a fork", "Context: temper only reads" | owned | one of the three |
| home | check "Make it the project's home", for a single pick | checked only if the project has no home | a context repository cannot be the home; hidden for plural picks |
| prefix | text field "Branch prefix" | `temper/` | required |
| landing | text field "Landing branch" | empty, or `main` for a first repository (below) | required: the forge connector refuses an adoption without one |
| ci | check "It runs CI" | checked | without CI, at least one check |
| checks | text field "Checks: gate numbers, separated by spaces or commas" | empty: none | numbers from 1, each once, at most the client's limit on checks |

The foot’s primary names the selected repository or counts the picks;
request state and refusals remain above the foot.

**From a project with no repository** (`../pages/README.md`; `../../draft/first-run-projects.md`,
4 and 5): the chats page's "Adopt a repository" opens this form into
its project, always a project's: no form makes a project. While a
set up's first repository is made and adopted nowhere
(`../../protocol/setup/owned.md`, 7), as the `forges` watch's rows say it (a
real forge's row whose `first` is made, and no row's `repositories`
naming its organization and name, `records.md`), the form opens filled for it,
so that adopting it is one choice away: its forge's number, its
organization as the owner, its name, and `main`, the branch its
`Create` made, as the landing branch; the other fields at their
defaults. With several, the one on the forge numbered lowest. The person may
change any field, and "Adopt" sends `Adopt` alone.

**The picker** is below the optional Forge and Account fields. It reads
`Reachable` pages from 1, retains rows in forge order up to `picker.rows`,
and reads on when the list nears its end or a search needs more rows
(`repositories.md`, 1). More than eight rows, or more still to read, gives it
“Find by name”, its own height and a count; a small completed list has
none of those extras. A row is a checkbox named OWNER/NAME, marked private
where supplied. “In PROJECT” and “Copying…” rows are disabled. Picks
survive search, but a change of forge or installation clears them. No
Next or First page button is drawn. “Not listed” holds Owner and Name
and the forge-specific remedy; a completed search with no matches opens
it with the typed name. GitHub’s account field appears only for several
installations; the wire carries no selected/all mode, so its remedy asks
the person to check existence and grant the repository to the App.

- **The ids ride, never shown.** A row carries the forge's id for the
  repository and its owner's id, besides the owner's login, the name and
  whether it is private; the form keeps the two ids for the `Adopt`
  request while "Owner" and "Name" are the row's, and drops them when
  either is edited by hand (`records.md`).
- **Refused, in words.** A forge not connected (`NotConnected`):
  "Forge F is not connected.", linked to the forges' page, and the primary
  disabled for that forge until a later page lists its repositories. A
  forge the store does not keep: "No forge F is configured." Refused for the
  person's role, which does not hold the deployment's connectors:
  "Your role does not hold the deployment's forges: give the owner and
  the name.", and the person adopts by owner and name, the
  engine judging the adoption as any other (`../README.md`, section 5). A refusal at
  admission: jig's words. A forge that did not answer: "Forge F did not
  answer.", with "Try again". Busy, another read of the forge's picker
  running or the setup's host full: "Forge F is busy.", with "Try
  again". On a managed or owned Forgejo, the admin's token missing or
  not a site admin's: "Forge F's admin's token is missing." or "Forge
  F's admin, NAME, is not a site admin.", linked to the forges' page.
  None of these empties the fields.
- **Read on as `repositories.md`, 1 has it.** The form keeps the rows read, up to
  `picker.rows`; an answer for a forge or installation no longer chosen is
  dropped. A page that answers no rows ends the reading: more no longer
  follow.

- **Every field is bounded** by the client's limit on a field's bytes,
  set no larger than the forge connector's name bound in temper's small
  profile (`temper-engine-domain-forge`'s `name_bytes`). A value past
  it is not taken: the field keeps its last value and is marked
  invalid, with the bound in words ("At most N bytes.").
- **Checked before sending:** each rule of the table that the draft
  alone decides (all but whether the forge is kept) is the
  form's own check, never one of what the person may do. A field that
  fails is marked invalid, with one status line under it saying the
  rule in words ("Required.",
  "Checks are numbers from 1, each once.", "A context repository
  cannot be the project's home.", "Without CI, name at least one
  check."), and nothing is sent. The engine checks everything again.
- **Adopt** makes one keyed request through `jig-client` (`records.md`), as the
  forge's family's bytes, made once (jig's `client/domain/client/requests.md`
  ). While it is pending the button is disabled and a status
  says its progress as `jig-client` gives it, in jig's words.
- **The forge by name, home by what is read.** The form chooses among the
  deployment's forges by name; with the forge's rows not read, the source
  waits for them. Home is on where the project has none it can read, and the
  engine refuses a second home, saying so.
- **After the outcome,** the outcome stays shown until the person edits
  a field or leaves the page. Adopted, the fields go back to their
  defaults (written by the domain, so the view writes them).
- **Out of date:** when the engine does not serve this client's version
  of the forge's records (jig's `protocol/client/README.md`, section 6), the
  page says "This temper is out of date for adopting repositories: run
  the temper the engine runs.", the button is disabled, the picker
  sends nothing, and a pending adoption stays in the small store.

## 5. Its address and state

- **Address:** `Adopt { project }`, one of temper's addresses (`../pages/README.md`).
- **State:** each field's text and the domain's count of its own
  writes; the pending request's key and progress, at most one; the last
  outcome; the version range the engine serves for the forge's records;
  the source; the forge and the installation chosen; the search; the rows
  read, the last page's number and whether more follow, or its refusal; its
  pending `Reachable` key, at most one; the picks, each with its ids; the
  empty source's org and name; the addresses as typed. Bounded by its limits:
  the field bytes, the checks, `picker.rows` rows kept, a constant of 2000
  in `temper-client-domain`, and as many picks.
- **Records:** none of its own; its pending adoption is `jig-client`'s
  kept request (`records.md`).

## 6. The outcome in words

| Outcome | The form says |
|---|---|
| adopted | "Adopted OWNER/NAME on forge F as ROLE." then "It is the project's home." if so; "temper may: KINDS." in the forge's words; "temper's forge user can only read OWNER/NAME on forge F: work there fails until it has write." if it may not push and the role isn't context; on GitHub, "Commit statuses stay temper's own: GitHub's rules cannot require them."; "temper's forge user may not read the branch protection: the forge shows it to repository administrators only." if unknown; "It merges by STYLE."; then each rule as `repositories.md`, 1 words it, on Forgejo its branch protection alone |
| refused by the core | jig's words for its refusal |
| no such forge | "No forge F is configured." |
| not connected | "Forge F is not connected.", linked to the forges' page (`../pages/README.md`) |
| not installed | "temper's App is not installed on OWNER.", with "Install on another account" (`repositories.md`, 4) |
| not granted | "OWNER/NAME is not granted to temper's App.", linked to the installation's page on GitHub (`repositories.md`, 4) |
| not found | "No repository OWNER/NAME on forge F that temper's forge user can see." |
| no access | "temper's forge user may not read OWNER/NAME on forge F."; `temper ctl` adds "Forgejo collaborator reads can require repository admin even when temper's forge user already has write. A repository administrator must grant temper's forge user admin; then run repository adopt again with a new key, preserving its options, and verify the human's required project role with project show. If setup could not grant access on a managed or owned Forgejo, its site admin's token needs write:repository. Check the setup failure in the server's log." |
| refused | "The forge connector refused it: another project writes OWNER/NAME, the project has a home already, or branches under PREFIX are not this deployment's." |
| in flight | "OWNER/NAME is being adopted already; try again when that ends." |
| too large | "A field is past the forge's bounds, or the forge's repositories are full." |
| unreachable | "Forge F did not answer; nothing was adopted. Try again." |
| invalid answer | "Forge F answered what temper did not expect; nothing was adopted." |
| unknown past the retention | "It is not known whether OWNER/NAME was adopted." |

KINDS lists what temper may do, in this order and these words: read;
push; open pull requests; land; review; set statuses; comment; create
issues; create and delete branches. STYLE is "merge", "squash" or
"rebase". The forge connector maps the forge client's errors to these
reasons (`../engine.md`). The links are the addresses the forge's record keeps,
shown as external links and never opened by the client (`../line.md`, 5).

## 7. The playground

The playground is ordinary records, made when the person chooses it:
a fake forge, the scripted model's account, the deployment's, and the
playground's project, with `playground/widget` adopted into it; its
reset removes them, and the person is on the welcome's Forge step as if
they had never tried it (`../../protocol/setup/playground.md`). The forge's side
reads it, starts it and resets it; the engine makes and removes it. The
client removes no account and closes no chat of its own.

- **Read once a link is live.** The forge's side reads `playground`
  (`records.md`) when the link first becomes live, again each time it becomes
  live after it was not, again once a start or a reset it sent is
  answered, and again while its answer reads starting or resetting,
  until it reads running or none; and again whenever the `forges` watch
  (`records.md`) gains or loses the playground's fake row, a row of mode fake, so
  that a start or a reset another client made reaches it (`../tui/box.md`). It
  keeps the last answer, and drops
  the answer of a read it has replaced. None, and no answer yet, both
  mean no playground is shown. *The playground is read* while its last
  answer names one, starting, running or resetting; *it runs* while
  that answer reads running.
- **Started when chosen** (`../../protocol/setup/playground.md`). "Try the
  playground", the welcome's, sends `StartPlayground` while the
  playground is offered (`../pages/welcome.md`, 1) and none is read, one start
  at a time, held by the forge's side across pages, and opens
  `NewChat`. While the start is out, or the read answers starting,
  the playground's start on `NewChat` says it is under way, its
  suggestions waiting; once it failed, why, with "Try again", which
  sends a start under a new key (`../pages/welcome.md`, 2). Answered running, or unknown,
  the side reads the playground again. It sends no start of its own
  beyond the person's: a start a restart cut short is the engine's to
  mend, with no client (`../../protocol/setup/playground.md`).
- **Reset by "set up temper".** The banner's "set up temper" (`../pages/welcome.md`, 2)
  sends `EndPlayground` while a playground is read and no reset is
  under way. From its sending, the side holds the playground as
  leaving, and the banner says "Leaving the playground…"; once the read
  answers resetting, `NewChat` shows no playground's start.
  Answered reset, or its read answering
  none while it is leaving, the side drops the playground and opens the
  welcome's Forge step, `Welcome { Forge }` (`/welcome?at=forge`): the
  store as a person who pressed "Set up temper" from the welcome finds
  it (`../pages/welcome.md`, 1). Answered not offered, there being no
  playground, it holds it as leaving no more, and reads again. A read
  answering resetting, a reset another client sent or one a restart
  cut that the engine resumes (`../../protocol/setup/playground.md`), holds it
  as leaving the same way, until the read answers none.
- **The scripted model, the playground's alone.** Only the playground's
  project allows it, and no other project does (`../../protocol/setup/playground.md`
  ; jig's `domain/accounts.md`, 4.2). The client names its LLM
  provider from the read and leaves its account out of the accounts'
  lists and counts and its provider out of the ways to connect a model
  (`../pages/README.md`, `../pages/accounts.md`, 2). A model is usable for the welcome, Ready and the
  composer's line by an account of another provider: the person's own,
  from the `accounts` document, or, for a person whose deployment role
  is owner, the deployment's, from the `deployment` document (jig's
  `client/domain/contract/README.md`, 5.1), an account held by the deployment,
  uncut, with a usable source; that document counts only by such an
  account, and while it is not delivered, or is cut, it adds none. An
  account of the person's own added while the playground runs, or
  before a start over, stays theirs: it is not the playground's, and
  its reset leaves it.
- **One `deployment` watch,** for a person whose deployment role is
  owner alone, while `me` says the role and a playground is read; once
  either ends, the side drops it. For any other person it opens none.
- **What shows:** while the playground runs, or is leaving, the
  banner's one line above the main of `Chats` and `NewChat`, and while it
  is leaving on every other page as well (`../view.md`, 1, `../pages/welcome.md`, 2); the playground's start,
  on `NewChat`, while a playground is read that is not resetting
  and the person is in its project, or in none while its project is
  not made yet (`../pages/README.md`); the context line, on the playground's start and on a
  chat's page in its project, while it runs; on the Forges page its
  forge, and on the accounts page its account, each in a section of
  its own, neither page offering anything to add (`../pages/README.md`; `deployment.md`
  ). Each is worded in 5.6. Nothing hides them but the reset.
- **State:** the last answer, the pending read's key, at most one, the
  start's key while it is out, at most one, and its failure, the
  reset's key while it is out, at most one, whether it is leaving,
  whether the link was live at the last pass, and the `deployment`
  watch's key and the playground's provider while it is open, at most
  one. Bounded by the record's bounds (`records.md`).

## 8. Open questions


- **A repository being made, on a page** (4; `../../draft/repository-copies.md`,
  sections 7, 9 and 10): the forges' rows carry `makings`, `temper ctl
  forge list` says them, and Home's repository step says a project's while To
  begin shows (`../pages/home.md`, 1), and Settings' Repositories says each of the
  project's, with a failed one's "Copy again" and "Dismiss" (`../pages/settings.md`, 1).
  Default: with To begin skipped, on any other page, no line says a row.
- **A repository's role, health and landing rules on the Repositories
  tab** (`../pages/README.md`; `repositories.md`, 2 and 3). Default: not drawn, as the user's
  request of 2026-11-05 settled: a line says its name and its forge.
- **The form's place:** among a project's settings (`repositories.md`, 3),
  which `Settings` (`../pages/README.md`) links and does not hold. Default: a page of its
  own, from `Settings` and Home.
- **The forge's `Refused`, split** into its causes (another project
  writes the repository; the project has a home; another deployment's
  branches under the prefix), so the form names the one. Default: the
  sentence of 6 naming the three.
- **The picker for a person who may adopt but not connect:** the
  `Reachable` request's who-may is the deployment's connectors', so a
  project's maintainer without that role sees no list. Default: refused
  for the role (4), and the person adopts by owner and name, which
  the engine judges by the project's role as any adoption; a read the
  project's role may make is the candidate.
- **The picker's page as a read:** a page could be a read of the forge's
  family rather than a keyed request. Default: a request, since its
  answer is a fresh read of the forge by the setup's host's plan; keyed
  as any request, and its answer to a
  form that has closed dropped.
- **What the picker's ids do at adoption:** `Adopt` carries the row's
  repository id and owner id (`records.md`), while the forge connector locates a
  repository by owner and name (`../../domain/forge/adoption.md`).
  Default: the connector locates by owner and name as any adoption, and
  refuses as not found one whose id is not the carried one, so a
  repository renamed or replaced between the page and the adoption is
  never adopted in its stead; the domain's page is asked to say so.
- **The project's own repositories in the picker:** `Reachable` names
  no project, so the engine cannot leave out what the project holds,
  as `../../domain/forge/README.md`, what the picker offers, says the list does,
  and the client holds no list of a project's repositories
  to leave them out itself (3). Default: every reachable repository
  is listed, and adopting one the project holds is refused by the
  engine as any adoption; the request naming the project, the engine
  leaving those out, is the candidate (`README.md`, section 8).
- **A connect's end told:** the forges' page reads how a connect ended
  on the deployment's part. Default: that read, watched while the page
  is open; no notice in the inbox.
- **The pull request's link on the card** ("OWNER/NAME#N on
  Playground forge"): the forge's part, which the forge's
  side does not read yet (2). Default: the link to the task's page.
- **Whether `owned` asks the box** (`../../draft/serve-without-box.status.md`):
  the draft offers Forgejo on this machine from the forges' rows, and a
  first run's store keeps none. Default: the deployment's read says
  `owned` (`records.md`), the document's member and the box both, so that
  `temper-box serve` on a box's document draws no card its `SetUp` would
  refuse. The user's to decide: keep it; have `owned` read the document
  alone, the card then refused "not under the box"; or have the engine
  list an offered owned row from the machine at a first run, which
  changes what `forges` answers and what a first run's store reads as.
