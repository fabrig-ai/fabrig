use super::fixtures;
use crate::{jig, table};
use skein_lib::List;
const GOLDENS: &[(Option<u8>, &str)] = &[
    (Some(65), "help_repository_make"),
    (Some(66), "help_repository_copy"),
    (Some(jig::rows::PROJECT_AMEND_AT), "help_project_amend"),
    (Some(jig::rows::GOAL_START_AT), "help_goal_start"),
    (Some(jig::rows::ROLE_START_AT), "help_role_start"),
    (Some(jig::rows::ROLE_LIST_AT), "help_role_list"),
    (Some(jig::rows::ROLE_SHOW_AT), "help_role_show"),
    (Some(jig::rows::ROLE_TALK_AT), "help_role_talk"),
    (Some(jig::rows::CHAT_CREATE_AT), "help_chat_start"),
    (Some(jig::rows::TASK_TEXT_AT), "help_task_text"),
    (Some(jig::rows::LIFT_AT), "help_hold_lift"),
    (Some(3), "help_task_show"),
    (Some(12), "help_proposal_decide"),
    (Some(jig::rows::TASK_DECIDE_AT), "help_task_decide"),
    (Some(10), "help_chat_close"),
    (Some(16), "help_task_tree"),
    (Some(34), "help_forge_list"),
    (Some(50), "help_change_show"),
    (Some(57), "help_landing_show"),
    (Some(58), "help_landing_list"),
    (Some(20), "help_pending"),
    (Some(17), "help_repository_adopt"),
    (Some(31), "help_task_amend"),
    (Some(37), "help_forge_connect"),
    (Some(38), "help_forge_replace"),
    (Some(39), "help_forge_install"),
    (Some(jig::rows::EXPORT_AT), "help_account_export"),
    (Some(jig::rows::IMPORT_AT), "help_account_import"),
    (Some(jig::rows::SETTINGS_AT), "help_settings_show"),
    (Some(jig::rows::SETTING_AT), "help_settings_show"),
    (Some(jig::rows::REVISION_AT), "help_revision_show"),
    (Some(jig::rows::REVISIONS_AT), "help_revision_list"),
    (Some(jig::rows::CHANGE_SETTINGS_AT), "help_settings_apply"),
    (Some(jig::rows::PROPOSE_SETTINGS_AT), "help_settings_propose"),
    (Some(jig::rows::PROPOSAL_AT), "help_proposal_show"),
    (Some(jig::rows::MOVE_TO_NEWEST_AT), "help_revision_advance"),
    (Some(jig::rows::RESTORE_AT), "help_revision_restore"),
    (Some(jig::rows::PROJECT_SHOW_AT), "help_project_show"),
    (Some(jig::rows::PROJECT_FUND_AT), "help_project_fund"),
];
#[test]
fn the_help_is_the_page_ctl_then_the_tools() {
    let help = fixtures::help(None);
    let page = temper_docs::find(b"ctl").expect("ctl page").text.as_bytes();
    let tools = help.strip_prefix(page).expect("the ctl page precedes the tools");
    fixtures::check_golden("help", tools);
}
#[test]
fn the_help_prints_as_its_golden_files() {
    for (tool, name) in GOLDENS {
        if temper_test_env::rewrite() {
            std::fs::write(
                std::path::Path::new("src/tests/golden").join(name).with_extension("txt"),
                fixtures::help(*tool),
            )
            .expect("write help golden");
        }
        assert_eq!(fixtures::help(*tool), fixtures::golden(name), "{name}");
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the help test compares rendered UTF-8 with the page's quoted lines")]
fn review_recipe_quotes_the_waiting_changes_output() {
    use temper_client_protocol_wire::v1 as w;
    let page = temper_docs::find(b"review-before-landing").expect("review recipe");
    let checking = page.text.split("## Checking it took\n").nth(1).expect("checking section");
    let mut parts = super::change::approval_parts();
    parts.step = w::Step::First;
    parts.ci = w::Ci::Passed;
    let mut approval = parts.approvals.get(0).expect("waiting approval").clone().into_parts();
    approval.number = 1;
    approval.role = 1;
    approval.freshness = w::RuleFreshness::Exact;
    *parts.approvals.get_mut(0).expect("waiting approval") = w::Approval::new(&w::CEILINGS, approval).unwrap();
    let panel = w::ChangePanel::new(&w::CEILINGS, parts).expect("waiting change");
    let output =
        crate::change::print(Some(&panel), false, None, b"", &fixtures::limits_root(), &crate::change::Holds::new());
    assert_eq!(output.status, 0);
    let output = core::str::from_utf8(&output.stdout).expect("text output");
    let mut quoted = List::with_capacity(2);
    for block in checking.split("```\n").skip(1).step_by(2) {
        for line in block.lines() {
            if line.starts_with("approval:") || line.starts_with("next:") {
                quoted.push(line).expect("two quoted output lines");
            }
        }
    }
    assert_eq!(quoted.len(), 2, "quote both the approval and next lines");
    for line in quoted.as_slice() {
        let expected = line
            .replace("PROJECT", "1")
            .replace("ROLE", "1")
            .replace("URL", core::str::from_utf8(panel.pull().as_ref().expect("pull request").address()).unwrap());
        let mut found = false;
        for rendered in output.lines() {
            found |= rendered == expected;
        }
        assert!(found, "quoted line differs from ctl: {expected}\n{output}");
    }
}

#[test]
fn help_names_how_to_read_a_runs_brief() {
    for name in [b"chat".as_slice(), b"charter", b"configure"] {
        let page = temper_docs::find(name).expect("help page");
        for guidance in [
            "temper ctl DIR chat show TASK",
            "brief",
            "document sections",
            "commits",
            "--before CURSOR",
            "task text",
            "specification",
            "instructions",
        ] {
            assert!(
                skein_lib::bytes::find(page.text.as_bytes(), guidance.as_bytes()).is_some(),
                "{} help must explain: {guidance}",
                page.name
            );
        }
    }
    for (at, guidance) in [
        (jig::rows::CHAT_AT, "chat show TASK --before CURSOR"),
        (jig::rows::TURN_AT, "task turn TASK TURN --part PART --from OFFSET"),
    ] {
        let help = fixtures::help(Some(at));
        for fact in [
            "brief supplied to the run",
            "document sections",
            "commits",
            "task activity TASK",
            "A.T",
            "task turn TASK A.T",
            guidance,
        ] {
            assert!(skein_lib::bytes::find(&help, fact.as_bytes()).is_some(), "tool {at} help must explain: {fact}");
        }
    }
}

#[test]
fn chat_start_explains_subagent_choices() {
    let help = fixtures::help(Some(jig::rows::CHAT_CREATE_AT));
    for explanation in [
        "each model named once, with one effort",
        "effort is required when the model declares efforts",
        "--sub-agent MODEL_A:EFFORT_A --sub-agent MODEL_B:EFFORT_B",
        "distinct model names offered by `model list`",
        "Available choices only: this list neither starts children nor assigns",
        "choices by child order",
        "The parent's `sub_agent` call selects a model with",
        "optional `llm`; omitted, it uses main's model and effort",
        "One model cannot be offered at two child efforts",
        "multiple children may use the same choice",
    ] {
        assert!(
            skein_lib::bytes::find(&help, explanation.as_bytes()).is_some(),
            "chat start help must explain: {explanation}"
        );
    }
}
#[test]
fn chat_start_points_to_model_list_and_models_print_readably() {
    let help = fixtures::help(Some(jig::rows::CHAT_CREATE_AT));
    for reference in ["as model list shows it", "distinct model names offered by `model list`"] {
        assert!(skein_lib::bytes::find(&help, reference.as_bytes()).is_some(), "{reference}");
    }
    let mut root = fixtures::account_root(false);
    root.deliver(jig_client::contract::Delivery::Me(Box::new(readable_models())));
    let call = root.submit(&["model", "list", "--project", "1"]);
    fixtures::deliver_accounts(&mut root, fixtures::accounts());
    let printed = root.answer(call).expect("model panel");
    assert_eq!(printed.status, 0);
    for fact in [
        "models for project 1",
        "provider 9 · Scripted",
        "model chosen-model",
        "description: Careful reasoning",
        "window: 8192 tokens",
        "output: 1024 tokens",
        "efforts: high",
        "prices (per 1000 tokens)",
        "input: 7",
        "cached input: 2",
        "cache write: 9",
        "output: 11",
        "account 43",
        "source 71",
        "not usable",
        "model plain-model",
        "provider unknown",
        "efforts: none listed",
        "digest: models ",
    ] {
        assert!(fixtures::has(&printed, fact.as_bytes()), "{fact}: {printed:?}");
    }
    for raw in [b"provider_name:".as_slice(), b"bytes:", b"cut:", b"cache_write:", b"needs an account"] {
        assert!(!fixtures::has(&printed, raw), "{raw:?}: {printed:?}");
    }
    let json = root.submit(&["model", "list", "--project", "1", "--json"]);
    fixtures::deliver_accounts(&mut root, fixtures::accounts());
    let json = root.answer(json).expect("model records");
    assert_eq!(json.status, 0);
    assert!(fixtures::has(&printed, &fixtures::last_digest(&json.stdout)));
}

#[test]
fn model_panels_keep_projection_loss_and_output_bounds() {
    use jig_client::contract as c;
    for loss in 0..3u8 {
        let mut me = readable_models();
        match loss {
            0 => me.providers.get_mut(0).expect("provider").name.cut = true,
            1 => {
                let mut description =
                    skein_lib::Writer::new(usize::try_from(crate::limits::CLIENT_TEXT).expect("text bound"));
                for _ in 0..crate::limits::CLIENT_TEXT {
                    description.put(b"x").expect("description bound");
                }
                me.projects.get_mut(0).expect("project").models.get_mut(0).expect("model").description =
                    Some(c::Text::new(description.finish(), crate::limits::CLIENT_TEXT).expect("description"));
            }
            _ => {
                let project = me.projects.get_mut(0).expect("project");
                let model = project.models.get(0).expect("model").clone();
                project.models = List::with_capacity(64);
                for _ in 0..64u32 {
                    project.models.push(model.clone()).expect("model room");
                }
            }
        }
        let mut root = fixtures::account_root(false);
        root.deliver(c::Delivery::Me(Box::new(me)));
        let call = root.submit(&["model", "list", "--project", "1"]);
        fixtures::deliver_accounts(&mut root, fixtures::accounts());
        let printed = root.answer(call).expect("cut model panel");
        assert_eq!(printed.status, 0);
        assert!(printed.stdout.len() <= usize::try_from(crate::limits::TEXT).expect("text bound"));
        assert!(fixtures::has(&printed, b"cut: true"), "loss {loss}: {printed:?}");
        let json = root.submit(&["model", "list", "--project", "1", "--json"]);
        fixtures::deliver_accounts(&mut root, fixtures::accounts());
        let json = root.answer(json).expect("cut model records");
        assert!(fixtures::has(&json, b"\"cut\":true"));
        assert!(fixtures::has(&printed, &fixtures::last_digest(&json.stdout)));
    }
}

fn readable_models() -> jig_client::contract::Me {
    use jig_client::contract as c;
    let mut me = fixtures::choice_me(100);
    me.providers = fixtures::provider_offers();
    let model = me.projects.get_mut(0).expect("project").models.get_mut(0).expect("model");
    model.provider = Some(fixtures::PROVIDER);
    model.description = Some(fixtures::act_text(b"Careful reasoning"));
    model.prices = c::ModelPrices { input: 7, cached: 2, cache_write: Some(9), output: 11, unit: 1000 };
    me
}

// client/ctl/README.md 5.2--5.6, with the plan's typed proposals and close reason.
const USAGES: &[&[u8]] = &[
    b"me [--since DIGEST] [--until CONDITION] [--timeout DURATION]\n",
    b"inbox [--project PROJECT] [--kind questions|proposals|escalations|person-tasks|results|replies] [--root TASK] [--after CURSOR] [--since DIGEST] [--until CONDITION] [--timeout DURATION]\n",
    b"task list --project PROJECT [--label chat|goal] [--tracked] [--mine] [--ended] [--order activity|priority] [--after CURSOR] [--parent TASK] [--since DIGEST] [--until CONDITION] [--timeout DURATION]\n",
    b"task show TASK [--tree] [--activity COUNT] [--run] [--since DIGEST] [--until CONDITION] [--timeout DURATION]\n",
    b"chat show TASK [--before CURSOR]\n",
    b"task activity TASK [--before CURSOR] [--limit COUNT]\n",
    b"task turn TASK TURN [--part PART] [--from OFFSET]\n",
    b"task run TASK [--since DIGEST] [--until CONDITION] [--timeout DURATION]\n",
    b"chat start --project PROJECT [--template TEMPLATE] TEXT [--model MODEL] [--effort EFFORT] [--sub-agent MODEL[:EFFORT]]... --key KEY\n",
    b"task message TASK TEXT --key KEY\n",
    b"chat close TASK [--move TASK]... [--reason TEXT] --key KEY\n",
    b"task answer TASK QUESTION TEXT [--key KEY]\ntask answer TASK --rev REV CHOICE TEXT [--key KEY]\ntask answer TASK --rev REV TEXT [--key KEY]\ntask answer TASK --rev REV --approve STATE [TEXT] [--key KEY]\ntask answer TASK --rev REV --cannot REASON [--key KEY]\n",
    b"proposal decide PROPOSAL accept|reject|pass-up [REASON] [--running stays|moves] [--key KEY]\n",
    b"task release TASK --hold REV [--again] [--key KEY]\n",
    b"task stop TASK [--key KEY]\n",
    b"task cancel TASK [--grace DURATION] REASON [--key KEY]\n",
    b"task tree TASK [--since DIGEST] [--until CONDITION] [--timeout DURATION]\n",
    b"repository adopt --project PROJECT [--forge FORGE] --owner OWNER --name REPOSITORY --landing BRANCH [--role owned|fork|context] [--prefix PREFIX] [--no-home] [--no-ci] [--check GATE]... --key KEY\n",
    b"sign-in FILE\n",
    b"sign-out\n",
    b"pending\n",
    b"help [GROUP [VERB]]\n",
    b"account list [--since DIGEST] [--until CONDITION] [--timeout DURATION]\n",
    b"model list [--project PROJECT]\n",
    b"account connect PROVIDER --key KEY\n",
    b"account disconnect ACCOUNT [--source sign_in|key|program] [--key KEY]\n",
    b"account add-key PROVIDER FILE [--account ACCOUNT] [--deployment] --key KEY\n",
    b"account replace-key ACCOUNT FILE [--key KEY]\n",
    b"account forget ACCOUNT [--source sign_in|key|program] [--key KEY]\n",
    b"account rank ACCOUNT POSITION [--source sign_in|key|program] [--key KEY]\n",
    b"account retry ACCOUNT [--source sign_in|key|program] [--key KEY]\n",
    b"task amend TASK [--rev REV] [--priority PRIORITY | --untracked] [--deadline TIME | --no-deadline] [--instructions FILE] [--spec FILE] [--dependency TASK]... [--no-dependencies] [--model MODEL [--effort EFFORT]] [--tasks N] REASON [--key KEY]\n",
    b"account declare PROVIDER ADAPTER NAME [--account ACCOUNT] --key KEY\n",
    b"account verify ACCOUNT [--source sign_in|key|program] [--key KEY]\n",
    b"forge list [--since DIGEST] [--until CONDITION] [--timeout DURATION]\n",
    b"forge setup [FORGE] [--org ORG] [--repository REPOSITORY] [--project PROJECT] [--playground] [--key KEY]\n",
    b"playground start [--key KEY]\n",
    b"forge connect FORGE github-app APP FILE [--read-only] --key KEY\nforge connect FORGE github --key KEY\nforge connect FORGE forgejo FILE --key KEY\nforge connect FORGE forgejo-admin FILE [--first ORG/REPOSITORY] [--project PROJECT] [--label LABEL] --key KEY\nforge connect github-app APP FILE [--read-only] --key KEY\nforge connect github --key KEY\nforge connect forgejo-admin FILE [--first ORG/REPOSITORY] [--project PROJECT] [--label LABEL] --api HOST:PORT [--tls NAME] --git URL [--web URL] [--git-program FILE] --key KEY\nforge connect forgejo FILE --api HOST:PORT [--tls NAME] --git URL [--web URL] [--git-program FILE] --key KEY\n",
    b"forge replace FORGE FILE --key KEY\n",
    b"forge install FORGE [--key KEY]\n",
    b"forge repositories FORGE [--installation INSTALLATION] [--page PAGE]\n",
    b"account export ACCOUNT FILE\n",
    b"account import FILE --key KEY\n",
    b"settings show\n",
    b"settings show charter|template|policy|model NUMBER [--revision REVISION]\nsettings show rules\nsettings show landing-rule NUMBER [--project PROJECT] [--revision REVISION]\nsettings show forge-charter NUMBER [--revision REVISION]\nsettings show project-ways|project-questions|project-classes|project-words PROJECT [--revision REVISION]\nsettings show charter-words NUMBER [--revision REVISION]\nsettings show template-goal NUMBER [--revision REVISION]\nsettings show follow NUMBER --project PROJECT [--revision REVISION]\nsettings show dependency NUMBER --project PROJECT [--revision REVISION]\n",
    b"revision show REVISION\n",
    b"revision list [--before REVISION]\n",
    b"settings apply FILE... --against REVISION [--running stays|moves] --key KEY\n",
    b"settings propose FILE [REASON] --against REVISION --key KEY\n",
    b"proposal show PROPOSAL\n",
    b"change show TASK [--project PROJECT] [--since DIGEST] [--until CONDITION] [--timeout DURATION]\n",
    b"forge accept FORGE INSTALLATION [--key KEY]\n",
    b"revision advance REVISION --key KEY\n",
    b"task decide TASK --hold REV release|leave|pass-up [REASON] [--key KEY]\n",
    b"project show PROJECT\n",
    b"project fund PROJECT AMOUNT [--role ROLE] --key KEY\n",
    b"hold lift HOLD --rev REV [--with-tasks] [--key KEY]\n",
    b"landing show TASK [--project PROJECT]\n",
    b"landing list --project PROJECT [--ended]\n",
    b"forge remove FORGE [--key KEY]\n",
    b"playground end [--key KEY]\n",
    b"task text TASK [--field specification|instructions|result] [--from OFFSET]\n",
    b"goal start --project PROJECT [--template TEMPLATE] TEXT [--charter CHARTER] [--priority PRIORITY] [--budget AMOUNT] [--as-allowed] [--deadline TIME] [--model MODEL] [--effort EFFORT] [--sub-agent MODEL[:EFFORT]]... --key KEY\n",
    b"revision restore REVISION [REASON] --key KEY\n",
    b"project amend PROJECT --visibility public|private --key KEY\n",
    b"repository make --project PROJECT [--forge FORGE] ORG/NAME [--no-home] [--until CONDITION] [--timeout DURATION] --key KEY\n",
    b"repository copy --project PROJECT [--forge FORGE] ADDRESS [--as ORG/NAME] [--no-home] [--until CONDITION] [--timeout DURATION] --key KEY\n",
    b"task take TASK [--key KEY]\n",
    b"task hand-back TASK [--key KEY]\n",
    b"task notes TASK --project PROJECT [--after CURSOR]\n",
    b"project make NAME --key KEY\n",
    b"project delete PROJECT --key KEY\n",
    b"role start --project PROJECT --budget AMOUNT TEXT [--charter CHARTER] --key KEY\n",
    b"role list --project PROJECT [--after CURSOR]\n",
    b"role show TASK [--report]\n",
    b"role talk TASK TEXT [--timeout DURATION] --key KEY\nrole talk TASK --leave --key KEY\n",
    b"draft ready --project PROJECT [--owner OWNER --name NAME] [--chat TASK] PATH --key KEY\n",
    b"draft not-ready --project PROJECT [--owner OWNER --name NAME] PATH --key KEY\n",
    b"draft list --project PROJECT [--until CONDITION] [--timeout DURATION]\n",
    b"follow list --project PROJECT\n",
    b"follow show --project PROJECT FOLLOW\n",
    b"follow start --project PROJECT FOLLOW COMMIT --key KEY\n",
    b"follow hold --project PROJECT FOLLOW COMMIT --reason TEXT --key KEY\n",
    b"follow skip --project PROJECT FOLLOW COMMIT --reason TEXT --key KEY\n",
    b"follow merge --project PROJECT FOLLOW COMMIT [COMMIT]... --key KEY\n",
];
#[test]
fn a_tools_usage_reads_as_the_design_writes_it() {
    assert_eq!(table::TOOLS.len(), USAGES.len());
    for (tool, expected) in table::TOOLS.iter().zip(USAGES) {
        assert_eq!(&*fixtures::usage(tool), *expected, "{tool:?}");
    }
}
#[test]
fn each_derivation_names_every_tool() {
    let mut forms = List::with_capacity(fixtures::TOOL_FORMS);
    let restore: &[&[&str]] = &[&["revision", "restore", "4", "--key", "restore-4"]];
    for words in fixtures::EVERY_TOOL.iter().chain(restore) {
        let mut call = fixtures::call(words);
        if call.tool == jig::rows::CHANGE_SETTINGS_AT || call.tool == jig::rows::PROPOSE_SETTINGS_AT {
            let value = jig::call::Value::Text(br#"{"items":[]}"#.as_slice().into());
            *call.values.get_mut(0).expect("FILE") = Some(if call.tool == jig::rows::CHANGE_SETTINGS_AT {
                jig::call::Values::Many(Box::new([value]))
            } else {
                jig::call::Values::One(value)
            });
        }
        forms.push((call.tool, call.form)).expect("all table forms");
        let json = jig::json::write_call(table::TOOLS, &call).expect("call JSON");
        assert_eq!(fixtures::from_json(&json), call, "{words:?}");
    }
    assert_eq!(forms.len(), fixtures::TOOL_FORMS);
    let help = fixtures::help(None);
    let list = help
        .strip_prefix(temper_docs::find(b"ctl").expect("ctl page").text.as_bytes())
        .expect("the ctl page precedes the generated list");
    let mut expected = skein_lib::Writer::new(list.len());
    for group in table::GROUPS {
        expected.put(group.name).expect("group");
        expected.put(b"  ").expect("separator");
        expected.put(group.line).expect("line");
        expected.put(b"\n").expect("newline");
    }
    for (index, tool) in table::TOOLS.iter().enumerate() {
        for form in 0..tool.forms.len() {
            let pair = (u8::try_from(index).expect("row fits"), u8::try_from(form).expect("form fits"));
            let mut count = 0_u32;
            for value in &forms {
                if *value == pair {
                    count += 1;
                }
            }
            assert_eq!(count, fixtures::EXAMPLES_PER_FORM, "{tool:?} {form}");
        }
        let record = jig::json::write_tool(tool, &crate::limits::tools()).expect("tool record JSON");
        let prefix_length = b"{\"name\":\""
            .len()
            .checked_add(tool.name.len())
            .expect("record prefix size")
            .checked_add(1)
            .expect("record prefix size");
        let mut prefix = skein_lib::Writer::new(prefix_length);
        prefix.put(br#"{"name":""#).expect("record prefix");
        prefix.put(tool.name).expect("tool name");
        prefix.put(b"\"").expect("end name");
        assert!(record.starts_with(&prefix.finish()), "{tool:?}");
    }
    assert_eq!(list, &*expected.finish(), "the list has exactly the table's rows");
}

#[test]
fn help_includes_each_row_of_its_name() {
    assert_eq!(fixtures::help(Some(jig::rows::SETTINGS_AT)), fixtures::help(Some(jig::rows::SETTING_AT)));
}
#[test]
fn help_refuses_a_writer_one_byte_short() {
    let bytes = fixtures::help(None);
    let short = bytes.len().checked_sub(fixtures::HELP_SHORTFALL).expect("nonempty help");
    let mut out = skein_lib::Writer::new(short);
    assert_eq!(crate::help::text(None, &mut out), Err(skein_lib::Overflow));
    assert_eq!(out.written(), fixtures::NO_HELP_BYTES, "overflow does not publish a partial help");
}
#[test]
fn the_moves_places_name_their_rows_in_the_table() {
    for (at, row) in [(jig::rows::EXPORT_AT, &jig::rows::EXPORT), (jig::rows::IMPORT_AT, &jig::rows::IMPORT)] {
        assert_eq!(table::TOOLS[usize::from(at)].name, row.name);
    }
}

pub(super) const RECORD: jig::table::Tool = jig::table::Tool {
    name: b"fixture",
    line: b"Read a task",
    prints: b"task N",
    result: b"task",
    forms: &[
        &[
            jig::table::positional(
                b"task",
                b"N",
                jig::table::Kind::Number,
                jig::table::Occurs::Required,
                b"Task number",
            ),
            jig::table::flag(
                b"activity",
                b"K",
                jig::table::Kind::Count,
                jig::table::Occurs::Optional,
                b"Latest activity lines",
            ),
            jig::table::switch(b"tree", b"Include the live tree"),
            jig::table::repeated(b"word", b"a|b|c", jig::table::Kind::Word { set: &[b"a", b"b", b"c"] }, 2, b"Words"),
        ],
        &[
            jig::table::positional(
                b"text",
                b"TEXT",
                jig::table::Kind::Text,
                jig::table::Occurs::Required,
                b"Words to send",
            ),
            jig::table::flag(
                b"until",
                b"K",
                jig::table::Kind::NamedNumber {
                    words: &[b"settled"],
                    prefixes: &[b"connected:", b"made:"],
                    min: 1,
                    max: 65535,
                },
                jig::table::Occurs::Optional,
                b"Condition",
            ),
            jig::table::flag(b"timeout", b"D", jig::table::Kind::Duration, jig::table::Occurs::Optional, b"Timeout"),
        ],
    ],
    keyed: jig::table::Keyed::Optional,
    area: 1,
    notes: b"First note.\nSecond note.\n",
};
const RECORD_FORMS: &[u8] = b"fixture N [--activity K] [--tree] [--word a|b|c]... [--key KEY]\n  N (task): a number; required. Task number\n  --activity K: a count from 1 to 32; optional. Latest activity lines\n  --tree: a switch. Include the live tree\n  --word a|b|c: one of a, b or c; repeated, at most 2. Words\nfixture TEXT [--until K] [--timeout D] [--key KEY]\n  TEXT (text): text, at most 1048576 bytes, or - to read stdin; required. Words to send\n  --until K: one of settled, connected:N or made:N with N from 1 to 65535; optional. Condition\n  --timeout D: a duration: seconds, or a number with s, m or h, at most 3600 seconds; optional. Timeout\n";
#[test]
fn a_tools_record_reads_as_written() {
    let expected = expected_record(b"Read a task\nPrints: task N\n--json: the record task.\nKey: --key KEY, optional.\nFirst note.\nSecond note.\n");
    let mut out = skein_lib::Writer::new(expected.len());
    jig::help::tool(&RECORD, &crate::limits::tools(), &mut out).expect("fixture record fits");
    assert_eq!(out.finish(), expected);
}
#[test]
fn an_empty_prints_and_notes_are_left_out() {
    let record = jig::table::Tool { prints: b"", notes: b"", ..RECORD };
    let expected = expected_record(b"Read a task\n--json: the record task.\nKey: --key KEY, optional.\n");
    let mut out = skein_lib::Writer::new(expected.len());
    jig::help::tool(&record, &crate::limits::tools(), &mut out).expect("fixture record fits");
    assert_eq!(out.finish(), expected);
}
#[test]
fn each_row_keeps_its_notes_within_their_bound() {
    for row in table::TOOLS {
        assert!(row.notes.len() <= jig::table::NOTES_BYTES, "{row:?}");
        assert!(row.notes.is_empty() || row.notes.ends_with(b"\n"), "{row:?}");
    }
}

fn expected_record(ending: &[u8]) -> Box<[u8]> {
    let mut expected = skein_lib::Writer::new(RECORD_FORMS.len().checked_add(ending.len()).expect("record size"));
    expected.put(RECORD_FORMS).expect("forms");
    expected.put(ending).expect("ending");
    expected.finish()
}

#[test]
fn each_row_says_what_it_does_and_prints() {
    for row in table::TOOLS {
        assert!(!row.line.is_empty(), "{row:?}");
        assert!(!row.prints.is_empty(), "{row:?}");
        for form in row.forms {
            for argument in *form {
                if row.name != jig::rows::HELP.name {
                    assert!(!argument.line.is_empty(), "{row:?}: {argument:?}");
                }
            }
        }
    }
}

#[test]
fn the_project_reference_is_what_the_table_renders() {
    let mut project = None;
    for group in table::GROUPS {
        if group.name == b"project" {
            project = Some(group);
            break;
        }
    }
    let group = project.expect("project group");
    let mut measured = skein_lib::Writer::new(temper_docs::PAGE_BYTES);
    crate::help::reference(group, &mut measured).expect("project reference fits");
    let mut out = skein_lib::Writer::new(measured.written());
    crate::help::reference(group, &mut out).expect("project reference measured");
    let rendered = out.finish();
    let path = std::path::Path::new("../../docs/help/reference/ctl-project.md");
    let write = temper_test_env::rewrite();
    if write {
        std::fs::write(path, &rendered).expect("write project reference");
    }
    assert_eq!(
        std::fs::read(path).expect("project reference"),
        rendered.as_ref(),
        "regenerate with SKEIN_CODEGEN_WRITE=1 and this test"
    );
}

#[test]
fn the_task_chat_and_inbox_references_are_what_the_table_renders() {
    for (group, page) in
        [(b"chat".as_slice(), b"ctl-chat".as_slice()), (b"task", b"ctl-task"), (b"inbox", b"ctl-inbox")]
    {
        reference_matches(group, page);
    }
}
#[test]
fn the_account_reference_is_what_the_table_renders() {
    reference_matches(b"account", b"ctl-account");
}
#[test]
fn the_draft_reference_is_what_the_table_renders() {
    reference_matches(b"draft", b"ctl-draft");
}
#[test]
fn the_forge_reference_is_what_the_table_renders() {
    reference_matches(b"forge", b"ctl-forge");
}
#[test]
fn the_playground_reference_is_what_the_table_renders() {
    reference_matches(b"playground", b"ctl-playground");
}
fn reference_matches(group_name: &[u8], page_name: &[u8]) {
    let mut group = None;
    for candidate in table::GROUPS {
        if candidate.name == group_name {
            group = Some(candidate);
            break;
        }
    }
    let group = group.expect("help group");
    let mut measured = skein_lib::Writer::new(temper_docs::PAGE_BYTES);
    crate::help::reference(group, &mut measured).expect("reference fits");
    let mut out = skein_lib::Writer::new(measured.written());
    crate::help::reference(group, &mut out).expect("reference measured");
    let rendered = out.finish();
    let page = temper_docs::find(page_name).expect("reference page");
    let write = temper_test_env::rewrite();
    if write {
        let path = std::path::Path::new("../../docs/help/reference").join(page.name).with_extension("md");
        std::fs::write(path, &rendered).expect("write group reference");
        return;
    }
    assert_eq!(page.text.as_bytes(), rendered.as_ref(), "{}: the lead updates the reference page", page.name);
}

#[test]
fn the_settings_reference_is_what_the_table_renders() {
    reference_matches(b"settings", b"ctl-settings");
}
#[test]
fn the_proposal_reference_is_what_the_table_renders() {
    reference_matches(b"proposal", b"ctl-proposal");
}

#[test]
fn the_goal_reference_is_what_the_table_renders() {
    reference_matches(b"goal", b"ctl-goal");
    reference_matches(b"role", b"ctl-role");
}

#[test]
fn the_revision_reference_is_what_the_table_renders() {
    reference_matches(b"revision", b"ctl-revision");
}

#[test]
fn settings_help_explains_check_reserve_recovery_through_held_revisions() {
    let page = temper_docs::find(b"settings").expect("settings help");
    for guidance in [
        "check timeout",
        "reserve_time",
        "settings show charter 2 --revision 13",
        "60000000000",
        "revision advance 13 --key KEY",
        "moved and stayed",
        "live run keeps",
        "equality is sufficient",
    ] {
        assert!(page.text.contains(guidance), "settings help must explain: {guidance}");
    }
    let fields = temper_docs::find(b"settings-fields").expect("settings fields help");
    assert!(fields.text.contains("current or held charter"));
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "the help test collapses whitespace to check the person's words"
)]
fn task_answer_help_says_a_person_task_may_ask_for_less() {
    let page = temper_docs::find(b"ctl-task").expect("task reference");
    let text: String = page.text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(text.contains("a person task may ask for less"));
    let help = fixtures::help(Some(jig::rows::ANSWER_AT));
    assert!(skein_lib::bytes::find(&help, b"a person task may ask for less").is_some());
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "the help test collapses whitespace to check the answer form's words"
)]
fn task_answer_help_distinguishes_choices_from_words() {
    let sentence = b"CHOICE answers only a task with choices; a task taking words is answered with TEXT alone.";
    let page = temper_docs::find(b"ctl-task").expect("task reference");
    let text: String = page.text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(skein_lib::bytes::find(text.as_bytes(), sentence).is_some());
    let help = fixtures::help(Some(jig::rows::ANSWER_AT));
    assert!(skein_lib::bytes::find(&help, sentence).is_some());
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "the help test collapses whitespace to check the key rule's words"
)]
fn ctl_help_says_what_a_key_is() {
    let page = temper_docs::find(b"ctl").expect("ctl page");
    let text: String = page.text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(text.contains("A key is 1 to 64 lower-case letters, digits or hyphens."));
}

#[test]
fn public_reading_walk_names_every_figure_and_visibility_remedy() {
    let page = temper_docs::find(b"open-public-reading").expect("public reading walk");
    assert_eq!(page.kind, temper_docs::Kind::Walk);
    assert!(temper_docs::find(b"index").unwrap().text.contains("temper help open-public-reading"));
    assert!(temper_docs::find(b"project").unwrap().text.contains("temper help open-public-reading"));
    for words in [
        "clients.public.reads",
        "clients.public.rate",
        "clients.public.rate_burst",
        "clients.public.bytes",
        "clients.public.bytes_burst",
        "clients.public.answer_bytes",
        "clients.public.max_age",
        "clients_http.public_calls",
        "core.public.goals",
        "core.public.ended",
        "core.public.lines",
        "core.public.depth",
        "engine.limits.overrides",
        "restart",
        "project amend PROJECT --visibility public --key KEY",
        "project show PROJECT",
        "/public/projects/",
        "--visibility private",
    ] {
        assert!(page.text.contains(words), "walk must name {words}");
    }
}

#[test]
fn help_names_invalid_run_end_causes() {
    let page = temper_docs::find(b"task").expect("task help");
    for cause in ["broke its contract", "form", "verdict", "words", "change", "followups", "couldn't be read"] {
        assert!(page.text.contains(cause), "task help must explain {cause}");
    }
}
