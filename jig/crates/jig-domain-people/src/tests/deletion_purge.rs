//! Task and project deletion retire inbox projections while retaining keyed answers.
use super::*;

const PARTY: u64 = 1;
const OTHER_PARTY: u64 = 2;
const SIGN_IN: u64 = 10;
const PROJECT: u32 = 2;
const OTHER_PROJECT: u32 = 3;
const TASK: u64 = 7;
const OTHER_TASK: u64 = 8;
// Two projects each supply a need, reply and result to the same party's prefix.
const PREFIX: u32 = 6;
const LAST_REPLY: u64 = 3;
const READ_REPLY: u64 = 1;

fn entry(task: u64, project: u32, whom: Whom, kind: EntryKind) -> Entry {
    Entry { task, project, whom, kind, at: Wall::from_nanos(task) }
}

fn populated() -> Test {
    let mut test = Test::new(Limits { inbox_entries: PREFIX, ..LIMITS });
    test.signin(PARTY, SIGN_IN, identity(0, PARTY));
    test.signin(OTHER_PARTY, SIGN_IN + 1, identity(0, OTHER_PARTY));
    for (task, project) in [(TASK, PROJECT), (OTHER_TASK, OTHER_PROJECT)] {
        test.send(Event::Roles { project, holdings: Box::new([Holding { person: PARTY, role: Role::Member }]) });
        test.send(Event::Waiting {
            task,
            entries: Box::new([
                entry(task, project, Whom::Person(PARTY), EntryKind::PersonTask),
                entry(
                    task,
                    project,
                    Whom::Role { project, role: Role::Member.number() },
                    EntryKind::Proposal { number: 1 },
                ),
                entry(task, project, Whom::Person(PARTY), EntryKind::Reply { message: 2 }),
            ]),
        });
        test.send(Event::ChatState {
            party: PARTY,
            project,
            task,
            last_reply: LAST_REPLY,
            live: true,
            seen: None,
            replies: Box::new([ReplyRef {
                entry: entry(task, project, Whom::Person(PARTY), EntryKind::ChatReply { message: LAST_REPLY }),
                activity: task,
            }]),
        });
        let request = route(&test.request(
            SIGN_IN,
            u8::try_from(task).unwrap(),
            Ask::MarkSeen { seen: Seen::Conversation { task, through: READ_REPLY } },
        ));
        test.send(Event::ConversationSeen { request, project, last_reply: LAST_REPLY });
        for party in [PARTY, OTHER_PARTY] {
            test.d.remember_result(
                &test.env.limits,
                party,
                ResultRef { task, project, at: Wall::from_nanos(task), position: task, forgettable: None },
            );
        }
    }
    // Fill the two pending places, then record one unsaved pressure observation per project.
    let first = route(&test.request(SIGN_IN, 20, Ask::Stop { project: PROJECT, task: TASK }));
    let second = route(&test.request(SIGN_IN, 21, Ask::Stop { project: OTHER_PROJECT, task: OTHER_TASK }));
    test.request(SIGN_IN, 22, Ask::Stop { project: PROJECT, task: TASK });
    test.request(SIGN_IN, 23, Ask::Stop { project: OTHER_PROJECT, task: OTHER_TASK });
    for request in [first, second] {
        test.send(Event::Decided { request, outcome: Outcome::Again(Again::Busy) });
    }
    test
}

fn assert_other_project_remains(test: &Test) {
    assert_eq!(test.d.project_members(OTHER_PROJECT), Some([Holding { person: PARTY, role: Role::Member }].as_slice()));
    assert_eq!(test.d.need_counts(PARTY), Some(Needs { questions: 0, proposals: 1, escalations: 0, person_tasks: 1 }));
    assert_eq!(test.d.chat_owner(OTHER_TASK), Some(PARTY));
    // Two unread chat turns, one result and one reply notice survive.
    assert_eq!(test.d.counts(&test.env.limits, PARTY), Some(Counts { needs: 2, unread: 4, more_unread: false }));
    assert_eq!(test.d.conversation_seen(PARTY, OTHER_TASK), Some(READ_REPLY));
    assert_eq!(test.d.chat_unread(&test.env.limits, PARTY, OTHER_TASK).unwrap().unread, 2);
    for party in [PARTY, OTHER_PARTY] {
        let results = test.d.cached_results(party).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].task, OTHER_TASK);
        for update in test.d.cached_updates(&test.env.limits, party).unwrap() {
            let task = match update {
                Update::Result(row) => row.task,
                Update::Reply(row) => row.task,
                Update::ChatReply(row) => row.entry.task,
            };
            assert_eq!(task, OTHER_TASK);
        }
    }
    let refusals = test.d.capacity_refusals();
    assert_eq!(refusals.len(), 1);
    for refusal in &refusals {
        assert_eq!(refusal.project, Some(OTHER_PROJECT));
        assert_eq!(refusal.action, CapacityAction::Stop { task: OTHER_TASK });
    }
    assert_eq!(test.d.person(SIGN_IN, test.env.now, test.env.wall), Some(PARTY));
    assert!(test.d.identity(OTHER_PARTY).is_some());
}

#[test]
fn erasing_a_task_drops_its_inbox_entries_positions_and_results() {
    let mut test = populated();
    let out = test.send(Event::EraseTask { task: TASK, project: PROJECT });
    for key in [
        Key::ReadConversation { party: PARTY, task: TASK },
        Key::EndedRead { task: TASK, party: PARTY },
        Key::EndedRead { task: TASK, party: OTHER_PARTY },
    ] {
        assert!(out.contains(&Request::Erase { key }));
    }
    assert!(!test.d.task_needs(PARTY, TASK));
    assert_eq!(test.d.chat_owner(TASK), None);
    assert!(!test.d.read_conversations().contains_key(&(PARTY, TASK)));
    for cached in test.d.cached_waiting(&test.env.limits, PARTY).unwrap() {
        assert_eq!(cached.task, OTHER_TASK);
    }
    assert_other_project_remains(&test);
}

#[test]
fn erasing_a_task_twice_changes_nothing_more() {
    let mut test = populated();
    test.send(Event::EraseTask { task: TASK, project: PROJECT });
    assert!(test.send(Event::EraseTask { task: TASK, project: PROJECT }).is_empty());
    assert_other_project_remains(&test);
}

#[test]
fn dropping_a_project_drops_its_roster_and_role_entries() {
    let mut test = populated();
    let seen = Seen::Results { through: 1 };
    let request = route(&test.request(SIGN_IN, 1, Ask::MarkSeen { seen }));
    test.send(Event::Decided { request, outcome: Outcome::MarkedSeen { seen } });
    let out = test.send(Event::DropProject { project: PROJECT });
    assert!(out.contains(&Request::Erase { key: Key::Roles(PROJECT) }));
    assert!(!test.d.has_project(PROJECT));
    assert_eq!(test.d.project_members(PROJECT), None);
    for party in [PARTY, OTHER_PARTY] {
        for holding in &test.d.project_roles(party).unwrap() {
            assert_ne!(holding.project, PROJECT);
        }
    }
    assert!(!test.d.task_needs(PARTY, TASK));
    assert_eq!(test.d.chat_owner(TASK), None);
    assert!(!test.d.read_conversations().contains_key(&(PARTY, TASK)));
    for cached in test.d.cached_waiting(&test.env.limits, PARTY).unwrap() {
        assert_eq!(cached.project, OTHER_PROJECT);
    }
    assert_eq!(test.d.results_seen(PARTY), Some(1));
    assert_other_project_remains(&test);
    assert!(test.send(Event::DropProject { project: PROJECT }).is_empty());
    assert_other_project_remains(&test);
}

#[test]
fn a_retained_roles_answer_restores_after_its_project_is_dropped() {
    for selected in [false, true] {
        let mut test = Test::new(LIMITS);
        test.d = Domain::new(&LIMITS, Box::new([]), 2, Box::new([]));
        if selected {
            test.d.start_restore(test.env.wall);
        }
        let ask =
            Ask::SetRoles { project: PROJECT, holdings: Box::new([Holding { person: PARTY, role: Role::Owner }]) };
        let key = RequestKey { person: PARTY, key: [1; 16] };
        for record in [
            Stored::Person { number: PARTY, identity: identity(0, PARTY) },
            Stored::Answer {
                key,
                models: Box::new([]),
                ask: Box::new(ask.clone()),
                outcome: Outcome::RolesSet { project: PROJECT },
                at: Wall::EPOCH,
            },
        ] {
            assert!(test.send(Event::Restore { record }).is_empty());
        }
        if selected {
            assert!(test.d.start_link(&test.env));
            test.d.start_sweep();
            assert!(test.d.start_complete(&test.env));
        } else {
            assert!(test.send(Event::Restored).is_empty());
        }
        assert!(test.d.ready());
        assert!(!test.d.has_project(PROJECT));
        test.signin(PARTY, SIGN_IN, identity(0, PARTY));
        assert_eq!(reply(&test.request(SIGN_IN, 1, ask)), Reply::Outcome(Outcome::RolesSet { project: PROJECT }));
        test.env.now = Time::from_nanos(LIMITS.request_retention.as_nanos());
        let mut out = Queue::uncapped();
        fire(&mut test.d, &test.env, &mut out);
        assert_eq!(out.pop(), Some(Request::Erase { key: Key::Answer(key) }));
        assert!(out.is_empty());
    }
}

#[test]
fn deletion_events_wait_until_restore_completes() {
    let mut test = Test::new(LIMITS);
    test.d = Domain::new(&LIMITS, Box::new([]), 2, Box::new([]));
    for record in [
        Stored::Person { number: PARTY, identity: identity(0, PARTY) },
        Stored::Roles { project: PROJECT, holdings: Box::new([Holding { person: PARTY, role: Role::Member }]) },
        Stored::ReadConversation { party: PARTY, task: TASK, through: READ_REPLY },
    ] {
        test.send(Event::Restore { record });
    }
    assert!(test.send(Event::EraseTask { task: TASK, project: PROJECT }).is_empty());
    assert!(test.send(Event::DropProject { project: PROJECT }).is_empty());
    test.send(Event::Restored);
    assert!(test.d.has_project(PROJECT));
    assert_eq!(test.d.conversation_seen(PARTY, TASK), Some(READ_REPLY));
}
