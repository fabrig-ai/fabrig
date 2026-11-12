//! The draft's sheet through the host (client/tui/pages/design-session.md, 1).
use super::*;
use temper_client_protocol_wire::{self as wire, v1 as f};

pub(super) fn host(linked: bool, proposed: bool) -> Host {
    let mut host = super::host();
    let first = start(&mut host, Some(Secret::new(Box::new([7; 40]))));
    let hello = hello_for(&host);
    let mut duties =
        host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Record(hello) }));
    duties.asked.extend(first.asked);
    settle(&mut host, duties, linked, proposed);
    duties = give(&mut host, Event::Act { act: Act::Go { address: Address::Chat { task: 9 } } });
    settle(&mut host, duties, linked, proposed);
    for _ in 0..host.env.limits.client.domain.watches {
        let chat = host.root.pages().chat().unwrap();
        if chat.draft.is_some()
            && matches!(root::pages::home::repository(&host.root, 3), root::pages::home::Repository::In { .. })
            && (!linked || chat.goal_from(&host.root).is_some())
        {
            break;
        }
        let due = root::next_deadline(&host.root).expect("fixture reads retry");
        host.clocks(due, Wall::EPOCH);
        let duties = host.report(Report::Due);
        settle(&mut host, duties, linked, proposed);
    }
    assert!(host.root.pages().chat().unwrap().draft.is_some());
    assert_eq!(host.root.pages().chat().unwrap().goal_from(&host.root), linked.then_some((25, 1)));
    assert_eq!(temper_client_tui::goal_offered(&host.root), !linked);
    host
}

fn settle(host: &mut Host, duties: Duties, linked: bool, proposed: bool) {
    let mut pending = std::collections::VecDeque::from(duties.asked);
    while let Some(request) = pending.pop_front() {
        let Request::Exchange { exchange, message, .. } = &request else {
            if let Some(report) = background_reply(&request) {
                pending.extend(host.report(report).asked);
            }
            continue;
        };
        if let Message::Read(bytes) = message {
            let owned = w::Owned::decode(&w::CEILINGS, &mut Reader::new(bytes)).unwrap();
            if owned.owner() == &w::Owner::Jig {
                let read = w::Read::decode(&w::CEILINGS, &mut Reader::new(owned.record())).unwrap();
                if let w::Read::Notes(notes) = read {
                    pending.extend(read_reply(host, *exchange, &document(notes.scope().clone())).asked);
                    continue;
                }
            } else {
                let envelope = w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(owned.record())).unwrap();
                if let f::Read::Setting(setting) = wire::decode_reading(envelope.record(), &f::CEILINGS).unwrap().read()
                    && setting.kind() == 7
                {
                    let value = f::ProjectWays::new(
                        &f::CEILINGS,
                        f::ProjectWaysParts { drafts: true, goals: true, changes: false, way: 4 },
                    )
                    .unwrap();
                    let mut writer = Writer::new(usize::try_from(value.measure()).unwrap());
                    value.encode(&mut writer).unwrap();
                    let record = wire::setting_answer(
                        &f::SettingAnswer::new(&f::CEILINGS, f::SettingAnswerParts { value: Some(writer.finish()) })
                            .unwrap(),
                    );
                    let answer = w::ReadAnswer::Value(family(record));
                    let mut writer = Writer::new(usize::try_from(answer.measure()).unwrap());
                    answer.encode(&mut writer).unwrap();
                    pending.extend(reply(host, *exchange, writer.finish()).asked);
                    continue;
                }
            }
        }
        if let Some(report) = background_reply(&request) {
            pending.extend(host.report(report).asked);
        }
        if let Message::Watch(bytes) = message {
            let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(bytes)).unwrap();
            if open.subject().owner() == &w::Owner::Jig {
                let subject = w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).unwrap();
                if subject == w::Subject::Me {
                    pending.extend(delivery(host, open.watch(), &w::DeliveryValue::Me(me(proposed))).asked);
                } else if let w::Subject::Task(task) = subject
                    && task.task() == 9
                {
                    let mut task = records::task_record().into_parts();
                    task.project = 3;
                    task.chip = chat_chip(9, b"Plan");
                    task.activity = 0;
                    task.result = None;
                    task.delegates = Box::default();
                    pending.extend(
                        delivery(
                            host,
                            open.watch(),
                            &w::DeliveryValue::Task(Box::new(w::Task::new(&w::CEILINGS, task).unwrap())),
                        )
                        .asked,
                    );
                }
            } else {
                let envelope =
                    w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).unwrap();
                let watch = wire::decode_watching(envelope.record(), &f::CEILINGS).unwrap();

                let bytes = if watch.watch() == &f::Watch::Forges {
                    Some(forges())
                } else if watch.watch() == &f::Watch::Dismissed {
                    Some(dismissed(linked))
                } else {
                    None
                };
                if let Some(bytes) = bytes {
                    let event = w::StreamEvent::Delivery(
                        w::Delivery::new(
                            &w::CEILINGS,
                            w::DeliveryParts { watch: open.watch(), document: family(bytes) },
                        )
                        .unwrap(),
                    );
                    pending.extend(
                        host.report(Report::Heard(jig_client::Event::Streamed {
                            message: jig_client::Streamed::Record(stream_bytes(&event)),
                        }))
                        .asked,
                    );
                }
            }
        }
    }
}

fn family(record: Box<[u8]>) -> w::Owned {
    w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts {
            owner: w::Owner::Family(w::Family::new(&w::CEILINGS, w::FamilyParts { number: wire::FORGE }).unwrap()),
            version: wire::CLIENT_VERSION,
            record,
        },
    )
    .unwrap()
}
fn me(proposed: bool) -> w::Me {
    let mut me = launchpad_me(true).into_parts();
    let mut reader = Reader::new(&me.projects);
    let project = skein_codec::piece::split(&mut reader).unwrap().unwrap();
    let mut project = w::MeProject::decode(&w::CEILINGS, &mut Reader::new(&project)).unwrap().into_parts();
    let mut reader = Reader::new(&project.templates);
    let template = skein_codec::piece::split(&mut reader).unwrap().unwrap();
    let template = w::Template::decode(&w::CEILINGS, &mut Reader::new(&template)).unwrap();
    let mut templates = vec![];
    for number in [2, 4] {
        let mut way = template.clone().into_parts();
        way.number = number;
        way.name = records::wire_text(if number == 2 { b"goal" } else { b"goal for a draft" });
        way.tracked = Some(10);
        if proposed {
            way.budget = w::TemplateBudget::Amount(
                w::TemplateAmount::new(&w::CEILINGS, w::TemplateAmountParts { value: 101 }).unwrap(),
            );
        }
        let way = w::Template::new(&w::CEILINGS, way).unwrap();
        let mut writer = Writer::new(usize::try_from(way.measure()).unwrap());
        way.encode(&mut writer).unwrap();
        templates.push(writer.finish());
    }
    project.templates = [project.templates.as_ref(), pieces(&templates).as_ref()].concat().into();
    if proposed {
        let mut creating = project.creating.take().unwrap().into_parts();
        let mut within = creating.within.into_parts();
        within.maximum_budget = 200;
        creating.within = w::BoundsCreate::new(&w::CEILINGS, within).unwrap();
        creating.beyond = w::Beyond::Proposed(
            w::BeyondProposed::new(
                &w::CEILINGS,
                w::BeyondProposedParts {
                    to: w::Recipient::Party(
                        w::RecipientParty::new(&w::CEILINGS, w::RecipientPartyParts { value: 7 }).unwrap(),
                    ),
                },
            )
            .unwrap(),
        );
        project.creating = Some(w::CreatingBounds::new(&w::CEILINGS, creating).unwrap());
    }
    me.projects = pieces(&[project_bytes(&w::MeProject::new(&w::CEILINGS, project).unwrap())]);
    w::Me::new(&w::CEILINGS, me).unwrap()
}
fn document(scope: w::NoteScope) -> w::ReadValue {
    let note = w::Note::new(
        &w::CEILINGS,
        w::NoteParts {
            number: 1,
            revision: w::Revision::new(&w::CEILINGS, w::RevisionParts { value: 1 }).unwrap(),
            scope,
            description: records::wire_text(b"Plan"),
            body: records::wire_text(b"# Plan\n\nWords\n"),
            writer: w::NoteAuthor::Party(
                w::NoteAuthorParty::new(&w::CEILINGS, w::NoteAuthorPartyParts { party: 7 }).unwrap(),
            ),
            tasks: List::with_capacity(0),
            changed: None,
            acts: Box::default(),
            cut: false,
        },
    )
    .unwrap();
    let mut writer = Writer::new(usize::try_from(note.measure()).unwrap());
    note.encode(&mut writer).unwrap();
    w::ReadValue::Notes(
        w::NotePage::new(
            &w::CEILINGS,
            w::NotePageParts { rows: pieces(&[writer.finish()]), before: None, after: None, cut: false },
        )
        .unwrap(),
    )
}
fn forges() -> Box<[u8]> {
    let row = f::ForgeRow::new(
        &f::CEILINGS,
        f::ForgeRowParts {
            number: 2,
            host: b"git.example".as_slice().into(),
            kind: f::Kind::Forgejo,
            mode: f::Mode::Owned,
            connection: f::Connection::Connected(
                f::Connected::new(&f::CEILINGS, f::ConnectedParts { id: 1, login: b"temper".as_slice().into() })
                    .unwrap(),
            ),
            offer: f::Offer::NotOffered,
            first: None,
            starts_with: f::StartsWith::Repository,
            installations: List::with_capacity(0),
            ended: None,
            app: None,
            web: None,
            repositories: List::from_boxed(Box::new([f::Adoption::new(
                &f::CEILINGS,
                f::AdoptionParts { owner: b"work".as_slice().into(), name: b"shop".as_slice().into(), project: 3 },
            )
            .unwrap()])),
            owned: None,
            admin: false,
            ended_stop: None,
            read_only: false,
            makings: List::with_capacity(0),
        },
    )
    .unwrap();
    wire::forges(
        &f::Forges::new(&f::CEILINGS, f::ForgesParts { forges: List::from_boxed(Box::new([row])), linked: false })
            .unwrap(),
    )
}
fn dismissed(linked: bool) -> Box<[u8]> {
    let mut dismissed = List::with_capacity(wire::DISMISSED);
    if linked {
        dismissed
            .push(f::PersonFact::GoalFrom(
                f::GoalFrom::new(&f::CEILINGS, f::GoalFromParts { project: 3, chat: 9, goal: 25, at: 1 }).unwrap(),
            ))
            .unwrap();
    }
    wire::dismissals(&f::Dismissals::new(&f::CEILINGS, f::DismissalsParts { dismissed }).unwrap())
}

pub(super) fn events() {
    for send in [false, true] {
        let mut host = host(false, false);
        typed(&mut host, b"/goal");
        give(&mut host, if send { Event::Send } else { Event::Command { command: Command::Goal } });
        let start = host.root.pages().start().unwrap();
        assert_eq!(start.given.as_deref(), Some(b"docs/design/draft/plan.md".as_slice()));
        assert_eq!(start.chat, Some(9));
        assert_eq!(start.name.as_ref(), b"plan");
        assert!(!start.always);
        assert_eq!(start.way, Some(4));
        assert_eq!(host.state().start_chat(), Some(9));
        assert_eq!(host.words(), []);
        assert_eq!(host.root.pages().address(), &Address::Chat { task: 9 });
        let duties = give(&mut host, Event::Way { template: 2 });
        let start = host.root.pages().start().unwrap();
        assert_eq!(start.way, Some(2));
        assert!(start.pending.is_some());
        let asks = sent_ask(&duties);
        assert_eq!(asks.len(), 1);
        assert!(matches!(asks[0].1, w::Ask::Create(_)));
        reply(
            &mut host,
            asks[0].0,
            final_bytes(&w::Final::Refused(
                w::Refused::new(&w::CEILINGS, w::RefusedParts { refusal: w::Refusal::NotOffered, at: None }).unwrap(),
            )),
        );
        assert!(host.root.pages().start().unwrap().error.is_some());
        assert!(text(&mut host).contains("This act is not offered"));
        assert!(text(&mut host).contains("Start a goal from plan.md. How should it run?"));
        host.frame();
        assert_eq!(host.event_of(&Key::Escape), Some(Event::Act { act: Act::StartCancel }));
        host.report(Report::Key(Key::Escape));
        assert!(host.root.pages().start().is_none());
        assert_eq!(host.state().start_chat(), None);
        assert_eq!(
            give(&mut host, Event::Own { command: Own::Open }).open.as_deref(),
            Some(b"http://127.0.0.1:8367/chats/9".as_slice())
        );
    }
}

pub(super) fn navigation() {
    for end in ["made", "proposed", "cancel", "leave"] {
        let mut host = host(false, end == "proposed");
        give(&mut host, Event::Command { command: Command::Goal });
        if end == "cancel" {
            give(&mut host, Event::Act { act: Act::StartCancel });
        } else if end == "leave" {
            give(&mut host, Event::Command { command: Command::New });
        } else {
            let duties = give(&mut host, Event::Way { template: 4 });
            let asks = sent_ask(&duties);
            assert_eq!(asks.len(), 1, "{end}: {:?}", host.root.pages().start());
            let answer = if end == "made" {
                assert!(matches!(asks[0].1, w::Ask::Create(_)));
                created_bytes(25)
            } else {
                assert!(matches!(asks[0].1, w::Ask::Propose(_)));
                let outcome = w::DocumentOutcome::Proposed(
                    w::OutcomeProposed::new(&w::CEILINGS, w::OutcomeProposedParts { proposal: 1, to: 7 }).unwrap(),
                );
                final_bytes(&w::Final::Done(
                    w::Done::new(
                        &w::CEILINGS,
                        w::DoneParts {
                            outcome: w::Outcome::Document(
                                w::OutcomeDocument::new(&w::CEILINGS, w::OutcomeDocumentParts { value: outcome })
                                    .unwrap(),
                            ),
                        },
                    )
                    .unwrap(),
                ))
            };
            reply(&mut host, asks[0].0, answer);
        }
        assert!(host.root.pages().start().is_none(), "{end}");
        assert_eq!(host.state().start_chat(), None, "{end}");
        assert_eq!(
            host.root.pages().address(),
            &if end == "leave" { Address::Home { project: 3 } } else { Address::Chat { task: 9 } },
            "{end}"
        );
        assert_eq!(
            host.state().notice(),
            if end == "proposed" { Some(&temper_client_tui::Notice::Proposed) } else { None },
            "{end}"
        );
        if end == "proposed" {
            assert!(text(&mut host).contains("Proposed: the goal starts once it is accepted."));
        }
    }
}
