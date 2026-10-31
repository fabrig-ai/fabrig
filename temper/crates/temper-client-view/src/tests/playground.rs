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
            f::playground_root(Address::Chats { filter: ChatsFilter::All, start: false }, project, model)
        } else {
            f::chats_root()
        };
        let view = f::page_view(&root);
        assert_eq!(
            f::find(view.tree(), Role::Heading { level: 1 }, b"What shall we change?").is_some(),
            shown,
            "{row}"
        );
        assert_eq!(f::find(view.tree(), Role::List, b"Suggested requests").is_some(), shown, "{row}");
        assert!(f::find(view.tree(), Role::Heading { level: 2 }, b"Your chats").is_some(), "{row}");
        if shown {
            assert!(
                f::find(
                    view.tree(),
                    Role::Paragraph,
                    b"Try a request in acme/shop. Follow the task from its branch to a landed change."
                )
                .is_some(),
                "{row}"
            );
            assert!(
                f::find(view.tree(), Role::Paragraph, b"The scripted model plays these requests from its script.")
                    .is_some(),
                "{row}"
            );
            let mut heading = None;
            let mut suggestion = None;
            let mut composer = None;
            for (index, node) in view.tree().nodes().iter().enumerate() {
                if node.role == (Role::Heading { level: 1 }) {
                    heading = Some(index);
                }
                if node.role == Role::List && node.name.as_deref() == Some(b"Suggested requests") {
                    suggestion = Some(index);
                }
                if node.role == Role::TextArea {
                    composer = Some(index);
                }
            }
            assert!(heading.unwrap() < suggestion.unwrap() && suggestion.unwrap() < composer.unwrap(), "{row}");
        }
    }
}
#[test]
fn suggestion_buttons_wait_for_a_start_and_press_their_own_cue() {
    for (row, pending) in [("ready", false), ("start pending", true)] {
        let mut root = f::playground_root(Address::Chats { filter: ChatsFilter::All, start: false }, f::PROJECT, true);
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
        let mut start = None;
        for candidate in view.tree().nodes() {
            if candidate.role == Role::Button && candidate.name.as_deref() == Some(b"Start chat") {
                start = Some(candidate);
            }
        }
        let start = start.expect("start button");
        assert_eq!(start.states.as_slice().contains(&State::Disabled), pending, "{row}: composer send");
        let field = f::find(view.tree(), Role::TextArea, b"Start a chat").expect("chat field");
        assert_eq!(
            crate::act(&mut view, field, skein_ui::Act::Submit),
            if pending { None } else { Some(Action::Pages(Act::StartChat)) },
            "{row}: Enter shares send readiness"
        );
    }
}

#[test]
fn the_playground_banner_is_absent_when_the_link_is_unavailable() {
    for (row, up) in [
        ("signed out", jig_client::Up::SignedOut),
        ("out of date", jig_client::Up::Served { served: false, versions: None }),
    ] {
        let mut root = f::playground_root(Address::Chats { filter: ChatsFilter::All, start: false }, f::PROJECT, true);
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
        let root = f::playground_root(Address::Chat { task: 1 }, project, model);
        let view = f::page_view(&root);
        let nodes = view.tree().nodes();
        let mut banner = None;
        let mut main = None;
        let mut chatbar = None;
        let mut playground = None;
        let mut thread = None;
        for (index, node) in nodes.iter().enumerate() {
            if node.role == Role::Banner {
                banner = Some(index);
            }
            if node.role == Role::Main {
                main = Some(index);
            }
            if node.styles.as_slice().contains(&crate::look::CHATBAR) {
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
        assert!(f::find(view.tree(), Role::Paragraph, b"Scripted model").is_none(), "{row}");
    }
}
#[test]
fn the_playground_account_always_names_default_whatever_the_model_or_project() {
    use jig_client::contract as c;
    for (row, project, model, requested, expected) in [
        ("default", f::PROJECT, true, false, "Using Scripted model · Default"),
        ("requested", f::PROJECT, true, true, "Using Scripted model · Default"),
        ("another project", f::SECOND_PROJECT, true, false, "Using Scripted model · Default"),
    ] {
        let choice = if requested {
            Some(c::LlmChoice {
                account: None,
                main: c::ModelEffort { model: c::Text::new(b"selected".as_slice().into(), 32).unwrap(), effort: None },
                subagents: skein_lib::List::with_capacity(0),
            })
        } else {
            None
        };
        let root = f::playground_choice_root(Address::Chat { task: 1 }, project, model, choice);
        let view = f::page_view(&root);
        assert!(skein_lib::bytes::find(&f::text(view.tree()), expected.as_bytes()).is_some(), "{row}");
    }
    let root = f::playground_root(Address::Chats { filter: ChatsFilter::All, start: false }, f::PROJECT, true);
    assert!(
        f::find(f::page_view(&root).tree(), Role::Paragraph, "Using Scripted model · Default".as_bytes()).is_some()
    );
}

#[test]
fn conversation_speakers_use_you_temper_or_the_other_person() {
    for (row, person, requester, person_word, party_word) in [
        ("mine", Some(f::chat::PARTY), true, b"You".as_slice(), b"You".as_slice()),
        ("another requester", Some(f::chat::PARTY + 1), false, b"Person", b"Person"),
    ] {
        let mut b = skein_ui::Builder::new(&f::tree_limits());
        // A request with words opens the conversation under its speaker.
        let mut request = f::task::document().spec;
        request.specification.words = f::task::document_words(b"Rotate tokens safely.");
        let mut thread = super::chat::structured_thread();
        thread.lines.edges.after = jig_client::thread::window::Edge::End;
        crate::pages::chat::conversation(
            &mut b,
            &thread,
            &f::words(),
            (f::chat::LINES, f::chat::TEXT),
            crate::pages::chat::Speakers { person, requester },
            crate::pages::chat::Drawn::Chat { request: Some(&request), delegations: None },
            &mut crate::frame::Bindings::new(f::tree_limits().bindings),
        );
        let tree = b.finish();
        let said = f::text(&tree);
        let mut region = None;
        for (index, node) in tree.nodes().iter().enumerate() {
            if node.role == Role::Region {
                region = Some(index);
                break;
            }
        }
        let request_text = f::node_text(&tree, region.expect("request region"));
        assert!(skein_lib::bytes::find(&request_text, person_word).is_some(), "{row}");
        assert!(f::find(&tree, Role::Paragraph, party_word).is_some(), "{row}");
        assert!(skein_lib::bytes::find(&said, b"Temper").is_none(), "{row}: a call-only message has no heading");
    }
}

#[test]
fn pending_words_without_a_party_are_named_you() {
    use jig_client::contract as c;
    let mut thread = jig_client::thread::Thread::new(
        1,
        &jig_client::thread::Limits { lines: f::chat::LINES, bytes: f::chat::BYTES, text: f::chat::TEXT },
    );
    assert!(thread.words.begin(
        jig_client::Key { asker: jig_client::Asker::JIG, number: 1 },
        None,
        c::Text::new(b"Pending".as_slice().into(), f::chat::TEXT).unwrap()
    ));
    let mut b = skein_ui::Builder::new(&f::tree_limits());
    crate::pages::chat::conversation(
        &mut b,
        &thread,
        &f::words(),
        (f::chat::LINES, f::chat::TEXT),
        crate::pages::chat::Speakers { person: Some(73), requester: true },
        crate::pages::chat::Drawn::Chat { request: None, delegations: None },
        &mut crate::frame::Bindings::new(f::tree_limits().bindings),
    );
    assert!(f::find(&b.finish(), Role::Paragraph, b"You").is_some());
}

#[test]
fn a_real_account_keeps_the_playgrounds_scripted_model() {
    let root = f::playground_kept_root();
    assert!(root.forge().playground().is_some());
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Paragraph, "Using Scripted model · Default".as_bytes()).is_some());
}

#[test]
fn the_hidden_deployment_playground_account_needs_only_its_public_provider() {
    use jig_client::contract::Readiness;
    for (ready, payer, expected) in [
        (Some(Readiness::Usable), 73, "Using Scripted model · Default"),
        (Some(Readiness::Unknown), 73, "Using Scripted model · Default"),
        (None, 73, "Using Scripted model · Default"),
        (Some(Readiness::Unavailable), 73, "Scripted model · needs an account"),
        (Some(Readiness::Usable), 74, "Using Scripted model · Default"),
    ] {
        let root = f::hidden_playground_root(ready, payer);
        let view = f::page_view(&root);
        assert!(f::find(view.tree(), Role::Paragraph, expected.as_bytes()).is_some());
    }
}

#[test]
fn the_playground_chooser_names_the_provider_and_model_but_keeps_the_configured_choice() {
    let mut root = f::playground_root(Address::Chats { filter: ChatsFilter::All, start: false }, f::PROJECT, true);
    let mut view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Paragraph, b"Playground").is_some());
    assert!(f::find(view.tree(), Role::Button, crate::words::SCRIPTED_MODEL).is_none());
    let model = f::find(view.tree(), Role::Button, b"Scripted model").expect("scripted model");
    let action = crate::act(&mut view, model, skein_ui::Act::Press).expect("model choice");
    assert_eq!(action, Action::Pages(Act::Choice(temper_client_domain::pages::choice::Act::Model { index: 0 })));
    f::root_event(&mut root, Event::Act { action });
    assert_eq!(root.pages().choice().requested.as_ref().unwrap().main.model.bytes(), crate::words::SCRIPTED_MODEL);
}

#[test]
fn the_playground_account_is_in_its_own_folded_section() {
    // The fixture supplies no provider name and an old display name: the playground's
    // public provider number alone identifies the playground's account.
    let root = f::playground_root(Address::Accounts, f::PROJECT, true);
    let view = f::page_view(&root);
    let nodes = view.tree().nodes();
    let mut count = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.role == (Role::Heading { level: 2 })
            && node.name.as_deref() == Some(b"Your accounts")
            && let Some(node) = nodes.get(index + usize::try_from(node.size).unwrap())
        {
            count = node.name.as_deref();
        }
    }
    assert_eq!(count, Some(b"0".as_slice()));
    assert!(f::find(view.tree(), Role::Paragraph, b"No model of yours yet.").is_some());
    assert!(
        f::find(view.tree(), Role::Disclosure, "Scripted model · the playground’s · held by the deployment".as_bytes())
            .is_some()
    );
}

#[test]
fn the_accounts_banner_precedes_the_playground_when_both_show() {
    for address in [Address::Chats { filter: ChatsFilter::All, start: false }, Address::Chat { task: 1 }] {
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
        let view = f::page_view(&root);
        let nodes = view.tree().nodes();
        let mut frame = None;
        let mut accounts = None;
        let mut playground = None;
        let mut main = None;
        for (index, node) in nodes.iter().enumerate() {
            if node.role == Role::Banner {
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
        assert!(
            frame.unwrap() < accounts.unwrap()
                && accounts.unwrap() < playground.unwrap()
                && playground.unwrap() < main.unwrap()
        );
    }
}
