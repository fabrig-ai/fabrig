//! GitHub's App paths, including a setup cut short by a box restart.
//! temper/docs/design/draft/playground-reset.md, 6.4 and 8;
//! temper/docs/design/protocol/setup.md, 17.2.
use super::*;

pub(super) fn go_on_after_a_restart() {
    let walk = Walk::new("walk-github-restart");
    let pem = walk.sandbox().join("data/github-app.pem");
    let mut steps = vec![
        Step::Run(Call::sign_in()),
        Step::Run(forges::make_github()),
        Step::Then(Box::new(move |seen| {
            Call::new(&[
                "forge",
                "connect",
                &made(&seen[1..]).to_string(),
                "github-app",
                "1",
                path(&pem),
                "--key",
                "walk-own-app",
                "--json",
            ])
        })),
        Step::Run(Call::new(&["forge", "list", "--until", "settled", "--json"])),
        // The seeded App survives each fake launch; no installation tab has opened.
        Step::KillBox,
        Step::StartBox,
        Step::Run(Call::new(&["forge", "list", "--json"])),
        Step::Then(Box::new(|seen| {
            let number = made(&seen[1..]);
            let [_, _, _, before, _signed_in_again, after] = seen else { panic!("forge reads straddle the restart") };
            for read in [before, after] {
                let document = ctl::json(read);
                let document = if document.get("met").is_some() { &document["forges"] } else { &document };
                assert_eq!(document["forges"].as_array().expect("forge rows").len(), 1, "one kept forge");
                let row = forge_number(read, number);
                assert_eq!(row["kind"], "github", "{row}");
                assert_eq!(row["state"], "connected", "the seeded App stays connected: {row}");
                assert!(row["first"].is_null(), "no repository adopted yet: {row}");
                let installations = row["installations"].as_array().expect("installations");
                assert!(installations.len() <= 1, "only the seeded installation: {row}");
                for installation in installations {
                    assert_eq!(installation["account"]["login"], "fake-person", "{row}");
                    assert_eq!(installation["stray"], true, "no installation accepted before Go on: {row}");
                }
            }
            eprintln!("forge {number} after restart: {}", forge_number(after, number));
            // Install can name the App only from the records restored after the kill.
            Call::new(&["forge", "install", &number.to_string(), "--key", "walk-go-on"])
        })),
        Step::Visit,
    ];
    let mut acts = vec![wait_forges(), adopt_github(), walk.add_key()];
    acts.extend(first_chat());
    steps.extend(acts.into_iter().map(|act| match act {
        Act::Now(call) => Step::Run(call),
        Act::Visit => Step::Visit,
        Act::ProgramLogin(name) => Step::ProgramLogin(name),
        Act::Mark(name) => Step::Mark(name),
        Act::After(next) => Step::Then(Box::new(move |seen| {
            let (_, answers) = seen.split_first().expect("signed in before the welcome's acts");
            next(answers)
        })),
    }));
    let outcome = boxed::run_ctl(
        walk.scratch.path(),
        super::super::flags(&walk.paths[0], &walk.paths[1], &walk.paths[2], Some(&walk.bundle)),
        walk.scratch.path(),
        steps,
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
    assert_eq!(outcome.calls.len(), 16, "all calls and the resumed sign-in completed: {diagnostic}");
    for (index, call) in outcome.calls.iter().enumerate() {
        assert_eq!(
            call.exit,
            Some(Exit::Code(0)),
            "call {index}: stdout {}; stderr {}; box: {diagnostic}",
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        );
    }
    let seen = Seen { answers: outcome.calls.into_iter().skip(1).collect(), visits: outcome.visits };
    let [visit] = seen.visits.as_slice() else { panic!("only Go on opens a tab") };
    assert_eq!(visit.iter().map(|hop| hop.status).collect::<Vec<_>>(), [302, 200]);
    assert_eq!(visit.iter().map(|hop| hop.method.as_str()).collect::<Vec<_>>(), ["GET", "GET"]);
    let number = made(&seen.answers);
    let [install, setup] = visit.as_slice() else { panic!("installation and setup, with no manifest visit") };
    assert!(install.path.starts_with("/apps/temper-fake/installations/new?"), "the kept seeded App: {}", install.path);
    assert!(setup.path.starts_with(&format!("/forges/{number}/github/setup?installation_id=")));
    assert_adopted(&seen, number, "widget");
    assert_reply(&seen);
    walk.scratch.remove();
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
