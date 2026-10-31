//! Page policies exercised through the client boundary.
use crate::state::page;

pub(crate) mod owner_effect_proposals;
pub(crate) mod task_proposals;

use crate::tests::fixtures::{AnsweredByEngine, Ended, Fixture, ReadPage, Served, Watched};
use crate::tests::state::domain::spine::{LIMITS, delivered, drive, frame, open_token, start};
use crate::{
    Event, Request,
    state::{
        address::{Address, Place, ProjectSection, TaskSection},
        page::{
            Arrangement, Page, Section,
            navigation::{Destination, Navigation},
        },
    },
};
use jig_client::Link;
use jig_client::contract::*;
use skein_lib::{List, Time};
fn text(value: &[u8]) -> Text {
    Text::new(Box::from(value), LIMITS.text).unwrap()
}
pub(in crate::tests) fn project(number: u32, chats: bool) -> MeProject {
    MeProject {
        models: List::with_capacity(0),
        templates: if chats { crate::tests::fixtures::chat_templates() } else { List::with_capacity(0) },
        creating: Some(crate::tests::fixtures::creating()),
        project: number,
        name: DocumentText::complete(text(b"shop")),
        role: HeldRole::Policy { role: 1 },
        pool: Some(PoolLedger {
            source: Funder::Pool { project: number, person: 7, period: 1 },
            parent: None,
            numbers: OwnerNumbers { budget: 20, spent: 0, spent_below: 0, reserved: 0 },
            left: 20,
            closed: false,
            began: None,
            next: None,
        }),
    }
}

fn the_frame_counts_needs_exactly_and_caps_updates_on_every_page() {
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    let mut value = frame(7);
    value.needs = Needs { questions: u32::MAX, proposals: 2, escalations: 3, person_tasks: 4 };
    value.needs_total = u32::MAX;
    value.updates = Count { value: 100, limited: false };
    delivered(&mut domain, Time::ZERO, f, value.clone());
    let summary = domain.frame_summary();
    assert_eq!(summary.needs, u64::from(u32::MAX) + 9);
    assert_eq!(summary.updates, Count { value: 99, limited: true });
    assert_eq!(summary.party, Some(7));
    assert_eq!(summary.link, Link::Live);
    drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    assert_eq!(domain.frame_summary().needs, summary.needs);
    value.updates = Count { value: 99, limited: false };
    delivered(&mut domain, Time::ZERO, f, value.clone());
    assert_eq!(domain.frame_summary().updates, Count { value: 99, limited: false });
    value.updates.limited = true;
    delivered(&mut domain, Time::ZERO, f, value);
    assert!(domain.frame_summary().updates.limited);
    drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Project { project: 1, section: ProjectSection::Overview }) },
    );
    assert_eq!(domain.frame_summary().project, Some(1));
    assert_eq!(Address::task(42), Address::Task { task: 42, section: TaskSection::Overview });
}
fn navigation_keeps_the_application_order_and_leaves_out_inapplicable_pages() {
    let mut domain = Fixture::new(&LIMITS);
    let mut navigation = Navigation::new(8);
    assert_eq!(navigation.home, Destination::Jig(Address::Inbox { project: None }));
    let own = Destination::Application { owner: 1, number: 3 };
    let chat = Destination::Jig(Address::Conversations { project: None });
    let deployment = Destination::Jig(Address::Deployment);
    navigation.home = own;
    for destination in [own, chat, deployment] {
        navigation.entries.push(destination).unwrap();
    }
    domain.configure_navigation(navigation).unwrap();
    let f = start(&mut domain, List::with_capacity(0));
    let mut value = frame(7);
    value.deployment_role = None;
    delivered(&mut domain, Time::ZERO, f, value.clone());
    assert_eq!(domain.navigation().entries.as_slice(), &[own, chat, deployment]);
    assert!(domain.navigation().offered(own, domain.frame().document));
    assert!(!domain.navigation().offered(chat, domain.frame().document));
    assert!(!domain.navigation().offered(deployment, domain.frame().document));
    value.projects = List::with_capacity(2);
    value.projects.push(project(1, false)).unwrap();
    value.projects.push(project(2, true)).unwrap();
    value.deployment_role = Some(HeldRole::Policy { role: 1 });
    delivered(&mut domain, Time::ZERO, f, value);
    assert!(domain.navigation().offered(chat, domain.frame().document));
    assert!(
        !domain
            .navigation()
            .offered(Destination::Jig(Address::Conversations { project: Some(1) }), domain.frame().document)
    );
    assert!(domain.navigation().offered(deployment, domain.frame().document));
    assert!(domain.configure_navigation(Navigation::new(0)).is_err());
    let mut fresh = Fixture::new(&LIMITS);
    assert!(fresh.configure_navigation(Navigation::new(9)).is_err());
    assert_eq!(own.place(), Place::Elsewhere);
}
fn arrangement(task: &[TaskSection], project: &[ProjectSection]) -> Arrangement {
    let mut value = Arrangement { task: List::with_capacity(7), project: List::with_capacity(11) };
    for section in task {
        value.task.push(*section).unwrap();
    }
    for section in project {
        value.project.push(*section).unwrap();
    }
    value
}
#[test]
fn an_arrangement_given_before_the_start_is_kept() {
    let mut domain = Fixture::new(&LIMITS);
    assert_eq!(domain.arrangement(), &Arrangement::none());
    assert_eq!(domain.arrangement().task.capacity(), 0);
    assert_eq!(domain.arrangement().project.capacity(), 0);
    let value = arrangement(&[TaskSection::Plan, TaskSection::History], &[ProjectSection::Goals]);
    assert_eq!(domain.configure_arrangement(value.clone()), Ok(()));
    start(&mut domain, List::with_capacity(0));
    assert_eq!(domain.arrangement(), &value);
}
#[test]
fn each_malformed_arrangement_is_refused() {
    let rows = [
        ("seven task sections", arrangement(&[TaskSection::Overview; 7], &[])),
        ("eleven project sections", arrangement(&[], &[ProjectSection::Overview; 11])),
        ("Plan twice", arrangement(&[TaskSection::Plan; 2], &[])),
        ("Goals twice", arrangement(&[], &[ProjectSection::Goals; 2])),
        ("Settings with Resources", arrangement(&[], &[ProjectSection::Settings, ProjectSection::Resources])),
        ("Settings with Policy", arrangement(&[], &[ProjectSection::Settings, ProjectSection::Policy])),
        ("Settings with Roles", arrangement(&[], &[ProjectSection::Settings, ProjectSection::Roles])),
        ("Settings with Spend", arrangement(&[], &[ProjectSection::Settings, ProjectSection::Spend])),
        ("Settings with Recurring", arrangement(&[], &[ProjectSection::Settings, ProjectSection::Recurring])),
    ];
    let mut domain = Fixture::new(&LIMITS);
    for (label, value) in rows {
        assert_eq!(domain.configure_arrangement(value.clone()), Err(value), "{label}");
        assert_eq!(domain.arrangement(), &Arrangement::none(), "{label}");
    }
    start(&mut domain, List::with_capacity(0));
    let value = arrangement(&[TaskSection::Plan], &[]);
    assert_eq!(domain.configure_arrangement(value.clone()), Err(value), "after the start");
}
fn non_work_pages_follow_their_causes_and_missing_tasks_do_not_reveal_visibility() {
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    assert_eq!(domain.page(), &Page::Starting);
    delivered(&mut domain, Time::ZERO, f, frame(7));
    drive(&mut domain, Time::ZERO, Event::Went { place: Place::Missing });
    assert_eq!(domain.page(), &Page::Missing);
    drive(&mut domain, Time::ZERO, jig_client::Up::SignedOut);
    assert_eq!(domain.page(), &Page::SigningIn);
    for end in [WatchEnd::Unknown, WatchEnd::NotAllowed] {
        let mut domain = Fixture::new(&LIMITS);
        let f = start(&mut domain, List::with_capacity(0));
        delivered(&mut domain, Time::ZERO, f, frame(7));
        let opened = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::task(42)) });
        let output = drive(&mut domain, Time::ZERO, Ended { watch: open_token(&opened), end });
        if end == WatchEnd::Unknown {
            let mut read = None;
            for request in &output {
                if let Request::Client {
                    asked:
                        jig_client::Event::Read { handle: token, read: jig_client::Readable::Jig(Read::Task { task: 42 }) },
                } = request
                {
                    assert!(read.replace(*token).is_none());
                }
            }
            let read = read.expect("an absent live task is checked in retained work");
            let Page::Work { address, .. } = domain.page() else {
                panic!("task page awaits its retained read");
            };
            assert_eq!(*address, Address::task(42));
            drive(&mut domain, Time::ZERO, ReadPage { read, result: ReadResult::None(ReadNone::Unknown) });
        } else {
            for request in &output {
                if let Request::Client { asked: jig_client::Event::Read { .. } } = request {
                    panic!("a forbidden task must not probe retained work");
                }
            }
        }
        assert_eq!(domain.page(), &Page::Missing);
    }
    let mut old = Fixture::new(&LIMITS);
    let f = start(&mut old, List::with_capacity(0));
    drive(&mut old, Time::ZERO, Served { served: false });
    assert!(delivered(&mut old, Time::ZERO, f, frame(7)).is_empty());
    assert_eq!(old.page(), &Page::OutOfDate);
    let mut other = Fixture::new(&LIMITS);
    let mut kept = List::with_capacity(1);
    kept.push(crate::state::store::Record::Draft {
        version: crate::state::store::STORE_VERSION,
        party: 8,
        task: 42,
        words: text(b"secret"),
        touched: 1,
    })
    .unwrap();
    let f = start(&mut other, kept);
    delivered(&mut other, Time::ZERO, f, frame(7));
    assert_eq!(other.page(), &Page::SomeoneElse);
}
fn deployment_without_a_role_is_missing_and_never_watched() {
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    let mut value = frame(7);
    value.deployment_role = None;
    delivered(&mut domain, Time::ZERO, f, value);
    let out = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Deployment) });
    assert_eq!(domain.page(), &Page::Missing);
    for request in &out {
        if let Request::Client {
            asked: jig_client::Event::Watch { subject: jig_client::Watchable::Jig(subject), .. },
        } = request
        {
            assert_ne!(*subject, Subject::Deployment);
        }
    }
}

fn task_sections_and_activity_sources_follow_the_executor() {
    use crate::state::{address::TaskSection, page::task};
    for executor in [
        Executor::Agent { charter: 1 },
        Executor::Procedure { connector: 2, kind: 3 },
        Executor::Party { party: 7 },
        Executor::Role { project: 1, role: 1 },
    ] {
        let mut document = super::confirmation::task();
        document.chip.executor = executor;
        let sections = task::offers(&document);
        for section in [TaskSection::Overview, TaskSection::Activity, TaskSection::History, TaskSection::Budget] {
            assert!(task::offered(&sections, section));
        }
        assert!(!task::offered(&sections, TaskSection::Plan));
        let mut domain = Fixture::new(&LIMITS);
        let f = start(&mut domain, List::with_capacity(0));
        let initial = delivered(&mut domain, Time::ZERO, f, frame(7));
        finish_initial_inbox_reads(&mut domain, &initial);
        let opened = drive(
            &mut domain,
            Time::ZERO,
            Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Activity }) },
        );
        let watch = open_token(&opened);
        let requests = drive(
            &mut domain,
            Time::ZERO,
            Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(document))) },
        );
        assert!(domain.task_page().header.is_some());
        match executor {
            Executor::Agent { .. } => {
                assert_eq!(requests.len(), 2);
                let Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            subject: jig_client::Watchable::Jig(Subject::Run { task: 8, turn: None, offset: 0 }),
                            ..
                        },
                } = requests[0]
                else {
                    panic!("agent run watch");
                };
                let Request::Client {
                    asked:
                        jig_client::Event::Read {
                            read: jig_client::Readable::Jig(Read::Activity { task: 8, before: None }),
                            ..
                        },
                } = requests[1]
                else {
                    panic!("agent activity read");
                };
            }
            Executor::Procedure { connector, .. } => {
                assert_eq!(
                    requests.as_ref(),
                    &[Request::ShowPart { part: Part::Panel { connector, task: 8 }, scope: PartScope::Project(1) }]
                );
            }
            Executor::Party { .. } | Executor::Role { .. } => {
                let Request::Client {
                    asked: jig_client::Event::Read { read: jig_client::Readable::Jig(Read::PersonTask { task: 8 }), .. },
                } = requests[0]
                else {
                    panic!("person task card read");
                };
                assert_eq!(requests.len(), 1);
            }
        }
    }
}

fn project_open(section: ProjectSection) -> (Fixture, Box<[Request]>) {
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    let mut value = frame(7);
    let role = project(1, true);
    value.projects = List::with_capacity(1);
    value.projects.push(role).unwrap();
    let out = delivered(&mut domain, Time::ZERO, f, value);
    finish_initial_inbox_reads(&mut domain, &out);
    let opened =
        drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Project { project: 1, section }) });
    (domain, opened)
}

fn goal_line(number: u64, priority: u32, phase: Phase, changed: u64) -> TaskLine {
    let mut chip = super::objects::chip(number);
    chip.phase = phase;
    *chip.details = super::objects::phase_details(phase);
    TaskLine {
        chip,
        tracked: Some(priority),
        phases: OwnerPhaseCounts::ZERO,
        budget: OwnerBudget {
            numbers: OwnerNumbers { budget: 20, spent: 0, spent_below: 0, reserved: 0 },
            left: 20,
            funder: Funder::Pool { project: 1, person: 7, period: 3 },
            deadline: None,
        },
        held_below: 2,
        changed: skein_lib::Wall::from_nanos(changed),
        last_activity: skein_lib::Wall::from_nanos(changed),
        ended_at: None,
        project: 1,
        requester: OwnerRequester::Person(7),
        label: 2,
        flags: TaskFlags { ended_by: EndedBy::Result, conversation: false, listed: true, retention: Retention::Kept },
        waits_on: WaitsOn::Nothing,
        conversation: None,
    }
}

// One process per page family keeps startup costs within this session's share;
// each scenario above still allocates its own state and retains every assertion.
#[test]
fn frame_navigation_and_non_work_pages_follow_owner_facts() {
    the_frame_counts_needs_exactly_and_caps_updates_on_every_page();
    navigation_keeps_the_application_order_and_leaves_out_inapplicable_pages();
    non_work_pages_follow_their_causes_and_missing_tasks_do_not_reveal_visibility();
    deployment_without_a_role_is_missing_and_never_watched();
}

pub(in crate::tests) fn read_named(requests: &[Request], expected: &Read) -> jig_client::Handle {
    for request in requests {
        if let Request::Client {
            asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(what) },
        } = request
            && what == expected
        {
            return *read;
        }
    }
    panic!("expected the named read");
}

#[test]
fn goal_read_from_start_merges_live_watch_by_key() {
    let (mut domain, opened) = project_open(ProjectSection::Goals);
    let watch = open_token(&opened);
    let mut live = List::with_capacity(1);
    live.push(goal_line(9, 2, Phase::Running, 2)).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        Watched { watch, event: WatchEvent::Delivered(Delivery::Tasks(Box::new(Tasks { rows: live, more: false }))) },
    );
    let query = Read::Tasks { query: crate::state::queries::ended_goals(1, 2), live: None, ended: None };
    let mut rows = List::with_capacity(2);
    rows.push(goal_line(9, 99, Phase::Running, 1)).unwrap();
    rows.push(goal_line(8, 1, Phase::Done, 1)).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage {
            read: read_named(&opened, &query),
            result: ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                live: List::with_capacity(0),
                live_after: None,
                ended: EndedGroup::Rows { rows, after: None, cut: false },
            }))),
        },
    );
    assert_eq!(domain.project_page().listed_goals.len(), 2);
    assert_eq!(domain.project_page().listed_goals.get(0).unwrap().chip.task, 9);
    assert_eq!(domain.project_page().listed_goals.get(0).unwrap().tracked, Some(2));
    assert_eq!(domain.project_page().listed_goals.get(1).unwrap().chip.phase, Phase::Done);
    let Some(crate::state::objects::Body::Chip(chip)) =
        &domain.objects().get(domain.objects().find(ObjectKey::Task(9)).unwrap()).unwrap().body
    else {
        panic!("live goal chip");
    };
    assert_eq!(chip.phase, Phase::Running);
}

#[test]
fn owner_started_chat_navigates_from_the_original_request_and_keeps_late_answers_whole() {
    use crate::{
        state::{action::Act, fields::Purpose},
        tree::notices::kept::Kind,
    };
    for away in [false, true] {
        let mut domain = Fixture::new(&LIMITS);
        let token = start(&mut domain, List::with_capacity(0));
        let mut value = frame(7);
        value.projects = List::with_capacity(1);
        value.projects.push(project(1, true)).unwrap();
        delivered(&mut domain, Time::ZERO, token, value);
        drive(
            &mut domain,
            Time::ZERO,
            Event::Act { action: Act::Edit { field: Purpose::NewChat, text: text(b"start") } },
        );
        let sent = drive(&mut domain, Time::ZERO, Event::Act { action: Act::BeginConversation { project: 1 } });
        let mut request = None;
        for row in &sent {
            if let Request::Client {
                asked: jig_client::Event::Send { key: sent, request: jig_client::Ask::Jig(ask), .. },
            } = row
            {
                assert_eq!(**ask, crate::tests::fixtures::chat_request(1, text(b"start")));
                assert!(request.replace(*sent).is_none(), "one original start request");
            }
        }
        let request = request.unwrap();
        if away {
            drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
        }
        let answer =
            Answer::Done(Outcome::Owner(OwnerOutcome::Created { first: u64::MAX, last: u64::MAX, given: None }));
        drive(&mut domain, Time::ZERO, AnsweredByEngine { request, answer: answer.clone() });
        assert!(domain.chats().starting.is_none());
        assert!(domain.fields().get(Purpose::NewChat).unwrap().text.bytes().is_empty());
        if away {
            assert_eq!(domain.notices.ids().len(), 1);
            for (notice, ()) in domain.notices.ids() {
                assert_eq!(
                    domain.notices.get(*notice).unwrap().kind,
                    Kind::Answer { object: ObjectKey::Task(u64::MAX), answer: answer.clone() }
                );
            }
            assert_eq!(domain.page(), &Page::Elsewhere);
        } else {
            let Page::Work { address, .. } = domain.page() else { panic!("actual chat task") };
            assert_eq!(*address, Address::Task { task: u64::MAX, section: TaskSection::Activity });
        }
    }
}

#[test]
fn goal_note_scope_keeps_its_supplied_project_instead_of_the_parent_page() {
    let mut domain = Fixture::new(&LIMITS);
    let frame_watch = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, frame_watch, frame(7));
    finish_initial_inbox_reads(&mut domain, &initial);
    drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Project { project: 1, section: ProjectSection::Notes }) },
    );
    let scope = NoteScope::Task { project: u32::MAX, task: u64::MAX };
    let output = drive(
        &mut domain,
        Time::ZERO,
        Event::Act {
            action: crate::state::action::Act::FilterNotes {
                filter: page::project::NoteFilter { scope: scope.clone(), description: Some(text(b"supplied")) },
            },
        },
    );
    let mut reads = List::with_capacity(1);
    for request in &output {
        if let Request::Client {
            asked:
                jig_client::Event::Read {
                    read: jig_client::Readable::Jig(Read::Notes { scope, description, after }), ..
                },
        } = request
        {
            reads.push((scope.clone(), description.clone(), after.clone())).unwrap();
        }
    }
    assert_eq!(reads.as_slice(), &[(scope.clone(), Some(text(b"supplied")), None)]);
    assert_eq!(domain.project_page().note_filter.as_ref().unwrap().scope, scope);
}

#[test]
fn note_reads_keep_full_writer_identity_and_optional_time_without_changing_the_named_revision() {
    for writer in [NoteAuthor::Party { party: u64::MAX }, NoteAuthor::Task { task: u64::MAX, attempt: u64::MAX }] {
        for changed in [None, Some(skein_lib::Wall::EPOCH)] {
            let (mut domain, opened) = project_open(ProjectSection::Notes);
            let Request::Client { asked: jig_client::Event::Read { handle: read, .. } } = opened[2] else {
                panic!("notes read")
            };
            let mut acts = List::with_capacity(1);
            acts.push(Offered { effect: None, act: ActKind::DeleteNote, admission: Admission::Allowed }).unwrap();
            let note = Note {
                number: 3,
                revision: Revision { value: u64::MAX },
                scope: NoteScope::Project(1),
                description: DocumentText::complete(text(b"description")),
                body: DocumentText { bytes: text(b"body"), cut: true },
                writer,
                tasks: List::with_capacity(0),
                changed,
                acts,
                cut: true,
            };
            let mut rows = List::with_capacity(1);
            rows.push(note.clone()).unwrap();
            drive(
                &mut domain,
                Time::ZERO,
                ReadPage {
                    read,
                    result: ReadResult::Value(ReadValue::Notes(Box::new(NotePage {
                        cut: false,
                        rows,
                        before: None,
                        after: None,
                    }))),
                },
            );
            let id = domain.project_page().notes.as_slice()[0];
            let Some(crate::state::objects::Body::Note { document, .. }) = &domain.objects().get(id).unwrap().body
            else {
                panic!("whole supplied note");
            };
            assert_eq!(**document, note);
            drive(
                &mut domain,
                Time::ZERO,
                Event::Act {
                    action: crate::state::action::Act::Intend { object: id.token(), act: ActKind::DeleteNote },
                },
            );
            assert_eq!(domain.confirmation().unwrap().revision, Some(note.revision));
            let output = drive(&mut domain, Time::ZERO, Event::Act { action: crate::state::action::Act::Confirm });
            let mut requests = List::with_capacity(1);
            for request in &output {
                if let Request::Client { asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. } } =
                    request
                {
                    requests.push(ask.as_ref().clone()).unwrap();
                }
            }
            assert_eq!(
                requests.as_slice(),
                &[Ask::DeleteNote { project: 1, note: 3, scope: note.scope, revision: note.revision }]
            );
        }
    }
}

#[test]
fn task_watch_replacements_preserve_requester_kinds_and_missing_link_facts_with_aggregate_loss() {
    let mut domain = Fixture::new(&LIMITS);
    let frame_watch = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, frame_watch, frame(7));
    finish_initial_inbox_reads(&mut domain, &initial);
    let opened = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::task(8)) });
    let watch = open_token(&opened);
    let mut supplied = super::confirmation::task();
    let mut chip = supplied.chip.clone();
    chip.task = u64::MAX;
    for requester in [
        OwnerRequester::Task(u64::MAX),
        OwnerRequester::Person(u64::MAX),
        OwnerRequester::Deployment { project: u32::MAX },
    ] {
        for cut in [false, true] {
            supplied.requester = requester;
            supplied.cut = cut;
            supplied.inputs = List::with_capacity(2);
            supplied.inputs.push(OwnerTaskLink::Stored { number: u64::MAX, status: None }).unwrap();
            supplied.dependencies = List::with_capacity(2);
            supplied.dependencies.push(OwnerTaskLink::Retained { chip: chip.clone(), at: None }).unwrap();
            supplied.references = List::with_capacity(2);
            supplied.references.push(OwnerTaskLink::Live(chip.clone())).unwrap();
            drive(
                &mut domain,
                Time::ZERO,
                Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(supplied.clone()))) },
            );
            let id = domain.task_page().header.unwrap();
            let Some(crate::state::objects::Body::Task(document)) = &domain.objects().get(id).unwrap().body else {
                panic!("whole watched task");
            };
            assert_eq!(**document, supplied);
            assert_eq!(document.inputs.capacity(), 2);
            assert_eq!(document.dependencies.get(0).unwrap().number(), u64::MAX);
            assert_eq!(document.references.get(0).unwrap().number(), u64::MAX);
            assert_eq!(document.inputs.get(0), Some(&OwnerTaskLink::Stored { number: u64::MAX, status: None }));
            assert_eq!(document.dependencies.get(0), Some(&OwnerTaskLink::Retained { chip: chip.clone(), at: None }));
        }
    }
}

#[test]
fn watched_task_policies_replace_every_channel_without_merging_standing_interests() {
    let mut domain = Fixture::new(&LIMITS);
    let frame_watch = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, frame_watch, frame(7));
    finish_initial_inbox_reads(&mut domain, &initial);
    let opened = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::task(8)) });
    let watch = open_token(&opened);
    let mut supplied = super::confirmation::task();
    for rule in [
        WakeRule::Never,
        WakeRule::Immediate,
        WakeRule::Batch { count: u32::MAX, age: skein_lib::Duration::from_nanos(u64::MAX) },
    ] {
        for results in [ResultsWake::Never, ResultsWake::Each, ResultsWake::LastOrFailure] {
            supplied.wake = OwnerWakePolicy {
                words: rule,
                notices: rule,
                news: rule,
                results,
                questions: true,
                answers: false,
                timers: true,
            };
            supplied.subscriptions.entries = List::with_capacity(2);
            supplied
                .subscriptions
                .entries
                .push(Subscription {
                    number: u64::MAX,
                    kind: SubscriptionKind::Topic { connector: u16::MAX, topic: u64::MAX },
                })
                .unwrap();
            drive(
                &mut domain,
                Time::ZERO,
                Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(supplied.clone()))) },
            );
            let id = domain.task_page().header.unwrap();
            let Some(crate::state::objects::Body::Task(document)) = &domain.objects().get(id).unwrap().body else {
                panic!("whole task policy");
            };
            assert_eq!(**document, supplied);
            assert_eq!(document.subscriptions.entries.capacity(), 2);
            supplied.subscriptions.entries = List::with_capacity(2);
            drive(
                &mut domain,
                Time::ZERO,
                Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(supplied.clone()))) },
            );
            let Some(crate::state::objects::Body::Task(document)) = &domain.objects().get(id).unwrap().body else {
                panic!("new empty interests");
            };
            assert_eq!(**document, supplied);
            assert_eq!(document.subscriptions.entries.capacity(), 2);
            assert!(document.subscriptions.entries.is_empty());
            assert_eq!(document.wake.words, rule);
        }
    }
}

#[test]
fn task_watch_replacements_keep_served_goal_identity_and_whole_contract_loss_independent() {
    let mut domain = Fixture::new(&LIMITS);
    let frame_watch = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, frame_watch, frame(7));
    finish_initial_inbox_reads(&mut domain, &initial);
    let opened = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::task(8)) });
    let watch = open_token(&opened);
    let mut choices = List::with_capacity(2);
    let mut fields = List::with_capacity(2);
    fields.push(FieldRule { name: Box::from(*b"\xff\0/"), max: u32::MAX }).unwrap();
    choices.push(VerdictChoice { code: u32::MAX, words: u32::MAX, followups: u32::MAX, fields }).unwrap();
    let part = OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Result, key: OwnerPartKey::Number(u64::MAX) };
    for (goal, contract) in [
        (Some(8), TaskContractDocument::Report { words: u32::MAX }),
        (Some(u64::MAX), TaskContractDocument::Approval { state: Box::from(*b"\xff\0/"), part }),
        (None, TaskContractDocument::Verdict { choices, cut: true }),
        (Some(0), TaskContractDocument::Change { connector: u16::MAX, kind: u16::MAX, words: u32::MAX }),
    ] {
        let mut row = super::confirmation::task();
        row.goal = goal;
        row.result_contract = contract;
        row.spec.specification.words = DocumentText::complete(text(b""));
        row.cut = false;
        let expected = row.clone();
        drive(&mut domain, Time::ZERO, Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(row))) });
        let id = domain.task_page().header.unwrap();
        let Some(crate::state::objects::Body::Task(retained)) = &domain.objects().get(id).unwrap().body else {
            panic!("whole task remains retained");
        };
        assert_eq!(**retained, expected);
        assert!(!retained.cut, "contract loss is not aggregate task-list loss");
        let sections = page::task::offers(retained);
        assert!(page::task::offered(&sections, TaskSection::Overview));
    }
}

#[test]
fn task_authority_watch_replacements_keep_bitmap_pattern_and_loss_facts_whole() {
    let mut domain = Fixture::new(&LIMITS);
    let frame_watch = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, frame_watch, frame(7));
    finish_initial_inbox_reads(&mut domain, &initial);
    let opened = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::task(8)) });
    let watch = open_token(&opened);
    let mut row = super::confirmation::task();
    row.authority.authority.tools = u64::MAX;
    row.authority.authority.notes = u8::MAX;
    row.authority.authority.delegation.kinds = List::with_capacity(2);
    row.authority.authority.delegation.kinds.push(OwnerAuthorityExecutor::Role(u32::MAX)).unwrap();
    let mut segments = List::with_capacity(2);
    segments.push(Box::from(*b"a/b")).unwrap();
    segments.push(Box::from(*b"\xff\0")).unwrap();
    row.authority.authority.grants = List::with_capacity(2);
    row.authority
        .authority
        .grants
        .push(OwnerGrant {
            connector: u16::MAX,
            kind: u16::MAX,
            pattern: OwnerPattern { segments, last: OwnerPatternEnd::Open(Box::from(*b"c/")) },
        })
        .unwrap();
    for cut in [false, true, false] {
        row.authority.cut = cut;
        row.cut = !cut;
        drive(
            &mut domain,
            Time::ZERO,
            Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(row.clone()))) },
        );
        let id = domain.task_page().header.unwrap();
        let Some(crate::state::objects::Body::Task(actual)) = &domain.objects().get(id).unwrap().body else {
            panic!("whole watched task");
        };
        assert_eq!(**actual, row);
        assert_eq!(actual.authority.cut, cut);
        assert_eq!(actual.cut, !cut);
        assert_eq!(actual.authority.authority.grants.capacity(), 2);
        assert_eq!(actual.authority.authority.grants.get(0).unwrap().pattern.segments.get(0).unwrap().as_ref(), b"a/b");
    }
}

#[test]
fn task_budget_watch_replacements_keep_all_owner_numbers_funders_and_optional_deadlines() {
    let mut domain = Fixture::new(&LIMITS);
    let frame_watch = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, frame_watch, frame(7));
    finish_initial_inbox_reads(&mut domain, &initial);
    let opened = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::task(8)) });
    let watch = open_token(&opened);
    for funder in [
        Funder::Task(u64::MAX),
        Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX - 1 },
        Funder::Recurring { project: u32::MAX, task: u64::MAX, period: u64::MAX - 2 },
        Funder::Period { project: u32::MAX, period: u64::MAX },
    ] {
        for deadline in [None, Some(skein_lib::Wall::EPOCH), Some(skein_lib::Wall::from_nanos(u64::MAX))] {
            let mut row = super::confirmation::task();
            row.budget = OwnerBudget {
                numbers: OwnerNumbers { budget: u64::MAX, spent: u64::MAX - 1, spent_below: 7, reserved: 11 },
                left: 17,
                funder,
                deadline,
            };
            row.authority.authority.deadline = Some(skein_lib::Wall::from_nanos(123));
            drive(
                &mut domain,
                Time::ZERO,
                Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(row.clone()))) },
            );
            let id = domain.task_page().header.unwrap();
            let Some(crate::state::objects::Body::Task(actual)) = &domain.objects().get(id).unwrap().body else {
                panic!("whole watched task");
            };
            assert_eq!(**actual, row);
            assert_eq!(actual.budget.left, 17, "availability is supplied independently of the numbers");
            assert_eq!(actual.budget.deadline, deadline, "authority is a separate supplied fact");
            assert!(page::task::offered(&page::task::offers(actual), TaskSection::Budget));
        }
    }
}

#[test]
fn owner_goal_read_and_watch_keep_all_ten_counters_absent_clock_and_exact_budget_by_key() {
    let (mut domain, opened) = project_open(ProjectSection::Goals);
    let watch = open_token(&opened);
    let mut row = goal_line(9, u32::MAX, Phase::Running, 9);
    row.changed = skein_lib::Wall::EPOCH;
    row.held_below = u32::MAX;
    row.phases = OwnerPhaseCounts {
        waiting: u32::MAX,
        idle: u32::MAX - 1,
        starting: u32::MAX - 2,
        running: u32::MAX - 3,
        backing_off: u32::MAX - 4,
        closing: u32::MAX - 5,
        held: u32::MAX - 6,
        done: u32::MAX - 7,
        failed: u32::MAX - 8,
        cancelled: u32::MAX - 9,
    };
    row.budget.funder = Funder::Task(u64::MAX);
    row.budget.deadline = Some(skein_lib::Wall::from_nanos(u64::MAX));
    let mut rows = List::with_capacity(1);
    rows.push(row.clone()).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        Watched { watch, event: WatchEvent::Delivered(Delivery::Tasks(Box::new(Tasks { rows, more: false }))) },
    );
    let mut duplicate = row.clone();
    duplicate.changed = skein_lib::Wall::EPOCH;
    duplicate.phases = OwnerPhaseCounts::ZERO;
    let mut rows = List::with_capacity(2);
    rows.push(duplicate).unwrap();
    rows.push(goal_line(8, 1, Phase::Done, 1)).unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage {
            read: read_named(
                &opened,
                &Read::Tasks { query: crate::state::queries::ended_goals(1, 2), live: None, ended: None },
            ),
            result: ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                live: List::with_capacity(0),
                live_after: None,
                ended: EndedGroup::Rows { rows, after: None, cut: false },
            }))),
        },
    );
    assert_eq!(domain.project_page().listed_goals.get(0), Some(&row));
    assert_eq!(domain.project_page().listed_goals.len(), 2);
}

#[test]
fn the_inbox_shows_both_groups_of_its_filter() {
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    for project in [None, Some(3)] {
        drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Inbox { project }) });
        assert_eq!(
            domain.sections(),
            &[
                Section::Inbox { group: InboxReadGroup::Needs, project },
                Section::Inbox { group: InboxReadGroup::Updates, project },
            ],
            "{project:?}"
        );
    }
}
#[test]
fn a_task_address_shows_its_own_section_alone() {
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 9, section: TaskSection::Plan }) },
    );
    assert_eq!(domain.sections(), &[Section::Task { task: 9, section: TaskSection::Plan }]);
}
#[test]
fn a_page_not_about_the_work_shows_no_section() {
    let mut domain = Fixture::new(&LIMITS);
    assert_eq!(domain.sections(), &[]);
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    for place in [Place::Elsewhere, Place::Missing] {
        drive(&mut domain, Time::ZERO, Event::Went { place });
        assert_eq!(domain.sections(), &[], "{place:?}");
    }
}

#[test]
fn an_application_page_shows_no_task_after_a_task_page() {
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::task(9)) });
    assert_eq!(domain.shown_task(), Some(9));
    drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    assert_eq!(domain.shown_task(), None);
}

#[test]
fn no_two_windows_of_a_page_share_a_token() {
    use page::{conversations::*, project::*, task::*};
    let mut numbers = [
        crate::state::activity::routing::ACTIVITY_WINDOW,
        INBOX_WINDOW,
        INBOX_UPDATES_WINDOW,
        CONVERSATIONS_WINDOW,
        HISTORY_WINDOW,
        ENDED_WINDOW,
        GOALS_WINDOW,
        NOTES_WINDOW,
        SPEND_WINDOW,
    ];
    numbers.sort_unstable();
    for pair in numbers.windows(2) {
        assert_ne!(pair[0], pair[1], "{numbers:?}");
    }
}
#[test]
fn no_two_reads_of_a_page_share_a_number() {
    use page::{conversations::*, project::*, task::*};
    let mut numbers = [
        crate::state::activity::routing::ACTIVITY_READ,
        INBOX_NEEDS_READ,
        INBOX_UPDATES_READ,
        CONVERSATIONS_READ,
        HISTORY_READ,
        ENDED_READ,
        PERSON_TASK_READ,
        GOALS_READ,
        NOTES_READ,
        SPEND_READ,
        PROJECT_READ,
        crate::state::ui::FINAL_TASK_READ,
    ];
    numbers.sort_unstable();
    for pair in numbers.windows(2) {
        assert_ne!(pair[0], pair[1], "{numbers:?}");
    }
}

#[test]
fn the_inbox_opens_its_watch_once_for_both_groups() {
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    let requests = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Inbox { project: None }) });
    let mut inbox_watches = 0_u32;
    for request in &requests {
        if let Request::Client {
            asked: jig_client::Event::Watch { subject: jig_client::Watchable::Jig(Subject::Inbox { .. }), .. },
        } = request
        {
            inbox_watches += 1;
        }
    }
    assert_eq!(inbox_watches, 1);
}

#[test]
fn an_arranged_task_page_lists_every_section_in_order() {
    let mut domain = Fixture::new(&LIMITS);
    domain
        .configure_arrangement(arrangement(
            &[
                TaskSection::Panels,
                TaskSection::History,
                TaskSection::Budget,
                TaskSection::Plan,
                TaskSection::Overview,
                TaskSection::Activity,
            ],
            &[],
        ))
        .unwrap();
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    let address = Address::Task { task: 8, section: TaskSection::Plan };
    drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(address) });
    assert_eq!(domain.address, address);
    assert_eq!(
        domain.sections(),
        &[
            Section::Task { task: 8, section: TaskSection::Overview },
            Section::Task { task: 8, section: TaskSection::Activity },
            Section::Task { task: 8, section: TaskSection::Plan },
            Section::Task { task: 8, section: TaskSection::History },
            Section::Task { task: 8, section: TaskSection::Budget },
            Section::Task { task: 8, section: TaskSection::Panels },
        ]
    );
}
#[test]
fn settings_stand_for_its_five_on_an_arranged_project_page() {
    let rows: [(&str, ProjectSection, &[ProjectSection]); 2] = [
        (
            "address Settings",
            ProjectSection::Settings,
            &[
                ProjectSection::Recurring,
                ProjectSection::Spend,
                ProjectSection::Roles,
                ProjectSection::Policy,
                ProjectSection::Resources,
            ],
        ),
        ("arranged Settings", ProjectSection::Policy, &[ProjectSection::Settings]),
    ];
    for (label, own, sections) in rows {
        let mut domain = Fixture::new(&LIMITS);
        let mut value = arrangement(&[], sections);
        value.project.push(ProjectSection::Goals).unwrap();
        value.project.push(ProjectSection::Overview).unwrap();
        domain.configure_arrangement(value).unwrap();
        let f = start(&mut domain, List::with_capacity(0));
        delivered(&mut domain, Time::ZERO, f, frame(7));
        let address = Address::Project { project: 1, section: own };
        drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(address) });
        assert_eq!(domain.address, address, "{label}");
        assert_eq!(
            domain.sections(),
            &[
                Section::Project { project: 1, section: ProjectSection::Overview },
                Section::Project { project: 1, section: ProjectSection::Goals },
                Section::Project { project: 1, section: ProjectSection::Settings },
            ],
            "{label}"
        );
    }
}
#[test]
fn an_arranged_task_section_its_document_does_not_offer_is_not_shown() {
    for section in [TaskSection::Plan, TaskSection::Budget, TaskSection::Panels] {
        for own in [TaskSection::Overview, section] {
            let mut domain = Fixture::new(&LIMITS);
            domain.configure_arrangement(arrangement(&[section], &[])).unwrap();
            let f = start(&mut domain, List::with_capacity(0));
            delivered(&mut domain, Time::ZERO, f, frame(7));
            let opened = drive(
                &mut domain,
                Time::ZERO,
                Event::Went { place: Place::Jig(Address::Task { task: 8, section: own }) },
            );
            let value = Section::Task { task: 8, section };
            assert!(domain.shows(value), "{own:?}, {section:?}: loading");
            let mut document = super::confirmation::task();
            document.chip.executor = Executor::Party { party: 7 };
            document.budget.numbers = OwnerNumbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 };
            drive(
                &mut domain,
                Time::ZERO,
                Watched {
                    watch: open_token(&opened),
                    event: WatchEvent::Delivered(Delivery::Task(Box::new(document))),
                },
            );
            assert!(domain.sections().contains(&value), "{own:?}, {section:?}: fixed list");
            assert!(!domain.shows(value), "{own:?}, {section:?}: document received");
        }
    }
}

fn source_requests(requests: &[Request]) -> (List<Subject>, List<Read>) {
    let mut watches = List::with_capacity(LIMITS.watches);
    let mut reads = List::with_capacity(LIMITS.reads);
    for request in requests {
        if let Request::Client {
            asked: jig_client::Event::Watch { subject: jig_client::Watchable::Jig(subject), .. },
        } = request
        {
            watches.push(subject.clone()).unwrap();
        }
        if let Request::Client { asked: jig_client::Event::Read { read: jig_client::Readable::Jig(what), .. } } =
            request
        {
            reads.push(what.clone()).unwrap();
        }
    }
    (watches, reads)
}
fn arranged_task() -> (Fixture, Box<[Request]>) {
    let mut domain = Fixture::new(&LIMITS);
    domain
        .configure_arrangement(arrangement(
            &[
                TaskSection::Overview,
                TaskSection::Activity,
                TaskSection::Plan,
                TaskSection::History,
                TaskSection::Budget,
                TaskSection::Panels,
            ],
            &[],
        ))
        .unwrap();
    let f = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, f, frame(7));
    finish_initial_inbox_reads(&mut domain, &initial);
    let opened = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::task(8)) });
    for request in &opened {
        if let Request::Client { asked: jig_client::Event::Unwatch { handle: watch } } = request {
            drive(&mut domain, Time::ZERO, Ended { watch: *watch, end: WatchEnd::Closed });
        }
    }
    domain.reclaim();
    (domain, opened)
}
#[test]
fn an_arranged_task_page_opens_its_watch_once() {
    let rows: [(Executor, Phase, &[Subject], &[Read]); 3] = [
        (
            Executor::Agent { charter: 1 },
            Phase::Running,
            &[Subject::Tree { task: 8, window: LIMITS.rows }, Subject::Run { task: 8, turn: None, offset: 0 }],
            &[Read::Activity { task: 8, before: None }],
        ),
        (
            Executor::Party { party: 7 },
            Phase::Held,
            &[Subject::Tree { task: 8, window: LIMITS.rows }],
            &[Read::PersonTask { task: 8 }],
        ),
        (
            Executor::Party { party: 7 },
            Phase::Done,
            &[],
            &[Read::Ended { task: 8, after: None }, Read::PersonTask { task: 8 }],
        ),
    ];
    for (executor, phase, watches, reads) in rows {
        let (mut domain, opened) = arranged_task();
        let (actual_watches, actual_reads) = source_requests(&opened);
        assert_eq!(actual_watches.as_slice(), &[Subject::Task { task: 8 }], "{executor:?}");
        assert_eq!(actual_reads.as_slice(), &[Read::History { task: 8, subtree: false, after: None }], "{executor:?}");
        let mut document = super::confirmation::task();
        document.chip.executor = executor;
        document.chip.phase = phase;
        document.context.as_mut().unwrap().impact.tasks.waiting = 1;
        let delivery =
            Watched { watch: open_token(&opened), event: WatchEvent::Delivered(Delivery::Task(Box::new(document))) };
        let requests = drive(&mut domain, Time::ZERO, delivery.clone());
        let (actual_watches, actual_reads) = source_requests(&requests);
        assert_eq!(actual_watches.as_slice(), watches, "{executor:?}");
        assert_eq!(actual_reads.as_slice(), reads, "{executor:?}");
        let again = drive(&mut domain, Time::ZERO, delivery);
        let (actual_watches, actual_reads) = source_requests(&again);
        assert_eq!(actual_watches.as_slice(), &[], "{executor:?}: repeated delivery");
        assert_eq!(actual_reads.as_slice(), &[], "{executor:?}: repeated delivery");
    }
}
#[test]
fn an_arranged_project_page_opens_the_goals_watch_once() {
    let mut domain = Fixture::new(&LIMITS);
    domain.configure_arrangement(arrangement(&[], &[ProjectSection::Goals, ProjectSection::Overview])).unwrap();
    let f = start(&mut domain, List::with_capacity(0));
    let mut me = frame(7);
    me.projects = crate::tests::tree::work_pages::list([project(1, true)]);
    let initial = delivered(&mut domain, Time::ZERO, f, me);
    finish_initial_inbox_reads(&mut domain, &initial);
    let requests = drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Project { project: 1, section: ProjectSection::Overview }) },
    );
    let (watches, reads) = source_requests(&requests);
    assert_eq!(
        watches.as_slice(),
        &[
            Subject::Tasks { query: crate::state::queries::goals(1), window: LIMITS.rows },
            Subject::Inbox { window: LIMITS.rows }
        ]
    );
    assert_eq!(
        reads.as_slice(),
        &[
            Read::Project { project: 1 },
            Read::Tasks { query: crate::state::queries::ended_goals(1, 2), live: None, ended: None }
        ]
    );
}

#[test]
fn an_object_no_source_names_is_owed_while_deciding() {
    let mut domain = Fixture::new(&LIMITS);
    let source = crate::state::objects::Source::Window { number: 0 };
    let id = domain.objects.name(ObjectKey::Task(8), source).unwrap();
    domain.objects.get_mut(id).unwrap().card =
        crate::state::objects::Card::Deciding { request: None, progress: crate::state::intents::Progress::Confirming };
    domain.objects.replace(source, &[]);
    assert!(domain.owes(id));
}

fn application_page() -> Fixture {
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, f, frame(7));
    finish_initial_inbox_reads(&mut domain, &initial);
    let left = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    for request in &left {
        if let Request::Client { asked: jig_client::Event::Unwatch { handle: watch } } = request {
            drive(&mut domain, Time::ZERO, Ended { watch: *watch, end: WatchEnd::Closed });
        }
    }
    domain.reclaim();
    domain
}
fn show_sections(domain: &mut Fixture, values: &[Section]) -> Box<[Request]> {
    let mut sections = List::with_capacity(u32::try_from(values.len()).unwrap());
    for section in values {
        sections.push(*section).unwrap();
    }
    let source = crate::state::objects::Source::Window { number: crate::state::application::WINDOW };
    let mut objects = List::with_capacity(LIMITS.objects);
    for (key, _) in domain.objects().ids() {
        if domain.objects().named(*key, source) {
            objects.push(*key).unwrap();
        }
    }
    drive(domain, Time::ZERO, Event::Named { objects, sections })
}
#[test]
fn an_application_page_naming_a_tasks_plan_opens_its_watch() {
    let mut domain = application_page();
    let opened = show_sections(&mut domain, &[Section::Task { task: 8, section: TaskSection::Plan }]);
    let (watches, reads) = source_requests(&opened);
    assert_eq!(watches.as_slice(), &[Subject::Task { task: 8 }]);
    assert_eq!(reads.as_slice(), &[]);
    let id = open_token(&opened);
    assert_eq!(domain.handles.watches.get(&id).unwrap().owner, crate::state::handles::Holder::Sections);
}
#[test]
fn naming_objects_anew_never_closes_a_sections_watch() {
    let mut domain = application_page();
    let opened = show_sections(&mut domain, &[Section::Task { task: 8, section: TaskSection::Plan }]);
    let watch = open_token(&opened);
    for keys in [&[ObjectKey::Task(8)][..], &[][..]] {
        let mut objects = List::with_capacity(u32::try_from(keys.len()).unwrap());
        for key in keys {
            objects.push(*key).unwrap();
        }
        let mut sections = List::with_capacity(1);
        sections.push(Section::Task { task: 8, section: TaskSection::Plan }).unwrap();
        let requests = drive(&mut domain, Time::ZERO, Event::Named { objects, sections });
        assert!(!requests.contains(&Request::Client { asked: jig_client::Event::Unwatch { handle: watch } }));
    }
    assert!(
        !show_sections(&mut domain, &[Section::Task { task: 8, section: TaskSection::Plan }])
            .contains(&Request::Client { asked: jig_client::Event::Unwatch { handle: watch } })
    );
}
#[test]
fn a_section_naming_a_second_task_is_not_shown() {
    let mut domain = application_page();
    let first = Section::Task { task: 8, section: TaskSection::Plan };
    let second = Section::Task { task: 9, section: TaskSection::Plan };
    show_sections(&mut domain, &[first, second]);
    assert_eq!(domain.sections(), &[first]);
    assert!(!domain.shows(second));
}
#[test]
fn an_empty_list_closes_every_sections_watch() {
    let mut domain = application_page();
    let opened = show_sections(&mut domain, &[Section::Task { task: 8, section: TaskSection::Plan }]);
    assert_eq!(
        show_sections(&mut domain, &[]).as_ref(),
        &[Request::Client { asked: jig_client::Event::Unwatch { handle: open_token(&opened) } }]
    );
}

#[test]
fn naming_sections_anew_never_closes_an_objects_watch() {
    let mut domain = application_page();
    let mut objects = List::with_capacity(1);
    objects.push(ObjectKey::Task(8)).unwrap();
    let opened = drive(&mut domain, Time::ZERO, Event::Named { objects, sections: List::with_capacity(0) });
    let watch = open_token(&opened);
    show_sections(&mut domain, &[Section::Task { task: 8, section: TaskSection::Plan }]);
    assert!(
        !show_sections(&mut domain, &[])
            .contains(&Request::Client { asked: jig_client::Event::Unwatch { handle: watch } })
    );
}
#[test]
fn replacing_sections_keeps_their_shared_subject_watch() {
    let mut domain = application_page();
    show_sections(&mut domain, &[Section::Task { task: 8, section: TaskSection::Overview }]);
    let requests = show_sections(&mut domain, &[Section::Task { task: 8, section: TaskSection::Plan }]);
    assert_eq!(requests.as_ref(), &[]);
}
#[test]
fn naming_the_same_sections_opens_nothing_again() {
    let mut domain = application_page();
    let sections = [Section::Task { task: 8, section: TaskSection::Plan }];
    show_sections(&mut domain, &sections);
    assert_eq!(show_sections(&mut domain, &sections).as_ref(), &[]);
}
#[test]
fn leaving_an_application_page_closes_its_sections() {
    let mut domain = application_page();
    let opened = show_sections(&mut domain, &[Section::Task { task: 8, section: TaskSection::Plan }]);
    let requests = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    assert_eq!(
        requests.as_ref(),
        &[Request::Client { asked: jig_client::Event::Unwatch { handle: open_token(&opened) } }, Request::PartsLeft]
    );
}

#[test]
fn a_section_whose_task_is_not_allowed_says_it_is_missing() {
    let mut domain = application_page();
    let section = Section::Task { task: 8, section: TaskSection::Plan };
    let opened = show_sections(&mut domain, &[section]);
    drive(&mut domain, Time::ZERO, Ended { watch: open_token(&opened), end: WatchEnd::NotAllowed });
    assert_eq!(
        domain.section_status(section),
        Some(page::SectionStatus { behind: false, excess: 0, refused: None, missing: true })
    );
    assert_eq!(domain.page(), &Page::Elsewhere);
}
#[test]
fn a_sections_dropped_watch_shows_that_section_behind() {
    let mut domain = application_page();
    let section = Section::Task { task: 8, section: TaskSection::Plan };
    let opened = show_sections(&mut domain, &[section, Section::Deployment]);
    drive(&mut domain, Time::ZERO, Ended { watch: open_token(&opened), end: WatchEnd::Dropped });
    let clear = page::SectionStatus { behind: false, excess: 0, refused: None, missing: false };
    assert_eq!(domain.section_status(section), Some(page::SectionStatus { behind: true, ..clear }));
    assert_eq!(domain.section_status(Section::Deployment), Some(clear));
}
#[test]
fn a_sections_missing_read_keeps_the_application_page() {
    let mut domain = application_page();
    let section = Section::Project { project: 1, section: ProjectSection::Goals };
    let opened = show_sections(&mut domain, &[section]);
    let mut project_read = None;
    for request in &opened {
        if let Request::Client {
            asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Project { .. }) },
        } = request
        {
            project_read = Some(*read);
        }
    }
    let read = project_read.unwrap();
    drive(&mut domain, Time::ZERO, ReadPage { read, result: ReadResult::None(ReadNone::Unknown) });
    assert_eq!(
        domain.section_status(section),
        Some(page::SectionStatus { behind: false, excess: 0, refused: None, missing: true })
    );
    assert_eq!(domain.page(), &Page::Elsewhere);
}

fn section_with_an_unrelated_section() -> (Fixture, Section) {
    let mut domain = application_page();
    let section = Section::Task { task: 8, section: TaskSection::Plan };
    show_sections(&mut domain, &[section, Section::Deployment]);
    page::shown::context_subject(&mut domain, &Subject::Tree { task: 8, window: LIMITS.rows });
    (domain, section)
}
#[test]
fn a_sections_limit_refusal_stays_with_that_section() {
    let (mut domain, section) = section_with_an_unrelated_section();
    page::routing::refuse(&mut domain, Limit::Reads);
    assert_eq!(domain.section_status(section).unwrap().refused, Some(Limit::Reads));
    assert_eq!(domain.section_status(Section::Deployment).unwrap().refused, None);
}
#[test]
fn a_sections_excess_rows_stay_with_that_section() {
    let (mut domain, section) = section_with_an_unrelated_section();
    page::routing::excess(&mut domain, 3);
    assert_eq!(domain.section_status(section).unwrap().excess, 3);
    assert_eq!(domain.section_status(Section::Deployment).unwrap().excess, 0);
}

pub(in crate::tests) fn finish_initial_inbox_reads(domain: &mut Fixture, out: &[Request]) {
    for request in out {
        if let Request::Client {
            asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Inbox { .. }) },
        } = request
        {
            drive(
                domain,
                Time::ZERO,
                ReadPage {
                    read: *read,
                    result: ReadResult::Value(ReadValue::OwnerInbox(Box::new(OwnerInboxPage {
                        selection: None,
                        cut: false,
                        rows: List::with_capacity(0),
                        before: None,
                        after: None,
                    }))),
                },
            );
        }
    }
    domain.reclaim();
}

#[test]
fn task_sections_and_activity_keep_the_executor_facts() {
    task_sections_and_activity_sources_follow_the_executor();
}
