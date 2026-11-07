//! temper's frames and project switcher (temper/docs/design/client/README.md, 5.1 and 5.3).
use alloc::boxed::Box;
use skein_lib::{List, Token};
use skein_ui::{Binding, Builder, Region, Role, Run};
use temper_client_domain::{Address, Domain, pages::Act as PageAct};
use temper_client_domain::{ChatsFilter, GoalsFilter, pages::switcher};
#[must_use]
pub const fn owner() -> skein_ui::Owner {
    skein_ui::Owner { number: temper_client_domain::owner::Owner::Pages.asker().0 }
}
#[must_use]
pub fn link_region() -> Region {
    Region { owner: owner(), number: 0 }
}
#[derive(Debug)]
enum Act {
    Page(PageAct),
    Reading,
}
#[derive(Debug)]
pub(crate) struct Bindings {
    entries: List<(Token, Act)>,
    next: u64,
    submissions: bool,
}
impl Bindings {
    pub fn new(capacity: u32) -> Self {
        Self { entries: List::with_capacity(capacity), next: 0, submissions: true }
    }
    pub fn begin(&mut self) {
        self.entries.clear();
        self.submissions = true;
    }
    pub(crate) fn block_submissions(&mut self) {
        self.submissions = false;
    }
    pub(crate) fn insert(&mut self, act: PageAct) -> Binding {
        self.meaning(Act::Page(act))
    }
    pub(crate) fn reading(&mut self) -> Binding {
        self.meaning(Act::Reading)
    }
    fn meaning(&mut self, act: Act) -> Binding {
        self.next = self.next.checked_add(1).expect("pages binding tokens fit");
        let token = Token::new(self.next);
        self.entries.push((token, act)).expect("pages bindings fit");
        Binding { owner: owner(), token }
    }
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
                    (PageAct::Filter { filter }, skein_ui::Act::Input { value }) => {
                        let mut filter = *filter;
                        filter.goal = if value.is_empty() { None } else { Some(skein_ui::number(&value)?) };
                        PageAct::Filter { filter }
                    }
                    (PageAct::StartWay { template }, skein_ui::Act::Toggle) => {
                        PageAct::StartWay { template: *template }
                    }
                    (PageAct::StartAsks { always }, skein_ui::Act::Toggle) => PageAct::StartAsks { always: *always },
                    (PageAct::StartName { .. }, skein_ui::Act::Input { value }) => PageAct::StartName { words: value },
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
                        PageAct::Away(choice @ temper_client_domain::pages::away::Act::Kinds { .. }),
                        skein_ui::Act::Toggle,
                    ) => PageAct::Away(*choice),
                    (
                        PageAct::Choice(temper_client_domain::pages::choice::Act::Find { .. }),
                        skein_ui::Act::Input { value },
                    ) => PageAct::Choice(temper_client_domain::pages::choice::Act::Find { words: value }),
                    (PageAct::ChatWords { .. }, skein_ui::Act::Input { value }) => PageAct::ChatWords { words: value },
                    (PageAct::MessageWords { .. }, skein_ui::Act::Input { value }) => {
                        PageAct::MessageWords { words: value }
                    }
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
                    (meaning, skein_ui::Act::Press) => meaning.clone(),
                    _ => return None,
                };
                return Some(temper_client_domain::Action::Pages(meaning));
            }
        }
        None
    }
}
fn text(builder: &mut Builder, role: Role, words: &[u8], style: Option<skein_ui::Style>) {
    builder.open(role);
    if let Some(style) = style {
        builder.style(style);
    }
    builder.name(words);
    builder.text(Run::Plain, words);
    builder.close();
}
fn action(builder: &mut Builder, role: Role, words: &[u8], act: PageAct, current: bool, bindings: &mut Bindings) {
    if role == Role::Link {
        crate::link(builder, words, bindings.insert(act));
        builder.text(Run::Plain, words);
    } else {
        builder.open(role);
        builder.name(words);
        builder.bind(bindings.insert(act));
    }
    if current {
        builder.state(skein_ui::State::Current);
    }
    builder.close();
}
pub(crate) fn unavailable(root: &Domain) -> Option<crate::words::Word> {
    match root.client().link().state {
        jig_client::Link::SignedOut => Some(crate::words::Word::SignedOut),
        jig_client::Link::OutOfDate => Some(crate::words::Word::OutOfDate),
        jig_client::Link::Starting | jig_client::Link::Live | jig_client::Link::Offline => None,
    }
}
pub(crate) fn unavailable_main(builder: &mut Builder, root: &Domain, words: &crate::words::Words) {
    use crate::words::Word;
    use jig_client::NotSignedIn;
    use skein_ui::phrases::Hole;
    if root.client().link().state == jig_client::Link::SignedOut {
        match root.client().link().not_signed_in {
            Some(NotSignedIn::Full { sign_ins, earliest, link_until }) => words.text(
                builder,
                Word::SignInFull,
                &[Hole::Number(u64::from(sign_ins)), Hole::Wall(earliest), Hole::Wall(link_until)],
            ),
            Some(NotSignedIn::Busy) => words.text(builder, Word::SignInBusy, &[]),
            Some(NotSignedIn::NotReady) => words.text(builder, Word::SignInNotReady, &[]),
            None => words.text(builder, Word::SignedOutMain, &[]),
        }
    } else {
        words.text(builder, Word::OutOfDateMain, &[]);
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Place {
    Rail,
    Banner,
}
pub(crate) fn switcher(
    builder: &mut Builder,
    root: &Domain,
    words: &crate::words::Words,
    bindings: &mut Bindings,
    project: Option<u32>,
    place: Place,
    sign_in: &crate::forge_sign_in::Forges,
) {
    if let Some(me) = root.pages().me(root.client()) {
        if me.projects.is_empty() {
            text(
                builder,
                Role::Paragraph,
                &words.say(crate::words::Word::NoProjectYet, &[]),
                Some(crate::look::PROJECT_EMPTY),
            );
            new_project_form(builder, root, words, bindings);
            return;
        }
        builder.open(Role::Disclosure);
        builder.style(crate::look::PROJECTS);
        builder.name(&words.say(crate::words::Word::Projects, &[]));
        if let Some(project) = project {
            let name = crate::pages::project_name(Some(me), project).expect("open project has a name");
            builder.name(&name);
        }
        if place == Place::Rail {
            builder.open(Role::Slot);
            builder.style(crate::look::PROJECTS_MENU);
        }
        for selected in [true, false] {
            for row in &me.projects {
                if (Some(row.project) == project) == selected {
                    let name = crate::pages::project_name(Some(me), row.project).expect("project from me has a name");
                    if selected && root.pages().switcher().renaming() == Some(row.project) {
                        rename_form(builder, root, words, &name, bindings);
                        continue;
                    }
                    let split = place == Place::Rail && words.shell == crate::words::Shell::Browser;
                    if split {
                        builder.open(Role::Slot);
                        builder.style(crate::look::PROJECTS_ROW);
                    }
                    builder.open(Role::Button);
                    if place == Place::Rail {
                        project_row(builder, root, words, row.project, &name);
                    } else {
                        builder.name(&name);
                    }
                    if selected {
                        builder.state(skein_ui::State::Pressed);
                    }
                    builder.bind(bindings.insert(PageAct::Project { project: row.project }));
                    builder.close();
                    if split {
                        project_repository(builder, root, words, row.project, sign_in);
                        builder.close();
                    }
                    if selected && place == Place::Banner && row.role == jig_client::contract::HeldRole::Owner {
                        builder.open(Role::Button);
                        builder.style(crate::look::PROJECT_RENAME);
                        builder.name(&words.say(crate::words::Word::Rename, &[]));
                        builder.bind(bindings.insert(PageAct::Switcher(switcher::Act::Rename)));
                        builder.close();
                    }
                }
            }
        }
        if place == Place::Rail {
            builder.open(Role::Slot);
            builder.style(crate::look::PROJECTS_ACTS);
            for row in &me.projects {
                if Some(row.project) != project {
                    continue;
                }
                let name = crate::pages::project_name(Some(me), row.project).expect("open project has a name");
                if root.pages().switcher().renaming().is_none() && row.role == jig_client::contract::HeldRole::Owner {
                    builder.open(Role::Button);
                    builder.style(crate::look::PROJECT_RENAME);
                    builder.name(&words.say(crate::words::Word::RenameProject, &[skein_ui::phrases::Hole::Name(name)]));
                    builder.bind(bindings.insert(PageAct::Switcher(switcher::Act::Rename)));
                    builder.close();
                }
            }
        }
        new_project_form(builder, root, words, bindings);
        if place == Place::Rail {
            if let Some(project) = project {
                let name = crate::pages::project_name(Some(me), project).expect("open project has a name");
                let label = words.say(crate::words::Word::SettingsFor, &[skein_ui::phrases::Hole::Name(name)]);
                crate::link(builder, &label, bindings.insert(PageAct::Go { address: Address::Settings { project } }));
                builder.text(Run::Plain, &label);
                builder.close();
            }
            builder.close();
            builder.close();
        }
        builder.close();
    }
}
fn project_row(b: &mut Builder, root: &Domain, w: &crate::words::Words, project: u32, name: &[u8]) {
    use crate::words::{Shell, Word};
    use skein_ui::phrases::Hole;
    use temper_client_domain::forge::listed::{self, Held};
    b.name(name);
    if w.shell == Shell::Browser {
        return;
    }
    let repository = match listed::held(root.forges(), root.forge().playground(), project) {
        Held::Unknown => None,
        Held::None => Some(w.say(Word::SwitcherNone, &[])),
        Held::One { owner, name, .. } => {
            Some(w.say(Word::SwitcherRepository, &[Hole::Name(owner.into()), Hole::Name(name.into())]))
        }
        Held::Several { owner, name, more, .. } => Some(w.say(
            Word::SwitcherSeveral,
            &[Hole::Name(owner.into()), Hole::Name(name.into()), Hole::Number(u64::from(more))],
        )),
    };
    if let Some(repository) = repository {
        w.text(b, Word::SwitcherRow, &[Hole::Name(name.into()), Hole::Name(repository)]);
    }
}
fn project_repository(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    project: u32,
    sign_in: &crate::forge_sign_in::Forges,
) {
    use crate::words::Word;
    use skein_ui::phrases::Hole;
    use temper_client_domain::forge::listed::{self, Held};
    let held = listed::held(root.forges(), root.forge().playground(), project);
    if held == Held::Unknown {
        return;
    }
    b.open(Role::Text);
    b.style(crate::look::PROJECTS_REPO);
    match held {
        Held::One { forge, owner, name } | Held::Several { forge, owner, name, .. } => {
            crate::forge::repository::name(b, w, forge, sign_in, owner, name);
        }
        Held::None => w.text(b, Word::SwitcherNone, &[]),
        Held::Unknown => {}
    }
    if let Held::Several { more, .. } = held {
        b.open(Role::Text);
        b.style(crate::look::PROJECTS_MORE);
        b.run(Run::Plain, &w.say(Word::SwitcherMore, &[Hole::Number(u64::from(more))]));
        b.close();
    }
    b.close();
}
// The switcher's forms (temper/docs/design/client/README.md, 5.3).
fn rename_form(b: &mut Builder, root: &Domain, words: &crate::words::Words, project: &[u8], bindings: &mut Bindings) {
    use crate::words::Word;
    let switcher = root.pages().switcher();
    let pending = switcher.pending().is_some();
    b.open(Role::Form);
    b.style(crate::look::PROJECT_RENAME_FORM);
    b.name(&words.say(Word::RenameProject, &[skein_ui::phrases::Hole::Name(project.into())]));
    b.open(Role::TextField);
    b.state(skein_ui::State::SubmitsOnEnter);
    b.name(&words.say(Word::Rename, &[]));
    b.value(switcher.rename_words(), temper_client_domain::pages::shown(root.pages()));
    b.focus(0);
    if pending {
        b.state(skein_ui::State::Disabled);
    }
    b.bind(bindings.insert(PageAct::Switcher(switcher::Act::RenameWords { words: Box::default() })));
    b.close();
    switcher_refusal(b, words, switcher, switcher::Form::Rename);
    switcher_button(b, words, Word::Save, switcher::Act::Save, pending, bindings);
    b.open(Role::Button);
    b.name(&words.say(Word::Cancel, &[]));
    if switcher.pending() == Some(switcher::Form::Rename) {
        b.state(skein_ui::State::Disabled);
    }
    b.bind(bindings.insert(PageAct::Switcher(switcher::Act::Cancel)));
    b.shortcut(skein_ui::Shortcut::ESCAPE);
    b.close();
    b.close();
}
fn new_project_form(b: &mut Builder, root: &Domain, words: &crate::words::Words, bindings: &mut Bindings) {
    use crate::words::Word;
    let switcher = root.pages().switcher();
    let pending = switcher.pending().is_some();
    b.open(Role::Form);
    b.style(crate::look::PROJECT_NEW);
    b.name(&words.say(Word::NewProject, &[]));
    b.open(Role::TextField);
    b.state(skein_ui::State::SubmitsOnEnter);
    b.name(&words.say(Word::NewProject, &[]));
    b.placeholder(&words.say(Word::NewProjectPlaceholder, &[]));
    b.value(switcher.new_words(), temper_client_domain::pages::shown(root.pages()));
    if pending {
        b.state(skein_ui::State::Disabled);
    }
    b.bind(bindings.insert(PageAct::Switcher(switcher::Act::NewWords { words: Box::default() })));
    b.close();
    switcher_refusal(b, words, switcher, switcher::Form::New);
    switcher_button(b, words, Word::CreateProject, switcher::Act::Create, pending, bindings);
    b.close();
}
fn switcher_button(
    b: &mut Builder,
    words: &crate::words::Words,
    word: crate::words::Word,
    act: switcher::Act,
    pending: bool,
    bindings: &mut Bindings,
) {
    b.open(Role::Button);
    b.name(&words.say(word, &[]));
    if pending {
        b.state(skein_ui::State::Disabled);
    }
    b.bind(bindings.insert(PageAct::Switcher(act)));
    b.close();
}
fn switcher_refusal(b: &mut Builder, words: &crate::words::Words, switcher: &switcher::Switcher, form: switcher::Form) {
    if let Some((refused, outcome)) = switcher.refusal()
        && refused == form
    {
        crate::pages::chats::outcome_text(b, words, outcome, words.name_bytes);
    }
}
/// The welcome's line between the brand and the steps: what temper box is
/// for, and where running temper on a server is told
/// (`temper/docs/design/client/README.md`, 3.5).
fn box_line(builder: &mut Builder, words: &crate::words::Words) {
    use crate::words::Word;
    let name = words.say(Word::WelcomeBoxName, &[]);
    let line = words.say(Word::WelcomeBoxLine, &[]);
    let server = words.say(Word::WelcomeBoxServer, &[]);
    builder.open(Role::Slot);
    builder.style(crate::look::WELCOME_BOX_LINE);
    builder.open(Role::Text);
    builder.run(Run::Strong, &name);
    builder.run(Run::Plain, &line);
    builder.close();
    crate::flow::control(
        builder,
        crate::flow::Control::External { name: &server, address: crate::forge::connect::DEPLOYMENT },
    );
    builder.close();
}
pub(crate) fn banner(
    builder: &mut Builder,
    root: &Domain,
    words: &crate::words::Words,
    bindings: &mut Bindings,
    forge: &mut crate::forge::bindings::Bindings,
) {
    let pages = root.pages();
    let me = pages.me(root.client());
    builder.open(Role::Banner);
    builder.style(crate::look::BANNER);
    let (brand, address) = if let Address::Welcome { .. } = pages.address() {
        (crate::words::Word::Brand, Address::Welcome { step: temper_client_domain::Step::Welcome })
    } else {
        (crate::words::Word::BrandChats, Address::Chats { filter: ChatsFilter::All })
    };
    crate::link(builder, &words.say(brand, &[]), bindings.insert(PageAct::Go { address }));
    builder.style(crate::look::BRAND);
    builder.text(Run::Plain, &words.say(crate::words::Word::Brand, &[]));
    builder.close();
    if let Address::Welcome { step } = pages.address()
        && unavailable(root).is_none()
    {
        if let Some(deployment) = root.forge().deployment()
            && deployment.welcome
        {
            box_line(builder, words);
            crate::pages::welcome::steps(builder, root, words, bindings, *step);
        }
        builder.close();
        return;
    }
    builder.open(Role::Disclosure);
    builder.name(&words.say(crate::words::Word::Menu, &[]));
    builder.style(crate::look::MENU);
    if let Some(word) = unavailable(root) {
        builder.open(Role::Slot);
        builder.style(crate::look::CORNER);
        builder.open(Role::Region);
        builder.region(link_region());
        text(builder, Role::Status, &words.say(word, &[]), None);
        builder.close();
        builder.close();
        builder.close();
        builder.close();
        return;
    }
    switcher(builder, root, words, bindings, pages.project(), Place::Banner, &crate::forge_sign_in::Forges::new());
    navigation(builder, root, words, bindings);
    builder.open(Role::Slot);
    builder.style(crate::look::CORNER);
    if root.forge().playground_running() && !root.forge().leaving() {
        let name = words.say(crate::words::Word::BannerSetUp, &[]);
        crate::link(builder, &name, forge.insert(crate::forge::bindings::Meaning::EndPlayground));
        builder.style(crate::look::LEAVE_PLAYGROUND);
        builder.text(Run::Plain, &name);
        builder.close();
    }
    builder.open(Role::Region);
    builder.region(link_region());
    let (status, style) = match root.client().link().state {
        jig_client::Link::Offline => (crate::words::Word::Offline, crate::look::OFFLINE),
        jig_client::Link::Live if me.is_some() => (crate::words::Word::Live, crate::look::LIVE),
        jig_client::Link::SignedOut
        | jig_client::Link::Starting
        | jig_client::Link::Live
        | jig_client::Link::OutOfDate => (crate::words::Word::Connecting, crate::look::CONNECTING),
    };
    text(builder, Role::Status, &words.say(status, &[]), Some(style));
    if behind(root) {
        text(builder, Role::Status, &words.say(crate::words::Word::Behind, &[]), Some(crate::look::BEHIND));
    }
    builder.close();
    if let Some(me) = me {
        text(builder, Role::Status, me.party.name.bytes(), None);
    }
    if root.client().link().sign_out_lost {
        text(builder, Role::Alert, &words.say(crate::words::Word::SignOutLost, &[]), None);
    }
    if me.is_some() || root.client().link().sign_out_lost {
        action(builder, Role::Button, &words.say(crate::words::Word::SignOut, &[]), PageAct::SignOut, false, bindings);
    }
    builder.close();
    builder.close();
    builder.close();
}
fn behind(root: &Domain) -> bool {
    let pages = root.pages();
    let me_behind = match root.client().document(temper_client_domain::pages::me_handle()) {
        Some(document) => document.state == jig_client::State::Behind,
        None => false,
    };
    let page_behind = match pages.inbox() {
        Some(inbox) => match root.client().document(inbox.entries) {
            Some(document) => document.state == jig_client::State::Behind,
            None => false,
        },
        None => false,
    };
    me_behind || page_behind
}

/// The project a page's frame is the rail of (temper/docs/design/client/README.md, 5.1).
pub(crate) fn rail_project(root: &Domain, words: &crate::words::Words, sign_in: &crate::sign_in::Ways) -> Option<u32> {
    if let Address::Welcome { .. } = root.pages().address() {
        return None;
    }
    if unavailable(root).is_some() || root.authorization().is_some() || crate::sign_in::offered(root, words, sign_in) {
        return None;
    }
    root.pages().me(root.client())?;
    root.pages().project()
}

/// The rail (temper/docs/design/client/README.md, 5.3).
#[expect(clippy::too_many_arguments, reason = "the rail carries page-owned forge facts beside its frame bindings")]
pub(crate) fn rail(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    bindings: &mut Bindings,
    forge: &mut crate::forge::bindings::Bindings,
    project: u32,
    times: &mut crate::words::Times,
    sign_in: &crate::forge_sign_in::Forges,
) {
    use crate::words::{Shell, Word};
    b.open(Role::Navigation);
    b.style(crate::look::RAIL);
    let name = crate::pages::project_name(root.pages().me(root.client()), project).expect("open project has a name");
    b.name(&name);
    crate::link(b, &w.say(Word::BrandHome, &[]), bindings.insert(PageAct::Go { address: Address::Home { project } }));
    b.style(crate::look::BRAND);
    w.text(b, Word::Brand, &[]);
    b.close();
    if w.shell == Shell::Browser
        && (root.pages().chat().is_some()
            || root.pages().address() == &(Address::NewChat { draft: true })
            || crate::pages::configure::reference(root).is_some())
    {
        b.open(Role::Disclosure);
        b.name(&w.say(Word::RailUnfold, &[]));
        b.state(skein_ui::State::FoldsAway);
        b.style(crate::look::RAIL_UNFOLD);
        b.close();
    }
    switcher(b, root, w, bindings, Some(project), Place::Rail, sign_in);
    if w.shell == Shell::Browser {
        away(b, root, w, bindings, times, project, crate::look::AWAY_CHIP);
    }
    b.open(Role::Slot);
    b.style(crate::look::RAIL_NAV);
    entries(b, root, w, bindings, project, shown(root, project));
    b.close();
    b.open(Role::Slot);
    b.style(crate::look::RAIL_NEW);
    crate::link(
        b,
        &w.say(Word::NewChat, &[]),
        bindings.insert(PageAct::Go { address: Address::NewChat { draft: false } }),
    );
    w.text(b, Word::NewChatLink, &[]);
    b.close();
    b.open(Role::Menu);
    b.name(&w.say(Word::RailNewMenu, &[]));
    for (name, description, draft) in [
        (Word::RailNewChat, Word::RailNewChatDescription, false),
        (Word::RailNewGoal, Word::RailNewGoalDescription, true),
    ] {
        if draft && !temper_client_domain::pages::drafts_offered(root, project) {
            continue;
        }
        b.open(Role::MenuItem);
        b.name(&w.say(name, &[]));
        b.description(&w.say(description, &[]));
        b.bind(bindings.insert(PageAct::Go { address: Address::NewChat { draft } }));
        b.close();
    }
    b.open(Role::MenuItem);
    b.style(crate::look::RAIL_NEW_ALL);
    b.name(&w.say(Word::RailNewAll, &[]));
    b.bind(bindings.insert(PageAct::Go { address: Address::Settings { project } }));
    b.close();
    b.close();
    b.close();
    b.open(Role::Slot);
    b.style(crate::look::RAIL_FOOT);
    if root.forge().playground_running() && !root.forge().leaving() {
        let name = w.say(Word::BannerSetUp, &[]);
        crate::link(b, &name, forge.insert(crate::forge::bindings::Meaning::EndPlayground));
        b.style(crate::look::LEAVE_PLAYGROUND);
        b.text(Run::Plain, &name);
        b.close();
    }
    b.open(Role::Region);
    b.region(link_region());
    let (word, style) = match root.client().link().state {
        jig_client::Link::Offline => (Word::Offline, crate::look::OFFLINE),
        jig_client::Link::Live => (Word::Live, crate::look::LIVE),
        jig_client::Link::Starting | jig_client::Link::SignedOut | jig_client::Link::OutOfDate => {
            (Word::Connecting, crate::look::CONNECTING)
        }
    };
    text(b, Role::Status, &w.say(word, &[]), Some(style));
    let behind = behind(root);
    if behind && w.shell == Shell::Terminal {
        text(b, Role::Status, &w.say(Word::Behind, &[]), Some(crate::look::BEHIND));
    }
    b.close();
    if behind && w.shell == Shell::Browser {
        text(b, Role::Status, &w.say(Word::Behind, &[]), Some(crate::look::BEHIND));
    }
    away(b, root, w, bindings, times, project, crate::look::AWAY);
    settings_link(b, root, w, bindings, project);
    person(b, root, w, bindings);
    if root.client().link().sign_out_lost {
        text(b, Role::Alert, &w.say(Word::SignOutLost, &[]), None);
    }
    b.close();
    b.close();
}

/// The rail's person: a line in the terminal, a menu in a browser
/// (temper/docs/design/client/README.md, 5.3).
fn person(b: &mut Builder, root: &Domain, w: &crate::words::Words, bindings: &mut Bindings) {
    use crate::words::{Shell, Word};
    let Some(me) = root.pages().me(root.client()) else { return };
    match w.shell {
        Shell::Terminal => text(b, Role::Status, me.party.name.bytes(), None),
        Shell::Browser => {
            b.open(Role::Slot);
            b.style(crate::look::RAIL_PERSON);
            b.open(Role::Menu);
            b.name(me.party.name.bytes());
            for (word, act) in [
                (Word::SettingsAccountsTab, PageAct::Go { address: Address::Accounts }),
                (Word::SignOut, PageAct::SignOut),
            ] {
                b.open(Role::MenuItem);
                b.name(&w.say(word, &[]));
                b.bind(bindings.insert(act));
                b.close();
            }
            b.close();
            b.close();
        }
    }
}

/// The shared switch and browser chip (temper/docs/design/client/README.md, 3.5, Away).
fn away(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    bindings: &mut Bindings,
    times: &mut crate::words::Times,
    project: u32,
    style: skein_ui::Style,
) {
    use crate::words::{TimeForm, Word};
    if !root.away_shown(project, times.now) {
        return;
    }
    let on = root.away_on(project, times.now);
    let until = match root.forge().away(project) {
        Some(row) => row.until,
        None => None,
    };
    let said = if on {
        match until {
            Some(until) => {
                times.next = [times.next, Some(until)].into_iter().flatten().min();
                let clock = times.say(w, TimeForm::Clock, until);
                w.say(Word::AwayUntil, &[skein_ui::phrases::Hole::Name(clock)])
            }
            None => w.say(Word::Away, &[]),
        }
    } else {
        w.say(if style == crate::look::AWAY_CHIP { Word::AwayChipOff } else { Word::Away }, &[])
    };
    b.open(Role::Button);
    b.style(style);
    b.name(&said);
    b.bind(bindings.insert(PageAct::Away(temper_client_domain::pages::away::Act::Toggle { project })));
    if on {
        b.state(skein_ui::State::Pressed);
    }
    if root.pages().away_busy() {
        b.state(skein_ui::State::Disabled);
    }
    if style == crate::look::AWAY {
        b.open(Role::Slot);
        b.style(crate::look::AWAY_TRACK);
        b.close();
    }
    b.close();
}

/// A phone's tab bar, the browser's alone (temper/docs/design/client/README.md, 5.3).
pub(crate) fn tabbar(b: &mut Builder, root: &Domain, w: &crate::words::Words, bindings: &mut Bindings, project: u32) {
    b.open(Role::Navigation);
    b.style(crate::look::TABBAR);
    b.name(&w.say(crate::words::Word::Sections, &[]));
    entries(b, root, w, bindings, project, Shown { changes: false, ..shown(root, project) });
    if !goals_shown(root, project) {
        settings_link(b, root, w, bindings, project);
    }
    b.close();
}

fn goals_current(root: &Domain) -> bool {
    if let Address::Goals { .. } = root.pages().address() {
        return true;
    }
    if let Some(page) = root.pages().task()
        && let Some(task) = page.document(root.client())
    {
        return page.shape == temper_client_domain::pages::task::Shape::Tracked || task.goal.is_some();
    }
    false
}

fn goals_shown(root: &Domain, project: u32) -> bool {
    let shown = match root.forge().ways(project) {
        Some(ways) => ways.goals,
        None => false,
    };
    shown || goals_current(root) || root.has_goals(project)
}

#[expect(clippy::struct_excessive_bools, reason = "the frame keeps each entry’s visibility or current state together")]
struct Shown {
    home: bool,
    inbox: bool,
    chats: bool,
    changes: bool,
}
// Navigation follows project ways and keeps the open page (client/README.md, 5.3).
fn shown(root: &Domain, project: u32) -> Shown {
    let (home, inbox, chats) = match root.pages().address() {
        Address::Home { .. } => (true, false, false),
        Address::Inbox { .. } => (false, true, false),
        Address::Chats { .. } | Address::NewChat { .. } => (false, false, true),
        Address::Chat { .. } => (false, false, !crate::pages::chat::configuring(root)),
        Address::Task { .. }
        | Address::Accounts
        | Address::ConnectModel { .. }
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::Adopt { .. }
        | Address::Changes { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Repositories { .. }
        | Address::Goals { .. }
        | Address::Configure { .. }
        | Address::Welcome { .. } => (false, false, false),
    };
    let changes = match root.forge().ways(project) {
        Some(ways) => ways.changes,
        None => false,
    } || root.pages().address() == &(Address::Changes { project });
    Shown { home, inbox, chats, changes }
}
fn entries(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    bindings: &mut Bindings,
    project: u32,
    shown: Shown,
) {
    use crate::words::Word;
    let Shown { home, inbox, chats, changes } = shown;
    action(
        b,
        Role::Link,
        &w.say(Word::RailHome, &[]),
        PageAct::Go { address: Address::Home { project } },
        home,
        bindings,
    );
    let count = root.pages().needs().count(root.client(), project);
    let name = if count.count == 0 {
        w.say(Word::NeedsYou, &[])
    } else {
        w.say(
            if count.cut { Word::NeedsYouCountCut } else { Word::NeedsYouCount },
            &[skein_ui::phrases::Hole::Number(u64::from(count.count))],
        )
    };
    crate::link(
        b,
        &name,
        bindings.insert(PageAct::Go {
            address: Address::Inbox {
                filter: temper_client_domain::address::Filter { project: Some(project), kind: None, goal: None },
            },
        }),
    );
    if inbox {
        b.state(skein_ui::State::Current);
    }
    if count.count == 0 {
        b.text(Run::Plain, &name);
    } else {
        let number = skein_lib::Decimal::of(u64::from(count.count));
        let suffix = number.as_bytes().len().checked_add(usize::from(count.cut)).expect("count suffix fits");
        let prefix = name.len().checked_sub(suffix).expect("count is the phrase suffix");
        b.text(Run::Plain, name.get(..prefix).expect("prefix fits"));
        b.open(Role::Text);
        b.style(crate::look::NAV_COUNT);
        b.run(Run::Plain, name.get(prefix..).expect("count suffix fits"));
        b.close();
    }
    b.close();
    if goals_shown(root, project) {
        action(
            b,
            Role::Link,
            &w.say(Word::Goals, &[]),
            PageAct::Go { address: Address::Goals { project, filter: GoalsFilter::All } },
            goals_current(root),
            bindings,
        );
    }
    action(
        b,
        Role::Link,
        &w.say(Word::Chats, &[]),
        PageAct::Go { address: Address::Chats { filter: ChatsFilter::All } },
        chats,
        bindings,
    );
    if changes {
        action(
            b,
            Role::Link,
            &w.say(Word::Changes, &[]),
            PageAct::Go { address: Address::Changes { project } },
            root.pages().address() == &(Address::Changes { project }),
            bindings,
        );
    }
}

fn settings_link(b: &mut Builder, root: &Domain, w: &crate::words::Words, bindings: &mut Bindings, project: u32) {
    let name = w.say(crate::words::Word::RailSettings, &[]);
    crate::link(b, &name, bindings.insert(PageAct::Go { address: Address::Settings { project } }));
    b.text(Run::Plain, &name);
    let current = match root.pages().address() {
        Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Repositories { .. }
        | Address::Configure { .. }
        | Address::Accounts
        | Address::ConnectModel { .. }
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::Adopt { .. } => true,
        Address::Chat { .. } => crate::pages::chat::configuring(root),
        Address::Goals { .. }
        | Address::Changes { .. }
        | Address::Home { .. }
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Task { .. }
        | Address::Welcome { .. } => false,
    };
    if current {
        b.state(skein_ui::State::Current);
    }
    forge_mark(b, root, w, true);
    account_mark(b, root, w, true);
    b.close();
}

pub(crate) fn navigation(builder: &mut Builder, root: &Domain, words: &crate::words::Words, bindings: &mut Bindings) {
    if unavailable(root).is_some() {
        return;
    }
    let (inbox_current, adopt_current) = match root.pages().address() {
        Address::Inbox { .. } => (true, false),
        Address::Adopt { .. } => (false, true),
        Address::Home { .. }
        | Address::Changes { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Repositories { .. }
        | Address::Goals { .. }
        | Address::Configure { .. }
        | Address::ConnectForge { .. }
        | Address::Forges
        | Address::Welcome { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::ConnectModel { .. }
        | Address::Accounts => (false, false),
    };
    let count = match root.pages().me(root.client()) {
        Some(me) => me.needs_total,
        None => 0,
    };
    let name = if count == 0 {
        words.say(crate::words::Word::InboxTitle, &[])
    } else {
        words.say(crate::words::Word::Inbox, &[skein_ui::phrases::Hole::Number(u64::from(count))])
    };
    builder.open(Role::Navigation);
    builder.name(&words.say(crate::words::Word::MainNavigation, &[]));
    builder.style(crate::look::NAVIGATION);
    crate::link(
        builder,
        &name,
        bindings.insert(PageAct::Go { address: Address::Inbox { filter: temper_client_domain::address::Filter::ALL } }),
    );
    if inbox_current {
        builder.state(skein_ui::State::Current);
    }
    if count == 0 {
        builder.text(Run::Plain, &name);
    } else {
        let number = skein_lib::Decimal::of(u64::from(count));
        let prefix = name.len().checked_sub(number.as_bytes().len()).expect("count is the phrase suffix");
        builder.text(Run::Plain, name.get(..prefix).expect("prefix fits"));
        builder.open(Role::Text);
        builder.style(crate::look::NAV_COUNT);
        builder.run(Run::Plain, number.as_bytes());
        builder.close();
    }
    builder.close();
    action(
        builder,
        Role::Link,
        &words.say(crate::words::Word::Chats, &[]),
        PageAct::Go { address: Address::Chats { filter: ChatsFilter::All } },
        match root.pages().address() {
            Address::Chats { .. } | Address::NewChat { .. } | Address::Chat { .. } => true,
            Address::ConnectForge { .. }
            | Address::Forges
            | Address::Welcome { .. }
            | Address::Inbox { .. }
            | Address::Task { .. }
            | Address::Home { .. }
            | Address::Changes { .. }
            | Address::Settings { .. }
            | Address::Models { .. }
            | Address::Builders { .. }
            | Address::Repositories { .. }
            | Address::Goals { .. }
            | Address::Configure { .. }
            | Address::Adopt { .. }
            | Address::ConnectModel { .. }
            | Address::Accounts => false,
        },
        bindings,
    );
    if let Some(project) = root.pages().project() {
        action(
            builder,
            Role::Link,
            &words.say(crate::words::Word::Adoption, &[]),
            PageAct::Go { address: Address::Adopt { project } },
            adopt_current,
            bindings,
        );
    }
    forges_link(builder, root, words, bindings);
    accounts_link(builder, root, words, bindings);
    builder.close();
}

/// The playground's reset line (temper/docs/design/client/README.md, 5.1).
pub(crate) fn playground_banner(
    b: &mut Builder,
    root: &Domain,
    words: &crate::words::Words,
    bindings: &mut crate::forge::bindings::Bindings,
) {
    use crate::words::Word;
    if unavailable(root).is_some() || (!root.forge().playground_running() && !root.forge().leaving()) {
        return;
    }
    b.open(Role::Region);
    b.style(crate::look::ACCOUNTS);
    b.name(&words.say(Word::Playground, &[]));
    b.open(Role::Paragraph);
    if root.forge().leaving() {
        words.text(b, Word::PlaygroundLeaving, &[]);
    } else {
        words.text(b, Word::PlaygroundBanner, &[]);
        b.text(Run::Plain, b" ");
        let name = words.say(Word::PlaygroundSetUp, &[]);
        crate::link(b, &name, bindings.insert(crate::forge::bindings::Meaning::EndPlayground));
        b.text(Run::Plain, &name);
        b.close();
    }
    b.close();
    b.close();
}

pub(crate) fn forges_link(builder: &mut Builder, root: &Domain, words: &crate::words::Words, bindings: &mut Bindings) {
    crate::link(
        builder,
        &words.say(crate::words::Word::ForgesTitle, &[]),
        bindings.insert(PageAct::Go { address: Address::Forges }),
    );
    let current = match root.pages().address() {
        Address::Forges | Address::ConnectForge { .. } => true,
        Address::Accounts
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Inbox { .. }
        | Address::Home { .. }
        | Address::Changes { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Repositories { .. }
        | Address::Goals { .. }
        | Address::Configure { .. }
        | Address::Adopt { .. } => false,
    };
    if current {
        builder.state(skein_ui::State::Current);
    }
    builder.text(Run::Plain, &words.say(crate::words::Word::ForgesTitle, &[]));
    forge_mark(builder, root, words, false);
    builder.close();
}

/// The accounts link and its mark (temper/docs/design/client/README.md, 5.3).
pub(crate) fn accounts_link(
    builder: &mut Builder,
    root: &Domain,
    words: &crate::words::Words,
    bindings: &mut Bindings,
) {
    let name = crate::accounts::words::say(words, crate::accounts::words::Word::Accounts, &[]);
    crate::link(builder, &name, bindings.insert(PageAct::Go { address: Address::Accounts }));
    builder.text(Run::Plain, &name);
    if match root.pages().address() {
        Address::Accounts | Address::ConnectModel { .. } => true,
        Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Welcome { .. }
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Home { .. }
        | Address::Changes { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Repositories { .. }
        | Address::Goals { .. }
        | Address::Configure { .. }
        | Address::Adopt { .. } => false,
    } {
        builder.state(skein_ui::State::Current);
    }
    account_mark(builder, root, words, false);
    builder.close();
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

/// The session's transient status and held answer's Undo (client/README.md, 3.5).
pub(crate) fn toast(
    b: &mut Builder,
    root: &Domain,
    w: &crate::words::Words,
    bindings: &mut Bindings,
    times: &mut crate::words::Times,
) {
    use crate::words::Word;
    use temper_client_domain::pages::needs::{self, Toasted};
    let session = root.pages().needs();
    let Some(toast) = session.toast() else { return };
    b.open(Role::Region);
    b.name(&w.say(Word::ToastRegion, &[]));
    let (said, undo) = match toast {
        Toasted::Answered { object, label } => {
            let label = crate::pages::ask::answered_label(w, label);
            (w.say(Word::ToastAnswered, &[skein_ui::phrases::Hole::Name(label)]), Some(*object))
        }
        Toasted::Later => (w.say(Word::ToastLater, &[]), None),
        Toasted::NotSent => (w.say(Word::ToastNotSent, &[]), None),
        Toasted::Paused => (w.say(Word::ToastPaused, &[]), None),
        Toasted::Resumed => (w.say(Word::ToastResumed, &[]), None),
        Toasted::AwayOn { project, until } => {
            let name =
                crate::pages::project_name(root.pages().me(root.client()), *project).expect("Away project has a name");
            let said = match until {
                Some(until) => {
                    let clock = times.say(w, crate::words::TimeForm::Clock, *until);
                    w.say(
                        Word::ToastAwayOnUntil,
                        &[skein_ui::phrases::Hole::Name(clock), skein_ui::phrases::Hole::Name(name)],
                    )
                }
                None => w.say(Word::ToastAwayOn, &[skein_ui::phrases::Hole::Name(name)]),
            };
            (said, None)
        }
        Toasted::AwayOff => (w.say(Word::ToastAwayOff, &[]), None),
        Toasted::AwayRefused => (w.say(Word::ToastAwayRefused, &[]), None),
        Toasted::Overturned => (w.say(Word::ToastOverturned, &[]), None),
        Toasted::OverturnEnded => (w.say(Word::ToastOverturnEnded, &[]), None),
        Toasted::OverturnAlready => (w.say(Word::ToastOverturnAlready, &[]), None),
        Toasted::Proposed { .. } | Toasted::Refused { .. } => (Box::default(), None),
    };
    b.open(Role::Status);
    b.style(crate::look::TOAST);
    match toast {
        Toasted::Proposed { to } => {
            proposed(b, w, to);
        }
        Toasted::Refused { outcome } => {
            crate::confirm::outcome_words(b, w, outcome);
            if let jig_client::Outcome::Answered(jig_client::Answered::Jig(jig_client::contract::Answer::Refused {
                refusal: jig_client::contract::Refusal::Moved { .. },
                ..
            })) = outcome
            {
                action(
                    b,
                    Role::Button,
                    &w.say(Word::LookAgain, &[]),
                    PageAct::Needs(needs::Act::Dismiss),
                    false,
                    bindings,
                );
            }
        }
        Toasted::AwayOn { .. }
        | Toasted::AwayOff
        | Toasted::AwayRefused
        | Toasted::Overturned
        | Toasted::OverturnEnded
        | Toasted::OverturnAlready
        | Toasted::Answered { .. }
        | Toasted::Later
        | Toasted::NotSent
        | Toasted::Paused
        | Toasted::Resumed => b.text(Run::Plain, &said),
    }
    if let Toasted::AwayOn { project, .. } = toast {
        action(
            b,
            Role::Button,
            &w.say(Word::AwayLimits, &[]),
            PageAct::Away(temper_client_domain::pages::away::Act::Limits { project: *project }),
            false,
            bindings,
        );
    }
    if let Some(object) = undo
        && session.held(object)
    {
        action(
            b,
            Role::Button,
            &w.say(Word::ToastUndo, &[]),
            PageAct::Needs(needs::Act::Undo { object }),
            false,
            bindings,
        );
    }
    b.close();
    b.close();
}

fn proposed(b: &mut Builder, w: &crate::words::Words, to: &jig_client::contract::Recipient) {
    use crate::{
        cards::{P, words::say},
        words::Word,
    };
    use jig_client::contract::Recipient;
    use skein_ui::phrases::Hole;
    w.text(b, Word::ToastProposed, &[]);
    match to {
        Recipient::Party(number) => b.text(Run::Plain, &say(w, P::ToPerson, &[Hole::Number(*number)])),
        Recipient::Task(number) => b.text(Run::Plain, &say(w, P::ToTask, &[Hole::Number(*number)])),
        Recipient::Role { project, role } => {
            b.text(
                Run::Plain,
                &say(w, P::ToRole, &[Hole::Number(u64::from(*role)), Hole::Number(u64::from(*project))]),
            );
        }
        Recipient::Roles { project, roles } => {
            for role in roles {
                b.text(
                    Run::Plain,
                    &say(w, P::ToRole, &[Hole::Number(u64::from(*role)), Hole::Number(u64::from(*project))]),
                );
            }
        }
    }
    w.text(b, Word::ToastProposedEnd, &[]);
}

#[cfg(test)]
mod start_goal_tests {
    use super::*;
    use crate::tests::fixtures as f;
    use jig_client::contract::Recipient;
    #[test]
    fn proposed_toast_names_the_recipient_and_waits_for_acceptance() {
        for w in [f::words(), f::browser_words()] {
            for (to, expected) in [
                (Recipient::Party(4), "Proposed to Person 4: it starts once accepted."),
                (Recipient::Task(4), "Proposed to Task 4: it starts once accepted."),
                (Recipient::Role { project: 1, role: 2 }, "Proposed to Role 2 in project 1: it starts once accepted."),
                (
                    Recipient::Roles { project: 1, roles: List::from_boxed(Box::new([2])) },
                    "Proposed to Role 2 in project 1: it starts once accepted.",
                ),
            ] {
                let mut b = Builder::new(&f::tree_limits());
                b.open(Role::Status);
                proposed(&mut b, &w, &to);
                b.close();
                let tree = b.finish();
                assert_eq!(f::text(&tree).as_ref(), expected.as_bytes());
            }
        }
    }
}
