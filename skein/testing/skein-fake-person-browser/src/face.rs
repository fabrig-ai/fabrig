//! Queries and acts translated without executing script in the page.
use skein_browser::boundary::{Event, Go, Key, Match, Query, Read, Refusal, Request, Seen};
use skein_fake_person::{Doing, Face, Find, Found, Target};
use skein_lib::{Time, Token};
use skein_ui::{Act, NodeId, Role};
use skein_ui_web::mapping::accessible;
use std::{os::unix::ffi::OsStrExt, path::PathBuf};

/// One completed download, in the order the browser reported it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Download {
    pub name: Box<[u8]>,
    pub path: PathBuf,
}

enum Input {
    Select { node: u64, value: Box<[u8]> },
    Type { node: u64, value: Box<[u8]> },
}

struct Choice {
    option: u64,
    selects: std::collections::VecDeque<u64>,
    select: Option<u64>,
    name: Box<[u8]>,
    checking: bool,
}

struct Search {
    title: bool,
    boxes: bool,
    find: Find,
    depth: usize,
    role: usize,
    within: Option<u64>,
    op: Token,
}

struct Keeping {
    op: Token,
    selector: Box<[u8]>,
    attribute: Box<[u8]>,
    name: Box<[u8]>,
}
struct Press {
    find: Find,
    at: Option<skein_browser::boundary::Point>,
    until: Time,
    again: Option<Time>,
}
enum CommandState {
    Idle,
    Pending,
    Done,
}
type KeptValue = (Box<[u8]>, Box<[u8]>);
mod look;
use look::Capture;

/// One page's asynchronous semantic face. Nodes are numbered locally.
pub struct BrowserFace<'a> {
    page: Token,
    pub(crate) first_url: String,
    pub(crate) origins: Vec<String>,
    address: std::borrow::Cow<'a, str>,
    serial: u64,
    requests: Vec<Request>,
    search: Option<Search>,
    answer: Option<Found>,
    seen: Vec<(Seen, Option<Role>)>,
    boxed_matches: Vec<Seen>,
    boxed_more: u32,
    acting: Option<Token>,
    now: Time,
    last_find: Option<(NodeId, Find)>,
    look_again: Option<(Find, Time)>,
    press: Option<Press>,
    expected_path: Option<Box<[u8]>>,
    failure: Option<String>,
    refusal: Option<Refusal>,
    read_address: Option<Box<[u8]>>,
    read_tabs: Option<(skein_lib::List<skein_browser::boundary::Tab>, u32)>,
    input: Option<Input>,
    trouble: Vec<Box<[u8]>>,
    downloads: Vec<Download>,
    warned: bool,
    go: Option<Token>,
    choice: Option<Choice>,
    keep: Option<Keeping>,
    kept: Option<KeptValue>,
    keep_done: bool,
    command: Option<crate::harness::CommandAction>,
    command_state: CommandState,
    viewport_size: (u32, u32),
    viewport_change: Option<(Token, u32, u32)>,
    capture: Option<Capture>,
    snapshot: Option<Box<[u8]>>,
    screenshot: Option<Box<[u8]>>,
}

impl<'a> BrowserFace<'a> {
    #[must_use]
    pub fn new(page: Token, address: impl Into<std::borrow::Cow<'a, str>>) -> Self {
        Self::continuing(page, address, page.raw().checked_mul(1_000_000).expect("page operation range fits"))
    }
    /// Make a face whose next operation follows `serial` from the previous face.
    #[must_use]
    pub fn continuing(page: Token, address: impl Into<std::borrow::Cow<'a, str>>, serial: u64) -> Self {
        Self {
            page,
            first_url: String::new(),
            origins: Vec::new(),
            address: address.into(),
            serial,
            requests: Vec::new(),
            search: None,
            answer: None,
            seen: Vec::new(),
            boxed_matches: Vec::new(),
            boxed_more: 0,
            acting: None,
            now: Time::ZERO,
            last_find: None,
            look_again: None,
            press: None,
            expected_path: None,
            failure: None,
            refusal: None,
            read_address: None,
            read_tabs: None,
            input: None,
            trouble: Vec::new(),
            downloads: Vec::new(),
            warned: false,
            go: None,
            choice: None,
            keep: None,
            kept: None,
            keep_done: false,
            command: None,
            command_state: CommandState::Idle,
            viewport_size: (1280, 800),
            viewport_change: None,
            capture: None,
            snapshot: None,
            screenshot: None,
        }
    }
    pub fn title(&mut self, words: &[u8]) -> Found {
        if let Some(answer) = self.answer.take() {
            return answer;
        }
        let find = Find::exactly_named(Role::Page, words);
        if !self.look_due(&find) {
            return Found::Pending;
        }
        if self.search.is_none() {
            self.ask(Search { title: true, boxes: false, find, depth: 0, role: 0, within: None, op: Token::new(0) });
        }
        self.answer.take().unwrap_or(Found::Pending)
    }
    #[must_use]
    pub fn warned(&self) -> bool {
        self.warned
    }
    #[must_use]
    pub fn trouble(&self) -> &[Box<[u8]>] {
        &self.trouble
    }
    /// Completed files belonging to this person, in completion order.
    #[must_use]
    pub fn downloads(&self) -> &[Download] {
        &self.downloads
    }
    fn token(&mut self) -> Token {
        self.serial = self.serial.checked_add(1).expect("operation numbers fit");
        Token::new(self.serial)
    }
    /// Drain requests into the browser kit before the loop waits.
    pub fn requests(&mut self) -> impl Iterator<Item = Request> + '_ {
        self.requests.drain(..)
    }
    #[must_use]
    pub fn pending(&self) -> bool {
        self.search.is_some()
            || self.acting.is_some()
            || self.press.is_some()
            || self.keep.is_some()
            || matches!(self.command_state, CommandState::Pending)
    }
    /// Advance bounded re-finds with the loop's clock (skein/docs/design/testing/browser.md, 6).
    pub fn tick(&mut self, now: Time) {
        self.now = now;
        let Some(press) = &self.press else { return };
        if now >= press.until {
            self.press = None;
            self.search = None;
            self.acting = None;
            self.fail("browser refused act: Gone (press deadline passed)".into());
        } else if press.again.is_some_and(|again| now >= again) {
            let find = press.find.clone();
            self.press.as_mut().expect("pending press").again = None;
            self.ask(Search { title: false, boxes: false, find, depth: 0, role: 0, within: None, op: Token::new(0) });
        }
    }
    fn keep_press(&mut self, node: NodeId, at: Option<skein_browser::boundary::Point>) {
        self.press = self.last_find.as_ref().filter(|(found, _)| *found == node).map(|(_, find)| Press {
            find: find.clone(),
            at,
            until: self.now.saturating_add(skein_browser::Limits::default().answer),
            again: None,
        });
    }
    fn absent(&mut self, find: Find) {
        let again = self.now.saturating_add(skein_browser::Limits::default().poll);
        self.look_again = Some((find, again));
        if let Some(press) = &mut self.press {
            press.again = Some(again);
        } else {
            self.answer = Some(Found::Absent);
        }
    }
    fn look_due(&self, find: &Find) -> bool {
        self.look_again.as_ref().is_none_or(|(looked, again)| looked != find || self.now >= *again)
    }
    pub(crate) fn command(&mut self, action: crate::harness::CommandAction) {
        self.command = Some(action);
        self.command_state = CommandState::Pending;
    }
    pub(crate) fn command_action(&mut self) -> Option<crate::harness::CommandAction> {
        self.command.take()
    }
    pub(crate) fn command_complete(&mut self) {
        self.command_state = CommandState::Done;
    }
    pub(crate) fn command_done(&mut self) -> bool {
        if matches!(self.command_state, CommandState::Done) {
            self.command_state = CommandState::Idle;
            true
        } else {
            false
        }
    }
    pub(crate) fn navigating(&self) -> bool {
        self.acting.is_some()
    }
    pub(crate) fn failed(&self) -> bool {
        self.failure.is_some()
    }
    pub(crate) fn keep(&mut self, selector: Box<[u8]>, attribute: Option<Box<[u8]>>, name: Box<[u8]>) {
        let op = self.token();
        let attribute = attribute.unwrap_or_else(|| b"value".as_slice().into());
        self.requests.push(Request::Attribute {
            page: self.page,
            op,
            selector: selector.clone(),
            name: attribute.clone(),
        });
        self.keep = Some(Keeping { op, selector, attribute, name });
        self.keep_done = false;
    }
    pub(crate) fn kept(&mut self) -> Option<KeptValue> {
        self.kept.take()
    }
    pub(crate) fn kept_done(&mut self) -> bool {
        std::mem::take(&mut self.keep_done)
    }
    fn keep_failed(&mut self, why: &str) {
        let Keeping { selector, attribute, .. } = self.keep.take().expect("pending keep");
        self.fail(format!(
            "keep selector {} attribute {}: {why}",
            String::from_utf8_lossy(&selector),
            String::from_utf8_lossy(&attribute)
        ));
        self.keep_done = true;
    }
    pub(crate) fn fail(&mut self, why: String) {
        self.failure = Some(why);
    }
    /// The last allocated operation number, for the next face's continuation.
    #[must_use]
    pub fn serial(&self) -> u64 {
        self.serial
    }
    /// The kit's refusal of the last act, if it refused one.
    #[must_use]
    pub fn refusal(&self) -> Option<Refusal> {
        self.refusal
    }
    /// The last observation of a node numbered by this face.
    #[must_use]
    pub fn seen(&self, node: NodeId) -> Option<&Seen> {
        self.seen.get(usize::try_from(node.0).expect("node fits")).map(|(seen, _)| seen)
    }
    pub fn failure(&mut self) -> Option<String> {
        self.failure.take()
    }
    pub(crate) fn text(&mut self, node: NodeId, words: Box<[u8]>, compose: bool) -> Result<(), String> {
        let seen = &self.seen.get(usize::try_from(node.0).expect("node fits")).ok_or("unknown field")?.0;
        let node = seen.node;
        self.refusal = None;
        let op = self.token();
        let request = if compose {
            Request::Compose { page: self.page, op, node, text: words }
        } else {
            Request::Type { page: self.page, op, node, text: words }
        };
        self.acting = Some(op);
        self.requests.push(request);
        Ok(())
    }
    /// Find a semantic node and read its viewport rectangle for layout assertions.
    pub fn find_boxed(&mut self, find: &Find) -> Found {
        if let Some(answer) = self.answer.take() {
            return answer;
        }
        if self.search.is_none() {
            self.ask(Search {
                title: false,
                boxes: true,
                find: find.clone(),
                depth: 0,
                role: 0,
                within: None,
                op: Token::new(0),
            });
        }
        self.answer.take().unwrap_or(Found::Pending)
    }
    /// All retained matches of the last completed boxed query and the count beyond
    /// the browser's match limit. Nodes without geometry keep their missing rectangle.
    #[must_use]
    pub fn boxed_matches(&self) -> (&[Seen], u32) {
        (&self.boxed_matches, self.boxed_more)
    }
    pub(crate) fn rect(&self, node: NodeId) -> Option<skein_browser::boundary::Rect> {
        self.seen.get(usize::try_from(node.0).expect("node fits"))?.0.rect
    }
    pub(crate) fn keep_value(&mut self, name: Box<[u8]>, value: Box<[u8]>) {
        self.kept = Some((name, value));
    }
    pub(crate) fn height(&self, node: NodeId) -> Option<u64> {
        self.seen.get(usize::try_from(node.0).expect("node fits"))?.0.rect.map(|rect| rect.height)
    }
    pub(crate) fn press_at(&mut self, node: NodeId, at: skein_browser::boundary::Point) -> Result<(), String> {
        let seen = &self.seen.get(usize::try_from(node.0).expect("node fits")).ok_or("unknown drawing")?.0;
        let backend = seen.node;
        self.keep_press(node, Some(at));
        self.refusal = None;
        let op = self.token();
        self.acting = Some(op);
        self.requests.push(Request::Press { page: self.page, op, node: backend, at: Some(at) });
        Ok(())
    }
    /// Press a key into the focused control, optionally with Shift held.
    pub fn key(&mut self, key: Key, shift: bool) {
        self.refusal = None;
        let op = self.token();
        self.acting = Some(op);
        self.requests.push(Request::Key { page: self.page, op, key, shift });
    }
    /// Set this page's viewport in CSS pixels, keeping desktop input behaviour.
    pub fn viewport(&mut self, width: u32, height: u32) {
        self.refusal = None;
        let op = self.token();
        self.acting = Some(op);
        self.viewport_change = Some((op, width, height));
        self.requests.push(Request::Viewport { page: self.page, op, width, height });
    }
    /// Read the current whole URL; take its answer once the face is no longer pending.
    pub fn address(&mut self) {
        self.refusal = None;
        self.expected_path = None;
        self.read_address = None;
        let op = self.token();
        self.acting = Some(op);
        self.requests.push(Request::Address { page: self.page, op });
    }
    pub(crate) fn tabs(&mut self) {
        self.read_tabs = None;
        let op = self.token();
        self.acting = Some(op);
        self.requests.push(Request::Tabs { page: self.page, op });
    }
    pub(crate) fn take_tabs(&mut self) -> Option<(skein_lib::List<skein_browser::boundary::Tab>, u32)> {
        self.read_tabs.take()
    }
    /// Take the last address read once, after its operation completes.
    pub fn take_address(&mut self) -> Option<Box<[u8]>> {
        self.read_address.take()
    }
    /// Read the page's address; the failure names it unless its path, query kept, is `path`.
    pub(crate) fn read_address(&mut self, path: &[u8]) {
        let op = self.token();
        self.acting = Some(op);
        self.expected_path = Some(path.into());
        self.requests.push(Request::Address { page: self.page, op });
    }
    pub(crate) fn focused(&self, node: NodeId) -> bool {
        self.seen.get(usize::try_from(node.0).expect("node fits")).is_some_and(|(seen, _)| seen.states.focused)
    }
    pub(crate) fn pressed(&self, node: NodeId) -> bool {
        self.seen.get(usize::try_from(node.0).expect("node fits")).is_some_and(|(seen, _)| seen.states.pressed)
    }
    pub(crate) fn field_is(&self, node: NodeId, words: &[u8]) -> bool {
        self.seen
            .get(usize::try_from(node.0).expect("node fits"))
            .is_some_and(|(seen, _)| seen.states.focused && seen.value.as_ref() == words)
    }
    /// Move back or forward, or reload, accepting the page's closing warning.
    pub fn history(&mut self, to: Go) {
        self.refusal = None;
        let op = self.token();
        self.warned = false;
        self.go = Some(op);
        self.acting = Some(op);
        self.requests.push(Request::Go { page: self.page, op, to });
    }
    pub(crate) fn reopen(&mut self) {
        self.absolute(self.first_url.as_bytes().into());
    }
    pub(crate) fn visit(&mut self, origin: u32, path: &[u8]) {
        if let Some(origin) = self.origins.get(usize::try_from(origin).expect("origin fits")) {
            self.absolute(format!("{origin}{}", String::from_utf8_lossy(path)).into_bytes().into());
        } else {
            self.fail(format!("unknown origin {origin}"));
        }
    }
    pub(crate) fn absolute(&mut self, url: Box<[u8]>) {
        self.go_to(url);
    }
    /// Navigate to a whole URL, accepting the page's closing warning.
    pub fn go_to(&mut self, url: Box<[u8]>) {
        self.history(Go::Address(url));
    }
    fn ask(&mut self, mut search: Search) {
        let target = search.find.within.get(search.depth).unwrap_or(&search.find.target);
        let (mut roles, name, matching) = query_parts(target);
        if search.title {
            roles = &[b"RootWebArea"];
        }
        let Some(role) = roles.get(search.role) else {
            self.absent(search.find);
            return;
        };
        search.op = self.token();
        self.requests.push(Request::Find {
            page: self.page,
            op: search.op,
            query: Query {
                role: Box::from(*role),
                name: name.into(),
                matching,
                within: search.within,
                boxes: search.boxes && search.depth == search.find.within.len(),
            },
        });
        self.search = Some(search);
    }
    fn remember(&mut self, seen: &Seen, role: Option<Role>) -> NodeId {
        if let Some(at) = self.seen.iter().position(|(old, _)| old.node == seen.node) {
            self.seen[at] = (seen.clone(), role);
            return NodeId(u32::try_from(at).expect("node numbers fit"));
        }
        let id = NodeId(u32::try_from(self.seen.len()).expect("node numbers fit"));
        self.seen.push((seen.clone(), role));
        id
    }
    fn search_found(&mut self, op: Token, seen: &skein_lib::List<Seen>, more: u32) {
        if self.choice.is_some() && self.acting == Some(op) {
            self.choice_found(seen, more);
            return;
        }
        if self.search.as_ref().map(|s| s.op) != Some(op) {
            return;
        }
        let mut search = self.search.take().expect("matching search");
        if search.boxes && search.depth == search.find.within.len() {
            self.boxed_matches = seen.iter().cloned().collect();
            self.boxed_more = more;
        }
        let nth = if search.depth < search.find.within.len() { 0 } else { search.find.nth };
        if let Some(seen) = seen.get(nth) {
            if search.depth < search.find.within.len() {
                search.depth += 1;
                search.role = 0;
                search.within = Some(seen.node);
                self.ask(search);
            } else {
                let role = target_role(&search.find.target);
                let node = self.remember(seen, role);
                self.look_again =
                    Some((search.find.clone(), self.now.saturating_add(skein_browser::Limits::default().poll)));
                self.last_find = Some((node, search.find));
                if let Some(press) = &self.press {
                    let at = press.at;
                    let op = self.token();
                    self.refusal = None;
                    self.acting = Some(op);
                    self.requests.push(Request::Press { page: self.page, op, node: seen.node, at });
                } else {
                    self.answer = Some(Found::Node(node));
                }
            }
        } else if more > 0 {
            self.fail(format!(
                "find {} needs match {nth}, past the kit's Limits::matches ({} kept)",
                search.find,
                seen.len()
            ));
            self.answer = Some(Found::Absent);
        } else if seen.is_empty() {
            search.role += 1;
            self.ask(search);
        } else {
            self.absent(search.find);
        }
    }
    /// Deliver operation events and downloads routed by the harness to this person.
    pub fn event(&mut self, event: &Event) {
        if self.look_event(event) {
            return;
        }
        match event {
            Event::Warned { op } => {
                if self.go == Some(*op) && self.acting == Some(*op) {
                    self.warned = true;
                }
            }
            Event::Found { op, seen, more } => self.search_found(*op, seen, *more),
            Event::Downloaded { name, path, .. } => {
                self.downloads.push(Download { name: name.clone(), path: std::ffi::OsStr::from_bytes(path).into() });
            }
            Event::Done { op } => {
                if self.acting == Some(*op) {
                    if let Some((change, width, height)) = self.viewport_change.take()
                        && change == *op
                    {
                        self.viewport_size = (width, height);
                    }
                    self.acting = None;
                    self.press = None;
                    self.input_next();
                    self.choice_check();
                }
            }
            Event::Tabs { op, tabs, more } => {
                if self.acting == Some(*op) {
                    self.acting = None;
                    self.read_tabs = Some((tabs.clone(), *more));
                }
            }
            Event::Address { op, url } => self.address_reported(*op, url),
            Event::Attribute { op, read } => self.attribute_reported(*op, read),
            Event::Refused { op, why, detail } => self.refused(*op, *why, detail),
            Event::Trouble { page, text, .. } => {
                if *page == self.page {
                    self.trouble.push(text.clone());
                }
            }
            Event::Ready { .. }
            | Event::Opened { .. }
            | Event::Met { .. }
            | Event::Missed { .. }
            | Event::Snapshot { .. }
            | Event::Screenshot { .. }
            | Event::Closed { .. } => {}
        }
    }
    fn address_reported(&mut self, op: Token, url: &[u8]) {
        if self.acting == Some(op) {
            self.acting = None;
            self.read_address = Some(url.into());
            if let Some(path) = self.expected_path.take()
                && url_path(url) != &*path
            {
                self.failure = Some(format!(
                    "the page's address was {}, expected the path {}",
                    String::from_utf8_lossy(url),
                    String::from_utf8_lossy(&path)
                ));
            }
        }
    }
    fn attribute_reported(&mut self, op: Token, read: &Read) {
        if self.keep.as_ref().map(|keep| keep.op) == Some(op) {
            match read {
                Read::NoElement => self.keep_failed("no element"),
                Read::NoAttribute => self.keep_failed("no attribute"),
                Read::Value(value) => {
                    let Keeping { name, .. } = self.keep.take().expect("pending keep");
                    self.kept = Some((name, value.clone()));
                    self.keep_done = true;
                }
            }
        }
    }
    /// Translate one person's operation, replacing a typed field's whole value.
    ///
    /// # Errors
    /// An unknown node, drawing act or invalid native field value cannot be acted on.
    pub fn perform(&mut self, doing: Doing<Box<[u8]>>) -> Result<(), String> {
        self.refusal = None;
        let op = self.token();
        let request = match doing {
            Doing::Go { address } => {
                self.warned = false;
                self.go = Some(op);
                Request::Go {
                    page: self.page,
                    op,
                    to: Go::Address(
                        format!("http://{}{}", self.address, String::from_utf8_lossy(&address)).into_bytes().into(),
                    ),
                }
            }
            Doing::Reload => {
                self.warned = false;
                self.go = Some(op);
                Request::Go { page: self.page, op, to: Go::Reload }
            }
            Doing::Act { node, act } => {
                let Some(request) = self.act_request(op, node, act)? else { return Ok(()) };
                request
            }
        };
        self.acting = Some(op);
        self.requests.push(request);
        Ok(())
    }
    fn act_request(&mut self, op: Token, node: NodeId, act: Act) -> Result<Option<Request>, String> {
        let (seen, role) = self
            .seen
            .get(usize::try_from(node.0).expect("node fits"))
            .ok_or_else(|| format!("unknown node {}", node.0))?;
        if *role == Some(Role::Option) && act == Act::Press {
            let option = seen.node;
            self.choice = Some(Choice {
                option,
                selects: std::collections::VecDeque::new(),
                select: None,
                name: seen.name.clone(),
                checking: false,
            });
            self.choice_query(b"combobox", None);
            return Ok(None);
        }
        let request = match act {
            Act::Press | Act::Toggle => {
                let backend = seen.node;
                self.keep_press(node, None);
                Request::Press { page: self.page, op, node: backend, at: None }
            }
            Act::Input { value } => {
                let keys = match seen.role.as_ref() {
                    b"Date" => Some(crate::keys::date(&value)),
                    b"InputTime" => Some(crate::keys::time(&value)),
                    _ => None,
                };
                if let Some(keys) = keys {
                    let text = keys.ok_or_else(|| {
                        format!(
                            "field '{}' takes no value '{}'",
                            String::from_utf8_lossy(&seen.name),
                            String::from_utf8_lossy(&value)
                        )
                    })?;
                    Request::Keys { page: self.page, op, node: seen.node, text }
                } else {
                    self.input = Some(Input::Select { node: seen.node, value });
                    Request::Type { page: self.page, op, node: seen.node, text: Box::new([]) }
                }
            }
            Act::Submit => Request::Key { page: self.page, op, key: Key::Enter, shift: false },
            Act::Near { .. } => return Err("a person scrolls a log; its `Near` is the page part's".into()),
            Act::Drawing { .. } => return Err("a person cannot press a drawing mark".into()),
        };
        Ok(Some(request))
    }

    fn input_next(&mut self) {
        match self.input.take() {
            Some(Input::Select { node, value }) => {
                self.key(Key::SelectAll, false);
                self.input = Some(Input::Type { node, value });
            }
            Some(Input::Type { node, value }) => {
                let op = self.token();
                self.acting = Some(op);
                self.requests.push(Request::Type { page: self.page, op, node, text: value });
            }
            None => {}
        }
    }
    fn choice_query(&mut self, role: &[u8], within: Option<u64>) {
        let op = self.token();
        self.acting = Some(op);
        self.requests.push(Request::Find {
            page: self.page,
            op,
            query: Query { role: role.into(), name: Box::from([]), matching: Match::Words, within, boxes: false },
        });
    }
    fn choice_found(&mut self, seen: &skein_lib::List<Seen>, more: u32) {
        if more > 0 {
            self.choice = None;
            self.acting = None;
            self.fail("the option's select query exceeded the kit's bound".into());
            return;
        }
        let choice = self.choice.as_mut().expect("choice active");
        if choice.checking {
            let chosen = seen.iter().find(|node| node.states.selected);
            let failure = match chosen {
                Some(chosen) if chosen.node == choice.option => None,
                Some(chosen) => Some(format!(
                    "type-ahead requested option '{}', but selected '{}'",
                    String::from_utf8_lossy(&choice.name),
                    String::from_utf8_lossy(&chosen.name)
                )),
                None => Some(format!(
                    "type-ahead requested option '{}', but no option was selected",
                    String::from_utf8_lossy(&choice.name)
                )),
            };
            self.choice = None;
            self.acting = None;
            if let Some(failure) = failure {
                self.fail(failure);
            }
            return;
        }
        if let Some(select) = choice.select {
            if seen.iter().any(|node| node.node == choice.option) {
                choice.checking = true;
                let text = choice.name.clone();
                let op = self.token();
                self.acting = Some(op);
                self.requests.push(Request::Keys { page: self.page, op, node: select, text });
                return;
            }
        } else {
            choice.selects.extend(seen.iter().map(|node| node.node));
        }
        if let Some(select) = choice.selects.pop_front() {
            choice.select = Some(select);
            self.choice_query(b"option", Some(select));
        } else {
            self.choice = None;
            self.acting = None;
            self.fail("the option no longer belongs to a select".into());
        }
    }
    fn choice_check(&mut self) {
        let Some(choice) = self.choice.as_ref() else { return };
        if choice.checking {
            let select = choice.select.expect("typing into the option's select");
            self.choice_query(b"option", Some(select));
        }
    }
}
impl Face for BrowserFace<'_> {
    fn find(&mut self, find: &Find) -> Found {
        if let Some(answer) = self.answer.take() {
            return answer;
        }
        if !self.look_due(find) {
            return Found::Pending;
        }
        if self.search.is_none() {
            self.ask(Search {
                title: false,
                boxes: false,
                find: find.clone(),
                depth: 0,
                role: 0,
                within: None,
                op: Token::new(0),
            });
        }
        self.answer.take().unwrap_or(Found::Pending)
    }
    fn allows(&self, node: NodeId, _act: &Act) -> bool {
        self.seen.get(usize::try_from(node.0).expect("node fits")).is_some_and(|(seen, _)| !seen.states.disabled)
    }
    fn role(&self, node: NodeId) -> Option<Role> {
        self.seen.get(usize::try_from(node.0).expect("node fits")).and_then(|(_, role)| *role)
    }
}
fn target_role(target: &Target) -> Option<Role> {
    match target {
        Target::Role { role, .. } | Target::ExactRole { role, .. } => Some(*role),
        Target::Text { .. } => None,
    }
}
fn query_parts(target: &Target) -> (&'static [&'static [u8]], &[u8], Match) {
    match target {
        Target::Role { role, name } => (accessible(*role), name, Match::Words),
        Target::ExactRole { role, name } => (accessible(*role), name, Match::Exact),
        Target::Text { text } => (&[b"StaticText"], text, Match::Words),
    }
}

/// The URL from the first `/` after its `://`, query and fragment kept.
fn url_path(url: &[u8]) -> &[u8] {
    let rest =
        url.windows(3).position(|window| window == b"://").map_or(url, |at| url.get(at + 3..).unwrap_or_default());
    rest.iter().position(|byte| *byte == b'/').map_or(&[], |at| rest.get(at..).unwrap_or_default())
}

impl BrowserFace<'_> {
    fn refused(&mut self, op: Token, why: Refusal, detail: &[u8]) {
        let detail = if detail.is_empty() { String::new() } else { format!(" ({})", String::from_utf8_lossy(detail)) };
        if self.keep.as_ref().map(|keep| keep.op) == Some(op) {
            self.keep_failed(&format!("browser refused read: {why:?}{detail}"));
        }
        if self.search.as_ref().map(|s| s.op) == Some(op) {
            let search = self.search.take().expect("matching search");
            if self.press.is_some() && why != Refusal::Gone {
                self.press = None;
                self.refusal = Some(why);
                self.fail(format!("browser refused act: {why:?}{detail}"));
                return;
            }
            self.absent(search.find);
        }
        if self.acting != Some(op) {
            return;
        }
        self.acting = None;
        self.choice = None;
        self.input = None;
        self.refusal = Some(why);
        if let Some(press) = self.press.as_mut().filter(|_| why == Refusal::Gone) {
            press.again = Some(self.now.saturating_add(skein_browser::Limits::default().poll));
        } else {
            self.press = None;
            self.failure = Some(format!("browser refused act: {why:?}{detail}"));
        }
    }
}
