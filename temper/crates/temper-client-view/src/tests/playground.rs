use super::fixtures as f;

use skein_ui::{Role, State};

use temper_client_domain::ChatsFilter;

use temper_client_domain::{Action, Address, Event, pages::Act};

#[test]
fn suggestions_show_only_in_the_playground_project() {
    for (row, project, model, playground, shown) in [
        ("playground project", f::PROJECT, true, true, true),
        ("other project", f::SECOND_PROJECT, true, true, false),
        ("no playground", f::PROJECT, true, false, false),
    ] {
        let root = if playground {
            f::playground_root(Address::NewChat { draft: false }, project, model)
        } else {
            let mut root = f::chats_root();
            f::root_event(
                &mut root,
                Event::Went { place: temper_client_domain::Place::Address(Address::NewChat { draft: false }) },
            );
            root
        };
        let view = f::page_view(&root);
        assert_eq!(f::find(view.tree(), Role::List, b"Suggested requests").is_some(), shown, "{row}");
        assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"New chat").is_some(), "{row}");
        if shown {
            let mut heading = None;
            let mut suggestion = None;
            for (index, node) in view.tree().nodes().iter().enumerate() {
                if node.role == (Role::Heading { level: 1 }) {
                    heading = Some(index);
                }
                if node.role == Role::List && node.name.as_deref() == Some(b"Suggested requests") {
                    suggestion = Some(index);
                }
            }
            assert!(heading.unwrap() < suggestion.unwrap(), "{row}");
        }
    }
}

#[test]
fn suggestion_buttons_wait_for_a_start_and_press_their_own_cue() {
    for (row, pending) in [("ready", false), ("start pending", true)] {
        let mut root = f::playground_root(Address::NewChat { draft: false }, f::PROJECT, true);
        if pending {
            for act in [Act::ChatWords { words: b"My draft".as_slice().into() }, Act::StartChat] {
                f::root_event(&mut root, Event::Act { action: Action::Pages(act) });
            }
            assert!(root.pages().chats().unwrap().start.is_some());
        }
        let mut view = f::page_view(&root);
        for (cue, name) in [(0, f::PLAYGROUND_CUE), (1, f::PLAYGROUND_OTHER_CUE)] {
            let button = f::find(view.tree(), Role::Button, name).expect("suggestion");
            let mut node = None;
            for candidate in view.tree().nodes() {
                if candidate.role == Role::Button && candidate.name.as_deref() == Some(name) {
                    node = Some(candidate);
                }
            }
            let node = node.expect("suggestion node");
            assert_eq!(node.states.as_slice().contains(&State::Disabled), pending, "{row}");
            assert_eq!(
                crate::act(&mut view, button, skein_ui::Act::Press),
                if pending { None } else { Some(Action::Pages(Act::Suggest { cue })) },
                "{row}"
            );
        }
    }
}

#[test]
fn the_playground_banner_is_absent_when_the_link_is_unavailable() {
    for (row, up) in [
        ("signed out", jig_client::Up::SignedOut),
        ("out of date", jig_client::Up::Served { served: false, versions: None }),
    ] {
        let mut root = f::playground_root(Address::Chats { filter: ChatsFilter::All }, f::PROJECT, true);
        f::root_event(&mut root, Event::Client { event: jig_client::Event::Up { up } });
        let view = f::page_view(&root);
        assert!(f::find(view.tree(), Role::Region, b"Playground").is_none(), "{row}");
        assert!(f::find(view.tree(), Role::List, b"Suggested requests").is_none(), "{row}");
    }
}

#[test]
fn a_chats_banner_precedes_main_and_its_chatbar_in_every_project() {
    for (row, project, model) in [
        ("playground project", f::PROJECT, true),
        ("another project", f::SECOND_PROJECT, true),
        ("forge alone", f::PROJECT, false),
    ] {
        let mut root = f::playground_root(Address::Chat { task: 1 }, project, model);
        // While it runs a chat's page has no line; while it is leaving, the line (client/view.md, 1).
        assert!(f::find(f::page_view(&root).tree(), Role::Region, b"Playground").is_none(), "{row}");
        f::root_event(&mut root, Event::Act { action: Action::Forge(temper_client_domain::forge::Act::EndPlayground) });
        let view = f::page_view(&root);
        let nodes = view.tree().nodes();
        let mut banner = None;
        let mut main = None;
        let mut chatbar = None;
        let mut playground = None;
        let mut thread = None;
        for (index, node) in nodes.iter().enumerate() {
            if node.styles.as_slice().contains(&crate::look::RAIL) {
                banner = Some(index);
            }
            if node.role == Role::Main {
                main = Some(index);
            }
            if node.styles.as_slice().contains(&crate::look::SESSION_TOP) {
                chatbar = Some(index);
            }
            if node.styles.as_slice().contains(&crate::look::THREAD) {
                thread = Some(index);
            }
            if node.role == Role::Region && node.name.as_deref() == Some(b"Playground") {
                playground = Some(index);
            }
        }
        assert!(banner.unwrap() < playground.unwrap() && playground.unwrap() < main.unwrap(), "{row}");
        // The playground is a frame sibling before Main, never inside it.
        assert_eq!(
            main.unwrap(),
            playground.unwrap() + usize::try_from(nodes[playground.unwrap()].size).unwrap(),
            "{row}"
        );
        assert!(main.unwrap() < chatbar.unwrap() && chatbar.unwrap() < thread.unwrap(), "{row}");
        assert!(f::find(view.tree(), Role::Heading { level: 1 }, "T1 · Ship the update".as_bytes()).is_some(), "{row}");
        for node in &nodes[chatbar.unwrap()..thread.unwrap()] {
            assert!(!(node.role == Role::Paragraph && node.name.as_deref() == Some(b"Scripted model")), "{row}");
        }
    }
}

#[test]
fn the_playground_chooser_names_the_provider_and_model_but_keeps_the_configured_choice() {
    let mut root = f::playground_root(Address::NewChat { draft: false }, f::PROJECT, true);
    let mut view = f::page_view(&root);
    assert!(skein_lib::bytes::find(&f::text(view.tree()), b"Playground").is_some());
    assert!(f::find(view.tree(), Role::Button, crate::words::SCRIPTED_MODEL).is_none());
    let model = f::find(view.tree(), Role::Button, b"Scripted model").expect("scripted model");
    let action = crate::act(&mut view, model, skein_ui::Act::Press).expect("model choice");
    assert_eq!(
        action,
        Action::Pages(Act::Choice(temper_client_domain::pages::choice::Act::Model { index: 0, account: None }))
    );
    f::root_event(&mut root, Event::Act { action });
    assert_eq!(
        root.pages()
            .choice()
            .chosen(temper_client_domain::pages::accounts::current_me(root.client()).unwrap())
            .unwrap()
            .main
            .model
            .bytes(),
        crate::words::SCRIPTED_MODEL
    );
}

#[test]
fn the_playground_account_is_in_its_own_folded_section() {
    // The fixture supplies no provider name and an old display name: the playground's
    // public provider number alone identifies the playground's account.
    let root = f::playground_root(Address::Accounts, f::PROJECT, true);
    let view = f::page_view(&root);
    let nodes = view.tree().nodes();
    let mut counted = false;
    for node in nodes {
        if node.role == (Role::Heading { level: 2 }) && node.name.as_deref() == Some("Your accounts · 0".as_bytes()) {
            counted = true;
        }
    }
    assert!(counted);
    assert!(f::find(view.tree(), Role::Paragraph, b"No model of yours yet.").is_some());
    assert!(
        f::find(view.tree(), Role::Disclosure, "Scripted model · the playground’s · held by the deployment".as_bytes())
            .is_some()
    );
}

#[test]
fn the_accounts_banner_is_hidden_only_in_the_running_or_leaving_playground_project() {
    for (address, project, leaving, chats) in [
        (Address::Chats { filter: ChatsFilter::All }, f::PROJECT, false, true),
        (Address::Chat { task: 1 }, f::PROJECT, false, false),
        (Address::Chats { filter: ChatsFilter::All }, f::SECOND_PROJECT, false, true),
        (Address::Chat { task: 1 }, f::SECOND_PROJECT, false, false),
        (Address::Chats { filter: ChatsFilter::All }, f::PROJECT, true, true),
        (Address::Chat { task: 1 }, f::PROJECT, true, false),
    ] {
        let (mut root, _, _, _, watch) =
            f::root_at_with_playground(address, &f::domain_limits(), b"Shop", Some((f::PROJECT, true)));
        let mut doc = temper_client_domain::pages::accounts::document(root.client()).unwrap().clone();
        doc.usable = false;
        f::root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: watch.unwrap(),
                        event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Accounts(
                            Box::new(doc),
                        )),
                    },
                },
            },
        );
        f::root_event(&mut root, Event::Act { action: Action::Pages(Act::Project { project }) });
        if leaving {
            f::root_event(
                &mut root,
                Event::Act { action: Action::Forge(temper_client_domain::forge::Act::EndPlayground) },
            );
        }
        let view = f::page_view(&root);
        let nodes = view.tree().nodes();
        let mut frame = None;
        let mut accounts = None;
        let mut playground = None;
        let mut main = None;
        for (index, node) in nodes.iter().enumerate() {
            if node.styles.as_slice().contains(&crate::look::RAIL) {
                frame = Some(index);
            }
            if node.name.as_deref() == Some(b"Agents need an LLM account to run.") {
                accounts = Some(index);
            }
            if node.role == Role::Region && node.name.as_deref() == Some(b"Playground") {
                playground = Some(index);
            }
            if node.role == Role::Main {
                main = Some(index);
            }
        }
        assert_eq!(accounts.is_some(), project != f::PROJECT);
        // Outside the chats pane the line shows while it is leaving alone (client/view.md, 1).
        if !chats && !leaving {
            assert!(playground.is_none());
            continue;
        }
        assert!(frame.unwrap() < playground.unwrap());
        if let Some(accounts) = accounts {
            assert!(frame.unwrap() < accounts && accounts < main.unwrap());
        }
        assert!(playground.unwrap() < main.unwrap());
    }
}

#[test]
fn the_frame_banner_follows_the_page_and_leaving_rules() {
    for (address, leaving, shown) in [
        (Address::Chats { filter: ChatsFilter::All }, false, true),
        (Address::NewChat { draft: false }, false, true),
        (Address::Chat { task: 1 }, false, false),
        (Address::Chat { task: 1 }, true, true),
    ] {
        let mut root = f::playground_root(address.clone(), f::PROJECT, true);
        if leaving {
            f::root_event(
                &mut root,
                Event::Act { action: Action::Forge(temper_client_domain::forge::Act::EndPlayground) },
            );
        }
        let view = f::page_view(&root);
        let nodes = view.tree().nodes();
        let mut banner = None;
        let mut main = None;
        for (index, node) in nodes.iter().enumerate() {
            if node.role == Role::Region && node.name.as_deref() == Some(b"Playground") {
                banner = Some(index);
            }
            if node.role == Role::Main {
                main = Some(index);
            }
        }
        let main = main.expect("main");
        assert_eq!(banner.is_some(), shown, "{address:?}, leaving={leaving}");
        if let Some(banner) = banner {
            assert_eq!(banner + usize::try_from(nodes[banner].size).unwrap(), main, "banner is before main");
        }
    }
}
