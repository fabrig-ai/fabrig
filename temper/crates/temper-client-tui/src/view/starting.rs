//! The starting screen follows a box set up before (temper/docs/design/client/tui/box.md, The starting screen).
use super::{foot, front, text};
use crate::{
    BoxStage, Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::{Link, NotSignedIn};
use skein_lib::{
    List,
    bytes::{concat, copy_of},
};
use skein_ui::{Builder, Role, Run, Style};
use temper_client_domain::{Domain, forge::words::lossy};

#[derive(Debug)]
pub(crate) struct Starting {
    address: Box<[u8]>,
    plain: bool,
    version: Box<[u8]>,
    steps: List<Step>,
    failure: Option<Box<[u8]>>,
    notes: foot::Notes,
}
#[derive(Debug)]
struct Step {
    words: Box<[u8]>,
    style: Style,
    detail: Option<Box<[u8]>>,
}

pub(crate) fn shown(root: &Domain, tui: &Tui) -> bool {
    tui.front()
        && tui.set_up()
        && !tui.me_read()
        && root.pages().me(root.client()).is_none()
        && root.client().link().state != Link::OutOfDate
        && !super::sign_in_drawn(root, tui)
}
fn answered(root: &Domain) -> bool {
    let link = root.client().link();
    match link.not_signed_in {
        Some(NotSignedIn::NotReady | NotSignedIn::Busy) => false,
        Some(
            NotSignedIn::LinkUsed
            | NotSignedIn::LinkExpired
            | NotSignedIn::LinkUnknown
            | NotSignedIn::Full { .. }
            | NotSignedIn::Approval(_),
        ) => true,
        None => link.expires.is_some() || link.party.is_some(),
    }
}
fn step(words: &Words, ahead: Word, now: Word, done: Word, underway: bool, finished: bool) -> Step {
    let (word, style) = if finished {
        (done, look::STEP_DONE)
    } else if underway {
        (now, look::STEP_NOW)
    } else {
        (ahead, look::STEP_AHEAD)
    };
    Step { words: words.say(word, &[]), style, detail: None }
}
pub(crate) fn read(root: &Domain, tui: &Tui, words: &Words) -> Starting {
    let mut steps = List::with_capacity(4);
    let (box_done, failed) = match tui.box_stage() {
        BoxStage::Finding | BoxStage::Fetching { .. } | BoxStage::Starting => (false, false),
        BoxStage::Running | BoxStage::Stopping | BoxStage::Ended { .. } => (true, false),
        BoxStage::Failed { .. } => (false, true),
    };
    let stage = front::stage(tui, words);
    let failure = if failed {
        Some(stage.expect("a failed box has its failure line").0)
    } else {
        steps
            .push(if box_done {
                Step {
                    words: words.say(Word::BoxStarted, &[]),
                    style: look::STEP_DONE,
                    detail: Some(concat(&[&words.say(Word::BoxLog, &[]), &lossy(tui.box_log())])),
                }
            } else {
                Step {
                    words: stage.expect("a box not yet running has a stage line").0,
                    style: look::STEP_NOW,
                    detail: None,
                }
            })
            .expect("four starting steps");
        None
    };
    let temper_done = box_done && answered(root);
    let signed_in = temper_done && root.client().link().state == Link::Live && root.client().link().party.is_some();
    steps
        .push(step(words, Word::TemperAhead, Word::TemperStarting, Word::TemperStarted, box_done, temper_done))
        .expect("four starting steps");
    steps
        .push(step(words, Word::SignInAhead, Word::SigningIn, Word::SignedIn, temper_done, signed_in))
        .expect("four starting steps");
    steps
        .push(step(words, Word::ProjectsAhead, Word::ProjectsReading, Word::ProjectsAhead, signed_in, false))
        .expect("four starting steps");
    Starting {
        address: copy_of(tui.address()),
        plain: tui.plain(),
        version: copy_of(tui.version()),
        steps,
        failure,
        notes: foot::read_notes(root, tui, words),
    }
}
pub(crate) fn draw(b: &mut Builder, part: &Starting, words: &Words) {
    b.open(Role::Main);
    super::header::draw(b, true, part.plain, &part.version, &part.address, words);
    b.open(Role::Region);
    b.region(look::SECTIONS);
    text(b, Role::Paragraph, b"", None);
    b.open(Role::Group);
    b.style(look::SPACED);
    b.open(Role::Region);
    text(b, Role::Paragraph, &words.say(Word::StartingTemper, &[]), Some(look::HEAD));
    b.open(Role::Description);
    if let Some(failure) = &part.failure {
        text(b, Role::Paragraph, failure, Some(look::REFUSED));
        text(b, Role::Paragraph, &words.say(Word::FailedBoxHint, &[]), Some(look::HINT));
    }
    b.open(Role::List);
    for step in &part.steps {
        b.open(Role::Item);
        b.style(step.style);
        b.text(Run::Plain, &step.words);
        if let Some(detail) = &step.detail {
            b.open(Role::Text);
            b.style(look::HINT);
            b.run(Run::Plain, detail);
            b.close();
        }
        b.close();
    }
    b.close();
    b.close();
    b.close();
    if part.failure.is_none() {
        text(b, Role::Paragraph, &words.say(Word::StartingFollowed, &[]), Some(look::HINT));
    }
    if part.notes.follows_end() {
        b.open(Role::Group);
    } else {
        b.close();
    }
    foot::draw_notes(b, &part.notes, words);
    if part.notes.follows_end() {
        b.close();
        b.close();
    }
    b.close();
    b.close();
}
