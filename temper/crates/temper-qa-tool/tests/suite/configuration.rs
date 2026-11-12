//! The deployment's strict, home-relative configuration (qa/README.md, 3).
use serde_json::json;
use std::path::Path;
use temper_qa_tool::Configuration;

fn configuration() -> serde_json::Value {
    json!({"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent","{effort}"],"triage":"low","pass":"high"},"request":{"command":["file"]}})
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

#[test]
fn pass_configuration_defaults_and_helpers_and_nested_refusals() {
    let home = Path::new("/qa/home");
    let decode = |value: &serde_json::Value| Configuration::decode(&serde_json::to_vec(value).expect("JSON"), home);
    let mut value = configuration();
    let config = decode(&value).expect("defaults");
    assert_eq!(config.agent.pass, "high");
    assert_eq!(config.bounds.pass, 7200);
    assert_eq!(config.places, ["places/1"]);
    assert_eq!(config.surfaces, ["web"]);
    assert_eq!(config.request.area, "qa");
    assert_eq!(config.request.state, None);
    assert_eq!(config.finding_area("web"), "qa");
    assert_eq!(config.place_directory(home, "places/1").expect("place"), home.join("places/1"));
    assert!(config.place_directory(home, "unknown").is_err());
    value["bounds"] = json!({"pass":10});
    value["places"] = json!(["a", "/absolute/b"]);
    value["surfaces"] = json!(["web", "tui"]);
    value["areas"] = json!({"web":"frontend"});
    value["request"] = json!({"command":["file","arg"],"state":["state"],"area":"fixes"});
    let config = decode(&value).expect("Q3 keys");
    assert_eq!(config.bounds.pass, 10);
    assert_eq!(config.finding_area("web"), "frontend");
    assert_eq!(config.finding_area("tui"), "qa");
    assert_eq!(config.place_directory(home, "/absolute/b").expect("absolute"), Path::new("/absolute/b"));
    assert_eq!(config.request.command, ["file", "arg"]);
    assert_eq!(config.request.state, Some(vec!["state".into()]));
    assert_eq!(config.request.area, "fixes");
    let before = value.clone();
    value["request"]["surprise"] = json!(1);
    assert_eq!(decode(&value).expect_err("nested unknown").to_string(), "qa.json: request: unknown field surprise");
    value = before.clone();
    value["agent"].as_object_mut().expect("agent").remove("pass");
    assert_eq!(decode(&value).expect_err("required pass").to_string(), "qa.json: agent.pass: missing pass");
    for (key, bad) in [
        ("places", json!([])),
        ("places", json!(["a", "a"])),
        ("surfaces", json!(["native"])),
        ("areas", json!({"native":"qa"})),
        ("request", json!({})),
        ("request", json!({"command":[]})),
    ] {
        value = before.clone();
        value[key] = bad;
        assert!(decode(&value).is_err(), "{key}");
    }
}
