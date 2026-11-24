//! The hosted browser replies at the kit's boundary (skein/docs/design/ui/web/testing.md, 2).
use super::BROWSER;
use skein_browser::boundary::{Event, Expect, Go, Match, Query, Read, Refusal, Request, Seen, States};
use skein_lib::{List, Queue, Time, Token};
use std::{collections::BTreeMap, io::Write as _};

type Bytes = Box<[u8]>;

/// A page's accessible nodes, in document order, and elements for markup reads.
#[derive(Default)]
pub struct Shown {
    pub nodes: Vec<(Bytes, Bytes)>,
    pub elements: Vec<Element>,
}

/// The first listed element with a selector is the one that selector finds.
pub struct Element {
    pub selector: Bytes,
    pub attributes: Vec<(Bytes, Bytes)>,
}

/// Static pages for every person's browser. No navigation sends a request.
#[derive(Default)]
pub struct Scripted {
    shown: BTreeMap<Bytes, Shown>,
    pages: BTreeMap<Token, Bytes>,
    awaits: Vec<Await>,
}

struct Await {
    page: Token,
    op: Token,
    query: Query,
    expect: Expect,
    until: Time,
}

impl Scripted {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn page(mut self, path: &[u8], page: Shown) -> Self {
        assert!(self.shown.insert(path.into(), page).is_none(), "one scripted page per path");
        self
    }

    fn shown(&self, page: Token) -> Option<&Shown> {
        self.pages.get(&page).and_then(|path| self.shown.get(path))
    }

    fn navigate(&mut self, page: Token, url: &[u8]) {
        let address = url.split(|b| matches!(b, b'?' | b'#')).next().expect("address part");
        let path =
            if let Some(authority) = address.strip_prefix(b"http://").or_else(|| address.strip_prefix(b"https://")) {
                authority.iter().position(|b| *b == b'/').map_or(b"/".as_slice(), |start| &authority[start..])
            } else {
                address
            };
        self.pages.insert(page, path.into());
    }

    fn find(&self, page: Token, query: &Query, limit: u32) -> (List<Seen>, u32) {
        let mut seen = List::with_capacity(limit);
        let cap = usize::try_from(limit).expect("matches fit");
        let mut more = 0_u32;
        for (index, (role, name)) in self.shown(page).iter().flat_map(|shown| shown.nodes.iter().enumerate()) {
            let matches = match query.matching {
                Match::Exact => *name == query.name,
                Match::Words => {
                    query.name.is_empty() || name.windows(query.name.len()).any(|words| words == query.name.as_ref())
                }
            };
            if *role == query.role && matches {
                if seen.as_slice().len() == cap {
                    more = more.saturating_add(1);
                } else {
                    seen.push(Seen {
                        node: u64::try_from(index).expect("node fits") + 1,
                        role: role.clone(),
                        name: name.clone(),
                        value: Box::new([]),
                        states: States {
                            focused: false,
                            disabled: false,
                            checked: false,
                            expanded: false,
                            selected: false,
                            pressed: false,
                        },
                        rect: None,
                    })
                    .expect("retained matches fit their limit");
                }
            }
        }
        (seen, more)
    }

    /// A person's context is ready synchronously; all other answers are events.
    pub(super) fn request(&mut self, request: Request, now: Time, limit: u32, events: &mut Queue<Event>) -> bool {
        let event = match request {
            Request::Person { .. } => return true,
            Request::Page { page, url, .. } => {
                self.navigate(page, &url);
                Event::Opened { page }
            }
            Request::Go { page, op, to: Go::Address(url) } => {
                self.navigate(page, &url);
                Event::Done { op }
            }
            Request::Find { page, op, query } if query.within.is_none() => {
                let (seen, more) = self.find(page, &query, limit);
                Event::Found { op, seen, more }
            }
            Request::Await { page, op, query, expect, within } if query.within.is_none() => {
                let wait = Await { page, op, query, expect, until: now.saturating_add(within) };
                if let Some(event) = self.awaited(&wait, now, limit) {
                    event
                } else {
                    self.awaits.push(wait);
                    return false;
                }
            }
            Request::Attribute { page, op, selector, name } => {
                let read = self
                    .shown(page)
                    .and_then(|shown| shown.elements.iter().find(|element| element.selector == selector))
                    .map_or(Read::NoElement, |element| {
                        element
                            .attributes
                            .iter()
                            .find(|(attribute, _)| *attribute == name)
                            .map_or(Read::NoAttribute, |(_, value)| Read::Value(value.clone()))
                    });
                Event::Attribute { op, read }
            }
            Request::Snapshot { page, op } => {
                let mut text = Vec::new();
                for (role, name) in self.shown(page).iter().flat_map(|shown| &shown.nodes) {
                    writeln!(text, "{} {}", String::from_utf8_lossy(role), String::from_utf8_lossy(name))
                        .expect("write to bytes");
                }
                Event::Snapshot { op, text: text.into_boxed_slice() }
            }
            Request::Close { entity } => {
                if entity == BROWSER {
                    self.pages.clear();
                    for wait in self.awaits.drain(..) {
                        events.push(refused(wait.op, "Await closed"));
                    }
                } else {
                    self.pages.remove(&entity);
                }
                Event::Closed { owner: entity }
            }
            Request::Find { op, .. } => refused(op, "scoped Find"),
            Request::Await { op, .. } => refused(op, "scoped Await"),
            Request::Go { op, .. } => refused(op, "Go without an address"),
            Request::Press { op, .. } => refused(op, "Press"),
            Request::Type { op, .. } => refused(op, "Type"),
            Request::Compose { op, .. } => refused(op, "Compose"),
            Request::Keys { op, .. } => refused(op, "Keys"),
            Request::Key { op, .. } => refused(op, "Key"),
            Request::Viewport { op, .. } => refused(op, "Viewport"),
            Request::Address { op, .. } => refused(op, "Address"),
            Request::Tabs { op, .. } => refused(op, "Tabs"),
            Request::Screenshot { op, .. } => refused(op, "Screenshot"),
        };
        events.push(event);
        false
    }

    fn awaited(&self, wait: &Await, now: Time, limit: u32) -> Option<Event> {
        let (seen, more) = self.find(wait.page, &wait.query, limit);
        let count = seen.len().saturating_add(more);
        let met = match wait.expect {
            Expect::Present => count > 0,
            Expect::Absent => count == 0,
            Expect::Count(wanted) => count == wanted,
        };
        if met {
            Some(Event::Met { op: wait.op, seen })
        } else if now >= wait.until {
            Some(Event::Missed { op: wait.op, seen, more })
        } else {
            None
        }
    }

    pub(super) fn fire(&mut self, now: Time, limit: u32, events: &mut Queue<Event>) {
        let mut index = 0;
        while let Some(wait) = self.awaits.get(index) {
            if let Some(event) = self.awaited(wait, now, limit) {
                self.awaits.remove(index);
                events.push(event);
            } else {
                index += 1;
            }
        }
    }

    pub(super) fn next_deadline(&self) -> Option<Time> {
        self.awaits.iter().map(|wait| wait.until).min()
    }
}

fn refused(op: Token, what: &str) -> Event {
    Event::Refused {
        op,
        why: Refusal::Protocol,
        detail: format!("scripted browser does not support {what}").into_bytes().into_boxed_slice(),
    }
}
