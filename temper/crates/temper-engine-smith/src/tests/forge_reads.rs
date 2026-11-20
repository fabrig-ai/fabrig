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
        (
            "read_ci",
            r#"{"what":"job","commit":"bad"}"#,
            "read_ci: what=job needs the members commit, run, job and attempt.",
        ),
        (
            "read_issues",
            r#"{"what":"pull_for","base":"main"}"#,
            "read_issues: what=pull_for needs the members branch and base.",
        ),
        ("read_forge", r#"{"what":"file"}"#, "read_forge: what=file needs the member path."),
        (
            "read_forge",
            r#"{"what":"file","commit":"bad","branch":"main","path":"x"}"#,
            "read_forge: what=file takes commit or branch, not both.",
        ),
        ("read_ci", "{}", "read_ci: what is one of checks, job or statuses."),
        (
            "read_forge",
            "{}",
            "read_forge: what is one of branch, branches, compare, file, protection, settings, collaborators or permission.",
        ),
        (
            "read_issues",
            "{}",
            "read_issues: what is one of items, item, pull, pull_for, reviews, remarks or pull_files.",
        ),
        ("read_ci", r#"{"what":"unknown"}"#, "read_ci: what is one of checks, job or statuses."),
        (
            "read_forge",
            r#"{"what":"unknown"}"#,
            "read_forge: what is one of branch, branches, compare, file, protection, settings, collaborators or permission.",
        ),
        (
            "read_issues",
            r#"{"what":"unknown"}"#,
            "read_issues: what is one of items, item, pull, pull_for, reviews, remarks or pull_files.",
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
        assert!(properties.required(b"what").unwrap().field(b"type").unwrap().is_none());
        let crate::json::Value::Object(members) = properties else { panic!("properties") };
        for (_, member) in members {
            assert!(member.field(b"minimum").unwrap().is_none(), "bounds stay in the whole form");
        }
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

#[test]
fn forge_default_branches_and_branch_files_decode_under_kept_contracts() {
    use skein_forge::api::Read;
    let rows = [
        (
            r#"{"what":"branch"}"#,
            engine::Tool::ReadNamed { repository: None, read: Read::Branch { branch: Box::from([]) } },
        ),
        (
            r#"{"what":"branch","branch":""}"#,
            engine::Tool::ReadNamed { repository: None, read: Read::Branch { branch: Box::from([]) } },
        ),
        (
            r#"{"what":"branch","branch":"main"}"#,
            engine::Tool::ReadNamed { repository: None, read: Read::Branch { branch: b"main".as_slice().into() } },
        ),
        (
            r#"{"what":"protection"}"#,
            engine::Tool::ReadNamed { repository: None, read: Read::Protection { branch: Box::from([]) } },
        ),
        (
            r#"{"what":"file","branch":"main","path":"README.md"}"#,
            engine::Tool::ReadBranchFile {
                repository: None,
                branch: b"main".as_slice().into(),
                path: b"README.md".as_slice().into(),
                max_bytes: 0,
            },
        ),
        (
            r#"{"what":"file","path":"."}"#,
            engine::Tool::ReadBranchFile { repository: None, branch: Box::from([]), path: Box::from([]), max_bytes: 0 },
        ),
        (
            r#"{"what":"file","branch":"","path":"./","max_bytes":99,"repository":"qa/pilot"}"#,
            engine::Tool::ReadBranchFile {
                repository: Some(b"qa/pilot".as_slice().into()),
                branch: Box::from([]),
                path: Box::from([]),
                max_bytes: 99,
            },
        ),
        (
            r#"{"what":"file","path":"x","repository":""}"#,
            engine::Tool::ReadBranchFile {
                repository: None,
                branch: Box::from([]),
                path: b"x".as_slice().into(),
                max_bytes: 0,
            },
        ),
        (
            r#"{"what":"file","commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","path":"x"}"#,
            engine::Tool::ReadNamed {
                repository: None,
                read: Read::File {
                    head: crate::forge::commit(
                        &crate::json::Value::String(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".as_slice().into()),
                        b"commit",
                    )
                    .unwrap(),
                    path: b"x".as_slice().into(),
                    max_bytes: 0,
                },
            },
        ),
    ];
    let name = run::CallName { activation: 7, completion: 2, position: 1 };
    for (input, expected) in rows {
        let input = run::HostInput::attested(input.as_bytes().into()).unwrap();
        for version in 6..=crate::wire::VERSION {
            let Decoded::Call(call) =
                crate::call_for_contract(version, name, b"read_forge", &input, &[], &[], &[]).unwrap()
            else {
                panic!("read decodes")
            };
            assert_eq!(call.tool, expected, "contract {version}");
        }
    }
    for (input, expected) in [
        (r#"{"what":7}"#, Problem::Type(b"what")),
        (r#"{"what":"branch","branch":7}"#, Problem::Type(b"branch")),
        (r#"{"what":"file","path":"x","max_bytes":0}"#, Problem::Range(b"max_bytes")),
        (r#"{"what":"file","path":"x","max_bytes":4294967296}"#, Problem::Range(b"max_bytes")),
        (r#"{"what":"file","path":"x","max_bytes":false}"#, Problem::Type(b"max_bytes")),
        (r#"{"what":"file","path":false}"#, Problem::Type(b"path")),
        (r#"{"what":"file","path":"x","repository":false}"#, Problem::Type(b"repository")),
    ] {
        assert_eq!(decoded(b"read_forge", input.as_bytes()).unwrap_err(), expected);
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "parse the declaration's UTF-8 Needs clause")]
fn declared_forge_needs_name_each_required_member_once_in_order() {
    for tool in tools(Duration::from_secs(1)) {
        let table = crate::forge_reads::table(&tool.name);
        if table.is_empty() {
            continue;
        }
        let schema = crate::json::parse(&tool.schema).unwrap();
        let description = schema
            .required(b"properties")
            .unwrap()
            .required(b"what")
            .unwrap()
            .required(b"description")
            .unwrap()
            .text(b"description")
            .unwrap();
        let description = core::str::from_utf8(&description).unwrap();
        let clause = description.strip_prefix("Needs: ").unwrap().split_once('.').unwrap().0;
        let mut declared = alloc::collections::BTreeMap::new();
        for entry in clause.split("; ") {
            let boundary = entry.as_bytes().windows(2).position(|pair| pair[1] == b' ' && pair[0] != b',').unwrap() + 1;
            let (whats, members) = entry.split_at(boundary);
            let members: Box<[_]> = members.trim_start().split(", ").map(str::as_bytes).collect();
            for what in whats.split(", ") {
                assert!(declared.insert(what.as_bytes(), members.clone()).is_none(), "each what appears once");
            }
        }
        let required: alloc::collections::BTreeMap<_, _> = table
            .iter()
            .filter(|read| !read.required.is_empty())
            .map(|read| (read.what, Box::<[&[u8]]>::from(read.required)))
            .collect();
        assert_eq!(declared, required);
    }
}
