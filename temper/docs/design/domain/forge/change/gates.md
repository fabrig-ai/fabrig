# A change's gates

## 1. Gates

- **A gate** is a verdict a change asks for before it lands, at its exact
  head:
  - *a review:* a task for an agent with a lens (memory safety,
    performance, a design), under the forge's review charter, a
    reviewer's (`../../agent/agents.md`), whose verdict is approve or request
    changes, with remarks;
  - *a person's approval,* through a person task addressed to the
    project's maintainers and answered in the client (jig's
    `domain/people/README.md`, section 7): the change approved at the head, or
    declined as "cannot do it", with a reason;
  - *a check:* one of the checks a repository without CI is adopted
    with, a task at exactly the head;
  - *a follow's check,* on a follow change alone: its follow's workflow,
    dispatched by the change at exactly its head with the followed
    source commit, its verdict the run's report (`../follows.md`, section
    6).
- **Blocking or advisory.** A blocking gate's verdict holds the landing;
  an advisory one's is read, and reported, and holds nothing.
- **A review's brief names what it judges,** so the reviewer reads
  nothing to find it: the pull request's number, its branch, its base
  and the head the verdict is asked at, each as the forge's reads take
  them (a commit at the forge's own length, never padded), and its
  address. These open the pull section (`../README.md`, section 3, brief sections),
  before its title, body, files and what differs from the base. The pull
  section's budget is a review's working room, 4,608 bytes on the stock,
  within the assembled brief's and the journal result's 14,592: the
  number, branch, base, head and title are kept whole, and the body,
  the files and the diff cut from their end, the cut marked; a budget
  that leaves no diff is a configuration a test refuses. The reviewer's
  reads go to what the brief leaves out (a file's whole, a check's log),
  never to finding its own pull request.
- **A review that cannot review fails; it never asks for changes.** A
  reviewer that could not read what it judges (the pull request, its
  files, the head), or could not judge it for any other reason of its
  own, ends its task failed, saying why, within the verdict contract's
  failure (`../../agent/charter.md`, 3); its charter's words say so. A failed
  review is no verdict: the change asks for the gate again once, a
  fresh review at the same head, whose brief says it is asked again
  and gives the first review's failure in its words, so the reviewer
  knows what to look past, and a second failure holds the change,
  naming the review's failure, for its requester or a person to
  release or amend. Released, the change's own hold goes with its
  task's, in the same decision, and it asks for the gate afresh at that
  head, its tries counted anew, though not its tasks (`repairs.md`); amended, it decides as any amended
  change. Cancelled, its panel ends with it: a change's state is never
  left held or gating under a task that has ended. It never starts a repair, which would spend a
  repair's bound on nothing the change's diff did. A change's agents
  hold the forge's reads (kind 1) beside the kinds it makes (`../README.md`, section 4),
  so that a reviewer can read its own pull request.
  Temporary local read contention follows the host backoff in `../keeping-up.md`, 3;
  it supplies no review failure or verdict while the read can retry.
- **A blocking gate that says no:** a review that asks for changes, or a
  check that fails, is a repair, given its remarks (`repairs.md`); a declined
  approval holds the change, with the person's reason. The verdict
  stands at that head, so a release alone holds the change again there:
  a person amends the change to drop the gate, then releases it, or
  cancels it.
- **A person's request for changes on the forge is a repair too.** Where
  a landing rule's approval (2, requirement 4) refuses a change's merge
  because a person it counts asked for changes on the forge, at a head it
  counts at (the head itself or, where it counts `clean`, a head the
  change's clean updates led through), the change does not hold: it
  repairs, as after a blocking gate's request, its repair's reason that
  person's request at the head they reviewed (`repairs.md`). So You approve's loop
  closes on the forge: the person reads the change there and approves it
  or asks for changes, and their words reach the agent that makes them.
  - *Where it is found:* where the approval is judged, at the merge
    (2), so a change repairs a person's request once its CI and gates
    hold and it asks to land. The forge's route, which turns the core's
    refusal of the merge back into the change's state, turns a refusal on
    a person's request into that request, as it turns a review's verdict
    into its gate's report, and holds the change's task no longer for it.
    A refusal that is not a person's request (CI failed at the head, a
    gate's verdict, a task its dependencies or question keep) goes as
    before.
  - *Who counts:* only a request the approval counts: a request by an
    account the lookup does not know (`../adoption.md`), by a person of another
    role, or by temper's own writer, refuses nothing (2) and starts no
    repair.
  - *A head moved past:* a request at a head the approval no longer counts
    (one the change moved past by a repair, a resolution or a push, or any
    earlier head where it counts `exact`) neither refuses nor repairs: the
    change waits for the approval at its head, as before.
  - *Bounded as repairs are:* the repair counts against the change's
    repairs; past their bound the change holds on its repairs, as any
    repair past its bound does, and its requester hears (`repairs.md`).
  - *A request with no remarks* still repairs: its brief says that no
    review at that head has words or a remark on a line, and the repair
    reads the pull request with its forge tools.
  - *Words, not instructions:* what people wrote on the forge reaches the
    repair as a reviewer's remarks on the change, quoted in its brief's
    reviews section; the repair's words say so, and the remarks never
    instruct it beyond the task's own words and charter.
  - *Then as after a gate's repair:* the repair's push is a new head,
    which asks for every gate and approval again (below), so the person
    reviews it there.
  - *Remembered for one step:* the request is the route's to give and the
    change's next step's to take, kept in the forge domain's working set
    with no row of its own; after a restart the merge is judged again and
    refused again, and the change repairs then.
- **Asked for by the change, the project and the plan.** A change's own
  gates are its parameters, amended while it is live (a coordinator adds
  a memory-safety review to one change); the landing rules add theirs to
  every change into a branch they cover (2). A gate added while a
  change is queued takes it out of the queue until it has its verdict.
  The rules' gates (3) are not copied into a change when it is made,
  but taken at each step from the rules the forge holds then, every rule
  covering the branch adding its own, enforced or not; a gate a changed
  rule drops is no longer waited on, and a review already running for it
  reports as an advisory one.
- **Valid at a head, and carried over clean updates.** A verdict is given
  at a head. Each gate says which later heads it still counts at
  (`freshness`): *exact*, the head itself; *clean*, every head reached
  from it by updates temper made that merged cleanly, since the change's
  own diff is unchanged. A check is always exact. Any other new head (a
  repair, a resolved conflict) asks for every gate again. The change
  keeps, in its state, the heads its clean updates led through, which is
  the fact the requirements read (2).
- **Posted, if the project wants:** a gate's verdict as a commit status,
  such as `temper/review/memory-safety`, which protection can require
  independently of temper. On GitHub, temper's App has `statuses: read`
  and no more, so temper writes none: `status` is false in the kinds
  there (`../adoption.md`, 4), the post, `Write::Status`, is refused at the connector,
  and nothing waits for it (skein's `forge/github.md`, 4.2). Every
  adoption there says so, and the verdict stays temper's own; the App
  asks for no more (`../../../protocol/setup/github.md`, 1). Where an installation
  grants `statuses: write` (`../../../protocol/setup/github.md`, 1), adoption
  reads it, `status` is true in that repository's kinds, and statuses
  are posted there as on Forgejo; an installation that has not accepted
  the new permission keeps the refusal.
- **When they run:** once CI passes on the head, by default, so a head
  CI fails costs no review; the change may ask to run them at once.

## 2. The forge's requirements

The forge judges four requirements on its own facts (jig's
`domain/connectors.md`, section 7), which landing rules attach to its
merge, kind 4, on a pattern of landing branches (jig's
`domain/authority.md`, section 10):

| Requirement | Its parameters | Met | Wait | Refuse | Guarded or observed |
|---|---|---|---|---|---|
| 1, CI passed | none | CI passed at the head; for a repository without CI, every one of its checks passed at exactly the head; no CI verdict is needed for the explicit adoption choice to land without CI described below | CI pending or unknown; a check with no verdict at the head | CI failed; a check failed; a repository with neither CI nor checks unless explicitly adopted to land without CI as described below | guarded |
| 2, up to date | none | the head contains the base's tip | the ancestry range unread or failed | the head lacks the base's tip: the range counts a commit (`../keeping-up.md`, 3) | observed, freshness 0 |
| 3, a gate | its number; `exact` or `clean` (1) | its verdict, at the head or, where `clean`, at a head the change's clean updates led through, approves | no such verdict yet, or one pending | such a verdict asks for changes | guarded |
| 4, an approval | a role; how many people; `exact` or `clean` | that many distinct people holding the role in the project approved the pull request at the head, or where `clean` at a head its clean updates led through | fewer have; the reviews too many to read whole | one of them asked for changes at such a head, which a change repairs (1) | guarded |

- **The explicit choice to land without CI** is offered at GitHub
  adoption only when Actions is on, there is no workflow and the landing
  branch has no required checks (`../../../draft/github.md`, 10.3). Choosing it
  adopts with CI off and allows adoption without replacement checks. Otherwise a
  repository without CI names the checks that stand for it. Neither this
  choice nor replacement checks waive a landing rule or required checks;
  CI pending or unknown on a repository adopted with CI on remains a wait.
- **Judged at the merge's head.** The core asks the forge for each
  verdict as it checks the merge (jig's `domain/root.md`, section 9),
  and the forge answers from the fresh reads its step made for that
  decision (`../keeping-up.md`, 3). Facts read for another head are wait.
- **Guarded or observed.** Each forge applies a merge only at the head
  it names, and a forge's merge queue takes a pull request in only at
  the head it names (`../README.md`, section 4; skein's `forge/forgejo.md`, section 2;
  `skein/docs/design/forge/github.md`, 5.1), so a fact pinned to that head still holds as
  the merge lands: CI, gates and approvals are guarded, and are marked
  as requirements that must be guarded. Up to date is observed: the base
  may move between the decision and the merge, and the forge does not
  check it then. Its freshness is zero, so only the reads of the step
  that decides the merge count. A base that moves in between lands the
  change on a tip it was not checked against, which the branch's own CI
  then checks (`../queues.md`). Through a forge's merge queue, the forge
  itself merges the head onto the queue's base and checks the group
  before it lands (`../queues.md`, 1).
- **An approval,** the requirement, is a review a person posts on the
  pull request on the forge, as themselves, apart from a person's gate
  answered in the client (1). The forge reads the pull request's
  reviews, keeps each author's latest, and counts the distinct people
  among them whom the lookup by forge and user (`../adoption.md`) knows on the
  pull request's forge, with the role in the project; the same user's number
  on another forge is another account, shown as one unmapped. A review that
  only comments counts for
  nothing. temper's retained forge writer never counts as a human, even if
  an existing identity holds the required project role, including after restart.
  It is the one fact temper judges of what people write on its pull
  requests (`../README.md`, section 7); where one of them asks for changes, their
  review's words and remarks become a repair's brief (1). A project has approvals only if its rules ask
  for them: temper's own default has none (`../../../README.md`, section 2).
- **The panel's approval evidence** is the last judged review set per
  change, kept only in the forge domain's working set, bounded like
  `change_due`, with when it was first kept at its head, and removed when
  the change's merge is admitted or the change ends: a review that counts
  takes the change from Needs you (`questions.md`, 2) once a step's facts
  count it, or as its merge goes, not when it
  lands, and until a merge refused or failed is judged again its panel
  says reviews not read yet. Both procedure and
  landing merge waits supply its head, whether the read is complete,
  the verified reviewers and the reviews from unmapped forge accounts.
  So does each step of a change ready in a queue (`Queued` or `First`)
  that an enforcing rule covering it asks an approval of: the forge hands
  the root the step's evidence, `Request::Reviewed { task, evidence }`,
  and the root judges its reviews as it judges a merge's
  (`landing_reviewers`) and keeps them, so the facts that take a change
  out of its queue and bring it back (`../queues.md`) are a step's own.
  No row, event or restore carries it. The panel selects requirements
  from the live enforcing rules, as the merge judgement does, and shares
  its reviewer count: role, freshness, distinct people and conflicting
  verdicts have one meaning. Each requirement shows its scope and rule,
  role, freshness, required count, counted people and review heads, and
  remaining count. No evidence at the current head says reviews not
  read yet, including after restart; an incomplete read says incomplete;
  a complete read with none qualifying says zero counted. Evidence at
  another head is unread for this head. An unmapped review names its
  forge account and head, without inferring a person from a display
  name. Removing a rule removes its panel line at once.
- **Landing rules** are settings, the forge's own kinds (3): the
  deployment's and each project's, each covering a pattern of landing
  branches with CI, up to date, its gates and its approvals. A merge
  meets every rule that covers its branch: a project's rules add to the
  deployment's, and never replace them; gates a plan asks for on one
  change are the change's own (1), and add to the rules', never
  replace them.
- **A rule by path** names path sets of its project's classes (`sets`,
  `../../../protocol/configuration/engine.md`; `../attention.md`, 6) and
  covers a change into a branch its pattern covers only where the change
  touches one of them at the head judged; a rule naming none covers every
  change there, as every rule did before (`../../../draft/what-reaches-you.md`,
  section 7). Covering is one predicate, `covers(rule, branch, touched)`
  in `temper-engine-domain-forge`'s `gates.rs`, for every reader of the
  rules: the procedure's waits (`requires`, below), the gates a change
  asks for (1), requirements 5 and 6 at the merge, the panel's
  requirements (`README.md`, 5) and the approvals' rows (`questions.md`, 2).
- **What a change touches** at a head is the paths of its diff from its
  merge base with the landing branch to that head: the forge's comparison
  of the landing branch's tip and the head, `Read::Compare { before: tip,
  after: head }`, which counts from their merge base (skein's
  `forge/README.md`, 3.3), so an update bringing the landing branch's own
  edits into the change touches nothing. A step reads it with the change's
  review facts, once its base and head are read, wherever a rule covering
  the branch, enforcing or not, names a set, and nowhere else. The decision
  that reads it matches its paths against the project's classes as the
  forge holds them then (`PathSet::matched`, `temper-protocol-reaches`),
  giving a `Touched { sets, whole }`, in `gates.rs` beside `covers`:
  - `sets`, a u8: bit `n − 1` for set `n` where one of its patterns
    matches one of the paths, and for each number 1 to 6 at which the
    classes hold no set (`../../../protocol/configuration/engine.md`);
  - `whole`, false where the comparison left a path out (its `changed`
    past its paths), named other endpoints than asked, or failed; then
    `sets` has every bit: a change whose files the forge can't give whole
    touches every set, as an unknown overlap wakes (`../README.md`, 5.3);
  - `Touched::UNKNOWN`, every bit and `whole` false, for a change whose
    head has no comparison read: one before its first step reads it, or
    after a restart until its next does.
  A rule naming sets covers where `touched.sets` holds one of their bits.
- **At that exact head:** the step's `Touched` is the merge's evidence,
  `ChangeEvidence`'s `touched`, judged with its review facts at the head
  the merge names, so a merge lands only what was checked and a head that
  newly touches a set waits again for what its rule asks. The forge keeps
  each change's last `Touched` at its head in its working set, beside the
  review facts and dropped as they are, for the readers outside a step:
  the panel and the approvals' rows judge a change by the kept one at its
  current head, else `Touched::UNKNOWN`. Under the turn, the turn's merge
  of a member reads the comparison of its main's tip, read afresh, and its
  candidate, with the member's reviews, into its `LandingEvidence`'s
  change evidence (`../../landing.md`, 8).
- **The `paths` kind and a rule by path are two judgements:** `decided`
  judges whether a choice may be made for a person, on the paths a call
  names (`../attention.md`, 6); the rule judges whether a change may
  land, on its diff at the merge. A choice decided inside a path set
  still lands only with the rule's sign-off.
- **The usual rule** for `main`: CI passed at the head (or the
  repository's checks, without CI), the head up to date with the branch,
  and an agent's review as a gate. A project adds what it wants on every
  landing (a review with a lens, people's approvals) to its rule.
- **Enforced, and waited for.** A rule marked `enforce` makes each of its
  criteria a requirement the core checks at the merge, whatever any
  task's authority says: the rule. Every change also waits, on its own,
  for its blocking gates and for what its landing branch's rules ask of
  CI and of its base's tip (`README.md`, 2, below): the mechanics. A
  rule not marked `enforce` leaves its criteria to those waits alone. A
  requirement that waits keeps the change at its turn until it is asked
  again; one that refuses holds the change, saying which, and never
  lands it. Approvals are waited for at the merge, and a ready change
  whose step's facts show one short waits out of its queue for it
  (`../queues.md`), never holding the branch; for what the
  procedure waits for itself, a refusal at the merge is a bug in the
  procedure, counted (jig's `domain/authority.md`, section 10).
- **What the procedure asks of the rules:** for a repository's landing
  branch, `Requires { ci, up_to_date }`, from `temper-engine-domain-forge`'s
  `requires` (in its `gates.rs`, beside the gates the rules name) over
  the rules as the forge holds them (3), with the step's `Touched`: `ci`
  where some rule covering the change, the deployment's or its project's,
  enforcing or not, asks CI passed; `up_to_date` where some such rule asks
  up to date; both where no rule covers the change, so a branch without
  rules, or one whose only rules are by path and untouched, keeps the
  procedure's own waits. It is read at each step of the change, never
  kept on it, so a rule changed reaches a live change at its next step.
  A rule asking neither lets a change merge at its head once the forge
  finds it mergeable, unchecked by CI and never updated (`README.md`, 2 steps 3
  and 6; `../README.md`, section 6), a conflict going to the merge job and a
  resolution (`repairs.md`): the rule a goal's own branch lands under (`../branches.md`, 1).
- **Force-pushing does not exist.** No kind forces a push or moves a
  branch backwards: a run's push is a fast-forward
  (`../../../protocol/git.md`, 4.8), an update merges the base in
  (`../README.md`, section 6), and a branch found moved backwards is drift (`README.md`, 3).

- **Two standing requirements** replace the four above as landing rules'
  attachments (`../../../protocol/configuration/settings.md`, 3). temper's stock
  rules attach both to the merge on every landing branch, and the forge
  judges each over every enforcing rule covering the merge's branch, as
  it holds the rules then:

  | Requirement | Its parameters | Met | Wait | Refuse | Guarded or observed |
  |---|---|---|---|---|---|
  | 5, the landing rules | none | each such rule's CI, gates and approvals are met, as 1, 3 and 4 above; or no rule enforces there | one waits, and none refuses | one refuses | guarded |
  | 6, up to date by the landing rules | none | each such rule asking for it is met, as 2 above; or none asks | the ancestry range unread or failed | the head lacks the base's tip: the range counts a commit (`../keeping-up.md`, 3) | observed, freshness 0 |

  The change keeps the rule and the criterion of its last wait or
  refusal, by the rule's scope and number and the criterion's kind
  (a gate's number, an approval's role); its panel shows it (`README.md`, 5), and
  its hold says it, in the forge's words.

## 3. The landing rules as settings

As `../../../protocol/configuration/settings.md` has them, the forge owns
three of temper's kinds of settings: the deployment's landing rules
(kind 1), each project's (kind 2), and the forge's charter (kind 3),
whose values it keeps in its own family, `Setting`
(`../../../protocol/store.md`, section 2), while jig keeps their states and
sizes and never reads them (jig's `domain/connectors.md`, section 17).

- **It judges them in the change's decision,** from its own state: a
  rule decodes, its pattern is a landing branch's within jig's pattern
  bounds, its gates and approvals within their bounds, and the forge's
  charter a charter's number (`../../../protocol/configuration/settings.md`, 2). A
  refusal names the item and what fails, and the whole change is
  refused with it.
- **The root checks approval eligibility** when a rule is set, after the
  forge's structural judgement, including when a proposal is accepted.
  For each project in the setting's scope, and each adopted writable
  repository of it whose branch names overlap the pattern, each of them
  for a project's pattern beginning at `branch`
  (`../../../protocol/configuration/settings.md`, 2), each enforcing
  approval needs at least its required count of distinct project members
  with the named role who count as people on **that repository's forge**,
  `forge_human` (`../adoption.md`): an account known on another forge of the
  project doesn't make a rule on this one satisfiable. The root's
  `eligible_people` counts them, joining the people's identities and roles
  with the forge's rule and link (`../adoption.md`, 6); a deployment login or display name
  supplies no forge identity. On a box, the person linked on a forge
  counts there with their own role, so `people: 1` naming it isn't refused,
  and `people: 2` is unless a seeded collaborator makes up the count. A
  refusal names the project, the forge by its host, the role, and the
  required and eligible counts: "project 1 role 0 on forge HOST requires 1
  distinct known forge people; currently 0 eligible." It then asks for
  human collaborators added on the forge, the project's policy giving the
  permission that role where needed and the repository adopted again,
  followed by a fresh proposal against the newest revision, or offers
  removal of the approval. It names no command or help page: whoever
  reads it may be in the browser, or an agent talking to someone who is
  (`../../agent/agents.md`, 1).
- **The playground's fake forge** has no account and no address, so no one
  can approve there: a rule with an enforcing approval covering one of its
  repositories is refused whatever the count, in its own words: "project 1:
  the playground's forge has no accounts, so no one can approve there. Name a
  branch pattern or a project that leaves its repository out, or remove the
  approval requirement."
  temper's retained forge writer is ineligible even with an existing role. No rule
  or accompanying settings become effective on refusal. Later membership
  changes can leave a held rule waiting; the merge still counts only
  people known on the review's forge (2).
- **It holds them** decoded, the deployment's rules and each project's,
  its rule 1 the stock's while the project sets none
  (`../../../protocol/configuration/stock.md`),
  within `landing_rules` per scope and `judge_projects` projects, keeps
  their values in its family `Setting`, and replaces a scope's whole
  when the core's word to keep a change's items reaches it, in the
  decision that committed them. Then each live change into a repository a
  changed scope covers is stepped: a gate a rule added is asked for, a
  gate it dropped is no longer waited on, and a change queued for a
  gate it now needs leaves its queue until it has its verdict (1).
- **It reads the newest:** every merge is judged by the rules held at
  its decision, whatever revision its change was made in.
- **A change keeps its charters and its LLM choices.** Its charters are
  read when the change is made: its producer's, the one `change` named
  (`README.md`, 1) or else the forge's change charter, under which its producing,
  repairs, resolutions and checks' gates run, and the forge's review
  charter, under which its agents' gates run. Its procedure's authority
  grants those charters' delegation alone, and is never rewritten. Its
  LLM choices, by role, are kept beside them and given to each delegate
  of the role. A change whose charter of a role the newest settings no
  longer offer, or whose choice for a role that charter's models at the
  newest no longer allow (jig's `domain/tasks/README.md`, section 4), is held
  when it next needs a delegate of that role, saying so; it is
  cancelled, or made again.

## 4. Open questions

- **Project-scoped rules on a real forge** (`gates.md`, 3): observe the
  every-project default, its override and reset, and branch-relative
  coverage through the existing process and live landing stories after
  landing. Default: those stories; focused and fuzzy checks on fakes
  establish no real-forge outcome.
