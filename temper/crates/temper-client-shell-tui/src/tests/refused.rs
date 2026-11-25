//! A refusal stops the host at the report that carries it (client/tui/pages/refused.md, 1).
use super::*;
use crate::host::{Need, Phase};
use jig_client::{Event as ClientEvent, Link, Streamed, Up};

fn not_served_hello(family: bool) -> Box<[u8]> {
    let mut served = List::with_capacity(1);
    if family {
        served
            .push(w::Served { owner: w::Owner::Jig, first: w::VERSION, last: w::VERSION })
            .expect("Jig served, application family omitted");
    }
    let hello = w::Hello { stream: [7; 16], served, heartbeat: Duration::from_secs(30) };
    stream_bytes(&if family { records::opened(hello, records::frame_record()) } else { w::StreamEvent::Hello(hello) })
}

#[test]
fn a_reported_build_refusal_draws_and_stops_every_host_send_until_close() {
    for signed_in in [false, true] {
        for family in [false, true] {
            let mut host = host();
            let report = if signed_in {
                start(&mut host, Some(Secret::new(Box::new([7; 40]))));
                let hello = not_served_hello(family);
                ClientEvent::Streamed { message: Streamed::Record(hello) }
            } else {
                // The unlinked seam supplies jig's version report while approval is outstanding.
                // Linked approval answers do not yet carry a version refusal.
                host.root = root::unlinked(&host.env.limits, 7);
                host.step(root::Event::Client { event: ClientEvent::StartUnlinked { kept: List::with_capacity(0) } });
                start(&mut host, None);
                assert_eq!(host.root.client().link().state, Link::SignedOut);
                assert!(host.root.client().link().credential.is_none());
                assert_eq!(host.root.asked(), Some(root::asked::State::Asking));
                assert!(temper_client_tui::sign_in_drawn(&host.root, host.state()));
                ClientEvent::Up {
                    up: if family {
                        Up::FamilyServed { family: temper_client_protocol_wire::FORGE, versions: None, served: false }
                    } else {
                        Up::Served { served: false, versions: None }
                    },
                }
            };
            host.local = true;
            host.need = Some(Need { held: false, reads: 1, digest: None, phase: Phase::Again(Time::ZERO) });
            host.state.as_mut().expect("started TUI").header(false, b"0.0.0 (b948f40d6df1)".as_slice().into());
            empty(&host.report(Report::Heard(report)));
            if family {
                assert_ne!(host.root.client().link().not_served, []);
                assert_ne!(host.root.client().link().state, Link::OutOfDate);
            } else {
                assert_eq!(host.root.client().link().state, Link::OutOfDate);
            }
            let frame = text(&mut host);
            assert!(frame.contains("127.0.0.1:8367 runs another build of temper."));
            assert!(frame.contains("This one is temper 0.0.0 (b948f40d6df1)."));
            assert!(frame.contains("Start the build 127.0.0.1:8367 runs, then try again."));
            assert!(frame.contains("● Refused"));
            assert!(frame.contains("Ctrl+C to quit"));
            let events = host.events.len();
            let shown = root::shown(&host.root);
            host.clocks(Time::from_nanos(60_000_000_000), Wall::from_nanos(60_000_000_000));
            for key in [
                Key::Enter,
                Key::Escape,
                Key::Control(b'p'),
                Key::Char('n'),
                Key::Char('1'),
                Key::Paste(b"/quit".as_slice().into()),
                Key::Tab,
                Key::Up,
                Key::Down,
            ] {
                assert_eq!(host.event_of(&key), None);
                empty(&host.report(Report::Key(key)));
            }
            let mut duties = Duties::default();
            host.event(Event::Open { address: b"http://127.0.0.1:8367".as_slice().into() }, &mut duties);
            host.event(Event::SignIn, &mut duties);
            host.approval(&mut duties);
            empty(&duties);
            for report in [
                Report::Due,
                Report::Heard(ClientEvent::StreamGone),
                Report::Heard(ClientEvent::Streamed { message: Streamed::SignedOut }),
                Report::Local { link: Some(Secret::new(Box::new([1; 32]))) },
                Report::Opened { opened: false },
                Report::Kept { written: false },
                Report::Resize(Size { columns: 100, rows: 30 }),
            ] {
                empty(&host.report(report));
                assert!(text(&mut host).contains("● Refused"));
            }
            assert_eq!(host.events.len(), events, "no root step after the refused report");
            assert_eq!(root::shown(&host.root), shown, "no root fire or take after refusal");
            assert_eq!(host.deadline(), None, "a refusal waits for input, not root deadlines");
            assert_eq!(host.event_of(&Key::Control(b'c')), None);
            let duties = host.report(Report::Key(Key::Control(b'c')));
            assert!(duties.close);
            assert!(host.closed);
            empty(&Duties { close: false, ..duties });
        }
    }
}
