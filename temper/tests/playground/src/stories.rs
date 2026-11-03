//! The native person's stories (temper/docs/design/playground.md, 5.2 and 9).

use skein_fake_person::{Find, Step, Target};
use skein_lib::Duration;
use skein_ui::Role;
use temper_client_domain::{Address, ChatsFilter};

const WAIT: Duration = Duration::from_secs(30);

/// The three suggested requests, each reply read, then the CSV change seen landed.
#[must_use]
pub fn a() -> Vec<Step<Address>> {
    let scripts: serde_json::Value =
        serde_json::from_slice(temper_playground::stories::A.llm).expect("story A's scripts");
    let mut steps = vec![
        Step::Go { address: Address::Welcome { step: temper_client_domain::Step::Welcome } },
        see(Find::named(Role::Button, b"Try the playground")),
        press(Role::Button, b"Try the playground"),
        see(Find::named(Role::Button, b"Add CSV export to widget")),
        see(Find::text(b"playground/widget")),
    ];
    for (index, script) in scripts["asked"].as_array().expect("three suggested requests").iter().enumerate() {
        if index > 0 {
            steps.push(chats());
            steps.push(see(Find::named(Role::Button, script["cue"].as_str().expect("suggestion").as_bytes())));
        }
        steps.push(press(Role::Button, script["cue"].as_str().expect("suggestion").as_bytes()));
        steps.push(see(Find::named(Role::TextArea, b"Start a chat")));
        steps.push(press(Role::Button, b"Start chat"));
        for turn in script["turns"].as_array().expect("scripted turns") {
            for line in turn["lines"].as_array().expect("reply lines") {
                if let Some(text) = line["text"].as_str() {
                    steps.push(see(Find::text(text.as_bytes())));
                }
            }
        }
    }
    let csv_chat = Find::named(Role::Link, b"Chat")
        .within(Box::new([Target::Role { role: Role::Item, name: b"Add CSV export to widget".as_slice().into() }]));
    steps.push(chats());
    steps.push(see(csv_chat.clone()));
    steps.push(Step::Press { find: csv_chat });
    steps.push(see(Find::exactly_named(Role::Status, b"Landed")));
    steps
}

fn chats() -> Step<Address> {
    Step::Go { address: Address::Chats { filter: ChatsFilter::All, start: false } }
}

fn see(find: Find) -> Step<Address> {
    Step::See { find, within: WAIT }
}

fn press(role: Role, name: &[u8]) -> Step<Address> {
    Step::Press { find: Find::named(role, name) }
}
