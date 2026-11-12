use super::*;
use jig_client::{Readable, shapes};

const CHAT_LABEL: u32 = 7;
const GOAL_LABEL: u32 = 9;

fn offer_goals(fixture: &mut Across) {
    let mut me = fixture.root.pages().me(fixture.root.client()).expect("me delivered").clone();
    for project in me.projects.iter_mut() {
        let mut goal = project.templates.as_slice().first().expect("chat template").clone();
        goal.number = shapes::GOAL;
        goal.label = GOAL_LABEL;
        let chat = project.templates.as_slice().first().expect("chat template").clone();
        project.templates = List::from_boxed(Box::new([chat, goal]));
    }
    fixture.frame_me(me);
}
fn task_subject(fixture: &Across, project: Option<u32>, tracked: Option<bool>) -> Watchable {
    Watchable::Jig(c::Subject::Tasks {
        query: c::TaskQuery {
            project,
            requester: Some(c::Requester::Person(f::PARTY)),
            root: None,
            parent: None,
            label: Some(if tracked.is_some() { GOAL_LABEL } else { CHAT_LABEL }),
            phase: None,
            executor: None,
            tracked,
            order: c::QueryOrder::Activity,
        },
        window: fixture.limits.client.domain.rows,
    })
}
fn inbox_watch(fixture: &Across) -> Token {
    fixture
        .watch(Watchable::Jig(c::Subject::Inbox { window: fixture.root.pages().inbox().expect("Home inbox").window }))
}
fn inbox(fixture: &mut Across, projects: &[u32]) -> Queue<Request> {
    let mut rows = List::uncapped();
    for (index, &project) in projects.iter().enumerate() {
        rows.push(f::inbox_entry(
            c::ObjectKey::Question {
                task: u64::from(project),
                message: f::acts::MESSAGE
                    .checked_add(u64::try_from(index).expect("fixture index fits"))
                    .expect("fixture messages fit"),
            },
            c::InboxGroup::Questions,
            project,
            u64::try_from(index).expect("fixture position fits"),
        ))
        .expect("fixture rows grow");
    }
    let document = crate::tests::documents::inbox(
        rows,
        u32::try_from(projects.len()).expect("fixture count fits"),
        c::Count { value: 0, limited: false },
        0,
        false,
    );
    fixture.up(Up::Watched {
        watch: inbox_watch(fixture),
        event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(document))),
    })
}
fn questions(fixture: &mut Across, out: &Queue<Request>) {
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Read { read, what: Readable::Jig(c::Read::Question { task, message }) },
                },
        } = request
        {
            let mut value = f::acts::question();
            if let c::ReadValue::Question(question) = &mut value {
                question.task.task = *task;
                question.message = *message;
            }
            fixture.up(Up::ReadDone { read: *read, result: Readout::Jig(c::ReadResult::Value(value)) });
        }
    }
}
fn task_line(project: u32, tracked: Option<bool>) -> c::TaskLine {
    let mut line = super::super::chats::chat_line();
    line.project = project;
    line.tracked = if tracked == Some(true) { Some(1) } else { None };
    line.label = if tracked.is_some() { GOAL_LABEL } else { CHAT_LABEL };
    line
}
fn tasks(fixture: &mut Across, watch: Token, project: u32, tracked: Option<bool>, more: bool) {
    let mut rows = List::uncapped();
    rows.push(task_line(project, tracked)).expect("fixture rows grow");
    fixture.up(Up::Watched {
        watch,
        event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks { rows, more }))),
    });
}

#[test]
fn launchpad_is_opt_in_and_applies_on_the_next_home_entry() {
    for enabled in [false, true] {
        let mut fixture = Across::new(&[FIRST, SECOND, THIRD], Address::Accounts);
        offer_goals(&mut fixture);
        assert!(fixture.root.pages().left_off().is_none(), "before Home");
        if enabled {
            fixture.event(Event::Launchpad);
        }
        fixture.go(Address::Home { project: FIRST });
        for project in [FIRST, SECOND, THIRD] {
            assert_eq!(
                fixture.watches(forge::status::watch(project, &fixture.limits.forge).expect("status subject")),
                usize::from(enabled || project == FIRST),
                "enabled={enabled}, project={project}"
            );
        }
        assert_eq!(
            fixture.root.pages().inbox().expect("Home inbox").filter,
            if enabled { Filter::ALL } else { Filter { project: Some(FIRST), kind: None, goal: None } }
        );
        let out = inbox(&mut fixture, &[]);
        let mut filtered = 0_u32;
        for request in &out {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: Down::Read { what: Readable::Jig(c::Read::Inbox { project, .. }), .. },
                    },
            } = request
            {
                assert_eq!(*project, Some(FIRST));
                filtered = filtered.checked_add(1).expect("two inbox reads");
            }
        }
        assert_eq!(filtered, if enabled { 0 } else { 2 });
        for tracked in [None, Some(true)] {
            assert_eq!(fixture.watches(task_subject(&fixture, None, tracked)), usize::from(enabled));
        }
        let mut across_tasks = 0_u32;
        for (_, subject) in &fixture.opened {
            if let Watchable::Jig(c::Subject::Tasks { query: c::TaskQuery { project: None, .. }, .. }) = subject {
                across_tasks = across_tasks.checked_add(1).expect("two left-off reads");
            }
        }
        assert_eq!(across_tasks, if enabled { 2 } else { 0 }, "only the two left-off queries");
        assert_eq!(fixture.root.pages().left_off().is_some(), enabled);
        if !enabled {
            fixture.event(Event::Launchpad);
            assert!(fixture.root.pages().left_off().is_none(), "event waits for entry");
            fixture.go(Address::Accounts);
            fixture.go(Address::Home { project: FIRST });
            assert!(fixture.root.pages().left_off().is_some(), "flag kept for next entry");
        }
    }
}

#[test]
fn left_off_delivers_independently_and_across_reads_leave_and_return() {
    let mut fixture = Across::new(&[FIRST, SECOND, THIRD], Address::Accounts);
    offer_goals(&mut fixture);
    fixture.event(Event::Launchpad);
    fixture.go(Address::Home { project: FIRST });
    let chats = task_subject(&fixture, None, None);
    let goals = task_subject(&fixture, None, Some(true));
    let chat_watch = fixture.watch(chats.clone());
    let goal_watch = fixture.watch(goals.clone());
    let inbox_handle = fixture.root.pages().inbox().expect("Home inbox").entries;
    let owned = [fixture.status_watch(SECOND), fixture.status_watch(THIRD), chat_watch, goal_watch];
    let left = fixture.root.pages().left_off().expect("Home holds left-off handles");
    assert!(left.chats(fixture.root.client()).is_none());
    assert!(left.goals(fixture.root.client()).is_none());
    tasks(&mut fixture, chat_watch, SECOND, None, true);
    let left = fixture.root.pages().left_off().expect("Home holds left-off handles");
    let delivered = left.chats(fixture.root.client()).expect("chats delivered");
    assert!(delivered.more);
    assert_eq!(delivered.rows.len(), 1);
    assert_eq!(delivered.rows.as_slice().first(), Some(&task_line(SECOND, None)));
    assert!(left.goals(fixture.root.client()).is_none(), "goals still unknown");
    tasks(&mut fixture, goal_watch, THIRD, Some(true), false);
    let left = fixture.root.pages().left_off().expect("Home holds left-off handles");
    let delivered = left.goals(fixture.root.client()).expect("goals delivered");
    assert!(!delivered.more);
    assert_eq!(delivered.rows.as_slice().first(), Some(&task_line(THIRD, Some(true))));
    fixture.ack();
    let out = fixture.go(Address::Chat { task: f::acts::TASK });
    for watch in owned {
        assert!(closes(&out).contains(&watch), "left across watch {watch:?}");
    }
    assert!(fixture.root.pages().left_off().is_none());
    assert!(fixture.root.client().document(inbox_handle).is_none(), "Home's unfiltered inbox unwatched; rail's stays");
    fixture.go(Address::Home { project: FIRST });
    for subject in [
        chats,
        goals,
        forge::status::watch(SECOND, &fixture.limits.forge).expect("status subject"),
        forge::status::watch(THIRD, &fixture.limits.forge).expect("status subject"),
    ] {
        assert_eq!(fixture.watches(subject), 2, "reads remade on return");
    }
    let returned = fixture.root.pages().inbox().expect("returned inbox");
    assert_eq!(returned.filter, Filter::ALL);
    assert_ne!(returned.entries, inbox_handle, "new unfiltered inbox watch");
    assert!(fixture.root.client().document(returned.entries).is_some());
    let left = fixture.root.pages().left_off().expect("returned handles");
    assert!(left.chats(fixture.root.client()).is_none(), "old delivery not reused");
    assert!(left.goals(fixture.root.client()).is_none());
}

#[test]
fn launchpad_looks_once_per_project_and_joins_home_and_across_statuses() {
    let mut fixture = Across::new(&[FIRST, SECOND, THIRD], Address::Accounts);
    fixture.event(Event::Launchpad);
    let out = fixture.go(Address::Home { project: FIRST });
    for request in &out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        Down::Read {
                            read,
                            what: Readable::Jig(c::Read::Tasks { query: c::TaskQuery { project: Some(FIRST), .. }, .. }),
                        },
                },
        } = request
        {
            fixture.up(Up::ReadDone {
                read: *read,
                result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(Box::new(c::TaskPage {
                    live: List::uncapped(),
                    live_after: None,
                    ended: c::EndedGroup::None,
                })))),
            });
        }
    }
    let own_chats = fixture.watch(task_subject(&fixture, Some(FIRST), None));
    tasks(&mut fixture, own_chats, FIRST, None, false);
    assert!(looks(&fixture.frame(&[FIRST, SECOND, THIRD])).is_empty(), "forge unsettled");
    let out = fixture.settle();
    assert_eq!(looks(&out).as_ref(), [SECOND, THIRD]);
    let last = last_key(&fixture);
    fixture.told(
        Key { number: last.number.checked_sub(1).expect("two looks"), ..last },
        family_answer(w::Outcome::Looked),
    );
    fixture.told(last, family_answer(w::Outcome::Looked));
    let mut dismissed = List::with_capacity(wire::DISMISSED);
    dismissed
        .push(w::PersonFact::ToBegin(
            w::ToBegin::new(&w::CEILINGS, w::ToBeginParts { project: FIRST }).expect("skip row"),
        ))
        .expect("fixture rows grow");
    let out = fixture.family(
        fixture.watch(forge::dismissed::watch(&fixture.limits.forge).expect("dismissals subject")),
        wire::dismissals(&w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed }).expect("dismissals")),
    );
    assert_eq!(looks(&out).as_ref(), [FIRST], "Home's own look, no across duplicate");
    let own = last_key(&fixture);
    fixture.told(own, family_answer(w::Outcome::Looked));
    assert!(looks(&fixture.settle()).is_empty(), "no second look");
    assert!(looks(&fixture.frame(&[FIRST, SECOND, THIRD])).is_empty(), "reconfiguration keeps looks");
    for (project, at) in [(FIRST, 10), (SECOND, 20), (THIRD, 30)] {
        let mut document = crate::tests::status::document(3, 1, Some(3)).into_parts();
        document.project = project;
        document.previous = Some(5);
        if project == THIRD {
            document.brief = None;
        } else if let Some(brief) = document.brief.take() {
            let mut brief = brief.into_parts();
            brief.at = at;
            brief.lede = b"Ready! More follows.".as_slice().into();
            document.brief = Some(w::StatusBrief::new(&w::CEILINGS, brief).expect("brief"));
        }
        fixture.family(
            fixture.status_watch(project),
            wire::project_status(&w::ProjectStatus::new(&w::CEILINGS, document).expect("status")),
        );
    }
    let out = inbox(&mut fixture, &[THIRD, SECOND]);
    questions(&mut fixture, &out);
    let facts = all_home::facts(&fixture.root, Wall::EPOCH);
    for row in &facts.projects {
        match &row.status {
            Status::Shown(document) => {
                assert_eq!(document.project(), row.project);
                assert_eq!(document.landed(), 3);
                assert_eq!(document.seen(), 1);
            }
            Status::Unknown => panic!("status not shown for {}", row.project),
        }
    }
    assert_eq!(facts.projects.len(), 3);
    assert_eq!(facts.needs, Needs::Count { count: 2, cut: false });
    assert_eq!(all_home::brief(&facts), Some(joined(&[SECOND, FIRST], 20)));
    assert_eq!(all_home::intro(&facts).landed, Landed::Across { count: 6, projects: 3 });
    assert_eq!(pages::home::facts(&fixture.root, FIRST, Wall::EPOCH).needs, Needs::Unknown);
}

#[test]
fn launchpad_decided_prioritizes_root_then_distinct_open_needs_then_me_to_four() {
    use temper_protocol_reaches::{Class, Kind, Reach, Reason};
    let mut fixture = Across::waiting(Address::Accounts);
    fixture.event(Event::Launchpad);
    fixture.go(Address::Home { project: FIRST });
    let out = inbox(&mut fixture, &[THIRD, THIRD, SECOND]);
    questions(&mut fixture, &out);
    fixture.frame(&[FIRST, SECOND, THIRD, FOURTH, FIFTH]);
    let mut projects = List::uncapped();
    for (_, subject) in &fixture.opened {
        if let Watchable::Family { record, .. } = subject
            && let w::Watch::Decided(row) =
                wire::decode_watching(&record.bytes, &w::CEILINGS).expect("forge subject").watch()
        {
            projects.push(row.project()).expect("fixture projects grow");
        }
    }
    assert_eq!(projects.as_slice(), [FIRST, THIRD, SECOND, FOURTH]);
    let reach = Reach {
        kind: Kind::Implementation,
        class: Class::Ask,
        reason: Reason::Kind,
        under: 0,
        sets: 0,
        matched: 0,
        away: false,
    };
    let mut asked = List::with_capacity(1);
    asked
        .push(
            w::Asked::new(
                &w::CEILINGS,
                w::AskedParts {
                    number: 1,
                    task: u64::from(THIRD),
                    at: 0,
                    reach: temper_protocol_reaches::reach_record(&reach),
                },
            )
            .expect("asked row"),
        )
        .expect("fixture rows grow");
    fixture.family(
        fixture.decided_watch(THIRD),
        wire::decisions(
            &w::Decisions::new(
                &w::CEILINGS,
                w::DecisionsParts { project: THIRD, rows: List::with_capacity(0), asked, cut: false },
            )
            .expect("decisions"),
        ),
    );
    let open = fixture
        .root
        .pages()
        .needs()
        .needs(fixture.root.pages().inbox().expect("Home inbox"), fixture.root.forge().approvals());
    let need = *open.first().expect("project 3 need");
    assert_eq!(needs::why(&fixture.root, need), Some(needs::Why::Reach { project: THIRD, reach }));
    let rows = [crate::tests::decided::decision(1, 0)];
    let out = fixture.decisions(THIRD, &rows);
    assert_eq!(task_reads(&out).len(), 1, "across showing task read even while Home's To begin is unknown");
    assert_eq!(task_reads(&out).first().expect("showing task").0, 1);
    assert_eq!(all_home::decided(&fixture.root, Wall::EPOCH).len(), 1);
    assert_eq!(parts::needs_facts(&fixture.root, Wall::EPOCH).showing, 1);
}

#[test]
fn launchpad_findings_and_decided_keep_the_root_once_before_needs() {
    for needs in [Box::<[u32]>::from([SECOND, THIRD, FOURTH, FIFTH]), Box::from([SECOND, FIRST, THIRD, FOURTH, FIFTH])]
    {
        let mut fixture = Across::waiting(Address::Accounts);
        fixture.event(Event::Launchpad);
        fixture.go(Address::Home { project: FIRST });
        inbox(&mut fixture, &needs);
        fixture.frame(&[FIFTH, FOURTH, THIRD, SECOND, FIRST]);
        for kind in ["findings", "decided"] {
            let mut projects = List::uncapped();
            for (_, subject) in &fixture.opened {
                if let Watchable::Family { record, .. } = subject {
                    let watch = wire::decode_watching(&record.bytes, &w::CEILINGS).expect("forge subject");
                    if kind == "findings" {
                        if let w::Watch::Findings(row) = watch.watch() {
                            projects.push(row.project()).expect("fixture projects grow");
                        }
                    } else if let w::Watch::Decided(row) = watch.watch() {
                        projects.push(row.project()).expect("fixture projects grow");
                    }
                }
            }
            assert_eq!(projects.as_slice(), [FIRST, SECOND, THIRD, FOURTH], "{kind}: needs {needs:?}");
        }
    }
}

fn now_reads(out: &Queue<Request>) -> Box<[(u64, Token)]> {
    let mut reads = List::uncapped();
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        Down::Read {
                            read,
                            what:
                                Readable::Jig(c::Read::Notes {
                                    scope: c::NoteScope::Task { project, task },
                                    description,
                                    after,
                                }),
                        },
                },
        } = request
        {
            assert_eq!(*project, FIRST);
            assert_eq!(description.as_ref().expect("description").bytes(), b"now");
            assert_eq!(*after, None);
            reads.push((*task, *read)).expect("fixture reads grow");
        }
    }
    reads.into_boxed()
}
fn now_answer(fixture: &mut Across, read: Token, task: u64, body: &[u8]) -> Queue<Request> {
    let text = c::DocumentText::complete(c::Text::new(body.into(), 128).expect("Now fits"));
    let note = c::Note {
        number: 1,
        revision: c::Revision { value: 1 },
        scope: c::NoteScope::Task { project: FIRST, task },
        description: c::DocumentText::complete(c::Text::new(b"now".as_slice().into(), 3).expect("description")),
        body: text,
        writer: c::NoteAuthor::Task { task, attempt: 1 },
        tasks: List::uncapped(),
        changed: Some(Wall::EPOCH),
        acts: List::uncapped(),
        cut: false,
    };
    fixture.up(Up::ReadDone {
        read,
        result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(c::NotePage {
            rows: List::from_boxed(Box::new([note])),
            cut: false,
            before: None,
            after: None,
        })))),
    })
}
#[test]
fn launchpad_home_reads_roles_nows_one_at_a_time_and_leaves_them() {
    use pages::team;
    for enabled in [false, true] {
        let mut fixture = Across::new(&[FIRST], Address::Accounts);
        if enabled {
            fixture.event(Event::Launchpad);
        }
        fixture.go(Address::Home { project: FIRST });
        let query = c::TaskQuery {
            project: Some(FIRST),
            label: Some(3),
            order: c::QueryOrder::Number,
            requester: None,
            root: None,
            parent: None,
            phase: None,
            executor: None,
            tracked: None,
        };
        let watch = fixture.watch(Watchable::Jig(c::Subject::Tasks { query, window: team::ROLES }));
        let mut first = task_line(FIRST, None);
        first.chip.task = 40;
        first.label = 3;
        first.ended_at = None;
        let mut second = first.clone();
        second.chip.task = 41;
        let rows = List::from_boxed(Box::new([first, second]));
        let out = fixture.up(Up::Watched {
            watch,
            event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks { rows: rows.clone(), more: false }))),
        });
        assert_eq!(now_reads(&out).len(), usize::from(enabled));
        if enabled {
            let reads = now_reads(&out);
            let (task, read) = *reads.first().expect("first Now");
            assert_eq!(task, 40);
            let out = now_answer(&mut fixture, read, task, b"First role's Now\nMore");
            let reads = now_reads(&out);
            assert_eq!(reads.len(), 1, "only the second Now starts");
            let (task, read) = *reads.first().expect("second Now");
            assert_eq!(task, 41);
            let out = now_answer(&mut fixture, read, task, b"Second role's Now");
            assert!(now_reads(&out).is_empty());
            let shown = team::shown_at(&fixture.root, FIRST, Wall::EPOCH);
            let facts = shown.facts().expect("roles don't wait for people");
            assert_eq!(facts.roles.len(), 2);
            assert_eq!(facts.roles.get(0).expect("first").now.as_deref(), Some(b"First role's Now".as_slice()));
            assert_eq!(facts.roles.get(1).expect("second").now.as_deref(), Some(b"Second role's Now".as_slice()));
            for (_, subject) in &fixture.opened {
                if let Watchable::Jig(c::Subject::Task { .. }) = subject {
                    panic!("Team-only card watch stays closed");
                }
            }
            let out = fixture.up(Up::Watched {
                watch,
                event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false }))),
            });
            let reads = now_reads(&out);
            let (task, pending) = *reads.first().expect("refresh Now");
            fixture.go(Address::Work { project: FIRST, filter: crate::WorkFilter::All });
            let out = now_answer(&mut fixture, pending, task, b"Late Now");
            assert!(now_reads(&out).is_empty(), "leaving Home leaves Now reads");
            let shown = team::shown_at(&fixture.root, FIRST, Wall::EPOCH);
            for role in &shown.facts().expect("roles kept on project page").roles {
                assert!(role.now.is_none(), "Nows closed off Home");
            }
        }
    }
}
