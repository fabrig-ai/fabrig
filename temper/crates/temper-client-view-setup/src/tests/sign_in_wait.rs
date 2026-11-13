//! The waiting page's links and the page's fact of a tab refused.
use super::*;
use crate::tests::fixtures;

const ADDRESS: &[u8] = b"https://issuer.example/visit";

#[expect(clippy::manual_find, reason = "the view's lint profile uses loops instead of iterator closures")]
fn address_link(tree: &skein_ui::Tree) -> Option<&skein_ui::Node> {
    for node in tree.nodes() {
        if node.role == Role::Link && node.external.as_deref() == Some(ADDRESS) {
            return Some(node);
        }
    }
    None
}

fn drawn(w: &Words, refused: bool, text_per_node: u32) -> skein_ui::Tree {
    let root = fixtures::waiting_root();
    waiting_tree(&root, w, refused, text_per_node)
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

#[test]
fn refused_tab_keeps_the_plain_title_and_a_primary_external_link() {
    let tree = drawn(&fixtures::browser_words(), true, fixtures::tree_limits().text_per_node);
    assert!(fixtures::find(&tree, Role::Status, "ChatGPT’s sign-in".as_bytes()).is_some());
    let link = address_link(&tree).expect("refused tab’s primary link");
    assert_eq!(link.external.as_deref(), Some(b"https://issuer.example/visit".as_slice()));
    assert!(link.styles.as_slice().contains(&look::CONNECT_PRIMARY));
    let text = fixtures::text(&tree);
    assert!(contains(&text, "Open ChatGPT’s sign-in ›".as_bytes()));
    assert!(!contains(&text, b"Open it again"));
    assert!(!contains(&text, b"in the new tab"));
    assert!(
        fixtures::find(&tree, Role::Disclosure, "Signed in, but the page couldn’t be reached?".as_bytes()).is_some()
    );
}

#[test]
fn address_link_fits_at_the_text_bound_and_alerts_one_byte_past_it() {
    const ADDRESS: &[u8] = b"https://issuer.example/visit";
    for (w, label) in [
        (fixtures::browser_words(), "Open it again ›".as_bytes()),
        (fixtures::words(), b"Open the sign-in page".as_slice()),
    ] {
        // An external link's text bound includes its label and target.
        let bound = u32::try_from(ADDRESS.len() + label.len()).unwrap();
        let tree = drawn(&w, false, bound);
        let mut linked = false;
        for node in tree.nodes() {
            assert_ne!(node.role, Role::Alert);
            if node.role == Role::Link && node.external.as_deref() == Some(ADDRESS) {
                linked = true;
            }
        }
        assert!(linked, "the sign-in link fits at the text bound");
        let tree = drawn(&w, false, bound - 1);
        for node in tree.nodes() {
            assert!(node.external.is_none());
        }
        let expected = words::say(&w, W::AddressLong, &[Hole::Number(u64::try_from(ADDRESS.len()).unwrap())]);
        assert!(fixtures::find(&tree, Role::Alert, &expected).is_some());
        assert!(fixtures::find(&tree, Role::Button, b"Finish connecting").is_some());
    }
}

fn contains(text: &[u8], words: &[u8]) -> bool {
    for part in text.windows(words.len()) {
        if part == words {
            return true;
        }
    }
    false
}
