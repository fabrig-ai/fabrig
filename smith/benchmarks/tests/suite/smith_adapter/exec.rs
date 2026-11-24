//! Synthetic checked Exec arms exercise preparation, shared logins and outside classification.
use super::{Duration, Path, PathBuf, root};

pub(super) struct ExecFixture {
    _scratch: skein_scratch::Scratch,
    source: PathBuf,
    guard: smith_bench::guard::Guard,
    private: smith_bench::isolate::Private,
    pin: smith_bench::agent::PinnedConfiguration,
    pin_file: PathBuf,
    task: smith_bench::Task,
    arm: smith_bench::arm::ArmRef,
    environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    model: smith_bench::ModelChoice,
}

impl ExecFixture {
    pub(super) fn new(name: &str, check_code: i32) -> Self {
        Self::fixture(name, Some(check_code))
    }

    fn fixture(name: &str, check_code: Option<i32>) -> Self {
        use sha2::{Digest, Sha256};
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        let scratch = root(&format!("{name}-outside-repository"));
        let root = scratch.path().join("root");
        std::fs::DirBuilder::new().mode(0o700).create(&root).expect("private fixture root");
        let repository = root.join("source");
        std::fs::create_dir_all(&repository).expect("fixture repository");
        for file in ["config/smith/config.toml", "host-id", "tokens/chatgpt"] {
            let path = root.join("logins/smith").join(file);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("synthetic parent");
            std::fs::write(path, []).expect("existence marker");
        }
        let guard = smith_bench::guard::Guard::new(&root.join("user"), &root.join("config"), &root.join("state"), &[])
            .expect("guard");
        let private = smith_bench::isolate::empty(&guard, &root.join("scratch")).expect("scratch");
        let git = |arguments: &[&str]| {
            let output = std::process::Command::new("/usr/bin/git")
                .arg("-C")
                .arg(&repository)
                .args(arguments)
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .output()
                .expect("fixture source identity");
            assert!(output.status.success());
            String::from_utf8(output.stdout).expect("fixture identity text").trim().to_owned()
        };
        let (commit, program) = if let Some(check_code) = check_code {
            git(&["init", "-q"]);
            git(&[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.test",
                "-c",
                "core.hooksPath=/dev/null",
                "commit",
                "--allow-empty",
                "-qm",
                "Exec fixture",
            ]);
            let program = format!(
                "#!/bin/sh\nif [ \"$1\" = check ]; then\n/usr/bin/printf '%s\\n' '{{\"version\":1,\"limits\":[{{\"name\":\"service.teardown\",\"value\":12345,\"unit\":\"ms\"}}]}}'\nexit {check_code}\nfi\n/bin/cat >/dev/null\n"
            );
            (git(&["rev-parse", "HEAD"]), program)
        } else {
            ("a".repeat(40), "synthetic verified identity; never executed\n".into())
        };
        let digest = format!("{:x}", Sha256::digest(program.as_bytes()));
        let built = root.join(smith_bench::arm::arm_name(&commit, &digest).expect("arm name"));
        std::fs::create_dir(&built).expect("arm directory");
        std::fs::write(built.join("smith"), &program).expect("fixture check executable");
        if check_code.is_some() {
            std::fs::set_permissions(built.join("smith"), std::fs::Permissions::from_mode(0o700)).expect("executable");
        }
        let manifest = serde_json::json!({"version":1,"commit":commit,"sha256":digest,"toolchain":"scripted fixture",
            "binaries":[{"name":"smith","sha256":digest}]});
        std::fs::write(built.join("arm.json"), serde_json::to_vec(&manifest).expect("arm manifest")).expect("manifest");
        let arm =
            smith_bench::arm::ArmRef::Built(smith_bench::arm::read_build(&built.join("smith")).expect("built fixture"));
        std::fs::write(repository.join("faces.toml"), format!("[[face]]\ncommit='{commit}'\nshape='exec'\nreceiving={{string=128,content=1024,items=16,encoded_line=16384}}\n")).expect("Exec registry");
        std::fs::write(repository.join("settings.toml"), include_bytes!("../../../agents/smith/standard.toml"))
            .expect("pin template");
        let pin_file = repository.join("exec.pin.toml");
        std::fs::write(
            &pin_file,
            b"name='exec'\nagent='smith'\nprovider='codex'\nversion='built-arm'\nconfiguration='settings.toml'\n",
        )
        .expect("fixture pin");
        let pin = smith_bench::read_configuration(&pin_file).expect("pin");
        let project = Path::new(env!("CARGO_MANIFEST_DIR"));
        let task = smith_bench::read_task(&project.join("tasks/probes/first-request/task.toml")).expect("task");
        let environment = smith_bench::isolate::environment(
            &smith_bench::isolate::Machine { path: "/usr/bin:/bin".into(), rustup_home: None },
            &private,
            &std::collections::BTreeMap::new(),
        )
        .expect("environment");
        Self {
            _scratch: scratch,
            source: root.join("logins/smith"),
            guard,
            private,
            pin,
            pin_file,
            task,
            arm,
            environment,
            model: smith_bench::ModelChoice { model: Some("gpt-6-luna".into()), effort: Some("low".into()) },
        }
    }

    pub(super) fn context(&self) -> smith_bench::agent::Context<'_> {
        let smith_bench::arm::ArmRef::Built(build) = &self.arm else { unreachable!("fixture arm") };
        smith_bench::agent::Context {
            guard: &self.guard,
            private: &self.private,
            pin: &self.pin,
            pin_file: &self.pin_file,
            task: &self.task,
            variant: None,
            model: &self.model,
            arm: &self.arm,
            installed_version: &build.arm.commit,
            home: &self.source,
            wall: skein_lib::Wall::from_nanos(100_000_000_000),
            lifetime: Duration::from_secs(20),
            environment: &self.environment,
        }
    }
}

#[test]
fn an_exec_arm_refused_by_check_is_a_setup_refusal() {
    let fixture = ExecFixture::new("exec-check-refused", 2);
    let refusal = smith_bench::agent::smith::prepare(&fixture.context()).expect_err("check refuses");
    assert_eq!(refusal.kind, smith_bench::FaultKind::Setup(smith_bench::isolate::SetupStep::Configuration));
    assert!(refusal.detail.contains("Code(2)"));
    assert!(!fixture.private.root.join("smith/tokens").exists());
}
