//! The waiting page's links and the page's fact of a tab refused.
use crate::pages::sign_in_wait::{tab_refused, waiting};
use crate::tests::fixtures;
use crate::{accounts::Bindings, look, words::Words};
use jig_client::contract as c;
use skein_ui::{Builder, Role};
use temper_client_domain as d;
use temper_client_domain::{Domain, pages::accounts};

const ADDRESS: &[u8] = b"https://issuer.example/visit";

const PROVIDER: u32 = 17;

const SOURCE: u32 = 43;

fn pressed(welcome: bool) -> (Domain, crate::View, skein_lib::Token) {
    let way = Some(d::ModelWay { provider: PROVIDER, kind: d::WayKind::SignIn, source: None });
    let address =
        if welcome { d::Address::Welcome { step: d::Step::Model { way } } } else { d::Address::ConnectModel { way } };
    let (mut root, watch) = fixtures::root_with_watch(None);
    fixtures::root_event(&mut root, d::Event::Went { place: d::Place::Address(address) });
    let offered = fixtures::waiting_root();
    let me = accounts::current_me(offered.client()).expect("waiting fixture's provider offers").clone();
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
            },
        },
    );
    let view = fixtures::browser_page_view(&root);
    let button = fixtures::find(view.tree(), Role::Button, b"Continue with ChatGPT").expect("sign-in button");
    let event = crate::event(&view, button, skein_ui::Act::Press).expect("sign-in button's press");
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits: fixtures::limits() };
    let mut out = skein_lib::Queue::uncapped();
    d::step(&mut root, &env, event, &mut out);
    d::step(&mut root, &env, d::Event::Client { event: jig_client::Event::Stored }, &mut out);
    let mut attempt = None;
    for request in &out {
        match request {
            d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt: sent, .. } },
            } => attempt = Some(*sent),
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
            | d::Request::Fact { .. } => {}
        }
    }
    (root, view, attempt.expect("press sends the sign-in"))
}

#[expect(clippy::manual_find, reason = "the view's lint profile uses loops instead of iterator closures")]
fn address_link(tree: &skein_ui::Tree) -> Option<&skein_ui::Node> {
    for node in tree.nodes() {
        if node.role == Role::Link && node.external.as_deref() == Some(ADDRESS) {
            return Some(node);
        }
    }
    None
}

fn answered(root: &mut Domain, attempt: skein_lib::Token) {
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                        c::DocumentOutcome::AccountSignInStarted { source: SOURCE, address: ADDRESS.into() },
                    )))),
                    why: None,
                },
            },
        },
    );
}

#[test]
fn refused_press_draws_its_note_then_the_primary_address_link() {
    for welcome in [false, true] {
        let (mut root, mut view, attempt) = pressed(welcome);
        fixtures::root_event(&mut root, d::Event::TabRefused);
        let tree = waiting_tree(
            &root,
            &fixtures::browser_words(),
            tab_refused(root.pages().connect_model().expect("model page")),
            fixtures::tree_limits().text_per_node,
        );
        assert!(fixtures::find(&tree, Role::Status, "ChatGPT’s sign-in".as_bytes()).is_some());
        assert!(!contains(&fixtures::text(&tree), b"in the new tab"));
        assert!(address_link(&tree).is_none());

        answered(&mut root, attempt);
        let tree = fixtures::build(&mut view, &root);
        assert!(fixtures::find(tree, Role::Status, "ChatGPT’s sign-in".as_bytes()).is_some());
        let link = address_link(tree).expect("refused tab’s primary link");
        assert_eq!(link.name.as_deref(), Some("Open ChatGPT’s sign-in ›".as_bytes()));
        assert_eq!(link.external.as_deref(), Some(ADDRESS));
        assert!(link.styles.as_slice().contains(&if welcome { look::FLOW_PRIMARY } else { look::CONNECT_PRIMARY }));
        let text = fixtures::text(tree);
        assert!(!contains(&text, b"in the new tab"));
        assert!(!contains(&text, b"Open it again"));
    }
}

#[test]
fn unrefused_press_keeps_the_new_tab_title_and_reopen_link() {
    for welcome in [false, true] {
        let (mut root, mut view, attempt) = pressed(welcome);
        answered(&mut root, attempt);
        let tree = fixtures::build(&mut view, &root);
        assert!(
            fixtures::find(tree, Role::Status, "Waiting for ChatGPT’s sign-in, in the new tab".as_bytes()).is_some(),
            "welcome={welcome}"
        );
        let link = address_link(tree).expect("opened tab’s reopen link");
        assert_eq!(link.name.as_deref(), Some(b"Open it again".as_slice()));
        assert!(contains(&fixtures::text(tree), "Open it again ›".as_bytes()));
        assert!(!link.styles.as_slice().contains(&look::FLOW_PRIMARY));
    }
}

fn waiting_tree(root: &Domain, w: &Words, refused: bool, text_per_node: u32) -> skein_ui::Tree {
    let limits = fixtures::limits();
    let mut bindings = Bindings::new(limits.client.domain.accounts, limits.row_uses, limits.client.domain.rows);
    let mut b = Builder::new(&fixtures::tree_limits());
    b.open(Role::Main);
    waiting(&mut b, root, w, &mut bindings, &root.pages().connect_model().unwrap().request, refused, text_per_node);
    b.close();
    b.finish()
}

fn contains(text: &[u8], words: &[u8]) -> bool {
    for part in text.windows(words.len()) {
        if part == words {
            return true;
        }
    }
    false
}
