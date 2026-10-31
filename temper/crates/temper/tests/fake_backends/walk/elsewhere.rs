use super::*;

fn adopted_project(seen: &[Ended]) -> String {
    let [_, _, connected, ..] = seen else { panic!("connection answered") };
    adoption(connected)["project"].as_u64().expect("adopted project number").to_string()
}

fn chat_task(seen: &[Ended]) -> String {
    let [_, _, _, _, _, _, _, started, ..] = seen else { panic!("first chat started") };
    ctl::json(started)["outcome"]["done"]["owner"]["created"]["first"].as_u64().expect("chat's task number").to_string()
}

fn assert_ready(seen: &[Ended]) {
    let [before_model, before_forge, connected, _, _, ready_model, ready_forge] = seen else {
        panic!("welcome ready reads completed")
    };
    assert!(!usable(before_model), "no model before the key: the welcome must offer its Model step");
    assert!(forge_rows(before_forge).is_empty(), "Connect makes the first forge record");
    let forge = adopted_forge(connected);
    let connected = adoption(connected);
    let adopted = &connected["outcome"]["adopted"];
    assert_eq!(adopted["forge"], forge);
    assert_eq!(adopted["owner"], "fake-person");
    assert_eq!(adopted["name"], "hello");
    assert_eq!(adopted["role"], "owned");
    assert_eq!(adopted["home"], true);
    assert!(connected["project"].as_u64().expect("adopted project") > 0, "repository adopted into a project");
    let ready = only_forge(ready_forge);
    assert_eq!(number(&ready), forge);
    assert_eq!(ready["state"], "connected");
    assert_eq!(ready["first"]["state"], "made");
    assert_eq!(ready["first"]["org"], "fake-person");
    assert_eq!(ready["first"]["repository"], "hello");
    assert_eq!(ready["first"]["id"], adopted["repository"]);
    assert!(usable(ready_model), "key account usable at Ready");
}

pub(super) fn run() {
    let walk = Walk::new("walk-elsewhere-key");
    let admin = walk.sandbox().join("data/forgejo-admin.token");
    let seen = walk.run(vec![
        Act::Now(Call::new(&["account", "list", "--json"])),
        Act::Now(Call::new(&["forge", "list", "--json"])),
        Act::After(Box::new(move |_| {
            let peer = super::super::value(
                &admin.parent().expect("sandbox data").parent().expect("sandbox").join("config/forgejo.json"),
            );
            let socket = peer["listen"]["socket"].as_str().expect("fake Forgejo address");
            let remote = format!("http://{socket}");
            Call::new(&[
                "forge",
                "connect",
                "forgejo-admin",
                path(&admin),
                "--api",
                socket,
                "--git",
                &remote,
                "--git-program",
                peer["git_program"].as_str().expect("fake git program"),
                "--web",
                &remote,
                "--first",
                "fake-person/hello",
                "--key",
                "walk-forge",
                "--json",
            ])
        })),
        Act::After(Box::new(|seen| {
            let forge = adopted_forge(seen.last().expect("connection answered"));
            Call::new(&["forge", "list", "--until", &format!("connected:{forge}"), "--timeout", "30s"])
        })),
        walk.add_key(),
        Act::Now(Call::new(&["account", "list", "--until", "usable", "--timeout", "10s", "--json"])),
        Act::Now(Call::new(&["forge", "list", "--json"])),
        Act::After(Box::new(|seen| {
            assert_ready(seen);
            Call::new(&[
                "chat",
                "start",
                "--project",
                &adopted_project(seen),
                "--key",
                "walk-chat",
                "Say hello",
                "--json",
            ])
        })),
        Act::After(Box::new(|seen| {
            Call::new(&["task", "show", &chat_task(seen), "--until", "replied", "--timeout", "10s", "--json"])
        })),
        Act::After(Box::new(|seen| Call::new(&["chat", "show", &chat_task(seen), "--json"]))),
    ]);
    assert_reply(&seen);
    walk.scratch.remove();
}
