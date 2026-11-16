//! A party fills a task without becoming its delegates' requester or funder.
use super::*;
use jig_core_world::stories::offers::world_with_policy;
use jig_domain_authority as a;
use skein_lib::Wall;

const G: u64 = FIRST;
const A: u64 = 3;
const B: u64 = 4;
const NO_QUESTIONS: &[jig_domain_core::documents::InboxObject] = &[];

fn world(seed: u64) -> World {
    with_role(seed, Some(people::Role::Member), true)
}
fn with_role(seed: u64, role: Option<people::Role>, holder: bool) -> World {
    world_with_policy(seed, role, |limits, rules, policy| {
        limits.core.people.batch = 2;
        limits.core.tasks.batch = 2;
        limits.core.authority.batch = 2;
        limits.core.tasks.tasks = 16;
        limits.core.tasks.project_tasks = 16;
        limits.core.tasks.tree_tasks = 16;
        limits.core.tasks.delegates = 8;
        if !holder {
            limits.core.authority.roles = 2;
            let mut member = policy.roles[0].clone();
            member.number = MEMBER_ROLE;
            member.decides = a::Proposals(0);
            policy.roles = Box::new([policy.roles[0].clone(), member]);
            policy.escalation_role = None;
        }
        let kinds: Box<[a::Executor]> =
            Box::new([a::Executor::Charter(CHARTER), a::Executor::Procedure(jig_domain_core::GATHERING)]);
        for authority in [&mut rules.ceiling, &mut policy.ceiling] {
            authority.delegation = a::Delegation { kinds: kinds.clone(), tasks: 12, depth: 3 };
        }
        for role in &mut policy.roles {
            role.authority.delegation = a::Delegation { kinds: kinds.clone(), tasks: 10, depth: 3 };
            if role.number == OWNER_ROLE {
                role.authority.budget.spend = 300;
            }
        }
    })
}

fn gathering() -> people::Member {
    let mut member = people_member(b"gather", 100);
    member.executor =
        people::TaskExecutor::Procedure { connector: jig_domain_core::RECURRING, code: jig_domain_core::GATHERING };
    member.flags.ended_by = people::EndedBy::Requester;
    member.contract = people::Contract::Report { words: 128 };
    member.authority.delegation =
        people::Delegation { kinds: Box::new([people::Executor::Charter(CHARTER)]), tasks: 5, depth: 1 };
    member
}

fn delegate(world: &mut World, key: u8, task: u64, batch: Box<[people::Member]>) {
    ask_as(world, world.sign_in.expect("signed in"), key, people::Ask::Delegate { project: PROJECT, task, batch });
}

fn made(world: &World, first: u64, last: u64, given: Option<people::Given>) {
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Created { first, last, given }))
    );
}

#[test]
fn a_party_delegates_beneath_gatherings_and_agents_with_task_funding_and_party_history() {
    for agent in [false, true] {
        let mut world = world(370 + u64::from(agent));
        let mut parent = gathering();
        if agent {
            parent.executor = people::TaskExecutor::Agent { charter: CHARTER };
            parent.flags = people::Flags::PLAIN;
        }
        create(&mut world, Box::new([parent]));
        made(&world, G, G, None);
        let reserved = pool_reserved(&world, PARTY);
        let mut second = people_member(b"second", 20);
        second.dependencies = Box::new([people::Dependency::Batch(0)]);
        delegate(&mut world, 90, G, Box::new([people_member(b"first", 30), second]));
        made(&world, A, B, None);
        assert_eq!(pool_reserved(&world, PARTY), reserved);
        assert_eq!(live_row(&world, G).numbers.reserved, 50);
        for number in [A, B] {
            let row = live_row(&world, number);
            assert_eq!(
                (row.requester, row.funder, row.root, row.depth),
                (tasks::Party::Task(G), tasks::Funder::Task(G), G, 1)
            );
            assert!(world.store.rows.values().any(|row| matches!(row, jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::History(history))) if history.task == number && history.by == tasks::Party::Person(PARTY) && history.change == tasks::Change::Made)));
        }
        assert_eq!(live_row(&world, B).waiting_on.as_ref(), &[A]);
    }
}

#[test]
fn a_party_delegate_is_within_the_tasks_remaining_budget_slots_depth_and_deadline() {
    for case in 0..4 {
        let mut world = world(372 + case);
        let mut parent = gathering();
        if case == 1 {
            parent.authority.delegation.tasks = 1;
        }
        if case == 2 {
            parent.authority.delegation.depth = 0;
        }
        if case == 3 {
            parent.authority.deadline = Some(Wall::from_nanos(1_000_000_000));
        }
        create(&mut world, Box::new([parent]));
        made(&world, G, G, None);
        if case == 0 {
            delegate(&mut world, 89, G, Box::new([people_member(b"reserved", 30)]));
            made(&world, A, A, None);
        }
        let mut first = people_member(b"first", if case == 0 { 50 } else { 10 });
        if case == 3 {
            first.authority.deadline = Some(Wall::from_nanos(2_000_000_000));
        }
        let batch: Box<[people::Member]> =
            if case == 3 { Box::new([first]) } else { Box::new([first, people_member(b"second", 30)]) };
        delegate(&mut world, 91, G, batch);
        let lacking = match case {
            0 => people::Lack::Budget { needs: 80, left: 70 },
            1 => people::Lack::Tasks { left: 1 },
            2 => people::Lack::Depth { most: 0 },
            3 => people::Lack::Deadline,
            _ => unreachable!(),
        };
        refusal(&world, people::Refusal::Authority { lacking, proposable: None });
        assert_eq!(live_row(&world, G).numbers.reserved, if case == 0 { 30 } else { 0 });
        assert!(!world.store.rows.contains_key(&task_key(tasks::Key::Live(if case == 0 { B } else { A }))));
    }
}

#[test]
fn delegate_as_allowed_uses_the_tasks_remaining_after_named_amounts() {
    let mut world = world(376);
    create(&mut world, Box::new([gathering()]));
    delegate(&mut world, 92, G, Box::new([people_member(b"reserved", 30)]));
    made(&world, A, A, None);
    let mut rest = people_member(b"rest", 999);
    rest.budget = people::Budget::AsAllowed { cap: None };
    delegate(&mut world, 93, G, Box::new([people_member(b"named", 20), rest]));
    made(&world, B, 5, Some(people::Given { task: 5, amount: 50 }));
    assert_eq!(live_row(&world, G).numbers.reserved, 100);
    assert_eq!(live_row(&world, 5).numbers.budget, 50);
    let mut capped = with_role(387, Some(people::Role::Maintainer), true);
    let maintainer = capped.sign_in.expect("maintainer");
    let owner = owner(&mut capped);
    let mut parent = gathering();
    parent.authority.spend = 200;
    ask_as(&mut capped, owner, 114, people::Ask::Create { project: PROJECT, batch: Box::new([parent]) });
    made(&capped, G, G, None);
    capped.sign_in = Some(maintainer);
    let mut member = people_member(b"role cap", 999);
    member.budget = people::Budget::AsAllowed { cap: None };
    delegate(&mut capped, 115, G, Box::new([member.clone()]));
    made(&capped, A, A, Some(people::Given { task: A, amount: 100 }));
    member.budget = people::Budget::AsAllowed { cap: Some(17) };
    delegate(&mut capped, 116, G, Box::new([member]));
    made(&capped, B, B, Some(people::Given { task: B, amount: 17 }));
}

#[test]
fn delegate_reaches_as_the_party_and_judges_ended_dependencies() {
    for how in [None, Some(tasks::Status::Done), Some(tasks::Status::Failed)] {
        let mut world = world(377);
        create(&mut world, Box::new([gathering()]));
        if let Some(how) = how {
            end_prerequisite(&mut world, how);
        }
        let mut dependent = depending(&[CHAT]);
        if how == Some(tasks::Status::Done) {
            dependent.inputs = Box::new([CHAT]);
        }
        delegate(&mut world, 94, G, Box::new([dependent]));
        if how == Some(tasks::Status::Failed) {
            refusal(&world, people::Refusal::Dependency { dependency: CHAT, how: people::DependencyStatus::Failed });
        } else {
            made(&world, A, A, None);
            assert_eq!(live_row(&world, A).dependencies.as_ref(), &[CHAT]);
            let waits: &[u64] = if how.is_none() { &[CHAT] } else { &[] };
            assert_eq!(live_row(&world, A).waiting_on.as_ref(), waits);
        }
    }
}

#[test]
fn a_party_fills_then_concludes_a_gathering_and_its_two_delegates_end_it() {
    let mut world = world(378);
    create(&mut world, Box::new([gathering()]));
    delegate(&mut world, 95, G, Box::new([people_member(b"first", 30)]));
    let mut second = people_member(b"second", 20);
    second.dependencies = Box::new([people::Dependency::Existing(A)]);
    delegate(&mut world, 96, G, Box::new([second]));
    made(&world, B, B, None);
    let sign_in = world.sign_in.expect("signed in");
    ask_as(&mut world, sign_in, 97, people::Ask::Conclude { project: PROJECT, task: G });
    let at = live_row(&world, G).concluded.expect("concluded").at;
    delegate(&mut world, 98, G, Box::new([people_member(b"too late", 1)]));
    refusal(&world, people::Refusal::Concluded { at });
    finish(&mut world); // the pre-existing chat frees the worker
    assert_eq!(world.assigned.as_ref().expect("first").task, A);
    finish(&mut world);
    assert_eq!(world.assigned.as_ref().expect("second").task, B);
    finish(&mut world);
    let jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row))) =
        world.store.rows.get(&task_key(tasks::Key::Ended(G))).expect("ended gathering")
    else {
        panic!("ended row")
    };
    assert_eq!(
        row.phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report {
            words: b"Concluded: 2 done, 0 failed, 0 cancelled.".as_slice().into()
        }))
    );
}

fn question_world(seed: u64) -> World {
    let mut world = world(seed);
    world.domain.core_mut().person_answers();
    let mut parent = gathering();
    parent.authority.tools = jig_domain_core::ToolFamilies::JIG.message.0;
    create(&mut world, Box::new([parent]));
    let mut child = people_member(b"ask", 30);
    child.authority.tools = jig_domain_core::ToolFamilies::JIG.message.0;
    delegate(&mut world, 117, G, Box::new([child]));
    made(&world, A, A, None);
    world
}

fn gathering_question(world: &mut World) -> u64 {
    use jig_domain_core as core;
    use skein_lib::{ReplyTo, Token};
    assert_eq!(world.assigned.as_ref().expect("delegate run").task, A);
    let attempt = world.assigned.as_ref().expect("delegate run").attempt;
    world.send(jig_test_domain::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(40_001)),
        key: core::CallKey { task: A, attempt, completion: 1, position: 0 },
        action: core::NamedAction::Message {
            target: G,
            kind: tasks::MessageKind::Question,
            words: b"proceed?".as_slice().into(),
        },
    }));
    let Some((_, _, core::CallPart::Sent { message })) = world.answers.last() else {
        panic!("question admitted: {:?}", world.answers.last())
    };
    *message
}

#[test]
fn a_gatherings_delegate_asks_its_person_and_hears_the_answer_from_its_requester() {
    use jig_domain_core::{documents, reads};
    let mut world = question_world(388);
    finish(&mut world);
    let question = gathering_question(&mut world);
    assert_eq!(live_row(&world, A).questions[0].answerer, tasks::Party::Person(PARTY));
    assert!(live_row(&world, G).inbox.is_empty(), "the gathering keeps no question");
    let sign_in = world.sign_in.expect("signed in");
    assert_eq!(
        inbox(&mut world, sign_in, reads::Group::Questions),
        vec![documents::InboxObject::Question { message: question }]
    );
    jig_core_world::reads::ask(&mut world, 911, reads::Read::Question { task: G, message: question });
    let reads::Answer::Document(frame) = jig_core_world::reads::latest(&world) else { panic!("live question") };
    let documents::Document::Question(document) = frame.as_ref() else { panic!("question document") };
    assert_eq!(document.from, tasks::Party::Task(A));
    assert_eq!(document.words.bytes.as_ref(), b"proceed?");
    ask_as(
        &mut world,
        sign_in,
        118,
        people::Ask::AnswerQuestion { project: PROJECT, task: G, question, words: b"proceed".as_slice().into() },
    );
    assert!(live_row(&world, A).questions.is_empty(), "answer: {:?}", world.people_answers.last());
    assert!(live_row(&world, A).inbox.iter().any(|word| word.from == tasks::Party::Task(G)
        && word.kind == tasks::MessageKind::Answer { question }
        && word.words.as_ref() == b"proceed"));
    assert!(live_row(&world, G).inbox.is_empty(), "the gathering keeps no answer notice");
    assert_eq!(live_row(&world, G).question_lines[0].answer.expect("retained answer").task, A);
    assert_eq!(inbox(&mut world, sign_in, reads::Group::Questions).as_slice(), NO_QUESTIONS);
}

#[test]
fn a_deployments_gathering_keeps_its_delegates_question_out_of_person_inboxes() {
    use jig_domain_core::{self as core, reads};
    let seed = 389;
    let mut world = question_world(seed);
    // A deployment-owned gathering is a restored fixture, not a person's Create.
    let mut row = live_row(&world, G).clone();
    row.requester = tasks::Party::Deployment { project: PROJECT };
    world
        .store
        .seed(Box::new([jig_test_domain::Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(row))))]));
    world.restart(seed, false);
    world.domain.core_mut().person_answers();
    crate::closed_proofs::hello(&mut world);
    let question = gathering_question(&mut world);
    assert_eq!(live_row(&world, A).questions[0].answerer, tasks::Party::Task(G));
    assert!(live_row(&world, G).inbox.iter().any(|word| word.number == question
        && word.from == tasks::Party::Task(A)
        && word.kind == tasks::MessageKind::Question));
    let sign_in = world.sign_in.expect("signed in");
    assert_eq!(inbox(&mut world, sign_in, reads::Group::Questions).as_slice(), NO_QUESTIONS);
    other_party(&mut world, people::Role::Member);
    let other = world.sign_in.expect("second party signed in");
    assert_eq!(inbox(&mut world, other, reads::Group::Questions).as_slice(), NO_QUESTIONS);
}

#[test]
fn a_delegate_checks_the_partys_role_and_names_its_missing_executor_without_a_proposal() {
    for role in [people::Role::Member, people::Role::Observer] {
        let mut world = world_with_policy(379, Some(role), |_, _, policy| {
            policy.roles.iter_mut().find(|r| r.number == role.number()).expect("role").authority.delegation.kinds =
                Box::new([a::Executor::Procedure(PROCEDURE)]);
        });
        delegate(&mut world, 99, CHAT, Box::new([people_member(b"agent", 10)]));
        let why = if role == people::Role::Observer {
            people::Refusal::Role { held: Some(role) }
        } else {
            people::Refusal::Authority {
                lacking: people::Lack::TaskKind { executor: Some(people::TaskExecutor::Agent { charter: CHARTER }) },
                proposable: None,
            }
        };
        refusal(&world, why);
    }
}

#[test]
fn delegate_has_task_act_reach_and_refuses_ended_and_person_creators() {
    for maintainer in [false, true] {
        let role = if maintainer { people::Role::Maintainer } else { people::Role::Member };
        let mut world = with_role(380, Some(role), true);
        create(&mut world, Box::new([gathering()]));
        other_party(&mut world, role);
        delegate(&mut world, 100, G, Box::new([people_member(b"child", 10)]));
        if maintainer {
            made(&world, A, A, None);
        } else {
            refusal(&world, people::Refusal::Unknown);
        }
    }
    let mut world = world(381);
    create(&mut world, Box::new([gathering()]));
    let sign_in = world.sign_in.expect("signed in");
    ask_as(&mut world, sign_in, 101, people::Ask::Conclude { project: PROJECT, task: G });
    delegate(&mut world, 102, G, Box::new([people_member(b"ended", 10)]));
    assert!(matches!(
        world.people_answers.last(),
        Some(people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Ended {
            how: people::Ending::Done,
            ..
        })))
    ));
    let mut below = self::world(386);
    create(&mut below, Box::new([gathering()]));
    delegate(&mut below, 110, G, Box::new([people_member(b"ended delegate", 10)]));
    finish(&mut below);
    finish(&mut below);
    for root_ended in [false, true] {
        if root_ended {
            let sign_in = below.sign_in.expect("signed in");
            ask_as(&mut below, sign_in, 111, people::Ask::Conclude { project: PROJECT, task: G });
        }
        delegate(&mut below, if root_ended { 113 } else { 112 }, A, Box::new([people_member(b"too late", 1)]));
        assert!(matches!(
            below.people_answers.last(),
            Some(people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Ended {
                how: people::Ending::Done,
                ..
            })))
        ));
    }
    let mut world = person_world(382);
    let mut person = people_member(b"person", 10);
    person.executor = people::TaskExecutor::Person(people::PersonAddress::Role(MEMBER_ROLE));
    create(&mut world, Box::new([person]));
    delegate(&mut world, 103, G, Box::new([people_member(b"child", 10)]));
    refusal(
        &world,
        people::Refusal::Authority { lacking: people::Lack::TaskKind { executor: None }, proposable: None },
    );
}

#[test]
fn delegate_rejects_dependencies_and_inputs_outside_the_partys_tree() {
    for input in [false, true] {
        let mut world = world(383);
        create(&mut world, Box::new([gathering()]));
        let original = world.sign_in.expect("original party");
        other_party(&mut world, people::Role::Member);
        create_at(&mut world, 104, Box::new([people_member(b"outside", 10)]));
        made(&world, A, A, None);
        if input {
            finish(&mut world);
            finish(&mut world);
        }
        world.sign_in = Some(original);
        let mut member = people_member(b"consumer", 10);
        if input {
            member.inputs = Box::new([A]);
        } else {
            member.dependencies = Box::new([people::Dependency::Existing(A)]);
        }
        delegate(&mut world, 105, G, Box::new([member]));
        refusal(&world, people::Refusal::Unknown);
    }
}

#[test]
fn delegate_needs_no_escalation_holder_and_refuses_a_task_requested_conversation() {
    let mut world = with_role(384, None, false);
    create(&mut world, Box::new([gathering()]));
    made(&world, G, G, None);
    let sign_in = world.sign_in.expect("signed in");
    ask_as(
        &mut world,
        sign_in,
        106,
        people::Ask::SetRoles {
            project: PROJECT,
            holdings: Box::new([people::Holding { person: PARTY, role: people::Role::Member }]),
        },
    );
    delegate(&mut world, 107, G, Box::new([people_member(b"no holder needed", 10)]));
    made(&world, A, A, None);
    let mut conversation = people_member(b"conversation", 10);
    conversation.flags.conversation = true;
    delegate(&mut world, 108, G, Box::new([conversation]));
    refusal(&world, people::Refusal::NotWellFormed { at: 0, why: people::Malformed::Conversation });
    let mut requester = people_member(b"requester ended", 10);
    requester.flags.ended_by = people::EndedBy::Requester;
    delegate(&mut world, 109, G, Box::new([requester]));
    refusal(&world, people::Refusal::NotWellFormed { at: 0, why: people::Malformed::EndedBy });
}
