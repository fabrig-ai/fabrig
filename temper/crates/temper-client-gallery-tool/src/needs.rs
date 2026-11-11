//! Needs you's question states, through public documents and acts (client/web/gallery.md, 2).
use crate::{
    driver,
    samples::{Then, World},
    worlds,
};
use jig_client::contract as c;
use skein_lib::{Duration, List, Queue, Wall, Writer};
use temper_client_domain::{
    Action, Address, Domain, Event, Request,
    address::Filter,
    pages::{Act, needs, stop_asking},
};
use temper_fixtures::client_documents as documents;
const FIRST: &str = "Fix the dropped watch in jig’s store, or keep it alive in temper serve?\n\nMakeProject answers after the client’s watch drops. jig’s fix changes a stored record; serve’s fix is local but leaves jig’s behaviour as it is.\n\nSaw: The box test reproduces it every time: the store’s write lands, then the watch closes before the answer comes. The test and its log are on the goal.\nChoice: In jig’s store (recommended)\nChoice: In temper serve";
const SECOND: &str = "The “connect your deployment” help page isn’t written. Link the server help on GitHub for now, or hide the link until the page exists?\n\nThe welcome’s design names the link a placeholder.\nChoice: Link the GitHub page (recommended)\nChoice: Hide the link";
const THIRD: &str = "The repository has no license. Name one in the README?\n\nThe README is written apart from this line.\nChoice: MIT (recommended)\nChoice: Apache-2.0\nChoice: No license line";
fn text(words: &str) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(words.as_bytes().into(), 4096).expect("fixture words"))
}
fn object(task: u64) -> c::ObjectKey {
    c::ObjectKey::Question { task, message: 1 }
}
fn first_object(name: &str) -> c::ObjectKey {
    match name {
        "needs-proposal" => c::ObjectKey::Proposal(c::ProposalKey { proposer: c::Proposer::Task(1), number: 1 }),
        "needs-held" | "needs-funds" => c::ObjectKey::Escalation(1),
        _ => object(1),
    }
}
fn list(name: &str) -> bool {
    matches!(name, "needs-list" | "needs-list-second" | "needs-tell" | "needs-approval-first")
}
fn entries(name: &str) -> List<c::InboxEntry> {
    if name == "needs-only-you-done" {
        return List::with_capacity(0);
    }
    if name == "needs-only-you" {
        return List::from_boxed(Box::new([documents::entry(object(1), 1, 1, Wall::EPOCH, None)]));
    }
    if list(name) {
        return List::from_boxed(Box::new([
            documents::entry(object(1), 1, 1, Wall::EPOCH, None),
            documents::entry(c::ObjectKey::Escalation(2), 2, 2, Wall::EPOCH, None),
        ]));
    }
    if name.starts_with("needs-approval") {
        return List::with_capacity(0);
    }
    let mut entries = List::with_capacity(3);
    for task in 1..=3 {
        entries
            .push(documents::entry(
                if task == 1 { first_object(name) } else { object(task) },
                1,
                task,
                Wall::EPOCH,
                None,
            ))
            .expect("three questions");
    }
    entries
}
fn act(root: &mut Domain, act: needs::Act, out: &mut Queue<Request>) {
    driver::step(root, Event::Act { action: Action::Pages(Act::Needs(act)) }, out);
}
#[expect(clippy::too_many_lines, reason = "one fixture answers each card kind through the domain's read requests")]
pub fn domain(name: &str) -> Domain {
    let source = worlds::world(World::Solo, Address::Inbox { filter: Filter::ALL }, Then::Shown).0;
    let mut me = source.pages().me(source.client()).expect("fixture me").clone();
    me.projects.get_mut(0).expect("project").name = text("fabrig");
    me.projects.get_mut(0).expect("project").role =
        if name == "needs-stop-asking-set" { c::HeldRole::Member } else { c::HeldRole::Owner };
    if list(name) {
        let mut second = me.projects.get(0).expect("first project").clone();
        second.project = 2;
        second.name = text("vsr-rs");
        me.projects = List::from_boxed(Box::new([me.projects.get(0).expect("first project").clone(), second]));
    }
    me.needs_total = if name == "needs-only-you" {
        1
    } else if name == "needs-only-you-done" {
        0
    } else if list(name) {
        2
    } else if name.starts_with("needs-approval") {
        0
    } else {
        3
    };
    me.needs = c::Needs {
        questions: if name.starts_with("needs-only-you") {
            me.needs_total
        } else if list(name) {
            1
        } else if name.starts_with("needs-approval") {
            0
        } else if name == "needs-proposal" || name == "needs-held" || name == "needs-funds" {
            2
        } else {
            3
        },
        proposals: u32::from(name == "needs-proposal"),
        escalations: u32::from(list(name) || name == "needs-held" || name == "needs-funds"),
        person_tasks: 0,
    };
    let prototype = source
        .pages()
        .inbox()
        .expect("fixture inbox")
        .cards
        .iter()
        .find_map(|card| if let Some(c::ReadValue::Question(q)) = &card.page { Some(q.as_ref().clone()) } else { None })
        .expect("fixture question");
    let mut root = temper_client_domain::unlinked(&driver::limits(), 11);
    let mut out = Queue::uncapped();
    driver::step(
        &mut root,
        Event::Start {
            address: Address::Inbox {
                filter: if list(name) || name.starts_with("needs-approval") || name.starts_with("needs-only-you") {
                    Filter::ALL
                } else {
                    Filter { project: Some(1), ..Filter::ALL }
                },
            },
            kept: List::with_capacity(0),
            with: None,
        },
        &mut out,
    );
    for event in [
        jig_client::Event::StartUnlinked { kept: List::with_capacity(0) },
        jig_client::Event::SignIn { with: jig_client::With::Transport },
        jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
    ] {
        driver::step(&mut root, Event::Client { event }, &mut out);
    }
    while let Some(request) = out.pop() {
        let Some(down) = driver::down(request) else { continue };
        match down {
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Accounts) }
                if name.starts_with("needs-only-you") =>
            {
                driver::deliver(
                    &mut root,
                    watch,
                    c::Delivery::Accounts(Box::new(worlds::accounts(World::Reference, Then::Shown))),
                    &mut out,
                );
                crate::samples::account_states(&mut root, name, Some(watch), &mut out);
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Me) } => {
                driver::deliver(&mut root, watch, c::Delivery::Me(Box::new(me.clone())), &mut out);
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Settings) }
                if name == "needs-why" || name.starts_with("needs-stop-asking") =>
            {
                driver::deliver(
                    &mut root,
                    watch,
                    c::Delivery::Settings(Box::new(c::SettingsDocument {
                        newest: 1,
                        settings: List::from_boxed(Box::new([c::SettingLine {
                            setting: c::Setting::Application { kind: 8, scope: 1, number: 1 },
                            name: None,
                            follows: c::SettingFollows::Change,
                            last: Some(1),
                            application: None,
                        }])),
                        held: List::with_capacity(0),
                        pending: List::with_capacity(0),
                    })),
                    &mut out,
                );
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Family { record, .. } }
                if (list(name) || name.starts_with("needs-approval"))
                    && temper_client_protocol_wire::decode_watching(
                        &record.bytes,
                        &temper_client_protocol_wire::v1::CEILINGS,
                    )
                    .is_some_and(|watching| *watching.watch() == temper_client_protocol_wire::v1::Watch::Approvals) =>
            {
                let document = approval(name);
                driver::step(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: temper_client_protocol_wire::FORGE,
                                    version: temper_client_protocol_wire::CLIENT_VERSION,
                                    bytes: temper_client_protocol_wire::approvals(&document),
                                },
                            },
                        },
                    },
                    &mut out,
                );
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Family { record, .. } }
                if list(name)
                    && matches!(
                        temper_client_protocol_wire::decode_watching(
                            &record.bytes,
                            &temper_client_protocol_wire::v1::CEILINGS
                        )
                        .expect("Needs you watch")
                        .watch(),
                        temper_client_protocol_wire::v1::Watch::Decided(_)
                    ) =>
            {
                let watching = temper_client_protocol_wire::decode_watching(
                    &record.bytes,
                    &temper_client_protocol_wire::v1::CEILINGS,
                )
                .expect("Needs you decisions");
                let temper_client_protocol_wire::v1::Watch::Decided(project) = watching.watch() else {
                    unreachable!("decisions watch")
                };
                driver::step(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: temper_client_protocol_wire::FORGE,
                                    version: temper_client_protocol_wire::CLIENT_VERSION,
                                    bytes: worlds::whole_story::needs_decisions(project.project()),
                                },
                            },
                        },
                    },
                    &mut out,
                );
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Task { task }) } if list(name) => {
                driver::answer(
                    &mut root,
                    read,
                    c::ReadValue::Task(Box::new(worlds::whole_story::needs_task(task))),
                    &mut out,
                );
            }
            jig_client::Down::Open { watch, subject }
                if Some(&subject)
                    == temper_client_domain::forge::dismissed::watch(&driver::limits().forge).as_ref()
                    && (name == "needs-why" || name.starts_with("needs-stop-asking")) =>
            {
                use temper_client_protocol_wire::{self as wire, v1 as w};
                let document = w::Dismissals::new(
                    &w::CEILINGS,
                    w::DismissalsParts { dismissed: List::with_capacity(0) },
                )
                .expect("empty dismissals");
                driver::step(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::dismissals(&document),
                                },
                            },
                        },
                    },
                    &mut out,
                );
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Inbox { .. }) } => {
                driver::deliver(
                    &mut root,
                    watch,
                    c::Delivery::Inbox(Box::new(documents::inbox(
                        entries(name),
                        if name.starts_with("needs-only-you") {
                            me.needs_total
                        } else if list(name) {
                            2
                        } else if name.starts_with("needs-approval") {
                            0
                        } else {
                            3
                        },
                        c::Count { value: 0, limited: false },
                        0,
                        false,
                    ))),
                    &mut out,
                );
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Family { record, .. } }
                if (list(name) || name.starts_with("needs-approval"))
                    && temper_client_protocol_wire::decode_watching(
                        &record.bytes,
                        &temper_client_protocol_wire::v1::CEILINGS,
                    )
                    .is_some_and(|watching| *watching.watch() == temper_client_protocol_wire::v1::Watch::Approvals) =>
            {
                let document = approval(name);
                driver::step(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: temper_client_protocol_wire::FORGE,
                                    version: temper_client_protocol_wire::CLIENT_VERSION,
                                    bytes: temper_client_protocol_wire::approvals(&document),
                                },
                            },
                        },
                    },
                    &mut out,
                );
            }
            jig_client::Down::Open { watch, subject }
                if Some(&subject)
                    == temper_client_domain::forge::decided::watch(1, &driver::limits().forge).as_ref() =>
            {
                use temper_client_protocol_wire::{self as wire, v1 as w};
                let mut asked = List::with_capacity(1);
                if name == "needs-why" || name.starts_with("needs-stop-asking") {
                    let reach = temper_protocol_reaches::Reach {
                        kind: temper_protocol_reaches::Kind::Implementation,
                        class: temper_protocol_reaches::Class::Ask,
                        reason: if name == "needs-stop-asking-set" {
                            temper_protocol_reaches::Reason::Set(1)
                        } else {
                            temper_protocol_reaches::Reason::Kind
                        },
                        under: 0,
                        sets: u8::from(name == "needs-stop-asking-set"),
                        matched: u8::from(name == "needs-stop-asking-set"),
                        away: false,
                    };
                    let task = match prototype.from {
                        Some(c::Requester::Task(task)) => task,
                        Some(c::Requester::Person(_) | c::Requester::Deployment { .. }) | None => 1,
                    };
                    asked
                        .push(
                            w::Asked::new(
                                &w::CEILINGS,
                                w::AskedParts {
                                    number: 1,
                                    task,
                                    at: 0,
                                    reach: temper_protocol_reaches::reach_record(&reach),
                                },
                            )
                            .expect("asked row"),
                        )
                        .expect("one asked row");
                }
                let document = w::Decisions::new(
                    &w::CEILINGS,
                    w::DecisionsParts { project: 1, rows: List::with_capacity(0), asked, cut: false },
                )
                .expect("decided document");
                driver::step(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::decisions(&document),
                                },
                            },
                        },
                    },
                    &mut out,
                );
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } }
                if temper_client_protocol_wire::decode_reading(
                    &record.bytes,
                    &temper_client_protocol_wire::v1::CEILINGS,
                )
                .is_some_and(|reading| matches!(reading.read(), temper_client_protocol_wire::v1::Read::Setting(setting) if setting.kind() == 8)) =>
            {
                classes(&mut root, read, name, &mut out);
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Inbox { group, .. }) } => {
                let rows = if group == c::InboxReadGroup::Needs { entries(name) } else { List::with_capacity(0) };
                driver::answer(
                    &mut root,
                    read,
                    c::ReadValue::Inbox(Box::new(c::InboxPage {
                        selection: None,
                        before: None,
                        rows,
                        after: None,
                        cut: false,
                    })),
                    &mut out,
                );
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Question { task, .. }) } => {
                let mut q = prototype.clone();
                let (title, words, minutes) = match task {
                    1 => ("welcome-stall", FIRST, 40),
                    2 => ("forge-cards", SECOND, 180),
                    _ => ("Add a README", THIRD, 4),
                };
                q.task.task = task;
                q.task.title = text(title);
                q.message = 1;
                q.words = text(if name == "needs-plain-words" && task == 1 {
                    "Fix the dropped watch in jig’s store, or keep it alive in temper serve?\n\nMakeProject answers after the client’s watch drops."
                } else {
                    words
                });
                q.since = Wall::from_nanos(
                    worlds::now(World::Solo).as_nanos().saturating_sub(Duration::from_secs(minutes * 60).as_nanos()),
                );
                driver::answer(&mut root, read, c::ReadValue::Question(Box::new(q)), &mut out);
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Proposal { .. }) } => {
                let mut p = source
                    .pages()
                    .inbox()
                    .expect("source inbox")
                    .cards
                    .iter()
                    .find_map(|card| {
                        if let Some(c::ReadValue::Proposal(p)) = &card.page { Some(p.as_ref().clone()) } else { None }
                    })
                    .expect("source proposal");
                p.key = c::ProposalKey { proposer: c::Proposer::Task(1), number: 1 };
                p.reason = text("Accept how temper will run your work. New work follows this plan.");
                driver::answer(&mut root, read, c::ReadValue::Proposal(Box::new(p)), &mut out);
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Escalation { .. }) } => {
                let mut held = escalation(if list(name) { "needs-funds" } else { name });
                if list(name) {
                    held.task.chip.task = 2;
                }
                driver::answer(&mut root, read, c::ReadValue::Escalation(Box::new(held)), &mut out);
            }
            _ => {}
        }
    }
    actions(&mut root, name, &mut out);
    root
}

fn actions(root: &mut Domain, name: &str, out: &mut Queue<Request>) {
    match name {
        "needs-stop-asking" | "needs-stop-asking-set" => driver::step(
            root,
            Event::Act { action: Action::Pages(Act::StopAsking(stop_asking::Act::Open { object: object(1) })) },
            out,
        ),
        "needs-list-second" | "needs-tell" => {
            let sample = crate::samples::Sample {
                name: "needs-list",
                world: World::Solo,
                address: Address::Inbox { filter: Filter::ALL },
                then: Then::Shown,
            };
            let (mut view, _, _) = crate::built_root(&sample, crate::ViewInputs::new(), root);
            let node = if name == "needs-list-second" {
                view.tree()
                    .nodes()
                    .iter()
                    .find(|n| n.styles.as_slice().contains(&temper_client_view::look::NEED_ROW))
                    .expect("second row")
                    .id
            } else {
                view.tree()
                    .nodes()
                    .iter()
                    .find(|n| {
                        n.role == skein_ui::Role::TextArea && n.name.as_deref() == Some(b"Or tell it something else")
                    })
                    .expect("tell field")
                    .id
            };
            let input = if name == "needs-list-second" {
                skein_ui::Act::Press
            } else {
                skein_ui::Act::Input { value: b"Keep the fix in the store, with a regression test.".as_slice().into() }
            };
            let action = temper_client_view::act(&mut view, node, input).expect("page binding");
            driver::step(root, Event::Act { action }, out);
        }
        "needs-approval-first" => act(root, needs::Act::Open { object: c::ObjectKey::Task(81) }, out),
        "needs-reply-open" => act(root, needs::Act::OpenReply { object: object(1) }, out),
        "needs-second-answered" => {
            act(root, needs::Act::Choose { object: object(1), words: "In jig’s store".as_bytes().into() }, out);
        }
        "needs-later" => act(root, needs::Act::Open { object: object(2) }, out),
        "needs-last" | "needs-done" => {
            for (task, words) in [(1, "In jig’s store"), (2, "Link the GitHub page")] {
                act(root, needs::Act::Choose { object: object(task), words: words.as_bytes().into() }, out);
            }
            if name == "needs-done" {
                act(root, needs::Act::Choose { object: object(3), words: b"MIT".as_slice().into() }, out);
            }
        }
        _ => {}
    }
}

fn escalation(name: &str) -> c::Escalation {
    let source = worlds::world(World::Solo, Address::Task { task: 3 }, Then::Held).0;
    let mut task = source.pages().task().expect("task page").document(source.client()).expect("task").clone();
    task.chip.task = 1;
    task.chip.title = text("first-run-projects");
    task.spec.specification.words = text("Both halves of the store task run. The Ready page waits for this decision.");
    task.budget.numbers.spent = 31;
    task.budget.numbers.budget = 40;
    let hold = task.hold.as_mut().expect("held fixture");
    hold.reason = if name == "needs-funds" {
        c::HoldCategory::Budget
    } else {
        c::HoldCategory::Failures { class: c::HoldFailure::Run }
    };
    hold.words = Some(text("Without release the goal stops after they land, with the Ready page undone."));
    // The escalation's offers are independent of its task's offers.
    let mut acts = List::with_capacity(2);
    acts.push(c::Offered {
        act: c::ActKind::Release,
        admission: c::Admission::Allowed,
        effect: Some(c::Effect::Release(c::ReleaseEffect::RunAgain)),
    })
    .expect("release");
    acts.push(c::Offered { act: c::ActKind::LeaveHeld, admission: c::Admission::Allowed, effect: None })
        .expect("leave held");
    c::Escalation { task: Box::new(task), acts }
}

fn approval(name: &str) -> temper_client_protocol_wire::v1::Approvals {
    use temper_client_protocol_wire::v1 as w;
    let no_account = name == "needs-approval-no-account";
    let row = w::ApprovalRow::new(
        &w::CEILINGS,
        w::ApprovalRowParts {
            task: 81,
            project: 1,
            goal: None,
            forge: 3,
            kind: if no_account || list(name) { w::Kind::Github } else { w::Kind::Forgejo },
            owner: Box::from(*b"work"),
            name: Box::from(*b"hello"),
            pull: 24,
            address: if no_account || list(name) {
                Box::from(b"https://github.com/work/hello/pull/24/files".as_slice())
            } else {
                Box::from(b"https://forge.example/work/hello/pulls/24/files".as_slice())
            },
            title: Box::from(*b"A change"),
            since: worlds::now(World::Solo).as_nanos().saturating_sub(Duration::from_secs(240).as_nanos()),
            scope: u32::from(!no_account),
            rule: if no_account { 2 } else { 1 },
            role: u32::from(no_account),
            required: if no_account { 2 } else { 1 },
            counted: u32::from(no_account),
            sets: if name == "needs-approval-path" {
                List::from_boxed(Box::new([w::ApprovalSet::new(
                    &w::CEILINGS,
                    w::ApprovalSetParts { number: 1, name: b"vsr's protocol core".as_slice().into() },
                )
                .expect("path set")]))
            } else {
                List::with_capacity(0)
            },
            whole: true,
            yours: if no_account { w::ApprovalYours::NoAccount } else { w::ApprovalYours::Counts },
        },
    )
    .expect("approval fixture");
    w::Approvals::new(&w::CEILINGS, w::ApprovalsParts { rows: List::from_boxed(Box::new([row])), more: 0 })
        .expect("approval document")
}

fn classes(root: &mut Domain, read: skein_lib::Token, name: &str, out: &mut Queue<Request>) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    use temper_protocol_reaches::{Class, Classes, Item, PathSet, Subject, Way, classes_record};
    let mut classes = Classes::stock(Way::AgentsReview);
    if name == "needs-stop-asking-set" {
        classes.items = Box::new([Item { subject: Subject::Set(1), words: Box::default(), class: Class::Ask }]);
        classes.sets = Box::new([PathSet {
            number: 1,
            name: Box::from(b"protocol core".as_slice()),
            patterns: Box::new([Box::from(b"src/wire/**".as_slice()), Box::from(b"src/store/**".as_slice())]),
        }]);
    }
    let record = classes_record(&classes).expect("gallery classes");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("classes bytes"));
    record.encode(&mut writer).expect("measured classes");
    let document = w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: Some(writer.finish()) })
        .expect("classes answer");
    driver::step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: wire::FORGE,
                        record: Some(wire::setting_answer(&document)),
                    },
                },
            },
        },
        out,
    );
}
