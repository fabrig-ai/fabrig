use super::fixtures as f;
use crate::{Refusal, Told, Tui, View, look};
use skein_ui::{Role, Tree};

fn fits(tree: &Tree, limits: &skein_ui::Limits) {
    assert_eq!(skein_ui::outline::check(tree), None);
    assert!(tree.len() <= limits.nodes);
    let mut total = 0_u32;
    let mut bindings = 0_u32;
    for (index, node) in tree.nodes().iter().enumerate() {
        let mut depth = 1_u32;
        for (start, parent) in tree.nodes().iter().enumerate().take(index) {
            if start.checked_add(usize::try_from(parent.size).unwrap()).unwrap() > index {
                depth = depth.checked_add(1).unwrap();
            }
        }
        assert!(depth <= limits.depth);
        assert!(node.text_bytes() <= limits.text_per_node);
        total = total.checked_add(node.text_bytes()).unwrap();
        bindings = bindings.checked_add(u32::from(node.binding.is_some())).unwrap();
        assert!(node.styles.len() <= limits.styles_per_node);
    }
    assert!(total <= limits.text_bytes);
    assert!(bindings <= limits.bindings);
    assert_eq!(f::count(tree, Role::Page), 1);
    let mut regions = 0_u32;
    for node in tree.nodes() {
        if let Some(region) = node.region {
            regions = regions.checked_add(1).unwrap();
            if node.role == Role::Main {
                assert_eq!(region, look::TOP);
            } else {
                assert_eq!(node.role, Role::ContentInfo);
                assert_eq!(region, look::BOTTOM);
            }
        }
    }
    assert_eq!(f::count(tree, Role::Main), 1);
    assert_eq!(f::count(tree, Role::ContentInfo), 1);
    assert_eq!(regions, 2);
}

#[test]
fn the_trees_fit_their_limits() {
    let domain = f::limits();
    let bounds = crate::limits(&domain, 512).expect("fixture limits fit");
    assert!(skein_ui::valid(&bounds));
    let name = [b'n'; 32];
    let address = *b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:65535";
    let mut me = f::me(&[&name, &name]);
    me.party.name = jig_client::contract::DocumentText::complete(
        jig_client::contract::Text::new(Box::from(name), domain.client.domain.text).unwrap(),
    );
    assert_eq!(me.projects.len(), domain.client.domain.projects);
    let root = f::live(Some(me)).0;
    let mut tui = Tui::new(&address);
    tui.step(Told::Kept { written: false });
    let mut view = View::new(&bounds, f::words());
    f::build(&mut view, &tui, &root);
    fits(view.tree(), &bounds);
    let projects = f::find(
        view.tree(),
        Role::Paragraph,
        b"Projects: nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn \xc2\xb7 nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn",
    );
    assert_eq!(f::text(view.tree(), projects).len(), 78);
    let full = f::refused(f::full());
    let approval = super::approval::asked(Some(temper_client_domain::asked::State::Asked {
        code: super::approval::code(),
        expires: skein_lib::Wall::from_nanos(600_000_000_000),
    }));
    for (refusal, browser) in [
        (Some(Refusal::OtherAddress { address: Box::from(address) }), false),
        (None, false),
        (Some(Refusal::OtherAddress { address: Box::from(address) }), true),
    ] {
        let mut tui = Tui::new(&address);
        tui.step(Told::Link { link: jig_client::Link::Live });
        tui.step(Told::Link { link: jig_client::Link::SignedOut });
        if browser {
            tui.step(Told::Browse);
            tui.step(Told::Opened { opened: false });
        } else {
            tui.step(Told::Asked);
        }
        tui.step(Told::Typed { value: Box::new([b'x'; 512]) });
        match refusal {
            Some(refusal) => tui.step(Told::Refused { refusal }),
            None => tui.step(Told::Answered { lost: false }),
        }
        f::build(&mut view, &tui, if browser { &approval } else { &full });
        fits(view.tree(), &bounds);
        let field = f::find(view.tree(), Role::TextField, b"Link");
        assert_eq!(field.value.as_ref().unwrap().text.len(), 512);
        assert_eq!(view.tree().len(), if browser { 28 } else { 18 });
    }
    assert_eq!(crate::limits(&domain, u32::MAX), None);
    let mut huge = domain;
    huge.client.domain.text = u32::MAX;
    huge.client.domain.projects = u32::MAX;
    assert_eq!(crate::words::projects_bytes(&huge), u32::MAX.checked_sub(10).unwrap());
    assert_eq!(crate::limits(&huge, 512), None);
}
