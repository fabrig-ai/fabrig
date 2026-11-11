//! Judge evidence and verdicts: `temper/docs/design/testing/checks.md`, 5.4.
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use serde_json::{Value, json};
use skein_scratch::Scratch;

use crate::check::{self, Outcome, Report};
use crate::evidence::{self, Evidence};
use crate::golden::{self, Golden, Item};
use crate::judge::{self, JudgeVerdict};

type EvidenceEdit = fn(&mut Evidence);

fn unedited(_items: &mut Vec<Item>) {}
fn profile(dir: &Path) {
    super::check::profile_evidence(dir, unedited);
    for (name, text) in [
        ("description.txt", include_str!("fixtures/judge/description.txt")),
        ("person/reference-code-production.md", include_str!("fixtures/judge/reference-code-production.md")),
        ("person/reference-landing.md", include_str!("fixtures/judge/reference-landing.md")),
        ("documents/people/kit/AGENTS.md", include_str!("fixtures/judge/agents.md")),
        (
            "documents/people/kit/skein/docs/foundation/programming-model.md",
            include_str!("fixtures/judge/programming-model.md"),
        ),
        ("documents/people/kit/DESIGN.md", include_str!("fixtures/judge/design.md")),
    ] {
        let path = dir.join(name);
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture directory");
        fs::write(path, text).expect("judge fixture");
    }
}
fn setup() -> (Golden, Evidence, Report) {
    let scratch = Scratch::new("stage-one-judge");
    profile(scratch.path());
    let golden = golden::reference().expect("reference");
    let evidence = evidence::read(scratch.path()).expect("evidence");
    let report = check::check(&golden, &golden::profile().expect("profile"), &evidence);
    (golden, evidence, report)
}
fn whole(golden: &Golden) -> Value {
    json!({
        "model":"judge-model",
        "rules":golden.rules.iter().map(|r| json!({"rule":r.id,"verdict":"reaches","where":"charter instructions","quote":"the rule's words"})).collect::<Vec<_>>(),
        "description":{"verdict":"matches","differences":[]},
        "restart":{"verdict":"said","quote":"The box already supplies the models and repository."}
    })
}
fn decode(value: &Value, golden: &Golden) -> JudgeVerdict {
    judge::decode_verdict(&serde_json::to_vec(value).expect("JSON"), golden).expect("verdict")
}
fn rule_mut(value: &mut Value) -> &mut Value {
    value["rules"]
        .as_array_mut()
        .expect("rules")
        .iter_mut()
        .find(|r| r["rule"] == "coding.push-early")
        .expect("coding rule")
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "cli C4: SKEIN_CODEGEN_WRITE, through temper's test table once it exists"
)]
fn the_brief_for_the_profile_matches_its_golden() {
    let (golden, evidence, report) = setup();
    let brief = judge::brief(&golden, &evidence, &report).expect("brief");
    let path = Path::new("src/tests/fixtures/judge/brief.md");
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some() {
        fs::write(path, &brief).expect("write brief golden");
    }
    assert_eq!(brief, fs::read_to_string(path).expect("brief golden"));
}

#[test]
fn a_brief_names_what_it_lacks() {
    let (golden, evidence, report) = setup();
    let cases: [(&str, EvidenceEdit); 4] = [
        ("description.txt", |e| e.description = None),
        ("documents/people/kit/AGENTS.md", |e| {
            e.documents.remove("people/kit/AGENTS.md");
        }),
        ("person/reference-code-production.md", |e| {
            e.person.remove("reference-code-production.md");
        }),
        ("person/reference-landing.md", |e| {
            e.person.remove("reference-landing.md");
        }),
    ];
    for (path, edit) in cases {
        let mut evidence = evidence.clone();
        edit(&mut evidence);
        let error = judge::brief(&golden, &evidence, &report).expect_err("missing evidence");
        assert_eq!(error.path, path);
    }
}

#[test]
fn a_section_is_cut_at_its_heading() {
    let (golden, evidence, report) = setup();
    let brief = judge::brief(&golden, &evidence, &report).expect("brief");
    assert!(brief.contains("### Questions\n\nAsk with options and a recommendation"));
    assert!(!brief.contains("This section is outside the named heading."));
    assert!(!brief.contains("Use explicit states."));
    assert!(brief.contains(include_str!("fixtures/judge/design.md")));
}

#[test]
fn headings_inside_fences_do_not_cut_a_section() {
    let (golden, mut evidence, report) = setup();
    evidence.documents.insert(
        "people/kit/AGENTS.md".into(),
        "## 1. In one page\n```markdown\n## Example\n```\nKeep this.\n## Next\nOmit this.\n".into(),
    );
    let brief = judge::brief(&golden, &evidence, &report).expect("brief");
    assert!(brief.contains("## Example\n```\nKeep this."));
    assert!(!brief.contains("Omit this."));
}

#[test]
fn a_missing_heading_names_its_document() {
    let (golden, mut evidence, report) = setup();
    evidence.documents.insert("people/kit/AGENTS.md".into(), "## A different section\ntext\n".into());
    let error = judge::brief(&golden, &evidence, &report).expect_err("missing heading");
    assert_eq!(error.path, "documents/people/kit/AGENTS.md");
    assert_eq!(error.reason, "missing heading \"1. In one page\"");
}

#[test]
fn a_verdict_decodes() {
    let golden = golden::reference().expect("reference");
    let value = whole(&golden);
    let verdict = decode(&value, &golden);
    assert_eq!(serde_json::to_value(verdict).expect("JSON"), value);
}

#[test]
fn unknown_members_are_refused_at_every_level() {
    let golden = golden::reference().expect("reference");
    for pointer in ["", "/rules/0", "/description", "/restart"] {
        let mut value = whole(&golden);
        value
            .pointer_mut(pointer)
            .expect("object path")
            .as_object_mut()
            .expect("object")
            .insert("unexpected".into(), json!(true));
        let error =
            judge::decode_verdict(&serde_json::to_vec(&value).expect("JSON"), &golden).expect_err("unknown member");
        assert!(error.to_string().contains("unknown field `unexpected`"), "{error}");
    }
}

#[test]
fn a_rule_the_golden_lacks_is_refused() {
    let golden = golden::reference().expect("reference");
    let mut value = whole(&golden);
    rule_mut(&mut value)["rule"] = json!("coding.unknown");
    let error = judge::decode_verdict(&serde_json::to_vec(&value).expect("JSON"), &golden).expect_err("unknown rule");
    assert_eq!(error.to_string(), "judge.json: rule coding.unknown: unknown rule");
}

#[test]
fn a_missing_rule_is_refused() {
    let golden = golden::reference().expect("reference");
    let mut value = whole(&golden);
    value["rules"].as_array_mut().expect("rules").retain(|r| r["rule"] != "coding.push-early");
    let error = judge::decode_verdict(&serde_json::to_vec(&value).expect("JSON"), &golden).expect_err("missing rule");
    assert_eq!(error.to_string(), "judge.json: rule coding.push-early: missing rule");
}

#[test]
fn a_rule_twice_is_refused() {
    let golden = golden::reference().expect("reference");
    let mut value = whole(&golden);
    let repeated = rule_mut(&mut value).clone();
    value["rules"].as_array_mut().expect("rules").push(repeated);
    let error = judge::decode_verdict(&serde_json::to_vec(&value).expect("JSON"), &golden).expect_err("repeated rule");
    assert_eq!(error.to_string(), "judge.json: rule coding.push-early: rule listed twice");
}

#[test]
fn an_unknown_verdict_word_is_refused() {
    let golden = golden::reference().expect("reference");
    for pointer in ["/rules/0/verdict", "/description/verdict", "/restart/verdict"] {
        let mut value = whole(&golden);
        *value.pointer_mut(pointer).expect("verdict word") = json!("unknown-word");
        let error =
            judge::decode_verdict(&serde_json::to_vec(&value).expect("JSON"), &golden).expect_err("unknown verdict");
        assert!(error.to_string().contains("unknown variant `unknown-word`"), "{error}");
    }
}

#[test]
fn an_incomplete_or_ill_typed_verdict_is_refused() {
    let golden = golden::reference().expect("reference");
    for (pointer, key) in [
        ("", "model"),
        ("/rules/0", "where"),
        ("/rules/0", "quote"),
        ("/description", "differences"),
        ("/restart", "quote"),
    ] {
        let mut value = whole(&golden);
        value.pointer_mut(pointer).expect("object path").as_object_mut().expect("object").remove(key);
        let error =
            judge::decode_verdict(&serde_json::to_vec(&value).expect("JSON"), &golden).expect_err("missing member");
        assert!(error.to_string().contains(&format!("missing field `{key}`")), "{error}");
    }
    let mut value = whole(&golden);
    value["description"]["differences"] = json!([false]);
    let error = judge::decode_verdict(&serde_json::to_vec(&value).expect("JSON"), &golden).expect_err("wrong type");
    assert!(error.to_string().contains("expected a string"), "{error}");
}

#[test]
fn a_second_json_object_is_refused() {
    let golden = golden::reference().expect("reference");
    let bytes = format!("{} {{}}", whole(&golden));
    let error = judge::decode_verdict(bytes.as_bytes(), &golden).expect_err("trailing JSON");
    assert!(error.to_string().contains("trailing characters"), "{error}");
}

#[test]
fn the_stage_passes_only_whole() {
    let (golden, _, report) = setup();
    let verdict = whole(&golden);
    assert_eq!(judge::stage(&report, &decode(&verdict, &golden)), judge::Stage { passed: true, reasons: vec![] });
    for (pointer, word, reason) in [
        ("/rules/0/verdict", "missing", "rule designer.design-docs: missing"),
        ("/rules/0/verdict", "elsewhere", "rule designer.design-docs: elsewhere"),
        ("/description/verdict", "differs", "description: differs: The workflow omits triage."),
        ("/restart/verdict", "proposed", "restart: proposed: \"The box already supplies the models and repository.\""),
    ] {
        let mut value = verdict.clone();
        value["description"]["differences"] = json!(["The workflow omits triage."]);
        *value.pointer_mut(pointer).expect("verdict path") = json!(word);
        let stage = judge::stage(&report, &decode(&value, &golden));
        assert_eq!(stage, judge::Stage { passed: false, reasons: vec![reason.into()] });
    }
    let mut report = report;
    report.verdicts.first_mut().expect("check verdict").outcome = Outcome::Missing;
    assert_eq!(
        judge::stage(&report, &decode(&verdict, &golden)),
        judge::Stage { passed: false, reasons: vec!["check: failed".into()] }
    );
}

#[test]
fn verdict_main_without_a_judge_file_exits_2() {
    let scratch = Scratch::new("stage-one-judge-no-file");
    profile(scratch.path());
    assert_eq!(judge::verdict_main(&[scratch.path().display().to_string()]), ExitCode::from(2));
}

#[test]
fn verdict_main_exits_by_the_stage_outcome() {
    let scratch = Scratch::new("stage-one-judge-command");
    profile(scratch.path());
    let golden = golden::reference().expect("reference");
    for (word, status) in
        [("reaches", ExitCode::SUCCESS), ("elsewhere", ExitCode::from(1)), ("unknown-word", ExitCode::from(2))]
    {
        let mut value = whole(&golden);
        rule_mut(&mut value)["verdict"] = json!(word);
        fs::write(scratch.path().join("judge.json"), serde_json::to_vec(&value).expect("JSON")).expect("judge file");
        for flags in [vec![], vec!["--json".into()]] {
            let mut args = vec![scratch.path().display().to_string()];
            args.extend(flags);
            assert_eq!(judge::verdict_main(&args), status);
        }
    }
}

#[test]
fn judge_main_prints_the_brief_or_exits_2() {
    let scratch = Scratch::new("stage-one-judge-brief-command");
    profile(scratch.path());
    let args = [scratch.path().display().to_string()];
    assert_eq!(judge::main(&args), ExitCode::SUCCESS);
    fs::remove_file(scratch.path().join("description.txt")).expect("remove description");
    assert_eq!(judge::main(&args), ExitCode::from(2));
}

#[test]
fn commands_refuse_usage_errors() {
    for args in [vec![], vec!["--json".into()], vec!["evidence".into(), "--unknown".into()]] {
        assert_eq!(judge::main(&args), ExitCode::from(2));
        assert_eq!(judge::verdict_main(&args), ExitCode::from(2));
    }
}
