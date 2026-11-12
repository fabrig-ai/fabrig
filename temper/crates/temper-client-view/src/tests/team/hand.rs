//! Team's hand-off controls and sheets (client/pages/team.md, 1).
use super::*;
use crate::{
    frame::Bindings,
    words::{Times, Words},
};
use skein_ui::{Node, Shortcut, State, Tree};
use temper_client_domain::{
    Action,
    pages::{
        Act,
        team::hand::{self, Open},
    },
};

fn node<'a>(tree: &'a Tree, role: Role, name: &[u8]) -> &'a Node {
    for n in tree.nodes() {
        if n.role == role && n.name.as_deref() == Some(name) {
            return n;
        }
    }
    panic!("missing {role:?}: {name:?}")
}
fn sheet(open: &Open, words: &Words) -> (Tree, Bindings) {
    let root = f::root_in_project(Some(f::PROJECT));
    let mut b = Builder::new(&f::tree_limits());
    let mut bindings = Bindings::new(f::tree_limits().bindings);
    crate::pages::team::hand_sheet(&mut b, &root, open, words, &mut bindings);
    (b.finish(), bindings)
}
fn decode(tree: &Tree, bindings: &Bindings, role: Role, name: &[u8], act: skein_ui::Act) -> Action {
    bindings.decode(node(tree, role, name).binding.unwrap().token, act).unwrap()
}
fn has_text(tree: &Tree, words: &[u8]) -> bool {
    skein_lib::bytes::find(&f::text(tree), words).is_some()
}

#[test]
fn team_hand_and_give_controls_are_in_their_heads_and_follow_parts() {
    for w in [f::words(), f::browser_words()] {
        let base = areas::facts();
        for (member, count) in [(true, 1_u32), (true, 2), (false, 2), (true, 0)] {
            let mut facts = base.clone();
            facts.member = member;
            if count == 2 {
                let first = facts.areas.as_ref().unwrap().rows.get(0).unwrap().clone();
                let mut second = first.clone();
                second.name = b"core".as_slice().into();
                facts.areas.as_mut().unwrap().rows = List::from_boxed(Box::new([first, second]));
            }
            if count == 0 {
                facts.areas.as_mut().unwrap().rows = List::with_capacity(0);
            }
            let mut b = Builder::new(&f::tree_limits());
            let mut bindings = Bindings::new(f::tree_limits().bindings);
            draw(&mut b, &w, b"vsr", &Shown::Read(facts), f::PROJECT, &mut bindings, &mut Times::new(f::NOW));
            let tree = b.finish();
            assert_eq!(f::find(&tree, Role::Button, b"+ Hand an area to a role").is_some(), member);
            if member {
                assert_eq!(
                    decode(&tree, &bindings, Role::Button, b"+ Hand an area to a role", skein_ui::Act::Press),
                    Action::Pages(Act::HandArea(hand::Act::Open))
                );
            }
            let label = if count == 1 { b"Give the area back".as_slice() } else { b"Give the areas back".as_slice() };
            assert_eq!(f::find(&tree, Role::Button, label).is_some(), member && count > 0);
            if member && count > 0 {
                assert_eq!(
                    decode(&tree, &bindings, Role::Button, label, skein_ui::Act::Press),
                    Action::Pages(Act::HandArea(hand::Act::GiveBack))
                );
                let nodes = tree.nodes();
                let pause =
                    nodes.iter().position(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Pause")).unwrap();
                let give =
                    nodes.iter().position(|n| n.role == Role::Button && n.name.as_deref() == Some(label)).unwrap();
                assert!(pause < give);
            }
        }
    }
}
#[test]
fn the_hand_sheet_names_choices_fields_errors_and_acts() {
    for w in [f::words(), f::browser_words()] {
        let mut open = Open::new(f::PROJECT, b"vsr".as_slice().into(), areas::facts(), true, None);
        let (tree, bindings) = sheet(&open, &w);
        node(&tree, Role::Dialog, b"Hand an area to a role");
        assert!(node(&tree, Role::Choice, b"tests").states.as_slice().contains(&State::Selected));
        assert!(node(&tree, Role::Choice, b"Prover").states.as_slice().contains(&State::Selected));
        assert!(has_text(&tree, b"Tests, fuzzing and the tiers"));
        assert!(
            tree.nodes().iter().any(|n| n.text.iter().any(|r| r.run == Run::Strong && r.text.as_ref() == b"tests"))
        );
        assert_eq!(
            decode(&tree, &bindings, Role::Choice, b"A new area", skein_ui::Act::Toggle),
            Action::Pages(Act::HandArea(hand::Act::Area(None)))
        );
        assert_eq!(
            decode(&tree, &bindings, Role::Choice, b"Write one with Ask temper", skein_ui::Act::Toggle),
            Action::Pages(Act::HandArea(hand::Act::Role(None)))
        );
        assert_eq!(
            decode(&tree, &bindings, Role::Button, b"Hand it over", skein_ui::Act::Press),
            Action::Pages(Act::HandArea(hand::Act::Send))
        );
        let cancel = node(&tree, Role::Button, b"Not now");
        assert_eq!(cancel.shortcut, Some(Shortcut::ESCAPE));
        assert_eq!(
            decode(&tree, &bindings, Role::Button, b"Not now", skein_ui::Act::Press),
            Action::Pages(Act::HandArea(hand::Act::Close))
        );
        assert!(f::find(&tree, Role::TextField, b"Name").is_none());
        open.area = None;
        let (tree, bindings) = sheet(&open, &w);
        assert!(!has_text(&tree, b"A name is 1 to 32"));
        for (field, value, expected) in [
            (b"Name".as_slice(), b"tests".as_slice(), hand::Act::Name(b"tests".as_slice().into())),
            (b"What it covers", b"Tests", hand::Act::Words(b"Tests".as_slice().into())),
        ] {
            assert_eq!(
                decode(&tree, &bindings, Role::TextField, field, skein_ui::Act::Input { value: value.into() }),
                Action::Pages(Act::HandArea(expected))
            );
        }
        open.judged = true;
        let (tree, _) = sheet(&open, &w);
        assert!(has_text(&tree, b"A name is 1 to 32 lowercase letters, digits and -, starting with a letter."));
        assert!(has_text(&tree, b"What it covers is 1 to 480 bytes, on one line."));
        open.name = b"tests".as_slice().into();
        let (tree, _) = sheet(&open, &w);
        assert!(has_text(&tree, b"vsr has an area named tests already."));
        open.apply = false;
        let (tree, _) = sheet(&open, &w);
        node(&tree, Role::Button, b"Propose");
        open.role = None;
        let (tree, _) = sheet(&open, &w);
        node(&tree, Role::Button, b"Ask temper");
        open.failure = Some(hand::Failure::Outcome(jig_client::Outcome::Answered(jig_client::Answered::Jig(
            jig_client::contract::Answer::Refused { refusal: jig_client::contract::Refusal::NotOffered, at: None },
        ))));
        let (tree, _) = sheet(&open, &w);
        assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::SHEET_ERROR)));
        let area = open.facts.areas.as_ref().unwrap().rows.get(0).unwrap().clone();
        open.facts.areas.as_mut().unwrap().rows = List::from_boxed(vec![area; 16].into_boxed_slice());
        let (tree, _) = sheet(&open, &w);
        assert!(f::find(&tree, Role::Choice, b"A new area").is_none());
    }
}
#[test]
fn give_back_confirmation_names_the_requester_and_singular_or_plural_acts() {
    for w in [f::words(), f::browser_words()] {
        for (count, you, apply, title, line, keep, send) in [
            (
                1,
                true,
                true,
                b"Give tests back?".as_slice(),
                b"Its owner becomes you.".as_slice(),
                b"Keep it".as_slice(),
                b"Give it back".as_slice(),
            ),
            (2, false, true, b"Give 2 areas back?", b"Their owner becomes Sam.", b"Keep them", b"Give them back"),
            (2, false, false, b"Give 2 areas back?", b"Their owner becomes Sam.", b"Keep them", b"Propose"),
        ] {
            let facts = areas::facts();
            let requester = facts.people.get(u32::from(!you)).unwrap().party;
            let give = hand::GiveBack {
                task: 52,
                party: requester,
                names: List::from_boxed(vec![b"tests".as_slice().into(); count].into_boxed_slice()),
                charters: List::with_capacity(0),
            };
            let open = Open::new(f::PROJECT, b"vsr".as_slice().into(), facts, apply, Some(give));
            let (tree, bindings) = sheet(&open, &w);
            node(&tree, Role::Dialog, title);
            assert!(has_text(&tree, line));
            assert_eq!(
                decode(&tree, &bindings, Role::Button, keep, skein_ui::Act::Press),
                Action::Pages(Act::HandArea(hand::Act::Close))
            );
            assert_eq!(
                decode(&tree, &bindings, Role::Button, send, skein_ui::Act::Press),
                Action::Pages(Act::HandArea(hand::Act::Send))
            );
        }
    }
}
