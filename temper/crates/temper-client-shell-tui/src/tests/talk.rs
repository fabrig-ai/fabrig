//! Talk commands and leaving through the shipped host (client/tui/pages/talk.md, 1).
use super::*;
use temper_client_view::shared;

const ROLE: u64 = 7;
const MINUTE: u64 = 60_000_000_000;
fn role_chip() -> w::Chip {
    let chip = chat_chip(ROLE, b"Porter");
    let mut reader = Reader::new(chip.rows());
    let mut rows = Vec::new();
    while let Some(bytes) = skein_codec::piece::split(&mut reader).unwrap() {
        let mut fact = w::Fact::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
        if let w::Fact::ChipDetails(row) = fact {
            let mut row = row.into_parts();
            let mut fields = row.value.into_parts();
            fields.live_delegates = 0;
            row.value = w::ChipDetailsFields::new(&w::CEILINGS, fields).unwrap();
            fact = w::Fact::ChipDetails(Box::new(w::ChipDetailsRow::new(&w::CEILINGS, row).unwrap()));
        }
        rows.push(fact);
    }
    records::chip_from_rows(&rows)
}
fn roles_host() -> Host {
    roles_fixture().0
}
fn roles_fixture() -> (Host, Duties) {
    let (mut host, _, mut duties) = signed_host(true);
    let mut watched = None;
    for _ in 0..host.env.limits.client.domain.watches {
        for request in &duties.asked {
            if let Request::Exchange { message: Message::Watch(bytes), .. } = request {
                let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(bytes)).unwrap();
                if open.subject().owner() != &w::Owner::Jig {
                    continue;
                }
                if let w::Subject::Tasks(tasks) =
                    w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).unwrap()
                    && tasks.query().label() == &Some(3)
                {
                    watched = Some(open.watch());
                }
            }
        }
        if watched.is_some() {
            break;
        }
        let due = root::next_deadline(&host.root).expect("roles retry");
        host.clocks(due, Wall::EPOCH);
        let more = host.report(Report::Due);
        duties.asked.extend(admit(&mut host, more).asked);
    }
    let mut line = records::task_line_record().into_parts();
    line.project = 3;
    line.label = 3;
    line.tracked = None;
    line.chip = role_chip();
    line.waits_on = w::WaitsOn::News;
    let line = w::TaskLine::new(&w::CEILINGS, line).unwrap();
    let mut writer = Writer::new(usize::try_from(line.measure()).unwrap());
    line.encode(&mut writer).unwrap();
    let tasks = w::Tasks::new(&w::CEILINGS, w::TasksParts { rows: pieces(&[writer.finish()]), more: false }).unwrap();
    let arrived = delivery(&mut host, watched.expect("roles watch"), &w::DeliveryValue::Tasks(tasks));
    duties.asked.extend(admit(&mut host, arrived).asked);
    (host, duties)
}
fn activity(position: u64, line: w::Activity) -> w::DecodedActivityLine {
    let source =
        w::ActivityLine::new(&w::CEILINGS, w::ActivityLineParts { task: ROLE, position, at: 0, line }).unwrap();
    w::DecodedActivityLine::new(&w::CEILINGS, w::DecodedActivityLineParts { source, decoded: None }).unwrap()
}
fn talk_host(open: bool, cost: bool) -> Host {
    talk_fixture(open, cost, false, false)
}
#[expect(
    clippy::fn_params_excessive_bools,
    reason = "fixture flags independently supply the open talk, cost, reply and proposal"
)]
fn talk_fixture(open: bool, cost: bool, choices: bool, proposed: bool) -> Host {
    talk_setup(open, cost, choices, proposed).0
}
#[expect(clippy::fn_params_excessive_bools, reason = "fixture flags independently supply talk facts")]
fn talk_setup(open: bool, cost: bool, choices: bool, proposed: bool) -> (Host, Duties) {
    let (mut host, mut history) = roles_fixture();
    typed(&mut host, b"/talk");
    give(&mut host, Event::Send);
    assert!(host.state().talk_list());
    assert!(text(&mut host).contains("Talk with a role on Project"));
    host.frame();
    let duties = host.report(Report::Key(Key::Enter));
    let mut duties = admit_watches(&mut host, duties);
    assert_eq!(host.root.pages().address(), &Address::Talk { project: 3, task: ROLE });
    let task_watch = watch(
        &duties,
        &w::Subject::Task(w::SubjectTask::new(&w::CEILINGS, w::SubjectTaskParts { task: ROLE }).unwrap()),
    );
    let read = read_exchange(&duties, "activity");
    let mut task = records::task_record().into_parts();
    task.project = 3;
    task.chip = role_chip();
    task.activity = if choices {
        3
    } else if cost {
        2
    } else {
        0
    };
    task.result = None;
    task.delegates = Box::default();
    task.waits_on = w::WaitsOn::News;
    let party = host.root.pages().me(host.root.client()).unwrap().party.number;
    task.talk = open.then(|| w::Talk::new(&w::CEILINGS, w::TalkParts { party, opened: 0, last: 0 }).unwrap());
    let offers: Vec<_> = [w::ActKind::Talk, w::ActKind::Leave]
        .into_iter()
        .map(|act| {
            let offer =
                w::Offered::new(&w::CEILINGS, w::OfferedParts { act, admission: w::Admission::Allowed, effect: None })
                    .unwrap();
            let mut writer = Writer::new(usize::try_from(offer.measure()).unwrap());
            offer.encode(&mut writer).unwrap();
            writer.finish()
        })
        .collect();
    task.acts = pieces(&offers);
    if proposed {
        let key = records::pending_proposal_record().key().clone();
        task.proposals = List::from_boxed(Box::new([key]));
    }
    let arrived =
        delivery(&mut host, task_watch, &w::DeliveryValue::Task(Box::new(w::Task::new(&w::CEILINGS, task).unwrap())));
    duties.asked.extend(admit_watches(&mut host, arrived).asked);
    let mut rows = if cost {
        vec![
            activity(
                1,
                w::Activity::TalkOpened(w::TalkOpened::new(&w::CEILINGS, w::TalkOpenedParts { party }).unwrap()),
            ),
            activity(
                2,
                w::Activity::RunEnded(
                    w::ActivityRunEnded::new(
                        &w::CEILINGS,
                        w::ActivityRunEndedParts { attempt: 44, end: w::RunEnd::Parked, spent: 42 },
                    )
                    .unwrap(),
                ),
            ),
        ]
    } else {
        Vec::new()
    };
    if choices {
        use smith_transcript as s;
        let mut turn = s::decode_archived(&s::CEILINGS, records::FULL_TURN).unwrap().into_parts();
        let text = s::Block::Text(s::Text::new(&s::CEILINGS, s::TextParts { text: "Add it to my rules?\n\n1. Yes, add it — it applies from my next run\n2. Just for this port\n3. Tell it something else".as_bytes().into(), replay: None }).unwrap());
        turn.messages = List::from_boxed(Box::new([s::Message::new(
            &s::CEILINGS,
            s::MessageParts { role: s::Role::Assistant, blocks: List::from_boxed(Box::new([text])) },
        )
        .unwrap()]));
        let turn = s::Turn::new(&s::CEILINGS, turn).unwrap();
        let mut writer = Writer::new(usize::try_from(turn.measure()).unwrap());
        turn.encode(&mut writer).unwrap();
        rows.push(records::turn_line(3, 0, 1, &writer.finish()));
    }
    if proposed {
        let query = read_exchange(&duties, "proposal");
        let mut proposal = records::pending_proposal_record().into_parts();
        proposal.project = 3;
        proposal.action = w::ProposalAction::Settings(Box::new(
            w::ProposalActionSettings::new(
                &w::CEILINGS,
                w::ProposalActionSettingsParts { against: 0, items: List::with_capacity(0), review: None, made: None },
            )
            .unwrap(),
        ));
        let offer = w::Offered::new(
            &w::CEILINGS,
            w::OfferedParts { act: w::ActKind::Accept, admission: w::Admission::Allowed, effect: None },
        )
        .unwrap();
        let mut writer = Writer::new(usize::try_from(offer.measure()).unwrap());
        offer.encode(&mut writer).unwrap();
        proposal.acts = pieces(&[writer.finish()]);
        duties.asked.extend(
            read_reply(&mut host, query, &w::ReadValue::Proposal(w::Proposal::new(&w::CEILINGS, proposal).unwrap()))
                .asked,
        );
    }
    let mut page = records::activity_rows(&rows).into_parts();
    page.after = None;
    page.cut = false;
    duties.asked.extend(
        read_reply(&mut host, read, &w::ReadValue::Activity(w::ActivityPage::new(&w::CEILINGS, page).unwrap())).asked,
    );
    duties.asked.retain(
        |request| matches!(request, Request::Exchange { exchange, message: Message::Read(_), .. } if *exchange != read),
    );
    history.asked.extend(admit(&mut host, duties).asked);
    host.clocks(host.env.now, Wall::from_nanos(9 * MINUTE));
    host.report(Report::Due);
    host.frame();
    (host, history)
}
#[test]
fn talk_opens_by_prefix_and_its_list_lowers_on_escape_or_typing() {
    for words in [b"/talk por".as_slice(), b"/talk POR"] {
        let mut host = roles_host();
        typed(&mut host, words);
        give(&mut host, Event::Send);
        assert_eq!(host.root.pages().address(), &Address::Talk { project: 3, task: ROLE });
        assert!(host.words().is_empty());
    }
    let mut host = roles_host();
    typed(&mut host, b"/talk zed");
    give(&mut host, Event::Send);
    assert_eq!(host.words(), b"/talk zed");
    assert!(text(&mut host).contains("No role on Project is named zed · /talk lists them"));
    let long = format!("/talk {}", "é".repeat(4096));
    typed(&mut host, long.as_bytes());
    give(&mut host, Event::Send);
    assert_eq!(host.words(), long.as_bytes(), "a long not-found name stays in the box");
    for typing in [false, true] {
        give(&mut host, Event::Own { command: Own::Talk });
        assert!(host.state().talk_list());
        if typing {
            typed(&mut host, b"Other words");
        } else {
            host.report(Report::Key(Key::Escape));
        }
        assert!(!host.state().talk_list());
        assert_eq!(host.root.pages().address(), &Address::Home { project: 3 });
    }
}
#[test]
fn leaving_a_talk_notices_its_minutes_and_known_cost_then_goes_home() {
    for (open, cost, exit, notice) in [
        (true, true, false, Some("Left Porter after 9 min · 42. It writes its Now.")),
        (true, false, true, Some("Left Porter after 9 min. It writes its Now.")),
        (false, false, false, None),
    ] {
        let mut host = talk_host(open, cost);
        assert!(text(&mut host).contains("talking with Porter"));
        let duties = if exit {
            typed(&mut host, b"/exit");
            give(&mut host, Event::Send)
        } else {
            assert_eq!(host.event_of(&Key::Escape), Some(Event::Leave));
            host.report(Report::Key(Key::Escape))
        };
        let duties = stored(&mut host, duties);
        assert!(!duties.close);
        assert_eq!(host.root.pages().address(), &Address::Home { project: 3 });
        if let Some(notice) = notice {
            assert!(text(&mut host).contains(notice), "{}", text(&mut host));
        } else {
            assert!(host.state().notice().is_none());
        }
        assert_eq!(
            sent_ask(&duties).iter().filter(|(_, ask)| matches!(ask, w::Ask::Leave(_))).count(),
            usize::from(open),
            "duties={:?}, link={:?}",
            duties.asked,
            host.root.client().link().state
        );
    }
    let mut host = talk_host(true, true);
    host.clocks(host.env.now, Wall::EPOCH);
    give(&mut host, Event::Leave);
    assert!(text(&mut host).contains("after 1 min"), "the minimum length is one minute");
}
#[test]
fn a_talk_keeps_tell_open_and_quit_but_exit_leaves() {
    let mut host = talk_host(true, true);
    give(&mut host, Event::Own { command: Own::Open });
    assert!(host.opens.iter().any(|opening| matches!(opening, crate::host::Opening::Page { address } if address.as_ref() == b"http://127.0.0.1:8367/projects/3/team?role=7")));
    typed(&mut host, b"/tell The export button is grey");
    give(&mut host, Event::Send);
    let tell = host.root.pages().tell().unwrap();
    assert_eq!(tell.project, 3);
    assert_eq!(tell.words.as_ref(), b"The export button is grey");
    typed(&mut host, b"/quit");
    assert!(give(&mut host, Event::Send).close);
}

#[test]
fn a_roles_choices_send_talk_words_and_accept_confirms_the_proposal_in_place() {
    let mut host = talk_fixture(true, true, true, true);
    assert_eq!(
        temper_client_tui::reply_number(&host.view, b"1"),
        Some(Event::Answer { words: b"Yes, add it".as_slice().into() }),
        "{}",
        text(&mut host)
    );
    typed(&mut host, b"1");
    let duties = give(&mut host, Event::Send);
    assert!(sent_ask(&duties).iter().any(|(_, ask)| matches!(ask, w::Ask::Talk(talk) if talk.project() == 3 && talk.task() == ROLE && talk.words() == b"Yes, add it")));
    assert!(host.words().is_empty());
    let mut host = talk_fixture(true, true, false, true);
    assert!(text(&mut host).contains("1. Accept it"));
    let object = jig_client::contract::ObjectKey::Proposal(jig_client::contract::ProposalKey {
        proposer: jig_client::contract::Proposer::Party(11),
        number: 13,
    });
    assert_eq!(host.event_of(&Key::Enter), Some(Event::Accept { object, revision: None }));
    let duties = host.report(Report::Key(Key::Enter));
    let duties = stored(&mut host, duties);
    let asks = sent_ask(&duties);
    let (exchange, ask) =
        asks.iter().find(|(_, ask)| matches!(ask, w::Ask::Decide(_))).expect("Accept then Confirm sends Decide");
    let w::Ask::Decide(decide) = ask else { unreachable!() };
    assert!(matches!(decide.on(), w::DecideOn::Proposal(proposal) if proposal.running() == &Some(w::Running::Moves)));
    let final_answer = w::Final::Refused(
        w::Refused::new(&w::CEILINGS, w::RefusedParts { refusal: w::Refusal::NotOffered, at: None }).unwrap(),
    );
    reply(&mut host, *exchange, final_bytes(&final_answer));
    let card = host.root.pages().chat().unwrap().cards.get(0).unwrap();
    let expected = shared::outcome_refusal(card.outcome.as_ref().unwrap(), host.view.words().view()).unwrap();
    assert!(text(&mut host).contains(core::str::from_utf8(&expected).unwrap()));
    typed(&mut host, b"Other words");
    assert!(host.state().proposal_aside(&object));
    typed(&mut host, b"");
    assert!(!text(&mut host).contains("1. Accept it"));
}

#[test]
fn telling_on_a_roles_talk_files_in_its_area() {
    use temper_client_protocol_wire::{self as wire, v1 as f};
    let (mut host, duties) = talk_setup(true, false, false, false);
    let watched = duties.asked.iter().find_map(|request| {
        let Request::Exchange { message: Message::Watch(bytes), .. } = request else { return None };
        let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(bytes)).unwrap();
        if open.subject().owner() == &w::Owner::Jig { return None; }
        let envelope = w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).unwrap();
        let reading = wire::decode_watching(envelope.record(), &f::CEILINGS).unwrap();
        matches!(reading.watch(), f::Watch::Findings(subject) if subject.project() == 3).then_some(open.watch())
    }).expect("the talk's project reads findings");
    let task = f::Holder::Task(f::HolderTask::new(&f::CEILINGS, f::HolderTaskParts { number: ROLE }).unwrap());
    let document = f::Findings::new(&f::CEILINGS, f::FindingsParts {
        project: 3, rows: List::with_capacity(0),
        areas: List::from_boxed(Box::new([f::AreaRow::new(&f::CEILINGS, f::AreaRowParts {
            name: b"ports".as_slice().into(), words: b"Porting".as_slice().into(),
            owner: f::AreaOwner::Task(f::AreaTask::new(&f::CEILINGS, f::AreaTaskParts { number: ROLE }).unwrap()),
            holder: Some(task.clone()),
        }).unwrap()])), triager: None, holder: task, cut: false,
    }).unwrap();
    let document = w::Owned::new(&w::CEILINGS, w::OwnedParts {
        owner: w::Owner::Family(w::Family::new(&w::CEILINGS, w::FamilyParts { number: wire::FORGE }).unwrap()),
        version: wire::CLIENT_VERSION, record: wire::findings(&document),
    }).unwrap();
    let event = w::StreamEvent::Delivery(w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch: watched, document }).unwrap());
    let duties = host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Record(stream_bytes(&event)) }));
    admit(&mut host, duties);
    let (_, file) = super::tell::send(&mut host, b"/tell The port is stuck").expect("one filing");
    assert_eq!(file.area().as_deref(), Some(b"ports".as_slice()));
    assert_eq!(file.body(), b"The port is stuck\n\nPage: /projects/3/talk/7 (Porter)");
}
