//! Two screens from the client and the TUI's state (temper/docs/design/client/README.md, 6.14).
use crate::{
    Event, Refusal, Signing, Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::{Link, NotSignedIn};
use skein_lib::{Queue, Wall, Writer, bytes::copy_of};
use skein_ui::phrases::Hole;
use skein_ui::{Act, Binding, Builder, EntryKind, Modifier, NodeId, Patch, Role, Run, Shortcut, State, Style, Tree};
use temper_client_domain::Domain;
use temper_client_view::words::Word as Shared;

#[derive(Debug)]
pub struct View {
    render: skein_ui::View,
    words: Words,
    built: Option<(u64, u64)>,
}
impl View {
    #[must_use]
    pub fn new(limits: &skein_ui::Limits, words: Words) -> View {
        View { render: skein_ui::View::new(limits), words, built: None }
    }
    #[must_use]
    pub fn tick(&self) -> Option<Wall> {
        None
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
fn sign_in(b: &mut Builder, tui: &Tui, root: &Domain, words: &Words) {
    if tui.signed_out() {
        text(b, Role::Paragraph, &words.view().say(Shared::SignedOutMain, &[]), Some(look::DOWN));
    }
    text(b, Role::Paragraph, &words.say(Word::SignIn, &[Hole::Name(copy_of(tui.address()))]), None);
    text(b, Role::Paragraph, &words.say(Word::Paste, &[]), Some(look::HINT));
    b.open(Role::TextField);
    b.name(&words.say(Word::Link, &[]));
    b.entry(EntryKind::Secret);
    b.state(State::SubmitsOnEnter);
    b.value(tui.field(), tui.written());
    b.placeholder(&words.say(Word::LinkShape, &[Hole::Name(copy_of(tui.address()))]));
    b.bind(Binding { owner: look::OWNER, token: look::LINK });
    b.focus(tui.openings());
    b.close();
    if let Some((line, style)) = sign_in_line(tui, root, words) {
        text(b, Role::Paragraph, &line, Some(style));
    }
}
fn projects(me: &jig_client::contract::Me, bound: u32) -> Box<[u8]> {
    let mut bytes = 0_u32;
    let mut count = 0_usize;
    for project in &me.projects {
        let separator = if count == 0 { 0 } else { 4 };
        let name = u32::try_from(project.name.bytes().len()).expect("domain text fits u32");
        let Some(next) = bytes.checked_add(separator) else { break };
        let Some(next) = next.checked_add(name) else { break };
        if next > bound {
            break;
        }
        bytes = next;
        count = count.saturating_add(1);
    }
    let mut out = Writer::new(skein_lib::usize_of(bytes));
    for (index, project) in me.projects.as_slice().iter().take(count).enumerate() {
        if index != 0 {
            out.put(" · ".as_bytes()).expect("separator counted");
        }
        out.put(project.name.bytes()).expect("name counted");
    }
    out.finish()
}
fn first(b: &mut Builder, tui: &Tui, root: &Domain, words: &Words) {
    if root.client().link().state == Link::OutOfDate {
        text(b, Role::Paragraph, &words.view().say(Shared::OutOfDateMain, &[]), None);
    } else if let Some(me) = root.pages().me(root.client()) {
        text(
            b,
            Role::Paragraph,
            &words.say(
                Word::Connected,
                &[Hole::Name(copy_of(tui.address())), Hole::Name(copy_of(me.party.name.bytes()))],
            ),
            None,
        );
        let line = if me.projects.is_empty() {
            words.view().say(Shared::NoProjectYet, &[])
        } else {
            words.say(Word::Projects, &[Hole::Name(projects(me, words.projects_bytes))])
        };
        text(b, Role::Paragraph, &line, None);
    } else {
        text(b, Role::Paragraph, &words.say(Word::Connecting, &[Hole::Name(copy_of(tui.address()))]), None);
    }
    if tui.unkept() {
        text(b, Role::Alert, &words.say(Word::NotKept, &[]), Some(look::REFUSED));
    }
}
fn state(root: &Domain) -> (Shared, Style) {
    match root.client().link().state {
        Link::Live => {
            if root.pages().me(root.client()).is_some() {
                (Shared::Live, look::LIVE)
            } else {
                (Shared::Connecting, look::WAITING)
            }
        }
        Link::Starting => (Shared::Connecting, look::WAITING),
        Link::Offline => (Shared::Offline, look::DOWN),
        Link::OutOfDate => (Shared::OutOfDate, look::DOWN),
        Link::SignedOut => (Shared::SignedOut, look::DOWN),
    }
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
    let shown = root_shown.saturating_add(revision);
    let words = &view.words;
    let mut b = view.render.builder();
    b.open(Role::Page);
    b.open(Role::Main);
    b.region(look::TOP);
    text(&mut b, Role::Heading { level: 1 }, &words.say(Word::Title, &[]), Some(look::TITLE));
    let signed_out = root.client().link().state == Link::SignedOut;
    if signed_out {
        sign_in(&mut b, tui, root, words);
    } else {
        first(&mut b, tui, root, words);
    }
    b.close();
    b.open(Role::ContentInfo);
    b.region(look::BOTTOM);
    if signed_out {
        text(&mut b, Role::Paragraph, &words.say(Word::QuitKeys, &[]), Some(look::HINT));
    } else {
        b.open(Role::Button);
        b.name(&words.say(Word::Quit, &[]));
        b.bind(Binding { owner: look::OWNER, token: look::QUIT });
        b.shortcut(Shortcut { key: b'q', modifier: Modifier::Plain });
        b.style(look::HINT);
        b.close();
    }
    let (state, style) = state(root);
    text(&mut b, Role::Status, &words.say(Word::State, &[Hole::Name(words.view().say(state, &[]))]), Some(style));
    b.close();
    b.close();
    view.render.finish(b, shown, out);
    view.built = Some((root_shown, revision));
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
            if bound.binding.token == look::LINK {
                Some(Event::Typed { value })
            } else {
                None
            }
        }
        Act::Submit => {
            if bound.binding.token == look::LINK {
                Some(Event::SignIn)
            } else {
                None
            }
        }
        Act::Press => {
            if bound.binding.token == look::QUIT {
                Some(Event::Quit)
            } else {
                None
            }
        }
        Act::Toggle | Act::Drawing { .. } | Act::Near { .. } => None,
    }
}
