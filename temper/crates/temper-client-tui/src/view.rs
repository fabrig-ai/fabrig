//! The TUI's screens from the client and the TUI's state (temper/docs/design/client/tui/pages/README.md, temper/docs/design/client/tui/pages/launchpad.md and temper/docs/design/client/sign-in.md, 1.1).
use crate::{
    Browser, Event, Local, Own, Refusal, Row, Signing, Tui, approval_address, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::{ApprovalCode, Link, NotAsked, NotSignedIn, Taking};
use skein_lib::{List, Queue, Token, Wall, bytes::copy_of};
use skein_ui::phrases::Hole;
use skein_ui::{Act, Binding, Builder, EntryKind, NodeId, Patch, Role, Run, State, Style, Tree};
use temper_client_domain::{Address, Domain, asked, pages::chat::document::Mode};
use temper_client_view_words::words::{TimeForm, Word as Shared};

pub(crate) mod chat;
pub(crate) mod foot;
pub(crate) mod pane;
pub(crate) mod talk;
pub use pane::HALVES;
pub(crate) mod front;
pub(crate) mod header;
pub(crate) mod launchpad;
pub(crate) mod printed;
pub(crate) mod starting;
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
    Starting(starting::Starting),
    Welcome(welcome::Welcome),
    Launchpad(launchpad::Launchpad),
    Chat(chat::Chat),
    Place(foot::Notes),
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
    back: Option<Event>,
    reply: Box<[Event]>,
    rows: List<Row>,
    picked: Option<(u32, bool)>,
    row_event: Option<Event>,
    page: Option<Box<[u8]>>,
    focus: Option<Box<[u8]>>,
    pane: Option<(Mode, Mode)>,
    box_empty: bool,
    list_entry: bool,
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
            back: None,
            reply: Box::default(),
            rows: List::with_capacity(0),
            picked: None,
            row_event: None,
            page: None,
            focus: None,
            pane: None,
            box_empty: false,
            list_entry: false,
        }
    }
    /// The pick on the displayed launchpad, absent if the row is no longer drawn.
    #[must_use]
    pub fn picked(&self) -> Option<(Row, bool)> {
        let (index, in_list) = self.picked?;
        Some((*self.rows.get(index)?, in_list))
    }
    #[must_use]
    pub const fn list_focused(&self) -> bool {
        match self.picked {
            Some((_, in_list)) => in_list,
            None => false,
        }
    }
    /// The box's node for a key that returns from the list to edit it.
    #[must_use]
    pub fn box_node(&self) -> Option<NodeId> {
        for node in self.tree().nodes() {
            if self.tree().bound(node.id) == Some(Binding { owner: look::OWNER, token: BOX }) {
                return Some(node.id);
            }
        }
        None
    }
    #[must_use]
    pub const fn words(&self) -> &Words {
        &self.words
    }
    /// Recover the checked vocabulary for a fresh tree.
    #[must_use]
    pub fn into_words(self) -> Words {
        self.words
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
    #[cfg(test)]
    pub(crate) const fn rows(&self) -> &List<Row> {
        &self.rows
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
fn approval_line(tui: &Tui, root: &Domain, words: &Words) -> Option<(Box<[u8]>, Style)> {
    let word = match root.asked() {
        None => return None,
        Some(asked::State::Asking) => return Some((words.say(Word::ApprovalAsking, &[]), look::WAITING)),
        Some(asked::State::NotAsked(why)) => match why {
            NotAsked::Refused if tui.address().at_name() => {
                return Some((
                    words.say(Word::ApprovalNameRefused, &[Hole::Name(crate::place(tui.address(), false))]),
                    look::REFUSED,
                ));
            }
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
                NotSignedIn::Approval(Taking::Waiting | Taking::NotGranted)
                | NotSignedIn::LinkUsed
                | NotSignedIn::LinkExpired
                | NotSignedIn::LinkUnknown,
            ) => return None,
        },
    };
    Some((words.say(word, &[]), look::REFUSED))
}
fn port(origin: &crate::Origin) -> Box<[u8]> {
    let port = match origin {
        crate::Origin::Http(socket) => socket.port(),
        crate::Origin::Https { port, .. } => *port,
    };
    copy_of(skein_lib::Decimal::of(u64::from(port)).as_bytes())
}
fn approval_waiting(
    b: &mut Builder,
    tui: &Tui,
    words: &Words,
    code: ApprovalCode,
    expires: Wall,
    wall: Wall,
) -> Option<Wall> {
    let code_in_address = if tui.address().at_name() { None } else { Some(code) };
    let address = approval_address(tui.address(), code_in_address);
    if tui.address().at_name() {
        approval_name_waiting(b, tui, words, code, &address);
    } else {
        approval_loopback_waiting(b, tui, words, code, address);
    }
    approval_expiry(b, words, expires, wall)
}
fn approval_loopback_waiting(b: &mut Builder, tui: &Tui, words: &Words, code: ApprovalCode, address: Box<[u8]>) {
    match tui.browser() {
        Some(Browser::None) => {
            text(b, Role::Paragraph, &words.say(Word::ApprovalOpen, &[]), None);
            text(b, Role::Paragraph, &address, None);
            text(b, Role::Paragraph, &words.say(Word::ApprovalCheck, &[Hole::Name(copy_of(&code.shown()))]), None);
            text(
                b,
                Role::Paragraph,
                &words.say(Word::ApprovalTunnel, &[Hole::Name(port(tui.address()))]),
                Some(look::HINT),
            );
        }
        None | Some(Browser::Opening | Browser::Opened) => {
            text(b, Role::Paragraph, &words.say(Word::ApprovalTab, &[]), None);
            text(b, Role::Paragraph, &words.say(Word::ApprovalCode, &[Hole::Name(copy_of(&code.shown()))]), None);
            text(b, Role::Paragraph, &words.say(Word::ApprovalNoTab, &[Hole::Name(address)]), None);
        }
    }
}
fn clipped_line(b: &mut Builder, role: Role, bytes: &[u8]) {
    b.open(Role::Region);
    b.region(look::SIGN_IN_LINE);
    text(b, role, bytes, None);
    b.close();
}
fn approval_name_waiting(b: &mut Builder, tui: &Tui, words: &Words, code: ApprovalCode, address: &[u8]) {
    let code = Hole::Name(copy_of(&code.shown()));
    match tui.browser() {
        Some(Browser::None) => {
            text(b, Role::Paragraph, &words.say(Word::ApprovalNameOpen, &[]), None);
            clipped_line(b, Role::Paragraph, address);
            text(b, Role::Paragraph, &words.say(Word::ApprovalNameType, &[code]), None);
        }
        None | Some(Browser::Opening | Browser::Opened) => {
            text(b, Role::Paragraph, &words.say(Word::ApprovalNameTab, &[code]), None);
            text(b, Role::Paragraph, &words.say(Word::ApprovalNameNoTab, &[]), None);
            clipped_line(b, Role::Paragraph, address);
        }
    }
}
fn approval_name_one_click(
    b: &mut Builder,
    tui: &Tui,
    words: &Words,
    code: ApprovalCode,
    expires: Wall,
    wall: Wall,
) -> Option<Wall> {
    let fallback = match tui.browser() {
        Some(Browser::None) => {
            text(b, Role::Paragraph, &words.say(Word::ApprovalNameOpen, &[]), None);
            clipped_line(b, Role::Paragraph, &approval_address(tui.address(), Some(code)));
            text(b, Role::Paragraph, &words.say(Word::ApprovalClickOpen, &[]), None);
            Word::ApprovalClickOr
        }
        None | Some(Browser::Opening | Browser::Opened) => {
            text(b, Role::Paragraph, &words.say(Word::ApprovalClickTab, &[]), None);
            text(b, Role::Paragraph, &words.say(Word::ApprovalClickCode, &[Hole::Name(copy_of(&code.shown()))]), None);
            Word::ApprovalClickNoTab
        }
    };
    let tick = approval_expiry(b, words, expires, wall);
    text(b, Role::Paragraph, &words.say(fallback, &[]), None);
    tick
}
fn approval_expiry(b: &mut Builder, words: &Words, expires: Wall, wall: Wall) -> Option<Wall> {
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
    if tui.listener_opening() {
        text(b, Role::Paragraph, &words.say(Word::ApprovalAsking, &[]), Some(look::WAITING));
        return None;
    }
    if tui.pasted() {
        return None;
    }
    if let Some((line, style)) = approval_line(tui, root, words) {
        text(b, Role::Paragraph, &line, Some(style));
        return None;
    }
    match root.asked() {
        Some(asked::State::Asked { code, expires, port: Some(_) }) if tui.address().at_name() => {
            approval_name_one_click(b, tui, words, code, expires, wall)
        }
        Some(asked::State::Asked { code, expires, .. }) => approval_waiting(b, tui, words, code, expires, wall),
        None | Some(asked::State::Asking | asked::State::NotAsked(_)) => None,
    }
}
fn sign_in(b: &mut Builder, tui: &Tui, root: &Domain, words: &Words, wall: Wall, focus: u64) -> Option<Wall> {
    if tui.signed_out() {
        text(b, Role::Paragraph, &words.view().say(Shared::SignedOutMain, &[]), Some(look::DOWN));
    }
    if tui.address().at_name() {
        clipped_line(
            b,
            Role::Heading { level: 2 },
            &words.say(Word::SignIn, &[Hole::Name(crate::place(tui.address(), false))]),
        );
        let tick = approval(b, tui, root, words, wall);
        if crate::may_ask_again(tui, root) {
            b.open(Role::Button);
            b.name(&words.say(Word::AskAgain, &[]));
            b.bind(Binding { owner: look::OWNER, token: look::ASK_AGAIN });
            b.focus(focus);
            b.close();
        } else if crate::may_type_instead(tui, root) {
            b.open(Role::Button);
            b.name(&words.say(Word::TypeCode, &[]));
            b.bind(Binding { owner: look::OWNER, token: look::TYPE_CODE });
            b.focus(focus);
            b.close();
        }
        return tick;
    }
    text(b, Role::Paragraph, &words.say(Word::SignIn, &[Hole::Name(crate::place(tui.address(), false))]), None);
    let tick = approval(b, tui, root, words, wall);
    text(b, Role::Paragraph, &words.say(Word::Paste, &[]), Some(look::HINT));
    b.open(Role::TextField);
    b.name(&words.say(Word::Link, &[]));
    b.entry(EntryKind::Secret);
    b.state(State::SubmitsOnEnter);
    b.value(tui.field(), tui.written());
    b.placeholder(&words.say(Word::LinkShape, &[Hole::Name(tui.address().authority())]));
    b.bind(Binding { owner: look::OWNER, token: look::LINK });
    b.focus(tui.focus());
    b.close();
    if let Some((line, style)) = sign_in_line(tui, root, words) {
        text(b, Role::Paragraph, &line, Some(style));
    }
    tick
}
/// A build refused replaces every screen out of front (temper/docs/design/client/tui/pages/refused.md, 1).
#[must_use]
pub fn refused(root: &Domain, tui: &Tui) -> bool {
    let link = root.client().link();
    !tui.front() && (link.state == Link::OutOfDate || !link.not_served.is_empty())
}

fn refused_lines(b: &mut Builder, tui: &Tui, words: &Words) {
    let place = crate::place(tui.address(), tui.front() || tui.boxed());
    text(b, Role::Paragraph, &words.say(Word::AnotherBuild, &[Hole::Name(place.clone())]), None);
    if !tui.version().is_empty() {
        text(b, Role::Paragraph, &words.say(Word::ThisBuild, &[Hole::Name(copy_of(tui.version()))]), None);
    }
    text(b, Role::Paragraph, &words.say(Word::StartBuild, &[Hole::Name(place)]), None);
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
    view.page = page_focus(root, tui.address());
    view.focus = view.page.clone();
    let refused = refused(root, tui);
    if refused || sign_in_drawn(root, tui) {
        view.back = None;
        view.reply = Box::default();
        view.row_event = None;
        view.pane = None;
        view.rows.clear();
        view.picked = None;
        view.box_empty = false;
        view.list_entry = false;
        view.shown = view.shown.wrapping_add(1);
        view.binds.events.clear();
        let words = &view.words;
        let mut b = view.render.builder();
        b.open(Role::Page);
        b.open(Role::Main);
        b.region(look::TOP);
        text(&mut b, Role::Heading { level: 1 }, &words.say(Word::Title, &[]), Some(look::TITLE));
        view.tick = if refused {
            refused_lines(&mut b, tui, words);
            None
        } else {
            sign_in(&mut b, tui, root, words, wall, view.shown)
        };
        b.close();
        view.escape = if refused {
            let state = words.say(Word::State, &[Hole::Name(words.say(Word::BuildRefused, &[]))]);
            foot::bottom(&mut b, &state, look::DOWN, words);
            None
        } else {
            foot::draw(&mut b, &foot::read(root, tui, words, wall), words, &mut view.binds, false)
        };
        b.close();
        view.render.finish(b, view.shown, out);
    } else {
        let body = if tui.place().is_some() {
            Body::Place(foot::read_notes(root, tui, &view.words))
        } else if starting::shown(root, tui) {
            Body::Starting(starting::read(root, tui, &view.words))
        } else if welcome::shown(root, tui) {
            Body::Welcome(welcome::read(root, tui, &view.words, wall))
        } else {
            match root.pages().address() {
                Address::Chat { .. } | Address::Talk { .. } => Body::Chat(chat::read(root, tui, &view.words, wall)),
                Address::Home { .. }
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
                | Address::Waits { .. }
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
    view.tick = signed.foot.tick;
    let body_tick = match &signed.body {
        Body::Launchpad(part) => part.tick,
        Body::Place(_) | Body::Chat(_) | Body::Welcome(_) | Body::Starting(_) => None,
    };
    if let Some(tick) = body_tick {
        view.tick = Some(view.tick.unwrap_or(tick).min(tick));
    }
    view.pane = match &signed.body {
        Body::Chat(part) => match &part.pane {
            Some(pane) => Some((pane.mode, pane.next)),
            None => None,
        },
        Body::Place(_) | Body::Welcome(_) | Body::Launchpad(_) | Body::Starting(_) => None,
    };
    view.picked = None;
    view.row_event = None;
    view.focus = view.page.clone();
    view.list_entry = signed.foot.composer.is_some() && signed.foot.list.is_none();
    if let Body::Launchpad(part) = &signed.body {
        view.rows = part.rows();
        if let Some((row, in_list)) = part.picked {
            for index in 0..view.rows.len() {
                if view.rows.get(index) == Some(&row) {
                    view.picked = Some((index, in_list && view.list_entry));
                    view.row_event = Some(row_event(row, &part.address));
                    let page = match row {
                        Row::Chat { task } => Some(Address::Chat { task }),
                        Row::Goal { task } => Some(Address::Task { task }),
                        Row::Need { .. } | Row::Role { .. } | Row::Start { .. } => None,
                    };
                    if let Some(page) = page {
                        view.focus = Some(page_address(&part.address, &page));
                    }
                    break;
                }
            }
        }
    } else {
        view.rows.clear();
    }
    view.reply.clone_from(&signed.foot.reply);
    view.box_empty = match &signed.foot.composer {
        Some(composer) => composer.value.is_empty(),
        None => false,
    };
    let list_focused = view.list_focused();
    view.binds.events.clear();
    let mut b = view.render.builder();
    b.open(Role::Page);
    match &signed.body {
        Body::Place(notes) => {
            b.open(Role::Main);
            b.open(Role::Log);
            b.close();
            foot::draw_notes(&mut b, notes, &view.words);
            b.close();
        }
        Body::Starting(part) => starting::draw(&mut b, part, &view.words),
        Body::Welcome(part) => welcome::draw(&mut b, part, &view.words),
        Body::Launchpad(part) => launchpad::draw(&mut b, part, &view.words, &mut view.binds),
        Body::Chat(part) => chat::draw(&mut b, part, &view.words, &mut view.binds),
    }
    view.escape = foot::draw(&mut b, &signed.foot, &view.words, &mut view.binds, list_focused);
    view.back = back_event(signed, view.escape.as_ref());
    if let Body::Chat(_) = &signed.body
        && view.escape.is_none()
        && view.reply.is_empty()
        && signed.foot.pause
    {
        view.escape = Some(Event::Command { command: temper_client_domain::pages::terminal::Command::Pause });
    }
    if signed.foot.list.is_none() && view.picked.is_some() {
        view.escape = Some(Event::Picked { row: None });
    }
    b.close();
    view.render.finish(b, view.shown, out);
}
fn back_event(signed: &Signed, list_escape: Option<&Event>) -> Option<Event> {
    // Questions, proposals and replies stand on their own; other lists must be closed first.
    if let Some(event) = list_escape {
        let (Event::Aside { .. } | Event::ReplyAside { .. }) = event else { return None };
    }
    match &signed.body {
        Body::Place(_) => Some(Event::Back),
        Body::Chat(_) if signed.foot.talk_name.is_some() => Some(Event::Leave),
        Body::Chat(_) => Some(Event::Command { command: temper_client_domain::pages::terminal::Command::New }),
        Body::Starting(_) | Body::Welcome(_) | Body::Launchpad(_) => None,
    }
}

/// What Left at the box's start makes in the tree last built.
#[must_use]
pub fn back(view: &View, node: NodeId, caret: u32) -> Option<Event> {
    // The renderer clamps the caret to the field's value, even after a send clears it.
    if caret != 0 && !view.box_empty {
        return None;
    }
    let event = view.back.as_ref()?;
    if view.tree().find(node)?.binding == Some(Binding { owner: look::OWNER, token: BOX }) {
        Some(event.clone())
    } else {
        None
    }
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
                Some(Event::Send)
            } else {
                None
            }
        }
        Act::Press => {
            if bound.binding.token == look::ASK_AGAIN {
                return Some(Event::SignIn);
            }
            if bound.binding.token == look::TYPE_CODE {
                return Some(Event::TypeCode);
            }
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
        if screen && temper_client_view_parts::shared::offered(root, task, command) {
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
    if !temper_client_view_parts::shared::offered(root, chat.task(root.client()), Command::Goal) {
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

/// The launchpad's selection keys (temper/docs/design/client/tui/pages/launchpad.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arrow {
    Up,
    Down,
    Tab,
    BackTab,
    Open,
}

fn row_event(row: Row, origin: &crate::Origin) -> Event {
    use temper_client_domain::pages::Act;
    match row {
        Row::Chat { task } => Event::Act { act: Act::Go { address: Address::Chat { task } } },
        Row::Goal { task } => Event::Open { address: page_address(origin, &Address::Task { task }) },
        Row::Need { object } => Event::WalkFrom { object },
        Row::Role { project, task } => Event::Act { act: Act::Go { address: Address::Talk { project, task } } },
        Row::Start { command } => Event::Own { command },
    }
}

fn section(row: Row) -> u8 {
    match row {
        Row::Chat { .. } | Row::Goal { .. } => 0,
        Row::Need { .. } => 1,
        Row::Role { .. } => 2,
        Row::Start { .. } => 3,
    }
}

/// The event made on the launchpad list last built, or None for the key reader.
#[must_use]
pub fn arrow(view: &View, arrow: Arrow) -> Option<Event> {
    if !view.list_entry {
        return None;
    }
    let Some((index, true)) = view.picked else {
        return if arrow == Arrow::Up { Some(Event::Picked { row: Some(*view.rows.get(0)?) }) } else { None };
    };
    let current = section(*view.rows.get(index)?);
    let next = match arrow {
        Arrow::Up => index.saturating_sub(1),
        Arrow::Down => index.saturating_add(1),
        Arrow::Tab => {
            let mut next = index;
            for at in index.saturating_add(1)..view.rows.len() {
                if section(*view.rows.get(at)?) != current {
                    next = at;
                    break;
                }
            }
            next
        }
        Arrow::BackTab => {
            let mut first = 0;
            for at in 1..=index {
                let before = section(*view.rows.get(at.saturating_sub(1))?);
                let here = section(*view.rows.get(at)?);
                if here == current {
                    break;
                }
                if here != before {
                    first = at;
                }
            }
            first
        }
        Arrow::Open => return view.row_event.clone(),
    };
    Some(Event::Picked { row: view.rows.get(next).copied() })
}

fn page_focus(root: &Domain, origin: &crate::Origin) -> Option<Box<[u8]>> {
    if root.client().link().state == Link::SignedOut {
        return None;
    }
    let page = if let Some(chat) = root.pages().chat() {
        if let Some(talk) = chat.talk() {
            Address::Team { project: talk.project, role: Some(talk.task) }
        } else {
            match chat.goal_from(root) {
                Some((goal, _)) => Address::Task { task: goal },
                None => Address::Chat { task: chat.task },
            }
        }
    } else {
        Address::Home { project: root.pages().project()? }
    };
    Some(page_address(origin, &page))
}
fn page_address(origin: &crate::Origin, page: &Address) -> Box<[u8]> {
    crate::origin::page(origin, &temper_client_domain::address::print(page))
}
fn has_needs(root: &Domain) -> bool {
    match root.pages().inbox() {
        Some(inbox) => !root.pages().needs().needs(inbox, root.forge().approvals()).is_empty(),
        None => false,
    }
}
/// The whole web address of the displayed focus (client/tui/pages/README.md).
#[must_use]
pub fn focus(view: &View) -> Option<&[u8]> {
    view.focus.as_deref()
}
// In front alone, beside the screen under a place not yet started (client/tui/box.md, `/web`).
fn web_offered(root: &Domain, tui: &Tui) -> bool {
    tui.front()
        && tui.box_stage() == &crate::BoxStage::Running
        && !starting::shown(root, tui)
        && !sign_in_drawn(root, tui)
        && match tui.box_state() {
            crate::BoxState::GaveUp { .. } | crate::BoxState::Stopping | crate::BoxState::Stopped { .. } => false,
            crate::BoxState::Running | crate::BoxState::Starting { .. } | crate::BoxState::Restarting { .. } => true,
        }
}
/// The TUI's own commands in their offered order (client/tui/pages/README.md).
#[must_use]
pub fn offered_own(root: &Domain, tui: &Tui) -> Box<[Own]> {
    let mut commands = List::with_capacity(10);
    let web = web_offered(root, tui);
    if root.client().link().state == Link::SignedOut || welcome::shown(root, tui) {
        if web {
            commands.push(Own::Web).expect("web counted");
        }
        commands.push(Own::Theme).expect("one theme command");
        return commands.into_boxed();
    }
    if root.pages().chat().is_some() {
        commands.push(Own::Open).expect("chat commands counted");
        if web {
            commands.push(Own::Web).expect("web counted");
        }
        commands.push(Own::Tell).expect("chat commands counted");
    } else if let Some(project) = root.pages().project() {
        // Only the brief is read here; its availability does not depend on the clock.
        let brief = temper_client_domain::pages::home::facts(root, project, Wall::from_nanos(0)).brief().is_some();
        let no_list = root.pages().listing().is_none()
            && temper_client_view_parts::shared::confirmation(root).is_none()
            && !tui.quitting();
        for command in Own::ALL {
            let offered = match command {
                Own::Draft => temper_client_domain::pages::drafts_offered(root, project),
                Own::Change | Own::Ask | Own::Open | Own::Tell => true,
                Own::Talk => talk::available(root, project),
                Own::Theme => false,
                Own::Web => web,
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

/// Ctrl-o, decoded against the tree last built.
#[must_use]
pub fn pane_key(view: &View) -> Option<Event> {
    let (_shown, mode) = view.pane?;
    Some(Event::Act { act: temper_client_domain::pages::Act::Mode { mode } })
}
