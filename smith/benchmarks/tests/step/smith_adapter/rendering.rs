//! Smith settings render over the benchmark test tree, then enter the product's startup parser.
//! See `smith/docs/design/benchmarks/README.md`, section 13.

use skein_fake_machine::Item;
use skein_lib::{Duration, Wall};
use smith_bench::agent::smith::product_settings_with;
use smith_bench::agent::{Context, PinnedConfiguration};
use smith_bench::arm::{ArmRef, Build, BuiltArm};
use smith_bench::files::Files;
use smith_bench::guard::Guard;
use smith_bench::isolate::{Machine, Private, environment};
use smith_bench::testing::task::parse_task;
use smith_bench::{Fault, ModelChoice, Task, read_configuration_with};
use smith_bench_fixtures::files::Tree;
use smith_local_shell::{command, settings};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;

const PIN_FILE: &str = "/agents/smith/standard.pin.toml";
const CASE_ROOT: &str = "/attempt";
pub(super) const LOGIN: &str = "/logins/smith";

pub(super) struct Fixture {
    pub(super) tree: Tree,
    guard: Guard,
    private: Private,
    pin: PinnedConfiguration,
    arm: ArmRef,
    model: ModelChoice,
    environment: Vec<(OsString, OsString)>,
}

impl Fixture {
    pub(super) fn new(extra: &[Item]) -> Self {
        let root = Path::new(CASE_ROOT);
        let private = Private {
            root: root.into(),
            workspace: root.join("workspace"),
            home: root.join("home"),
            temporary: root.join("tmp"),
            cargo_home: root.join("cargo"),
            target: root.join("target"),
            warm_cargo: root.join("warm-cargo"),
            warm_target: root.join("warm-target"),
            config: root.join("config"),
            state: root.join("state"),
        };
        let mut items = vec![
            Item::directory(b"attempt"),
            Item::directory(b"attempt/workspace"),
            Item::directory(b"agents"),
            Item::directory(b"agents/smith"),
            Item::file(b"agents/smith/standard.pin.toml", include_bytes!("../../../agents/smith/standard.pin.toml")),
            Item::file(b"agents/smith/standard.toml", include_bytes!("../../../agents/smith/standard.toml")),
        ];
        items.extend_from_slice(extra);
        let tree = Tree::new(&items);
        let guard =
            Guard::new_with(&tree, Path::new("/user"), Path::new("/config"), Path::new("/state"), &[]).expect("guard");
        let pin = read_configuration_with(&tree, Path::new(PIN_FILE)).expect("pin");
        let environment =
            environment(&Machine { path: "/usr/bin:/bin".into(), rustup_home: None }, &private, &BTreeMap::new())
                .expect("rendered environment");
        let arm = ArmRef::Built(Build {
            directory: "/supplied-arm".into(),
            arm: BuiltArm {
                version: 1,
                commit: "a".repeat(40),
                sha256: "b".repeat(64),
                toolchain: "supplied identity".into(),
                binaries: Vec::new(),
            },
        });
        let model = ModelChoice { model: Some("gpt-6-luna".into()), effort: Some("low".into()) };
        Self { tree, guard, private, pin, arm, model, environment }
    }

    pub(super) fn context<'a>(&'a self, task: &'a Task) -> Context<'a> {
        let ArmRef::Built(build) = &self.arm else { unreachable!("supplied arm") };
        Context {
            guard: &self.guard,
            private: &self.private,
            pin: &self.pin,
            pin_file: Path::new(PIN_FILE),
            task,
            variant: None,
            model: &self.model,
            arm: &self.arm,
            installed_version: &build.arm.commit,
            home: Path::new(LOGIN),
            wall: Wall::EPOCH,
            lifetime: Duration::from_secs(20),
            environment: &self.environment,
        }
    }

    pub(super) fn render(&self, context: &Context<'_>) -> Result<Rendered, Fault> {
        let path = context.pin_file.parent().expect("pin directory").join(&context.pin.pin.configuration);
        let base: toml::Value = toml::from_str(&self.tree.read_to_string(&path).expect("user layer")).expect("TOML");
        let base = serde_json::to_value(base).expect("layer");
        let mut layer = product_settings_with(&self.tree, context, &base, context.home)?;
        let invocation = layer.as_object_mut().expect("layer object").remove("invocation").expect("rendered flags");
        let arguments = serde_json::from_value(invocation).expect("string flags");
        let value: toml::Value = serde_json::from_value(layer).expect("layer values");
        let source = settings::Source {
            layer: settings::Layer::User,
            path: context.private.config.join("smith/config.toml"),
            text: toml::to_string(&value).expect("layer TOML"),
        };
        Ok(Rendered { arguments, source })
    }
}

#[derive(Debug)]
pub(super) struct Rendered {
    pub(super) arguments: Vec<String>,
    source: settings::Source,
}

impl Rendered {
    pub(super) fn settings(&self) -> settings::Settings {
        self.settings_from(std::slice::from_ref(&self.source))
    }

    fn settings_from(&self, sources: &[settings::Source]) -> settings::Settings {
        let command = command::parse(self.arguments.iter().map(Into::into).collect()).expect("rendered flags");
        let command::Command::Interactive { shared, .. } = command else { panic!("shared flags") };
        let workspace = shared.directory.clone().expect("workspace flag");
        settings::from_sources(&shared, workspace, sources).unwrap_or_else(|why| panic!("{}", why.line()))
    }

    pub(super) fn resolve(&self) -> settings::resolve::Resolved {
        let settings = self.settings_from(&[self.source.clone(), offline_transports()]);
        settings::resolve::resolve_startup(&settings).unwrap_or_else(|why| panic!("{}", why.line())).0
    }
}

fn offline_transports() -> settings::Source {
    // Keep startup's TLS parsing and admission, using loopback addresses and only the committed root bytes.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/protocol-llm/fixtures/root.der");
    let root = toml::Value::String(path.to_str().expect("fixture path").into());
    settings::Source {
        layer: settings::Layer::User,
        path: "/fixtures/transport.toml".into(),
        text: format!(
            r#"
[endpoints.openai]
address = "127.0.0.1:443"
trust = {{ system = false, der = [{root}] }}
[endpoints.anthropic]
address = "127.0.0.1:443"
trust = {{ system = false, der = [{root}] }}
[accounts.chatgpt]
token_url = "https://127.0.0.1:443/token"
trust = {{ der = [{root}] }}
"#
        ),
    }
}

pub(super) fn probe_directory() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tasks/probes")
}

pub(super) fn authored_task(name: &str) -> Task {
    read_probe(&probe_directory().join(name).join("task.toml"))
}

pub(super) fn read_probe(path: &Path) -> Task {
    parse_task(path, &std::fs::read_to_string(path).expect("authored task bytes")).expect("authored task")
}
