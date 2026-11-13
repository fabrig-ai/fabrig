#![expect(clippy::disallowed_methods, reason = "semantic-tree assertions use ordinary Rust iterators")]
use super::*;
use crate::tests::fixtures as f;
use skein_lib::Token;
use skein_ui::{Act as UiAct, LogEdge, Tree};
use temper_client_domain::Action;
use temper_client_protocol_wire::v1 as wire;

fn root() -> Domain {
    f::sheet_root()
}

fn reached(id: u64, name: &[u8], private: bool) -> wire::Reached {
    wire::Reached::new(
        &wire::CEILINGS,
        wire::ReachedParts {
            id,
            owner: wire::Account::new(&wire::CEILINGS, wire::AccountParts { id: 3, login: b"acme".as_slice().into() })
                .unwrap(),
            name: name.into(),
            private,
        },
    )
    .unwrap()
}

fn tree(sheet: &sheet::Sheet) -> (Tree, Bindings) {
    let root = root();
    let mut b = Builder::new(&f::tree_limits());
    let mut bindings = Bindings::new(200, 8, 1);
    choices(&mut b, &root, sheet, &f::words(), &mut bindings);
    picker(&mut b, &root, sheet, b"Shop", &f::words(), &mut bindings);
    let mut form = forge::form(f::PROJECT);
    form.confirmed = true;
    more(&mut b, sheet, &form, &f::words(), &mut bindings);
    (b.finish(), bindings)
}

fn node<'a>(tree: &'a Tree, role: Role, name: &[u8]) -> &'a skein_ui::Node {
    tree.nodes().iter().find(|node| node.role == role && node.name.as_deref() == Some(name)).expect("named node")
}

#[test]
fn the_list_reads_on_and_holds_while_busy() {
    let root = root();
    let mut sheet = sheet::Sheet::new(f::PROJECT, root.forges());
    sheet.rows.push(reached(1, b"shop", false)).unwrap();
    sheet.more = true;
    sheet.reading = sheet::Reading::Asked { key: Token::new(7), page: 2 };
    let (tree, bindings) = tree(&sheet);
    let list = node(&tree, Role::List, b"Repositories");
    assert!(list.states.as_slice().contains(&State::Continues));
    assert!(list.states.as_slice().contains(&State::Busy));
    assert_eq!(
        super::super::bindings::decode(&bindings, list.binding.unwrap().token, UiAct::Near { edge: LogEdge::Foot }),
        Some(Action::Forge(forge::Act::Sheet(Act::NearEnd)))
    );
    assert!(f::find(&tree, Role::Search, b"Find by name").is_some());
    assert!(f::text(&tree).windows(b"1 so far".len()).any(|s| s == b"1 so far"));
    sheet.more = false;
    sheet.reading = sheet::Reading::Idle;
    let (tree, _) = self::tree(&sheet);
    let list = node(&tree, Role::List, b"Repositories");
    assert!(!list.states.as_slice().contains(&State::Continues));
    assert!(!list.states.as_slice().contains(&State::Busy));
    assert!(f::find(&tree, Role::Search, b"Find by name").is_none());
    assert!(f::find(&tree, Role::Select, b"Forge").is_none());
    assert!(f::find(&tree, Role::Select, b"Account").is_none());
    for id in 2..=9 {
        sheet.rows.push(reached(id, b"other", false)).unwrap();
    }
    sheet.find.text = b"shop".as_slice().into();
    let (filtered, _) = self::tree(&sheet);
    assert!(f::text(&filtered).windows(b"1 of 9 repositories".len()).any(|s| s == b"1 of 9 repositories"));
    for picked in [false, true] {
        if picked {
            sheet
                .picks
                .push(sheet::Pick {
                    id: 1,
                    owner_id: 3,
                    owner: b"acme".as_slice().into(),
                    name: b"shop".as_slice().into(),
                })
                .unwrap();
        }
        sheet.find.text = b" \tMiSsInG\n ".as_slice().into();
        let (empty, _) = self::tree(&sheet);
        let none = empty.nodes().iter().position(|n| n.styles.as_slice().contains(&look::CONNECT_NONE)).unwrap();
        assert_eq!(f::node_text(&empty, none).as_ref(), "Nothing here matches “MiSsInG”.".as_bytes());
        assert!(!empty.nodes().iter().any(|n| n.styles.as_slice().contains(&look::CONNECT_COUNT)));
    }
    sheet.rows.clear();
    sheet.find.text = b" \t ".as_slice().into();
    let (blank, _) = self::tree(&sheet);
    let none = blank.nodes().iter().position(|n| n.styles.as_slice().contains(&look::CONNECT_NONE)).unwrap();
    assert_eq!(f::node_text(&blank, none).as_ref(), b"None reachable.");
}

#[test]
fn checks_keep_names_marks_picks_and_hide_home_for_plural() {
    let root = root();
    let mut sheet = sheet::Sheet::new(f::PROJECT, root.forges());
    sheet.more = false;
    sheet.rows.push(reached(1, b"secret", true)).unwrap();
    sheet.rows.push(reached(2, b"second", false)).unwrap();
    for row in &sheet.rows {
        sheet
            .picks
            .push(sheet::Pick {
                id: row.id(),
                owner_id: row.owner().id(),
                owner: row.owner().login().into(),
                name: row.name().into(),
            })
            .unwrap();
    }
    let (tree, bindings) = tree(&sheet);
    let check = node(&tree, Role::Check, b"acme/secret");
    assert!(check.states.as_slice().contains(&State::Pressed));
    assert_eq!(
        super::super::bindings::decode(&bindings, check.binding.unwrap().token, UiAct::Toggle),
        Some(Action::Forge(forge::Act::Sheet(Act::Pick { id: 1, on: false })))
    );
    assert!(
        tree.nodes()
            .iter()
            .flat_map(|n| n.text.iter())
            .any(|run| run.run == Run::Emphasis && run.text.as_ref() == b"private")
    );
    assert!(f::find(&tree, Role::Check, b"Make it the project's home").is_none());
    assert_eq!(
        button_name(&sheet::Button::Adopt { count: 2, first: b"acme/secret".as_slice().into() }, &f::words()).as_ref(),
        b"Adopt 2 repositories"
    );
    assert_eq!(
        button_name(&sheet::Button::Adopt { count: 1, first: b"acme/secret".as_slice().into() }, &f::words()).as_ref(),
        b"Adopt acme/secret"
    );
    sheet.rows.push(reached(3, b"kept", false)).unwrap();
    sheet.rows.push(reached(4, b"copy", false)).unwrap();
    let (marked, _) = self::tree(&sheet);
    for name in [b"acme/kept".as_slice(), b"acme/copy"] {
        assert!(node(&marked, Role::Check, name).states.as_slice().contains(&State::Disabled));
    }
    for mark in [b"In Shop".as_slice(), "Copying…".as_bytes()] {
        assert!(
            marked
                .nodes()
                .iter()
                .flat_map(|n| n.text.iter())
                .any(|run| run.run == Run::Emphasis && run.text.as_ref() == mark)
        );
    }
}

#[test]
fn more_keeps_the_forms_fields_and_summary() {
    let root = root();
    let sheet = sheet::Sheet::new(f::PROJECT, root.forges());
    let (tree, _) = tree(&sheet);
    let more = node(&tree, Role::Disclosure, b"More");
    assert!(more.states.is_empty());
    assert_eq!(more.description.as_deref(), Some("· Owned, it runs CI".as_bytes()));
    for name in [b"Branch prefix".as_slice(), b"Landing branch", b"Checks: gate numbers, separated by spaces or commas"]
    {
        assert!(f::find(&tree, Role::TextField, name).is_some());
    }
    for (name, expected) in [
        (b"Branch prefix".as_slice(), b"temper/".as_slice()),
        (b"Landing branch", b""),
        (b"Checks: gate numbers, separated by spaces or commas", b""),
    ] {
        let field = node(&tree, Role::TextField, name).value.as_ref().unwrap();
        assert_eq!((field.text.as_ref(), field.written), (expected, 0));
    }
    assert!(f::find(&tree, Role::Check, b"It runs CI").is_some());
    assert!(f::find(&tree, Role::Check, b"Make it the project's home").is_some());
}
