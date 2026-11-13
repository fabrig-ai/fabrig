//! Deployment read states in both shells (temper/docs/design/client/pages/README.md).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary tests inspect semantic subtrees and authentic document rows"
)]
use super::fixtures as f;
use crate::look;
use jig_client::{Down, Readable, UnlinkedRequest, contract as c};
use skein_lib::{List, Queue, Token};
use skein_ui::{Node, Role, State, Tree};
use temper_client_domain::{self as d, Address};

fn drive(root: &mut d::Domain, event: d::Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        event,
        &mut out,
    );
    out
}
fn deliver(root: &mut d::Domain, watch: Token, delivery: c::Delivery) -> Queue<d::Request> {
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) },
            },
        },
    )
}
fn answer(root: &mut d::Domain, read: Token, result: c::ReadResult) -> Queue<d::Request> {
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone { read, result: jig_client::Readout::Jig(result) },
            },
        },
    )
}
fn subtree<'a>(tree: &'a Tree, node: &Node) -> &'a [Node] {
    let at = tree.nodes().iter().position(|n| n.id == node.id).unwrap();
    &tree.nodes()[at..at.checked_add(usize::try_from(node.size).unwrap()).unwrap()]
}
fn main(tree: &Tree) -> &[Node] {
    subtree(tree, tree.nodes().iter().find(|n| n.role == Role::Main).unwrap())
}
fn named(tree: &Tree, role: Role, name: &str) {
    assert!(main(tree).iter().any(|n| n.role == role && n.name.as_deref() == Some(name.as_bytes())), "missing {name}");
}
fn text(bytes: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(bytes.into(), 64).unwrap())
}
fn project(cut: bool) -> c::Project {
    let limits = f::domain_limits();
    let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).unwrap();
    let mut doc =
        jig_client_protocol_link::translate::documents::project(jig_client_test_records::owner_project_record(), &link)
            .unwrap();
    doc.number = f::PROJECT;
    doc.members = List::uncapped();
    for (name, login, kind, role) in [
        ("Pat", "pat-login", c::PartyKind::Person, c::HeldRole::Owner),
        ("", "bot", c::PartyKind::Service, c::HeldRole::Maintainer),
        ("", "", c::PartyKind::Service, c::HeldRole::Member),
        ("Observer", "", c::PartyKind::Person, c::HeldRole::Observer),
        ("On call", "", c::PartyKind::Person, c::HeldRole::Policy { role: 7 }),
    ] {
        doc.members
            .push(c::ProjectMember { party: 9, name: text(name.as_bytes()), login: text(login.as_bytes()), kind, role })
            .unwrap();
    }
    doc.cut = cut;
    doc
}
fn deployment(mark: u64, cut: bool) -> c::Deployment {
    let limits = f::domain_limits();
    let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).unwrap();
    let mut doc = jig_client_protocol_link::translate::documents::deployment(
        jig_client_test_records::deployment_rows_record(),
        &link,
    )
    .unwrap();
    doc.tasks = c::Capacity { used: 3, limit: 20 };
    doc.workers = c::Capacity { used: 1, limit: 4 };
    doc.attempts_capacity = c::Capacity { used: 2, limit: 8 };
    doc.waiting_attempts = 5;
    doc.proposals = c::Capacity { used: 7, limit: 16 };
    doc.store = c::StoreFill { bytes: 4_100_000, mark };
    doc.projects = List::uncapped();
    for (project, used) in [(f::PROJECT, 2), (99, 1)] {
        doc.projects
            .push(c::ProjectCapacity { project, tasks: c::Capacity { used, limit: 10 }, proposals: 0 })
            .unwrap();
    }
    doc.cut = cut;
    doc
}
fn project_read(out: &Queue<d::Request>) -> Token {
    out.iter()
        .find_map(|r| {
            if let d::Request::Unlinked {
                request:
                    UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(c::Read::Project { .. }) } },
            } = r
            {
                Some(*read)
            } else {
                None
            }
        })
        .expect("project read")
}
fn deployment_watch(out: &Queue<d::Request>) -> Option<Token> {
    out.iter().find_map(|r| {
        if let d::Request::Unlinked {
            request:
                UnlinkedRequest::Down {
                    down: Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Deployment) },
                },
        } = r
        {
            Some(*watch)
        } else {
            None
        }
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Read {
    Unread,
    Refused,
    Full,
    Cut,
}
#[derive(Clone, Copy, Debug)]
struct Case {
    members: bool,
    read: Read,
    role: Option<c::HeldRole>,
    alone: bool,
    mark: u64,
}
#[test]
fn deployment_pages_keep_their_read_states() {
    let owner = Some(c::HeldRole::Owner);
    for case in [
        Case { members: true, read: Read::Full, role: owner, alone: false, mark: 0 },
        Case { members: true, read: Read::Cut, role: owner, alone: false, mark: 0 },
        Case { members: true, read: Read::Unread, role: owner, alone: false, mark: 0 },
        Case { members: true, read: Read::Refused, role: owner, alone: false, mark: 0 },
        Case { members: true, read: Read::Full, role: None, alone: false, mark: 0 },
        Case { members: true, read: Read::Full, role: Some(c::HeldRole::Policy { role: 2 }), alone: false, mark: 0 },
        Case { members: true, read: Read::Unread, role: None, alone: true, mark: 0 },
        Case { members: false, read: Read::Full, role: owner, alone: false, mark: 64_000_000 },
        Case { members: false, read: Read::Full, role: Some(c::HeldRole::Member), alone: false, mark: 0 },
        Case { members: false, read: Read::Unread, role: owner, alone: false, mark: 0 },
        Case { members: false, read: Read::Full, role: None, alone: false, mark: 0 },
        Case { members: false, read: Read::Cut, role: owner, alone: false, mark: 64_000_000 },
        Case { members: false, read: Read::Full, role: owner, alone: true, mark: 64_000_000 },
    ] {
        let (mut root, me_watch, _) = f::root_at(Address::Settings { project: f::PROJECT }, &f::domain_limits());
        let mut me = root.pages().me(root.client()).unwrap().clone();
        // A role is removed after delivery in the no-role Limits row, to keep a stale document present.
        me.deployment_role = if !case.members && case.role.is_none() { owner } else { case.role };
        if case.alone {
            me.projects.clear();
        }
        deliver(&mut root, me_watch, c::Delivery::Me(Box::new(me.clone())));
        let address = if case.members { Address::Members } else { Address::Limits };
        let out = drive(&mut root, d::Event::Went { place: d::Place::Address(address) });
        if case.members && !case.alone {
            let read = project_read(&out);
            match case.read {
                Read::Full | Read::Cut => {
                    let out = answer(
                        &mut root,
                        read,
                        c::ReadResult::Value(c::ReadValue::Project(Box::new(project(case.read == Read::Cut)))),
                    );
                    // A separate refusal stays beside the first project's successful outcome.
                    let second = project_read(&out);
                    answer(&mut root, second, c::ReadResult::None(c::ReadNone::Refused));
                }
                Read::Refused => {
                    answer(&mut root, read, c::ReadResult::None(c::ReadNone::Refused));
                }
                Read::Unread => {}
            }
        } else if !case.members {
            if case.read != Read::Unread {
                let watch = deployment_watch(&out).unwrap();
                deliver(
                    &mut root,
                    watch,
                    c::Delivery::Deployment(Box::new(deployment(case.mark, case.read == Read::Cut))),
                );
            }
            if case.role.is_none() {
                me.deployment_role = None;
                deliver(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
            }
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            check(tree, case);
        }
    }
}

fn check(tree: &Tree, case: Case) {
    assert_eq!(skein_ui::outline::check(tree), None, "{case:?}");
    let body = main(tree);
    let nav = body.iter().find(|n| n.styles.as_slice().contains(&look::SETTINGS_TABS));
    if case.alone {
        assert!(nav.is_none());
    } else {
        let current: Vec<_> =
            subtree(tree, nav.unwrap()).iter().filter(|n| n.states.as_slice().contains(&State::Current)).collect();
        assert_eq!(current.len(), 1);
        assert_eq!(
            current[0].name.as_deref(),
            Some(if case.members { b"Members".as_slice() } else { b"Limits".as_slice() }),
            "current Deployment tab: {case:?}"
        );
    }
    let all = String::from_utf8_lossy(&f::nodes_text(body)).into_owned();
    let lines: Vec<_> = body.iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_LINES)).collect();
    for lines in &lines {
        assert!(subtree(tree, lines).iter().all(|n| n.binding.is_none() && n.external.is_none()));
    }
    if case.members {
        named(tree, Role::Heading { level: 2 }, "The deployment");
        named(
            tree,
            Role::Paragraph,
            match case.role {
                Some(c::HeldRole::Owner) => "You are its owner.",
                None => "You hold no role in it.",
                Some(
                    c::HeldRole::Maintainer | c::HeldRole::Member | c::HeldRole::Observer | c::HeldRole::Policy { .. },
                ) => "Your deployment role: role 2.",
            },
        );
        named(tree, Role::Paragraph, "Who owns an area is Team’s, on each project.");
        if !case.alone {
            match case.read {
                Read::Full | Read::Cut => {
                    named(tree, Role::Heading { level: 2 }, "Shop · 5");
                    let rows: Vec<_> = subtree(tree, lines[0])
                        .iter()
                        .filter(|n| n.role == Role::Text && n.styles.as_slice().contains(&look::SETTINGS_WHEN))
                        .collect();
                    assert_eq!(
                        rows.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
                        [b"owner".as_slice(), b"maintainer", b"member", b"observer", b"role 7"]
                    );
                    for name in ["Pat", "bot", "Party 9", "person", "service"] {
                        named(tree, Role::Text, name);
                    }
                    named(tree, Role::Heading { level: 2 }, "Second");
                    named(tree, Role::Paragraph, "Read by the project’s members.");
                    assert_eq!(all.contains("Some members were omitted."), case.read == Read::Cut);
                }
                Read::Unread | Read::Refused => {
                    named(tree, Role::Heading { level: 2 }, "Shop");
                    assert_eq!(lines.len(), 0);
                    assert_eq!(all.contains("Read by the project’s members."), case.read == Read::Refused);
                }
            }
        }
    } else if case.role.is_none() {
        named(tree, Role::Paragraph, "Limits are seen by the deployment’s owners.");
        assert!(!all.contains("Capacities"), "no role must not draw capacities");
        assert_eq!(lines.len(), 0);
    } else if case.read == Read::Unread {
        named(tree, Role::Heading { level: 2 }, "Limits");
        assert!(!all.contains("Capacities"));
        assert_eq!(lines.len(), 0);
    } else {
        named(tree, Role::Heading { level: 2 }, "Capacities");
        named(tree, Role::Heading { level: 2 }, "By project");
        for value in ["3 of 20", "1 of 4", "2 of 8 · 5 waiting", "7 of 16", "2 of 10 tasks", "1 of 10 tasks"] {
            named(tree, Role::Text, value);
        }
        named(tree, Role::Text, if case.alone { "Project 1" } else { "Shop" });
        named(tree, Role::Text, "Project 99");
        named(tree, Role::Text, if case.mark == 0 { "4.1 MB" } else { "4.1 MB of 64 MB" });
        assert_eq!(all.contains("Some capacity facts were omitted."), case.read == Read::Cut);
        named(
            tree,
            Role::Paragraph,
            "The deployment’s owner sets these in its configuration; temper reads them at start.",
        );
    }
}
