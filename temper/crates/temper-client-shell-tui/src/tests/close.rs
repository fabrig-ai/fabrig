//! Closing a chat through the shipped host (client/tui/pages/chat.md, 1).
use super::*;
use temper_client_tui::Notice;

const CHAT: u64 = 12;
const PROJECT: u32 = 3;

fn task(title: &[u8], ended: bool) -> w::Task {
    let chip =
        if ended { records::owner_chip_record(w::ChipPhase::Ended, w::Phase::Ended) } else { chat_chip(CHAT, title) };
    let mut reader = Reader::new(chip.rows());
    let mut rows = Vec::new();
    while let Some(bytes) = skein_codec::piece::split(&mut reader).unwrap() {
        let mut fact = w::Fact::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
        if let w::Fact::Chip(row) = fact {
            let mut row = row.into_parts();
            let mut fields = row.value.into_parts();
            fields.task = CHAT;
            fields.title = records::wire_text(title);
            fields.executor = w::ChipExecutor::Agent(
                w::ChipExecutorAgent::new(&w::CEILINGS, w::ChipExecutorAgentParts { charter: 5 }).unwrap(),
            );
            if ended {
                fields.phase = w::Phase::Ended;
            }
            row.value = w::ChipFields::new(&w::CEILINGS, fields).unwrap();
            fact = w::Fact::Chip(w::ChipRow::new(&w::CEILINGS, row).unwrap());
        } else if let w::Fact::ChipDetails(row) = fact {
            let mut row = row.into_parts();
            let mut fields = row.value.into_parts();
            fields.live_delegates = 0;
            if ended {
                fields.ended_at = Some(0);
            }
            row.value = w::ChipDetailsFields::new(&w::CEILINGS, fields).unwrap();
            fact = w::Fact::ChipDetails(Box::new(w::ChipDetailsRow::new(&w::CEILINGS, row).unwrap()));
        }
        rows.push(fact);
    }
    let mut task = records::task_record().into_parts();
    task.project = PROJECT;
    task.chip = records::chip_from_rows(&rows);
    task.activity = 0;
    task.result = None;
    task.delegates = Box::default();
    task.waits_on = w::WaitsOn::News;
    let mut offers = Vec::new();
    for act in [w::ActKind::Cancel, w::ActKind::Message] {
        let offer =
            w::Offered::new(&w::CEILINGS, w::OfferedParts { act, admission: w::Admission::Allowed, effect: None })
                .unwrap();
        let mut writer = Writer::new(usize::try_from(offer.measure()).unwrap());
        offer.encode(&mut writer).unwrap();
        offers.push(writer.finish());
    }
    task.acts = pieces(&offers);
    w::Task::new(&w::CEILINGS, task).unwrap()
}

fn opened(title: &[u8]) -> (Host, u64) {
    let (mut host, me_watch, _) = signed_host(true);
    let mut me = launchpad_me(true).into_parts();
    me.retention = Duration::from_secs(1);
    let mut reader = Reader::new(&me.projects);
    let mut projects = Vec::new();
    while let Some(bytes) = skein_codec::piece::split(&mut reader).unwrap() {
        let mut project = w::MeProject::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts();
        if project.project == PROJECT {
            project.name = records::wire_text(b"Hello");
        }
        projects.push(project_bytes(&w::MeProject::new(&w::CEILINGS, project).unwrap()));
    }
    me.projects = pieces(&projects);
    let duties = delivery(&mut host, me_watch, &w::DeliveryValue::Me(w::Me::new(&w::CEILINGS, me).unwrap()));
    admit(&mut host, duties);
    let duties = give(&mut host, Event::Act { act: Act::Go { address: Address::Chat { task: CHAT } } });
    let duties = admit_watches(&mut host, duties);
    let watched = watch(
        &duties,
        &w::Subject::Task(w::SubjectTask::new(&w::CEILINGS, w::SubjectTaskParts { task: CHAT }).unwrap()),
    );
    let duties = delivery(&mut host, watched, &w::DeliveryValue::Task(Box::new(task(title, false))));
    admit(&mut host, duties);
    (host, watched)
}

fn close(host: &mut Host) -> skein_lib::Token {
    typed(host, b"/close");
    give(host, Event::Send);
    assert!(text(host).contains("Keep it open"));
    assert_eq!(host.event_of(&Key::Enter), Some(Event::Act { act: Act::Confirm }));
    let duties = host.report(Report::Key(Key::Enter));
    let mut duties = stored(host, duties);
    if !sent_ask(&duties).iter().any(|(_, ask)| matches!(ask, w::Ask::Compound(_))) {
        host.clocks(host.env.now.saturating_add(host.env.limits.client.domain.requests.first), host.env.wall);
        duties.asked.extend(host.report(Report::Due).asked);
    }
    let asks = sent_ask(&duties);
    let (exchange, ask) = asks.iter().find(|(_, ask)| matches!(ask, w::Ask::Compound(_))).unwrap_or_else(|| {
        panic!(
            "close sent: asks={asks:?}, confirmation={:?}, pending={:?}, link={:?}",
            host.root.pages().confirmation(),
            host.root.pages().chat().unwrap().pending,
            host.root.client().link().state
        )
    });
    let w::Ask::Compound(compound) = ask else { unreachable!() };
    assert_eq!(compound.project(), PROJECT);
    assert_eq!(host.root.pages().address(), &Address::Chat { task: CHAT });
    assert!(host.state().notice().is_none());
    *exchange
}

fn done(host: &mut Host, exchange: skein_lib::Token) {
    let outcome = w::DocumentOutcome::Compound(
        w::OutcomeCompound::new(
            &w::CEILINGS,
            w::OutcomeCompoundParts { project: PROJECT, operations: 1, first_message: 0 },
        )
        .unwrap(),
    );
    let outcome = w::Outcome::Document(
        w::OutcomeDocument::new(&w::CEILINGS, w::OutcomeDocumentParts { value: outcome }).unwrap(),
    );
    reply(host, exchange, final_bytes(&w::Final::Done(w::Done::new(&w::CEILINGS, w::DoneParts { outcome }).unwrap())));
}

#[test]
fn a_done_close_goes_home_with_the_chats_notice_for_six_seconds() {
    for (title, notice) in [(b"Away mode".as_slice(), "Closed T12 · Away mode"), (b"", "Closed T12")] {
        let (mut host, _) = opened(title);
        let exchange = close(&mut host);
        let closing = host.state().closing().unwrap();
        assert_eq!((closing.task, closing.project), (CHAT, PROJECT));
        assert_eq!(closing.title.as_deref(), (!title.is_empty()).then_some(title));
        let focus = host.state().focus();
        done(&mut host, exchange);
        assert_eq!(host.root.pages().address(), &Address::Home { project: PROJECT });
        assert_eq!(host.state().focus(), focus + 1, "Went also reaches the TUI");
        assert!(host.state().closing().is_none());
        assert!(host.state().closed_chat(CHAT));
        assert_eq!(
            host.state().notice(),
            Some(&Notice::Closed { task: CHAT, title: (!title.is_empty()).then(|| title.into()) })
        );
        let printed = text(&mut host);
        assert!(printed.contains(notice), "{printed}");
        assert_eq!(printed.matches(notice).count(), 1);
        let end = host.env.now.saturating_add(temper_client_tui::limits::NOTICE);
        assert_eq!(host.notice_end, Some(end));
        host.clocks(Time::from_nanos(end.as_nanos() - 1), host.env.wall);
        host.report(Report::Due);
        assert!(host.state().notice().is_some());
        host.clocks(end, host.env.wall);
        host.report(Report::Due);
        assert!(host.state().notice().is_none());
    }
}

#[test]
fn an_unsuccessful_close_stays_and_a_later_close_can_go_home() {
    for lost in [false, true] {
        let (mut host, _) = opened(b"Away mode");
        let exchange = close(&mut host);
        if lost {
            let duties = host.report(Report::Heard(jig_client::Event::Lost { exchange }));
            stored(&mut host, duties);
            host.clocks(host.env.now.saturating_add(Duration::from_secs(2)), host.env.wall);
            let duties = host.report(Report::Due);
            stored(&mut host, duties);
        } else {
            let answer = w::Final::Refused(
                w::Refused::new(&w::CEILINGS, w::RefusedParts { refusal: w::Refusal::NotOffered, at: None }).unwrap(),
            );
            let duties = reply(&mut host, exchange, final_bytes(&answer));
            stored(&mut host, duties);
        }
        assert_eq!(host.root.pages().address(), &Address::Chat { task: CHAT });
        assert!(host.state().closing().is_none(), "answered close forgotten, lost={lost}");
        assert!(host.state().notice().is_none());
        assert!(!host.state().closed_chat(CHAT));
        if lost {
            assert_eq!(host.root.pages().confirmation().unwrap().failed, Some(jig_client::Outcome::Unknown));
        }
        let exchange = close(&mut host);
        done(&mut host, exchange);
        assert_eq!(host.root.pages().address(), &Address::Home { project: PROJECT });
    }
}

#[test]
fn leaving_before_the_answer_does_not_move_or_notice() {
    for new in [false, true] {
        let (mut host, _) = opened(b"Away mode");
        let exchange = close(&mut host);
        assert!(host.state().closing().is_some());
        if new {
            typed(&mut host, b"/new");
            give(&mut host, Event::Send);
        } else {
            give(&mut host, Event::Act { act: Act::Go { address: Address::Chat { task: 8 } } });
        }
        let address = host.root.pages().address().clone();
        let focus = host.state().focus();
        assert!(host.state().closing().is_none());
        done(&mut host, exchange);
        assert_eq!(host.root.pages().address(), &address);
        assert_eq!(host.state().focus(), focus);
        assert!(host.state().notice().is_none());
        assert!(!host.state().closed_chat(CHAT));
    }
}

#[test]
fn cancelling_starting_over_and_an_external_end_are_not_a_confirmed_close() {
    let (mut host, _) = opened(b"Away mode");
    typed(&mut host, b"/close");
    give(&mut host, Event::Send);
    give(&mut host, Event::Act { act: Act::Cancel });
    assert_eq!(host.root.pages().address(), &Address::Chat { task: CHAT });
    assert!(host.state().closing().is_none());
    assert!(host.state().notice().is_none());
    give(&mut host, Event::Act { act: Act::StartOver });
    assert!(host.root.pages().chat().unwrap().restarting);
    let duties = give(&mut host, Event::Act { act: Act::Confirm });
    let exchange = sent_ask(&duties).iter().find(|(_, ask)| matches!(ask, w::Ask::Compound(_))).unwrap().0;
    assert!(host.state().closing().is_none());
    done(&mut host, exchange);
    assert!(host.state().notice().is_none());
    assert!(!host.state().closed_chat(CHAT));
    let (mut host, task_watch) = opened(b"Away mode");
    // A task document ending without a confirmation leaves the chat to read.
    let duties = delivery(&mut host, task_watch, &w::DeliveryValue::Task(Box::new(task(b"Away mode", true))));
    admit(&mut host, duties);
    assert_eq!(host.root.pages().address(), &Address::Chat { task: CHAT });
    assert!(host.state().closing().is_none());
    assert!(host.state().notice().is_none());
    let frame = text(&mut host);
    assert!(
        frame.contains("/new starts another chat"),
        "{frame}; phase={:?}",
        host.root.pages().chat().unwrap().task(host.root.client()).unwrap().chip.details.phase
    );
    typed(&mut host, b"More words");
    give(&mut host, Event::Send);
    assert!(text(&mut host).contains("This chat is closed. /new starts another."));
}
