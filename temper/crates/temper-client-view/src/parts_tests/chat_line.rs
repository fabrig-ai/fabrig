use super::{fixtures, fixtures::first_run as f};

use crate::{
    look,
    common::chat_line::Run,
};

use jig_client::contract as c;

use skein_lib::{List, Token, Wall};

use skein_ui::{Builder, Role, Tree};

const CALLS: u32 = 16;

fn call(number: u64, tool: &[u8], input: &[u8], outcome: Option<&[u8]>) -> c::Call {
    let c::Block::Call(call) = f::call(number, tool, input, outcome) else { panic!("call") };
    call
}

fn current() -> c::CurrentCall {
    c::CurrentCall {
        identity: c::CallIdentity { owner: Token::new(1), sequence: 1, retry: 0, id: f::text(b"current") },
        tool: f::text(b"settings"),
        input: f::text(br#"{"setting":{"kind":"template","number":2}}"#),
        cut: false,
        began: None,
    }
}

fn line(
    calls: &[c::Call],
    summaries: &[(u64, c::Text)],
    current: Option<&c::CurrentCall>,
    running: bool,
    seconds: u64,
) -> Tree {
    line_with_words(calls, summaries, current, running, seconds, &fixtures::browser_words())
}

fn line_with_words(
    calls: &[c::Call],
    summaries: &[(u64, c::Text)],
    current: Option<&c::CurrentCall>,
    running: bool,
    seconds: u64,
    words: &crate::words::Words,
) -> Tree {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut run = Run::with_capacity(CALLS);
    for (at, call) in calls.iter().enumerate() {
        run.call(
            call,
            Wall::from_nanos(
                u64::try_from(at).unwrap().checked_mul(seconds).unwrap().checked_mul(1_000_000_000).unwrap(),
            ),
            None,
        );
    }
    for (call, summary) in summaries {
        run.result(*call, summary);
    }
    run.flush(&mut b, current, running, words, false);
    b.finish()
}

fn disclosure(tree: &Tree) -> &skein_ui::Node {
    for node in tree.nodes() {
        if node.role == Role::Disclosure {
            return node;
        }
    }
    panic!("run disclosure")
}

#[test]
fn read_docs_counts_topics_not_calls() {
    let tree = line(
        &[
            call(1, b"read_docs", br#"{"topic":"hold"}"#, None),
            call(2, b"read_docs", br#"{"topic":"hold","from":12}"#, None),
            call(3, b"read_docs", br#"{"search":"word"}"#, None),
        ],
        &[],
        None,
        false,
        0,
    );
    assert_eq!(disclosure(&tree).name.as_deref(), Some(b"Read 2 docs".as_slice()));
}

#[test]
fn one_call_alone_says_what_it_touched() {
    for (tool, input, expected) in [
        (b"read_docs".as_slice(), br#"{"topic":"hold"}"#.as_slice(), b"Read the page hold".as_slice()),
        (b"read_docs", br#"{"search":"word"}"#, b"Searched the docs for word"),
        (b"settings", br#"{"setting":{"kind":"template","number":2}}"#, b"Checked template 2"),
    ] {
        assert_eq!(
            disclosure(&line(&[call(1, tool, input, None)], &[], None, false, 0)).name.as_deref(),
            Some(expected)
        );
    }
}

#[test]
fn a_tool_outside_the_table_is_its_name() {
    assert_eq!(
        disclosure(&line(
            &[call(1, b"read_file", b"{}", None), call(2, b"read_file", b"{}", None)],
            &[],
            None,
            false,
            0
        ))
        .name
        .as_deref(),
        Some("read_file ×2".as_bytes())
    );
}

#[test]
fn a_failed_call_is_said_on_the_line_and_its_row() {
    let mut failed = call(1, b"read_file", b"{}", Some(b"denied\nbody"));
    failed.failed = true;
    let tree = line(&[failed.clone()], &[], None, false, 0);
    assert_eq!(disclosure(&tree).description.as_deref(), Some(b"1 failed".as_slice()));
    assert!(disclosure(&tree).styles.as_slice().contains(&look::WORK_FAILED));
    assert!(fixtures::find(&tree, Role::Paragraph, b"read_file: failed: denied").is_some());
    for node in tree.nodes() {
        if node.role == Role::Paragraph {
            assert!(node.styles.as_slice().contains(&look::WORK_FAILED));
        }
    }
    let tree = line(&[failed.clone()], &[], Some(&current()), true, 0);
    assert_eq!(disclosure(&tree).styles.len(), 5);
    failed.outcome = None;
    let tree = line(&[failed], &[], None, false, 0);
    assert!(fixtures::find(&tree, Role::Paragraph, b"read_file: failed: ").is_some());
}

#[test]
fn a_call_under_way_with_none_ended_is_a_line() {
    let tree = line(&[], &[], Some(&current()), true, 0);
    assert_eq!(disclosure(&tree).name.as_deref(), Some(b"Checking template 2".as_slice()));
    assert!(fixtures::find(&tree, Role::Paragraph, b"settings template 2").is_some());
}

#[test]
fn a_table_tools_row_holds_no_results_body() {
    let tree = line(&[call(1, b"read", b"{}", Some(b"summary\nsecret body"))], &[], None, false, 0);
    assert!(fixtures::find(&tree, Role::Paragraph, b"read: summary").is_some());
    assert_eq!(disclosure(&tree).name.as_deref(), Some(b"Read 1 file".as_slice()));
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"secret body").is_none());
}

#[test]
fn the_terminal_prints_the_name_description_and_rows() {
    let tree = line_with_words(
        &[call(1, b"read", b"{}", Some(b"first")), call(2, b"read", b"{}", Some(b"last"))],
        &[],
        None,
        false,
        29,
        &fixtures::words(),
    );
    let mut print = skein_ui_text::Print::new(&fixtures::tree_limits()).unwrap();
    skein_ui_text::render(&tree, b"temper", None, &mut print);
    assert!(skein_lib::bytes::find(print.text.as_slice(), "Read 2 files: 29 s".as_bytes()).is_some());
    assert!(skein_lib::bytes::find(print.text.as_slice(), b"`read`: first").is_some());
    assert!(skein_lib::bytes::find(print.text.as_slice(), b"`read`: last").is_some());
}

#[test]
fn rows_and_names_end_on_character_and_phrase_boundaries() {
    // The contract bounds a row at 160 bytes and a name at 256 (client/pages/README.md).
    let tool = [b'x'; 256];
    let tree = line(&[call(1, &tool, b"{}", Some(b"outcome"))], &[], None, false, 0);
    assert_eq!(disclosure(&tree).name.as_ref().unwrap().len(), 256);
    for node in tree.nodes() {
        if node.role == Role::Paragraph {
            assert_eq!(node.name.as_ref().unwrap().len(), 160);
        }
    }
    let tree = line(&[call(1, &tool, b"{}", None), call(2, b"another", b"{}", None)], &[], None, false, 0);
    assert_eq!(disclosure(&tree).name.as_deref(), Some("…".as_bytes()));
    let mut input = skein_lib::Writer::new(268);
    input.put(b"{\"topic\":\"").unwrap();
    input.put(&tool).unwrap();
    input.put(b"\"}").unwrap();
    let tree = line(&[call(1, b"read_docs", &input.finish(), None)], &[], None, false, 0);
    assert_eq!(disclosure(&tree).name.as_ref().unwrap().len(), 256);
    let mut name = skein_lib::Writer::new(258);
    name.put(&[b'x'; 252]).unwrap();
    name.put("ééé".as_bytes()).unwrap();
    let tree = line(&[call(1, &name.finish(), b"{}", None)], &[], None, false, 0);
    assert_eq!(disclosure(&tree).name.as_ref().unwrap().get(252..), Some("…".as_bytes()));
}

#[test]
fn settings_and_minutes_keep_their_counted_words() {
    let tree = line(&[call(1, b"settings", b"{}", None), call(2, b"settings", b"{}", None)], &[], None, false, 240);
    assert_eq!(disclosure(&tree).name.as_deref(), Some("Checked settings 2×".as_bytes()));
    assert_eq!(disclosure(&tree).description.as_deref(), Some(b"4 min".as_slice()));
}


fn live(tool: &[u8], input: &[u8]) -> c::CurrentCall {
    let mut call = current();
    call.tool = f::text(tool);
    call.input = f::text(input);
    call
}

#[test]
fn file_tools_count_what_they_touched() {
    for (tool, expected) in [
        ("read", "Read 3 files"),
        ("Read", "Read 3 files"),
        ("list", "Listed 3 folders"),
        ("Glob", "Listed 3 folders"),
        ("write", "Wrote 3 files"),
        ("Write", "Wrote 3 files"),
        ("edit", "Edited 3 files"),
        ("Edit", "Edited 3 files"),
        ("remove", "Removed 3 files"),
        ("search", "Searched the files 5×"),
        ("Grep", "Searched the files 5×"),
        ("shell", "Ran 5 commands"),
        ("Bash", "Ran 5 commands"),
    ] {
        let tree = line(
            &[
                call(1, tool.as_bytes(), br#"{"path":"same"}"#, None),
                call(2, tool.as_bytes(), br#"{"path":"same"}"#, None),
                call(3, tool.as_bytes(), br#"{"path":"same"}"#, None),
                call(4, tool.as_bytes(), b"{}", None),
                call(5, tool.as_bytes(), b"bad", None),
            ],
            &[],
            None,
            false,
            0,
        );
        assert_eq!(disclosure(&tree).name.as_deref(), Some(expected.as_bytes()), "{tool}");
    }
    let tree = line(&[call(1, b"read", b"{}", None), call(2, b"shell", b"{}", None)], &[], None, false, 0);
    assert_eq!(disclosure(&tree).name.as_deref(), Some(b"Read 1 file, ran 1 command".as_slice()));
}

#[test]
fn one_call_alone_says_its_argument() {
    for (tool, expected, counted) in [
        ("read", "Read arg", "Read 1 file"),
        ("Read", "Read arg", "Read 1 file"),
        ("list", "Listed arg", "Listed 1 folder"),
        ("Glob", "Listed arg", "Listed 1 folder"),
        ("write", "Wrote arg", "Wrote 1 file"),
        ("Write", "Wrote arg", "Wrote 1 file"),
        ("edit", "Edited arg", "Edited 1 file"),
        ("Edit", "Edited arg", "Edited 1 file"),
        ("remove", "Removed arg", "Removed 1 file"),
        ("search", "Searched for arg", "Searched the files"),
        ("Grep", "Searched for arg", "Searched the files"),
        ("shell", "Ran arg", "Ran 1 command"),
        ("Bash", "Ran arg", "Ran 1 command"),
    ] {
        for (input, name) in [(br#"{"text":"arg"}"#.as_slice(), expected), (b"bad", counted)] {
            assert_eq!(
                disclosure(&line(&[call(1, tool.as_bytes(), input, None)], &[], None, false, 0)).name.as_deref(),
                Some(name.as_bytes()),
                "{tool}"
            );
        }
    }
}

#[test]
fn a_call_under_way_says_its_present_form() {
    for (tool, expected) in [
        ("read", "reading arg"),
        ("Read", "reading arg"),
        ("list", "listing arg"),
        ("Glob", "listing arg"),
        ("write", "writing arg"),
        ("Write", "writing arg"),
        ("edit", "editing arg"),
        ("Edit", "editing arg"),
        ("remove", "removing arg"),
        ("search", "searching for arg"),
        ("Grep", "searching for arg"),
        ("shell", "running arg"),
        ("Bash", "running arg"),
    ] {
        let tree = line(&[], &[], Some(&live(tool.as_bytes(), br#"{"text":"arg"}"#)), true, 0);
        assert_eq!(disclosure(&tree).description.as_deref(), Some(expected.as_bytes()), "{tool}");
    }
    let tree = line(&[], &[], Some(&live(b"read", br#"{"path":"arg"}"#)), true, 0);
    assert_eq!(disclosure(&tree).name.as_deref(), Some(b"Reading arg".as_slice()));
}

#[test]
fn an_unreadable_call_under_way_is_its_name() {
    for tool in [
        "read_docs",
        "settings",
        "read",
        "Read",
        "list",
        "Glob",
        "write",
        "Write",
        "edit",
        "Edit",
        "remove",
        "search",
        "Grep",
        "shell",
        "Bash",
        "read_file",
    ] {
        let tree = line(&[], &[], Some(&live(tool.as_bytes(), b"bad")), true, 0);
        assert_eq!(disclosure(&tree).description.as_deref(), Some(tool.as_bytes()));
        assert_eq!(disclosure(&tree).name.as_deref(), Some(tool.as_bytes()));
    }
}

#[test]
fn a_cut_input_says_its_first_member() {
    let mut current = live(b"write", br#"{"path":"src/a.rs","content":"aaaa"#);
    current.cut = true;
    let tree = line(&[], &[], Some(&current), true, 0);
    assert_eq!(disclosure(&tree).description.as_deref(), Some(b"writing src/a.rs".as_slice()));
    assert!(fixtures::find(&tree, Role::Paragraph, b"write src/a.rs").is_some());
}

#[test]
fn a_command_is_its_first_line() {
    let input = br#"{"command":"cargo test\necho body"}"#;
    let tree = line(&[call(1, b"shell", input, None)], &[], Some(&live(b"shell", input)), true, 0);
    assert_eq!(disclosure(&tree).name.as_deref(), Some(b"Ran cargo test".as_slice()));
    assert_eq!(disclosure(&tree).description.as_deref(), Some(b"running cargo test".as_slice()));
    assert!(fixtures::find(&tree, Role::Paragraph, b"shell cargo test").is_some());
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"echo body").is_none());
}

#[test]
fn the_terminal_writes_the_ellipsis() {
    let tree = line_with_words(
        &[call(1, b"read", b"{}", None), call(2, b"read", b"{}", None)],
        &[],
        Some(&live(b"read", br#"{"path":"src/main.rs"}"#)),
        true,
        0,
        &fixtures::words(),
    );
    let mut print = skein_ui_text::Print::new(&fixtures::tree_limits()).unwrap();
    skein_ui_text::render(&tree, b"temper", None, &mut print);
    assert!(skein_lib::bytes::find(print.text.as_slice(), "Read 2 files: reading src/main.rs…".as_bytes()).is_some());
}


#[test]
fn failed_settings_proposals_do_not_claim_a_change() {
    let input = br#"{"reason":"private reason","items":[]}"#;
    let refusal = b"owner approval cannot be proposed";
    let cases: &[(u64, &[bool], &str, &str)] = &[
        (8, &[true], "see ×8, failed to propose a change", "1 failed"),
        (2, &[true], "see ×2, failed to propose a change", "1 failed"),
        (0, &[true], "Failed to propose a change", "1 failed"),
        (0, &[true, true], "Failed to propose a change 2×", "2 failed"),
        (0, &[false, true], "Proposed a change, failed to propose a change", "1 failed"),
        (0, &[true, false], "Failed to propose a change, proposed a change", "1 failed"),
        (0, &[false, true, false, true], "Proposed a change 2×, failed to propose a change 2×", "2 failed"),
    ];
    for (reads, proposals, name, description) in cases {
        let mut calls = List::with_capacity(CALLS);
        for number in 0..*reads {
            calls.push(call(number, b"see", b"{}", Some(b"seen"))).unwrap();
        }
        for (index, failed) in proposals.iter().enumerate() {
            let number = reads.checked_add(u64::try_from(index).unwrap()).unwrap();
            let outcome = if *failed { Some(refusal.as_slice()) } else { None };
            let mut proposal = call(number, b"propose_settings", input, outcome);
            proposal.failed = *failed;
            calls.push(proposal).unwrap();
        }
        let tree = line(calls.as_slice(), &[], None, false, 0);
        assert_eq!(disclosure(&tree).name.as_deref(), Some(name.as_bytes()));
        assert_eq!(disclosure(&tree).description.as_deref(), Some(description.as_bytes()));
        assert!(disclosure(&tree).styles.as_slice().contains(&look::WORK_FAILED));
        let mut failed_row = false;
        for node in tree.nodes() {
            if node.role == Role::Paragraph
                && node.name.as_deref()
                    == Some(b"propose_settings: failed: owner approval cannot be proposed".as_slice())
            {
                assert!(node.styles.as_slice().contains(&look::WORK_FAILED));
                failed_row = true;
            }
        }
        assert!(failed_row);
        assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"private reason").is_none());
    }
}


#[test]
fn proposing_settings_says_a_change_without_its_reason() {
    let input = br#"{"reason":"private reason","items":[]}"#;
    let one = call(1, b"propose_settings", input, None);
    let tree = line(core::slice::from_ref(&one), &[], None, false, 0);
    assert_eq!(disclosure(&tree).name.as_deref(), Some(b"Proposed a change".as_slice()));
    let tree = line(&[one.clone(), call(2, b"propose_settings", input, None)], &[], None, false, 0);
    assert_eq!(disclosure(&tree).name.as_deref(), Some("Proposed a change 2×".as_bytes()));
    let tree = line(&[call(2, b"settings", b"{}", None), one], &[], None, false, 0);
    assert_eq!(disclosure(&tree).name.as_deref(), Some(b"Checked settings, proposed a change".as_slice()));
    let tree = line(&[], &[], Some(&live(b"propose_settings", input)), true, 0);
    assert_eq!(disclosure(&tree).name.as_deref(), Some(b"Proposing a change".as_slice()));
    assert_eq!(disclosure(&tree).description.as_deref(), Some(b"proposing a change".as_slice()));
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"private reason").is_none());
}


type SeeRow = (&'static [&'static [u8]], Option<&'static [u8]>, &'static [u8], Option<&'static [u8]>);

#[test]
fn see_says_its_addresses_first_word() {
    let rows: &[SeeRow] = &[
        (&[b"changes", b"change 1", b"change 2", b"change 3"], None, b"Listed changes, read 3 changes", None),
        (&[b"change 3"], None, b"Read change 3", None),
        (&[b"change 3", b"change 3"], None, b"Read 1 change", None),
        (&[b"changes", b"task 9", b"task 9 tree"], None, b"Listed changes, read 1 task", None),
        (&[b"changes", b"goals", b"goals"], None, "Listed changes, listed goals 2×".as_bytes(), None),
        (&[b"help hold"], None, b"Read the page hold", None),
        (&[b"help search landing"], None, b"Searched the docs for landing", None),
        (&[b"changes", b"more x1"], None, b"Listed changes", None),
        (&[b"more x1"], None, b"Read on", None),
        (&[b"frobnicate"], None, b"see", None),
        (&[b"changes"], Some(b"change 2"), b"Listed changes", Some("reading change 2…".as_bytes())),
        (&[b"tasks"], None, b"Listed tasks", None),
        (&[b"task 9"], None, b"Read task 9", None),
        (&[b"inbox"], None, b"Checked the inbox", None),
        (&[b"me"], None, b"Checked your role", None),
        (&[b"project"], None, b"Read the project", None),
        (&[b"settings charter 1"], None, b"Checked settings", None),
        (&[b"ways"], None, b"Read the ways", None),
        (&[b"revision 2", b"revisions"], None, b"Read the history, read the history", None),
        (&[b"proposal t9/1"], None, b"Read proposal t9/1", None),
        (&[b"proposal t9/1", b"proposal t9/1", b"proposal p1/1"], None, b"Read 2 proposals", None),
        (&[b"help"], None, b"Read the docs", None),
        (&[b"help hold", b"help hold", b"help search landing", b"help search landing"], None, b"Read 3 docs", None),
        (&[b"help search landing", b"help hold", b"help hold", b"help search landing"], None, b"Read 3 docs", None),
        (&[b"file acme/shop src/main.rs main", b"file acme/shop src/main.rs other"], None, b"Read 1 file", None),
        (&[b"file acme/shop src/main.rs"], None, b"Read src/main.rs", None),
        (&[b"", b"frobnicate"], None, "see ×2".as_bytes(), None),
        (&[], Some(b"change 2"), b"Reading change 2", Some("reading change 2…".as_bytes())),
    ];
    let mut failures = List::uncapped();
    for (addresses, current, expected, description) in rows {
        let mut calls = List::with_capacity(CALLS);
        for (at, address) in addresses.iter().enumerate() {
            let input = skein_lib::bytes::concat(&[br#"{"address":""#, address, br#""}"#]);
            calls.push(call(u64::try_from(at).unwrap().checked_add(1).unwrap(), b"see", &input, None)).unwrap();
        }
        let current = match current {
            Some(address) => Some(live(b"see", &skein_lib::bytes::concat(&[br#"{"address":""#, address, br#""}"#]))),
            None => None,
        };
        let tree = line_with_words(calls.as_slice(), &[], current.as_ref(), current.is_some(), 0, &fixtures::words());
        let node = disclosure(&tree);
        if node.name.as_deref() != Some(*expected) || node.description.as_deref() != *description {
            failures.push((addresses, expected, description, node.name.clone(), node.description.clone())).unwrap();
        }
    }
    assert_eq!(failures.as_slice(), &[], "every see row");
}
