use super::*;
use crate::pages::{
    chat::document::Mode,
    terminal::{self, Font, Look, Theme},
};

#[test]
fn the_look_moves_by_its_acts() {
    let limits = fixtures::limits();
    let root = crate::unlinked(&limits, fixtures::SEED);
    let dark = Look::default();
    let light = Look { terminal: true, theme: Theme::Light, font: Font::Plex };
    for (before, act, after) in [
        (dark, terminal::Act::Enter, Look { terminal: true, ..dark }),
        (dark, terminal::Act::Leave, dark),
        (dark, terminal::Act::Theme, Look { theme: Theme::Light, ..dark }),
        (dark, terminal::Act::Font, Look { font: Font::Plex, ..dark }),
        (light, terminal::Act::Enter, light),
        (light, terminal::Act::Leave, Look { terminal: false, ..light }),
        (light, terminal::Act::Theme, Look { theme: Theme::Dark, ..light }),
        (light, terminal::Act::Font, Look { font: Font::DejaVu, ..light }),
    ] {
        let mut pages = Pages::new();
        assert_eq!(pages.look(), dark);
        let env = fixtures::env(&limits);
        let mut out = Queue::uncapped();
        if before == light {
            for act in [terminal::Act::Enter, terminal::Act::Theme, terminal::Act::Font] {
                pages::step(&mut pages, &env, root.client(), Event::Act { act: Act::Terminal(act) }, &mut out);
            }
        }
        pages::step(&mut pages, &env, root.client(), Event::Went { address: Address::Chat { task: 9 } }, &mut out);
        pages::step(&mut pages, &env, root.client(), Event::Act { act: Act::Mode { mode: Mode::Split } }, &mut out);
        pages::step(
            &mut pages,
            &env,
            root.client(),
            Event::Act { act: Act::MessageWords { words: Box::from(b"Keep these words".as_slice()) } },
            &mut out,
        );
        pages::step(
            &mut pages,
            &fixtures::env(&limits),
            root.client(),
            Event::Act { act: Act::Terminal(act) },
            &mut Queue::uncapped(),
        );
        assert_eq!(pages.look(), after);
        assert_eq!(pages.address(), &Address::Chat { task: 9 });
        let chat = pages.chat().unwrap();
        assert_eq!(chat.mode, Mode::Split);
        assert_eq!(chat.words.as_ref(), b"Keep these words");
    }
    for (address, expected) in [
        (Address::Chat { task: 9 }, true),
        (Address::NewChat { draft: false }, true),
        (Address::Configure { project: 1 }, true),
        (Address::Chats { filter: ChatsFilter::All }, false),
        (Address::Home { project: 1 }, false),
    ] {
        assert_eq!(terminal::draws(&address), expected, "{address:?}");
    }
    for (mode, draft) in [(Mode::Chat, false), (Mode::Split, true)] {
        let mut pages = Pages::new();
        let env = fixtures::env(&limits);
        let mut out = Queue::uncapped();
        pages::step(
            &mut pages,
            &env,
            root.client(),
            Event::Went { address: Address::NewChat { draft: !draft } },
            &mut out,
        );
        pages::step(
            &mut pages,
            &env,
            root.client(),
            Event::Act { act: Act::ChatWords { words: Box::from(b"Keep the opening".as_slice()) } },
            &mut out,
        );
        pages::step(&mut pages, &env, root.client(), Event::Act { act: Act::Terminal(terminal::Act::Enter) }, &mut out);
        pages::step(&mut pages, &env, root.client(), Event::Act { act: Act::Mode { mode } }, &mut out);
        assert_eq!(pages.address(), &Address::NewChat { draft });
        assert_eq!(pages.chats().unwrap().words.as_ref(), b"Keep the opening");
        assert_eq!(pages.look(), Look { terminal: true, ..dark });
    }
}
