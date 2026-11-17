# A change opened from a pushed branch

A person pushes a branch and opens a change from it with one act, and
temper lands it as it lands a change an agent produced, under the same
rules: the act's request, the root's expansion of it into the person's
own change task, the change's first state, taking, and what the change
keeps and serves. Written from `../../../draft/mechanical-changes.md`,
sections 1 to 10; what that draft leaves for later is section 8's.

## 1. In one page

- **One request,** `OpenChange` on the client's wire (2), sent by
  `temper ctl change open` (`../../../client/ctl/areas.md`, area 6).
- **The person's own change** (3): the root builds one change task by
  the builder a run's `change` call uses (`README.md`, 1), from the
  person's creating offer in place of a run's delegation context, and
  the core admits it as the person's batch: the person is its requester
  and its funder. No goal and no producing task.
- **Taking** (4), its first state: it reads the source branch's head
  once, keeps it and makes the change's own branch at it; then it goes
  on as a change whose producer pushed that head.
- **Then any change** (5): its way, its gates, its trouble and its end
  are `README.md`, 2 to 4's.
- **The pushed branch stays the person's:** nothing writes, holds or
  deletes it, and pushing it again changes nothing of the change.
- **Kept and served** (6): the change's row keeps its source and its
  kept head; its panel shows them.

## 2. The request

`OpenChange`, of the project it names, from any party with a role there;
whether the role may make the change is the core's to judge (3). Its
members are the act's, as the person gave them; their record and bounds
are `../../../client/forge/records.md`'s:

- `project`; `repository`, an owner and a name, none for the project's
  only repository;
- `branch`, the source branch, in that repository; `at`, the commit the
  person means, as 40 or 64 lowercase hexadecimal digits, none for its
  head when the change takes it;
- `into`, the landing branch, none for the repository's default branch;
- `title`; `body`, none or more words;
- `budget`: an amount, or as allowed.

The root resolves it before anything is made. Each refusal is
`open_refused`, nothing made:

- **The repository:** among the project's repositories a change may
  land into, adopted with a role other than context and with the
  forge's `open` kind held. With none named, the only one; with several
  of them, refused naming each by owner and name (`several`); with none
  at all, refused (`no_repository`, a case the draft leaves unsaid); a
  named one not among them, refused (`not_a_repository`).
- **The landing branch:** `into`, or the repository's default branch. A
  branch under the repository's prefix, temper's own (`../branches.md`),
  is refused naming the repository and the prefix (`under_prefix`): a
  goal's branch as the target is section 8's. A branch the repository
  lacks is not refused here: the change is held at its opening, the
  forge refusing its pull request, as a run's call naming one is
  (`../../agent/tools.md`, 1).
- **Their names:** a source or landing branch past the deployment's
  forge name bound, which the client cannot know before sending, is
  refused naming the bound (`name_past`).

Its outcomes: `opened`, the change's task; `open_refused`, one of the
above; the family's `core_refused`, jig's refusal facts whole (3); or
unknown, no longer held. A replay under the same key answers the first,
as every keyed request is answered (jig's `domain/people/acts.md`, 1).

## 3. The change it makes

- **One builder, two origins.** The root builds the change's one member
  by the builder that builds a run's change (`README.md`, 1). Its origin
  is a run's delegation context, or the person's creating offer in the
  project, what a root task they make may have under their role (jig's
  `domain/authority.md`, section 6): the role's authority
  and models, the task allowance and depth a root the person makes is
  checked against before it takes its own slot and level, and the spend
  the person's pool allows. The builder counts the change's own slot
  and level from either origin alike, so a person's change and a run's
  of the same branch are narrowed alike.
- **Its authority,** the offer's, narrowed as a run's change is: its
  grants the repository's a change holds; its delegation to its
  producer's charter, the forge's change charter since it names none,
  and the review charter, the person gate's role where a rule covering
  its landing branch asks for one, and the merge job where the role's
  delegation holds it; its task allowance a change's own, within the
  offer's less its own slot; its depth a change's, within the role's
  less its own level. No note scopes.
- **Its budget:** an amount, that amount, named; as allowed, what the
  offer allows, which the core resolves at admission as for a goal the
  person starts, the builder checking the amount it reads from the
  offer. Either is reserved from the person's role's pool at admission
  (jig's `domain/authority.md`, section 7). Its reviews, resolutions and
  repairs spend it; the change itself spends nothing.
- **Its parameters:** 1, the repository; 2, the landing branch as
  resolved; 10, the title; 19, the source (3.1); and 14 only where the
  project's route is `up`, as for a change that names none
  (`README.md`, 1). No 3, 4 or 11: no handed-in branch, no body of its
  own and no producer's charter, so its resolutions, repairs and checks'
  gates run under the forge's change charter and its agents' gates
  under its review charter, their numbers kept on its row as any
  change's. They count against jig's `tasks.parameters` as the builder
  counts a run's change.
- **Its words:** `Opened by person N from BRANCH at COMMIT into INTO.`,
  N the person's number, COMMIT the `at` given or `its head`, INTO the
  landing branch as resolved; then, where a body is given, a blank line
  and the body. Its pull request's body is its words, as a change's with
  no body of its own, and its title is parameter 10, fitted as a
  handed-in branch's title is (`README.md`, 1).
- **Admitted as the person's batch.** The root sends the member as a
  core `Create` for the person, a part of their application (jig's
  `domain/tasks/README.md`, 4): the core judges the role, the executor,
  the allowance, the depth and the budget against the creating offer,
  reserves the budget from the person's pool and makes the task with the
  person as its creator, its requester and its funder. The core's answer
  names the task: the request's `opened`.
- **Refusals,** nothing made, each carried as `core_refused`'s facts so
  that the client shows it in jig's words, as `goal start` shows its own
  (`../../../client/ctl/areas.md`, area 2): the core's, whole (a role
  that may not make the change, an executor it does not offer, its task
  allowance, its depth, a budget past the offer); and the builder's,
  before the core sees the member, put in the same facts: an offer
  with one task left, since the procedure takes one and its producer
  another, as the task allowance lacking; a depth of one as the depth
  lacking; a budget at or below the rules' least run spend, since no
  review, resolution or repair could run, as the budget lacking, needing
  one more than the least run spend; a member past `tasks.parameters` as
  jig's limit on a spec's parameters; a charter the deployment lacks as
  the executor lacking.
- **Nothing above it:** no goal, no goal branch, no coordinator and no
  closer. Its tree is the change alone until a review, a resolution or a
  repair delegates under it.
- **Its questions:** its requester is a person, with no task to pass a
  question to, so it starts with route `none`, as every change a person
  requests does, whatever parameter 14 says.

### 3.1 Parameter 19, the source

Bytes, `BRANCH` or `BRANCH:COMMIT`: the source branch's name, within
the forge's name bound, then, where `at` was given, a colon and the
commit in 40 or 64 lowercase hexadecimal digits. A branch name holds no
colon, so the value's last colon parts them. The start keeps it with the
change, `source`. A value in another form, or one beside parameter 3,
number parameter 11 of 1, or parameter 16, 17 or 18, refuses the
change's start as a malformed parameter.

A change with parameter 19 has no producing task. Its own branch is
`temper/<root>/c<task>`, named in its batch as a produced change's is
(`../branches.md`), and its first state is taking (4).

## 4. Taking

- **Its own branch is held from admission:** the hold is taken as the
  task becomes active, before the branch exists (jig's
  `domain/tasks/holds.md`, 2).
- **Select:** it reads the source branch's head through the forge's
  reads, once:
  - the branch missing, it ends failed, `BRANCH is not in OWNER/NAME`;
  - with `at` and a head that is another commit, it ends failed,
    `BRANCH is at HEAD, not COMMIT: push it and open again`, HEAD in
    hexadecimal;
  - otherwise the head is kept on the change's row, `kept`, in the step
    that reads it, before any effect is sent.

  A change that ends here made no branch; its hold is released.
- **Make:** the change's effect that makes a branch makes its own branch
  at the kept head, under temper's identity. One found already at the
  kept head counts as made.
- **Taken:** the kept head is its pushed head, and it goes on as a
  change whose producer pushed it (`README.md`, 2): a head its landing
  branch already contains ends it with no change, in that section's
  words; any other opens its pull request at once, its title
  parameter 10, with no producer's result to wait for.
- **Never read twice:** once a head is kept, the source branch is never
  read again. A restart before the head is kept reads it, nothing having
  been kept; a restart after makes the branch again at the kept head,
  one found there counting as made, and one found at another head holds
  the change as its branch moved (`README.md`, 3).
- **A read or a make that errors** is retried as the change's effects
  are, the wait counted against the change's stall bound from its
  entering taking, as its producing's would be (`repairs.md`). Past the
  bound, or at an error the forge's client gives up on, the change is
  held, its words `taking BRANCH: WHY`, BRANCH the source and WHY the
  error, and the hold reaches the person as a hold of a task they made
  does. Released, it resumes where it stopped, reading where nothing was
  kept and making otherwise, its stall counted afresh.
- **Cancelled while taking,** it ends as a cancel does, its own branch
  deleted where it was made (`README.md`, 4); one not made only loses
  its hold.
- **temper's move, then temper's moves:** temper's identity made the
  branch, so its repairs', resolutions' and updates' pushes are the
  change's own, and a push to it by another identity is drift
  (`README.md`, 3), as for any change. Drift reads heads and writers,
  not who authored the commits.
- **The pushed branch is never written, held or deleted.** Pushed again
  or deleted after the change kept its head, nothing of the change
  moves; to land a newer head the person opens another change.

## 5. Then as any change

- **Its way** is its project's at its first readiness
  (`../../landing.md`, section 1): under `turn`, it gates on what the
  rules covering its branch ask for, waits for its train, joins its next
  landing and lands when the branch contains its prefix
  (`../../landing.md`, section 3), a rule asking CI and up to date alone
  asking nothing a landing doesn't meet; under `queue`, CI at its head,
  its gates, its turn, its updates and its merge (`README.md`, 2).
- **The rules decide what it needs, unchanged:** a rule asking for an
  agent's review gets one, run under the review charter as any
  change's (`gates.md`, 1).
- **Trouble** (`repairs.md`): a conflict at prepare goes to a
  resolution, a red blamed on it or a review that says no to a repair,
  each an agent's run under the forge's change charter, within the
  change's bounds and its budget. Past them it is held, and the hold
  reaches the person.
- **Its end:** landed, its branch deleted as a landed change's is;
  cancelled, it leaves its landing as a cancel does
  (`../../landing.md`, 4.3) and its branch is deleted; failed, its
  branch kept for a person to read until its tree's root, the change
  itself, closes (`../branches.md`).
- **What the opener sees:** the act's answer, the change's task; then
  `change show`; the change among the project's changes in the changes
  view and `landing history`, shown as a produced change is; its news
  and holds in their inbox, as a task's they made.

## 6. Kept and served

- **Its row** (`../../../protocol/store.md`, 2, *A change's two parts*):
  `source`, the branch and the `at` named or none, kept from the start;
  `kept`, the head taking selected, none before; and the state `taking`.
  Each defaults to none, so a stored change reads as before: temper's
  store is a registered family, its version stays and nothing is
  patched (`../../../protocol/store.md`, section 4).
- **Its panel** (`../../../client/forge/served.md`, 2): its step
  `taking` while it takes, and its source, the branch, the `at` named
  and the kept head, while it takes and after; none for a change with
  no source.
- **The client's wire** moves its version for the new request and the
  panel's source and step, with no bridge
  (`../../../draft/redeploys.md`, section 8): a client of the new
  version meeting an engine of the previous one is refused, `version not
  served`, before any task is made. Engine and workers: nothing moves,
  no job runs and no worker is asked.

## 7. Testing

Focused and fuzzy suites alone (`../../../draft/mechanical-changes.md`,
section 10, whose items are the stories):

- **The connector's world** (`temper/tests/engine/forge`), with a change
  made as the root makes it, parameter 19 given: taking, its failures,
  its restarts, its hold and its cancel, the pushed branch never
  written, and the change landing under `turn` and under `queue`, under
  the stock's rule 1, through a resolution and a repair; its fuzzy
  sequences under `turn` with the trains' combine as `merge` and as
  `squash`.
- **The engine's world** (`temper/tests/engine/domain`) and the root's
  own tests: the request's expansion, its refusals, and one story from
  the request to the change's pull request.
- **ctl** (`../../../client/ctl/testing.md`): the act's usage, its answer,
  its replay and its refusals.
- **Versions:** the client wire's version refused by an engine of the
  previous one.

## 8. Open questions

The draft's later steps, each drafted with the user
(`../../../draft/mechanical-changes.md`, section 11): trouble back to
the opener rather than to a resolution or a repair; what may be opened,
by path set and role; a goal's branch as the target; out-of-band fixes
through the act; opening from the web's Changes page; and moving an
opened change to its branch's newer head.
