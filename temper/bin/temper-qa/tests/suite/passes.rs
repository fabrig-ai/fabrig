//! Pass recording through the binary, with ledger-created sessions (qa/ledger.md, 4.9–4.11 and 5).
use super::commands::{home, run, success};
use serde_json::{Value, json};
use skein_scratch::Scratch;
use std::fs;
use temper_qa_tool::faces::RealFiles;
use temper_qa_tool::{CaseInput, Exit, Home, Ledger};

const BUILD: &str = "0123456789abcdef0123456789abcdef01234567";

fn pass_home() -> (Scratch, Home) {
    let (scratch, home) = home();
    home.transact(&mut RealFiles, |ledger, _| {
        for (surface, page) in [("web", "work"), ("web", "home"), ("tui", "chat")] {
            let case = json!({"surface":surface,"page":page,"setup":"fresh","steps":[{"do":"open","expect":"ready"}]});
            ledger.add_case(1, &CaseInput::decode(&serde_json::to_vec(&case).expect("JSON"), "-")?, 1)?;
        }
        ledger.end_session(1, Exit::Code(0), None, 2, "qa")?;
        ledger.keep_build(BUILD, 3)?;
        ledger.set_build_holds(BUILD, vec![1], 1)?;
        let pass = ledger.start_pass("places/1", temper_qa_tool::Kind::Box, &["web".into()], "high", 4).expect("pass");
        assert_eq!((pass.pass, pass.session), (1, 2));
        Ok(())
    })
    .expect("start pass");
    (scratch, home)
}
fn finding(case: u64) -> Value {
    json!({"case":case,"title":"Start does nothing","severity":"medium","setup":"fresh",
        "lines":["web press button Start"],"expected":"a goal starts","seen":"no goal"})
}
fn ledger(home: &Home) -> Ledger {
    Ledger::decode(&fs::read(&home.ledger).expect("ledger")).expect("decode")
}
#[test]
fn findings_from_file_and_stdin_copy_evidence_inherit_the_case_and_choose_area() {
    let (_scratch, home) = pass_home();
    let configuration = home.directory.join("qa.json");
    let mut config: Value = serde_json::from_slice(&fs::read(&configuration).expect("configuration")).expect("JSON");
    config["areas"] = json!({"web":"frontend"});
    fs::write(&configuration, serde_json::to_vec(&config).expect("JSON")).expect("areas");
    fs::create_dir_all(home.directory.join("passes/1/setups/project")).expect("recipe");
    fs::write(home.directory.join("look.png"), b"pixels").expect("look");
    let mut value = finding(1);
    value["setup"] = json!("project");
    value["evidence"] = json!(["look.png"]);
    fs::write(home.directory.join("finding.json"), serde_json::to_vec(&value).expect("JSON")).expect("finding");
    success(&run(&home, &["pass", "finding", "finding.json"], Some("2"), None), b"finding 1 recorded\n");
    assert_eq!(fs::read(home.directory.join("evidence/1/look.png")).expect("kept"), b"pixels");
    let stored = ledger(&home);
    let first = &stored.findings[0];
    assert_eq!((first.pass, first.case, first.session), (1, 1, 2));
    assert_eq!(
        (&first.surface, &first.page, first.landings.as_slice()),
        (&"web".to_owned(), &"work".to_owned(), &[1][..])
    );
    assert_eq!(
        (&first.area, &first.setup, first.evidence.as_slice()),
        (&"frontend".to_owned(), &"project".to_owned(), &["look.png".to_owned()][..])
    );
    assert_eq!(
        (&first.title, &first.severity, &first.expected, &first.seen),
        (&"Start does nothing".to_owned(), &"medium".to_owned(), &"a goal starts".to_owned(), &"no goal".to_owned())
    );
    assert_eq!(first.lines, ["web press button Start"]);
}
