//! Each forge's removal says what goes and stays (temper/docs/design/draft/playground-reset.md, 4).
use super::{connect_forge as cf, fixtures as f};
use crate::words::Word;
use jig_client::{Down, Up};
use skein_lib::{List, Queue, Token};
use skein_ui::{Act, Role};
use temper_client_domain::{self as d, Address, forge};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn rows(kind: w::Kind, mode: w::Mode) -> List<w::ForgeRow> {
    let mut rows = List::with_capacity(1);
    rows.push(cf::row(3, kind, mode, cf::connected(), w::Offer::NotOffered)).unwrap();
    rows
}
fn press(root: &mut d::Domain, name: &[u8]) {
    let mut view = f::page_view(root);
    let node = f::find(view.tree(), Role::Button, name).expect("button");
    let action = crate::act(&mut view, node, Act::Press).expect("bound button");
    f::root_event(root, d::Event::Act { action });
}
fn assert_words(tree: &skein_ui::Tree, role: Role, word: Word) {
    assert!(f::find(tree, role, &f::words().say(word, &[])).is_some(), "{word:?}");
}
#[test]
fn each_kind_says_what_goes_and_what_stays() {
    for (kind, mode, stays, name) in [
        (w::Kind::Github, w::Mode::Managed, Word::ForgeRemoveGitHub, b"GitHub".as_slice()),
        (w::Kind::Forgejo, w::Mode::Owned, Word::ForgeRemoveOwned, b"Forgejo on this machine"),
        (w::Kind::Forgejo, w::Mode::Managed, Word::ForgeRemoveManaged, b"Forgejo"),
        (w::Kind::Forgejo, w::Mode::Connected, Word::ForgeRemoveConnected, b"Forgejo"),
    ] {
        let mut root = f::forge_root(Address::Forges, rows(kind.clone(), mode));
        press(&mut root, b"Remove");
        assert_eq!(root.forge().removal(3), Some(&forge::remove::State::Confirming(3)));
        let view = f::page_view(&root);
        let tree = view.tree();
        let title = f::words().say(Word::ForgeRemoveTitle, &[skein_ui::phrases::Hole::Name(name.into())]);
        assert!(f::find(tree, Role::Alert, &title).is_some());
        assert_words(tree, Role::Paragraph, Word::ForgeRemoveGoes);
        assert_words(tree, Role::Paragraph, stays);
        assert_words(tree, Role::Button, Word::ForgeRemoveConfirm);
        assert_words(tree, Role::Button, Word::Cancel);
        if kind == w::Kind::Github {
            let link = f::find(tree, Role::Link, &f::words().say(Word::ForgeAppSettings, &[])).unwrap();
            for node in tree.nodes() {
                if node.id == link {
                    assert_eq!(node.external.as_deref(), Some(b"https://github.com/settings/apps".as_slice()));
                }
            }
        }
        press(&mut root, b"Cancel");
        let view = f::page_view(&root);
        assert_words(view.tree(), Role::Button, Word::ForgeRemove);
        assert!(f::find(view.tree(), Role::Alert, &title).is_none());
    }
}
fn confirm(root: &mut d::Domain) -> Token {
    press(root, b"Remove");
    let mut view = f::page_view(root);
    let node = f::find(view.tree(), Role::Button, b"Remove forge").unwrap();
    let action = crate::act(&mut view, node, Act::Press).unwrap();
    let mut out = Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        d::Event::Act { action },
        &mut out,
    );
    for request in &out {
        if let d::Request::Client {
            request:
                jig_client::Request::Down {
                    down: Down::Send { request: jig_client::Ask::Family { record, .. }, attempt, .. },
                },
        } = request
            && let w::Request::RemoveForge(_) =
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
        {
            return *attempt;
        }
    }
    panic!("removal sent")
}
fn answer(root: &mut d::Domain, key: Token, outcome: w::Outcome) {
    f::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: Up::Answer {
                    attempt: key,
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
}
#[test]
fn removing_and_refused_are_said() {
    for why in [w::RemoveRefusal::NotOffered, w::RemoveRefusal::Role] {
        let mut root = f::forge_root(Address::Forges, rows(w::Kind::Forgejo, w::Mode::Owned));
        let key = confirm(&mut root);
        let view = f::page_view(&root);
        assert!(f::find(view.tree(), Role::Status, "Removing Forgejo on this machine…".as_bytes()).is_some());
        let mut count = 0_u32;
        for (index, node) in view.tree().nodes().iter().enumerate() {
            if node.styles.as_slice().contains(&crate::look::FORGE_ROW) {
                count += 1;
                let end = index.checked_add(usize::try_from(node.size).unwrap()).unwrap();
                for child in &view.tree().nodes()[index..end] {
                    assert_ne!(child.role, Role::Button);
                }
            }
        }
        assert_eq!(count, 1);
        let refusal = match why {
            w::RemoveRefusal::NotOffered => jig_client::contract::Refusal::NotOffered,
            w::RemoveRefusal::Role => jig_client::contract::Refusal::Role { held: None },
            w::RemoveRefusal::NoSuchForge => unreachable!(),
        };
        answer(
            &mut root,
            key,
            w::Outcome::RemoveRefused(w::RemoveRefused::new(&w::CEILINGS, w::RemoveRefusedParts { why }).unwrap()),
        );
        let view = f::page_view(&root);
        let reason = f::words().say_word(jig_client::words::Word::refusal(&refusal), &[]);
        let expected = f::words().say(Word::ForgeRemoveRefused, &[skein_ui::phrases::Hole::Name(reason)]);
        assert!(f::find(view.tree(), Role::Status, &expected).is_some());
        assert_words(view.tree(), Role::Button, Word::ForgeRemove);
    }
}
#[test]
fn removed_draws_nothing_until_the_watch_drops_it() {
    let (mut root, watch) = f::forge_root_watch(Address::Forges, rows(w::Kind::Forgejo, w::Mode::Owned));
    let key = confirm(&mut root);
    answer(&mut root, key, w::Outcome::Removing);
    let view = f::page_view(&root);
    for node in view.tree().nodes() {
        assert!(!node.styles.as_slice().contains(&crate::look::FORGE_ROW));
    }
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::forges(
                            &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: List::with_capacity(0) }).unwrap(),
                        ),
                    },
                },
            },
        },
    );
    let view = f::page_view(&root);
    assert_words(view.tree(), Role::Paragraph, Word::NoForges);
}
#[test]
fn the_playgrounds_row_has_no_remove() {
    let mut rows = List::with_capacity(1);
    rows.push(cf::row(wire::FORGE, w::Kind::Forgejo, w::Mode::Fake, cf::connected(), w::Offer::NotOffered)).unwrap();
    let root = f::demo_forges_root(Address::Forges, rows);
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Heading { level: 2 }, b"The playground").is_some());
    assert!(f::find(view.tree(), Role::Button, b"Remove").is_none());
}
