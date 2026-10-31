//! Requests consume explicit machine observations and verified arms without git or environment mutation.

use super::*;

const ATTEMPTS: u32 = 2;
const RUN: &str = "lower-plan";
const STARTED: &str = "2026-10-29T00:00:00Z";

struct Fixture {
    directory: PathBuf,
    guard: Guard,
    arm: ArmRef,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let directory = std::env::temp_dir().join(format!("smith-lower-plan-{name}-{}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&directory).expect("private output");
        let guard = Guard::new(&directory.join("home"), &directory.join("config"), &directory.join("state"), &[])
            .expect("guard");
        let arm = ArmRef::Built(crate::arm::Build {
            directory: "/verified-arm".into(),
            arm: crate::arm::BuiltArm {
                version: 1,
                commit: "a".repeat(40),
                sha256: "b".repeat(64),
                toolchain: "supplied verified arm".into(),
                binaries: Vec::new(),
            },
        });
        Self { directory, guard, arm }
    }

    fn requests(&self, agent: Agent, arm: &ArmRef, attempts: u32) -> Vec<crate::run::Prepared> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let inputs = RequestInputs {
            repository: root.parent().expect("smith tree"),
            codex_home: self.directory.join("logins/codex"),
            host: "supplied-host".into(),
            cpus: vec![0],
        };
        let order = Order { probe: "probes/smoke-tools".into(), agent, attempts, source: Source::Head };
        let value = narrowed(&root.join("suites/probes.toml"), &order).expect("narrow order");
        let suite: Suite = value.try_into().expect("suite");
        let manifest = root.join("tasks/probes/smoke-tools/task.toml");
        let entry =
            crate::CatalogueTask { name: order.probe, task: crate::read_task(&manifest).expect("task"), manifest };
        let repetitions = suite.task_repetitions(&entry.name, entry.task.repetitions, true).expect("repetitions");
        let mut requests = Vec::new();
        task_requests_in(
            &inputs,
            &entry,
            &suite,
            (arm, &[]),
            (RUN, STARTED, Some(crate::SmithFace::Events), &"c".repeat(40)),
            repetitions,
            &mut requests,
        )
        .expect("construct requests");
        write_requests(&self.guard, &self.directory, &requests).expect("persist requests");
        requests
    }

    fn clean(self) {
        fs::remove_dir_all(self.directory).expect("owned cleanup");
    }
}

#[test]
fn supplied_arm_plans_each_attempt_with_pin_and_tier() {
    let fixture = Fixture::new("smith");
    let requests = fixture.requests(Agent::Smith, &fixture.arm, ATTEMPTS);
    assert_eq!(requests.len(), 2);
    for (request, attempt) in requests.iter().zip(["01", "02"]) {
        assert_eq!(request.identity.attempt, attempt);
        assert_eq!(request.identity.model.requested_model, "gpt-6-luna");
        assert_eq!(request.identity.model.requested_effort, "low");
        assert_eq!(request.identity.agent_version, "a".repeat(40));
        assert_eq!(request.identity.binary_sha256, "b".repeat(64));
        assert_eq!(request.program, Path::new("/verified-arm/smith"));
        assert!(request.configuration.as_ref().expect("pin").ends_with("smith/everything.pin.toml"));
        assert_eq!(request.codex_home.as_ref().expect("explicit home"), &fixture.directory.join("logins/codex"));
        assert_eq!(request.identity.host, "supplied-host");
        assert_eq!(request.identity.cpus, [0]);
        let persisted: crate::run::Prepared = serde_json::from_slice(
            &fs::read(fixture.directory.join(format!("{attempt}.json"))).expect("one request file per attempt"),
        )
        .expect("persisted request");
        assert_eq!(persisted.identity.attempt, attempt);
        assert_eq!(persisted.identity.run, RUN);
    }
    fixture.clean();
}

#[test]
fn installed_codex_selects_the_standard_pin_without_environment() {
    let fixture = Fixture::new("codex");
    let program = fixture.directory.join("unexecuted-codex");
    fs::write(&program, b"identity only, never executed").expect("digest input");
    let requests = fixture.requests(Agent::Codex, &ArmRef::Installed { program }, 1);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].identity.agent, Agent::Codex);
    assert!(requests[0].configuration.as_ref().expect("pin").ends_with("codex/standard.pin.toml"));
    fixture.clean();
}

#[test]
fn program_order_keeps_provider_model_pin_and_arm_identity() {
    let fixture = Fixture::new("program");
    let requests = fixture.requests(Agent::ClaudeCode, &fixture.arm, 1);
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(request.identity.agent, Agent::Smith);
    assert_eq!(request.identity.provider, crate::Provider::Anthropic);
    assert_eq!(request.identity.model.requested_model, "claude-haiku-5-5");
    assert_eq!(request.identity.arm, "smith-claude-code-bbbbbbbb");
    assert!(request.configuration.as_ref().expect("program pin").ends_with("smith/smith-claude-code.pin.toml"));
    fixture.clean();
}
