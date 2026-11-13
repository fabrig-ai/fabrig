#[expect(unused_imports, reason = "the whole-view fixture facade also exposes the staged render adapters")]
pub use crate::parts_tests::fixtures::{owner_part_text, part_text, part_tree};
pub use crate::view_kit::*;
use skein_lib::Queue;

pub fn built() -> crate::View {
    let domain = domain_limits();
    let mut root = temper_client_domain::unlinked(&domain, SEED);
    root_event(
        &mut root,
        temper_client_domain::Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Transport } },
    );
    let forge = words();
    let envelope = crate::Limits::of(&domain, &forge).expect("composed envelope fits");
    assert!(envelope.tree.nodes >= 128);
    let limits = crate::Limits { tree: tree_limits() };
    let mut view = crate::View::new(&limits, &domain, forge, FORGE);
    crate::build(&mut view, &root, NOW, &mut Queue::with_capacity(limits.tree.patches));
    view
}
pub fn view() -> crate::View {
    view_of(words())
}
pub fn view_of(words: crate::words::Words) -> crate::View {
    let domain = domain_limits();
    let limits = crate::Limits { tree: tree_limits() };
    crate::View::new(&limits, &domain, words, FORGE)
}
pub fn build<'a>(view: &'a mut crate::View, root: &temper_client_domain::Domain) -> &'a skein_ui::Tree {
    crate::build(view, root, NOW, &mut Queue::with_capacity(view.limits().tree.patches));
    view.tree()
}
pub fn signed_in_with_project() -> crate::View {
    let mut view = view();
    build(&mut view, &root_in_project(Some(PROJECT)));
    view
}
/// Act `Input` on the named text area, as a person typing, and return the area's value after the root took it.
pub fn type_into(root: &mut temper_client_domain::Domain, name: &[u8], words: &[u8]) -> Box<[u8]> {
    let mut view = page_view(root);
    let area = find(view.tree(), skein_ui::Role::TextArea, name).expect("text area");
    let action = crate::act(&mut view, area, skein_ui::Act::Input { value: Box::from(words) }).expect("input action");
    root_event(root, temper_client_domain::Event::Act { action });
    let view = page_view(root);
    for node in view.tree().nodes() {
        if node.role == skein_ui::Role::TextArea && node.name.as_deref() == Some(name) {
            return node.value.as_ref().expect("area value").text.clone();
        }
    }
    panic!("text area");
}
/// Build a page once for the status-only page tests.
pub fn page_view(root: &temper_client_domain::Domain) -> crate::View {
    let mut view = view();
    build(&mut view, root);
    view
}
/// `page_view`'s, with the browser's words.
pub fn browser_page_view(root: &temper_client_domain::Domain) -> crate::View {
    let mut view = view_of(browser_words());
    build(&mut view, root);
    view
}
/// Page-advertised sign-in ways, in either shell.
pub fn sign_in_view(
    root: &temper_client_domain::Domain,
    shell: crate::words::Shell,
    ways: crate::sign_in::Ways,
) -> crate::View {
    let domain = domain_limits();
    let limits = crate::Limits { tree: tree_limits() };
    let words = crate::words::Words::english(FORGE, domain.client.domain.text, shell).unwrap();
    let mut view = crate::View::with_sign_in(&limits, &domain, words, FORGE, ways);
    build(&mut view, root);
    view
}

pub mod chat;
