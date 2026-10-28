//! Parts shared by `ConnectForge`, `ConnectModel` and the welcome (client/README.md, 5.2).
use crate::{
    look,
    words::{Word, Words},
};
use skein_ui::{Binding, Builder, Role, Run, State};

pub(crate) enum Control<'a> {
    External { name: &'a [u8], address: &'a [u8] },
    Link { name: &'a [u8], to: Binding },
    SecondaryLink { name: &'a [u8], to: Binding },
    QuietDisabled { name: &'a [u8] },
    Button { name: &'a [u8], to: Binding },
    PrimaryLink { name: &'a [u8], to: Binding },
    PrimaryButton { name: &'a [u8], to: Binding },
    PrimaryDisabled { name: &'a [u8] },
    Disabled { name: &'a [u8] },
}
pub(crate) enum Mark {
    Done,
    UnderWay,
    Failed,
    ToCome,
}
/// The list along the top, named Steps, with earlier items styled done.
pub(crate) fn steps(b: &mut Builder, w: &Words, names: &[&[u8]], current: usize) {
    b.open(Role::List);
    b.name(&w.say(Word::WelcomeSteps, &[]));
    b.style(look::FLOW_STEPS);
    for (index, name) in names.iter().enumerate() {
        b.open(Role::Item);
        b.style(look::FLOW_STEP);
        if index == current {
            b.state(State::Current);
        }
        if index < current {
            b.style(look::FLOW_DONE);
        }
        b.text(Run::Plain, name);
        b.close();
    }
    b.close();
}
/// One choice's region, named by its level-2 heading.
pub(crate) fn choice(b: &mut Builder, heading: &[u8], words: &[u8], note: &[u8], action: Control<'_>) {
    choice_card(b, heading);
    crate::cards::text(b, Role::Paragraph, words);
    b.open(Role::Paragraph);
    b.style(look::FLOW_MUTED);
    b.name(note);
    b.text(Run::Plain, note);
    b.close();
    b.open(Role::Slot);
    b.style(look::FLOW_ACTIONS);
    control(b, action);
    b.close();
    b.close();
}
/// A list whose lines say their state in words.
pub(crate) fn progress(b: &mut Builder, w: &Words, name: &[u8], lines: &[(&[u8], Mark)]) {
    b.open(Role::List);
    b.name(name);
    b.style(look::FLOW_PROGRESS);
    for (line, mark) in lines {
        b.open(Role::Item);
        match mark {
            Mark::Done => b.style(look::FLOW_DONE),
            Mark::UnderWay => b.state(State::Current),
            Mark::Failed => b.style(look::FLOW_FAILED),
            Mark::ToCome => {}
        }
        b.text(Run::Plain, line);
        let word = match mark {
            Mark::Done => Some(Word::FlowDone),
            Mark::UnderWay => Some(Word::FlowUnderWay),
            Mark::Failed => Some(Word::FlowFailed),
            Mark::ToCome => None,
        };
        if let Some(word) = word {
            b.open(Role::Status);
            b.style(look::FLOW_MUTED);
            b.name(&w.say(word, &[]));
            w.text(b, word, &[]);
            b.close();
        }
        b.close();
    }
    b.close();
}
/// Open the step's card; the caller closes it.
pub(crate) fn card(b: &mut Builder, heading: &[u8]) {
    b.open(Role::Region);
    b.style(look::FLOW_CARD);
    b.name(heading);
    crate::cards::text(b, Role::Heading { level: 2 }, heading);
}
/// Open a choice card; its action group takes the remaining height.
pub(crate) fn choice_card(b: &mut Builder, heading: &[u8]) {
    card(b, heading);
    b.style(look::FLOW_CHOICE);
}
/// Open a details card without changing Model's outer Choose card.
pub(crate) fn details(b: &mut Builder, heading: &[u8]) {
    card(b, heading);
    b.style(look::FLOW_DETAILS);
}
/// A repository’s muted line, tight beneath a setup or Ready heading.
pub(crate) fn subtitle(b: &mut Builder, words: &[u8]) {
    b.open(Role::Paragraph);
    b.style(look::FLOW_SUBTITLE);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
/// Draw the control in its given kind.
pub(crate) fn control(b: &mut Builder, action: Control<'_>) {
    let style = match action {
        Control::PrimaryLink { .. } | Control::PrimaryButton { .. } | Control::PrimaryDisabled { .. } => {
            look::FLOW_PRIMARY
        }
        Control::SecondaryLink { .. } | Control::Button { .. } | Control::Disabled { .. } => look::FLOW_SECONDARY,
        Control::External { .. } | Control::Link { .. } | Control::QuietDisabled { .. } => look::FLOW_QUIET,
    };
    match action {
        Control::External { name, address } => {
            b.open(Role::Link);
            b.name(name);
            b.external(address);
            b.text(Run::Plain, name);
        }
        Control::PrimaryLink { name, to } | Control::SecondaryLink { name, to } | Control::Link { name, to } => {
            crate::link(b, name, to);
            b.text(Run::Plain, name);
        }
        Control::PrimaryButton { name, to } | Control::Button { name, to } => {
            b.open(Role::Button);
            b.name(name);
            b.bind(to);
        }
        Control::PrimaryDisabled { name } | Control::Disabled { name } | Control::QuietDisabled { name } => {
            b.open(Role::Button);
            b.name(name);
            b.state(State::Disabled);
        }
    }
    b.style(style);
    b.close();
}
