//! The deployment's strict, home-relative configuration (qa/README.md, 3).
use serde_json::json;
use std::path::Path;
use temper_qa_tool::Configuration;

fn configuration() -> serde_json::Value {
    json!({"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent","{effort}"],"triage":"low"}})
}
#[test]
fn configuration_defaults_paths_required_keys_and_unknown_names() {
    let home = Path::new("/qa/home");
    let good = configuration();
    let decode = |value: &serde_json::Value| Configuration::decode(&serde_json::to_vec(value).expect("JSON"), home);
    let config = decode(&good).expect("defaults");
    assert_eq!(config.owner, "qa");
    assert_eq!(config.git.directory, home.join("repo"));
    assert_eq!(config.git.branch, "main");
    assert_eq!(config.git.fetch, None);
    assert_eq!(config.builds, home.join("builds"));
    assert_eq!((config.bounds.poll, config.bounds.triage, config.bounds.build_wait), (60, 900, 21600));
    assert_eq!(config.agent.triage, "low");
    assert_eq!(config.agent.command, ["agent", "{effort}"]);
    assert_eq!(config.notify, None);
    for pointer in ["", "/git", "/agent"] {
        let mut value = good.clone();
        value
            .pointer_mut(pointer)
            .expect("object")
            .as_object_mut()
            .expect("object")
            .insert("surprise".into(), json!(1));
        let refusal = decode(&value).expect_err("unknown").to_string();
        assert!(refusal.starts_with("qa.json: ") && refusal.contains("surprise"), "{refusal}");
    }
    for (pointer, key) in
        [("", "owner"), ("", "git"), ("/git", "directory"), ("", "agent"), ("/agent", "command"), ("/agent", "triage")]
    {
        let mut value = good.clone();
        value.pointer_mut(pointer).expect("object").as_object_mut().expect("object").remove(key);
        assert!(decode(&value).expect_err("required").to_string().contains(key), "{pointer}/{key}");
    }
    let mut full = good;
    full["git"] = json!({"directory":"/absolute/repo","branch":"trunk","fetch":"origin"});
    full["bounds"] = json!({"poll":3,"triage":4,"build_wait":5});
    full["builds"] = json!("kept");
    full["notify"] = json!(["notify", "area"]);
    let config = decode(&full).expect("all fields");
    assert_eq!(config.git.directory, Path::new("/absolute/repo"));
    assert_eq!(config.git.branch, "trunk");
    assert_eq!(config.git.fetch.as_deref(), Some("origin"));
    assert_eq!((config.bounds.poll, config.bounds.triage, config.bounds.build_wait), (3, 4, 5));
    assert_eq!(config.builds, home.join("kept"));
    assert_eq!(config.notify.expect("notify"), ["notify", "area"]);
    full["bounds"]["unknown"] = json!(0);
    assert!(decode(&full).expect_err("unknown bound").to_string().contains("unknown"));
}
