//! Anonymous view checks (temper/docs/design/client/web/public.md, 4).
use super::fixtures;
use crate::sign_in::Ways;
use skein_ui::Role;

// The board and card fixtures each carry a goal/task and one delegate with the same title.

#[test]
fn the_signed_out_page_links_to_public() {
    let root = fixtures::signed_out();
    let view =
        fixtures::sign_in_view(&root, crate::words::Shell::Browser, Ways::parse("google", b"https://temper.example"));
    let link = fixtures::find(view.tree(), Role::Link, b"Read public projects").expect("public reading link");
    assert_eq!(
        view.tree().find(link).expect("link node").external.as_deref(),
        Some(b"https://temper.example/public".as_slice())
    );
}
