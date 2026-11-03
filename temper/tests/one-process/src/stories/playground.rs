//! The native visitor's walk (temper/docs/design/playground.md, 2, 5.2 and 9).
use super::{press, see};
use skein_fake_forge_domain::api;
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use temper_client_domain::{Address, ChatsFilter};

const REPOSITORY: &[u8] = b"playground/widget";

#[derive(Debug)]
pub struct Outcome {
    pub main: u64,
    pub csv: bool,
    pub green: bool,
    pub answered: u32,
}

fn scripts() -> serde_json::Value {
    serde_json::from_slice(temper_playground::stories::A.llm).expect("story A's scripts")
}

pub(crate) fn codec() -> jig_protocol_store::codec::View {
    let deployment = serde_json::from_slice(temper_playground::stories::A.deployment).expect("story A's deployment");
    let engine = temper_playground::documents::settings::configuration(&deployment);
    jig_protocol_store::codec::limits(&engine.limits.core).expect("story A's stored records")
}

pub(crate) fn person() -> (Vec<Step<Address>>, Vec<u32>) {
    let scripts = scripts();
    let mut replies = Vec::new();
    let mut steps = vec![
        Step::Go { address: Address::Welcome { step: temper_client_domain::Step::Welcome } },
        see(Find::named(Role::Button, b"Try the playground")),
        press(Role::Button, b"Try the playground"),
        see(Find::named(Role::Button, b"Add CSV export to widget")),
        see(Find::text(REPOSITORY)),
    ];
    for (index, script) in scripts["asked"].as_array().expect("three suggested requests").iter().enumerate() {
        if index > 0 {
            steps.push(Step::Go { address: Address::Chats { filter: ChatsFilter::All, start: false } });
            steps.push(see(Find::named(Role::Button, script["cue"].as_str().expect("suggestion").as_bytes())));
        }
        steps.push(press(Role::Button, script["cue"].as_str().expect("suggestion").as_bytes()));
        steps.push(see(Find::named(Role::TextArea, b"Start a chat")));
        steps.push(press(Role::Button, b"Start"));
        for turn in script["turns"].as_array().expect("scripted turns") {
            for line in turn["lines"].as_array().expect("reply lines") {
                if let Some(text) = line["text"].as_str() {
                    steps.push(Step::See {
                        find: Find::text(text.as_bytes()),
                        within: skein_lib::Duration::from_secs(30),
                    });
                }
            }
        }
        replies.push(u32::try_from(steps.len() - 1).expect("reply step"));
    }
    (steps, replies)
}

pub(crate) fn failed(now: skein_lib::Time, step: u32, why: &skein_fake_person::Why, tree: &skein_ui::Tree) {
    if !super::playground() {
        return;
    }
    eprintln!("story A at {} ns: person step {step}: {why:?}", now.as_nanos());
    for node in tree.nodes() {
        if let Some(name) = &node.name {
            eprintln!("story A page {:?}: {}", node.role, String::from_utf8_lossy(name));
        }
    }
    crate::world::playground_records();
}

pub(crate) fn outcome(transport: &temper_fake_git_hosted::Typed) -> Outcome {
    let forge = transport.forge.borrow();
    let api::Answer::Commit(main) = forge
        .domain
        .inspect(&forge.config, REPOSITORY, &api::Read::Branch { branch: b"main".as_slice().into() })
        .expect("widget's main")
    else {
        panic!("branch commit")
    };
    let object = forge.domain.object(main).expect("main's object");
    let scripts = scripts();
    let edited = scripts["cued"][0]["turns"][1]["lines"][0]["arguments"]["edits"][0]["new"].as_str().expect("CSV edit");
    let csv = object
        .tree
        .get(b"src/main.rs".as_slice())
        .is_some_and(|content| skein_lib::bytes::find(content, edited.as_bytes()).is_some());
    let api::Answer::Checks(checks) =
        forge.domain.inspect(&forge.config, REPOSITORY, &api::Read::Checks { commit: main }).expect("main's CI")
    else {
        panic!("checks")
    };
    let green = !checks.is_empty() && checks.iter().all(|check| check.status.state == api::Check::Passed);
    Outcome { main, csv, green, answered: crate::world::playground_answers() }
}
