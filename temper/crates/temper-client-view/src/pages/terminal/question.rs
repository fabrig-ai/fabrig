//! A waiting chat shares Needs you's question and answer (temper/docs/design/client/README.md, 3.5).
use super::{Act, Bindings, Builder, Domain, Role, Run, State, Words, look};
use crate::words::Word;
use jig_client::contract as c;
use temper_client_domain::{acts, pages::needs};

pub(super) fn under(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) -> bool {
    if root.pages().confirmation().is_some() {
        return false;
    }
    let Some(inbox) = root.pages().inbox() else { return false };
    for card in &root.pages().needs().open(inbox) {
        if card.group != c::InboxGroup::Questions || card.key.is_some() || card.project.is_none() {
            continue;
        }
        let Some(page) = &card.page else { continue };
        let Some(c::Admission::Allowed | c::Admission::Bounded { .. }) =
            jig_client::facts::offer(acts::offers(page), c::ActKind::Answer)
        else {
            continue;
        };
        let asked = super::super::ask::read(card, w);
        let Some(parsed) = asked.parsed else { continue };
        b.open(Role::Slot);
        b.open(Role::Text);
        b.style(look::TERMINAL_HEAD);
        b.name(&parsed.question);
        b.text(Run::Plain, &parsed.question);
        b.close();
        if parsed.choices.is_empty() {
            let name = super::say(w, super::T::AnswerInNeeds);
            crate::link(b, &name, bindings.insert(Act::Needs(needs::Act::Open { object: card.object })));
            b.text(Run::Plain, &name);
            b.close();
        } else {
            b.open(Role::Menu);
            b.style(look::TERMINAL_CHOICES);
            b.name(&parsed.question);
            b.state(State::Options);
            for (index, choice) in parsed.choices.iter().enumerate() {
                b.open(Role::MenuItem);
                b.name(&choice.words);
                if Some(index) == parsed.primary() {
                    b.state(State::Selected);
                }
                if choice.recommended {
                    b.description(&w.say(Word::NeedsRecommended, &[]));
                }
                b.bind(
                    bindings
                        .insert(Act::Needs(needs::Act::Choose { object: card.object, words: choice.words.clone() })),
                );
                b.close();
            }
            b.open(Role::MenuItem);
            b.name(&w.say(Word::NeedsLater, &[]));
            b.bind(bindings.insert(Act::Needs(needs::Act::Later { object: card.object })));
            b.close();
            b.close();
        }
        if let Some(outcome) = root.pages().needs().refused(card.object) {
            crate::confirm::outcome_words(b, w, outcome);
        }
        b.close();
        return true;
    }
    false
}
