//! The Copy leaves in its press’s turn (temper/docs/design/client/pages/README.md).
use super::*;
use temper_client_domain::copy::Written;

fn welcome() -> Host {
    let mut host =
        Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, temper_client_view::sign_in::Ways::NONE);
    host.root = root::unlinked(&host.env.limits, 11);
    host.client(jig_client::Event::StartUnlinked { kept: List::with_capacity(0) });
    host.event(Event::Start {
        address: Address::Welcome {
            step: root::Step::Model {
                way: Some(root::ModelWay { provider: 17, source: None, kind: root::WayKind::Program }),
            },
        },
        kept: List::with_capacity(0),
        with: Some(jig_client::With::Transport),
    });
    host.client(jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } });
    let watch = host
        .out
        .iter()
        .find_map(|request| {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Me) },
                    },
            } = request
            {
                Some(*watch)
            } else {
                None
            }
        })
        .expect("me watch");
    let link = jig_client_protocol_link::Limits::new(host.env.limits.client.domain, limits::EXCHANGES).unwrap();
    let mut frame =
        jig_client_protocol_link::translate::documents::frame(jig_client_test_records::frame_record(), &link).unwrap();
    frame.providers = List::with_capacity(1);
    let mut programs = List::with_capacity(1);
    programs.push(c::Text::new(b"claude_code".as_slice().into(), 32).unwrap()).unwrap();
    frame
        .providers
        .push(c::ProviderOffer {
            provider: 17,
            name: c::DocumentText::complete(c::Text::new(b"anthropic".as_slice().into(), 32).unwrap()),
            key_bytes: 4096,
            sign_in: false,
            signed_in_plan: false,
            key: false,
            programs,
            programs_cut: false,
            none: false,
            manage_usage: None,
        })
        .unwrap();
    host.client(jig_client::Event::Up {
        up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(frame))) },
    });
    let deployment = host
        .out
        .iter()
        .find_map(|request| {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                    },
            } = request
            {
                let reading = temper_client_protocol_wire::decode_reading(
                    &record.bytes,
                    &temper_client_protocol_wire::v1::CEILINGS,
                )
                .unwrap();
                if reading.read() == &temper_client_protocol_wire::v1::Read::Deployment { Some(*read) } else { None }
            } else {
                None
            }
        })
        .expect("deployment read");
    let answer = temper_client_protocol_wire::v1::DeploymentAnswer::new(
        &temper_client_protocol_wire::v1::CEILINGS,
        temper_client_protocol_wire::v1::DeploymentAnswerParts {
            push: None,
            welcome: true,
            owned: true,
            program_login: None,
        },
    )
    .unwrap();
    host.client(jig_client::Event::Up {
        up: jig_client::Up::ReadDone {
            read: deployment,
            result: jig_client::Readout::Family {
                family: temper_client_protocol_wire::FORGE,
                record: Some(temper_client_protocol_wire::deployment_answer(&answer)),
            },
        },
    });
    host.requests();
    host.patches();
    host
}

#[test]
fn the_press_returns_its_copy_duty_and_reports_rebuild_until_the_deadline() {
    let mut host = welcome();
    let nodes = host.tree().nodes();
    let start = nodes.iter().position(|n| n.role == Role::Code).unwrap();
    let end = start + usize::try_from(nodes[start].size).unwrap();
    let words: Box<[u8]> =
        nodes[start..end].iter().flat_map(|n| n.text.iter().flat_map(|run| run.text.iter().copied())).collect();
    let button = node(&host, Role::Button, b"Copy");
    let duties = host.report(Report::Act { node: button, act: Act::Press });
    assert_eq!(duties.copy, Some((1, words.clone())));
    host.report(Report::Copied { token: 1, written: Written::Done });
    let patches = build(&mut host);
    assert_ne!(patches.len(), 0);
    node(&host, Role::Button, b"Copied");
    let deadline = Time::ZERO.checked_add(root::copy::SHOWN).unwrap();
    assert_eq!(host.deadline(), Some(deadline));
    host.clocks(deadline, Wall::EPOCH);
    host.report(Report::Due);
    assert_ne!(build(&mut host).len(), 0);
    node(&host, Role::Button, b"Copy");
}

#[test]
fn clipboard_records_round_trip_and_redaction_replaces_the_commands_words() {
    use crate::journey::{Line, lines, redact};
    let mut recording = Vec::new();
    Line::Journey { version: 2 }.write(&mut recording);
    for line in [
        Line::Copy { token: 7, words: b"temper secret-word".as_slice().into() },
        Line::Copied { token: 7, written: Written::Done },
        Line::Copied { token: 8, written: Written::Refused },
        Line::Copied { token: 9, written: Written::Failed },
    ] {
        let mut bytes = Vec::new();
        line.write(&mut bytes);
        assert_eq!(Line::read(bytes.strip_suffix(b"\n").unwrap()).unwrap(), line);
        recording.extend(bytes);
    }
    assert_eq!(
        recording,
        b"journey 2\ncopy 7 \"temper secret-word\"\ncopied 7 done\ncopied 8 refused\ncopied 9 failed\n"
    );
    let redacted = redact(&recording, &[b"secret-word".as_slice().into()]).unwrap();
    assert_eq!(redacted.places, 1);
    assert_eq!(
        lines(&redacted.recording).unwrap()[1],
        Line::Copy { token: 7, words: b"temper xxxxxxxxxxx".as_slice().into() }
    );
    assert!(Line::read(b"copied 1 unknown").is_err());
}
