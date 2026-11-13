//! temper's addresses and the pages' filters (client/pages/README.md), and their paths (client/web/README.md, 2.3).
use alloc::boxed::Box;
use skein_lib::{Decimal, Writer};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Address {
    Inbox { filter: Filter },
    Chats { filter: ChatsFilter },
    NewChat { draft: bool },
    Accounts,
    Forges,
    ConnectForge { forge: Option<Choice> },
    ConnectModel { way: Option<ModelWay> },
    Welcome { step: Step },
    Chat { task: u64 },
    Task { task: u64 },
    Home { project: u32 },
    AllHome,
    Goals { project: u32, filter: GoalsFilter },
    Changes { project: u32 },
    Settings { project: u32 },
    Repositories { project: u32 },
    Models { project: u32 },
    Builders { project: u32 },
    Role { project: u32, charter: u32 },
    Rule { project: u32, kind: u32, number: u32 },
    Team { project: u32, role: Option<u64> },
    Ask { project: u32 },
    Adopt { project: u32 },
    Work { project: u32, filter: WorkFilter },
    Members,
    Remembered,
    Told,
    Limits,
}
/// The welcome's query names its step (client/web/README.md, 2.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// `/welcome`: decides by the store's state (client/pages/README.md).
    Welcome,
    /// `/welcome?at=welcome`: the same step, asked for by its link.
    WelcomeAsked,
    Forge {
        forge: Option<Choice>,
    },
    Model {
        way: Option<ModelWay>,
    },
    Ready,
}
/// A forge kind to make, a kept record, or the Forgejo instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    GitHub,
    Owned,
    Managed,
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
    Drafts,
    Closed,
}
/// The project goals' filter is its query (temper/docs/design/client/web/README.md, 2.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoalsFilter {
    All,
    Needs,
    Live,
    Ended,
}
/// Work's own filters; Goals and Changes have their own addresses (temper/docs/design/client/web/README.md, 2.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkFilter {
    All,
    Drafts,
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
    Finding,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    Address(Address),
    /// The approval page (client/pages/README.md, The approval page): a code, or none to type one.
    Approve {
        code: Option<jig_client::ApprovalCode>,
    },
    /// Raw authorization query; never canonicalized or printed.
    Authorize {
        query: Box<[u8]>,
    },
    /// An authorization query beyond jig's bound.
    AuthorizeMalformed,
    Missing,
    Public(Public),
}

/// The page's path (client/web/README.md, 2.3): `/`, `/chats`, `/chats/new`, `/chats/<task>`, `/tasks/<task>`,
/// `/projects/<project>/team`, `/projects/<project>/work`, `/deployment/members`, `/deployment/limits`, `/projects/<project>/goals`, `/projects/<project>`, `/projects/<project>/settings`, `/projects/<project>/settings/repositories`, `/projects/<project>/settings/models`, `/projects/<project>/settings/builders`, `/projects/<project>/settings/roles/<charter>`, `/projects/<project>/settings/rules/<kind>/<number>`, `/projects/<project>/ask`, `/projects/<project>/adopt`, `/accounts`. The pages add their set filter fields as a query.
#[must_use]
#[expect(clippy::too_many_lines, reason = "one closed address match prints each path and its query")]
pub fn print(address: &Address) -> Box<[u8]> {
    let (name, number, tail) = match address {
        Address::Inbox { filter } => return print_filter(*filter),
        Address::AllHome => (b"home".as_slice(), None, b"".as_slice()),
        Address::Welcome { step } => {
            return match step {
                Step::Welcome => b"/welcome".as_slice().into(),
                Step::WelcomeAsked => b"/welcome?at=welcome".as_slice().into(),
                Step::Forge { forge } => print_choice(b"/welcome?at=forge", *forge, b"&forge="),
                Step::Model { way } => print_way(b"/welcome?at=model", *way, b"&"),
                Step::Ready => b"/welcome?at=ready".as_slice().into(),
            };
        }
        Address::ConnectForge { forge } => return print_choice(b"/forges/connect", *forge, b"?forge="),
        Address::Forges => (b"forges".as_slice(), None, b"".as_slice()),
        Address::ConnectModel { way } => return print_way(b"/accounts/connect", *way, b"?"),
        Address::Accounts => (b"accounts".as_slice(), None, b"".as_slice()),
        Address::Told => (b"told".as_slice(), None, b"".as_slice()),
        Address::Remembered => (b"remembered".as_slice(), None, b"".as_slice()),
        Address::Members => (b"deployment/members".as_slice(), None, b"".as_slice()),
        Address::Limits => (b"deployment/limits".as_slice(), None, b"".as_slice()),
        Address::Chats { filter } => {
            return match filter {
                ChatsFilter::All => b"/chats".as_slice().into(),
                ChatsFilter::Live => b"/chats?state=live".as_slice().into(),
                ChatsFilter::Drafts => b"/chats?state=drafts".as_slice().into(),
                ChatsFilter::Closed => b"/chats?state=closed".as_slice().into(),
            };
        }
        Address::NewChat { draft } => {
            return if *draft { b"/chats/new?draft=1".as_slice().into() } else { b"/chats/new".as_slice().into() };
        }
        Address::Chat { task } => (b"chats".as_slice(), Some(*task), b"".as_slice()),
        Address::Task { task } => (b"tasks".as_slice(), Some(*task), b"".as_slice()),
        Address::Home { project } => (b"projects".as_slice(), Some(u64::from(*project)), b"".as_slice()),
        Address::Work { project, .. } => (b"projects".as_slice(), Some(u64::from(*project)), b"work".as_slice()),
        Address::Goals { project, .. } => (b"projects".as_slice(), Some(u64::from(*project)), b"goals".as_slice()),
        Address::Team { project, role: Some(role) } => {
            return skein_lib::bytes::concat(&[
                b"/projects/",
                Decimal::of(u64::from(*project)).as_bytes(),
                b"/team?role=",
                Decimal::of(*role).as_bytes(),
            ]);
        }
        Address::Team { project, role: None } => {
            (b"projects".as_slice(), Some(u64::from(*project)), b"team".as_slice())
        }
        Address::Changes { project } => (b"projects".as_slice(), Some(u64::from(*project)), b"changes".as_slice()),
        Address::Settings { project } => (b"projects".as_slice(), Some(u64::from(*project)), b"settings".as_slice()),
        Address::Models { project } => {
            (b"projects".as_slice(), Some(u64::from(*project)), b"settings/models".as_slice())
        }
        Address::Builders { project } => {
            (b"projects".as_slice(), Some(u64::from(*project)), b"settings/builders".as_slice())
        }
        Address::Role { project, charter } => {
            return skein_lib::bytes::concat(&[
                b"/projects/",
                Decimal::of(u64::from(*project)).as_bytes(),
                b"/settings/roles/",
                Decimal::of(u64::from(*charter)).as_bytes(),
            ]);
        }
        Address::Rule { project, kind, number } => {
            return skein_lib::bytes::concat(&[
                b"/projects/",
                Decimal::of(u64::from(*project)).as_bytes(),
                b"/settings/rules/",
                Decimal::of(u64::from(*kind)).as_bytes(),
                b"/",
                Decimal::of(u64::from(*number)).as_bytes(),
            ]);
        }
        Address::Repositories { project } => {
            (b"projects".as_slice(), Some(u64::from(*project)), b"settings/repositories".as_slice())
        }
        Address::Ask { project } => (b"projects".as_slice(), Some(u64::from(*project)), b"ask".as_slice()),
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
    let query = if let Address::Goals { filter, .. } = address {
        match filter {
            GoalsFilter::All => b"".as_slice(),
            GoalsFilter::Needs => b"?state=needs".as_slice(),
            GoalsFilter::Live => b"?state=live".as_slice(),
            GoalsFilter::Ended => b"?state=ended".as_slice(),
        }
    } else if let Address::Work { filter, .. } = address {
        match filter {
            WorkFilter::All => b"".as_slice(),
            WorkFilter::Drafts => b"?show=drafts".as_slice(),
        }
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
#[expect(clippy::too_many_lines, reason = "one codec reads each page alongside its project paths")]
pub fn parse(path: &[u8]) -> Place {
    if path == b"/public" || path.starts_with(b"/public/") || path.starts_with(b"/public?") {
        return Place::Public(parse_public(path).unwrap_or(Public::Projects));
    }
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
        (b"terminal", b"", b"") => return Place::Approve { code: approve_code(fields) },
        (b"forges", b"", b"") => Address::Forges,
        (b"forges", b"connect", b"") => Address::ConnectForge { forge: forge_choice(fields) },
        (b"accounts", b"", b"") => Address::Accounts,
        (b"home", b"", b"") => Address::AllHome,
        (b"deployment", b"members", b"") => Address::Members,
        (b"told", b"", b"") => Address::Told,
        (b"remembered", b"", b"") => Address::Remembered,
        (b"deployment", b"limits", b"") => Address::Limits,
        (b"accounts", b"connect", b"") => Address::ConnectModel { way: parse_way(fields) },
        (b"welcome", b"", b"") => Address::Welcome { step: welcome_step(fields) },
        (b"chats", b"", b"") => Address::Chats { filter: parse_chats(fields) },
        (b"chats", b"new", b"") => Address::NewChat { draft: draft_query(fields) },
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
            Address::Task { task }
        }
        (b"projects", _, b"") => {
            let Some(project) = project_decimal(number) else { return Place::Missing };
            Address::Home { project }
        }
        (b"projects", _, b"work") => {
            let Some(project) = project_decimal(number) else { return Place::Missing };
            Address::Work { project, filter: parse_work(fields) }
        }
        (b"projects", _, b"goals") => {
            let Some(project) = project_decimal(number) else { return Place::Missing };
            Address::Goals { project, filter: parse_goals(fields) }
        }
        (b"projects", _, b"team") => {
            let Some(project) = project_decimal(number) else { return Place::Missing };
            Address::Team { project, role: parse_team(fields) }
        }
        (b"projects", _, b"changes") => {
            let Some(project) = project_decimal(number) else { return Place::Missing };
            Address::Changes { project }
        }
        (b"projects", _, b"settings") => {
            let Some(project) = project_decimal(number) else { return Place::Missing };
            Address::Settings { project }
        }
        (b"projects", _, b"settings/models") => {
            let Some(project) = project_decimal(number) else { return Place::Missing };
            Address::Models { project }
        }
        (b"projects", _, b"settings/builders") => {
            let Some(project) = project_decimal(number) else { return Place::Missing };
            Address::Builders { project }
        }
        (b"projects", _, tail) if tail.starts_with(b"settings/roles/") => {
            let Some(project) = project_decimal(number) else { return Place::Missing };
            let Some(charter) = natural(tail.strip_prefix(b"settings/roles/").expect("role prefix")) else {
                return Place::Missing;
            };
            Address::Role { project, charter }
        }
        (b"projects", _, tail) if tail.starts_with(b"settings/rules/") => {
            let Some(project) = project_decimal(number) else { return Place::Missing };
            let (kind, number) = segment(tail.strip_prefix(b"settings/rules/").expect("rule prefix"));
            let Some(kind) = project_decimal(kind) else { return Place::Missing };
            if kind != 1 && kind != 2 {
                return Place::Missing;
            }
            let Some(number) = natural(number) else { return Place::Missing };
            Address::Rule { project, kind, number }
        }
        (b"projects", _, b"settings/repositories") => {
            let Some(project) = project_decimal(number) else { return Place::Missing };
            Address::Repositories { project }
        }
        (b"projects", _, b"ask") => {
            let Some(project) = project_decimal(number) else { return Place::Missing };
            Address::Ask { project }
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

fn parse_chats(query: &[u8]) -> ChatsFilter {
    let mut filter = ChatsFilter::All;
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
                b"state=drafts" => ChatsFilter::Drafts,
                b"state=closed" => ChatsFilter::Closed,
                _ => ChatsFilter::All,
            };
        }
    }
    filter
}

fn parse_work(query: &[u8]) -> WorkFilter {
    let mut rest = query;
    for _ in 0..query.len() {
        if rest.is_empty() {
            break;
        }
        let end = skein_lib::bytes::find(rest, b"&").unwrap_or(rest.len());
        let field = rest.get(..end).expect("separator in query");
        rest = rest.get(end..).expect("separator in query").strip_prefix(b"&").unwrap_or(b"");
        if field == b"show=drafts" {
            return WorkFilter::Drafts;
        }
    }
    WorkFilter::All
}

fn parse_team(query: &[u8]) -> Option<u64> {
    let mut rest = query;
    for _ in 0..query.len() {
        if rest.is_empty() {
            break;
        }
        let end = skein_lib::bytes::find(rest, b"&").unwrap_or(rest.len());
        let field = rest.get(..end).expect("separator in query");
        rest = rest.get(end..).expect("separator in query").strip_prefix(b"&").unwrap_or(b"");
        if let Some(value) = field.strip_prefix(b"role=")
            && let Some(role) = decimal(value)
        {
            return Some(role);
        }
    }
    None
}

fn parse_goals(query: &[u8]) -> GoalsFilter {
    let mut rest = query;
    for _ in 0..query.len() {
        if rest.is_empty() {
            break;
        }
        let end = skein_lib::bytes::find(rest, b"&").unwrap_or(rest.len());
        let field = rest.get(..end).expect("separator in query");
        rest = rest.get(end..).expect("separator in query").strip_prefix(b"&").unwrap_or(b"");
        if field == b"state=needs" {
            return GoalsFilter::Needs;
        }
        if field == b"state=live" {
            return GoalsFilter::Live;
        }
        if field == b"state=ended" {
            return GoalsFilter::Ended;
        }
    }
    GoalsFilter::All
}

fn print_filter(filter: Filter) -> Box<[u8]> {
    let project = Decimal::of(u64::from(filter.project.unwrap_or(0)));
    let goal = Decimal::of(filter.goal.unwrap_or(0));
    let kind = match filter.kind {
        Some(Kind::Proposal) => b"proposal".as_slice(),
        Some(Kind::Question) => b"question".as_slice(),
        Some(Kind::Held) => b"held".as_slice(),
        Some(Kind::PersonTask) => b"person-task".as_slice(),
        Some(Kind::Finding) => b"finding".as_slice(),
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
                    b"finding" => Some(Kind::Finding),
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

fn natural(bytes: &[u8]) -> Option<u32> {
    let number = project_decimal(bytes)?;
    if number == 0 { None } else { Some(number) }
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
                b"welcome" => (Step::WelcomeAsked, true),
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
        Choice::GitHub => b"github",
        Choice::Owned => b"owned",
        Choice::Managed => b"managed",
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
            // A value that doesn't read is dropped, so a repeat after it is the first that reads (client/web/README.md, 2.3).
            choice = if value == b"github" {
                Some(Choice::GitHub)
            } else if value == b"owned" {
                Some(Choice::Owned)
            } else if value == b"managed" {
                Some(Choice::Managed)
            } else if value == b"elsewhere" {
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

/// A public page, its identifiers public ones (jig/docs/design/domain/people/public-reading.md, 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Public {
    Projects,
    Board { project: u64 },
    Task { project: u64, task: u64 },
    Landings { project: u64 },
    Landing { project: u64, task: u64 },
}

/// A public page's canonical path (temper/docs/design/client/web/public.md, 1).
#[must_use]
pub fn print_public(page: &Public) -> Box<[u8]> {
    let (project, tail, task) = match *page {
        Public::Projects => return b"/public".as_slice().into(),
        Public::Board { project } => (project, b"".as_slice(), None),
        Public::Task { project, task } => (project, b"/tasks/".as_slice(), Some(task)),
        Public::Landings { project } => (project, b"/landings".as_slice(), None),
        Public::Landing { project, task } => (project, b"/landings/".as_slice(), Some(task)),
    };
    let project = public_digits(project);
    let task_digits = public_digits(task.unwrap_or(0));
    let task = if task.is_some() { task_digits.as_slice() } else { b"" };
    let parts = [b"/public/projects/".as_slice(), project.as_slice(), tail, task];
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("public path fits");
    }
    let mut path = Writer::new(size);
    for part in parts {
        path.put(part).expect("measured public path");
    }
    path.finish()
}

fn parse_public(path: &[u8]) -> Option<Public> {
    if path == b"/public" {
        return Some(Public::Projects);
    }
    let rest = path.strip_prefix(b"/public/projects/")?;
    let (project, rest) = segment(rest);
    let project = public_id(project)?;
    let page = if rest.is_empty() {
        Public::Board { project }
    } else {
        let (kind, rest) = segment(rest);
        match (kind, rest) {
            (b"landings", b"") => Public::Landings { project },
            (b"landings", task) => Public::Landing { project, task: public_id(task)? },
            (b"tasks", task) => Public::Task { project, task: public_id(task)? },
            _ => return None,
        }
    };
    if print_public(&page).as_ref() == path { Some(page) } else { None }
}
fn public_id(bytes: &[u8]) -> Option<u64> {
    if bytes.len() != 16 {
        return None;
    }
    let mut id = 0_u64;
    for byte in bytes {
        let digit = match byte {
            b'0'..=b'9' => byte.checked_sub(b'0')?,
            b'a'..=b'f' => byte.checked_sub(b'a')?.checked_add(10)?,
            _ => return None,
        };
        id = id.checked_mul(16)?.checked_add(u64::from(digit))?;
    }
    Some(id)
}

fn public_digits(id: u64) -> [u8; 16] {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut digits = [0; 16];
    for (byte, pair) in id.to_be_bytes().iter().zip(digits.as_chunks_mut::<2>().0.iter_mut()) {
        for (digit, nibble) in pair.iter_mut().zip([byte >> 4_u8, byte & 15_u8]) {
            *digit = *HEX.get(usize::from(nibble)).expect("nibble in hex alphabet");
        }
    }
    digits
}

fn draft_query(query: &[u8]) -> bool {
    let mut rest = query;
    for _ in 0..query.len() {
        if rest.is_empty() {
            break;
        }
        let end = skein_lib::bytes::find(rest, b"&").unwrap_or(rest.len());
        let field = rest.get(..end).expect("separator in query");
        if field == b"draft=1" {
            return true;
        }
        rest = rest.get(end..).expect("separator in query").strip_prefix(b"&").unwrap_or(b"");
    }
    false
}

/// The approval page's code as shown (client/web/README.md, 2.3).
#[must_use]
pub fn print_approve(code: Option<jig_client::ApprovalCode>) -> Box<[u8]> {
    let Some(code) = code else { return b"/terminal".as_slice().into() };
    let base = b"/terminal?code=";
    let shown = code.shown();
    let size = base.len().checked_add(shown.len()).expect("approval path fits");
    let mut out = Writer::new(size);
    out.put(base).expect("measured approval path");
    out.put(&shown).expect("measured approval code");
    out.finish()
}
fn approve_code(query: &[u8]) -> Option<jig_client::ApprovalCode> {
    let mut rest = query;
    for _ in 0..query.len() {
        if rest.is_empty() {
            break;
        }
        let end = skein_lib::bytes::find(rest, b"&").unwrap_or(rest.len());
        let field = rest.get(..end).expect("separator in query");
        rest = rest.get(end..).expect("separator in query").strip_prefix(b"&").unwrap_or(b"");
        if let Some(value) = field.strip_prefix(b"code=")
            && let Some(code) = jig_client::ApprovalCode::parse(value)
        {
            return Some(code);
        }
    }
    None
}
