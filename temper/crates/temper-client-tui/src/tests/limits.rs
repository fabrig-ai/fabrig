use super::fixtures as f;
use crate::words::Word;
use crate::{Refusal, Told, Tui, View, look};
use skein_ui::{Role, Tree, phrases::Hole};

pub(super) fn fits(tree: &Tree, limits: &skein_ui::Limits) {
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
    let mut top = false;
    let mut bottom = false;
    let mut feet = 0_u32;
    for node in tree.nodes() {
        top |= node.region == Some(look::TOP);
        bottom |= node.region == Some(look::BOTTOM) || node.region == Some(look::LIST);
        if node.role == Role::ContentInfo || node.region == Some(look::LIST) {
            feet = feet.checked_add(1).unwrap();
        }
    }
    assert!(top && bottom);
    assert_eq!(feet, 1, "a quiet line or list is the foot");
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks all own phrases and both sources of the tree bound")]
fn the_trees_fit_their_limits() {
    let domain = f::limits();
    let bounds = crate::limits(&domain, &skein_ui::Limits::EMPTY, 512).expect("fixture limits fit");
    assert!(skein_ui::valid(&bounds));
    let chat = skein_ui::Limits {
        nodes: 1024,
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
    let front_text = 50 * 512 + 8192 + 4096;
    let with_front = skein_ui::Limits {
        nodes: chat.nodes + 192,
        text_bytes: chat.text_bytes + front_text,
        bindings: chat.bindings + 1,
        patches: chat.patches + 192 * 4,
        ..chat
    };
    assert_eq!(crate::limits(&domain, &chat, 512), Some(with_front));
    let words = f::words();
    assert!(bounds.text_per_node >= u32::try_from(crate::BoxLine::MOST).unwrap());
    assert!(bounds.nodes >= 610);
    assert!(bounds.bindings >= 15);
    assert_eq!((crate::limits::BOX_LOG, crate::limits::LOG_LINE, crate::limits::STEP), (50, 512, 64));
    let step = "s".repeat(64);
    let title = "t".repeat(usize::try_from(domain.client.domain.text).unwrap());
    let label = "l".repeat(32);
    let step_words = skein_lib::bytes::concat(&[b"18446744073709551615 ", label.as_bytes()]);
    let quit_one = skein_lib::bytes::concat(&[b"Stop the box? ", title.as_bytes(), b"'s running task stops with it."]);
    let restarting = skein_lib::bytes::concat(&[step.as_bytes(), b" restarting"]);
    let gave_up = skein_lib::bytes::concat(&[step.as_bytes(), b" gave up"]);
    let gave_up_note =
        skein_lib::bytes::concat(&[b"The box gave up: ", step.as_bytes(), b". /log shows why; Ctrl+C closes."]);
    let mut failures = skein_lib::List::with_capacity(56);
    for (word, holes, want) in [
        (Word::BoxPurpose, &[][..], "on this machine, to try it".as_bytes()),
        (Word::WelcomeOpen, &[][..], "Welcome. Set temper up in your browser: open this link on this machine.".as_bytes()),
        (Word::LinkOnce, &[][..], "It signs you in as this temper's owner, once, within 2 hours.".as_bytes()),
        (Word::InTheBrowser, &[][..], "In the browser".as_bytes()),
        (Word::Step, &[Hole::Number(u64::MAX), Hole::Name(label.as_bytes().into())][..], &step_words),
        (Word::Following, &[][..], "Setting temper up in your browser".as_bytes()),
        (Word::SignedInOwner, &[][..], "Signed in as the owner".as_bytes()),
        (Word::StepsFollowed, &[][..], "The steps are the browser's; this screen follows them.".as_bytes()),
        (Word::Fetching, &[][..], "fetching Forgejo…".as_bytes()),
        (Word::SettingUp, &[][..], "setting up Forgejo…".as_bytes()),
        (Word::ConnectingForgejo, &[][..], "connecting Forgejo…".as_bytes()),
        (Word::ConnectingGitHub, &[][..], "connecting GitHub…".as_bytes()),
        (Word::FirstRepository, &[][..], "making its first repository…".as_bytes()),
        (Word::SetUpStopped, &[][..], "stopped: the browser says why".as_bytes()),
        (Word::Playground, &[][..], "playground".as_bytes()),
        (Word::PlaygroundBanner, &[][..], "This is the playground: a scripted model and in-memory forge. Set temper up in your browser when you're ready to leave it.".as_bytes()),
        (Word::QuitOne, &[Hole::Name(title.as_bytes().into())][..], &quit_one),
        (Word::QuitMany, &[Hole::Number(u64::MAX)][..], "Stop the box? 18446744073709551615 running tasks stop with it.".as_bytes()),
        (Word::QuitNone, &[][..], "Stop the box?".as_bytes()),
        (Word::StopBox, &[][..], "Stop the box".as_bytes()),
        (Word::DontQuit, &[][..], "Don't quit".as_bytes()),
        (Word::Restarting, &[Hole::Name(step.as_bytes().into())][..], &restarting),
        (Word::GaveUp, &[Hole::Name(step.as_bytes().into())][..], &gave_up),
        (Word::GaveUpNote, &[Hole::Name(step.as_bytes().into())][..], &gave_up_note),
        (Word::LogCommand, &[][..], "The box's recent log".as_bytes()),
        (Word::LogHead, &[][..], "/log".as_bytes()),
        (Word::LogEmpty, &[][..], "Nothing yet.".as_bytes()),
        (Word::ExitFront, &[][..], "Close temper and stop the box".as_bytes()),
        (Word::KeyCloseFront, &[][..], "Ctrl+C closes temper and stops the box".as_bytes()),
        (Word::Separator, &[][..], " · ".as_bytes()),
        (Word::Between, &[][..], " / ".as_bytes()),
        (Word::Byline, &[Hole::Name(Box::from(*b"00:00"))][..], "written by the status agent 00:00".as_bytes()),
        (Word::LeftOff, &[][..], "Where you left off".as_bytes()),
        (Word::NeedsYou, &[][..], "Needs you".as_bytes()),
        (Word::Question, &[][..], "?".as_bytes()),
        (Word::OtherNeed, &[][..], "•".as_bytes()),
        (
            Word::NoProject,
            &[Hole::Name(Box::from(f::ADDRESS))][..],
            "Setting temper up makes your first project: open http://127.0.0.1:8367 in a browser.".as_bytes(),
        ),
        (Word::Needing, &[Hole::Number(0)][..], "0 need you".as_bytes()),
        (Word::Needing, &[Hole::Number(1)][..], "1 needs you".as_bytes()),
        (Word::Needing, &[Hole::Number(2)][..], "2 need you".as_bytes()),
        (Word::Needing, &[Hole::Number(u64::MAX)][..], "18446744073709551615 need you".as_bytes()),
        (Word::NeedingCut, &[Hole::Number(3)][..], "3+ need you".as_bytes()),
        (Word::AskTemper, &[][..], "Ask temper".as_bytes()),
        (Word::Message, &[][..], "Message".as_bytes()),
        (Word::Reply, &[][..], "Reply".as_bytes()),
        (Word::Ended, &[][..], "/new starts another chat".as_bytes()),
        (Word::Commands, &[][..], "Commands".as_bytes()),
        (Word::Exit, &[][..], "Close temper".as_bytes()),
        (Word::KeyEnter, &[][..], "Enter sends".as_bytes()),
        (Word::KeyTab, &[][..], "Tab completes a command".as_bytes()),
        (Word::KeyPage, &[][..], "PgUp and PgDn read back".as_bytes()),
        (Word::KeyEsc, &[][..], "Esc closes a list".as_bytes()),
        (Word::KeyClose, &[][..], "Ctrl+C closes temper".as_bytes()),
        (Word::Else, &[][..], "Tell it something else".as_bytes()),
        (Word::Dots, &[][..], "…".as_bytes()),
        (Word::Numbered, &[Hole::Number(3)][..], "3. ".as_bytes()),
    ] {
        let said = words.say(word, holes);
        if said.as_ref() != want {
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
