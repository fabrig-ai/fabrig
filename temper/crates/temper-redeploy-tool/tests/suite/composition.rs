//! Compose the real stage and the three module seams, independent of sibling bodies (redeploy.md, 7).
use temper_redeploy_tool::{
    cli::{Action, Steps, execute, execute_with_steps},
    description::Description,
    fake::{Deployment, NEW, description},
    hosts::{Hosts, Part},
    record::{Before, Journal, Record},
    redeploy::{Mode, Outcome},
};

fn before(_description: &Description, hosts: &mut dyn Hosts) -> Result<Before, String> {
    if hosts.run(Part::Engine, "ctl engine snapshot").code != Some(0) {
        return Err("snapshot refused".into());
    }
    hosts.sleep(1);
    Ok(Before { projects: vec![2], cut: false, tasks: Vec::new() })
}
fn redeploy(
    _description: &Description,
    _commit: &str,
    _mode: Mode,
    journal: &mut Journal,
    hosts: &mut dyn Hosts,
) -> Outcome {
    assert_eq!(journal.record.before.projects, [2]);
    assert_eq!(journal.record.began, hosts.now());
    assert_eq!(journal.record.lines.len(), 4);
    assert!(journal.record.lines.iter().all(|line| line.line.starts_with("temper-redeploy stage: ")));
    hosts.run(Part::Engine, "ctl engine redeploy");
    hosts.sleep(1);
    Outcome::Done
}
fn judge(_description: &Description, journal: &mut Journal, hosts: &mut dyn Hosts) -> bool {
    assert_eq!(journal.record.outcome, Some(Outcome::Done));
    assert_eq!(journal.record.ended, Some(hosts.now()));
    hosts.run(Part::Engine, "ctl engine judge");
    true
}
fn steps() -> Steps {
    Steps { before, redeploy, judge }
}
#[test]
fn run_stages_then_begins_redeploys_and_judges_and_refuses_existing_records_first() {
    let scratch = skein_scratch::Scratch::new("redeploy-composition");
    let description = description(1);
    let mut hosts = Deployment::new(description.clone());
    for command in ["snapshot", "redeploy", "judge"] {
        hosts.ctl_answer(command, temper_redeploy_tool::fake::output(0, ""));
    }
    let path = scratch.path().join("record.json");
    let action = Action::Run { commit: NEW.into(), mode: Mode::Engine, record: path.clone() };
    assert_eq!(execute_with_steps(&description, &action, &mut hosts, Box::new(std::io::sink()), steps()).unwrap(), 0);
    let record: Record = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(record.began, 1_000_000_001);
    assert_eq!(record.before.projects, [2]);
    assert_eq!(record.outcome, Some(Outcome::Done));
    assert_eq!(record.ended, Some(1_000_000_002));
    let lines: Vec<_> = hosts.lines.iter().map(|call| call.line.as_str()).collect();
    assert_eq!(&lines[lines.len() - 3..], &["ctl engine snapshot", "ctl engine redeploy", "ctl engine judge"]);
    let bytes = std::fs::read(&path).unwrap();
    hosts.lines.clear();
    // The production entry has the same existing-record refusal, before any seam is called.
    assert!(execute(&description, &action, &mut hosts, Box::new(std::io::sink())).unwrap_err().contains("record"));
    assert_eq!(hosts.lines, [], "not even staging on an existing record");
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(
        execute_with_steps(
            &description,
            &Action::Referee { record: path.clone() },
            &mut hosts,
            Box::new(std::io::sink()),
            steps()
        )
        .unwrap(),
        0
    );
    assert_eq!(std::fs::read(path).unwrap(), bytes, "referee alone does not rewrite");
}
#[test]
fn a_staging_failure_records_refused_without_begun_redeploy_or_referee() {
    let scratch = skein_scratch::Scratch::new("redeploy-refused");
    let description = description(1);
    let mut hosts = Deployment::new(description.clone());
    hosts.versions.insert(
        (Part::Engine, format!("/builds/{NEW}/temper-engine")),
        temper_redeploy_tool::fake::output(0, "temper-engine 1.0 (000000000000)"),
    );
    let path = scratch.path().join("record.json");
    let action = Action::Run { commit: NEW.into(), mode: Mode::Engine, record: path.clone() };
    assert_eq!(execute_with_steps(&description, &action, &mut hosts, Box::new(std::io::sink()), steps()).unwrap(), 1);
    let record: Record = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(record.outcome, Some(Outcome::Refused));
    assert_eq!(record.began, 0);
    assert!(hosts.lines.iter().all(|call| !call.line.starts_with("ctl")));
    hosts.versions.clear();
    hosts.lines.clear();
    hosts.ctl_answer("snapshot", temper_redeploy_tool::fake::output(1, ""));
    let path = scratch.path().join("snapshot-refused.json");
    let action = Action::Run { commit: NEW.into(), mode: Mode::Engine, record: path.clone() };
    assert_eq!(execute_with_steps(&description, &action, &mut hosts, Box::new(std::io::sink()), steps()).unwrap(), 1);
    let record: Record = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(record.outcome, Some(Outcome::Refused));
    assert_eq!(record.began, 0);
    assert_eq!(
        hosts
            .lines
            .iter()
            .filter(|call| call.line.starts_with("ctl"))
            .map(|call| call.line.as_str())
            .collect::<Vec<_>>(),
        ["ctl engine snapshot"]
    );
}
