//! Scripts remain usable by both sibling changes (redeploy.md, 8).
use temper_redeploy_tool::{
    fake::{Deployment, Opening, WIRES, description, output},
    hosts::{Hosts, Part, fill, words},
};

#[test]
fn fake_scripts_checks_openings_ctl_documents_and_failed_lines_on_its_own_clock() {
    let description = description(3);
    let mut hosts = Deployment::new(description.clone());
    hosts.check_answer(4, "workers may stay: no\n");
    assert_eq!(hosts.run(Part::Engine, "check '/builds/new/temper-engine'").code, Some(4));
    hosts.opening(0, Opening::After { nanoseconds: 2_000_000_000, wires: "a-wire 7".into() });
    hosts.opening(1, Opening::Refused { nanoseconds: 1_000_000_000, text: "too old".into() });
    hosts.opening(2, Opening::Never);
    hosts.run(Part::Engine, "stop engine");
    hosts.run(Part::Engine, "start engine");
    assert_eq!(hosts.log, []);
    hosts.sleep(2_000_000_000);
    assert_eq!(hosts.now(), 3_000_000_000);
    let log = hosts.run(Part::Engine, &fill(&description.engine.log, &[("since", "2")]));
    assert!(log.stdout.starts_with("temper-engine serve: worker w2 refused at its opening: wires: too old\n"));
    assert!(log.stdout.ends_with("temper-engine serve: worker w1 opened: a-wire 7\n"));
    assert!(!log.stdout.contains("w3"));
    hosts.opening(2, Opening::default());
    hosts.run(Part::Worker(2), "stop w3");
    hosts.run(Part::Worker(2), "start w3");
    assert_eq!(hosts.log.last().unwrap().1, format!("temper-engine serve: worker w3 opened: {WIRES}"));
    hosts.ctl_answer("me --json", output(0, "{\"projects\": []}"));
    assert_eq!(hosts.run(Part::Engine, "ctl engine me --json").stdout, "{\"projects\": []}");
    let document = hosts.run(Part::Worker(0), "document w1");
    let document: serde_json::Value = serde_json::from_str(&document.stdout).unwrap();
    assert_eq!(document["workers"][0]["graces"]["grace_ms"], 30000);
    hosts.fail_line(Part::Worker(0), "stop w1", 23);
    assert_eq!(hosts.run(Part::Worker(0), "stop w1").code, Some(23));
    assert!(hosts.workers[0].running);
    assert_eq!(hosts.lines.last().unwrap().at, 3_000_000_000);
    hosts.failures.clear();
    hosts.run(Part::Worker(0), "stop w1");
    assert_eq!(hosts.run(Part::Worker(0), &words(&["mkdir", "-p", "/builds/old"])).code, Some(0));
    assert_eq!(
        hosts.run(Part::Worker(0), &words(&["cp", "/bin/temper-worker", "/builds/old/temper-worker"])).code,
        Some(0)
    );
    assert_eq!(hosts.workers[0].staged["/builds/old/temper-worker"], temper_redeploy_tool::fake::OLD);
    hosts.stage_all(temper_redeploy_tool::fake::NEW);
    let from = format!("/builds/{}/temper-worker", temper_redeploy_tool::fake::NEW);
    assert_eq!(
        hosts
            .run(Part::Worker(0), &fill(&description.workers[0].swap, &[("from", &from), ("to", "/bin/temper-worker")]))
            .code,
        Some(0)
    );
    assert_eq!(hosts.workers[0].installed["temper-worker"], temper_redeploy_tool::fake::NEW);
}
