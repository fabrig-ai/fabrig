//! Talk commands and leaving through the shipped host (client/tui/pages/talk.md, 1).
use super::*;
use smith_transcript::Record;
use temper_client_view::shared;

const ROLE: u64 = 7;
const MINUTE: u64 = 60_000_000_000;
fn role_chip() -> w::Chip {
    let chip = chat_chip(ROLE, b"Porter");
    let mut reader = Reader::new(chip.rows.as_ref());
    let mut rows = Vec::new();
    while let Some(bytes) = skein_codec::piece::split(&mut reader).unwrap() {
        let mut fact = w::Fact::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
        if let w::Fact::ChipDetails(row) = fact {
            let mut row = row;
            let mut fields = row.value;
            fields.live_delegates = 0;
            row.value = fields;
            fact = w::Fact::ChipDetails(row);
        }
        rows.push(fact);
    }
    records::chip_from_rows(&rows)
}
pub(super) fn roles_host() -> Host {
    roles_fixture().0
}
fn roles_fixture() -> (Host, Duties) {
    let (mut host, _, mut duties) = signed_host(true);
    let mut watched = None;
    for _ in 0..host.env.limits.client.domain.watches {
        for open in all_openings(&duties) {
            {
                if open.subject.owner != w::Owner::Jig {
                    continue;
                }
                if let w::Subject::Tasks(tasks) =
                    w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject.record.as_ref())).unwrap()
                    && tasks.query.label == Some(3)
                {
                    watched = Some(open.watch);
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
    let mut line = records::task_line_record();
    line.project = 3;
    line.label = 3;
    line.tracked = None;
    line.chip = role_chip();
    line.waits_on = w::WaitsOn::News;
    let line = line;
    let mut writer = Writer::new(
        usize::try_from(line.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    line.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
    let tasks = w::Tasks { ended_mark: 0, rows: pieces(&[writer.finish()]), more: false };
    let arrived = delivery(&mut host, watched.expect("roles watch"), &w::DeliveryValue::Tasks(tasks));
    duties.asked.extend(admit(&mut host, arrived).asked);
    (host, duties)
}
fn activity(position: u64, line: w::Activity) -> w::DecodedActivityLine {
    let source = w::ActivityLine { task: ROLE, position, at: 0, line };
    w::DecodedActivityLine { source, decoded: None }
}
pub(super) fn talk_host(open: bool, cost: bool) -> Host {
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
    let task_watch = watch(&duties, &w::Subject::Task(w::SubjectTask { task: ROLE }));
    let read = read_exchange(&duties, "activity");
    let mut task = records::task_record();
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
    task.talk = open.then_some(w::Talk { party, opened: 0, last: 0 });
    let offers: Vec<_> = [w::ActKind::Talk, w::ActKind::Leave]
        .into_iter()
        .map(|act| {
            let offer = w::Offered { act, admission: w::Admission::Allowed, effect: None };
            let mut writer = Writer::new(
                usize::try_from(offer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
            );
            offer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
            writer.finish()
        })
        .collect();
    task.acts = pieces(&offers);
    if proposed {
        let key = records::pending_proposal_record().key.clone();
        task.proposals = List::from_boxed(Box::new([key]));
    }
    let arrived = delivery(&mut host, task_watch, &w::DeliveryValue::Task(Box::new(task)));
    duties.asked.extend(admit_watches(&mut host, arrived).asked);
    let mut rows = if cost {
        vec![
            activity(1, w::Activity::TalkOpened(w::TalkOpened { party })),
            activity(2, w::Activity::RunEnded(w::ActivityRunEnded { attempt: 44, end: w::RunEnd::Parked, spent: 42 })),
        ]
    } else {
        Vec::new()
    };
    if choices {
        use smith_transcript as s;
        let mut turn = s::decode_archived(&s::CEILINGS, &records::full_turn()).unwrap();
        let text = s::Block::Text(s::Text { text: "Add it to my rules?\n\n1. Yes, add it — it applies from my next run\n2. Just for this port\n3. Tell it something else".as_bytes().into(), replay: None });
        turn.messages = List::from_boxed(Box::new([s::Message {
            role: s::Role::Assistant,
            blocks: List::from_boxed(Box::new([text])),
        }]));
        let turn = turn;
        let mut writer = Writer::new(
            usize::try_from(turn.measure(&smith_transcript::CEILINGS).expect("bounded transcript")).unwrap(),
        );
        turn.write(&smith_transcript::CEILINGS, &mut writer).unwrap();
        rows.push(records::turn_line(3, 0, 1, &writer.finish()));
    }
    if proposed {
        let query = read_exchange(&duties, "proposal");
        let mut proposal = records::pending_proposal_record();
        proposal.project = 3;
        proposal.action = w::ProposalAction::Settings(Box::new(w::ProposalActionSettings {
            against: 0,
            items: List::with_capacity(0),
            review: None,
            made: None,
        }));
        let offer = w::Offered { act: w::ActKind::Accept, admission: w::Admission::Allowed, effect: None };
        let mut writer = Writer::new(
            usize::try_from(offer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
        );
        offer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
        proposal.acts = pieces(&[writer.finish()]);
        duties.asked.extend(read_reply(&mut host, query, &w::ReadValue::Proposal(proposal)).asked);
    }
    let mut page = records::activity_rows(&rows);
    page.after = None;
    page.cut = false;
    duties.asked.extend(read_reply(&mut host, read, &w::ReadValue::Activity(page)).asked);
    duties.asked.retain(
        |request| matches!(request, Request::Exchange { exchange, message: Message::Read(_), .. } if *exchange != read.exchange),
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
        assert_eq!(host.words(), []);
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
fn a_talk_restores_unsent_words_when_chosen_again() {
    let mut host = talk_host(false, false);
    no_box_writes(&give(&mut host, Event::Typed { value: b"x".as_slice().into() }));
    let duties = give(&mut host, Event::Leave);
    no_box_writes(&duties);
    admit(&mut host, duties);
    assert_eq!(host.root.pages().address(), &Address::Home { project: 3 });
    assert_eq!(host.words(), b"");
    no_box_writes(&give(&mut host, Event::Own { command: Own::Talk }));
    host.frame();
    assert_eq!(
        host.event_of(&Key::Enter),
        Some(Event::Act { act: Act::Go { address: Address::Talk { project: 3, task: ROLE } } })
    );
    no_box_writes(&host.report(Report::Key(Key::Enter)));
    assert_eq!(host.root.pages().address(), &Address::Talk { project: 3, task: ROLE });
    assert_eq!(host.words(), b"x");
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
            host.frame();
            assert_eq!(host.event_of(&Key::Enter), Some(Event::Send));
            host.report(Report::Key(Key::Enter))
        } else {
            assert_eq!(host.event_of(&Key::Escape), None);
            empty(&host.report(Report::Key(Key::Escape)));
            assert_eq!(host.root.pages().address(), &Address::Talk { project: 3, task: ROLE });
            assert_eq!(host.event_of(&Key::Left), Some(Event::Leave));
            host.report(Report::Key(Key::Left))
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
fn escape_sets_aside_a_talks_proposal_without_leaving() {
    let mut host = talk_fixture(false, false, false, true);
    let object = jig_client::contract::ObjectKey::Proposal(jig_client::contract::ProposalKey {
        proposer: jig_client::contract::Proposer::Party(11),
        number: 13,
    });
    assert!(has_option(&host, b"1. Accept it"));
    assert_eq!(host.event_of(&Key::Escape), Some(Event::Aside { object }));
    host.report(Report::Key(Key::Escape));
    assert!(host.state().proposal_aside(&object));
    assert_eq!(host.root.pages().address(), &Address::Talk { project: 3, task: ROLE });
    assert!(!has_option(&host, b"1. Accept it"));
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
    assert!(sent_ask(&duties).iter().any(|(_, ask)| matches!(ask, w::Ask::Talk(talk) if talk.project == 3 && talk.task == ROLE && talk.words.as_ref() == b"Yes, add it")));
    assert_eq!(host.words(), []);
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
    assert!(matches!( &decide.on, w::DecideOn::Proposal(proposal) if proposal.running == Some(w::Running::Moves)));
    let final_answer = w::Final::Refused(w::Refused { refusal: w::Refusal::NotOffered, at: None });
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
    let watched = all_openings(&duties)
        .into_iter()
        .find_map(|open| {
            if open.subject.owner == w::Owner::Jig {
                return None;
            }
            let envelope =
                w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(open.subject.record.as_ref())).unwrap();
            let reading = wire::decode_watching(envelope.record.as_ref(), &f::CEILINGS).unwrap();
            matches!(&reading.watch, f::Watch::Findings(subject) if subject.project == 3).then_some(open.watch)
        })
        .expect("the talk's project reads findings");
    let task = f::Holder::Task(f::HolderTask { number: ROLE });
    let document = f::Findings {
        project: 3,
        rows: List::with_capacity(0),
        areas: List::from_boxed(Box::new([f::AreaRow {
            name: b"ports".as_slice().into(),
            words: b"Porting".as_slice().into(),
            owner: f::AreaOwner::Task(f::AreaTask { number: ROLE }),
            holder: Some(task.clone()),
        }])),
        triager: None,
        holder: task,
        cut: false,
    };
    let document = w::Owned {
        owner: w::Owner::Family(w::Family { number: wire::FORGE }),
        version: wire::CLIENT_VERSION,
        record: wire::findings(&document),
    };
    let event = w::StreamEvent::Delivery(w::Delivery { watch: watched, document });
    let duties = host.report(Report::Heard(jig_client::Event::Streamed {
        message: jig_client::Streamed::Record(stream_bytes(&event)),
    }));
    admit(&mut host, duties);
    let (_, file) = super::tell::send(&mut host, b"/tell The port is stuck").expect("one filing");
    assert_eq!(file.area.as_deref(), Some(b"ports".as_slice()));
    assert_eq!(file.body.as_ref(), b"The port is stuck\n\nPage: /projects/3/talk/7 (Porter)");
}
