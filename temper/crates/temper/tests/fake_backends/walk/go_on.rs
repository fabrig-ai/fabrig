//! GitHub goes on after a restart: temper/docs/design/draft/playground-reset.md, 6.4 and 8.
use super::*;

fn unfinished(call: &Ended) {
    let row = forge_number(call, 2);
    assert_eq!(row["app"], "manifest", "the converted App is kept: {row}");
    assert_eq!(row["finished"], false, "GitHub still needs its installation and repository: {row}");
    assert!(
        row["installations"].as_array().expect("installation rows").iter().all(|row| row["stray"] == true),
        "no installation has been accepted: {row}"
    );
}

pub(super) fn run() {
    let walk = Walk::new("walk-github-go-on");
    let mut acts = github::manifest_acts();
    for act in &mut acts {
        if matches!(act, Act::Visit) {
            *act = Act::VisitHops(3);
        }
    }
    let first = walk.run(acts);
    unfinished(first.answers.last().expect("forge read after conversion, before stopping the box"));
    let [before_restart] = first.visits.as_slice() else { panic!("one manifest visit before restart") };
    assert_eq!(before_restart.iter().map(|hop| hop.status).collect::<Vec<_>>(), [200, 200, 302]);

    // The welcome's Go on before Start over is checked by
    // temper-client-view/src/tests/start_over.rs::github_not_finished_offers_go_on_before_start_over.
    // Install uses the kept App's slug; Connect would ask for another manifest.
    let mut acts = vec![
        forges(),
        Act::After(Box::new(|seen| {
            let [retained] = seen else { panic!("first forge read after restart") };
            unfinished(retained);
            Call::new(&["forge", "install", "2", "--key", "walk-go-on"])
        })),
        Act::Visit,
        wait_forges(),
        adopt_github(),
        walk.add_key(),
    ];
    acts.extend(first_chat());
    let second = walk.run(acts);
    assert_adopted(&second, 2, "widget");
    assert_reply(&second);
    let [after_restart] = second.visits.as_slice() else { panic!("one installation visit after restart") };
    assert_eq!(after_restart.iter().map(|hop| hop.status).collect::<Vec<_>>(), [302, 200]);

    let hops: Vec<_> = before_restart.iter().chain(after_restart).collect();
    assert_eq!(hops.iter().map(|hop| hop.method.as_str()).collect::<Vec<_>>(), ["GET", "POST", "GET", "GET", "GET"]);
    let [manifest, create, convert, install, setup] = hops.as_slice() else { panic!("five hops across both boxes") };
    assert!(manifest.path.starts_with("/forges/2/github/manifest?"));
    assert!(create.path.starts_with("/settings/apps/new?"));
    assert!(convert.path.starts_with("/forges/2/github/redirect?"));
    assert!(install.path.starts_with("/apps/temper-fake/installations/new?"));
    assert!(setup.path.starts_with("/forges/2/github/setup?installation_id="));
    walk.scratch.remove();
}
