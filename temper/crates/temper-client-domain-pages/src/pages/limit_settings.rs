//! Limits and one-act raises from Needs you (configuration/limit-settings.md, 7).
#[cfg(test)]
mod tests;
use super::{Pages, Request, accounts, inbox::Card};
use crate::{Address, Limits, forge, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Handle, Key, Latest, Readable, Readout, Told, contract as c};
use skein_lib::{Decimal, Duration, List, Map, Queue, Reader, Time, Writer, bytes};
use temper_client_protocol_wire::v1::Record;
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Next,
    First,
    Raise { task: u64, revision: u64 },
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "rows and refusal are view facts; cursors belong to the page")]
pub struct LimitsPage {
    pub rows: Option<w::LimitsAnswer>,
    pub refused: bool,
    after: Option<Box<[u8]>>,
    pending: Option<Handle>,
}
impl LimitsPage {
    fn new() -> Self {
        Self { rows: None, refused: false, after: None, pending: None }
    }
}
#[derive(Debug)]
pub struct State {
    page: Option<LimitsPage>,
    holds: Map<u64, w::LimitHold>,
    holds_after: Option<u64>,
    holds_pending: Option<Handle>,
    holds_done: bool,
    inbox: Option<Handle>,
    value: Option<w::LimitOverrides>,
    value_read: bool,
    value_pending: Option<(Handle, u32)>,
    revision: Option<u32>,
    party: Option<u64>,
    retry_at: Option<Time>,
    sending: Option<(u64, Key)>,
    refused: Option<(u64, jig_client::Outcome)>,
}
impl State {
    pub(super) fn new() -> Self {
        Self {
            page: None,
            holds: Map::uncapped(),
            holds_after: None,
            holds_pending: None,
            holds_done: false,
            inbox: None,
            value: None,
            value_read: false,
            value_pending: None,
            revision: None,
            party: None,
            retry_at: None,
            sending: None,
            refused: None,
        }
    }
    pub(super) fn next_deadline(&self) -> Option<Time> {
        if self.page.is_some() || self.inbox.is_some() { self.retry_at } else { None }
    }
    pub(super) fn refresh_holds(&mut self) {
        self.holds = Map::uncapped();
        self.holds_after = None;
        self.holds_pending = None;
        self.holds_done = false;
    }
    #[must_use]
    pub const fn page(&self) -> Option<&LimitsPage> {
        self.page.as_ref()
    }
    #[must_use]
    pub fn hold(&self, card: &Card) -> Option<&w::LimitHold> {
        let task = card.document()?;
        let held = task.hold.as_ref()?;
        let row = self.holds.get(&task.chip.task)?;
        if row.revision == held.revision { Some(row) } else { None }
    }
    #[must_use]
    pub fn ready(&self) -> bool {
        self.value.is_some() && self.sending.is_none()
    }
    #[must_use]
    pub fn refused(&self, task: u64) -> Option<&jig_client::Outcome> {
        let (number, outcome) = self.refused.as_ref()?;
        if *number == task { Some(outcome) } else { None }
    }
}
impl Pages {
    #[must_use]
    pub fn limit_settings(&self) -> &State {
        &self.limit_settings
    }
}
#[must_use]
pub fn owner(me: Option<&c::Me>) -> bool {
    match me {
        Some(me) => me.deployment_role == Some(c::HeldRole::Owner),
        None => false,
    }
}
fn revision(client: &jig_client::Client) -> Option<u32> {
    let doc = client.document(forge::ways::handle())?;
    if doc.state != jig_client::State::Live {
        return None;
    }
    let Some(Latest::Jig(c::Delivery::Settings(settings))) = doc.latest else { return None };
    Some(settings.newest)
}
fn handle(next: &mut u64) -> Handle {
    let handle = Handle { asker: Owner::Pages.asker(), number: *next };
    *next = next.checked_add(1).expect("page handles fit");
    handle
}
fn read(handle: Handle, query: w::Read, bounds: &w::Limits, out: &mut Queue<Request>) {
    let record = w::Reading { read: query };
    out.push(Request::Client(jig_client::Event::Read {
        handle,
        read: Readable::Family {
            scope: c::Scope::Deployment,
            window: c::Window { rows: 0, after: None },
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: record.encode(bounds).expect("bounded read cursor"),
            },
        },
    }));
}
pub(super) fn configure(
    pages: &mut Pages,
    client: &jig_client::Client,
    now: Time,
    limits: &Limits,
    out: &mut Queue<Request>,
) {
    if !pages.watching {
        *pages.limit_settings = State::new();
        return;
    }
    let me = accounts::current_me(client);
    let party = match me {
        Some(me) => Some(me.party.number),
        None => None,
    };
    if pages.limit_settings.party != party {
        *pages.limit_settings = State::new();
        pages.limit_settings.party = party;
    }
    let show = pages.address == Address::Limits
        && match me {
            Some(me) => me.deployment_role.is_some(),
            None => false,
        };
    let newest = revision(client);
    let inbox = match &pages.inbox {
        Some(inbox) => Some(inbox.entries),
        None => None,
    };
    let mut held_card = false;
    if let Some(inbox) = &pages.inbox {
        for card in &inbox.cards {
            if let Some(task) = card.document() {
                held_card |= task.hold.is_some();
            }
        }
    }
    let state = &mut pages.limit_settings;
    if state.revision != newest {
        state.revision = newest;
        state.value = None;
        state.value_read = false;
        state.value_pending = None;
        if show {
            state.page = Some(LimitsPage::new());
        }
    }
    if show && state.page.is_none() {
        state.page = Some(LimitsPage::new());
    }
    if !show {
        state.page = None;
    }
    if state.inbox != inbox {
        state.inbox = inbox;
        state.refresh_holds();
    }
    if let Some(at) = state.retry_at
        && now < at
    {
        return;
    }
    state.retry_at = None;
    let Some(bounds) = forge::wire_limits(&limits.forge) else { return };
    if state.revision.is_some()
        && let Some(page) = &mut state.page
        && page.rows.is_none()
        && page.pending.is_none()
        && !page.refused
    {
        let pending = handle(&mut pages.next);
        let query = w::LimitsRead { after: page.after.clone() };
        read(pending, w::Read::Limits(query), &bounds, out);
        page.pending = Some(pending);
    }
    if party.is_some() && held_card && !state.holds_done && state.holds_pending.is_none() {
        let pending = handle(&mut pages.next);
        let query = w::LimitHoldsRead { after: state.holds_after };
        read(pending, w::Read::LimitHolds(query), &bounds, out);
        state.holds_pending = Some(pending);
    }
    if held_card
        && !state.holds.is_empty()
        && owner(me)
        && !state.value_read
        && state.value_pending.is_none()
        && let Some(newest) = newest
    {
        let pending = handle(&mut pages.next);
        let query = w::SettingRead { kind: 22, scope: 0, number: 1 };
        read(pending, w::Read::Setting(query), &bounds, out);
        state.value_pending = Some((pending, newest));
    }
}
pub(super) fn told(pages: &mut Pages, told: &Told, now: Time, limits: &Limits) -> bool {
    if let Some(inbox) = &pages.inbox {
        let refresh = match told {
            Told::Changed { handle } => *handle == inbox.entries,
            Told::Page { handle, .. } => inbox.queries.contains(handle),
            Told::Link { .. }
            | Told::Answered { .. }
            | Told::Pending { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Text { .. }
            | Told::Ended { .. } => false,
        };
        if refresh {
            pages.limit_settings.refresh_holds();
        }
    }
    let state = &mut pages.limit_settings;
    if let Told::Answered { key, outcome } = told
        && let Some((task, sent)) = state.sending
        && *key == sent
    {
        state.sending = None;
        state.refused = Some((task, outcome.clone()));
        if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::SettingsChanged(
            _,
        )))) = outcome
        {
            state.refused = None;
            state.value = None;
            state.value_read = false;
            state.refresh_holds();
        }
        return true;
    }
    let Told::Page { handle, page } = told else { return false };
    let Some(bounds) = forge::wire_limits(&limits.forge) else { return false };
    let limited = match page {
        Readout::Limit => true,
        Readout::Family { .. } | Readout::Jig(_) | Readout::NotServed { .. } => false,
    };
    let encoded = match page {
        Readout::Family { family: wire::FORGE, record: Some(encoded) } => Some(encoded.as_ref()),
        Readout::Family { .. } | Readout::Jig(_) | Readout::Limit | Readout::NotServed { .. } => None,
    };
    if let Some(open) = &mut state.page
        && open.pending == Some(*handle)
    {
        open.pending = None;
        open.rows = match encoded {
            Some(encoded) => wire::decode_limits_answer(encoded, &bounds),
            None => None,
        };
        open.refused = !limited && open.rows.is_none();
        if limited {
            state.retry_at = Some(now.saturating_add(Duration::from_secs(1)));
        }
        return true;
    }
    if state.holds_pending == Some(*handle) {
        state.holds_pending = None;
        state.holds_done = !limited;
        if limited {
            state.retry_at = Some(now.saturating_add(Duration::from_secs(1)));
        }
        if let Some(encoded) = encoded
            && let Some(answer) = wire::decode_limit_holds_answer(encoded, &bounds)
        {
            let w::LimitHoldsAnswer { rows, next } = answer;
            for row in rows.into_boxed() {
                state.holds.insert(row.task, row).expect("one row per task");
            }
            if next > state.holds_after {
                state.holds_after = next;
                state.holds_done = false;
            }
        }
        return true;
    }
    if let Some((pending, at)) = state.value_pending
        && pending == *handle
    {
        state.value_pending = None;
        state.value_read = !limited;
        if limited {
            state.retry_at = Some(now.saturating_add(Duration::from_secs(1)));
        }
        if state.revision == Some(at) {
            state.value = match encoded {
                Some(encoded) => value(encoded, &bounds),
                None => None,
            };
        }
        return true;
    }
    false
}
fn value(encoded: &[u8], bounds: &w::Limits) -> Option<w::LimitOverrides> {
    let answer = wire::decode_setting_answer(encoded, bounds)?;
    match answer.value {
        Some(encoded) => {
            let mut reader = Reader::new(&encoded);
            let value = w::LimitOverrides::decode(bounds, &mut reader).ok()?;
            if reader.is_empty() { Some(value) } else { None }
        }
        None => {
            let value = w::LimitOverrides { overrides: List::with_capacity(bounds.limit_overrides_overrides) };
            value.measure(bounds).ok()?;
            Some(value)
        }
    }
}
pub(super) fn act(pages: &mut Pages, client: &jig_client::Client, limits: &Limits, act: Act, out: &mut Queue<Request>) {
    match act {
        Act::First => {
            if let Some(page) = &mut pages.limit_settings.page {
                *page = LimitsPage::new();
            }
        }
        Act::Next => {
            if let Some(page) = &mut pages.limit_settings.page
                && let Some(rows) = &page.rows
                && rows.next.is_some()
            {
                page.after = rows.next.clone();
                page.rows = None;
                page.refused = false;
            }
        }
        Act::Raise { task, revision } => raise(pages, client, limits, task, revision, out),
    }
}
fn raise(
    pages: &mut Pages,
    client: &jig_client::Client,
    limits: &Limits,
    task: u64,
    held_revision: u64,
    out: &mut Queue<Request>,
) {
    if !owner(accounts::current_me(client)) || !pages.limit_settings.ready() {
        return;
    }
    let Some(inbox) = &pages.inbox else { return };
    let mut held = None;
    for card in &inbox.cards {
        if let Some(row) = pages.limit_settings.hold(card)
            && card.key.is_none()
            && row.task == task
            && row.revision == held_revision
        {
            held = Some(row);
        }
    }
    let Some(held) = held else { return };
    let Some(against) = revision(client) else { return };
    if pages.limit_settings.revision != Some(against) {
        return;
    }
    let Some(bounds) = forge::wire_limits(&limits.forge) else { return };
    let Some(value) = pages.limit_settings.value.as_ref() else { return };
    let Some(item) = raised_item(value, held, &bounds) else { return };
    let mut items = List::with_capacity(1);
    items.push(item).expect("one limits item");
    pages.serial = pages.serial.checked_add(1).expect("page keys fit");
    let key = Key { asker: Owner::Pages.asker(), number: pages.serial };
    pages.limit_settings.sending = Some((task, key));
    pages.limit_settings.refused = None;
    out.push(Request::Client(jig_client::Event::Send {
        key,
        name: None,
        request: jig_client::Ask::Jig(Box::new(c::Ask::ChangeSettings(c::ChangeSettings {
            against,
            items,
            running: c::Running::Stays,
        }))),
    }));
}
fn raised_item(value: &w::LimitOverrides, held: &w::LimitHold, bounds: &w::Limits) -> Option<c::SettingItem> {
    let figure = raise_figure(held.configured, held.needed)?;
    let reason = bytes::concat(&[
        b"raised for task ",
        Decimal::of(held.task).as_bytes(),
        b", which needed ",
        Decimal::of(held.needed).as_bytes(),
    ]);
    let raised = w::LimitOverride { name: held.limit.clone(), value: figure, reason };
    let mut overrides = List::with_capacity(bounds.limit_overrides_overrides);
    let mut added = false;
    for old in &value.overrides {
        if !added && old.name >= held.limit {
            overrides.push(raised.clone()).ok()?;
            added = true;
        }
        if old.name != held.limit {
            overrides.push(old.clone()).ok()?;
        }
    }
    if !added {
        overrides.push(raised).ok()?;
    }
    let value = w::LimitOverrides { overrides };
    let mut writer = Writer::new(usize::try_from(value.measure(bounds).ok()?).ok()?);
    value.write(bounds, &mut writer).ok()?;
    Some(c::SettingItem {
        setting: c::Setting::Application { kind: 22, scope: 0, number: 1 },
        edit: c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(writer.finish()) }),
    })
}
/// The least representable round figure at least twice the configured figure and at least needed.
/// No raise is offered when either the doubling or every round candidate exceeds u64.
#[must_use]
pub fn raise_figure(configured: u64, needed: u64) -> Option<u64> {
    let floor = configured.checked_mul(2)?.max(needed);
    let mut best = None;
    for exponent in 0..64 {
        let power = 1_u64.checked_shl(exponent).expect("exponent below 64");
        for candidate in [Some(power), power.checked_mul(3)].into_iter().flatten() {
            if candidate >= floor {
                best = Some(match best {
                    Some(old) => core::cmp::min(old, candidate),
                    None => candidate,
                });
            }
        }
    }
    for exponent in 0..20 {
        if let Some(power) = 10_u64.checked_pow(exponent) {
            for digit in 1..10 {
                if let Some(candidate) = power.checked_mul(digit)
                    && candidate >= floor
                {
                    best = Some(match best {
                        Some(old) => core::cmp::min(old, candidate),
                        None => candidate,
                    });
                }
            }
        }
    }
    best
}
