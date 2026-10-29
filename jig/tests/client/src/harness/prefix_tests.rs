//! Rendered inbox navigation through the shell's codec
//! (jig/docs/design/client/domain/testing.md, 4).
use super::{ClientId, Until, World};
use crate::{Scripted, Test, stories, test::list};
use jig_client::contract::{InboxEntry, InboxGroup, InboxReadGroup, ObjectKey, Read, ReadValue};
use jig_client_scripted_engine::{Change, Rows, Source};
use jig_client_test_domain::Action;
use jig_client_view::{address, page::Page};
use skein_fake_person::{Find, Step};
use skein_lib::{Duration, Time, Wall};
use skein_ui::{Act, Role};
use skein_world::domain::Verdict;

const PARTY: u64 = 7;
const PROJECT: u32 = 1;
const TASK: u64 = 9;
const LINK: &[u8] = b"Task 9: Chat nine";
const HEADING_LEVEL: u8 = 1;
const CODEC: address::Codec<'_> = address::Codec { prefix: b"/test/prefixed", ..address::REFERENCE };

fn inbox() -> (World<Test, Scripted>, ClientId) {
    let mut world = stories::chat_ready(5091);
    let task = stories::task();
    for change in [
        Change::Object {
            project: PROJECT,
            key: ObjectKey::Result(TASK),
            value: ReadValue::Result(Box::new(jig_client::contract::Result {
                task: task.chip,
                ending: jig_client::contract::Ending::Done,
                value: None,
                at: Some(Wall::EPOCH),
                details: None,
                acts: list([]),
            })),
        },
        Change::Source(Source {
            party: PARTY,
            read: Read::Inbox { group: InboxReadGroup::Updates, project: None, kind: None, goal: None, after: None },
            rows: Rows::Inbox(vec![InboxEntry {
                reply: None,
                lineage: list([]),
                goal: None,
                recipient: None,
                stall: None,
                object: ObjectKey::Result(TASK),
                project: PROJECT,
                group: InboxGroup::Results,
                position: 1,
                at: Wall::EPOCH,
            }]),
        }),
    ] {
        world.front().engine.act(Time::ZERO, Wall::EPOCH, change).expect("inbox fixture fits");
    }
    let client = world.client(PARTY, stories::inbox());
    (world, client)
}

/// Replaces `browser_tier::under_a_prefix_the_inbox_s_links_carry_the_prefix`.
#[test]
fn an_inbox_link_under_a_prefix_opens_its_task() {
    let (mut world, client) = inbox();
    let link = Find::exactly_named(Role::Link, LINK);
    world.person(client, vec![Step::See { find: link, within: Duration::from_secs(1) }]);
    world.run(Until::Done);
    let node = world
        .tree(client)
        .nodes()
        .iter()
        .find(|node| node.role == Role::Link && node.name.as_deref() == Some(LINK))
        .expect("inbox task link")
        .id;
    let action =
        jig_client_test_view::decode(&mut world.clients[client.0].view, node, Act::Press).expect("link has an action");
    let Action::Jig(jig_client_view::action::Act::Go { address }) = action else { panic!("inbox link navigates") };
    let href = address::print(&CODEC, address);
    assert_eq!(&*href, b"/test/prefixed/tasks/9");
    world.act(client, node, Act::Press);
    world.person(
        client,
        vec![Step::See {
            find: Find::exactly_named(Role::Heading { level: HEADING_LEVEL }, b"Task 9"),
            within: Duration::from_secs(1),
        }],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(address::parse(&CODEC, &href), address::Place::Jig(address::Address::task(TASK)));
    assert!(
        matches!(world.root(client).jig().page(), Page::Work { address, .. } if *address == address::Address::task(TASK))
    );
}
