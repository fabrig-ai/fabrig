//! Takes belong to standalone and given scripts, not a deployment demo.
use super::*;

fn taking() -> Value {
    json!({"lines":[{"take":"pull","before":"pull request #","after":" opened"},
        {"call":"read_pull","arguments":{"number":"{{#pull}}"}}],"finish":"tool_calls","tokens":1})
}

#[test]
fn standalone_and_given_documents_read_take_lines() {
    let value = taking();
    let d = crate::read::document(&serde_json::to_vec(&value).unwrap()).unwrap();
    let checked = turn::into_skein(turn::Turn::read_standalone(&d, 0, &Path::root(), Some(1)).unwrap());
    assert_eq!(
        checked.lines.first(),
        Some(&api::Line::Take {
            name: b"pull".as_slice().into(),
            before: b"pull request #".as_slice().into(),
            after: b" opened".as_slice().into(),
        })
    );
    let mut document = llm_document();
    document["otherwise"] = json!([value]);
    let given = json!({"otherwise":[taking()]});
    let bytes = serde_json::to_vec(&document).unwrap();
    for scripts in [None, Some(serde_json::to_vec(&given).unwrap())] {
        let document = llm::Document::decode_with_scripts(&bytes, scripts.as_deref()).unwrap();
        let answer = answer(&document, 23, query(b"", b"pull request #7 opened"));
        let Some(api::Part::ToolCall { name, arguments, .. }) = answer.parts.first() else { panic!("call") };
        assert_eq!(name.as_ref(), b"read_pull");
        assert_eq!(arguments.as_ref(), b"{\"number\":7}");
        assert_eq!(answer.parts.len(), 1);
    }
}

#[test]
fn deployment_demo_refuses_take_lines() {
    let mut document = demo::fixture();
    document["application"]["demo"]["llm"]["otherwise"] = json!([taking()]);
    let error = demo::read(&document).unwrap_err();
    assert!(error.line().starts_with(b"application.demo.llm.otherwise[0].lines[0]."));
    assert!(error.line().ends_with(b": unknown member\n"));
}

#[test]
fn take_document_refusals_name_the_turn() {
    for (take, expected) in [
        (json!({"take":"pull","before":"","after":" opened"}), "otherwise[0].lines[0].before: must be non-empty\n"),
        (
            json!({"take":"pull","before":"pull request #","after":""}),
            "otherwise[0].lines[0].after: must be non-empty\n",
        ),
        (json!({"take":"bad-name","before":"<","after":">"}), "otherwise[0]: invalid turn\n"),
        (json!({"take":"pull","before":"<"}), "otherwise[0].lines[0].after: missing\n"),
        (json!({"take":1,"before":"<","after":">"}), "otherwise[0].lines[0].take: wrong value kind\n"),
        (
            json!({"take":"pull","before":"<","after":">","text":"mixed"}),
            "otherwise[0].lines[0]: expected take, before and after alone\n",
        ),
    ] {
        let mut document = llm_document();
        document["otherwise"] = json!([{"lines":[take],"finish":"stop","tokens":1}]);
        let error = llm::Document::decode(&serde_json::to_vec(&document).unwrap()).unwrap_err();
        assert_eq!(error.line().as_ref(), expected.as_bytes());
    }
}
