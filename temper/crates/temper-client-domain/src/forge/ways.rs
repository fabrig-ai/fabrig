//! The session's settings and project values (temper/docs/design/client/pages/settings.md, 2).
use super::{Limits, Request, Side, wire_limits};
use alloc::boxed::Box;
use jig_client::{Handle, Latest, Link, Readable, Readout, Told, contract as c};
use skein_lib::{Duration, List, Queue, Reader, Time, Token, Writer};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_protocol_reaches::{Classes, Way, classes_from, classes_record};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ways {
    pub drafts: bool,
    pub goals: bool,
    pub changes: bool,
    pub way: u32,
}
impl Ways {
    pub const STOCK: Self = Self { drafts: true, goals: false, changes: false, way: 0 };
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Route {
    None,
    Up,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Control {
    Ways,
    Questions,
    Classes,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Proposed {
    Ways { project: u32, ways: Ways },
    Questions { project: u32, route: Route },
    Classes { project: u32, classes: Classes },
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Asked {
    Ways(Ways),
    Questions(Route),
    Classes(Classes),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Act {
    Set { project: u32, asked: Asked },
}

#[derive(Debug, PartialEq, Eq)]
enum Watch {
    Unknown,
    Open,
    Ended,
}
#[derive(Debug)]
pub(super) struct State {
    proposals: List<ProposalRead>,
    watch: Watch,
    document: Option<Box<c::SettingsDocument>>,
    project: Option<u32>,
    on_tab: bool,
    live: bool,
    ways: Slot,
    questions: Slot,
    classes: Slot,
    shown_project: Option<u32>,
    shown_classes: Slot,
}
#[derive(Debug)]
struct ProposalRead {
    number: u64,
    key: u64,
    pending: Option<Handle>,
    values: Option<List<Proposed>>,
    owned_approval: bool,
    again: bool,
}
#[derive(Debug)]
struct Slot {
    value: Option<Asked>,
    pending: Option<Reading>,
    again: bool,
    line: Option<(Option<u32>, c::SettingFollows)>,
    setting: Option<Setting>,
    refused: Option<c::Refusal>,
    retry_at: Option<Time>,
    wait: Duration,
}
#[derive(Clone, Copy, Debug)]
struct Reading {
    handle: Handle,
    project: u32,
    publish: bool,
}
#[derive(Debug)]
struct Setting {
    request: Token,
    project: u32,
    asked: Asked,
    retried: bool,
}
impl Slot {
    const fn new() -> Self {
        Self {
            value: None,
            pending: None,
            again: false,
            line: None,
            setting: None,
            refused: None,
            retry_at: None,
            wait: Duration::from_secs(1),
        }
    }
    fn forget(&mut self) {
        self.value = None;
        self.line = None;
        self.again = true;
        self.refused = None;
        self.retry_at = None;
        self.wait = Duration::from_secs(1);
    }
}
impl State {
    pub fn new() -> Self {
        Self {
            proposals: List::with_capacity(super::forges::ROWS),
            watch: Watch::Unknown,
            document: None,
            project: None,
            on_tab: false,
            live: false,
            ways: Slot::new(),
            questions: Slot::new(),
            classes: Slot::new(),
            shown_project: None,
            shown_classes: Slot::new(),
        }
    }
    fn slot(&self, control: Control) -> &Slot {
        match control {
            Control::Ways => &self.ways,
            Control::Questions => &self.questions,
            Control::Classes => &self.classes,
        }
    }
    fn slot_mut(&mut self, control: Control) -> &mut Slot {
        match control {
            Control::Ways => &mut self.ways,
            Control::Questions => &mut self.questions,
            Control::Classes => &mut self.classes,
        }
    }
    fn reading_slot(&self, control: Control, shown: bool) -> &Slot {
        if shown { &self.shown_classes } else { self.slot(control) }
    }
    fn reading_slot_mut(&mut self, control: Control, shown: bool) -> &mut Slot {
        if shown { &mut self.shown_classes } else { self.slot_mut(control) }
    }
    const fn reading_project(&self, shown: bool) -> Option<u32> {
        if shown { self.shown_project } else { self.project }
    }
}
impl Side {
    /// Whether the proposal's items read answered `owned_approval`; false until it has.
    #[must_use]
    pub fn owned_approval(&self, proposal: u64) -> bool {
        for read in &self.ways.proposals {
            if read.number == proposal {
                return read.owned_approval;
            }
        }
        false
    }

    /// Kinds 6, 7 and 8 from a proposal's deployment-scoped items read.
    #[must_use]
    pub fn proposed(&self, number: u64) -> Option<&[Proposed]> {
        for read in &self.ways.proposals {
            if read.number == number {
                return Some(read.values.as_ref()?.as_slice());
            }
        }
        None
    }

    #[must_use]
    pub fn ways(&self, project: u32) -> Option<Ways> {
        if self.ways.project != Some(project) {
            return None;
        }
        match self.ways.ways.value {
            Some(Asked::Ways(value)) => Some(value),
            Some(Asked::Questions(_) | Asked::Classes(_)) | None => None,
        }
    }
    #[must_use]
    pub fn questions(&self, project: u32) -> Option<Route> {
        if self.ways.project != Some(project) {
            return None;
        }
        match self.ways.questions.value {
            Some(Asked::Questions(value)) => Some(value),
            Some(Asked::Ways(_) | Asked::Classes(_)) | None => None,
        }
    }
    /// The open or shown need's project's classes (client/pages/settings.md, 2): stock with no value; None until read or refused.
    #[must_use]
    pub fn classes(&self, project: u32) -> Option<&Classes> {
        let slot = self.classes_slot(project)?;
        match &slot.value {
            Some(Asked::Classes(value)) => Some(value),
            Some(Asked::Ways(_) | Asked::Questions(_)) | None => None,
        }
    }
    fn classes_slot(&self, project: u32) -> Option<&Slot> {
        if self.ways.project == Some(project) {
            Some(&self.ways.classes)
        } else if self.ways.shown_project == Some(project) {
            Some(&self.ways.shown_classes)
        } else {
            None
        }
    }
    /// The settings revision delivered to this session, until its watch ends.
    #[must_use]
    pub fn settings_newest(&self) -> Option<u32> {
        Some(self.ways.document.as_ref()?.newest)
    }
    #[must_use]
    pub fn classes_busy(&self, project: u32) -> bool {
        self.classes_request(project).is_some()
    }
    #[must_use]
    pub fn classes_refused(&self, project: u32) -> Option<&c::Refusal> {
        self.classes_slot(project)?.refused.as_ref()
    }
    pub(crate) fn classes_request(&self, project: u32) -> Option<Token> {
        let pending = self.classes_slot(project)?.setting.as_ref()?;
        if pending.project == project { Some(pending.request) } else { None }
    }
    #[must_use]
    pub fn ways_asked(&self, control: Control) -> Option<&Asked> {
        let pending = self.ways.slot(control).setting.as_ref()?;
        if self.ways.project == Some(pending.project) { Some(&pending.asked) } else { None }
    }
    #[must_use]
    pub fn ways_refused(&self, control: Control) -> Option<&c::Refusal> {
        self.ways.slot(control).refused.as_ref()
    }
    #[must_use]
    pub fn settings_handle(&self) -> Handle {
        handle()
    }
}
pub(crate) fn handle() -> Handle {
    Handle { asker: crate::owner::Owner::Forge.asker(), number: super::me::SETTINGS }
}
pub(crate) fn configure(side: &mut Side, out: &mut Queue<Request>) {
    if side.link == Link::SignedOut || side.link == Link::OutOfDate {
        end(side, out);
        side.ways = State::new();
        side.changed();
        return;
    }
    if side.party.is_some() && side.ways.watch == Watch::Unknown {
        side.ways.watch = Watch::Open;
        out.push(Request::Client {
            asked: jig_client::Event::Watch {
                handle: handle(),
                subject: jig_client::Watchable::Jig(c::Subject::Settings),
            },
        });
        side.changed();
    }
}
pub(crate) fn ended(side: &Side) -> bool {
    side.ways.watch == Watch::Ended
}
fn end(side: &mut Side, out: &mut Queue<Request>) {
    if side.ways.watch == Watch::Open {
        out.push(Request::Client { asked: jig_client::Event::Unwatch { handle: handle() } });
    }
    side.ways.watch = Watch::Ended;
    side.ways.document = None;
}

/// Reconcile the open project and the How work runs tab after every step and fire.
pub(crate) fn progress(
    side: &mut Side,
    now: Time,
    open: Option<u32>,
    shown: Option<u32>,
    on_tab: bool,
    limits: Limits,
    out: &mut Queue<Request>,
) {
    let open = if side.party.is_some() { open } else { None };
    if side.ways.project != open {
        side.ways.project = open;
        side.ways.ways.forget();
        side.ways.questions.forget();
        side.ways.classes.forget();
        side.changed();
    }
    let shown = if side.party.is_some() && shown != open { shown } else { None };
    if side.ways.shown_project != shown {
        side.ways.shown_project = shown;
        side.ways.shown_classes.forget();
        side.changed();
    }
    if let Some(project) = shown {
        let line = line(side.ways.document.as_deref(), project, Control::Classes);
        if side.ways.shown_classes.line != line {
            side.ways.shown_classes.line = line;
            side.ways.shown_classes.again = true;
        }
        read_slot(side, now, Control::Classes, true, limits, out);
    }
    if on_tab && !side.ways.on_tab {
        side.ways.questions.again = true;
    }
    side.ways.on_tab = on_tab;
    if !on_tab {
        side.ways.questions.retry_at = None;
    }
    for control in [Control::Ways, Control::Questions, Control::Classes] {
        let Some(project) = open else { continue };
        if control == Control::Questions && !on_tab {
            continue;
        }
        let line = line(side.ways.document.as_deref(), project, control);
        let slot = side.ways.slot_mut(control);
        if slot.line != line {
            slot.line = line;
            slot.again = true;
        }
        read(side, now, control, limits, out);
    }
}
fn line(
    document: Option<&c::SettingsDocument>,
    project: u32,
    control: Control,
) -> Option<(Option<u32>, c::SettingFollows)> {
    for line in &document?.settings {
        if line.setting == setting(project, control) {
            return Some((line.last, line.follows.clone()));
        }
    }
    None
}
fn read(side: &mut Side, now: Time, control: Control, limits: Limits, out: &mut Queue<Request>) {
    read_slot(side, now, control, false, limits, out);
}
fn read_slot(side: &mut Side, now: Time, control: Control, shown: bool, limits: Limits, out: &mut Queue<Request>) {
    let Some(project) = side.ways.reading_project(shown) else { return };
    if control == Control::Questions && !side.ways.on_tab {
        return;
    }
    let slot = side.ways.reading_slot(control, shown);
    if !slot.again || slot.pending.is_some() || side.party.is_none() {
        return;
    }
    if let Some(at) = slot.retry_at
        && at > now
    {
        return;
    }
    let Some(bounds) = wire_limits(&limits) else { return };
    let value = w::SettingRead::new(&bounds, w::SettingReadParts { kind: kind(control), scope: project, number: 1 })
        .expect("fixed setting read fits");
    let record =
        w::Reading::new(&bounds, w::ReadingParts { read: w::Read::Setting(value) }).expect("setting read fits");
    side.serial = side.serial.checked_add(1).expect("forge handles fit");
    let handle = Handle { asker: crate::owner::Owner::Forge.asker(), number: side.serial };
    let slot = side.ways.reading_slot_mut(control, shown);
    slot.pending = Some(Reading { handle, project, publish: true });
    slot.again = false;
    slot.retry_at = None;
    out.push(Request::Client {
        asked: jig_client::Event::Read {
            handle,
            read: Readable::Family {
                scope: c::Scope::Project(project),
                window: c::Window { rows: 0, after: None },
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::reading(&record),
                },
            },
        },
    });
}
#[expect(clippy::too_many_lines, reason = "settings deliveries reconcile the open and shown project slots together")]
pub(crate) fn told(
    side: &mut Side,
    now: Time,
    client: &jig_client::Client,
    told: &Told,
    limits: Limits,
    out: &mut Queue<Request>,
) -> bool {
    match told {
        Told::Link { link } => {
            if *link == Link::Live && !side.ways.live {
                for index in 0..side.ways.proposals.len() {
                    let read = side.ways.proposals.get_mut(index).expect("bounded proposal read");
                    read.again = true;
                }
                side.ways.ways.again = true;
                side.ways.questions.again = true;
                side.ways.classes.again = true;
                side.ways.shown_classes.again = true;
            }
            side.ways.live = *link == Link::Live;
            false
        }
        Told::Changed { handle: changed } if *changed == handle() => {
            if side.ways.watch != Watch::Open {
                return true;
            }
            match client.document(*changed) {
                Some(document) if document.state == jig_client::State::Ended => end(side, out),
                Some(document) => match document.latest {
                    Some(Latest::Jig(c::Delivery::Settings(value))) => side.ways.document = Some(value.clone()),
                    Some(Latest::Jig(_) | Latest::Family(_)) => end(side, out),
                    None => side.ways.document = None,
                },
                None => side.ways.document = None,
            }
            side.changed();
            true
        }
        Told::Ended { handle: ended, .. } if *ended == handle() => {
            end(side, out);
            side.changed();
            true
        }
        Told::Page { handle, page } => {
            for index in 0..side.ways.proposals.len() {
                let read = side.ways.proposals.get_mut(index).expect("bounded proposal read");
                if read.pending == Some(*handle) {
                    read.pending = None;
                    match proposal_values(page, limits) {
                        Some((values, owned_approval)) => {
                            read.values = Some(values);
                            read.owned_approval = owned_approval;
                        }
                        None => {
                            read.values = None;
                            read.owned_approval = false;
                        }
                    }
                    side.changed();
                    return true;
                }
            }
            for (control, shown) in [
                (Control::Ways, false),
                (Control::Questions, false),
                (Control::Classes, false),
                (Control::Classes, true),
            ] {
                if let Some(pending) = side.ways.reading_slot(control, shown).pending
                    && pending.handle == *handle
                {
                    let current = side.ways.reading_project(shown) == Some(pending.project) && pending.publish;
                    let slot = side.ways.reading_slot_mut(control, shown);
                    slot.pending = None;
                    if current {
                        if transient(client, page) {
                            slot.again = true;
                            slot.retry_at = Some(now.saturating_add(slot.wait));
                            slot.wait = slot.wait.saturating_mul(2).min(Duration::from_secs(30));
                        } else {
                            slot.value = value(page, control, limits);
                            slot.retry_at = None;
                            slot.wait = Duration::from_secs(1);
                        }
                    }
                    side.changed();
                    read_slot(side, now, control, shown, limits, out);
                    return true;
                }
            }
            false
        }
        Told::Answered { key, outcome } => {
            for (control, shown) in [
                (Control::Ways, false),
                (Control::Questions, false),
                (Control::Classes, false),
                (Control::Classes, true),
            ] {
                if let Some(pending) = &side.ways.reading_slot(control, shown).setting
                    && key.asker == crate::owner::Owner::Forge.asker()
                    && key.number == pending.request.raw()
                {
                    answered(side, now, control, shown, outcome, limits, out);
                    return true;
                }
            }
            false
        }
        Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Text { .. }
        | Told::Pending { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => false,
    }
}
pub(crate) const fn next_deadline(side: &Side) -> Option<Time> {
    earlier(
        earlier(side.ways.ways.retry_at, side.ways.questions.retry_at),
        earlier(side.ways.classes.retry_at, side.ways.shown_classes.retry_at),
    )
}
const fn earlier(left: Option<Time>, right: Option<Time>) -> Option<Time> {
    match (left, right) {
        (Some(left), Some(right)) => Some(if left.as_nanos() <= right.as_nanos() { left } else { right }),
        (Some(at), None) | (None, Some(at)) => Some(at),
        (None, None) => None,
    }
}
fn transient(client: &jig_client::Client, page: &Readout) -> bool {
    match page {
        Readout::Limit => true,
        Readout::Jig(c::ReadResult::None(why)) => match why {
            c::ReadNone::Busy | c::ReadNone::Unreachable => true,
            c::ReadNone::SignedOut => client.link().state != Link::SignedOut,
            c::ReadNone::Unknown
            | c::ReadNone::Refused
            | c::ReadNone::Gone
            | c::ReadNone::TooLinked { .. }
            | c::ReadNone::TooLarge { .. }
            | c::ReadNone::Malformed => false,
        },
        Readout::Jig(c::ReadResult::Value(_)) | Readout::Family { .. } | Readout::NotServed { .. } => false,
    }
}
fn value(page: &Readout, control: Control, limits: Limits) -> Option<Asked> {
    let bytes = match page {
        Readout::Family { family, record } if *family == wire::FORGE => match record {
            Some(record) => wire::decode_setting_answer(record, &wire_limits(&limits)?)?.into_parts().value,
            None => return None,
        },
        Readout::Family { .. } | Readout::Jig(_) | Readout::NotServed { .. } | Readout::Limit => return None,
    };
    let Some(bytes) = bytes else {
        return Some(stock(control));
    };
    let mut reader = Reader::new(&bytes);
    let value = match control {
        Control::Ways => {
            let value = w::ProjectWays::decode(&wire_limits(&limits)?, &mut reader).ok()?;
            Asked::Ways(Ways {
                drafts: value.drafts(),
                goals: value.goals(),
                changes: value.changes(),
                way: value.way(),
            })
        }
        Control::Questions => {
            let value = w::ProjectQuestions::decode(&wire_limits(&limits)?, &mut reader).ok()?.into_parts();
            Asked::Questions(match value.route {
                w::QuestionsRoute::None => Route::None,
                w::QuestionsRoute::Up => Route::Up,
            })
        }
        Control::Classes => {
            let record = w::ProjectClasses::decode(&wire_limits(&limits)?, &mut reader).ok()?;
            let classes = classes_from(&record);
            classes.check().ok()?;
            Asked::Classes(classes)
        }
    };
    if reader.is_empty() { Some(value) } else { None }
}
fn stock(control: Control) -> Asked {
    match control {
        Control::Ways => Asked::Ways(Ways::STOCK),
        Control::Questions => Asked::Questions(Route::None),
        Control::Classes => Asked::Classes(Classes::stock(Way::AgentsReview)),
    }
}
const fn kind(control: Control) -> u32 {
    match control {
        Control::Ways => 7,
        Control::Questions => 6,
        Control::Classes => 8,
    }
}
const fn setting(project: u32, control: Control) -> c::Setting {
    c::Setting::Application { kind: kind(control), scope: project, number: 1 }
}
const fn control(asked: &Asked) -> Control {
    match asked {
        Asked::Ways(_) => Control::Ways,
        Asked::Questions(_) => Control::Questions,
        Asked::Classes(_) => Control::Classes,
    }
}
fn bytes(asked: &Asked, limits: Limits) -> Option<Box<[u8]>> {
    let bounds = wire_limits(&limits)?;
    Some(match asked {
        Asked::Ways(value) => {
            let record = w::ProjectWays::new(
                &bounds,
                w::ProjectWaysParts {
                    drafts: value.drafts,
                    goals: value.goals,
                    changes: value.changes,
                    way: value.way,
                },
            )
            .ok()?;
            let mut writer = Writer::new(usize::try_from(record.measure()).ok()?);
            record.encode(&mut writer).ok()?;
            writer.finish()
        }
        Asked::Questions(value) => {
            let record = w::ProjectQuestions::new(
                &bounds,
                w::ProjectQuestionsParts {
                    route: match value {
                        Route::None => w::QuestionsRoute::None,
                        Route::Up => w::QuestionsRoute::Up,
                    },
                },
            )
            .ok()?;
            let mut writer = Writer::new(usize::try_from(record.measure()).ok()?);
            record.encode(&mut writer).ok()?;
            writer.finish()
        }
        Asked::Classes(classes) => {
            let record = classes_record(classes)?;
            let mut writer = Writer::new(usize::try_from(record.measure()).ok()?);
            record.encode(&mut writer).ok()?;
            writer.finish()
        }
    })
}
pub(crate) fn act(side: &mut Side, limits: Limits, act: Act, out: &mut Queue<Request>) {
    let Act::Set { project, asked } = act;
    let control = control(&asked);
    let shown =
        side.ways.project != Some(project) && side.ways.shown_project == Some(project) && control == Control::Classes;
    if side.party.is_none()
        || side.served == super::Served::OutOfDate
        || side.ways.reading_project(shown) != Some(project)
        || side.ways.reading_slot(control, shown).setting.is_some()
    {
        return;
    }
    let Some(document) = &side.ways.document else { return };
    let newest = document.newest;
    side.ways.reading_slot_mut(control, shown).refused = None;
    send(side, Setting { request: Token::new(0), project, asked, retried: false }, shown, newest, limits, out);
}
/// One project's setting, encoded as the set path and a settings proposal both send it.
pub(crate) fn setting_item(project: u32, asked: &Asked, limits: Limits) -> Option<c::SettingItem> {
    Some(c::SettingItem {
        setting: setting(project, control(asked)),
        edit: c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(bytes(asked, limits)?) }),
    })
}
fn send(side: &mut Side, mut pending: Setting, shown: bool, against: u32, limits: Limits, out: &mut Queue<Request>) {
    let Some(item) = setting_item(pending.project, &pending.asked, limits) else { return };
    let control = control(&pending.asked);
    let mut items = List::with_capacity(1);
    items.push(item).expect("one setting");
    side.serial = side.serial.checked_add(1).expect("forge requests fit");
    pending.request = Token::new(side.serial);
    out.push(Request::Send {
        request: pending.request,
        name: None,
        ask: jig_client::Ask::Jig(Box::new(c::Ask::ChangeSettings(c::ChangeSettings {
            against,
            items,
            running: c::Running::Stays,
        }))),
    });
    side.ways.reading_slot_mut(control, shown).setting = Some(pending);
    side.changed();
}
fn answered(
    side: &mut Side,
    now: Time,
    control: Control,
    shown: bool,
    outcome: &jig_client::Outcome,
    limits: Limits,
    out: &mut Queue<Request>,
) {
    let Some(mut pending) = side.ways.reading_slot_mut(control, shown).setting.take() else { return };
    if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::Stale(stale),
        ..
    })) = outcome
        && !pending.retried
    {
        pending.retried = true;
        send(side, pending, shown, stale.newest, limits, out);
        return;
    }
    let current = side.ways.reading_project(shown) == Some(pending.project);
    let slot = side.ways.reading_slot_mut(control, shown);
    if current {
        if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal, .. })) = outcome {
            slot.refused = Some(refusal.clone());
        }
        if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::SettingsChanged(
            _,
        )))) = outcome
        {
            slot.value = Some(pending.asked);
            if let Some(read) = &mut slot.pending {
                read.publish = false;
            }
            slot.again = true;
        }
    }
    read_slot(side, now, control, shown, limits, out);
    side.changed();
}

/// The selected tracked template offered by this project, or the lowest numbered one.
#[must_use]
pub fn default_way(me: &c::Me, project: u32, ways: &Ways) -> Option<u32> {
    let mut first = None;
    for row in &me.projects {
        if row.project != project {
            continue;
        }
        for template in &row.templates {
            if template.tracked.is_some() {
                if template.number == ways.way {
                    return Some(template.number);
                }
                first = Some(match first {
                    Some(number) => core::cmp::min(number, template.number),
                    None => template.number,
                });
            }
        }
    }
    first
}

/// Reconcile only proposals on the current chat or inbox page (client/pages/settings.md, 2).
pub(crate) fn proposals(
    side: &mut Side,
    cards: Option<&List<crate::pages::inbox::Card>>,
    limits: Limits,
    out: &mut Queue<Request>,
) {
    let Some(cards) = cards else { return };
    let skip = cards.len().saturating_sub(super::forges::ROWS);
    for card in cards.iter().skip(usize::try_from(skip).expect("bounded cards")) {
        let Some(c::ReadValue::Proposal(proposal)) = &card.page else { continue };
        let c::ProposalAction::Settings(settings) = &proposal.action else { continue };
        let Some(review) = &settings.review else { continue };
        let Some(key) = review.key else { continue };
        let number = proposal.key.number;
        let mut needed = false;
        for item in &review.items {
            if let c::Setting::Application { kind: 1 | 2 | 6 | 7 | 8, .. } = item.setting {
                needed = true;
            }
        }
        if !needed {
            continue;
        }
        let mut found = None;
        for (index, read) in side.ways.proposals.iter().enumerate() {
            if read.number == number {
                found = Some(u32::try_from(index).expect("bounded proposal index"));
            }
        }
        let index = match found {
            Some(index) => index,
            None => {
                if side.ways.proposals.len() == side.ways.proposals.capacity() {
                    let old = core::mem::replace(&mut side.ways.proposals, List::with_capacity(super::forges::ROWS));
                    for (index, read) in old.into_boxed().into_iter().enumerate() {
                        if index != 0 {
                            side.ways.proposals.push(read).expect("retained proposals fit");
                        }
                    }
                }
                let index = side.ways.proposals.len();
                side.ways
                    .proposals
                    .push(ProposalRead { number, key, pending: None, values: None, owned_approval: false, again: true })
                    .expect("bounded proposals");
                index
            }
        };
        let read = side.ways.proposals.get(index).expect("retained proposal");
        if !read.again || read.pending.is_some() || side.party.is_none() {
            continue;
        }
        let Some(bounds) = wire_limits(&limits) else { continue };
        let record = w::Reading::new(
            &bounds,
            w::ReadingParts {
                read: w::Read::ProposalItems(
                    w::ProposalItemsRead::new(&bounds, w::ProposalItemsReadParts { key: read.key })
                        .expect("fixed proposal read"),
                ),
            },
        )
        .expect("fixed reading");
        side.serial = side.serial.checked_add(1).expect("forge handles fit");
        let handle = Handle { asker: crate::owner::Owner::Forge.asker(), number: side.serial };
        let read = side.ways.proposals.get_mut(index).expect("retained proposal");
        read.pending = Some(handle);
        read.again = false;
        out.push(Request::Client {
            asked: jig_client::Event::Read {
                handle,
                read: Readable::Family {
                    scope: c::Scope::Deployment,
                    window: c::Window { rows: 1, after: None },
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::reading(&record),
                    },
                },
            },
        });
    }
}
fn proposal_values(page: &Readout, limits: Limits) -> Option<(List<Proposed>, bool)> {
    let Readout::Family { family: wire::FORGE, record: Some(bytes) } = page else { return None };
    let bounds = wire_limits(&limits)?;
    let answer = wire::decode_proposal_items_answer(bytes, &bounds)?;
    let owned_approval = answer.owned_approval();
    let mut values = List::with_capacity(answer.items().len());
    for item in answer.into_parts().items.into_boxed() {
        let item = item.into_parts();
        let control = if item.kind == 6 {
            Control::Questions
        } else if item.kind == 8 {
            Control::Classes
        } else if item.kind == 7 {
            Control::Ways
        } else {
            continue;
        };
        let decoded = match item.edit {
            w::SettingEdit::Set(set) => {
                let bytes = wire::setting_answer(
                    &w::SettingAnswer::new(&bounds, w::SettingAnswerParts { value: Some(set.into_parts().value) })
                        .ok()?,
                );
                value(&Readout::Family { family: wire::FORGE, record: Some(bytes) }, control, limits)?
            }
            w::SettingEdit::Removed | w::SettingEdit::Reset => stock(control),
        };
        values
            .push(match decoded {
                Asked::Ways(ways) => Proposed::Ways { project: item.scope, ways },
                Asked::Questions(route) => Proposed::Questions { project: item.scope, route },
                Asked::Classes(classes) => Proposed::Classes { project: item.scope, classes },
            })
            .expect("one value per item");
    }
    Some((values, owned_approval))
}
