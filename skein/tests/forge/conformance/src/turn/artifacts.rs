//! Run artifacts through the runner's inline host-shell upload protocol (forge/forgejo.md, 2).
use super::{Probe, REPO, Result, actions, number, pause, rows};

pub(super) const NAMES: &[&str] = &[
    "actions-artifact-run-read-keeps-dispatch-head",
    "actions-artifacts-unknown-run-is-404",
    "actions-artifact-upload-completes",
    "actions-artifacts-name-selects-only-that-artifact",
    "actions-artifacts-absent-name-is-empty",
    "actions-artifact-zip-is-direct",
    "actions-artifact-zip-keeps-one-failed-id-file",
];

pub(super) fn record(p: &mut Probe<'_>) -> Result<()> {
    let script = include_str!("artifact-upload.sh");
    let mut workflow = String::from(
        "name: artifact\non: workflow_dispatch\njobs:\n  upload:\n    runs-on: kit-host\n    steps:\n      - run: |\n",
    );
    for line in script.lines() {
        workflow.push_str("          ");
        workflow.push_str(line);
        workflow.push('\n');
    }
    p.recorder.f.put(REPO, "main", ".forgejo/workflows/artifact.yaml", &workflow)?;
    let head = p.head("actions-artifact-dispatch-head", "main")?;
    let answer = p.send(
        "actions-artifact-dispatch",
        "POST",
        &format!("{REPO}/actions/workflows/artifact.yaml/dispatches"),
        Some(r#"{"ref":"main"}"#),
    )?;
    if answer.status != 204 {
        return Err("artifact dispatch was not accepted".into());
    }
    let mut run = None;
    for _ in 0..300 {
        let read = p.recorder.f.get(&format!("{REPO}/actions/runs?head_sha={head}"))?;
        if let Some(id) = rows(&read, "workflow_runs")?.first().and_then(|r| r.get("id").u64()) {
            run = Some(id);
            break;
        }
        pause();
    }
    let run = run.ok_or("artifact run not found in 30 seconds")?;
    actions::settled(p, run)?;
    let read = p.get("actions-artifact-run", &format!("{REPO}/actions/runs/{run}"))?;
    p.check(NAMES[0], read.root().get("commit_sha").is(&head), "artifact run did not keep its dispatch head");
    let jobs = p.get("actions-artifact-jobs", &format!("{REPO}/actions/runs/{run}/jobs"))?;
    for job in rows(&jobs, "jobs")? {
        let id = number(job.get("id"))?;
        let attempt = number(job.get("attempt"))?;
        p.send(
            &format!("actions-artifact-job-{id}-log"),
            "GET",
            &format!("{REPO}/actions/jobs/{id}/logs?attempt={attempt}"),
            None,
        )?;
    }
    let answer = p.send(
        "actions-artifacts-unknown-run",
        "GET",
        &format!("{REPO}/actions/runs/9999/artifacts?name=tests"),
        None,
    )?;
    p.fact(NAMES[1], answer.status == 404, &answer);
    let all = p.get("actions-artifacts-all", &format!("{REPO}/actions/runs/{run}/artifacts"))?;
    let all = rows(&all, "artifacts")?;
    p.check(
        NAMES[2],
        read.root().get("status").is("success")
            && all.len() == 2
            && all.iter().all(|a| a.get("size_in_bytes").u64() == Some(4) && a.get("run_id").u64() == Some(run))
            && all.iter().any(|a| a.get("name").is("tests"))
            && all.iter().any(|a| a.get("name").is("other")),
        "expected a successful upload job and two four-byte artifacts for its run",
    );
    let named = p.get("actions-artifacts-named", &format!("{REPO}/actions/runs/{run}/artifacts?name=tests"))?;
    let named = rows(&named, "artifacts")?;
    p.check(
        NAMES[3],
        named.len() == 1 && named.first().is_some_and(|a| a.get("name").is("tests")),
        "expected only the artifact named tests",
    );
    let absent = p.get("actions-artifacts-absent", &format!("{REPO}/actions/runs/{run}/artifacts?name=absent"))?;
    p.check(NAMES[4], rows(&absent, "artifacts")?.is_empty(), "absent artifact name did not answer an empty array");
    let Some(id) = named.first().and_then(|a| a.get("id").u64()) else {
        for name in &NAMES[5..] {
            p.check(name, false, "no named artifact to download");
        }
        return Ok(());
    };
    let zip = p.send("actions-artifact-zip", "GET", &format!("{REPO}/actions/artifacts/{id}/zip"), None)?;
    let file = skein_forge::forgejo::zip::read(&zip.body, 4, 4096);
    p.check(
        NAMES[5],
        zip.status == 200 && zip.body.starts_with(b"PK\x03\x04"),
        "expected a direct HTTP 200 ZIP, not a redirect",
    );
    p.check(
        NAMES[6],
        file.is_ok_and(|f| f.path.as_ref() == b"failed.txt" && f.bytes.as_ref() == b"a\nb\n" && !f.truncated),
        "expected a checked single-entry ZIP containing failed.txt with a and b, each followed by LF",
    );
    Ok(())
}
