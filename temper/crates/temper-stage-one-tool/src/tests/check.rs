//! Profile evidence and mutations: `temper/docs/design/testing/checks.md`, 5.4.
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use serde_json::{Value, json};
use skein_scratch::Scratch;

use crate::check::{self, BoundBy, Outcome, Report, TextName};
use crate::evidence::{self, Evidence};
use crate::golden::{self, Edit, Item, ObligationKind, SettingsFile};

const FORGE: &str = "forge.box:3000";
const REPOSITORY: &str = "people/kit";
const PROJECT: u32 = 7;
// The reference has 70 settings and 13 box obligations.
const OBLIGATIONS: usize = 77;

fn box_placeholders(value: &mut Value) {
    match value {
        Value::String(s) => {
            for (from, to) in [
                ("127.0.0.1:3000", FORGE),
                ("acme/widget", REPOSITORY),
                ("3132372e302e302e313a33303030", "666f7267652e626f783a33303030"),
                ("61636d65", "70656f706c65"),
                ("776964676574", "6b6974"),
            ] {
                if s == from {
                    *s = to.into();
                    break;
                }
            }
        }
        Value::Object(map) => {
            for v in map.values_mut() {
                box_placeholders(v);
            }
        }
        Value::Array(values) => {
            for v in values {
                box_placeholders(v);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}
fn item_json(item: &Item) -> Value {
    match &item.edit {
        Edit::Set(value) => json!({"setting": item.setting, "set": value}),
        Edit::Removed => json!({"setting": item.setting, "edit": "removed"}),
        Edit::Reset => json!({"setting": item.setting, "edit": "reset"}),
    }
}
fn write_json(dir: &Path, name: &str, value: &Value) {
    fs::write(dir.join(name), serde_json::to_vec(value).expect("JSON")).expect("write evidence");
}
/// Evidence shared with the judge's tests; the callback edits accepted items before writing.
pub(crate) fn profile_evidence(dir: &Path, edit: fn(&mut Vec<Item>)) {
    let mut items = golden::profile().expect("profile").items;
    for item in &mut items {
        if let Edit::Set(value) = &mut item.edit {
            box_placeholders(value);
        }
    }
    for (setting, bytes) in [
        (json!({"kind":"rules"}), include_bytes!("fixtures/profile/rules.json").as_slice()),
        (json!({"kind":"policy", "number":1}), include_bytes!("fixtures/profile/policy.json").as_slice()),
        (
            json!({"kind":"project-policy", "project":PROJECT}),
            include_bytes!("fixtures/profile/project-policy.json").as_slice(),
        ),
        (
            json!({"kind":"project-ways", "project":PROJECT}),
            br#"{"drafts":true,"goals":true,"changes":true,"way":4}"#.as_slice(),
        ),
    ] {
        let value: Value = serde_json::from_slice(bytes).expect("authority fixture");
        let file = golden::decode_settings(
            &serde_json::to_vec(&json!({"items":[{"setting":setting,"set":value}]})).expect("JSON"),
        )
        .expect("fixture item");
        items.extend(file.items);
    }
    edit(&mut items);
    let mut current = golden::decode_settings(include_bytes!("fixtures/profile/stock.json")).expect("stock").items;
    for item in &items {
        current.retain(|stock| stock.setting != item.setting);
        if matches!(item.edit, Edit::Set(_)) {
            current.push(item.clone());
        }
    }
    write_json(dir, "items.json", &json!({"items":items.iter().map(item_json).collect::<Vec<_>>()}));
    write_json(dir, "current.json", &json!({"items":current.iter().map(item_json).collect::<Vec<_>>()}));
    write_json(
        dir,
        "box.json",
        &json!({"forge":FORGE,"repository":REPOSITORY,"project":PROJECT,"revision":1,"proposal":"t1/1","chat":1}),
    );
    write_json(dir, "revision.json", &json!({}));
    let confirmations: Vec<_> = golden::reference()
        .expect("reference")
        .obligations
        .iter()
        .filter(|o| matches!(o.kind, ObligationKind::Box(_)))
        .map(|o| json!({"id":o.id,"present":true,"seen":"confirmed by the pass"}))
        .collect();
    write_json(dir, "confirmed.json", &json!({"box":confirmations}));
    fs::write(dir.join("proposal.txt"), "Apply these settings from the process documents.\n").expect("proposal");
    fs::write(dir.join("chat.txt"), "The box already supplies the models and repository.\n").expect("chat");
}
fn unedited(_items: &mut Vec<Item>) {}

#[test]
fn project_ways_and_questions_follow_the_golden_only_when_accepted() {
    for accepted in [vec![0], vec![1], vec![0, 1], vec![]] {
        let (recorded, settings) = super::collect::project_settings_outputs(PROJECT, &accepted, &[0, 1]);
        let (items, current) = crate::collect::assemble(&recorded).expect("project settings evidence");
        let mut evidence = proposed(unedited);
        for item in &settings {
            evidence.items.items.retain(|prior| prior.setting != item.setting);
            evidence.current.items.retain(|prior| prior.setting != item.setting);
        }
        evidence.items.items.extend(items.items.clone());
        evidence.current.items.extend(current.items.into_iter().filter(|item| item.setting.kind != "project-policy"));
        for (at, item) in settings.iter().enumerate() {
            assert_eq!(evidence.items.items.contains(item), accepted.contains(&at));
            assert!(evidence.current.items.contains(item));
        }
        let collected: Vec<_> = evidence.items.items.iter().filter(|item| settings.contains(item)).cloned().collect();
        assert_eq!(collected, items.items);
        let result = report(&evidence);
        assert!(result.extras.is_empty(), "{}", result.text());
        let expected = if accepted.contains(&1) { Outcome::Met } else { Outcome::Missing };
        assert_eq!(outcome(&result, "project.way"), &expected);
        assert_eq!(result.passed(), accepted.contains(&1), "{}", result.text());
        let mut baseline = report(&proposed(unedited));
        baseline.verdicts.iter_mut().find(|v| v.id == "project.way").expect("project way").outcome = expected;
        assert_eq!(result.verdicts, baseline.verdicts);
    }
}

fn proposed(edit: fn(&mut Vec<Item>)) -> Evidence {
    let scratch = Scratch::new("stage-one-check");
    profile_evidence(scratch.path(), edit);
    evidence::read(scratch.path()).expect("profile evidence")
}
fn report(evidence: &Evidence) -> Report {
    check::check(&golden::reference().expect("reference"), &golden::profile().expect("profile"), evidence)
}
fn outcome<'a>(report: &'a Report, id: &str) -> &'a Outcome {
    &report.verdicts.iter().find(|v| v.id == id).expect("verdict").outcome
}
fn set_value<'a>(items: &'a mut [Item], kind: &str, number: u32) -> &'a mut Value {
    let item =
        items.iter_mut().find(|item| item.setting.kind == kind && item.setting.number == Some(number)).expect("item");
    let Edit::Set(value) = &mut item.edit else { panic!("set item") };
    value
}
fn change_ci(items: &mut [Item]) {
    set_value(items, "landing-rule", 2)["ci"] = json!(true);
}

#[test]
fn the_profile_as_proposed_passes() {
    let report = report(&proposed(unedited));
    assert!(report.passed(), "{}", report.text());
    assert_eq!(report.verdicts.len(), OBLIGATIONS);
    assert_eq!(report.extras.len(), 0);
    let bindings: Vec<_> = report.binding.iter().map(|b| (b.role.as_str(), b.number, b.by)).collect();
    assert_eq!(
        bindings,
        vec![
            ("coordinator", Some(5), BoundBy::Anchor),
            ("designer", Some(7), BoundBy::Assignment),
            ("coding", Some(2), BoundBy::Anchor),
            ("triage", Some(8), BoundBy::Assignment),
            ("goal-template", Some(4), BoundBy::Anchor),
            ("small-goal-template", Some(2), BoundBy::Anchor),
            ("main-rule", Some(1), BoundBy::Anchor),
            ("goal-rule", Some(2), BoundBy::Anchor),
        ]
    );
    assert_eq!(outcome(&report, "coordinator.instructions"), &Outcome::Judged);
    assert_eq!(outcome(&report, "small-goal-template.stock"), &Outcome::Unchanged);
    assert_eq!(report.json()["passed"], true);
    assert_eq!(report.json()["verdicts"].as_array().expect("verdicts").len(), OBLIGATIONS);
    assert!(report.text().starts_with("coordinator: charter 5, by its anchor\n"));
    assert!(report.text().ends_with("passed\n"));
}
#[test]
fn the_goal_template_requires_design_and_no_close_out() {
    for (member, wrong, required) in [("design", false, true), ("close_out", true, false)] {
        let mut value = json!({"design":true,"close_out":false});
        value[member] = json!(wrong);
        for edit in [Edit::Set(value), Edit::Removed, Edit::Reset] {
            let mut evidence = proposed(unedited);
            for file in [&mut evidence.items, &mut evidence.current] {
                let item = file.items.iter_mut().find(|item| item.setting.kind == "template-goal").expect("goal work");
                item.edit = edit.clone();
            }
            let result = report(&evidence);
            assert!(!result.passed(), "{}", result.text());
            let expected = match edit {
                Edit::Set(_) => Outcome::Wrong { found: json!(wrong), expected: json!(required) },
                Edit::Removed | Edit::Reset => Outcome::Missing,
            };
            assert_eq!(outcome(&result, &format!("goal-template.{member}")), &expected);
        }
    }
}

fn renumber(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (name, child) in map {
                if name == "charter" {
                    match child.as_u64() {
                        Some(5) => *child = json!(11),
                        Some(7) => *child = json!(12),
                        Some(8) => *child = json!(13),
                        _ => {}
                    }
                } else {
                    renumber(child);
                }
            }
        }
        Value::Array(values) => {
            for child in values {
                renumber(child);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}
#[test]
fn renumbered_charters_still_pass() {
    let report = report(&proposed(|items| {
        for item in items {
            if item.setting.kind == "charter" {
                let number = match item.setting.number {
                    Some(5) => Some(11),
                    Some(7) => Some(12),
                    Some(8) => Some(13),
                    n => n,
                };
                item.setting.number = number;
                if let Edit::Set(value) = &mut item.edit {
                    value["number"] = json!(number);
                }
            }
            if let Edit::Set(value) = &mut item.edit {
                renumber(value);
            }
        }
    }));
    assert!(report.passed(), "{}", report.text());
    for (role, number) in [("coordinator", 11), ("designer", 12), ("triage", 13)] {
        assert_eq!(report.binding.iter().find(|b| b.role == role).expect("role").number, Some(number));
    }
}
#[test]
fn a_value_changed_is_wrong() {
    let report = report(&proposed(|items| change_ci(items)));
    assert!(!report.passed());
    assert_eq!(outcome(&report, "goal-rule.ci"), &Outcome::Wrong { found: json!(true), expected: json!(false) });
    assert!(report.text().contains("goal-rule.ci  wrong: found true, expected false\n"));
    assert!(report.text().ends_with("failed: 1 of 77\n"));
    let json = report.json();
    let verdict =
        json["verdicts"].as_array().expect("verdicts").iter().find(|v| v["id"] == "goal-rule.ci").expect("CI verdict");
    assert_eq!(verdict["outcome"], json!({"kind":"wrong","found":true,"expected":false}));
}
#[test]
fn an_added_charter_is_an_extra() {
    let report = report(&proposed(|items| {
        let mut item =
            items.iter().find(|i| i.setting.kind == "charter" && i.setting.number == Some(8)).expect("triage").clone();
        item.setting.number = Some(9);
        if let Edit::Set(value) = &mut item.edit {
            value["number"] = json!(9);
        }
        items.push(item);
    }));
    assert!(!report.passed());
    assert_eq!(report.extras.len(), 1);
    assert_eq!(report.extras[0].setting.number, Some(9));
    assert_eq!(report.extras[0].why, "no obligation asks for it");
    assert!(report.text().contains("extra: charter 9: no obligation asks for it\n"));
}
#[test]
fn a_role_removed_is_missing() {
    let report =
        report(&proposed(|items| items.retain(|i| !(i.setting.kind == "charter" && i.setting.number == Some(8)))));
    assert!(!report.passed());
    assert_eq!(report.binding.iter().find(|b| b.role == "triage").expect("triage").by, BoundBy::Unbound);
    for id in [
        "triage.instructions",
        "triage.model",
        "triage.inspect",
        "triage.modify",
        "triage.shell",
        "triage.agents",
        "rules.delegation",
    ] {
        assert_eq!(outcome(&report, id), &Outcome::Missing, "{id}");
    }
}
#[test]
fn the_stock_rule_left_fails_only() {
    let mut evidence = proposed(unedited);
    let mut item = evidence.current.items.iter().find(|i| i.setting.kind == "landing-rule").expect("rule").clone();
    item.setting.number = Some(3);
    if let Edit::Set(value) = &mut item.edit {
        value["pattern"]["last"] = json!({"open":""});
    }
    evidence.current.items.push(item.clone());
    let report = report(&evidence);
    assert!(!report.passed());
    assert_eq!(
        outcome(&report, "landing-rules.only"),
        &Outcome::Wrong { found: json!([item.setting]), expected: json!("only bound roles") }
    );
    assert_eq!(report.extras.len(), 0);
}
#[test]
fn the_small_goal_template_set_is_wrong() {
    let report = report(&proposed(|items| {
        items.push(Item {
            setting: golden::SettingKey { kind: "template".into(), number: Some(2), project: None },
            edit: Edit::Set(json!({"changed":true})),
        });
    }));
    assert!(!report.passed());
    assert_eq!(
        outcome(&report, "small-goal-template.stock"),
        &Outcome::Wrong { found: json!({"set":{"changed":true}}), expected: json!("unchanged") }
    );
}
#[test]
fn a_box_obligation_lacking_fails() {
    let mut evidence = proposed(unedited);
    let confirmation = evidence.confirmed.iter_mut().find(|c| c.id == "box.project").expect("confirmation");
    confirmation.present = false;
    confirmation.seen = "repository not adopted".into();
    let report = report(&evidence);
    assert!(!report.passed());
    assert_eq!(outcome(&report, "box.project"), &Outcome::Lacking { seen: "repository not adopted".into() });
}
#[test]
fn a_gap_named_and_unnamed() {
    let golden = golden::decode(br#"{"golden":"test","placeholders":{"forge":"host","repository":"owner/repository"},"roles":[],"obligations":[{"id":"gap.test","row":"a gap","kind":"gap","status":"test status, 1","names":[["cannot","model"],["not","supported"]]}],"rules":[]}"#).expect("gap golden");
    let mut evidence = proposed(unedited);
    evidence.items.items.clear();
    evidence.chat = "We CANNOT yet MODEL this field.".into();
    let profile = SettingsFile { items: vec![] };
    let named = check::check(&golden, &profile, &evidence);
    assert!(named.passed());
    assert_eq!(outcome(&named, "gap.test"), &Outcome::Named { text: TextName::Chat });
    evidence.chat.clear();
    let unnamed = check::check(&golden, &profile, &evidence);
    assert!(!unnamed.passed());
    assert_eq!(outcome(&unnamed, "gap.test"), &Outcome::Unnamed);
}
#[test]
fn run_exits_by_outcome() {
    let scratch = Scratch::new("stage-one-command");
    profile_evidence(scratch.path(), unedited);
    let dir = scratch.path().to_str().expect("path").to_owned();
    assert_eq!(check::run(&[dir.clone(), "--json".into()]), Ok(true));
    assert_eq!(check::main(std::slice::from_ref(&dir)), ExitCode::SUCCESS);
    profile_evidence(scratch.path(), |items| change_ci(items));
    assert_eq!(check::run(std::slice::from_ref(&dir)), Ok(false));
    assert_eq!(check::main(std::slice::from_ref(&dir)), ExitCode::from(1));
    let missing = scratch.path().join("absent").to_str().expect("path").to_owned();
    assert!(check::run(std::slice::from_ref(&missing)).expect_err("unreadable directory").contains("box.json"));
    assert_eq!(check::main(&[missing]), ExitCode::from(2));
    for args in [vec![], vec![dir.clone(), "--unknown".into()], vec!["--json".into(), dir]] {
        assert!(check::run(&args).expect_err("usage").starts_with("temper-stage-one check: "));
    }
}

#[test]
fn integer_bounds_and_permitted_surpluses_pass() {
    let report = report(&proposed(|items| {
        let coding = set_value(items, "charter", 2);
        coding["turns"] = json!(801);
        coding["time"] = json!(28_800_000_000_001_u64);
        let coordinator = set_value(items, "charter", 5);
        coordinator["documents"]
            .as_array_mut()
            .expect("documents")
            .push(json!({"forge":FORGE,"repository":REPOSITORY,"branch":"main","path":"extra.md"}));
        for item in items {
            if ["rules", "policy", "project-policy"].contains(&item.setting.kind.as_str()) {
                let Edit::Set(value) = &mut item.edit else { panic!("authority") };
                value["ceiling"]["tools"] = json!(u64::MAX);
                value["ceiling"]["delegation"]["kinds"].as_array_mut().expect("kinds").push(json!({"job":7}));
            }
        }
    }));
    assert!(report.passed(), "{}", report.text());
}
#[test]
fn wrong_types_and_values_fail_comparisons() {
    let original = proposed(unedited);
    for (kind, number, pointer, found, id, expected) in [
        ("charter", 2, "/turns", json!(16), "coding.turns", json!(800)),
        ("charter", 2, "/turns", json!("800"), "coding.turns", json!(800)),
        ("charter", 2, "/turns", json!(800.0), "coding.turns", json!(800)),
        ("charter", 2, "/turns", json!(-1), "coding.turns", json!(800)),
        ("charter", 2, "/time", json!(120_000_000_000_u64), "coding.time", json!(28_800_000_000_000_u64)),
        ("template", 4, "/contract/report", Value::Null, "goal-template.report", json!("non-null")),
        ("charter", 5, "/model/name", json!(true), "coordinator.model", json!("claude-opus-5-5")),
    ] {
        let mut evidence = original.clone();
        *set_value(&mut evidence.current.items, kind, number).pointer_mut(pointer).expect("member") = found.clone();
        let report = report(&evidence);
        assert!(!report.passed(), "{id}");
        assert_eq!(outcome(&report, id), &Outcome::Wrong { found, expected }, "{id}");
    }
}
#[test]
fn a_missing_member_is_missing() {
    let mut evidence = proposed(unedited);
    set_value(&mut evidence.current.items, "charter", 5)["model"].as_object_mut().expect("model").remove("name");
    let report = report(&evidence);
    assert_eq!(outcome(&report, "coordinator.model"), &Outcome::Missing);
    assert!(!report.passed());
}
#[test]
fn judged_text_must_be_a_nonempty_string() {
    let original = proposed(unedited);
    for found in [json!(""), Value::Null, json!(4), json!([])] {
        let mut evidence = original.clone();
        set_value(&mut evidence.current.items, "charter", 5)["instructions"] = found;
        let report = report(&evidence);
        assert_eq!(outcome(&report, "coordinator.instructions"), &Outcome::Missing);
    }
}
#[test]
fn a_role_with_the_effect_family_or_without_the_project_note_scope_is_wrong() {
    let report = report(&proposed(|items| {
        for item in items {
            if item.setting.kind == "project-policy" {
                let Edit::Set(value) = &mut item.edit else { panic!("policy") };
                value["roles"][0]["authority"]["tools"] = json!(2047);
                value["roles"][0]["authority"]["notes"] = json!(1);
            }
        }
    }));
    assert_eq!(outcome(&report, "project.role0.tools"), &Outcome::Met);
    assert!(matches!(outcome(&report, "project.role0.effect"), Outcome::Wrong { .. }));
    assert!(matches!(outcome(&report, "project.role0.notes"), Outcome::Wrong { .. }));
}
#[test]
fn a_mask_short_of_a_bit_or_of_the_wrong_type_is_wrong() {
    let original = proposed(unedited);
    for found in [json!(0), json!("1983")] {
        let mut evidence = original.clone();
        let item =
            evidence.current.items.iter_mut().find(|i| i.setting.kind == "project-policy").expect("project's policy");
        let Edit::Set(value) = &mut item.edit else { panic!("policy") };
        value["roles"][0]["authority"]["tools"] = found.clone();
        let report = report(&evidence);
        assert_eq!(outcome(&report, "project.role0.tools"), &Outcome::Wrong { found, expected: json!(1983) });
    }
}
#[test]
fn policy_one_may_be_set_or_left_as_the_stock_has_it() {
    let set = report(&proposed(unedited));
    assert!(set.passed(), "{}", set.text());
    assert!(set.extras.is_empty(), "{}", set.text());
    let left = report(&proposed(|items| items.retain(|item| item.setting.kind != "policy")));
    assert!(left.passed(), "{}", left.text());
}
#[test]
fn a_way_must_be_the_goal_template_for_drafts() {
    let report = report(&proposed(|items| {
        for item in items {
            if item.setting.kind == "project-ways" {
                let Edit::Set(value) = &mut item.edit else { panic!("ways") };
                value["way"] = json!(2);
            }
        }
    }));
    assert_eq!(outcome(&report, "project.way"), &Outcome::Wrong { found: json!(2), expected: json!(4) });
}
#[test]
fn a_projects_questions_are_free_and_a_project_with_no_ways_is_missing() {
    let report = report(&proposed(|items| {
        items.retain(|item| item.setting.kind != "project-ways");
        let setting = golden::decode_settings(
            &serde_json::to_vec(
                &json!({"items":[{"setting":{"kind":"project-questions","project":PROJECT},"set":"up"}]}),
            )
            .expect("JSON"),
        )
        .expect("item");
        items.extend(setting.items);
    }));
    assert!(report.extras.is_empty(), "{}", report.text());
    assert_eq!(outcome(&report, "project.way"), &Outcome::Missing);
}
#[test]
fn a_documents_heading_is_free_unless_the_golden_asks_it_whole() {
    let narrowed = report(&proposed(|items| {
        let designer = set_value(items, "charter", 7);
        let documents = designer["documents"].as_array_mut().expect("documents");
        let document = documents.iter_mut().find(|d| d["path"] == "DESIGN.md").expect("document");
        document["heading"] = json!("One section");
    }));
    assert!(matches!(outcome(&narrowed, "designer.documents"), Outcome::Wrong { .. }));
    let report = report(&proposed(|items| {
        for (number, path, heading) in [
            (5, "AGENTS.md", json!(null)),
            (5, "skein/docs/foundation/programming-model.md", json!("Any")),
            (7, "DESIGN.md", json!(null)),
        ] {
            let charter = set_value(items, "charter", number);
            let documents = charter["documents"].as_array_mut().expect("documents");
            let document = documents.iter_mut().find(|d| d["path"] == path).expect("document");
            document["heading"] = heading;
        }
    }));
    assert_eq!(outcome(&report, "coordinator.documents"), &Outcome::Met);
    assert_eq!(outcome(&report, "designer.documents"), &Outcome::Met);
}
#[test]
fn contains_requires_a_list_holding_each_element() {
    let original = proposed(unedited);
    for found in [
        json!({}),
        json!([]),
        json!([{"forge":FORGE,"repository":REPOSITORY,"branch":"main","path":"OTHER.md"}]),
        json!([{"forge":FORGE,"repository":REPOSITORY,"branch":"other","path":"DESIGN.md"}]),
    ] {
        let mut evidence = original.clone();
        set_value(&mut evidence.current.items, "charter", 7)["documents"] = found;
        let report = report(&evidence);
        assert!(matches!(outcome(&report, "designer.documents"), Outcome::Wrong { .. }));
    }
}
#[test]
fn absent_box_confirmation_is_lacking() {
    let mut evidence = proposed(unedited);
    evidence.confirmed.retain(|c| c.id != "box.project");
    let report = report(&evidence);
    assert_eq!(outcome(&report, "box.project"), &Outcome::Lacking { seen: "not confirmed".into() });
}
#[test]
fn removed_and_reset_stock_edits_are_wrong() {
    for edit in [Edit::Removed, Edit::Reset] {
        let mut evidence = proposed(unedited);
        evidence.items.items.push(Item {
            setting: golden::SettingKey { kind: "forge-charter".into(), number: Some(2), project: None },
            edit,
        });
        let report = report(&evidence);
        assert!(
            matches!(outcome(&report, "forge-charter.reviews"), Outcome::Wrong { expected, .. } if expected == "unchanged")
        );
    }
}
#[test]
fn project_landing_rules_can_bind_the_roles() {
    let report = report(&proposed(|items| {
        for item in items {
            if item.setting.kind == "landing-rule" {
                item.setting.project = Some(golden::ProjectRef::Number(PROJECT));
            }
        }
    }));
    assert!(report.passed(), "{}", report.text());
}
#[test]
fn a_goal_rule_open_beneath_its_segment_binds() {
    let report = report(&proposed(|items| {
        let rule = set_value(items, "landing-rule", 2);
        rule["pattern"]["segments"].as_array_mut().expect("segments").push(json!("74656d706572"));
        rule["pattern"]["last"] = json!({"open":""});
        rule["enforce"] = json!(false);
    }));
    assert!(report.passed(), "{}", report.text());
    let other = self::report(&proposed(|items| {
        let rule = set_value(items, "landing-rule", 2);
        rule["pattern"]["segments"].as_array_mut().expect("segments").push(json!("6f74686572"));
        rule["pattern"]["last"] = json!({"open":""});
    }));
    assert_eq!(outcome(&other, "goal-rule.pattern"), &Outcome::Missing);
}
#[test]
fn the_stock_rule_removed_with_the_roles_on_the_project_is_no_extra() {
    let mut evidence = proposed(|items| {
        for item in items.iter_mut().filter(|item| item.setting.kind == "landing-rule") {
            item.setting.project = Some(golden::ProjectRef::Number(PROJECT));
        }
    });
    let mut removed = evidence.items.items.iter().find(|i| i.setting.kind == "landing-rule").expect("rule").clone();
    removed.setting.project = None;
    removed.edit = Edit::Removed;
    evidence.items.items.push(removed);
    let report = report(&evidence);
    assert!(report.extras.is_empty(), "{}", report.text());
    assert!(report.passed(), "{}", report.text());
}
#[test]
fn only_checks_both_landing_rule_scopes() {
    let mut evidence = proposed(unedited);
    let mut extra = evidence.current.items.iter().find(|i| i.setting.kind == "landing-rule").expect("rule").clone();
    extra.setting.project = Some(golden::ProjectRef::Number(PROJECT));
    evidence.current.items.push(extra.clone());
    let report = report(&evidence);
    assert_eq!(
        outcome(&report, "landing-rules.only"),
        &Outcome::Wrong { found: json!([extra.setting]), expected: json!("only bound roles") }
    );
    assert_eq!(report.extras.len(), 0);
}
#[test]
fn another_projects_rule_cannot_bind() {
    let mut evidence = proposed(unedited);
    evidence.current.items.retain(|i| !(i.setting.kind == "landing-rule" && i.setting.number == Some(1)));
    let mut other = evidence
        .items
        .items
        .iter()
        .find(|i| i.setting.kind == "landing-rule" && i.setting.number == Some(1))
        .expect("main rule")
        .clone();
    other.setting.project = Some(golden::ProjectRef::Number(PROJECT + 1));
    evidence.current.items.push(other);
    let report = report(&evidence);
    assert_eq!(report.binding.iter().find(|b| b.role == "main-rule").expect("main role").number, None);
    assert_eq!(outcome(&report, "main-rule.pattern"), &Outcome::Missing);
}
#[test]
fn hex_case_does_not_change_a_pattern() {
    let report = report(&proposed(|items| {
        for item in items {
            if item.setting.kind != "landing-rule" {
                continue;
            }
            let Edit::Set(value) = &mut item.edit else { panic!("rule") };
            for segment in value["pattern"]["segments"].as_array_mut().expect("segments") {
                *segment = json!(segment.as_str().expect("hex").to_uppercase());
            }
            for end in value["pattern"]["last"].as_object_mut().expect("last").values_mut() {
                *end = json!(end.as_str().expect("hex").to_uppercase());
            }
        }
    }));
    assert!(report.passed(), "{}", report.text());
}
#[test]
fn a_pattern_for_the_wrong_repository_is_wrong() {
    let report =
        report(&proposed(|items| set_value(items, "landing-rule", 2)["pattern"]["segments"][3] = json!("6f74686572")));
    assert!(!report.passed());
    let Outcome::Wrong { found, expected } = outcome(&report, "goal-rule.pattern") else {
        panic!("expected wrong pattern: {report:?}");
    };
    assert_eq!(found["segments"][3], "other");
    assert_eq!(expected["segments"][3], "kit");
    assert_eq!(expected["segments"][1], FORGE);
    assert_eq!(expected["last"]["open"], "temper");
}

#[test]
fn a_malformed_pattern_member_is_wrong() {
    let original = proposed(unedited);
    for found in [json!(42), json!("not hex"), json!("a"), json!("ff")] {
        let mut evidence = original.clone();
        set_value(&mut evidence.current.items, "landing-rule", 2)["pattern"]["segments"][3] = found;
        let report = report(&evidence);
        assert!(matches!(outcome(&report, "goal-rule.pattern"), Outcome::Wrong { .. }));
    }
}
#[test]
fn assignment_ties_follow_role_order_then_numbers() {
    let golden = golden::decode(br#"{
      "golden":"test", "placeholders":{"forge":"host","repository":"owner/repository"},
      "roles":[{"role":"triage","kind":"charter","profile":8,"anchor":null},
               {"role":"designer","kind":"charter","profile":7,"anchor":null}],
      "obligations":[{"id":"triage.instructions","row":"triage","kind":"setting","subject":{"role":"triage"},"member":"instructions","test":"judged"},
                     {"id":"designer.instructions","row":"designer","kind":"setting","subject":{"role":"designer"},"member":"instructions","test":"judged"}],
      "rules":[]
    }"#).expect("assignment golden");
    let mut evidence = proposed(unedited);
    evidence.items.items.retain(|i| i.setting.kind == "charter" && [Some(5), Some(7)].contains(&i.setting.number));
    evidence.items.items.reverse();
    let report = check::check(&golden, &golden::profile().expect("profile"), &evidence);
    assert!(report.passed(), "{}", report.text());
    let binding: Vec<_> = report.binding.iter().map(|b| (b.role.as_str(), b.number, b.by)).collect();
    assert_eq!(binding, vec![("triage", Some(5), BoundBy::Assignment), ("designer", Some(7), BoundBy::Assignment)]);
}
#[test]
fn anchors_choose_the_lowest_setting_numbers() {
    let mut evidence = proposed(unedited);
    let mut template = evidence
        .current
        .items
        .iter()
        .find(|i| i.setting.kind == "template" && i.setting.number == Some(4))
        .expect("goal template")
        .clone();
    template.setting.number = Some(3);
    evidence.current.items.push(template);
    let mut rule = evidence
        .current
        .items
        .iter()
        .find(|i| i.setting.kind == "landing-rule" && i.setting.number == Some(2))
        .expect("goal rule")
        .clone();
    rule.setting.number = Some(1);
    rule.setting.project = Some(golden::ProjectRef::Number(PROJECT));
    evidence.current.items.push(rule);
    let report = report(&evidence);
    assert_eq!(report.binding.iter().find(|b| b.role == "goal-template").expect("goal template").number, Some(3));
    assert_eq!(report.binding.iter().find(|b| b.role == "goal-rule").expect("goal rule").number, Some(1));
    assert_eq!(outcome(&report, "goal-template.executor"), &Outcome::Met);
    assert_eq!(outcome(&report, "goal-rule.pattern"), &Outcome::Met);
}
