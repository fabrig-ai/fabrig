//! Debug-only tab fixture for the browser story (skein/docs/design/ui/web/README.md, 8, and skein/docs/design/ui/web/testing.md, 3).
use skein_lib::{Queue, Token};
use skein_ui::{Act, Binding, Builder, NodeId, Owner, Role, Run, View};
use skein_ui_web::{
    Look, fetch,
    inbox::Inbox,
    mapping::Primary,
    page::{Page, web::Dom},
    platform::{self, Platform},
};
use std::{cell::RefCell, rc::Rc};

const OWNER: Owner = Owner { number: 1 };

enum Report {
    Start,
    Act(NodeId, Act),
    Fetched,
}

pub(super) fn start(mount: web_sys::Element, primary: Primary) {
    let limits = skein_ui_example_view::limits::limits();
    let page = Rc::new(RefCell::new(Page::new(
        Look { names: &[], pictures: &[], components: &[], elements: &[], draw: skein_ui_example_view::draw::draw },
        &limits,
        b"tab",
        primary,
    )));
    let inbox = Inbox::new();
    let fetch = fetch::web::Fetch::new(1, Rc::clone(&inbox), |_| Report::Fetched);
    let mut platform = platform::Browser::new(fetch);
    let dom = Dom::new(mount, &page, &inbox, Report::Act, &[]);
    let mut view = View::new(&limits);
    let mut patches = Queue::with_capacity(limits.patches);
    let mut revision = 0;
    let mut status = b"Ready".as_slice();
    let at_start = web_sys::window().expect("window").location().search().expect("page query") == "?at-start";
    inbox.start(Box::new(move |inbox| {
        while let Some(report) = inbox.next() {
            let open = match report {
                Report::Start => at_start,
                Report::Act(node, act) => {
                    let Some(bound) = view.act(node, act) else { continue };
                    match bound.binding.token.raw() {
                        1 => true,
                        2 => {
                            platform.point_tab(b"/test/tab/landed");
                            false
                        }
                        3 => {
                            platform.point_tab(b"javascript:alert(1)");
                            false
                        }
                        4 => {
                            platform.close_tab();
                            false
                        }
                        _ => unreachable!("the fixture's four bindings"),
                    }
                }
                Report::Fetched => false,
            };
            if open {
                status = if platform.open_tab("Opening the page…".as_bytes()) { b"Opened" } else { b"Refused" };
            }
        }
        let mut builder = view.builder();
        build(&mut builder, status);
        view.finish(builder, revision, &mut patches);
        revision += 1;
        let patches: Vec<_> = std::iter::from_fn(|| patches.pop()).collect();
        let mut ops = Vec::new();
        page.borrow_mut().apply(&patches, &mut ops);
        dom.apply(&ops);
    }));
    inbox.raise(Report::Start);
}

fn build(builder: &mut Builder, status: &[u8]) {
    builder.open(Role::Page);
    builder.open(Role::Main);
    for (name, token) in [(b"Open".as_slice(), 1), (b"Point", 2), (b"Point outside", 3), (b"Close", 4)] {
        builder.open(Role::Button);
        builder.name(name);
        builder.bind(Binding { owner: OWNER, token: Token::new(token) });
        builder.close();
    }
    builder.open(Role::Status);
    builder.name(status);
    builder.text(Run::Plain, status);
    builder.close();
    builder.close();
    builder.close();
}
