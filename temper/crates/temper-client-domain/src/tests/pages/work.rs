use crate::pages::work::{self, Belongs, Card, Change, Chat, Facts, Filters, Goal, Item, Landing, Where};
use skein_lib::{List, Set, Wall};

const NOW: u64 = 1_209_600;
const WEEK_AGO: u64 = 604_800;

const fn wall(seconds: u64) -> Wall {
    Wall::from_nanos(seconds.saturating_mul(1_000_000_000))
}

fn list<T: Clone>(items: &[T]) -> List<T> {
    List::from_boxed(items.into())
}

fn facts(goals: &[Goal], changes: &[Change], chats: &[Chat]) -> Facts {
    let mut landed_default = List::uncapped();
    let mut seen = Set::with_capacity(u32::try_from(changes.len()).unwrap());
    for change in changes {
        if !seen.insert(change.task).unwrap() {
            continue;
        }
        match change.at {
            Where::Landed { at, default_branch: true } => {
                landed_default
                    .push(Landing {
                        signed: change.signed.clone(),
                        task: change.task,
                        commit: [0xab; 32],
                        at,
                        belongs: change.belongs,
                    })
                    .unwrap();
            }
            Where::Landed { default_branch: false, .. } | Where::Queued { .. } => {}
        }
    }
    Facts { goals: list(goals), changes: list(changes), chats: list(chats), landed_default, ..Facts::default() }
}

fn goal(task: u64, live: bool, activity: u64) -> Goal {
    Goal { task, live, activity: wall(activity) }
}

fn chat(task: u64, activity: u64, draft: bool) -> Chat {
    Chat { task, activity: wall(activity), draft }
}

fn queued(task: u64, place: u32, since: u64, belongs: Belongs) -> Change {
    Change { signed: None, task, at: Where::Queued { place, since: wall(since) }, belongs }
}

fn landed(task: u64, at: u64, belongs: Belongs) -> Change {
    Change { signed: None, task, at: Where::Landed { at: wall(at), default_branch: true }, belongs }
}

fn card(item: Item, changes: &[u64]) -> Card {
    let mut rows = List::with_capacity(u32::try_from(changes.len()).unwrap());
    for task in changes {
        rows.push(work::Row { task: *task, signed: None }).unwrap();
    }
    Card { item, changes: rows }
}

struct Expected {
    in_flight: List<Card>,
    drafts: List<Card>,
    landed: List<u64>,
    filters: Filters,
}

fn work(in_flight: &[Card], drafts: &[Card], landed: &[u64], filters: Filters) -> Expected {
    Expected { in_flight: list(in_flight), drafts: list(drafts), landed: list(landed), filters }
}

struct Case {
    name: &'static str,
    facts: Facts,
    wall: Wall,
    want: Expected,
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table compares Work for each placement rule and the whole project")]
fn each_item_is_placed_by_its_own_facts() {
    let cases = [
        Case {
            name: "empty project",
            facts: Facts::default(),
            wall: wall(NOW),
            want: work(&[], &[], &[], Filters::default()),
        },
        Case {
            name: "live goals get cards; ended goals still count",
            facts: facts(&[goal(1, true, 10), goal(2, false, 20)], &[], &[]),
            wall: wall(NOW),
            want: work(&[card(Item::Goal(1), &[])], &[], &[], Filters { goals: 2, ..Filters::default() }),
        },
        Case {
            name: "a live goal's queued change has no card of its own",
            facts: facts(&[goal(1, true, 10)], &[queued(11, 1, 20, Belongs::Goal(1))], &[]),
            wall: wall(NOW),
            want: work(&[card(Item::Goal(1), &[11])], &[], &[], Filters { goals: 1, changes: 1, ..Filters::default() }),
        },
        Case {
            name: "an ended goal's queued change has its own card",
            facts: facts(&[goal(1, false, 10)], &[queued(11, 1, 20, Belongs::Goal(1))], &[]),
            wall: wall(NOW),
            want: work(
                &[card(Item::Change(11), &[])],
                &[],
                &[],
                Filters { goals: 1, changes: 1, ..Filters::default() },
            ),
        },
        Case {
            name: "an unlisted goal's queued change has its own card",
            facts: facts(&[], &[queued(11, 1, 20, Belongs::Goal(1))], &[]),
            wall: wall(NOW),
            want: work(&[card(Item::Change(11), &[])], &[], &[], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "a listed chat holds its queued change",
            facts: facts(&[], &[queued(11, 1, 20, Belongs::Chat(1))], &[chat(1, 10, false)]),
            wall: wall(NOW),
            want: work(&[card(Item::Chat(1), &[11])], &[], &[], Filters { changes: 1, chats: 1, ..Filters::default() }),
        },
        Case {
            name: "an unlisted chat's queued change has its own card",
            facts: facts(&[], &[queued(11, 1, 20, Belongs::Chat(1))], &[]),
            wall: wall(NOW),
            want: work(&[card(Item::Change(11), &[])], &[], &[], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "a queued change belonging to nothing has its own card",
            facts: facts(&[], &[queued(11, 1, 20, Belongs::Nothing)], &[]),
            wall: wall(NOW),
            want: work(&[card(Item::Change(11), &[])], &[], &[], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "a chat with only old landed work counts without an In flight card",
            facts: facts(&[], &[landed(11, WEEK_AGO - 1, Belongs::Chat(1))], &[chat(1, 10, false)]),
            wall: wall(NOW),
            want: work(&[], &[], &[], Filters { changes: 1, chats: 1, ..Filters::default() }),
        },
        Case {
            name: "a draft with a queued change belongs to both sections",
            facts: facts(&[], &[queued(11, 1, 20, Belongs::Chat(1))], &[chat(1, 10, true)]),
            wall: wall(NOW),
            want: work(
                &[card(Item::Chat(1), &[11])],
                &[card(Item::Draft(1), &[])],
                &[],
                Filters { drafts: 1, changes: 1, chats: 1, goals: 0 },
            ),
        },
        Case {
            name: "a draft alone is no chat with work; an empty chat gets no card",
            facts: facts(&[], &[], &[chat(1, 10, true), chat(2, 20, false)]),
            wall: wall(NOW),
            want: work(&[], &[card(Item::Draft(1), &[])], &[], Filters { drafts: 1, ..Filters::default() }),
        },
        Case {
            name: "a goal holds queued changes by place then all its landed changes newest first",
            facts: facts(
                &[goal(1, true, 10)],
                &[
                    landed(14, WEEK_AGO - 1, Belongs::Goal(1)),
                    queued(12, 2, 50, Belongs::Goal(1)),
                    landed(13, NOW, Belongs::Goal(1)),
                    queued(11, 1, 20, Belongs::Goal(1)),
                    landed(15, NOW + 1, Belongs::Goal(1)),
                ],
                &[],
            ),
            wall: wall(NOW),
            want: work(
                &[card(Item::Goal(1), &[11, 12, 15, 13, 14])],
                &[],
                &[13],
                Filters { goals: 1, changes: 5, ..Filters::default() },
            ),
        },
        Case {
            name: "a chat holds queued changes by place then landed changes newest first",
            facts: facts(
                &[],
                &[
                    landed(14, NOW - 1, Belongs::Chat(1)),
                    queued(12, 2, 50, Belongs::Chat(1)),
                    landed(13, NOW, Belongs::Chat(1)),
                    queued(11, 1, 20, Belongs::Chat(1)),
                ],
                &[chat(1, 10, false)],
            ),
            wall: wall(NOW),
            want: work(
                &[card(Item::Chat(1), &[11, 12, 13, 14])],
                &[],
                &[13, 14],
                Filters { changes: 4, chats: 1, ..Filters::default() },
            ),
        },
        Case {
            name: "In flight uses each card's own activity, never its changes' times or queue place",
            facts: facts(
                &[goal(1, true, 20)],
                &[
                    queued(11, 1, 80, Belongs::Goal(1)),
                    queued(12, 3, 30, Belongs::Nothing),
                    queued(13, 2, 5, Belongs::Chat(2)),
                ],
                &[chat(2, 40, false)],
            ),
            wall: wall(NOW),
            want: work(
                &[card(Item::Chat(2), &[13]), card(Item::Change(12), &[]), card(Item::Goal(1), &[11])],
                &[],
                &[],
                Filters { goals: 1, changes: 3, chats: 1, drafts: 0 },
            ),
        },
        Case {
            name: "In flight ties use the lower task across all three kinds",
            facts: facts(
                &[goal(3, true, 20)],
                &[queued(1, 2, 20, Belongs::Nothing), queued(11, 1, 90, Belongs::Chat(2))],
                &[chat(2, 20, false)],
            ),
            wall: wall(NOW),
            want: work(
                &[card(Item::Change(1), &[]), card(Item::Chat(2), &[11]), card(Item::Goal(3), &[])],
                &[],
                &[],
                Filters { goals: 1, changes: 2, chats: 1, drafts: 0 },
            ),
        },
        Case {
            name: "drafts use newest activity, then lower task",
            facts: facts(&[], &[], &[chat(3, 10, true), chat(2, 20, true), chat(1, 20, true)]),
            wall: wall(NOW),
            want: work(
                &[],
                &[card(Item::Draft(1), &[]), card(Item::Draft(2), &[]), card(Item::Draft(3), &[])],
                &[],
                Filters { drafts: 3, ..Filters::default() },
            ),
        },
        Case {
            name: "Landed includes wall",
            facts: facts(&[], &[landed(1, NOW, Belongs::Nothing)], &[]),
            wall: wall(NOW),
            want: work(&[], &[], &[1], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "Landed includes exactly seven days ago",
            facts: facts(&[], &[landed(1, WEEK_AGO, Belongs::Nothing)], &[]),
            wall: wall(NOW),
            want: work(&[], &[], &[1], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "Landed excludes one second before the week",
            facts: facts(&[], &[landed(1, WEEK_AGO - 1, Belongs::Nothing)], &[]),
            wall: wall(NOW),
            want: work(&[], &[], &[], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "Landed excludes the future",
            facts: facts(&[], &[landed(1, NOW + 1, Belongs::Nothing)], &[]),
            wall: wall(NOW),
            want: work(&[], &[], &[], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "Landed sorts newest first, then lower task",
            facts: facts(
                &[],
                &[
                    landed(1, NOW - 1, Belongs::Nothing),
                    landed(3, NOW, Belongs::Nothing),
                    landed(2, NOW, Belongs::Nothing),
                ],
                &[],
            ),
            wall: wall(NOW),
            want: work(&[], &[], &[2, 3, 1], Filters { changes: 3, ..Filters::default() }),
        },
        Case {
            name: "the week saturates at the epoch",
            facts: facts(&[], &[landed(1, 0, Belongs::Nothing)], &[]),
            wall: wall(1),
            want: work(&[], &[], &[1], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "the week preserves nanoseconds",
            facts: Facts {
                landed_default: list(&[
                    Landing {
                        signed: None,
                        task: 1,
                        commit: [0xab; 32],
                        at: wall(WEEK_AGO),
                        belongs: Belongs::Nothing,
                    },
                    Landing {
                        signed: None,
                        task: 2,
                        commit: [0xab; 32],
                        at: Wall::from_nanos(wall(WEEK_AGO).as_nanos() + 1),
                        belongs: Belongs::Nothing,
                    },
                ]),
                changes: list(&[
                    Change {
                        signed: None,
                        task: 1,
                        at: Where::Landed { at: wall(WEEK_AGO), default_branch: true },
                        belongs: Belongs::Nothing,
                    },
                    Change {
                        signed: None,
                        task: 2,
                        at: Where::Landed { at: Wall::from_nanos(wall(WEEK_AGO).as_nanos() + 1), default_branch: true },
                        belongs: Belongs::Nothing,
                    },
                ]),
                ..Facts::default()
            },
            wall: Wall::from_nanos(wall(NOW).as_nanos() + 1),
            want: work(&[], &[], &[2], Filters { changes: 2, ..Filters::default() }),
        },
        Case {
            name: "first goal row wins, including ended before live",
            facts: facts(&[goal(1, false, 10), goal(1, true, 20)], &[queued(11, 1, 20, Belongs::Goal(1))], &[]),
            wall: wall(NOW),
            want: work(
                &[card(Item::Change(11), &[])],
                &[],
                &[],
                Filters { goals: 1, changes: 1, ..Filters::default() },
            ),
        },
        Case {
            name: "first live goal row wins, including its activity",
            facts: facts(&[goal(2, true, 10), goal(2, false, 30), goal(1, true, 20)], &[], &[]),
            wall: wall(NOW),
            want: work(
                &[card(Item::Goal(1), &[]), card(Item::Goal(2), &[])],
                &[],
                &[],
                Filters { goals: 2, ..Filters::default() },
            ),
        },
        Case {
            name: "first change row wins before sorting, including its owner and state",
            facts: facts(
                &[goal(1, true, 10)],
                &[queued(11, 2, 20, Belongs::Goal(1)), landed(11, NOW, Belongs::Chat(2))],
                &[chat(2, 30, false)],
            ),
            wall: wall(NOW),
            want: work(&[card(Item::Goal(1), &[11])], &[], &[], Filters { goals: 1, changes: 1, ..Filters::default() }),
        },
        Case {
            name: "first chat row wins, including draft and activity",
            facts: facts(
                &[],
                &[queued(11, 1, 50, Belongs::Chat(2)), queued(12, 2, 50, Belongs::Nothing)],
                &[chat(2, 10, true), chat(2, 90, false)],
            ),
            wall: wall(NOW),
            want: work(
                &[card(Item::Change(12), &[]), card(Item::Chat(2), &[11])],
                &[card(Item::Draft(2), &[])],
                &[],
                Filters { drafts: 1, goals: 0, changes: 2, chats: 1 },
            ),
        },
        Case {
            name: "task numbers are deduplicated within each kind alone",
            facts: facts(&[goal(1, true, 10)], &[queued(1, 1, 20, Belongs::Chat(1))], &[chat(1, 30, true)]),
            wall: wall(NOW),
            want: work(
                &[card(Item::Chat(1), &[1]), card(Item::Goal(1), &[])],
                &[card(Item::Draft(1), &[])],
                &[],
                Filters { drafts: 1, goals: 1, changes: 1, chats: 1 },
            ),
        },
        whole_project(),
    ];
    for case in cases {
        let before = case.facts.clone();
        let got = work::place(&case.facts, case.wall);
        assert_eq!(got.in_flight, case.want.in_flight, "{}", case.name);
        assert_eq!(got.drafts, case.want.drafts, "{}", case.name);
        assert_eq!(got.filters, case.want.filters, "{}", case.name);
        let mut tasks = List::uncapped();
        for row in &got.landed {
            tasks.push(row.change).unwrap();
        }
        assert_eq!(tasks.as_slice(), case.want.landed.as_slice(), "{}", case.name);
        assert_eq!(case.facts, before, "{} leaves its facts alone", case.name);
    }
}

// The whole story's work.html: two goals, an independent change, a chat and two drafts.
fn whole_project() -> Case {
    Case {
        name: "the whole small project",
        facts: facts(
            &[goal(1, true, 100), goal(2, true, 90)],
            &[
                queued(11, 1, 10, Belongs::Goal(1)),
                queued(12, 3, 80, Belongs::Nothing),
                queued(13, 2, 20, Belongs::Chat(3)),
                landed(14, NOW - 20, Belongs::Goal(1)),
                landed(15, NOW - 10, Belongs::Goal(2)),
                landed(16, NOW, Belongs::Nothing),
                landed(17, NOW - 30, Belongs::Chat(3)),
                landed(18, WEEK_AGO - 1, Belongs::Goal(2)),
                landed(19, WEEK_AGO - 2, Belongs::Nothing),
            ],
            &[chat(3, 70, false), chat(4, 60, true), chat(5, 50, true)],
        ),
        wall: wall(NOW),
        want: work(
            &[
                card(Item::Goal(1), &[11, 14]),
                card(Item::Goal(2), &[15, 18]),
                card(Item::Change(12), &[]),
                card(Item::Chat(3), &[13, 17]),
            ],
            &[card(Item::Draft(4), &[]), card(Item::Draft(5), &[])],
            &[16, 15, 14, 17],
            Filters { drafts: 2, goals: 2, changes: 9, chats: 1 },
        ),
    }
}

use crate::{Address, Domain, Event, Place, Request, WorkFilter, tests::fixtures as f};
use jig_client::{Down, Readable, Readout, Up, Watchable, contract as c};
use skein_lib::{Queue, Token, Writer};
use temper_client_protocol_wire::{self as wire, v1 as w};

struct Fixture {
    root: Domain,
    out: Queue<Request>,
    goals: Token,
    chats: Token,
    changes: Token,
    drafts: Token,
}
impl Fixture {
    fn new() -> Self {
        let mut root = crate::unlinked(&work_limits(), f::SEED);
        let watch = f::start(&mut root, List::uncapped());
        let mut me = f::chats::me();
        let project = me.projects.get_mut(0).unwrap();
        let mut template = project.templates.get(0).unwrap().clone();
        template.number = jig_client::shapes::GOAL;
        template.tracked = Some(1);
        template.label = 8;
        project.templates = list(&[project.templates.get(0).unwrap().clone(), template]);
        let mut initial = super::welcome::deliver(&mut root, watch, c::Delivery::Me(Box::new(me)));
        let mut out = super::welcome::event(
            &mut root,
            Event::Went { place: Place::Address(Address::Work { project: f::PROJECT, filter: WorkFilter::All }) },
        );
        let mut goals = None;
        let mut chats = None;
        let mut changes = None;
        let mut drafts = None;
        for request in &out {
            if let Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } },
            } = request
            {
                match subject {
                    Watchable::Jig(c::Subject::Tasks { query, .. }) => {
                        if query.tracked == Some(true) {
                            goals = Some(*watch);
                        } else {
                            chats = Some(*watch);
                        }
                    }
                    Watchable::Family { .. } => {
                        if Some(subject.clone()) == crate::forge::changes::watch(f::PROJECT, &f::limits().forge) {
                            changes = Some(*watch);
                        } else if Some(subject.clone()) == crate::forge::drafts::watch(f::PROJECT, &f::limits().forge) {
                            drafts = Some(*watch);
                        }
                    }
                    Watchable::Jig(_) => {}
                }
            }
        }
        while let Some(request) = initial.pop() {
            out.push(request);
        }
        Self {
            root,
            out,
            goals: goals.unwrap(),
            chats: chats.unwrap(),
            changes: changes.unwrap(),
            drafts: drafts.unwrap(),
        }
    }
    fn up(&mut self, up: Up) {
        let mut out = super::welcome::event(&mut self.root, Event::Client { event: jig_client::Event::Up { up } });
        while let Some(request) = out.pop() {
            self.out.push(request);
        }
    }
    fn answer(&mut self, read: Token, value: c::ReadValue) {
        self.up(Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(value)) });
    }
    fn live(&mut self, watch: Token, rows: &[c::TaskLine]) {
        self.up(Up::Watched {
            watch,
            event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks { rows: list(rows), more: false }))),
        });
    }
    fn lists(&mut self) {
        let mut ended_goal = super::chats::chat_line();
        ended_goal.chip.task = 2;
        ended_goal.last_activity = wall(20);
        let mut live_goal = ended_goal.clone();
        live_goal.chip.task = 1;
        live_goal.last_activity = wall(10);
        self.live(self.goals, &[live_goal]);
        let mut chat = super::chats::chat_line();
        chat.chip.task = 3;
        chat.last_activity = wall(30);
        self.live(self.chats, &[chat.clone()]);
        let mut reads = List::uncapped();
        for request in &self.out {
            if let Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(what) } },
            } = request
            {
                reads.push((*read, what.clone())).unwrap();
            }
        }
        for (read, what) in reads.into_boxed() {
            if let c::Read::Tasks { query, .. } = what {
                let rows =
                    if query.tracked == Some(true) { list(&[ended_goal.clone()]) } else { list(&[chat.clone()]) };
                self.answer(
                    read,
                    c::ReadValue::Tasks(Box::new(c::TaskPage {
                        live: List::uncapped(),
                        live_after: None,
                        ended: c::EndedGroup::Rows { rows, after: None, cut: false },
                    })),
                );
            } else if let c::Read::Notes { scope: c::NoteScope::Tasks { .. }, .. } = what {
                self.answer(
                    read,
                    c::ReadValue::Notes(Box::new(c::NotePage {
                        rows: list(&[draft_note()]),
                        after: None,
                        before: None,
                        cut: false,
                    })),
                );
            }
        }
    }
    fn drafts(&mut self, rows: &[w::LandedDraft]) {
        let panel =
            w::DraftsPanel::new(&w::CEILINGS, w::DraftsPanelParts { project: f::PROJECT, drafts: list(rows) }).unwrap();
        self.up(Up::FamilyDelivered {
            watch: self.drafts,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::drafts_panel(&panel),
            },
        });
    }
    fn panel(&mut self) -> w::ChangesPanel {
        let mut queued = List::with_capacity(4);
        for task in 11..=14 {
            queued
                .push(
                    w::ChangesQueued::new(
                        &w::CEILINGS,
                        w::ChangesQueuedParts {
                            task,
                            title: b"Change".as_slice().into(),
                            ready_since: wall(40).as_nanos(),
                            alone: false,
                            checks_since: None,
                        },
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        let panel = w::ChangesPanel::new(
            &w::CEILINGS,
            w::ChangesPanelParts {
                landed_default: List::with_capacity(0),
                landed_default_more: 0,
                project: f::PROJECT,
                landing: b"main".as_slice().into(),
                queues: list(&[w::ChangesQueue::new(
                    &w::CEILINGS,
                    w::ChangesQueueParts {
                        way: w::Way::Queue,
                        in_flight: None,
                        repository: b"owner/repo".as_slice().into(),
                        branch: b"main".as_slice().into(),
                        red: false,
                        changes: queued,
                    },
                )
                .unwrap()]),
                more: 0,
                landed: list(&[w::ChangesLanded::new(
                    &w::CEILINGS,
                    w::ChangesLandedParts {
                        signed: List::with_capacity(8),
                        commit: [0xab; 32],
                        default_branch: true,
                        task: 15,
                        title: b"Landed".as_slice().into(),
                        at: wall(NOW).as_nanos(),
                    },
                )
                .unwrap()]),
                landed_more: 0,
            },
        )
        .unwrap();
        self.up(Up::FamilyDelivered {
            watch: self.changes,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::changes_panel(&panel),
            },
        });
        panel
    }
}

#[test]
fn facts_wait_for_all_lists_and_gather_each_kind_and_owner() {
    let mut fixture = Fixture::new();
    assert_eq!(work::facts(&fixture.root, f::PROJECT), None, "unread");
    fixture.lists();
    assert_eq!(work::facts(&fixture.root, f::PROJECT), None, "changes unread");
    fixture.panel();
    fixture.drafts(&[]);
    let mut reads = List::uncapped();
    for request in &fixture.out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(c::Read::Task { task }) } },
        } = request
        {
            reads.push((*read, *task)).unwrap();
        }
    }
    for (read, task) in reads.into_boxed() {
        if task == 14 {
            continue;
        }
        let mut document = f::chat::task();
        document.chip.task = task;
        document.goal = if task == 11 || task == 15 { Some(1) } else { None };
        document.lineage_chips.clear();
        if task == 12 {
            let mut root = document.chip.clone();
            root.task = 3;
            root.executor = c::ChipExecutor::Agent { charter: 1 };
            document.lineage_chips.push(root).unwrap();
        }
        fixture.answer(read, c::ReadValue::Task(Box::new(document)));
    }
    // A read document makes the person's chat a draft, independent of its changes.
    let facts = work::facts(&fixture.root, f::PROJECT).expect("all lists answered");
    assert_eq!(facts.goals, list(&[goal(1, true, 10), goal(2, false, 20)]));
    assert_eq!(facts.chats, list(&[chat(3, 30, true)]), "live/closed duplicate kept once");
    for (task, expected) in [
        (11, Belongs::Goal(1)),
        (12, Belongs::Chat(3)),
        (13, Belongs::Nothing),
        (14, Belongs::Nothing),
        (15, Belongs::Goal(1)),
    ] {
        let mut found = None;
        for row in &facts.changes {
            if row.task == task {
                found = Some(row);
            }
        }
        let found = found.unwrap();
        assert_eq!(found.belongs, expected, "task {task}");
        if task == 15 {
            assert_eq!(found.at, Where::Landed { at: wall(NOW), default_branch: true });
        } else {
            assert_eq!(found.at, Where::Queued { place: u32::try_from(task - 10).unwrap(), since: wall(40) });
        }
    }
    assert_eq!(work::place(&facts, wall(NOW)).drafts, list(&[card(Item::Draft(3), &[])]));
    assert_eq!(work::facts(&fixture.root, 99), None);
}

#[test]
fn shown_requires_a_count_and_respects_only_its_own_override() {
    use work::{Overrides, Shown};
    for (filters, overrides, want) in [
        (Filters::default(), Overrides::default(), Shown { drafts: false, goals: false, changes: false, chats: false }),
        (
            Filters { drafts: 1, goals: 1, changes: 1, chats: 1 },
            Overrides::default(),
            Shown { drafts: true, goals: true, changes: true, chats: true },
        ),
        (
            Filters { drafts: 1, goals: 1, changes: 1, chats: 1 },
            Overrides { goals: true, changes: false },
            Shown { drafts: true, goals: false, changes: true, chats: true },
        ),
        (
            Filters { drafts: 1, goals: 1, changes: 1, chats: 1 },
            Overrides { goals: false, changes: true },
            Shown { drafts: true, goals: true, changes: false, chats: true },
        ),
    ] {
        assert_eq!(work::shown(&filters, overrides), want);
    }
}

#[test]
fn only_read_hidden_settings_override_filters() {
    use work::Overrides;
    for (value, want) in [
        (Some(None), Overrides::default()),
        (None, Overrides::default()),
        (Some(Some([true, true])), Overrides::default()),
        (Some(Some([false, false])), Overrides { goals: true, changes: true }),
        (Some(Some([false, true])), Overrides { goals: true, changes: false }),
    ] {
        let mut root = crate::unlinked(&work_limits(), f::SEED);
        let watch = f::start(&mut root, List::uncapped());
        let out = super::welcome::deliver(&mut root, watch, c::Delivery::Me(Box::new(f::chats::me())));
        let mut kind7 = None;
        for request in &out {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Family { record, .. } } },
            } = request
                && let w::Read::Setting(setting) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
                && setting.kind() == 7
            {
                kind7 = Some(*read);
            }
        }
        if let Some(value) = value {
            let result = match value {
                None => Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)),
                Some([goals, changes]) => {
                    let ways =
                        w::ProjectWays::new(&w::CEILINGS, w::ProjectWaysParts { drafts: true, goals, changes, way: 0 })
                            .unwrap();
                    let mut bytes = Writer::new(usize::try_from(ways.measure()).unwrap());
                    ways.encode(&mut bytes).unwrap();
                    Readout::Family {
                        family: wire::FORGE,
                        record: Some(wire::setting_answer(
                            &w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: Some(bytes.finish()) })
                                .unwrap(),
                        )),
                    }
                }
            };
            super::welcome::event(
                &mut root,
                Event::Client { event: jig_client::Event::Up { up: Up::ReadDone { read: kind7.unwrap(), result } } },
            );
        }
        assert_eq!(work::overrides(&root, f::PROJECT), want, "{value:?}");
        assert_eq!(work::overrides(&root, 99), Overrides::default());
    }
}

fn draft_note() -> c::Note {
    let mut note = super::chat::draft_note(1, 1, b"Draft body");
    note.scope = c::NoteScope::Task { project: f::PROJECT, task: 3 };
    note.description = c::DocumentText::complete(c::Text::new(b"Draft".as_slice().into(), 32).unwrap());
    note
}

fn work_limits() -> crate::Limits {
    let mut limits = f::limits();
    // Work opens three lists, their provenance and reports at once.
    limits.client.domain.handles = 64;
    limits.client.domain.reads = 32;
    limits.client.exchanges = 32;
    limits
}

fn draft(path: &[u8], ready: bool, goal: Option<u64>, at: u64) -> w::LandedDraft {
    w::LandedDraft::new(
        &w::CEILINGS,
        w::LandedDraftParts {
            forge: 1,
            owner: b"owner".as_slice().into(),
            name: b"repo".as_slice().into(),
            path: path.into(),
            branch: b"main".as_slice().into(),
            commit: b"abc".as_slice().into(),
            title: b"Catch up".as_slice().into(),
            writer: 9,
            at: wall(at).as_nanos(),
            chat: Some(8),
            goal,
            ready,
        },
    )
    .unwrap()
}
#[test]
fn landed_drafts_place_only_when_listed_and_mix_newest_first_with_chats() {
    let mut facts = facts(&[], &[], &[chat(2, 36_000, true), chat(1, 36_000, true)]);
    for row in [
        draft(b"new.md", true, None, 39_600),
        draft(b"not-ready.md", false, None, 50_000),
        draft(b"started.md", true, Some(42), 50_000),
        draft(b"neither.md", false, Some(42), 50_000),
        draft(b"z.md", true, None, 36_000),
        draft(b"a.md", true, None, 36_000),
    ] {
        facts.landed.push(work::LandedDraft::row(&row)).unwrap();
    }
    let base = facts.landed.get(5).unwrap().clone();
    for row in [
        work::LandedDraft { name: b"aaa".as_slice().into(), ..base.clone() },
        work::LandedDraft { owner: b"aaa".as_slice().into(), ..base.clone() },
        work::LandedDraft { forge: 0, ..base },
    ] {
        facts.landed.push(row).unwrap();
    }
    facts.landed.push(facts.landed.get(0).unwrap().clone()).unwrap();
    let placed = work::place(&facts, wall(NOW));
    assert!(placed.in_flight.is_empty());
    assert_eq!(
        placed.drafts,
        list(&[
            card(Item::Landed(0), &[]),
            card(Item::Draft(1), &[]),
            card(Item::Draft(2), &[]),
            card(Item::Landed(8), &[]),
            card(Item::Landed(7), &[]),
            card(Item::Landed(6), &[]),
            card(Item::Landed(5), &[]),
            card(Item::Landed(4), &[]),
        ])
    );
    assert_eq!(placed.filters.drafts, 8);
    facts.chats.clear();
    let placed = work::place(&facts, wall(NOW));
    assert_eq!(placed.filters.drafts, 6);
    assert!(work::shown(&placed.filters, work::Overrides::default()).drafts);
}
#[test]
fn facts_wait_for_drafts_and_gather_the_wire_rows() {
    let mut fixture = Fixture::new();
    fixture.lists();
    fixture.panel();
    assert!(work::facts(&fixture.root, f::PROJECT).is_none(), "drafts unread");
    let rows = [
        draft(b"ready.md", true, None, 40),
        draft(b"started.md", true, Some(42), 50),
        draft(b"later.md", false, None, 60),
    ];
    fixture.drafts(&rows);
    let facts = work::facts(&fixture.root, f::PROJECT).unwrap();
    let mut expected = List::uncapped();
    for row in &rows {
        expected.push(work::LandedDraft::row(row)).unwrap();
    }
    assert_eq!(facts.landed, expected);
    assert_eq!(work::place(&facts, wall(NOW)).drafts, list(&[card(Item::Landed(0), &[]), card(Item::Draft(3), &[])]));
}
#[test]
fn refused_or_ended_drafts_answer_with_no_rows() {
    for reason in [c::WatchEnd::NotAllowed, c::WatchEnd::SubjectEnded] {
        let mut fixture = Fixture::new();
        fixture.lists();
        fixture.panel();
        fixture.up(Up::WatchEnded { watch: fixture.drafts, end: reason });
        assert!(work::facts(&fixture.root, f::PROJECT).unwrap().landed.is_empty());
    }
}
#[test]
fn work_goals_and_changes_keep_the_drafts_watch_until_leaving() {
    let mut fixture = Fixture::new();
    fixture.lists();
    fixture.panel();
    fixture.drafts(&[draft(b"ready.md", true, None, 40)]);
    fixture.out = Queue::uncapped();
    for address in [
        Address::Goals { project: f::PROJECT, filter: crate::GoalsFilter::All },
        Address::Changes { project: f::PROJECT },
        Address::Work { project: f::PROJECT, filter: WorkFilter::Drafts },
    ] {
        let out = super::welcome::event(&mut fixture.root, Event::Went { place: Place::Address(address) });
        for request in &out {
            if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
                request
            {
                assert_ne!(*watch, fixture.drafts);
            }
            if let Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Open { subject, .. } },
            } = request
            {
                assert_ne!(Some(subject.clone()), crate::forge::drafts::watch(f::PROJECT, &work_limits().forge));
            }
        }
        assert_eq!(work::facts(&fixture.root, f::PROJECT).unwrap().landed.len(), 1);
    }
    let out = super::welcome::event(
        &mut fixture.root,
        Event::Went { place: Place::Address(Address::Home { project: f::PROJECT }) },
    );
    let mut closed = false;
    for request in &out {
        if *request
            == (Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch: fixture.drafts } },
            })
        {
            closed = true;
        }
    }
    assert!(closed);
    assert!(fixture.root.pages().drafts().is_none());
}

#[test]
fn main_landings_group_goals_but_keep_chat_and_unread_changes_and_count_goal_branches() {
    let mut facts = Facts::default();
    for (task, at, belongs) in [
        (22, NOW - 10_800, Belongs::Goal(7)),
        (30, NOW - 7_200, Belongs::Chat(9)),
        (21, NOW - 3_600, Belongs::Goal(7)),
        (40, NOW - 7_200, Belongs::Nothing),
        (21, NOW - 3_600, Belongs::Goal(7)),
        (50, WEEK_AGO, Belongs::Nothing),
        (51, WEEK_AGO - 1, Belongs::Nothing),
        (52, NOW + 1, Belongs::Goal(8)),
    ] {
        facts
            .landed_default
            .push(Landing { signed: None, task, at: wall(at), commit: [u8::try_from(task).unwrap(); 32], belongs })
            .unwrap();
    }
    for (task, at, belongs, default_branch) in [
        (21, WEEK_AGO - 1, Belongs::Goal(7), false),
        (22, WEEK_AGO, Belongs::Goal(7), false),
        (21, NOW, Belongs::Goal(7), false),
        (23, NOW, Belongs::Goal(7), true),
        (24, WEEK_AGO - 1, Belongs::Goal(7), false),
        (25, NOW + 1, Belongs::Goal(7), false),
        (26, NOW, Belongs::Chat(9), false),
        (27, NOW, Belongs::Nothing, false),
    ] {
        facts
            .changes
            .push(Change { signed: None, task, at: Where::Landed { at: wall(at), default_branch }, belongs })
            .unwrap();
    }
    let got = work::place(&facts, wall(NOW));
    assert_eq!(
        got.landed.as_slice(),
        &[
            work::Landed { signed: None, item: Item::Goal(7), change: 21, at: wall(NOW - 3_600), commit: [21; 32] },
            work::Landed { signed: None, item: Item::Change(30), change: 30, at: wall(NOW - 7_200), commit: [30; 32] },
            work::Landed { signed: None, item: Item::Change(40), change: 40, at: wall(NOW - 7_200), commit: [40; 32] },
            work::Landed { signed: None, item: Item::Change(50), change: 50, at: wall(WEEK_AGO), commit: [50; 32] },
        ]
    );
    assert_eq!(got.goal_branches, 2, "each goal-branch task once, only in the window");
    assert_eq!(got.filters.changes, 7, "default-branch history adds no filter rows");
}

#[test]
fn signed_names_the_reader_members_and_unlisted_parties() {
    use super::team::{member, project};
    let people = project(
        Box::new([
            member(1, c::PartyKind::Person, b"Reader", c::HeldRole::Owner),
            member(5, c::PartyKind::Person, b"Sam", c::HeldRole::Member),
            member(6, c::PartyKind::Person, b"Kit", c::HeldRole::Member),
            member(7, c::PartyKind::Person, b"", c::HeldRole::Member),
        ]),
        false,
    );
    for (parties, words) in [
        (b"".as_slice(), None),
        (&[1], Some("signed by you")),
        (&[5, 1], Some("signed by you and Sam")),
        (&[1, 5, 6], Some("signed by you, Sam and Kit")),
        (&[5, 9], Some("signed by Sam and one other")),
        (&[9, 10], Some("signed by two others")),
        (&[7], Some("signed by login")),
        (&[5, 6, 9, 10], Some("signed by Sam, Kit and two others")),
        (&[9, 10, 11, 12, 13, 14, 15, 16, 17, 18], Some("signed by 10 others")),
    ] {
        let mut parties64 = List::with_capacity(u32::try_from(parties.len()).unwrap());
        for party in parties {
            parties64.push(u64::from(*party)).unwrap();
        }
        assert_eq!(
            work::signed(1, parties64.as_slice(), Some(&people)).as_deref(),
            match words {
                Some(words) => Some(words.as_bytes()),
                None => None,
            }
        );
        assert_eq!(work::signed(1, parties64.as_slice(), None), None, "people unread, refused or failed");
    }
}

#[test]
fn landed_rows_keep_their_own_signers_on_cards_and_the_newest_goal_landing() {
    let mut old = landed(11, NOW - 2, Belongs::Goal(1));
    old.signed = Some(b"signed by Sam".as_slice().into());
    let mut newest = landed(12, NOW - 1, Belongs::Goal(1));
    newest.signed = Some(b"signed by you".as_slice().into());
    let mut chat_change = landed(13, NOW, Belongs::Chat(3));
    chat_change.signed = Some(b"signed by Kit".as_slice().into());
    let placed = work::place(
        &facts(
            &[goal(1, true, NOW)],
            &[old, newest, chat_change, queued(14, 1, NOW, Belongs::Chat(3))],
            &[chat(3, NOW, false)],
        ),
        wall(NOW),
    );
    assert_eq!(placed.landed.get(1).unwrap().signed.as_deref(), Some(b"signed by you".as_slice()));
    assert_eq!(placed.landed.get(0).unwrap().signed.as_deref(), Some(b"signed by Kit".as_slice()));
    let goal_rows = &placed.in_flight.get(0).unwrap().changes;
    assert_eq!(goal_rows.get(0).unwrap().signed.as_deref(), Some(b"signed by you".as_slice()));
    assert_eq!(goal_rows.get(1).unwrap().signed.as_deref(), Some(b"signed by Sam".as_slice()));
    let chat_rows = &placed.in_flight.get(1).unwrap().changes;
    assert_eq!(chat_rows.get(0).unwrap().signed, None, "queued");
    assert_eq!(chat_rows.get(1).unwrap().signed.as_deref(), Some(b"signed by Kit".as_slice()));
}

#[test]
fn facts_name_signers_from_the_rails_project_read_and_stay_silent_without_it() {
    for answer in 0_u32..4 {
        let mut fixture = Fixture::new();
        fixture.lists();
        fixture.drafts(&[]);
        let mut panel = fixture.panel().into_parts();
        let mut row = panel.landed.get(0).unwrap().clone().into_parts();
        row.signed =
            list(&[
                w::ChangesSigned::new(&w::CEILINGS, w::ChangesSignedParts { party: 5, head: [1; 32], at: 0 }).unwrap()
            ]);
        let row = w::ChangesLanded::new(&w::CEILINGS, row).unwrap();
        panel.landed = list(core::slice::from_ref(&row));
        panel.landed_default = list(&[row]);
        let panel = w::ChangesPanel::new(&w::CEILINGS, panel).unwrap();
        fixture.up(Up::FamilyDelivered {
            watch: fixture.changes,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::changes_panel(&panel),
            },
        });
        let mut reads = List::uncapped();
        for request in &fixture.out {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: Down::Read { read, what: Readable::Jig(c::Read::Project { project }) },
                    },
            } = request
            {
                assert_eq!(*project, f::PROJECT);
                reads.push(*read).unwrap();
            }
        }
        assert_eq!(reads.len(), 1, "the rail's read serves Work, with no second read");
        let read = *reads.get(0).unwrap();
        if answer == 1 {
            let mut project = super::team::project(
                Box::new([super::team::member(5, c::PartyKind::Person, b"Sam", c::HeldRole::Member)]),
                false,
            );
            project.number = f::PROJECT;
            fixture.answer(read, c::ReadValue::Project(Box::new(project)));
        } else if answer == 2 {
            fixture.up(Up::ReadDone { read, result: Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)) });
        } else if answer == 3 {
            fixture.up(Up::ReadDone { read, result: Readout::Limit });
        }
        let facts = work::facts(&fixture.root, f::PROJECT).unwrap();
        let expected = if answer == 1 { Some(b"signed by Sam".as_slice()) } else { None };
        assert_eq!(facts.changes.get(4).unwrap().signed.as_deref(), expected, "card fact, answer {answer}");
        assert_eq!(facts.landed_default.get(0).unwrap().signed.as_deref(), expected, "landed fact, answer {answer}");
    }
}
