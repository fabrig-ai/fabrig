//! temper's addresses and the pages' filters (client/README.md, 3.5), and their paths (client/browser.md, 2.3).
use alloc::boxed::Box;
use skein_lib::{Decimal, Writer};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Address {
    Inbox { filter: Filter },
    Chats { filter: ChatsFilter, start: bool },
    Accounts,
    Forges,
    ConnectForge { forge: Option<Choice> },
    ConnectModel { way: Option<ModelWay> },
    Welcome { step: Step },
    Chat { task: u64 },
    Task { task: u64, filter: PlanFilter },
    Adopt { project: u32 },
}
/// The welcome's query names its step (client/browser.md, 2.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Welcome,
    Forge { forge: Option<Choice> },
    Model { way: Option<ModelWay> },
    Ready,
}
/// A configured forge, or the instructions for adding a Forgejo elsewhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Forge(u16),
    Elsewhere,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelWay {
    pub source: Option<u32>,
    pub provider: u32,
    pub kind: WayKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WayKind {
    SignIn,
    Program,
    Key,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatsFilter {
    All,
    Live,
    Closed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanFilter {
    All,
    Live,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Filter {
    pub project: Option<u32>,
    pub kind: Option<Kind>,
    pub goal: Option<u64>,
}
impl Filter {
    pub const ALL: Self = Self { project: None, kind: None, goal: None };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Proposal,
    Question,
    Held,
    PersonTask,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    Address(Address),
    /// Raw authorization query; never canonicalized or printed.
    Authorize {
        query: Box<[u8]>,
    },
    /// An authorization query beyond jig's bound.
    AuthorizeMalformed,
    Missing,
}

/// The page's path (client/browser.md, 2.3): `/`, `/chats`, `/chats/<task>`, `/tasks/<task>`,
/// `/projects/<project>/adopt`, `/accounts`. The pages add their set filter fields as a query.
#[must_use]
pub fn print(address: &Address) -> Box<[u8]> {
    let (name, number, tail) = match address {
        Address::Inbox { filter } => return print_filter(*filter),
        Address::Welcome { step } => {
            return match step {
                Step::Welcome => b"/welcome".as_slice().into(),
                Step::Forge { forge } => print_choice(b"/welcome?at=forge", *forge, b"&forge="),
                Step::Model { way } => print_way(b"/welcome?at=model", *way, b"&"),
                Step::Ready => b"/welcome?at=ready".as_slice().into(),
            };
        }
        Address::ConnectForge { forge } => return print_choice(b"/forges/connect", *forge, b"?forge="),
        Address::Forges => (b"forges".as_slice(), None, b"".as_slice()),
        Address::ConnectModel { way } => return print_way(b"/accounts/connect", *way, b"?"),
        Address::Accounts => (b"accounts".as_slice(), None, b"".as_slice()),
        Address::Chats { filter, start } => {
            return match (filter, start) {
                (ChatsFilter::All, false) => b"/chats".as_slice().into(),
                (ChatsFilter::Live, false) => b"/chats?state=live".as_slice().into(),
                (ChatsFilter::Closed, false) => b"/chats?state=closed".as_slice().into(),
                (ChatsFilter::All, true) => b"/chats?at=start".as_slice().into(),
                (ChatsFilter::Live, true) => b"/chats?state=live&at=start".as_slice().into(),
                (ChatsFilter::Closed, true) => b"/chats?state=closed&at=start".as_slice().into(),
            };
        }
        Address::Chat { task } => (b"chats".as_slice(), Some(*task), b"".as_slice()),
        Address::Task { task, .. } => (b"tasks".as_slice(), Some(*task), b"".as_slice()),
        Address::Adopt { project } => (b"projects".as_slice(), Some(u64::from(*project)), b"adopt".as_slice()),
    };
    let digits = Decimal::of(number.unwrap_or(0));
    let number = if number.is_some() { digits.as_bytes() } else { b"" };
    let parts = [name, number, tail];
    let mut size = 0_usize;
    for part in parts {
        if !part.is_empty() {
            size = size.checked_add(part.len()).expect("path fits usize");
            size = size.checked_add(1).expect("path fits usize");
        }
    }
    let query = if let Address::Task { filter: PlanFilter::Live, .. } = address {
        b"?state=live".as_slice()
    } else {
        b"".as_slice()
    };
    size = size.checked_add(query.len()).expect("query fits usize");
    let mut path = Writer::new(size);
    for part in parts {
        if !part.is_empty() {
            path.put(b"/").expect("separator fits");
            path.put(part).expect("part fits");
        }
    }
    path.put(query).expect("query fits");
    path.finish()
}

/// The page a path names, reading the page's query and ignoring other queries and one trailing slash.
/// Unreadable query fields are dropped; any other path is `Place::Missing`.
#[must_use]
pub fn parse(path: &[u8]) -> Place {
    let query = skein_lib::bytes::find(path, b"?").unwrap_or(path.len());
    let fields = path.get(query..).expect("query in path").strip_prefix(b"?").unwrap_or(b"");
    let route = path.get(..query).expect("query in path");
    if route == b"/oauth/authorize" {
        return if fields.len() <= usize::try_from(crate::authorize::QUERY_BYTES).expect("jig query ceiling fits usize")
        {
            Place::Authorize { query: fields.into() }
        } else {
            Place::AuthorizeMalformed
        };
    }
    let path = route;
    let path = path.strip_suffix(b"/").unwrap_or(path);
    if path.is_empty() {
        return Place::Address(Address::Inbox { filter: parse_filter(fields) });
    }
    let Some(path) = path.strip_prefix(b"/") else {
        return Place::Missing;
    };
    if path.ends_with(b"/") {
        return Place::Missing;
    }
    let (name, rest) = segment(path);
    let (number, tail) = segment(rest);
    let address = match (name, number, tail) {
        (b"forges", b"", b"") => Address::Forges,
        (b"forges", b"connect", b"") => Address::ConnectForge { forge: forge_choice(fields) },
        (b"accounts", b"", b"") => Address::Accounts,
        (b"accounts", b"connect", b"") => Address::ConnectModel { way: parse_way(fields) },
        (b"welcome", b"", b"") => Address::Welcome { step: welcome_step(fields) },
        (b"chats", b"", b"") => {
            let (filter, start) = parse_chats(fields);
            Address::Chats { filter, start }
        }
        (b"chats", _, b"") => {
            let Some(task) = decimal(number) else {
                return Place::Missing;
            };
            Address::Chat { task }
        }
        (b"tasks", _, b"") => {
            let Some(task) = decimal(number) else {
                return Place::Missing;
            };
            Address::Task { task, filter: parse_plan(fields) }
        }
        (b"projects", _, b"adopt") => {
            let Some(project) = project_decimal(number) else {
                return Place::Missing;
            };
            Address::Adopt { project }
        }
        _ => return Place::Missing,
    };
    Place::Address(address)
}

fn parse_chats(query: &[u8]) -> (ChatsFilter, bool) {
    let mut filter = ChatsFilter::All;
    let mut start = false;
    let mut rest = query;
    for _ in 0..query.len() {
        if rest.is_empty() {
            break;
        }
        let end = skein_lib::bytes::find(rest, b"&").unwrap_or(rest.len());
        let field = rest.get(..end).expect("separator in query");
        rest = rest.get(end..).expect("separator in query").strip_prefix(b"&").unwrap_or(b"");
        if filter == ChatsFilter::All {
            filter = match field {
                b"state=live" => ChatsFilter::Live,
                b"state=closed" => ChatsFilter::Closed,
                _ => ChatsFilter::All,
            };
        }
        start |= field == b"at=start";
    }
    (filter, start)
}

fn parse_plan(query: &[u8]) -> PlanFilter {
    let mut rest = query;
    for _ in 0..query.len() {
        if rest.is_empty() {
            break;
        }
        let end = skein_lib::bytes::find(rest, b"&").unwrap_or(rest.len());
        let field = rest.get(..end).expect("separator in query");
        rest = rest.get(end..).expect("separator in query").strip_prefix(b"&").unwrap_or(b"");
        if field == b"state=live" {
            return PlanFilter::Live;
        }
    }
    PlanFilter::All
}

fn print_filter(filter: Filter) -> Box<[u8]> {
    let project = Decimal::of(u64::from(filter.project.unwrap_or(0)));
    let goal = Decimal::of(filter.goal.unwrap_or(0));
    let kind = match filter.kind {
        Some(Kind::Proposal) => b"proposal".as_slice(),
        Some(Kind::Question) => b"question".as_slice(),
        Some(Kind::Held) => b"held".as_slice(),
        Some(Kind::PersonTask) => b"person-task".as_slice(),
        None => b"".as_slice(),
    };
    let fields = [
        (b"project=".as_slice(), project.as_bytes(), filter.project.is_some()),
        (b"kind=".as_slice(), kind, filter.kind.is_some()),
        (b"goal=".as_slice(), goal.as_bytes(), filter.goal.is_some()),
    ];
    let mut size = 1_usize;
    for (name, value, set) in fields {
        if set {
            size = size.checked_add(1).expect("query fits usize");
            size = size.checked_add(name.len()).expect("query fits usize");
            size = size.checked_add(value.len()).expect("query fits usize");
        }
    }
    let mut path = Writer::new(size);
    path.put(b"/").expect("root fits");
    let mut separator = b"?";
    for (name, value, set) in fields {
        if set {
            path.put(separator).expect("separator fits");
            path.put(name).expect("field name fits");
            path.put(value).expect("field value fits");
            separator = b"&";
        }
    }
    path.finish()
}

fn parse_filter(query: &[u8]) -> Filter {
    let mut filter = Filter::ALL;
    let mut rest = query;
    for _ in 0..query.len() {
        if rest.is_empty() {
            break;
        }
        let end = skein_lib::bytes::find(rest, b"&").unwrap_or(rest.len());
        let field = rest.get(..end).expect("separator in query");
        rest = rest.get(end..).expect("separator in query").strip_prefix(b"&").unwrap_or(b"");
        let Some(split) = skein_lib::bytes::find(field, b"=") else {
            continue;
        };
        let name = field.get(..split).expect("separator in field");
        let value = field.get(split..).expect("separator in field").strip_prefix(b"=").expect("field separator");
        match name {
            b"project" if filter.project.is_none() => filter.project = project_decimal(value),
            b"goal" if filter.goal.is_none() => filter.goal = decimal(value),
            b"kind" if filter.kind.is_none() => {
                filter.kind = match value {
                    b"proposal" => Some(Kind::Proposal),
                    b"question" => Some(Kind::Question),
                    b"held" => Some(Kind::Held),
                    b"person-task" => Some(Kind::PersonTask),
                    _ => None,
                };
            }
            _ => {}
        }
    }
    filter
}

fn segment(path: &[u8]) -> (&[u8], &[u8]) {
    let split = skein_lib::bytes::find(path, b"/").unwrap_or(path.len());
    let first = path.get(..split).expect("separator in path");
    let rest = path.get(split..).expect("separator in path").strip_prefix(b"/").unwrap_or(b"");
    (first, rest)
}

fn decimal(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() || bytes.len() > 20 || (bytes.len() > 1 && bytes.starts_with(b"0")) {
        return None;
    }
    let mut value = 0_u64;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}

fn project_decimal(bytes: &[u8]) -> Option<u32> {
    let number = decimal(bytes)?;
    u32::try_from(number).ok()
}

fn welcome_step(query: &[u8]) -> Step {
    let mut step = Step::Welcome;
    let mut seen = false;
    let mut rest = query;
    for _ in 0..query.len() {
        if rest.is_empty() {
            break;
        }
        let end = skein_lib::bytes::find(rest, b"&").unwrap_or(rest.len());
        let field = rest.get(..end).expect("separator in query");
        rest = rest.get(end..).expect("separator in query").strip_prefix(b"&").unwrap_or(b"");
        if let Some(value) = field.strip_prefix(b"at=") {
            if seen {
                continue;
            }
            (step, seen) = match value {
                b"forge" => (Step::Forge { forge: forge_choice(query) }, true),
                b"model" => (Step::Model { way: parse_way(query) }, true),
                b"ready" => (Step::Ready, true),
                _ => (Step::Welcome, false),
            };
        }
    }
    step
}

fn print_choice(base: &[u8], choice: Option<Choice>, separator: &[u8]) -> Box<[u8]> {
    let Some(choice) = choice else { return base.into() };
    let digits;
    let value = match choice {
        Choice::Forge(number) => {
            digits = Decimal::of(u64::from(number));
            digits.as_bytes()
        }
        Choice::Elsewhere => b"elsewhere",
    };
    let size = base
        .len()
        .checked_add(separator.len())
        .expect("choice path fits")
        .checked_add(value.len())
        .expect("choice path fits");
    let mut out = Writer::new(size);
    for part in [base, separator, value] {
        out.put(part).expect("measured choice path");
    }
    out.finish()
}
fn forge_choice(query: &[u8]) -> Option<Choice> {
    let mut rest = query;
    let mut choice = None;
    for _ in 0..query.len() {
        if rest.is_empty() {
            break;
        }
        let end = skein_lib::bytes::find(rest, b"&").unwrap_or(rest.len());
        let field = rest.get(..end).expect("separator in query");
        rest = rest.get(end..).expect("separator in query").strip_prefix(b"&").unwrap_or(b"");
        if let Some(value) = field.strip_prefix(b"forge=") {
            if choice.is_some() {
                continue;
            }
            // A value that doesn't read is dropped, so a repeat after it is the first that reads (browser.md, 2.3).
            choice = if value == b"elsewhere" {
                Some(Choice::Elsewhere)
            } else {
                match decimal(value) {
                    Some(value) => match u16::try_from(value) {
                        Ok(value) => Some(Choice::Forge(value)),
                        Err(_) => None,
                    },
                    None => None,
                }
            };
        }
    }
    choice
}

fn print_way(base: &[u8], way: Option<ModelWay>, separator: &[u8]) -> Box<[u8]> {
    let Some(way) = way else { return skein_lib::bytes::copy_of(base) };
    let provider = Decimal::of(u64::from(way.provider));
    let kind = match way.kind {
        WayKind::SignIn => b"sign-in".as_slice(),
        WayKind::Program => b"program",
        WayKind::Key => b"key",
    };
    let source = match (way.kind, way.source) {
        (WayKind::SignIn, Some(source)) => Some(Decimal::of(u64::from(source))),
        _ => None,
    };
    let (source_name, source_value) = match &source {
        Some(source) => (b"&source=".as_slice(), source.as_bytes()),
        None => (b"".as_slice(), b"".as_slice()),
    };
    let parts = [base, separator, b"provider=", provider.as_bytes(), b"&way=", kind, source_name, source_value];
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("way address fits");
    }
    let mut path = Writer::new(size);
    for part in parts {
        path.put(part).expect("measured way fits");
    }
    path.finish()
}
fn parse_way(query: &[u8]) -> Option<ModelWay> {
    let mut provider = None;
    let mut kind = None;
    let mut source = None;
    let mut rest = query;
    for _ in 0..query.len() {
        if rest.is_empty() {
            break;
        }
        let end = skein_lib::bytes::find(rest, b"&").unwrap_or(rest.len());
        let field = rest.get(..end).expect("separator in query");
        rest = rest.get(end..).expect("separator in query").strip_prefix(b"&").unwrap_or(b"");
        if let Some(value) = field.strip_prefix(b"provider=") {
            if provider.is_none() {
                provider = project_decimal(value);
            }
        } else if let Some(value) = field.strip_prefix(b"source=") {
            if source.is_none() {
                source = project_decimal(value);
            }
        } else if let Some(value) = field.strip_prefix(b"way=")
            && kind.is_none()
        {
            kind = match value {
                b"sign-in" => Some(WayKind::SignIn),
                b"program" => Some(WayKind::Program),
                b"key" => Some(WayKind::Key),
                _ => None,
            };
        }
    }
    let kind = kind?;
    Some(ModelWay { provider: provider?, kind, source: if kind == WayKind::SignIn { source } else { None } })
}
