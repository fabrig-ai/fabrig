use crate::pages::calls::{Touched, first_line, touched};
#[test]
fn tool_shapes_keep_the_argument_the_line_needs() {
    let rows: &[(&[u8], &[u8], Touched)] = &[
        (
            b"read_docs",
            br#"{"topic":"hold","from":1450}"#,
            Touched::Doc { topic: Box::from(b"hold".as_slice()), from: Some(1450) },
        ),
        (b"read_docs", br#"{"search":"a phrase"}"#, Touched::Search { words: Box::from(b"a phrase".as_slice()) }),
        (b"settings", b"{}", Touched::Inventory),
        (
            b"settings",
            br#"{"setting":{"kind":"template","number":2}}"#,
            Touched::Setting { kind: Box::from(b"template".as_slice()), number: Some(2) },
        ),
        (b"settings", br#"{"history":true}"#, Touched::SettingHistory),
        (b"other", br#"{"nested":{},"task":3}"#, Touched::Other { argument: Some(Box::from(b"task 3".as_slice())) }),
        (b"other", br#"{"path":"file"}"#, Touched::Other { argument: Some(Box::from(b"file".as_slice())) }),
        (b"other", br#"{"path":"file\nnext"}"#, Touched::Other { argument: Some(Box::from(b"file\nnext".as_slice())) }),
        (b"read_docs", br#"{"topic":"cut"#, Touched::Other { argument: None }),
    ];
    for (tool, input, expected) in rows {
        assert_eq!(&touched(tool, input), expected);
    }
}
#[test]
fn an_outcome_stops_at_its_first_line() {
    assert_eq!(first_line(b"summary\nbody"), b"summary");
}
#[test]
fn the_current_call_is_the_runs_supplied_snapshot() {
    use super::fixtures;
    use jig_client::{Event, Told, Watchable, contract as c};
    use skein_lib::{Queue, Time, Token, Wall};
    const OUTPUTS: u32 = 64;
    let mut fixture = fixtures::chat::opened();
    let mut emitted = Queue::with_capacity(OUTPUTS);
    while let Some(request) = fixture.out.pop() {
        if let crate::pages::Request::Client(event) = request {
            fixture.client.step_unlinked(Time::ZERO, Wall::EPOCH, event, &mut emitted);
        }
    }
    let mut watch = None;
    while let Some(request) = emitted.pop() {
        if let jig_client::UnlinkedRequest::Down {
            down: jig_client::Down::Open { watch: token, subject: Watchable::Jig(c::Subject::Run { .. }) },
        } = request
        {
            watch = Some(token);
        }
    }
    let snapshot = c::CurrentCall {
        identity: c::CallIdentity { owner: Token::new(1), sequence: 1, retry: 0, id: fixtures::acts::text(b"call") },
        tool: fixtures::acts::text(b"settings"),
        input: fixtures::acts::text(b"{}"),
        cut: false,
        began: Some(Wall::EPOCH),
    };
    for current in [Some(snapshot), None] {
        fixture.client.step_unlinked(
            Time::ZERO,
            Wall::EPOCH,
            Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::RunState(Box::new(c::RunState {
                        task: fixtures::chat::TASK,
                        attempt: Some(fixtures::chat::ATTEMPT),
                        committed_turn: 0,
                        writing: None,
                        call: current.clone(),
                    }))),
                },
            },
            &mut emitted,
        );
        fixture.chat.told(
            Time::ZERO,
            &fixture.client,
            Told::Changed { handle: fixture.chat.run.unwrap() },
            &mut fixture.out,
        );
        assert_eq!(fixture.chat.current_call(), current.as_ref());
    }
}
#[test]
fn touched_text_is_bounded_on_a_character_boundary() {
    // The line admits 256 bytes of argument text (client/README.md, 3.5).
    const LIMIT: usize = 256;
    let prefix = [b'a'; LIMIT];
    for (ascii, expected, tail) in [(254_usize, 256_usize, "é".as_bytes()), (255, 255, b"a".as_slice())] {
        let mut input = skein_lib::Writer::new(ascii.checked_add(13).unwrap());
        input.put(b"{\"path\":\"").unwrap();
        input.put(prefix.get(..ascii).unwrap()).unwrap();
        input.put("é".as_bytes()).unwrap();
        input.put(b"\"}").unwrap();
        let Touched::Other { argument: Some(argument) } = touched(b"other", &input.finish()) else {
            panic!("argument")
        };
        assert_eq!(argument.len(), expected);
        assert_eq!(argument.get(expected.saturating_sub(tail.len())..), Some(tail));
    }
}
