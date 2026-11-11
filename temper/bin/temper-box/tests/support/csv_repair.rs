//! A real owned-Forgejo failure and repair, on the first-repository fixtures.
use super::{
    ctl::{Call, Step},
    first_repository::{self, call},
};
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Line, Piece, Script};
use skein_lib::{Duration, Time};
use std::{
    fs,
    path::{Path, PathBuf},
};

const REPAIR: &str = "Repair the failed CI on the change branch";
const REPOSITORY: &str = "/api/v1/repos/work/hello";
fn now() -> Time {
    skein_shell::Clock::new().now().now
}

const NOTE: &str = "Widget prints its table with --csv.\n";
// The release-note gate a walk against Forgejo proved, recorded byte for byte on
// 2026-10-27. No action or external service.
const WORKFLOW: &str = r#"on:
  push:
    branches: [main]
  pull_request:
jobs:
  check:
    runs-on: temper-host
    steps:
      - run: echo "temper's starter check passed; replace this step with your own checks"
  changes:
    if: ${{ github.event_name == 'pull_request' }}
    runs-on: temper-host
    steps:
      - name: Each pull request adds changes/<its number>.md
        run: |
          echo 'pull_request.number=${{ github.event.pull_request.number }}'
          auth=$(printf 'x-access-token:%s' "${{ github.token }}" | base64 | tr -d '\n')
          git -c "http.extraHeader=Authorization: basic $auth" clone --quiet --no-checkout "${{ github.server_url }}/${{ github.repository }}.git" repo
          git -C repo checkout --quiet '${{ github.event.pull_request.head.sha }}'
          test -f 'repo/changes/${{ github.event.pull_request.number }}.md' || { echo 'Missing changes/${{ github.event.pull_request.number }}.md'; exit 1; }
"#;
const CONTRIBUTING: &str = "# Contributing\n\nEach pull request adds `changes/<its number>.md`: one line saying what it\nchanges, for the next release's notes. CI checks it.\n";

#[must_use]
pub fn llm(config: &Path) -> super::boxed::CtlLlm {
    let mut scripts = first_repository::csv_scripts().into_vec();
    let Line::Call { arguments, .. } = &mut scripts[0].turns[0].lines[0] else { panic!("delegate") };
    let mut delegate: Value = serde_json::from_slice(arguments).expect("delegate JSON");
    delegate["batch"][0]["contract"]["words"] = 512.into();
    delegate["batch"][0]["spec"]["words"] = "@hello-readme Add CSV export to work/hello".into();
    *arguments = serde_json::to_vec(&delegate).expect("delegate JSON").into_boxed_slice();
    // Keep the shared README, CSV source and pending-review script; the repair
    // supplies the release note whose absence made the owned runner fail.
    let mut write = call(b"write", &json!({"path":"changes/1.md", "content":NOTE}));
    let mut lines = write.lines.into_vec();
    lines.insert(
        0,
        Line::Pieces {
            pieces: Box::new([Piece {
                text: b"Repairing the missing release note.".as_slice().into(),
                hold: Some(b"csv-repair".as_slice().into()),
            }]),
        },
    );
    write.lines = lines.into_boxed_slice();
    scripts.push(Script {
        cue: REPAIR.as_bytes().into(),
        turns: Box::new([write, call(b"finish", &json!({"form":"change", "fields":{
            "title":"Add CSV export to widget", "body":"Widget prints its table with --csv and has its release note."
        }}))]),
    });
    first_repository::model(config, scripts.into_boxed_slice())
}

struct Forge {
    number: u64,
    address: String,
    header: PathBuf,
}
impl Forge {
    fn at(paths: &[PathBuf; 3], client: &Path) -> Self {
        let document: Value =
            serde_json::from_slice(&fs::read(paths[0].join("temper.json")).expect("config")).expect("JSON");
        let number = first_repository::owned_forge(client)["number"].as_u64().expect("forge number");
        Self {
            number,
            address: document["application"]["machine"]["owned"]["listen"]["socket"].as_str().expect("address").into(),
            header: paths[0].join("person.header"),
        }
    }
    fn person(paths: &[PathBuf; 3], client: &Path) -> Self {
        let forge = Self::at(paths, client);
        let document: Value =
            serde_json::from_slice(&fs::read(paths[0].join("temper.json")).expect("config")).expect("JSON");
        let login = document["application"]["machine"]["owned"]["person"]["login"].as_str().expect("person login");
        let work = paths[1].join(format!("forge-{}/forgejo", forge.number));
        let output = std::process::Command::new(super::owned::binaries().0)
            .arg("--work-path")
            .arg(&work)
            .arg("--config")
            .arg(work.join("app.ini"))
            .args(["admin", "user", "must-change-password", "--unset", login])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", paths[1].join(format!("forge-{}/home", forge.number)))
            .stdin(std::process::Stdio::null())
            .output()
            .expect("person password flag");
        assert!(output.status.success(), "clear person's first-password flag (output redacted)");
        let token = super::forge_land::scoped_token(paths, true, forge.number, login, "write:repository,read:issue");
        super::write_private(&forge.header, format!("Authorization: token {token}\n").as_bytes());
        forge
    }
    fn request(&self, method: &str, suffix: &str, body: Option<&Value>) -> Value {
        use std::process::Command;
        let mut command = Command::new("/usr/bin/curl");
        command
            .args(["--silent", "--show-error", "--fail", "--request", method, "--header"])
            .arg(format!("@{}", self.header.display()))
            .args(["--header", "Content-Type: application/json"])
            .arg(format!("http://{}{REPOSITORY}/{suffix}", self.address));
        if body.is_some() {
            command.args(["--data-binary", "@-"]);
        }
        let body = body.map(|body| serde_json::to_vec(body).expect("JSON"));
        let output = super::story_command::output(
            &mut command,
            "the CSV repair's forge request",
            Duration::from_secs(10),
            body.as_deref(),
        );
        assert!(output.status.success(), "{method} {suffix}: {}", String::from_utf8_lossy(&output.stderr));
        serde_json::from_slice(&output.stdout).expect("API JSON")
    }
    fn get(&self, suffix: &str) -> Value {
        self.request("GET", suffix, None)
    }
    fn statuses(&self, head: &str) -> Value {
        self.get(&format!("commits/{head}/status"))
    }
    fn install(&self) {
        let workflow = self.get("contents/.forgejo/workflows/ci.yml?ref=main");
        let previous = self.get("branches/main")["commit"]["id"].clone();
        let committed = self.request("POST", "contents", Some(&json!({
            "branch":"main", "message":"CI: each pull request adds changes/<its number>.md",
            "files":[
                {"operation":"update", "path":".forgejo/workflows/ci.yml", "sha":workflow["sha"], "content":encode(WORKFLOW)},
                {"operation":"create", "path":"CONTRIBUTING.md", "content":encode(CONTRIBUTING)}
            ]
        })));
        let head = committed["commit"]["sha"].as_str().expect("gate commit");
        let commit = self.get(&format!("git/commits/{head}"));
        assert_eq!(commit["parents"].as_array().expect("parents").len(), 1);
        assert_eq!(commit["parents"][0]["sha"], previous, "one person commit");
        assert_eq!(self.contents(".forgejo/workflows/ci.yml", head), WORKFLOW);
        assert_eq!(self.contents("CONTRIBUTING.md", head), CONTRIBUTING);
        let mut patience = super::patience::Patience::arm("the main gate's CI", Duration::from_secs(60));
        loop {
            let status = self.statuses(head);
            if status["state"] == "success" {
                assert_contexts(&status, "push", "skipped");
                break;
            }
            if let Some(figures) = patience.failed() {
                panic!("main gate CI phase: {status}; {figures}");
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let protection = self.get("branch_protections");
        for rule in protection.as_array().expect("protection rules") {
            assert!(
                rule["status_check_contexts"].as_array().is_none_or(Vec::is_empty),
                "the combined CI verdict must include both contexts: {rule}"
            );
        }
    }
    fn contents(&self, path: &str, head: &str) -> String {
        let value = self.get(&format!("contents/{path}?ref={head}"));
        let content = value["content"].as_str().expect("base64 content");
        let bytes = skein_lib::base64::decode(skein_lib::base64::Alphabet::Standard, content.as_bytes(), 1 << 20)
            .expect("file bytes");
        String::from_utf8(bytes.into_vec()).expect("UTF-8 file")
    }
    fn head(&self, panel: &Value, changes: &str) {
        let pull = self.get("pulls/1");
        assert_eq!(pull["number"], 1);
        let head = pull["head"]["sha"].as_str().expect("pull head");
        assert_eq!(panel["head"], head, "ctl and Forgejo agree at {changes}");
        assert_eq!(panel["pull"]["address"], format!("http://{}/work/hello/pulls/1", self.address));
        // ctl and the API are separate samples. Forgejo can publish another
        // pending context after a brief success containing only the first job.
        // Accept the repaired head's CI only after both jobs have finished.
        let mut patience = super::patience::Patience::arm("the change head's two CI contexts", Duration::from_secs(60));
        let statuses = loop {
            let statuses = self.statuses(head);
            if changes == "failure"
                || (statuses["state"] == "success"
                    && statuses["statuses"]
                        .as_array()
                        .is_some_and(|rows| rows.len() == 2 && rows.iter().all(|row| row["status"] == "success")))
            {
                break statuses;
            }
            if let Some(figures) = patience.failed() {
                panic!("H2's two CI contexts phase: {statuses}; {figures}");
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        };
        assert_eq!(statuses["state"], if changes == "failure" { "failure" } else { "success" });
        assert_contexts(&statuses, "pull_request", changes);
        let files = self.get("pulls/1/files");
        let names: Vec<_> = files
            .as_array()
            .expect("PR files")
            .iter()
            .map(|file| file["filename"].as_str().expect("filename"))
            .collect();
        if changes == "failure" {
            assert_eq!(names, ["Cargo.toml", "README.md", "src/main.rs"], "H1 has no release note");
        } else {
            assert!(names.contains(&"src/main.rs") && names.contains(&"changes/1.md"));
            assert_eq!(self.contents("changes/1.md", head), NOTE);
        }
        eprintln!("owned CSV {changes}: head {head}; statuses {}", statuses["statuses"]);
    }
}
fn encode(text: &str) -> String {
    let mut writer = skein_lib::Writer::new(text.len().div_ceil(3) * 4);
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Standard, text.as_bytes(), &mut writer).expect("base64");
    String::from_utf8(writer.finish().into_vec()).expect("base64 text")
}
fn assert_contexts(status: &Value, event: &str, changes: &str) {
    let rows = status["statuses"].as_array().expect("two status contexts");
    assert_eq!(rows.len(), 2, "{status}");
    for (job, state) in [("check", "success"), ("changes", changes)] {
        let context = format!("/ {job} ({event})");
        let row = rows.iter().find(|row| row["context"] == context).expect("exact Forgejo job context");
        assert_eq!(row["status"], state, "{context}: {status}");
    }
}

#[must_use]
pub fn task_steps(paths: &[PathBuf; 3], client: &Path, began: Time) -> Vec<Step> {
    let gate_paths = paths.clone();
    let gate_client = client.to_path_buf();
    let mut steps = vec![Step::Then(Box::new(move |_| {
        Forge::person(&gate_paths, &gate_client).install();
        Call::new(&["forge", "list", "--json"])
    }))];
    let mut task = first_repository::csv_task_steps(client);
    let review = task.split_off(5);
    steps.extend(task);
    steps.push(Step::Until(Call::new(&["change", "show", "2", "--json"]), br#""step":"repairing""#));
    let red_paths = paths.clone();
    let red_client = client.to_path_buf();
    steps.push(Step::Then(Box::new(move |calls| {
        let panel = super::ctl::json(calls.last().expect("repairing panel"));
        assert_eq!(panel["ci"], "failed");
        assert_eq!(panel["repairs"]["used"], 1);
        Forge::at(&red_paths, &red_client).head(&panel, "failure");
        eprintln!("owned CSV H1 failed: {} ms", now().saturating_since(began).as_nanos() / 1_000_000);
        Call::new(&["change", "show", "2", "--json"])
    })));
    steps.push(Step::ReleaseLlm { name: b"csv-repair", cue: REPAIR.as_bytes() });
    steps.extend(review);
    steps
}

#[must_use]
pub fn landed(paths: &[PathBuf; 3], client: &Path, began: Time) -> Value {
    let panel = super::ctl::alone(client, Call::new(&["change", "show", "2", "--json"]));
    let panel = super::ctl::json(&panel);
    let forge = Forge::at(paths, client);
    let pull = forge.get("pulls/1");
    assert_eq!(panel["step"], "landed");
    assert_eq!(panel["ci"], "passed");
    forge.head(&panel, "success");
    assert_eq!(pull["merged"], true);
    assert_eq!(pull["head"]["sha"], panel["head"]);
    assert_eq!(pull["merge_commit_sha"], panel["merge"]);
    let main = forge.get("branches/main");
    assert_eq!(main["commit"]["id"], panel["merge"]);
    let merge = panel["merge"].as_str().expect("merge SHA");
    assert_eq!(forge.contents("changes/1.md", merge), NOTE);
    assert!(forge.contents("src/main.rs", merge).contains("name,count"), "main holds CSV header");
    eprintln!("owned CSV landed: {} ms; {panel}", now().saturating_since(began).as_nanos() / 1_000_000);
    panel
}

pub fn assert_briefs(model: &super::forge_land::Seen) {
    let brief = |cue: &str| {
        model
            .requests
            .iter()
            .find_map(|query| {
                let system = String::from_utf8_lossy(&query.system);
                system.contains(cue).then(|| system.into_owned())
            })
            .unwrap_or_else(|| panic!("model never received {cue}"))
    };
    let repair = brief(REPAIR);
    eprintln!("owned CSV repair brief: {repair}");
    assert!(repair.contains("/ changes (pull_request)"), "repair brief names the failing context: {repair}");
    assert!(repair.contains("Missing changes/1.md"), "repair brief carries the real failed step's output: {repair}");
}
