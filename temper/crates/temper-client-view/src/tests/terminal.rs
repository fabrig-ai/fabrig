//! A chat's terminal frame (temper/docs/design/client/README.md, 3.5).
#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect semantic trees and bindings")]
use super::{
    document::{ProposalState, document_root},
    fixtures as f,
};
use crate::{
    look,
    words::{Shell, Words},
};
use jig_client::contract as c;
use skein_ui::{Role, State, Style, Tree};
use temper_client_domain::{
    self as d, Action, Address, Event,
    pages::{Act, chat::document::Mode, terminal},
};
fn act(root: &mut d::Domain, act: Act) {
    f::root_event(root, Event::Act { action: Action::Pages(act) });
}
fn view(root: &d::Domain, shell: Shell) -> crate::View {
    let words = Words::english(f::FORGE, 512, shell).unwrap();
    let mut limits = f::domain_limits();
    limits.client.domain.text = 512;
    limits.client.domain.long_text = 65536;
    let bounds = crate::Limits::of(&limits, &words).unwrap();
    let mut view = crate::View::new(&bounds, &limits, words, f::FORGE);
    f::build(&mut view, root);
    view
}
fn styled(tree: &Tree, token: Style) -> bool {
    tree.nodes().iter().any(|n| n.styles.as_slice().contains(&token))
}
#[test]
fn the_terminal_draws_a_chats_pages_in_the_browser_alone() {
    for address in [
        Address::Chat { task: f::chat::TASK },
        Address::NewChat { draft: false },
        Address::NewChat { draft: true },
        Address::Configure { project: f::PROJECT },
        Address::Chats { filter: d::ChatsFilter::All },
        Address::Home { project: f::PROJECT },
    ] {
        let eligible = terminal::draws(&address);
        for on in [false, true] {
            let mut root = f::chat::messaging_root();
            if address != *root.pages().address() {
                f::root_event(&mut root, Event::Went { place: d::Place::Address(address.clone()) });
            }
            if on {
                act(&mut root, Act::Terminal(terminal::Act::Enter));
            }
            for shell in [Shell::Browser, Shell::Terminal] {
                let mut view = view(&root, shell);
                let terminal = eligible && on && shell == Shell::Browser;
                assert_eq!(
                    view.tree()
                        .nodes()
                        .iter()
                        .any(|n| n.role == Role::Main && n.styles.as_slice().contains(&look::TERMINAL)),
                    terminal,
                    "{address:?} {shell:?} {on}"
                );
                assert_eq!(styled(view.tree(), look::RAIL), !terminal, "rail: {address:?} {shell:?} {on}");
                if terminal {
                    assert!(!styled(view.tree(), look::BANNER));
                }
                assert_eq!(styled(view.tree(), look::TABBAR), shell == Shell::Browser && !terminal, "tabs");
                assert_eq!(styled(view.tree(), look::TO_TERMINAL), eligible && shell == Shell::Browser && !terminal);
                if let Some(id) = f::find(view.tree(), Role::Button, b"Terminal, full screen") {
                    assert!(view.tree().find(id).unwrap().styles.as_slice().contains(&look::TO_TERMINAL));
                    assert_eq!(
                        crate::act(&mut view, id, skein_ui::Act::Press),
                        Some(Action::Pages(Act::Terminal(terminal::Act::Enter)))
                    );
                }
                if terminal && address == (Address::NewChat { draft: true }) {
                    let id = f::find(view.tree(), Role::Button, b"Draft").unwrap();
                    assert!(view.tree().find(id).unwrap().states.as_slice().contains(&State::Pressed));
                    let decoded = crate::act(&mut view, id, skein_ui::Act::Press).unwrap();
                    let before = root.pages().chats().unwrap().words.clone();
                    f::root_event(&mut root, Event::Act { action: decoded });
                    assert_eq!(root.pages().address(), &Address::NewChat { draft: false });
                    assert_eq!(root.pages().chats().unwrap().words, before);
                }
                if terminal {
                    act(&mut root, Act::Terminal(terminal::Act::Theme));
                    act(&mut root, Act::Terminal(terminal::Act::Font));
                    let changed = self::view(&root, shell);
                    assert!(styled(changed.tree(), look::TERMINAL_LIGHT));
                    assert!(styled(changed.tree(), look::TERMINAL_PLEX));
                }
            }
        }
    }
}
fn state_root(state: u8) -> d::Domain {
    let mut task = super::chat::configuring_task();
    task.chip.phase = c::Phase::Active;
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Idle);
    task.waits_on = c::WaitsOn::News;
    task.chip.details.details.live_delegates = 0;
    task.delegates.clear();
    task.run_wait = None;
    task.requester = c::Requester::Person(f::chat::PARTY);
    task.wake.words = c::WakeRule::Immediate;
    match state {
        0 => {}
        1 => {
            task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Due);
            task.waits_on = c::WaitsOn::Nothing;
        }
        2 => {
            task.waits_on = c::WaitsOn::Delegates;
            task.chip.details.details.live_delegates = 1;
        }
        3 => super::chat::pause_chip(&mut task.chip),
        4 => {
            task.chip.phase = c::Phase::Ended;
            task.chip.details.phase = c::ChipPhase::Ended;
        }
        5 => {
            task.run_wait = Some(c::TaskRunWait::Account { account: None, source: None, why: c::TaskWaitWhy::Empty });
        }
        _ => unreachable!(),
    }
    task.chip.phase = task.chip.details.phase.compact();
    super::chat::configuring_task_root(task, true, 1)
}
#[test]
fn the_terminal_says_where_it_is_and_what_runs() {
    for (state, running, paused, ended) in [
        (0, false, false, false),
        (1, true, false, false),
        (2, true, false, false),
        (3, false, true, false),
        (4, false, false, true),
        (5, true, false, false),
    ] {
        let mut root = state_root(state);
        let browser = view(&root, Shell::Browser);
        let status = browser
            .tree()
            .nodes()
            .iter()
            .find(|n| {
                n.role == Role::Status
                    && n.styles.as_slice().iter().any(|s| {
                        [look::STATUS_QUIET, look::STATUS_ACTIVE, look::STATUS_WAITING, look::STATUS_YOURS].contains(s)
                    })
            })
            .unwrap()
            .name
            .clone();
        act(&mut root, Act::Terminal(terminal::Act::Enter));
        let mut view = view(&root, Shell::Browser);
        let tree = view.tree();
        assert!(f::find(tree, Role::Heading { level: 1 }, b"T9 \xc2\xb7 Hello").is_some());
        assert!(tree.nodes().iter().any(|n| n.text.iter().any(|r| r.text.as_ref() == "✦".as_bytes())));
        for name in [b"Shop".as_slice(), b"/", b"T9"] {
            assert!(tree.nodes().iter().any(|n| n.name.as_deref() == Some(name)), "{name:?}");
        }
        let run = tree.nodes().iter().find(|n| n.styles.as_slice().contains(&look::TERMINAL_RUN));
        assert_eq!(run.is_some(), running, "state {state}");
        if let Some(run) = run {
            assert_eq!(run.name, status);
            assert_eq!(run.role, Role::Status);
        }
        assert_eq!(styled(tree, look::SESSION_COMPOSER), !ended);
        assert_eq!(styled(tree, look::TERMINAL_LINE), !ended);
        if !ended {
            let hint = if paused { "paused · a message resumes it" } else { "/ for commands" };
            assert!(f::find(tree, Role::Paragraph, hint.as_bytes()).is_some(), "state {state}");
            assert!(f::find(tree, Role::TextArea, b"Message").is_some());
            assert!(f::find(tree, Role::Paragraph, "7 spent · of 7".as_bytes()).is_some());
        }
        for name in [b"Send".as_slice(), b"Pause", b"Resume", b"Close", b"Start over", "Start goal…".as_bytes()] {
            assert!(f::find(tree, Role::Button, name).is_none(), "{name:?}");
        }
        let exit = f::find(tree, Role::Button, b"Exit").unwrap();
        assert_eq!(
            crate::act(&mut view, exit, skein_ui::Act::Press),
            Some(Action::Pages(Act::Terminal(terminal::Act::Leave)))
        );
    }
    for (body, mode, state, draft) in [
        (Some(b"# Draft\n".as_slice()), Mode::Split, ProposalState::Ordinary, Some(true)),
        (Some(b"# Draft\n".as_slice()), Mode::Chat, ProposalState::Ordinary, Some(false)),
        (None, Mode::Chat, ProposalState::DraftOffered, None),
        (None, Mode::Chat, ProposalState::Ordinary, None),
    ] {
        let mut root = document_root(body, mode, &[], state);
        act(&mut root, Act::Terminal(terminal::Act::Enter));
        let mut view = view(&root, Shell::Browser);
        if let Some(pressed) = draft {
            let id = f::find(view.tree(), Role::Button, b"Draft").unwrap();
            let node = view.tree().find(id).unwrap();
            assert_eq!(node.states.as_slice().contains(&State::Pressed), pressed);
            assert_eq!(node.shortcut, Some(skein_ui::Shortcut { key: b'\\', modifier: skein_ui::Modifier::Primary }));
            assert_eq!(
                crate::act(&mut view, id, skein_ui::Act::Press),
                Some(Action::Pages(Act::Mode { mode: if pressed { Mode::Chat } else { Mode::Split } }))
            );
        } else {
            assert!(f::find(view.tree(), Role::Button, b"Draft").is_none());
            let plus = f::find(view.tree(), Role::Button, b"+ Draft");
            assert_eq!(plus.is_some(), state == ProposalState::DraftOffered);
            if let Some(id) = plus {
                assert_eq!(crate::act(&mut view, id, skein_ui::Act::Press), Some(Action::Pages(Act::AddDraft)));
            }
        }
    }
}
