//! Scripts remain usable by both sibling changes (redeploy.md, 8).
use temper_redeploy_tool::{
    fake::{Deployment, Opening, WIRES, description, output},
    hosts::{Hosts, Part, fill, words},
};

#[test]
fn fake_checkpoints_require_a_stopped_engine_and_verdict_and_ctl_are_scriptable() {
    let d = description(1);
    let mut hosts = Deployment::new(d.clone());
    let checkpoint = fill(&d.engine.checkpoint, &[("binary", "/bin/temper-engine")]);
    assert_eq!(hosts.run(Part::Engine, &checkpoint).code, Some(1));
    let serving = temper_redeploy_tool::hosts::ctl(&d, &["deployment", "checkpoint"], &mut hosts).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&serving).unwrap()["name"],
        "00000000000000000001-0123456789abcdef"
    );
    hosts.run(Part::Engine, &d.engine.stop);
    for number in [2, 3] {
        let answer = hosts.run(Part::Engine, &checkpoint);
        assert_eq!(answer.code, Some(0));
        assert_eq!(
            answer.stdout,
            format!(
                "temper-engine checkpoint: {number:020}-0123456789abcdef at commit 42, secrets at 7: 12 files linked, 3 copied (4096 bytes) in 15 ms; 4096 bytes held; 0 older removed, 0 skipped\n"
            )
        );
    }
    hosts.verdict_answer(0, "workers may stay: yes");
    hosts.verdict_answer(7, "verdict refused");
    for code in [0, 7, 7] {
        assert_eq!(
            hosts
                .run(
                    Part::Engine,
                    &fill(&d.engine.verdict, &[("binary", "/staged/temper-engine"), ("checkpoint", "a-name")])
                )
                .code,
            Some(code)
        );
    }
    hosts.ctl_answer("deployment checkpoint --json", output(2, "checkpoint refused"));
    assert!(temper_redeploy_tool::hosts::ctl(&d, &["deployment", "checkpoint"], &mut hosts).is_err());
}
#[test]
fn fake_scripts_checks_openings_ctl_documents_and_failed_lines_on_its_own_clock() {
    let description = description(3);
    let mut hosts = Deployment::new(description.clone());
    hosts.check_answer(4, "workers may stay: no\n");
    assert_eq!(
        hosts.run(Part::Engine, "check '/builds/new/temper-engine' '00000000000000000001-0123456789abcdef'").code,
        Some(4)
    );
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
    let file = temper_config_shell::parse(
        document.stdout.as_bytes(),
        temper_config_shell::Shape::WorkerAlone,
        &temper_config_shell::Facts { login: "qa".into(), hostname: "worker".into(), provider_key: false },
    )
    .expect("worker configuration")
    .file;
    assert_eq!(file.name.as_deref(), Some("w1"));
    assert_eq!(file.engine, Some("127.0.0.1:7200".parse().expect("engine listener")));
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
