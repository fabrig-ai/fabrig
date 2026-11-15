//! The box's welcome and the browser's steps followed (temper/docs/design/client/tui/box.md).
use super::{foot, front, text};
use crate::{
    BoxStage, Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::Link;
use skein_lib::{List, Wall, bytes::copy_of};
use skein_ui::{Builder, Role, Run, Style, phrases::Hole};
use temper_client_domain::{
    Address, Domain, Step,
    forge::{self, forges},
    pages::welcome,
};
use temper_client_view_words::words::Word as Shared;

#[derive(Debug)]
pub(crate) struct Welcome {
    pub address: Box<[u8]>,
    pub plain: bool,
    pub version: Box<[u8]>,
    pub out_of_date: Option<Box<[u8]>>,
    pub unkept: Option<Box<[u8]>>,
    pub signed_in: bool,
    pub printed: Option<Box<[u8]>>,
    pub failed: Option<(Box<[u8]>, Style)>,
    pub failed_hint: Option<Box<[u8]>>,
    pub steps: List<StepLine>,
    pub notes: foot::Notes,
}
#[derive(Debug)]
pub(crate) struct StepLine {
    pub words: Box<[u8]>,
    pub style: Style,
    pub detail: Option<Box<[u8]>>,
    pub detail_style: Style,
}

pub(crate) fn shown(root: &Domain, tui: &Tui) -> bool {
    if !tui.front() || super::starting::shown(root, tui) {
        return false;
    }
    match root.pages().address() {
        Address::Chat { .. } => return false,
        Address::Home { .. }
        | Address::Talk { .. }
        | Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Ask { .. }
        | Address::Tell { .. }
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
fn model_detail(root: &Domain, words: &Words) -> Option<Box<[u8]>> {
    let (account, program) = temper_client_view_parts::shared::model_account(root, words.view())?;
    Some(if program {
        skein_lib::bytes::concat(&[&account, &words.say(Word::ThroughClaudeCode, &[])])
    } else {
        account
    })
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
                detail_style: if number == 3 { look::STEP_DONE } else { look::STEP_NOW },
                detail: if number == 2 {
                    forge_underway(root, words)
                } else if number == 3 && mark.ticked {
                    model_detail(root, words)
                } else {
                    None
                },
            })
            .expect("four steps");
    }
    Welcome {
        address: copy_of(tui.address()),
        plain: tui.plain(),
        version: copy_of(tui.version()),
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
        failed: match tui.box_stage() {
            BoxStage::Failed { .. } => front::stage(tui, words),
            BoxStage::Finding
            | BoxStage::Fetching { .. }
            | BoxStage::Starting
            | BoxStage::Running
            | BoxStage::Stopping
            | BoxStage::Ended { .. } => None,
        },
        failed_hint: if tui.printed().is_none()
            && let BoxStage::Failed { .. } = tui.box_stage()
        {
            Some(words.say(Word::FailedBoxHint, &[]))
        } else {
            None
        },
        steps,
        notes: foot::read_notes(root, tui, words),
    }
}
fn before(b: &mut Builder, part: &Welcome, words: &Words) {
    text(b, Role::Paragraph, &words.say(Word::WelcomeOpen, &[]), None);
    if let Some(link) = &part.printed {
        text(b, Role::Paragraph, link, None);
        text(b, Role::Paragraph, &words.say(Word::LinkOnce, &[]), Some(look::HINT));
    } else if let Some((line, style)) = &part.failed {
        text(b, Role::Paragraph, line, Some(*style));
        if let Some(hint) = &part.failed_hint {
            text(b, Role::Paragraph, hint, Some(look::HINT));
        }
    } else {
        text(b, Role::Paragraph, &words.say(Word::LinkPending, &[]), Some(look::HINT));
        text(b, Role::Paragraph, &words.say(Word::LinkOnce, &[]), Some(look::HINT));
    }
    b.open(Role::Paragraph);
    text(b, Role::Text, &words.say(Word::InTheBrowser, &[]), Some(look::HEAD));
    text(b, Role::Text, b"  ", None);
    b.open(Role::Text);
    b.style(look::HINT);
    let mut separator = false;
    for step in &part.steps {
        if separator {
            b.run(Run::Plain, &words.say(Word::Separator, &[]));
        }
        b.run(Run::Plain, &step.words);
        separator = true;
    }
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
        if step.style == step.detail_style
            && let Some(detail) = &step.detail
        {
            b.open(Role::Text);
            b.style(look::HINT);
            b.run(Run::Plain, &words.say(Word::Separator, &[]));
            b.close();
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
    text(b, Role::Paragraph, &words.say(Word::StepsFollowed, &[]), Some(look::HINT));
}
pub(crate) fn draw(b: &mut Builder, part: &Welcome, words: &Words) {
    b.open(Role::Main);
    super::header::draw(b, true, part.plain, &part.version, &part.address, words);
    b.open(Role::Region);
    b.region(look::SECTIONS);
    text(b, Role::Paragraph, b"", None);
    b.open(Role::Group);
    b.style(look::SPACED);
    if let Some(line) = &part.out_of_date {
        text(b, Role::Paragraph, line, None);
    }
    if let Some(line) = &part.unkept {
        text(b, Role::Alert, line, Some(look::REFUSED));
    }
    if part.signed_in {
        after(b, part, words);
    } else {
        before(b, part, words);
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
