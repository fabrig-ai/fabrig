//! The forge's adoption form, playground read, panel watch and receiving state.
use alloc::boxed::Box;
use skein_lib::{Env, Queue, Time};
pub mod admin;
pub mod approvals;
pub mod away;
pub mod change;
pub mod changes;
pub mod connect;
pub mod decided;
pub mod deployment;
pub mod devices;
pub mod dismissed;
pub mod family;
pub mod forges;
mod form;
pub mod github;
pub mod listed;
pub mod making;
pub(crate) mod me;
pub mod model;
pub mod panel;
pub mod playground;
pub mod replace;
pub mod sheet;
pub mod start;
pub mod status;
pub mod stock_ways;
pub mod tokens;
pub mod trials;
mod vocabulary;
pub mod ways;
pub mod words;
pub use family::dismiss_making;
pub use form::{Act, Broken, Check, Field, Form, Marks, Text, Typed, adoption, broken, form, write};
pub use vocabulary::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub field_bytes: u32,
    pub checks: u32,
}
#[derive(Debug)]
pub enum Event {
    Act { act: Act },
    SheetRead { request: skein_lib::Token, answer: Result<temper_client_protocol_wire::v1::Page, sheet::Refused> },
    SheetMade { request: skein_lib::Token, answer: Result<(), Box<[u8]>> },
    Approvals { event: approvals::Event },
    Trials { event: trials::Event, adoption: Option<Answer> },
    AdminToken { value: skein_lib::Secret },
    ForgejoToken { field: tokens::Field, value: skein_lib::Secret },
    ReplaceValue { forge: u16, field: replace::Field, value: skein_lib::Secret },
    AppKey { value: skein_lib::Secret },
    Panel { subject: Option<panel::Subject> },
    RefreshPanel { subject: panel::Subject },
    Watched { handle: jig_client::Handle, watched: panel::Watched },
    Progress { request: skein_lib::Token, progress: skein_lib::RequestProgress },
    Answered { request: skein_lib::Token, answer: Answer },
    Went { address: Address },
    Left,
    Served { served: Served },
}
#[derive(Debug)]
pub enum Request {
    Repositories { project: u32 },
    PlaygroundReset,
    PanelEnded { subject: panel::Subject },
    Show { address: Address },
    ConnectForge { forge: u16 },
    Leave { to: Box<[u8]> },
    Client { asked: jig_client::Event },
    Send { request: skein_lib::Token, name: Option<skein_lib::RequestKey>, ask: jig_client::Ask },
}
#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "each session watch keeps whether its own handle is open")]
pub struct Side {
    ways: ways::State,
    playground: playground::State,
    deployment: deployment::State,
    forges: forges::State,
    linked: Option<bool>,
    forges_open: bool,
    dismissed: dismissed::State,
    dismissed_open: bool,
    trials: trials::Trials,
    approvals: approvals::Approvals,
    approvals_open: bool,
    away: away::State,
    away_open: bool,
    set_up: Option<Box<forges::SetUp>>,
    setup_form: forges::Draft,
    admin_form: admin::Draft,
    admin: Option<admin::Pending>,
    tokens_form: tokens::Draft,
    tokens: Option<tokens::Pending>,
    replacements: skein_lib::List<replace::Draft>,
    app_form: github::Draft,
    app: Option<github::Pending>,
    panel: Option<panel::Panel>,
    panel_retry: bool,
    address: Option<Address>,
    served: Served,
    form: Option<Form>,
    sheet: Option<sheet::Sheet>,
    sheet_again: Option<sheet::Act>,
    sheet_dismissals: skein_lib::List<sheet::Dismissal>,
    serial: u64,
    party: Option<u64>,
    me: bool,
    link: jig_client::Link,
    shown: u64,
}
impl Side {
    #[must_use]
    pub fn new(_limits: &Limits, seed: u64) -> Side {
        Side {
            ways: ways::State::new(),
            playground: playground::State::new(),
            deployment: deployment::State::new(),
            forges: forges::State::Unknown,
            linked: None,
            forges_open: false,
            dismissed: dismissed::State::Unknown,
            dismissed_open: false,
            trials: trials::Trials::new(),
            approvals: approvals::Approvals::default(),
            approvals_open: false,
            away: away::State::Unknown,
            away_open: false,
            set_up: None,
            setup_form: forges::Draft::new(),
            admin_form: admin::Draft::new(),
            admin: None,
            tokens_form: tokens::Draft::new(),
            tokens: None,
            replacements: skein_lib::List::with_capacity(forges::ROWS),
            app_form: github::Draft::new(),
            app: None,
            panel: None,
            panel_retry: false,
            address: None,
            served: Served::Unknown,
            form: None,
            sheet: None,
            sheet_again: None,
            sheet_dismissals: skein_lib::List::uncapped(),
            serial: skein_lib::Rng::new(seed).next_u64(),
            party: None,
            me: false,
            link: jig_client::Link::Starting,
            shown: 0,
        }
    }
    #[must_use]
    pub const fn address(&self) -> Option<Address> {
        self.address
    }
    #[must_use]
    pub const fn served(&self) -> Served {
        self.served
    }
    #[must_use]
    pub const fn sheet(&self) -> Option<&sheet::Sheet> {
        self.sheet.as_ref()
    }
    #[must_use]
    pub fn making_dismissal(&self, forge: u16, org: &[u8], name: &[u8]) -> Option<&sheet::Dismissal> {
        sheet::dismissal(self, forge, org, name)
    }
    #[must_use]
    pub const fn form(&self) -> Option<&Form> {
        self.form.as_ref()
    }
    #[must_use]
    pub const fn playground(&self) -> Option<&Playground> {
        self.playground.answer.as_ref()
    }
    #[must_use]
    pub const fn playground_answered(&self) -> bool {
        self.playground.answered
    }
    #[must_use]
    pub const fn deployment(&self) -> Option<&deployment::Answer> {
        self.deployment.answer.as_ref()
    }
    #[must_use]
    pub fn playground_starting(&self) -> bool {
        if self.playground.start.is_some() {
            return true;
        }
        match &self.playground.answer {
            Some(answer) => answer.state == PlaygroundState::Starting,
            None => false,
        }
    }
    #[must_use]
    pub const fn playground_failure(&self) -> Option<&playground::StartFailure> {
        self.playground.failure.as_ref()
    }
    /// From the act's send until the read is none (temper/docs/design/client/forge/family.md, 7).
    #[must_use]
    pub const fn leaving(&self) -> bool {
        self.playground.leaving
    }
    #[must_use]
    pub fn playground_provider(&self) -> Option<u32> {
        Some(self.playground.answer.as_ref()?.provider)
    }
    #[must_use]
    pub fn playground_running(&self) -> bool {
        match self.playground() {
            Some(answer) => answer.state == PlaygroundState::Running,
            None => false,
        }
    }
    #[must_use]
    pub fn playground_request(&self) -> Option<skein_lib::Token> {
        Some(self.playground.start.as_ref()?.request)
    }
    #[must_use]
    pub const fn panel(&self) -> Option<&panel::Panel> {
        self.panel.as_ref()
    }
    /// Whether the printed link signed in, once the forges watch delivered (client/forge/records.md).
    #[must_use]
    pub const fn linked(&self) -> Option<bool> {
        self.linked
    }
    #[must_use]
    pub const fn forges(&self) -> &forges::State {
        &self.forges
    }
    #[must_use]
    pub fn set_up(&self) -> Option<&forges::SetUp> {
        self.set_up.as_deref()
    }
    #[must_use]
    pub const fn setup_form(&self) -> &forges::Draft {
        &self.setup_form
    }
    #[must_use]
    pub const fn admin_form(&self) -> &admin::Draft {
        &self.admin_form
    }
    #[must_use]
    pub const fn admin(&self) -> Option<&admin::Pending> {
        self.admin.as_ref()
    }
    #[must_use]
    pub const fn tokens_form(&self) -> &tokens::Draft {
        &self.tokens_form
    }
    #[must_use]
    pub const fn tokens(&self) -> Option<&tokens::Pending> {
        self.tokens.as_ref()
    }
    #[must_use]
    pub const fn app_form(&self) -> &github::Draft {
        &self.app_form
    }
    #[must_use]
    pub const fn app(&self) -> Option<&github::Pending> {
        self.app.as_ref()
    }
    #[must_use]
    #[expect(clippy::manual_find, reason = "step code uses loops rather than iterator callbacks")]
    pub fn replacement(&self, forge: u16) -> Option<&replace::Draft> {
        for draft in &self.replacements {
            if draft.forge == forge {
                return Some(draft);
            }
        }
        None
    }
    fn changed(&mut self) {
        // Opening the form and delivering forge rows both publish through here.
        if let Some(draft) = &mut self.form {
            let mut unwritten = true;
            for field in &draft.texts {
                unwritten &= field.written == 0;
            }
            if unwritten && let Some((forge, org, name)) = listed::unadopted_first(&self.forges) {
                form::fill_first(draft, forge, org, name);
            }
        }
        self.shown = self.shown.checked_add(1).expect("forge shown count fits");
    }
}
pub fn step(side: &mut Side, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::SheetRead { request, answer } => sheet::received(side, request, answer, env.limits, out),
        Event::SheetMade { request, answer } => sheet::made(side, request, answer, env.limits, out),
        Event::Approvals { event } => approvals::receive(side, event, out),
        Event::Trials { event, adoption } => {
            let request = match &event {
                trials::Event::Answered { key, .. } => Some(skein_lib::Token::new(key.number)),
                trials::Event::Rows { .. }
                | trials::Event::RowsEnded
                | trials::Event::Wait { .. }
                | trials::Event::TaskEnded => None,
            };
            trials::receive(side, event, out);
            if let Some(request) = request
                && let Some(answer) = adoption
            {
                answered(side, request, answer, env.limits, out);
            }
        }
        Event::AdminToken { value } => admin::token(side, value),
        Event::ForgejoToken { field, value } => tokens::write(side, field, value),
        Event::ReplaceValue { forge, field, value } => replace::write(side, forge, field, value),
        Event::AppKey { value } => github::key(side, value),
        Event::Panel { subject } => panel::subject(side, env.limits, subject, out),
        Event::RefreshPanel { subject } => panel::refresh(side, env.limits, subject, out),
        Event::Watched { handle, watched } => panel::watched(side, handle, watched, out),
        Event::Went { address } => {
            side.replacements.clear();
            side.address = Some(address);
            let Address::Adopt { project } = address;
            let mut draft = form(project);
            draft.served = side.served;
            draft.confirmed = side.party.is_some();
            side.form = Some(draft);
            side.sheet = Some(sheet::Sheet::new(project, &side.forges));
            if let Some(act) = side.sheet_again.take() {
                sheet::again(side, env.limits, act, out);
            }
            sheet::progress(side, env.limits, out);
            side.changed();
        }
        Event::Left => {
            side.replacements.clear();
            side.form = None;
            side.sheet = None;
            if side.address.take().is_some() {
                side.changed();
            }
        }
        Event::Served { served: value } => served(side, value),
        Event::Act { act } => act_on(side, env.limits, act, out),
        Event::Progress { request, progress } => {
            if let Some(draft) = &mut side.form
                && let Some((pending, _)) = draft.pending
                && pending == request
            {
                match progress {
                    skein_lib::RequestProgress::Unknown => {
                        draft.pending = None;
                        draft.outcome = Some(Answer::Unknown);
                    }
                    skein_lib::RequestProgress::InFlight
                    | skein_lib::RequestProgress::Retrying
                    | skein_lib::RequestProgress::Parked => {
                        let progress = match draft.served {
                            Served::OutOfDate => skein_lib::RequestProgress::Parked,
                            Served::Unknown | Served::Yes => progress,
                        };
                        draft.pending = Some((request, progress));
                    }
                }
                side.changed();
            }
        }
        Event::Answered { request, answer } => answered(side, request, answer, env.limits, out),
    }
}
fn answered(side: &mut Side, request: skein_lib::Token, answer: Answer, limits: Limits, out: &mut Queue<Request>) {
    if sheet::adopted(side, request, &answer, limits, out) {
        return;
    }
    if let Some(draft) = &mut side.form
        && let Some((pending, _)) = draft.pending
        && pending == request
    {
        draft.pending = None;
        match answer {
            Answer::Adopted(_) => form::reset(draft, answer),
            Answer::CoreRefused(_) | Answer::ForgeRefused(_) | Answer::EntranceRefused(_) | Answer::Unknown => {
                draft.outcome = Some(answer);
            }
        }
        side.changed();
    }
}
fn act_on(side: &mut Side, limits: Limits, act: Act, out: &mut Queue<Request>) {
    match act {
        Act::Sheet(act) => sheet::act(side, limits, act, out),
        Act::Ways(act) => ways::act(side, limits, act, out),
        Act::Forges(act) => forges::act(side, limits, act, out),
        Act::Go { address } => out.push(Request::Show { address }),
        Act::AdminWay(way) => admin::way(side, way),
        Act::AdminWrite { field, text } => admin::write(side, limits, field, text),
        Act::ConnectAdmin { forge } => admin::send(side, limits, forge, out),
        Act::ConnectTokens { forge } => tokens::send(side, limits, forge, out),
        Act::AppWrite { text } => github::write(side, text, limits),
        Act::ConnectManifest { forge, back } => github::manifest(side, limits, forge, back, out),
        Act::ConnectApp { forge } => github::send(side, limits, forge, None, out),
        Act::Install { forge, back } => github::send(side, limits, forge, Some(back), out),
        Act::TryAccount { account, again } => trials::send(side, account, again, limits, out),
        Act::StartPlayground => playground::start(side, limits, out),
        Act::EndPlayground => playground::end(side, limits, out),
        Act::SetUp { forge } => forges::set_up(side, limits, forge, out),
        Act::SetUpAtOnce { forge } => forges::set_up_at_once(side, limits, forge, out),
        Act::SetupWrite { field, text } => forges::write(side, limits, field, text),
        Act::CopiesWrite { text } => forges::write_copies(side, limits, text),
        Act::Adopt => {
            let Some(_party) = side.party else {
                return;
            };
            let Some(draft) = &mut side.form else {
                return;
            };
            if draft.pending.is_some() {
                return;
            }
            match draft.served {
                Served::OutOfDate => return,
                Served::Yes | Served::Unknown => {}
            }
            form::mark(draft, limits.checks);
            if let Some(adoption) = adoption(draft, limits.checks) {
                side.serial = side.serial.checked_add(1).expect("forge request serial fits u64");
                let request = skein_lib::Token::new(side.serial);
                draft.pending = Some((request, skein_lib::RequestProgress::Parked));
                out.push(Request::Send {
                    request,
                    name: None,
                    ask: ask(&adoption, &limits).expect("validated adoption fits wire"),
                });
            }
            side.changed();
        }
        Act::Write { field, text, typed } => {
            if let Some(draft) = &mut side.form {
                write(draft, field, text, typed, limits.field_bytes, limits.checks);
                side.changed();
            }
        }
        Act::Choose { role } => {
            if let Some(draft) = &mut side.form {
                draft.role = role;
                draft.outcome = None;
                draft.home.broken = None;
                side.changed();
            }
        }
        Act::Home => {
            if let Some(draft) = &mut side.form {
                draft.home.on = !draft.home.on;
                draft.home.broken = None;
                draft.outcome = None;
                side.changed();
            }
        }
        Act::Ci => {
            if let Some(draft) = &mut side.form {
                draft.ci.on = !draft.ci.on;
                draft.outcome = None;
                side.changed();
            }
        }
    }
}
#[must_use]
pub const fn next_deadline(side: &Side) -> Option<Time> {
    ways::next_deadline(side)
}
pub const fn reclaim(_side: &mut Side) {}
#[must_use]
pub const fn shown(side: &Side) -> u64 {
    side.shown
}
fn served(side: &mut Side, served: Served) {
    side.served = served;
    if served == Served::OutOfDate
        && let Some(set_up) = &mut side.set_up
        && set_up.progress.is_some()
    {
        set_up.progress = Some(skein_lib::RequestProgress::Parked);
    }
    if let Some(draft) = &mut side.form {
        draft.served = side.served;
        if side.served == Served::OutOfDate
            && let Some((request, _)) = draft.pending
        {
            draft.pending = Some((request, skein_lib::RequestProgress::Parked));
        }
    }
    side.changed();
}

/// The forge's receiving limits, shared by its bytes and bounds accounting.
#[must_use]
pub fn wire_limits(limits: &Limits) -> Option<temper_client_protocol_wire::v1::Limits> {
    temper_client_protocol_wire::limits(&temper_client_protocol_wire::Bounds {
        segments: temper_client_protocol_wire::v1::CEILINGS.landing_pattern_segments,
        segment_bytes: temper_client_protocol_wire::v1::CEILINGS.landing_pattern_segments_item,
        gates: temper_client_protocol_wire::v1::CEILINGS.landing_rule_gates,
        approvals: temper_client_protocol_wire::v1::CEILINGS.landing_rule_approvals,
        field_bytes: limits.field_bytes,
        checks: limits.checks,
        contexts: limits.checks,
        refusal_bytes: 65_536,
    })
}
pub(crate) fn told(client: &jig_client::Client, limits: &crate::Limits, told: jig_client::Told) -> Option<Event> {
    use jig_client::{Outcome, Told};
    if let Some(event) = approvals::told(client, &told, limits.forge) {
        return Some(Event::Approvals { event });
    }
    if let Some(event) = trials::told(client, &told, limits.forge) {
        let adoption = match &told {
            Told::Answered { outcome: Outcome::Answered(value), .. } => answer(limits, value),
            Told::Pending { .. } | Told::Answered { .. } => Some(Answer::Unknown),
            Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Link { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Text { .. }
            | Told::Page { .. } => None,
        };
        return Some(Event::Trials { event, adoption });
    }
    match told {
        Told::Changed { handle } => {
            Some(Event::Watched { handle, watched: panel::decode(client, handle, limits.forge) })
        }
        Told::Ended { handle, why } => Some(Event::Watched { handle, watched: panel::Watched::Ended(why) }),
        Told::Pending { key } => {
            let pending = client.pending(key)?;
            let progress = match pending.progress {
                jig_client::Progress::InFlight => skein_lib::RequestProgress::InFlight,
                jig_client::Progress::Retrying { .. } => skein_lib::RequestProgress::Retrying,
                jig_client::Progress::NotServed | jig_client::Progress::Parked => skein_lib::RequestProgress::Parked,
                jig_client::Progress::Unknown => skein_lib::RequestProgress::Unknown,
            };
            Some(Event::Progress { request: skein_lib::Token::new(key.number), progress })
        }
        Told::Answered { key, outcome } => {
            let request = skein_lib::Token::new(key.number);
            let answer = match outcome {
                Outcome::Answered(answered) => answer(limits, &answered)?,
                Outcome::Unknown
                | Outcome::Limit
                | Outcome::Held
                | Outcome::NotKept
                | Outcome::Unsent
                | Outcome::Full { .. }
                | Outcome::Lost
                | Outcome::Unconfirmed => Answer::Unknown,
            };
            Some(Event::Answered { request, answer })
        }
        Told::Kept { .. } | Told::Dropped { .. } | Told::Link { .. } | Told::Text { .. } | Told::Page { .. } => None,
    }
}

/// Encode the validated adoption in its project scope.
#[must_use]
pub fn ask(adoption: &Adoption, limits: &Limits) -> Option<jig_client::Ask> {
    Some(jig_client::Ask::Family {
        scope: jig_client::contract::Scope::Project(adoption.project),
        record: family::request(adoption, &wire_limits(limits)?)?,
    })
}
/// Encode a checked making request in its destination's scope (client/forge/records.md).
#[must_use]
pub fn make_repository(
    request: &temper_client_protocol_wire::v1::MakeRepository,
    limits: &Limits,
) -> Option<jig_client::Ask> {
    let scope = match request.into() {
        Some(into) => jig_client::contract::Scope::Project(into.project()),
        None => jig_client::contract::Scope::Deployment,
    };
    Some(jig_client::Ask::Family { scope, record: family::make_repository(request, &wire_limits(limits)?)? })
}
/// Ready's copies are named in address order, starting at one (client/pages/README.md).
#[must_use]
pub fn copy_key(number: u64) -> skein_lib::RequestKey {
    let number = skein_lib::Decimal::of(number);
    let text = skein_lib::bytes::concat(&[b"welcome-copy-", number.as_bytes()]);
    crate::key::named(crate::key::TOOLS, &crate::key::Name::new(&text).expect("copy key fits"))
}

/// Decode the same terminal answers the adoption side receives.
#[must_use]
pub fn answer(limits: &crate::Limits, answered: &jig_client::Answered) -> Option<Answer> {
    match answered {
        jig_client::Answered::Family(record) => {
            let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).ok()?;
            family::answer(record, &wire_limits(&limits.forge)?, &link)
        }
        jig_client::Answered::Jig(jig_client::contract::Answer::Refused { refusal, .. }) => {
            Some(Answer::EntranceRefused(jig_client::contract::EntranceRefusal::new(refusal.clone()).ok()?))
        }
        jig_client::Answered::Jig(jig_client::contract::Answer::Done(_)) => Some(Answer::Unknown),
    }
}

/// Ready and ctl name the same adoption (temper/docs/design/client/pages/README.md).
#[must_use]
pub fn first_repository_key(forge: u16, additional: bool) -> skein_lib::RequestKey {
    let prefix: &[u8] = if additional { b"first-repository-additional-" } else { b"first-repository-" };
    let number = skein_lib::Decimal::of(u64::from(forge));
    let mut text = skein_lib::Writer::new(prefix.len().checked_add(number.as_bytes().len()).expect("key fits"));
    text.put(prefix).expect("key prefix fits");
    text.put(number.as_bytes()).expect("key number fits");
    crate::key::named(crate::key::TOOLS, &crate::key::Name::new(&text.finish()).expect("first repository key fits"))
}

/// Each refusal gives Home's next adoption a new name (temper/docs/design/client/pages/README.md).
#[must_use]
pub fn first_repository_again_key(forge: u16, attempt: u64) -> skein_lib::RequestKey {
    let prefix = b"first-repository-again-";
    let forge = skein_lib::Decimal::of(u64::from(forge));
    let attempt = skein_lib::Decimal::of(attempt);
    let length = prefix
        .len()
        .checked_add(forge.as_bytes().len())
        .expect("forge fits")
        .checked_add(1)
        .expect("separator fits")
        .checked_add(attempt.as_bytes().len())
        .expect("key fits");
    let mut text = skein_lib::Writer::new(length);
    text.put(prefix).expect("prefix fits");
    text.put(forge.as_bytes()).expect("forge fits");
    text.put(b"-").expect("separator fits");
    text.put(attempt.as_bytes()).expect("attempt fits");
    crate::key::named(crate::key::TOOLS, &crate::key::Name::new(&text.finish()).expect("adoption key fits"))
}

/// Project creation has its own dedup name; a refused attempt needs a new one.
#[must_use]
pub fn start_project_key(forge: u16, attempt: u64) -> skein_lib::RequestKey {
    let prefix = b"start-project-";
    let forge = skein_lib::Decimal::of(u64::from(forge));
    let number = skein_lib::Decimal::of(attempt);
    let suffix = if attempt == 0 { 0 } else { number.as_bytes().len().checked_add(1).expect("suffix fits") };
    let length =
        prefix.len().checked_add(forge.as_bytes().len()).expect("forge fits").checked_add(suffix).expect("key fits");
    let mut text = skein_lib::Writer::new(length);
    text.put(prefix).expect("key prefix fits");
    text.put(forge.as_bytes()).expect("forge fits");
    if attempt != 0 {
        text.put(b"-").expect("separator fits");
        text.put(number.as_bytes()).expect("attempt fits");
    }
    crate::key::named(crate::key::TOOLS, &crate::key::Name::new(&text.finish()).expect("start project key fits"))
}
