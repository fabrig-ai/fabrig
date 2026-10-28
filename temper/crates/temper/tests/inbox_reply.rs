pub mod support;

use skein_fake_person::{Face, Find, Found, Step, TreeFace};
use skein_lib::Queue;
use skein_ui::{Builder, Limits, NodeId, Role, Run, View};
use support::person::{ReplyFace, inbox_reply};

// Page, optional outer Item, and two cards of Item/heading/text/link/text each.
const LIMITS: Limits = Limits {
    nodes: 12,
    depth: 5,
    text_per_node: 64,
    text_bytes: 256,
    drawing_bytes: 0,
    styles_per_node: 0,
    patches: 72,
    bindings: 0,
    slots: 0,
    markdown_depth: 0,
    scene_marks: 0,
    scene_bytes: 0,
};
const TITLE: &[u8] = b"T2 \xc2\xb7 Land a small change";
// Preserve the story's case-insensitive substring matching of Item names.
const QUERY: &[u8] = b"land a SMALL change";
// Page, Result Item/heading/text/link/text, Reply Item/heading.
const REPLY: NodeId = NodeId(8);
fn card(builder: &mut Builder, title: &[u8], heading: &[u8]) {
    builder.open(Role::Item);
    builder.open(Role::Heading { level: 3 });
    builder.name(heading);
    builder.text(Run::Plain, heading);
    builder.close();
    builder.open(Role::Link);
    builder.text(Run::Plain, title);
    builder.close();
    builder.close();
}
// A reply card is headed by its own task, as the inbox draws it: "T2 Land a small change replied".
fn fixture(reply: Option<(&[u8], &[u8])>, nested: bool) -> View {
    let mut view = View::new(&LIMITS);
    let mut builder = view.builder();
    builder.open(Role::Page);
    if nested {
        builder.open(Role::Item);
        builder.name(b"Other task");
    }
    card(&mut builder, TITLE, b"Updates");
    if let Some((title, heading)) = reply {
        card(&mut builder, title, heading);
    }
    if nested {
        builder.close();
    }
    builder.close();
    view.finish(builder, 1, &mut Queue::with_capacity(LIMITS.patches));
    view
}
fn reply() -> Find {
    let Step::See { find, .. } = inbox_reply(QUERY) else { panic!("Reply expectation") };
    find
}

#[test]
fn a_reply_is_found_after_the_same_tasks_result() {
    let view = fixture(Some((TITLE, b"T2 Land a small change replied")), false);
    let find = reply();
    assert_eq!(TreeFace::new(view.tree()).find(&find), Found::Absent, "kit keeps first-match scoping");
    assert_eq!(ReplyFace::new(view.tree()).find(&find), Found::Node(REPLY));
}

#[test]
fn a_result_alone_does_not_satisfy_the_reply() {
    let view = fixture(None, false);
    assert_eq!(ReplyFace::new(view.tree()).find(&reply()), Found::Absent);
}

#[test]
fn another_tasks_reply_does_not_satisfy_the_reply() {
    let view = fixture(Some((b"T3 \xc2\xb7 Another task", b"T3 Another task replied")), true);
    assert_eq!(ReplyFace::new(view.tree()).find(&reply()), Found::Absent);
}
