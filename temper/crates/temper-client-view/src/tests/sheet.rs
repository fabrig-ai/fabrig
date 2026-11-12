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
fn adopt_is_settings_repositories_with_a_dialog_and_escape_cancel() {
    let mut root = root();
    let mut view = f::page_view(&root);
    let tree = view.tree();
    assert!(f::find(tree, Role::Dialog, b"Add repositories to Shop").is_some());
    assert!(f::find(tree, Role::Heading { level: 1 }, b"What Shop works on.").is_some());
    assert!(f::find(tree, Role::Link, b"Add repositories").is_some());
    assert!(f::find(tree, Role::TextField, b"Forge number").is_none());
    let cancel = node(tree, Role::Button, b"Cancel");
    assert_eq!(cancel.shortcut, Some(Shortcut::ESCAPE));
    let cancel = cancel.id;
    let owner = f::find(view.tree(), Role::TextField, b"Owner").unwrap();
    let event = crate::act_event(&mut view, owner, UiAct::Input { value: b"acme".as_slice().into() }).unwrap();
    f::root_event(&mut root, event);
    assert_eq!(root.forge().sheet().unwrap().not_listed.0.text.as_ref(), b"acme");
    let name = f::find(view.tree(), Role::TextField, b"Name").unwrap();
    let event = crate::act_event(&mut view, name, UiAct::Input { value: b"secret".as_slice().into() }).unwrap();
    f::root_event(&mut root, event);
    f::build(&mut view, &root);
    let send = f::find(view.tree(), Role::Button, b"Adopt acme/secret").unwrap();
    assert_eq!(crate::act(&mut view, send, UiAct::Press), Some(Action::Forge(forge::Act::Sheet(Act::Send))));

    assert_eq!(
        crate::act(&mut view, cancel, UiAct::Press),
        Some(Action::Pages(pages::Act::Go { address: Address::Repositories { project: f::PROJECT } }))
    );
}
#[test]
fn the_out_of_date_form_uses_its_shells_words_and_disables_adoption() {
    let mut root = root();
    let mut view = f::page_view(&root);
    input_sheet(&mut root, &mut view, Role::TextField, b"Owner", b"acme");
    input_sheet(&mut root, &mut view, Role::TextField, b"Name", b"secret");
    f::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyServed { family: temper_client_protocol_wire::FORGE, versions: None },
            },
        },
    );
    for (words, said) in [
        (f::browser_words(), b"This page is out of date for adopting repositories: reload it.".as_slice()),
        (f::words(), b"This temper is out of date for adopting repositories: run the temper the engine runs.".as_slice()),
    ] {
        let mut view = f::view_of(words);
        f::build(&mut view, &root);
        assert!(f::find(view.tree(), Role::Status, said).is_some());
        let button = node(view.tree(), Role::Button, b"Adopt acme/secret");
        assert!(button.states.as_slice().contains(&State::Disabled));
        let button = button.id;
        assert!(crate::act_event(&mut view, button, UiAct::Press).is_none());
    }
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

fn sheet_act(root: &mut Domain, act: Act) {
    f::root_event(root, temper_client_domain::Event::Act { action: Action::Forge(forge::Act::Sheet(act)) });
}
fn press_sheet(root: &mut Domain, view: &mut crate::View, role: Role, name: &[u8]) {
    let id = node(view.tree(), role, name).id;
    let act = if role == Role::Choice { UiAct::Toggle } else { UiAct::Press };
    let action = crate::act(view, id, act).expect("sheet action");
    f::root_event(root, temper_client_domain::Event::Act { action });
    f::build(view, root);
}
fn input_sheet(root: &mut Domain, view: &mut crate::View, role: Role, name: &[u8], text: &[u8]) {
    let id = node(view.tree(), role, name).id;
    let event = crate::act_event(view, id, UiAct::Input { value: text.into() }).expect("sheet input");
    f::root_event(root, event);
    f::build(view, root);
}
type HeadNode = (Role, Option<Box<[u8]>>, Box<[skein_ui::Style]>);
fn head(tree: &Tree) -> Box<[HeadNode]> {
    let start =
        tree.nodes().iter().position(|node| node.styles.as_slice().contains(&look::CONNECT_SHEET_HEAD)).unwrap();
    let count = usize::try_from(tree.nodes()[start].size).unwrap();
    let end = start.checked_add(count).unwrap();
    tree.nodes()[start..end].iter().map(|n| (n.role, n.name.clone(), Box::from(n.styles.as_slice()))).collect()
}
#[test]
fn empty_source_keeps_the_head_and_names_its_forge_and_button() {
    let mut root = root();
    let mut view = f::browser_page_view(&root);
    let header = head(view.tree());
    press_sheet(&mut root, &mut view, Role::Choice, b"Make empty");
    assert_eq!(head(view.tree()), header, "source bodies follow the unchanged head");
    assert!(node(view.tree(), Role::Button, b"Make it").states.as_slice().contains(&State::Disabled));
    assert_eq!(node(view.tree(), Role::TextField, b"Organization").value.as_ref().unwrap().text.as_ref(), b"work");
    assert!(f::find(view.tree(), Role::Select, b"Forge").is_none());
    assert!(
        f::text(view.tree())
            .windows(b"On Forgejo on this machine, with a README and a starter CI check.".len())
            .any(|part| part == b"On Forgejo on this machine, with a README and a starter CI check.")
    );
    input_sheet(&mut root, &mut view, Role::TextField, b"Name", b"hello");
    let send = node(view.tree(), Role::Button, b"Make work/hello");
    assert!(!send.states.as_slice().contains(&State::Disabled));
    let send = send.id;
    assert_eq!(crate::act(&mut view, send, UiAct::Press), Some(Action::Forge(forge::Act::Sheet(Act::Send))));
    input_sheet(&mut root, &mut view, Role::TextField, b"Name", &b"x".repeat(257));
    assert!(node(view.tree(), Role::Button, b"Make work/hello").states.as_slice().contains(&State::Disabled));
    input_sheet(&mut root, &mut view, Role::TextField, b"Organization", b"");
    assert!(node(view.tree(), Role::Button, b"Make it").states.as_slice().contains(&State::Disabled));
    press_sheet(&mut root, &mut view, Role::Choice, b"Copy public");
    assert_eq!(head(view.tree()), header);
    assert!(node(view.tree(), Role::Button, b"Copy").states.as_slice().contains(&State::Disabled));
    press_sheet(&mut root, &mut view, Role::Choice, b"Adopt");
    assert_eq!(head(view.tree()), header);
}
#[test]
fn copy_lines_have_separate_words_and_their_repairs_reach_the_sheet() {
    let mut root = root();
    let mut view = f::browser_page_view(&root);
    press_sheet(&mut root, &mut view, Role::Choice, b"Copy public");
    input_sheet(
        &mut root,
        &mut view,
        Role::TextArea,
        b"Their addresses, one per line",
        b"bad\ngithub/acme/free\ngithub.com/acme/kept\ngithub.com/acme/copy",
    );
    let area = node(view.tree(), Role::TextArea, b"Their addresses, one per line");
    assert!(area.styles.as_slice().contains(&look::CONNECT_ADDRESSES));
    let area_at = view.tree().nodes().iter().position(|n| n.id == area.id).unwrap();
    let caption_at = view
        .tree()
        .nodes()
        .iter()
        .enumerate()
        .position(|(at, n)| {
            n.role == Role::Paragraph && f::node_text(view.tree(), at).as_ref() == b"Their addresses, one per line"
        })
        .expect("visible addresses caption");
    let caption = &view.tree().nodes()[caption_at];
    assert!(caption.styles.as_slice().contains(&look::CONNECT_TITLE));
    assert_eq!(caption_at + usize::try_from(caption.size).unwrap(), area_at, "the caption directly precedes the area");
    let terminal = f::page_view(&root);
    assert!(f::find(terminal.tree(), Role::TextArea, b"Their addresses, one per line").is_some());
    assert!(!terminal.tree().nodes().iter().enumerate().any(|(at, n)| n.role == Role::Paragraph
        && f::node_text(terminal.tree(), at).as_ref() == b"Their addresses, one per line"));
    assert_eq!(area.placeholder.as_deref(), Some(b"github.com/owner/name".as_slice()));
    let list_at = view.tree().nodes().iter().position(|n| n.styles.as_slice().contains(&look::CONNECT_ALERTS)).unwrap();
    let list = &view.tree().nodes()[list_at];
    let items: Box<[_]> = view.tree().nodes()[list_at + 1..list_at + usize::try_from(list.size).unwrap()]
        .iter()
        .filter(|n| n.role == Role::Item)
        .collect();
    assert_eq!(items.len(), 4);
    for (item, expected) in items.iter().zip([
        "Not an address: bad. Write it as github.com/owner/name.",
        "github isn’t a host. Did you mean github.com/acme/free? ",
        "acme/kept is already on this forge. ",
        "acme/copy is already on this forge. ",
    ]) {
        let at = view.tree().nodes().iter().position(|node| node.id == item.id).unwrap();
        let count = usize::try_from(item.size).unwrap();
        assert_eq!(
            view.tree().nodes()[at..at + count]
                .iter()
                .flat_map(|n| n.text.iter())
                .flat_map(|r| r.text.iter().copied())
                .collect::<Box<[_]>>()
                .as_ref(),
            expected.as_bytes()
        );
    }
    assert!(node(view.tree(), Role::Button, b"Copy 4 repositories").states.as_slice().contains(&State::Disabled));
    press_sheet(&mut root, &mut view, Role::Button, b"Use it");
    assert_eq!(
        root.forge().sheet().unwrap().addresses.text.as_ref(),
        b"bad\ngithub.com/acme/free\ngithub.com/acme/kept\ngithub.com/acme/copy"
    );
    let fix = node(view.tree(), Role::Button, b"Adopt it").id;
    assert_eq!(
        crate::act(&mut view, fix, UiAct::Press),
        Some(Action::Forge(forge::Act::Sheet(Act::AdoptIt {
            org: b"acme".as_slice().into(),
            name: b"kept".as_slice().into(),
        })))
    );
    press_sheet(&mut root, &mut view, Role::Button, b"Adopt it");
    assert_eq!(root.forge().sheet().unwrap().source, Source::Forge);
    assert_eq!(
        root.forge().sheet().unwrap().addresses.text.as_ref(),
        b"bad\ngithub.com/acme/free\ngithub.com/acme/copy"
    );
    let sheet::Reading::Asked { key, .. } = root.forge().sheet().unwrap().reading else { panic!("target read") };
    f::root_event(
        &mut root,
        temper_client_domain::Event::Forge(forge::Event::SheetRead {
            request: key,
            answer: Ok(wire::Page::new(
                &wire::CEILINGS,
                wire::PageParts {
                    rows: skein_lib::List::from_boxed(Box::new([reached(9, b"kept", false)])),
                    more: false,
                },
            )
            .unwrap()),
        }),
    );
    assert_eq!(root.forge().sheet().unwrap().picks.get(0).unwrap().id, 9);
}
#[test]
fn copy_buttons_count_lines_and_disable_every_invalid_line() {
    let mut root = root();
    sheet_act(&mut root, Act::Source(Source::Copy));
    let mut view = f::page_view(&root);
    for (addresses, label, disabled) in [
        (b"".as_slice(), b"Copy".as_slice(), true),
        (b"acme/one", b"Copy acme/one", false),
        (b"acme/one\nacme/two", b"Copy 2 repositories", false),
        (b"acme/one\nbad", b"Copy 2 repositories", true),
        (b"github/acme/one", b"Copy github/acme/one", true),
        (b"acme/kept", b"Copy acme/kept", true),
    ] {
        input_sheet(&mut root, &mut view, Role::TextArea, b"Their addresses, one per line", addresses);
        assert_eq!(node(view.tree(), Role::Button, label).states.as_slice().contains(&State::Disabled), disabled);
    }
    assert!(
        f::text(view.tree())
            .windows(b"Copied into Forgejo on this machine from GitHub".len())
            .any(|part| part == b"Copied into Forgejo on this machine from GitHub")
    );
    input_sheet(&mut root, &mut view, Role::TextArea, b"Their addresses, one per line", b"acme/one");
    input_sheet(&mut root, &mut view, Role::TextArea, b"Their addresses, one per line", &b"x".repeat(257));
    assert!(node(view.tree(), Role::Button, b"Copy acme/one").states.as_slice().contains(&State::Disabled));
}
fn forges(modes: &[(wire::Kind, wire::Mode)]) -> Domain {
    let mut rows = skein_lib::List::with_capacity(u32::try_from(modes.len()).unwrap());
    for (index, (kind, mode)) in modes.iter().enumerate() {
        rows.push(
            wire::ForgeRow::new(
                &wire::CEILINGS,
                wire::ForgeRowParts {
                    number: u16::try_from(index.checked_add(1).unwrap()).unwrap(),
                    kind: kind.clone(),
                    mode: mode.clone(),
                    host: b"forge.invalid".as_slice().into(),
                    connection: wire::Connection::Connected(
                        wire::Connected::new(
                            &wire::CEILINGS,
                            wire::ConnectedParts { id: 1, login: b"bot".as_slice().into() },
                        )
                        .unwrap(),
                    ),
                    starts_with: wire::StartsWith::Nothing,
                    makings: skein_lib::List::with_capacity(0),
                    repositories: skein_lib::List::with_capacity(0),
                    installations: skein_lib::List::with_capacity(0),
                    offer: wire::Offer::NotOffered,
                    first: None,
                    ended: None,
                    app: None,
                    web: None,
                    owned: None,
                    admin: false,
                    read_only: false,
                    ended_stop: None,
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    f::forge_root(Address::Adopt { project: f::PROJECT }, rows)
}
#[test]
fn only_several_eligible_forgejos_get_a_target_field() {
    let mut root = forges(&[
        (wire::Kind::Github, wire::Mode::Managed),
        (wire::Kind::Forgejo, wire::Mode::Owned),
        (wire::Kind::Forgejo, wire::Mode::Connected),
    ]);
    for source in [Source::Empty, Source::Copy] {
        sheet_act(&mut root, Act::Source(source));
        let view = f::page_view(&root);
        assert!(f::find(view.tree(), Role::Select, b"Forge").is_none());
    }
    let mut root = forges(&[
        (wire::Kind::Github, wire::Mode::Managed),
        (wire::Kind::Forgejo, wire::Mode::Owned),
        (wire::Kind::Forgejo, wire::Mode::Managed),
        (wire::Kind::Forgejo, wire::Mode::Connected),
    ]);
    for source in [Source::Empty, Source::Copy] {
        sheet_act(&mut root, Act::Source(source));
        let mut view = f::page_view(&root);
        assert!(f::find(view.tree(), Role::Select, b"Forge").is_some());
        let options: Box<[_]> = view.tree().nodes().iter().filter(|n| n.role == Role::Option).collect();
        assert_eq!(options.len(), 2);
        let target = options[1].id;
        let action = crate::act(&mut view, target, UiAct::Press).unwrap();
        assert_eq!(action, Action::Forge(forge::Act::Sheet(Act::Target(3))));
        f::root_event(&mut root, temper_client_domain::Event::Act { action });
        f::build(&mut view, &root);
        assert_eq!(root.forge().sheet().unwrap().target, Some(3));
        assert!(f::text(view.tree()).windows(b"Forgejo".len()).any(|part| part == b"Forgejo"));
        assert!(f::find(view.tree(), Role::Select, b"Account").is_none());
    }
}
#[test]
fn without_an_eligible_forgejo_the_sources_and_their_control_go() {
    let mut root = forges(&[
        (wire::Kind::Github, wire::Mode::Managed),
        (wire::Kind::Forgejo, wire::Mode::Connected),
        (wire::Kind::Forgejo, wire::Mode::Fake),
    ]);
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::ChoiceGroup, b"Where they come from").is_none());
    for name in [b"Make empty".as_slice(), b"Copy public"] {
        assert!(f::find(view.tree(), Role::Choice, name).is_none());
    }
    for source in [Source::Empty, Source::Copy] {
        sheet_act(&mut root, Act::Source(source));
        assert_eq!(root.forge().sheet().unwrap().source, Source::Forge);
    }
}
