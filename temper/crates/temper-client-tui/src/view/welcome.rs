//! The box's welcome and the browser's steps followed (temper/docs/design/client/tui/box.md).
use super::{foot, text};
use crate::{
    Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::Link;
use skein_lib::{List, Wall};
use skein_ui::{Builder, Role, Run, Style, phrases::Hole};
use temper_client_domain::{
    Address, Domain, Step,
    forge::{self, forges},
    pages::welcome,
};
use temper_client_view::words::Word as Shared;

#[derive(Debug)]
pub(crate) struct Welcome {
    pub address: Box<[u8]>,
    pub out_of_date: Option<Box<[u8]>>,
    pub unkept: Option<Box<[u8]>>,
    pub signed_in: bool,
    pub printed: Option<Box<[u8]>>,
    pub steps: List<StepLine>,
    pub notes: foot::Notes,
}
#[derive(Debug)]
pub(crate) struct StepLine {
    pub words: Box<[u8]>,
    pub style: Style,
    pub forge: Option<Box<[u8]>>,
}

pub(crate) fn shown(root: &Domain, tui: &Tui) -> bool {
    if !tui.front() {
        return false;
    }
    match root.pages().address() {
        Address::Chat { .. } => return false,
        Address::Home { .. }
        | Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Ask { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Task { .. }
        | Address::Work { .. }
        | Address::Goals { .. }
        | Address::Changes { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Team { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Repositories { .. }
        | Address::Adopt { .. } => {}
    }
    let me = if root.client().link().state == Link::SignedOut { None } else { root.pages().me(root.client()) };
    match me {
        Some(me) => me.projects.is_empty(),
        None => true,
    }
}
fn connecting(kind: forges::Kind) -> Word {
    match kind {
        forges::Kind::Forgejo => Word::ConnectingForgejo,
        forges::Kind::GitHub => Word::ConnectingGitHub,
    }
}
fn forge_word(row: &forges::Row) -> Option<Word> {
    if forge::connect::underway(row) {
        return match &row.connection {
            forges::Connection::SettingUp { step } => Some(match row.kind {
                forges::Kind::GitHub => Word::ConnectingGitHub,
                forges::Kind::Forgejo => match step {
                    Some(forges::Step::Fetch) => Word::Fetching,
                    Some(forges::Step::Prepare | forges::Step::Run | forges::Step::Source) | None => Word::SettingUp,
                },
            }),
            forges::Connection::Connecting => Some(connecting(row.kind)),
            forges::Connection::Connected { .. } => {
                if row.kind == forges::Kind::GitHub && !forges::accepted(row) {
                    Some(Word::ConnectingGitHub)
                } else {
                    Some(Word::FirstRepository)
                }
            }
            forges::Connection::NotConnected
            | forges::Connection::NotSetUp
            | forges::Connection::SetUpFailed { .. }
            | forges::Connection::Stopped { .. } => None,
        };
    }
    if forge::connect::failed(row) { Some(Word::SetUpStopped) } else { None }
}
fn forge_underway(root: &Domain, words: &Words) -> Option<Box<[u8]>> {
    let rows = match root.forges() {
        forges::State::Rows(rows) => rows,
        forges::State::Unknown | forges::State::Ended => return None,
    };
    let playground = root.forge().playground();
    for row in rows {
        if row.mode == forges::Mode::Fake {
            continue;
        }
        if let Some(playground) = playground
            && playground.forge == row.number
        {
            continue;
        }
        let word = forge_word(row)?;
        return Some(words.say(word, &[]));
    }
    None
}
#[expect(clippy::manual_map, reason = "step code reads the optional printed link without a closure")]
pub(crate) fn read(root: &Domain, tui: &Tui, words: &Words, _wall: Wall) -> Welcome {
    let at = match welcome::decision(root) {
        Some(welcome::FirstRun::Empty) => Some(Step::Welcome),
        Some(welcome::FirstRun::Unfinished { step }) => Some(step),
        Some(welcome::FirstRun::Whole) => Some(Step::Ready),
        None => None,
    };
    let marks = match at {
        Some(at) => welcome::step_marks(at, welcome::forge_kept(root), welcome::model_usable(root) == Some(true)),
        None => [welcome::StepMark { current: false, ticked: false }; 4],
    };
    let mut steps = List::with_capacity(4);
    for ((number, label), mark) in [
        (1, Shared::WelcomeStep),
        (2, Shared::WelcomeForgeStep),
        (3, Shared::WelcomeModelStep),
        (4, Shared::WelcomeReadyStep),
    ]
    .into_iter()
    .zip(marks)
    {
        steps
            .push(StepLine {
                words: words.say(Word::Step, &[Hole::Number(number), Hole::Name(words.view().say(label, &[]))]),
                style: if mark.ticked {
                    look::STEP_DONE
                } else if mark.current {
                    look::STEP_NOW
                } else {
                    look::STEP_AHEAD
                },
                forge: if number == 2 { forge_underway(root, words) } else { None },
            })
            .expect("four steps");
    }
    Welcome {
        address: crate::place(tui.address(), true),
        out_of_date: if root.client().link().state == Link::OutOfDate {
            Some(words.view().say(Shared::OutOfDateMain, &[]))
        } else {
            None
        },
        unkept: if tui.unkept() { Some(words.say(Word::NotKept, &[])) } else { None },
        signed_in: welcome::signed_in(root),
        printed: match tui.printed() {
            Some(link) => Some(forge::words::lossy(link.expose())),
            None => None,
        },
        steps,
        notes: foot::read_notes(root, tui, words),
    }
}
fn indented(b: &mut Builder, words: &[u8], style: Option<Style>) {
    b.open(Role::Description);
    text(b, Role::Paragraph, words, style);
    b.close();
}
fn before(b: &mut Builder, part: &Welcome, words: &Words) {
    text(b, Role::Paragraph, &words.say(Word::WelcomeOpen, &[]), None);
    if let Some(link) = &part.printed {
        indented(b, link, None);
        indented(b, &words.say(Word::LinkOnce, &[]), Some(look::HINT));
    }
    b.open(Role::Region);
    text(b, Role::Paragraph, &words.say(Word::InTheBrowser, &[]), Some(look::HEAD));
    b.open(Role::Description);
    b.open(Role::Paragraph);
    b.style(look::HINT);
    let mut separator = false;
    for step in &part.steps {
        if separator {
            b.text(Run::Plain, &words.say(Word::Separator, &[]));
        }
        b.text(Run::Plain, &step.words);
        separator = true;
    }
    b.close();
    b.close();
    b.close();
}
fn after(b: &mut Builder, part: &Welcome, words: &Words) {
    b.open(Role::Region);
    text(b, Role::Paragraph, &words.say(Word::Following, &[]), Some(look::HEAD));
    b.open(Role::Description);
    b.open(Role::List);
    b.open(Role::Item);
    b.style(look::STEP_DONE);
    b.text(Run::Plain, &words.say(Word::SignedInOwner, &[]));
    b.close();
    for step in &part.steps {
        b.open(Role::Item);
        b.style(step.style);
        b.text(Run::Plain, &step.words);
        if step.style == look::STEP_NOW
            && let Some(forge) = &step.forge
        {
            b.open(Role::Text);
            b.style(look::HINT);
            b.run(Run::Plain, &words.say(Word::Separator, &[]));
            b.close();
            b.open(Role::Text);
            b.style(look::HINT);
            b.run(Run::Plain, forge);
            b.close();
        }
        b.close();
    }
    b.close();
    b.close();
    b.close();
    text(b, Role::Paragraph, &words.say(Word::StepsFollowed, &[]), Some(look::HINT));
}
pub(crate) fn draw(b: &mut Builder, part: &Welcome, words: &Words) {
    b.open(Role::Main);
    b.open(Role::Region);
    b.region(look::TOP);
    b.open(Role::Heading { level: 1 });
    text(b, Role::Text, &words.say(Word::Title, &[]), Some(look::TITLE));
    text(b, Role::Text, &part.address, Some(look::ADDRESS));
    b.close();
    if let Some(line) = &part.out_of_date {
        text(b, Role::Paragraph, line, None);
    } else {
        b.open(Role::Paragraph);
        text(b, Role::Text, &words.view().say(Shared::WelcomeBoxName, &[]), Some(look::NAME));
        text(b, Role::Text, &words.say(Word::Separator, &[]), Some(look::FAINT));
        text(b, Role::Text, &words.say(Word::BoxPurpose, &[]), Some(look::HINT));
        b.close();
    }
    if let Some(line) = &part.unkept {
        text(b, Role::Alert, line, Some(look::REFUSED));
    }
    b.close();
    b.open(Role::Region);
    b.region(look::SECTIONS);
    b.open(Role::Group);
    b.style(look::SPACED);
    if part.signed_in {
        after(b, part, words);
    } else {
        before(b, part, words);
    }
    if part.notes.log.is_some() {
        b.open(Role::Group);
    } else {
        b.close();
    }
    foot::draw_notes(b, &part.notes, words);
    if part.notes.log.is_some() {
        b.close();
        b.close();
    }
    b.close();
    b.close();
}
