//! Union inputs preserve each read and keep former contracts readable.
use super::*;
use crate::Decoded;

#[test]
fn the_three_forge_tables_decode_every_read_and_keep_former_names() {
    let rows = [
        ("read_forge", "branch", r#""branch":"main""#),
        ("read_forge", "branches", r#""prefix":"temper/""#),
        (
            "read_forge",
            "compare",
            r#""before":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","after":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb""#,
        ),
        (
            "read_forge",
            "file",
            r#""commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","path":"src/lib.rs","max_bytes":99"#,
        ),
        ("read_forge", "protection", r#""branch":"main""#),
        ("read_forge", "settings", ""),
        ("read_forge", "collaborators", r#""page":3"#),
        ("read_forge", "permission", r#""user":3"#),
        ("read_issues", "items", r#""since":12,"page":3,"item_kind":"pull""#),
        ("read_issues", "item", r#""number":7,"comment":12"#),
        ("read_issues", "pull", r#""number":7"#),
        ("read_issues", "pull_for", r#""branch":"temper/x","base":"main""#),
        ("read_issues", "reviews", r#""number":7,"page":3,"skip":2"#),
        ("read_issues", "remarks", r#""number":7,"review":12,"page":3,"skip":2"#),
        (
            "read_issues",
            "pull_files",
            r#""number":7,"commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","page":3,"skip":2"#,
        ),
        ("read_ci", "checks", r#""commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa""#),
        (
            "read_ci",
            "job",
            r#""commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","run":2,"job":0,"attempt":0,"max_bytes":99"#,
        ),
        ("read_ci", "statuses", r#""commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","page":3"#),
    ];
    let declarations = tools(Duration::from_secs(1));
    let name = run::CallName { activation: 7, completion: 2, position: 1 };
    for (tool, what, members) in rows {
        let comma = if members.is_empty() { "" } else { "," };
        let input = format!(r#"{{"repository":"qa/pilot","what":"{what}"{comma}{members}}}"#);
        let current = decided(tool.as_bytes(), input.as_bytes()).unwrap();
        let former = format!("read_{what}");
        let members = match what {
            "file" | "pull_files" | "job" => members.replace("\"commit\"", "\"head\""),
            "pull_for" => members.replace("\"branch\"", "\"head\""),
            "item" => members.replace("\"comment\"", "\"after\""),
            _ => members.into(),
        };
        let old = format!(r#"{{"repository":"qa/pilot"{comma}{members}}}"#);
        for declaration in &declarations {
            assert_ne!(declaration.name.as_ref(), former.as_bytes());
        }
        let engine::Tool::ReadNamed { read, .. } = &current.tool else { panic!("read") };
        for error in [
            skein_forge::api::Error::Empty,
            skein_forge::api::Error::Missing,
            skein_forge::api::Error::TooLarge(skein_forge::api::Excess::unnamed()),
        ] {
            let answer = engine::CallAnswer::ForgeRead(Box::new(Err(error)));
            assert_eq!(
                crate::answer_for_read(&answer, read, 4096, crate::ForgeReadBounds::STOCK),
                crate::answer_for_tool(&answer, former.as_bytes(), 4096, crate::ForgeReadBounds::STOCK)
            );
        }
        for version in 1..=4 {
            let input = run::HostInput::attested(old.as_bytes().into()).unwrap();
            let Decoded::Call(kept) =
                crate::call_for_contract(version, name, former.as_bytes(), &input, &[], &[], &[]).unwrap()
            else {
                panic!("kept read")
            };
            assert_eq!(kept, current, "{tool} what={what}, contract {version}");
        }
    }
}

#[test]
fn forge_whats_refuse_missing_members_at_decode_and_ignore_other_reads_members() {
    for (tool, input, expected) in [
        ("read_ci", r#"{"what":"job","commit":"bad"}"#, "read_ci job needs commit, run, job and attempt"),
        ("read_issues", r#"{"what":"pull_for","base":"main"}"#, "read_issues pull_for needs branch and base"),
        ("read_forge", r#"{"what":"file","path":"x"}"#, "read_forge file needs commit and path"),
        ("read_ci", r#"{"what":"unknown"}"#, "read_ci what must be checks, job, statuses"),
        (
            "read_forge",
            r#"{"what":"unknown"}"#,
            "read_forge what must be branch, branches, compare, file, protection, settings, collaborators, permission",
        ),
        (
            "read_issues",
            r#"{"what":"unknown"}"#,
            "read_issues what must be items, item, pull, pull_for, reviews, remarks, pull_files",
        ),
    ] {
        let Decoded::Answered(answer) = decoded(tool.as_bytes(), input.as_bytes()).unwrap() else {
            panic!("answered without a decision")
        };
        assert!(answer.error());
        assert_eq!(answer.text(), expected.as_bytes());
    }
    assert_eq!(
        decided(b"read_issues", br#"{"what":"item","number":7,"comment":12,"after":99,"commit":false,"kind":42}"#)
            .unwrap(),
        decided(b"read_item", br#"{"number":7,"after":12}"#).unwrap(),
    );
}

#[test]
fn the_three_forge_declarations_fit_their_byte_share_and_type_the_union() {
    let declarations = tools(Duration::from_secs(1));
    let mut total = 0;
    for name in [b"read_forge".as_slice(), b"read_issues", b"read_ci"] {
        let mut found = None;
        for tool in &declarations {
            if tool.name.as_ref() == name {
                found = Some(tool);
            }
        }
        let tool = found.unwrap();
        let bytes = tool.name.len() + tool.description.len() + tool.schema.len();
        assert!(bytes <= 1024, "{name:?}: {bytes}");
        total += bytes;
        let schema = crate::json::parse(&tool.schema).unwrap();
        let properties = schema.required(b"properties").unwrap();
        assert_eq!(
            schema.required(b"required").unwrap(),
            &crate::json::Value::Array(Box::new([crate::json::Value::String(b"what".as_slice().into())]))
        );
        for row in crate::forge_reads::table(name) {
            assert!(
                properties
                    .required(b"what")
                    .unwrap()
                    .required(b"enum")
                    .unwrap()
                    .array(b"enum")
                    .unwrap()
                    .contains(&crate::json::Value::String(row.what.into()))
            );
            for member in row.required.iter().chain(row.optional) {
                assert!(properties.required(member).unwrap().field(b"type").unwrap().is_some());
            }
        }
    }
    assert!(total <= 2688, "three forge declarations: {total}");
}
