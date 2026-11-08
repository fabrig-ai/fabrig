//! How work runs' heading, path and budget (temper/docs/design/client/README.md, 5.13).
use crate::{
    frame::Bindings,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{
    List,
    bytes::{concat, copy_of},
};
use skein_ui::{Builder, Role, Run, Spans, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act,
        how::{self, Budget, Column, Head, Lane, Path},
    },
};
use temper_client_protocol_wire::v1 as wire;

pub(crate) fn head(root: &Domain, w: &Words, project: u32) -> (Box<[u8]>, Box<[u8]>) {
    let change = w.say(Word::HowChange, &[]);
    match how::parts(&how::facts(root, project)).head {
        Head::Reading => (w.say(Word::SettingsHowHeading, &[Hole::Name(project_name(root, w, project))]), change),
        Head::Own => (
            w.say(Word::HowOwn, &[]),
            concat(&[&w.say(Word::HowOwnLine, &[Hole::Name(project_name(root, w, project))]), b" ", &change]),
        ),
        Head::Way(index) => {
            let ways = root.pages().stock_ways().expect("parts names a read way");
            let tabs = ways.tabs();
            let way = tabs.get(index).expect("the current way is in the read");
            (copy_of(&way.name), concat(&[&way.line, b" ", &change]))
        }
    }
}
fn project_name(root: &Domain, w: &Words, project: u32) -> Box<[u8]> {
    match super::project_name(root.pages().me(root.client()), project) {
        Some(name) => name,
        None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
    }
}
pub(crate) fn ask(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    if how::parts(&how::facts(root, project)).ask {
        super::settings::button_to(
            b,
            &w.say(Word::HowAsk, &[]),
            bindings.insert(Act::Go { address: Address::Ask { project } }),
            true,
        );
    }
}
pub(crate) fn path(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    let parts = how::parts(&how::facts(root, project));
    super::settings::section_head(b, &w.say(Word::HowPath, &[]));
    match parts.path {
        Path::Reading => {
            b.close();
            b.close();
            return;
        }
        Path::Ended => {
            b.close();
            text(b, Role::Paragraph, &w.say(Word::WaysEnded, &[]), look::WAYS_SAYS);
            b.close();
            return;
        }
        Path::Shown => {}
    }
    let ways = root.pages().stock_ways().expect("parts has a shown read");
    let tabs = ways.tabs();
    let name = match parts.head {
        Head::Way(index) => copy_of(&tabs.get(index).expect("current way is in the read").name),
        Head::Own => w.say(Word::HowItsOwn, &[]),
        Head::Reading => unreachable!("shown parts has a current or own way"),
    };
    text(b, Role::Text, &w.say(Word::HowWay, &[Hole::Name(name)]), look::HOW_WAY);
    b.close();
    super::ways::tabs(b, ways, w, bindings, project);
    if let Some(index) = ways.shown()
        && let Some(tab) = tabs.get(index)
    {
        b.open(Role::Group);
        b.name(&tab.name);
        b.style(look::WAYS_PANEL);
        if !tab.line.is_empty() {
            text(b, Role::Paragraph, &tab.line, look::WAYS_SAYS);
        }
        table(b, &tab.path, ways.open(), w, bindings, project);
        if let Some(open) = ways.open()
            && let Some(stop) = tab.path.stops().get(open)
        {
            super::ways::card(b, stop, w, bindings, project);
        }
        super::ways::act(b, ways, index, w, bindings, project);
        super::ways::refused(b, ways, w);
        b.close();
    }
    b.close();
}
fn table(
    b: &mut Builder,
    path: &wire::BranchPath,
    open: Option<u32>,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
) {
    let columns = how::lanes(path);
    b.open(Role::Slot);
    b.style(look::HOW_SCROLL);
    b.open(Role::Table);
    b.name(&w.say(Word::HowPath, &[]));
    b.style(look::HOW_PATH);
    b.open(Role::Row);
    text(b, Role::ColumnHeader, &[], look::HOW_HEAD);
    for column in &columns {
        let (index, bar) = match column {
            Column::Stop { stop, lane: _ } => (*stop, false),
            Column::Bar { stop } => (*stop, true),
        };
        let stop = path.stops().get(index).expect("lanes names a path stop");
        b.open(Role::ColumnHeader);
        b.style(look::HOW_HEAD);
        if bar {
            b.style(look::HOW_BAR);
        }
        let words = step(w, stop);
        b.name(&words);
        b.text(Run::Plain, &words);
        b.close();
    }
    b.close();
    lane(b, path, &columns, Lane::You, open, w, bindings, project);
    lane(b, path, &columns, Lane::Agents, open, w, bindings, project);
    b.open(Role::Row);
    b.open(Role::Cell);
    b.spans(Spans {
        rows: 1,
        columns: u16::try_from(columns.len().checked_add(1).expect("bounded path plus lane header"))
            .expect("wire bounds columns"),
    });
    b.style(look::HOW_LINE);
    b.text(Run::Plain, &w.say(Word::HowAuthority, &[]));
    b.close();
    b.close();
    lane(b, path, &columns, Lane::Code, open, w, bindings, project);
    b.close();
    b.close();
}
#[expect(clippy::too_many_arguments, reason = "each lane draws the same path columns with its actor and open card")]
fn lane(
    b: &mut Builder,
    path: &wire::BranchPath,
    columns: &List<Column>,
    lane: Lane,
    open: Option<u32>,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
) {
    let (word, style) = match lane {
        Lane::You => (Word::HowYou, look::HOW_PERSON),
        Lane::Agents => (Word::HowAgents, look::HOW_AGENT),
        Lane::Code => (Word::HowCode, look::HOW_CODE),
    };
    b.open(Role::Row);
    text(b, Role::RowHeader, &w.say(word, &[]), look::HOW_LANE);
    for column in columns {
        b.open(Role::Cell);
        match column {
            Column::Bar { stop: _ } => b.style(look::HOW_BAR),
            Column::Stop { stop: index, lane: actor } => {
                if *actor == lane {
                    let stop = path.stops().get(*index).expect("lanes names a path stop");
                    let words = match lane {
                        Lane::You => stop.words(),
                        Lane::Agents | Lane::Code => stop.card().name(),
                    };
                    b.open(Role::Button);
                    b.name(words);
                    if w.shell == crate::words::Shell::Terminal {
                        b.text(Run::Plain, words);
                    }
                    b.style(style);
                    b.bind(bindings.insert(Act::WayStop { project, stop: *index }));
                    if open == Some(*index) {
                        b.state(State::Current);
                    }
                    b.close();
                }
            }
        }
        b.close();
    }
    b.close();
}
fn step(w: &Words, stop: &wire::Stop) -> Box<[u8]> {
    use wire::PathStep as S;
    let word = match stop.step() {
        S::Ask => Word::HowAskStep,
        S::Draft => Word::HowDraft,
        S::Plan => Word::HowPlan,
        S::Produce | S::ProduceMany => Word::HowProduce,
        S::Check => Word::HowCheck,
        S::Review => Word::HowReview,
        S::Repair => Word::HowRepair,
        S::Approve => Word::HowApprove,
        S::Conflict => Word::HowConflict,
        S::CatchUp => Word::HowCatchUp,
        S::Resolve => Word::HowResolve,
        S::Merge => Word::HowMerge,
        S::Prepare => Word::HowPrepare,
        S::Prove => Word::HowProve,
        S::Turn => Word::HowTurn,
        S::Verify => Word::HowVerify,
        S::Branch => return copy_of(stop.words()),
    };
    w.say(word, &[])
}
pub(crate) fn budgets(b: &mut Builder, root: &Domain, w: &Words, project: u32) {
    let Some(budget) = how::parts(&how::facts(root, project)).budget else { return };
    super::settings::section_head(b, &w.say(Word::HowBudgets, &[]));
    b.close();
    b.open(Role::Slot);
    b.style(look::HOW_ROW);
    crate::cards::text(b, Role::Text, &w.say(Word::SettingsGoals, &[]));
    text(b, Role::Text, &w.say(Word::HowBudgetLine, &[]), look::SETTINGS_DETAIL);
    let (word, number) = match budget {
        Budget::Amount(number) => (Word::HowAmount, number),
        Budget::UpTo(number) => (Word::HowUpTo, number),
    };
    crate::cards::text(b, Role::Text, &w.say(word, &[Hole::Number(number)]));
    b.close();
    b.close();
}
fn text(b: &mut Builder, role: Role, words: &[u8], style: skein_ui::Style) {
    b.open(role);
    b.style(style);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
