//! Tool-output fixtures enter the codec; none are provider evidence.

#[test]
fn tool_output_seed_exercises_capture_and_read_windows() {
    let root = crate::common::repository();
    let suite_file = root.join("suites/probes.toml");
    let mut suite = smith_bench::read_suite(&suite_file).expect("four authored variants");
    suite.tasks.retain(|name| name.starts_with("probes/tool-output/") && !name.ends_with("/background"));
    suite.agents.retain(|agent| agent.tasks.iter().any(|name| name == "probes/tool-output"));
    suite.agents[0].tasks.clone_from(&suite.tasks);
    let catalogue =
        smith_bench::catalogue(&root.join("tasks"), &root.join("../docs/design")).expect("frozen catalogue");
    let models = smith_bench::read_model_tiers(&root.join("agents/models.toml")).expect("tiers");
    assert_eq!(
        smith_bench::validate_suite(&suite_file, &suite, &catalogue, &root.join("agents"), &models)
            .expect("selectors")
            .len(),
        4
    );
    let output = std::process::Command::new("sh")
        .arg(root.join("tasks/probes/tool-output/seed/test.sh"))
        .output()
        .expect("credential-free seed command");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.len() > 10 * 1024);
    assert!(output.stdout.ends_with(b"ASSERT budget_total: expected 17, observed 19\n"));
    let document = std::fs::read_to_string(root.join("tasks/probes/tool-output/seed/manual.txt")).expect("document");
    assert!(document.len() > 30_000);
    assert_eq!(
        document.lines().nth(699),
        Some("QUOTE: The copper heron carries seven blue lanterns across the silent harbor.")
    );
}
