//! temper's banner and project switcher (temper/docs/design/client/README.md, 5.1 and 5.6).
use alloc::boxed::Box;
use skein_lib::{List, Token};
use skein_ui::{Binding, Builder, Region, Role, Run};
use temper_client_domain::{Address, Domain, pages::Act};
use temper_client_domain::{ChatsFilter, pages::switcher};
#[must_use]
pub const fn owner() -> skein_ui::Owner {
    skein_ui::Owner { number: temper_client_domain::owner::Owner::Pages.asker().0 }
}
#[must_use]
pub fn link_region() -> Region {
    Region { owner: owner(), number: 0 }
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
    pub(crate) fn insert(&mut self, act: Act) -> Binding {
        self.next = self.next.checked_add(1).expect("pages binding tokens fit");
        let token = Token::new(self.next);
        self.entries.push((token, act)).expect("pages bindings fit");
        Binding { owner: owner(), token }
    }
    pub fn decode(&self, token: Token, act: skein_ui::Act) -> Option<temper_client_domain::Action> {
        for (key, meaning) in &self.entries {
            if *key == token {
                if !self.submissions
                    && let (Act::StartChat | Act::Send, skein_ui::Act::Press)
                    | (Act::ChatWords { .. } | Act::MessageWords { .. }, skein_ui::Act::Submit) = (meaning, &act)
                {
                    return None;
                }
                let meaning = match (meaning, act) {
                    (Act::Filter { filter }, skein_ui::Act::Input { value }) => {
                        let mut filter = *filter;
                        filter.goal = if value.is_empty() { None } else { Some(skein_ui::number(&value)?) };
                        Act::Filter { filter }
                    }
                    (Act::AmendWords { .. }, skein_ui::Act::Input { value }) => Act::AmendWords { words: value },
                    (Act::AmendPriority { .. }, skein_ui::Act::Input { value }) => {
                        Act::AmendPriority { priority: u32::try_from(skein_ui::number(&value)?).ok()? }
                    }
                    (Act::AmendTracked, skein_ui::Act::Toggle) => Act::AmendTracked,
                    (Act::ChatWords { .. }, skein_ui::Act::Input { value }) => Act::ChatWords { words: value },
                    (Act::MessageWords { .. }, skein_ui::Act::Input { value }) => Act::MessageWords { words: value },
                    (Act::CardWords { object, .. }, skein_ui::Act::Input { value }) => {
                        Act::CardWords { object: *object, words: value }
                    }
                    (Act::Switcher(switcher::Act::RenameWords { .. }), skein_ui::Act::Input { value }) => {
                        Act::Switcher(switcher::Act::RenameWords { words: value })
                    }
                    (Act::Switcher(switcher::Act::NewWords { .. }), skein_ui::Act::Input { value }) => {
                        Act::Switcher(switcher::Act::NewWords { words: value })
                    }
                    (Act::Switcher(switcher::Act::RenameWords { .. }), skein_ui::Act::Submit) => {
                        Act::Switcher(switcher::Act::Save)
                    }
                    (Act::Switcher(switcher::Act::NewWords { .. }), skein_ui::Act::Submit) => {
                        Act::Switcher(switcher::Act::Create)
                    }
                    (Act::ChatWords { .. }, skein_ui::Act::Submit) => Act::StartChat,
                    (Act::MessageWords { .. }, skein_ui::Act::Submit) => Act::Send,
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
fn action(builder: &mut Builder, role: Role, words: &[u8], act: Act, current: bool, bindings: &mut Bindings) {
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
pub(crate) fn switcher(
    builder: &mut Builder,
    root: &Domain,
    words: &crate::words::Words,
    bindings: &mut Bindings,
    project: Option<u32>,
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
        for selected in [true, false] {
            for row in &me.projects {
                if (Some(row.project) == project) == selected {
                    let name = crate::pages::project_name(Some(me), row.project).expect("project from me has a name");
                    if selected {
                        builder.name(&name);
                    }
                    builder.open(Role::Button);
                    builder.name(&name);
                    if selected {
                        builder.state(skein_ui::State::Pressed);
                    }
                    builder.bind(bindings.insert(Act::Project { project: row.project }));
                    builder.close();
                    if selected {
                        if root.pages().switcher().renaming() == Some(row.project) {
                            rename_form(builder, root, words, &name, bindings);
                        } else if row.role == jig_client::contract::HeldRole::Owner {
                            builder.open(Role::Button);
                            builder.style(crate::look::PROJECT_RENAME);
                            builder.name(&words.say(crate::words::Word::Rename, &[]));
                            builder.bind(bindings.insert(Act::Switcher(switcher::Act::Rename)));
                            builder.close();
                        }
                    }
                }
            }
        }
        new_project_form(builder, root, words, bindings);
        builder.close();
    }
}
// The banner's forms (temper/docs/design/client/README.md, 5.3).
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
    b.bind(bindings.insert(Act::Switcher(switcher::Act::RenameWords { words: Box::default() })));
    b.close();
    switcher_refusal(b, words, switcher, switcher::Form::Rename);
    switcher_button(b, words, Word::Save, switcher::Act::Save, pending, bindings);
    b.open(Role::Button);
    b.name(&words.say(Word::Cancel, &[]));
    if switcher.pending() == Some(switcher::Form::Rename) {
        b.state(skein_ui::State::Disabled);
    }
    b.bind(bindings.insert(Act::Switcher(switcher::Act::Cancel)));
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
    b.bind(bindings.insert(Act::Switcher(switcher::Act::NewWords { words: Box::default() })));
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
    b.bind(bindings.insert(Act::Switcher(act)));
    b.close();
}
fn switcher_refusal(b: &mut Builder, words: &crate::words::Words, switcher: &switcher::Switcher, form: switcher::Form) {
    if let Some((refused, outcome)) = switcher.refusal()
        && refused == form
    {
        crate::pages::chats::outcome_text(b, words, outcome, words.name_bytes);
    }
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
    crate::link(
        builder,
        &words.say(crate::words::Word::BrandChats, &[]),
        bindings.insert(Act::Go { address: Address::Chats { filter: ChatsFilter::All, start: false } }),
    );
    builder.style(crate::look::BRAND);
    builder.text(Run::Plain, &words.say(crate::words::Word::Brand, &[]));
    builder.close();
    if let Address::Welcome { .. } = pages.address()
        && unavailable(root).is_none()
    {
        builder.open(Role::Slot);
        builder.style(crate::look::CORNER);
        if let Some(me) = me {
            text(builder, Role::Status, me.party.name.bytes(), None);
            action(builder, Role::Button, &words.say(crate::words::Word::SignOut, &[]), Act::SignOut, false, bindings);
        }
        builder.close();
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
    switcher(builder, root, words, bindings, pages.project());
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
    if me_behind || page_behind {
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
        action(builder, Role::Button, &words.say(crate::words::Word::SignOut, &[]), Act::SignOut, false, bindings);
    }
    builder.close();
    builder.close();
    builder.close();
}
#[expect(clippy::too_many_lines, reason = "one navigation tree keeps page states and account marks together")]
pub(crate) fn navigation(builder: &mut Builder, root: &Domain, words: &crate::words::Words, bindings: &mut Bindings) {
    if unavailable(root).is_some() {
        return;
    }
    let (inbox_current, adopt_current) = match root.pages().address() {
        Address::Inbox { .. } => (true, false),
        Address::Adopt { .. } => (false, true),
        Address::ConnectForge { .. }
        | Address::Forges
        | Address::Welcome { .. }
        | Address::Chats { .. }
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
        bindings.insert(Act::Go { address: Address::Inbox { filter: temper_client_domain::address::Filter::ALL } }),
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
        Act::Go { address: Address::Chats { filter: ChatsFilter::All, start: false } },
        match root.pages().address() {
            Address::Chats { .. } | Address::Chat { .. } => true,
            Address::ConnectForge { .. }
            | Address::Forges
            | Address::Welcome { .. }
            | Address::Inbox { .. }
            | Address::Task { .. }
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
            Act::Go { address: Address::Adopt { project } },
            adopt_current,
            bindings,
        );
    }
    forges_entry(builder, root, words, bindings);
    let name = crate::accounts::words::say(words, crate::accounts::words::Word::Accounts, &[]);
    crate::link(builder, &name, bindings.insert(Act::Go { address: Address::Accounts }));
    builder.text(Run::Plain, &name);
    if match root.pages().address() {
        Address::Accounts | Address::ConnectModel { .. } => true,
        Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::Welcome { .. }
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Adopt { .. } => false,
    } {
        builder.state(skein_ui::State::Current);
    }
    if let Some(mark) = crate::accounts::words::mark(root) {
        use crate::accounts::words::{Mark, Word};
        let (word, style) = match mark {
            Mark::Needs => (Word::NavNeeds, crate::look::NAV_NEEDS),
            Mark::Limited => (Word::NavLimited, crate::look::NAV_LIMITED),
        };
        let words = crate::accounts::words::say(words, word, &[]);
        builder.open(Role::Text);
        builder.style(style);
        builder.name(&words);
        builder.run(Run::Plain, &words);
        builder.close();
    }
    builder.close();
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

fn forges_entry(builder: &mut Builder, root: &Domain, words: &crate::words::Words, bindings: &mut Bindings) {
    use temper_client_domain::forge::listed::{self, Dot};
    crate::link(
        builder,
        &words.say(crate::words::Word::ForgesTitle, &[]),
        bindings.insert(Act::Go { address: Address::Forges }),
    );
    let current = match root.pages().address() {
        Address::Forges | Address::ConnectForge { .. } => true,
        Address::Accounts
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Chats { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Inbox { .. }
        | Address::Adopt { .. } => false,
    };
    if current {
        builder.state(skein_ui::State::Current);
    }
    builder.text(Run::Plain, &words.say(crate::words::Word::ForgesTitle, &[]));
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
        builder.run(Run::Plain, &said);
        builder.close();
    }
    builder.close();
}
