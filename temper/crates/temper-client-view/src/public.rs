//! Pages from public records alone (temper/docs/design/client/web/public.md, 3).
use crate::{
    sign_in::Ways,
    words::{Times, Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{List, Queue, Token, Wall};
use skein_ui::{Act, Binding, Builder, NodeId, Patch, Role, Run, Tree};
use temper_client_domain::{
    address::{Public, print_public},
    public::{PublicEvent, Reader},
};

mod forge;
pub(crate) use forge::short;
mod lines;
mod pages;

/// The anonymous semantic tree and only its public navigation bindings.
#[derive(Debug)]
pub struct PublicView {
    render: skein_ui::View,
    words: Words,
    ways: Ways,
    links: List<Public>,
    revision: u64,
    next: Option<Wall>,
}
impl PublicView {
    #[must_use]
    pub fn new(limits: &skein_ui::Limits, words: Words, ways: Ways) -> Self {
        Self {
            render: skein_ui::View::new(limits),
            words,
            ways,
            links: List::with_capacity(limits.bindings),
            revision: 0,
            next: None,
        }
    }
    #[must_use]
    pub fn tree(&self) -> &Tree {
        self.render.tree()
    }
    #[must_use]
    pub const fn next_change(&self) -> Option<Wall> {
        self.next
    }
}
struct Draw<'a> {
    words: &'a Words,
    links: &'a mut List<Public>,
    times: Times,
    project: u64,
}
impl Draw<'_> {
    fn link(&mut self, b: &mut Builder, name: &[u8], page: Public) {
        self.styled_link(b, name, page, &[]);
    }
    fn styled_link(&mut self, b: &mut Builder, name: &[u8], page: Public, styles: &[skein_ui::Style]) {
        let binding = Binding { owner: crate::frame::owner(), token: Token::new(u64::from(self.links.len())) };
        self.links.push(page).expect("public links within view bounds");
        b.open(Role::Link);
        for style in styles {
            b.style(*style);
        }
        b.name(name);
        b.bind(binding);
        b.link(binding);
        b.text(Run::Plain, name);
        b.close();
    }
    fn text(&self, b: &mut Builder, word: Word) {
        self.words.text(b, word, &[]);
    }
    fn paragraph(&self, b: &mut Builder, word: Word) {
        b.open(Role::Paragraph);
        self.text(b, word);
        b.close();
    }
    fn as_of(&mut self, b: &mut Builder, elapsed: Option<skein_lib::Duration>) {
        if let Some(elapsed) = elapsed {
            let at = Wall::from_nanos(self.times.now.as_nanos().saturating_sub(elapsed.as_nanos()));
            let time = self.times.say(self.words, crate::words::TimeForm::Clock, at);
            b.open(Role::Paragraph);
            self.words.text(b, Word::PublicAsOf, &[skein_ui::phrases::Hole::Name(time)]);
            b.close();
        }
    }
}
/// Rebuild only from the reader's current public records and the supplied clock.
pub fn build_public(view: &mut PublicView, reader: &Reader, wall: Wall, patches: &mut Queue<Patch>) {
    view.links.clear();
    let mut d =
        Draw { words: &view.words, links: &mut view.links, times: Times::new(wall), project: project(reader.page()) };
    let mut b = view.render.builder();
    b.open(Role::Page);
    b.open(Role::Banner);
    b.style(crate::look::TOP);
    d.styled_link(&mut b, &d.words.say(Word::Brand, &[]), Public::Projects, &[crate::look::BRAND]);
    if let Some(origin) = &view.ways.origin {
        let mut address = skein_lib::Writer::new(origin.len().checked_add(1).expect("bounded sign-in address"));
        address.put(origin).expect("origin");
        address.put(b"/").expect("home");
        b.open(Role::Slot);
        b.style(crate::look::TOP_END);
        external(&mut b, &d.words.say(Word::SignInTitle, &[]), &address.finish());
        b.close();
    }
    b.close();
    b.open(Role::Main);
    b.style(crate::look::MAIN);
    b.style(crate::look::PAGE);
    b.style(crate::look::PUBLIC_PAGE);
    d.main(&mut b, reader.shown());
    b.close();
    b.close();
    view.next = d.times.next;
    view.revision = view.revision.checked_add(1).expect("public view revisions fit");
    view.render.finish(b, view.revision, patches);
}
#[must_use]
pub fn act(view: &mut PublicView, node: NodeId, act: Act) -> Option<PublicEvent> {
    let bound = view.render.act(node, act)?;
    Some(PublicEvent::Open { page: *view.links.get(u32::try_from(bound.binding.token.raw()).ok()?)? })
}
#[must_use]
pub fn href(view: &mut PublicView, node: NodeId) -> Option<Box<[u8]>> {
    for shown in view.tree().nodes() {
        if shown.id == node
            && let Some(target) = &shown.external
        {
            return Some(target.clone());
        }
    }
    match act(view, node, Act::Press)? {
        PublicEvent::Open { page } => Some(print_public(&page)),
        PublicEvent::Heard { .. } | PublicEvent::Visible { .. } | PublicEvent::Due => None,
    }
}
fn project(page: &Public) -> u64 {
    match *page {
        Public::Projects => 0,
        Public::Board { project }
        | Public::Task { project, .. }
        | Public::Landings { project }
        | Public::Landing { project, .. } => project,
    }
}
fn external(b: &mut Builder, text: &[u8], address: &[u8]) {
    if !skein_ui::markdown::safe_url(address) {
        return;
    }
    b.open(Role::Link);
    b.name(text);
    b.external(address);
    b.text(Run::Plain, text);
    b.close();
}

fn forge_text(
    words: &Words,
    word: temper_client_domain::forge::words::ChangeWord,
    holes: &[skein_ui::phrases::Hole],
) -> Box<[u8]> {
    words.book.say(words.language, word.id(words.connector), holes).expect("checked forge phrase").text
}

fn heading(b: &mut Builder, text: &[u8]) {
    b.open(Role::Heading { level: 1 });
    b.name(text);
    b.text(Run::Plain, text);
    b.close();
}
