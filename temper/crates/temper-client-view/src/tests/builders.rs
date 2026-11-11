//! Builders' read-only facts (temper/docs/design/client/pages/README.md, Builders).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic subtrees and their words"
)]
use super::fixtures as f;
use crate::look;
use jig_client::{Down, UnlinkedRequest, Watchable, contract as c};
use skein_lib::{List, Queue, Token};
use skein_ui::{Role, State, Tree};
use temper_client_domain::{self as d, Address};

fn address() -> Address {
    Address::Builders { project: f::PROJECT }
}
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
fn opened(role: bool) -> (d::Domain, Token, Option<Token>) {
    let (mut root, me_watch, _) = f::root_at(address(), &f::domain_limits());
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.deployment_role = role.then_some(c::HeldRole::Owner);
    let out = drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
    );
    let watch = out.iter().find_map(|r| match r {
        d::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Open { watch, subject } } }
            if *subject == Watchable::Jig(c::Subject::Deployment) =>
        {
            Some(*watch)
        }
        d::Request::Client { .. }
        | d::Request::Unlinked { .. }
        | d::Request::Show { .. }
        | d::Request::Leave { .. }
        | d::Request::SignOut
        | d::Request::Dropped { .. }
        | d::Request::Screen { .. }
        | d::Request::KeepLook { .. }
        | d::Request::KeepSignIn { .. }
        | d::Request::Tab { .. }
        | d::Request::Copy { .. }
        | d::Request::Push(_)
            | d::Request::Fact { .. } => None,
    });
    (root, me_watch, watch)
}
fn document() -> c::Deployment {
    let limits = f::domain_limits();
    let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).unwrap();
    let mut doc = jig_client_protocol_link::translate::documents::deployment(
        jig_client_test_records::deployment_rows_record(),
        &link,
    )
    .unwrap();
    doc.hosts = List::uncapped();
    doc.cut = false;
    doc
}
fn host(name: Option<&[u8]>, connected: bool, draining: bool) -> c::Host {
    c::Host {
        name: name.map(Into::into),
        channel: 7,
        slots: 2,
        used: 1,
        free: 1,
        connected,
        draining,
        awaiting_logins: false,
        rule: c::HostRule::Open,
        programs: List::with_capacity(0),
        kind: c::HostKind::Worker,
        grace_until: None,
        grace_remaining: None,
        runs: List::with_capacity(0),
        cut: false,
    }
}
fn delivered(doc: c::Deployment) -> d::Domain {
    let (mut root, _, watch) = opened(true);
    drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Deployment(Box::new(doc))),
                },
            },
        },
    );
    root
}
fn main(tree: &Tree) -> &[skein_ui::Node] {
    let (index, node) = tree.nodes().iter().enumerate().find(|(_, n)| n.role == Role::Main).unwrap();
    &tree.nodes()[index..index.checked_add(usize::try_from(node.size).unwrap()).unwrap()]
}
fn named<'a>(tree: &'a Tree, role: Role, name: &str) -> &'a skein_ui::Node {
    main(tree)
        .iter()
        .find(|n| n.role == role && n.name.as_deref() == Some(name.as_bytes()))
        .unwrap_or_else(|| panic!("missing {name}: {}", String::from_utf8_lossy(&f::nodes_text(main(tree)))))
}
fn rows(tree: &Tree) -> &[skein_ui::Node] {
    let (index, node) =
        tree.nodes().iter().enumerate().find(|(_, n)| n.styles.as_slice().contains(&look::SETTINGS_LINES)).unwrap();
    &tree.nodes()[index..index.checked_add(usize::try_from(node.size).unwrap()).unwrap()]
}
#[test]
fn each_host_is_a_line_with_its_slots_and_state() {
    let mut doc = document();
    for (name, connected, draining) in
        [(b"one".as_slice(), true, false), (b"two", true, true), (b"three", false, false), (b"four", false, true)]
    {
        doc.hosts.push(host(Some(name), connected, draining)).unwrap();
    }
    let root = delivered(doc);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let tree = view.tree();
        assert_eq!(skein_ui::outline::check(tree), None);
        named(tree, Role::Heading { level: 2 }, "Workers · 4");
        let mut lines = rows(tree).iter().filter(|n| n.role == Role::Slot && n.size == 8);
        for (name, state) in
            [("one", "Connected"), ("two", "Draining"), ("three", "Not connected"), ("four", "Not connected")]
        {
            let row = lines.next().unwrap();
            let start = tree.nodes().iter().position(|n| n.id == row.id).unwrap();
            let nodes = &tree.nodes()[start..start.checked_add(usize::try_from(row.size).unwrap()).unwrap()];
            assert_eq!(String::from_utf8_lossy(&f::nodes_text(nodes)), format!("{name}2 slots · 1 in use{state}"));
            assert!(nodes.iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_DETAIL)
                && n.name.as_deref() == Some("2 slots · 1 in use".as_bytes())));
            assert!(
                nodes.iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_WHEN)
                    && n.name.as_deref() == Some(state.as_bytes()))
            );
        }
        assert!(lines.next().is_none());
        let facts =
            named(tree, Role::Paragraph, "Each agent’s role names which workers it runs on. Ask temper to change it.");
        assert!(facts.styles.as_slice().contains(&look::SETTINGS_FACTS));
    }
}
#[test]
fn one_slot_says_slot() {
    let mut doc = document();
    let mut h = host(Some(b"small"), true, false);
    h.slots = 1;
    h.used = 0;
    doc.hosts.push(h).unwrap();
    let root = delivered(doc);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        named(view.tree(), Role::Text, "1 slot · 0 in use");
    }
}
#[test]
fn a_host_with_no_name_says_its_channel() {
    let mut doc = document();
    doc.hosts.push(host(None, true, false)).unwrap();
    let root = delivered(doc);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        named(view.tree(), Role::Text, "Worker 7");
    }
}
#[test]
fn no_host_connected_says_so_above_any_lines() {
    for retained in [false, true] {
        let mut doc = document();
        if retained {
            doc.hosts.push(host(Some(b"offline"), false, false)).unwrap();
        }
        let root = delivered(doc);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            named(tree, Role::Heading { level: 2 }, if retained { "Workers · 1" } else { "Workers · 0" });
            let empty = named(tree, Role::Paragraph, "No worker is connected.");
            assert!(empty.styles.as_slice().contains(&look::SETTINGS_EMPTY));
            if retained {
                assert!(
                    tree.nodes().iter().position(|n| n.id == empty.id).unwrap()
                        < tree.nodes().iter().position(|n| n.id == rows(tree)[0].id).unwrap()
                );
                assert_eq!(
                    rows(tree).iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_WHEN)).count(),
                    usize::from(retained)
                );
            } else {
                assert!(!main(tree).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_LINES)));
            }
        }
    }
}
#[test]
fn no_role_is_one_line() {
    let (root, _, watch) = opened(false);
    assert!(watch.is_none());
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let tree = view.tree();
        named(tree, Role::Paragraph, "Workers are seen by the deployment’s owners.");
        assert!(!main(tree).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_SECTION)));
    }
}
#[test]
fn an_unread_document_claims_nothing() {
    for end in [None, Some(c::WatchEnd::NotAllowed), Some(c::WatchEnd::Closed)] {
        let (mut root, _, watch) = opened(true);
        if let Some(end) = end {
            drive(
                &mut root,
                d::Event::Client {
                    event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch: watch.unwrap(), end } },
                },
            );
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            named(view.tree(), Role::Heading { level: 2 }, "Workers");
            assert!(!main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_LINES)
                || n.styles.as_slice().contains(&look::SETTINGS_EMPTY)
                || n.styles.as_slice().contains(&look::SETTINGS_FACTS)));
        }
    }
    let (mut root, me, _) = opened(false);
    drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch: me, end: c::WatchEnd::Closed } },
        },
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        named(view.tree(), Role::Heading { level: 2 }, "Workers");
    }
}
#[test]
fn a_cut_document_says_so() {
    for cut in [false, true] {
        let mut doc = document();
        doc.cut = cut;
        let root = delivered(doc);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert_eq!(
                main(view.tree()).iter().any(|n| n.name.as_deref() == Some(b"Some worker facts were omitted.")),
                cut
            );
        }
    }
}
#[test]
fn the_tab_has_no_act() {
    let mut doc = document();
    doc.hosts.push(host(None, true, false)).unwrap();
    let root = delivered(doc);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(
            rows(view.tree())
                .iter()
                .all(|n| n.binding.is_none() && n.external.is_none() && n.role != Role::Button && n.role != Role::Link)
        );
        assert!(main(view.tree()).iter().all(|n| n.role != Role::Button));
        let rail = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&look::RAIL)).unwrap();
        assert!(view.tree().nodes().iter().any(|n| n.role == Role::Link
            && n.name.as_deref() == Some(b"Deployment")
            && n.states.as_slice().contains(&State::Current)));
        assert_eq!(rail.role, Role::Navigation);
    }
}
#[test]
fn the_page_says_builder() {
    let root = delivered(document());
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let text = String::from_utf8_lossy(&f::nodes_text(main(view.tree()))).to_lowercase();
        assert!(text.contains("worker"));
        assert!(!text.contains("host"));
        assert!(!text.contains("builder"));
    }
}
#[test]
fn a_full_document_fits_the_tree_bound() {
    let mut doc = document();
    let count = f::domain_limits().client.domain.rows;
    let name = vec![b'b'; usize::try_from(c::PROGRAM_HOST_NAME).unwrap()];
    for _ in 0..count {
        doc.hosts.push(host(Some(&name), true, false)).unwrap();
    }
    let root = delivered(doc);
    for words in [f::words(), f::browser_words()] {
        let limits = crate::Limits::of(&f::domain_limits(), &words).unwrap();
        let mut view = crate::View::new(&limits, &f::domain_limits(), words, f::FORGE);
        f::build(&mut view, &root);
        assert_eq!(
            rows(view.tree()).iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_WHEN)).count(),
            usize::try_from(count).unwrap()
        );
        named(
            view.tree(),
            Role::Paragraph,
            "Each agent’s role names which workers it runs on. Ask temper to change it.",
        );
    }
}
#[test]
fn an_unlisted_project_keeps_its_missing_message() {
    let root = f::root_at(Address::Builders { project: 3 }, &f::domain_limits()).0;
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        named(view.tree(), Role::Paragraph, "Project 3 is not one of yours.");
        assert!(!main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_SECTION)));
    }
}
