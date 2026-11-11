//! The TUI's screens from the client and the TUI's state (temper/docs/design/client/tui/pages/README.md and temper/docs/design/client/README.md, 4.1).
use crate::{
    Browser, Event, Local, Own, Refusal, Signing, Tui, approval_address, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::{ApprovalCode, Link, NotAsked, NotSignedIn, Taking};
use skein_lib::{List, Queue, Token, Wall, bytes::copy_of};
use skein_ui::phrases::Hole;
use skein_ui::{Act, Binding, Builder, EntryKind, NodeId, Patch, Role, Run, State, Style, Tree};
use temper_client_domain::{Address, Domain, asked, pages::chat::document::Mode};
use temper_client_view::words::{TimeForm, Word as Shared};

pub(crate) mod chat;
pub(crate) mod foot;
pub(crate) mod pane;
pub use pane::HALVES;
pub(crate) mod front;
pub(crate) mod launchpad;
pub(crate) mod printed;
pub(crate) mod welcome;
pub(crate) mod work;
pub use work::work_runs;

/// The box's binding, beside the look's BOX region.
pub(crate) const BOX: Token = Token::new(3);
#[derive(Debug)]
pub(crate) struct Binds {
    events: List<Event>,
}
impl Binds {
    pub(crate) fn bind(&mut self, event: Event) -> Binding {
        let token = Token::new(u64::from(self.events.len()).checked_add(4).expect("binding token fits"));
        self.events.push(event).expect("tree binding bound");
        Binding { owner: look::OWNER, token }
    }
}
#[derive(Debug)]
pub(crate) enum Body {
    Welcome(welcome::Welcome),
    Launchpad(launchpad::Launchpad),
    Chat(chat::Chat),
}
#[derive(Debug)]
pub(crate) struct Signed {
    pub body: Body,
    pub foot: foot::Foot,
}

#[derive(Debug)]
pub struct View {
    render: skein_ui::View,
    words: Words,
    built: Option<(u64, u64)>,
    tick: Option<Wall>,
    shown: u64,
    binds: Binds,
    escape: Option<Event>,
    reply: Box<[Event]>,
    left_off: List<(u64, launchpad::LeftKind)>,
    selected_event: Option<Event>,
    authority: Box<[u8]>,
    page: Option<Box<[u8]>>,
    focus: Option<Box<[u8]>>,
    walkable: bool,
    pane: Option<(Mode, Mode)>,
    selected: Option<u64>,
    box_empty: bool,
    left_off_arrows: bool,
}
impl View {
    #[must_use]
    pub fn new(limits: &skein_ui::Limits, words: Words) -> View {
        View {
            render: skein_ui::View::new(limits),
            words,
            built: None,
            tick: None,
            shown: 0,
            binds: Binds { events: List::with_capacity(limits.bindings) },
            escape: None,
            reply: Box::default(),
            left_off: List::with_capacity(3),
            selected_event: None,
            authority: Box::default(),
            page: None,
            focus: None,
            walkable: false,
            pane: None,
            selected: None,
            box_empty: false,
            left_off_arrows: false,
        }
    }
    #[must_use]
    pub const fn words(&self) -> &Words {
        &self.words
    }
    #[must_use]
    pub fn tick(&self) -> Option<Wall> {
        self.tick
    }
    /// The pane state in the tree last built, absent when it cannot show.
    #[must_use]
    pub fn pane_mode(&self) -> Option<Mode> {
        let (mode, _next) = self.pane?;
        Some(mode)
    }
    #[must_use]
    pub fn tree(&self) -> &Tree {
        self.render.tree()
    }
}
fn text(b: &mut Builder, role: Role, bytes: &[u8], style: Option<Style>) {
    b.open(role);
    if let Some(style) = style {
        b.style(style);
    }
    b.text(Run::Plain, bytes);
    b.close();
}
fn sign_in_line(tui: &Tui, root: &Domain, words: &Words) -> Option<(Box<[u8]>, Style)> {
    match tui.signing() {
        Signing::Idle => None,
        Signing::Refused(refusal) => Some((
            match refusal {
                Refusal::NotALink => words.say(Word::NotALink, &[]),
                Refusal::OtherAddress { address } => {
                    words.say(Word::OtherAddress, &[Hole::Name(address.clone()), Hole::Name(address.clone())])
                }
            },
            look::REFUSED,
        )),
        Signing::Asked | Signing::Answered { .. } if !tui.pasted() && root.asked().is_some() => None,
        Signing::Asked => Some((words.say(Word::SigningIn, &[]), look::WAITING)),
        Signing::Answered { lost: true } => Some((words.say(Word::NoAnswer, &[]), look::WAITING)),
        Signing::Answered { lost: false } => Some(match root.client().link().not_signed_in {
            Some(NotSignedIn::Busy) => (words.say(Word::Busy, &[]), look::WAITING),
            Some(NotSignedIn::NotReady) => (words.say(Word::NotReady, &[]), look::WAITING),
            Some(NotSignedIn::LinkUsed) => (words.view().say(Shared::SignInLinkUsed, &[]), look::REFUSED),
            Some(NotSignedIn::LinkExpired) => (words.view().say(Shared::SignInLinkExpired, &[]), look::REFUSED),
            Some(NotSignedIn::LinkUnknown) => (words.view().say(Shared::SignInLinkUnknown, &[]), look::REFUSED),
            Some(NotSignedIn::Full { sign_ins, earliest, link_until }) => (
                words.view().say(
                    Shared::SignInFull,
                    &[Hole::Number(u64::from(sign_ins)), Hole::Wall(earliest), Hole::Wall(link_until)],
                ),
                look::REFUSED,
            ),
            Some(NotSignedIn::Approval(_)) | None => (words.say(Word::Refused, &[]), look::REFUSED),
        }),
    }
}
fn approval_line(root: &Domain, words: &Words) -> Option<(Box<[u8]>, Style)> {
    let word = match root.asked() {
        None => return None,
        Some(asked::State::Asking) => return Some((words.say(Word::ApprovalAsking, &[]), look::WAITING)),
        Some(asked::State::NotAsked(why)) => match why {
            NotAsked::Refused => Word::ApprovalNotAskedRefused,
            NotAsked::Busy => Word::ApprovalNotAskedBusy,
            NotAsked::NotReady | NotAsked::Lost => Word::ApprovalNotAskedDown,
        },
        Some(asked::State::Asked { .. }) => match root.client().link().not_signed_in {
            Some(NotSignedIn::Approval(Taking::Refused)) => Word::ApprovalRefused,
            Some(NotSignedIn::Approval(Taking::Expired)) => Word::ApprovalExpired,
            Some(NotSignedIn::Approval(Taking::Used)) => Word::ApprovalUsed,
            Some(NotSignedIn::Approval(Taking::Unknown)) => Word::ApprovalUnknown,
            Some(NotSignedIn::Full { sign_ins, earliest, .. }) => {
                return Some((
                    words.say(Word::ApprovalFull, &[Hole::Number(u64::from(sign_ins)), Hole::Wall(earliest)]),
                    look::REFUSED,
                ));
            }
            Some(NotSignedIn::Busy | NotSignedIn::NotReady) => {
                return Some((words.say(Word::ApprovalBusy, &[]), look::WAITING));
            }
            None
            | Some(
                NotSignedIn::Approval(Taking::Waiting)
                | NotSignedIn::LinkUsed
                | NotSignedIn::LinkExpired
                | NotSignedIn::LinkUnknown,
            ) => return None,
        },
    };
    Some((words.say(word, &[]), look::REFUSED))
}
fn port(address: &[u8]) -> &[u8] {
    let mut port = b"".as_slice();
    for (index, byte) in address.iter().enumerate() {
        if *byte == b':' {
            port = address.get(index.saturating_add(1)..).expect("colon is in the address");
        }
    }
    port
}
fn approval_waiting(
    b: &mut Builder,
    tui: &Tui,
    words: &Words,
    code: ApprovalCode,
    expires: Wall,
    wall: Wall,
) -> Option<Wall> {
    let address = approval_address(tui.address(), code);
    match tui.browser() {
        Some(Browser::None) => {
            text(b, Role::Paragraph, &words.say(Word::ApprovalOpen, &[]), None);
            text(b, Role::Paragraph, &address, None);
            text(b, Role::Paragraph, &words.say(Word::ApprovalCheck, &[Hole::Name(copy_of(&code.shown()))]), None);
            text(
                b,
                Role::Paragraph,
                &words.say(Word::ApprovalTunnel, &[Hole::Name(copy_of(port(tui.address())))]),
                Some(look::HINT),
            );
        }
        None | Some(Browser::Opening | Browser::Opened) => {
            text(b, Role::Paragraph, &words.say(Word::ApprovalTab, &[]), None);
            text(b, Role::Paragraph, &words.say(Word::ApprovalCode, &[Hole::Name(copy_of(&code.shown()))]), None);
            text(b, Role::Paragraph, &words.say(Word::ApprovalNoTab, &[Hole::Name(address)]), None);
        }
    }
    // The shared span counts up from its first wall to its second; here they
    // describe the remaining time. The countdown crosses each bound just after it.
    let (span, _) = words.view().time(TimeForm::Span, wall, expires);
    text(b, Role::Paragraph, &words.say(Word::ApprovalExpires, &[Hole::Name(span)]), Some(look::HINT));
    let remaining = expires.as_nanos().saturating_sub(wall.as_nanos());
    let unit = if remaining < 86_400_000_000_000 { 60_000_000_000 } else { 86_400_000_000_000 };
    let whole = remaining.checked_div(unit)?.checked_mul(unit)?;
    if whole == 0 {
        return None;
    }
    let next = expires.as_nanos().checked_sub(whole)?.checked_add(1)?;
    Some(Wall::from_nanos(next))
}
fn approval(b: &mut Builder, tui: &Tui, root: &Domain, words: &Words, wall: Wall) -> Option<Wall> {
    if tui.pasted() {
        return None;
    }
    if let Some((line, style)) = approval_line(root, words) {
        text(b, Role::Paragraph, &line, Some(style));
        return None;
    }
    match root.asked() {
        Some(asked::State::Asked { code, expires }) => approval_waiting(b, tui, words, code, expires, wall),
        None | Some(asked::State::Asking | asked::State::NotAsked(_)) => None,
    }
}
fn sign_in(b: &mut Builder, tui: &Tui, root: &Domain, words: &Words, wall: Wall) -> Option<Wall> {
    if tui.signed_out() {
        text(b, Role::Paragraph, &words.view().say(Shared::SignedOutMain, &[]), Some(look::DOWN));
    }
    text(b, Role::Paragraph, &words.say(Word::SignIn, &[Hole::Name(copy_of(tui.address()))]), None);
    let tick = approval(b, tui, root, words, wall);
    text(b, Role::Paragraph, &words.say(Word::Paste, &[]), Some(look::HINT));
    b.open(Role::TextField);
    b.name(&words.say(Word::Link, &[]));
    b.entry(EntryKind::Secret);
    b.state(State::SubmitsOnEnter);
    b.value(tui.field(), tui.written());
    b.placeholder(&words.say(Word::LinkShape, &[Hole::Name(copy_of(tui.address()))]));
    b.bind(Binding { owner: look::OWNER, token: look::LINK });
    b.focus(tui.focus());
    b.close();
    if let Some((line, style)) = sign_in_line(tui, root, words) {
        text(b, Role::Paragraph, &line, Some(style));
    }
    tick
}
/// The sign-in screen is drawn while signed out with no sign-in from a link under way.
#[must_use]
pub fn sign_in_drawn(root: &Domain, tui: &Tui) -> bool {
    root.client().link().state == Link::SignedOut && tui.local() != Local::Signing
}

/// The sign-in has not landed: the link is signed out, or `me` is not yet read.
#[must_use]
pub fn landing(root: &Domain) -> bool {
    root.client().link().state == Link::SignedOut || root.pages().me(root.client()).is_none()
}

/// Rebuild when the root's shown count or the TUI's revision changes, or the tick is reached.
pub fn build(view: &mut View, tui: &Tui, root: &Domain, wall: Wall, out: &mut Queue<Patch>) {
    let root_shown = temper_client_domain::shown(root);
    let revision = tui.revision();
    let ticked = match view.tick() {
        Some(tick) => wall >= tick,
        None => false,
    };
    if view.built == Some((root_shown, revision)) && !ticked {
        return;
    }
    view.authority = copy_of(tui.address());
    view.page = page_focus(root, tui.address());
    view.focus = view.page.clone();
    view.walkable = has_needs(root);
    if sign_in_drawn(root, tui) {
        view.reply = Box::default();
        view.selected_event = None;
        view.walkable = false;
        view.pane = None;
        view.left_off.clear();
        view.selected = None;
        view.box_empty = false;
        view.left_off_arrows = false;
        view.shown = view.shown.wrapping_add(1);
        view.binds.events.clear();
        let words = &view.words;
        let mut b = view.render.builder();
        b.open(Role::Page);
        b.open(Role::Main);
        b.region(look::TOP);
        text(&mut b, Role::Heading { level: 1 }, &words.say(Word::Title, &[]), Some(look::TITLE));
        view.tick = sign_in(&mut b, tui, root, words, wall);
        b.close();
        view.escape = foot::draw(&mut b, &foot::read(root, tui, words, wall), words, &mut view.binds);
        b.close();
        view.render.finish(b, view.shown, out);
    } else {
        let body = if welcome::shown(root, tui) {
            Body::Welcome(welcome::read(root, tui, &view.words, wall))
        } else {
            match root.pages().address() {
                Address::Chat { .. } => Body::Chat(chat::read(root, tui, &view.words, wall)),
                Address::Home { .. }
                | Address::AllHome
                | Address::Inbox { .. }
                | Address::Chats { .. }
                | Address::NewChat { .. }
                | Address::Ask { .. }
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
                | Address::Adopt { .. } => Body::Launchpad(launchpad::read(root, tui, &view.words, wall)),
            }
        };
        let data = Signed { body, foot: foot::read(root, tui, &view.words, wall) };
        signed(view, &data, out);
    }
    view.built = Some((root_shown, revision));
}
/// The signed-in tree from data alone (client/tui/pages/README.md).
#[expect(clippy::manual_map, reason = "step code reads optional pane facts with an explicit match")]
pub(crate) fn signed(view: &mut View, signed: &Signed, out: &mut Queue<Patch>) {
    view.shown = view.shown.wrapping_add(1);
    view.tick = match &signed.body {
        Body::Launchpad(part) => part.tick,
        Body::Chat(_) | Body::Welcome(_) => None,
    };
    view.pane = match &signed.body {
        Body::Chat(part) => match &part.pane {
            Some(pane) => Some((pane.mode, pane.next)),
            None => None,
        },
        Body::Welcome(_) | Body::Launchpad(_) => None,
    };
    view.left_off.clear();
    view.selected = None;
    view.selected_event = None;
    view.focus = view.page.clone();
    match &signed.body {
        Body::Launchpad(part) => {
            for line in &part.left_off {
                view.left_off.push((line.task, line.kind)).expect("three left-off tasks");
                if part.selected == Some(line.task) {
                    view.selected = part.selected;
                    let page = match line.kind {
                        launchpad::LeftKind::Chat { .. } => Address::Chat { task: line.task },
                        launchpad::LeftKind::Goal => Address::Task { task: line.task },
                    };
                    let address = page_address(&view.authority, &page);
                    view.selected_event = Some(match line.kind {
                        launchpad::LeftKind::Chat { .. } => {
                            Event::Act { act: temper_client_domain::pages::Act::Go { address: page } }
                        }
                        launchpad::LeftKind::Goal => Event::Open { address: address.clone() },
                    });
                    view.focus = Some(address);
                }
            }
        }
        Body::Chat(_) | Body::Welcome(_) => {}
    }
    view.reply.clone_from(&signed.foot.reply);
    view.box_empty = match &signed.foot.composer {
        Some(composer) => composer.value.is_empty(),
        None => false,
    };
    view.left_off_arrows = view.box_empty && signed.foot.list.is_none();
    view.walkable &= view.left_off_arrows
        && match &signed.body {
            Body::Launchpad(part) => part.project.is_some(),
            Body::Chat(_) | Body::Welcome(_) => false,
        };
    view.binds.events.clear();
    let mut b = view.render.builder();
    b.open(Role::Page);
    match &signed.body {
        Body::Welcome(part) => welcome::draw(&mut b, part, &view.words),
        Body::Launchpad(part) => launchpad::draw(&mut b, part, &view.words, &mut view.binds),
        Body::Chat(part) => chat::draw(&mut b, part, &view.words, &mut view.binds),
    }
    view.escape = foot::draw(&mut b, &signed.foot, &view.words, &mut view.binds);
    if signed.foot.list.is_none() && view.selected.is_some() {
        view.escape = Some(Event::LeftOff { task: None });
    }
    b.close();
    view.render.finish(b, view.shown, out);
}
/// What Escape makes in the tree last built.
#[must_use]
pub fn escape(view: &View) -> Option<Event> {
    match &view.escape {
        Some(event) => Some(event.clone()),
        None => view.reply_aside(),
    }
}
impl View {
    /// Set aside the reply on the tree last built, even behind typed words.
    #[must_use]
    pub fn reply_aside(&self) -> Option<Event> {
        self.reply.last().cloned()
    }
}

/// A reply's numbered event on the tree last built, even behind typed words.
#[must_use]
pub fn reply_number(view: &View, words: &[u8]) -> Option<Event> {
    for (index, event) in view.reply.iter().enumerate() {
        let number = skein_lib::Decimal::of(u64::try_from(index).ok()?.checked_add(1)?);
        if number.as_bytes() == words {
            return Some(event.clone());
        }
    }
    None
}
/// Decode against the displayed tree alone.
#[must_use]
pub fn event(view: &View, node: NodeId, act: Act) -> Option<Event> {
    let bound = view.render.act(node, act)?;
    if bound.binding.owner != look::OWNER {
        return None;
    }
    match bound.act {
        Act::Input { value } => {
            if bound.binding.token == look::LINK || bound.binding.token == BOX {
                Some(Event::Typed { value })
            } else {
                None
            }
        }
        Act::Submit => {
            if bound.binding.token == look::LINK {
                Some(Event::SignIn)
            } else if bound.binding.token == BOX {
                if view.box_empty
                    && let Some(event) = &view.selected_event
                {
                    Some(event.clone())
                } else {
                    Some(Event::Send)
                }
            } else {
                None
            }
        }
        Act::Press => {
            let index = u32::try_from(bound.binding.token.raw().checked_sub(4)?).ok()?;
            view.binds.events.get(index).cloned()
        }
        Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => None,
    }
}

/// The shared commands narrowed to the TUI's screen (client/tui/pages/README.md, 2, Offered).
#[must_use]
pub fn offered(root: &Domain, _words: &Words) -> Box<[temper_client_domain::pages::terminal::Command]> {
    use temper_client_domain::pages::terminal::Command;
    let task = if let Some(chat) = root.pages().chat() { chat.task(root.client()) } else { None };
    let chat = root.pages().chat().is_some();
    let project = root.pages().project().is_some();
    let picker = match (root.pages().me(root.client()), root.pages().choice().project()) {
        (Some(me), Some(project)) => match temper_client_domain::pages::choice::offer(me, project) {
            Some(models) => !models.is_empty(),
            None => false,
        },
        (Some(_), None) | (None, _) => false,
    };
    let no_delegate = match task {
        Some(task) => task.delegates.is_empty(),
        None => false,
    };
    let mut commands = List::with_capacity(13);
    for command in Command::ALL {
        let screen = match command {
            Command::Model | Command::Effort => project && picker,
            Command::Draft => match root.pages().chat() {
                Some(chat) => chat.draft.is_some() || chat.writing,
                None => false,
            },
            Command::Pause => chat,
            Command::Close | Command::StartOver => chat && no_delegate,
            Command::New | Command::Chats => project,
            Command::Help | Command::Exit => true,
            Command::Goal => goal_offered(root),
            Command::Font | Command::Theme => false,
        };
        if screen && temper_client_view::shared::offered(root, task, command) {
            commands.push(command).expect("at most Command::ALL commands");
        }
    }
    commands.into_boxed()
}

/// Whether /goal is offered on this chat (client/tui/pages/README.md, Offered).
#[must_use]
pub fn goal_offered(root: &Domain) -> bool {
    use temper_client_domain::pages::{chat::document, terminal::Command};
    let Some(chat) = root.pages().chat() else { return false };
    if !temper_client_view::shared::offered(root, chat.task(root.client()), Command::Goal) {
        return false;
    }
    let Some(draft) = &chat.draft else { return false };
    for section in document::sections(&draft.body) {
        if !section.heading.is_empty() {
            return true;
        }
    }
    false
}

/// The launchpad's selection keys (temper/docs/design/client/tui/pages/README.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arrow {
    Up,
    Down,
}

/// The event made on the tree last built, or None for the key reader.
#[must_use]
pub fn arrow(view: &View, arrow: Arrow) -> Option<Event> {
    if !view.left_off_arrows {
        return None;
    }
    let mut picked = 0;
    for index in 0..view.left_off.len() {
        let task = view.left_off.get(index).expect("bounded left-off row").0;
        if Some(task) == view.selected {
            picked = match arrow {
                Arrow::Up => index.saturating_sub(1),
                Arrow::Down => index.saturating_add(1).min(view.left_off.len().saturating_sub(1)),
            };
            break;
        }
    }
    Some(Event::LeftOff { task: Some(view.left_off.get(picked)?.0) })
}

fn page_focus(root: &Domain, authority: &[u8]) -> Option<Box<[u8]>> {
    if root.client().link().state == Link::SignedOut {
        return None;
    }
    let page = if let Some(chat) = root.pages().chat() {
        match chat.goal_from(root) {
            Some((goal, _)) => Address::Task { task: goal },
            None => Address::Chat { task: chat.task },
        }
    } else {
        Address::Home { project: root.pages().project()? }
    };
    Some(page_address(authority, &page))
}
fn page_address(authority: &[u8], page: &Address) -> Box<[u8]> {
    skein_lib::bytes::concat(&[b"http://", authority, &temper_client_domain::address::print(page)])
}
fn has_needs(root: &Domain) -> bool {
    match root.pages().inbox() {
        Some(inbox) => !root.pages().needs().needs(inbox, root.forge().approvals()).is_empty(),
        None => false,
    }
}
/// The launchpad's walk key on the tree last built (client/tui/pages/README.md).
#[must_use]
pub fn walk_key(view: &View) -> Option<Event> {
    if view.walkable { Some(Event::Own { command: Own::Needs }) } else { None }
}
/// A chat's number among the displayed left-off rows (client/tui/pages/README.md).
#[must_use]
pub fn chat_number(view: &View, number: u32) -> Option<u64> {
    for (task, kind) in &view.left_off {
        match kind {
            launchpad::LeftKind::Chat { number: held, .. } if *held == number => return Some(*task),
            launchpad::LeftKind::Chat { .. } | launchpad::LeftKind::Goal => {}
        }
    }
    None
}
/// The whole web address of the displayed focus (client/tui/pages/README.md).
#[must_use]
pub fn focus(view: &View) -> Option<&[u8]> {
    view.focus.as_deref()
}
/// The TUI's own commands in their offered order (client/tui/pages/README.md).
#[must_use]
pub fn offered_own(root: &Domain, tui: &Tui) -> Box<[Own]> {
    let mut commands = List::with_capacity(7);
    if root.client().link().state == Link::SignedOut || welcome::shown(root, tui) {
        commands.push(Own::Theme).expect("one theme command");
        return commands.into_boxed();
    }
    if root.pages().chat().is_some() {
        commands.push(Own::Open).expect("one chat command");
    } else if let Some(project) = root.pages().project() {
        // Only the brief is read here; its availability does not depend on the clock.
        let brief = temper_client_domain::pages::home::facts(root, project, Wall::from_nanos(0)).brief().is_some();
        let no_list = root.pages().listing().is_none()
            && temper_client_view::shared::confirmation(root).is_none()
            && !tui.quitting();
        for command in Own::ALL {
            let offered = match command {
                Own::Draft => temper_client_domain::pages::drafts_offered(root, project),
                Own::Change | Own::Ask | Own::Open => true,
                Own::Theme => false,
                Own::Brief => brief,
                Own::Needs => no_list && has_needs(root),
            };
            if offered {
                commands.push(command).expect("at most Own::ALL commands");
            }
        }
    }
    commands.push(Own::Theme).expect("theme counted");
    commands.into_boxed()
}

/// Ctrl-P, decoded against the tree last built.
#[must_use]
pub fn pane_key(view: &View) -> Option<Event> {
    let (_shown, mode) = view.pane?;
    Some(Event::Act { act: temper_client_domain::pages::Act::Mode { mode } })
}
