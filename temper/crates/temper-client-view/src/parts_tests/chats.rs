use super::fixtures;

use jig_client::contract::*;

use skein_lib::{Env, List, Queue, Time, Token, Wall};

use temper_client_domain::ChatsFilter;

use temper_client_domain::{Domain, Request};

pub(super) use super::fixtures::chats::{TASK, TEXT, line};

fn timed(task: u64, activity: u64, ended: Option<u64>) -> TaskLine {
    let mut line = line();
    line.chip.task = task;
    line.last_activity = Wall::from_nanos(activity);
    if let Some(ended) = ended {
        line.ended_at = Some(Wall::from_nanos(ended));
    }
    line
}

const DRAFT_GOAL: u64 = 42;

struct DraftsPage {
    root: Domain,
    out: Queue<Request>,
    goal_watch: Option<Token>,
    note_reads: u32,
}

impl DraftsPage {
    #[expect(clippy::too_many_lines, reason = "the fixture answers Chats through the public client events")]
    fn new(offered: bool, asking: bool, linked: bool) -> Self {
        use jig_client::{Down, Readable, Up, Watchable};
        use temper_client_domain::{Address, Event};
        use temper_client_protocol_wire::{self as wire, v1 as w};
        let limits = fixtures::domain_limits();
        let env = Env { now: Time::ZERO, wall: fixtures::NOW, limits };
        let source = fixtures::chat::messaging_root();
        let mut me = source.pages().me(source.client()).unwrap().clone();
        if asking {
            me = fixtures::asking_me(me);
        }
        let mut page = Self {
            root: temper_client_domain::unlinked(&limits, fixtures::SEED),
            out: Queue::uncapped(),
            goal_watch: None,
            note_reads: 0,
        };
        for event in [
            Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } },
            Event::Start {
                address: Address::Chats { filter: ChatsFilter::All },
                kept: List::with_capacity(0),
                with: Some(jig_client::With::Transport),
            },
            Event::Client { event: jig_client::Event::Up { up: Up::Served { served: true, versions: None } } },
        ] {
            temper_client_domain::step(&mut page.root, &env, event, &mut page.out);
        }
        for _ in 0_u32..128 {
            let Some(request) = page.out.pop() else { break };
            let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
            let up = match down {
                Down::Open { watch, subject: Watchable::Jig(Subject::Me) } => {
                    Up::Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(me.clone()))) }
                }
                Down::Open { watch, subject: Watchable::Jig(Subject::Settings) } => Up::Watched {
                    watch,
                    event: WatchEvent::Delivered(Delivery::Settings(Box::new(SettingsDocument {
                        newest: 0,
                        settings: List::with_capacity(0),
                        held: List::with_capacity(0),
                        pending: List::with_capacity(0),
                    }))),
                },
                Down::Open { watch, subject: Watchable::Jig(Subject::Tasks { .. }) } => {
                    let mut rows = List::with_capacity(2);
                    let mut chat = line();
                    chat.last_activity = Wall::from_nanos(1);
                    if asking {
                        chat.chip.executor = ChipExecutor::Agent { charter: 37 };
                    }
                    rows.push(chat).unwrap();
                    rows.push(timed(7, 0, None)).unwrap();
                    Up::Watched {
                        watch,
                        event: WatchEvent::Delivered(Delivery::Tasks(Box::new(Tasks { rows, more: false }))),
                    }
                }
                Down::Open { watch, subject: Watchable::Jig(Subject::Chips { tasks }) } => {
                    assert_eq!(tasks.as_slice(), [DRAFT_GOAL], "one newest goal, rows only");
                    assert!(page.goal_watch.is_none(), "one chips watch for the rows");
                    page.goal_watch = Some(watch);
                    let mut chip = line().chip;
                    chip.task = DRAFT_GOAL;
                    chip.title = DocumentText::complete(Text::new(b"Keep working".as_slice().into(), TEXT).unwrap());
                    Up::Watched {
                        watch,
                        event: WatchEvent::Delivered(Delivery::Chips(List::from_boxed(Box::new([chip])))),
                    }
                }
                Down::Open { watch, subject: Watchable::Family { record, .. } }
                    if wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch() == &w::Watch::Dismissed =>
                {
                    let mut dismissed = List::with_capacity(4);
                    if linked {
                        // Newer clock, then higher goal, win; another chat isn't watched.
                        for (chat, goal, at) in [(TASK, 40, 1), (TASK, 41, 2), (TASK, DRAFT_GOAL, 2), (99, 43, 3)] {
                            dismissed
                                .push(w::PersonFact::GoalFrom(
                                    w::GoalFrom::new(
                                        &w::CEILINGS,
                                        w::GoalFromParts { project: fixtures::PROJECT, chat, goal, at },
                                    )
                                    .unwrap(),
                                ))
                                .unwrap();
                        }
                    }
                    Up::FamilyDelivered {
                        watch,
                        record: jig_client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: wire::dismissals(
                                &w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed }).unwrap(),
                            ),
                        },
                    }
                }
                Down::Read { read, what: Readable::Family { record, .. } } => {
                    let reading = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap();
                    let w::Read::Setting(setting) = reading.read() else { continue };
                    let value = if setting.kind() == 7 {
                        let value = w::ProjectWays::new(
                            &w::CEILINGS,
                            w::ProjectWaysParts { drafts: offered, goals: false, changes: false, way: 0 },
                        )
                        .unwrap();
                        let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).unwrap());
                        value.encode(&mut writer).unwrap();
                        Some(writer.finish())
                    } else {
                        None
                    };
                    Up::ReadDone {
                        read,
                        result: jig_client::Readout::Family {
                            family: wire::FORGE,
                            record: Some(wire::setting_answer(
                                &w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value }).unwrap(),
                            )),
                        },
                    }
                }
                Down::Read { read, what: Readable::Jig(Read::Notes { scope, description, after }) } => {
                    assert_eq!(scope, NoteScope::Tasks { project: fixtures::PROJECT });
                    assert_eq!((description, after), (None, None));
                    page.note_reads = page.note_reads.checked_add(1).unwrap();
                    let mut rows = List::with_capacity(3);
                    for task in [TASK, 8, 99] {
                        rows.push(Note {
                            number: 1,
                            revision: Revision { value: 1 },
                            scope: NoteScope::Task { project: fixtures::PROJECT, task },
                            description: DocumentText::complete(
                                Text::new(b"Away mode".as_slice().into(), TEXT).unwrap(),
                            ),
                            body: DocumentText::complete(Text::new(Box::default(), TEXT).unwrap()),
                            writer: NoteAuthor::Task { task, attempt: 1 },
                            tasks: List::with_capacity(0),
                            changed: None,
                            acts: List::with_capacity(0),
                            cut: false,
                        })
                        .unwrap();
                    }
                    Up::ReadDone {
                        read,
                        result: jig_client::Readout::Jig(ReadResult::Value(ReadValue::Notes(Box::new(NotePage {
                            rows,
                            before: None,
                            after: None,
                            cut: false,
                        })))),
                    }
                }
                Down::Read { read, what: Readable::Jig(Read::Tasks { .. }) } => Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: EndedGroup::Rows {
                            rows: List::from_boxed(Box::new([timed(8, 0, Some(0))])),
                            after: None,
                            cut: false,
                        },
                    })))),
                },
                Down::Save { .. }
                | Down::Erase { .. }
                | Down::Send { .. }
                | Down::Open { .. }
                | Down::Continue { .. }
                | Down::Close { .. }
                | Down::Read { .. } => continue,
            };
            temper_client_domain::step(
                &mut page.root,
                &env,
                Event::Client { event: jig_client::Event::Up { up } },
                &mut page.out,
            );
        }
        assert!(page.out.is_empty(), "fixture settled");
        page
    }
}

#[test]
fn watches_the_goals_chips_and_closes_them() {
    use temper_client_domain::{Address, Event, Place};
    let mut page = DraftsPage::new(true, false, true);
    let watch = page.goal_watch.unwrap();
    temper_client_domain::step(
        &mut page.root,
        &Env { now: Time::ZERO, wall: fixtures::NOW, limits: fixtures::domain_limits() },
        Event::Went { place: Place::Address(Address::NewChat { draft: false }) },
        &mut page.out,
    );
    let mut closed = false;
    while let Some(request) = page.out.pop() {
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Close { watch: ended } },
        } = request
        {
            closed |= ended == watch;
        }
    }
    assert!(closed);
    temper_client_domain::step(
        &mut page.root,
        &Env { now: Time::ZERO, wall: fixtures::NOW, limits: fixtures::domain_limits() },
        Event::Went { place: Place::Address(Address::Chats { filter: ChatsFilter::Drafts }) },
        &mut page.out,
    );
    assert!(page.root.pages().chats().unwrap().documents.is_empty());
    let mut reopened = false;
    while let Some(request) = page.out.pop() {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Read {
                            what: jig_client::Readable::Jig(Read::Notes { scope: NoteScope::Tasks { project }, .. }),
                            ..
                        },
                },
        } = request
        {
            reopened |= project == fixtures::PROJECT;
        }
    }
    assert!(reopened, "the next open reads a later document");
}
