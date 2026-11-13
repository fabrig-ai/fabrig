//! Parts shared by `ConnectForge`, `ConnectModel` and the welcome (client/view.md, 2).
use crate::{
    look,
    words::{Word, Words},
};
use skein_ui::{Binding, Builder, Role, Run, State};

#[derive(Debug)]
pub enum Control<'a> {
    External { name: &'a [u8], address: &'a [u8] },
    Link { name: &'a [u8], to: Binding },
    SecondaryLink { name: &'a [u8], to: Binding },
    Button { name: &'a [u8], to: Binding },
    PrimaryLink { name: &'a [u8], to: Binding },
    PrimaryButton { name: &'a [u8], to: Binding },
    PrimaryDisabled { name: &'a [u8] },
    Disabled { name: &'a [u8] },
}
#[derive(Debug)]
pub enum Mark {
    Done,
    UnderWay,
    Failed,
    Stopped,
    ToCome,
}
/// A list whose lines say their state in words.
pub fn progress(b: &mut Builder, w: &Words, name: &[u8], lines: &[(&[u8], Mark)]) {
    b.open(Role::List);
    b.name(name);
    b.style(look::FLOW_PROGRESS);
    for (line, mark) in lines {
        b.open(Role::Item);
        match mark {
            Mark::Done => b.style(look::FLOW_DONE),
            Mark::UnderWay => b.state(State::Current),
            Mark::Failed | Mark::Stopped => b.style(look::FLOW_FAILED),
            Mark::ToCome => {}
        }
        b.text(Run::Plain, line);
        let word = match mark {
            Mark::Done => Some(Word::FlowDone),
            Mark::UnderWay => Some(Word::FlowUnderWay),
            Mark::Failed => Some(Word::FlowFailed),
            Mark::Stopped => Some(Word::FlowStopped),
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
pub fn choice_card(b: &mut Builder, heading: &[u8]) {
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
pub fn control(b: &mut Builder, action: Control<'_>) {
    let style = match action {
        Control::PrimaryLink { .. } | Control::PrimaryButton { .. } | Control::PrimaryDisabled { .. } => {
            look::FLOW_PRIMARY
        }
        Control::SecondaryLink { .. } | Control::Button { .. } | Control::Disabled { .. } => look::FLOW_SECONDARY,
        Control::External { .. } | Control::Link { .. } => look::FLOW_QUIET,
    };
    control_in(b, action, style);
}
/// Settings' connect controls use the same roles and bindings in their own look.
pub(crate) fn connect_control(b: &mut Builder, action: Control<'_>) {
    let style = match action {
        Control::PrimaryLink { .. } | Control::PrimaryButton { .. } | Control::PrimaryDisabled { .. } => {
            look::CONNECT_PRIMARY
        }
        Control::SecondaryLink { .. } | Control::Button { .. } | Control::Disabled { .. } => look::CONNECT_SECONDARY,
        Control::External { .. } | Control::Link { .. } => look::CONNECT_QUIET,
    };
    control_in(b, action, style);
}
fn control_in(b: &mut Builder, action: Control<'_>, style: skein_ui::Style) {
    match action {
        Control::External { name, address } => {
            b.open(Role::Link);
            b.name(name);
            b.external(address);
            b.text(Run::Plain, name);
        }
        Control::PrimaryLink { name, to } | Control::SecondaryLink { name, to } | Control::Link { name, to } => {
            crate::bindings::link(b, name, to);
            b.text(Run::Plain, name);
        }
        Control::PrimaryButton { name, to } | Control::Button { name, to } => {
            b.open(Role::Button);
            b.name(name);
            b.bind(to);
        }
        Control::PrimaryDisabled { name } | Control::Disabled { name } => {
            b.open(Role::Button);
            b.name(name);
            b.state(State::Disabled);
        }
    }
    b.style(style);
    b.close();
}

/// A command and its Copy share the model panel's clipboard action and feedback
/// (temper/docs/design/client/pages/README.md).
pub fn command(
    b: &mut Builder,
    root: &temper_client_domain::Domain,
    w: &Words,
    bindings: &mut crate::accounts::Bindings,
    words: &[u8],
    connect: bool,
) {
    let browser = w.shell == crate::words::Shell::Browser;
    if browser {
        b.open(Role::Slot);
        b.style(if connect { look::CONNECT_CMD } else { look::WELCOME_CMD_BOX });
    }
    b.open(Role::Code);
    if !connect {
        b.style(look::WELCOME_CMD);
    }
    b.text(Run::Plain, words);
    b.close();
    if browser {
        let word = match root.copy_label() {
            temper_client_domain::copy::Label::Copy => Word::WelcomeCopy,
            temper_client_domain::copy::Label::Copied => Word::WelcomeCopied,
            temper_client_domain::copy::Label::NotCopied => Word::WelcomeNotCopied,
        };
        if let Some(to) = bindings.insert(crate::accounts::Meaning::Copy { words: skein_lib::bytes::copy_of(words) }) {
            b.open(Role::Button);
            if !connect {
                b.style(look::FLOW_QUIET);
                b.style(look::WELCOME_COPY);
            }
            b.name(&w.say(word, &[]));
            b.bind(to);
            b.close();
        }
        b.close();
    }
}

/// Settings' row carries progress without repeating the state after each line.
pub(crate) fn live(b: &mut Builder, lines: &[(&[u8], Mark)]) {
    b.open(Role::List);
    b.style(look::CONNECT_LIVE);
    for (line, mark) in lines {
        b.open(Role::Item);
        match mark {
            Mark::Done => {}
            Mark::UnderWay => b.state(State::Current),
            Mark::ToCome => b.style(look::CONNECT_LATER),
            Mark::Failed | Mark::Stopped => b.style(look::CONNECT_FAILED),
        }
        b.text(Run::Plain, line);
        b.close();
    }
    b.close();
}
