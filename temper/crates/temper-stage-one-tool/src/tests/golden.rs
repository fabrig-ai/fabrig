//! Golden decoding and settings-form coverage: `temper/docs/design/testing/checks.md`, 5.4.
use crate::golden::{self, Edit, ObligationKind, Subject, Test};
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn source() -> Value {
    serde_json::from_slice(include_bytes!("../../golden/reference.json")).expect("golden JSON")
}
fn refused(value: &Value, path: &str) {
    let error = golden::decode(&serde_json::to_vec(value).expect("JSON")).expect_err("invalid golden");
    assert_eq!(error.path, path, "{error}");
}
#[test]
fn the_reference_golden_decodes() {
    let golden = golden::reference().expect("reference");
    assert_eq!(golden.roles.len(), 8);
    let mut counts = [0; 3];
    let mut ids = BTreeSet::new();
    for obligation in &golden.obligations {
        assert!(ids.insert(&obligation.id));
        counts[match &obligation.kind {
            ObligationKind::Setting(_) => 0,
            ObligationKind::Box(_) => 1,
            ObligationKind::Gap(_) => 2,
        }] += 1;
    }
    assert_eq!(counts, [64, 13, 0]);
    assert_eq!(golden.free, ["project-questions", "policy"]);
    assert_eq!(golden.rules.len(), 15);
    for rule in &golden.rules {
        assert!(ids.insert(&rule.id));
    }
}
#[test]
fn every_setting_member_resolves_in_the_form() {
    let golden = golden::reference().expect("reference");
    for obligation in &golden.obligations {
        let ObligationKind::Setting(setting) = &obligation.kind else { continue };
        if setting.member.is_empty() {
            continue;
        }
        let kind = match &setting.subject {
            Subject::Role(name) => golden::kind_name(golden.roles.iter().find(|r| r.role == *name).expect("role").kind),
            Subject::Setting(key) => {
                if key.kind == "project-policy" {
                    "policy"
                } else {
                    &key.kind
                }
            }
        };
        let mut path = kind.to_owned();
        for word in setting.member.split('.') {
            if word.parse::<usize>().is_ok() {
                path.push_str("[]");
            } else {
                path.push('.');
                path.push_str(word);
            }
        }
        assert!(settings_form::fields::resolve(path.as_bytes()).is_some(), "{}: {path}", obligation.id);
    }
}
#[test]
fn every_profile_value_is_found() {
    let golden = golden::reference().expect("reference");
    let profile = golden::profile().expect("profile");
    let mut count = 0;
    for obligation in &golden.obligations {
        let ObligationKind::Setting(setting) = &obligation.kind else { continue };
        if setting.value.is_some() || !matches!(setting.test, Test::Equal | Test::Contains) {
            continue;
        }
        let Subject::Role(name) = &setting.subject else { panic!("profile value needs a role") };
        let role = golden.roles.iter().find(|r| r.role == *name).expect("role");
        let item = profile
            .items
            .iter()
            .find(|item| item.setting.kind == golden::kind_name(role.kind) && item.setting.number == Some(role.profile))
            .expect("profile item");
        let Edit::Set(value) = &item.edit else { panic!("set profile value") };
        assert!(golden::member(value, &setting.member).is_some(), "{}", obligation.id);
        count += 1;
    }
    assert_eq!(count, 18);
}
#[test]
fn a_golden_is_refused_naming_the_path() {
    let mut v = source();
    v["surprise"] = json!(true);
    refused(&v, "$.surprise");
    let mut v = source();
    v["obligations"][1]["id"] = v["obligations"][0]["id"].clone();
    refused(&v, "$.obligations[1].id");
    let mut v = source();
    v["obligations"][0]["subject"]["role"] = json!("absent");
    refused(&v, "$.obligations[0].subject.role");
    let mut v = source();
    v["obligations"][16]["value"] = json!({"role":"absent"});
    refused(&v, "$.obligations[16].value.role");
    let mut v = source();
    v["obligations"][1]["member"] = json!("absent");
    refused(&v, "$.obligations[1].value");
}
#[test]
fn role_and_anchor_failures_name_the_path() {
    let mut v = source();
    v["roles"][1]["role"] = json!("coordinator");
    refused(&v, "$.roles[1].role");
    let mut v = source();
    v["roles"][0]["anchor"] = json!({"executes":"coding"});
    refused(&v, "$.roles[0].anchor.executes");
    let mut v = source();
    v["roles"][0]["anchor"] = json!({"executes":"absent"});
    refused(&v, "$.roles[0].anchor.executes");
    let mut v = source();
    v["roles"][0]["anchor"] = json!({"resource":"goal-branch"});
    refused(&v, "$.roles[0].anchor");
    let mut v = source();
    v["roles"][0]["anchor"] = json!({"branch":{"exact":"main","extra":true}});
    refused(&v, "$.roles[0].anchor.branch.extra");
    let mut v = source();
    v["rules"][0]["role"] = json!("absent");
    refused(&v, "$.rules[0].role");
    let mut v = source();
    v["obligations"][16]["value"] = json!([{"charter":{"role":"absent"}}]);
    refused(&v, "$.obligations[16].value[0].charter.role");
}
#[test]
fn the_profile_decodes_as_a_settings_file() {
    let profile = golden::profile().expect("profile");
    assert_eq!(profile.items.len(), 8);
    assert!(profile.items.iter().all(|item| matches!(item.edit, Edit::Set(_))));
}
#[test]
fn settings_envelopes_are_strict_but_values_remain_json() {
    let file = golden::decode_settings(br#"{"items":[{"setting":{"kind":"charter","number":1},"set":{"role":0,"free":null}},{"setting":{"kind":"template","number":2},"edit":"removed"},{"setting":{"kind":"rules"},"edit":"reset"}]}"#).expect("settings");
    assert_eq!(file.items[0].edit, Edit::Set(json!({"role":0,"free":null})));
    assert_eq!(file.items[1].edit, Edit::Removed);
    assert_eq!(file.items[2].edit, Edit::Reset);
    for (value, path) in [
        (json!({"items":[],"extra":1}), "$.extra"),
        (json!({"items":[{"setting":{"kind":"rules","extra":1},"set":{}}]}), "$.items[0].setting.extra"),
        (json!({"items":[{"setting":{"kind":"rules"},"set":{},"edit":"reset"}]}), "$.items[0]"),
        (
            json!({"items":[{"setting":{"kind":"rules"},"set":{}},{"setting":{"kind":"rules"},"edit":"reset"}]}),
            "$.items[1].setting",
        ),
        (json!({"items":[{"setting":{"kind":"charter"},"set":{}}]}), "$.items[0].setting"),
    ] {
        let error = golden::decode_settings(&serde_json::to_vec(&value).expect("JSON")).expect_err("invalid settings");
        assert_eq!(error.path, path);
    }
}

#[test]
fn a_current_inventory_includes_project_settings() {
    let file = golden::decode_settings(br#"{"items":[{"setting":{"kind":"rules"},"set":{}},{"setting":{"kind":"project-policy","project":7},"set":{"roles":[]}},{"setting":{"kind":"landing-rule","project":7,"number":1},"set":{}}]}"#).expect("whole current inventory");
    assert_eq!(file.items.len(), 3);
    assert_eq!(file.items[1].setting.project, Some(golden::ProjectRef::Number(7)));
    assert_eq!(file.items[2].setting.project, Some(golden::ProjectRef::Number(7)));
    assert_eq!(file.items[2].setting.number, Some(1));
}

#[test]
fn project_ways_and_questions_require_a_project_without_a_number() {
    for kind in ["project-ways", "project-questions"] {
        for project in [json!(1), json!(u32::MAX), json!("box")] {
            let setting = json!({"kind":kind,"project":project});
            let file = json!({"items":[{"setting":setting,"set":{"free":null}}]});
            let decoded = golden::decode_settings(&serde_json::to_vec(&file).expect("JSON")).expect("project key");
            assert_eq!(serde_json::to_value(&decoded.items[0].setting).expect("key JSON"), setting);
            assert_eq!(decoded.items[0].edit, Edit::Set(json!({"free":null})));
        }
        for (setting, path) in [
            (json!({"kind":kind}), "$.items[0].setting"),
            (json!({"kind":kind,"project":1,"number":1}), "$.items[0].setting"),
            (json!({"kind":kind,"project":"box","number":1}), "$.items[0].setting"),
            (json!({"kind":kind,"number":1}), "$.items[0].setting"),
            (json!({"kind":kind,"project":0}), "$.items[0].setting"),
            (json!({"kind":kind,"project":null}), "$.items[0].setting.project"),
            (json!({"kind":kind,"project":-1}), "$.items[0].setting.project"),
            (json!({"kind":kind,"project":4_294_967_296_u64}), "$.items[0].setting.project"),
            (json!({"kind":kind,"project":"1"}), "$.items[0].setting.project"),
        ] {
            let file = json!({"items":[{"setting":setting,"edit":"reset"}]});
            let error =
                golden::decode_settings(&serde_json::to_vec(&file).expect("JSON")).expect_err("invalid project key");
            assert_eq!(error.path, path, "{kind}: {error}");
        }
    }
}
