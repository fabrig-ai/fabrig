//! Mount the example's live tree fixture (skein/docs/design/ui/web/testing.md, 3).
use super::NAMES;
use skein_lib::Queue;
use skein_ui::{Act, NodeId};
use skein_ui_example_view::tree::{self, Case, Tree};
use skein_ui_web::{
    Look,
    inbox::Inbox,
    page::{Page, web::Dom},
};
use std::{cell::RefCell, rc::Rc};

enum Report {
    Start,
    Act(NodeId, Act),
}

pub(super) fn start(mount: web_sys::Element, path: &str, primary: skein_ui_web::mapping::Primary) {
    let mut tree = Tree::new(Case::from_path(path).expect("declared fixture route"));
    let limits = tree::limits();
    let page = Rc::new(RefCell::new(Page::new(
        Look { names: NAMES, pictures: &[], components: &[], elements: &[], draw: skein_ui_example_view::draw::draw },
        &limits,
        b"notes",
        primary,
    )));
    let inbox = Inbox::new();
    let dom = Dom::new(mount, &page, &inbox, Report::Act, &[]);
    let mut patches = Queue::with_capacity(limits.patches);
    inbox.start(Box::new(move |inbox| {
        while let Some(report) = inbox.next() {
            match report {
                Report::Start => {}
                Report::Act(node, act) => tree.act(node, act),
            }
        }
        while tree.patch(&mut patches) {
            let patches: Vec<_> = std::iter::from_fn(|| patches.pop()).collect();
            let mut ops = Vec::new();
            page.borrow_mut().apply(&patches, &mut ops);
            dom.apply(&ops);
        }
    }));
    inbox.raise(Report::Start);
}
