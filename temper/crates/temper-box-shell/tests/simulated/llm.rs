use crate::{
    OBSERVATIONS, SEED, arguments,
    disk::{Tree, directories},
    document, observations, private_hex,
    wire::{Reply, Running, http},
};
use serde_json::{Value, json};
use std::path::Path;
use temper_box_shell::fake::{Kind, Source};
use temper_fake::credential::{KEY, added_key, credential};

const CHAT: &[u8] = br#"{"model":"qa","store":false,"instructions":"","input":[{"role":"user","content":[{"type":"input_text","text":"say hello"}]}],"stream":true}"#;
const RESPONSES: &str = "/v1/responses";
const KEY_PATH: &str = "/fixture/key";
const DEMO_KEY_PATH: &str = "/fixture/demo/llm.key";
const DEMO_KEY: &[u8] = b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn llm_document() -> Value {
    json!({"listen":{"socket":"127.0.0.1:0"},"key":KEY_PATH,"tls":null,
        "models":[{"name":"qa","window":4096,"output":1024}],"cued":[],
        "asked":[{"cue":"say hello","turns":["hello from QA"]}],"otherwise":["fallback"]})
}
fn bearer(key: &[u8]) -> String {
    format!("Bearer {}", std::str::from_utf8(key).expect("key is text"))
}
fn queries(tree: &Tree) -> Vec<Value> {
    observations(&tree.bytes(OBSERVATIONS.as_ref())).into_iter().filter(|row| row["observation"] == "query").collect()
}

#[test]
fn llm_run() {
    for demo in [false, true] {
        let mut tree = Tree::new();
        let document = document(&mut tree, &llm_document());
        let seeded = credential(SEED, KEY);
        let (source, path, key, hello, fallback) = if demo {
            tree.seed(DEMO_KEY_PATH.as_ref(), &[DEMO_KEY, b"\n"].concat());
            (
                Source::Demo { directories: directories() },
                DEMO_KEY_PATH,
                DEMO_KEY,
                "I am the scripted model",
                "I am the scripted model",
            )
        } else {
            (document, KEY_PATH, seeded.as_slice(), "hello from QA", "fallback")
        };
        let mut run = Running::start(arguments(Kind::Llm, source), &mut tree);
        assert!(run.control.is_none());
        if demo {
            assert_eq!(run.address, "127.0.0.1:8370".parse().expect("stock scripted address"));
        } else {
            private_hex(&tree, path.as_ref(), key);
        }
        let before = tree.snapshot(OBSERVATIONS.as_ref());
        let auth = bearer(key);
        let chat: Value = serde_json::from_slice(CHAT).expect("query");
        let body = serde_json::to_vec(&chat).expect("query");
        for (credential, status, text) in [(&*auth, 200, hello), ("Bearer wrong", 401, "invalid_api_key")] {
            let reply = run.request("POST", RESPONSES, credential, &body);
            reply.status(status);
            assert!(reply.text().contains(text), "{}", reply.text());
        }
        let mut other = chat.clone();
        other["model"] = json!("another-model");
        let reply = run.request("POST", RESPONSES, &auth, &serde_json::to_vec(&other).expect("query"));
        reply.status(200);
        assert!(reply.text().contains(hello), "{}", reply.text());
        let mut other = chat;
        other["input"][0]["content"][0]["text"] = json!("another request");
        let reply = run.request("POST", RESPONSES, &auth, &serde_json::to_vec(&other).expect("query"));
        reply.status(200);
        assert!(reply.text().contains(fallback), "{}", reply.text());
        run.stop(&mut tree);
        assert_eq!(tree.snapshot(OBSERVATIONS.as_ref()), before, "files and modes retained after stop (demo: {demo})");
        assert_eq!(tree.bytes(path.as_ref()), if demo { [key, b"\n"].concat() } else { key.to_vec() });
        assert_eq!(queries(&tree).len(), 3, "the three accepted queries were recorded");
    }
}

#[test]
fn llm_holds_a_piece_until_control_releases_it() {
    let mut tree = Tree::new();
    let mut value = llm_document();
    value["control"] = json!({"socket":"127.0.0.1:0"});
    value["asked"][0]["turns"] = json!([{"lines":[{"pieces":[
        {"text":"first piece","hold":null}, {"text":"second piece","hold":"gate"}
    ]}],"finish":"stop","tokens":6}]);
    let source = document(&mut tree, &value);
    let mut run = Running::start(arguments(Kind::Llm, source), &mut tree);
    let control = run.control.expect("control bound before ready");
    let first = http("POST", RESPONSES, &bearer(&tree.bytes(KEY_PATH.as_ref())), CHAT);
    let mut query = run.open(run.address, &first);
    let mut response = Vec::new();
    for _ in 0..16 {
        let chunk = query.read(&mut run);
        assert!(!chunk.is_empty(), "first delta arrives on the open stream");
        response.extend(chunk);
        if delta(held_text(&response), "first piece") {
            break;
        }
    }
    assert!(response.starts_with(b"HTTP/1.1 200 "));
    assert!(delta(held_text(&response), "first piece"));
    assert_first_only(&response);
    run.request_at(control, "GET", "/", "", b"").status(404);
    // The 404 is a barrier on a second connection. Drain everything which
    // can reach the query without moving time, leaving an outstanding read.
    while let Some(chunk) = query.available(&mut run) {
        assert!(!chunk.is_empty(), "held stream ended before release");
        response.extend(chunk);
        assert!(response.len() <= 16_384, "held reply exceeded its bound before release");
    }
    assert_first_only(&response);
    assert_eq!(queries(&tree).len(), 1, "query recorded while held");
    let release = |run: &mut Running| {
        let reply = run.request_at(control, "POST", "/release/gate", "", b"");
        assert!(reply.0.starts_with(b"HTTP/1.1 204 No Content\r\n"), "{}", reply.text());
    };
    release(&mut run);
    response.extend(query.read_to_end(&mut run));
    let reply = Reply(response);
    let text = reply.text();
    assert!(delta(text, "second piece"), "{text}");
    assert!(text.contains("first piecesecond piece"), "{text}");
    assert_eq!(text.matches("event: response.completed").count(), 1);
    release(&mut run);
    assert!(query.read(&mut run).is_empty(), "stream stays complete");
    query.close(&mut run);
    run.stop(&mut tree);
}
fn delta(text: &str, piece: &str) -> bool {
    text.split("event: response.output_text.delta\n")
        .skip(1)
        .any(|event| event.split_once("\n\n").is_some_and(|(fields, _)| fields.contains(piece)))
}
fn held_text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("ASCII held response")
}
fn assert_first_only(bytes: &[u8]) {
    let text = held_text(bytes);
    assert!(text.contains("first piece"), "{text}");
    assert!(!text.contains("second piece"), "{text}");
    assert!(!text.contains("response.completed"), "{text}");
}

#[test]
fn llm_fails_a_credential_its_line_names_and_accepts_the_added_key() {
    let mut tree = Tree::new();
    let mut value = llm_document();
    let second_path = Path::new("/fixture/second.key");
    value["added_keys"] = json!([second_path]);
    let turn =
        |error| json!({"lines":[{"fail":{"error":error,"credentials":[0]}},{"text":"ok"}],"finish":"stop","tokens":1});
    value["asked"][0]["turns"] = json!([turn(json!("unauthorized"))]);
    value["cued"] = json!([{"cue":"limited","turns":[turn(json!({"exhausted":{"retry_after_ms":1000}}))]}]);
    let source = document(&mut tree, &value);
    let mut run = Running::start(arguments(Kind::Llm, source), &mut tree);
    let primary = credential(SEED, KEY);
    let second = credential(SEED, added_key(0));
    private_hex(&tree, KEY_PATH.as_ref(), &primary);
    private_hex(&tree, second_path, &second);
    assert_ne!(primary, second);
    let refused = run.request("POST", RESPONSES, &bearer(&primary), CHAT);
    refused.status(401);
    assert_eq!(refused.json()["error"]["code"], "invalid_api_key");
    let accepted = run.request("POST", RESPONSES, &bearer(&second), CHAT);
    accepted.status(200);
    assert!(accepted.text().contains("\"text\":\"ok\""));
    let mut limited: Value = serde_json::from_slice(CHAT).expect("query");
    limited["instructions"] = json!("limited");
    let exhausted = run.request("POST", RESPONSES, &bearer(&primary), &serde_json::to_vec(&limited).expect("query"));
    exhausted.status(429);
    assert_eq!(exhausted.json()["error"]["resets_in_seconds"], 1);
    run.stop(&mut tree);
}
