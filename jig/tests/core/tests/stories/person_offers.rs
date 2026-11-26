//! Stalled person tasks reach another inbox without losing their first addressee.
use super::*;
use skein_lib::Duration;

const STALL: Duration = Duration::from_secs(1);
const MEMBER: people::Role = people::Role::Member;
const POLICY: people::Role = people::Role::Maintainer;
const NO_TASKS: [u64; 0] = [];

#[derive(Clone, Copy)]
struct Party {
    person: u64,
    sign_in: u64,
}

fn party(world: &mut World, subject: u64) -> Party {
    world.send(signin(Token::new(7800 + subject), 0, &subject.to_be_bytes(), b"holder", b"Holder"));
    let Some(people::Reply::SignedIn { person, sign_in, .. }) = world.people_answers.last().cloned() else {
        panic!("holder signed in")
    };
    Party { person, sign_in }
}

fn configuration(seed: u64) -> (root::Config, root::Limits) {
    let (mut configuration, mut limits) = person_world_fixture(seed);
    limits.core.tasks.person_stall = STALL;
    // One role for each party and one permitted executor per role plus the worker's charter.
    limits.core.authority.roles = 3;
    limits.core.documents.roles = 3;
    limits.core.authority.executors = 4;
    limits.core.tasks.executor_kinds = 4;
    limits.core.documents.executors = 4;
    limits.core.people.inbox_recipients = 2 * limits.core.tasks.inbox_messages + limits.core.authority.roles + 5;
    limits.core.people.people = 3;
    limits.core.people.sign_ins = 3;
    limits.core.people.holdings = 3;
    let kinds = [
        authority::Executor::Charter(1),
        authority::Executor::Role(0),
        authority::Executor::Role(POLICY.number()),
        authority::Executor::Role(MEMBER.number()),
    ];
    configuration.core.defaults.rules.ceiling.delegation.kinds = kinds.into();
    let policy = &mut configuration.core.seed_projects[0].policy.policy;
    policy.escalation_role = Some(POLICY.number());
    policy.ceiling.delegation.kinds = kinds.into();
    let mut owner = policy.roles[0].clone();
    owner.authority.delegation.kinds = kinds.into();
    policy.roles = Box::new([
        owner.clone(),
        authority::Role { number: POLICY.number(), ..owner.clone() },
        authority::Role { number: MEMBER.number(), ..owner },
    ]);
    (configuration, limits)
}

struct Offer {
    world: World,
    goal: u64,
    task: u64,
    requester: Party,
    addressee: Party,
    policy: Party,
}

fn offer(seed: u64, role_address: bool) -> Offer {
    let (configuration, limits) = configuration(seed);
    let mut world = World::new(Setup { seed, configuration: Some(configuration), limits, ..Setup::default() });
    let parent = world.assignments[0];
    let tasks::Party::Person(person) = live_row(&world, parent.0).requester else { panic!("person's goal") };
    let requester = Party { person, sign_in: world.sign_in.expect("requester signed in") };
    let addressee = party(&mut world, 8);
    let policy = party(&mut world, 9);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 1,
        holdings: Box::new([
            people::Holding { person: requester.person, role: people::Role::Owner },
            people::Holding { person: addressee.person, role: MEMBER },
            people::Holding { person: policy.person, role: POLICY },
        ]),
    })));
    let address = if role_address {
        tasks::PersonAddress::Role(MEMBER.number())
    } else {
        tasks::PersonAddress::Person(addressee.person)
    };
    let mut child = member(&world, tasks::Executor::Person(address), b"check the report");
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let task = delegate(&mut world, parent, 1, child);
    Offer { world, goal: parent.0, task, requester, addressee, policy }
}

fn needs(world: &mut World, party: Party) -> Vec<u64> {
    world
        .domain
        .core_mut()
        .people
        .inbox_needs(party.person)
        .unwrap_or_default()
        .iter()
        .filter(|entry| entry.kind == people::EntryKind::PersonTask)
        .map(|entry| entry.task)
        .collect()
}

fn person_need(world: &mut World, party: Party, request: u64) -> core::documents::InboxEntry {
    use jig_domain_core::reads::{Answer, Filter, Group, PageRow, Read};
    world.sign_in = Some(party.sign_in);
    jig_core_world::reads::ask(
        world,
        request,
        Read::Inbox { group: Group::PersonTasks, filter: Filter { project: Some(1), goal: None }, from: None },
    );
    let Answer::Page(page) = jig_core_world::reads::latest(world) else { panic!("person tasks page") };
    let [PageRow::Inbox(row)] = page.rows.as_slice() else { panic!("one person task need") };
    assert!(page.next.is_none());
    row.clone()
}

#[test]
fn both_holders_inboxes_take_the_person_tasks_current_wait() {
    const SEED: u64 = 190;
    let Offer { mut world, task, requester, addressee, policy, .. } = offer(SEED, false);
    world.reads = Some(jig_core_world::reads::Parties::new(world.store.applied));
    let first = person_need(&mut world, addressee, 7801);
    world.advance(STALL);
    let since = live_row(&world, task).person_wait.unwrap().offer.unwrap().since;
    let bound = core::Bound::At {
        at: skein_lib::Wall::from_nanos(since.as_nanos() + STALL.as_nanos()),
        then: core::AtBound::PassUp,
    };
    assert!(first.at.expect("first arrival") < since);
    let offered = person_need(&mut world, requester, 7802);
    let original = person_need(&mut world, addressee, 7803);
    for entry in [original, offered] {
        assert_eq!(entry.object, core::documents::InboxObject::PersonTask);
        assert_eq!(entry.at, Some(since));
        assert_eq!(entry.bound, Some(bound));
        assert_eq!(entry.cursor, core::documents::Cursor::Inbox { task, object: entry.object, at: Some(since) });
    }
    let waits = world.domain.core().waits(task);
    let [wait] = waits.own.as_slice() else { panic!("one wait, not one per addressee") };
    assert_eq!((wait.since, wait.bound), (since, bound));

    world.advance(STALL);
    // The original addressee can also acquire the final holder's role, without
    // getting two rows with the same cursor or an extra count in the frame.
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 1,
        holdings: Box::new([
            people::Holding { person: requester.person, role: people::Role::Owner },
            people::Holding { person: addressee.person, role: POLICY },
            people::Holding { person: policy.person, role: POLICY },
        ]),
    })));
    let final_entry = person_need(&mut world, policy, 7804);
    assert_eq!(person_need(&mut world, addressee, 7805), final_entry);
    assert_eq!(world.domain.core().people.needs(addressee.person), Some(1));
}

fn answer(world: &mut World, party: Party, key: u8, task: u64, expected: people::Outcome) {
    ask_as(
        world,
        party.sign_in,
        key,
        people::Ask::AnswerPerson {
            project: 1,
            task,
            revision: 0,
            result: people::PersonResult::Report { words: b"a decision".as_slice().into() },
        },
    );
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(expected)));
}

fn retained_answer(world: &World, task: u64) -> tasks::PersonAnswer {
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))]
    else {
        panic!("retained result")
    };
    row.person_answer.expect("retained decision")
}

#[test]
fn the_original_and_offered_person_can_each_win_and_the_census_keeps_both_entries() {
    const SEED: u64 = 188;
    for offered_wins in [false, true] {
        let Offer { mut world, goal, task, requester, addressee, policy } = offer(SEED, false);
        assert_eq!(needs(&mut world, requester), NO_TASKS);
        assert_eq!(needs(&mut world, addressee), [task]);
        world.advance(STALL);
        assert_eq!(needs(&mut world, requester), [task]);
        assert_eq!(needs(&mut world, addressee), [task]);
        assert_eq!(needs(&mut world, policy), NO_TASKS);
        let stall = tasks::MessageKind::PersonStalled { task };
        assert_eq!(live_row(&world, goal).inbox.iter().filter(|word| word.kind == stall).count(), 1);
        world.restart_with(configuration(SEED).0);
        assert!(world.domain.ready());
        assert_eq!(needs(&mut world, requester), [task]);
        assert_eq!(needs(&mut world, addressee), [task]);
        let (winner, late) = if offered_wins { (requester, addressee) } else { (addressee, requester) };
        answer(&mut world, winner, 195, task, people::Outcome::PersonAnswered { task });
        let settled = retained_answer(&world, task);
        assert_eq!(settled.by, winner.person);
        assert!(live_row(&world, goal).inbox.iter().any(|word| word.from == tasks::Party::Task(task)
            && word.kind == tasks::MessageKind::Result(tasks::ResultKind::Report)));
        answer(
            &mut world,
            late,
            196,
            task,
            people::Outcome::Refused(people::Refusal::Ended {
                how: people::Ending::Decided { by: winner.person, at: settled.at },
            }),
        );
        world.close();
    }
}

#[test]
fn a_claim_is_exclusive_at_either_holder_and_the_final_role_replaces_the_person_offer() {
    let Offer { mut world, task, requester, addressee, policy, .. } = offer(189, true);
    let take = people::Ask::TakePerson { project: 1, task };
    let hand_back = people::Ask::HandBackPerson { project: 1, task };
    world.advance(STALL);
    assert_eq!(needs(&mut world, requester), [task]);
    ask_as(&mut world, addressee.sign_in, 195, take.clone());
    assert_eq!(live_row(&world, task).taken_by, Some(addressee.person));
    answer(&mut world, requester, 196, task, people::Outcome::Refused(people::Refusal::Taken { by: addressee.person }));
    ask_as(&mut world, addressee.sign_in, 197, hand_back.clone());
    assert_eq!(live_row(&world, task).taken_by, None);
    world.advance(STALL);
    assert_eq!(needs(&mut world, requester), NO_TASKS);
    assert_eq!(needs(&mut world, addressee), [task]);
    assert_eq!(needs(&mut world, policy), [task]);
    answer(&mut world, requester, 198, task, people::Outcome::Refused(people::Refusal::Unknown));
    answer(&mut world, policy, 199, task, people::Outcome::Refused(people::Refusal::NotWaiting));
    ask_as(&mut world, policy.sign_in, 200, take.clone());
    assert_eq!(live_row(&world, task).taken_by, Some(policy.person));
    answer(&mut world, addressee, 201, task, people::Outcome::Refused(people::Refusal::Taken { by: policy.person }));
    ask_as(&mut world, policy.sign_in, 202, hand_back);
    assert_eq!(live_row(&world, task).taken_by, None);
    ask_as(&mut world, policy.sign_in, 203, take);
    answer(&mut world, policy, 204, task, people::Outcome::PersonAnswered { task });
    let settled = retained_answer(&world, task);
    assert_eq!(settled.by, policy.person);
    answer(
        &mut world,
        addressee,
        205,
        task,
        people::Outcome::Refused(people::Refusal::Ended {
            how: people::Ending::Decided { by: policy.person, at: settled.at },
        }),
    );
    world.close();
}
