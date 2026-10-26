# Session 03: parties' `Create`, `Propose` and `Withdraw`; proposing made explicit

The second half of the engine's plumbing for parties (jig's
`docs/design/STATUS.md`, section 3, step 2). A party makes a batch of
tasks as an agent's `delegate` does, against the same checks with their
role and pool as the source of authority: agents', procedures' and person
tasks, their dependencies within the party's reach, the resources named
for them, a budget `AsAllowed`, tracked, labelled and flagged. A request
beyond the party's authority is refused, saying what it lacked and
whether a proposal of it could be accepted, and by whom; the party then
asks with `Propose` (a batch, or an amendment) and may `Withdraw` it. The
two paths that made proposals unasked go, each with the increment that
lands its explicit replacement. Nothing reaches the client's wire here:
jig 14 carries these operations to it, and jig 22 the documents.

## Read first

- jig's design: `domain/people.md`, sections 1, 4 and 5.1 (the table's
  `Create`, `Propose` and `Withdraw` rows; "A request refused";
  "Proposing is explicit"; "A proposal needs someone who may accept it";
  "A party's root has someone to escalate to"; "Where a party reaches");
  `domain/tasks.md`, sections 3, 4 ("A party's batch is an agent's",
  "Checked whole", "Made at once", "Resources named for it"), 9 and 12
  (the stories of what parties and agents ask alike);
  `domain/authority.md`, sections 7, 8.1, 8.4 and 9;
  `client/domain/contract.md`, 3.2 (`Create`, `Propose`, `Withdraw`; "Where
  a party may reach"), 3.5 and 4; `domain/testing.md`, sections 3 to 6;
  `protocol/store.md`, 5.1.
- The plan: `README.md` (Rules), `code-organization.md` (6, 7),
  `decisions.md` 3 and 9; session 02's file (its Choices and sketches).
- `findings/engine-parties-and-tasks.md`: section 1 (Create; Propose and
  Withdraw; Explicit proposing), section 2 ("Downstream"), sections 3, 4
  and 5.
- skein's `programming-model.md`, sections 4.4, 4.5, 6.3 and 10.3;
  `testing-strategy.md`, sections 2.1, 2.2 and 7.

## What the design asks

- **`Create`** (people.md, 5.1): "a batch | member, within their role's
  authority and their pool (authority.md, 8.4); its dependencies,
  references and inputs within their reach, and a holder for its
  escalations | its tasks, with their dependencies, checked whole and
  made at once (tasks.md, section 4), each with the fields it names
  (tasks.md, section 3); the party their requester and their funder; a
  budget given `AsAllowed` resolved, and what was given answered". The
  base has no such ask (`crates/jig-domain-people/src/boundary.rs:259-339`):
  parties create only through `StartChat` and `SetGoal`.
- **Explicit proposing** (people.md, 5.1): "A `Create` or an `Amend`
  beyond the party's authority is refused, naming what it lacked and
  whether a proposal could be accepted; it never becomes a proposal by
  itself". "A request refused says why, with its facts: ... the authority
  it lacked and by how much, and whether a proposal could be accepted,
  and by whom". The base turns a goal beyond the pool into `GoalProposed`
  (`crates/jig-domain-core/src/acts/bounds.rs:293-303`) and an amendment
  beyond the role into `AmendProposed`, whose proposer is the task
  (`acts/bounds.rs:376-394`; STATUS: "a party's amendment proposal names
  the task as its proposer").
- **`Propose`** (people.md, 5.1): "member; a batch or an amendment beyond
  their authority, of a kind a role the project's policy names may accept
  | a proposal of theirs, waiting for that role (tasks.md, section 9), its
  number and to whom answered". "A proposed batch giving `AsAllowed` is
  refused as not well formed" (authority.md, 7).
- **`Withdraw`**: "their own pending proposal | withdrawn". The base
  withdraws a task's proposal only (`crates/jig-domain-tasks/src/boundary.rs:1278`).

## Choices this plan makes

- **The people child's vocabulary for a batch**, `jig_domain_people::batch`:
  a `Member` mirrors the tasks child's `New` and the core's `Delegate`
  field by field, as `Amendment` mirrors the tasks child's. Its contract
  is a report, a verdict or a connector's result; an approval is a
  procedure's, never a party's (tasks.md, 3: "an approval ... of a state a
  procedure names"). Its spec is its words and parameters, and from 03.2
  its inputs, as fields of its own rather than the people child's `Spec`,
  so that each increment's type holds only what that increment makes. The
  types grow with the increments that use them.
- **A new people limit, `batch`**: the members of one `Create` or one
  proposed batch, the Small profile's 2 and the Standard profile's 4, as
  the tasks child's and the authority child's `batch` (which no check
  ties together at the base either). A member's bytes are within
  `amendment_bytes`, as an amendment's are (`amendment.rs:141`).
- **A batch's answer names its tasks by range**: `Created { first, last,
  given }`, since the core numbers a batch's tasks in order, as the core
  family's `Created { first, last }` record does.
- **"By whom"** is the lowest-numbered role of the project's policy that
  decides the proposal's kind (`acts/bounds.rs:11`, `recipient`), or
  nobody; `Proposed` answers that role. A request beyond the project's
  ceiling or the deployment's rules is answered `Accepter::Nobody`.
- **A party's proposal stays apart from a task's**: the tasks child's
  `PersonProposal` (one pending per party, at the base) holds a batch or
  an amendment in place of its goal; a task's `Proposal` (one per task)
  is unchanged.
- **References are not named by a `Create`**: `New` and an agent's
  `Delegate` carry none, as people.md, 5.1 says: "a batch gives its
  tasks no references" (ruling 156).
- **The wire stays v8** (jig 14 changes its requests, jig 22 its
  documents). The clients' component maps `Accepter::Role(_)` onto v8's
  `proposable: Some(true)` and `Accepter::Nobody` onto `Some(false)`; it
  answers its translation's `problem()` for the outcomes `Created`,
  `Proposed` and `Withdrawn`, which no v8 ask can lead to, and for the
  document of a party's proposal of an amendment, which only `Propose`
  makes. jig 14 maps the outcomes, jig 22 the document.
- **The reference UI's amendment beyond its party's role keeps working**
  from 03.5 to jig 14.5. Its confirmation sends v8's `Ask::Amend` for an
  amendment offered as proposable (`crates/jig-domain-client/src/tests/confirmation.rs:311`
  at the base), and v8's `Amend` cannot say which it is: only the engine
  judges. So the clients' component, and nothing in the engine, turns a
  v8 `Ask::Amend` that the engine refuses as proposable (`Authority {
  proposable: Some(Accepter::Role(_)) }`) into the party's `Propose` of
  that amendment, and answers v8's `AmendProposed`; jig 14.5 moves the
  confirmation onto `Propose` and removes it (03.5). This is
  transitional code with its end named (code-organization.md, 6).
- **The reference UI loses its implicit goal proposal** at 03.4, as the
  design asks: from then on its goal form beyond the pool shows a refusal
  that names who could accept, until it proposes through `Propose` (jig 14
  to 16). The client's joint story that relied on it asserts that refusal
  (README, Rules, "a removed mechanism's tests, moved to its
  replacement").

## Rules that bind it

- **Format versions** (README, Rules): the tasks and people families are
  v3 from jig 02.1 and 02.3; each increment here edits v3's schema in
  place and regenerates (`SKEIN_CODEGEN_WRITE=1 cargo nextest run
  -p jig-store-codecs-world --test drift`); never a v2 file.
- **Files over 800 lines get at most 40 more lines each**
  (code-organization.md, 7; `git diff --numstat origin/main...HEAD --
  <file>`, first column): here `crates/jig-domain-people/src/domain.rs`,
  `crates/jig-domain-tasks/src/{boundary.rs, domain.rs,
  stored.rs, proposals.rs}`, `crates/jig-domain-authority/src/check.rs`,
  `crates/jig-domain-core/src/{effects.rs, routing/tasks.rs,
  reads/rows.rs, documents/objects.rs}` and the test files
  `crates/jig-domain-people/src/tests.rs`, `crates/jig-domain-core/src/tests.rs`,
  `tests/core/tests/stories.rs`, `testing/jig-test-domain/src/routes.rs`,
  `tests/people/src/world.rs`, `tests/tasks/src/world.rs`. New code goes
  in the new modules this file names.
- **Nothing new is added to the shapes of work**, and jig's files never
  name the repository downstream of jig (session 02's Rules, with its
  check command). `people::Ask` is matched exhaustively downstream
  (findings, 2, "Downstream"): its changes are absorbed by the downstream
  repository's repin, and no increment here edits that repository.

## Increments

### 03.1 Parties' `Create`: a batch of agents' tasks
Scope: people.md, 5.1 (`Create`; "A party's root has someone to escalate
to"); tasks.md, 4 ("A party's batch is an agent's", "Checked whole": "no
cycle", "Made at once"; "A batch that fails a check other than authority
is refused, saying which task and why"); contract.md, 3.2 and 4 (*not
well formed*); authority.md, 8.4 ("A party's
batch is 8.1's, with the party as its creator"). A party's batch of
agents' tasks, depending on each other, with spec, contract, authority,
budget, wake policy, tracked and priority, label and flags, made at once
as the party's roots.
Needs: jig 02, its last landing (the session's start): its
`tasks::Flags` on `New` (02.1), `authority::Domain::escalations_held` and
`people::Refusal::NoEscalationHolder` in people v3 (02.3), and 02.4's
`authority::as_allowed`, which 3.2 uses.
Files:
- `crates/jig-domain-people/src/batch.rs` (new: `Member`, `TaskExecutor`,
  `Contract`, `Verdict`, `FieldRule`, `Flags`, `Retention`,
  `member_bytes`), `lib.rs`, `boundary.rs` (`Ask::Create`,
  `Outcome::Created`), `refusals.rs` (`Refusal::NotWellFormed`,
  `Malformed::Cycle`), `limits.rs` (`batch`, and the worst case of
  a `Create` among pending and answered keys), `domain.rs` (at most 40:
  an arm in each of its matches over `Ask` (search S9) and over
  `Outcome`), `capacity.rs`, `words.rs`.
- `crates/jig-protocol-config/src/profile/{small.rs, standard.rs}` (people
  `batch=2` and `4`, with the doc line), `overrides.rs` (:174 and :195,
  `people.batch`); `src/tests/overrides.rs`, `SMALL` (:131; people's rows
  :171-189), gains `(b"people.batch", 2)`, as jig 04.1's `grace` row did.
  When a `Create` raises people's worst case, the values derived from it
  move too: `JOURNAL` and `STORE_KV` in `profile/small.rs` (:235, :305) and
  `standard.rs` (:141, :144), and `src/tests/domain.rs:169` (`What::Memory
  { over: 12_090_553_463 }` at jig `e1b483760`); refresh any further derived value the gate
  names, and list it in the commit body.
- `crates/jig-domain-core/src/routing/create.rs` (new: the route),
  `routing/mod.rs` (`Creation` at :109 holds a batch, as
  `DelegateCreation` at :119 does; chats and goals give a batch of one),
  `routing/parties.rs` (`begin_creation` :248 and `created_holdings` :293
  ask each connector for each member's holdings and make the batch),
  `routing/people.rs` (the route's arm), `acts/requests.rs`,
  `person_task.rs`, `roles.rs`, `routing/party_controls.rs` (arms, search
  S9), `routing/application_routes.rs`, `routing/tasks.rs` (arms for
  `Created`, search S10).
- `crates/jig-protocol-store`: `schema/people-v3.schema` (`record Member`
  and its parts, `record AskCreate { project: u32 batch: list 8 Member }`,
  `record OutcomeCreated { first: u64 last: u64 }`, `enum Malformed {
  cycle }`, `record RefusalNotWellFormed { at: u32 why: Malformed }`),
  regenerated with its goldens; `src/codec/people/asks.rs` (`Create`,
  both ways, in a new module `src/codec/people/batch.rs` for `Member`),
  `src/codec/people/outcomes.rs`, `src/codec/people/outcomes/facts.rs`;
  the codec's limit from people `batch` (`limits`, `src/codec/people.rs:25`).
- `crates/jig-protocol-engine-clients/src/translate/answers.rs` and
  `refusals.rs` (Choices). Arms in the other exhaustive matches over
  `people::Ask`, `Outcome` and `Refusal` (searches S8 to S10):
  `testing/jig-test-domain/src/{routes.rs, application_facts.rs}`,
  `tests/core/src/{peers.rs, referee/offers.rs}`,
  `tests/client/src/joint/wire/{outcomes.rs, refusals.rs}` (`outcome`,
  `outcomes.rs:17`, has no problem variant: its `Created`, and from 03.4
  its `Proposed` and `Withdrawn`, arms are `unreachable!("no v8 ask leads
  to <the outcome>")`, as the clients' component answers `problem()` for
  them, Choices),
  `tests/people/src/{referee.rs, world.rs}`.
- `tests/core/src/stories/offers.rs` (`world_with`, beside `world` at :21;
  Test shape).
- Users of `people::Limits` (11 files, search S7), edited to the new form.
  The downstream repository's repin absorbs the new ask, outcome, refusal
  and limit (search S18: 28 files match `people::Ask::`).
Shape:
- `people::Member { executor, words, parameters, contract, authority,
  wake, dependencies: Box<[u32]>, tracked, label, flags }` here, with
  `TaskExecutor::Agent { charter }` only; 03.2 adds the rest of sketch 1.
  `Ask::Create { project, batch: Box<[Member]> }`; `Outcome::Created {
  first, last }`; `Refusal::NotWellFormed { at: u32, why: Malformed }`,
  `Malformed::Cycle`.
- The people child admits a `Create` like `SetGoal`, refusing no role:
  `admission` (`domain.rs:1838`) routes the ask, and the core refuses by
  role (step 0), as it does a goal (`acts/bounds.rs:29`, :239). The people
  child checks 1 to `batch` members, each member's words within `words`
  and its `member_bytes` within `amendment_bytes`, else `Limit { which:
  Empty | Batch | Words }`.
- `routing::create::create_request(core, env, out, work, request, person,
  project, role, batch)`, in order, each refusal naming the member's
  position where it has one:
  0. `Role { held }` unless the party's role allows `RequestKind::Create`,
     as `goal_start` refuses (`acts/bounds.rs:236-241`): an observer's
     `Create` is refused;
  1. `NotWellFormed { at, why: Cycle }` when the batch's own dependencies
     (positions) close a cycle, `at` the lowest position on it; a cycle
     through a live task stays the tasks child's `Limit { which: Cycle }`;
  2. `NoEscalationHolder` unless `escalations_held(project, role)`;
  3. `NotOffered` for a charter the deployment lacks;
  4. the authority check: `PersonRequest::Create` of each member's
     `Delegate`, `check_request`, refused by `request_refusal` unless
     `Allow`, and the project's period checked as `chat_admit` checks it
     (`roles.rs:107-112`);
  5. the members numbered in order (`crate::fresh`), each dependency
     `Batch(at)` resolved to `at`'s number, each `New` funded by the
     party's pool for the current period (`goal_open_pool` opens it), and
     the batch made as `Creation` makes a goal: holdings asked of each
     connector, then `tasks::Event::Make { creator: Party::Person(person)
     }`, answered `Created { first, last }`; the tasks child's refusals
     answered through `person_task_refusal` (`routing/refusals.rs:11`),
     as a person task's are (its `task_answer`, :23, is private).
- The test mirrors of `Refusal`, `testing/jig-test-connector/src/application_core.rs:6`
  and `testing/jig-fake-parties/src/facts.rs:6`, stay as they are, with
  their codecs and schema: their mappers, `refusal`
  (`testing/jig-test-domain/src/application_facts.rs:6`) and
  `party_refusal` (`tests/core/src/peers.rs:600`), translate
  `NotWellFormed { why: Cycle, .. }` onto the mirror's nearest refusal,
  `Limit { which: Cycle }`. 03.2's and 03.4's `Malformed` variants are
  translated the same way, each onto the nearest refusal the landing's
  report names.
Test shape:
- `crates/jig-domain-people/src/tests/create.rs` (new, a module of
  `tests.rs`; fixtures of `tests.rs`: `LIMITS` :10, whose `batch` this
  increment sets to 2, the Small profile's; `Test::new` :248,
  `Test::member` :310, `Test::request` :305, `reply` :316; and a helper of
  this file, `member(words)`, an agent's task of charter 1 with a report
  contract): `a_members_create_routes_once_to_the_core`,
  `an_observers_create_is_refused_by_role` (the ask routed, then the
  root's `Event::Decided { request, outcome: Outcome::Refused(Refusal::Role
  { held: Some(Role::Observer) }) }` replayed, as `tests.rs:478` does),
  `an_empty_create_is_refused_as_empty`,
  `a_create_past_its_batch_limit_is_refused`,
  `a_create_whose_words_pass_their_bound_is_refused`. Reference:

  ```rust
  #[test]
  fn a_create_past_its_batch_limit_is_refused() {
      let mut test = Test::new(LIMITS);
      test.member();
      let batch = Box::new([member(b"one"), member(b"two"), member(b"three")]);
      let rows = test.request(10, 1, Ask::Create { project: 1, batch });
      assert_eq!(reply(&rows), Reply::Outcome(Outcome::Refused(Refusal::Limit { which: LimitKind::Batch })));
  }
  ```
- `crates/jig-protocol-store`: `fixtures::people_asks()`
  (`src/tests/fixtures.rs:418`) gains a `Create` of two members, so
  `each_kind_of_ask_reads_back` covers it; `outcome_values()` and
  `refusal_values()` (`src/tests/fixtures/people_outcomes.rs:153`, :123)
  gain `Created` and `NotWellFormed`, so `people_outcomes_read_back_as_written`
  does; the drift test regenerates v3.
- `tests/core/tests/parties.rs` (new, the joint world; the world
  `jig_core_world::stories::offers::world_with(seed, Some(people::Role::Member),
  edit)`, a new function of `tests/core/src/stories/offers.rs`, since
  `world` (:21) builds its world itself and its limits cannot be set
  afterwards: `edit` changes the fixture's limits before the world is
  configured, `world` is `world_with` with no edit, and before
  `World::configured` `world_with` raises the journal's `writes` and
  `held` to the edited core's `core::room_max`, as
  `tests/core/src/world.rs:178-187` sizes them for the world's own
  limits, else `root::worst_case` refuses the journal
  (`testing/jig-test-domain/src/limits.rs:30-31`) and the world's heap
  observer panics, "testing root bound fits"; the edit sets
  `limits.core.tasks.batch`, `limits.core.authority.batch` and
  `limits.core.people.batch` to 2, the Small profile's, beside the world's 1, and `limits.core.tasks.tasks`
  and `project_tasks` to 8, the Small profile's, beside the world's 2, the
  reason being a batch of two beside the world's chat): the Proof's test,
  and `a_partys_batch_with_a_dependency_cycle_is_refused_naming_its_member`,
  `a_partys_batch_is_refused_while_nobody_holds_its_escalations` (on
  jig 02.3's `no_holder_fixture`, `tests/core/src/stories/offers.rs:93`,
  built as `tests/core/tests/fields.rs:41` builds its member world),
  `a_partys_batch_beyond_their_pool_is_refused_naming_the_budget`,
  `a_partys_batch_naming_an_unknown_charter_is_not_offered`. Reference,
  the Proof's:

  ```rust
  #[test]
  fn a_partys_batch_of_two_agents_tasks_is_made_as_their_roots() {
      let mut world = batch_world(301);
      let mut first = member(b"gather the facts", 20);
      first.tracked = Some(3);
      first.label = 41;
      let mut second = member(b"write the summary", 30);
      second.label = 42;
      second.dependencies = Box::new([0]);
      ask(&mut world, 1, people::Ask::Create { project: 1, batch: Box::new([first, second]) });
      assert_eq!(
          world.people_answers.last(),
          Some(&people::Reply::Outcome(people::Outcome::Created { first: 2, last: 3 }))
      );
      let party = member_party(&world);
      assert_eq!(live(&world, 2).requester, tasks::Party::Person(party));
      assert_eq!((live(&world, 2).tracked, live(&world, 2).label, live(&world, 2).numbers.budget), (Some(3), 41, 20));
      assert_eq!((live(&world, 3).label, live(&world, 3).dependencies.as_ref()), (42, [2_u64].as_slice()));
      assert_eq!(live(&world, 3).funder, tasks::Funder::Pool { project: 1, person: party, period: 1 });
  }
  ```

  (`batch_world`, `member(words, spend)` (an agent's task of charter 1,
  report contract, `Flags` plain), `ask` and `member_party` are helpers
  of this file; `live` is `jig_core_world::fixtures::live_row`
  (`tests/core/src/fixtures.rs:24`), which reads the store's row, not a
  new helper; README, "A test's helpers once".)
- Ceilings: each unit test under 20 ms, each world test under 0.5 s.
Proof: joint layer, `tests/core/tests/parties.rs`,
`a_partys_batch_of_two_agents_tasks_is_made_as_their_roots`: a member's
`Create` of two agents' tasks, the second depending on the first, is
answered `Created { first: 2, last: 3 }` once durable, and the store holds
both as the member's roots, funded by their pool, with the tracked
priority, labels and flags asked. Under 0.5 s. Run once, `cargo
nextest run -p jig-core-world -E
'test(a_partys_batch_of_two_agents_tasks_is_made_as_their_roots)'`; it
passes, and nothing else is wanted.
Don't: make a party's batch through anything but the tasks child's
`Make`; add an act to the offers' `Act` (documents' offers are jig 22's);
accept procedures, person tasks, dependencies outside the batch, inputs,
named resources or `AsAllowed` (03.2).
Size: at most 300 lines (a codec increment).
Removes: `Creation`'s single `new` (now a batch).

### 03.2 `Create`'s procedures, person tasks, reach, named resources and `AsAllowed`
Scope: tasks.md, 4 ("Dependencies", "A budget as `AsAllowed`", "Resources
named for it", "Procedure tasks' parameters go to their connector in the
same decision", "Checked whole": "at most one task giving `AsAllowed`",
"the conversation flag on an agent's task only"); people.md, 5.1 ("Where
a party reaches"), 8 (person tasks); authority.md, 7 and 8.4 (its
delegation's kinds); contract.md, 3.2 ("each `AsAllowed` budget with what
it was given") and 4 (*not well formed*; *authority*: "a kind of task").
Needs: nothing beyond 3.1.
Files:
- `crates/jig-domain-people/src/batch.rs` (`TaskExecutor::{Procedure,
  Person}`, `Contract::Change`, `Budget`, `Dependency`, `Member.inputs`,
  `Member.named`),
  `boundary.rs` (`Outcome::Created.given`, `Given`), `refusals.rs`
  (`Malformed::{AsAllowedTwice, Conversation}`; `Lack::TaskKind {
  executor: Option<TaskExecutor> }`, whose other users pass `None`).
- `crates/jig-domain-core/src/routing/create.rs`; `delegation.rs` (the
  person executor through `delegation_executor`, :165); `roles.rs` (a new
  `Core::reaches(person, role, project, task, depth) -> bool`, the
  standing `amendment_admit` computes at `acts/bounds.rs:320-327`, which
  then calls it).
- `crates/jig-protocol-store`: `schema/people-v3.schema` in place; the
  codec of `Member`'s new parts, `Created.given`, the new `Malformed` and
  `LackTaskKind`.
- The engine-clients' and the tests' arms as 3.1's, for the new variants;
  `crates/jig-protocol-engine-clients/src/translate/refusals.rs` leaves
  v8's `LackingTaskKind.executor` `None`, as at the base (jig 14 fills
  it).
- The other users of `Lack::TaskKind`, passing `None` or matching `{ .. }`:
  `crates/jig-domain-core/src/authority_facts.rs` (:42, :69),
  `crates/jig-domain-core/src/routing/refusals.rs:47`,
  `crates/jig-domain-people/src/tests.rs:1178`,
  `crates/jig-protocol-engine-clients/src/tests/fixtures/answers/lacks.rs:28`,
  `tests/people/src/referee.rs:121`.
Shape: sketch 1, with each field and variant it marks 03.2. In
`create_request`, beside 3.1's steps:
- first, `NotWellFormed { at, why: Conversation }` for the conversation
  flag on a member whose executor is not an agent's, and `NotWellFormed {
  at, why: AsAllowedTwice }` at the second member giving
  `Budget::AsAllowed`;
- a procedure's executor offered as `goal_start` checks it
  (`acts/bounds.rs:245-259`), `NotOffered` otherwise, and its parameters
  reach its connector in the decision that makes the batch, as a
  procedure goal's do (the goal of `tests/core/tests/stories/procedure_goals.rs`);
  a person executor mapped by `delegation_executor`; each executor checked
  by the authority check's delegation kinds, and a refusal for a kind the
  role lacks (`Authority { lacking: TaskKind { .. } }`) names the first
  member's executor the role's `delegation.kinds` lacks, in `executor`;
- `Dependency::Existing(task)` and each `inputs` entry must be a task
  the party reaches, `Refusal::Unknown` otherwise: live in their tree
  (`Core::person_tree`, `roles.rs:197`), or any task of the project for
  an owner or a maintainer, or a policy role allowed `Amend`; inputs are
  loaded as an agent's delegate's are (`Core::delegate_inputs`,
  `delegation.rs:130`, then the root's stub loads), through a load of
  its own (ruling 156): a core `Request` keyed by the party request's
  `Token`, with its person, project and its bounded unique input task
  numbers, as `Request::HistoricalPersonTask` is keyed, and a matching
  `Event` carrying the rows the root loaded, as
  `Event::HistoricalPersonTask` carries its `row`: each input's ended
  `TaskRecord`, its tree's root's where that root has left live memory
  (`None` for a row not found), and `busy` (ruling 157). The core judges
  reach from them and builds the stubs, or refuses as
  `DelegateInputFailed` does; no validator of the core's runs in a root
  (root.md, 11). The shared historical transport loads the rows one
  `Get` each, as it loads one today. The core keeps the checked creation
  until it returns. Each root that answers
  `HistoricalPersonTask` answers it the same way (`git -C
  ~/src/rust/jig grep -l 'HistoricalPersonTask' -- '*.rs'` names them:
  `examples/ops/domain/src/route.rs`, `testing/jig-test-service/src/route.rs`,
  `testing/jig-test-domain/src/routes.rs`, `testing/jig-test-store/src/route.rs`,
  `tests/client-protocol/src/engine.rs`, `tests/core/src/world.rs`,
  `tests/ops/src/joint/engine.rs` among them, at jig `840274718`); temper's
  root follows at its first repin past this increment's landing, as the
  README's "What a temper repin repairs" says;
- `named` grants narrowed with each member's number by
  `crate::resolved_delegate_authority` (`delegation.rs:210`), as an
  agent's are (`routing/run_delegation.rs:91-99`);
- `AsAllowed`: the named amounts summed, `authority::as_allowed(&core.authority,
  &ask, named, cap)` (jig 02.4); zero refused `Authority { lacking:
  Budget { needs: named.saturating_add(1), left: <what the pool has left
  under the role> }, proposable: None }`; otherwise that member's `authority.spend`
  and `numbers.budget` are set to it before the check, and the answer is
  `Created { first, last, given: Some(Given { task, amount }) }`.
Test shape:
- `tests/core/tests/parties.rs` (3.1's file and helpers): the Proof's test
  and `a_budget_as_allowed_beside_named_amounts_is_named_in_the_answer`,
  `a_budget_as_allowed_that_comes_to_nothing_is_refused`,
  `a_batch_where_two_tasks_give_as_allowed_is_not_well_formed`,
  `a_partys_batch_with_a_conversation_on_a_procedure_is_not_well_formed`,
  `a_partys_procedure_task_is_offered_and_its_parameters_reach_its_connector`,
  `a_member_whose_charter_the_role_lacks_is_refused_naming_it`,
  `a_members_dependency_outside_their_tree_is_unknown`,
  `a_members_input_outside_their_tree_is_unknown`,
  `a_maintainers_dependency_on_any_project_task_is_made`,
  `a_named_resource_is_narrowed_to_the_new_tasks_number`. The procedure
  test takes its connector's fixture from `tests/core/tests/stories/procedure_goals.rs`
  (`fixture(seed, false)`, `tests/core/src/effects/constructors.rs:15`,
  whose first connector has procedure 1). Reference:

  ```rust
  #[test]
  fn a_budget_as_allowed_beside_named_amounts_is_named_in_the_answer() {
      let mut world = batch_world(302);
      let named = member(b"named", 30);
      let mut allowed = member(b"as allowed", 0);
      allowed.budget = people::Budget::AsAllowed { cap: None };
      ask(&mut world, 2, people::Ask::Create { project: 1, batch: Box::new([named, allowed]) });
      assert_eq!(
          world.people_answers.last(),
          Some(&people::Reply::Outcome(people::Outcome::Created {
              first: 2,
              last: 3,
              given: Some(people::Given { task: 3, amount: 70 }),
          }))
      );
  }
  ```

  (`batch_world`'s member role has spend 100 and its pool 1000, so 100
  less the named 30 is given.)
- `crates/jig-protocol-store`: the fixtures of 3.1 gain a member of each
  executor and dependency, a `Created` with `given`, `AsAllowedTwice`,
  `Conversation`, and a `TaskKind` naming an executor in `lack_values()`
  (`src/tests/fixtures/people_outcomes.rs:60`).
- Ceilings: each world test under 0.5 s.
Proof: joint layer, `tests/core/tests/parties.rs`,
`a_partys_batch_of_an_agents_task_and_a_person_task_is_made` (tasks.md,
12: "a party's batch of an agent's task and a person task, made as an
agent's would be"): a member's batch of an agent's task and a person task
addressed to their own role, the person task depending on the agent's, is
made; the person task waits for its dependency, then shows in the
member's inbox as a task they carry out. Under 0.5 s. Run once, `cargo nextest run -p jig-core-world -E
'test(a_partys_batch_of_an_agents_task_and_a_person_task_is_made)'`; it
passes, and nothing else is wanted.
Don't: let a party's member hand down holds (a party holds none); accept
`AsAllowed` in a proposed batch (03.4 refuses it).
Size: at most 290 lines (a codec increment), and the roots' routing of
the inputs' load beside them, aim for at most 80 lines; past it, report
why (ruling 156).
Removes: nothing.

### 03.3 Refusals say who could accept a proposal
Scope: people.md, 5.1 ("A request refused ... whether a proposal could be
accepted, and by whom"); authority.md, 8.1 ("Beyond any of these, the
answer is propose; beyond the project's ceiling, refuse" ... "Propose is
an answer, not a proposal"), 8.4; contract.md, 3.5 and 4.
Needs: nothing beyond 3.2.
Files:
- `crates/jig-domain-people/src/refusals.rs` (`Accepter`;
  `Refusal::Authority.proposable`).
- `crates/jig-domain-authority/src/check.rs` (at most 40: `batch` at
  :129, whose `task_answer` is `Propose` for `Source::Role` too, and
  `give` at :650, `Propose` for a shortage against the role or the pool).
- `crates/jig-domain-core/src/authority_facts.rs` (:12,
  `request_refusal` names the accepter), `acts/bounds.rs` (`unproposable`
  :404; `amendment_admit`'s rewrite of a role shortage to `Propose`,
  :376-394, goes: the check answers it), and the literals of
  `proposable: Some(false)`, `Some(true)` and `None` in the core, the
  people child and their tests (search S14, the files of crates that
  depend on `jig-domain-people`).
- `crates/jig-protocol-store`: `schema/people-v3.schema` in place (`enum
  Accepter { nobody, role: AccepterRole }`; `RefusalAuthority.proposable:
  option Accepter`), `src/codec/people/outcomes/facts.rs`.
- `crates/jig-protocol-engine-clients/src/translate/refusals.rs` (Choices);
  the tests' arms (`tests/core/src/peers.rs`, `tests/client/src/joint/wire/refusals.rs`,
  `testing/jig-test-domain/src/application_facts.rs`); the test mirrors
  `testing/jig-fake-parties/src/facts.rs` and
  `testing/jig-test-connector/src/application_core.rs` stay as they are
  (Shape).
Shape: `people::Accepter { Nobody, Role(u32) }`; `Refusal::Authority {
lacking, proposable: Option<Accepter> }`, `None` still making no claim. For
a party's request whose authority check answers `Propose`, the refusal
names `Some(Accepter::Role(n))`, `n` the lowest-numbered role of
`recipient(core, project, kind)`, or `Some(Accepter::Nobody)` when the
policy names none; for one that answers `Refuse` on an authority finding,
`Some(Accepter::Nobody)`. The callers that read a party's `Answer` (`rg -n
'Answer::(Refuse|Propose)' crates/jig-domain-core/src`) treat any answer
but `Allow` as refused, as they do at the base; the search is run once and
its lines listed in the landing's report. The test mirrors of
`Refusal` keep `proposable: Option<bool>`, as in 3.1: their mappers,
`refusal` (`testing/jig-test-domain/src/application_facts.rs:6`) and
`party_refusal` (`tests/core/src/peers.rs:600`), translate
`Some(Accepter::Role(_))` onto `Some(true)`, `Some(Accepter::Nobody)` onto
`Some(false)` and `None` onto `None`, as the clients' component maps
them onto v8 (Choices).
Test shape:
- `crates/jig-domain-authority/src/tests_party.rs` (jig 02.3's file and
  fixtures): `a_partys_batch_beyond_their_role_answers_propose`,
  `a_partys_batch_beyond_the_project_ceiling_answers_refuse`,
  `a_partys_widening_beyond_their_role_answers_propose`,
  `one_batch_answers_alike_under_a_task_and_under_a_party` (testing.md, 3:
  "one batch under a task and under a party answering alike": the same
  delegates checked by `check_batch` under a task's authority and by
  `check_request` under a role of that authority and a pool of the same
  numbers give the same answer, their findings differing only in their
  source).
- `tests/core/tests/parties.rs`: the Proof's test and
  `a_partys_batch_beyond_the_project_ceiling_says_nobody_could_accept`,
  `a_partys_batch_no_role_may_accept_says_nobody`. Fixture for the last:
  `no_acceptor_fixture` (`tests/core/src/stories/offers.rs:72`).
- `crates/jig-protocol-store`: `lack_values()`/`refusal_values()` gain an
  authority refusal with each accepter.
- Ceilings: unit under 20 ms, world under 0.5 s.
Proof: joint layer, `tests/core/tests/parties.rs`,
`a_partys_batch_beyond_their_role_is_refused_naming_who_may_accept`: a
member's `Create` of a task whose budget passes their role's spend but
not the project's ceiling is refused `Authority { lacking: Budget { needs:
150, left: 100 }, proposable: Some(Accepter::Role(0)) }`, and nothing of it
is made. Under 0.5 s. Run once, `cargo nextest run -p
jig-core-world -E
'test(a_partys_batch_beyond_their_role_is_refused_naming_who_may_accept)'`;
it passes, and nothing else is wanted.
Don't: turn a refusal into a proposal; change a task's `propose`
answers (an agent's tool vocabulary is jig 06's).
Size: at most 120 lines.
Removes: `amendment_admit`'s `role_shortage` rewrite.

### 03.4 `Propose` of a batch, and `Withdraw`; the goal's implicit proposal goes
Scope: people.md, 5.1 (`Propose`, `Withdraw`; "A proposal needs someone
who may accept it"; "Proposing is explicit"); tasks.md, 9 ("Proposing is
asked for", "Admitted once", "Where they wait", "A proposal nobody may
accept is refused", "Deciding", "Withdrawn": "when the state it was made
against has changed so that it no longer applies (a batch naming a task
since ended)"); authority.md, 7 (a proposed batch giving `AsAllowed` is
not well formed) and 9 ("Never what the project's policy or the
deployment's rules refuse"); people.md, 5.1 ("Two parties deciding one
thing: the first commit decides; the second is told it was decided, by
whom, and how"); contract.md, 3.5 and 4. A
party proposes a batch, which waits for the policy's role for batches and
is made as the party's roots, funded by the accepter, when accepted; a
rejection reaches the party; the party may withdraw it. `SetGoal` beyond
the party's pool is refused, saying by whom it could be accepted, in
place of `GoalProposed`.
Needs: nothing beyond 3.3.
Files:
- From 23.1 (ruling 182): `testing/jig-conformance/src/referee.rs`
  (`Snapshot.party_proposals: BTreeMap<u64, u64>`, each pending party
  proposal's number and its proposing party; the accepted-task check
  takes an accepted party batch's tasks from it, its source-ceiling check
  against `accepted.by` unchanged), the three adapters that build
  `Snapshot` (`tests/core/src/observations.rs`, `tests/conformance/src/lib.rs`,
  `tests/ops/src/conformance/observations.rs`: the `Pending`
  `tasks::Stored::PersonProposal` rows alone), a referee case for it in
  `tests/core/tests/referee.rs`; and the party `ProposalDecisionRecord`'s
  `created` range for an accepted batch
  (`crates/jig-domain-core/src/proposals.rs:354`, `created: None` at
  `983862753`). The asks and their credits stay 23.1's.
- `crates/jig-domain-people/src/batch.rs` (`Action::Batch`),
  `boundary.rs` (`Ask::{Propose, Withdraw}`, `Outcome::{Proposed,
  Withdrawn}`; `Outcome::GoalProposed` goes), `refusals.rs`
  (`Malformed::ProposedAsAllowed`), `domain.rs` (at most 40), `capacity.rs`,
  `words.rs` (the reason within `words`).
- The proposer's update for a decided party proposal (ruling 175):
  `crates/jig-domain-tasks/src/index.rs` (`UpdateObject`, :28, a
  variant for it), `crates/jig-protocol-store/schema/tasks-v3.schema`,
  `src/codec/tasks/index.rs`, `src/keys/tasks.rs`, `src/reads/indexed.rs`
  (the join to `Stored::DecidedProposal` with its
  `CoreRecord::ProposalDecision`), the generated family and goldens;
  `crates/jig-domain-core/src/{indexes.rs, reads/indexed.rs, forget.rs}`;
  and the exhaustive matches over `UpdateObject` the compiler names
  (jig only: temper names none), an arm each.
- `crates/jig-domain-tasks/src/proposals.rs` (at most 40: `PersonProposal`
  at :143 holds `action` and `reason` in place of `goal`;
  `valid_person_proposal` :160 checks each member as a root's;
  `decide_person` :262 for a batch), a new module
  `crates/jig-domain-tasks/src/person_proposals.rs` for the withdrawal
  and the action's checks, `boundary.rs` (`Event::WithdrawPersonProposal`,
  at most 40), `stored.rs` (restore, at most 40), `closing.rs` (`end_task`,
  :295: a party's pending proposals naming the ended task are
  withdrawn in its decision), and its other users in the tasks child
  (search S13: `capacity.rs`, `domain.rs`, `owned.rs`).
- `crates/jig-domain-core/src/routing/propose.rs` (new: `Propose` and
  `Withdraw`), `routing/party_decisions.rs` (`goal_decide`, :16-97, for a
  party's proposal of a batch), `routing/parties.rs` (the goal's
  `CreationKind::Goal { direct: false }` and `GoalRoute::Proposing` go:
  `goal_start` refuses), `run.rs` (`GoalRoute::Proposing`, :397, goes),
  `acts/bounds.rs` (`goal_start` :221 refuses
  beyond the pool with 3.3's refusal), and the users of `PersonProposal.goal`
  (search S13: in the core `documents/objects.rs`, `acts/objects/decisions.rs`,
  `reads/rows.rs`, `reads/vocabulary.rs`, `proposals.rs`, `sibling_routes.rs`,
  `routing/tasks.rs`, and `goals.rs` (:104, :117), `inbox/census.rs:51`,
  `stored.rs:327`; in `jig-protocol-engine-clients`
  `translate/{deployment.rs, objects.rs, objects/historical.rs}`, and
  `translate/answers.rs:81-84`, whose `p::Outcome::GoalProposed` arm goes
  with `GoalProposed`), and
  the tests among search S13's files (`tests/tasks/{src/world.rs,
  tests/person.rs}`, `crates/jig-protocol-store/src/tests/{codec/phase.rs,
  fixtures.rs, fixtures/reads.rs}`, `tests/client/src/joint/{truth.rs,
  wire/deployment.rs, wire/deployment/tests.rs, wire/historical.rs,
  wire/historical/tests.rs}`), edited to the new form.
- `crates/jig-protocol-store`: `schema/tasks-v3.schema` and
  `schema/people-v3.schema` in place (`PersonProposal.action:
  PersonAction`, `enum PersonAction { batch: ProposedBatch, amend: boxed
  ProposedAmendment }`, `PersonProposalState.withdrawn`; `AskPropose`,
  `AskWithdraw`, `OutcomeProposed`, `OutcomeWithdrawn`), the codecs
  `src/codec/tasks/{proposals.rs, carried.rs}` and `src/codec/people/{asks.rs,
  outcomes.rs}`.
- Tests that relied on `GoalProposed`, moved to `Propose` (search S11):
  `tests/core/tests/stories/{offers.rs, procedure_goals.rs, historical.rs,
  watches.rs}`, `tests/core/tests/stories.rs` (`a_goal_with_a_deadline_is_held_past_it`,
  :1313), `tests/core/src/{peers.rs, referee/offers.rs}`,
  `crates/jig-protocol-engine-clients/src/tests/fixtures/answers/outcomes.rs`
  (`GoalProposed`'s row, :37, goes); the client's joint
  story `tests/client/src/joint/role_stories/proposed_story.rs`, which
  asserts the refusal (Choices). Arms as 3.1's for the new variants.
  (`tests/people/{src/world.rs, tests/simulation.rs}` match search S11 for
  `AmendProposed` only: 03.5's.)
Shape:
- `people::Action::Batch(Box<[Member]>)`; `Ask::Propose { project, action,
  reason }`, `Ask::Withdraw { project, proposal }`; `Outcome::Proposed {
  proposal, to }`, `Outcome::Withdrawn { proposal }`.
- `tasks::PersonProposal { number, proposer, project, action:
  PersonAction, reason, state }`, `PersonAction::Batch(Box<[New]>)`,
  `PersonProposalState::Withdrawn { at }`,
  `Event::WithdrawPersonProposal { reply_to, proposer, proposal }`,
  answered `PersonProposalDecided` with `ProposalOutcome::Withdrawn`, or
  refused `Unknown` (sketch 2).
- `routing::propose::propose_request`, in order: `Role { held }` unless
  the party's role allows `RequestKind::Create` (an observer's is
  refused); `NotWellFormed { at, why: ProposedAsAllowed }` for a member
  giving `AsAllowed`; 3.1's and 3.2's checks of each member but the
  authority check; `NoEscalationHolder` for the proposer's roots;
  `authority::needs(&Action::Batch(..))`, refused `Authority { lacking,
  proposable: Some(Accepter::Nobody) }` when the needs pass the project's
  ceiling or the deployment's rules (`authority::at_most` against each, as
  `goal_start`'s `hard` is computed, `acts/bounds.rs:267`), and `Authority
  { lacking: Approver, proposable: Some(Accepter::Nobody) }` when
  `recipient(core, project, ProposalKind::Batch)` is `None`; then
  `tasks::Event::ProposePerson`, answered `Proposed { proposal, to }`.
  Accepting it makes the batch as `goal_decide` makes a goal today:
  `creator: Party::Person(proposer)`, each member funded by the accepter's
  pool. A rejection is the existing message to the proposer's inbox.
- A party's pending proposal of a batch with a dependency on a task that
  ends is withdrawn in the decision that ends it, and its proposer's inbox
  shows it withdrawn.
- `withdraw_request`: the party's own pending proposal, else `Unknown`;
  one decided first is answered as a second decider is, `ProposalDecided
  { proposer, proposal, by, at, choice }` (who, how and when). A
  `DecideProposal` of a withdrawn proposal is answered `ProposalDecided {
  .., by: <the proposer>, choice: ProposalChoice::Withdrawn }`, from the
  decided row (`Stored::DecidedProposal`), live or after a restart.
Test shape:
- `tests/tasks/tests/proposals.rs` (the tasks world; `World::new`,
  `task`, `tests/tasks/src/world.rs:192`, :73): `a_partys_batch_proposal_is_kept_until_decided`,
  `a_partys_withdrawn_proposal_is_decided_withdrawn`,
  `withdrawing_another_partys_proposal_is_unknown`,
  `a_batch_proposal_naming_a_task_since_ended_is_withdrawn`. Reference:

  ```rust
  #[test]
  fn a_partys_withdrawn_proposal_is_decided_withdrawn() {
      let mut world = World::new(341, LIMITS);
      propose_batch(&mut world, 9, 100, vec![task(1, &[])]);
      let reply_to = world.to();
      world.send(Event::WithdrawPersonProposal { reply_to, proposer: 9, proposal: 100 });
      assert!(matches!(
          world.records[&Key::DecidedProposal(100)],
          Stored::DecidedProposal(ref row) if matches!(row.state, PersonProposalState::Withdrawn { .. })
      ));
  }
  ```

  (`propose_batch` is a helper of this file sending
  `Event::ProposePerson`.)
- `tests/core/tests/parties.rs`: the owner who decides is a second party,
  signed in as `tests/core/tests/client_peer.rs:21-43` signs one in and
  given role 0 by `people::Event::Roles`, a helper `owner(world)` of this
  file. The Proof's test and
  `another_partys_proposal_rejected_reaches_that_party`,
  `a_party_withdrawing_their_proposal_is_answered_withdrawn`,
  `a_proposed_batch_giving_as_allowed_is_not_well_formed`,
  `a_proposal_no_role_may_accept_is_refused_at_admission`,
  `a_proposal_beyond_the_project_ceiling_is_refused_naming_nobody`,
  `a_proposed_batch_is_refused_while_nobody_holds_its_escalations` (on
  `no_holder_fixture`, as 3.1's),
  `an_observers_propose_is_refused_by_role`,
  `withdrawing_a_proposal_decided_first_says_by_whom_and_how`,
  `deciding_a_withdrawn_proposal_answers_it_was_withdrawn`,
  `a_goal_beyond_the_pool_is_refused_naming_who_may_accept`.
- `crates/jig-protocol-store`: `task_person_proposal()`
  (`src/tests/fixtures.rs:291`) with a batch action and a reason; the
  people fixtures gain the asks and outcomes.
- Ceilings: each world test under 0.5 s.
Proof: joint layer, `tests/core/tests/parties.rs`,
`a_partys_batch_beyond_their_role_is_proposed_and_accepted` (tasks.md, 12:
"a party's batch beyond their role refused, naming what it lacked, then
proposed and accepted"): a member's `Create` is refused proposable by
role 0; their `Propose` of the same batch with a reason is answered
`Proposed { proposal, to: 0 }` and shows in the owner's inbox; the owner
accepts it; the batch is live as the member's roots, funded by the
owner's pool, and the member's inbox shows the decision. Under 0.5 s.
Run once, `cargo nextest run -p jig-core-world -E
'test(a_partys_batch_beyond_their_role_is_proposed_and_accepted)'`; it
passes, and nothing else is wanted.
Don't: let a proposal wait for nobody; give a party's proposal an
effect's or a widening of their own authority (a party makes no effect,
and their authority is their role's); change a task's `Proposal`;
serve the proposer's update by scanning decided proposals, or put a
pending proposal in `Group::Updates` (ruling 175).
Size: at most 420 lines (a codec increment).
Removes: `Outcome::GoalProposed` and the goal's proposing path
(`CreationKind::Goal { direct: false }`, `GoalRoute::Proposing`).

### 03.5 `Propose` of an amendment; the amendment's implicit proposal goes
Scope: people.md, 5.1 ("A party proposes a batch, or an amendment, a
widening of a task's authority among them"; "Proposing is explicit");
authority.md, 9 ("The proposer is whoever asked: the task, or the party,
never the task an amendment would change"); tasks.md, 9 ("A party's
proposal to amend a task is the party's, not that task's"); authority.md,
7 ("Whoever widens a task funds it": "From that commit, the giver is the
task's funder"). An `Amend` beyond the party's role is refused, saying by
whom it could be accepted, in place of `AmendProposed`; the party
proposes the amendment, and its decision reaches them. An accepted
amendment or widening, a party's or a task's, makes its accepter the
task's funder. The reference UI's amendment beyond its party's role keeps
working through the clients' component until jig 14.5 (Choices).
Needs: nothing beyond 3.4.
Files:
- `crates/jig-domain-people/src/batch.rs` (`Action::Amend`),
  `boundary.rs` (`Outcome::AmendProposed` goes), `domain.rs` (at most 40),
  `capacity.rs`.
- `crates/jig-domain-tasks/src/person_proposals.rs` (`PersonAction::Amend`:
  the task live, the amendment checked as `control.rs` checks one;
  withdrawn when its task ends, by 3.4's withdrawal in `end_task`).
- `crates/jig-domain-core/src/routing/propose.rs` (an amendment's
  proposal), `routing/party_decisions.rs` (accepting a party's amendment
  applies it as `person_proposal_accept`'s `Amend` arm does, :238 on,
  `by: Party::Person(proposer)`, `funder: Some(<the accepter's pool>)`;
  that function's `Amend` and `Widen` arms, :242 and :267, pass the
  accepter's pool too), `routing/runs.rs` (`named_proposal_accept`'s
  `Amend` and `Widen` arms, :123 and :143, pass
  `Funder::Task(key.task)`), `routing/party_controls.rs` (:62-86, the
  rewrite to a proposal goes), `acts/bounds.rs` (`amendment_admit`
  answers a refusal), `routing/people.rs`, `routing/tasks.rs`,
  `routing/application_routes.rs`, `sibling_routes.rs` (`AmendProposed`'s
  arms, search S11), `run.rs` (`PersonTaskRoute::AmendProposed`, :392,
  goes).
- The acceptance's funding, in the tasks child: `boundary.rs`
  (`Event::Amend`'s `funder`, :1345), `domain.rs` (:535), `control.rs`
  (`check_amend`, :250, and `apply_amend`, :401; at most 40), `funders.rs`
  (the move's funding, factored out of `moving.rs`), `moving.rs` (calls
  it). The tests that build `Event::Amend` gain `funder: None`:
  `crates/jig-domain-tasks/src/tests/{revisions.rs, revisions/tracking.rs,
  address.rs, instructions.rs}`, `crates/jig-domain-core/src/tests.rs`
  (:937), `src/tests/{task_query.rs, acts/tracking.rs}`,
  `tests/tasks/tests/lifecycle/{grace.rs, goals.rs}`.
- The referee's evidence for an accepted amendment or widening (ruling
  195): `testing/jig-conformance/src/referee.rs` (`Snapshot::accepted_tasks`,
  :232, and `durable`, :462-501; at most 40 lines), the two adapters that
  fill it, `tests/core/src/observations.rs` (:781-795) and
  `tests/ops/src/conformance/observations.rs` (:230-236; at most 25 lines
  each), and a referee case in `tests/core/tests/referee.rs`.
- `crates/jig-protocol-store`: the people codec of `Action::Amend`; the
  tasks codec of `PersonAction::Amend` (in place).
- The transitional piece: `crates/jig-protocol-engine-clients/src/component.rs`
  (`Waiting::Ask`, :515-534, keeps a v8 `Amend`'s translated ask until
  answered and sends the follow-up request as `ServiceMade` turns into
  `Waiting::Made`, :516-522; `Waiting::Ask` built with the kept ask at
  :711), `src/sign_in.rs` (`Waiting`, :15, whose `Ask` variant, :25,
  holds the kept ask), `src/translate/answers.rs` (v8's `AmendProposed` from
  `Proposed`), `src/limits.rs` (the kept ask in the component's worst
  case); `src/tests/fixtures/answers/outcomes.rs` (`AmendProposed`'s row,
  :143, goes).
- Tests that relied on `AmendProposed`, moved to `Propose`:
  `tests/core/tests/stories.rs` (`an_amendment_and_a_release_are_one_decision`,
  :981, through its helper
  `a_widening_amendment_with_release_is_proposed_and_released_on_acceptance`,
  :1224), `tests/people/tests/simulation.rs` (the test of that name,
  :615), `tests/people/src/world.rs` (:713).
Shape: `people::Action::Amend { task, revision, amendment: Box<Amendment>
}`; `tasks::PersonAction::Amend { task, amendment: Box<Amendment> }`
(sketches 1 and 2). `propose_request` for an amendment: the task in the
party's reach (`Core::reaches`), its revision as `Amend` names it, the
widening's needs (`authority::needs(&Action::Amend(..))`), refused
`Approver`/`Nobody` when no role decides `ProposalKind::Amend`, then
`ProposePerson`, answered `Proposed`. Accepted, the amendment is the
proposer's, funded by the accepter (below); rejected, the reason reaches
the party's inbox. A party's pending proposal to amend a task that ends
is withdrawn in the decision that ends it.

The acceptance's funding (authority.md, 7: "*when it already was,* its
reservation is resized to the task's new budget; *otherwise,* the task is
funded anew from it, as a moved task is"), for every acceptance of an
amendment or a widening, whoever proposed it and whoever accepts it:
- `tasks::Event::Amend` gains `funder: Option<Funder>`. `None` is an
  amendment made directly (`party_controls.rs`, an agent's `amend` at
  `runs.rs:624`), its present funder resized as on main. `Some(f)` is an
  acceptance, `f` the accepter's: `Funder::Pool { project, person,
  period: core.period(project) }` for a party, opened first as the batch
  arms open it (`party_decisions.rs:50`, `:220`); `Funder::Task(key.task)`
  for a task.
- `check_amend`: when `f` is the task's funder, `can_resize` as now
  (`control.rs:340`); otherwise the move's funding preflight
  (`moving.rs:198-229`) at the amendment's budget, refused `Funding` when
  it fails. `apply_amend`: `resize` as now (:412), or the move's funding
  (`moving.rs:347-368`, `:403-408`): the old funder's reserved less the
  task's budget, its spent below plus what the task spent; `f`'s reserved
  plus the new budget less what the task spent; the task's funder `f`,
  its budget that amount, its authority's spend the same, its spent and
  spent below 0. The move and the amendment share that code, in
  `funders.rs`. The task's requester, root, depth and delegates are
  unchanged.
- The acceptance's authority check is unchanged: the accepter covers the
  whole widened authority (`jig-domain-authority/src/check.rs:653`).

The transitional piece, from this increment to jig 14.5: the clients'
component keeps the `people::Ask::Amend` it translated from a v8
`Ask::Amend` in that exchange's `Waiting::Ask`. When the answer is
`Refused(Authority { proposable: Some(Accepter::Role(_)), .. })`, it sends,
under the same sign-in, `people::Ask::Propose { project, action:
Action::Amend { task, revision, amendment }, reason: <the amendment's
reason> }` keyed by the request's key with its last byte's high bit
flipped, so that the request sent again replays both answers; it answers
the client v8's `AmendProposed { task, proposal, released: false }` from
`Proposed`, and any other answer as it would the `Amend`'s. The other
answers of a v8 `Amend` are translated as at the base. jig 14.5 moves the
reference UI's amend confirmation onto `Propose` and removes this piece.
Test shape:
- `tests/tasks/tests/proposals.rs`: `a_partys_amendment_proposal_names_the_party_as_proposer`,
  `an_amendment_proposal_whose_task_ends_is_withdrawn`.
- `tests/tasks/tests/funding.rs` (helpers `turn`, :14, and `ledger`, :38):
  `an_amendment_accepted_by_another_funder_funds_its_task_anew`. A task
  funded by one pool with 10 spends 3 in a turn. An `Event::Amend` to 15
  with `funder: Some(<a second pool>)` leaves the first pool's ledger
  with 10 less reserved and 3 more spent below, and the second's with 12
  more reserved; the task's funder is the second pool, its budget 12.
  Then an amendment to 14 with `funder: Some(<the second pool>)` resizes
  it to 14, its funder unchanged.
- `tests/core/tests/parties.rs`: the Proof's test and
  `a_widening_amendment_beyond_the_role_is_refused_naming_who_may_accept`,
  `a_rejected_amendment_proposal_reaches_the_party_not_the_task`.
- `tests/engine-clients/tests/requests.rs` (the clients' component over
  the real core; helpers `signed_in` :9 and `request` :20):
  `a_v8_amend_beyond_the_role_is_proposed_and_answered_amend_proposed`,
  `a_v8_amend_sent_again_replays_the_same_proposal`,
  `a_v8_amend_beyond_the_ceiling_stays_refused`. The client's unit test
  at `crates/jig-domain-client/src/tests/confirmation.rs:311` is
  unchanged: the client still sends `Ask::Amend`.
- `crates/jig-protocol-store`: the fixtures gain an amendment action.
- Ceilings: each world test under 0.5 s.
Proof: joint layer, `tests/core/tests/parties.rs`,
`a_partys_widening_amendment_is_proposed_by_the_party_and_accepted`: a
member's `Amend` widening a task's budget past their role is refused
proposable by role 0; their `Propose` of it is answered `Proposed`, and
the proposal's document names the member, not the task, as its proposer;
the owner accepts it, and the task's authority is the widened one. Its
funder is the owner's pool, which reserves the widened budget (the task
spent nothing), and its funder before is given back the budget it had
reserved for it. Under 0.5 s. Run once, `cargo
nextest run -p jig-core-world -E
'test(a_partys_widening_amendment_is_proposed_by_the_party_and_accepted)'`,
and once `cargo nextest run -p jig-engine-clients-world -E
'test(a_v8_amend_beyond_the_role_is_proposed_and_answered_amend_proposed)'`
for the transitional piece; each passes, and nothing else is wanted.
Don't: keep any path in the engine that commits a proposal its proposer
did not ask for, but a result's follow-ups (tasks.md, 5.6); turn into a
`Propose` any v8 ask but `Amend`, or an `Amend` refused on anything but a
proposable authority finding; charge an accepted amendment or widening to
its task's present funder when the accepter is another; change the
funding of an amendment made directly (`funder: None`; `later.md`).
Size: at most 320 lines (a codec increment, and the acceptance's
funding).
Removes: `Outcome::AmendProposed` and `party_controls.rs`'s rewrite to a
proposal.

## Interface sketches

Compiled at jig's base `a3b53546` as scratch modules of
`jig-domain-people` and `jig-domain-tasks`, `cargo clippy -p
jig-domain-tasks -p jig-domain-authority -p jig-domain-people -p
jig-domain-core --all-targets -- -D warnings`: passed; compiled again
at jig `45b1479f3`, after jig 02 (ruling 146), with
sketch 1's `Flags::PLAIN`: passed. The landed tasks child's `Flags` and
`Retention` (`crates/jig-domain-tasks/src/flags.rs:4`, :18) match the
people child's mirrors field for field. The additions to `Ask`, `Outcome` and `Refusal` were compiled
as enums of their own beside the base's. `Outcome`, `Reply`, `Refusal` and
`Lack` are `Copy`, and the tests copy replies
(`crates/jig-domain-people/src/tests.rs:319`), so what they hold (`Given`,
`Accepter`, `Malformed`, `TaskExecutor`) derives `Copy`, as `Flags` and
`Retention` do in the tasks child; the other types derive the rest.

```rust
// Sketch 1. crates/jig-domain-people/src/batch.rs, boundary.rs and refusals.rs (jig 03.1 to 03.5)
/// One task of a party's `Create`, or of a batch they propose, in the people child's vocabulary
/// (domain/tasks.md, sections 3 and 4; client/domain/contract.md, 3.2). Mirrors the tasks child's
/// `New` and the core's `Delegate`, and `Member` in `people-v3.schema`, field for field.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Member {
    pub executor: TaskExecutor,
    /// Its spec's words.
    pub words: Box<[u8]>,
    /// Its spec's parameters, its connector's for a procedure.
    pub parameters: Box<[Parameter]>,
    /// Ended tasks in the party's reach whose results it takes.
    pub inputs: Box<[u64]>,              // 03.2
    pub contract: Contract,
    /// Its authority; `authority.spend` is its budget unless `budget` is `AsAllowed`.
    pub authority: Authority,
    pub budget: Budget,                  // 03.2
    pub wake: WakePolicy,
    /// What it starts after: members of this batch, or live tasks in the party's reach.
    pub dependencies: Box<[Dependency]>, // 03.1: Box<[u32]>, positions in the batch
    /// Open grants narrowed with the new task's number: the resources named after it.
    pub named: Box<[Grant]>,             // 03.2
    /// Its priority among the project's goals when tracked.
    pub tracked: Option<u32>,
    /// The application's opaque label.
    pub label: u32,
    pub flags: Flags,
}
/// Who carries a member out.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskExecutor {
    /// An agent running a configured charter.
    Agent { charter: u32 },
    /// A connector's procedure (03.2).
    Procedure { connector: u16, code: u32 },
    /// A party, or any holder of a project role (03.2).
    Person(PersonAddress),
}
/// A member's result contract; an approval is a procedure's, never a party's (domain/tasks.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Contract {
    /// A report of at most `words` bytes.
    Report { words: u32 },
    /// One of the listed verdicts.
    Verdict { choices: Box<[Verdict]> },
    /// A connector's result of this kind, with a report of at most `words` bytes (03.2).
    Change { connector: u16, kind: u16, words: u32 },
}
/// One verdict a contract allows; mirrors the tasks child's `Verdict`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Verdict { pub code: u32, pub words: u32, pub followups: u32, pub fields: Box<[FieldRule]> }
/// One machine field a verdict requires; mirrors the tasks child's `FieldRule`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FieldRule { pub name: Box<[u8]>, pub max: u32 }
/// How a member's budget is given (domain/authority.md, section 7; 03.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Budget {
    /// `authority.spend`, as given.
    Named,
    /// As much as fits under the party at admission, at most `cap`; `authority.spend` is not read.
    AsAllowed { cap: Option<u64> },
}
/// What a member starts after (03.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Dependency {
    /// Another member of the same batch, by zero-based position.
    Batch(u32),
    /// A live task in the party's reach.
    Existing(u64),
}
/// A member's flags; mirrors the tasks child's `Flags` (jig 02.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Flags { pub conversation: bool, pub listed: bool, pub retention: Retention }
impl Flags {
    /// As the tasks child's `Flags::PLAIN` (crates/jig-domain-tasks/src/flags.rs:14).
    pub const PLAIN: Flags = Flags { conversation: false, listed: false, retention: Retention::Kept };
}
/// Mirrors the tasks child's `Retention` (jig 02.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Retention { Kept, ForgottenOnceRead }
/// What a party proposes (domain/people.md, 5.1): a batch, or an amendment of a task, a widening
/// of its authority among them; never an effect.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    /// A batch whose tasks would be the party's roots, each budget named (03.4).
    Batch(Box<[Member]>),
    /// An amendment of `task`, naming the spec's revision the party saw (03.5).
    Amend { task: u64, revision: u64, amendment: Box<Amendment> },
}
// Ask gains (mirrors `AskCreate`, `AskPropose`, `AskWithdraw` in `people-v3.schema`):
//     /// Make `batch` at once, its tasks this party's roots (03.1).
//     Create { project: u32, batch: Box<[Member]> },
//     /// Propose `action` with a reason, to whoever the project's policy names (03.4).
//     Propose { project: u32, action: Action, reason: Box<[u8]> },
//     /// Withdraw this party's own pending proposal (03.4).
//     Withdraw { project: u32, proposal: u64 },
// Outcome gains, and loses `GoalProposed` (03.4) and `AmendProposed` (03.5):
//     /// The batch was made, its tasks numbered `first` to `last` in batch order.
//     Created { first: u64, last: u64, given: Option<Given> }, // `given` from 03.2
//     /// The proposal waits for the parties holding role `to`, the lowest-numbered that decides its kind.
//     Proposed { proposal: u64, to: u32 },
//     /// The party's proposal was withdrawn.
//     Withdrawn { proposal: u64 },
/// The budget a member gave `AsAllowed`, and what it was given (03.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Given { pub task: u64, pub amount: u64 }
/// Who could accept a proposal of a refused request (client/domain/contract.md, 3.5 and 4; 03.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Accepter {
    /// No proposal of it could be accepted: beyond the ceiling or the rules, or no role decides its kind.
    Nobody,
    /// The lowest-numbered role whose parties decide its kind.
    Role(u32),
}
// Refusal changes and gains:
//     /// `proposable` was `Option<bool>`; `None` still makes no claim (03.3).
//     Authority { lacking: Lack, proposable: Option<Accepter> },
//     /// A batch's member, by position, is not well formed (03.1).
//     NotWellFormed { at: u32, why: Malformed },
/// Why a member is not well formed (client/domain/contract.md, 4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Malformed {
    /// The batch's own dependencies close a cycle through this member (03.1).
    Cycle,
    /// A second member gives its budget `AsAllowed` (03.2).
    AsAllowedTwice,
    /// The conversation flag on a task whose executor is not an agent (03.2).
    Conversation,
    /// A proposed batch gives a budget `AsAllowed` (03.4).
    ProposedAsAllowed,
}
// Lack changes (03.2):
//     /// The executor's kind is outside the creator's delegation; names the member's executor
//     /// where a batch's member is refused, `None` elsewhere.
//     TaskKind { executor: Option<TaskExecutor> },

// Sketch 2. crates/jig-domain-tasks/src/proposals.rs and boundary.rs (jig 03.4, 03.5)
/// A party's proposal, kept apart from the task-origin `Proposal` (domain/tasks.md, section 9).
/// Replaces `PersonProposal { goal: New }` in place; mirrors `PersonProposal` in `tasks-v3.schema`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PersonProposal {
    pub number: u64,
    /// The proposing party.
    pub proposer: u64,
    pub project: u32,
    pub action: PersonAction,
    /// The party's reason, in words.
    pub reason: Box<[u8]>,
    pub state: PersonProposalState,
}
/// What a party's proposal asks for.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PersonAction {
    /// Roots of the proposer, funded by the accepter (a goal is one tracked member).
    Batch(Box<[New]>),
    /// An amendment of a live task, the proposer's when accepted (03.5).
    Amend { task: u64, amendment: Box<Amendment> },
}
/// A party proposal's state.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PersonProposalState {
    Pending { since: Wall },
    Accepted { by: Party },
    Rejected { by: Party, reason: Box<[u8]>, message: u64, at: Wall },
    /// The proposer withdrew it before a decision (03.4).
    Withdrawn { at: Wall },
}
// Event gains (03.4):
//     /// Withdraw party `proposer`'s pending proposal; answered `PersonProposalDecided` with
//     /// `ProposalOutcome::Withdrawn`, or refused `Unknown`.
//     WithdrawPersonProposal { reply_to: ReplyTo, proposer: u64, proposal: u64 },
```

Who owns what: the people child owns the asks, their admission and the
answers it keeps by key; the core owns the routes (`routing/create.rs`,
`routing/propose.rs`), which check, number and translate; the authority
child answers each check; the tasks child makes the batch, keeps a
party's proposal and its decision, and refuses what is not well formed
among its own checks.

## Questions to raise when reached

- **3.2: references and inputs' loads:** answered by ruling 156
  (Choices; 03.2's inputs). `routing/create.rs` past 80 lines of inputs,
  counted by code-organization.md 7's command on that file, is still
  raised in the increment's report.

## Done when

- The five increments have landed on jig's `origin/main`, each with its
  Proof passing in its landing.
- `git -C <a jig worktree at origin/main> grep -n -E
  'GoalProposed|AmendProposed' -- crates/jig-domain-people
  crates/jig-domain-core crates/jig-domain-tasks` prints nothing.
- `git -C <the same worktree> grep -n -E '^    (Create|Propose|Withdraw) \{'
  -- crates/jig-domain-people/src/boundary.rs` prints three lines, the
  variants of `Ask`.
