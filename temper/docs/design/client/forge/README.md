# The forge in temper's client

Provisional, 2026-10-05, revised 2026-11-03. What the forge connector
adds to temper's client: a change's way to landing and its result, a
project's landing queues, approvals at a head, the forge's holds in
words, repositories with their health and landing rules, the picker of
repositories to adopt, goals, findings and landings as the forge shows
them, and the forge's part of the deployment, where a forge is set
up, connected or removed. jig's pages and cards (jig's `client/domain/pages.md`
and jig's `client/domain/objects.md`) leave a slot for each part a
connector shows (jig's `client/domain/root.md`, section 6), and
temper's pages and cards, its own after them, show the same parts
(`../README.md`, section 1); this document says what the forge shows in
those slots and on the pages only temper has, `Changes`, `Adopt` and
`Forges` (`../pages/README.md`), in temper's words. The forge's client
family, its words for resources and kinds, its adoption form and its
requests are `family.md` and `../engine.md`. Each part shows what `../../domain/forge/README.md` keeps or reads,
and links to the forge for code. What is still open is listed in
`README.md`, section 8.

## 1. In one page

- **In jig's slots, in the forge's words.** jig's pages show tasks,
  cards and holds; the forge fills the parts they name (a panel, a
  result, an effect, an outbox entry, a hold, a project's part, the
  deployment's part) on a task's page, in a chat, in the inbox, on a
  project's pages and on the deployment's (`served.md`). A part the
  forge's side does not read says "Shown on the forge." (`family.md`,
  2).
- **The first milestone needs two of them** (`../../README.md`, section
  2): a change's result and its hold, in the forge's words, with a link
  to each on the forge (2.5).
- **A change is shown as its way to landing,** one step at a time, each
  with its reason in words: CI running, a review asked for, second in
  the queue, updating from main, resolving a conflict.
- **Queues are shown, not managed.** A landing branch's queue is its
  ready changes in order, on temper's `Changes` page; a person changes
  the order through goals' priorities on the board, never by moving
  changes about. Where a branch lands through GitHub's merge queue,
  temper's queue decides only what enters it and when, and the panel
  says "in GitHub's merge queue, n" (2.1).
- **Code stays on the forge.** Parts link to a pull request, its diff,
  its files and its failing jobs; the client shows states and verdicts,
  never code, and has no code review of its own.
- **Approvals are at a head,** given in the client as a person task or
  on the forge as a review (section 4): each counts at the head it was
  given at, and the panel says when a later head still carries it.
- **A hold says what happened on the forge,** and what each act would
  do; nothing on the forge is written over until the person decides
  (section 5).
- **What the forge did not tell temper is said:** a repository with no
  webhooks, protection temper could not read, a branch someone else
  moved, a job's log on a host temper does not read (`deployment.md`).
- **Forges are made and connected from the client.** A forge is a
  record the store keeps, made by the set up or the connect that brings
  it (`../../protocol/forge/engine.md`, 1 and 2); no configuration names one.
  On the deployment's `Forges` page (`deployment.md`) and its flow, a person whose
  role holds the deployment's connectors sets up a Forgejo on this
  machine, or connects GitHub through an App made from temper's
  manifest, or one of their own, its key pasted, or a Forgejo of
  theirs with a site admin's token from which temper makes its bot.
  The page lists the forges kept, says what temper is there and what
  stopped it, offers Replace, and removes a forge, confirmed, saying
  what goes and what stays on the forge.
- **Adoption picks from what temper can reach** (`repositories.md`, 1): per connected
  forge, the repositories temper can reach there, a page at a time by
  number, and "Not listed" for one named by owner and name, with what
  would make it reachable. A forge not connected offers none.
- **The forge's refusals are said with their remedy** (`repositories.md`, 4): not
  connected, not installed on an account, not granted to temper's App
  (linked to the installation's page on GitHub), stopped and why.
- **The browser's steps of a connect are temper's routes** on the
  clients' listener (`served.md`, 1): the manifest's page, GitHub's redirect and
  its setup, each authenticated by a one-use state alone. A browser's
  tab goes to a connect's link and comes back to the page that sent
  it; the terminal prints the link and never opens it.

## 2. A change's panel

On a change task's page, as its panel: a procedure's activity is its
connector's panel (jig's `client/domain/pages.md`, 5.2). Folded to its
step in words, it is the change's phase wherever its chip shows (jig's
`client/domain/objects.md`, section 3): a chat's conversation, a goal's
plan, the inbox, the `Changes` page.

### 2.1 Its way to landing

The steps of `../../domain/forge/change/README.md`, 2, with the one it is at and
why:

```
producing ─ opening ─ checking ─ gating ─ queued ─ first ─ landing ─ landed ─ verifying ─ cleared
                 repairing · updating · resolving · recreating · reopening        red
```

| Step | It says |
|---|---|
| producing | T21 producing it, as a chip |
| opening | opening its pull request |
| checking | CI on its head, and for how long |
| gating | which gates are running, as chips, and which requirements wait |
| queued | its place in its landing branch's queue, and who is ahead; handed to GitHub's merge queue, "in GitHub's merge queue, n", n its place there |
| first | first in the queue: updating from its base, CI on the updated head, or merging |
| repairing | why (CI's failure, a gate's remarks, a person's request for changes on the forge, a semantic conflict, taken out of GitHub's merge queue and GitHub's reason), the repair as a chip, and how many repairs are left |
| updating | the base it takes in |
| resolving | the conflicting files, the resolution as a chip, and how many are left |
| recreating, reopening | after drift released (5.2): its branch made again at the head temper last knew, or its pull request reopened or retargeted back |
| landing | merging at its head |
| landed | its merge commit, and when |
| verifying | landed, its merge commit, and the tiers following its landing branch that have not judged it yet, each with its run in flight, if any (`../../domain/after-landing.md`, section 6) |
| red | landed, its merge commit, and each tier whose verdict blames it, with the run and the failing tests blamed on it, counted, or "many" (`../../domain/after-landing.md`, 5.3) |
| cleared | landed and cleared by every tier following its landing branch, and the run that cleared it in each; or, past the project's verification bound with no verdict or cancelled while verifying, "landed, uncleared": the change ended there (`../../domain/after-landing.md`, section 6) |

On the change's task page, the steps its panel carries (`served.md`, 2) and their
words are `../pages/task.md`'s, 3: the steps as the mockup's six, each side
step at the one it serves, and a line for the step it is at, with the
task it runs as a link. What this table names beyond the panel (how
long CI has run, which requirements wait, when it landed) waits for
the panel to carry it (`README.md`, section 8).

Where the landing branch requires GitHub's merge queue
(`../../domain/forge/README.md`, on merging through it), temper's own queue
decides only which change enters GitHub's and when: the first change's
merge hands it to GitHub's queue at the head its decision saw, and
until it lands or leaves, the panel says "in GitHub's merge queue, n",
n the position the forge reads for its pull request (`Pull.queued`) as
each read gives it. GitHub builds the merge group, runs CI on it and lands
it; the landed step names the commit the group landed (`Pull.merged`),
which is not the head temper saw. A pull request GitHub takes out of
its queue for any reason but its merge is repaired, as a failed CI is,
the repairing step naming GitHub's reason. Forgejo has no such queue.

### 2.2 What it shows

- **Its pull request:** number, title, head and base, state,
  mergeability, or that it is not known yet while the forge settles it
  (`../../domain/forge/keeping-up.md`, 2); links to it, its diff and its files on
  the forge.
- **CI on its head:** the combined state and each context's, with a link
  to a failing job, and the failed step and the end of its log a repair
  was given, cut as the repair got it (`../../domain/forge/change/repairs.md`).
  Where the repository or its effective landing rules do not ask for
  CI, the panel says “CI not required”. A landed change retains the
  last observation at its exact head, which may still be pending when
  it landed without CI; that observation does not claim a current
  forge verdict (`../../domain/forge/change/README.md`, 5).
- **Its gates** (`../../domain/forge/change/gates.md`, 1): each with its kind (a
  review by a lens, a person's approval, checks), blocking or advisory,
  who asked for it (the change, the project's landing rule, its plan),
  its verdict and the head it was given at, and whether it carries over
  to the current head; the task running it, as a chip. An advisory gate
  that failed is reported and holds nothing, and the panel says so.
- **Its requirements:** the landing rule's, each met, waiting (and for
  what) or refused at the current head: CI at the head, up to date with
  the branch, its blocking gates, approvals on the forge by so many
  people of a role (section 4; `../../domain/forge/change/gates.md`, 2).
- **Its heads,** in order, each with what made it (produced, a repair, a
  clean update, a resolution) and its CI and verdicts, so the person can
  see why a gate is asked again: a repair's head asks every gate again,
  a clean update's carries them.
- **Its bounds:** repairs, resolutions and updates used of their limits;
  each wait against its stall: "CI has not reported for 25 minutes; held
  at 60".
- **Its delegates:** producing, repairs, reviews, resolutions, as the
  task page's plan (jig's `client/domain/pages.md`, 5.2).

### 2.3 What may be done

- **Amend its gates:** add a review by a lens or a person's approval, or
  drop one the change or its plan asked for; a landing rule's gate
  stays. A change's gates are its parameters, amended
  through the forge's form in the panel, the forge's own request (jig's
  `client/domain/root.md`, section 6). A gate added while it is queued
  takes it out of the queue until its verdict, which the form says.
- **Release** it, held (section 5; jig's `client/domain/objects.md`,
  section 5).
- **Cancel** it (jig's `client/domain/objects.md`, section 11): the
  panel says what the forge will do: its pull request closed with a
  comment saying why, its branch deleted, its commits still readable
  under the closed pull request; a merge already sent may still land,
  and then it ends landed (`../../domain/forge/change/README.md`, 4).

### 2.4 A chat's small change

A chat that makes a small change itself delivers it to its run's own
branch, and delegates a change task handed that branch
(`../../domain/forge/change/README.md`, 1 and `../../domain/forge/branches.md`). In the conversation the
delegation shows as the change's chip (jig's `client/domain/objects.md`,
section 10), its phase the panel's step in words, following it through
CI, its gates and its queue to landing while the chat carries on. The
change's result reaches the chat as its delegate's result (2.5), and the
chat tells the person.

### 2.5 Its result, and why it stopped

What the first milestone needs of the forge's parts (`../../README.md`,
section 2, steps 6 and 7):

- **Its result,** a connector's result (jig's
  `client/domain/objects.md`, section 8), given at cleared
  (`../../domain/after-landing.md`, section 6): landed, with its pull
  request's number and title, its merge commit (through GitHub's merge
  queue, the commit its group landed), and the address of each on the
  forge, and cleared, or uncleared past the verification bound; or ended otherwise, with what became of its pull request
  and its branch. It shows on the change's page, in the chat that
  delegated it, as its delegate's result, and in the update of the
  person who asked for it.
- **Its hold,** the forge's hold part (jig's `client/domain/objects.md`,
  section 5): its reason in section 5's words, and the head and the
  failure it is about, each with a link on the forge, on the held card
  wherever it shows: the change's page, the chat, the inbox's
  escalation.
- **A repository named by owner and name,** in place of its numbers
  (`family.md`, 1), from the same reads.

Beyond it: the way to landing step by step (2.1 to 2.3), the queues
(section 3), health (`repositories.md`, 2), what each act on drift would do (5.2), the
outbox and the deployment's part (`deployment.md`).

## 3. Changes and landing queues

temper's `Changes { project }` page (`../pages/README.md`): what is on the
way to a project's landing branches and what last landed there, from
the subject `changes` (`records.md`):

- **The queue,** per landing branch with ready changes: those changes
  in order (`../../domain/forge/queues.md`), each with its title and
  where it came from. The change that owns the queue leads, at whichever
  step of its turn; the first says it is checking, and since when, and
  the others their place.
- **Landed recently:** the project's last landings, newest first, across
  its repositories, from the landing log
  (`../../domain/after-landing.md`, section 10), which keeps each change's
  title past its close.
- **A branch's red,** in the page's line: while a retained verdict blames
  a landing on the branch, unrepaired (`../../domain/after-landing.md`,
  5.2), or a live repair of the branch runs (`../../domain/forge/queues.md`
  ).
- **A project that lands by turn** has no queue (`../../domain/landing.md`,
  sections 8 and 9): its page's queue is empty, and what it lands is
  listed as landed.

Each row opens its change's `Task { task }`: a change has no page of
its own. What else the page could draw is section 8's.

## 4. Approvals

A person approves a change in one of two ways, as its gates and its
project's landing rule ask (`../../domain/forge/change/gates.md`, 1 and 2).
Neither is temper's default: a person's approval is a gate a plan or a
change is given when a person asks for it, or a project's rule
(`../../README.md`, section 2).

- **In the client:** a person task addressed to the project's
  maintainers, one of whom takes it, whose contract is an approval of
  the change at a head (jig's `client/domain/objects.md`, section 7;
  jig's `domain/people/README.md`, section 7). Its card, in the inbox and on
  the change's page, says:
  - *what is asked:* approve this change at this head, and who asked
    for the approval: the change, its plan or a landing rule;
  - *the change at that head,* the forge's part: its pull request, CI at
    the head, the other gates' verdicts, what changed since an earlier
    approval if there was one, and a link to its diff on the forge;
  - *the answer:* approve, naming the head; or cannot do it, with a
    reason, which holds the change when the approval is blocking, the
    reason in the person's words (5.1).
- **On the forge:** reviews by people of a role, counted by the landing
  rule at the head or, where the rule allows, a clean update of it. The
  panel shows each approval counted, by whom and at which head, and how
  many more the rule asks; the person reviews through the panel's link,
  or through the change's card under Needs you while it waits (4.1).

If the change's head moves while the card is open, the card says so. A
clean update temper made carries the approval where its gate counts
clean updates (its freshness, `../../domain/forge/change/gates.md`, 1); any other
new head asks again, as a new person task at that head, and the old card
says it has been replaced.

### 4.1 Under Needs you

A change whose merge waits on a review on the forge is a card under Needs
you for each person whose review it waits on, and for the project's owners: a
row of the subject `approvals` (`records.md`; `../../domain/forge/change/questions.md`, 2),
merged with the inbox's cards (`../pages/README.md`, *Needs you*). It draws as
every other card does, in the greenfield mock's `needs.html`, and no look of
its own:

- **Its mono line:** TITLE, the change's title, a link to its task; " · an
  approval"; " · " and its age, since the row's `since`, as 5.12's span; " ·
  later" with it.
- **Its question:** "Approve TITLE on Forgejo", or "on GitHub", by the row's
  `kind`. It is the card's SHORT too.
- **Its context,** in `.ctx`, these sentences in order:
  - the rule: "The deployment's landing rule N" for a rule of scope 0, else
    "This project's landing rule N"; then, where it asks for one person, "
    asks for one approval by ROLE.", ROLE "an owner", "a maintainer", "a
    member" or "an observer" for roles 0 to 3, "someone with role R" for
    another; where it asks for more, " asks for P approvals by ROLES; C so
    far.", ROLES "owners", "maintainers", "members", "observers" or "people
    with role R", P and C in digits;
  - for a rule by path, the row's `sets` (`../../domain/forge/change/questions.md`,
    2): "Signs off changes to NAME.", NAME each set's name, joined "A and
    B" or "A, B and C", "path set N" for one with no name, so "Signs off
    changes to vsr's protocol core." (`../../draft/what-reaches-you.md`,
    section 7); none for a rule naming no set. Nothing waits behind such a
    change, which is out of its queue, so the card counts none;
  - "There are too many reviews to read them all, so one may already count.",
    where its reviews weren't read whole;
  - where the viewer's review wouldn't count, why. Too few: "Fewer people can
    approve it than the rule asks for, so it waits until the rule or the
    project's people change." No account: "temper knows no Forgejo account of
    yours, so your review wouldn't count.", "GitHub" on GitHub. Another role:
    "It waits for a review by ROLE; yours wouldn't count.", ROLE as above.
- **Its acts:** where the viewer's review would count and the card has an
  address, the primary "Review on Forgejo ↗" or "Review on GitHub ↗", or,
  for a row with `sets`, "Sign off on Forgejo ↗" or "Sign off on GitHub ↗"
  (`../../draft/what-reaches-you.md`, section 7), in
  `.choices`, a link opening the review in a new tab; otherwise no primary, so
  an owner still sees a wait nobody can end and is never sent to a review that
  would come back unknown. Then the pair: "Open", to the change's `Task { task
  }`, and "Later", which marks it as any card is marked (`../pages/README.md`,
  *What the client keeps*). Nothing is answered in the client: a person who
  won't approve asks for changes on the forge, saying what to change there,
  and the change repairs with their remarks (`../../domain/forge/change/gates.md`, 1), then
  waits for their review at the repaired head; past its repairs it holds as
  any change past them does (5.1).
- **Its address,** the pull request's files page, where the review box is:
  - on a Forgejo whose page facts offer sign-in (`../../protocol/openid-provider.md`,
    3): `ROOT/user/oauth2/temper?redirect_to=` and the path
    `/OWNER/NAME/pulls/N/files`, encoded as a repository's link is (`deployment.md`), ROOT
    as for "Open Forgejo ↗", so that one click signs the person in and lands
    on it;
  - otherwise the row's `address`, made by the engine from the forge's
    configuration as a change's panel's pull request address is (`served.md`, 2), with
    `/files` after it: `ROOT/OWNER/NAME/pulls/N/files` on a Forgejo and
    `WEB/OWNER/NAME/pull/N/files` on GitHub; empty for a forge with none, as
    the playground's;
  - an address that isn't a safe `http` or `https` URL is none.
- **On Home,** the first card as here, its primary in `.choices`, then quiet
  "Open" and quiet "Later"; as one of the others, its line, SHORT and at the
  right its kind and age (`../pages/README.md`, *Home*).
- **On a goal's page,** a band for a change under that goal (`../pages/goal.md`,
  2).
- **In the terminal,** none: its inbox's list stays as it is
  (`../../draft/v0.1.0.md`, section 6).

## 5. Holds, in the forge's words

A held change's card (jig's `client/domain/objects.md`, section 5)
shows the forge's hold part: what happened, the head and the failure it
is about, by whom and when where another hand did it, a link to each on
the forge, and what each act will do. A release resets the change's
repairs, resolutions and updates, and counts its stall from the release.
Nothing on the forge is written over until the person decides
(`../../domain/forge/change/repairs.md` and `../../domain/forge/change/README.md`, 3).

A local watch-admission wait needs no release: recovery or a departing
read can temporarily consume the client's capacity, and temper retries.
A persistent resource refusal names the operation, its resource, the
configuration member, its value and the required usage. Its words say
what to reduce or reconfigure before release. Release asks for a refused
watch set again, and holds again while the same bound refuses it. A branch, base or pull
read failure names its backend cause and what a release retries.

### 5.1 The change's own

| Held because | The card says | A release |
|---|---|---|
| stalled | the wait, and since when: CI that never reported on its head, a mergeability never known, a gate's task stuck | takes the change back to the step it was at |
| repairs spent | the repairs made of their limit, and the last failure: CI's failed step, a gate's remarks | takes it back, its repairs counted again; amending its gates or its producing charter's instructions first, where the failure is not the change's |
| resolutions spent | the conflicts resolved of their limit, and the base it conflicts with | takes it back, its resolutions counted again |
| updates spent | the updates made of their limit | takes it back, its updates counted again |
| an approval declined | who declined it, at which head, and why | amended to drop the approval and released, in one request; released alone, it is held again at the same head |
| a requirement refused at the merge (`../../domain/forge/change/gates.md`, 2) | which of its landing rule's requirements refused, at which head: CI failed, a gate asked for changes; a person's review on the forge asking for changes starts a repair instead (`../../domain/forge/change/gates.md`, 1) | takes it back, judged again at its next merge: held again while the refusal stands |
| a delegate or a write failed for good | which: its producing task's failure, or the forge's refusal of a write (a protected branch, a forbidden write) | takes it back, to decide afresh |
| a write uncertain, of a kind that cannot be recovered (`../../domain/forge/README.md`, section 4) | the write, that it may or may not have been made, and where to look on the forge: the branch, the pull request's reviews or comments, the issue | makes it again, or goes on without it, as the person says (jig's `client/domain/objects.md`, section 5) |

### 5.2 Drift

A held change's card for drift (`../../domain/forge/change/README.md`, 3; jig's
`domain/connectors.md`, section 11): what changed on the forge, by whom
and when, and a link to it there.

| What changed | Release | Otherwise |
|---|---|---|
| its pull request closed by someone else | reopens it | cancel: the change closes |
| its branch moved by another identity, backwards, or while nothing of temper's wrote it | takes the new head as the change's, checks it and asks every gate again | cancel |
| its branch deleted | creates it again at the head temper last knew, and reopens the pull request | cancel |
| its pull request retargeted | retargets it back | amend its landing branch to the new base, then release; cancel |
| merged by someone else | not held: it is landed, like any landing | |

## 6. Goals, findings and news on the forge

### 6.1 A goal's issue

- **In its project's home repository** (`../../domain/forge/issues.md`
  ): the goal's page links to it, says when it was last written, and
  that it is a projection: what people write there is not read, and the
  client is where temper is reached.
- **Pull requests name their goal's issue,** so a person reading the
  forge finds the goal, and from it temper's client.

### 6.2 Landings as news

On a goal's page, with what it hears (jig's `client/domain/pages.md`,
5.2): each landing in its repositories, marked as its own, overlapping
its work (it woke the goal), not overlapping (kept), or unknown, as for
a landing too large to compare (it woke the goal)
(`../../domain/forge/README.md`, 5.3).

### 6.3 Findings

The issues agents filed (`../../domain/forge/issues.md`): a bug
found while exploring, a follow-up a review raised, a flaky test. They
are the forge's part of a project's page (jig's
`client/domain/pages.md`, 6.1):

- **Each** shows its title, its repository and issue, the task that
  filed it as a chip, when, and its kind and state as its labels say
  (`../../domain/findings.md`, section 2): `new`, `routed`, `backlog` or
  `dropped`.
- **Taking one up** sets a goal that names it, the board's form with its
  words filled in (jig's `client/domain/pages.md`, 6.2), or starts a
  chat about it, with the finding named in the chat's first words, so
  its brief carries it (jig's `client/domain/pages.md`, section 4).
- **Dropping one** gives a reason, the forge's own request (jig's
  `client/domain/root.md`, section 6).

## 7. Taking part

What the forge shows of work on what others own
(`../../domain/forge/README.md`, sections 7 and 8), possible with the parts
above:

- **A participating object** (an outside contribution, an issue others
  opened) shows on the page of the task dealing with it: what others
  wrote there, as the news it heard, and what temper would write, a
  reply or a review, as a proposal card with its exact text, when the
  project's policy has a person accept it first.
- **Where temper may not merge,** a ready change waits for a maintainer's
  merge, and people's reviews on its pull request show as its news.
- **An upstream submission** is a proposal a person accepts before
  anything is posted, showing exactly what will be.

## 8. Open questions

- **The approval card on a real forge** (4.1): after-landing QA checks
  the rendered look, owned-Forgejo sign-in and a real human review on each
  forge (`../../testing/README.md`, section 7). Default: the joint-world
  Forgejo card and files-page link in `../../testing/checks.md`, 3 of that document, with client
  and view tables for both kinds and the reasons for no review link.
  Those checks establish no rendered comparison or real review.
- A repository link in a folded forge row: `deployment.md`, section 1.
- **A visitor's links to the forge** (9.3): a commit, a pull request or a
  run linked where its repository is known public. Default: none. The
  forge's state would have to keep whether a repository is private, and
  notice when that changes.
- **A public queue merge past the log** (9.2): a merge shows while the
  landing log's ring holds it and jig hands its change's task. Default:
  no older history. The board's line names the loss of ended goals before
  publication and across a restart (`../web/public.md`, 3), not eviction from
  the log's ring or jig's index.
- **What else `Changes` could draw** (section 3): a ready change's goal
  and priority and when it became ready; a change past the queue's window
  marked as going ahead; the order's rule in a line; changes not ready
  yet, by step; held changes, each with its reason; the branch's tip and
  CI, its landings paged; a paused queue in words, with its repair's chip
  and its release; "waiting for someone else's merge". Default: none is
  drawn, as the greenfield mock draws none, and held work is in Needs you
  (the user, 2026-11-05).
- **A change's requirements, heads and CI contexts in its panel** (2.2;
  `served.md`, 2). Default: left out until the connector keeps them beside the
  change's record (a judgment of each requirement at the current head,
  its heads with what made each, each context's state and its failing
  job, and a step's and a stall's times, which 2.1's "for how long"
  and 5.1's "since when" need), each a field of `ChangePanel` added in
  the version move that first needs it.
- **A change's files in the client:** whether the panel lists the files a
  change touches, from the connector's read, short of showing the diff.
- **A change's questions on its page** (`served.md`, 2; `../../domain/forge/change/questions.md`,
  1). Default: the panel carries them and `ctl`'s `change show`
  prints them; the change's task page shows none until the web client's
  pages for a plan's work are designed (`../../draft/reference-case.execution.md`,
  section 6, leaves them to rung W).
- **Conflicts seen early** (`../../domain/forge/README.md`, section 11):
  warning two goals whose open changes conflict before either lands, as
  a panel on both.
- **A person's approval as a rule,** posted on the forge as the person
  (`../../README.md`, section 8), which the approval's card would then
  say.
- A secret text area for the pasted key: `deployment.md`, section 1.
- **One percentage for both binaries** (`deployment.md`). Default: each binary says
  its own, so the bar starts again for the runner, until their sizes
  are pinned beside their digests (`../../protocol/setup/README.md`, section 9).
- **The connect-flows comparison** (`repositories.md`, 1, `deployment.md`): temper/ui checks the
  request's forge panels, setup rows and repository sheet after landing
  (`../web/look-requests.md`, 4; `../README.md`, section 5). Default:
  retain the shared welcome panels and the sheet's search and scroll;
  Home keeps its own sheet, and the gallery's `sheet-from-home` and
  `sheet-from-chats` draw the two entry paths (`family.md`, 3).
  Compare the request-owned controls, including the sheet's select
  chevrons, source chooser, captions, Copy public's four-line box, list
  margins, empty search and More summary, and the connect actions' type.
  Settings' shared H1 and the open project menu row keep their frame's
  design, as the repair request leaves them.
- An App of one's own, offered again: `deployment.md`, section 1.
- A connect's stop told from the one before it: `deployment.md`, section 1.
- A browser on another machine: `deployment.md`, section 1.
- Mending a stopped managed or owned forge: `deployment.md`, section 1.
- The picker's pages kept: `repositories.md`, section 5.
- **The project's own repositories in the picker:** `Reachable` names no
  project, so the engine lists them. The client joins its project’s
  repositories to mark “In PROJECT” and disable their checkboxes (`repositories.md`, 1).
  Filtering them at the wire would need a project in the request;
  default: keep them visible and marked in the client.

## 9. Public documents

What a visitor reads of a public project's forge, without signing in
(jig's `domain/people/public-reading.md`; `../../draft/first-release.md`,
5.3): the forge's public projections, beside jig's board and cards. Each
is a record of its own, made by a projection from the forge's state,
never a cut of a panel: a field is in it only once added there. None
holds words, a configured name, an internal number or a forge's address:
no string at all (9.3).

### 9.1 Their kinds

Declared by the forge family to jig's clients' component at start, read
by jig's `PublicRead` (jig's `protocol/client/public-reads.md`; over HTTP,
17.5), each naming a public project first, made by the forge child from
its state and encoded there in `public-v1`, and carried in jig's
`PublicApplication`, whose list names every task a record refers to by
its place (jig's `client/domain/contract/README.md`, section 10):

| Kind | Names | Answered with |
|---|---|---|
| *change* | a project, a public change's task | `PublicChange`: the change's task; its head's hash; its step, without words; its hold's kind; CI's state; its gates, each its number, kind, whether it blocks and its verdict; its repairs, resolutions and updates, used and allowed; its way; and whether it is a member of a public landing |
| *landings* | a project | `PublicLandings`: the public landing that holds the turn, if one does; the project's public landings, newest first, then its public queue merges, newest first, together within the panel's bound, each its members' tasks, its phase and its landed commits' hashes, a queue merge's phase landed and its one commit; and each landing branch's queue, as its public changes in order, each its task, its public goal and when it became ready |
| *landing* | a project, a public change's task | `PublicLanding`, of the public landing that change is a member of: its phase; its hold's kind and counts; its members, each its change's task, its base's, candidate's and landed commits' hashes, and its gates, each its number, kind, whether it blocks and its proof's state; its stale proofs' count; whether it holds the turn. Of a public queue merge, where the change has no landing: phase landed, no hold, each of its changes a member with its landed commit alone and no gates, its counts zero, and no turn |

### 9.2 Which are public

jig's rule (jig's `domain/people/public-reading.md`, 7), applied by the forge to
the public tasks jig's core hands it with the ask:

- **A public change** is a change whose task is public. A change
  outside every goal's tree (`../../domain/forge/queues.md` lets one
  queue) is not, nor is a change of another project.
- **A public landing** is a landing of the project whose every member
  is a public change. A landing with a member that is not, a pin-only
  member among them (`../../domain/landing.md`, 4.1), is not public at
  all: it is absent from *landings*, and *landing* through any of its
  members is unknown. A landing's own task, which the deployment
  requests for the project (`../../domain/landing.md`, 4.2), is never
  named: a public landing is reached through one of its members.
- **A public queue merge** is a landing the landing log holds
  (`../../domain/after-landing.md`, section 10) with at least one change,
  every one of them a public change that went by the queue. A queue's
  merge has no landing's row or task, so the log is its record and
  its change's task the way to it. Past the log's ring, or once jig no
  longer hands one of its changes' tasks, it leaves *landings* and its
  *landing* is unknown. A change by turn is never one: it is read through
  its landing, while that landing's row lasts.
- **Filtered before it is ordered or counted:** a queue lists its
  public changes alone, in the queue's order; a change's place and the
  tasks ahead of it are not given, since they count changes that are
  not public; the turn is given only when a public landing holds it;
  every count is of public members.
- **Named through the list:** every task a record names, and the goal
  a queue's change serves, is one of the tasks jig handed, listed in
  the answer's tasks for jig's core to check again before it answers
  (jig's `domain/people/public-reading.md`, 10). A task jig did not hand is never
  named.

### 9.3 What crosses, and what never

- **Structure and mechanics:** steps and phases, holds' kinds, CI's
  states, gates' numbers, kinds and verdicts, counts, commit hashes,
  times to the second, a run's state.
- **No address on the forge:** no record holds a repository's, a pull
  request's, a commit's or a CI run's address. A forge answers a stranger
  404 on a private repository, as every box's is, and the forge's state
  does not hold which are public: whether a repository is private is read
  only in the list of what can be adopted, and kept nowhere (section 8).
  So a visitor reads a commit's hash as text, never a title, body or
  message, and a repository's owner and name never cross.
- **A step without words:** the step's name; for a repair, its
  reason's kind (CI, a gate's number, semantic) without its words; for
  a landed change, its merge's hash.
- **Never:** a question's line and every question (2.2), an authority
  hold's reason, a verifying or red tier's text (the step carries the
  tiers' count alone), a change's or a landing branch's name, a gate's
  or a charter's configured name, a review's words, a result, findings,
  the outbox's descriptions, a forge's credentials, installations or
  accounts, the settings, a logged landing's branch, title or pull
  request's number, and a task's or a project's number.
- **Every field's source:**

  | Field | Source | Made by |
  |---|---|---|
  | a task, a member's task, a queue's change and its goal | the change's or landing member's task, the goal of its public tree | its place in the answer's tasks |
  | a head, base, candidate, landed or merge hash | the change's and landing's commits; a queue merge's landed hash, the landing log's commit | as is |
  | step, repair kind, hold's kind, phase, CI's state, way, proof's state | the change's and landing's state | its kind alone, never its words |
  | a gate's number, kind, whether it blocks, verdict | the landing rule's gates and their verdicts | never the gate's name |
  | counts used and allowed, stale proofs | the change's and landing's counters | public members alone |
  | when it became ready | the change's `ready_since` | to the second |

### 9.4 Their records and golden

- **A schema of their own,** `temper-client-protocol-wire`'s
  `public-v1.schema`, versioned apart from the family's other records:
  its records name only one another and scalars, never a record of the
  panels' schema, so a field added to a panel never reaches a visitor.
  A test reads the schema and refuses a name from elsewhere.
- **The golden of the whole reachable wire shape:** a text file beside
  the schema listing every record and enum reachable from the kinds'
  answers, each field with its type, tag and default and each variant
  with its tag and payload, which a test compares with the schema. Any
  change shows in review as a public one; adding a field passes the
  user's test, that no agent wrote it.
- **Tested for meaning:** each projection compared with an expected
  record built independently from the source table, from a forge state
  whose forbidden sources (a question line, a branch, a gate's name, a
  reason, tiers' text, a task number) hold distinct markers that no
  answer's bytes contain; mutation tests show the golden failing on a
  nested field added to a copy of the schema and the projection test
  failing on a field filled from the wrong source.
- **Bounded** as the panels are (`Bounds`), each list within its
  panel's bound, so each kind's largest answer is known at start and
  declared to jig.

### 9.5 Their world

In the forge's world (`../../domain/forge/README.md`, section 10): a public
project's change, from producing to landed, read at each step as its
public document, with no question line, branch, gate name or reason in
it; a landing with two members and a failed gate read as its public
document, the failed proof's state in it, reached through either member; a
change of a private project, one outside every goal's tree, and one in
a conversation's tree, unknown; a queue holding a change outside every
goal's tree, which the landings leave out and count nowhere; a landing
with a member that is not public, absent and unknown; a queue's merge
read as a landed line and through its change as its landing, its commit
the change's merge and no branch, title or pull request's number of the
log's in it, and absent and unknown once its task is not handed; a task
jig did not hand never named; and the golden test failing on a field
added to `public-v1.schema` until the golden says it.
