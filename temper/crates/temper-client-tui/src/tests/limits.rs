use super::fixtures as f;
use crate::words::Word;
use crate::{Refusal, Told, Tui, View, look};
use skein_ui::{Role, Tree, phrases::Hole};

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
    assert_eq!(f::count(tree, Role::Main), 1);
    assert_eq!(f::count(tree, Role::ContentInfo), 1);
    let mut top = false;
    let mut bottom = false;
    for node in tree.nodes() {
        top |= node.region == Some(look::TOP);
        bottom |= node.region == Some(look::BOTTOM);
    }
    assert!(top && bottom);
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks all own phrases and both sources of the tree bound")]
fn the_trees_fit_their_limits() {
    let domain = f::limits();
    let bounds = crate::limits(&domain, &skein_ui::Limits::EMPTY, 512).expect("fixture limits fit");
    assert!(skein_ui::valid(&bounds));
    let chat = skein_ui::Limits {
        nodes: 512,
        depth: 64,
        text_per_node: 1_048_576,
        text_bytes: 2_097_152,
        drawing_bytes: 512,
        styles_per_node: 5,
        patches: 4096,
        bindings: 128,
        slots: 32,
        markdown_depth: 5,
        scene_marks: 2,
        scene_bytes: 256,
    };
    assert_eq!(crate::limits(&domain, &chat, 512), Some(chat));
    let words = f::words();
    let mut failures = skein_lib::List::with_capacity(27);
    for (word, holes, want) in [
        (Word::Separator, &[][..], " · "),
        (Word::Between, &[][..], " / "),
        (Word::Byline, &[Hole::Name(Box::from(*b"00:00"))][..], "written by the status agent 00:00"),
        (Word::LeftOff, &[][..], "Where you left off"),
        (Word::NeedsYou, &[][..], "Needs you"),
        (Word::Question, &[][..], "?"),
        (Word::OtherNeed, &[][..], "•"),
        (
            Word::NoProject,
            &[Hole::Name(Box::from(f::ADDRESS))][..],
            "Setting temper up makes your first project: open http://127.0.0.1:8367 in a browser.",
        ),
        (Word::Needing, &[Hole::Number(0)][..], "0 need you"),
        (Word::Needing, &[Hole::Number(1)][..], "1 needs you"),
        (Word::Needing, &[Hole::Number(2)][..], "2 need you"),
        (Word::Needing, &[Hole::Number(u64::MAX)][..], "18446744073709551615 need you"),
        (Word::NeedingCut, &[Hole::Number(3)][..], "3+ need you"),
        (Word::AskTemper, &[][..], "Ask temper"),
        (Word::Message, &[][..], "Message"),
        (Word::Reply, &[][..], "Reply"),
        (Word::Ended, &[][..], "/new starts another chat"),
        (Word::Commands, &[][..], "Commands"),
        (Word::Exit, &[][..], "Close temper"),
        (Word::KeyEnter, &[][..], "Enter sends"),
        (Word::KeyTab, &[][..], "Tab completes a command"),
        (Word::KeyPage, &[][..], "PgUp and PgDn read back"),
        (Word::KeyEsc, &[][..], "Esc closes a list"),
        (Word::KeyClose, &[][..], "Ctrl+C closes temper"),
        (Word::Else, &[][..], "Tell it something else"),
        (Word::Dots, &[][..], "…"),
        (Word::Numbered, &[Hole::Number(3)][..], "3. "),
    ] {
        let said = words.say(word, holes);
        if said.as_ref() != want.as_bytes() {
            failures.push(word).unwrap();
        }
        assert!(u32::try_from(said.len()).unwrap() <= word.capacity(&domain).unwrap(), "{word:?}");
    }
    assert!(failures.is_empty(), "word rows: {failures:?}");
    let name = [b'n'; 32];
    let address = *b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:65535";
    let me = f::me(&[&name]);
    let root = f::live(Some(me)).0;
    let mut tui = Tui::new(&address);
    tui.step(Told::Kept { written: false });
    let mut view = View::new(&bounds, f::words());
    f::build(&mut view, &tui, &root);
    fits(view.tree(), &bounds);
    f::find(view.tree(), Role::Text, b"nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn");
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
    assert_eq!(crate::limits(&domain, &skein_ui::Limits::EMPTY, u32::MAX), None);
    let mut huge = domain;
    huge.client.domain.text = u32::MAX;
    assert_eq!(crate::limits(&huge, &skein_ui::Limits::EMPTY, 512), None);
}
