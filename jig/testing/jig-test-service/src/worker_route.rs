//! Smith payloads are translated in the worker layer (jig/docs/design/protocol/channel/README.md, 5.1 and 5.4).
use alloc::boxed::Box;
use jig_domain_host as hub;
use jig_protocol_worker_agents::route;
use jig_test_worker::{Encodable, Encoded};
use skein_lib::Token;
use smith_host_domain as smith;
use smith_host_protocol::Limits;
use smith_protocol_channel::Endpoints;

pub(super) use jig_protocol_worker_agents::route::activity;

pub(super) fn encode(value: Encodable, limits: &Limits, endpoints: &Endpoints) -> Encoded {
    match value {
        Encodable::Turn(turn) => match route::encode_turn(turn, limits, endpoints) {
            Some(turn) => Encoded::Turn(turn),
            None => Encoded::Undecodable,
        },
        Encodable::CallName(name) => match route::encode_call_name(name, limits) {
            Some(name) => Encoded::CallName(name),
            None => Encoded::Undecodable,
        },
        Encodable::Result(result) => match route::encode_result(result, limits) {
            Some(result) => Encoded::Result(result),
            None => Encoded::Undecodable,
        },
    }
}

pub(super) fn decode(
    run: Token,
    source: hub::Start,
    agent: &crate::Agent,
) -> Result<Box<smith::Start>, hub::Undecoded> {
    route::decode(run, source, &agent.limits, &agent.host_limits, &agent.endpoints, route::Evidence::Text)
}

pub(super) fn process(event: smith_host_protocol::ProcessEvent) -> Option<jig_test_worker::Event> {
    match route::process(event)? {
        route::Processed::Live { agent, piece } => Some(jig_test_worker::Event::Live { agent, piece }),
        route::Processed::Process(event) => Some(jig_test_worker::Event::Process(event)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smith_host_protocol::{ProcessEvent, Received};

    const AGENT: Token = Token::new(71);
    const RUN: Token = Token::new(43);
    const TURN: u32 = 7;
    const RETRY: u32 = 3;
    // An offset wider than the core keeps must still cross this hand-out whole.
    const OFFSET: u64 = 1_u64 << 32;

    fn call() -> smith_channel::LiveCall {
        smith_channel::LiveCall::new(
            &smith_channel::CEILINGS,
            smith_channel::LiveCallParts { id: b"provider-call".as_slice().into(), tool: b"tool".as_slice().into() },
        )
        .expect("bounded call")
    }

    fn live(kind: smith_channel::LiveKind, call: Option<smith_channel::LiveCall>) -> ProcessEvent {
        ProcessEvent::Channel {
            agent: AGENT,
            event: Box::new(Received::Live {
                turn: TURN,
                retry: RETRY,
                kind,
                offset: OFFSET,
                bytes: b"piece".as_slice().into(),
                call,
            }),
        }
    }

    #[test]
    fn each_live_kind_keeps_every_field() {
        for (kind, call, expected) in [
            (smith_channel::LiveKind::Text, None, hub::LiveKind::Text),
            (smith_channel::LiveKind::Reasoning, None, hub::LiveKind::Reasoning),
            (
                smith_channel::LiveKind::Input,
                Some(call()),
                hub::LiveKind::Input { call: b"provider-call".as_slice().into(), tool: b"tool".as_slice().into() },
            ),
        ] {
            assert_eq!(
                process(live(kind, call)),
                Some(jig_test_worker::Event::Live {
                    agent: AGENT,
                    piece: hub::Piece {
                        turn: TURN,
                        retry: RETRY,
                        kind: expected,
                        offset: OFFSET,
                        bytes: b"piece".as_slice().into()
                    },
                })
            );
        }
    }

    #[test]
    fn a_live_kind_disagreeing_with_its_call_is_dropped() {
        for (kind, call) in [
            (smith_channel::LiveKind::Text, Some(call())),
            (smith_channel::LiveKind::Reasoning, Some(call())),
            (smith_channel::LiveKind::Input, None),
        ] {
            assert_eq!(process(live(kind, call)), None);
        }
    }

    #[test]
    fn the_start_passes_each_ask_and_omits_an_empty_one() {
        let agent = crate::tests::fixtures::agent();
        for text in [false, true] {
            for reasoning in [false, true] {
                for input in [false, true] {
                    let hub::LinkDown::Assign { assignment } = crate::tests::fixtures::worker_assignment() else {
                        panic!("fixture assignment")
                    };
                    let source = hub::Start {
                        live: hub::LiveAsk { text, reasoning, input },
                        workspace: None,
                        charter: assignment.assignment.charter,
                        activation: 1,
                        identity: assignment.assignment.identity,
                        turns: Box::new([]),
                        answered: Box::new([]),
                        messages: Box::new([]),
                        grants: Box::new([]),
                    };
                    assert_eq!(
                        decode(RUN, source, &agent).expect("decoded fixture").live,
                        (text || reasoning || input).then_some(smith::channel::LiveAsk { text, reasoning, input })
                    );
                }
            }
        }
    }
}
