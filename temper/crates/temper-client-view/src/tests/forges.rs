#![allow(clippy::disallowed_types, clippy::disallowed_methods, reason = "tests inspect semantic trees")]
use super::{connect_forge as cf, fixtures as f};
use crate::look;
use skein_lib::List;
use skein_ui::{Act, Role, State};
use temper_client_domain::ChatsFilter;
use temper_client_domain::{self as d, Address, Choice};
use temper_client_protocol_wire::{self as wire, v1 as w};
const HERE: u16 = 3;
fn root(address: Address, connection: w::Connection) -> d::Domain {
    f::forge_root(address, cf::rows(connection, w::Offer::Offered))
}
fn action(root: &d::Domain, role: Role, name: &[u8]) -> d::Action {
    let mut view = f::page_view(root);
    let node = f::find(view.tree(), role, name).expect("named action");
    crate::act(&mut view, node, Act::Press).expect("bound action")
}
fn go(address: Address) -> d::Action {
    d::Action::Pages(d::pages::Act::Go { address })
}
#[test]
fn the_forges_page_says_its_words_and_folds_each_row() {
    let (mut root, watch, ..) = f::root_at_with_forges(
        Address::Forges,
        &f::domain_limits(),
        b"Shop",
        None,
        None,
        Some(cf::rows(cf::connected(), w::Offer::Offered)),
    );
    for project in [true, false] {
        if !project {
            let mut me = root.pages().me(root.client()).unwrap().clone();
            me.projects.clear();
            f::deliver_me(&mut root, watch, me);
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            cf::words_in_order(
                view.tree(),
                &[
                    if project { "Where your code lives." } else { "Forges" },
                    "Each forge temper is connected to, or setting up. Add one to bring its repositories to temper.",
                    "Your forges · 2",
                    "+ Add a forge",
                ],
            );
            let row = view
                .tree()
                .nodes()
                .iter()
                .find(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Forgejo on this machine"))
                .unwrap();
            assert!(!row.states.as_slice().contains(&State::Expanded));
            assert_eq!(row.description.as_deref(), Some("Forge 3 · Forgejo · Connected as temper".as_bytes()));
            assert!(f::find(view.tree(), Role::Term, b"Account").is_some());
            assert!(f::find(view.tree(), Role::Description, "temper · user 42".as_bytes()).is_some());
        }
    }
}

#[test]
fn each_forgejo_mode_says_its_access_in_a_slot() {
    for (mode, words) in [
        (w::Mode::Owned, b"Site admin, with temper's own token".as_slice()),
        (w::Mode::Managed, b"Site admin, with a token you gave"),
        (w::Mode::Connected, b"Its bot's tokens, which you gave"),
    ] {
        let mut rows = List::with_capacity(1);
        rows.push(cf::row(HERE, w::Kind::Forgejo, mode, cf::connected(), w::Offer::NotOffered)).unwrap();
        let root = f::forge_root(Address::Forges, rows);
        let view = f::page_view(&root);
        assert!(view.tree().nodes().iter().enumerate().any(|(index, node)| {
            if node.role != Role::Slot {
                return false;
            }
            let slot = &view.tree().nodes()[index..index + usize::try_from(node.size).unwrap()];
            slot.iter().any(|n| n.role == Role::Term && n.name.as_deref() == Some(b"Access"))
                && slot.iter().any(|n| n.role == Role::Description && n.name.as_deref() == Some(words))
        }));
    }
}
#[test]
fn an_owned_forge_keeps_its_access_when_stopped() {
    let root = root(
        Address::Forges,
        w::Connection::Stopped(
            w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: b"token refused".as_slice().into() }).unwrap(),
        ),
    );
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Term, b"Access").is_some());
    assert!(f::find(view.tree(), Role::Description, b"Site admin, with temper's own token").is_some());
}
#[test]
fn the_playground_forge_has_no_access_line() {
    let mut rows = List::with_capacity(1);
    rows.push(cf::row(wire::FORGE, w::Kind::Forgejo, w::Mode::Fake, cf::connected(), w::Offer::NotOffered)).unwrap();
    let root = f::playground_forges_root(Address::Forges, rows);
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Heading { level: 2 }, b"The playground").is_some());
    assert!(f::find(view.tree(), Role::Term, b"Account").is_some());
    assert!(f::find(view.tree(), Role::Term, b"Access").is_none());
}
#[test]
fn a_forge_row_opened_offers_its_action() {
    let failed = w::Connection::SetUpFailed(
        w::SetUpFailed::new(&w::CEILINGS, w::SetUpFailedParts { why: b"fetch failed".as_slice().into(), at: None })
            .unwrap(),
    );
    let root = root(Address::Forges, failed);
    assert_eq!(action(&root, Role::Button, b"Try again"), d::Action::Forge(d::forge::Act::SetUp { forge: HERE }));
    let view = f::page_view(&root);
    assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::FLOW_PRIMARY)));
    let row = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Forgejo on this machine"))
        .unwrap();
    assert_eq!(row.description.as_deref(), Some("Forge 3 · Forgejo · Set up failed: fetch failed".as_bytes()));
    let root = self::root(Address::Forges, w::Connection::Connecting);
    let view = f::page_view(&root);
    assert!(view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::CONNECT_LIVE)));
    let root = self::root(Address::Forges, cf::connected());
    assert_eq!(action(&root, Role::Link, b"Adopt a repository"), go(Address::Adopt { project: f::PROJECT }));
    let root = self::root(
        Address::Forges,
        w::Connection::Stopped(
            w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: b"token refused".as_slice().into() }).unwrap(),
        ),
    );
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Paragraph, b"Restart temper box: its preparation mends it").is_some());
}
#[test]
fn no_forge_of_yours_yet_offers_connect_a_forge() {
    let root = f::forge_root(Address::Forges, List::with_capacity(0));
    let view = f::page_view(&root);
    assert!(String::from_utf8_lossy(&f::text(view.tree())).contains("No forge of yours yet."));
    for name in [b"+ Add a forge".as_slice(), b"Connect a forge"] {
        assert_eq!(action(&root, Role::Link, name), go(Address::ConnectForge { forge: None }));
    }
}
#[test]
fn the_navigation_has_forges_with_its_dot_and_words() {
    for (connection, style, text) in [
        (w::Connection::Connecting, look::NAV_LIMITED, b"a forge is setting up".as_slice()),
        (
            w::Connection::Stopped(w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: Box::default() }).unwrap()),
            look::NAV_NEEDS,
            b"a forge needs you",
        ),
    ] {
        for address in [Address::Forges, Address::ConnectForge { forge: None }] {
            let root = root(address, connection.clone());
            let view = f::page_view(&root);
            let nav = view
                .tree()
                .nodes()
                .iter()
                .find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Settings"))
                .unwrap();
            assert!(nav.states.as_slice().contains(&State::Current));
            let dot = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&style)).unwrap();
            assert_eq!(dot.role, Role::Text);
            assert_eq!(dot.text.get(0).unwrap().text.as_ref(), b" \xc2\xb7 ");
            assert_eq!(dot.text.get(1).unwrap().text.as_ref(), text);
        }
    }
}

#[test]
fn the_playground_forge_shows_while_the_forge_half_holds() {
    let mut rows = List::with_capacity(2);
    rows.push(cf::row(wire::FORGE, w::Kind::Forgejo, w::Mode::Fake, cf::connected(), w::Offer::NotOffered)).unwrap();
    rows.push(cf::row(HERE, w::Kind::Forgejo, w::Mode::Owned, w::Connection::Connecting, w::Offer::Offered)).unwrap();
    let mut root = f::playground_forges_root(Address::Forges, rows);
    assert!(d::forge::listed::playground(&root).is_some());
    let view = f::page_view(&root);
    let row = f::find(view.tree(), Role::Disclosure, b"Playground forge").unwrap();
    assert!(!view.tree().nodes().iter().find(|n| n.id == row).unwrap().states.as_slice().contains(&State::Expanded));
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Chats { filter: ChatsFilter::All }) });
    assert!(d::forge::listed::playground(&root).is_some());
    let root = f::playground_forges_root(Address::Forges, cf::rows(cf::connected(), w::Offer::Offered));
    assert!(d::forge::listed::playground(&root).is_none());
    assert!(f::find(f::page_view(&root).tree(), Role::Heading { level: 2 }, b"The playground").is_none());
}

#[test]
fn set_up_again_waits_for_an_offer() {
    let failed = w::Connection::SetUpFailed(
        w::SetUpFailed::new(&w::CEILINGS, w::SetUpFailedParts { why: b"fetch failed".as_slice().into(), at: None })
            .unwrap(),
    );
    let root = f::forge_root(Address::Forges, cf::rows(failed, w::Offer::NotOffered));
    assert!(f::find(f::page_view(&root).tree(), Role::Button, b"Try again").is_none());
}

#[test]
fn a_full_forge_reason_and_login_fit_after_utf8_replacement() {
    for connection in [
        w::Connection::Stopped(
            w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: Box::from([255_u8; 256]) }).unwrap(),
        ),
        w::Connection::Connected(
            w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: u64::MAX, login: Box::from([255_u8; 256]) })
                .unwrap(),
        ),
    ] {
        let root = root(Address::Forges, connection);
        let domain = f::domain_limits();
        let words = f::words();
        let limits = crate::Limits::of(&domain, &words).unwrap();
        let mut view = crate::View::new(&limits, &domain, words, wire::FORGE);
        crate::build(&mut view, &root, f::NOW, &mut skein_lib::Queue::with_capacity(limits.tree.patches));
        let row = view
            .tree()
            .nodes()
            .iter()
            .find(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Forgejo on this machine"))
            .unwrap();
        let summary = core::str::from_utf8(row.description.as_deref().unwrap()).unwrap();
        assert_eq!(summary.matches('�').count(), 256_usize);
    }
}

#[test]
fn the_github_row_draws_its_app_installations_and_last_connect() {
    let root = f::forge_root(Address::Forges, cf::github_rows(true));
    let view = f::browser_page_view(&root);
    assert!(f::find(view.tree(), Role::Term, b"Access").is_none());
    cf::words_in_order(
        view.tree(),
        &[
            "Account",
            "my-app[bot] · user 9",
            "temper’s App",
            "my-app · ",
            "acme",
            ", installation 2 · suspended",
            ", installation 3 · not accepted · kept",
            "The last connect stopped: fixture stopped",
            "Install on another account",
        ],
    );
}
#[test]
fn install_on_another_account_follows_an_accepted_installation() {
    for (accepted, name) in [(true, b"Install on another account".as_slice()), (false, b"Install on GitHub")] {
        let root = f::forge_root(Address::Forges, cf::github_rows(accepted));
        let view = f::page_view(&root);
        let install = view
            .tree()
            .nodes()
            .iter()
            .find(|node| node.role == Role::Button && node.name.as_deref() == Some(name))
            .unwrap();
        let style = if accepted { look::FLOW_SECONDARY } else { look::FLOW_PRIMARY };
        assert!(install.styles.as_slice().contains(&style));
        assert_eq!(
            action(&root, Role::Button, name),
            d::Action::Forge(d::forge::Act::Install { forge: 2, back: b"/forges".as_slice().into() })
        );
    }
}
#[test]
fn all_sixty_four_installations_fit_without_shortening_their_logins() {
    let mut rows = cf::github_rows(true);
    let mut row = rows.as_slice()[0].clone().into_parts();
    rows.clear();
    row.installations = List::with_capacity(d::forge::forges::INSTALLATIONS);
    for id in 1..=u64::from(d::forge::forges::INSTALLATIONS) {
        row.installations
            .push(
                w::Installation::new(
                    &w::CEILINGS,
                    w::InstallationParts {
                        id,
                        account: Some(
                            w::Account::new(&w::CEILINGS, w::AccountParts { id, login: Box::from([255_u8; 256]) })
                                .unwrap(),
                        ),
                        suspended: true,
                        stray: true,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    rows.push(w::ForgeRow::new(&w::CEILINGS, row).unwrap()).unwrap();
    let root = f::forge_root(Address::Forges, rows);
    let domain = f::limits();
    let words = f::browser_words();
    let limits = crate::Limits::of(&domain, &words).unwrap();
    let mut view = crate::View::new(&limits, &domain, words, wire::FORGE);
    crate::build(&mut view, &root, f::NOW, &mut skein_lib::Queue::with_capacity(limits.tree.patches));
    let text = f::text(view.tree());
    let text = core::str::from_utf8(&text).unwrap();
    assert_eq!(text.matches('�').count(), 64 * 256);
}

#[test]
fn a_connected_forge_in_the_page_facts_opens_through_temper() {
    let root = root(Address::Forges, cf::connected());
    let mut view = f::view_of(f::browser_words());
    let mut facts = crate::forge_sign_in::Forges::new();
    facts.read("3 forgejo https://forge.example:3000 owned");
    view.forge_sign_in(facts);
    f::build(&mut view, &root);
    let link = f::find(view.tree(), Role::Link, "Open Forgejo ↗".as_bytes()).expect("forge sign-in link");
    assert_eq!(
        crate::href(&mut view, link).as_deref(),
        Some(b"https://forge.example:3000/user/oauth2/temper".as_slice())
    );
    assert!(
        core::str::from_utf8(&f::main_text(view.tree()))
            .unwrap()
            .contains("Open Forgejo signs you in as pat, through temper: Forgejo’s sign-in is temper’s.")
    );
    let view = f::browser_page_view(&root);
    assert!(f::find(view.tree(), Role::Link, "Open Forgejo ↗".as_bytes()).is_none());
}

#[test]
fn an_owned_forge_opens_at_the_public_origin_in_the_page_facts() {
    let root = root(Address::Forges, cf::connected());
    let mut view = f::view_of(f::browser_words());
    let mut facts = crate::forge_sign_in::Forges::new();
    facts.read("3 forgejo https://git.example.com owned");
    assert_eq!(facts.number(3).unwrap().name(&f::browser_words()).as_ref(), b"Forgejo on this machine");
    view.forge_sign_in(facts);
    f::build(&mut view, &root);
    let link = f::find(view.tree(), Role::Link, "Open Forgejo ↗".as_bytes()).expect("forge sign-in link");
    assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"https://git.example.com/user/oauth2/temper".as_slice()));
}

fn owned_row(mode: w::Mode, connection: w::Connection, runner: Option<&[u8]>) -> w::ForgeRow {
    let mut parts = cf::row(HERE, w::Kind::Forgejo, mode, connection, w::Offer::Offered).into_parts();
    if parts.mode == w::Mode::Owned {
        parts.owned = Some(
            w::OwnedForge::new(
                &w::CEILINGS,
                w::OwnedForgeParts {
                    forgejo: b"17.0.0".as_slice().into(),
                    runner: runner.map(Into::into),
                    password: b"/fake/forge/person_password".as_slice().into(),
                },
            )
            .unwrap(),
        );
    }
    w::ForgeRow::new(&w::CEILINGS, parts).unwrap()
}
fn slot_has(tree: &skein_ui::Tree, label: &[u8], words: &[u8]) -> bool {
    tree.nodes().iter().enumerate().any(|(index, node)| {
        if node.role != Role::Slot {
            return false;
        }
        let slot = &tree.nodes()[index..index.checked_add(usize::try_from(node.size).unwrap()).unwrap()];
        slot.iter().any(|n| n.role == Role::Term && n.name.as_deref() == Some(label))
            && slot.iter().any(|n| n.role == Role::Description && n.name.as_deref() == Some(words))
    })
}
#[test]
fn owned_ready_rows_say_the_versions_and_first_password_path_in_slots() {
    for (runner, runs) in [
        (
            Some(b"13.2.0".as_slice()),
            b"Forgejo 17.0.0 and its runner 13.2.0, on this machine. CI jobs run here, as you.".as_slice(),
        ),
        (None, b"Forgejo 17.0.0, on this machine, with no runner.".as_slice()),
    ] {
        for connection in [
            cf::connected(),
            w::Connection::Connecting,
            w::Connection::Stopped(
                w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: b"stopped".as_slice().into() }).unwrap(),
            ),
        ] {
            let mut rows = List::with_capacity(1);
            rows.push(owned_row(w::Mode::Owned, connection, runner)).unwrap();
            let root = f::forge_root(Address::Forges, rows);
            let view = f::page_view(&root);
            assert!(slot_has(view.tree(), b"Runs", runs));
            assert!(slot_has(
                view.tree(),
                b"First password",
                b"Your first password is in /fake/forge/person_password. Change it on Forgejo when you like."
            ));
        }
    }
}
#[test]
fn unprepared_owned_rows_and_managed_rows_have_no_runs_or_password_lines() {
    for (mode, connection) in [
        (w::Mode::Owned, w::Connection::NotConnected),
        (w::Mode::Owned, w::Connection::NotSetUp),
        (
            w::Mode::Owned,
            w::Connection::SettingUp(
                w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: None, fetched: None }).unwrap(),
            ),
        ),
        (
            w::Mode::Owned,
            w::Connection::SetUpFailed(
                w::SetUpFailed::new(&w::CEILINGS, w::SetUpFailedParts { why: b"failed".as_slice().into(), at: None })
                    .unwrap(),
            ),
        ),
        (w::Mode::Managed, cf::connected()),
    ] {
        let mut rows = List::with_capacity(1);
        rows.push(owned_row(mode, connection, Some(b"13.2.0"))).unwrap();
        let root = f::forge_root(Address::Forges, rows);
        let view = f::page_view(&root);
        for label in [b"Runs".as_slice(), b"First password"] {
            assert!(f::find(view.tree(), Role::Term, label).is_none());
        }
        assert!(
            f::find(
                view.tree(),
                Role::Description,
                b"Your first password is in /fake/forge/person_password. Change it on Forgejo when you like."
            )
            .is_none()
        );
    }
}

#[test]
fn the_github_app_line_names_an_empty_installation_set_only_when_empty() {
    for empty in [false, true] {
        let mut parts = cf::github_rows(true).get(0).unwrap().clone().into_parts();
        if empty {
            parts.installations.clear();
        }
        let mut rows = List::with_capacity(1);
        rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
        let root = f::forge_root(Address::Forges, rows);
        let view = f::browser_page_view(&root);
        assert!(f::find(view.tree(), Role::Term, "temper’s App".as_bytes()).is_some());
        assert_eq!(f::find(view.tree(), Role::Slot, "my-app · installed nowhere yet".as_bytes()).is_some(), empty);
        assert_eq!(f::find(view.tree(), Role::Slot, b"acme").is_some(), !empty);
    }
}

#[test]
fn the_github_rows_install_block_follows_the_apps_way_and_acceptance() {
    for app in [None, Some(w::AppWay::Manifest), Some(w::AppWay::Pasted)] {
        for accepted in [false, true] {
            let mut parts = cf::github_rows(accepted).get(0).unwrap().clone().into_parts();
            parts.app = app.clone();
            let mut rows = List::with_capacity(1);
            rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
            let root = f::forge_root(Address::Forges, rows);
            let view = f::browser_page_view(&root);
            let manifest = "my-app is made on GitHub, under your account, and installed nowhere yet. Install it, and choose the repositories temper may use.";
            let pasted = "my-app is connected to temper, and installed nowhere yet. Install it, and choose the repositories temper may use.";
            let explanation = if app == Some(w::AppWay::Manifest) { manifest } else { pasted };
            assert_eq!(
                f::find(view.tree(), Role::Heading { level: 2 }, "Install temper’s App".as_bytes()).is_some(),
                !accepted
            );
            for words in [manifest, pasted] {
                assert_eq!(
                    f::find(view.tree(), Role::Paragraph, words.as_bytes()).is_some(),
                    !accepted && words == explanation
                );
            }
            if !accepted {
                cf::words_in_order(
                    view.tree(),
                    &[
                        "temper’s App",
                        "The last connect stopped: fixture stopped",
                        "Install temper’s App",
                        explanation,
                        "Install on GitHub",
                    ],
                );
            }
        }
    }
}

#[test]
fn the_github_row_opens_its_configured_web_address_only_when_carried() {
    for kind in [w::Kind::Github, w::Kind::Forgejo] {
        for web in [None, Some(b"https://github.test/base/?a=1&b=2".as_slice())] {
            let mut parts = cf::github_rows(true).get(0).unwrap().clone().into_parts();
            parts.kind = kind.clone();
            parts.web = web.map(Into::into);
            let mut rows = List::with_capacity(1);
            rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
            let root = f::forge_root(Address::Forges, rows);
            let view = f::browser_page_view(&root);
            let link = f::find(view.tree(), Role::Link, "Open on GitHub ↗".as_bytes());
            assert_eq!(link.is_some(), kind == w::Kind::Github && web.is_some());
            if let Some(link) = link {
                let node = view.tree().nodes().iter().find(|node| node.id == link).unwrap();
                assert_eq!(node.external.as_deref(), web);
            }
        }
    }
}

#[test]
fn a_stopped_github_row_has_no_app_or_install_block() {
    let mut parts = cf::github_rows(false).get(0).unwrap().clone().into_parts();
    parts.connection = w::Connection::Stopped(
        w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: b"fixture stopped".as_slice().into() }).unwrap(),
    );
    parts.installations.clear();
    let mut rows = List::with_capacity(1);
    rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
    let root = f::forge_root(Address::Forges, rows);
    let view = f::browser_page_view(&root);
    assert!(f::find(view.tree(), Role::Term, "temper’s App".as_bytes()).is_none());
    assert!(f::find(view.tree(), Role::Slot, "my-app · installed nowhere yet".as_bytes()).is_none());
    assert!(f::find(view.tree(), Role::Heading { level: 2 }, "Install temper’s App".as_bytes()).is_none());
    assert!(f::find(view.tree(), Role::Button, b"Install on GitHub").is_none());
    assert!(f::find(view.tree(), Role::Link, b"Connect an App of your own").is_some());
}

#[test]
fn the_owned_repository_body_names_its_project_without_a_numeric_made_line() {
    let root = f::forge_root(Address::Forges, cf::first_rows(cf::connected(), cf::made()));
    let view = f::page_view(&root);
    assert!(slot_has(view.tree(), b"Repositories", "work/hello".as_bytes()));
    let row = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Forgejo on this machine"))
        .unwrap();
    assert!(row.description.as_deref().unwrap().ends_with("work/hello".as_bytes()));
    assert!(!String::from_utf8_lossy(&f::text(view.tree())).contains("made, in 1"));
}
#[test]
fn a_repository_making_or_failed_keeps_its_existing_body() {
    for (state, message) in [
        (w::FirstState::Making, "Making work/hello"),
        (
            w::FirstState::Failed(
                w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step: w::CreateStep::Repository }).unwrap(),
            ),
            "Making work/hello failed: making the repository",
        ),
    ] {
        let root = f::forge_root(Address::Forges, cf::first_rows(cf::connected(), state));
        let view = f::page_view(&root);
        assert!(f::find(view.tree(), Role::Paragraph, message.as_bytes()).is_some());
        assert!(f::find(view.tree(), Role::Term, b"Repositories").is_none());
    }
}

#[test]
fn forges_lists_the_playground_row_with_nothing_to_add() {
    let mut rows = List::with_capacity(1);
    rows.push(cf::row(wire::FORGE, w::Kind::Forgejo, w::Mode::Fake, cf::connected(), w::Offer::NotOffered)).unwrap();
    let root = f::playground_forges_root(Address::Forges, rows);
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Heading { level: 2 }, b"The playground").is_some());
    for name in [b"+ Add a forge".as_slice(), b"Connect a forge"] {
        assert!(f::find(view.tree(), Role::Link, name).is_none());
    }
    assert!(f::find(view.tree(), Role::Button, b"Remove").is_none());
    assert_eq!(
        view.tree()
            .nodes()
            .iter()
            .filter(|n| n.role == Role::Disclosure && n.styles.as_slice().contains(&look::SETTINGS_ROW))
            .count(),
        1
    );
}
#[test]
fn remove_confirms_with_the_kinds_words_then_sends() {
    for (kind, mode, words) in [
        (w::Kind::Forgejo, w::Mode::Owned, "Its Forgejo, its directory and its repositories' work here go."),
        (w::Kind::Forgejo, w::Mode::Managed, "Its bot and hooks stay on your Forgejo."),
        (w::Kind::Github, w::Mode::Connected, "The GitHub App stays yours: delete it in its settings."),
    ] {
        let mut rows = List::with_capacity(1);
        let mut row = cf::row(HERE, kind.clone(), mode, cf::connected(), w::Offer::NotOffered).into_parts();
        row.web = Some(b"https://forge.example/".as_slice().into());
        rows.push(w::ForgeRow::new(&w::CEILINGS, row).unwrap()).unwrap();
        let mut root = f::forge_root(Address::Forges, rows);
        let first = action(&root, Role::Button, b"Remove");
        assert_eq!(first, d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::Remove { forge: HERE })));
        f::root_event(&mut root, d::Event::Act { action: first });
        let mut view = f::page_view(&root);
        cf::words_in_order(view.tree(), &["Remove this forge?", words, "Remove"]);
        if kind == w::Kind::Github {
            let link = f::find(view.tree(), Role::Link, b"the App's settings").unwrap();
            assert_eq!(
                crate::href(&mut view, link).as_deref(),
                Some(b"https://forge.example/settings/apps".as_slice())
            );
        }
        let confirmed = action(&root, Role::Button, b"Remove");
        assert_eq!(confirmed, d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::ConfirmRemove)));
        f::root_event(&mut root, d::Event::Act { action: confirmed });
        assert!(root.forge().setup_form().removal.as_ref().unwrap().request.is_some());
    }
}

#[test]
fn a_disconnected_connected_mode_forgejo_links_to_its_tokens_form() {
    for (connection, offered) in [
        (w::Connection::NotConnected, true),
        (
            w::Connection::Stopped(
                w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: b"stopped".as_slice().into() }).unwrap(),
            ),
            true,
        ),
        (w::Connection::Connecting, false),
        (cf::connected(), false),
    ] {
        let mut rows = List::with_capacity(1);
        rows.push(cf::row(HERE, w::Kind::Forgejo, w::Mode::Connected, connection, w::Offer::NotOffered)).unwrap();
        let root = f::forge_root(Address::Forges, rows);
        let view = f::page_view(&root);
        assert_eq!(f::find(view.tree(), Role::Link, "Connect ›".as_bytes()).is_some(), offered);
        if offered {
            assert_eq!(
                action(&root, Role::Link, "Connect ›".as_bytes()),
                go(Address::ConnectForge { forge: Some(Choice::Forge(HERE)) })
            );
        }
    }
}

fn replace_root(kind: w::Kind, mode: w::Mode, connection: w::Connection) -> d::Domain {
    let mut rows = List::with_capacity(1);
    rows.push(cf::row(HERE, kind, mode, connection, w::Offer::NotOffered)).unwrap();
    f::forge_root(Address::Forges, rows)
}
fn replace_act(root: &mut d::Domain, act: d::forge::forges::Act) {
    f::root_event(root, d::Event::Forge(d::forge::Event::Act { act: d::forge::Act::Forges(act) }));
}
fn stopped() -> w::Connection {
    w::Connection::Stopped(
        w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: b"stopped".as_slice().into() }).unwrap(),
    )
}
#[test]
fn replace_is_offered_only_on_connected_or_stopped_supported_forges() {
    for (kind, mode, connection, offered) in [
        (w::Kind::Github, w::Mode::Connected, cf::connected(), true),
        (w::Kind::Github, w::Mode::Connected, stopped(), true),
        (w::Kind::Forgejo, w::Mode::Connected, cf::connected(), true),
        (w::Kind::Forgejo, w::Mode::Connected, stopped(), true),
        (w::Kind::Forgejo, w::Mode::Managed, cf::connected(), false),
        (w::Kind::Forgejo, w::Mode::Owned, cf::connected(), false),
        (w::Kind::Forgejo, w::Mode::Connected, w::Connection::NotConnected, false),
        (w::Kind::Github, w::Mode::Connected, w::Connection::NotConnected, false),
    ] {
        let root = replace_root(kind, mode, connection);
        assert_eq!(f::find(f::page_view(&root).tree(), Role::Button, b"Replace").is_some(), offered);
    }
}
#[test]
fn replacement_fields_are_secret_and_local_refusals_keep_the_form() {
    use d::forge::{forges::Act as A, replace::Field};
    for (kind, names, button, refusal) in [
        (w::Kind::Github, &[b"Private key".as_slice()][..], b"Replace the key".as_slice(), b"Required.".as_slice()),
        (
            w::Kind::Forgejo,
            &[b"API token".as_slice(), b"Git token", b"Webhook secret"][..],
            b"Replace the tokens",
            b"Paste a new API token, git token or both.",
        ),
    ] {
        let mut root = replace_root(kind.clone(), w::Mode::Connected, cf::connected());
        replace_act(&mut root, A::OpenReplace { forge: HERE });
        replace_act(&mut root, A::Replace { forge: HERE });
        let view = f::browser_page_view(&root);
        assert!(f::find(view.tree(), Role::Alert, refusal).is_some());
        assert!(f::find(view.tree(), Role::Button, button).is_some());
        let mut rng = skein_lib::Rng::new(f::SEED);
        let value: Box<[u8]> =
            (0_u32..32).map(|_| b'a'.checked_add(u8::try_from(rng.next_u64() % 26).unwrap()).unwrap()).collect();
        for name in names {
            let id = f::find(view.tree(), Role::TextField, name).unwrap();
            let field = view.tree().nodes().iter().find(|node| node.id == id).unwrap();
            assert_eq!(field.entry, Some(skein_ui::EntryKind::Secret));
            let event = crate::event(&view, id, Act::Input { value: value.clone() }).expect("secret event");
            f::root_event(&mut root, event);
        }
        let view = f::browser_page_view(&root);
        assert!(skein_lib::bytes::find(&f::text(view.tree()), &value).is_none());
        for name in names {
            let id = f::find(view.tree(), Role::TextField, name).unwrap();
            assert!(
                view.tree().nodes().iter().find(|node| node.id == id).unwrap().value.as_ref().unwrap().text.is_empty()
            );
        }
        let field = if kind == w::Kind::Github { Field::Key } else { Field::Git };
        f::root_event(
            &mut root,
            d::Event::Forge(d::forge::Event::ReplaceValue {
                forge: HERE,
                field,
                value: skein_lib::Secret::new(skein_lib::bytes::zeroed(d::forge::forges::PASTED_PAYLOAD + 1)),
            }),
        );
        replace_act(&mut root, A::Replace { forge: HERE });
        assert!(
            f::find(
                f::browser_page_view(&root).tree(),
                Role::Alert,
                b"The pasted values are past 4 KiB, the most a request carries."
            )
            .is_some()
        );
    }
}
#[test]
fn terminal_replacement_names_the_key_file_command() {
    let mut root = replace_root(w::Kind::Github, w::Mode::Connected, cf::connected());
    replace_act(&mut root, d::forge::forges::Act::OpenReplace { forge: HERE });
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::TextField, b"Private key").is_none());
    assert!(f::find(view.tree(), Role::Paragraph, b"Paste the key on this page in a browser, or run temper ctl forge replace 3 FILE, FILE the key's file, readable by you alone.").is_some());
}

fn replacement_answer(outcome: w::Outcome) -> d::Domain {
    let mut root = replace_root(w::Kind::Forgejo, w::Mode::Connected, cf::connected());
    replace_act(&mut root, d::forge::forges::Act::OpenReplace { forge: HERE });
    let mut rng = skein_lib::Rng::new(f::SEED);
    let bytes: Box<[u8]> =
        (0_u32..32).map(|_| b'a'.checked_add(u8::try_from(rng.next_u64() % 26).unwrap()).unwrap()).collect();
    f::root_event(
        &mut root,
        d::Event::Forge(d::forge::Event::ReplaceValue {
            forge: HERE,
            field: d::forge::replace::Field::Git,
            value: skein_lib::Secret::new(bytes),
        }),
    );
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        d::Event::Act {
            action: d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::Replace { forge: HERE })),
        },
        &mut out,
    );
    let mut attempt = None;
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt: token, .. } },
        } = request
        {
            attempt = Some(token);
        }
    }
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: attempt.unwrap(),
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
                    })),
                },
            },
        },
    );
    root
}
#[test]
fn a_replacement_refusal_is_under_its_button_in_forge_words() {
    let root = replacement_answer(w::Outcome::ConnectRefused(
        w::ConnectRefused::new(&w::CEILINGS, w::ConnectRefusedParts { why: w::ForgeRefusal::NotConnected }).unwrap(),
    ));
    cf::words_in_order(f::browser_page_view(&root).tree(), &["Replace the tokens", "Connect it first."]);
}
#[test]
fn a_connecting_answer_updates_the_row_before_the_watch() {
    let root = replacement_answer(w::Outcome::Connecting);
    let view = f::browser_page_view(&root);
    assert!(view.tree().nodes().iter().any(|node| node.role == Role::Disclosure
        && node.description.as_ref().is_some_and(|text| skein_lib::bytes::find(text, b"Connecting").is_some())));
}

fn styled(tree: &skein_ui::Tree, style: skein_ui::Style) -> &[skein_ui::Node] {
    let (index, node) = tree.nodes().iter().enumerate().find(|(_, n)| n.styles.as_slice().contains(&style)).unwrap();
    &tree.nodes()[index..index.checked_add(usize::try_from(node.size).unwrap()).unwrap()]
}

#[test]
fn the_tab_draws_its_section_head_and_its_act_in_it() {
    let root = root(Address::Forges, cf::connected());
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let section = styled(view.tree(), look::FORGES);
        assert!(section[0].styles.as_slice().contains(&look::SETTINGS_SECTION));
        let head = styled(view.tree(), look::SETTINGS_SECTION_HEAD);
        assert_eq!(head[0].role, Role::Slot);
        assert!(head.iter().any(|n| n.role == Role::Heading { level: 2 }
            && n.name.as_deref() == Some("Your forges · 2".as_bytes())
            && n.styles.as_slice().contains(&look::SETTINGS_SECTION_TITLE)));
        let act = head.iter().find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"+ Add a forge")).unwrap();
        assert!(act.styles.as_slice().contains(&look::SETTINGS_BTN));
        assert!(act.styles.as_slice().contains(&look::SETTINGS_SMALL));
    }
    assert_eq!(action(&root, Role::Link, b"+ Add a forge"), go(Address::ConnectForge { forge: None }));
    let root = f::playground_forges_root(Address::Forges, List::with_capacity(0));
    let view = f::page_view(&root);
    assert!(!styled(view.tree(), look::SETTINGS_SECTION_HEAD).iter().any(|n| n.role == Role::Link));
}

#[test]
fn no_forge_is_a_quiet_band_and_an_unread_watch_claims_nothing() {
    let root = f::forge_root(Address::Forges, List::with_capacity(0));
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let band = styled(view.tree(), look::SETTINGS_QUIET_BAND);
        assert!(band[0].styles.as_slice().contains(&look::SETTINGS_BAND));
        assert!(
            band.iter().any(
                |n| n.styles.as_slice().contains(&look::SETTINGS_TAG) && n.name.as_deref() == Some(b"No forge yet")
            )
        );
        assert!(String::from_utf8_lossy(&f::text(view.tree())).contains("No forge of yours yet."));
        let act = band.iter().find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Connect a forge")).unwrap();
        assert!(act.styles.as_slice().contains(&look::SETTINGS_PRIMARY));
        assert!(!act.styles.as_slice().contains(&look::SETTINGS_SMALL));
    }
    assert_eq!(action(&root, Role::Link, b"Connect a forge"), go(Address::ConnectForge { forge: None }));
    let limits = f::domain_limits();
    let mut unknown = d::unlinked(&limits, f::SEED);
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits };
    let mut out = skein_lib::Queue::uncapped();
    for event in [
        d::Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } },
        d::Event::Start {
            address: Address::Forges,
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
        },
    ] {
        d::step(&mut unknown, &env, event, &mut out);
    }
    let mut me_watch = None;
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Open {
                            watch,
                            subject: jig_client::Watchable::Jig(jig_client::contract::Subject::Me),
                        },
                },
        } = request
        {
            me_watch = Some(watch);
        }
    }
    let template = f::forge_root(Address::Forges, List::with_capacity(0));
    d::step(
        &mut unknown,
        &env,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch.unwrap(),
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(
                        template.pages().me(template.client()).unwrap().clone(),
                    ))),
                },
            },
        },
        &mut out,
    );
    let mut forge_watch = false;
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { subject, .. } },
        } = request
        {
            forge_watch |= Some(subject) == d::forge::forges::watch(&limits.forge);
        }
    }
    assert!(forge_watch, "the forges watch is open and unanswered");
    let view = f::page_view(&unknown);
    assert!(f::find(view.tree(), Role::Heading { level: 2 }, b"Your forges").is_some());
    assert!(
        !view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_BAND)
            || n.styles.as_slice().contains(&look::SETTINGS_LINES))
    );
    assert!(f::find(view.tree(), Role::Link, b"+ Add a forge").is_none());
}

#[test]
fn each_forge_is_a_folded_row_with_its_facts_as_settings() {
    for rows in [
        cf::rows(cf::connected(), w::Offer::Offered),
        cf::github_rows(true),
        cf::first_rows(cf::connected(), cf::made()),
    ] {
        let root = f::forge_root(Address::Forges, rows);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let lines = styled(view.tree(), look::SETTINGS_LINES);
            let mut count = 0_usize;
            for (index, row) in lines.iter().enumerate().filter(|(_, n)| n.role == Role::Disclosure) {
                count += 1;
                assert!(row.styles.as_slice().contains(&look::SETTINGS_ROW));
                assert!(!row.states.as_slice().contains(&State::Expanded));
                let body = &lines[index..index.checked_add(usize::try_from(row.size).unwrap()).unwrap()];
                for (at, fact) in
                    body.iter().enumerate().filter(|(_, n)| n.styles.as_slice().contains(&look::SETTINGS_SETTING))
                {
                    assert_eq!(fact.role, Role::Slot);
                    let children = &body[at + 1..at.checked_add(usize::try_from(fact.size).unwrap()).unwrap()];
                    assert_eq!(children[0].role, Role::Term);
                    assert_eq!(children[usize::try_from(children[0].size).unwrap()].role, Role::Description);
                }
                for (at, _) in body.iter().enumerate().filter(|(_, n)| n.role == Role::Term) {
                    assert!(body[at - 1].styles.as_slice().contains(&look::SETTINGS_SETTING));
                }
            }
            assert!(count > 0);
        }
    }
}

fn assert_no_old_chrome(root: &d::Domain) {
    for view in [f::page_view(root), f::browser_page_view(root)] {
        for node in view.tree().nodes() {
            for style in [look::CARD, look::PAGEBAR, look::HEADING, look::SUBTLE_COUNT] {
                assert!(!node.styles.as_slice().contains(&style), "old chrome {style:?} on {:?}", node.name);
            }
        }
    }
}

#[test]
fn the_old_chrome_is_gone() {
    for connection in [
        w::Connection::NotConnected,
        w::Connection::NotSetUp,
        w::Connection::SettingUp(
            w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: Some(w::SetUpStep::Prepare), fetched: None })
                .unwrap(),
        ),
        w::Connection::SetUpFailed(
            w::SetUpFailed::new(&w::CEILINGS, w::SetUpFailedParts { why: b"fetch failed".as_slice().into(), at: None })
                .unwrap(),
        ),
        stopped(),
        w::Connection::Connecting,
        cf::connected(),
    ] {
        let mut root = root(Address::Forges, connection);
        assert_no_old_chrome(&root);
        replace_act(&mut root, d::forge::forges::Act::OpenReplace { forge: HERE });
        assert_no_old_chrome(&root);
        replace_act(&mut root, d::forge::forges::Act::Remove { forge: HERE });
        assert_no_old_chrome(&root);
    }
    for rows in [
        cf::github_rows(true),
        cf::github_rows(false),
        cf::first_rows(cf::connected(), cf::made()),
        cf::first_rows(cf::connected(), w::FirstState::Making),
        cf::first_rows(
            cf::connected(),
            w::FirstState::Failed(
                w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step: w::CreateStep::Repository }).unwrap(),
            ),
        ),
    ] {
        assert_no_old_chrome(&f::forge_root(Address::Forges, rows));
    }
    assert_no_old_chrome(&f::playground_forges_root(Address::Forges, List::with_capacity(0)));
}

#[test]
fn only_the_forges_row_named_by_the_off_fact_says_its_password_is_needed_in_the_browser() {
    let root = root(Address::Forges, cf::connected());
    let line = "This Forgejo asks for its password: temper couldn't connect its sign-in. box.log says why.";
    for (browser, number, expected) in [(true, "3", 1), (true, "2", 0), (false, "3", 0)] {
        let mut view = f::view_of(if browser { f::browser_words() } else { f::words() });
        let mut facts = crate::forge_sign_in::Forges::new();
        facts.read_off(number);
        facts.read_off(number);
        view.forge_sign_in(facts);
        f::build(&mut view, &root);
        let text = f::main_text(view.tree());
        assert_eq!(core::str::from_utf8(&text).unwrap().matches(line).count(), expected);
    }
}

#[test]
fn the_source_stage_says_it_is_connecting_sign_in_after_forgejo_has_started() {
    let connection = w::Connection::SettingUp(
        w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: Some(w::SetUpStep::Source), fetched: None }).unwrap(),
    );
    let root = root(Address::ConnectForge { forge: Some(Choice::Forge(HERE)) }, connection);
    let view = f::browser_page_view(&root);
    let text = f::main_text(view.tree());
    let text = core::str::from_utf8(&text).unwrap();
    assert!(text.contains("Connecting sign-in through temper"));
    assert!(!text.contains("Starting Forgejo and its runner"));
}

#[test]
fn a_settings_connection_lives_in_its_row_and_confirms_only_this_visit() {
    let mut parts = cf::first_rows(cf::connected(), w::FirstState::Making).get(0).unwrap().clone().into_parts();
    parts.number = 2;
    parts.mode = w::Mode::Managed;
    parts.admin = true;
    let mut rows = List::with_capacity(1);
    rows.push(w::ForgeRow::new(&w::CEILINGS, parts.clone()).unwrap()).unwrap();
    let root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Forge(2)) }, rows);
    assert_eq!(root.pages().address(), &Address::Forges);
    for view in [f::browser_page_view(&root), f::page_view(&root)] {
        let row = view
            .tree()
            .nodes()
            .iter()
            .find(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Your Forgejo"))
            .unwrap();
        assert!(row.states.as_slice().contains(&State::Expanded));
        assert_eq!(row.description.as_deref(), Some("Forge 2 · Forgejo · Setting up · work/hello".as_bytes()));
        let live = styled(view.tree(), look::CONNECT_LIVE);
        let items: Vec<_> = live.iter().filter(|n| n.role == Role::Item).collect();
        assert_eq!(items.len(), 4);
        assert_eq!(
            f::nodes_text(live).as_ref(),
            "Checked its address and your tokenMade temper’s botMaking work/helloGiving the bot access".as_bytes()
        );
        assert!(items[0].styles.is_empty());
        assert!(items[2].states.as_slice().contains(&State::Current));
        assert!(items[3].styles.as_slice().contains(&look::CONNECT_LATER));
        assert!(!live.iter().any(|n| n.role == Role::Status));
        assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::CONNECT_CONFIRM)));
    }
    let mut first = parts.first.take().unwrap().into_parts();
    first.state = cf::made();
    parts.first = Some(w::FirstRepository::new(&w::CEILINGS, first).unwrap());
    let mut rows = List::with_capacity(1);
    rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
    let connected = f::forge_root(Address::ConnectForge { forge: Some(Choice::Forge(2)) }, rows.clone());
    let view = f::browser_page_view(&connected);
    let confirm = styled(view.tree(), look::CONNECT_CONFIRM);
    assert!(f::nodes_text(confirm).starts_with(b"Connected Your Forgejo."));
    assert!(confirm.iter().any(|n| n.role == Role::Link
        && n.name.as_deref() == Some(b"Adopt a repository")
        && n.styles.as_slice().contains(&look::CONNECT_QUIET)));
    assert!(styled(view.tree(), look::SETTINGS_ROW)[0].states.as_slice().contains(&State::Expanded));
    let reload = f::forge_root(Address::Forges, rows);
    let view = f::browser_page_view(&reload);
    assert!(!styled(view.tree(), look::SETTINGS_ROW)[0].states.as_slice().contains(&State::Expanded));
    assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::CONNECT_CONFIRM)));
    let github = f::forge_root(Address::ConnectForge { forge: Some(Choice::Forge(2)) }, cf::github_rows(false));
    assert_eq!(github.pages().address(), &Address::Forges);
    let mut view = f::browser_page_view(&github);
    assert!(view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::CONNECT_LIVE)));
    let install = f::find(view.tree(), Role::Button, b"Install on GitHub").unwrap();
    assert_eq!(
        crate::act(&mut view, install, Act::Press),
        Some(d::Action::Forge(d::forge::Act::Install { forge: 2, back: b"/forges".as_slice().into() }))
    );
}
