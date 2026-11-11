//! The owned Forgejo keeps its ordinary cache, fetch, prepare and supervision.
use super::{Act, Call, Ended, Walk, usable};

pub(super) fn assert_offered(seen: &[Ended]) {
    let [model, forges] = seen else { panic!("welcome model and forge reads") };
    assert!(!usable(model), "the welcome offers its Model step");
    assert!(super::forge_rows(forges).is_empty(), "SetUp makes the first forge record");
}

pub(super) fn assert_adopted(call: &Ended) -> u16 {
    // forge setup emits a progress document before its final adoption document.
    let mut answers = serde_json::Deserializer::from_slice(&call.stdout).into_iter::<serde_json::Value>();
    assert_eq!(answers.next().expect("setup progress").expect("progress JSON"), serde_json::json!({"setting_up":{}}));
    let adopted = answers.next().expect("adoption answer").expect("adoption JSON");
    assert!(answers.next().is_none(), "one progress and one final adoption");
    assert!(adopted["project"].as_u64().expect("setup project") > 0);
    let forge = super::adopted_forge(call);
    assert_eq!(adopted["outcome"]["adopted"]["owner"], "work");
    assert_eq!(adopted["outcome"]["adopted"]["name"], "hello");
    assert_eq!(adopted["outcome"]["adopted"]["role"], "owned");
    assert_eq!(adopted["outcome"]["adopted"]["home"], true);
    forge
}

pub(super) fn assert_ready(seen: &[Ended]) {
    let [.., forges, accounts] = seen else { panic!("ready reads") };
    let forge = super::only_forge(forges);
    assert_eq!(super::number(&forge), super::adopted_forge(&seen[2]));
    assert_eq!(forge["state"], "connected", "{forge}");
    assert_eq!(forge["first"]["state"], "made", "{forge}");
    assert_eq!(forge["first"]["org"], "work");
    assert_eq!(forge["first"]["repository"], "hello");
    assert!(usable(accounts), "the welcome reaches Ready");
}

#[test]
fn forgejo_here_and_a_key_reach_a_first_chat_on_the_fake_box() {
    let walk = Walk::owned("walk-here-key");
    let mut acts = vec![
        Act::Now(Call::new(&["account", "list", "--json"])),
        Act::Now(Call::new(&["forge", "list", "--json"])),
        Act::Mark("welcome preconditions"),
        Act::After(Box::new(|seen| {
            assert_offered(seen);
            Call::new(&["forge", "setup", "--key", "walk-here", "--json"])
        })),
        Act::Mark("owned forge adopted"),
    ];
    acts.extend(super::program_here::steps());
    acts.push(walk.add_key());
    acts.extend([
        Act::Now(Call::new(&["account", "list", "--until", "usable", "--json"])),
        Act::Now(Call::new(&["forge", "list", "--json"])),
        Act::Now(Call::new(&["account", "list", "--json"])),
        Act::After(Box::new(|seen| {
            assert_ready(seen);
            super::program_here::assert_program_usable(seen.last().expect("accounts"));
            Call::new(&[
                "chat",
                "start",
                "--project",
                "hello",
                "--model",
                "gpt-6-luna",
                "--key",
                "walk-chat",
                "Hello",
                "--json",
            ])
        })),
        // No other task precedes this first chat in the fresh deployment.
        Act::Now(Call::new(&["task", "show", "1", "--until", "replied", "--json"])),
        Act::Now(Call::new(&["chat", "show", "1", "--json"])),
        Act::Mark("first chat committed"),
    ]);
    let seen = walk.run_then(acts, super::open_default::open_and_reload);
    assert_adopted(seen.answers.get(2).expect("setup adoption"));
    super::assert_reply(&seen);
    walk.scratch.remove();
}
