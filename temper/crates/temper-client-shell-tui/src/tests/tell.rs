use super::*;
use temper_client_protocol_wire::{self as wire, v1 as f};
use temper_client_tui::Notice;

const WORDS: &[u8] = b"The export button is grey";

fn request(duties: &Duties) -> Option<(skein_lib::Token, f::FileFinding)> {
    for request in &duties.asked {
        let Request::Exchange { exchange, message: Message::Request(bytes), .. } = request else { continue };
        let keyed = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(bytes)).unwrap();
        if keyed.owner() == &w::Owner::Jig {
            continue;
        }
        let envelope = w::FamilyRequest::decode(&w::CEILINGS, &mut Reader::new(keyed.request())).unwrap();
        let reading = wire::decode_requesting(envelope.record(), &f::CEILINGS).unwrap();
        if let f::Request::FileFinding(file) = reading.request() {
            return Some((*exchange, file.clone()));
        }
    }
    None
}
fn answer(host: &mut Host, exchange: skein_lib::Token, outcome: f::Outcome) {
    let answer = f::Answer::new(&f::CEILINGS, f::AnswerParts { outcome }).unwrap();
    let document = w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts {
            owner: w::Owner::Family(w::Family::new(&w::CEILINGS, w::FamilyParts { number: wire::FORGE }).unwrap()),
            version: wire::CLIENT_VERSION,
            record: wire::answer(&answer),
        },
    )
    .unwrap();
    reply(host, exchange, keyed_bytes(document));
}
fn send(host: &mut Host, words: &[u8]) -> Option<(skein_lib::Token, f::FileFinding)> {
    typed(host, words);
    let duties = give(host, Event::Send);
    let mut duties = admit(host, duties);
    for _ in 0..3 {
        if let Some(request) = request(&duties) {
            return Some(request);
        }
        if let Some(due) = root::next_deadline(&host.root) {
            host.clocks(due, Wall::EPOCH);
            let more = host.report(Report::Due);
            duties.asked.extend(admit(host, more).asked);
        }
    }
    None
}
#[test]
fn tell_files_in_the_screens_project_and_says_each_outcome_once() {
    for chat in [false, true] {
        let mut host = home_host(false);
        if chat {
            person_chat(&mut host);
        }
        for pressed in [false, true] {
            if pressed {
                give(&mut host, Event::Own { command: Own::Tell });
            } else {
                typed(&mut host, b"/tell");
                give(&mut host, Event::Send);
            }
            assert_eq!(host.words(), b"/tell ", "bare or pressed, chat={chat}");
        }
        if chat {
            typed(&mut host, b"/tell The export button is grey");
            give(&mut host, Event::Send);
            let tell = host.root.pages().tell().expect("chat sends the Tell act");
            assert_eq!(tell.project, 3);
            assert_eq!(tell.words.as_ref(), WORDS);
            assert_eq!(host.words(), []);
            continue;
        }
        let sent = send(&mut host, b"/tell The export button is grey");
        assert!(
            sent.is_some(),
            "chat={chat}, tell={:?}, page={:?}, words={:?}",
            host.root.pages().tell(),
            host.root.pages().address(),
            host.words()
        );
        let (exchange, file) = sent.unwrap();
        assert_eq!(file.project(), 3);
        assert_eq!(file.title(), WORDS);
        assert_eq!(file.body(), WORDS);
        assert_eq!(host.words(), []);
        assert_eq!(host.root.pages().tell().unwrap().words.as_ref(), WORDS);
        answer(
            &mut host,
            exchange,
            f::Outcome::Filed(f::Filed::new(&f::CEILINGS, f::FiledParts { number: 12 }).unwrap()),
        );
        assert_eq!(host.state().tell_serial(), 1);
        assert_eq!(host.state().notice(), Some(&Notice::Filed { number: 12, project: b"Project".as_slice().into() }));
        assert!(text(&mut host).contains("Filed as finding 12 on Project."));
        let serial = host.state().tell_serial();
        host.clocks(host.env.now.saturating_add(temper_client_tui::limits::NOTICE), Wall::EPOCH);
        host.report(Report::Due);
        assert_eq!(host.state().tell_serial(), serial);
        assert_eq!(host.state().notice(), None, "an old outcome never renews its notice");
    }
}
#[test]
fn tell_restores_refused_lost_and_long_words_until_send_or_screen() {
    for outcome in ["refused", "lost", "long"] {
        let mut host = home_host(false);
        let words = if outcome == "long" {
            format!("/tell {}", "x".repeat(4097))
        } else {
            "/tell The export button is grey".into()
        };
        let sent = send(&mut host, words.as_bytes());
        let expected = match outcome {
            "refused" => {
                let refusal = f::FindingRefusal::NoFinding;
                let expected = temper_client_view::shared::finding_refusal(host.view.words().view(), &refusal);
                answer(
                    &mut host,
                    sent.unwrap().0,
                    f::Outcome::FindingRefused(
                        f::FindingRefused::new(&f::CEILINGS, f::FindingRefusedParts { reason: refusal }).unwrap(),
                    ),
                );
                expected
            }
            "lost" => {
                answer(&mut host, sent.unwrap().0, f::Outcome::Looked);
                b"temper couldn't file that. Try again.".as_slice().into()
            }
            "long" => {
                assert!(sent.is_none());
                b"Too long to file: 1 bytes past 4,096.".as_slice().into()
            }
            _ => unreachable!(),
        };
        assert_eq!(host.words(), words.as_bytes(), "{outcome}");
        assert_eq!(host.state().tell_refusal(), Some(expected.as_ref()), "{outcome}");
        if outcome != "long" {
            assert!(
                text(&mut host).contains(core::str::from_utf8(&expected).unwrap()),
                "{outcome}: {}",
                text(&mut host)
            );
        }
        let write = host.state().box_written();
        typed(&mut host, b"keep my edit");
        host.report(Report::Resize(Size { columns: 110, rows: 32 }));
        assert_eq!(host.words(), b"keep my edit", "{outcome}: no replay of restoration");
        assert_eq!(host.state().box_written(), write);
        assert_eq!(host.state().tell_refusal(), Some(expected.as_ref()), "typing keeps the refusal");
        give(&mut host, Event::Send);
        assert_eq!(host.state().tell_refusal(), None, "sending clears it");
        host.tell(Told::Tell { serial: 2, refusal: Some(expected) });
        host.went(Address::Home { project: 3 });
        assert_eq!(host.state().tell_refusal(), None, "another screen clears it");
    }
}
