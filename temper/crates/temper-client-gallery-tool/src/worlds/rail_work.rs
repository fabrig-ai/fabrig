//! The whole story's Work and Deployment deliveries (temper/docs/design/client/web/gallery.md, 2).
use super::{Reference, Then, answer, base_frame, base_task, c, deliver, doc, step};
use jig_client::{Down, Readable, Up, Watchable};
use skein_lib::{List, Queue, Token, Wall};
use temper_client_domain::{Address, Domain, Event, Place, Request, WorkFilter};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub(super) const NOW: Wall = Wall::from_nanos(1_794_123_120_000_000_000);
fn at(minutes: i64) -> Wall {
    let delta = minutes.unsigned_abs().checked_mul(60_000_000_000).expect("sample offset fits");
    Wall::from_nanos(NOW.as_nanos().checked_sub(delta).expect("sample follows its records"))
}

#[derive(Clone, Copy)]
enum Work {
    Whole,
    Pair,
    Empty,
}
impl Work {
    fn of(name: &str) -> Self {
        match name {
            "work" | "work-drafts" | "work-goals" | "work-changes" => Self::Whole,
            "work-pair" | "rail-new-change" => Self::Pair,
            _ => Self::Empty,
        }
    }
}
pub(super) fn world(name: &str, address: Address) -> Domain {
    let work = Work::of(name);
    // Deployment and the across-project page retain the project opened before them.
    let start = Address::Work { project: 1, filter: WorkFilter::All };
    let (mut root, mut out) = super::started(start.clone());
    settle(&mut root, &mut out, work, name);
    if address != start {
        step(&mut root, Event::Went { place: Place::Address(address) }, &mut out);
        settle(&mut root, &mut out, work, name);
    }
    root
}
fn settle(root: &mut Domain, out: &mut Queue<Request>, work: Work, name: &str) {
    for _ in 0..256 {
        let Some(request) = out.pop() else { break };
        let Some(down) = crate::driver::down(request) else { continue };
        match down {
            Down::Open { watch, subject: Watchable::Jig(subject) } => {
                let value = match subject {
                    c::Subject::Me => Some(c::Delivery::Me(Box::new(frame(work, name)))),
                    c::Subject::Tasks { query, window } => {
                        let mut rows = rows(work, &query, false);
                        while rows.len() > window {
                            let _row = rows.swap_remove(rows.len() - 1).expect("last window row");
                        }
                        Some(c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false })))
                    }
                    c::Subject::Inbox { .. } => Some(c::Delivery::Inbox(Box::new(inbox(name)))),
                    c::Subject::Settings => Some(c::Delivery::Settings(Box::new(c::SettingsDocument {
                        newest: 0,
                        settings: List::with_capacity(0),
                        held: List::with_capacity(0),
                        pending: List::with_capacity(0),
                    }))),
                    c::Subject::Deployment => Some(c::Delivery::Deployment(Box::new(deployment()))),
                    _ => None,
                };
                if let Some(value) = value {
                    deliver(root, watch, value, out);
                }
            }
            Down::Open { watch, subject: Watchable::Family { record, .. } } => {
                let decoded = wire::decode_watching(&record.bytes, &w::CEILINGS).expect("gallery watch");
                if let w::Watch::Changes(_) = decoded.watch() {
                    family(root, watch, wire::changes_panel(&changes(work)), out);
                } else if let w::Watch::Forges = decoded.watch() {
                    super::forges_watched(root, Reference, watch, out);
                } else if let w::Watch::Decided(key) = decoded.watch() {
                    family(root, watch, decisions(key.project()), out);
                }
            }
            Down::Read { read, what: Readable::Jig(c::Read::Project { project }) } => {
                if name == "members-refused" && project == 2 {
                    step(
                        root,
                        Event::Client {
                            event: jig_client::Event::Up {
                                up: Up::ReadDone {
                                    read,
                                    result: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)),
                                },
                            },
                        },
                        out,
                    );
                } else {
                    answer(root, read, c::ReadValue::Project(Box::new(project_document(project))), out);
                }
            }
            Down::Read { read, what: Readable::Jig(what) } => {
                let value = match what {
                    c::Read::Inbox { group, .. } if name == "rail-across" => {
                        Some(c::ReadValue::Inbox(Box::new(inbox_page(name, group))))
                    }
                    c::Read::Question { task, .. } if name == "rail-across" => {
                        Some(c::ReadValue::Question(Box::new(across_question(task))))
                    }
                    c::Read::Tasks { query, .. } => Some(c::ReadValue::Tasks(Box::new(c::TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: c::EndedGroup::Rows { rows: rows(work, &query, true), after: None, cut: false },
                    }))),
                    c::Read::Notes { scope, .. } => Some(c::ReadValue::Notes(Box::new(notes(work, &scope)))),
                    c::Read::Task { task } => Some(c::ReadValue::Task(Box::new(change_task(work, task)))),
                    _ => super::read_value(super::World::Bare, &what, Then::Shown, None),
                };
                if let Some(value) = value {
                    answer(root, read, value, out);
                }
            }
            Down::Read { read, what: Readable::Family { record, .. } } => {
                let decoded = wire::decode_reading(&record.bytes, &w::CEILINGS).expect("gallery read");
                match decoded.read() {
                    w::Read::Setting(setting) if setting.kind() == 7 => {
                        let drafts =
                            matches!(work, Work::Whole) || name == "rail-all-projects" || name == "rail-across";
                        super::project_ways(root, read, drafts, true, true, out);
                    }
                    w::Read::Deployment => super::deployment_read(root, read, true, out),
                    _ => {}
                }
            }
            Down::Continue { .. } | Down::Close { .. } | Down::Save { .. } | Down::Erase { .. } | Down::Send { .. } => {
            }
        }
    }
    assert!(out.is_empty(), "rail, Work and Deployment sample settled");
}
fn family(root: &mut Domain, watch: Token, bytes: Box<[u8]>, out: &mut Queue<Request>) {
    for up in [
        Up::Watched { watch, event: c::WatchEvent::Opened },
        Up::FamilyDelivered {
            watch,
            record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes },
        },
    ] {
        step(root, Event::Client { event: jig_client::Event::Up { up } }, out);
    }
}
fn decisions(project: u32) -> Box<[u8]> {
    wire::decisions(
        &w::Decisions::new(
            &w::CEILINGS,
            w::DecisionsParts { project, rows: List::with_capacity(0), asked: List::with_capacity(0), cut: false },
        )
        .expect("no decisions"),
    )
}
fn inbox_page(name: &str, group: c::InboxReadGroup) -> c::InboxPage {
    c::InboxPage {
        rows: if group == c::InboxReadGroup::Needs { inbox(name).needs } else { List::with_capacity(0) },
        before: None,
        after: None,
        cut: false,
        selection: None,
    }
}
fn across_question(task: u64) -> c::Question {
    let mut chip = base_task(task, if task == 1 { "Witness rejoin" } else { "Budget alerts" }).chip;
    chip.details.details.project = u32::try_from(task).expect("two projects");
    c::Question {
        task: chip,
        from: Some(c::Requester::Task(task)),
        message: 1,
        words: doc(if task == 1 {
            "Count the witness only once caught up?\nChoice: Only once caught up (recommended)\nChoice: Add a catch-up barrier"
        } else {
            "Send budget alerts by email?\nChoice: Only in the app (recommended)\nChoice: By email too"
        }),
        since: at(-60),
        acts: super::offers(&[c::ActKind::Answer]),
        ending: None,
    }
}
fn inbox(name: &str) -> c::Inbox {
    if name != "rail-across" {
        return super::inbox_document(super::World::Bare);
    }
    let mut rows = List::with_capacity(2);
    for project in [1, 2] {
        rows.push(temper_fixtures::client_documents::entry(
            c::ObjectKey::Question { task: u64::from(project), message: 1 },
            project,
            1,
            at(-60),
            None,
        ))
        .expect("a need in each project");
    }
    temper_fixtures::client_documents::inbox(rows, 2, c::Count { value: 0, limited: false }, 0, false)
}
fn frame(work: Work, name: &str) -> c::Me {
    let mut me = base_frame("Owner", if matches!(work, Work::Pair) { "pocket" } else { "vsr" });
    if name == "limits-not-owner" {
        me.deployment_role = None;
    }
    let project = me.projects.get_mut(0).expect("project");
    let chat = project.templates.get(0).expect("chat template").clone();
    project.templates = List::with_capacity(3);
    project.templates.push(chat.clone()).expect("chat");
    let mut goal = super::goal_template(&chat, 2, "Draft first");
    goal.label = 2;
    project.templates.push(goal).expect("goal");
    project.templates.push(super::asking_template(&chat)).expect("Ask temper");
    if matches!(
        name,
        "rail-all-projects" | "rail-across" | "members" | "members-refused" | "limits" | "limits-not-owner"
    ) {
        let mut second = project.clone();
        second.project = 2;
        second.name = doc("cowsql");
        let mut projects = List::with_capacity(2);
        projects.push(project.clone()).expect("vsr");
        projects.push(second).expect("cowsql");
        me.projects = projects;
    }
    if name == "rail-across" {
        me.needs.questions = 2;
        me.needs_total = 2;
    }
    me
}
const REPORTS: [(u64, &str, &str, u32, u32, u64, u64); 2] = [
    (
        80,
        "membership-handoff",
        "Six of nine tasks landed on the goal's branch. The witness rejoin fix waits on your answer; two tasks wait on it.",
        6,
        9,
        31,
        50,
    ),
    (
        81,
        "fuzz-corpus",
        "Two of three tasks in: the view-change seeds and the handoff seeds landed; the corpus minimiser runs.",
        2,
        3,
        6,
        15,
    ),
];
const WHOLE_CHATS: [(u64, &str); 3] = [
    (10, "the comment on vsr_step_many is stale"),
    (11, "Fresh quorum reads against applied-minimum local reads, and what the help promises. Not started."),
    (12, "The catch-up barrier, if your answer on the handoff fix asks for it."),
];
const PAIR_CHATS: [(u64, &str); 7] = [
    (10, "Monthly totals by category"),
    (11, "Budget alerts"),
    (12, "Dark mode for the dashboard"),
    (13, "Export a year as CSV"),
    (14, "CSV import keeps the last row"),
    (15, "Accounts page loads in one request"),
    (16, "Why is the sync slow?"),
];
fn rows(work: Work, query: &c::TaskQuery, ended: bool) -> List<c::TaskLine> {
    let mut rows = List::with_capacity(7);
    if query.tracked == Some(true) {
        if matches!(work, Work::Whole) && !ended {
            for (number, title, _, done, total, spent, budget) in REPORTS {
                let mut task = base_task(number, title);
                task.label = 2;
                task.flags.conversation = false;
                task.flags.listed = true;
                task.chip.phase = c::Phase::Running;
                task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: 1 });
                task.last_activity = at(if number == 80 { -1 } else { -37 });
                task.budget.numbers.spent = spent;
                task.budget.numbers.budget = budget;
                let mut line = super::task_line(task);
                line.delegates_done = done;
                line.delegates_total = total;
                rows.push(line).expect("two live goals");
            }
        }
    } else {
        let chats: &[(u64, &str)] = match work {
            Work::Whole => &WHOLE_CHATS,
            Work::Pair => &PAIR_CHATS,
            Work::Empty => &[],
        };
        for (number, title) in chats {
            if ended != (matches!(work, Work::Pair) && *number >= 14) {
                continue;
            }
            let mut task = base_task(*number, title);
            task.chip.phase = if ended { c::Phase::Done } else { c::Phase::Running };
            task.last_activity = at(if matches!(work, Work::Whole) && *number == 10 {
                -100
            } else {
                -i64::try_from(*number).expect("few chats")
            });
            rows.push(super::task_line(task)).expect("seven chats");
        }
    }
    rows
}
fn note(task: u64, description: &str, body: &str) -> c::Note {
    c::Note {
        number: 1,
        revision: c::Revision { value: 1 },
        scope: c::NoteScope::Task { project: 1, task },
        description: doc(description),
        body: doc(body),
        writer: c::NoteAuthor::Task { task, attempt: 1 },
        tasks: List::with_capacity(0),
        changed: Some(at(if task == 81 { -37 } else { -1 })),
        acts: List::with_capacity(0),
        cut: false,
    }
}
fn notes(work: Work, scope: &c::NoteScope) -> c::NotePage {
    let mut rows = List::with_capacity(2);
    if matches!(work, Work::Whole) {
        match scope {
            c::NoteScope::Tasks { project: 1 } => {
                for (task, title) in [(11, "read-barriers"), (12, "witness-catch-up")] {
                    rows.push(note(task, title, "")).expect("two drafts");
                }
            }
            c::NoteScope::Task { task, .. } => {
                for (number, _, report, ..) in REPORTS {
                    if *task == number {
                        rows.push(note(number, &format!("report: {report}"), "")).expect("goal report");
                    }
                }
            }
            _ => {}
        }
    }
    c::NotePage { rows, before: None, after: None, cut: false }
}
const WHOLE_QUEUED: [(u64, &str); 3] =
    [(100, "Witness rejoin fix"), (101, "Comment rewritten from the code"), (102, "Witness rejoin test")];
const WHOLE_LANDED: [(u64, &str); 4] = [
    (105, "Fuzz harness: ignore a short read at end of input"),
    (104, "Witness retention across a handoff"),
    (106, "io_uring: retry a short write once"),
    (103, "Handoff states and their tests"),
];
type ChangeRows = &'static [(u64, &'static str)];
fn changes(work: Work) -> w::ChangesPanel {
    let (pending, landed): (ChangeRows, ChangeRows) = match work {
        Work::Whole => (&WHOLE_QUEUED, &WHOLE_LANDED),
        Work::Pair => (
            &[
                (100, "Monthly totals by category"),
                (101, "Budget alerts"),
                (102, "Dark mode for the dashboard"),
                (103, "Export a year as CSV"),
            ],
            &[(104, "CSV import keeps the last row"), (105, "Accounts page loads in one request")],
        ),
        Work::Empty => (&[], &[]),
    };
    let mut queues = List::with_capacity(1);
    if !pending.is_empty() {
        let mut changes = List::with_capacity(u32::try_from(pending.len()).expect("few changes"));
        for (task, title) in pending {
            changes
                .push(
                    w::ChangesQueued::new(
                        &w::CEILINGS,
                        w::ChangesQueuedParts {
                            task: *task,
                            title: title.as_bytes().into(),
                            ready_since: at(-90).as_nanos(),
                            checks_since: (*task == 100).then(|| at(-4).as_nanos()),
                        },
                    )
                    .expect("queued change"),
                )
                .expect("queued rows");
        }
        queues
            .push(
                w::ChangesQueue::new(
                    &w::CEILINGS,
                    w::ChangesQueueParts {
                        repository: b"work/project".as_slice().into(),
                        branch: b"main".as_slice().into(),
                        red: false,
                        changes,
                    },
                )
                .expect("queue"),
            )
            .expect("one queue");
    }
    let mut rows = List::with_capacity(u32::try_from(landed.len()).expect("few landings"));
    for (index, (task, title)) in landed.iter().enumerate() {
        rows.push(
            w::ChangesLanded::new(
                &w::CEILINGS,
                w::ChangesLandedParts {
                    commit: [0xab; 32],
                    default_branch: true,
                    task: *task,
                    title: title.as_bytes().into(),
                    at: at(if matches!(work, Work::Whole) {
                        match task {
                            103 => -2482,
                            104 => -592,
                            105 => -80,
                            106 => -1212,
                            _ => unreachable!("listed landing"),
                        }
                    } else {
                        -60 - i64::try_from(index).expect("few landings")
                    })
                    .as_nanos(),
                },
            )
            .expect("landed change"),
        )
        .expect("landed rows");
    }
    w::ChangesPanel::new(
        &w::CEILINGS,
        w::ChangesPanelParts {
            landed_default: rows.clone(),
            landed_default_more: 0,
            project: 1,
            landing: b"main".as_slice().into(),
            queues,
            more: 0,
            landed: rows,
            landed_more: 0,
        },
    )
    .expect("changes panel")
}
fn change_task(work: Work, number: u64) -> c::Task {
    let mut task = base_task(number, "Change");
    let origin = match work {
        Work::Whole => match number {
            100 | 103 | 104 => Some((true, 80, "membership-handoff")),
            101 | 106 => Some((false, 10, WHOLE_CHATS[0].1)),
            _ => None,
        },
        Work::Pair => Some((false, number - 90, PAIR_CHATS[usize::try_from(number - 100).expect("few changes")].1)),
        Work::Empty => None,
    };
    if let Some((goal, number, title)) = origin {
        task.goal = goal.then_some(number);
        let mut chip = base_task(number, title).chip;
        chip.executor = c::ChipExecutor::Agent { charter: 1 };
        task.lineage_chips = List::with_capacity(1);
        task.lineage_chips.push(chip).expect("change origin");
    }
    task
}
fn project_document(number: u32) -> c::Project {
    let mut members = List::with_capacity(2);
    for (party, name, role) in [(73, "Owner", c::HeldRole::Owner), (74, "Prover", c::HeldRole::Maintainer)] {
        members
            .push(c::ProjectMember {
                party,
                name: doc(name),
                login: doc(name),
                kind: if party == 73 { c::PartyKind::Person } else { c::PartyKind::Service },
                role,
            })
            .expect("two members");
    }
    c::Project {
        number,
        name: doc(if number == 1 { "vsr" } else { "cowsql" }),
        members,
        acts: List::with_capacity(0),
        resources: List::with_capacity(0),
        policy: c::ProjectPolicy {
            templates: List::with_capacity(0),
            ceiling: base_task(1, "").authority,
            period_spend: 0,
            escalation_role: None,
            projections: List::with_capacity(0),
            roles: List::with_capacity(0),
            requirements: List::with_capacity(0),
            permissions: None,
            permissions_cut: false,
            cut: false,
        },
        current_period: None,
        pools: List::with_capacity(0),
        configured_period: 0,
        configured_pools: List::with_capacity(0),
        configured_pools_cut: false,
        spend: List::with_capacity(0),
        recurring: List::with_capacity(0),
        cut: false,
    }
}
fn deployment() -> c::Deployment {
    let mut doc = super::deployment(Reference);
    doc.hosts = super::builders();
    doc.tasks = c::Capacity { used: 12, limit: 100 };
    doc.workers = c::Capacity { used: 3, limit: 8 };
    doc.attempts_capacity = c::Capacity { used: 4, limit: 16 };
    doc.waiting_attempts = 2;
    doc.proposals = c::Capacity { used: 1, limit: 32 };
    doc.store = c::StoreFill { bytes: 4_100_000, mark: 64_000_000 };
    doc.projects = List::with_capacity(2);
    for (project, used) in [(1, 9), (2, 3)] {
        doc.projects
            .push(c::ProjectCapacity { project, tasks: c::Capacity { used, limit: 50 }, proposals: 0 })
            .expect("two projects");
    }
    doc
}
