//! Volatile public connects; only handles cross this boundary (client/engine.md).
use crate::{AcceptAnswer, Answered, Ask, ConnectAnswer, Connection, Domain, ForgeRefusal, InstallAnswer, Request};
use alloc::boxed::Box;
use jig_domain_core::Carried;
use jig_domain_people::Role;
use skein_lib::{Map, Queue, Token};

/// The run's admission determines its end (temper/docs/design/protocol/forge/engine.md, 2).
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
enum RunKind {
    Admin,
    Tokens,
    Replace,
    App,
}

#[derive(Hash, Debug)]
pub(crate) struct State {
    pub(crate) answers: Map<Token, Answered>,
    pending: Map<Token, Token>,
    running: Map<u16, (Token, Option<Carried>, RunKind)>,
    reading: Map<u16, Token>,
    previous: Map<u16, (Connection, Option<crate::FailedStep>)>,
}
impl State {
    pub(crate) fn new(bound: u32) -> Self {
        Self {
            answers: Map::with_capacity(bound),
            pending: Map::with_capacity(bound),
            running: Map::with_capacity(bound),
            reading: Map::with_capacity(bound),
            previous: Map::with_capacity(bound),
        }
    }
    pub(crate) fn answer(&mut self, request: Token, token: Token, answer: Answered, out: &mut Queue<Request>) {
        if self.answers.len() == self.answers.capacity() && !self.answers.contains_key(&token) {
            let oldest = *self.answers.first().expect("full answers").0;
            self.answers.remove(&oldest);
        }
        self.answers.insert(token, answer).expect("bounded answers");
        out.push(Request::ApplicationDone { request, token });
    }
}
#[expect(clippy::too_many_lines, reason = "one admission dispatcher checks each public connect request")]
pub(crate) fn start(
    d: &mut Domain,
    request: Token,
    token: Token,
    ask: Ask,
    role: Option<Role>,
    carried: Option<Carried>,
    out: &mut Queue<Request>,
) {
    let ask = match resolve(d, ask, role, carried, out) {
        Ok(ask) => ask,
        Err(why) => {
            refused(d, request, token, Answered::Connect(ConnectAnswer::Refused(why)), carried, out);
            return;
        }
    };
    let answer = match &ask {
        Ask::RemoveForge { .. } => Some(Answered::RemoveForge(crate::RemoveAnswer::Refused(ForgeRefusal::NoSuchForge))),
        Ask::StartPlayground => Some(Answered::StartPlayground(crate::StartAnswer::NotOffered)),
        Ask::EndPlayground => Some(Answered::EndPlayground(crate::EndAnswer::NotOffered)),
        Ask::Connect { .. }
        | Ask::Replace { .. }
        | Ask::DismissMaking { .. }
        | Ask::MakeRepository { .. }
        | Ask::SetDraft(_)
        | Ask::Adopt(_)
        | Ask::SetUp { .. }
        | Ask::Install { .. }
        | Ask::Accept { .. }
        | Ask::Reachable { .. } => None,
    };
    if let Some(answer) = answer {
        refused(d, request, token, answer, carried, out);
        return;
    }
    if let Ask::Reachable { forge, installation, page } = ask {
        reachable(d, request, token, forge, installation, page, role, carried, out);
        return;
    }
    if let Ask::Connect { forge: crate::Making::Kept(forge), how: how @ crate::How::ForgejoAdmin { .. } } = ask {
        admin(d, request, token, forge, how, role, carried, out);
        return;
    }
    if let Ask::Connect { forge: crate::Making::Kept(forge), how: crate::How::ForgejoTokens } = ask {
        tokens_or_replace(d, request, token, forge, RunKind::Tokens, role, carried, out);
        return;
    }
    if let Ask::Replace { forge } = ask {
        tokens_or_replace(d, request, token, forge, RunKind::Replace, role, carried, out);
        return;
    }
    let (forge, install, manifest) = match &ask {
        Ask::Replace { .. } | Ask::Connect { how: crate::How::ForgejoTokens, .. } => {
            unreachable!("tokens and Replace use their own admission")
        }
        Ask::Connect { forge: crate::Making::Kept(forge), how: crate::How::GitHubApp { .. } } => (*forge, false, false),
        Ask::Connect { forge: crate::Making::Kept(forge), how: crate::How::GitHubManifest { .. } } => {
            (*forge, false, true)
        }
        Ask::Install { forge, .. } | Ask::Accept { forge, .. } => (*forge, true, false),
        Ask::Connect { forge: crate::Making::New(_), .. }
        | Ask::Connect { how: crate::How::ForgejoAdmin { .. }, .. }
        | Ask::Reachable { .. }
        | Ask::DismissMaking { .. }
        | Ask::MakeRepository { .. }
        | Ask::SetDraft(_)
        | Ask::Adopt(_)
        | Ask::SetUp { .. }
        | Ask::RemoveForge { .. }
        | Ask::StartPlayground
        | Ask::EndPlayground => unreachable!("connect admission"),
    };
    let (connected, connecting) = match d.connection(forge) {
        Connection::Connected { .. } => (true, false),
        Connection::Connecting => (false, true),
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Stopped { .. } => (false, false),
    };
    let refusal = if role != Some(Role::Owner) {
        Some(ForgeRefusal::Role)
    } else if !d.configured.contains_key(&forge) {
        Some(ForgeRefusal::NoSuchForge)
    } else if let Some(config) = d.configured.get(&forge)
        && (config.kind != crate::Kind::GitHub || config.mode == crate::Mode::Fake || manifest && config.read_only)
    {
        Some(ForgeRefusal::NotTaken)
    } else if d.connect.running.contains_key(&forge)
        || d.connect.pending.len() == d.connect.pending.capacity()
        || d.connect.running.len() == d.connect.running.capacity()
    {
        Some(ForgeRefusal::Busy)
    } else if install && !connected {
        Some(ForgeRefusal::NotConnected)
    } else if !install && (connected || connecting) {
        Some(ForgeRefusal::Connected)
    } else if !install && !manifest && carried.is_none() {
        Some(ForgeRefusal::NoKey)
    } else {
        None
    };
    if let Some(why) = refusal {
        if let Some(value) = carried {
            out.push(Request::CarriedUnused { number: value.number });
        }
        let answer = if let Ask::Accept { .. } = ask {
            Answered::Accept(AcceptAnswer::Refused(why))
        } else if install {
            Answered::Install(InstallAnswer::Refused(why))
        } else {
            Answered::Connect(ConnectAnswer::Refused(why))
        };
        d.connect.answer(request, token, answer, out);
        return;
    }
    if d.ended.remove(&forge).is_some() {
        out.push(Request::MarkedForges);
    }
    d.connect.pending.insert(request, token).expect("room checked");
    send(d, request, forge, ask, carried, out);
}
fn refused(
    d: &mut Domain,
    request: Token,
    token: Token,
    answer: Answered,
    carried: Option<Carried>,
    out: &mut Queue<Request>,
) {
    if let Some(value) = carried {
        out.push(Request::CarriedUnused { number: value.number });
    }
    d.connect.answer(request, token, answer, out);
}
fn resolve(
    d: &mut Domain,
    ask: Ask,
    role: Option<Role>,
    carried: Option<Carried>,
    out: &mut Queue<Request>,
) -> Result<Ask, ForgeRefusal> {
    match ask {
        Ask::Connect { forge: crate::Making::New(forge), how } => {
            let forge = make(d, forge, &how, role, carried, out)?;
            Ok(Ask::Connect { forge: crate::Making::Kept(forge), how })
        }
        ask @ (Ask::Connect { forge: crate::Making::Kept(_), .. }
        | Ask::RemoveForge { .. }
        | Ask::StartPlayground
        | Ask::EndPlayground
        | Ask::DismissMaking { .. }
        | Ask::MakeRepository { .. }
        | Ask::SetDraft(_)
        | Ask::Adopt(_)
        | Ask::SetUp { .. }
        | Ask::Install { .. }
        | Ask::Accept { .. }
        | Ask::Reachable { .. }
        | Ask::Replace { .. }) => Ok(ask),
    }
}
/// Admission precedes the first save (temper/docs/design/protocol/forge/engine.md, 2; temper/docs/design/protocol/setup/owned.md, 7).
fn make(
    d: &mut Domain,
    forge: crate::ForgeAsk,
    how: &crate::How,
    role: Option<Role>,
    carried: Option<Carried>,
    out: &mut Queue<Request>,
) -> Result<u16, ForgeRefusal> {
    if role != Some(Role::Owner) {
        return Err(ForgeRefusal::Role);
    }
    let (kind, mode, place, git_program) = match forge {
        crate::ForgeAsk::GitHub => {
            match how {
                crate::How::GitHubApp { .. } | crate::How::GitHubManifest { .. } => {}
                crate::How::ForgejoAdmin { .. } | crate::How::ForgejoTokens => return Err(ForgeRefusal::NotTaken),
            }
            let Some(machine) = &d.machine else { return Err(ForgeRefusal::NoSuchForge) };
            (crate::Kind::GitHub, crate::Mode::Connected, machine.github.clone(), machine.github_git_program.clone())
        }
        crate::ForgeAsk::Managed { place, git_program } => {
            match how {
                crate::How::ForgejoAdmin { .. } => {}
                crate::How::GitHubApp { .. } | crate::How::GitHubManifest { .. } | crate::How::ForgejoTokens => {
                    return Err(ForgeRefusal::NotTaken);
                }
            }
            (crate::Kind::Forgejo, crate::Mode::Managed, place, git_program)
        }
        crate::ForgeAsk::Connected { place, git_program } => {
            match how {
                crate::How::ForgejoTokens => {}
                crate::How::ForgejoAdmin { .. } | crate::How::GitHubApp { .. } | crate::How::GitHubManifest { .. } => {
                    return Err(ForgeRefusal::NotTaken);
                }
            }
            if carried.is_none() {
                return Err(ForgeRefusal::NoKey);
            }
            (crate::Kind::Forgejo, crate::Mode::Connected, place, git_program)
        }
        crate::ForgeAsk::Owned => return Err(ForgeRefusal::NotTaken),
    };
    if d.connect.pending.len() == d.connect.pending.capacity()
        || d.connect.running.len() == d.connect.running.capacity()
    {
        return Err(ForgeRefusal::Busy);
    }
    let first = match how {
        crate::How::GitHubApp { .. } => {
            if carried.is_none() {
                return Err(ForgeRefusal::NoKey);
            }
            None
        }
        crate::How::ForgejoTokens | crate::How::GitHubManifest { .. } => None,
        crate::How::ForgejoAdmin { first } => {
            if carried.is_none() {
                return Err(ForgeRefusal::NoKey);
            }
            match first {
                Some(first) => {
                    if first.org.eq_ignore_ascii_case(b"temper") {
                        return Err(ForgeRefusal::FirstIsBot);
                    }
                    Some(crate::FirstIntent::Empty(crate::EmptyIntent {
                        playground: false,
                        org: first.org.clone(),
                        repository: first.repository.clone(),
                        label: first.label.clone(),
                    }))
                }
                None => None,
            }
        }
    };
    let read_only = match how {
        crate::How::GitHubApp { read_only, .. } => *read_only,
        crate::How::GitHubManifest { .. } | crate::How::ForgejoAdmin { .. } | crate::How::ForgejoTokens => false,
    };
    match crate::forges::make(
        d,
        kind,
        mode,
        place,
        git_program,
        read_only,
        crate::domain::make::bot(),
        false,
        first,
        out,
    ) {
        Some(forge) => Ok(forge),
        None => Err(ForgeRefusal::Busy),
    }
}

fn send(d: &mut Domain, request: Token, forge: u16, ask: Ask, carried: Option<Carried>, out: &mut Queue<Request>) {
    match ask {
        Ask::Replace { .. } | Ask::Connect { how: crate::How::ForgejoTokens, .. } => {
            unreachable!("tokens and Replace use their own admission")
        }
        Ask::Connect { how: how @ crate::How::GitHubApp { read_only, .. }, .. } => {
            if let Some(record) = d.record(forge)
                && record.read_only != read_only
            {
                let mut record = record.clone();
                record.read_only = read_only;
                crate::forges::save(d, record.clone(), out);
                out.push(Request::ForgeKept(record));
            }
            let carried = carried.expect("admitted key");
            d.connect.running.insert(forge, (request, Some(carried), RunKind::App)).expect("room checked");
            d.connections.insert(forge, (Connection::Connecting, None)).expect("configured forge");
            out.push(Request::MarkedForges);
            out.push(Request::Connect { forge, request, how, carried });
        }
        Ask::Connect { how: crate::How::GitHubManifest { back }, .. } => {
            if let Some(value) = carried {
                out.push(Request::CarriedUnused { number: value.number });
            }
            d.connect.running.insert(forge, (request, None, RunKind::App)).expect("room checked");
            out.push(Request::Manifest { forge, request, back });
        }
        Ask::Accept { installation, .. } => {
            if let Some(value) = carried {
                out.push(Request::CarriedUnused { number: value.number });
            }
            d.connect.running.insert(forge, (request, None, RunKind::App)).expect("room checked");
            out.push(Request::Accept { forge, request, installation });
        }
        Ask::Install { back, .. } => {
            if let Some(value) = carried {
                out.push(Request::CarriedUnused { number: value.number });
            }
            d.connect.running.insert(forge, (request, None, RunKind::App)).expect("room checked");
            out.push(Request::Install { forge, request, back });
        }
        Ask::Connect { how: crate::How::ForgejoAdmin { .. }, .. }
        | Ask::Reachable { .. }
        | Ask::DismissMaking { .. }
        | Ask::MakeRepository { .. }
        | Ask::SetDraft(_)
        | Ask::Adopt(_)
        | Ask::SetUp { .. }
        | Ask::RemoveForge { .. }
        | Ask::StartPlayground
        | Ask::EndPlayground => unreachable!("connect admission"),
    }
}
fn with_intent(d: &Domain, forge: u16, how: crate::How) -> crate::How {
    match how {
        crate::How::ForgejoAdmin { first: None } => {
            let first = match d.record(forge) {
                Some(record) => match &record.first {
                    Some(crate::FirstIntent::Empty(first)) => Some(crate::FirstAsk {
                        org: first.org.clone(),
                        repository: first.repository.clone(),
                        label: first.label.clone(),
                    }),
                    Some(crate::FirstIntent::Copies { .. } | crate::FirstIntent::Nothing) | None => None,
                },
                None => None,
            };
            crate::How::ForgejoAdmin { first }
        }
        how @ (crate::How::ForgejoAdmin { first: Some(_) }
        | crate::How::GitHubApp { .. }
        | crate::How::GitHubManifest { .. }
        | crate::How::ForgejoTokens) => how,
    }
}
/// Tokens and replacement preserve the connection until their run finishes (temper/docs/design/protocol/forge/engine.md, 2).
#[expect(clippy::too_many_arguments, reason = "the admitted request and its caller travel together")]
fn tokens_or_replace(
    d: &mut Domain,
    request: Token,
    token: Token,
    forge: u16,
    kind: RunKind,
    role: Option<Role>,
    carried: Option<Carried>,
    out: &mut Queue<Request>,
) {
    let config = d.configured.get(&forge);
    let previous = d.connection(forge);
    let (occupied, replaceable) = match &previous {
        Connection::Connected { .. } => (true, true),
        Connection::Stopped { .. } => (false, true),
        Connection::Connecting | Connection::SettingUp { .. } => (true, false),
        Connection::NotConnected | Connection::NotSetUp | Connection::SetUpFailed { .. } => (false, false),
    };
    let takes = match config {
        Some(config) => match config.kind {
            crate::Kind::Forgejo => config.mode == crate::Mode::Connected,
            crate::Kind::GitHub => kind == RunKind::Replace && config.mode != crate::Mode::Fake,
        },
        None => false,
    };
    let refusal = if role != Some(Role::Owner) {
        Some(ForgeRefusal::Role)
    } else if config.is_none() || d.forge_ended(forge) {
        Some(ForgeRefusal::NoSuchForge)
    } else if !takes {
        Some(ForgeRefusal::NotTaken)
    } else if kind == RunKind::Tokens && occupied {
        Some(ForgeRefusal::Connected)
    } else if d.connect.running.contains_key(&forge)
        || previous == Connection::Connecting
        || d.connect.pending.len() == d.connect.pending.capacity()
        || d.connect.running.len() == d.connect.running.capacity()
    {
        Some(ForgeRefusal::Busy)
    } else if kind == RunKind::Replace && !replaceable {
        Some(ForgeRefusal::NotConnected)
    } else if carried.is_none() {
        Some(ForgeRefusal::NoKey)
    } else {
        None
    };
    if let Some(why) = refusal {
        refused(d, request, token, Answered::Connect(ConnectAnswer::Refused(why)), carried, out);
        return;
    }
    let carried = carried.expect("admitted carried value");
    d.ended.remove(&forge);
    d.connect.pending.insert(request, token).expect("room checked");
    d.connect.running.insert(forge, (request, Some(carried), kind)).expect("room checked");
    let failed = match d.connections.get(&forge) {
        Some((_, failed)) => *failed,
        None => None,
    };
    d.connect.previous.insert(forge, (previous, failed)).expect("one previous connection per running connect");
    d.connections.insert(forge, (Connection::Connecting, None)).expect("configured forge");
    out.push(Request::MarkedForges);
    if kind == RunKind::Replace {
        out.push(Request::Replace { forge, request, carried });
    } else {
        out.push(Request::Connect { forge, request, how: crate::How::ForgejoTokens, carried });
    }
}

/// Admission for the string-managed Forgejo form (client/engine.md).
#[expect(clippy::too_many_arguments, reason = "the admitted request and its caller travel together")]
fn admin(
    d: &mut Domain,
    request: Token,
    token: Token,
    forge: u16,
    how: crate::How,
    role: Option<Role>,
    carried: Option<Carried>,
    out: &mut Queue<Request>,
) {
    if let crate::How::ForgejoTokens = how {
        refused(d, request, token, Answered::Connect(ConnectAnswer::Refused(ForgeRefusal::NotTaken)), carried, out);
        return;
    }
    let how = with_intent(d, forge, how);
    let previous = d.connection(forge);
    let config = d.configured.get(&forge);
    let occupied = match &previous {
        Connection::Connected { .. } | Connection::Connecting | Connection::SettingUp { .. } => true,
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SetUpFailed { .. }
        | Connection::Stopped { .. } => false,
    };
    let refusal = if role != Some(Role::Owner) {
        Some(ForgeRefusal::Role)
    } else if match config {
        Some(config) => {
            config.kind != crate::Kind::Forgejo || config.mode != crate::Mode::Managed || !config.admin_connect
        }
        None => true,
    } {
        Some(ForgeRefusal::NotTaken)
    } else if occupied {
        Some(ForgeRefusal::Connected)
    } else if d.connect.running.contains_key(&forge)
        || d.connect.pending.len() == d.connect.pending.capacity()
        || d.connect.running.len() == d.connect.running.capacity()
    {
        Some(ForgeRefusal::Busy)
    } else if carried.is_none() {
        Some(ForgeRefusal::NoKey)
    } else if let crate::How::ForgejoAdmin { first: Some(first) } = &how
        && first.org.eq_ignore_ascii_case(&config.expect("admitted configured forge").bot)
    {
        Some(ForgeRefusal::FirstIsBot)
    } else {
        None
    };
    if let Some(why) = refusal {
        if let Some(value) = carried {
            out.push(Request::CarriedUnused { number: value.number });
        }
        d.connect.answer(request, token, Answered::Connect(ConnectAnswer::Refused(why)), out);
        return;
    }
    let carried = carried.expect("admitted admin token");
    d.ended.remove(&forge);
    d.connect.pending.insert(request, token).expect("room checked");
    d.connect.running.insert(forge, (request, Some(carried), RunKind::Admin)).expect("room checked");
    let failed = match d.connections.get(&forge) {
        Some((_, failed)) => *failed,
        None => None,
    };
    d.connect.previous.insert(forge, (previous, failed)).expect("one previous connection per running connect");
    d.connections.insert(forge, (Connection::Connecting, None)).expect("configured forge");
    if let crate::How::ForgejoAdmin { first } = &how {
        d.first.remove(&forge);
        if let Some(first) = first {
            d.first
                .insert(
                    forge,
                    crate::First {
                        playground: false,
                        org: first.org.clone(),
                        repository: first.repository.clone(),
                        state: crate::FirstState::Making,
                        label: first.label.clone(),
                    },
                )
                .expect("configured forges bound first repositories");
        }
    }
    keep_first(d, forge, out);
    out.push(Request::MarkedForges);
    out.push(Request::Connect { forge, request, how, carried });
}
fn keep_first(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    if let Some(record) = d.record(forge) {
        let mut record = record.clone();
        record.first = match d.first.get(&forge) {
            Some(first) => Some(crate::FirstIntent::Empty(crate::EmptyIntent {
                playground: first.playground,
                org: first.org.clone(),
                repository: first.repository.clone(),
                label: first.label.clone(),
            })),
            None => None,
        };
        if d.record(forge) != Some(&record) {
            crate::forges::save(d, record, out);
        }
    }
}
/// Keep the user found by Check (temper/docs/design/protocol/forge/engine.md, 2).
pub(crate) fn bot_login(d: &mut Domain, forge: u16, login: Box<[u8]>, out: &mut Queue<Request>) {
    if login.is_empty() || login.len() > 255 || d.forge_ended(forge) {
        return;
    }
    let Some(record) = d.record(forge) else { return };
    if record.kind != crate::Kind::Forgejo || record.mode != crate::Mode::Connected || record.bot.login == login {
        return;
    }
    let mut record = record.clone();
    record.bot.login = login;
    crate::forges::save(d, record.clone(), out);
    out.push(Request::ForgeKept(record));
}

pub(crate) fn connected(d: &mut Domain, forge: u16, request: Token, answer: ConnectAnswer, out: &mut Queue<Request>) {
    if let Some(token) = d.connect.pending.remove(&request) {
        if let ConnectAnswer::Link(_) = &answer
            && let Some((held, _, _)) = d.connect.running.get(&forge)
            && *held == request
        {
            d.connect.running.remove(&forge);
        }
        if let ConnectAnswer::Refused(_) = answer
            && let Some((held, _, _)) = d.connect.running.get(&forge)
            && *held == request
        {
            if let Some((_, Some(carried), _)) = d.connect.running.remove(&forge) {
                out.push(Request::CarriedUnused { number: carried.number });
            }
            let (previous, failed) = d.connect.previous.remove(&forge).unwrap_or((Connection::NotConnected, None));
            d.connections.insert(forge, (previous, failed)).expect("configured forge");
            out.push(Request::MarkedForges);
        }
        d.connect.answer(request, token, Answered::Connect(answer), out);
    }
}
pub(crate) fn installed(
    d: &mut Domain,
    forge: u16,
    request: Token,
    answer: InstallAnswer,
    stopped: Option<Box<[u8]>>,
    out: &mut Queue<Request>,
) {
    if let Some(token) = d.connect.pending.remove(&request) {
        if let Some((held, _, _)) = d.connect.running.get(&forge)
            && *held == request
        {
            d.connect.running.remove(&forge);
            if let Some(why) = stopped {
                stopped_end(d, forge, &why, None, out);
            }
        }
        d.connect.answer(request, token, Answered::Install(answer), out);
    }
}
pub(crate) fn accepted(
    d: &mut Domain,
    forge: u16,
    request: Token,
    answer: AcceptAnswer,
    stopped: Option<Box<[u8]>>,
    out: &mut Queue<Request>,
) {
    if let Some(token) = d.connect.pending.remove(&request) {
        if let Some((held, _, _)) = d.connect.running.get(&forge)
            && *held == request
        {
            d.connect.running.remove(&forge);
            if let Some(why) = stopped {
                stopped_end(d, forge, &why, None, out);
            }
        }
        d.connect.answer(request, token, Answered::Accept(answer), out);
    }
}
pub(crate) fn ended(
    d: &mut Domain,
    forge: u16,
    request: Token,
    connected: bool,
    why: Option<Box<[u8]>>,
    stop: Option<crate::AdminStop>,
    out: &mut Queue<Request>,
) {
    if let Some((held, _, _)) = d.connect.running.get(&forge)
        && *held == request
    {
        let (_, carried, kind) = d.connect.running.remove(&forge).expect("matching run");
        if let Some(carried) = carried {
            out.push(Request::EraseCarried { number: carried.number });
        }
        let admin = kind == RunKind::Admin;
        if connected {
            d.connect.previous.remove(&forge);
        }
        if connected && !admin && d.configured.get(&forge).expect("running forge").kind == crate::Kind::GitHub {
            d.apps.insert(forge, crate::AppWay::Pasted).expect("configured forge App way");
            out.push(Request::MarkedForges);
        }
        if !connected {
            if let Some(why) = why {
                stopped_end(d, forge, &why, if admin { stop } else { None }, out);
            }
            let (previous, failed) = d.connect.previous.remove(&forge).unwrap_or((Connection::NotConnected, None));
            d.connections.insert(forge, (previous, failed)).expect("configured forge");
            out.push(Request::MarkedForges);
        }
    }
}

/// Volatile per-forge diagnostics, bounded in bytes without splitting UTF-8 (client/forge/deployment.md).
fn stopped_end(d: &mut Domain, forge: u16, why: &[u8], stop: Option<crate::AdminStop>, out: &mut Queue<Request>) {
    // The host supplies UTF-8. Back over continuation bytes at the cut;
    // text validation belongs to the protocol boundary.
    let mut end = why.len().min(crate::CONNECT_WHY_BYTES);
    while end > 0 {
        let Some(byte) = why.get(end) else { break };
        if byte & 0xc0 != 0x80 {
            break;
        }
        end = end.saturating_sub(1);
    }
    d.ended
        .insert(forge, (Box::from(why.get(..end).expect("bounded reason prefix")), stop))
        .expect("configured forge ends");
    out.push(Request::MarkedForges);
}

#[expect(clippy::too_many_arguments, reason = "the admitted request and its caller travel together")]
fn reachable(
    d: &mut Domain,
    request: Token,
    token: Token,
    forge: u16,
    installation: Option<u64>,
    page: u32,
    role: Option<Role>,
    carried: Option<Carried>,
    out: &mut Queue<Request>,
) {
    if let Some(value) = carried {
        out.push(Request::CarriedUnused { number: value.number });
    }
    let config = d.configured.get(&forge);
    let mut selected = installation;
    let mut accepted = false;
    if let Some(config) = config {
        match config.kind {
            crate::Kind::GitHub => {
                for ((owner, id), state) in &d.installations {
                    if *owner == forge
                        && !state.stray
                        && !state.suspended
                        && (selected.is_none() || selected == Some(*id))
                    {
                        selected = Some(*id);
                        accepted = true;
                        break;
                    }
                }
            }
            crate::Kind::Forgejo => accepted = installation.is_none(),
        }
    }
    let connected = match d.connection(forge) {
        Connection::Connected { .. } => true,
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Connecting
        | Connection::Stopped { .. } => false,
    };
    let why = if role != Some(Role::Owner) {
        Some(ForgeRefusal::Role)
    } else if config.is_none() {
        Some(ForgeRefusal::NoSuchForge)
    } else if config.expect("configured").mode == crate::Mode::Fake || page == 0 {
        Some(ForgeRefusal::NotTaken)
    } else if d.connect.reading.contains_key(&forge)
        || d.connect.reading.len() == d.connect.reading.capacity()
        || d.connect.pending.len() == d.connect.pending.capacity()
    {
        Some(ForgeRefusal::Busy)
    } else if !connected {
        Some(ForgeRefusal::NotConnected)
    } else if !accepted {
        Some(ForgeRefusal::NotTaken)
    } else {
        None
    };
    if let Some(why) = why {
        d.connect.answer(request, token, Answered::Reachable(crate::ReachableAnswer::Refused(why)), out);
        return;
    }
    d.connect.pending.insert(request, token).expect("room checked");
    d.connect.reading.insert(forge, request).expect("room checked");
    out.push(Request::Reachable { forge, request, installation: selected, page });
}
pub(crate) fn reached(
    d: &mut Domain,
    forge: u16,
    request: Token,
    answer: crate::ReachableAnswer,
    out: &mut Queue<Request>,
) {
    if d.connect.reading.get(&forge) != Some(&request) {
        return;
    }
    d.connect.reading.remove(&forge);
    if let Some(token) = d.connect.pending.remove(&request) {
        d.connect.answer(request, token, Answered::Reachable(answer), out);
    }
}
