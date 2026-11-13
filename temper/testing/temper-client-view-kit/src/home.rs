use super as f;
use jig_client::contract as c;
use skein_lib::{List, Queue, Token};
use temper_client_domain::{self as d, Address};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[must_use]
pub fn home_limits() -> d::Limits {
    let mut limits = f::domain_limits();
    // Home, stock ways, goal presence, Goals, standing Needs, trials and a running trial task (client/pages/README.md).
    limits.client.domain.handles = 24;
    limits.client.domain.watches = 22;
    limits.client.domain.reads = 16;
    limits
}

pub fn drive(root: &mut d::Domain, event: d::Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(root, &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: home_limits() }, event, &mut out);
    out
}

pub fn deliver(root: &mut d::Domain, watch: Token, value: c::Delivery) {
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(value) },
            },
        },
    );
}

pub struct NeedsFixture<'a> {
    pub objects: &'a [c::ObjectKey],
    pub cut: bool,
    pub read: bool,
    pub decided: bool,
}

pub struct ChatFixture {
    pub live: c::Tasks,
    pub ended: c::TaskPage,
}

pub struct StatusFixture {
    pub document: w::ProjectStatus,
    pub answered: bool,
    pub goals: bool,
    pub shown: bool,
    pub goal_rows: Option<ChatFixture>,
    pub away: Option<AwayFixture>,
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one fixture delivers Home documents and their public replies together"
)]
#[must_use]
pub fn home_documents_with_status(
    rows: List<w::ForgeRow>,
    configured: bool,
    chat: bool,
    needs: Option<NeedsFixture<'_>>,
    chats: Option<ChatFixture>,
    dismissed: Option<&[u32]>,
    settings_delivered: bool,
    status: Option<&StatusFixture>,
) -> (d::Domain, Token) {
    let (mut root, me_watch, accounts, settings, needs_watch, away_watch) =
        f::home_root(&home_limits(), rows, dismissed, status.is_some_and(|fixture| fixture.shown));
    if let Some(spell) = status.and_then(|s| s.away.as_ref()).and_then(|a| a.spell.clone()) {
        drive(
            &mut root,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::FamilyDelivered {
                        watch: away_watch,
                        record: jig_client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: wire::aways(
                                &w::Aways::new(
                                    &w::CEILINGS,
                                    w::AwaysParts { rows: List::from_boxed(Box::new([spell])) },
                                )
                                .unwrap(),
                            ),
                        },
                    },
                },
            },
        );
    }
    let template_root = f::chat::messaging_root();
    let template_me = template_root.pages().me(template_root.client()).unwrap();
    let mut me = root.pages().me(root.client()).unwrap().clone();
    let member = me.projects.get_mut(0).unwrap();
    member.role = c::HeldRole::Owner;
    member.templates = f::asking_me(template_me.clone()).projects.get(0).unwrap().templates.clone();
    if status.is_some_and(|fixture| fixture.goals) {
        let mut goal = member.templates.get(0).unwrap().clone();
        goal.number = jig_client::shapes::GOAL;
        goal.tracked = Some(1);
        let mut templates = List::uncapped();
        for template in &member.templates {
            templates.push(template.clone()).unwrap();
        }
        templates.push(goal).unwrap();
        member.templates = templates;
    }
    member.models = List::with_capacity(1);
    member
        .models
        .push(c::MeModel {
            new_until: None,
            number: 1,
            source: jig_client::contract::ModelSource::Stock,
            estimated: false,
            effort: None,
            shown: None,
            accounts: List::with_capacity(0),
            picker: true,
            name: c::Text::new(b"gpt-6-luna".as_slice().into(), 32).unwrap(),
            provider: Some(17),
            description: None,
            prices: c::ModelPrices { input: 0, cached: 0, cache_write: None, output: 0, unit: 1 },
            window: 200_000,
            output: 16_384,
            efforts: List::with_capacity(0),
        })
        .unwrap();
    let mut out = drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
    );
    let mut entered =
        drive(&mut root, d::Event::Went { place: d::Place::Address(Address::Home { project: f::PROJECT }) });
    while let Some(request) = entered.pop() {
        out.push(request);
    }
    let mut delivered = drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: needs_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(
                        temper_fixtures::client_documents::inbox(
                            List::with_capacity(0),
                            0,
                            c::Count { value: 0, limited: false },
                            0,
                            false,
                        ),
                    ))),
                },
            },
        },
    );
    while let Some(request) = delivered.pop() {
        out.push(request);
    }
    let mut live = None;
    for request in &out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(subject) },
                },
        } = request
        {
            match subject {
                c::Subject::Tasks { query, .. } if query.tracked != Some(true) => live = Some(*watch),
                c::Subject::Tasks { .. }
                | c::Subject::Settings
                | c::Subject::Me
                | c::Subject::Task { .. }
                | c::Subject::Tree { .. }
                | c::Subject::Chips { .. }
                | c::Subject::Inbox { .. }
                | c::Subject::Run { .. }
                | c::Subject::Deployment
                | c::Subject::Terminal { .. }
                | c::Subject::Accounts => {}
            }
        }
    }
    let mut lines = List::with_capacity(1);
    lines
        .push(c::SettingLine {
            setting: c::Setting::Application { kind: 2, scope: f::PROJECT, number: 1 },
            name: None,
            follows: if configured { c::SettingFollows::Change } else { c::SettingFollows::Default },
            last: None,
            application: None,
        })
        .unwrap();
    if settings_delivered {
        deliver(
            &mut root,
            settings,
            c::Delivery::Settings(Box::new(c::SettingsDocument {
                newest: 0,
                settings: lines,
                held: List::with_capacity(0),
                pending: List::with_capacity(0),
            })),
        );
    }
    let mut rows = List::with_capacity(1);
    if chat {
        rows.push(super::chats::line()).unwrap();
    }
    let mut refreshed = drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: live.expect("Home chats"),
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(
                        chats.as_ref().map_or(c::Tasks { rows, more: false }, |chats| chats.live.clone()),
                    ))),
                },
            },
        },
    );
    while let Some(request) = refreshed.pop() {
        out.push(request);
    }
    settle_home(&mut root, &mut out, needs.as_ref(), chats.as_ref(), status);
    (root, accounts)
}

pub fn read_done(root: &mut d::Domain, read: Token, value: c::ReadValue) -> Queue<d::Request> {
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone { read, result: jig_client::Readout::Jig(c::ReadResult::Value(value)) },
            },
        },
    )
}

#[must_use]
pub fn home_need_page(object: c::ObjectKey) -> c::ReadValue {
    let mut task = f::task::document();
    task.chip.title = f::task::document_words(b"Add a README");
    match object {
        c::ObjectKey::Question { task: number, message } => {
            task.chip.task = number;
            if message == 7 {
                task.chip.title = f::task::document_words(b"");
            }
            c::ReadValue::Question(Box::new(c::Question {
                task: task.chip,
                from: None,
                message,
                words: c::DocumentText::complete(
                    c::Text::new((if message == 8 { HOME_QUESTION } else { b"Which license?" }).into(), 4096).unwrap(),
                ),
                since: skein_lib::Wall::EPOCH,
                acts: {
                    let mut acts = List::with_capacity(1);
                    acts.push(c::Offered { act: c::ActKind::Answer, admission: c::Admission::Allowed, effect: None })
                        .unwrap();
                    acts
                },
                ending: None,
            }))
        }
        c::ObjectKey::PersonTask(_) => c::ReadValue::PersonTask(Box::new(c::PersonTask {
            task: task.chip,
            revision: task.spec_revision,
            requester: task.requester,
            spec: task.spec,
            address: c::PersonAddress::Party(73),
            taken_by: None,
            since: Some(skein_lib::Wall::EPOCH),
            stall: None,
            deadline: None,
            contract: task.result_contract,
            results: List::with_capacity(0),
            cut: false,
            acts: List::with_capacity(0),
        })),
        c::ObjectKey::Escalation(number) => {
            let held = f::task::held_root();
            let mut task = held.pages().task().unwrap().document(held.client()).unwrap().clone();
            task.chip.task = number;
            task.chip.title = f::task::document_words(b"Add a README");
            if number == 10 {
                task.hold.as_mut().unwrap().reason = c::HoldCategory::Budget;
                task.budget.numbers.spent = 31;
                task.budget.numbers.budget = 40;
            }
            let mut acts = List::with_capacity(2);
            for act in [c::ActKind::Release, c::ActKind::LeaveHeld] {
                acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).unwrap();
            }
            c::ReadValue::Escalation(Box::new(c::Escalation { task: Box::new(task), acts }))
        }
        c::ObjectKey::Proposal(_) => super::cards::batch_proposal(),
        c::ObjectKey::Task(_) | c::ObjectKey::Result(_) | c::ObjectKey::Note { .. } => {
            c::ReadValue::Task(Box::new(task))
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
pub fn settle_home(
    root: &mut d::Domain,
    out: &mut Queue<d::Request>,
    needs: Option<&NeedsFixture<'_>>,
    chats: Option<&ChatFixture>,
    status: Option<&StatusFixture>,
) {
    while let Some(request) = out.pop() {
        let d::Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
        let mut more = match down {
            jig_client::Down::Open { watch, subject }
                if status.and_then(|s| s.away.as_ref()).is_none()
                    && needs.is_some_and(|n| n.decided)
                    && Some(&subject) == d::forge::decided::watch(f::PROJECT, &home_limits().forge).as_ref() =>
            {
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::decisions(
                                        &w::Decisions::new(
                                            &w::CEILINGS,
                                            w::DecisionsParts {
                                                project: f::PROJECT,
                                                rows: List::with_capacity(0),
                                                asked: List::with_capacity(0),
                                                cut: false,
                                            },
                                        )
                                        .unwrap(),
                                    ),
                                },
                            },
                        },
                    },
                )
            }
            jig_client::Down::Open { watch, subject }
                if status.and_then(|s| s.away.as_ref()).is_some()
                    && (Some(&subject) == d::forge::away::watch(&home_limits().forge).as_ref()
                        || Some(&subject) == d::forge::decided::watch(f::PROJECT, &home_limits().forge).as_ref()) =>
            {
                let fixture = status.unwrap().away.as_ref().unwrap();
                let bytes = if Some(&subject) == d::forge::away::watch(&home_limits().forge).as_ref() {
                    let Some(spell) = fixture.spell.clone() else { continue };
                    wire::aways(
                        &w::Aways::new(&w::CEILINGS, w::AwaysParts { rows: List::from_boxed(Box::new([spell])) })
                            .unwrap(),
                    )
                } else if Some(&subject) == d::forge::decided::watch(f::PROJECT, &home_limits().forge).as_ref() {
                    wire::decisions(
                        &w::Decisions::new(
                            &w::CEILINGS,
                            w::DecisionsParts {
                                project: f::PROJECT,
                                rows: fixture.rows.clone(),
                                asked: List::with_capacity(0),
                                cut: false,
                            },
                        )
                        .unwrap(),
                    )
                } else {
                    unreachable!("away fixture watch")
                };
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes,
                                },
                            },
                        },
                    },
                )
            }

            jig_client::Down::Open { watch, subject }
                if Some(&subject) == d::forge::status::watch(f::PROJECT, &home_limits().forge).as_ref()
                    && status.is_some() =>
            {
                let status = status.unwrap();
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::project_status(&status.document),
                                },
                            },
                        },
                    },
                )
            }
            jig_client::Down::Send { attempt, request, .. }
                if Some(&request) == d::forge::status::ask(f::PROJECT, &home_limits().forge).as_ref()
                    && status.is_some_and(|status| status.answered) =>
            {
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Answer {
                                attempt,
                                envelope: skein_lib::RequestEnvelope::Final,
                                why: None,
                                answer: Some(jig_client::Answered::Family(jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::answer(
                                        &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Looked })
                                            .unwrap(),
                                    ),
                                })),
                            },
                        },
                    },
                )
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Tasks { query, .. }) }
                if query.tracked == Some(true) =>
            {
                if let Some(goals) = status.and_then(|fixture| fixture.goal_rows.as_ref()) {
                    let mut more = drive(
                        root,
                        d::Event::Client {
                            event: jig_client::Event::Up {
                                up: jig_client::Up::Watched {
                                    watch,
                                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(goals.live.clone()))),
                                },
                            },
                        },
                    );
                    while let Some(request) = more.pop() {
                        out.push(request);
                    }
                    continue;
                }
                let mut rows = List::uncapped();
                if status.is_some_and(|fixture| fixture.goals) {
                    let mut goal = super::chats::line();
                    goal.tracked = Some(1);
                    rows.push(goal).unwrap();
                }
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Watched {
                                watch,
                                event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                                    rows,
                                    more: false,
                                }))),
                            },
                        },
                    },
                )
            }

            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Inbox { .. }) }
                if needs.is_some() =>
            {
                let value = temper_fixtures::client_documents::inbox(
                    List::with_capacity(0),
                    0,
                    c::Count { value: 0, limited: false },
                    0,
                    false,
                );
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Watched {
                                watch,
                                event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(value))),
                            },
                        },
                    },
                )
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Inbox { group, .. }) } => {
                let Some(needs) = needs else { continue };
                let mut rows = List::with_capacity(u32::try_from(needs.objects.len()).unwrap());
                if group == c::InboxReadGroup::Needs {
                    for object in needs.objects {
                        rows.push(temper_fixtures::client_documents::entry(*object, f::PROJECT, 1, f::NOW, None))
                            .unwrap();
                    }
                }
                read_done(
                    root,
                    read,
                    c::ReadValue::Inbox(Box::new(c::InboxPage {
                        rows,
                        cut: group == c::InboxReadGroup::Needs && needs.cut,
                        before: None,
                        after: None,
                        selection: status.filter(|fixture| fixture.goals).map(|_| {
                            List::from_boxed(Box::new([c::InboxSelection {
                                cursor: c::InboxCursor::Number { number: 1 },
                                goals: List::from_boxed(Box::new([9])),
                            }]))
                        }),
                    })),
                )
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Question { task, message }) } => {
                if !needs.is_some_and(|needs| needs.read) {
                    continue;
                }
                read_done(root, read, home_need_page(c::ObjectKey::Question { task, message }))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Proposal { proposal }) } => {
                if !needs.is_some_and(|needs| needs.read) {
                    continue;
                }
                read_done(root, read, home_need_page(c::ObjectKey::Proposal(proposal)))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Escalation { task }) } => {
                if !needs.is_some_and(|needs| needs.read) {
                    continue;
                }
                read_done(root, read, home_need_page(c::ObjectKey::Escalation(task)))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Task { task }) }
                if status.and_then(|s| s.away.as_ref()).is_some() =>
            {
                let fixture = status.unwrap().away.as_ref().unwrap();
                if fixture.state == d::pages::decided::DecisionTask::Unknown {
                    continue;
                }
                let mut document = f::task::document();
                document.chip.task = task;
                document.chip.title = f::task::document_words(b"welcome-stall");
                match fixture.state {
                    d::pages::decided::DecisionTask::Unknown => unreachable!(),
                    d::pages::decided::DecisionTask::Live => {}
                    d::pages::decided::DecisionTask::Done(at) | d::pages::decided::DecisionTask::Ended(at) => {
                        document.chip.details.phase = c::ChipPhase::Ended;
                        document.chip.details.details.ended_at = Some(at);
                        document.chip.details.details.ending =
                            if let d::pages::decided::DecisionTask::Done(_) = fixture.state {
                                Some(c::ChipEnding::Done(c::ResultValue::Report(f::task::document_words(b"Landed."))))
                            } else {
                                None
                            };
                    }
                }
                read_done(root, read, c::ReadValue::Task(Box::new(document)))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::PersonTask { task }) } => {
                if !needs.is_some_and(|needs| needs.read) {
                    continue;
                }
                read_done(root, read, home_need_page(c::ObjectKey::PersonTask(task)))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { query, .. }) }
                if query.tracked == Some(true) && status.and_then(|fixture| fixture.goal_rows.as_ref()).is_some() =>
            {
                read_done(
                    root,
                    read,
                    c::ReadValue::Tasks(Box::new(status.unwrap().goal_rows.as_ref().unwrap().ended.clone())),
                )
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Notes { scope, .. }) }
                if status.is_some_and(|fixture| fixture.goals) =>
            {
                read_done(root, read, goal_notes(scope))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { .. }) } => read_done(
                root,
                read,
                c::ReadValue::Tasks(Box::new(chats.map_or(
                    c::TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: c::EndedGroup::Rows { rows: List::with_capacity(0), after: None, cut: false },
                    },
                    |chats| chats.ended.clone(),
                ))),
            ),
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Tasks { .. }) }
                if chats.is_some() =>
            {
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Watched {
                                watch,
                                event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(
                                    chats.map_or(c::Tasks { rows: List::with_capacity(0), more: false }, |chats| {
                                        chats.live.clone()
                                    }),
                                ))),
                            },
                        },
                    },
                )
            }
            jig_client::Down::Close { watch } => drive(
                root,
                d::Event::Client {
                    event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::Closed } },
                },
            ),
            jig_client::Down::Save { .. }
            | jig_client::Down::Erase { .. }
            | jig_client::Down::Send { .. }
            | jig_client::Down::Open { .. }
            | jig_client::Down::Continue { .. }
            | jig_client::Down::Read { .. } => continue,
        };
        while let Some(request) = more.pop() {
            out.push(request);
        }
    }
}

#[must_use]
pub fn forge_rows(in_project: bool) -> List<w::ForgeRow> {
    let mut rows = super::connect_forge::first_rows(super::connect_forge::connected(), super::connect_forge::made());
    if in_project {
        let mut parts = rows.get(0).unwrap().clone().into_parts();
        parts.repositories = List::with_capacity(1);
        parts
            .repositories
            .push(
                w::Adoption::new(
                    &w::CEILINGS,
                    w::AdoptionParts {
                        owner: b"work".as_slice().into(),
                        name: b"hello".as_slice().into(),
                        project: f::PROJECT,
                    },
                )
                .unwrap(),
            )
            .unwrap();
        *rows.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
    }
    rows
}

pub const STATUS_NOW: u64 = 345_600_000_000_000;

pub const STATUS_MINUTE: u64 = 60_000_000_000;

#[must_use]
pub fn status_parts() -> w::ProjectStatusParts {
    w::ProjectStatusParts {
        project: f::PROJECT,
        landed: 3,
        last_landed: Some(STATUS_NOW - 6 * STATUS_MINUTE),
        brief: None,
        previous: Some(STATUS_NOW - 9 * 60 * STATUS_MINUTE),
        seen: 1,
        queued: 3,
        red: false,
        branch: b"main".as_slice().into(),
    }
}

pub const HOME_QUESTION: &[u8] = b"Which license?\n\nThe README needs a license.\n\nSaw: No license file.\nChoice: MIT (recommended)\nChoice: Apache";

#[must_use]
pub fn goal_notes(scope: c::NoteScope) -> c::ReadValue {
    let mut rows = List::uncapped();
    if scope == (c::NoteScope::Task { project: f::PROJECT, task: 9 }) {
        rows.push(c::Note {
            number: 1,
            revision: c::Revision { value: 1 },
            scope,
            description: f::task::document_words(b"report: A goal report."),
            body: f::task::document_words(b""),
            writer: c::NoteAuthor::Task { task: 9, attempt: 1 },
            tasks: List::with_capacity(0),
            changed: Some(f::NOW),
            acts: List::with_capacity(0),
            cut: false,
        })
        .unwrap();
    }
    c::ReadValue::Notes(Box::new(c::NotePage { rows, before: None, after: None, cut: false }))
}

pub struct AwayFixture {
    pub spell: Option<w::AwayRow>,
    pub rows: List<w::Decision>,
    pub state: d::pages::decided::DecisionTask,
}

#[must_use]
pub fn away_fixture(count: u32) -> AwayFixture {
    let mut rows = List::with_capacity(count);
    for number in 1..=count {
        rows.push(
            w::Decision::new(
                &w::CEILINGS,
                w::DecisionParts {
                    number,
                    task: 99,
                    at: STATUS_NOW - 55 * STATUS_MINUTE,
                    question: b"Fix in the store or serve?".as_slice().into(),
                    took: b"In serve".as_slice().into(),
                    why: b"The store changes a record.".as_slice().into(),
                    overturned: false,
                    reach: w::Reach::new(
                        &w::CEILINGS,
                        w::ReachParts {
                            kind: w::ReachKind::Wording,
                            class: w::ReachClass::Decide,
                            reason: w::ReachReason::Kind,
                            under: 0,
                            sets: 0,
                            matched: 0,
                            away: false,
                        },
                    )
                    .unwrap(),
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    AwayFixture {
        spell: Some(
            w::AwayRow::new(
                &w::CEILINGS,
                w::AwayRowParts {
                    project: f::PROJECT,
                    since: STATUS_NOW - 50 * STATUS_MINUTE,
                    until: None,
                    landed: 1,
                    off: None,
                },
            )
            .unwrap(),
        ),
        rows,
        state: d::pages::decided::DecisionTask::Live,
    }
}

#[must_use]
pub fn away_home(
    away: AwayFixture,
    beginning: bool,
    objects: &[c::ObjectKey],
    parts: w::ProjectStatusParts,
) -> d::Domain {
    away_home_with_needs(away, beginning, NeedsFixture { objects, cut: false, read: true, decided: true }, parts)
}

#[must_use]
pub fn away_home_with_needs(
    away: AwayFixture,
    beginning: bool,
    needs: NeedsFixture<'_>,
    parts: w::ProjectStatusParts,
) -> d::Domain {
    let fixture = StatusFixture {
        document: w::ProjectStatus::new(&w::CEILINGS, parts).unwrap(),
        answered: true,
        goals: false,
        shown: false,
        goal_rows: None,
        away: Some(away),
    };
    home_documents_with_status(
        forge_rows(true),
        !beginning,
        !beginning,
        Some(needs),
        None,
        Some(&[]),
        true,
        Some(&fixture),
    )
    .0
}

#[must_use]
pub fn default_away(away: AwayFixture) -> d::Domain {
    away_home(away, false, &[], status_parts())
}

pub fn overturn_drive(root: &mut d::Domain, event: d::Event) -> Queue<d::Request> {
    let mut limits = home_limits();
    limits.client.domain.text = 512;
    let mut out = Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::from_nanos(STATUS_NOW), limits },
        event,
        &mut out,
    );
    out
}

#[must_use]
pub fn overturn_attempt(out: &Queue<d::Request>) -> Token {
    for request in out {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        } = request
        {
            return *attempt;
        }
    }
    panic!("overturn attempt")
}

#[must_use]
pub fn overturn_answer(outcome: w::Outcome, message: Option<c::Answer>) -> d::Domain {
    let mut root = default_away(away_fixture(2));
    overturn_drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyServed {
                    family: wire::FORGE,
                    versions: Some(c::VersionRange { first: wire::CLIENT_VERSION, last: wire::CLIENT_VERSION }),
                },
            },
        },
    );
    let out = overturn_drive(
        &mut root,
        d::Event::Act {
            action: d::Action::Pages(d::pages::Act::Away(d::pages::away::Act::Overturn {
                project: f::PROJECT,
                number: 1,
            })),
        },
    );
    let attempt = overturn_attempt(&out);
    let out = overturn_drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
                    })),
                },
            },
        },
    );
    if let Some(answer) = message {
        let attempt = overturn_attempt(&out);
        overturn_drive(
            &mut root,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        why: None,
                        answer: Some(jig_client::Answered::Jig(answer)),
                    },
                },
            },
        );
    }
    root
}
