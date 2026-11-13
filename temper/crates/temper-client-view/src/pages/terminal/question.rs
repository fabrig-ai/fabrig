//! A waiting chat shares Needs you's question and answer (temper/docs/design/client/pages/README.md).
use super::{Act, Bindings, Builder, Domain, Role, Run, State, Words, look};
use crate::words::Word;
use temper_client_domain::pages::needs;

pub(super) fn under(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) -> bool {
    let Some(question) = crate::shared::question(root, w) else { return false };
    b.open(Role::Slot);
    b.open(Role::Text);
    b.style(look::TERMINAL_HEAD);
    b.name(&question.head);
    b.text(Run::Plain, &question.head);
    b.close();
    if question.choices.is_empty() {
        let name = super::say(w, super::T::AnswerInNeeds);
        crate::bindings::link(b, &name, bindings.insert(Act::Needs(needs::Act::Open { object: question.object })));
        b.text(Run::Plain, &name);
        b.close();
    } else {
        b.open(Role::Menu);
        b.style(look::TERMINAL_CHOICES);
        b.name(&question.head);
        b.state(State::Options);
        for (index, choice) in question.choices.iter().enumerate() {
            b.open(Role::MenuItem);
            b.name(&choice.words);
            if Some(index) == question.primary {
                b.state(State::Selected);
            }
            if choice.recommended {
                b.description(&w.say(Word::NeedsRecommended, &[]));
            }
            b.bind(
                bindings
                    .insert(Act::Needs(needs::Act::Choose { object: question.object, words: choice.words.clone() })),
            );
            b.close();
        }
        b.open(Role::MenuItem);
        b.name(&w.say(Word::NeedsLater, &[]));
        b.bind(bindings.insert(Act::Needs(needs::Act::Later { object: question.object })));
        b.close();
        b.close();
    }
    if let Some(outcome) = root.pages().needs().refused(question.object) {
        crate::confirm::outcome_words(b, w, outcome);
    }
    b.close();
    true
}
