//! Preparation consumes admitted identity and check evidence without a program or git.

use super::*;
use crate::agent::PinnedConfiguration;
use crate::arm::{ArmRef, Build, BuiltArm};
use crate::guard::Guard;
use crate::isolate::Private;
use crate::process::lower_tests::{check_receipt, exec_prompt_receipt};
use crate::{ModelChoice, Provider, Task};
use std::os::unix::fs::DirBuilderExt;

struct Fixture {
    _scratch: skein_scratch::Scratch,
    root: PathBuf,
    guard: Guard,
    private: Private,
    pin: PinnedConfiguration,
    task: Task,
    arm: ArmRef,
    model: ModelChoice,
    home: PathBuf,
}

impl Fixture {
    fn new(name: &str, provider: Provider, model: &str) -> Self {
        let scratch = skein_scratch::Scratch::new("bench-lower_tests");
        let root = scratch.path().to_path_buf().join(format!("smith-lower-prepare-{name}-{}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&root).expect("owned root");
        let guard = Guard::new(&root.join("home"), &root.join("config"), &root.join("state"), &[]).expect("guard");
        let private = crate::isolate::empty(&guard, &root.join("scratch")).expect("private paths");
        crate::run::private_parent(&guard, &private.root.join("smith")).expect("trace parent");
        let project = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut pin = crate::read_configuration(&project.join("agents/smith/standard.pin.toml")).expect("pin");
        pin.pin.provider = provider;
        let mut task = crate::read_task(&project.join("tasks/probes/first-request/task.toml")).expect("task");
        task.smith.clear();
        task.budget.turns = None;
        task.budget.seconds = None;
        task.budget.spend = None;
        Self {
            _scratch: scratch,
            home: root.join("admitted-sign-in"),
            arm: ArmRef::Built(Build {
                directory: root.join("verified-arm"),
                arm: BuiltArm {
                    version: 1,
                    commit: "a".repeat(40),
                    sha256: "b".repeat(64),
                    toolchain: "supplied identity".into(),
                    binaries: Vec::new(),
                },
            }),
            model: ModelChoice { model: Some(model.into()), effort: Some("low".into()) },
            root,
            guard,
            private,
            pin,
            task,
        }
    }

    fn prepared(&self, serial: Option<fs::File>) -> Prepared {
        let context = self.context();
        let ArmRef::Built(build) = context.arm else { unreachable!("supplied build") };
        let base: toml::Value = toml::from_str(include_str!("../../../../agents/smith/standard.toml")).expect("layer");
        let registry = FaceSince {
            commit: build.arm.commit.clone(),
            shape: super::super::events::Shape::Exec,
            receiving: super::super::Receiving { string: 128, content: 1024, items: 16, encoded_line: 16384 },
        };
        render(&context, &serde_json::to_value(base).expect("layer"), build.directory.join("smith"), registry, serial)
            .expect("render admitted inputs")
    }

    fn context(&self) -> Context<'_> {
        let ArmRef::Built(build) = &self.arm else { unreachable!("supplied build") };
        Context {
            guard: &self.guard,
            private: &self.private,
            pin: &self.pin,
            pin_file: Path::new("supplied.pin.toml"),
            task: &self.task,
            variant: None,
            model: &self.model,
            arm: &self.arm,
            installed_version: &build.arm.commit,
            home: &self.home,
            wall: skein_lib::Wall::EPOCH,
            lifetime: skein_lib::Duration::from_secs(20),
            environment: &[],
        }
    }
}

#[test]
fn admitted_sign_in_keeps_shared_credentials_and_private_state() {
    let fixture = Fixture::new("sign-in", Provider::Codex, "gpt-6-luna");
    let prepared = fixture.prepared(None);
    assert_eq!(prepared.rendered["credentials"], serde_json::json!(fixture.home));
    assert_eq!(prepared.rendered["accounts"]["chatgpt"]["source"], "sign_in");
    assert_eq!(prepared.rendered["accounts"]["claude"]["source"], "program");
    assert!(prepared.rendered["accounts"].get("codex").is_none());
    assert_eq!(prepared.state, fixture.private.state);
    assert!(prepared.settings.starts_with(&fixture.private.config));
    assert!(!fixture.private.root.join("smith/tokens").exists());
    assert!(!prepared.program.exists(), "identity is an input, no executable fixture");
}

const CHECK_V1: &[u8] = br#"{"version":1,"limits":[{"name":"service.teardown","value":12345,"unit":"ms"}]}"#;

#[test]
fn an_exec_arm_reads_its_prompt_to_eof_without_a_trace_file() {
    let fixture = Fixture::new("exec-eof", Provider::Codex, "gpt-6-luna");
    let context = fixture.context();
    let checked = checked(&context, fixture.prepared(None), check_receipt(CHECK_BOUNDS, 0, CHECK_V1, b""));
    let prepared = require_success(checked.expect("settled check")).expect("checked Exec");
    assert!(prepared.trace.is_none());
    assert_eq!(prepared.command().teardown, skein_lib::Duration::from_millis(12_345));
    let capture = crate::process::Capture::Agent(Box::new(
        crate::agent::PreparedAgent::Smith(prepared).observe().expect("Exec observer"),
    ));
    let result = exec_prompt_receipt(capture, b"read this prompt to EOF\n");
    assert_eq!(result.exit, crate::Exit::Code(0));
    assert_eq!(result.forced, crate::Forced::No);
    assert!(result.error.is_none() && result.tree.live_pids.is_empty());
}

#[test]
fn an_exec_arm_refused_by_check_is_a_setup_refusal() {
    let fixture = Fixture::new("exec-check-refused", Provider::Codex, "gpt-6-luna");
    let context = fixture.context();
    let stderr = b"configuration refused\n";
    let check = check_receipt(CHECK_BOUNDS, 2, CHECK_V1, stderr);
    assert!(check.error.is_none() && check.tree.live_pids.is_empty());
    let checked = checked(&context, fixture.prepared(None), check).expect("settled check");
    let directory = checked.0.settings.parent().expect("check logs");
    assert_eq!(fs::read(directory.join("check.json")).expect("bounded stdout"), CHECK_V1);
    assert_eq!(fs::read(directory.join("check-stderr.log")).expect("bounded stderr"), stderr);
    let refusal = require_success(checked).expect_err("check refuses");
    assert_eq!(refusal.kind, crate::FaultKind::Setup(SetupStep::Configuration));
    assert!(refusal.detail.contains("Code(2)"));
    assert!(!fixture.private.root.join("smith/tokens").exists());
}

#[test]
fn supplied_check_bound_reaches_the_prepared_exec_command() {
    let fixture = Fixture::new("check", Provider::Codex, "gpt-6-luna");
    let bytes = br#"{"version":2,"limits":[{"name":"service.teardown","value":12345,"unit":"ms"}]}"#;
    let prepared = accept_check(fixture.prepared(None), bytes).expect("successful supplied check");
    let command = prepared.command();
    assert_eq!(prepared.teardown.as_nanos(), 12_345_000_000);
    assert_eq!(command.teardown.as_nanos(), 12_345_000_000);
    assert_eq!(
        command.arguments.iter().map(|arg| arg.to_str().expect("argument")).collect::<Vec<_>>(),
        [
            "exec",
            "--json",
            "-C",
            fixture.private.workspace.to_str().expect("workspace"),
            "-m",
            "openai/gpt-6-luna",
            "--effort",
            "low",
            "-"
        ]
    );
}

#[test]
fn admitted_program_keeps_its_model_trace_and_serial_guard() {
    let fixture = Fixture::new("program", Provider::Anthropic, "claude-haiku-5-5");
    let serial = fs::File::create(fixture.root.join("supplied-serial")).expect("admitted serial guard");
    let prepared = fixture.prepared(Some(serial));
    assert_eq!(prepared.rendered["endpoints"]["anthropic"]["account"], "claude");
    assert_eq!(prepared.rendered["endpoints"]["anthropic"]["program"], "claude_code");
    assert_eq!(prepared.rendered["credentials"], serde_json::json!(fixture.home));
    assert!(prepared.serial.is_some());
    assert!(prepared.program_source());
    assert!(prepared.command().arguments.windows(2).any(|pair| pair == ["-m", "anthropic/claude-haiku-5-5"]));
    assert_eq!(prepared.rendered["trace"]["path"], serde_json::json!(prepared.trace.as_ref().expect("program trace")));
    drop(prepared);
}
