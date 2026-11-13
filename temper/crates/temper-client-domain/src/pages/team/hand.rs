//! Hand an area to a role, or give its named areas back (client/pages/team.md, 1).
use super::{Facts, Role};
use crate::{
    Address, Domain, Limits, Request,
    owner::Owner,
    pages::{Pages, needs::Toasted, stop_asking},
};
use alloc::boxed::Box;
use jig_client::{Handle, Key, Outcome, Readable, Readout, Told, contract as c};
use skein_lib::{Env, List, Queue, Reader, Writer, bytes::concat};
use temper_client_protocol_wire::v1 as w;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Open,
    GiveBack,
    Area(Option<Box<[u8]>>),
    Role(Option<u64>),
    Stock(Stock),
    Budget(Box<[u8]>),
    Name(Box<[u8]>),
    Words(Box<[u8]>),
    Close,
    Send,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stock {
    Prover,
    Porter,
}
impl Stock {
    #[must_use]
    pub const fn charter(self) -> u32 {
        match self {
            Self::Prover => 15,
            Self::Porter => 16,
        }
    }
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Prover => b"Prover",
            Self::Porter => b"Porter",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameProblem {
    Shape,
    Duplicate,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GiveBack {
    pub task: u64,
    pub party: u64,
    pub names: List<Box<[u8]>>,
    /// Charter owners that stood as this task when the sheet opened.
    pub charters: List<u32>,
}
#[derive(Debug)]
pub enum Failure {
    Read,
    Outcome(Outcome),
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "sheet facts are public; request correlation stays private")]
pub struct Open {
    pub project: u32,
    pub project_name: Box<[u8]>,
    pub facts: Facts,
    pub apply: bool,
    pub area: Option<Box<[u8]>>,
    pub role: Option<u64>,
    pub stock: Option<Stock>,
    pub budget: Box<[u8]>,
    pub name: Box<[u8]>,
    pub words: Box<[u8]>,
    pub judged: bool,
    pub give: Option<GiveBack>,
    pub failure: Option<Failure>,
    reading: Option<(Handle, u32)>,
    pending: Option<Key>,
    retried: bool,
    settings_toast: Option<Toasted>,
    budget_beyond: u64,
}
impl Pages {
    #[must_use]
    pub const fn hand_area(&self) -> Option<&Open> {
        self.hand_area.as_ref()
    }
}
impl Open {
    /// A sheet's initial choices from the delivered Team facts.
    #[must_use]
    pub fn new(project: u32, project_name: Box<[u8]>, facts: Facts, apply: bool, give: Option<GiveBack>) -> Self {
        let area = match &facts.areas {
            Some(areas) => match areas.rows.get(0) {
                Some(area) => Some(area.name.clone()),
                None => None,
            },
            None => None,
        };
        let role = match facts.roles.get(0) {
            Some(role) => Some(role.task),
            None => None,
        };
        Self {
            project,
            project_name,
            facts,
            apply,
            area,
            role,
            stock: None,
            budget: Box::default(),
            name: Box::default(),
            words: Box::default(),
            judged: false,
            give,
            failure: None,
            reading: None,
            pending: None,
            retried: false,
            settings_toast: None,
            budget_beyond: 0,
        }
    }
    #[must_use]
    pub const fn busy(&self) -> bool {
        self.reading.is_some() || self.pending.is_some()
    }
    #[must_use]
    pub fn locked(&self) -> bool {
        self.busy() || self.settings_toast.is_some()
    }
    #[must_use]
    pub fn name_problem(&self) -> Option<NameProblem> {
        if !valid_name(&self.name) {
            return Some(NameProblem::Shape);
        }
        if let Some(areas) = &self.facts.areas {
            for area in &areas.rows {
                if area.name == self.name {
                    return Some(NameProblem::Duplicate);
                }
            }
        }
        None
    }
    #[must_use]
    pub fn words_valid(&self) -> bool {
        valid_words(&self.words)
    }
    #[must_use]
    #[expect(clippy::manual_find, reason = "step code uses explicit loops, not closures")]
    pub fn chosen_role(&self) -> Option<&Role> {
        for role in &self.facts.roles {
            if Some(role.task) == self.role {
                return Some(role);
            }
        }
        None
    }
    #[must_use]
    pub fn budget_value(&self) -> Option<u64> {
        if self.budget.is_empty() || self.budget_beyond != 0 {
            return None;
        }
        let mut value = 0_u64;
        for byte in &self.budget {
            if !byte.is_ascii_digit() {
                return None;
            }
            value = value.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
        }
        if value == 0 { None } else { Some(value) }
    }
    fn title(&self) -> Option<&[u8]> {
        match self.stock {
            Some(stock) => Some(stock.name()),
            None => Some(&self.chosen_role()?.name),
        }
    }
    fn chosen(&self) -> Option<(&[u8], &[u8])> {
        match &self.area {
            None => Some((&self.name, &self.words)),
            Some(name) => {
                for area in &self.facts.areas.as_ref()?.rows {
                    if &area.name == name {
                        return Some((&area.name, &area.words));
                    }
                }
                None
            }
        }
    }
    fn ask_words(&self) -> Option<Box<[u8]>> {
        let (name, words) = self.chosen()?;
        Some(if name.is_empty() {
            concat(&[b"Write a standing role to own a new area on ", &self.project_name, b"."])
        } else {
            concat(&[b"Write a standing role to own ", name, b" on ", &self.project_name, b": ", words])
        })
    }
}
fn valid_name(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > 32 {
        return false;
    }
    if let Some(first) = name.first()
        && !first.is_ascii_lowercase()
    {
        return false;
    }
    for byte in name {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' {
            return false;
        }
    }
    true
}
fn valid_words(words: &[u8]) -> bool {
    if words.is_empty() || words.len() > 480 {
        return false;
    }
    if !skein_lib::bytes::is_utf8(words) {
        return false;
    }
    let mut c2 = false;
    for byte in words {
        if byte.is_ascii_control() || (c2 && (0x80..=0x9f).contains(byte)) {
            return false;
        }
        c2 = *byte == 0xc2;
    }
    true
}
#[must_use]
pub(crate) fn member(root: &Domain, project: u32) -> bool {
    if let Some(me) = root.pages.me(&root.client) {
        for row in &me.projects {
            if row.project == project {
                return true;
            }
        }
    }
    false
}
fn open(root: &mut Domain, give: bool) {
    if root.pages.hand_area.is_some() {
        return;
    }
    let Address::Team { project, .. } = root.pages.address else { return };
    let shown = super::shown(root, project);
    let Some(facts) = shown.facts() else { return };
    let parts = super::parts(facts);
    if !parts.hand_area || (give && parts.give_areas == 0) {
        return;
    }
    let Some(me) = root.pages.me(&root.client) else { return };
    let mut project_name = None;
    for row in &me.projects {
        if row.project == project {
            project_name = Some(row.name.bytes().into());
            break;
        }
    }
    let Some(project_name) = project_name else { return };
    let areas = facts.areas.as_ref().expect("hand control requires areas read");
    let give = if give {
        let task = parts.card.expect("give back has selected card");
        let Some(document) = root.pages.team.standing.document(&root.client) else { return };
        let c::Requester::Person(requester) = document.requester else { return };
        let mut names = List::with_capacity(areas.rows.len());
        let mut charters = List::with_capacity(areas.rows.len());
        for area in &areas.rows {
            if let Some(w::Holder::Task(holder)) = &area.holder
                && holder.number() == task
            {
                names.push(area.name.clone()).expect("one name per area");
                if let w::AreaOwner::Charter(charter) = &area.owner {
                    charters.push(charter.number()).expect("at most one charter per area");
                }
            }
        }
        Some(GiveBack { task, party: requester, names, charters })
    } else {
        None
    };
    root.pages.hand_area =
        Some(Open::new(project, project_name, facts.clone(), stop_asking::owner(root, project), give));
}
pub(crate) fn act(root: &mut Domain, env: &Env<Limits>, act: Act, out: &mut Queue<Request>) {
    match act {
        Act::Open => open(root, false),
        Act::GiveBack => open(root, true),
        Act::Close => close(root, env, out),
        Act::Send => submit(root, env, out),
        Act::Area(area) => {
            if let Some(open) = &mut root.pages.hand_area
                && !open.busy()
                && open.give.is_none()
                && open.settings_toast.is_none()
            {
                let Some(areas) = &open.facts.areas else { return };
                let mut valid = area.is_none() && areas.rows.len() < 16;
                for row in &areas.rows {
                    valid |= Some(&row.name) == area.as_ref();
                }
                if valid {
                    open.area = area;
                    open.failure = None;
                    open.judged = false;
                }
            }
        }
        Act::Role(role) => {
            if let Some(open) = &mut root.pages.hand_area
                && !open.busy()
                && open.give.is_none()
                && open.settings_toast.is_none()
            {
                let mut valid = role.is_none();
                for row in &open.facts.roles {
                    valid |= Some(row.task) == role;
                }
                if valid {
                    open.role = role;
                    open.stock = None;
                    open.failure = None;
                }
            }
        }
        Act::Stock(stock) => {
            if let Some(open) = &mut root.pages.hand_area
                && !open.busy()
                && open.give.is_none()
                && open.settings_toast.is_none()
            {
                open.stock = Some(stock);
                open.role = None;
                open.judged = false;
                open.failure = None;
            }
        }
        Act::Budget(budget) => {
            if let Some(open) = &mut root.pages.hand_area
                && !open.busy()
            {
                let (budget, beyond) = crate::pages::composer::held(budget, 24);
                open.budget = budget;
                open.budget_beyond = beyond;
                open.failure = None;
            }
        }
        Act::Name(name) => {
            if let Some(open) = &mut root.pages.hand_area
                && !open.busy()
            {
                if open.settings_toast.is_some() {
                    return;
                }
                open.name = crate::pages::composer::held(name, 36).0;
                open.failure = None;
            }
        }
        Act::Words(words) => {
            if let Some(open) = &mut root.pages.hand_area
                && !open.busy()
            {
                if open.settings_toast.is_some() {
                    return;
                }
                open.words = crate::pages::composer::held(words, 484).0;
                open.failure = None;
            }
        }
    }
    root.pages.shown = root.pages.shown.checked_add(1).expect("pages shown fits");
}
pub(crate) fn leave(pages: &mut Pages, out: &mut Queue<crate::pages::Request>) {
    if let Some(open) = pages.hand_area.take()
        && let Some((handle, _)) = open.reading
    {
        out.push(crate::pages::Request::Client(jig_client::Event::Unwatch { handle }));
    }
}
fn close(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if let Some(open) = root.pages.hand_area.take()
        && let Some((handle, _)) = open.reading
    {
        crate::route::client(root, env, jig_client::Event::Unwatch { handle }, out);
    }
}
fn submit(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(open) = &mut root.pages.hand_area else { return };
    if open.busy() {
        return;
    }
    if open.settings_toast.is_some() {
        open.judged = true;
        if open.budget_value().is_some() {
            start(root, env, out);
        }
        return;
    }
    if open.give.is_none() && open.role.is_none() && open.stock.is_none() {
        let Some(words) = open.ask_words() else { return };
        close(root, env, out);
        crate::route::pages(root, env, crate::pages::Event::Act { act: crate::pages::Act::AskWith { words } }, out);
        return;
    }
    open.judged = true;
    if open.give.is_none()
        && ((open.chosen_role().is_none() && open.stock.is_none())
            || (open.stock.is_some() && open.budget_value().is_none())
            || (open.area.is_none() && (open.name_problem().is_some() || !open.words_valid())))
    {
        return;
    }
    let project = open.project;
    if !member(root, project) {
        return;
    }
    let Some(revision) = root.forge.settings_newest() else { return };
    let apply = stop_asking::owner(root, project);
    let open = root.pages.hand_area.as_mut().expect("sheet is open");
    open.apply = apply;
    open.retried = false;
    read(root, env, revision, out);
}
fn next(pages: &mut Pages) -> Key {
    pages.serial = pages.serial.checked_add(1).expect("page keys fit");
    Key { asker: Owner::Pages.asker(), number: pages.serial }
}
const fn setting(project: u32) -> c::Setting {
    c::Setting::Application { kind: 14, scope: project, number: 1 }
}
fn read(root: &mut Domain, env: &Env<Limits>, revision: u32, out: &mut Queue<Request>) {
    let handle = super::new_handle(&mut root.pages.next);
    let open = root.pages.hand_area.as_mut().expect("sheet is open");
    open.pending = None;
    open.reading = Some((handle, revision));
    open.failure = None;
    let read = c::ReadSetting { setting: setting(open.project), revision, proposal: None };
    crate::route::client(
        root,
        env,
        jig_client::Event::Read { handle, read: Readable::Jig(c::Read::Setting(read)) },
        out,
    );
}
fn record(page: &Readout, project: u32, revision: u32, bounds: &w::Limits) -> Option<w::ProjectAreas> {
    let Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(document))) = page else { return None };
    if document.setting != setting(project) || document.revision != revision {
        return None;
    }
    match &document.value {
        None => w::ProjectAreas::new(bounds, w::ProjectAreasParts { areas: List::with_capacity(0) }).ok(),
        Some(c::SettingValue::Application(bytes)) => {
            let mut reader = Reader::new(bytes);
            let record = w::ProjectAreas::decode(bounds, &mut reader).ok()?;
            if reader.is_empty() { Some(record) } else { None }
        }
        Some(
            c::SettingValue::Policy(_)
            | c::SettingValue::Overlay(_)
            | c::SettingValue::Charter(_)
            | c::SettingValue::Template(_)
            | c::SettingValue::Rules(_)
            | c::SettingValue::Model(_),
        ) => None,
    }
}
fn changed(open: &Open, record: w::ProjectAreas, bounds: &w::Limits) -> Option<w::ProjectAreas> {
    let mut rows = List::with_capacity(record.areas().len().saturating_add(1));
    let mut found = false;
    for row in record.into_parts().areas.into_boxed() {
        let mut row = row.into_parts();
        if let Some(give) = &open.give {
            let held = match &row.owner {
                w::AreaOwner::Task(task) => task.number() == give.task,
                w::AreaOwner::Charter(charter) => give.charters.as_slice().contains(&charter.number()),
                w::AreaOwner::Party(_) | w::AreaOwner::Role(_) => false,
            };
            if held {
                row.owner =
                    w::AreaOwner::Party(w::AreaParty::new(bounds, w::AreaPartyParts { number: give.party }).ok()?);
                found = true;
            }
        } else if Some(&row.name) == open.area.as_ref() {
            row.owner = chosen_owner(open, bounds)?;
            found = true;
        } else if open.area.is_none() && row.name == open.name {
            return None;
        }
        rows.push(w::Area::new(bounds, row).ok()?).ok()?;
    }
    if open.give.is_none() && open.area.is_none() {
        if rows.len() >= 16 || !valid_name(&open.name) || !valid_words(&open.words) {
            return None;
        }
        rows.push(
            w::Area::new(
                bounds,
                w::AreaParts { name: open.name.clone(), words: open.words.clone(), owner: chosen_owner(open, bounds)? },
            )
            .ok()?,
        )
        .ok()?;
    } else if !found {
        return None;
    }
    w::ProjectAreas::new(bounds, w::ProjectAreasParts { areas: rows }).ok()
}
fn chosen_owner(open: &Open, bounds: &w::Limits) -> Option<w::AreaOwner> {
    match open.stock {
        Some(stock) => Some(w::AreaOwner::Charter(
            w::AreaCharter::new(bounds, w::AreaCharterParts { number: stock.charter() }).ok()?,
        )),
        None => Some(w::AreaOwner::Task(
            w::AreaTask::new(bounds, w::AreaTaskParts { number: open.chosen_role()?.task }).ok()?,
        )),
    }
}
fn send(root: &mut Domain, env: &Env<Limits>, page: &Readout, revision: u32, out: &mut Queue<Request>) -> Option<()> {
    let open = root.pages.hand_area.as_ref()?;
    let bounds = crate::forge::wire_limits(&env.limits.forge)?;
    let record = changed(open, record(page, open.project, revision, &bounds)?, &bounds)?;
    let mut writer = Writer::new(usize::try_from(record.measure()).ok()?);
    record.encode(&mut writer).ok()?;
    let mut items = List::with_capacity(1);
    items
        .push(c::SettingItem {
            setting: setting(open.project),
            edit: c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(writer.finish()) }),
        })
        .expect("one whole areas item");
    let ask = if open.apply {
        c::Ask::ChangeSettings(c::ChangeSettings { against: revision, items, running: c::Running::Stays })
    } else {
        let reason = if let Some(give) = &open.give {
            if give.names.len() == 1 {
                concat(&[b"Give ", give.names.get(0)?.as_ref(), b" back"])
            } else {
                concat(&[b"Give ", skein_lib::Decimal::of(u64::from(give.names.len())).as_bytes(), b" areas back"])
            }
        } else {
            concat(&[b"Hand ", open.chosen()?.0, b" to ", open.title()?])
        };
        c::Ask::Propose {
            project: 0,
            action: c::ProposeAction::Settings(c::ProposeSettings { against: revision, items }),
            reason: c::Text::new(reason, env.limits.client.domain.text).ok()?,
        }
    };
    let key = next(&mut root.pages);
    root.pages.hand_area.as_mut()?.pending = Some(key);
    crate::route::client(
        root,
        env,
        jig_client::Event::Send { key, name: None, request: jig_client::Ask::Jig(Box::new(ask)) },
        out,
    );
    Some(())
}
fn start_ask(root: &Domain, env: &Env<Limits>) -> Result<c::Ask, c::Refusal> {
    use jig_client::shapes::{self, GoalDraft, Judged};
    let open = root.pages.hand_area.as_ref().expect("sheet is open");
    let stock = open.stock.expect("stock settings have answered");
    let Some(me) = root.pages.me(&root.client) else { return Err(c::Refusal::NotOffered) };
    let Some((name, _)) = open.chosen() else { return Err(c::Refusal::NotOffered) };
    let words = concat(&[stock.name(), b"\n\nOwns the area ", name, b" on ", &open.project_name, b"."]);
    let Ok(words) = c::Text::new(words, env.limits.client.domain.long_text) else {
        return Err(c::Refusal::NotOffered);
    };
    let draft = GoalDraft {
        words,
        executor: c::Executor::Agent { charter: stock.charter() },
        budget: open.budget_value().expect("budget judged before settings"),
        deadline: None,
        priority: None,
        llm: None,
    };
    let reason = c::Text::new(Box::default(), 0).expect("empty reason");
    match shapes::set_goal_from(me, open.project, 7, draft, reason) {
        Judged::Create(ask) | Judged::Propose { ask, .. } => Ok(ask),
        Judged::Refused(why) => Err(why),
        Judged::NotOffered(_) => Err(c::Refusal::NotOffered),
    }
}
fn start(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    match start_ask(root, env) {
        Ok(ask) => {
            let key = next(&mut root.pages);
            let open = root.pages.hand_area.as_mut().expect("sheet is open");
            open.pending = Some(key);
            open.failure = None;
            crate::route::client(
                root,
                env,
                jig_client::Event::Send { key, name: None, request: jig_client::Ask::Jig(Box::new(ask)) },
                out,
            );
        }
        Err(refusal) => {
            root.pages.hand_area.as_mut().expect("sheet is open").failure =
                Some(Failure::Outcome(Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
                    refusal,
                    at: None,
                }))));
        }
    }
}
fn started(root: &mut Domain, env: &Env<Limits>, outcome: &Outcome) {
    if let Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
        c::DocumentOutcome::Created { .. } | c::DocumentOutcome::Proposed { .. },
    )))) = outcome
    {
        let toast = root.pages.hand_area.take().expect("sheet is open").settings_toast.expect("settings answered");
        root.pages.needs.say(toast, env.now);
    } else {
        let open = root.pages.hand_area.as_mut().expect("sheet is open");
        open.pending = None;
        open.failure = Some(Failure::Outcome(outcome.clone()));
    }
    root.pages.shown = root.pages.shown.checked_add(1).expect("pages shown fits");
}
pub(crate) fn told(root: &mut Domain, env: &Env<Limits>, told: &Told, out: &mut Queue<Request>) -> bool {
    let Some(open) = &root.pages.hand_area else { return false };
    if let Some((reading, revision)) = open.reading {
        let page = match told {
            Told::Page { handle, page } if *handle == reading => Some(page.clone()),
            Told::Ended { handle, .. } if *handle == reading => None,
            Told::Changed { .. }
            | Told::Text { .. }
            | Told::Page { .. }
            | Told::Ended { .. }
            | Told::Link { .. }
            | Told::Dropped { .. }
            | Told::Kept { .. }
            | Told::Pending { .. }
            | Told::Answered { .. } => return false,
        };
        root.pages.hand_area.as_mut().expect("sheet is open").reading = None;
        crate::route::client(root, env, jig_client::Event::Unwatch { handle: reading }, out);
        let sent = match page {
            Some(page) => send(root, env, &page, revision, out).is_some(),
            None => false,
        };
        if !sent {
            root.pages.hand_area.as_mut().expect("sheet is open").failure = Some(Failure::Read);
        }
        root.pages.shown = root.pages.shown.checked_add(1).expect("pages shown fits");
        return true;
    }
    let Told::Answered { key, outcome } = told else { return false };
    if open.pending != Some(*key) {
        return false;
    }
    if open.settings_toast.is_some() {
        started(root, env, outcome);
        return true;
    }
    if let Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::Stale(stale), ..
    })) = outcome
        && !open.retried
    {
        root.pages.hand_area.as_mut().expect("sheet is open").retried = true;
        read(root, env, stale.newest, out);
        return true;
    }
    let toast =
        if let Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::SettingsChanged(changed)))) =
            outcome
            && open.apply
        {
            Some(if open.give.is_some() {
                Toasted::AreasGivenBack { revision: changed.revision }
            } else {
                Toasted::AreaHanded {
                    title: open.title().expect("sent to a chosen role").into(),
                    revision: changed.revision,
                }
            })
        } else if !open.apply
            && let Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::Proposed { .. },
            )))) = outcome
        {
            Some(Toasted::AreasProposed)
        } else {
            None
        };
    if let Some(toast) = toast {
        if open.stock.is_some() && open.give.is_none() {
            let open = root.pages.hand_area.as_mut().expect("sheet is open");
            open.pending = None;
            open.settings_toast = Some(toast);
            start(root, env, out);
        } else {
            root.pages.needs.say(toast, env.now);
            root.pages.hand_area = None;
        }
    } else {
        let open = root.pages.hand_area.as_mut().expect("sheet is open");
        open.pending = None;
        open.failure = Some(Failure::Outcome(outcome.clone()));
    }
    root.pages.shown = root.pages.shown.checked_add(1).expect("pages shown fits");
    true
}
