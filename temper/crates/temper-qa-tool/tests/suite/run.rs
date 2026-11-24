//! The real bounded-command witness (temper/docs/design/testing/qa/README.md, 6).
use super::world::{World, baseline, id};
use std::fs;
use temper_qa_tool::{Exit, faces::RealFiles};

fn request_directory(world: &World, pass: u64, request: u64) -> std::path::PathBuf {
    world.home.directory.join("passes").join(pass.to_string()).join("requests").join(request.to_string())
}
fn deployment(world: &World, name: &str, body: &str) -> Vec<String> {
    let script = world.home.directory.join(format!("{name}.sh"));
    fs::write(&script, format!(r#"set -eu
printf '%s\n' "$PWD" "$CAPTURED" "$TEMPER_INTERNAL_QA_HOME" "$TEMPER_INTERNAL_QA_AREA" "$@" >> "$TEMPER_INTERNAL_QA_HOME/{name}-calls"
find . -type f | sort > "$TEMPER_INTERNAL_QA_HOME/{name}-files"
if read -r ignored; then exit 94; fi
{body}
"#)).expect("deployment script");
    vec!["/bin/sh".into(), script.display().to_string()]
}
fn seed_request(world: &World, count: u64, time: u64, page: &str, commit: &str) -> u64 {
    let pass = world
        .home
        .transact(&mut RealFiles, |ledger, _| {
            let case = super::passes::queue(ledger, page, "web");
            let holds = ledger.landings.iter().map(|landing| landing.number).collect::<Vec<_>>();
            super::passes::build(ledger, commit, &holds, time);
            let start = super::passes::start(ledger);
            let mut findings = Vec::new();
            for index in 0..count {
                let mut finding = super::passes::finding(case);
                finding.title = format!("Start does nothing {index}");
                finding.setup = "recipe".into();
                finding.lines = vec!["web open work".into(), format!("web press button Start{index}")];
                finding.expected = format!("goal {index} starts");
                findings
                    .push(ledger.record_finding(start.session, &finding, &["recipe".into()], "product", time)?.number);
            }
            ledger.record_outcome(
                start.session,
                case,
                if count == 0 { "passed" } else { "failed" },
                "tested",
                &findings,
                time,
            )?;
            ledger.end_pass_session(start.session, Exit::Code(0), None, time, "qa", &["recipe".into()])?;
            Ok(start.pass)
        })
        .expect("request through ledger operations");
    let recipe = world.home.directory.join("passes").join(pass.to_string()).join("setups/recipe");
    fs::create_dir_all(recipe.join("nested")).expect("recipe directory");
    fs::write(recipe.join("README.md"), "# Recipe\n\nThe whole recipe.\n").expect("recipe");
    fs::write(recipe.join("nested/bytes"), [0, 1, 255, 10]).expect("nested binary recipe file");
    pass
}

#[test]
fn bounded_requests_are_not_filed_and_real_commands_are_killed_at_the_bound() {
    use temper_qa_tool::faces::{ProcessCommand, Processes, RealProcesses};
    let mut world = World::new();
    seed_request(&world, 1, 5, "work", &id(0));
    world.home.configuration.request.command = deployment(&world, "file", "printf 'goal 41\\n'");
    let mut run = world.start(baseline(), &[]);
    run.processes.command_bound = true;
    run.tick();
    assert_eq!(world.ledger().passes[0].requests[0].filed, None);
    run.processes.command_bound = false;
    run.tick();
    assert_eq!(world.ledger().passes[0].requests[0].goal, Some(41));
    let command = ProcessCommand {
        arguments: vec!["/bin/sh".into(), "-c".into(), "while :; do :; done".into()],
        directory: world.home.directory.clone(),
        stdin: "/dev/null".into(),
        output: world.home.directory.join("bounded-output"),
        environment: vec![],
    };
    assert_eq!(RealProcesses::default().run(&command, 0).expect("bounded command").exit, Exit::Bound);
    assert!(!command.output.with_extension("stdout").exists(), "temporary stdout is removed");
}
