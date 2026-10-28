use crate::{Capture, Result, invocation::base64};
use serde_json::{Value, json};
use skein_throwaway_forgejo::{Account, Forgejo, Init, Instance, Outgoing};
use std::{
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
    process::Output,
};
fn api(instance: &Instance, method: &str, target: &str, body: &Value) -> Result<Value> {
    let body = body.to_string();
    let response = instance
        .exchange(&Outgoing::json(method, target, Some(&instance.admin().token), body.as_bytes()))
        .map_err(|e| format!("{e:?}"))?;
    if !(200..300).contains(&response.status) {
        return Err(format!("fixture {target}: HTTP {}", response.status).into());
    }
    Ok(if response.body.is_empty() { Value::Null } else { serde_json::from_slice(&response.body)? })
}
fn put(instance: &Instance, branch: &str, content: &str) -> Result<String> {
    let route = "/api/v1/repos/fixture/specimen/contents/conflict.txt";
    let old = api(instance, "GET", &format!("{route}?ref={branch}"), &Value::Null).unwrap_or(Value::Null);
    let body = json!({"branch":branch,"message":"fixture","content":base64(content),"sha":old["sha"]});
    api(instance, if old["sha"].is_string() { "PUT" } else { "POST" }, route, &body)?;
    let head = api(instance, "GET", &format!("/api/v1/repos/fixture/specimen/branches/{branch}"), &Value::Null)?;
    Ok(head["commit"]["id"].as_str().ok_or("fixture head missing")?.into())
}
struct Fixture {
    instance: Instance,
    writer: Account,
    stranger: Account,
    workspace: PathBuf,
    repo: PathBuf,
    base: String,
    topic: String,
}
impl Fixture {
    fn new(mut instance: Instance) -> Result<Self> {
        let writer = instance.create_user("writer").map_err(|e| format!("{e:?}"))?;
        let stranger = instance.create_user("stranger").map_err(|e| format!("{e:?}"))?;
        let owner = instance.admin().clone();
        for (name, init) in [
            ("specimen", Init::Readme),
            ("empty", Init::Empty),
            ("private", Init::Readme),
            ("hashed", Init::Sha256),
            ("guarded", Init::Readme),
        ] {
            instance.create_repository(&owner, name, init).map_err(|e| format!("{e:?}"))?;
            instance.add_collaborator(&owner, name, &writer, "write").map_err(|e| format!("{e:?}"))?;
        }
        api(&instance, "PATCH", "/api/v1/repos/fixture/private", &json!({"private":true}))?;
        let base = put(&instance, "main", "base\n")?;
        api(
            &instance,
            "POST",
            "/api/v1/repos/fixture/specimen/branches",
            &json!({"new_branch_name":"topic","old_ref_name":"main"}),
        )?;
        let topic = put(&instance, "topic", "topic\n")?;
        for branch in ["main", "blocked"] {
            api(
                &instance,
                "POST",
                "/api/v1/repos/fixture/guarded/branch_protections",
                &json!({"branch_name":branch,"enable_push":false}),
            )?;
        }
        let workspace = instance.scratch().join("worker/workspaces/1");
        fs::create_dir_all(&workspace)?;
        let repo = workspace.join("specimen");
        Ok(Self { instance, writer, stranger, workspace, repo, base, topic })
    }
    fn remote(&self, repo: &str) -> String {
        format!("{}/fixture/{repo}.git", self.instance.base())
    }
    fn identity(&self) -> (&str, &str) {
        (&self.writer.login, self.writer.token.value())
    }
}
fn success(output: &Output) -> Result<()> {
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("git fixture failed: {}", String::from_utf8_lossy(&output.stderr)).into())
    }
}
fn clone(
    c: &mut Capture,
    f: &Fixture,
    case: Option<&str>,
    url: &str,
    dir: &str,
    identity: Option<(&str, &str)>,
) -> Result<Output> {
    c.execute(
        case,
        &f.workspace,
        &["clone", "--no-checkout", "--no-tags", "--origin", "origin", "--", url, dir],
        identity,
        false,
        b"",
    )
}
fn fetch(
    c: &mut Capture,
    case: Option<&str>,
    dir: &Path,
    url: &str,
    source: &str,
    identity: Option<(&str, &str)>,
) -> Result<Output> {
    c.execute(
        case,
        dir,
        &[
            "fetch",
            "--porcelain",
            "--verbose",
            "--no-tags",
            "--no-write-fetch-head",
            "--no-recurse-submodules",
            "--",
            url,
            &format!("+{source}:refs/temper/fetched"),
        ],
        identity,
        false,
        b"",
    )
}
fn push(
    c: &mut Capture,
    f: &Fixture,
    case: &str,
    url: &str,
    commit: &str,
    branch: &str,
    lease: Option<&str>,
) -> Result<Output> {
    let lease = lease.map(|old| format!("--force-with-lease=refs/heads/{branch}:{old}"));
    let refspec = format!("{commit}:refs/heads/{branch}");
    let mut args = vec!["push", "--porcelain"];
    if let Some(lease) = &lease {
        args.push(lease);
    }
    args.extend(["--", url, &refspec]);
    c.execute(Some(case), &f.repo, &args, Some(f.identity()), false, b"")
}
pub(crate) fn record(forgejo: &Forgejo, instance: Instance, c: &mut Capture) -> Result<()> {
    let workspace = instance.scratch().join("worker/workspaces/1");
    fs::create_dir_all(&workspace)?;
    let version = c.execute(Some("version"), &workspace, &["version"], None, false, b"")?;
    let printed = String::from_utf8_lossy(&version.stdout);
    let nums: Vec<u32> = printed
        .split_whitespace()
        .nth(2)
        .unwrap_or_default()
        .split('.')
        .take(2)
        .filter_map(|n| n.parse().ok())
        .collect();
    if !version.status.success() || nums.len() != 2 || nums[0] < 2 || (nums[0] == 2 && nums[1] < 41) {
        return Err(format!("git {}: requires 2.41 or later, printed {printed}", c.git.display()).into());
    }
    let f = Fixture::new(instance)?;
    c.secrets = f.instance.secrets();
    for account in [&f.writer, &f.stranger] {
        c.secrets.push(base64(&format!("{}:{}", account.login, account.token.value())));
    }
    clones(&f, c)?;
    fetches(&f, c)?;
    let url = f.remote("specimen");
    let identity = Some(f.identity());
    success(&push(c, &f, "create-new-branch", &url, &f.base, "created", Some(""))?)?;
    push(c, &f, "create-exists-at-commit", &url, &f.base, "created", Some(""))?;
    push(c, &f, "create-exists-elsewhere", &url, &f.topic, "created", Some(""))?;
    push(c, &f, "create-protected", &f.remote("guarded"), &f.topic, "blocked", Some(""))?;
    let commit = locals(&f, c)?;
    pushes(&f, c, &commit)?;
    api(&f.instance, "PATCH", "/api/v1/repos/fixture/specimen", &json!({"name":"renamed"}))?;
    fetch(c, Some("fetch-renamed"), &f.repo, &url, "refs/heads/main", identity)?;
    let exit = fs::read(c.out.join("transcripts/fetch-renamed/exit"))?;
    let stderr = fs::read(c.out.join("transcripts/fetch-renamed/stderr"))?;
    let redirected = followed(&exit, &stderr);
    let protected = fs::read(c.out.join("transcripts/push-protected/stdout"))?;
    let protected = String::from_utf8_lossy(&protected);
    let declined =
        protected.lines().any(|s| s.starts_with('!') && s.contains("[remote rejected]") && s.ends_with("declined)"));
    let facts = format!(
        "# forgejo {} sha256 {}\n# {}\n{} git-follows-a-renamed-repository-redirect{}\n{} protected-push-is-remote-rejected-declined{}\n",
        forgejo.version(),
        forgejo.sha256(),
        printed.trim(),
        if redirected { "holds" } else { "fails" },
        if redirected {
            String::new()
        } else {
            format!(
                ": exit {}; {}",
                String::from_utf8_lossy(&exit).trim(),
                String::from_utf8_lossy(&stderr).replace(['\r', '\n'], " ")
            )
        },
        if declined { "holds" } else { "fails" },
        if declined { String::new() } else { format!(": {}", protected.replace(['\r', '\n'], " ")) }
    );
    c.scan("facts.txt", facts.as_bytes())?;
    fs::write(c.out.join("facts.txt"), facts)?;
    f.instance.stop().map_err(|e| format!("{e:?}").into())
}
fn clones(f: &Fixture, c: &mut Capture) -> Result<()> {
    success(&clone(c, f, Some("clone"), &f.remote("specimen"), "specimen", Some(f.identity()))?)?;
    success(&clone(c, f, Some("clone-empty"), &f.remote("empty"), "empty", Some(f.identity()))?)?;
    clone(c, f, Some("clone-missing-repository"), &f.remote("missing"), "missing", Some(f.identity()))?;
    clone(c, f, Some("clone-wrong-token"), &f.remote("specimen"), "wrong", Some(("writer", "wrong-token")))?;
    clone(
        c,
        f,
        Some("clone-stranger-private"),
        &f.remote("private"),
        "private",
        Some((&f.stranger.login, f.stranger.token.value())),
    )?;
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let port = listener.local_addr()?.port();
    drop(listener);
    c.mask.port(port);
    clone(
        c,
        f,
        Some("clone-refused-connection"),
        &format!("http://127.0.0.1:{port}/fixture/specimen.git"),
        "refused",
        Some(f.identity()),
    )?;
    Ok(())
}
fn fetches(f: &Fixture, c: &mut Capture) -> Result<()> {
    let url = f.remote("specimen");
    for (name, source) in [
        ("fetch-branch", "refs/heads/topic"),
        ("fetch-commit", f.base.as_str()),
        ("fetch-default", "HEAD"),
        ("fetch-up-to-date", "HEAD"),
        ("fetch-missing-branch", "refs/heads/missing"),
        ("fetch-missing-commit", "ffffffffffffffffffffffffffffffffffffffff"),
    ] {
        fetch(c, Some(name), &f.repo, &url, source, Some(f.identity()))?;
    }
    fetch(c, Some("fetch-wrong-token"), &f.repo, &url, "HEAD", Some(("writer", "wrong-token")))?;
    success(&clone(c, f, None, &f.remote("hashed"), "hashed", Some(f.identity()))?)?;
    fetch(
        c,
        Some("fetch-sha256-branch"),
        &f.workspace.join("hashed"),
        &f.remote("hashed"),
        "refs/heads/main",
        Some(f.identity()),
    )?;
    Ok(())
}
fn local(c: &mut Capture, f: &Fixture, case: Option<&str>, args: &[&str]) -> Result<Output> {
    let commit =
        case.is_some_and(|name| name.starts_with("commit-")) || matches!(args[0], "add" | "diff-index" | "write-tree");
    c.execute(case, &f.repo, args, None, commit, b"")
}
fn locals(f: &Fixture, c: &mut Capture) -> Result<String> {
    success(&local(c, f, Some("checkout-1"), &["checkout", "--force", "--detach", &f.base])?)?;
    fs::write(f.repo.join("untracked"), b"discard\n")?;
    success(&local(c, f, Some("checkout-2"), &["clean", "-ffdxq"])?)?;
    success(&local(c, f, Some("merge-clean"), &["merge", "--no-commit", "--no-ff", "--", &f.topic])?)?;
    success(&local(c, f, None, &["checkout", "--force", "--detach", &f.base])?)?;
    success(&local(c, f, Some("merge-up-to-date"), &["merge", "--no-commit", "--no-ff", "--", &f.base])?)?;
    success(&local(c, f, None, &["checkout", "--force", "--detach", &f.base])?)?;
    success(&local(c, f, Some("commit-add"), &["add", "--all"])?)?;
    local(c, f, Some("commit-unchanged"), &["diff-index", "--cached", "--quiet", &f.base, "--"])?;
    fs::write(f.repo.join("conflict.txt"), b"ours\n")?;
    success(&local(c, f, None, &["add", "--all"])?)?;
    local(c, f, Some("commit-changed"), &["diff-index", "--cached", "--quiet", &f.base, "--"])?;
    let tree = local(c, f, Some("commit-write-tree"), &["write-tree"])?;
    success(&tree)?;
    let tree = String::from_utf8(tree.stdout)?.trim().to_owned();
    let commit = c.execute(
        Some("commit-tree-one-parent"),
        &f.repo,
        &["commit-tree", &tree, "-p", &f.base, "-F", "-"],
        None,
        true,
        b"Fixture\n\nOne parent.\n",
    )?;
    success(&commit)?;
    c.execute(
        Some("commit-tree-two-parents"),
        &f.repo,
        &["commit-tree", &tree, "-p", &f.base, "-p", &f.topic, "-F", "-"],
        None,
        true,
        b"Merge\n\nTwo parents.\n",
    )?;
    let commit = String::from_utf8(commit.stdout)?.trim().to_owned();
    success(&local(c, f, None, &["checkout", "--force", "--detach", &commit])?)?;
    local(c, f, Some("merge-conflict"), &["merge", "--no-commit", "--no-ff", "--", &f.topic])?;
    for name in ["merge-conflict-2", "commit-unmerged"] {
        local(c, f, Some(name), &["diff-files", "--name-only", "-z", "--diff-filter=U"])?;
    }
    Ok(commit)
}
fn pushes(f: &Fixture, c: &mut Capture, commit: &str) -> Result<()> {
    let url = f.remote("specimen");
    let identity = Some(f.identity());
    success(&push(c, f, "push-fast-forward", &url, commit, "main", None)?)?;
    success(&push(c, f, "push-new-branch", &url, commit, "pushed", None)?)?;
    success(&push(c, f, "push-up-to-date", &url, commit, "main", None)?)?;
    push(c, f, "push-non-fast-forward", &url, &f.base, "main", None)?;
    let tree = local(c, f, None, &["rev-parse", &format!("{commit}^{{tree}}")])?;
    success(&tree)?;
    let tree = String::from_utf8(tree.stdout)?.trim().to_owned();
    let later =
        c.execute(None, &f.repo, &["commit-tree", &tree, "-p", commit, "-F", "-"], None, true, b"Stale lease\n")?;
    success(&later)?;
    let later = String::from_utf8(later.stdout)?.trim().to_owned();
    push(c, f, "push-lease-stale", &url, &later, "main", Some(&f.base))?;
    let guarded = f.remote("guarded");
    let head = fetch(c, None, &f.repo, &guarded, "refs/heads/main", identity)?;
    success(&head)?;
    let head = api(&f.instance, "GET", "/api/v1/repos/fixture/guarded/branches/main", &Value::Null)?;
    let head = head["commit"]["id"].as_str().ok_or("guarded head missing")?;
    let tree = local(c, f, None, &["rev-parse", &format!("{commit}^{{tree}}")])?;
    success(&tree)?;
    let tree = String::from_utf8(tree.stdout)?.trim().to_owned();
    let guarded_commit =
        c.execute(None, &f.repo, &["commit-tree", &tree, "-p", head, "-F", "-"], None, true, b"Guarded\n")?;
    success(&guarded_commit)?;
    let guarded_commit = String::from_utf8(guarded_commit.stdout)?.trim().to_owned();
    push(c, f, "push-protected", &guarded, &guarded_commit, "main", None)?;
    c.execute(
        Some("push-no-permission"),
        &f.repo,
        &["push", "--porcelain", "--", &url, &format!("{commit}:refs/heads/forbidden")],
        Some((&f.stranger.login, f.stranger.token.value())),
        false,
        b"",
    )?;
    local(c, f, Some("push-ancestor-yes"), &["merge-base", "--is-ancestor", &f.base, commit])?;
    local(c, f, Some("push-ancestor-no"), &["merge-base", "--is-ancestor", commit, &f.base])?;
    Ok(())
}

/// Whether the recorded fetch followed the redirect to the renamed repository.
pub(crate) fn followed(exit: &[u8], stderr: &[u8]) -> bool {
    exit == b"0\n"
        && stderr.split(|byte| *byte == b'\n').any(|line| {
            line.starts_with(b"warning: redirecting to ") && line.windows(16).any(|part| part == b"/fixture/renamed")
        })
}
