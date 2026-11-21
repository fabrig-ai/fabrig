//! A serving-verdict refusal through the CLI journal file.
use temper_redeploy_tool::{
    fake::NEW,
    hosts::Output,
    record::Record,
    redeploy::{Mode, Outcome, redeploy},
    testing::redeploy::{CHECKPOINT, Printed, all_old, staged},
};

#[test]
fn a_failed_serving_verdict_refuses_without_stopping_or_swapping() {
    for mode in [Mode::Engine, Mode::Workers] {
        let mut hosts = staged(2);
        hosts.verdict.push_back(Output {
            code: Some(2),
            stdout: String::new(),
            stderr: "temper-engine: no command upgrade --workers; temper-engine --help lists them\n".into(),
        });
        let scratch = skein_scratch::Scratch::new("redeploy-serving-verdict");
        let path = scratch.path().join("record.json");
        let printed = Printed::default();
        let exit = temper_redeploy_tool::cli::execute_with_steps(
            &hosts.description.clone(),
            &temper_redeploy_tool::cli::Action::Run { commit: NEW.into(), mode, record: path.clone() },
            &mut hosts,
            Box::new(printed.clone()),
            temper_redeploy_tool::cli::Steps {
                before: |_, _| Ok(temper_redeploy_tool::record::Before::default()),
                redeploy,
                judge: |_, _, _| true,
            },
        )
        .unwrap();
        assert_eq!(exit, 1);
        let record: Record = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(record.outcome, Some(Outcome::Refused));
        assert_eq!(
            String::from_utf8(printed.0.borrow().clone()).unwrap().lines().collect::<Vec<_>>(),
            record.lines.iter().map(|line| line.line.as_str()).collect::<Vec<_>>()
        );
        all_old(&hosts);
        assert!(hosts.lines.iter().all(|call| !call.line.starts_with("stop ")
            && !call.line.starts_with("start ")
            && !call.line.starts_with("checkpoint ")));
        assert_eq!(hosts.lines.last().unwrap().line, format!("verdict '/builds/{NEW}/temper-engine' '{CHECKPOINT}'"));
        assert_eq!(record.verdict, None);
        assert_eq!(record.backup, None);
        assert_eq!(record.window, None);
        assert_eq!(record.openings, []);
        let lines: Vec<_> =
            record.lines.iter().filter_map(|line| line.line.strip_prefix("temper-redeploy run: ")).collect();
        let start = lines
            .iter()
            .position(|line| *line == "temper-engine: no command upgrade --workers; temper-engine --help lists them")
            .unwrap();
        assert_eq!(
            &lines[start..],
            [
                "temper-engine: no command upgrade --workers; temper-engine --help lists them",
                "serving verdict exit Some(2)",
                "serving verdict: a build without upgrade --workers is redeployed with --together",
                "serving verdict failed: exit Some(2): temper-engine: no command upgrade --workers; temper-engine --help lists them ",
                "refused: serving verdict: exit Some(2): temper-engine: no command upgrade --workers; temper-engine --help lists them ",
            ]
        );
    }
}
