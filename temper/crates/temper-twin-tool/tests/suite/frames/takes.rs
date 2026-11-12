//! Takes on recorded frames, including places where numbers mean different things.
use super::{CHAT, call, completion, first, fixture, save_first, text};
use serde_json::{Value, json};
use temper_twin_tool::{Refusal, Replace, from_document, read_takes, twin};

#[test]
fn takes_replace_number_tokens_decoded_strings_and_text_after_replacements() {
    let (_scratch, mut input) = fixture(&[CHAT]);
    let mut parts = first(&input);
    completion(&mut parts, vec![
        text("old-head and old-head"),
        call(br#"{ "n": 731, "different": [1731,731.0,7.31e2], "s": "x731\u0020old-head" }"#),
    ]);
    save_first(&input, parts);
    input.replaces = Box::new([Replace { old: b"old-head".as_slice().into(), new: b"new-head".as_slice().into() }]);
    input.values = read_takes(br#"[
        {"name":"HEAD","value":"new-head","before":"Head: ","after":"\n"},
        {"name":"TASK","value":"731","before":"task ","after":" created"}
    ]"#).expect("takes");
    let twin = twin(&input).expect("converted takes");
    let doc: Value = serde_json::from_slice(&twin.document).expect("JSON");
    assert_eq!(doc["cued"][0]["turns"][0]["lines"], json!([
        {"take":"HEAD","before":"Head: ","after":"\n"},
        {"take":"TASK","before":"task ","after":" created"},
        {"text":"{{HEAD}} and {{HEAD}}"},
        {"call":"delegate","arguments":{"n":"{{#TASK}}","different":[1731,731.0,731.0],"s":"x{{TASK}} {{HEAD}}"}}
    ]));
    assert_eq!(&*twin.report.taken, &[3, 2]);
    assert_eq!(&*twin.report.replaced, &[3]);
    assert!(doc["cued"][0]["turns"][1]["lines"].as_array().expect("next turn").iter().all(|line| line.get("take").is_none()));
    temper_fake::llm::Document::decode(&twin.document).expect("fake document with takes");
}

#[test]
fn takes_only_touch_their_places_and_overlapping_places_count_once() {
    let (_scratch, mut input) = fixture(&[CHAT]);
    let mut parts = first(&input);
    completion(&mut parts, vec![text("PR #2; depth 2"), call(br#"{"target":2,"depth":2,"a/b":{"~":["PR #2; unchanged 2"]}}"#)]);
    save_first(&input, parts);
    input.values = read_takes(br#"[
        {"name":"TASK","value":"2","before":"task ","after":" created","only":[
            {"call":"delegate","argument":"/target"},{"call":"delegate","argument":"/target"}]},
        {"name":"PULL","value":"2","before":"Pull request: #","after":"\n","only":[
            {"prefix":"PR #","suffix":";"},
            {"call":"delegate","argument":"/a~1b/~0/0","prefix":"PR #","suffix":";"}]}
    ]"#).expect("places");
    let twin = twin(&input).expect("scoped takes");
    let doc: Value = serde_json::from_slice(&twin.document).expect("JSON");
    let lines = &doc["cued"][0]["turns"][0]["lines"];
    assert_eq!(lines[2]["text"], "PR #{{PULL}}; depth 2");
    assert_eq!(lines[3]["arguments"], json!({"target":"{{#TASK}}","depth":2,"a/b":{"~":["PR #{{PULL}}; unchanged 2"]}}));
    assert_eq!(&*twin.report.taken, &[1,2]);
}

#[test]
fn an_unused_take_is_refused_by_name_even_when_its_value_exists_elsewhere() {
    let (_scratch, mut input) = fixture(&[CHAT]);
    input.values = read_takes(br#"[{"name":"MISSING","value":"2","before":"<","after":">","only":[{"call":"not_a_call"}]}]"#).expect("take");
    assert_eq!(twin(&input).map(|_| ()).expect_err("unused"), Refusal::UnusedTake { name: "MISSING".into() });
}

#[test]
fn from_document_preserves_members_and_transforms_all_script_groups() {
    let (_scratch, input) = fixture(&[CHAT]);
    let original = twin(&input).expect("recorded twin");
    let mut doc: Value = serde_json::from_slice(&original.document).expect("JSON");
    doc["asked"] = json!([{"cue":"asked","turns":["old"]}]);
    doc["otherwise"] = json!(["old"]);
    let takes = read_takes(br#"[{"name":"TEXT","value":"new","before":"<","after":">"}]"#).expect("take");
    let converted = from_document(&serde_json::to_vec(&doc).expect("document"), &[Replace {old:b"old".as_slice().into(),new:b"new".as_slice().into()}], &takes).expect("from");
    let after: Value = serde_json::from_slice(&converted.document).expect("JSON");
    assert_eq!(after["cued"], doc["cued"]);
    assert_eq!(after["models"], doc["models"]);
    for turn in [&after["asked"][0]["turns"][0], &after["otherwise"][0]] {
        assert_eq!(turn["lines"], json!([{"take":"TEXT","before":"<","after":">"},{"text":"{{TEXT}}"}]));
    }
    assert_eq!(&*converted.report.taken, &[2]);
    temper_fake::llm::Document::decode(&converted.document).expect("transformed document");
}
