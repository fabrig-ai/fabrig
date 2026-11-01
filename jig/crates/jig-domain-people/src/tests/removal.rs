use super::*;
use alloc::collections::BTreeSet;

const PROJECT: u32 = 1;
const OTHER: u32 = 2;
const OWNER: u64 = 1;
const SERVICE: u64 = 2;
const SHARED: u64 = 3;
const SIGN_IN: u64 = 10;
const SERVICE_SIGN_IN: u64 = 20;
const TASK: u64 = 70;
const OTHER_TASK: u64 = 71;
const ANSWERS: u32 = 16; // room for the retained project, other-project and person-wide fixtures

fn world() -> Test {
    let mut test = Test::new(Limits { requests: ANSWERS, ..LIMITS });
    test.signin(OWNER, SIGN_IN, identity(0, OWNER));
    for project in [PROJECT, OTHER] {
        test.send(Event::Roles { project, holdings: Box::new([Holding { person: OWNER, role: Role::Owner }]) });
    }
    test
}
fn erased(rows: &[Request]) -> BTreeSet<Key> {
    rows.iter().filter_map(|row| if let Request::Erase { key } = row { Some(*key) } else { None }).collect()
}
fn answer_key(person: u64, key: u8) -> Key {
    Key::Answer(RequestKey { person, key: [key; 16] })
}
fn save(test: &mut Test, key: u8, ask: Ask, outcome: Outcome) -> Vec<Request> {
    let request = route(&test.request(SIGN_IN, key, ask));
    test.send(Event::Decided { request, outcome })
}
fn service(test: &mut Test, person: u64, key: u8) -> Vec<Request> {
    let request = route(&test.request(
        SIGN_IN,
        key,
        Ask::MakeService { project: PROJECT, name: b"service".as_slice().into(), role: Role::Member },
    ));
    let mut rows = test.send(Event::MakeService { request, person });
    rows.extend(test.send(Event::Decided { request, outcome: Outcome::ServiceMade { person } }));
    rows
}
fn say(project: u32) -> Ask {
    Ask::Say { project, task: TASK, words: b"words".as_slice().into() }
}

#[test]
fn a_closed_project_erases_its_roles_and_policy_and_no_party_holds_a_role_in_it() {
    let mut test = world();
    project_task(&mut test, OWNER, TASK);
    let rows = test.send(Event::ProjectClosed { project: PROJECT });
    assert_eq!(test.d.counts(&test.env.limits, OWNER), Some(Counts { needs: 0, unread: 0, more_unread: false }));
    assert!(test.d.cached_updates(&test.env.limits, OWNER).unwrap().is_empty());
    assert_eq!(erased(&rows), BTreeSet::from([Key::Roles(PROJECT), Key::Policy(PROJECT)]));
    assert!(!test.d.has_project(PROJECT));
    assert_eq!(test.d.role(OWNER, PROJECT), None);
    assert_eq!(test.d.project_roles(OWNER).unwrap().as_slice(), &[ProjectRole { project: OTHER, role: Role::Owner }]);
    assert_eq!(test.d.role(OWNER, OTHER), Some(Role::Owner));
    let rows = test.request(SIGN_IN, 1, say(PROJECT));
    let routed = rows
        .iter()
        .find_map(|row| if let Request::Route { project, role, .. } = row { Some((*project, *role)) } else { None });
    assert_eq!(routed, Some((PROJECT, None)));
}

#[test]
fn a_closed_projects_services_go_with_their_sign_ins() {
    let mut test = world();
    service(&mut test, SERVICE, 1);
    service(&mut test, SHARED, 2);
    test.send(Event::Roles { project: OTHER, holdings: Box::new([Holding { person: SHARED, role: Role::Member }]) });
    let reply_to = test.to();
    test.send(Event::SignedIn {
        reply_to,
        person: SERVICE,
        sign_in: SERVICE_SIGN_IN,
        identity: test.d.identity(SERVICE).unwrap().clone(),
        kind: Kind::Service,
        held: None,
    });
    project_task(&mut test, SERVICE, TASK);
    let seen = Seen::Results { through: 1 };
    let request = route(&test.request(SERVICE_SIGN_IN, 8, Ask::MarkSeen { seen }));
    test.send(Event::Decided { request, outcome: Outcome::MarkedSeen { seen } });
    let rows = test.send(Event::ProjectClosed { project: PROJECT });
    assert!(erased(&rows).contains(&Key::ReadResults { party: SERVICE }));
    assert!(erased(&rows).contains(&Key::ReadConversation { party: SERVICE, task: TASK }));
    assert_eq!(test.d.chat_owner(TASK), None);
    assert!(erased(&rows).contains(&Key::Person(SERVICE)));
    assert!(erased(&rows).contains(&Key::SignIn(SERVICE_SIGN_IN)));
    assert_eq!(test.d.identity(SERVICE), None);
    assert_eq!(test.d.person(SERVICE_SIGN_IN, test.env.now, test.env.wall), None);
    assert!(test.d.identity(SHARED).is_some());
    assert!(test.d.identity(OWNER).is_some());
    assert_eq!(test.d.role(SHARED, OTHER), Some(Role::Member));
}

#[test]
fn a_closed_project_leaves_no_answer_naming_a_removed_service_or_roster() {
    let mut test = world();
    service(&mut test, SERVICE, 1);
    let reply_to = test.to();
    test.send(Event::SignedIn {
        reply_to,
        person: SERVICE,
        sign_in: SERVICE_SIGN_IN,
        identity: test.d.identity(SERVICE).unwrap().clone(),
        kind: Kind::Service,
        held: None,
    });
    // Services hold no accounts; even this person-wide admission refusal belongs to the service.
    test.request(SERVICE_SIGN_IN, 2, Ask::AccountSignIn { holder: AccountHolder::Own, provider: 1 });
    save(
        &mut test,
        3,
        Ask::SetRoles { project: PROJECT, holdings: Box::new([]) },
        Outcome::RolesSet { project: PROJECT },
    );
    save(
        &mut test,
        4,
        Ask::SetRoles { project: OTHER, holdings: Box::new([Holding { person: SERVICE, role: Role::Member }]) },
        Outcome::RolesSet { project: OTHER },
    );
    save(&mut test, 5, say(OTHER), Outcome::Refused(Refusal::Unknown));
    let rows = test.send(Event::ProjectClosed { project: PROJECT });
    let keys = erased(&rows);
    for key in [answer_key(OWNER, 1), answer_key(SERVICE, 2), answer_key(OWNER, 3), answer_key(OWNER, 4)] {
        assert!(keys.contains(&key), "missing erase {key:?}");
    }
    assert!(!keys.contains(&answer_key(OWNER, 5)));
    test.env.now = Time::from_nanos(LIMITS.request_retention.as_nanos());
    test.env.wall = Wall::from_nanos(LIMITS.request_retention.as_nanos());
    let mut out = Queue::uncapped();
    fire(&mut test.d, &test.env, &mut out);
    assert_eq!(out.pop(), Some(Request::Erase { key: answer_key(OWNER, 5) }));
    assert!(out.is_empty(), "removed answers left no expiry alarms");
}

#[test]
fn a_gone_project_erases_each_answer_and_act_naming_it_and_no_other() {
    let mut test = world();
    test.d = Domain::new(&test.env.limits, Box::new([]), 2, Box::new([]));
    test.send(Event::Restore { record: Stored::Person { number: OWNER, identity: identity(0, OWNER) } });
    test.send(Event::Restore {
        record: Stored::SignIn {
            number: SIGN_IN,
            person: OWNER,
            expires: Wall::from_nanos(LIMITS.sign_in_lifetime.as_nanos()),
        },
    });
    let mut expected = BTreeSet::new();
    let mut retained = BTreeSet::new();
    for (connector, act, remove) in [
        (1, Act::Close { project: PROJECT, task: TASK, grace: None, reason: Box::new([]) }, true),
        (2, Act::Cancel { project: OTHER, task: TASK, grace: None, reason: Box::new([]) }, false),
        (3, Act::RemoveAccount { account: 1 }, false),
    ] {
        let key = ActKey { connector, key: [1; 16] };
        test.send(Event::Restore {
            record: Stored::Acted { key, act, outcome: Outcome::Refused(Refusal::Unknown), at: Wall::EPOCH },
        });
        if remove {
            expected.insert(Key::Acted(key));
        } else {
            retained.insert(Key::Acted(key));
        }
    }
    test.send(Event::Restored);
    for project in [PROJECT, OTHER] {
        test.send(Event::Roles { project, holdings: Box::new([Holding { person: OWNER, role: Role::Owner }]) });
    }
    for (key, ask, project) in [
        (1, say(PROJECT), PROJECT),
        (2, say(OTHER), OTHER),
        (3, Ask::Application { scope: ApplicationScope::Project(PROJECT), connector: 1, digest: [0; 32] }, PROJECT),
        (4, Ask::Application { scope: ApplicationScope::Own, connector: 1, digest: [0; 32] }, OTHER),
        (
            5,
            Ask::Compound {
                project: PROJECT,
                operations: {
                    let mut ops = List::with_capacity(1);
                    ops.push(Operation::Cancel { task: TASK, grace: None, reason: Box::new([]) }).unwrap();
                    ops
                },
            },
            PROJECT,
        ),
        (6, Ask::AccountSignIn { holder: AccountHolder::Own, provider: 1 }, OTHER),
    ] {
        save(&mut test, key, ask, Outcome::Refused(Refusal::Unknown));
        if project == PROJECT {
            expected.insert(answer_key(OWNER, key));
        } else {
            retained.insert(answer_key(OWNER, key));
        }
    }
    let rows = test.send(Event::ProjectGone { project: PROJECT });
    assert_eq!(erased(&rows), expected);
    test.env.now = Time::from_nanos(LIMITS.request_retention.as_nanos());
    test.env.wall = Wall::from_nanos(LIMITS.request_retention.as_nanos());
    let mut expired = BTreeSet::new();
    for _ in 0..ANSWERS {
        let mut out = Queue::uncapped();
        fire(&mut test.d, &test.env, &mut out);
        while let Some(row) = out.pop() {
            if let Request::Erase { key } = row {
                expired.insert(key);
            }
        }
    }
    // Sign-in expiry is a separate deadline; the retained-key expiries are exactly the survivors.
    expired.remove(&Key::SignIn(SIGN_IN));
    assert_eq!(expired, retained);
}

#[test]
fn an_ask_sent_again_after_its_project_is_gone_is_judged_afresh() {
    let mut test = world();
    let ask = say(PROJECT);
    save(&mut test, 1, ask.clone(), Outcome::Said { task: TASK, message: 1 });
    test.send(Event::ProjectClosed { project: PROJECT });
    test.send(Event::ProjectGone { project: PROJECT });
    let request = route(&test.request(SIGN_IN, 1, ask.clone()));
    // The parent judges the absent project as it judges any unknown project today.
    let rows = test.send(Event::Decided { request, outcome: Outcome::Refused(Refusal::Unknown) });
    assert_eq!(reply(&rows), Reply::Outcome(Outcome::Refused(Refusal::Unknown)));
    assert_eq!(reply(&test.request(SIGN_IN, 1, ask)), Reply::Outcome(Outcome::Refused(Refusal::Unknown)));
}

fn project_task(test: &mut Test, party: u64, task: u64) {
    let entry = Entry {
        task,
        project: PROJECT,
        whom: Whom::Person(party),
        kind: EntryKind::Question { message: 1 },
        at: Wall::EPOCH,
    };
    test.send(Event::Waiting { task, entries: Box::new([entry]) });
    test.send(Event::ChatState {
        party,
        project: PROJECT,
        task,
        last_reply: 3,
        live: true,
        seen: None,
        replies: Box::new([ReplyRef {
            activity: 1,
            entry: Entry { kind: EntryKind::ChatReply { message: 3 }, ..entry },
        }]),
    });
    let request = route(&test.request(
        if party == OWNER { SIGN_IN } else { SERVICE_SIGN_IN },
        9,
        Ask::MarkSeen { seen: Seen::Conversation { task, through: 1 } },
    ));
    test.send(Event::ConversationSeen { request, project: PROJECT, last_reply: 3 });
    test.d.remember_result(
        &test.env.limits,
        party,
        ResultRef { task, project: PROJECT, at: Wall::EPOCH, position: task, forgettable: None },
    );
}

#[test]
fn an_erased_task_leaves_no_partys_read_position_or_inbox_entry() {
    let mut test = world();
    test.signin(SERVICE, SERVICE_SIGN_IN, identity(0, SERVICE));
    test.send(Event::Roles {
        project: PROJECT,
        holdings: Box::new([
            Holding { person: OWNER, role: Role::Owner },
            Holding { person: SERVICE, role: Role::Member },
        ]),
    });
    project_task(&mut test, OWNER, TASK);
    // Both parties can have positions for this task after an owner change.
    let request =
        route(&test.request(SERVICE_SIGN_IN, 8, Ask::MarkSeen { seen: Seen::Conversation { task: TASK, through: 1 } }));
    test.send(Event::Decided {
        request,
        outcome: Outcome::MarkedSeen { seen: Seen::Conversation { task: TASK, through: 1 } },
    });
    project_task(&mut test, SERVICE, OTHER_TASK);
    let rows = test.send(Event::TaskErased { task: TASK, project: PROJECT });
    for party in [OWNER, SERVICE] {
        assert!(erased(&rows).contains(&Key::ReadConversation { party, task: TASK }));
        assert!(erased(&rows).contains(&Key::EndedRead { party, task: TASK }));
        assert_eq!(test.d.conversation_seen(party, TASK), Some(0));
        assert!(test.d.cached_results(party).unwrap().iter().all(|entry| entry.task != TASK));
        assert!(test.d.cached_updates(&test.env.limits, party).unwrap().iter().all(|update| match update {
            Update::Result(row) => row.task != TASK,
            Update::Reply(entry) => entry.task != TASK,
            Update::ChatReply(reply) => reply.entry.task != TASK,
        }));
        assert!(!test.d.task_needs(party, TASK));
    }
    assert_eq!(test.d.counts(&test.env.limits, OWNER), Some(Counts { needs: 0, unread: 0, more_unread: false }));
    assert!(test.d.task_needs(SERVICE, OTHER_TASK));
    assert_eq!(test.d.conversation_seen(SERVICE, OTHER_TASK), Some(1));
}

#[test]
fn a_gone_project_erases_a_retained_set_uses_answer_naming_it() {
    let mut test = Test::new(Limits { requests: ANSWERS, words: 128, ..LIMITS });
    test.d = Domain::new(&test.env.limits, Box::new([]), 0, Box::new([]));
    test.send(Event::Restore { record: Stored::Person { number: OWNER, identity: identity(0, OWNER) } });
    for (key, uses) in [
        (1, Some(Box::<[Use]>::from([Use::Project(PROJECT)]))),
        (2, Some(Box::<[Use]>::from([Use::Project(OTHER)]))),
        (3, None),
    ] {
        let restored = test.send(Event::Restore {
            record: Stored::Answer {
                key: RequestKey { person: OWNER, key: [key; 16] },
                ask: Box::new(Ask::AccountChange {
                    holder: AccountHolder::Own,
                    change: AccountChange::SetUses { account: 1, uses },
                }),
                models: Box::new([]),
                outcome: Outcome::AccountChanged,
                at: Wall::EPOCH,
            },
        });
        assert!(restored.is_empty(), "{restored:?}");
    }
    test.send(Event::Restored);
    let rows = test.send(Event::ProjectGone { project: PROJECT });
    assert_eq!(erased(&rows), BTreeSet::from([answer_key(OWNER, 1)]));
    test.env.now = Time::from_nanos(LIMITS.request_retention.as_nanos());
    test.env.wall = Wall::from_nanos(LIMITS.request_retention.as_nanos());
    let mut expired = BTreeSet::new();
    for _ in 0..ANSWERS {
        let mut out = Queue::uncapped();
        fire(&mut test.d, &test.env, &mut out);
        while let Some(row) = out.pop() {
            if let Request::Erase { key } = row {
                expired.insert(key);
            }
        }
    }
    expired.remove(&Key::SignIn(SIGN_IN));
    assert_eq!(expired, BTreeSet::from([answer_key(OWNER, 2), answer_key(OWNER, 3)]));
}
