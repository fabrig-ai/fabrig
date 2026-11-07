//! One list child, whose only effects are a request and a bounded wire answer (smith/docs/design/protocol/agent.md, 6.1).
use skein_fake_peers::llm::Observation;
use skein_lib::{Reader, Secret, Writer};
use skein_llm::Provider;
use smith_agent_process_world::{fake, list_world};
use smith_channel::{self as wire, CEILINGS};

const ENDPOINT: u32 = 17;
const SECRET: &[u8] = b"list-private-fixture";
const BODY: &[u8] = br#"{"data":[{"id":"model-a","display_name":"Model A","max_input_tokens":200000,"max_tokens":8192,"capabilities":{"effort":{"low":{"supported":true},"high":{"supported":true}}}}]}"#;
// The exchange's response bound is 1 MiB; this passes it by one byte.
const LONG_BODY: usize = 1_048_577;

#[derive(Clone, Copy)]
enum Input {
    Ask,
    Malformed,
    Trailing,
    Empty,
    Long,
}
#[derive(Debug, PartialEq, Eq)]
enum Expected {
    Listed,
    Failed(wire::ListFailed),
    Startup,
}
struct Row {
    name: &'static str,
    provider: Provider,
    scheme: Option<wire::Scheme>,
    endpoint: u32,
    input: Input,
    peer: bool,
    status: Option<u16>,
    body: Option<usize>,
    startup: bool,
    program: bool,
    expected: Expected,
    requests: usize,
}

impl Row {
    fn anthropic(&self) -> bool {
        self.provider == Provider::Anthropic
    }

    fn ordinary(name: &'static str) -> Self {
        Self {
            name,
            provider: Provider::OpenAi,
            scheme: Some(wire::Scheme::Key),
            endpoint: ENDPOINT,
            input: Input::Ask,
            peer: true,
            status: None,
            body: None,
            startup: false,
            program: false,
            expected: Expected::Listed,
            requests: 1,
        }
    }
}

fn configuration(row: &Row) -> Vec<u8> {
    let mut document = serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":256,"read_window":256}},
        "endpoints":[{"name":"local","number":ENDPOINT,"dialect":if row.anthropic() { 2 } else { 3 },
            "provider":if row.anthropic() { "anthropic" } else { "openai" },
            "address":if row.startup { "192.0.2.1:443" } else { "127.0.0.1:443" },
            "transport":"plaintext","connect":10000,"handshake":10000,
            "limits":{"window":65536,"output":512,"reasoning_item":64,"head":60000,"idle":30000}}],
        "environment":[],
        "trace":{"path":"/events.jsonl","capture":"everything","delivery":"complete"}
    });
    if row.program {
        document["programs"] = serde_json::json!({"claude_code":{"path":"/programs/claude","directory":"/programs"}});
        document["endpoints"][0]["program"] = "claude_code".into();
    }
    serde_json::to_vec(&document).expect("configuration")
}

fn ask(row: &Row) -> Vec<u8> {
    let credential = row.scheme.clone().map(|scheme| {
        wire::CredentialValue::new(&CEILINGS, wire::CredentialValueParts { scheme, bytes: Secret::new(SECRET.into()) })
            .expect("fixture credential")
    });
    let value = wire::ListAsk::new(&CEILINGS, wire::ListAskParts { endpoint: row.endpoint, credential }).expect("ask");
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("ask size"));
    value.encode(&mut writer).expect("ask encoding");
    let bytes = writer.finish().into_vec();
    match row.input {
        Input::Ask => bytes,
        Input::Malformed => vec![0xff],
        Input::Trailing => [bytes, vec![0xff]].concat(),
        Input::Empty => vec![],
        Input::Long => vec![
            0;
            usize::try_from(wire::ListAsk::encoded_bound(&CEILINGS).expect("ask bound"))
                .expect("ask fits usize")
                .checked_add(1)
                .expect("one over bound")
        ],
    }
}

fn rows() -> Vec<Row> {
    vec![
        Row::ordinary("openai key"),
        Row { name: "openai bearer", scheme: Some(wire::Scheme::Bearer), ..Row::ordinary("") },
        Row { name: "anthropic key", provider: Provider::Anthropic, ..Row::ordinary("") },
        Row { name: "program endpoint with key", program: true, ..Row::ordinary("") },
        Row {
            name: "refused",
            status: Some(403),
            expected: Expected::Failed(wire::ListFailed::Refused(
                wire::ListRefused::new(&CEILINGS, wire::ListRefusedParts { status: 403 }).expect("status"),
            )),
            ..Row::ordinary("")
        },
        Row {
            name: "peer gone",
            peer: false,
            requests: 0,
            expected: Expected::Failed(wire::ListFailed::Unreachable),
            ..Row::ordinary("")
        },
        Row {
            name: "body too long",
            body: Some(LONG_BODY),
            expected: Expected::Failed(wire::ListFailed::Unreadable),
            ..Row::ordinary("")
        },
        Row {
            name: "unknown endpoint",
            endpoint: ENDPOINT + 1,
            requests: 0,
            expected: Expected::Failed(wire::ListFailed::Unreadable),
            ..Row::ordinary("")
        },
        Row {
            name: "anthropic bearer",
            provider: Provider::Anthropic,
            scheme: Some(wire::Scheme::Bearer),
            requests: 0,
            expected: Expected::Failed(wire::ListFailed::Unreadable),
            ..Row::ordinary("")
        },
        Row {
            name: "malformed ask",
            input: Input::Malformed,
            requests: 0,
            expected: Expected::Failed(wire::ListFailed::Unreadable),
            ..Row::ordinary("")
        },
        Row {
            name: "trailing ask",
            input: Input::Trailing,
            requests: 0,
            expected: Expected::Failed(wire::ListFailed::Unreadable),
            ..Row::ordinary("")
        },
        Row {
            name: "no ask",
            input: Input::Empty,
            requests: 0,
            expected: Expected::Failed(wire::ListFailed::Unreadable),
            ..Row::ordinary("")
        },
        Row {
            name: "oversized ask",
            input: Input::Long,
            requests: 0,
            expected: Expected::Failed(wire::ListFailed::Unreadable),
            ..Row::ordinary("")
        },
        Row {
            name: "startup refusal",
            startup: true,
            input: Input::Empty,
            requests: 0,
            expected: Expected::Startup,
            ..Row::ordinary("")
        },
    ]
}

#[test]
fn a_list_child_answers_once_without_starting_a_run() {
    for row in rows() {
        let run = list_world::run(
            &configuration(&row),
            &ask(&row),
            row.peer.then_some(|| {
                let mut peer = fake::peer_for(row.anthropic());
                let credential = match if row.anthropic() {
                    &wire::Scheme::Key
                } else {
                    row.scheme.as_ref().expect("fixture scheme")
                } {
                    wire::Scheme::Key => skein_llm::Credential::key(SECRET.into()),
                    wire::Scheme::Bearer => skein_llm::Credential::bearer(SECRET.into()),
                };
                peer.accept(credential).expect("fixture credential");
                peer.set_list(Some(row.body.map_or_else(|| BODY.into(), |size| vec![b' '; size].into())), row.status);
                peer
            }),
        );
        assert!(skein_lib::bytes::find(&run.output, SECRET).is_none(), "{} stdout secret", row.name);
        assert!(skein_lib::bytes::find(&run.errors, SECRET).is_none(), "{} stderr secret", row.name);
        let lists: Vec<_> = run
            .observations
            .iter()
            .filter_map(|observation| match observation {
                Observation::List { route, scheme, index, .. } => Some((*route, *scheme, *index)),
                Observation::Query { .. } | Observation::Answered { .. } => panic!("{} started a run", row.name),
                Observation::Accepted { .. }
                | Observation::Head { .. }
                | Observation::Credential { .. }
                | Observation::Closed { .. } => None,
            })
            .collect();
        assert_eq!(lists.len(), row.requests, "{} list count", row.name);
        if row.requests == 0 {
            assert!(run.observations.is_empty(), "{} no connection", row.name);
        }
        assert_eq!((run.opens, run.spawns), (0, 1), "{} no machine or event stream", row.name);
        for (route, scheme, index) in lists {
            assert_eq!(
                route,
                if row.anthropic() { skein_llm::Provider::Anthropic } else { skein_llm::Provider::OpenAi },
                "{} route",
                row.name
            );
            assert_eq!(
                scheme,
                row.scheme.as_ref().map(|scheme| match scheme {
                    wire::Scheme::Key => skein_llm::Scheme::Key,
                    wire::Scheme::Bearer => skein_llm::Scheme::Bearer,
                }),
                "{} scheme",
                row.name
            );
            assert!(index.is_some(), "{} credential accepted", row.name);
        }
        assert_answer(&row, &run);
    }
}

fn assert_answer(row: &Row, run: &list_world::Run) {
    match &row.expected {
        Expected::Startup => {
            assert_eq!(run.exit, skein_io::kernel::Exit::Code(1), "{} exit", row.name);
            assert!(run.output.is_empty(), "{} no answer", row.name);
            assert!(
                String::from_utf8_lossy(&run.errors).contains("plaintext endpoint address must be loopback"),
                "{} diagnostic",
                row.name
            );
        }
        expected => {
            assert_eq!(run.exit, skein_io::kernel::Exit::Code(0), "{} exit", row.name);
            assert!(run.errors.is_empty(), "{} diagnostic: {:?}", row.name, run.errors);
            let answer =
                wire::ListAnswer::decode(&CEILINGS, &mut Reader::new(&run.output)).expect("one complete answer");
            let value = match answer {
                wire::ListAnswer::Listed(rows) => {
                    assert_eq!(rows.rows().len(), 1, "{} row count", row.name);
                    let model = rows.rows().get(0).expect("model row");
                    assert_eq!(model.name(), b"model-a", "{} name", row.name);
                    assert_eq!(
                        model.shown().as_deref(),
                        if row.anthropic() { Some(b"Model A".as_slice()) } else { None },
                        "{} shown",
                        row.name
                    );
                    assert_eq!(*model.window(), row.anthropic().then_some(200_000), "{} window", row.name);
                    assert_eq!(*model.output(), row.anthropic().then_some(8192), "{} output", row.name);
                    let efforts: Vec<_> = model.efforts().iter().map(AsRef::as_ref).collect();
                    assert_eq!(
                        efforts,
                        if row.anthropic() { vec![b"low".as_slice(), b"high".as_slice()] } else { vec![] },
                        "{} efforts",
                        row.name
                    );
                    Expected::Listed
                }
                wire::ListAnswer::Failed(value) => Expected::Failed(value.into_parts().failure),
            };
            assert_eq!(&value, expected, "{} answer", row.name);
        }
    }
}
