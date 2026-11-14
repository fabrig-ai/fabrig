//! An owner's atomic go-live revision with running work kept (domain/landing.md, 13 and 15).
use super::{Judge, Line, Process, Script, Signal, Stage, Value, binary, calls, ctl, document, json, write};
use std::path::Path;

fn narrow(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for name in ["grants", "projections"] {
                if let Some(Value::Array(grants)) = object.get_mut(name) {
                    grants.retain(|grant| grant["kind"] != 15);
                }
            }
            if let Some(Value::Array(kinds)) = object.get_mut("kinds") {
                kinds.retain(|kind| kind["job"] != 1 && !(kind["kind"] == "job" && kind["number"] == 1));
            }
            for value in object.values_mut() {
                narrow(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                narrow(value);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}
fn text(hex: &Value) -> String {
    let hex = hex.as_str().expect("hex text");
    let bytes: Vec<_> =
        (0..hex.len()).step_by(2).map(|at| u8::from_str_radix(&hex[at..at + 2], 16).expect("hex byte")).collect();
    String::from_utf8(bytes).expect("fixture text")
}
fn grant(value: &mut Value) {
    let pattern = &mut value["pattern"];
    let segments: Vec<_> = pattern["segments"].as_array().expect("segments").iter().map(text).collect();
    let (kind, last) = pattern["last"].as_object().expect("last").iter().next().expect("terminal");
    *pattern = json!({"segments":segments,"last":kind,"value":text(last)});
}
fn authority(value: &mut Value) {
    let object = value.as_object_mut().expect("authority");
    assert_eq!(object.remove("models"), Some(json!("all")), "fixture models");
    let spend = object.remove("spend").expect("spend");
    assert!(object.remove("deadline").expect("deadline").is_null());
    object.insert("budget".into(), json!({"spend":spend,"deadline_unix_seconds":null}));
    for value in object["grants"].as_array_mut().expect("grants") {
        grant(value);
    }
    for value in object["delegation"]["kinds"].as_array_mut().expect("kinds") {
        let (kind, number) = value.as_object().expect("kind").iter().next().expect("executor");
        *value = json!({"kind":kind,"number":number});
    }
}
fn seed(mut value: Value) -> Value {
    let permissions = value.as_object_mut().expect("policy").remove("permissions").expect("permissions");
    authority(&mut value["ceiling"]);
    for role in value["roles"].as_array_mut().expect("roles") {
        authority(&mut role["authority"]);
    }
    for projection in value["projections"].as_array_mut().expect("projections") {
        grant(projection);
    }
    assert_eq!(value["requirements"], json!([]));
    assert_eq!(value["templates"], json!([]));
    json!({"number":1,"roles":permissions,"policy":value})
}
pub(super) fn setup(root: &Path, steps: &mut Vec<ctl::Step>) {
    for args in [
        vec!["settings", "show", "rules", "--json"],
        vec!["settings", "show", "policy", "1", "--json"],
        vec!["settings", "show", "--json"],
    ] {
        steps.push(ctl::Step::Run(ctl::Call::new(&args).in_directory("private")));
    }
    let root = root.to_path_buf();
    steps.push(ctl::Step::Then(Box::new(move |calls| {
        let rules = ctl::json(&calls[calls.len() - 3])["value"].clone();
        let policy = ctl::json(&calls[calls.len() - 2])["value"].clone();
        let mut own = policy.clone();
        own["permissions"][0]["role"] = 2.into();
        let period = own["period_spend"].as_u64().expect("project period");
        let role = own["roles"]
            .as_array_mut()
            .expect("roles")
            .iter_mut()
            .find(|role| role["number"] == 0)
            .expect("owner role");
        role["period_spend"] = (period / 48 * super::super::landing_by_turn::ROLE_PERIOD_IN_K).into();
        assert_ne!(own["permissions"], policy["permissions"], "project roles differ from policy 1");
        let after = json!({"items":[
            {"setting":{"kind":"rules"},"set":rules},
            {"setting":{"kind":"policy","number":1},"set":policy},
            {"setting":{"kind":"project-policy","project":1},"set":own}
        ]});
        write(&root.join("go-live-after.json"), &after);
        let mut initial_rules = rules;
        let mut initial_policy = policy;
        let mut initial_own = own;
        narrow(&mut initial_rules);
        narrow(&mut initial_policy);
        narrow(&mut initial_own);
        let file = root.join("go-live-before.json");
        write(
            &file,
            &json!({"items":[
                {"setting":{"kind":"rules"},"set":initial_rules},
                {"setting":{"kind":"policy","number":1},"set":initial_policy},
                {"setting":{"kind":"project-policy","project":0},"set":initial_policy},
                {"setting":{"kind":"project-policy","project":1},"set":initial_own}
            ]}),
        );
        let path = root.join("temper.json");
        let mut config = document(&path);
        // The owner's project-policy record supplies the startup seed's three parts.
        assert_eq!(config["engine"]["projects"][0]["number"], 1, "the owned fixture's seed project");
        config["engine"]["projects"][0] = seed(initial_own);
        write(&path, &config);
        ctl::settings_change(calls.last().expect("revision"), &file, "go-live-own-project").in_directory("private")
    })));
}
pub(super) fn scripts(scripts: &mut [Script]) {
    for script in scripts.iter_mut().filter(|script| script.cue.starts_with(b"## Task\n\n@drill-goal-")) {
        let Line::Call { arguments, .. } = &mut script.turns[0].lines[0] else { panic!("delegate") };
        let mut delegate: Value = serde_json::from_slice(arguments).expect("delegate");
        narrow(&mut delegate);
        *arguments = serde_json::to_vec(&delegate).expect("narrow delegate").into();
    }
    let producer =
        scripts.iter_mut().find(|script| script.cue.starts_with(b"## Task\n\n@drill-code-1")).expect("producer");
    producer.turns[0].lines =
        [vec![super::super::batch_landing::hold(b"drill-go-live")], producer.turns[0].lines.to_vec()].concat().into();
}
impl Judge<'_> {
    pub(super) fn go_live_cut(&mut self, procs: &mut [Process]) -> Result<(), String> {
        let running=procs.iter().any(|proc|if let Process::Peer(peer)=proc {
            peer.observations().iter().any(|event|if let skein_fake_peers::llm::Observation::Query {query,..}=event {
                query.messages.iter().flat_map(|message|message.parts.iter()).any(|part|matches!(part,skein_fake_llm_domain::api::Part::Text {text} if skein_lib::bytes::find(text,b"@drill-code-1").is_some()))
            } else {false})
        } else {false});
        if !running {
            return Ok(());
        }
        let Some(goal) = self.read(procs, &["task", "show", "2"]) else { return Ok(()) };
        self.go_live_revision = goal["settings"]["task"].as_u64();
        if self.go_live_revision.is_none() || goal["chip"]["phase"] == "ended" {
            return Err(format!("work in flight: {goal}"));
        }
        let path = self.root.join("temper.json");
        let mut config = document(&path);
        config["application"]["landing"]["projects"][0]["by"] = "turn".into();
        write(&path, &config);
        binary(procs, 0).signal(Signal::Kill);
        self.stage = Stage::Restart;
        Ok(())
    }
    pub(super) fn go_live_apply(&mut self, procs: &mut [Process]) -> Result<(), String> {
        if let Some(index) = self.go_live_call {
            if calls(procs).ends().get(index).is_none() {
                return Ok(());
            }
            self.stage = Stage::GoLiveStays;
            return Ok(());
        }
        let Some(settings) = self.read(procs, &["settings", "show"]) else { return Ok(()) };
        let revision = settings["newest"].as_u64().ok_or("settings revision")?;
        self.go_live_call = Some(calls(procs).ends().len());
        calls(procs).push(ctl::Step::Run(
            ctl::Call::new(&[
                "settings",
                "apply",
                self.root.join("go-live-after.json").to_str().ok_or("settings path")?,
                "--against",
                &revision.to_string(),
                "--key",
                "go-live-turn",
                "--running",
                "stays",
            ])
            .in_directory("private"),
        ));
        Ok(())
    }
    pub(super) fn go_live_stays(&mut self, procs: &mut [Process]) -> Result<(), String> {
        let Some(goal) = self.read(procs, &["task", "show", "2"]) else { return Ok(()) };
        if goal["settings"]["task"] != self.go_live_revision.ok_or("old revision")?
            || goal["settings"]["newest"].as_u64() <= self.go_live_revision
        {
            return Err(format!("running stays: {goal}"));
        }
        self.go_live_applied = true;
        self.stage = Stage::Observe;
        Ok(())
    }
    pub(super) fn release_go_live(procs: &mut [Process]) -> Result<(), String> {
        for proc in procs {
            if let Process::Peer(peer) = proc {
                peer.release(b"drill-go-live").map_err(|why| format!("go-live release: {why:?}"))?;
            }
        }
        Ok(())
    }
}
