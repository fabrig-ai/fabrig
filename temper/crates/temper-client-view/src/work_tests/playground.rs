use super::fixtures as f;

use skein_ui::{Role, State};

use temper_client_domain::ChatsFilter;

use temper_client_domain::{Action, Address, Event, pages::Act};

#[test]
fn the_playground_names_default_for_a_resolved_model() {
    use jig_client::contract as c;
    for (row, project, model, requested, expected) in [
        ("default", f::PROJECT, true, false, "Scripted model · Default"),
        ("requested", f::PROJECT, true, true, "Scripted model · Default"),
        ("another project", f::SECOND_PROJECT, true, false, "Scripted model · Default"),
    ] {
        let choice = if requested {
            Some(c::LlmChoice {
                later: None,
                account: None,
                main: c::ModelEffort { model: c::Text::new(b"selected".as_slice().into(), 32).unwrap(), effort: None },
                subagents: skein_lib::List::with_capacity(0),
            })
        } else {
            None
        };
        let root = f::playground_choice_root(Address::Chat { task: 1 }, project, model, choice);
        assert_eq!(super::choice::using_text(&root), expected, "{row}");
    }
    let root = f::playground_root(Address::NewChat { draft: false }, f::PROJECT, true);
    assert_eq!(super::choice::using_text(&root), "Scripted model · Default");
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
            &mut crate::bindings::Bindings::new(f::tree_limits().bindings),
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
        &jig_client::thread::Limits {
            lines: f::chat::LINES,
            bytes: f::chat::BYTES,
            text: f::chat::TEXT,
            live: f::chat::TEXT,
        },
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
        &mut crate::bindings::Bindings::new(f::tree_limits().bindings),
    );
    assert!(f::find(&b.finish(), Role::Paragraph, b"You").is_some());
}


#[test]
fn a_real_account_keeps_the_playgrounds_scripted_model() {
    let root = f::playground_kept_root();
    assert!(root.forge().playground().is_some());
    assert_eq!(super::choice::using_text(&root), "Scripted model · Default");
}


#[test]
fn the_hidden_deployment_playground_account_needs_only_its_public_provider() {
    use jig_client::contract::Readiness;
    for (ready, payer, expected) in [
        (Some(Readiness::Usable), 73, "Scripted model · Default"),
        (Some(Readiness::Unknown), 73, "Scripted model · Default"),
        (None, 73, "Scripted model · Default"),
        (Some(Readiness::Unavailable), 73, "Scripted model · Default"),
        (Some(Readiness::Usable), 74, "Scripted model · Default"),
    ] {
        let root = f::hidden_playground_root(ready, payer);
        assert!(super::choice::using_text(&root).ends_with(expected));
    }
}
