//! The process tier's owned Forgejo, credentials and landing scripts.
use super::{Forge, Scratch, owned, write_private};
use serde_json::{Value, json};
use skein_throwaway_forgejo::{Instance, Outgoing, Token};
use std::{
    fs,
    net::{SocketAddr, TcpListener},
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
};
use temper_real_loop_world::landing;
pub type BranchHead = (Box<[u8]>, Box<[u8]>);
// The landing story gathers its agent's brief before the person's next command.
const BRIEF_GATHER_MS: u64 = 5000;

/// Hold real host-executed jobs by run id (temper/docs/design/testing/process-tier.md).
pub struct JobBarrier {
    directory: PathBuf,
}
impl JobBarrier {
    #[must_use]
    pub fn new(root: &Path) -> Self {
        let directory = root.join("job-barrier");
        for part in ["", "started", "release"] {
            fs::DirBuilder::new().mode(0o700).create(directory.join(part)).expect("private job barrier");
        }
        Self { directory }
    }

    #[must_use]
    pub fn gate_workflow(&self, workflow: &str) -> String {
        let directory = self.directory.to_str().expect("barrier path").replace('\'', "'\\''");
        let step = format!(
            r#"      - run: |
          umask 077
          barrier='{directory}'
          run="${{{{ github.run_id }}}}"
          : > "$barrier/started/$run"
          while [ ! -f "$barrier/release/$run" ]; do
            kill -0 {} 2>/dev/null || exit 1
            sleep 0.1
          done
      - run: |"#,
            std::process::id()
        );
        assert!(workflow.contains("      - run: |"), "gate has a check step");
        workflow.replacen("      - run: |", &step, 1)
    }

    /// Keep the selected ids in the started marker before blocking the run.
    #[must_use]
    pub fn gate_workflow_by_test(&self, workflow: &str) -> String {
        self.gate_workflow(workflow).replace(
            ": > \"$barrier/started/$run\"",
            "printf '%s' '${{ inputs.temper_tests }}' > \"$barrier/$run\"\n          mv \"$barrier/$run\" \"$barrier/started/$run\"",
        )
    }

    #[must_use]
    pub(super) fn selection(&self, run: u64) -> Vec<u8> {
        fs::read(self.directory.join("started").join(run.to_string())).expect("started selection")
    }

    #[must_use]
    pub fn started(&self, run: u64) -> bool {
        self.directory.join("started").join(run.to_string()).is_file()
    }

    pub fn release(&self, run: u64) {
        assert!(self.started(run), "release a started run");
        write_private(&self.directory.join("release").join(run.to_string()), b"");
    }

    pub(super) fn waiting(&self, previous: Option<u64>) -> Option<u64> {
        fs::read_dir(self.directory.join("started")).expect("started jobs").find_map(|entry| {
            let run = entry.expect("job marker").file_name().to_str().expect("run id").parse().expect("numeric run");
            (Some(run) != previous
                && self.started(run)
                && !self.directory.join("release").join(run.to_string()).is_file())
            .then_some(run)
        })
    }
}

/// The owned Forgejo of testing/process-tier.md, prepared and run by temper's
/// commands with its runner, attached through skein's `Instance`, with a kept
/// bot, the person's token and repository.
pub struct Throwaway {
    instance: Instance,
    pid: u32,
    forgejo: PathBuf,
    runner: Option<owned::Running>,
    person: Option<Token>,
    repository: Option<u64>,
    directory: PathBuf,
    webhooks: Option<SocketAddr>,
}
impl Throwaway {
    #[must_use]
    pub fn start(scratch: &Scratch) -> Self {
        Self::start_with_job_sleep(scratch, 60)
    }

    #[must_use]
    pub fn start_with_job_sleep(scratch: &Scratch, sleep_seconds: u64) -> Self {
        Self::start_bare(scratch).with_repository(scratch, sleep_seconds, &workflow("'temper/**'"))
    }

    /// Keep the test's delivery receiver bound before configuring Forgejo's hook.
    #[must_use]
    pub fn start_receiving(scratch: &Scratch) -> (Self, TcpListener) {
        let [listener] = super::reserve_listeners(&[]);
        let address = listener.local_addr().expect("delivery receiver address");
        let fixture =
            Self::start_bare_with_webhooks(scratch, Some(address), true).with_repository(scratch, 0, &workflow("main"));
        (fixture, listener)
    }

    #[must_use]
    pub(super) fn with_repository(mut self, scratch: &Scratch, sleep_seconds: u64, workflow: &str) -> Self {
        let began = skein_shell::Clock::new().now().now;
        seed(scratch.path(), &mut self.instance);
        stage("seed forge", began);
        let person = person_token(&self.instance);
        let repository = repository(&self.instance, &person, sleep_seconds, workflow);
        owned::adopt(scratch.path(), &self.instance, &person, self.webhooks);
        self.person = Some(person);
        self.repository = Some(repository);
        self
    }

    /// The pinned server and runner with their administrator, before the fixture bot or adoption.
    #[must_use]
    pub fn start_bare(scratch: &Scratch) -> Self {
        Self::start_bare_with_webhooks(scratch, None, true)
    }

    /// The pinned server alone, for a story that never waits for Actions.
    #[must_use]
    pub fn start_without_runner(scratch: &Scratch) -> Self {
        Self::start_bare_with_webhooks(scratch, None, false)
    }

    fn start_bare_with_webhooks(scratch: &Scratch, webhooks: Option<SocketAddr>, runner: bool) -> Self {
        let address = owned::tier_configuration_with_runner(scratch.path(), runner);
        if let Some(webhooks) = webhooks {
            let path = scratch.path().join(temper_config_shell::FILE);
            let mut file = super::read_file(&path, temper_config_shell::Shape::Standalone);
            file.webhooks = Some(webhooks);
            super::write_file(&path, &file, temper_config_shell::Shape::Standalone);
        }
        let running = owned::start(scratch.path());
        let pid = running.id();
        let began = skein_shell::Clock::new().now().now;
        let instance = owned::attach(scratch.path(), address.port(), running);
        stage("attach", began);
        let runner = runner.then(|| {
            let began = skein_shell::Clock::new().now().now;
            let mut runner = owned::run(scratch.path(), "runner");
            runner.declared();
            stage("runner", began);
            runner
        });
        let webhooks =
            super::read_file(&scratch.path().join(temper_config_shell::FILE), temper_config_shell::Shape::Standalone)
                .webhooks;
        Self {
            instance,
            pid,
            forgejo: owned::binaries().0,
            runner,
            person: None,
            repository: None,
            directory: scratch.path().into(),
            webhooks,
        }
    }

    /// Forgejo's pid, which `temper forge run` became.
    #[must_use]
    pub fn pid(&self) -> u32 {
        self.pid
    }

    #[must_use]
    pub fn forge(&self) -> Forge<'_> {
        Forge::Throwaway {
            forgejo: &self.forgejo,
            api: SocketAddr::from(([127, 0, 0, 1], self.instance.port())),
            git: self.instance.base(),
        }
    }
    fn get(&self, suffix: &str) -> skein_throwaway_forgejo::Exchange {
        self.instance
            .exchange(&Outgoing::get(
                &format!("/api/v1/repos/acme/widget/{suffix}"),
                Some(self.person.as_ref().expect("fixture person")),
            ))
            .expect("throwaway forge answer")
    }
    #[must_use]
    pub fn merged_pull(&self) -> Option<u64> {
        let response = self.get("pulls?state=closed&limit=2");
        assert_eq!(response.status, 200, "closed pull list");
        let value: Value = serde_json::from_slice(&response.body).expect("pull JSON");
        value.as_array().expect("pull list").iter().find_map(|pull| {
            (pull["merged"].as_bool() == Some(true)).then(|| pull["number"].as_u64().expect("pull number"))
        })
    }
    #[must_use]
    pub fn main_readme(&self) -> Vec<u8> {
        let response = self.get("raw/README.md?ref=main");
        assert_eq!(response.status, 200, "main README");
        response.body
    }
    #[must_use]
    pub fn branch_head(&self, branch: &[u8]) -> Option<Box<[u8]>> {
        let branch = std::str::from_utf8(branch).expect("branch words");
        let response = self.get(&format!("branches/{branch}"));
        if response.status == 404 {
            return None;
        }
        assert_eq!(response.status, 200, "branch answer");
        let value: Value = serde_json::from_slice(&response.body).expect("branch JSON");
        Some(value["commit"]["id"].as_str().expect("branch commit").as_bytes().into())
    }
    #[must_use]
    pub fn branch_heads(&self) -> Vec<BranchHead> {
        let response = self.get("branches?limit=8");
        assert_eq!(response.status, 200, "branch list");
        let value: Value = serde_json::from_slice(&response.body).expect("branches JSON");
        value
            .as_array()
            .expect("branches")
            .iter()
            .map(|branch| {
                (
                    branch["name"].as_str().expect("branch name").as_bytes().into(),
                    branch["commit"]["id"].as_str().expect("branch commit").as_bytes().into(),
                )
            })
            .collect()
    }
    #[must_use]
    pub fn failure_state(&self) -> String {
        let response = self.get("pulls?state=all&limit=8");
        assert_eq!(response.status, 200, "all pull list");
        let pulls: Value = serde_json::from_slice(&response.body).expect("pull JSON");
        let response = self.get("branches?limit=8");
        assert_eq!(response.status, 200, "branch list");
        let branches: Value = serde_json::from_slice(&response.body).expect("branches JSON");
        let names: Vec<_> = branches
            .as_array()
            .expect("branches")
            .iter()
            .map(|branch| branch["name"].as_str().expect("branch name"))
            .collect();
        format!(
            "{} pulls, merged={}, branches={names:?}, main edited={}",
            pulls.as_array().expect("pulls").len(),
            self.merged_pull().is_some(),
            self.main_readme().ends_with(super::EDIT)
        )
    }
    pub fn stop(self) {
        if let Some(runner) = self.runner {
            runner.stop();
        }
        self.instance.stop().expect("owned Forgejo stopped");
    }
    #[must_use]
    pub fn repository(&self) -> u64 {
        self.repository.expect("fixture repository")
    }
    /// The seeded webhook secret, without its optional final line feed.
    #[must_use]
    pub fn secret(&self) -> Vec<u8> {
        let secret = fs::read(self.directory.join("forge-1/webhook_secret")).expect("webhook secret");
        secret.strip_suffix(b"\n").unwrap_or(&secret).to_vec()
    }
    /// Commits one file to main through the person's contents API.
    pub fn push(&self, file: &str) {
        self.add_file(file, b"hook\n");
    }

    /// Add a fixture file through the person's API, retaining its exact bytes.
    pub fn add_file(&self, file: &str, content: &[u8]) {
        let path = format!("/api/v1/repos/acme/widget/contents/{file}");
        let size = skein_lib::base64::encoded_len(
            skein_lib::base64::Alphabet::Standard,
            u32::try_from(content.len()).expect("fixture file size"),
        )
        .expect("base64 size");
        let mut encoded = skein_lib::Writer::new(usize::try_from(size).expect("base64 fits"));
        skein_lib::base64::encode(skein_lib::base64::Alphabet::Standard, content, &mut encoded).expect("base64 file");
        let encoded = encoded.finish();
        let content = std::str::from_utf8(&encoded).expect("base64 text");
        let body = serde_json::to_vec(&json!({"branch":"main","message":format!("Add {file}"),"content":content}))
            .expect("contents JSON");
        let response = self
            .instance
            .exchange(&Outgoing::json("POST", &path, Some(self.person.as_ref().expect("fixture person")), &body))
            .expect("contents answer");
        assert_eq!(response.status, 201, "contents commit");
    }

    /// Cancel the runner's live job as the repository's person, outside temper.
    pub fn cancel_run(&self, run: u64) {
        let response = self
            .instance
            .exchange(&Outgoing::json(
                "POST",
                &format!("/api/v1/repos/acme/widget/actions/runs/{run}/cancel"),
                Some(self.person.as_ref().expect("fixture person")),
                b"null",
            ))
            .expect("cancel answer");
        assert_eq!(response.status, 204, "cancel running gate");
    }
}

/// Seed the process fixture's bot and its kept access before serving it.
pub fn seed(directory: &Path, instance: &mut Instance) {
    bootstrap(directory, directory, directory);
    let bot = instance.create_user("temper").expect("fixture bot");
    let address = ([127, 0, 0, 1], instance.port()).into();
    let mut record = temper_real_loop_world::fixtures::forgejo_record(address);
    record.bot.login = b"temper".as_slice().into();
    record.mode = temper_engine_domain_forge::Mode::Owned;
    request(
        instance,
        &instance.admin().token,
        "PATCH",
        "/api/v1/admin/users/temper",
        &json!({"login_name":"temper","source_id":0,"full_name":"temper:bot","must_change_password":false}),
        200,
    );
    let hook = skein_throwaway_forgejo::fresh_secret().expect("fixture hook secret");
    write_private(&directory.join("forge-1/webhook_secret"), hook.as_bytes());
    temper_real_loop_world::fixtures::seed_forgejo(
        &directory.join("state"),
        record,
        bot.id,
        bot.token.value().as_bytes(),
        hook.as_bytes(),
        Some(instance.admin().token.value().as_bytes()),
    );
}

/// Let the first service start commit stock settings and owners before adding fixture records.
/// temper/docs/design/protocol/forge/engine.md, 1; jig's start owns the initial store mark.
pub fn bootstrap(config: &Path, data: &Path, cache: &Path) {
    use skein_world::end_to_end::{Binary, Command, Mode};
    let path = config.join(temper_config_shell::FILE);
    let original = fs::read(&path).expect("bootstrap configuration");
    let (shape, binary) = if super::read_file_shape(&path) == temper_config_shell::Shape::EngineAlone {
        (temper_config_shell::Shape::EngineAlone, "temper-engine")
    } else {
        (temper_config_shell::Shape::Standalone, "temper-box")
    };
    let mut file = super::read_file(&path, shape);
    // A delivery story keeps its listener bound in the enclosing test.
    file.webhooks = Some("127.0.0.1:0".parse().expect("bootstrap webhooks"));
    super::write_file(&path, &file, shape);
    super::limit_settings::install_paths(&[config.into(), data.into(), cache.into()]);
    let binary = Binary::start(
        Command {
            program: skein_scratch::paths::built("temper-box", "temper-box").with_file_name(binary),
            arguments: vec![
                "serve".into(),
                "--config".into(),
                config.into(),
                "--data".into(),
                data.into(),
                "--cache".into(),
                cache.into(),
            ],
            directory: config.into(),
            environment: Vec::new(),
        },
        Mode::Pipes,
        65_536,
    )
    .expect("fixture bootstrap service");
    let ended = super::webhooks::served_binary(binary, "127.0.0.1:0".parse().expect("unused listener"), &[]);
    write_private(&path, &original);
    assert_eq!(
        ended.exit,
        Some(skein_io::kernel::Exit::Code(0)),
        "fixture bootstrap: {}",
        String::from_utf8_lossy(&ended.stderr)
    );
}

/// Apply the throwaway forge's policy and limits to the recipe, returning its prospective agent.
#[must_use]
pub(crate) fn configuration(
    owned: &owned::Owned,
    value: &mut temper_playground::documents::Configuration,
    mut agent: Value,
) -> Value {
    value["application"] = owned::application(owned);
    value.temper_over("root.brief.gather_ms", BRIEF_GATHER_MS);
    temper_real_loop_world::settings::stage_policies(value);
    landing::configure(value, &mut agent, owned.forgejo);
    value.as_object_mut().expect("deployment").remove("world_settings");
    super::configuration::use_stock(value);
    // Nine slots hold the stock charters.
    value.jig_over("tasks.charters", 16);
    value.jig_over("core.templates", 7);
    // The process uses stock's fourteen grants, beyond the world's ten-grant policy.
    value.jig_over("tasks.authority_grants", 16);
    // The creation offer carries nine stock charters and four procedures.
    value.jig_over("documents.lines", 12);
    super::configuration::turn_bounds::apply(value, super::configuration::turn_bounds::turn_body(&agent));
    agent
}

#[must_use]
pub fn landing_scripts(answer: &[u8], forgejo: &Throwaway) -> Box<[skein_fake_llm_domain::api::Script]> {
    let Forge::Throwaway { api, .. } = forgejo.forge() else { panic!("throwaway forge") };
    let repository = u32::try_from(forgejo.repository()).expect("forge-local repository fits u32");
    native_scripts(answer, api, repository, &forgejo.main_readme())
}

#[must_use]
pub fn native_scripts(
    answer: &[u8],
    api: SocketAddr,
    repository: u32,
    readme: &[u8],
) -> Box<[skein_fake_llm_domain::api::Script]> {
    let mut scripts = landing::change_scripts(
        landing::ChangeAuthority { coding_kind: 2, scope: Some((api, "acme")) },
        temper_engine_domain_forge::tag(1, repository),
        readme,
        landing::EDIT,
        b"@forge_chat",
        "@coding Fix",
    )
    .into_vec();
    let skein_fake_llm_domain::api::Line::Call { arguments, .. } = &mut scripts[0].turns[0].lines[0] else {
        panic!("delegate call")
    };
    let mut delegate: Value = serde_json::from_slice(arguments).expect("delegate JSON");
    let models = temper_service::stock::models(&jig_protocol_config::Llm {
        providers: Box::new([]),
        endpoints: Box::new([]),
        models: Box::new([]),
    });
    let stock = temper_service::stock::with_models(models, 10_660);
    delegate["batch"][0]["authority"]["budget"]["spend"] = json!(stock.rules.maximum_run_spend * 2);
    *arguments = serde_json::to_vec(&delegate).expect("funded delegate JSON").into_boxed_slice();
    let mut initial = super::chat_script(answer);
    initial.cue = b"Hello".as_slice().into();
    scripts.push(initial);
    scripts.into_boxed_slice()
}

fn stage(name: &str, began: skein_lib::Time) {
    eprintln!(
        "stage {name}: {} ms",
        skein_shell::Clock::new().now().now.saturating_since(began).as_nanos() / 1_000_000
    );
}
fn request(instance: &Instance, token: &Token, method: &str, path: &str, body: &Value, status: u16) -> Value {
    let body = serde_json::to_vec(body).expect("request JSON");
    let response = instance.exchange(&Outgoing::json(method, path, Some(token), &body)).expect("Forgejo answer");
    assert_eq!(response.status, status, "{method} {path}");
    serde_json::from_slice(&response.body).expect("answer JSON")
}
fn person_token(instance: &Instance) -> Token {
    let began = skein_shell::Clock::new().now().now;
    let admin = &instance.admin().token;
    request(
        instance,
        admin,
        "PATCH",
        "/api/v1/admin/users/ada",
        &json!({"login_name":"ada","source_id":0,"must_change_password":false}),
        200,
    );
    let value = request(
        instance,
        admin,
        "POST",
        "/api/v1/admin/users/ada/tokens",
        &json!({"name":"person","scopes":["write:repository","write:organization"]}),
        201,
    );
    stage("person", began);
    Token::new(value["sha1"].as_str().expect("person token").into())
}
fn repository(instance: &Instance, person: &Token, sleep_seconds: u64, workflow: &str) -> u64 {
    let began = skein_shell::Clock::new().now().now;
    request(instance, person, "POST", "/api/v1/orgs", &json!({"username":"acme","visibility":"private"}), 201);
    let value = request(
        instance,
        person,
        "POST",
        "/api/v1/orgs/acme/repos",
        &json!({"name":"widget","auto_init":true,"default_branch":"main","private":true}),
        201,
    );
    request(instance, person, "PATCH", "/api/v1/repos/acme/widget", &json!({"has_actions":true}), 200);
    let check = format!(
        "test ! -e slow || sleep {sleep_seconds}\n\
         for file in batch-*.txt; do\n\
           [ -f \"$file\" ] || continue\n\
           if grep -qx red \"$file\"; then exit 1; fi\n\
         done\n\
         test -s README.md\n"
    );
    for (path, content) in [("check.sh", check.as_str()), (".forgejo/workflows/check.yaml", workflow)] {
        let length = skein_lib::base64::encoded_len(
            skein_lib::base64::Alphabet::Standard,
            u32::try_from(content.len()).expect("workflow length"),
        )
        .expect("base64 length");
        let mut encoded = skein_lib::Writer::new(usize::try_from(length).expect("bounded length"));
        skein_lib::base64::encode(skein_lib::base64::Alphabet::Standard, content.as_bytes(), &mut encoded)
            .expect("bounded workflow");
        let encoded = encoded.finish();
        let content = std::str::from_utf8(&encoded).expect("base64 text");
        request(
            instance,
            person,
            "POST",
            &format!("/api/v1/repos/acme/widget/contents/{path}"),
            &json!({"branch":"main","message":"Add check","content":content}),
            201,
        );
    }
    stage("repository", began);
    value["id"].as_u64().expect("repository id")
}
fn workflow(branch: &str) -> String {
    format!("on: {{ push: {{ branches: [{branch}] }} }}\n{CHECK_JOB}")
}

const CHECK_JOB: &str = r#"jobs:
  check:
    runs-on: temper-host
    steps:
      - run: |
          auth=$(printf 'x-access-token:%s' "${{ github.token }}" | base64 -w0)
          git -c "http.extraHeader=Authorization: basic $auth" clone "${{ github.server_url }}/${{ github.repository }}.git" .
          git checkout "${{ github.sha }}" && sh check.sh
"#;

// E4 observes a distinct managed deployment beside this owned fixture.
impl Throwaway {
    /// Mint a disposable admin paste, keeping its bytes out of argv and the environment.
    pub fn admin_token(&self, path: &Path) {
        let output = std::process::Command::new(&self.forgejo)
            .args([
                "--work-path",
                self.directory.join("forge-1/forgejo").to_str().expect("work path"),
                "--config",
                self.directory.join("forge-1/forgejo/app.ini").to_str().expect("config path"),
                "admin",
                "user",
                "generate-access-token",
                "--username",
                "temper-admin",
                "--token-name",
                "elsewhere-disposable",
                "--raw",
                "--scopes",
                "write:admin,read:user,write:organization,write:repository",
            ])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("TZ", "UTC")
            .env("HOME", self.directory.join("forge-1/home"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", self.directory.join("forge-1/home/gitconfig"))
            .current_dir(self.directory.join("forge-1/forgejo"))
            .output()
            .expect("mint disposable admin token");
        assert!(output.status.success(), "disposable token command failed (output redacted)");
        let token = output.stdout.strip_suffix(b"\n").unwrap_or(&output.stdout);
        assert!(!token.is_empty(), "nonempty disposable token");
        assert!(!output.stderr.windows(token.len()).any(|part| part == token), "token in CLI stderr (redacted)");
        write_private(path, token);
    }

    /// The fixture's original admin credential survives Reconcile, never `TakeOver`.
    pub fn witness_authenticates(&self) {
        let response = self
            .instance
            .exchange(&Outgoing::get("/api/v1/user", Some(&self.instance.admin().token)))
            .expect("witness API answer");
        assert_eq!(response.status, 200, "original admin still authenticates");
    }

    #[must_use]
    pub fn repository_observation(&self, org: &str, name: &str, suffix: &str) -> Value {
        let response = self
            .instance
            .exchange(&Outgoing::get(
                format!("/api/v1/repos/{org}/{name}/{suffix}").trim_end_matches('/'),
                Some(&self.instance.admin().token),
            ))
            .expect("elsewhere repository answer");
        assert_eq!(response.status, 200, "elsewhere observation");
        serde_json::from_slice(&response.body).expect("elsewhere JSON")
    }

    /// A merge train's members use Forgejo's manual-merge detection (domain/landing.md, 10).
    pub fn detect_manual_merges(&self) {
        let response = self
            .instance
            .exchange(&Outgoing::get("/api/v1/repos/acme/widget", Some(&self.instance.admin().token)))
            .expect("repository settings");
        assert_eq!(response.status, 200);
        let before: Value = serde_json::from_slice(&response.body).expect("repository settings JSON");
        // Forgejo edits pull-request options through the enabled repository unit.
        let mut options = json!({"has_pull_requests":true,"allow_manual_merge":true,"autodetect_manual_merge":true});
        for name in [
            "allow_merge_commits",
            "allow_rebase",
            "allow_rebase_explicit",
            "allow_squash_merge",
            "allow_fast_forward_only_merge",
            "allow_rebase_update",
            "default_merge_style",
            "default_delete_branch_after_merge",
            "default_allow_maintainer_edit",
        ] {
            options[name] = before[name].clone();
        }
        request(&self.instance, &self.instance.admin().token, "PATCH", "/api/v1/repos/acme/widget", &options, 200);
    }

    /// Remove a hook directly on the forge, so the next reconcile must make it again.
    pub fn remove_repository_hook(&self, org: &str, name: &str, id: u64) {
        let response = self
            .instance
            .exchange(&Outgoing::json(
                "DELETE",
                &format!("/api/v1/repos/{org}/{name}/hooks/{id}"),
                Some(&self.instance.admin().token),
                b"null",
            ))
            .expect("remove hook answer");
        assert_eq!(response.status, 204, "remove repository hook");
    }

    #[must_use]
    pub fn repositories_in(&self, org: &str) -> Value {
        let response = self
            .instance
            .exchange(&Outgoing::get(&format!("/api/v1/orgs/{org}/repos?limit=50"), Some(&self.instance.admin().token)))
            .expect("elsewhere repositories answer");
        assert_eq!(response.status, 200, "elsewhere repository list");
        serde_json::from_slice(&response.body).expect("repositories JSON")
    }
}
