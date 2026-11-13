//! Staging probes, fetches once per host and verifies again (redeploy.md, 3).
use temper_redeploy_tool::{
    fake::{Deployment, NEW, OLD, description, output},
    hosts::{Part, words},
    stage::stage,
};

#[test]
fn already_staged_binaries_are_left_alone_and_missing_ones_fetched_once_per_host() {
    let description = description(2);
    let mut hosts = Deployment::new(description.clone());
    hosts.stage_all(NEW);
    let mut journal = super::journal();
    stage(&description, NEW, &mut journal, &mut hosts).unwrap();
    assert_eq!(hosts.lines.len(), 6);
    assert!(hosts.lines.iter().all(|call| call.line.ends_with("'--version'")));
    hosts.lines.clear();
    hosts.workers[0].staged.clear();
    hosts.workers[1].staged.remove(&format!("/builds/{NEW}/temper-agent"));
    stage(&description, NEW, &mut journal, &mut hosts).unwrap();
    let fetches: Vec<_> = hosts.lines.iter().filter(|call| call.line.contains("'curl'")).collect();
    assert_eq!(fetches.len(), 2);
    assert_eq!(fetches[0].part, Part::Worker(0));
    assert_eq!(fetches[1].part, Part::Worker(1));
    assert!(!fetches[1].line.contains("Linux-x86_64/temper-worker"));
    assert!(fetches[0].line.contains("'sha256sum' '-c' '--ignore-missing' 'SHA256SUMS'"));
    assert!(fetches[0].line.contains("'gunzip' '-c' 'Linux-x86_64/temper-worker' > 'temper-worker'"));
    assert!(fetches[0].line.contains("'chmod' '0755' 'temper-agent'"));
    for name in ["temper-worker", "temper-agent"] {
        assert_eq!(hosts.workers[0].staged[&format!("/builds/{NEW}/{name}")], NEW);
        assert_eq!(
            hosts
                .lines
                .iter()
                .filter(|call| call.part == Part::Worker(0)
                    && call.line == words(&[&format!("/builds/{NEW}/{name}"), "--version"]))
                .count(),
            2
        );
    }
    assert!(journal.record.lines.iter().all(|line| line.line.starts_with("temper-redeploy stage: ")));
    assert!(hosts.engine.running && hosts.workers.iter().all(|worker| worker.running));
}
#[test]
fn wrong_answers_after_fetch_refuse_with_part_and_binary() {
    let description = description(1);
    let mut hosts = Deployment::new(description.clone());
    hosts.versions.insert(
        (Part::Worker(0), format!("/builds/{NEW}/temper-agent")),
        output(0, &format!("temper-agent 1.0 ({OLD})\n")),
    );
    let why = stage(&description, NEW, &mut super::journal(), &mut hosts).unwrap_err();
    assert!(why.contains("w1") && why.contains(&format!("/builds/{NEW}/temper-agent")), "{why}");
    assert!(hosts.engine.running && hosts.workers[0].running);
    let fetch =
        hosts.lines.iter().find(|call| call.part == Part::Engine && call.line.contains("'curl'")).unwrap().line.clone();
    let mut hosts = Deployment::new(description.clone());
    hosts.fail_line(Part::Engine, &fetch, 73);
    let why = stage(&description, NEW, &mut super::journal(), &mut hosts).unwrap_err();
    assert!(why.contains("engine") && why.contains(&format!("/builds/{NEW}/temper-engine")), "{why}");
    assert!(hosts.lines.iter().all(|call| call.part == Part::Engine));
    hosts.lines.clear();
    assert!(stage(&description, "ABCDEF012345", &mut super::journal(), &mut hosts).unwrap_err().contains("COMMIT"));
    assert_eq!(hosts.lines, []);
}
