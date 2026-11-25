//! A build refused replaces every page (client/tui/pages/refused.md, 1).
use super::{chat, fixtures as f};
use crate::{Own, Told, Tui, look};
use jig_client::{Event, Link, Up};
use skein_ui::Role;
use temper_client_domain::Domain;

pub(super) const VERSION: &[u8] = b"0.0.0 (b948f40d6df1)";

pub(super) fn root() -> Domain {
    let mut root = f::signed_out();
    f::client(&mut root, Event::Up { up: Up::Served { served: false, versions: None } });
    root
}

#[test]
fn a_build_refused_replaces_every_page_without_bindings_or_a_tick() {
    for family in [false, true] {
        for page in ["sign-in", "launchpad", "chat", "talk", "not yet started"] {
            let mut root = match page {
                "sign-in" => f::signed_out(),
                "launchpad" | "not yet started" => f::live(Some(f::me(&[b"Hello"]))).0,
                "chat" => chat::reply_root("plain chat"),
                "talk" => chat::talk_root("open"),
                _ => unreachable!("page row"),
            };
            let mut tui = Tui::new(f::ADDRESS);
            if page == "not yet started" {
                tui.step(Told::Place { command: Some(Own::Ask) });
            }
            let mut view = f::view();
            f::build(&mut view, &tui, &root);
            assert!(!crate::refused(&root, &tui), "{page}, family={family}");
            f::client(
                &mut root,
                Event::Up {
                    up: if family {
                        Up::FamilyServed { family: temper_client_protocol_wire::FORGE, versions: None, served: false }
                    } else {
                        Up::Served { served: false, versions: None }
                    },
                },
            );
            if family {
                assert_ne!(root.client().link().not_served, []);
                assert_ne!(root.client().link().state, Link::OutOfDate);
            } else {
                assert_eq!(root.client().link().state, Link::OutOfDate);
            }
            assert!(crate::refused(&root, &tui), "{page}, family={family}");
            f::build(&mut view, &tui, &root);
            let tree = view.tree();
            assert_eq!(skein_ui::outline::check(tree), None);
            assert_eq!(f::count(tree, Role::TextField), 0);
            assert_eq!(f::count(tree, Role::TextArea), 0);
            assert_eq!(f::count(tree, Role::Log), 0);
            assert_eq!(f::count(tree, Role::Paragraph), 3);
            for node in tree.nodes() {
                assert!(node.binding.is_none(), "{page}, family={family}");
                assert_ne!(node.region, Some(look::HEADER));
                assert_ne!(node.region, Some(look::BOX));
            }
            assert_eq!(view.tick(), None);
            assert_eq!(crate::escape(&view), None);
            assert_eq!(crate::pane_key(&view), None);
            assert_eq!(crate::walk_key(&view), None);
            let (left, right) = f::bottom_halves(tree);
            assert_eq!(left.role, Role::Status);
            assert_eq!(f::text(tree, left).as_ref(), "● Refused".as_bytes());
            assert_eq!(left.styles.as_slice(), &[look::DOWN]);
            assert_eq!(f::text(tree, right).as_ref(), b"Ctrl+C to quit");
            tui.step(Told::Front { set_up: false, box_log: Box::default() });
            assert!(!crate::refused(&root, &tui), "front keeps the welcome, {page}");
        }
    }
}

#[test]
fn the_refused_lines_name_the_place_and_omit_an_empty_build() {
    let root = root();
    let named = crate::Origin::parse(b"https://temper.test").expect("named origin");
    for (address, boxed, version, first, last) in [
        (
            f::ADDRESS,
            false,
            VERSION,
            b"127.0.0.1:8367 runs another build of temper.".as_slice(),
            b"Start the build 127.0.0.1:8367 runs, then try again.".as_slice(),
        ),
        (
            &named,
            false,
            VERSION,
            b"temper.test runs another build of temper.",
            b"Start the build temper.test runs, then try again.",
        ),
        (f::ADDRESS, true, VERSION, b"box runs another build of temper.", b"Start the build box runs, then try again."),
        (
            f::ADDRESS,
            false,
            b"",
            b"127.0.0.1:8367 runs another build of temper.",
            b"Start the build 127.0.0.1:8367 runs, then try again.",
        ),
    ] {
        let mut tui = Tui::new(address);
        tui.header(false, version.into());
        if boxed {
            tui.step(Told::Boxed);
        }
        let mut view = f::view();
        f::build(&mut view, &tui, &root);
        let tree = view.tree();
        let mut framed = false;
        for node in tree.nodes() {
            if node.region == Some(look::TOP) {
                assert_eq!(node.role, Role::Main);
                framed = true;
            }
        }
        assert!(framed);
        f::find(tree, Role::Heading { level: 1 }, b"temper");
        f::find(tree, Role::Paragraph, first);
        f::find(tree, Role::Paragraph, last);
        if !version.is_empty() {
            f::find(tree, Role::Paragraph, b"This one is temper 0.0.0 (b948f40d6df1).");
        }
        assert_eq!(f::count(tree, Role::Paragraph), if version.is_empty() { 3 } else { 4 });
    }
}
