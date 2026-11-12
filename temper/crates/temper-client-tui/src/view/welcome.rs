//! The box's welcome and the browser's steps followed (temper/docs/design/client/tui/box.md).
use super::{foot, front, text};
use crate::{
    BoxStage, Local, Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::Link;
use skein_lib::{
    List, Wall,
    bytes::{copy_of, utf8_prefix},
};
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
    pub stage: Option<(Box<[u8]>, Style)>,
    pub failed_hint: Option<Box<[u8]>>,
    pub last_log: Option<Box<[u8]>>,
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
fn model_detail(root: &Domain, words: &Words) -> Option<Box<[u8]>> {
    let (account, program) = temper_client_view::shared::model_account(root, words.view())?;
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
        stage: if tui.printed().is_none() { front::stage(tui, words) } else { None },
        failed_hint: if tui.printed().is_none()
            && let BoxStage::Failed { .. } = tui.box_stage()
        {
            Some(words.say(Word::FailedBoxHint, &[]))
        } else {
            None
        },
        last_log: if tui.local() == Local::Signing
            || super::landing(root)
            || root.client().link().state == Link::Starting
        {
            let mut last = None;
            for line in tui.box_log() {
                last = Some(line);
            }
            match last {
                Some(line) => Some(startup_line(line, tui.columns())),
                None => None,
            }
        } else {
            None
        },
        steps,
        notes: foot::read_notes(root, tui, words),
    }
}
#[expect(clippy::disallowed_methods, reason = "display widths read characters from the box's lossy UTF-8 line")]
fn startup_line(line: &[u8], columns: u16) -> Box<[u8]> {
    let lossy = forge::words::lossy(line);
    let bounded = utf8_prefix(&lossy, usize::try_from(crate::limits::LOG_LINE).expect("log bound fits memory"));
    let text = core::str::from_utf8(bounded).expect("lossy text is UTF-8");
    let room = if columns == 0 { 80 } else { columns }.saturating_sub(4);
    let mut width = 0_u16;
    let mut end = 0_usize;
    for character in text.chars() {
        width = width.saturating_add(u16::from(skein_ui_screen::width(character)));
        if width > room || character == '\n' || character == '\r' {
            break;
        }
        end = end.saturating_add(character.len_utf8());
    }
    copy_of(bounded.get(..end).expect("whole characters in the bounded line"))
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
    } else if let Some((line, style)) = &part.stage {
        indented(b, line, Some(*style));
        if let Some(hint) = &part.failed_hint {
            indented(b, hint, Some(look::HINT));
        }
    }
    if let Some(line) = &part.last_log {
        b.open(Role::Description);
        text(b, Role::Paragraph, line, Some(look::HINT));
        b.close();
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
