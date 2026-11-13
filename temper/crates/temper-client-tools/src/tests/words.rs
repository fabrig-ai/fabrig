#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary Rust assertions over the static table and help snapshots; testing-strategy.md step tests"
)]
use super::fixtures;
use crate::{
    help,
    jig::{
        self,
        argv::Usage,
        call::{Value, Values},
        table::{Kind, PLACEHOLDERS, Place},
    },
    table,
};

#[test]
fn every_tool_names_a_group() {
    for row in table::TOOLS {
        let words: Vec<_> = row.name.split(|byte| *byte == b' ').collect();
        assert!((1..=2).contains(&words.len()) && words.iter().all(|word| !word.is_empty()), "{row:?}");
        assert!(table::GROUPS.iter().any(|group| group.name == words[0]), "{row:?}");
    }
    for group in table::GROUPS {
        let rows: Vec<_> = table::TOOLS.iter().filter(|row| jig::table::name_words(row.name).0 == group.name).collect();
        assert!(!rows.is_empty(), "{group:?}");
        if rows.iter().any(|row| row.name == group.name) {
            assert!(rows.iter().all(|row| row.name == group.name), "{group:?}");
        }
    }
}

#[test]
fn every_shown_is_in_the_vocabulary() {
    for row in table::TOOLS {
        for form in row.forms {
            for argument in *form {
                let valid = if argument.place == Place::Switch {
                    argument.shown.is_empty()
                } else if argument.kind == Kind::Group {
                    argument.shown
                        == [
                            b"questions".as_slice(),
                            b"proposals",
                            b"escalations",
                            b"person-tasks",
                            b"results",
                            b"replies",
                        ]
                        .join(&b'|')
                } else if argument.kind == Kind::Order {
                    argument.shown == b"activity|priority"
                } else if argument.name == b"until" {
                    argument.shown
                        == if argument.kind == Kind::Literal { b"TIME".as_slice() } else { b"CONDITION".as_slice() }
                } else if let Kind::Word { set } = argument.kind {
                    argument.shown == set.join(&b'|')
                } else {
                    PLACEHOLDERS.contains(&argument.shown) || {
                        let pair: Vec<_> = argument.shown.split(|byte| *byte == b'/').collect();
                        pair.len() == 2 && pair.iter().all(|word| PLACEHOLDERS.contains(word))
                    }
                };
                assert!(
                    valid,
                    "{}: {} shows {}",
                    String::from_utf8_lossy(row.name),
                    String::from_utf8_lossy(argument.name),
                    String::from_utf8_lossy(argument.shown)
                );
            }
        }
    }
}

type ArgvCase = (&'static [&'static str], u8, &'static [Option<Values>]);

#[test]
fn argv_takes_a_group_and_a_verb() {
    let cases: &[ArgvCase] = &[
        (
            &["task", "show", "41"],
            jig::rows::TASK_AT,
            &[Some(Values::One(Value::Number(41))), None, None, None, None, None, None],
        ),
        (&["forge", "setup", "3", "--key", "k"], 35, &[Some(Values::One(Value::Number(3))), None, None, None, None]),
        (&["me"], jig::rows::ME_AT, &[None, None, None]),
        (
            &["settings", "show", "charter", "2"],
            jig::rows::SETTING_AT,
            &[Some(Values::One(Value::Word(0))), Some(Values::One(Value::Number(2))), None],
        ),
        (&["settings", "show"], jig::rows::SETTINGS_AT, &[]),
        (
            &["task", "decide", "4", "--hold", "2", "release"],
            jig::rows::TASK_DECIDE_AT,
            &[
                Some(Values::One(Value::Number(4))),
                Some(Values::One(Value::Number(2))),
                Some(Values::One(Value::Word(0))),
                None,
            ],
        ),
    ];
    for (words, tool, values) in cases {
        let call = fixtures::call(words);
        assert_eq!(call.tool, *tool, "{words:?}");
        assert_eq!(call.form, 0, "{words:?}");
        assert_eq!(call.values.as_ref(), *values, "{words:?}");
    }
    let proposal = fixtures::call(&["proposal", "decide", "t4/1", "accept"]);
    assert_eq!(proposal.tool, jig::rows::DECIDE_AT);
    assert_eq!(
        proposal.values.as_ref(),
        &[
            Some(Values::One(Value::Proposal(jig_client::contract::ProposalKey {
                proposer: jig_client::contract::Proposer::Task(4),
                number: 1
            }))),
            Some(Values::One(Value::Word(0))),
            None,
            None,
        ]
    );
    for (words, expected) in [
        (&["tasks", "--project", "1"][..], "temper ctl: no group tasks; temper ctl help lists them\n"),
        (&["forge-setup", "3"], "temper ctl: no group forge-setup; temper ctl help lists them\n"),
        (&["task", "41"], "temper ctl: no tool task 41; temper ctl help task lists them\n"),
        (&["task"], "temper ctl: task needs a verb; temper ctl help task lists them\n"),
    ] {
        assert_eq!(refusal(words).as_ref(), expected.as_bytes(), "{words:?}");
    }
    for (words, form, values) in [
        (
            &["settings", "show", "charter-words", "12", "--revision", "5"][..],
            5,
            [
                Some(Values::One(Value::Word(0))),
                Some(Values::One(Value::Number(12))),
                Some(Values::One(Value::Number(5))),
            ],
        ),
        (
            &["settings", "show", "project-words", "shop", "--revision", "5"][..],
            4,
            [
                Some(Values::One(Value::Word(3))),
                Some(Values::One(Value::Project(jig::call::Project::Name(b"shop".as_slice().into())))),
                Some(Values::One(Value::Number(5))),
            ],
        ),
    ] {
        let call = fixtures::call(words);
        assert_eq!((call.tool, call.form), (jig::rows::SETTING_AT, form), "{words:?}");
        assert_eq!(call.values.as_ref(), values.as_slice(), "{words:?}");
    }
}

#[test]
fn call_json_names_the_words() {
    let call = fixtures::from_json(br#"{"tool":"task show","arguments":{"task":41}}"#);
    assert_eq!(call, fixtures::call(&["task", "show", "41"]));
    assert_eq!(fixtures::json(br#"{"tool":"task","arguments":{"task":41}}"#), Err(Usage::Tool));
}

fn refusal(words: &[&str]) -> Box<[u8]> {
    let why = fixtures::parse(words).expect_err("refused words");
    let words: Vec<_> = words.iter().map(|word| word.as_bytes()).collect();
    let mut measured = skein_lib::Writer::new(1024);
    write_refusal(why, &words, &mut measured);
    let mut exact = skein_lib::Writer::new(measured.written());
    write_refusal(why, &words, &mut exact);
    exact.finish()
}
fn write_refusal(why: Usage, words: &[&[u8]], out: &mut skein_lib::Writer) {
    out.put(b"temper ctl: ").expect("prefix");
    table::refusal(why, words, out).expect("refusal");
    out.put(b"\n").expect("newline");
}

#[test]
fn help_lists_groups_then_a_group_then_a_tool() {
    for (words, golden) in [
        (&["help"][..], "help_groups"),
        (&["help", "task"], "help_task"),
        (&["help", "task", "show"], "help_task_show"),
        (&["help", "me"], "help_me"),
    ] {
        let call = fixtures::call(words);
        let selected = help::selection(&call).expect("help selection");
        fixtures::check_golden(golden, &fixtures::selected_help(selected));
    }
    for (words, golden) in [(&["help", "bogus"][..], "help_bogus"), (&["help", "task", "bogus"], "help_task_bogus")] {
        fixtures::check_golden(golden, &refusal(words));
    }
}

// The table's order through revision advance; a row's index is no stored value
// (requests are keyed by name), so a new row sits beside its group's.
const KEPT_INDICES: &[&[u8]] = &[
    b"me",
    b"inbox",
    b"task list",
    b"task show",
    b"chat show",
    b"task activity",
    b"task turn",
    b"task run",
    b"chat start",
    b"task message",
    b"chat close",
    b"task answer",
    b"proposal decide",
    b"task release",
    b"task stop",
    b"task cancel",
    b"task tree",
    b"repository adopt",
    b"sign-in",
    b"sign-out",
    b"pending",
    b"help",
    b"account list",
    b"model list",
    b"account connect",
    b"account disconnect",
    b"account add-key",
    b"account replace-key",
    b"account forget",
    b"account rank",
    b"account retry",
    b"task amend",
    b"account declare",
    b"account verify",
    b"forge list",
    b"forge setup",
    b"playground start",
    b"forge connect",
    b"forge replace",
    b"forge install",
    b"forge repositories",
    b"account export",
    b"account import",
    b"settings show",
    b"settings show",
    b"revision show",
    b"revision list",
    b"settings apply",
    b"settings propose",
    b"proposal show",
    b"change show",
    b"forge accept",
    b"revision advance",
];
#[test]
fn indices_are_kept() {
    for (index, name) in KEPT_INDICES.iter().enumerate() {
        assert_eq!(table::TOOLS[index].name, *name, "old index {index}");
    }
    assert_eq!(table::TOOLS[65].name, b"repository make");
    assert_eq!(table::TOOLS[66].name, b"repository copy");
    assert_eq!(table::TOOLS[usize::from(jig::rows::TASK_DECIDE_AT)].name, b"task decide");
    for (index, group) in table::GROUPS.iter().enumerate() {
        if table::TOOLS.iter().any(|row| row.name == group.name) {
            continue;
        }
        let mut expected = Vec::new();
        for row in table::TOOLS {
            if jig::table::name_words(row.name).0 == group.name {
                expected.extend_from_slice(&fixtures::usage(row));
                expected.extend_from_slice(b"  ");
                expected.extend_from_slice(row.line);
                expected.push(b'\n');
            }
        }
        assert_eq!(
            &*fixtures::selected_help(help::Selection::Group(u8::try_from(index).expect("group index fits"))),
            expected,
            "{group:?}"
        );
    }
}

const WATCHED: &[&[&str]] = &[
    &["me"],
    &["inbox"],
    &["task", "show", "1"],
    &["task", "tree", "1"],
    &["task", "run", "1"],
    &["task", "list", "--project", "1"],
    &["account", "list"],
    &["forge", "list"],
    &["change", "show", "1"],
];
#[test]
fn each_watched_read_waits_given_since_or_until() {
    for read in WATCHED {
        for flags in [&["--until", "changed"][..], &["--since", "0123456789abcdef"], &[]] {
            let words = fixtures::extended(read, flags);
            let call = fixtures::call(words.as_slice());
            let row = table::TOOLS[usize::from(call.tool)];
            assert_eq!(jig::call::waiting(row, &call), !flags.is_empty(), "{words:?}");
            assert_eq!(fixtures::printed(&call), jig::call::Parsed::Call { call: call.clone(), json: false });
            let encoded = jig::json::write_call(table::TOOLS, &call).unwrap();
            assert_eq!(fixtures::from_json(&encoded), call);
        }
    }
}
#[test]
fn a_read_with_parts_refuses_a_wait() {
    for (words, expected) in [
        (
            &["inbox", "--kind", "needs", "--until", "waiting"][..],
            "temper ctl: inbox waits on its subject alone: drop --kind\n",
        ),
        (
            &["task", "show", "1", "--tree", "--until", "ended"],
            "temper ctl: task show waits on its subject alone: drop --tree\n",
        ),
        (
            &["task", "list", "--project", "1", "--ended", "--until", "changed"],
            "temper ctl: task list waits on its subject alone: drop --ended\n",
        ),
        (
            &["task", "show", "1", "--run", "--tree", "--until", "ended"],
            "temper ctl: task show waits on its subject alone: drop --run\n",
        ),
        (
            &["inbox", "--root", "1", "--kind", "needs", "--until", "waiting"],
            "temper ctl: inbox waits on its subject alone: drop --root\n",
        ),
    ] {
        assert_eq!(refusal(words).as_ref(), expected.as_bytes());
    }
}
#[test]
fn timeout_alone_is_refused() {
    assert_eq!(refusal(&["me", "--timeout", "5s"]).as_ref(), b"temper ctl: --timeout needs --since or --until\n");
    for read in WATCHED {
        let words = fixtures::extended(read, &["--timeout", "5s"]);
        assert_eq!(fixtures::parse(words.as_slice()), Err(Usage::Timeout));
    }
    assert_eq!(fixtures::json(br#"{"tool":"me","arguments":{"timeout":5}}"#), Err(Usage::Timeout));
}
#[test]
fn no_tool_is_named_wait() {
    assert!(table::TOOLS.iter().all(|row| row.name != b"wait"));
    assert!(table::GROUPS.iter().all(|row| row.name != b"wait"));
    assert_eq!(refusal(&["wait", "me"]).as_ref(), b"temper ctl: no group wait; temper ctl help lists them\n");
    assert_eq!(fixtures::json(br#"{"tool":"wait","arguments":{"subject":"me"}}"#), Err(Usage::Tool));
}

#[test]
fn a_bounded_argument_refusal_names_the_tool_and_bound() {
    assert_eq!(
        refusal(&["task", "activity", "2", "--limit", "40"]).as_ref(),
        b"temper ctl: task activity: --limit COUNT takes a count from 1 to 32; temper ctl help task activity\n"
    );
}
#[test]
fn a_json_argument_refusal_names_the_tool_and_bound() {
    let why = fixtures::json(br#"{"tool":"task activity","arguments":{"task":2,"limit":40}}"#).expect_err("past limit");
    let mut measured = skein_lib::Writer::new(240);
    write_refusal(why, &[], &mut measured);
    let mut out = skein_lib::Writer::new(measured.written());
    write_refusal(why, &[], &mut out);
    assert_eq!(
        &out.finish()[..],
        b"temper ctl: task activity: --limit COUNT takes a count from 1 to 32; temper ctl help task activity\n"
    );
}

#[test]
fn every_argument_refusal_fits_one_line_with_its_help_call() {
    for (tool, row) in table::TOOLS.iter().enumerate() {
        for (form, arguments) in row.forms.iter().enumerate() {
            for (argument, _) in arguments.iter().enumerate() {
                let tool = u8::try_from(tool).unwrap();
                let form = u8::try_from(form).unwrap();
                let argument = u8::try_from(argument).unwrap();
                for why in [
                    Usage::Kind { tool, form, argument },
                    Usage::Rows { tool, form, argument },
                    Usage::Long { tool, form, argument },
                    Usage::Missing { tool, form, argument },
                    Usage::Twice { tool, form, argument },
                    Usage::Many { tool, form, argument },
                ] {
                    let mut measured = skein_lib::Writer::new(240);
                    write_refusal(why, &[], &mut measured);
                    let mut exact = skein_lib::Writer::new(measured.written());
                    write_refusal(why, &[], &mut exact);
                    let line = exact.finish();
                    assert_eq!(skein_lib::bytes::count(&line, b"\n", 2), 1, "{why:?}");
                    assert!(line.strip_suffix(b"\n").unwrap().ends_with(row.name), "{why:?}");
                }
            }
        }
    }
}

#[test]
fn a_json_argument_of_the_wrong_kind_keeps_its_form() {
    let why = fixtures::json(br#"{"tool":"task activity","arguments":{"task":2,"limit":"many"}}"#)
        .expect_err("count required");
    assert_eq!(why, Usage::Kind { tool: jig::rows::ACTIVITY_AT, form: 0, argument: 2 });
}
#[test]
fn a_call_that_names_no_tool_keeps_the_usage_line() {
    assert_eq!(
        refusal(&[]).as_ref(),
        b"temper ctl: usage: temper ctl DIR GROUP VERB [ARGUMENT...]; temper ctl help GROUP VERB prints one tool\n"
    );
}

#[test]
fn a_json_switch_refusal_says_how_to_omit_it() {
    let why =
        fixtures::json(br#"{"tool":"task show","arguments":{"task":2,"tree":false}}"#).expect_err("switch takes true");
    let mut measured = skein_lib::Writer::new(240);
    write_refusal(why, &[], &mut measured);
    let mut out = skein_lib::Writer::new(measured.written());
    write_refusal(why, &[], &mut out);
    assert_eq!(
        out.finish().as_ref(),
        b"temper ctl: task show: --tree is a switch: omit it or use true in --call; temper ctl help task show\n"
    );
}

#[test]
fn a_judged_settings_refusal_names_its_item_and_setting() {
    use jig_client::contract::{Refusal, Setting, Text};
    let why = Refusal::Judged {
        at: 1,
        setting: Setting::Application { kind: 7, scope: 2, number: 1 },
        why: Text::new(Box::from(b"way 4 is not offered".as_slice()), 128).expect("bounded reason"),
    };
    assert_eq!(
        jig::render::settings_refusal(&why, 128).as_ref(),
        b"refused; item 1; project-ways 2: way 4 is not offered"
    );
}

#[test]
fn a_refused_key_name_says_what_a_key_is() {
    let long = "a".repeat(65);
    for key in [Some("Bad"), Some("a_b"), Some(long.as_str()), None] {
        let mut words = ["task", "message", "41", "hello", "--key"].to_vec();
        if let Some(key) = key {
            words.push(key);
        }
        assert_eq!(
            refusal(&words).as_ref(),
            b"temper ctl: task message: --key KEY takes 1 to 64 lower-case letters, digits or hyphens; temper ctl help task message\n",
            "{words:?}"
        );
    }
}

#[test]
fn a_call_with_a_refused_key_name_says_what_a_key_is() {
    for (json, expected) in [
        (
            br#"{"tool":"task message","arguments":{"task":41,"text":"hello","key":"Bad"}}"#.as_slice(),
            b"temper ctl: task message: --key KEY takes 1 to 64 lower-case letters, digits or hyphens; temper ctl help task message\n".as_slice(),
        ),
        (
            br#"{"tool":"task stop","arguments":{"task":41,"key":"Bad"}}"#,
            b"temper ctl: task stop: --key KEY takes 1 to 64 lower-case letters, digits or hyphens; temper ctl help task stop\n",
        ),
    ] {
        let why = fixtures::json(json).expect_err("refused key name");
        let mut measured = skein_lib::Writer::new(240);
        write_refusal(why, &[], &mut measured);
        let mut out = skein_lib::Writer::new(measured.written());
        write_refusal(why, &[], &mut out);
        assert_eq!(out.finish().as_ref(), expected);
    }
}

#[test]
fn every_key_name_refusal_fits_one_line_with_its_help_call() {
    for (tool, row) in table::TOOLS.iter().enumerate() {
        if row.keyed == jig::table::Keyed::Never {
            continue;
        }
        let why = Usage::KeyName { tool: u8::try_from(tool).expect("row fits") };
        let mut measured = skein_lib::Writer::new(240);
        write_refusal(why, &[], &mut measured);
        let mut exact = skein_lib::Writer::new(measured.written());
        write_refusal(why, &[], &mut exact);
        let line = exact.finish();
        assert_eq!(skein_lib::bytes::count(&line, b"\n", 2), 1, "{row:?}");
        let mut help = b"; temper ctl help ".to_vec();
        help.extend_from_slice(row.name);
        assert!(line.strip_suffix(b"\n").expect("newline").ends_with(&help), "{row:?}");
    }
}
