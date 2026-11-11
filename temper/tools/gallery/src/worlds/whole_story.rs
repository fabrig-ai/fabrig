//! Whole-story deliveries (temper/docs/design/client/web/gallery.md, 2; client/pages/README.md).
use super::{Then, World, base_task, doc, offers};
use crate::driver::{down, limits};
use jig_client::{Down, Readable, Readout, Up, Watchable, contract as c};
use skein_lib::{Env, List, Queue, Time, Token, Wall};
use temper_client_domain::{Address, Domain, Event, Request};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_fixtures::client_documents as documents;

const MINUTE: u64 = 60_000_000_000;
const PROJECTS: [(u32, &str); 4] = [(1, "vsr"), (2, "vsr-rs"), (3, "cowsql"), (4, "fabrig")];
const QUORUM: &str = "Count a rejoining witness toward the view-change quorum only once it holds the log suffix?\n\nProver's test replays the fuzz case: a witness that rejoined during view 41 was counted toward the quorum before it held the log's last 64 entries, and the new primary truncated a committed entry. Both fixes pass the test.\nChoice: Count it, add a catch-up barrier\nChoice: Only once caught up (recommended)";
const ALERTS: &str = "Send budget alerts by email, or only show them in the app?\nChoice: By email too\nChoice: Only in the app (recommended)";
const COUPONS: &str =
    "Keep coupon codes case-insensitive?\nChoice: Keep them case-insensitive (recommended)\nChoice: Match case";

pub(super) fn now(name: &str) -> Wall {
    Wall::from_nanos(match name {
        "home-step-pair" => 1_792_523_640_000_000_000,
        "home-step-team" => 1_793_004_000_000_000_000,
        _ => 1_792_915_440_000_000_000,
    })
}
fn since(name: &str, minutes: u64) -> Wall {
    Wall::from_nanos(now(name).as_nanos() - minutes * MINUTE)
}
fn across(name: &str) -> bool {
    !name.starts_with("home-step-")
}
fn quiet(name: &str) -> bool {
    matches!(name, "all-home-nothing-new" | "home-step-stock")
}
fn projects(name: &str) -> &'static [(u32, &'static str)] {
    if name == "all-home-one-project" { &PROJECTS[..1] } else { &PROJECTS }
}
fn me(name: &str) -> c::Me {
    let mut me = super::frame(World::Solo, false, false);
    if across(name) {
        let mut member = me.projects.get(0).expect("Solo project").clone();
        let chat = member.templates.get(0).expect("chat").clone();
        member.templates.push(super::asking_template(&chat)).expect("Ask temper");
        me.projects = List::with_capacity(4);
        for (number, title) in projects(name) {
            let mut row = member.clone();
            row.project = *number;
            row.name = doc(title);
            me.projects.push(row).expect("four projects");
        }
    }
    me.needs = c::Needs {
        questions: u32::from(!quiet(name)),
        proposals: 0,
        escalations: u32::from(
            name == "home-step-scale" || (across(name) && !quiet(name) && name != "all-home-one-project"),
        ),
        person_tasks: 0,
    };
    me.needs_total = me.needs.questions + me.needs.escalations;
    me
}
fn entries(name: &str) -> List<c::InboxEntry> {
    let mut rows = List::with_capacity(2);
    if !quiet(name) {
        rows.push(documents::entry(
            c::ObjectKey::Question { task: 1, message: 1 },
            1,
            1,
            since(name, if name == "home-step-pair" { 20 } else { 60 }),
            None,
        ))
        .expect("question");
        if name == "home-step-scale" || (across(name) && name != "all-home-one-project") {
            rows.push(documents::entry(
                c::ObjectKey::Escalation(2),
                if across(name) { 2 } else { 1 },
                7,
                since(name, 20),
                None,
            ))
            .expect("funds");
        }
    }
    rows
}
fn status(name: &str, project: u32) -> w::ProjectStatus {
    let (landed, age, sentence, lede) = if across(name) {
        match project {
            1 => (
                12,
                12,
                "12 changes landed.",
                "vsr landed 12 changes, and its membership-handoff goal is six of nine tasks in. A design session on read barriers is with you, Friday.",
            ),
            2 => (
                4,
                24,
                "4 ports landed.",
                "vsr-rs landed 4 ports and is two landings behind vsr, every C test passing on Rust.",
            ),
            3 => (0, 30, "", ""),
            4 => (15, 32, "15 changes landed.", "fabrig landed 15, proof-6 whole among them."),
            _ => unreachable!("four projects"),
        }
    } else {
        match name {
            "home-step-pair" => (
                1,
                2,
                "One change landed today.",
                "Monthly totals by category and dark mode are both checked by CI; nothing reaches main until you approve. The budget alerts change asks whether alerts go by email. The CSV import fix you approved this afternoon landed at 16:20.",
            ),
            "home-step-team" => (
                9,
                9,
                "9 changes landed since Friday.",
                "Agents reviewed every change. The coupon goal is two of three changes in. Nothing failed after landing.",
            ),
            "home-step-stock" | "home-step-scale" => {
                (12, 12, "12 changes landed.", "Its membership-handoff goal is six of nine tasks in.")
            }
            _ => unreachable!("four Home steps"),
        }
    };
    let nothing_new = name == "all-home-nothing-new";
    let landed_age = if nothing_new {
        600
    } else if name == "home-step-pair" {
        174
    } else {
        6
    };
    w::ProjectStatus::new(
        &w::CEILINGS,
        w::ProjectStatusParts {
            project,
            landed,
            seen: if nothing_new { landed } else { 0 },
            previous: Some(since(name, if name == "home-step-team" { 3765 } else { 540 }).as_nanos()),
            last_landed: (landed > 0).then(|| since(name, landed_age).as_nanos()),
            queued: u32::from(project == 4) * 3,
            branch: b"main".as_slice().into(),
            red: across(name) && project == 4,
            brief: (!lede.is_empty()).then(|| {
                w::StatusBrief::new(
                    &w::CEILINGS,
                    w::StatusBriefParts {
                        sentence: sentence.as_bytes().into(),
                        lede: lede.as_bytes().into(),
                        at: since(name, if nothing_new { 601 } else { age }).as_nanos(),
                        landed,
                    },
                )
                .expect("whole-story brief")
            }),
        },
    )
    .expect("whole-story status")
}
fn question(name: &str) -> c::Question {
    let (title, words) = match name {
        "home-step-pair" => ("Budget alerts", ALERTS),
        "home-step-team" => ("coupons", COUPONS),
        _ => ("membership-handoff", QUORUM),
    };
    c::Question {
        task: base_task(1, title).chip,
        from: Some(c::Requester::Task(1)),
        message: 1,
        words: doc(words),
        since: since(name, if name == "home-step-pair" { 20 } else { 60 }),
        acts: offers(&[c::ActKind::Answer]),
        ending: None,
    }
}
fn funds(name: &str) -> c::Escalation {
    let mut task = super::shown_task(World::Solo, 3, Then::Held);
    task.chip.task = 2;
    task.chip.title = doc("Porter");
    task.project = if across(name) { 2 } else { 1 };
    task.chip.details.details.project = task.project;
    task.budget.funder = c::Funder::Pool { project: task.project, person: 73, period: 1 };
    task.budget.numbers.spent = 49;
    task.budget.numbers.budget = 60;
    task.chip.details.details.budget = task.budget.clone();
    let hold = task.hold.as_mut().expect("held task");
    hold.reason = c::HoldCategory::Budget;
    hold.words = Some(doc("Three ports remain."));
    task.chip.details.phase = c::ChipPhase::Held { reason: hold.reason, revision: hold.revision };
    task.chip.details.details.hold = Some(hold.as_ref().clone());
    let mut acts = offers(&[c::ActKind::Release, c::ActKind::LeaveHeld]);
    acts.get_mut(0).expect("release").effect = Some(c::Effect::Release(c::ReleaseEffect::RunAgain));
    c::Escalation { task: Box::new(task), acts }
}
fn approvals(name: &str) -> Box<[u8]> {
    let mut rows = List::with_capacity(2);
    if name == "home-step-pair" {
        for (task, pull, title, age) in
            [(81, 24, "Monthly totals by category", 34), (82, 25, "Dark mode for the dashboard", 1440)]
        {
            rows.push(
                w::ApprovalRow::new(
                    &w::CEILINGS,
                    w::ApprovalRowParts {
                        task,
                        project: 1,
                        goal: None,
                        forge: 2,
                        kind: w::Kind::Github,
                        owner: b"robin".as_slice().into(),
                        name: b"tally".as_slice().into(),
                        pull,
                        address: format!("https://github.com/robin/tally/pull/{pull}/files").into_bytes().into(),
                        title: title.as_bytes().into(),
                        since: since(name, age).as_nanos(),
                        scope: 1,
                        rule: 1,
                        role: 0,
                        required: 1,
                        counted: 0,
                        sets: List::with_capacity(0),
                        whole: true,
                        yours: w::ApprovalYours::Counts,
                        signed: List::with_capacity(0),
                        can_sign: List::with_capacity(0),
                        can_sign_more: 0,
                    },
                )
                .expect("Pair approval"),
            )
            .expect("two approvals");
        }
    }
    wire::approvals(&w::Approvals::new(&w::CEILINGS, w::ApprovalsParts { rows, more: 0 }).expect("approvals"))
}

/// The mockup's decisions name tasks, not standing-role records the client cannot read.
fn decision(task: u64) -> (u32, &'static str, u64, &'static str, &'static str, &'static str) {
    match task {
        101 => (
            1,
            "Prover",
            354,
            "A fuzz crash came from the test harness, not the library.",
            "Fixed the harness.",
            "Tests are Prover's area, and no library code changed.",
        ),
        102 => (
            2,
            "Porter",
            264,
            "vsr's new bounds check uses a compiler builtin.",
            "Ported it as checked_add.",
            "Line by line keeps C's shape; the builtin is C's way of saying checked arithmetic.",
        ),
        103 => (
            4,
            "a goal",
            172,
            "A task ran out of turns halfway through the queue view.",
            "Split it in two.",
            "Retrying or splitting a task is the goal's to decide.",
        ),
        104 => (
            4,
            "UI",
            94,
            "A look round's phone frame moved a button by 2 px.",
            "Accepted it.",
            "Inside the round's tolerance.",
        ),
        _ => unreachable!("four decision tasks"),
    }
}
fn decision_row(task: u64, wall: Wall) -> w::Decision {
    let (_, _, age, question, took, why) = decision(task);
    w::Decision::new(
        &w::CEILINGS,
        w::DecisionParts {
            number: u32::try_from(task).expect("decision number"),
            task,
            at: wall.as_nanos() - age * MINUTE,
            question: question.as_bytes().into(),
            took: took.as_bytes().into(),
            why: why.as_bytes().into(),
            overturned: false,
            reach: w::Reach::new(
                &w::CEILINGS,
                w::ReachParts {
                    kind: w::ReachKind::Tests,
                    class: w::ReachClass::Decide,
                    reason: w::ReachReason::Kind,
                    under: 0,
                    sets: 0,
                    matched: 0,
                    away: false,
                },
            )
            .expect("decision reach"),
        },
    )
    .expect("whole-story decision")
}
pub(crate) fn needs_decisions(project: u32) -> Box<[u8]> {
    let tasks: &[u64] = match project {
        1 => &[103, 104],
        2 => &[102],
        _ => &[],
    };
    let mut rows = List::with_capacity(2);
    for task in tasks {
        rows.push(decision_row(*task, super::now(World::Solo))).expect("Needs you decisions");
    }
    wire::decisions(
        &w::Decisions::new(
            &w::CEILINGS,
            w::DecisionsParts { project, rows, asked: List::with_capacity(0), cut: false },
        )
        .expect("decisions"),
    )
}
pub(crate) fn needs_task(task: u64) -> c::Task {
    let mut document = base_task(task, decision(task).1);
    document.project = if task == 102 { 2 } else { 1 };
    document.chip.details.details.project = document.project;
    document
}
fn decisions(name: &str, project: u32) -> Box<[u8]> {
    let mut rows = List::with_capacity(4);
    if across(name) && name != "all-home-nothing-new" {
        for task in 101..=104 {
            if decision(task).0 == project {
                rows.push(decision_row(task, now(name))).expect("four decisions");
            }
        }
    } else if matches!(name, "home-step-team" | "home-step-stock" | "home-step-scale") {
        let row = if name == "home-step-team" {
            w::Decision::new(
                &w::CEILINGS,
                w::DecisionParts {
                    number: 103,
                    task: 103,
                    at: since(name, 94).as_nanos(),
                    question: b"A task ran out of turns on the coupon rules.".as_slice().into(),
                    took: b"Split it in two.".as_slice().into(),
                    why: b"Retrying or splitting a task is decided and listed.".as_slice().into(),
                    overturned: false,
                    reach: w::Reach::new(
                        &w::CEILINGS,
                        w::ReachParts {
                            kind: w::ReachKind::Tests,
                            class: w::ReachClass::Decide,
                            reason: w::ReachReason::Kind,
                            under: 0,
                            sets: 0,
                            matched: 0,
                            away: false,
                        },
                    )
                    .expect("decision reach"),
                },
            )
            .expect("Team decision")
        } else {
            decision_row(101, now(name))
        };
        rows.push(row).expect("step decision");
    }
    wire::decisions(
        &w::Decisions::new(
            &w::CEILINGS,
            w::DecisionsParts { project, rows, asked: List::with_capacity(0), cut: false },
        )
        .expect("decisions"),
    )
}
fn task(name: &str, number: u64) -> c::Task {
    let mut task = base_task(number, if name == "home-step-team" { "coupons goal" } else { decision(number).1 });
    task.project = if across(name) { decision(number).0 } else { 1 };
    task.chip.details.details.project = task.project;
    task
}
fn forges(name: &str) -> Box<[u8]> {
    let source = wire::decode_forges(&super::forges(World::Solo), &w::CEILINGS).expect("Solo forges");
    let mut forge =
        source.forges().iter().find(|row| *row.kind() == w::Kind::Github).expect("GitHub").clone().into_parts();
    if across(name) {
        forge.repositories = List::with_capacity(4);
        for (project, title) in projects(name) {
            forge
                .repositories
                .push(
                    w::Adoption::new(
                        &w::CEILINGS,
                        w::AdoptionParts {
                            owner: b"work".as_slice().into(),
                            name: title.as_bytes().into(),
                            project: *project,
                        },
                    )
                    .expect("adoption"),
                )
                .expect("four repositories");
        }
    }
    wire::forges(
        &w::Forges::new(
            &w::CEILINGS,
            w::ForgesParts {
                forges: List::from_boxed(Box::new([w::ForgeRow::new(&w::CEILINGS, forge).expect("forge")])),
                linked: false,
            },
        )
        .expect("forges"),
    )
}
fn step(root: &mut Domain, name: &str, event: Event, out: &mut Queue<Request>) {
    temper_client_domain::step(root, &Env { now: Time::ZERO, wall: now(name), limits: limits() }, event, out);
}
fn family(root: &mut Domain, name: &str, watch: Token, bytes: Box<[u8]>, out: &mut Queue<Request>) {
    step(
        root,
        name,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes },
                },
            },
        },
        out,
    );
}
fn read_value(root: &mut Domain, name: &str, read: Token, value: c::ReadValue, out: &mut Queue<Request>) {
    step(
        root,
        name,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(value)) },
            },
        },
        out,
    );
}

#[expect(clippy::too_many_lines, reason = "the world answers each public watch, read and look through the root")]
pub(super) fn domain(name: &str, address: Address) -> Domain {
    let me = me(name);
    let mut away_watch = None;
    let mut root = temper_client_domain::unlinked(&limits(), 11);
    let mut out = Queue::uncapped();
    step(&mut root, name, Event::Start { address, kept: List::with_capacity(0), with: None }, &mut out);
    for event in [
        jig_client::Event::StartUnlinked { kept: List::with_capacity(0) },
        jig_client::Event::SignIn { with: jig_client::With::Transport },
        jig_client::Event::Up { up: Up::Served { served: true, versions: None } },
    ] {
        step(&mut root, name, Event::Client { event }, &mut out);
    }
    for _ in 0..512 {
        let Some(request) = out.pop() else { break };
        let Some(request) = down(request) else { continue };
        match request {
            Down::Open { watch, subject: Watchable::Jig(subject) } => {
                let value = match subject {
                    c::Subject::Me => c::Delivery::Me(Box::new(me.clone())),
                    c::Subject::Accounts => c::Delivery::Accounts(Box::new(super::accounts(World::Solo, Then::Shown))),
                    c::Subject::Settings => {
                        let mut settings = List::with_capacity(me.projects.len());
                        for project in &me.projects {
                            settings
                                .push(c::SettingLine {
                                    setting: c::Setting::Application { kind: 2, scope: project.project, number: 1 },
                                    name: None,
                                    follows: c::SettingFollows::Change,
                                    last: None,
                                    application: None,
                                })
                                .expect("one setting per project");
                        }
                        c::Delivery::Settings(Box::new(c::SettingsDocument {
                            newest: 1,
                            settings,
                            held: List::with_capacity(0),
                            pending: List::with_capacity(0),
                        }))
                    }
                    c::Subject::Tasks { .. } => {
                        c::Delivery::Tasks(Box::new(c::Tasks { rows: List::with_capacity(0), more: false }))
                    }
                    c::Subject::Inbox { .. } => c::Delivery::Inbox(Box::new(documents::inbox(
                        entries(name),
                        me.needs_total,
                        c::Count { value: 0, limited: false },
                        0,
                        false,
                    ))),
                    _ => continue,
                };
                step(
                    &mut root,
                    name,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: Up::Watched { watch, event: c::WatchEvent::Delivered(value) },
                        },
                    },
                    &mut out,
                );
            }
            Down::Open { watch, subject: Watchable::Family { record, .. } } => {
                let watching = wire::decode_watching(&record.bytes, &w::CEILINGS).expect("whole-story watch");
                let bytes = match watching.watch() {
                    w::Watch::Forges => forges(name),
                    w::Watch::Status(key) => wire::project_status(&status(name, key.project())),
                    w::Watch::Decided(key) => decisions(name, key.project()),
                    w::Watch::Approvals => approvals(name),
                    w::Watch::Dismissed => wire::dismissals(
                        &w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed: List::with_capacity(0) })
                            .expect("dismissals"),
                    ),
                    w::Watch::Away => {
                        away_watch = Some(watch);
                        wire::aways(
                            &w::Aways::new(&w::CEILINGS, w::AwaysParts { rows: away_rows(name) }).expect("aways"),
                        )
                    }
                    _ => continue,
                };
                family(&mut root, name, watch, bytes, &mut out);
            }
            Down::Read { read, what: Readable::Jig(what) } => {
                let value = match what {
                    c::Read::Inbox { group, .. } => c::ReadValue::Inbox(Box::new(c::InboxPage {
                        rows: if group == c::InboxReadGroup::Needs { entries(name) } else { List::with_capacity(0) },
                        before: None,
                        after: None,
                        cut: false,
                        selection: None,
                    })),
                    c::Read::Question { .. } => c::ReadValue::Question(Box::new(question(name))),
                    c::Read::Escalation { .. } => c::ReadValue::Escalation(Box::new(funds(name))),
                    c::Read::Task { task: number } => c::ReadValue::Task(Box::new(task(name, number))),
                    c::Read::Tasks { .. } => c::ReadValue::Tasks(Box::new(c::TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: c::EndedGroup::Rows { rows: List::with_capacity(0), after: None, cut: false },
                    })),
                    _ => continue,
                };
                read_value(&mut root, name, read, value, &mut out);
            }
            Down::Read { read, what: Readable::Family { record, .. } } => {
                let reading = wire::decode_reading(&record.bytes, &w::CEILINGS).expect("whole-story read");
                let bytes = match reading.read() {
                    w::Read::Setting(setting) if setting.kind() == 7 => {
                        let ways = w::ProjectWays::new(
                            &w::CEILINGS,
                            w::ProjectWaysParts { drafts: false, goals: false, changes: false, way: 0 },
                        )
                        .expect("project ways");
                        let mut writer = skein_lib::Writer::new(usize::try_from(ways.measure()).expect("ways size"));
                        ways.encode(&mut writer).expect("ways bytes");
                        Some(wire::setting_answer(
                            &w::SettingAnswer::new(
                                &w::CEILINGS,
                                w::SettingAnswerParts { value: Some(writer.finish()) },
                            )
                            .expect("ways answer"),
                        ))
                    }
                    _ => None,
                };
                step(
                    &mut root,
                    name,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: Up::ReadDone { read, result: Readout::Family { family: wire::FORGE, record: bytes } },
                        },
                    },
                    &mut out,
                );
            }
            Down::Send { attempt, request: jig_client::Ask::Family { record, .. }, .. }
                if wire::decode_requesting(&record.bytes, &w::CEILINGS)
                    .is_some_and(|request| matches!(request.request(), w::Request::Looked(_))) =>
            {
                step(
                    &mut root,
                    name,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: Up::Answer {
                                attempt,
                                envelope: skein_lib::RequestEnvelope::Final,
                                why: None,
                                answer: Some(jig_client::Answered::Family(jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::answer(
                                        &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Looked })
                                            .expect("look"),
                                    ),
                                })),
                            },
                        },
                    },
                    &mut out,
                );
            }
            _ => {}
        }
    }
    assert!(out.is_empty(), "whole-story deliveries settle");
    if name == "all-home-away-on" {
        toggle_all(&mut root, name, away_watch.expect("Away watch"));
    }
    root
}

fn away_rows(name: &str) -> List<w::AwayRow> {
    let mut rows = List::with_capacity(w::CEILINGS.aways_rows);
    if across(name) && name != "all-home-nothing-new" {
        rows.push(
            w::AwayRow::new(
                &w::CEILINGS,
                w::AwayRowParts {
                    project: 1,
                    since: since(name, 534).as_nanos(),
                    until: None,
                    landed: 12,
                    off: Some(since(name, 4).as_nanos()),
                },
            )
            .expect("Sunday's away spell"),
        )
        .expect("away spell");
    }
    if matches!(name, "all-home-away" | "all-home-away-on") {
        for project in [2, 3] {
            rows.push(
                w::AwayRow::new(
                    &w::CEILINGS,
                    w::AwayRowParts { project, since: since(name, 30).as_nanos(), until: None, landed: 0, off: None },
                )
                .expect("project away"),
            )
            .expect("away spell");
        }
    }
    rows
}
fn toggle_all(root: &mut Domain, name: &str, watch: Token) {
    let mut out = Queue::uncapped();
    step(
        root,
        name,
        Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Away(
                temper_client_domain::pages::away::Act::ToggleAll,
            )),
        },
        &mut out,
    );
    let mut changed = Vec::new();
    for _ in 0..512 {
        let Some(request) = out.pop() else { break };
        let Some(Down::Send { attempt, request: jig_client::Ask::Family { record, .. }, .. }) = down(request) else {
            continue;
        };
        let w::Request::SetAway(value) =
            wire::decode_requesting(&record.bytes, &w::CEILINGS).expect("Away request").into_parts().request
        else {
            continue;
        };
        changed.push(value.into_parts());
        super::sample_forge_answer(root, attempt, w::Outcome::Away, &mut out);
    }
    assert!(out.is_empty(), "Away answers settle");
    let mut rows = List::with_capacity(w::CEILINGS.aways_rows);
    for row in away_rows(name).into_boxed() {
        if !changed.iter().any(|request| request.project == row.project()) {
            rows.push(row).expect("away spell");
        }
    }
    for request in changed {
        rows.push(
            w::AwayRow::new(
                &w::CEILINGS,
                w::AwayRowParts {
                    project: request.project,
                    since: now(name).as_nanos(),
                    until: request.until,
                    landed: 0,
                    off: None,
                },
            )
            .expect("changed Away"),
        )
        .expect("away spell");
    }
    family(
        root,
        name,
        watch,
        wire::aways(&w::Aways::new(&w::CEILINGS, w::AwaysParts { rows }).expect("aways")),
        &mut out,
    );
}
