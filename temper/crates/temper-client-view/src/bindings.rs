//! Page bindings and shared controls (temper/docs/design/client/view-crates.md, 3.2).
use crate::look::owner;
use skein_lib::{List, Token};
use skein_ui::{Binding, Builder, Role, Run};
use temper_client_domain::{
    Address, Domain,
    pages::{Act as PageAct, switcher},
};

/// A page act carried by the browser shell rather than the domain (client/view.md, 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellEvent {
    Reload,
}

#[derive(Debug)]
enum Act {
    Page(PageAct),
    Approve(temper_client_domain::approve::Act),
    Reading,
    Reload,
}
#[derive(Debug)]
pub(crate) struct Bindings {
    pub(crate) remembered_field: Option<u64>,
    pub(crate) remembered_return: Option<u64>,
    pub(crate) model_times: crate::words::Times,
    entries: List<(Token, Act)>,
    next: u64,
    submissions: bool,
}
impl Bindings {
    pub fn new(capacity: u32) -> Self {
        Self {
            remembered_field: None,
            remembered_return: None,
            model_times: crate::words::Times::new(skein_lib::Wall::EPOCH),
            entries: List::with_capacity(capacity),
            next: 0,
            submissions: true,
        }
    }
    pub fn begin(&mut self) {
        self.remembered_return = None;
        self.entries.clear();
        self.submissions = true;
    }
    pub(crate) fn block_submissions(&mut self) {
        self.submissions = false;
    }
    pub(crate) fn insert(&mut self, act: PageAct) -> Binding {
        self.meaning(Act::Page(act))
    }
    pub(crate) fn insert_approve(&mut self, act: temper_client_domain::approve::Act) -> Binding {
        self.meaning(Act::Approve(act))
    }
    pub(crate) fn reading(&mut self) -> Binding {
        self.meaning(Act::Reading)
    }
    pub(crate) fn shell_event(&self, token: Token) -> Option<ShellEvent> {
        for (key, meaning) in &self.entries {
            if *key == token {
                return match meaning {
                    Act::Reload => Some(ShellEvent::Reload),
                    Act::Page(_) | Act::Approve(_) | Act::Reading => None,
                };
            }
        }
        None
    }
    pub(crate) fn reload(&mut self) -> Binding {
        self.meaning(Act::Reload)
    }
    fn meaning(&mut self, act: Act) -> Binding {
        self.next = self.next.checked_add(1).expect("pages binding tokens fit");
        let token = Token::new(self.next);
        self.entries.push((token, act)).expect("pages bindings fit");
        Binding { owner: owner(), token }
    }
    #[expect(clippy::too_many_lines, reason = "the binding table keeps each field input beside its page act")]
    pub fn decode(&self, token: Token, act: skein_ui::Act) -> Option<temper_client_domain::Action> {
        for (key, meaning) in &self.entries {
            if *key == token {
                let meaning = match meaning {
                    Act::Reading => {
                        return match act {
                            skein_ui::Act::Near { edge } => Some(temper_client_domain::Action::Pages(match edge {
                                skein_ui::LogEdge::Head => PageAct::Older,
                                skein_ui::LogEdge::Foot => PageAct::Newer,
                            })),
                            skein_ui::Act::Press
                            | skein_ui::Act::Input { .. }
                            | skein_ui::Act::Submit
                            | skein_ui::Act::Toggle
                            | skein_ui::Act::Drawing { .. } => None,
                        };
                    }
                    Act::Reload => return None,
                    Act::Approve(meaning) => return decode_approve(meaning, act),
                    Act::Page(meaning) => meaning,
                };
                if !self.submissions
                    && let (PageAct::StartChat | PageAct::Send, skein_ui::Act::Press)
                    | (PageAct::ChatWords { .. } | PageAct::MessageWords { .. }, skein_ui::Act::Submit) =
                        (meaning, &act)
                {
                    return None;
                }
                let meaning = match (meaning, act) {
                    (
                        PageAct::Finding(temper_client_domain::pages::findings::Act::Why { id, .. }),
                        skein_ui::Act::Input { value },
                    ) => PageAct::Finding(temper_client_domain::pages::findings::Act::Why { id: *id, words: value }),
                    (
                        PageAct::Finding(temper_client_domain::pages::findings::Act::Reason { id, .. }),
                        skein_ui::Act::Input { value },
                    ) => PageAct::Finding(temper_client_domain::pages::findings::Act::Reason { id: *id, words: value }),

                    (
                        PageAct::Remembered(temper_client_domain::pages::remembered::Act::Words { .. }),
                        skein_ui::Act::Input { value },
                    ) => PageAct::Remembered(temper_client_domain::pages::remembered::Act::Words { words: value }),
                    (
                        PageAct::Remembered(temper_client_domain::pages::remembered::Act::Words { .. }),
                        skein_ui::Act::Submit,
                    ) => PageAct::Remembered(temper_client_domain::pages::remembered::Act::Save),
                    (PageAct::Filter { filter }, skein_ui::Act::Input { value }) => {
                        let mut filter = *filter;
                        filter.goal = if value.is_empty() { None } else { Some(skein_ui::number(&value)?) };
                        PageAct::Filter { filter }
                    }
                    (
                        PageAct::HandArea(
                            act @ (temper_client_domain::pages::team::hand::Act::Area(_)
                            | temper_client_domain::pages::team::hand::Act::Role(_)
                            | temper_client_domain::pages::team::hand::Act::Stock(_)),
                        ),
                        skein_ui::Act::Toggle,
                    ) => PageAct::HandArea(act.clone()),
                    (PageAct::StartWay { template }, skein_ui::Act::Toggle) => {
                        PageAct::StartWay { template: *template }
                    }
                    (PageAct::StartAsks { always }, skein_ui::Act::Toggle) => PageAct::StartAsks { always: *always },
                    (PageAct::StartDraft { .. }, skein_ui::Act::Input { value }) => {
                        PageAct::StartDraft { words: value }
                    }
                    (PageAct::StartName { .. }, skein_ui::Act::Input { value }) => PageAct::StartName { words: value },
                    (
                        PageAct::HandArea(temper_client_domain::pages::team::hand::Act::Name(_)),
                        skein_ui::Act::Input { value },
                    ) => PageAct::HandArea(temper_client_domain::pages::team::hand::Act::Name(value)),
                    (
                        PageAct::HandArea(temper_client_domain::pages::team::hand::Act::Words(_)),
                        skein_ui::Act::Input { value },
                    ) => PageAct::HandArea(temper_client_domain::pages::team::hand::Act::Words(value)),
                    (
                        PageAct::HandArea(temper_client_domain::pages::team::hand::Act::Budget(_)),
                        skein_ui::Act::Input { value },
                    ) => PageAct::HandArea(temper_client_domain::pages::team::hand::Act::Budget(value)),
                    (PageAct::AmendWords { .. }, skein_ui::Act::Input { value }) => {
                        PageAct::AmendWords { words: value }
                    }
                    (PageAct::AmendPriority { .. }, skein_ui::Act::Input { value }) => {
                        PageAct::AmendPriority { priority: u32::try_from(skein_ui::number(&value)?).ok()? }
                    }
                    (PageAct::AmendTracked, skein_ui::Act::Toggle) => PageAct::AmendTracked,
                    (PageAct::Models(act), skein_ui::Act::Toggle) => PageAct::Models(act.clone()),
                    (
                        PageAct::Models(temper_client_domain::pages::models::Act::Name(..)),
                        skein_ui::Act::Input { value },
                    ) => PageAct::Models(temper_client_domain::pages::models::Act::Name(value)),
                    (PageAct::Models(temper_client_domain::pages::models::Act::Name(..)), skein_ui::Act::Submit) => {
                        PageAct::Models(temper_client_domain::pages::models::Act::Add)
                    }
                    (
                        PageAct::Choice(temper_client_domain::pages::choice::Act::Find { .. }),
                        skein_ui::Act::Input { value },
                    ) => PageAct::Choice(temper_client_domain::pages::choice::Act::Find { words: value }),
                    (PageAct::AskLine { .. }, skein_ui::Act::Input { value }) => PageAct::AskLine { words: value },
                    (PageAct::AskLine { words }, skein_ui::Act::Submit) => PageAct::AskWith { words: words.clone() },
                    (PageAct::TellWords { .. }, skein_ui::Act::Input { value }) => PageAct::TellWords { words: value },
                    (PageAct::ChatWords { .. }, skein_ui::Act::Input { value }) => PageAct::ChatWords { words: value },
                    (PageAct::MessageWords { .. }, skein_ui::Act::Input { value }) => {
                        PageAct::MessageWords { words: value }
                    }
                    (
                        PageAct::StopAsking(temper_client_domain::pages::stop_asking::Act::Words { .. }),
                        skein_ui::Act::Input { value },
                    ) => PageAct::StopAsking(temper_client_domain::pages::stop_asking::Act::Words { words: value }),
                    (PageAct::CardWords { object, .. }, skein_ui::Act::Input { value }) => {
                        PageAct::CardWords { object: *object, words: value }
                    }
                    (PageAct::Switcher(switcher::Act::RenameWords { .. }), skein_ui::Act::Input { value }) => {
                        PageAct::Switcher(switcher::Act::RenameWords { words: value })
                    }
                    (PageAct::Switcher(switcher::Act::NewWords { .. }), skein_ui::Act::Input { value }) => {
                        PageAct::Switcher(switcher::Act::NewWords { words: value })
                    }
                    (PageAct::Switcher(switcher::Act::RenameWords { .. }), skein_ui::Act::Submit) => {
                        PageAct::Switcher(switcher::Act::Save)
                    }
                    (PageAct::Switcher(switcher::Act::NewWords { .. }), skein_ui::Act::Submit) => {
                        PageAct::Switcher(switcher::Act::Create)
                    }
                    (PageAct::AddChoose { start }, skein_ui::Act::Toggle) => PageAct::AddChoose { start: *start },
                    (PageAct::AddWrite { field, .. }, skein_ui::Act::Input { value }) => {
                        PageAct::AddWrite { field: *field, words: value }
                    }
                    (PageAct::AddWrite { .. }, skein_ui::Act::Submit) => PageAct::AddSubmit,
                    (PageAct::DefaultName { .. }, skein_ui::Act::Input { value }) => {
                        PageAct::DefaultName { words: value }
                    }
                    (PageAct::DefaultName { .. }, skein_ui::Act::Submit) => PageAct::MakeDefault,
                    (PageAct::ChatWords { .. }, skein_ui::Act::Submit) => PageAct::StartChat,
                    (PageAct::MessageWords { .. }, skein_ui::Act::Submit) => PageAct::Send,
                    (PageAct::WayStop { project, .. }, skein_ui::Act::Drawing { data }) => {
                        PageAct::WayStop { project: *project, stop: crate::paths::pressed_stop(&data)? }
                    }
                    (meaning, skein_ui::Act::Press) => meaning.clone(),
                    _ => return None,
                };
                return Some(temper_client_domain::Action::Pages(meaning));
            }
        }
        None
    }
}
fn decode_approve(
    meaning: &temper_client_domain::approve::Act,
    act: skein_ui::Act,
) -> Option<temper_client_domain::Action> {
    use temper_client_domain::{Action, approve::Act};
    let decoded = match act {
        skein_ui::Act::Input { value } => match meaning {
            Act::Words { .. } => Act::Words { words: value },
            Act::Check | Act::Approve | Act::Refuse | Act::Type | Act::Again => return None,
        },
        skein_ui::Act::Submit => match meaning {
            Act::Words { .. } => Act::Check,
            Act::Check | Act::Approve | Act::Refuse | Act::Type | Act::Again => return None,
        },
        skein_ui::Act::Press => match meaning {
            Act::Check | Act::Approve | Act::Refuse | Act::Type | Act::Again => meaning.clone(),
            Act::Words { .. } => return None,
        },
        skein_ui::Act::Toggle | skein_ui::Act::Near { .. } | skein_ui::Act::Drawing { .. } => return None,
    };
    Some(Action::Approve(decoded))
}

/// Open a bound link with its address, leaving its contents to the caller.
pub(crate) fn link(b: &mut Builder, name: &[u8], to: Binding) {
    b.open(Role::Link);
    b.name(name);
    b.bind(to);
    b.link(to);
}

pub(crate) fn forge_mark(builder: &mut Builder, root: &Domain, words: &crate::words::Words, separated: bool) {
    use temper_client_domain::forge::listed::{self, Dot};
    let dot = match listed::dot(root.forges(), temper_client_domain::forge::connect::playground(root.forge())) {
        Dot::None => None,
        Dot::Amber => Some((crate::look::NAV_LIMITED, crate::words::Word::ForgeDotAmber)),
        Dot::Red => Some((crate::look::NAV_NEEDS, crate::words::Word::ForgeDotRed)),
    };
    // Drawn as the Accounts entry's dot: its state in words names the node.
    if let Some((style, word)) = dot {
        let said = words.say(word, &[]);
        builder.open(Role::Text);
        builder.style(style);
        builder.name(&said);
        if separated && words.shell == crate::words::Shell::Terminal {
            builder.run(Run::Plain, b" \xc2\xb7 ");
        }
        builder.run(Run::Plain, &said);
        builder.close();
    }
}

pub(crate) fn account_mark(builder: &mut Builder, root: &Domain, words: &crate::words::Words, separated: bool) {
    if let Some(mark) = crate::accounts::words::mark(root) {
        use crate::accounts::words::{Mark, Word};
        let (word, style) = match mark {
            Mark::Needs => (Word::NavNeeds, crate::look::NAV_NEEDS),
            Mark::Limited => (Word::NavLimited, crate::look::NAV_LIMITED),
        };
        let said = crate::accounts::words::say(words, word, &[]);
        builder.open(Role::Text);
        builder.style(style);
        builder.name(&said);
        if separated && words.shell == crate::words::Shell::Terminal {
            builder.run(Run::Plain, b" \xc2\xb7 ");
        }
        builder.run(Run::Plain, &said);
        builder.close();
    }
}

pub(crate) fn ask_address(root: &Domain, project: u32) -> Address {
    let task = match root.pages().chats() {
        Some(chats) => chats.open_asking(root.client(), root.pages().me(root.client()), project),
        None => None,
    };
    match task {
        Some(task) => Address::Chat { task },
        None => Address::Ask { project },
    }
}
