use smith_domain_session::{llm, record};
use smith_session_world::{bytes, recorded::scenario};

#[test]
fn a_domain_turn_round_trips_its_account_and_backend() {
    let first = scenario(30);
    let affinity = llm::Affinity { key: [0x42; 16], thread: 7 };
    for (kind, sequence) in [
        (
            record::Kind::First(Box::new(record::Opener {
                system: Some(bytes(b"selected instructions\n\n## Brief\nkept bytes")),
                affinity,
                account: Some(7),
                backend: record::Backend::Program { name: bytes(b"claude_code") },
            })),
            1,
        ),
        (
            record::Kind::Window(Box::new(record::Opener {
                system: Some(bytes(b"selected instructions\n\n## Brief\nkept bytes")),
                affinity,
                account: Some(7),
                backend: record::Backend::Program { name: bytes(b"claude_code") },
            })),
            3,
        ),
    ] {
        let mut turn = first.turns[0].clone();
        turn.kind = kind;
        turn.sequence = sequence;
        let decoded = round_trip(&turn);
        assert_eq!(decoded, turn, "sequence {sequence}");
    }
}

fn round_trip(turn: &record::Turn) -> record::Turn {
    let mut configured = skein_lib::List::with_capacity(1);
    configured
        .push(smith_protocol_translate::Endpoint {
            name: bytes(b"provider"),
            number: turn.endpoint.expect("named endpoint").0,
            dialect: turn.dialect,
        })
        .expect("endpoint");
    let endpoints = smith_protocol_translate::Endpoints::new(configured);
    let bytes =
        smith_protocol_translate::encode_turn(turn, &smith_transcript::CEILINGS, &endpoints).expect("encode turn");
    smith_protocol_translate::decode_turn(&bytes, turn.sequence, &smith_transcript::CEILINGS, &endpoints)
        .expect("decode turn")
}
