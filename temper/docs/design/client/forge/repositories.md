# Repositories, and the forge itself

## 1. Adopting one

The adoption form is `family.md`, 4, on temper's `Adopt` page,
and its outcome 4.7. With the forge's documents served, the form chooses
among the deployment's forges by name, says whether the project has a
home, and, before the adoption is confirmed, shows what temper found
there (`../../domain/forge/adoption.md`): temper's permission, the
merge styles the repository allows and so whether it may merge, the
protection on the landing branch (or that it could not read it), its
webhooks or none, CI or none, the people it knows there with the role
each would be given, and the ceiling that follows, in words.

What the adoption names there: the repository's role (owned, where
temper pushes and lands; a fork of an upstream; or context, read only),
its landing branch, temper's branch prefix (`temper/` by default),
whether it is the project's home, and whether it runs CI or the checks
its changes are gated on without it. For GitHub with Actions on, no
workflow and no required checks, before confirmation it says "No CI
found: Actions is on, but this repository has no workflow and no required
checks." It offers **Land without CI**, which adopts with CI off and
without replacement checks, or **Name the checks that stand for CI**,
which gates changes on those checks. The choice does not waive required
checks or other landing rules (`../../domain/forge/change/gates.md`, 2). The merge style is the forge's for
the repository, which the form shows before the adoption is confirmed:
on GitHub the first the repository allows of merge, squash and rebase,
on Forgejo its default (`../../domain/forge/README.md`, what adoption says of the
branch's rules). A repository where another
deployment's branches already live under the prefix is refused, saying
so (jig's `domain/connectors.md`, section 12).

**The picker.** Below the forge's choice, for the chosen forge, the form
lists the repositories temper can reach there (the `Reachable` request,
`../engine.md`), read a page at a time by number from 1, each
page at most `picker.page` rows (`../../protocol/configuration/README.md`, its
limits) clipped to the forge's own page cap; the answer says more follow
when the page is full. The list has a height of its own, so the rest of
the form stays where it is however many there are; a phone keeps it
shorter.

- **Search and scroll, in place of "Next" (the user, 2026-11-06).** The
  client reads the next page when the list scrolls near its end, and while
  a search has found less than it has left to read. So a search asks
  nothing of the forge, which GitHub's `GET /installation/repositories`
  could not answer, and nothing is read until it is wanted. A forge with
  more than 8 rows read, or more still to read, has the field "Find by
  name" over a list of its own height; a completed list of at most 8 has
  neither, and no count.
- **No total until the last page:** an answer says only whether more
  follow, so the count reads "100 so far · more as you scroll", or
  "Searching · 100 read so far", then "214 repositories", or "3 of 214
  repositories" under a search; each with " · N picked" while some are.
- **The rows keep the forge's order,** since recent first would need every
  page first. At most `picker.rows` are kept (`family.md`, 5); past it
  the count says so, and "Not listed" takes the rest by name.
- **Plural:** rows have checkboxes, what is picked stays through a search,
  a new forge or account picks nothing, and the button counts them:
  "Adopt OWNER/NAME", "Adopt N repositories". "Make it the project's home"
  goes with more than one; it is off where the project has a home and on
  where it has none.
- **Marks:** "private" where it is; "In PROJECT" on the project's own,
  disabled, which the request names no project to leave out; "Copying…" on
  a repository being made, disabled, so a person sees one list (4.4). The
  repository's id and its owner's id the row carries ride in the adoption
  request and are never shown.
- **A search that finds nothing,** with all read, says "Nothing here matches
  “QUERY”." in the list, has no count, and opens "Not listed" on what was
  typed, with the remedy below. The button waits for the remedy, unless the
  role is Context.
- **One forge, one account:** its field goes too.

What temper can reach:

| The forge | Listed |
|---|---|
| GitHub | one accepted installation's repositories of temper's App (GitHub's `GET /installation/repositories`, on a token the setup's host mints for the page and drops), the installation shown by its account's login and asked for by its id, which the picker's request carries, the installations as the deployment's page reads them (`deployment.md`), marked "selected repositories" where its installation has chosen repositories and "suspended" where GitHub suspended it; read by the setup's host's `Reachable { forge, installation, page }` plan (skein's `forge/setup/forgejo.md`, 1.8) |
| Forgejo, connected | the repositories its API token can push to, read by the same plan, with no installation |
| Forgejo, managed or owned | every repository the site admin can see, read by the setup's host's `Reachable { forge, installation?, page }` plan on the admin's token (skein's `forge/setup/forgejo.md`, 1.8), by id ascending, its `reachable` rows given from `picker.page`, since adopting one grants the bot its access (`../../protocol/forge/README.md`, the setup's host's `Reachable` and `Adopt`); the admin's token stays the setup's host's, and the picker gets the rows alone. The plan runs beside the forge's one writing run; a second read of the picker while one runs is refused busy, and so is one past `setup.runs`, which counts both. Its own refusals are the admin's token's, as the reconcile's first step has them (`skein/docs/design/forge/setup/forgejo.md`, 1.3): "the admin's token is missing" and "NAME is not a site admin" |

A forge not connected offers none: the picker says "Forge F is not
connected." and links to the deployment's page (`deployment.md`), and the "Adopt"
button stays disabled for it. A suspended installation's repositories are
listed and marked; adopting one is refused as no access until GitHub
unsuspends it.

**Not listed.** Below the list, "Not listed" takes a repository's owner
and name, and the form says what would make it reachable, by the forge
and what its read answered:

| Where | The form says |
|---|---|
| GitHub, an account with an installation of temper's App | "Check OWNER/NAME exists, and grant it to temper's App on GitHub.", linked to `https://github.com/apps/SLUG/installations/new` where the client holds the App's slug, with no link otherwise; the page asks for the account |
| GitHub, an account with no installation of temper's App | "temper's App is not installed on OWNER.", with "Install on another account" (`deployment.md`) |
| Forgejo, connected | "Make LOGIN a collaborator with write on OWNER/NAME.", LOGIN the bot's |
| Forgejo, managed or owned | nothing: adopting it grants the bot its access |
| any forge | a repository its credential can read may be adopted as context |

Once the forge says so (an installation's repositories changed, a
collaborator added), the repository is listed on the picker's next
page, with no restart.

**The rules, in words** (`../../domain/forge/README.md`, on what adoption reads):
the landing branch's rules as the forge applies them, each with what it
means for landing: a merge queue (temper enters it, `README.md`, 2.1), required
approvals (a person approves, or temper's App may bypass), required
status checks (CI as the landing rule's), code owners (a person
reviews), signed commits (temper's commits are unsigned, so where the
forge refuses its merge for it, its landings wait), linear history (a
merge style that keeps it), and whether
temper's App may bypass the rules. Where a rule keeps temper from
landing, the form names the one setting on the forge that would let it
land, for the person to choose; until it is changed, its ready changes
wait for someone else's merge (`README.md`, section 3). temper never changes a
repository's rules. On GitHub temper writes no commit status
(`../../domain/forge/adoption.md`, 4), so every GitHub adoption says "Commit
statuses stay temper's own: GitHub's rules cannot require them."
Forgejo's reads answer none of the merge queue,
signed commits, linear history and code owners, and no bypass, so on
Forgejo the form says its branch protection alone.

## 2. Health

Wherever a repository is shown (the project's page, its settings, the
deployment's page):

- **what temper may do** there, and whether that narrowed since adoption
  ("a merge was refused: temper may no longer land into main");
- **its webhooks:** when they were last heard, on GitHub the App's
  deliveries as its delivery log records them (`../../protocol/forge/README.md`,
  a GitHub App's delivery log); none, or none for days, means changes
  are seen by polling alone, more slowly;
- **its landing branches:** their tips and CI on them;
- **on GitHub, its installation:** the account's, marked suspended
  while GitHub suspends it; a repository that left the installation is
  marked "not granted to temper's App", linked to the installation's
  page (4);
- **what is held** on it for drift (`README.md`, 5.2).

## 3. In a project's settings

- **Resources:** each repository of the project, as the forge's part in
  the settings' resources (jig's `client/domain/pages.md`, 6.5): its
  role, whether it is the project's home, its landing branch, temper's
  branch prefix there, its merge style, CI or the checks that stand for
  it, what temper may do there in `family.md`, 6's words, and its
  health (2). Adopting one is the form of 1; where the form lives is
  `../README.md`, section 5's.
- **The landing rules,** in the policy's requirements (jig's
  `client/domain/pages.md`, 6.5), per pattern of landing branches, the
  deployment's and the project's, in words: "into main in acme/shop: CI
  passed at the head; up to date with main; a review for memory safety,
  at the head or a clean update of it; two maintainers' approvals on the
  forge" (`../../domain/forge/change/gates.md`, 2).

## 4. How the forge's refusals show

What the forge's protocol layer answers a call, or tells the root
(`../../protocol/forge/README.md`, a forge stopped, GitHub's installations,
what the vocabulary adds and the setup's host's outcomes), wherever a
part shows it: the deployment's
page, the adoption form and its outcome, a change's panel and its hold.

| The forge answers or tells | It shows |
|---|---|
| `NotConnected` | on the forge's row, "Not connected", with Connect (`deployment.md`); the adoption form lists none of its repositories, and an adoption on it is refused, "Forge F is not connected." |
| `NotInstalled`, no accepted installation on the owner | "temper's App is not installed on OWNER.", with "Install on another account"; a push to it is refused, as "temper's App is not installed on the account owning NAME." (`../../protocol/channel.md`, section 6) |
| `Missing`, on a repository whose account has an installation of selected repositories | "OWNER/NAME is not granted to temper's App.", linked to the installation's page on GitHub: `WEB/settings/installations/ID` for a person's account, `WEB/organizations/ORG/settings/installations/ID` for an organization's, WEB the forge's `web` address and ID the installation's |
| `Missing` otherwise | not found, as `family.md`, 6 says |
| `Stopped { why }`, told to the root | on the forge's row, "Stopped: WHY", with Replace (`deployment.md`); its calls answer `NotConnected` until its next connect, shown as that row says |
| an installation suspended | "suspended" on the installation, on the forge's row and on each repository of it (2); its calls are refused as no access |
| a stray installation | on the forge's row, "not accepted · kept" (`deployment.md`) |
| `Lacking`, a GitHub App's `Paste` (a connect of an App of the person's own, or a Replace) whose App lacks permissions or events temper needs | on the forge's row, connected, "temper's App lacks WHAT: grant them in the App's settings, then paste its key again.", WHAT each permission, with its level, and each event, as GitHub names them, until the forge's next check of the App ends (`deployment.md`); meanwhile a call that needs what the App lacks is refused by GitHub, as no access; with Replace where the forge is connected, Connect where it is not, and a link to the App's settings, `WEB/settings/apps/SLUG`, or `WEB/organizations/ORG/settings/apps/SLUG` for an organization's App |
| `Unmarked`, connecting as the admin a forge whose user lacks temper's mark | "LOGIN is not temper's bot: add `temper:bot` to its full name on Forgejo, then connect again." |

The installation's page and the App's settings are built by the engine
from the `web` address the forge's record keeps, never from an
address a forge's answer gave, so a link the client shows points at
that forge alone. SLUG is the App's slug, as the forge's row shows it (`deployment.md`).

## 5. Open questions

- **The picker's pages kept.** Default: none; each page is a fresh read
  of the forge through the setup host's `Reachable` plan, asked by
  number, so a
  repository added or removed between two pages can move a row across
  them, seen twice or missed until the list is read again from its
  first page. A scroll or search beyond a full last page gets an empty
  answer and ends the reading.

- **A repository's role, health and landing rules on the Repositories
  tab** (`../pages/README.md`; `repositories.md`, 2 and 3). Default: not drawn, as the user's
  request of 2026-11-05 settled: a line says its name and its forge.
