//! One client host shared by the browser and the journeys' replay (client/web/journeys.md, 2).
use crate::{limits, records};
use jig_client_protocol_http::Heard as Above;
use jig_client_shell_web::adapter::{Asked, Heard};
use skein_lib::{Env, List, Queue, Secret, Time, Token, Wall};
use skein_ui::{Act, NodeId, Patch};
use skein_ui_web::{Look, mapping::Primary, page::Page};
use std::collections::BTreeMap;
use temper_client_domain::{
    self as root, Address, Event, Place, Request,
    address::{Filter, parse, print},
};
use temper_client_view as view;

/// One report of a turn, as the shell gives it to the host (client/web/journeys.md, 2).
pub enum Report {
    Push(root::pages::told::Report),
    Screen(skein_ui_web::screen::Report),
    Copied { token: u32, written: root::copy::Written },
    Start,
    Due,
    TabRefused,
    Act { node: NodeId, act: Act },
    Place { path: Box<[u8]> },
    Heard(Heard),
    Carried { reports: Vec<skein_ui_web::transport::Carried>, writes: u32 },
}

/// A sign-in's tab, carried before the transport (client/web/README.md, 2.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TabAct {
    Open { words: Box<[u8]> },
    Point { to: Box<[u8]> },
    Close,
}

/// What the host asks after one report.
#[derive(Debug, Default)]
pub struct Duties {
    pub push: Vec<root::pages::told::Duty>,
    pub screen: Option<bool>,
    pub keep_look: Option<[u8; 4]>,
    pub tab: Vec<TabAct>,
    pub copy: Option<(u32, Box<[u8]>)>,
    pub asked: Vec<Asked>,
    pub writes: u32,
    pub navigate: Option<Box<[u8]>>,
    pub replace: Option<Box<[u8]>>,
    pub show: Option<(Box<[u8]>, bool)>,
}

impl Duties {
    /// Whether carrying these duties leaves this page (client/web/journeys.md, 3.1).
    #[must_use]
    pub fn departs(&self) -> bool {
        self.navigate.is_some() || self.replace.is_some()
    }
}

pub struct Host {
    root: root::Domain,
    view: view::View,
    pub bounds: view::Limits,
    env: Env<root::Limits>,
    out: Queue<Request>,
    patches: Queue<Patch>,
    address: Address,
    mapping: jig_client_protocol_http::Mapping,
    stream: jig_client_protocol_http::Stream,
    tickets: BTreeMap<u32, Token>,
    next: u32,
    navigate: Option<Box<[u8]>>,
    replace: Option<Box<[u8]>>,
    show: Option<(Box<[u8]>, bool)>,
    copy: Option<(u32, Box<[u8]>)>,
    keep_look: Option<[u8; 4]>,
    screen: Option<bool>,
    push: Vec<root::pages::told::Duty>,
}
impl Host {
    #[must_use]
    pub fn new(
        client: root::Limits,
        seed: u64,
        now: Time,
        wall: Wall,
        zone_minutes: i32,
        ways: view::sign_in::Ways,
    ) -> Self {
        assert!(limits::valid(&client), "web capacities are checked");
        let (bounds, words) = limits::view(&client).expect("web view capacities are checked");
        let mut root = root::new(&client, seed);
        root.set_zone_minutes(zone_minutes);
        Self {
            root,
            view: view::View::with_sign_in(&bounds, &client, words, temper_client_protocol_wire::FORGE, ways),
            bounds,
            env: Env { now, wall, limits: client },
            out: Queue::uncapped(),
            patches: Queue::with_capacity(bounds.tree.patches),
            address: Address::Inbox { filter: Filter::ALL },
            mapping: jig_client_protocol_http::Mapping::new(b"/jig").expect("mapping prefix"),
            stream: jig_client_protocol_http::Stream::new(&limits::stream().expect("stream bounds")),
            tickets: BTreeMap::new(),
            next: 0,
            show: None,
            copy: None,
            keep_look: None,
            screen: None,
            push: Vec::new(),
            navigate: None,
            replace: None,
        }
    }
    /// The kept look's night on every page (client/pages/README.md).
    #[must_use]
    pub fn night(&self) -> bool {
        self.root.pages().look().night
    }
    pub fn forge_sign_in(&mut self, forges: view::forge_sign_in::Forges) {
        self.view.forge_sign_in(forges);
    }
    pub fn start(&mut self, path: &[u8], records: Vec<skein_ui_web::store::Record>, secret: Option<Box<[u8]>>) {
        let mut kept = List::with_capacity(u32::try_from(records.len()).expect("bounded store"));
        let mut sign_in = None;
        let mut look = None;
        for record in records {
            if record.key.as_ref() == records::LOOK_KEY {
                look = records::decode_look(&record.value);
            }
            if record.key.as_ref() == records::SIGN_IN_KEY {
                sign_in = records::decode_sign_in(&record.value);
            }
            if let Some(record) = records::decode(record) {
                kept.push(record).expect("counted store");
            }
        }
        if let Some(look) = look {
            self.event(Event::LookKept { look });
        }
        if parse(path) == Place::AuthorizeMalformed {
            self.event(Event::Went { place: Place::AuthorizeMalformed });
            return;
        }
        let place = parse(path);
        self.address = self.opened(path);
        let with = match secret {
            Some(bytes) => jig_client::With::Link { secret: Secret::new(bytes), held: None },
            None => jig_client::With::Transport,
        };
        self.event(Event::Start { address: self.address.clone(), kept, with: Some(with) });
        if let Some(records::SignIn { source, address, refused }) = sign_in {
            self.event(Event::SignInKept { source, address, refused });
        }
        if !matches!(place, Place::Address(_)) {
            self.event(Event::Went { place });
        }
    }
    // The page a path names; a path that prints otherwise is replaced by the printed one.
    fn opened(&mut self, path: &[u8]) -> Address {
        let address = match parse(path) {
            Place::Approve { code } => {
                let printed = root::address::print_approve(code);
                if *printed != *path {
                    self.show = Some((printed, true));
                }
                return self.address.clone();
            }
            Place::Authorize { .. } | Place::AuthorizeMalformed => return self.address.clone(),
            Place::Address(address) => address,
            Place::Missing | Place::Public(_) => Address::Inbox { filter: Filter::ALL },
        };
        let printed = print(&address);
        if *printed != *path {
            self.show = Some((printed, true));
        }
        address
    }
    fn went(&mut self, path: &[u8]) {
        self.address = self.opened(path);
        self.event(Event::Went { place: parse(path) });
    }
    fn href(&mut self, node: NodeId) -> Option<Box<[u8]>> {
        view::href(&mut self.view, node)
    }
    pub fn clocks(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }
    pub fn report(&mut self, report: Report) -> Duties {
        match report {
            Report::Push(report) => self.event(Event::Push(report)),
            Report::Screen(report) => match report {
                skein_ui_web::screen::Report::Left => self.event(Event::ScreenLeft),
                skein_ui_web::screen::Report::Entered | skein_ui_web::screen::Report::Refused => {}
            },
            Report::Copied { token, written } => self.event(Event::Copied { token, written }),
            Report::Start => {}
            Report::Due => self.due(),
            Report::TabRefused => self.event(Event::TabRefused),
            Report::Act { node, act } => self.act(node, act),
            Report::Place { path } => self.went(&path),
            Report::Heard(heard) => self.heard(heard),
            Report::Carried { reports, writes } => self.carried(reports, writes),
        }
        let (asked, writes, tab) = self.duties();
        Duties {
            push: std::mem::take(&mut self.push),
            screen: self.screen.take(),
            keep_look: self.keep_look.take(),
            tab,
            asked,
            writes,
            copy: self.copy.take(),
            navigate: self.navigate.take(),
            replace: self.replace.take(),
            show: self.show.take(),
        }
    }
    pub fn draw(&mut self, page: &mut Page<'_>, ops: &mut Vec<skein_ui_web::page::Op>) {
        page.apply(&self.patches(), ops);
        for node in page.links().to_vec() {
            if let Some(path) = self.href(node) {
                page.set_href(node, &path, ops);
            }
        }
    }
    #[must_use]
    pub fn tree(&self) -> &skein_ui::Tree {
        self.view.tree()
    }
    fn event(&mut self, event: Event) {
        root::step(&mut self.root, &self.env, event, &mut self.out);
    }
    fn client(&mut self, event: jig_client::Event) {
        self.event(Event::Client { event });
    }
    fn due(&mut self) {
        root::fire(&mut self.root, &self.env, &mut self.out);
    }
    /// The event the view makes of `act` on `node`, if any, without giving it (client/web/journeys.md, 3.3).
    #[must_use]
    pub fn event_of(&self, node: NodeId, act: Act) -> Option<temper_client_domain::Event> {
        view::event(&self.view, node, act)
    }
    fn act(&mut self, node: NodeId, act: Act) {
        if let Some(event) = view::act_event(&mut self.view, node, act) {
            self.event(event);
        }
    }
    fn heard(&mut self, heard: Heard) {
        let mut above = Queue::uncapped();
        match heard {
            Heard::Answered { exchange, status, body, headers: _ } => {
                if let Some(token) = self.tickets.remove(&exchange) {
                    above.push(jig_client_protocol_http::answered(token, status, body));
                }
            }
            Heard::Failed { exchange } => {
                if let Some(exchange) = self.tickets.remove(&exchange) {
                    above.push(Above::Lost { exchange });
                }
            }
            Heard::StreamOpened => self.stream.opened(200, &mut above),
            Heard::StreamRefused { status } => self.stream.opened(status, &mut above),
            Heard::Piece { bytes } => self.stream.body(&bytes, &mut above),
            Heard::StreamEnded => self.stream.ended(&mut above),
            Heard::Stored { .. } => self.client(jig_client::Event::Stored),
            Heard::NotStored { .. } => self.client(jig_client::Event::NotStored),
        }
        while let Some(heard) = above.pop() {
            self.client(match heard {
                Above::Reply { exchange, message } => jig_client::Event::Reply { exchange, message },
                Above::Lost { exchange } => jig_client::Event::Lost { exchange },
                Above::Streamed { message } => jig_client::Event::Streamed { message },
                Above::StreamGone => jig_client::Event::StreamGone,
            });
        }
    }
    // The root can step the client several times in one turn. Each Save or
    // Erase needs its own answer, after the whole turn's storage has finished.
    fn carried(&mut self, reports: Vec<skein_ui_web::transport::Carried>, writes: u32) {
        let mut refused = false;
        for report in reports {
            match jig_client_shell_web::adapter::carried(report) {
                Heard::NotStored { .. } => refused = true,
                Heard::Stored { .. } => {}
                report @ (Heard::Answered { .. }
                | Heard::Failed { .. }
                | Heard::StreamOpened
                | Heard::StreamRefused { .. }
                | Heard::Piece { .. }
                | Heard::StreamEnded) => self.heard(report),
            }
        }
        for _ in 0..writes {
            self.client(if refused { jig_client::Event::NotStored } else { jig_client::Event::Stored });
        }
    }
    #[cfg(test)]
    pub fn requests(&mut self) -> Vec<Asked> {
        self.duties().0
    }
    fn duties(&mut self) -> (Vec<Asked>, u32, Vec<TabAct>) {
        let mut tab = Vec::new();
        let mut asked = Vec::new();
        let mut writes = 0_u32;
        while let Some(request) = self.out.pop() {
            match request {
                Request::Push(duty) => self.push.push(duty),
                Request::Screen { on } => self.screen = Some(on),
                Request::KeepLook { look } => self.keep_look = Some(records::encode_look(look)),
                Request::Tab { tab: request } => tab.push(match request {
                    root::Tab::Open { provider } => {
                        TabAct::Open { words: view::opening_tab(&self.view, &self.root, provider) }
                    }
                    root::Tab::Point { to } => TabAct::Point { to },
                    root::Tab::Close => TabAct::Close,
                }),
                Request::KeepSignIn { source, address, refused } => {
                    asked.push(Asked::Save {
                        key: records::SIGN_IN_KEY.into(),
                        value: records::encode_sign_in(&records::SignIn { source, address, refused }),
                    });
                }
                Request::Copy { token, words } => self.copy = Some((token, words)),
                Request::Client { request } => {
                    if matches!(request, jig_client::Request::Save { .. } | jig_client::Request::Erase { .. }) {
                        writes = writes.checked_add(1).expect("bounded client write batches");
                    }
                    self.carry(request, &mut asked);
                }
                Request::Show { address, replace } => {
                    if address != self.address || self.root.authorization().is_some() {
                        let path = print(&address);
                        if path != print(&self.address) || self.root.authorization().is_some() {
                            self.show = Some((path, replace));
                        }
                        self.address = address.clone();
                        self.event(Event::Went { place: Place::Address(address) });
                    }
                }
                Request::Leave { to, replace } => {
                    if replace {
                        self.replace = Some(to);
                    } else {
                        self.navigate = Some(to);
                    }
                }
                Request::SignOut => self.client(jig_client::Event::SignOut),
                Request::Unlinked { .. } | Request::Dropped { .. } | Request::Fact { .. } => {}
            }
        }
        root::take(&mut self.root, &self.env, &mut self.out);
        (asked, writes, tab)
    }
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "the root consumes notifications and keeps unlinked requests out of the linked browser transport"
    )]
    fn carry(&mut self, request: jig_client::Request, asked: &mut Vec<Asked>) {
        match request {
            jig_client::Request::Save { records } => {
                for record in records.into_boxed().into_vec() {
                    let (key, value) = records::encode(record);
                    asked.push(Asked::Save { key, value });
                }
            }
            jig_client::Request::Erase { records } => {
                for key in records.into_boxed().into_vec() {
                    asked.push(Asked::Erase { key });
                }
            }
            jig_client::Request::Exchange { exchange, credential, message } => {
                self.next = self.next.checked_add(1).expect("fetch tickets fit");
                self.tickets.insert(self.next, exchange);
                asked.push(Asked::Exchange {
                    id: self.next,
                    call: jig_client_protocol_http::exchange(&self.mapping, credential.as_ref(), message),
                    bound: limits::BODY_BYTES,
                    cache: skein_ui_web::fetch::Cache::Browser,
                    keep: &[],
                });
            }
            jig_client::Request::OpenStream { credential } => {
                self.stream = jig_client_protocol_http::Stream::new(&limits::stream().expect("stream bounds"));
                asked.push(Asked::OpenStream {
                    call: jig_client_protocol_http::stream(&self.mapping, credential.as_ref()),
                    piece: limits::PIECE,
                });
            }
            jig_client::Request::CloseStream => asked.push(Asked::CloseStream),
            _ => {}
        }
    }
    pub fn build(&mut self) {
        view::build(&mut self.view, &self.root, self.env.wall, &mut self.patches);
    }
    pub fn patches(&mut self) -> Vec<Patch> {
        self.build();
        let mut patches = Vec::with_capacity(usize::try_from(self.patches.len()).expect("patch count fits usize"));
        while let Some(patch) = self.patches.pop() {
            patches.push(patch);
        }
        patches
    }
    #[must_use]
    pub fn deadline(&self) -> Option<Time> {
        deadline(root::next_deadline(&self.root), self.view.next_change(), self.env.now, self.env.wall)
    }
}

/// The page with temper's look, for the shell's start and the replay.
#[must_use]
pub fn page(bounds: &skein_ui::Limits, primary: Primary) -> Page<'static> {
    Page::new(
        Look {
            names: &[view::look::NAMES, skein_ui::flow::NAMES],
            pictures: &[],
            draw: view::paths::draw_any,
            components: &[],
            elements: view::look::ELEMENTS,
        },
        bounds,
        b"temper",
        primary,
    )
}

#[cfg(test)]
#[path = "host_tests.rs"]
mod tests;

fn deadline(domain: Option<Time>, next: Option<Wall>, now: Time, wall: Wall) -> Option<Time> {
    let view = next.map(|next| {
        now.saturating_add(skein_lib::Duration::from_nanos(next.as_nanos().saturating_sub(wall.as_nanos())))
    });
    [domain, view].into_iter().flatten().min()
}

#[cfg(test)]
mod zone_tests {
    use super::*;

    #[test]
    fn the_host_keeps_the_shells_zone_before_start() {
        for zone in [0_i32, 120_i32, -300_i32] {
            let host = Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, zone, view::sign_in::Ways::NONE);
            assert_eq!(host.root.zone_minutes(), zone);
        }
    }
}

#[cfg(test)]
mod tab_tests {
    use super::*;

    #[test]
    fn tab_requests_keep_their_order_and_do_not_depart() {
        let mut host = Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, view::sign_in::Ways::NONE);
        let to = b"https://issuer.example/sign-in".as_slice();
        for tab in [root::Tab::Open { provider: 17 }, root::Tab::Point { to: to.into() }, root::Tab::Close] {
            host.out.push(Request::Tab { tab });
        }
        let duties = host.report(Report::Start);
        assert_eq!(
            duties.tab,
            vec![
                TabAct::Open { words: "Opening ’s sign-in…".as_bytes().into() },
                TabAct::Point { to: to.into() },
                TabAct::Close,
            ]
        );
        assert!(duties.asked.is_empty());
        assert!(!duties.departs());
    }

    #[test]
    fn a_tab_refusal_has_no_duty_yet() {
        let mut host = Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, view::sign_in::Ways::NONE);
        let duties = host.report(Report::TabRefused);
        assert_eq!(duties.tab.len(), 0);
        assert!(duties.asked.is_empty());
        assert_eq!(duties.writes, 0);
        assert!(duties.navigate.is_none());
        assert!(duties.replace.is_none());
        assert!(duties.show.is_none());
    }
}
