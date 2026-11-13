//! Current engine and a worker at kept wire versions (temper/docs/design/protocol/wires.md, 11).
use temper_one_process_world::across_versions;

#[test]
fn a_goal_lands_across_wire_versions() {
    across_versions::goal();
}

#[test]
fn a_renewed_identity_reaches_the_worker_at_the_wire_floors() {
    across_versions::renewal();
}

#[test]
fn the_worker_s_opening_states_chosen_versions() {
    use jig_protocol_worker_link::{self as link, Event, Request};
    use skein_channel::{Lower, LowerEvent};
    use skein_lib::{Queue, Reader, Time, Token, stream};
    use temper_worker_service_fixtures as fixtures;
    let settings = fixtures::worker_settings();
    let prepared = temper_worker_service::worker_settings::prepare(
        &settings,
        &fixtures::worker_agent(),
        &fixtures::worker_limits(),
    )
    .expect("worker bounds");
    // The seam accepts chosen versions too, not just the default floors.
    let chosen = [("smith-charter", 5)];
    let wires = across_versions::wires(&chosen);
    let mut credential =
        temper_worker_service::worker_settings::credential(&settings, fixtures::WORKER_SECRET, &prepared.payload)
            .expect("worker credential");
    credential.application = wires.application;
    let mut worker = link::Component::new(&prepared.link, wires.jig, credential, 7).expect("worker link");
    let mut out = Queue::uncapped();
    worker.fire(Time::ZERO, &mut out);
    assert!(matches!(out.pop(), Some(Request::Dial)));
    let connection = Token::new(1);
    worker.down(Time::ZERO, Event::Dialled { connection }, &mut out);
    let mut opening = None;
    while let Some(request) = out.pop() {
        match request {
            Request::Lower { lower: Lower::Write(stream::OutputDown::Room { right, .. }), .. } => worker.down(
                Time::ZERO,
                Event::Lower {
                    connection,
                    event: LowerEvent::Write(stream::OutputUp::Settled {
                        right,
                        outcome: stream::OutputOutcome::Granted,
                    }),
                },
                &mut out,
            ),
            Request::Lower { lower: Lower::Write(stream::OutputDown::Send { bytes, .. }), .. } => {
                let header = skein_channel::parse_header(&bytes[..8]).expect("channel frame");
                let channel_limits = link::channel_limits(&prepared.link).expect("channel limits");
                let skein_channel::Control::Open { credential, lowest, highest, .. } =
                    skein_channel::decode_control(header.kind, &bytes[8..], &channel_limits).expect("opening frame")
                else {
                    panic!("open control")
                };
                assert_eq!((lowest, highest), (11, 11));
                opening = Some(
                    jig_protocol_worker_wire::Credential::decode(
                        &jig_protocol_worker_wire::v12::CEILINGS,
                        &mut Reader::new(&credential),
                    )
                    .expect("worker credential"),
                );
            }
            _ => {}
        }
    }
    let opening = opening.expect("the worker sent an opening");
    let expected = across_versions::wires(&chosen);
    assert_eq!(opening.wires().len(), 5);
    let charter = opening.wires().iter().find(|wire| wire.name() == b"smith-charter").expect("charter stated");
    assert_eq!(charter.lowest(), 5, "the chosen version overrides the floor");
    for (stated, wire) in opening.wires().iter().zip(expected.jig.iter().chain(expected.application.spoken.iter())) {
        assert_eq!(stated.name(), wire.name.as_bytes());
        assert_eq!(stated.lowest(), wire.lowest);
        assert_eq!(stated.layouts().as_slice(), wire.layouts.as_slice());
        assert_eq!(stated.layouts().len(), 1, "one version of every wire");
    }
}
