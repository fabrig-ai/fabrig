//! A copy through the shipped ctl and fake box (temper/docs/design/domain/forge.md, 4.5).
use super::{Act, Call, Walk, adoption, ctl, path};
use std::path::Path;

fn document(sandbox: &Path) -> temper_fake::forge::Document {
    temper_fake::forge::Document::decode(
        &std::fs::read(sandbox.join("config/forgejo.json")).expect("retained fake Forgejo document"),
    )
    .expect("fake Forgejo document")
}

fn commit(count: u64) -> String {
    String::from_utf8(temper_fake_git::id::id(count).to_vec()).expect("fake commit spelling")
}

fn api(sandbox: &Path, route: &str) -> Call {
    let document = document(sandbox);
    // curl reads the private token file itself; neither argv nor captured output carries it.
    Call::command(
        Path::new("/usr/bin/curl"),
        &[
            "--silent",
            "--show-error",
            "--fail",
            "--noproxy",
            "*",
            "--variable",
            &format!("token@{}", String::from_utf8_lossy(document.admin_token())),
            "--expand-header",
            "Authorization: token {{token:trim}}",
            &format!("http://{}/api/v1/repos/{route}", document.listen()),
        ],
    )
}

#[test]
fn a_copy_is_made_and_adopted_through_ctl_on_the_fake_box() {
    let walk = Walk::new("walk-copy");
    let sandbox = walk.sandbox();
    let connect = sandbox.clone();
    let copy = sandbox.clone();
    let history = sandbox.clone();
    let copied_branch = sandbox.clone();
    let metadata = sandbox.clone();
    let workflow = sandbox.clone();
    let github = sandbox.clone();
    let seen = walk.run(vec![
        Act::After(Box::new(move |_| {
            let document = document(&connect);
            let address = document.listen().to_string();
            Call::new(&[
                "forge",
                "connect",
                "forgejo-admin",
                path(&connect.join("data/forgejo-admin.token")),
                "--api",
                &address,
                "--git",
                &format!("http://{address}"),
                "--first",
                "work/first",
                "--label",
                "copy-runner",
                "--key",
                "copy-forge",
                "--json",
            ])
        })),
        Act::After(Box::new(move |seen| {
            let connected = adoption(seen.last().expect("connected first repository"));
            let project = connected["project"].as_u64().expect("first repository's project").to_string();
            let forge = super::adopted_forge(seen.last().expect("connected forge")).to_string();
            let document = document(&copy);
            let source = document.sources().first().expect("copy source");
            Call::new(&[
                "repository",
                "copy",
                "--project",
                &project,
                "--forge",
                &forge,
                std::str::from_utf8(&source.address).expect("source address"),
                "--as",
                "work/widget",
                "--no-home",
                "--until",
                "adopted",
                "--key",
                "copy-widget",
                "--json",
            ])
        })),
        Act::Now(Call::new(&["forge", "list", "--json"])),
        Act::After(Box::new(move |_| api(&copied_branch, "work/widget/branches/main"))),
        Act::After(Box::new(move |_| api(&metadata, "work/widget"))),
        Act::After(Box::new(move |seen| {
            let document = document(&history);
            let source = document.sources().first().expect("copy source");
            let (forge, _) = document.seed(7);
            let base = forge.domain.branch(&source.repository, &source.landing).expect("source head");
            let base = commit(base);
            let tip = ctl::json(seen.get(3).expect("copied branch"))["commit"]["id"]
                .as_str()
                .expect("copied head")
                .to_owned();
            api(&history, &format!("work/widget/compare/{base}...{tip}?files=false"))
        })),
        Act::After(Box::new(move |seen| {
            api(
                &workflow,
                &format!(
                    "work/widget/raw/.forgejo/workflows/ci.yml?ref={}",
                    ctl::json(seen.get(3).expect("copied branch"))["commit"]["id"].as_str().expect("copied head")
                ),
            )
        })),
        Act::After(Box::new(move |seen| {
            api(
                &github,
                &format!(
                    "work/widget/raw/.github/workflows/ci.yml?ref={}",
                    ctl::json(seen.get(3).expect("copied branch"))["commit"]["id"].as_str().expect("copied head")
                ),
            )
        })),
        Act::After(Box::new(|seen| {
            let project = adoption(&seen[0])["project"].as_u64().expect("first project").to_string();
            Call::new(&["project", "show", &project, "--json"])
        })),
    ]);
    assert_copy(&seen, &sandbox);
    walk.scratch.remove();
}

fn assert_copy(seen: &super::Seen, sandbox: &Path) {
    let [_, copied, forges, branch, metadata, history, workflow, github, project] = seen.answers.as_slice() else {
        panic!("connect, copy, adopted forge and API reads")
    };
    assert_eq!(adoption(copied)["adopted"], true);
    let forge = super::only_forge(forges);
    assert!(forge["makings"].as_array().expect("making rows").is_empty());
    let project = ctl::json(project);
    assert_eq!(project["number"], adoption(&seen.answers[0])["project"]);
    let resources = project["resources"].as_array().expect("adopted project resources");
    assert_eq!(resources.len(), 2, "the first repository and the copy");
    // Project resources name the forge and provider id as big-endian hex segments.
    let provider =
        u32::try_from(ctl::json(metadata)["id"].as_u64().expect("copy's provider id")).expect("provider id fits");
    let resource = serde_json::json!([format!("{:04x}", super::number(&forge)), format!("{provider:08x}")]);
    let repository = resources
        .iter()
        .find(|row| row["part"]["key"]["path"] == resource)
        .expect("the copy's provider id is adopted into the first repository's project");
    assert_eq!(repository["role"], "owned");
    assert_eq!(repository["home"], false);
    assert_eq!(ctl::json(metadata)["default_branch"], "main");
    let history = ctl::json(history);
    assert_eq!(history["total_commits"], 1, "one workflow commit above the source");
    assert_eq!(history["commits"][0]["sha"], ctl::json(branch)["commit"]["id"]);
    let document = document(sandbox);
    let (forge, _) = document.seed(7);
    let source = document.sources().first().expect("source");
    let base = forge.domain.branch(&source.repository, &source.landing).expect("source head");
    assert_eq!(history["commits"][0]["parents"][0]["sha"], commit(base));
    assert_eq!(ctl::json(branch)["name"], "main");
    assert_eq!(
        workflow.stdout.as_slice(),
        temper_forge_setup::starter_workflow(b"copy-runner").expect("starter workflow").as_ref(),
        "the starter is read through the fake API"
    );
    let original = source
        .tree
        .iter()
        .find(|file| file.path.as_ref() == b".github/workflows/ci.yml")
        .expect("source's GitHub workflow");
    assert_eq!(github.stdout.as_slice(), original.content.as_ref(), "the source workflow is preserved");
}
