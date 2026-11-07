# A box's person on its forges

Draft, 2026-11-06, from the user that day: "If the box
user chooses to block PRs landing on his approval, it should be possible
without any other setup." And of GitHub: "You add temper as github app (or
something like that), so we should be able to integrate well just based on
that." So an owned Forgejo and GitHub, both in this round. The user chose this
over waiting until after v0.1.0, knowing that `v0.1.0.md` names no approvals.

This draft decides who counts as an approver, so its reasons are kept here
(section 6), as `../protocol/sign-in.md`, 9.9 keeps its own.

## 1. In one page

- **The walk:** the person tells the configuring agent nothing lands without
  their approval; accepting that isn't refused; a change waits and shows under
  Needs you with its link; the person approves on the forge, one click away; it
  lands. No `temper ctl`, no collaborator added, no adopting again (section 2).
- **Why it fails today:** a box's person signs in by link, on provider 1, and
  temper reads a person's forge account only from a key on provider 0, which
  adoption makes. So the owner is "forge identity unknown", and the root
  refuses the rule.
- **The fix is one stored fact and its evidence:** on this forge, the box's person is
  that user (section 3). temper knows it without asking on an owned Forgejo, since it made
  the account, and on GitHub, since the App's owner is the account that made it
  (section 4).
- **What it holds against:** on an owned Forgejo, an agent's mistake, not an agent
  trying to get past it (section 6). Said at acceptance, and the user's to weigh.
- **A hole it closes:** a lookup by a forge user's number alone can take a
  stranger's review for a known person's on another forge (section 3).
- **A box's one person only** in this round (section 9).

## 2. What the person does

1. **Says it.** "Nothing lands without my approval." The configuring agent's
   inventory now shows the person's forge account on each forge, so it proposes
   a rule whose approval names their role, 0 on a project they made, and it no
   longer raises forge identity unasked.
2. **Accepts it.** The root counts them as eligible (section 7), and the proposal
   says what the rule holds against (section 6).
3. **Sees a change wait.** Once everything else is ready, Needs you shows the
   change, with "Review on Forgejo ↗" or "Review on GitHub ↗" (section 8). Changes
   behind it on that branch wait their turn, as any wait on a merge holds the
   branch (`../domain/forge.md`, section 9), and the card says how many.
4. **Approves on the forge.** On Forgejo, the link signs them in as a
   repository's link does (`../client/forge.md`, 5.4). On GitHub, they're in
   their own browser.
5. **It lands.** A review already wakes its pull's change, by a hook on a box
   and through the delivery log on GitHub (`../domain/forge.md`, 5.2;
   `github.md`, section 7), so this step needs nothing new.

## 3. Who a forge account is

- **Three readers need the same lookup:**
  `human_forge_identity` (`temper-engine-domain/src/route/forge_route.rs`),
  `settings_people` (`route/host_route.rs`) and `landing_reviewers` (`forge_route.rs`).
- **The hole identified at `d893a93cc1`:** provider 0's subject did not name a forge,
  and `landing_reviewers` looked up `{ provider: 0, subject: review.author }`
  whichever forge the review was on. In a project on both an owned Forgejo and
  GitHub, GitHub's user 3 could be taken for the party adoption seeded from
  Forgejo's user 3, role and all. Anyone may post a review on a public repository's
  pull request, so a stranger's approval could count. This was read in the code
  and confirmed by this draft's review without running it. Both forges in this
  round require the lookup to name the forge as well as the user.
- **Provider 0's subject names the forge:** the forge's number then the user's, 10
  bytes, opaque to everything but one function of the forge's. A stored format
  changing in place: no migration before v0.1.0.
- **The link is a field of the forge's record:** `person: Option<(user, person)>`. One
  person on a box means at most one link per forge, so it needs no family, no bound
  and no restore of its own, and it goes with the forge if that's removed. A family
  can come when `temper serve`'s members do. jig's people stay as they are: a party
  keeps one key.
- **One lookup, by forge and user:** the link's person if it names that user, else the
  party whose key is provider 0's subject for that pair, else unknown. All three
  readers use it, the inventory line among them, which now names each forge.
- **A role is read as ever,** from the person's own key in the project. So a linked
  person's role is their own, with no seeding.

## 4. Where the link comes from

Each is made on a box alone (section 9), for the deployment's one owner. Each start
reads it again.

### 4.1 An owned Forgejo

`Reconcile { forge, adopted }` already runs before the first repository, with the
set-up's admin token (`../protocol/setup.md`, 5.2). It's also given the owned forge's
person's login, reads `GET /users/<login>`, and answers its `id`; a fake forge is given
none. The connector keeps the link. The same id again changes nothing. A different id
replaces it, as a person may remake their account; an account gone leaves it, and
that's logged. This changes skein's plan and its `Done` (skein's `forge-setup.md`, 4.1
and 4.3), with no system named there, so a plan of its own goes first.

### 4.2 GitHub

`GET /app`, on the App's JWT, runs in `Convert` and in `Reconcile` (skein's
`forge-setup.md`, 5.2 and 5.6), and each checks the App's id. GitHub's documentation
lists `owner` as required on it, with `id`, `login` and `type`; skein doesn't decode it
yet, and the live conformance adds it, as this hasn't been measured on `GET /app`
itself. The by-slug read `GET /apps/<slug>` answered the same for the probe App on
2026-11-06, but it's id-checked by nothing and an App can be renamed, so temper doesn't
use it.

- **Made by the manifest flow alone,** whose `target` is always `Personal`
  (`../protocol/setup.md`, 17.2). Manifest and pasted forges end in the same records,
  so `Convert` first writes a new secret item, `AppMade`, whose bytes are the App's id,
  as `Copying` is written first (skein's `forge-setup.md`, 6.4). It's kept in jig's
  secrets' store with `AppId`, as no write spans that store and the forge's record.
- **Each `GET /app` sets the link from there:** when `AppMade` equals `AppId` and
  `owner.type` is `User`, the link is `owner.id`. So a restart before the link was kept
  loses nothing, a pasted App replacing a manifest's has no link, and an App transferred
  away and back links again. Anything but `User` ends it. A read that fails changes
  nothing, and that's logged.

### 4.3 When it stays unknown

An App that's pasted (`github.md`, 6.5) has no such link, even with a person as its
owner: having its key doesn't show that the owner and the box's person are one, and a
wrong link would count someone else as them. So nothing changes there: what adoption
seeds counts, the refusal says so plainly (section 7), and the panel names a review
from an account temper doesn't know (`../domain/forge.md`, 8.7).

### 4.4 One person, two records

Adoption can seed a person from a collaborator, and on a repository in the person's
own account it does.

- **A linked collaborator isn't seeded:** adoption skips the link's user. The link comes
  first on both forges, as 4.1 and 4.2 run before any adoption.
- **A party seeded before the link** stays, role and all, but the lookup finds the
  link's person, and eligibility skips a party whose user is linked. It's never counted
  on that forge. Nothing merges.
- **Distinct people are distinct forge users:** one human counts once on a forge, and
  temper's writer never: a link to the forge's writer is refused, and
  `landing_reviewers` still skips the writer before any lookup.
- **A link replaced** leaves its old user a stranger, or a seed if one was made before
  the link. That's a second account of a person who made one on purpose, and it counts
  only if a rule asks for two people: left as it is.

## 5. Whether their review counts

temper counts a review by the reviewer's identity and role, not by collaborator, so
the question is whether the forge lets them post one. It can matter: on a box the
repository may be in another organization, where adoption returns only temper.

Measured on a throwaway Forgejo 17.0.1, on 2026-11-06, on a private repository in an
organization `ada` isn't in, with `temper` a write collaborator and the pull request's
author:

| Reading | Answer |
|---|---|
| `collaborators/ada` | 404, and the list holds only `temper` |
| `collaborators/ada/permission` | `owner`, as a site admin |
| `ada`'s `POST pulls/1/reviews`, `APPROVED` | accepted, `official: true` |
| the reviews read back, as `temper` | `APPROVED`, `ada`, her user's id |
| the files page, signed in as `ada` | 200, with Finish review, Approve and Request changes |
| `temper`'s own `APPROVED` | 422, "approve your own pull is not allowed" |

So it counts, with no collaborator added. That measures the API and that the page
draws, not the page's own post. On GitHub the author is the App's bot, whom the person
isn't (`github.md`, section 5), and P8 has the App's own approval refused.

## 6. What it holds against

This is an authority change: who counts as a person's approval. What could count
wrongly, and what stops it:

| Who | What stops it, or doesn't |
|---|---|
| temper's writer, on either forge | refused as a link; skipped before the lookup (4.4); on a box with sign-in, a person so named refuses the start (`../protocol/sign-in.md`, 9.9) |
| **an agent's run or a CI job, on an owned Forgejo** | **nothing, if it tries.** Both run as the box's user, who can read `person_write.header` (`../protocol/setup.md`, section 15; `../protocol/credentials.md`, section 8), and a review posted with it is the person's. Deleting that file wouldn't close it: the forge's work path and temper's store are as readable. Closing it needs agents and CI contained |
| the same, on GitHub | temper holds no token of the person's, and the App can't approve its own pull or push a workflow (`github.md`, P3 and P8). What's left is whatever the person's own tools leave under that user, which isn't looked at here |
| the person's own agent, handed that file by them | it approves as them: their delegation, and how a walk approves on an owned Forgejo |
| a stranger on GitHub | no link and no seed, once section 3 closes the hole |
| another account on the box's Forgejo | the person made it, as its site admin; unknown unless adopted as a collaborator, as today |
| the account owning the App, if it isn't the person's | it already holds more than an approval: it can make the App's key, and land as temper |
| a program on the machine, through the browser's cookies | the box's loopback trust (9.9), which the user is weighing |

So on an owned Forgejo the rule stops an agent's mistake and makes a person look at what
lands. It doesn't stop an agent trying to get past it. The proposal says so when it
holds such a rule, and the help does: "On this machine temper's agents and CI jobs run
as you, so this stops their mistakes, not one trying to get past it." The client writes
that sentence, on the proposal's card as on its page (`../client/README.md`, 5.13). The
user decides whether that's enough for now (`forge-identity.status.md`, *Open*).

The link on an owned Forgejo rests on what 9.9 already rests on: temper made every
account, and the printed link alone signs anyone in, always the owner.

## 7. Acceptance

- **Per forge:** for each repository the rule covers, the root counts people with the
  role and an identity on that repository's forge. A person known only on Forgejo
  doesn't make a rule on a GitHub repository satisfiable, which today's forge-blind
  count would allow.
- **The refusal names the forge,** beside the project, role, required and eligible
  counts, and keeps its remedy where it applies.
- **On a box,** for a forge in 4.1 or 4.2, `people: 1` isn't refused for lack of
  a person. `people: 2` is refused unless a seeded collaborator makes up the count.
- **The playground's fake forge** has no account and no address, so a rule covering its
  repository is refused in its own words: it can't be approved there, so name a branch
  pattern or a project that leaves it out.
- **A rule that can no longer be met,** as a repository adopted later from a forge with no
  link or an App moved to an organization, waits as "later membership changes" do
  (`../domain/forge.md`, 8.9), and the card says why (section 8).

## 8. Under Needs you

Today nothing shows there: an approval is waited for at the merge alone, and isn't a
hold (`../domain/forge.md`, 8.7), and Needs you has no such kind (`../client/README.md`,
3.5).

- **A read, not a task:** one subject the root makes, as `TRIALS_SUBJECT` is, with
  `ApplicationScope::Own`, since roles and eligibility live in the root and the forge
  child has no people. It answers each watcher's rows, filtered by their role, as that
  scope skips jig's project check; a test holds this, and the rows have a bound. A subject
  per party would make the root read the sign-in on every watch (`../domain/forge.md`,
  5.5). It stores nothing. It's marked wherever `changes` is.
- **A row** is a change in state `First` or `Landing`, in a turn project or not, whose
  review facts are at its last head, read whole, with an approval still short and none
  refused, in a project where the watcher holds the rule's role or is an owner. Facts are
  dropped when the merge is admitted, so the row goes when the review counts, not when
  it lands. Facts read in part show the card with "too many reviews to read them all". A
  change that left the queue or was held has no row, and after a restart rows fill as
  changes step again.
- **Where it shows:** the client merges these rows with jig's inbox on Needs you, its
  filtered addresses, Home's region and a goal's band, and the rail and banner add
  them to `needs_total`. Its key is `ObjectKey::Task(change)`, so Later works as on any
  card. The terminal's inbox stays as it is: section 6 of `v0.1.0.md` leaves it out.
- **The card:** kind "an approval"; the question "Approve CHANGE on Forgejo" or "on
  GitHub"; as its context, the rule and "N more changes wait behind this one" where there
  are any, counted from the forge's order of changes; the primary "Review on Forgejo ↗", a
  new tab; "Open" to the change; "Later". A person who won't approve asks for changes on
  the forge, which holds the change as a requirement refused does (`../client/forge.md`,
  6.1).
- **Its address:** the pull's files page, where the review box is: on Forgejo with sign-in
  offered, `ROOT/user/oauth2/temper?redirect_to=` `/OWNER/NAME/pulls/N/files`, encoded as
  `../client/forge.md`, 5.4 says, and without it `ROOT/OWNER/NAME/pulls/N/files`; on GitHub,
  `WEB/OWNER/NAME/pull/N/files`.
- **When this viewer's review couldn't count:** no primary, and the context says why: no
  account of theirs known on this forge, or the rule asks for more people than can give
  it. So an owner still sees a wait nobody can end, and is never sent to a review that
  comes back unknown.

**Why not a person task,** as a client approval's: it would reach jig's inbox everywhere
with no merging, but the stock ceiling has no role as an executor, so it asks for an
authority widened down the tree, to agents too, for a card; a change made before the rule
couldn't open one; and it needs a stored number, a delegate slot and a task per head.
**Why not a hold:** it would free the branch, but it escalates to the goal above first, so
the person may not see it, wakes an agent to no purpose, and offers a Release that does
nothing here. A read costs a wire document and a client merge, and it leaves the branch
held, which is what "nothing lands without my approval" asks of the change first in line.
The look comes through a look request, if the user wants one: the card draws as every
other does.

## 9. Scope

- **A box's one person:** serve running under the box, the `--box` the host tells the
  connector at construction (`../protocol/setup.md`, 7.1); one owner; and no
  `application.sign_in`, so no member can sign in and land on the person's account by name
  (`../protocol/sign-in.md`, 9.9). The start refuses `machine.owned.sign_in` beside
  `application.sign_in`, which nothing refuses today. So plain `temper serve` makes no link,
  though its document may look alike.
- **When that stops being true:** at each start, when core and forge are both loaded
  (`../domain/root.md`, 9.1), the root hands the connector the sole owner's party, or none;
  "sole" means the deployment's roster holds one owner, and it's the link's person. If not,
  the connector clears the link in that decision and logs it. Lookups fall back to seeds,
  and a rule resting on the link waits, saying so. jig's people have no accessor for this
  yet: a small generic one.
- **`temper serve` with several members** says what it says today: the refusal, with a
  human collaborator added and the repository adopted again as the way. The help keeps
  that, marked as being for such a deployment. The inventory says "unknown" too.
- **Managed Forgejo:** as `temper serve`, as temper didn't make its accounts.

## 10. Testing

Stories are written and compiled, by the usual rule; plans run focused and fuzzy only.

- **Focused:** the lookup's three outcomes; a project on two forges with one user's
  number, not one person; provider 0's new subject; eligibility per forge; adoption
  skipping a linked collaborator, a seeded party shadowed; a link to the writer refused;
  `Reconcile` answering the person's id, a fake forge none; `GET /app`'s owner, `User` and
  `Organization`, with and without the manifest's mark; a restart before the link was
  kept, a pasted App replacing a manifest's; the link cleared when section 9 no longer
  holds; the read's rows by state, head and role, a watcher of another project shown
  none; and the card with and without a primary.
- **A world, on the fake forges:** section 2 whole, on each.
- **Written for the slow tiers:** the card and its link in the browser; on GitHub, a real
  review counted, in the live suite.
- **A walk,** when QA likes: an owned Forgejo through `person_write.header`.
- **Changes an existing story:** `../domain/testing.md`'s, which asserts today's refusal
  for a box's owner.

## 11. What changes in temper's documents

`forge-identity.status.md` lists them, by section. Section 3 changes a stored format and
`forge_route.rs`, both shared, so its cut says so first.

## 12. Later

- **`temper serve`'s members:** `../protocol/sign-in.md`, 10's "by construction", once 9.9's
  reasoning is checked for several members, and a manifest's connecting owner. A family of
  links then.
- **A pasted App or a second account:** the App's user authorization, with the
  `client_id` and `client_secret` temper doesn't keep today; or an owner's "that's me"
  on a review the panel names as unknown.
- **Contained agents and CI,** which would make section 6's rule hold against one trying:
  `../protocol/setup.md`, section 15.
- **Approving in the client,** posted as the person: `../README.md`, section 8.
