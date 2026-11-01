//! GitHub's App paths, including a setup cut short by a box restart.
//! temper/docs/design/draft/playground-reset.md, 6.4 and 8;
//! temper/docs/design/protocol/setup.md, 17.2.
use super::*;
use std::fmt::Write as _;

pub(super) fn go_on_after_a_restart() {
    let walk = Walk::new("walk-github-restart");
    let manifest_request = walk.scratch.path().join("manifest-request");
    let create_request = walk.scratch.path().join("create-request");
    let convert_request = walk.scratch.path().join("convert-request");
    let install_request = walk.scratch.path().join("install-request");
    let outcome = boxed::run_ctl(
        walk.scratch.path(),
        super::super::flags(&walk.paths[0], &walk.paths[1], &walk.paths[2], Some(&walk.bundle)),
        walk.scratch.path(),
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(forges::make_github()),
            Step::Then(Box::new(|seen| {
                Call::new(&["forge", "connect", &made(&seen[1..]).to_string(), "github", "--key", "walk-github"])
            })),
            Step::Then(Box::new(move |seen| {
                let url = printed_address(seen.last().expect("manifest link"));
                assert!(url.contains(&format!("/forges/{}/github/manifest?", made(&seen[1..]))));
                browser_request(&manifest_request, url, None)
            })),
            Step::Then(Box::new(move |seen| {
                let html = response(seen.last().expect("manifest page"), 200);
                let form = html.split_once("<form ").expect("manifest form").1;
                let action = attribute(form, "action");
                assert!(action.contains("/settings/apps/new?"));
                let input =
                    html.split("<input ").find(|tag| tag.contains("name=\"manifest\"")).expect("manifest input");
                browser_request(&create_request, &action, Some(&attribute(input, "value")))
            })),
            Step::Then(Box::new(move |seen| {
                let html = response(seen.last().expect("GitHub's Create page"), 200);
                let link = html.split("<a ").find(|tag| tag.contains("id=\"create\"")).expect("Create link");
                let url = attribute(link, "href");
                assert!(url.contains(&format!("/forges/{}/github/redirect?", made(&seen[1..]))));
                // One request, without following Location: the App is converted
                // durably, but its installation page has not been visited.
                browser_request(&convert_request, &url, None)
            })),
            Step::Run(Call::new(&["forge", "list", "--json"])),
            Step::KillBox,
            Step::StartBox,
            Step::Run(Call::new(&["forge", "list", "--json"])),
            Step::Then(Box::new(|seen| {
                let number = made(&seen[1..]);
                let [.., converted, before, _signed_in_again, after] = seen else {
                    panic!("conversion and forge reads straddle the restart")
                };
                let headers = response(converted, 302);
                let install = location(headers);
                assert!(install.contains("/apps/") && install.contains("/installations/new?"));
                for read in [before, after] {
                    let row = forge_number(read, number);
                    assert_eq!(row["kind"], "github");
                    assert!(row["first"].is_null());
                    assert!(row["installations"].as_array().expect("installations").iter().all(|i| i["stray"] == true));
                }
                // Before the crash App authentication may already be connected,
                // even though no installation has been accepted. After restart
                // the peer no longer recognizes this converted App.
                assert_ne!(forge_number(after, number)["state"], "connected");
                eprintln!("forge {number} after restart: {}", forge_number(after, number));
                Call::new(&["forge", "install", &number.to_string(), "--key", "walk-go-on"])
            })),
            Step::Then(Box::new(move |seen| {
                let installed = seen.last().expect("Go on answered");
                assert_eq!(
                    installed.exit,
                    Some(Exit::Code(0)),
                    "Go on must reach the kept App's installation: stdout {}; stderr {}",
                    String::from_utf8_lossy(&installed.stdout),
                    String::from_utf8_lossy(&installed.stderr)
                );
                browser_request(&install_request, printed_address(installed), None)
            })),
        ],
        crate::BOX_DEADLINE,
        |_| (),
    );
    let diagnostic = String::from_utf8_lossy(&outcome.stderr);
    assert_eq!(outcome.exit, Some(Exit::Code(0)), "restarted fake box: {diagnostic}");
    assert_eq!(outcome.checked, Some(()), "Go on completed: {diagnostic}");
    assert_eq!(outcome.left, Some(Vec::new()), "no child outlives its box: {diagnostic}");
    let [killed, restarted] = outcome.runs.as_slice() else { panic!("two starts: {diagnostic}") };
    assert_eq!(killed.exit, Some(Exit::Signal(9)), "KillBox killed the first box");
    assert_eq!(restarted.exit, Some(Exit::Code(0)));
    for run in &outcome.runs {
        assert!(support::printed_link(&run.stdout).is_some(), "each start printed its sign-in link");
    }
    assert_eq!(outcome.calls.len(), 11, "all calls and the resumed sign-in completed: {diagnostic}");
    for call in &outcome.calls {
        assert_eq!(
            call.exit,
            Some(Exit::Code(0)),
            "call succeeded: {}; box: {diagnostic}",
            String::from_utf8_lossy(&call.stderr)
        );
    }
    let converted = response(&outcome.calls[5], 302);
    let install_url = location(converted);
    let resumed_url = printed_address(&outcome.calls[9]);
    // The JSON projection omits App details. Install can name this same
    // manifest App only from the records temper restored after the crash.
    assert_eq!(install_url.split('?').next(), resumed_url.split('?').next(), "Go on uses the kept App records");
    // The fake retains its seed, not its converted App (fake-backends.md, 3).
    // The spec permits stopping at the installation in this case. No second
    // manifest/create/conversion request is sent after StartBox.
    response(&outcome.calls[10], 404);
    walk.scratch.remove();
}

fn browser_request(file: &Path, url: &str, manifest: Option<&str>) -> Call {
    let authority = url.strip_prefix("http://").expect("plaintext browser URL").split('/').next().expect("authority");
    let address: std::net::SocketAddr = authority.parse().expect("literal browser address");
    assert!(address.ip().is_loopback(), "only the box's fake backends");
    let mut config = format!("url = {}\n", serde_json::to_string(url).expect("URL quoted for curl"));
    if let Some(manifest) = manifest {
        writeln!(
            config,
            "data-urlencode = {}",
            serde_json::to_string(&format!("manifest={manifest}")).expect("manifest quoted for curl")
        )
        .expect("curl request text");
    }
    super::super::write(file, config.as_bytes());
    Call::command(
        Path::new("/usr/bin/curl"),
        &[
            "--silent",
            "--show-error",
            "--max-time",
            "10",
            "--noproxy",
            "*",
            "--proto",
            "=http",
            "--include",
            "--config",
            path(file),
        ],
    )
}

fn printed_address(call: &Ended) -> &str {
    std::str::from_utf8(&call.stdout)
        .expect("printed link text")
        .lines()
        .find(|line| line.starts_with("http://"))
        .expect("printed browser address")
}

fn response(call: &Ended, status: u16) -> &str {
    let response = std::str::from_utf8(&call.stdout).expect("browser response text");
    assert_eq!(
        response.lines().next().expect("HTTP status").split_whitespace().nth(1),
        Some(status.to_string().as_str())
    );
    response
}

fn location(response: &str) -> &str {
    response
        .lines()
        .take_while(|line| !line.is_empty())
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("location").then(|| value.trim())
        })
        .expect("conversion's installation Location")
}

fn attribute(tag: &str, name: &str) -> String {
    let marker = format!("{name}=\"");
    let value = tag.split_once(&marker).expect("HTML attribute").1.split_once('"').expect("attribute end").0;
    value.replace("&quot;", "\"").replace("&#39;", "'").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
}

fn offered(seen: &[Ended]) {
    let [_, before_model, before_forge] = seen else { panic!("welcome preconditions read") };
    assert!(!usable(before_model), "the welcome offers its Model step");
    let row = forge_number(before_forge, made(seen));
    assert_eq!(row["state"], "not_connected", "the welcome offers GitHub: {row}");
    assert!(row["first"].is_null());
}
fn finish(walk: &Walk, mut acts: Vec<Act>) -> Seen {
    acts.push(adopt_github());
    acts.push(walk.add_key());
    acts.extend(first_chat());
    let seen = walk.run(acts);
    assert_adopted(&seen, made(&seen.answers), "widget");
    assert_reply(&seen);
    seen
}
pub(super) fn one_click() {
    let walk = Walk::new("walk-github-key");
    let seen = finish(&walk, manifest_acts());
    let [visit] = seen.visits.as_slice() else { panic!("one departing tab") };
    assert_eq!(visit.iter().map(|hop| hop.status).collect::<Vec<_>>(), [200, 200, 302, 302, 200]);
    assert_eq!(visit.iter().map(|hop| hop.method.as_str()).collect::<Vec<_>>(), ["GET", "POST", "GET", "GET", "GET"]);
    let number = made(&seen.answers);
    let [manifest, create, convert, install, setup] = visit.as_slice() else { panic!("five GitHub hops") };
    assert!(manifest.path.starts_with(&format!("/forges/{number}/github/manifest?")));
    assert!(create.path.starts_with("/settings/apps/new?"));
    assert!(convert.path.starts_with(&format!("/forges/{number}/github/redirect?")));
    assert!(install.path.starts_with("/apps/"));
    assert!(setup.path.starts_with(&format!("/forges/{number}/github/setup?")));
    walk.scratch.remove();
}
pub(super) fn own_app() {
    let walk = Walk::new("walk-own-app-key");
    let pem = walk.sandbox().join("data/github-app.pem");
    let seen = finish(
        &walk,
        vec![
            Act::Now(forges::make_github()),
            accounts(),
            forges(),
            Act::After(Box::new(move |seen| {
                offered(seen);
                Call::new(&[
                    "forge",
                    "connect",
                    &made(seen).to_string(),
                    "github-app",
                    "1",
                    path(&pem),
                    "--key",
                    "walk-own-app",
                    "--json",
                ])
            })),
            wait_forges(),
            Act::After(Box::new(|seen| {
                let row = forge_number(seen.last().expect("own App connected"), made(seen));
                let [installation] = row["installations"].as_array().expect("seeded installation").as_slice() else {
                    panic!("one seeded installation")
                };
                assert_eq!(installation["stray"], true, "Connecting still offers Install until the tab comes back");
                Call::new(&["forge", "install", &made(seen).to_string(), "--key", "walk-install"])
            })),
            Act::Visit,
            wait_forges(),
        ],
    );
    let [visit] = seen.visits.as_slice() else { panic!("own-App departing tab") };
    assert_eq!(visit.iter().map(|hop| hop.status).collect::<Vec<_>>(), [302, 200]);
    let number = made(&seen.answers);
    let [install, setup] = visit.as_slice() else { panic!("install and setup") };
    assert!(install.path.starts_with("/apps/temper-fake/installations/new?"));
    // Reconcile may already have pruned the unaccepted seed installation;
    // the return accepts the installation GitHub made or updated for this tab.
    assert!(setup.path.starts_with(&format!("/forges/{number}/github/setup?installation_id=")));
    walk.scratch.remove();
}

pub(super) fn manifest_acts() -> Vec<Act> {
    vec![
        Act::Now(forges::make_github()),
        accounts(),
        forges(),
        Act::After(Box::new(|seen| {
            offered(seen);
            Call::new(&["forge", "connect", &made(seen).to_string(), "github", "--key", "walk-github"])
        })),
        Act::Visit,
        wait_forges(),
    ]
}
