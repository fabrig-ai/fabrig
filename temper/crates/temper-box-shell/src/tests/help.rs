use crate::{Command, configure_with, help};
use std::{fs, os::unix::ffi::OsStrExt};
use temper_client_tools::jig::call::Parsed;

#[test]
fn tui_help_explains_closing_and_ending_the_sign_in() {
    let index = help::answer(&configured(&["help", "pages"])).expect("help index");
    assert!(index.contains("temper help tui"), "the index must list the TUI page");
    let page = help::answer(&configured(&["help", "tui"])).expect("TUI page");
    let text = page.split_whitespace().collect::<Vec<_>>().join(" ");
    for required in [
        "`/quit` (`/exit` works too) or Ctrl+C closes the TUI and keeps the sign-in",
        "On the sign-in screen, use Ctrl+C",
        "`q` types a character on every screen",
        "The TUI has no sign-out action",
        "Close the TUI",
        "`temper-box open DIR http://HOST:PORT`",
        "existing TUI directory",
        "engine's loopback HTTP address",
        "`$XDG_STATE_HOME/temper/tui`",
        "`$HOME/.local/state/temper/tui`",
        "choose **Sign out**",
        "reuses and ends the existing sign-in",
    ] {
        assert!(text.contains(required), "TUI help must explain {required}");
    }
    let search = help::answer(&configured(&["help", "--search", "sign out"])).expect("sign-out search");
    assert!(search.lines().any(|line| line.starts_with("tui:")), "sign out must discover the TUI page: {search}");
    let commands = temper_shell::rows::TUI.notes;
    assert!(commands.contains("`/exit` or Ctrl+C closes it without signing out"));
    assert!(commands.contains("`q` types a character on every screen"));
}

#[test]
fn box_help_configures_coding_run_tools() {
    let page = help::answer(&configured(&["help", "box"])).expect("box help");
    let (before_start, _) = page.split_once("\n## 2. ").expect("box startup step");
    let (_, recipe) = before_start.split_once("**Coding runs:**").expect("coding run tools recipe before startup");
    let text = recipe.split_whitespace().collect::<Vec<_>>().join(" ");
    for required in [
        "`CONFIG/agent.json`",
        "preserve the rest of the document",
        "exactly the configured environment",
        "deployment-specific absolute paths",
        "Install",
        "`cargo`",
        "`.cargo/bin`",
        "usable `HOME`",
        "credentials",
        "restart the box",
    ] {
        assert!(text.contains(required), "coding run tools recipe missing {required}");
    }
    let example = recipe.split("```json\n").nth(1).expect("command environment example");
    let example: serde_json::Value =
        serde_json::from_str(example.split("```").next().expect("JSON fence")).expect("valid environment JSON");
    let mut document = coding_run_agent_document();
    document["environment"] = example["environment"].clone();
    let configured = smith_agent_shell::config::from_value(document).expect("documented command environment");
    let entries = configured.service.environment.command_entries();
    for expected in
        [b"PATH=/home/coder/.cargo/bin:/usr/local/bin:/usr/bin:/bin".as_slice(), b"HOME=/home/coder".as_slice()]
    {
        assert!(entries.iter().any(|entry| entry.as_ref() == expected), "missing command entry {expected:?}");
    }
}

#[test]
fn server_walk_gives_the_agents_environment_as_entries() {
    let page = help::answer(&configured(&["help", "server"])).expect("server walk");
    let (_, steps) = page.split_once("\n## 5. ").expect("step 5");
    let (step, _) = steps.split_once("\n## 6. ").expect("step 6");
    let text = step.split_whitespace().collect::<Vec<_>>().join(" ");
    for required in ["`agent.json`'s `environment` is a list of entries", "not a map", "temper help box"] {
        assert!(text.contains(required), "step 5 must explain {required}");
    }
    let (_, recipe) = step.split_once("`environment`").expect("agent environment");
    let example = recipe.split('`').nth(1).expect("environment entries example");
    let environment: serde_json::Value = serde_json::from_str(example).expect("valid environment JSON");
    let mut document = coding_run_agent_document();
    document["environment"] = environment.clone();
    let configured = smith_agent_shell::config::from_value(document.clone()).expect("documented entries parse");
    assert_eq!(
        configured.service.environment.command_entries().as_ref(),
        [Box::from(b"PATH=/usr/bin:/bin".as_slice())]
    );
    let mut map = serde_json::Map::new();
    for entry in environment.as_array().expect("environment entries") {
        map.insert(entry["name"].as_str().expect("entry name").into(), entry["value"].clone());
    }
    document["environment"] = map.into();
    let why = smith_agent_shell::config::from_value(document).err().expect("environment map is refused").detail();
    assert!(why.contains("invalid type: map, expected a sequence"), "{why}");
}

#[test]
fn server_walk_says_when_a_forges_sign_in_meta_appears() {
    let page = help::answer(&configured(&["help", "server"])).expect("server walk");
    let (_, steps) = page.split_once("\n## 3. ").expect("step 3");
    let (step, _) = steps.split_once("\n## 4. ").expect("step 4");
    let text = step.split_whitespace().collect::<Vec<_>>().join(" ");
    let line = crate::host::note_line(
        "temper-engine",
        temper_service::Note::ForgeSignIn {
            number: 1,
            client: b"forgejo".as_slice().into(),
            why: temper_service::clients::forge_sign_in::Unserved::NotKept,
        },
    );
    assert!(text.contains(line.trim_end_matches('\n')), "step 3 must give the printed diagnostic: {line}");
    assert!(text.contains("With `application.sign_in.forges` on a new store, `serve`'s stderr says"));
    assert!(text.contains("until step 6 connects it"));
    let (_, steps) = page.split_once("\n## 6. ").expect("step 6");
    let (step, _) = steps.split_once("\n## 7. ").expect("step 7");
    let text = step.split_whitespace().collect::<Vec<_>>().join(" ");
    for required in [
        "The page's `temper-forge-sign-in` meta appears only once `forge connect` has kept the forge.",
        "If a page fetched after that still lacks the meta, restart the engine once:",
        "a start makes it from the kept record.",
    ] {
        assert!(text.contains(required), "step 6 must explain {required}");
    }
}

fn coding_run_agent_document() -> serde_json::Value {
    let port = |number| core::num::NonZeroU16::new(number).expect("fixed nonzero port");
    let facts = temper_init::Facts {
        sign_in: false,
        binary: b"/opt/temper/bin/temper-box",
        git: Some(b"/usr/bin/git"),
        claude: Some(b"/opt/temper/bin/claude"),
        ports: temper_init::Ports {
            clients: port(8367),
            owned: port(8368),
            forge: port(8369),
            llm: port(8370),
            webhooks: port(8371),
        },
        login: Some(b"coder"),
    };
    let directories = temper_init::Directories { config: b"/etc/temper", data: b"/var/lib/temper" };
    let answer = temper_init::files(&facts, &directories, &[0; 32], &[1; 32]).expect("init files");
    let file = answer.files.iter().find(|file| file.name == b"agent.json").expect("init agent document");
    let temper_init::Content::Bytes { bytes, .. } = &file.content else { panic!("agent document bytes") };
    let mut document: serde_json::Value = serde_json::from_slice(bytes).expect("init agent JSON");
    // Inert endpoints keep configuration parsing independent of DNS, trust files and services.
    for endpoint in document["endpoints"].as_array_mut().expect("init endpoints") {
        endpoint["address"] = "127.0.0.1:9".into();
        endpoint["transport"] = "plaintext".into();
        endpoint.as_object_mut().expect("endpoint object").remove("server_name");
    }
    document
}

#[test]
fn box_step_seven_names_every_ending_and_the_direct_report() {
    let page = help::answer(&configured(&["help", "box"])).expect("box help");
    let (_, steps) = page.split_once("\n## 7. ").expect("step 7");
    let (step, _) = steps.split_once("\n## 8. ").expect("step 8");
    let text = step.split_whitespace().collect::<Vec<_>>().join(" ");
    for words in [
        "`not met: ended`",
        "`not met: ended: cancelled`",
        "`not met: ended: failed`",
        "completed, no change",
        "`temper ctl DIR task show CHANGE` prints its report",
    ] {
        assert!(text.contains(words), "step 7 must explain {words}");
    }
    assert!(!text.contains("its tree says why"));
}

#[test]
fn charter_help_explains_documents_and_host_discovery() {
    let page = help::answer(&configured(&["help", "charter"])).expect("charter help");
    let text = page.split_whitespace().collect::<Vec<_>>().join(" ");
    for words in [
        "documents",
        "temper ctl DIR forge list",
        "temper ctl DIR forge list --json",
        "host",
        "forge number",
        "adopted repository",
        "next run",
        "next heading of its level or above",
    ] {
        assert!(text.contains(words), "charter help must explain {words}");
    }
    let example = page.split("```json\n").nth(1).expect("document example");
    let example: serde_json::Value =
        serde_json::from_str(example.split("```").next().expect("example JSON")).expect("valid document example");
    assert_eq!(example["documents"][0]["forge"], "git.example:3000");
    assert_eq!(example["documents"][0]["repository"], "work/hello");
    assert_eq!(example["documents"][0]["branch"], "main");
    assert_eq!(example["documents"][0]["path"], "AGENTS.md");
    assert_eq!(example["documents"][0]["heading"], "1. In one page");
    assert!(text.contains("whole file"));
}

#[test]
fn configure_example_preserves_stock_landing_gate() {
    use settings_form::view::{Setting, SettingEdit, SettingValue};
    let page = temper_docs::find(b"configure").expect("configure walk");
    let example = page.text.split("```json\n").nth(1).expect("landing settings example");
    let example = example.split("```").next().expect("JSON fence");
    let items = settings_form::file::decode(example.as_bytes(), &temper_client_tools::limits::form())
        .expect("documented settings file");
    assert_eq!(items.len(), 2, "one landing rule and one forge charter");
    let item = items
        .iter()
        .find(|item| item.setting == Setting::Application { kind: 2, scope: 1, number: 1 })
        .expect("project 1 landing rule 1");
    let SettingEdit::Set(set) = &item.edit else { panic!("example sets the rule") };
    let SettingValue::Application(bytes) = &set.value else { panic!("landing rule value") };
    let documented = temper_engine_smith::kinds::decode(
        2,
        bytes,
        &temper_engine_smith::kinds::Bounds { segments: 9, segment_bytes: 4096, gates: 64, approvals: 64 },
    )
    .expect("documented landing rule");
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    let temper = temper_service::limits::profile(temper_service::limits::Profile::Small);
    let limits = temper_service::limits::engine(&jig, &temper).expect("stock limits");
    let stock = temper_service::stock::forge(&limits);
    let rule = stock.iter().find(|item| (item.kind, item.scope, item.number) == (2, 0, 1)).expect("stock rule");
    assert_eq!(documented.value, rule.value.value, "example must preserve every stock landing requirement");
}

fn tester_stock() -> jig_domain_core::settings::Defaults {
    let owned: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../docs/examples/owned.json")).expect("owned configuration");
    let bytes = serde_json::to_vec(&owned["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16384, depth: 32 },
    )
    .expect("engine document");
    let engine = jig_protocol_config::engine(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        jig_protocol_config::Defaults::Passed,
    )
    .expect("engine configuration");
    temper_service::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes)
}

#[test]
fn tester_recipe_copies_a_stock_charter_without_landing_tools() {
    use jig_domain_tasks as tasks;
    use temper_engine_domain as engine;

    let stock = tester_stock();
    let coding = stock.charters.iter().find(|charter| charter.number == 2).expect("stock coding charter");
    assert_eq!(coding.policy.tools.0, 514, "the recipe keeps coding charter 2's tools");
    assert_eq!(coding.policy.tools.0 & jig_domain_core::ToolFamilies::JIG.delegate.0, 0);
    let assignment = engine::Assignment {
        check_timeout: None,
        task: 1,
        attempt: 1,
        conversation: 1,
        live: jig_domain_host::LiveAsk::NONE,
        charter: 3,
        run: Box::new(engine::RunCharter {
            revision: 1,
            ended_by: tasks::EndedBy::Result,
            sources: core::iter::once(None).chain(coding.policy.alternatives.iter().map(|_| None)).collect(),
            accounts: Box::new([]),
            placement: jig_domain_fleet::PlacementConstraint {
                channel: skein_lib::Token::new(1),
                kind: jig_domain_fleet::HostKind::Worker,
            },
            policy: coding.policy.clone(),
            contract: tasks::Contract::Report { words: 16000 },
            authority: tasks::Authority {
                // Even a creator offering every family cannot add these declarations.
                tools: tasks::Tools(u64::MAX),
                models: tasks::Models::NONE,
                grants: Box::new([]),
                delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
                budget: tasks::Budget { spend: 1000, deadline: None },
                notes: tasks::Scopes(0),
                note_resources: Box::new([]),
            },
            budget: 1000,
        }),
        sections: Box::new([]),
        messages: Box::new([]),
        saved: Box::new([]),
        workstream: 1,
        workspace: engine::ForgeWorkspace { repositories: Box::new([]) },
        transcript: Box::new([]),
        answered: Box::new([]),
        settled: Box::new([]),
        grants: Box::new([]),
    };
    // charter calls tools_for; the report/workspace gate is already pinned by
    // temper-engine-smith's a_report_run_cannot_modify_without_delivery.
    let charter = temper_engine_smith::charter(&assignment);
    for name in [b"change".as_slice(), b"goal", b"delegate"] {
        assert!(!charter.grants.host_tools.iter().any(|tool| tool.name.as_ref() == name), "not declared: {name:?}");
    }
    for name in [b"effect_forge".as_slice(), b"propose"] {
        assert!(charter.grants.host_tools.iter().any(|tool| tool.name.as_ref() == name), "still declared: {name:?}");
    }
}

#[test]
fn tester_recipe_template_reports_without_a_goal_branch() {
    use settings_form::view::{
        Setting, SettingEdit, SettingNumber, SettingValue, TaskContractDocument, TemplateExecutor,
    };

    let page = temper_docs::find(b"tester-that-cannot-land").expect("tester recipe");
    let example = page.text.split("```json\n").nth(1).expect("tester template");
    let example = example.split("```").next().expect("JSON fence");
    let items = settings_form::file::decode(example.as_bytes(), &temper_client_tools::limits::form())
        .expect("documented settings file");
    assert_eq!(items.len(), 1);
    let item = items.as_slice().first().expect("template item");
    assert_eq!(item.setting, Setting::Template(SettingNumber { number: 3 }));
    let SettingEdit::Set(set) = &item.edit else { panic!("example sets the template") };
    let SettingValue::Template(template) = &set.value else { panic!("template value") };
    assert_eq!(template.number, 3);
    assert_eq!(template.executor, TemplateExecutor::Charter(3), "the recipe's copied charter");
    assert_eq!(template.contract, TaskContractDocument::Report { words: 16000 });
    assert!(template.resources.is_empty(), "no goal branch");
    assert_eq!(template.flags.ended_by, settings_form::view::EndedBy::Result);
    assert_eq!(template.tracked, Some(1));
    let stock = tester_stock();
    assert!(stock.charters.iter().all(|charter| charter.number != 3), "free charter number");
    assert!(stock.templates.iter().all(|template| template.number != 3), "free template number");
}

#[test]
fn review_recipe_changes_only_approvals() {
    let page = temper_docs::find(b"review-before-landing").expect("review recipe");
    let command = page
        .text
        .lines()
        .find(|line| line.starts_with("temper ctl DIR settings show ") && line.contains(" | jq "))
        .expect("read and edit the whole rule");
    let expression = command
        .strip_prefix("temper ctl DIR settings show landing-rule 1 --project PROJECT --json | jq '")
        .and_then(|line| line.strip_suffix("' > FILE"))
        .expect("read project rule 1 as JSON");
    let approvals = expression
        .strip_prefix("{items: [{setting: .setting, set: (.value | .approvals = ")
        .and_then(|value| value.strip_suffix(")}]}"))
        .expect("preserve the selector and whole value, assigning only approvals");
    let approvals: serde_json::Value = serde_json::from_str(approvals).expect("one approval");
    assert_eq!(approvals, serde_json::json!([{"role": 1, "people": 1, "freshness": "exact"}]));
}

#[test]
fn policy_help_allows_other_settings_beside_a_set() {
    for name in [b"configure".as_slice(), b"settings-fields"] {
        let page = temper_docs::find(name).expect("policy help");
        let text = page.text.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(text.contains("only `set`"), "{name:?}: policy edit operation");
        assert!(text.contains("at most once per file"), "{name:?}: policy item count");
        assert!(text.contains("may share a file with other settings"), "{name:?}: combined settings decision");
        assert!(!text.contains("set alone"), "{name:?}: ambiguous operation wording");
    }
}

#[test]
fn box_walk_distinguishes_stock_policy_from_open_budgets() {
    let page = temper_docs::find(b"box").expect("box walk");
    let text = page.text.split_whitespace().collect::<Vec<_>>().join(" ");
    for words in [
        "Policy 1 sizes future projects",
        "project-policy",
        "immediately resizes the open period and parties' pools",
        "temper help configure",
    ] {
        assert!(text.contains(words), "missing budget guidance: {words}");
    }
    assert!(!text.contains("Settings never resize budgets"));
    let configure = temper_docs::find(b"configure").expect("configure walk");
    let text = configure.text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(text.contains("project-policy `period_spend` 8300000 beside rules `period_spend` 8400000"));
    assert!(text.contains("open period's budget is 8300000"));
}

#[test]
fn box_walk_creates_a_project_before_it_is_funded() {
    let page = temper_docs::find(b"box").expect("box walk");
    let (_, steps) = page.text.split_once("\n## 4. ").expect("step 4");
    let (step, _) = steps.split_once("\n## 5. ").expect("step 5");
    assert!(
        fenced_lines(step)
            .iter()
            .any(|(_, line)| { *line == "temper ctl DIR forge setup --org work --repository hello --key KEY" })
    );
    let text = step.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(text.contains("Use the printed `PROJECT` number in later commands"));
    assert!(text.contains("`--project PROJECT` to choose an existing project"));
    let qa = include_str!("../../../../docs/development/qa.md");
    let setup = qa.find("temper ctl DIR forge setup --org work --repository hello --key setup").expect("QA setup");
    let funding = qa.find("temper ctl DIR project fund PROJECT PERIOD").expect("QA period funding");
    assert!(setup < funding, "QA must create a project before funding it");
    assert!(!qa.contains("first start already makes project 1"));
}

fn fenced_lines(text: &str) -> Vec<(usize, &str)> {
    let mut fenced = false;
    let mut lines = Vec::new();
    for (at, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        } else if fenced {
            lines.push((at + 1, line));
        }
    }
    lines
}

// Only whole unquoted shell separator words end a command example.
fn command_words(line: &str) -> Result<Vec<String>, &'static str> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut quoted = false;
    let mut quote = None;
    for ch in line.chars() {
        if let Some(delimiter) = quote {
            if ch == delimiter {
                quote = None;
            } else {
                word.push(ch);
            }
        } else if ch == '#' && !started {
            return Ok(words);
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
            quoted = true;
            started = true;
        } else if ch.is_ascii_whitespace() {
            if started {
                if !quoted && is_command_cut(&word) {
                    return Ok(words);
                }
                words.push(substitute_placeholders(&word));
                word.clear();
                started = false;
                quoted = false;
            }
        } else {
            word.push(ch);
            started = true;
        }
    }
    if quote.is_some() {
        return Err("unclosed quote");
    }
    if started && (quoted || !is_command_cut(&word)) {
        words.push(substitute_placeholders(&word));
    }
    Ok(words)
}

fn is_command_cut(word: &str) -> bool {
    matches!(word, "|" | "||" | ">" | ">>" | "<" | "&&" | ";") || word.starts_with('#')
}

fn substitute_placeholders(word: &str) -> String {
    let mut result = String::new();
    let mut start = 0;
    while start < word.len() {
        let end =
            word[start..].find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).map_or(word.len(), |at| start + at);
        let run = &word[start..end];
        if !run.is_empty()
            && run.as_bytes()[0].is_ascii_uppercase()
            && run.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
        {
            result.push_str(if run == "PROPOSAL" { "t1/1" } else { "1" });
        } else {
            result.push_str(run);
        }
        if end == word.len() {
            break;
        }
        let separator = word[end..].chars().next().expect("separator");
        result.push(separator);
        start = end + separator.len_utf8();
    }
    result
}

#[test]
fn box_walk_reads_the_conversation_after_waiting_for_a_reply() {
    let page = temper_docs::find(b"box").expect("box walk");
    let (_, steps) = page.text.split_once("\n## 6. ").expect("step 6");
    let (step, _) = steps.split_once("\n## 7. ").expect("step 7");
    let commands = fenced_lines(step);
    assert!(
        commands.windows(2).any(|pair| {
            pair[0].1 == "temper ctl DIR task show TASK --until replied" && pair[1].1 == "temper ctl DIR chat show TASK"
        }),
        "box step 6 must read the conversation immediately after waiting for a reply"
    );
}

#[test]
fn box_help_creates_the_first_project_before_using_its_number() {
    let page = help::answer(&configured(&["help", "box"])).expect("box help");
    let (_, sign_in) = page.split_once("\n## 3. ").expect("sign-in step");
    let (sign_in, setup) = sign_in.split_once("\n## 4. ").expect("setup step");
    let (setup, _) = setup.split_once("\n## 5. ").expect("account step");
    let sign_in = sign_in.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(sign_in.contains("`me` lists no project yet"), "fresh sign-in must have no project: {sign_in}");
    let commands = fenced_lines(setup);
    let primary = commands.first().expect("primary setup command").1;
    assert!(primary.starts_with("temper ctl DIR forge setup --org work --repository hello "), "{primary}");
    assert!(!primary.contains("--project"), "first setup must create its project: {primary}");
    for (_, command) in commands {
        if !command.starts_with("temper ctl ") {
            continue;
        }
        let words = command_words(command).expect("setup command words");
        let arguments = words[2..].iter().map(String::as_bytes).collect::<Vec<_>>();
        assert!(matches!(crate::ctl::parse(&arguments), Ok((_, Parsed::Call { .. }))), "{command}");
    }
    let setup = setup.split_whitespace().collect::<Vec<_>>().join(" ");
    for guidance in [
        "When `me` lists no project, leave out `--project`",
        "creates a project named `hello`",
        "`project: PROJECT · hello`",
        "Use the printed `PROJECT` number in later commands",
        "`--project PROJECT` to choose an existing project",
        "`--playground` without `--project` creates `widget`",
        "`project: 1 · widget` on a new box",
    ] {
        assert!(setup.contains(guidance), "setup must explain {guidance}: {setup}");
    }
}

// The deployment dispatchers are private. Reach their own table first; shared
// rows use the box parser, while serve takes only the resolved directories.
fn deployment_page_command(table: &skein_shell::Table, arguments: &[Box<[u8]>]) -> Result<(), String> {
    use skein_shell::{Reach, Reached};
    let environment = [(Box::from(b"HOME".as_slice()), Box::from(b"/home/test".as_slice()))];
    match table.reach(table.binary.as_bytes(), arguments) {
        Reached::Unknown(why) => Err(why),
        Reached::Help | Reached::Version | Reached::CommandHelp(_) => Ok(()),
        Reached::Command(command, rest) => {
            if command.reach == Reach::Words(&["serve"]) {
                assert_eq!(command.forms, ["[DIRECTORIES]"]);
                let (_, rest) =
                    temper_shell::directories::resolve(rest, &environment, b"/").map_err(|why| why.to_string())?;
                if rest.is_empty() { Ok(()) } else { Err("invalid arguments".into()) }
            } else {
                let shared = crate::TABLE.commands.iter().find(|row| row.reach == command.reach);
                assert_eq!(shared, Some(command), "a shared row must have the same arguments");
                configure_with(arguments, &environment, b"/").map(|_| ())
            }
        }
    }
}

fn page_command(line: &str) -> Result<Option<&'static str>, String> {
    if !matches!(
        line.split_ascii_whitespace().next(),
        Some("temper" | "temper-box" | "temper-engine" | "temper-worker")
    ) {
        return Ok(None);
    }
    let parse = || {
        let words = command_words(line).map_err(String::from)?;
        let words = words.iter().map(String::as_str).collect::<Vec<_>>();
        let environment = [(Box::from(b"HOME".as_slice()), Box::from(b"/home/test".as_slice()))];
        match words.as_slice() {
            ["temper", "ctl", arguments @ ..] => {
                let arguments = arguments.iter().map(|word| word.as_bytes()).collect::<Vec<_>>();
                match crate::ctl::parse(&arguments) {
                    Ok((_, Parsed::Call { .. } | Parsed::File { .. })) => Ok(Some("ctl")),
                    Err(why) => Err(why),
                }
            }
            ["temper-box", arguments @ ..] | ["temper", "box", arguments @ ..] => {
                let arguments = arguments.iter().map(|word| Box::from(word.as_bytes())).collect::<Vec<_>>();
                configure_with(&arguments, &environment, b"/")?;
                Ok(Some(if words[0] == "temper-box" { "box" } else { "front" }))
            }
            [binary @ ("temper-engine" | "temper-worker"), arguments @ ..] => {
                let (table, kind) = if *binary == "temper-engine" {
                    (&temper_engine_shell::ENGINE, "engine")
                } else {
                    (&temper_worker_shell::WORKER, "worker")
                };
                let arguments = arguments.iter().map(|word| Box::from(word.as_bytes())).collect::<Vec<_>>();
                deployment_page_command(table, &arguments)?;
                Ok(Some(kind))
            }
            ["temper"] => Ok(None),
            // The ADDRESS placeholder becomes "1" in command_words.
            ["temper", address]
                if *address == "1" || address.starts_with("http://") || address.starts_with("https://") =>
            {
                Ok(None)
            }
            _ => Err("not a documented binary command".into()),
        }
    };
    parse().map_err(|why| format!("{line}: {why}"))
}

#[test]
fn every_command_line_in_the_pages_parses() {
    let mut counts = std::collections::BTreeMap::new();
    let mut failures = Vec::new();
    for page in &temper_docs::PAGES {
        for (at, line) in fenced_lines(page.text) {
            match page_command(line) {
                Ok(Some(kind)) => *counts.entry(kind).or_insert(0) += 1,
                Ok(None) => {}
                Err(why) => failures.push(format!("{}:{at}: {why}", page.name)),
            }
        }
    }
    assert!(failures.is_empty(), "{} command lines refused:\n{}", failures.len(), failures.join("\n"));
    for kind in ["ctl", "box", "front", "engine", "worker"] {
        assert!(counts.get(kind).is_some_and(|count| *count > 0), "no {kind} lines checked: {counts:?}");
    }
    println!("checked command lines: {counts:?}");
}

#[test]
fn page_commands_use_each_binarys_table_and_take_its_arguments() {
    for (line, expected) in [
        ("temper ctl DIR me", Some("ctl")),
        ("temper-box serve --config CONFIG", Some("box")),
        ("temper box --config CONFIG", Some("front")),
        ("temper-engine forge fetch --config CONFIG --data DATA --cache CACHE 1", Some("engine")),
        ("temper-worker program-login --config CONFIG claude", Some("worker")),
        ("temper-engine serve --config CONFIG --data DATA --cache CACHE", Some("engine")),
        ("temper-worker serve --config CONFIG --data DATA --cache CACHE", Some("worker")),
        ("curl https://example.com", None),
    ] {
        assert_eq!(page_command(line), Ok(expected), "{line}");
    }
    for line in [
        "temper-worker forge fetch --config CONFIG --data DATA --cache CACHE 1",
        "temper-engine program-login --config CONFIG claude",
        "temper-engine forge fetch --config CONFIG",
        "temper-worker program-login --config CONFIG",
        "temper-engine serve --config CONFIG extra",
        "temper-worker serve --box",
        "temper-worker serve --config CONFIG --config CONFIG",
    ] {
        let why = page_command(line).expect_err(line);
        assert!(why.contains(line), "refusal must name the line: {why}");
    }
}

#[test]
fn server_walk_runs_only_the_deployments_binaries() {
    let page = temper_docs::find(b"server").expect("server walk");
    for (at, line) in fenced_lines(page.text) {
        assert_ne!(line.split_ascii_whitespace().next(), Some("temper-box"), "server:{at}: {line}");
    }
}

#[test]
fn page_commands_keep_quoted_arguments_and_replace_only_capital_words() {
    assert_eq!(
        command_words("temper ctl DIR task message TASK 'two words' --key KEY # comment").expect("words"),
        ["temper", "ctl", "1", "task", "message", "1", "two words", "--key", "1"]
    );
    assert_eq!(
        command_words("  temper ctl DIR proposal decide PROPOSAL accept \"two words\"").expect("words"),
        ["temper", "ctl", "1", "proposal", "decide", "t1/1", "accept", "two words"]
    );
    for ending in
        ["# comment", "#don't parse quotes", "| cat", "|| cat", "> file", ">> file", "< file", "&& next", "; next"]
    {
        assert_eq!(command_words(&format!("temper help {ending}")).expect("words"), ["temper", "help"]);
    }
    assert_eq!(substitute_placeholders("connected:FORGE/taskTASK/N/_TASK/A2"), "connected:1/taskTASK/1/_TASK/1");
}

#[test]
fn only_whole_unquoted_separator_words_cut_a_page_command() {
    assert_eq!(
        command_words("temper ctl DIR forge connect 2 github-app 12345 <slug>.private-key.pem --key github-app")
            .expect("words"),
        [
            "temper",
            "ctl",
            "1",
            "forge",
            "connect",
            "2",
            "github-app",
            "12345",
            "<slug>.private-key.pem",
            "--key",
            "github-app"
        ]
    );
    assert_eq!(
        command_words("temper help '<' \"|\" '#' '&&' '||' '>' '>>' ';'").expect("quoted separators"),
        ["temper", "help", "<", "|", "#", "&&", "||", ">", ">>", ";"]
    );
    assert_eq!(
        command_words("temper help a|b >file <file a;b").expect("argument words"),
        ["temper", "help", "a|b", ">file", "<file", "a;b"]
    );
    assert_eq!(command_words("temper help 'unclosed"), Err("unclosed quote"));
}

fn configured(words: &[&str]) -> help::Request {
    let words = words.iter().map(|word| Box::from(word.as_bytes())).collect::<Vec<_>>();
    let Command::Help { request } = configure_with(&words, &[], b"/unused").expect("help arguments") else {
        panic!("help command")
    };
    request
}

#[test]
fn help_prints_the_page_and_index_before_init_with_no_environment_or_configuration() {
    let directory = std::env::temp_dir().join(format!("temper-help-{}", std::process::id()));
    fs::create_dir(&directory).expect("empty working directory");
    assert_eq!(fs::read_dir(&directory).expect("working directory").count(), 0);
    let command = configure_with(&[Box::from(b"help".as_slice())], &[], directory.as_os_str().as_bytes())
        .expect("help before init");
    assert_eq!(command, Command::Overview);
    assert_eq!(temper_shell::table::help(&crate::TABLE, &[]).expect("table help"), crate::TABLE.help());
    let answer = help::answer(&configured(&["help", "pages"])).expect("index page");
    assert_eq!(answer, temper_docs::find(b"index").expect("index page").text);
    assert_eq!(fs::read_dir(&directory).expect("still empty working directory").count(), 0);
    fs::remove_dir(directory).expect("remove empty working directory");
}

#[test]
fn help_prints_a_whole_page_and_names_an_unknown_one() {
    assert_eq!(
        help::answer(&configured(&["help", "ctl"])).expect("ctl page").as_bytes(),
        include_bytes!("../../../../docs/help/ctl.md")
    );
    assert_eq!(
        help::answer(&configured(&["help", "nope"])),
        Err("temper-box: no page nope; temper-box help lists them\n".into())
    );
}

#[test]
fn account_and_deployment_help_explain_offering_another_model() {
    let account = help::answer(&configured(&["help", "account"])).expect("account help");
    let account = account.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(account.contains("temper help deployment"), "{account}");
    assert!(account.contains("Offering another model"), "{account}");
    assert!(account.contains("The providers are the configuration's and the models are settings"), "{account}");
    let deployment = help::answer(&configured(&["help", "deployment"])).expect("deployment help");
    assert!(!deployment.contains("```json"), "model declarations belong in settings: {deployment}");
    let guidance = deployment
        .split("## Acting on it\n")
        .nth(1)
        .expect("deployment procedure")
        .split("\n## ")
        .next()
        .expect("procedure text")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for required in [
        "Offering another model",
        "**Deployment**",
        "**Models**",
        "Add a model by name",
        "choose a provider",
        "model's name",
        "Prices are estimated until set",
        "first run shows whether the provider serves it",
        "Ask temper",
        "prices, window or efforts",
        "A task keeps the model it has",
        "no restart",
        "local model",
        "`endpoint` in `engine.llm.providers`",
        "`CONFIG/temper.json`",
        "`CONFIG/agent.json`",
        "claude-haiku-5-5",
        "output",
        "reasoning_item",
        "`head`",
        "`idle`",
        "Preserve",
        "stop",
        "restart, then change the model",
        "same `CONFIG`, `DATA` and `CACHE`",
    ] {
        assert!(guidance.contains(required), "deployment help missing {required}: {guidance}");
    }
    for words in [vec!["help", "--search", "claude-haiku-5-5"], vec!["help", "--search", "model", "restart"]] {
        let hits = help::answer(&configured(&words)).expect("model configuration search");
        assert!(hits.lines().any(|line| line.starts_with("deployment: Acting on it: ")), "{hits}");
    }
}

#[test]
fn box_and_tier_help_name_the_host_runners_node_prerequisite() {
    for (page, heading) in [("box", "Before you start"), ("tier", "Acting on it")] {
        let text = help::answer(&configured(&["help", page])).expect("help page");
        let section = text
            .split(&format!("## {heading}\n"))
            .nth(1)
            .expect("prerequisite section")
            .split("\n## ")
            .next()
            .expect("section text")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        for prerequisite in [
            "`temper-host`",
            "host",
            "compatible Node.js",
            "`node`",
            "JavaScript `uses:` actions",
            "`actions/checkout@v4`",
            "`actions/upload-artifact@v3`",
            "`application.machine.owned.runner.path`",
            "`/usr/local/bin:/usr/bin:/bin`",
            "restart",
            "interactive shell is insufficient",
        ] {
            assert!(section.contains(prerequisite), "{page}: missing {prerequisite}: {section}");
        }
    }
}

#[test]
fn box_help_names_delivery_checks_path_and_tools() {
    let text = help::answer(&configured(&["help", "box"])).expect("box help");
    let prerequisites = text
        .split("## Before you start\n")
        .nth(1)
        .expect("prerequisites")
        .split("\n## ")
        .next()
        .expect("prerequisite section")
        .split("- **Host delivery checks:**")
        .nth(1)
        .expect("host delivery checks prerequisite")
        .split("\n- ")
        .next()
        .expect("delivery checks text")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for required in [
        "`.temper/pre-pr`",
        "inherit the box process's `PATH` captured at startup",
        "box unit's `PATH`",
        "before step 2",
        "fabrig",
        "`cargo`",
        "rustfmt",
        "`cargo fmt`",
        "`python3`",
        "`W/bin:/usr/local/bin:/usr/bin:/bin`",
        "`PATH=/home/box/.cargo/bin:W/bin:/usr/local/bin:/usr/bin:/bin`",
        "absolute paths",
        "restart the box unit after changing its environment",
        "`application.machine.owned.runner.path` configures the runner alone",
    ] {
        assert!(prerequisites.contains(required), "delivery checks missing {required}: {prerequisites}");
    }
}

#[test]
fn box_and_tier_help_explain_shared_capacity_and_existing_boxes() {
    for page in ["box", "tier"] {
        let text = help::answer(&configured(&["help", page])).expect("help page");
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        for required in [
            "CI and tiers",
            "`temper-host`",
            "`application.machine.owned.runner.capacity`",
            "defaults to 1",
            "2 for one single-job tier plus CI, or 3 for two such tiers plus CI",
            "multi-job workflows",
            "other repositories",
            "Ctrl-C or SIGTERM",
            "wait for it to exit",
            "`CONFIG/temper.json`",
            "same",
            "registration and secrets",
        ] {
            assert!(text.contains(required), "{page}: missing {required}: {text}");
        }
    }
}
#[test]
fn box_and_change_help_locate_temper_review_gates_in_change_show() {
    let box_page = help::answer(&configured(&["help", "box"])).expect("box page");
    let step = box_page
        .split("## 7. Follow the change to its landing\n")
        .nth(1)
        .expect("box step 7")
        .split("\n## ")
        .next()
        .expect("step text")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(!step.contains("diff, reviews"), "{step}");
    assert!(step.contains("The pull request printed by `change show` has its diff, CI runs and logs;"), "{step}");
    assert!(step.contains("temper's review gate is read with `change show`."), "{step}");

    let change = help::answer(&configured(&["help", "change"])).expect("change page");
    let change = change.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(change.contains("they run in temper; `change show` prints each"), "{change}");
    assert!(change.contains("Temper posts none as forge reviews or statuses"), "{change}");
}

#[test]
fn tier_help_explains_the_baseline_before_any_landing() {
    let page = help::answer(&configured(&["help", "tier"])).expect("tier page");
    let section = page
        .split("## What it is\n")
        .nth(1)
        .expect("what it is")
        .split("\n## ")
        .next()
        .expect("section text")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for explanation in [
        "A newly configured tier first runs at its followed branch's current tip as a baseline, without any landing.",
        "Baseline failures are blamed on no landing and inherited by the next run.",
        "Subsequent ranges start after the baseline tip.",
        "Landings arriving during the baseline wait for the next run to verify them.",
    ] {
        assert!(section.contains(explanation), "tier help must explain: {explanation}\n{section}");
    }
}

#[test]
fn help_prints_the_landing_page() {
    assert_eq!(
        help::answer(&configured(&["help", "landing"])).expect("landing page").as_bytes(),
        include_bytes!("../../../../docs/help/landing.md")
    );
}

#[test]
fn tier_help_contains_a_complete_landing_configuration_example() {
    let text = help::answer(&configured(&["help", "tier"])).expect("tier page");
    let example = fenced_lines(&text).into_iter().map(|(_, line)| line).collect::<Vec<_>>().join("\n");
    let example: serde_json::Value = serde_json::from_str(&example).expect("tier configuration JSON");
    let landing = &example["application"]["landing"];
    assert!(landing.get("deployment").is_none());
    let projects = landing["projects"].as_array().expect("landing projects");
    assert_eq!(projects.len(), 1);
    let project = &projects[0];
    assert_eq!(project["project"], 1);
    assert!(project.get("rules").is_none());
    assert_eq!(project["by"], "queue");
    assert_eq!(project["check_timeout_ms"], 900_000);
    assert_eq!(project["tier_retries"], 2);
    assert_eq!(project["verification_bound_ms"], 7_200_000);
    let repositories = project["gates"].as_array().expect("project repository entries");
    assert_eq!(repositories.len(), 1);
    let repository = &repositories[0];
    assert_eq!(repository["forge"], 1);
    assert_eq!(repository["repository"], 7);
    assert!(repository["gates"].is_array(), "repository workflow gates");
    let tiers = repository["tiers"].as_array().expect("repository tiers");
    assert_eq!(tiers.len(), 1);
    let tier = &tiers[0];
    assert_eq!(tier["name"], "browser");
    assert_eq!(tier["workflow"], "browser.yml");
    assert_eq!(tier["artifact"], "report");
    assert_eq!(tier["format"], "junit");
    assert_eq!(tier["ignored"], serde_json::json!(["docs"]));
}

#[test]
fn tier_help_explains_report_formats_and_preserving_configuration() {
    let text = help::answer(&configured(&["help", "tier"])).expect("tier page");
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for instruction in [
        "CONFIG/temper.json",
        "application.landing.projects[].gates[].tiers",
        "`junit` for JUnit reports",
        "`lines` for one failing test id per line",
        "project's `gates` array holds repository entries",
        "repository's `gates` array holds workflow gates",
        "project, forge and repository numbers",
        "Preserve the existing configuration",
        "restart",
        "`tier_retries` and `verification_bound_ms` beside the project's `by`",
    ] {
        assert!(text.contains(instruction), "missing {instruction}: {text}");
    }
}

#[test]
fn tier_configuration_is_reachable_through_help_search() {
    let search = help::answer(&configured(&["help", "--search", "tiers"])).expect("tier search");
    assert!(search.lines().any(|line| line.starts_with("tier: Acting on it: ")), "{search}");
    for name in ["tier", "landing", "deployment"] {
        let text = help::answer(&configured(&["help", name])).expect("help page");
        assert_eq!(text, temper_docs::find(name.as_bytes()).expect("compiled page").text);
    }
}

#[test]
fn help_search_names_the_server_section_or_says_nothing_matches() {
    let text = help::answer(&configured(&["help", "--search", "WireGuard"])).expect("WireGuard section");
    assert!(text.lines().any(|line| line.starts_with("server: 5. The workers' host: ")), "{text}");
    assert!(text.contains("WireGuard"), "{text}");
    assert_eq!(
        help::answer(&configured(&["help", "--search", "zzzznotadocword"])),
        Err("temper-box: nothing matches\n".into())
    );
}

#[test]
fn help_search_finds_port_selection_in_init_and_the_box_walk() {
    let text =
        help::answer(&configured(&["help", "--search", "ports", "init"])).expect("help explains init's port selection");
    for section in ["box: 1. Write the configuration: ", "commands: temper-box: "] {
        assert!(text.lines().any(|line| line.starts_with(section)), "{section} missing from {text}");
    }
}

#[test]
fn help_search_joins_words_and_prints_each_search_refusal() {
    assert_eq!(
        configured(&["help", "--search", "owned", "Forgejo"]),
        help::Request::Search(Box::from(b"owned Forgejo".as_slice()))
    );
    for (words, error) in [
        (" \t", "nothing to search for"),
        ("a a a a a a a a a", "at most 8 words"),
        ("123456789012345678901234567890123", "a word is at most 32 bytes"),
    ] {
        assert_eq!(
            help::answer(&help::Request::Search(Box::from(words.as_bytes()))),
            Err(format!("temper-box: {error}\n"))
        );
    }
}

#[test]
fn help_requires_one_page_or_search_words() {
    for words in [vec!["help", "--search"], vec!["help", "a", "b"], vec!["help", "--unknown"]] {
        let words = words.iter().map(|word| Box::from(word.as_bytes())).collect::<Vec<_>>();
        assert_eq!(
            configure_with(&words, &[], b"/unused"),
            Err("temper-box help: invalid arguments\nusage: temper-box help [NAME | pages | --search WORDS...]\n"
                .into())
        );
    }
}
