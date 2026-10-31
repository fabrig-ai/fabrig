//! Keep: bounded working sets, scoped read/watch lifetimes and owner routing.
//! Replace: the connector vocabulary, its families, words and revision-naming acts.
//! State is a bounded subject registry, page/part demand, scoped reads and watches.
//! New subjects begin Loading; Unknown never supplies an absent owner fact.
//! Entry points: step retains values and demand; reclaim frees terminals.
//! jig-client owns watch reopening and retry deadlines.
//! | Event | State change | Requests |
//! | Went/Show | acquire scoped demand | Read, Open |
//! | Read/Delivered | validate and retain whole owner document | `Named` |
//! | Left/PartsLeft | release demand, retain close obligation | Close |
//! | Closed/Unknown | retire watch; unknown hides supplied facts | replacement Read/Open |
//! | Confirmed/Link | restrict lifetimes to confirmed party and live link | reacquire or hide |
pub mod documents;
pub mod values;
use alloc::boxed::Box;
pub use documents::{Answer, Ask};
pub use documents::{Document, Kind, ReadResult, Scope, Subject, WatchEnd, WatchEvent};
use jig_client::Link;
use jig_client::contract::{ApplicationPart, Revision};
use skein_lib::{Env, Id, List, Map, Queue, RequestProgress, Slab, Token};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Address {
    Environment { project: u32, environment: values::Environment },
    Environments { project: u32, pool: Option<values::Pool> },
}
impl Address {
    #[must_use]
    pub fn subject(self) -> Subject {
        match self {
            Self::Environment { project, environment } => {
                Subject { scope: Scope::Project(project), kind: Kind::Environment(environment) }
            }
            Self::Environments { project, pool } => {
                Subject { scope: Scope::Project(project), kind: Kind::Environments { pool } }
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Go { address: Address },
    Until { object: Token, revision: Revision, value: u64 },
    MoveEnd { object: Token, revision: Revision },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Place,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Place { version: u16, party: u64, address: Address },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Out {
    Read { read: Token, subject: Subject },
    Open { watch: Token, subject: Subject },
    Close { watch: Token },
    Fact { fact: Fact },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    ReadNotAllowed,
    WatchTerminal { end: WatchEnd },
    Full,
    InvalidDocument,
    SignedOut,
}
#[derive(Debug)]
pub enum Event {
    Show { part: ApplicationPart, scope: Scope },
    PartsLeft,
    Went { address: Address },
    Left,
    Act { act: Act },
    Answered { request: Token, answer: Answer },
    Progress { request: Token, progress: RequestProgress },
    Read { read: Token, result: ReadResult },
    Watch { watch: Token, event: WatchEvent },
    Ended { watch: Token, end: WatchEnd },
    Restore { record: Record },
}
#[derive(Debug)]
pub enum Request {
    Client { asked: jig_client::Event },
    Show { address: Address },
    Save { record: Record },
    Send { request: Token, scope: u64, ask: Ask },
    Out(Out),
    Named { objects: List<jig_client::contract::ObjectKey>, sections: List<jig_client_view::page::Section> },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub objects: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Loading,
    Unknown,
    Known(Box<Document>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Card {
    Open,
    Deciding { request: Token, progress: RequestProgress },
    Refused { answer: Answer },
    Unknown,
    Done { answer: Answer },
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the view reads values; the side alone owns read and watch obligations")]
pub struct Object {
    pub subject: Subject,
    pub value: Value,
    pub card: Card,
    pub draft: Option<u64>,
    read: Option<Id<Reading>>,
    watch: Option<Id<Watching>>,
}
#[derive(Debug)]
struct Held {
    part: ApplicationPart,
    subject: Option<Subject>,
}
#[derive(Debug)]
struct Reading {
    live: bool,
    subject: Subject,
    party: u64,
}
#[derive(Debug)]
struct Watching {
    subject: Subject,
    closing: bool,
}
#[derive(Debug)]
struct Pending {
    live: bool,
    object: Id<Object>,
}
#[derive(Debug)]
pub struct Side {
    parts: List<Held>,
    objects: Slab<Object>,
    names: Map<Subject, Id<Object>>,
    reads: Slab<Reading>,
    watches: Slab<Watching>,
    watch_ids: Map<Id<Watching>, ()>,
    pending: Slab<Pending>,
    address: Option<Address>,
    generation: u32,
    link: Link,
    party: Option<u64>,
    me: bool,
    kept: Option<Record>,
    shown: u64,
}
impl Side {
    #[must_use]
    pub fn new(limits: &Limits, client: &jig_client::DomainLimits) -> Self {
        Self {
            parts: List::with_capacity(client.parts),
            objects: Slab::with_capacity(limits.objects.checked_mul(2).expect("object retirement room fits")),
            names: Map::with_capacity(limits.objects),
            reads: Slab::with_capacity(client.reads),
            watches: Slab::with_capacity(client.watches),
            watch_ids: Map::with_capacity(client.watches),
            pending: Slab::with_capacity(client.requests.requests),
            address: None,
            generation: 0,
            link: Link::Starting,
            party: None,
            me: false,
            kept: None,
            shown: 0,
        }
    }
    #[must_use]
    pub fn address(&self) -> Option<Address> {
        self.address.clone()
    }
    #[must_use]
    pub const fn generation(&self) -> u32 {
        self.generation
    }
    #[must_use]
    pub const fn link(&self) -> Link {
        self.link
    }
    #[must_use]
    pub const fn party(&self) -> Option<u64> {
        self.party
    }
    #[must_use]
    pub fn find(&self, subject: Subject) -> Option<Id<Object>> {
        self.names.get(&subject).copied()
    }
    #[must_use]
    pub fn object(&self, id: Id<Object>) -> Option<&Object> {
        let object = self.objects.get(id)?;
        if self.names.get(&object.subject) == Some(&id) { Some(object) } else { None }
    }
    #[must_use]
    pub fn page(&self) -> Option<Id<Object>> {
        let address = self.address.clone()?;
        self.find(address.subject())
    }
    #[must_use]
    pub fn part(&self, part: &ApplicationPart) -> Option<&Object> {
        for held in &self.parts {
            if &held.part == part {
                let subject = held.subject.clone()?;
                return self.object(self.find(subject)?);
            }
        }
        None
    }
    #[must_use]
    pub fn contains(&self, part: &ApplicationPart) -> bool {
        for held in &self.parts {
            if &held.part == part {
                return true;
            }
        }
        false
    }
    fn changed(&mut self) {
        self.shown = self.shown.checked_add(1).expect("shown count fits");
    }
}
#[must_use]
pub const fn shown(side: &Side) -> u64 {
    side.shown
}
pub fn step(side: &mut Side, env: &Env<jig_client::DomainLimits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Show { part, scope } => {
            if !part_fits(&part, env.limits.segments, env.limits.text) {
                full(out);
                return;
            }
            if side.contains(&part) {
                return;
            }
            let subject = documents::subject(&part, scope);
            if side.parts.push(Held { part, subject: subject.clone() }).is_err() {
                full(out);
                return;
            }
            if let Some(subject) = subject {
                hold(side, subject, out);
            }
            side.changed();
        }
        Event::PartsLeft => {
            side.parts.clear();
            prune(side, out);
            side.changed();
        }
        Event::Went { address } => {
            if documents::subject_price(&address.clone().subject(), &env.limits).is_none() {
                out.push(Request::Out(Out::Fact { fact: Fact::InvalidDocument }));
                return;
            }
            side.address = Some(address.clone());
            side.generation = side.generation.checked_add(1).expect("page generation fits");
            prune(side, out);
            side.changed();
            hold(side, address.subject(), out);
            save(side, out);
        }
        Event::Left => {
            side.address = None;
            out.push(Request::Named { objects: List::with_capacity(0), sections: List::with_capacity(0) });
            prune(side, out);
            side.changed();
        }
        Event::Act { act } => match act {
            Act::Go { address } => out.push(Request::Show { address }),
            Act::MoveEnd { object, revision } => move_end(side, Id::from_token(object), revision, out),
            Act::Until { object, revision, value } => {
                let id = Id::from_token(object);
                if let Some(object) = side.objects.get_mut(id)
                    && parameters_revision(&object.value) == Some(revision)
                {
                    object.draft = Some(value);
                    side.changed();
                }
            }
        },
        Event::Answered { request, answer } => answered(side, request, answer),
        Event::Progress { request, progress } => progress_changed(side, request, progress, out),
        Event::Read { read, result } => read_result(side, env, read, result, out),
        Event::Watch { watch, event } => watched(side, env, watch, event, out),
        Event::Ended { watch, end } => ended(side, Id::from_token(watch), end, out),
        Event::Restore { record } => match record {
            Record::Place { version, party, address } => {
                if version == 1 && documents::subject_price(&address.clone().subject(), &env.limits).is_some() {
                    side.kept = Some(Record::Place { version, party, address });
                }
            }
        },
    }
    prune(side, out);
}
fn save(side: &mut Side, out: &mut Queue<Request>) {
    if let Some(party) = side.party
        && let Some(address) = side.address.clone()
    {
        let record = Record::Place { version: 1, party, address };
        side.kept = Some(record.clone());
        out.push(Request::Save { record });
    }
}
fn document_revision(document: &Document) -> Option<Revision> {
    match document {
        Document::Value(document) => match &document.value {
            values::Value::Environment { end_revision, .. } => Some(Revision { value: *end_revision }),
            values::Value::Project { .. }
            | values::Value::Panel { .. }
            | values::Value::Result { .. }
            | values::Value::Effect { .. }
            | values::Value::Outbox { .. }
            | values::Value::Hold { .. }
            | values::Value::Deployment { .. } => None,
        },
    }
}
fn parameters_revision(value: &Value) -> Option<Revision> {
    match value {
        Value::Known(document) => document_revision(document),
        Value::Loading | Value::Unknown => None,
    }
}
fn hold(side: &mut Side, subject: Subject, out: &mut Queue<Request>) {
    if side.find(subject.clone()).is_none() {
        if side.names.len() == side.names.capacity() {
            full(out);
            return;
        }
        let object = Object {
            subject: subject.clone(),
            value: Value::Loading,
            card: Card::Open,
            draft: None,
            read: None,
            watch: None,
        };
        let Ok(id) = side.objects.insert(object) else {
            full(out);
            return;
        };
        assert!(side.names.insert(subject.clone(), id).is_ok(), "names mirror object capacity");
    }
    acquire(side, subject, out);
}
fn acquire(side: &mut Side, subject: Subject, out: &mut Queue<Request>) {
    if side.party.is_none() || side.link != Link::Live {
        return;
    }
    let Some(id) = side.find(subject.clone()) else {
        return;
    };
    let object = side.objects.get_mut(id).expect("named object exists");
    match object.value {
        Value::Unknown => return,
        Value::Known(_) | Value::Loading => {}
    }
    if object.read.is_none() && matches_loading(&object.value) {
        match side.reads.insert(Reading {
            live: true,
            subject: subject.clone(),
            party: side.party.expect("only a confirmed party reads"),
        }) {
            Ok(read) => {
                object.read = Some(read);
                out.push(Request::Out(Out::Read { read: read.token(), subject: subject.clone() }));
            }
            Err(_) => full(out),
        }
    }
    if object.watch.is_none() {
        match side.watches.insert(Watching { subject: subject.clone(), closing: false }) {
            Ok(watch) => {
                assert!(side.watch_ids.insert(watch, ()).is_ok(), "watch index mirrors slab");
                object.watch = Some(watch);
                out.push(Request::Out(Out::Open { watch: watch.token(), subject }));
            }
            Err(_) => full(out),
        }
    }
}
fn matches_loading(value: &Value) -> bool {
    match value {
        Value::Loading => true,
        Value::Known(_) | Value::Unknown => false,
    }
}
fn received(
    side: &mut Side,
    env: &Env<jig_client::DomainLimits>,
    subject: Subject,
    document: Document,
    out: &mut Queue<Request>,
) {
    if document.subject() != subject || !document.fits(&env.limits) {
        out.push(Request::Out(Out::Fact { fact: Fact::InvalidDocument }));
        return;
    }
    let Some(id) = side.find(subject.clone()) else {
        return;
    };
    let object = side.objects.get_mut(id).expect("named object exists");
    match &object.value {
        Value::Known(current) => {
            if document_revision(current) != document_revision(&document) {
                object.draft = None;
            }
        }
        Value::Loading | Value::Unknown => {}
    }
    object.value = Value::Known(Box::new(document));
    match object.card {
        Card::Unknown => object.card = Card::Open,
        Card::Open | Card::Deciding { .. } | Card::Refused { .. } | Card::Done { .. } => {}
    }
    if side.page() == Some(id) {
        let object = side.objects.get(id).expect("page object exists");
        match &object.value {
            Value::Known(document) => out
                .push(Request::Named { objects: document.objects(env.limits.rows), sections: List::with_capacity(0) }),
            Value::Loading | Value::Unknown => {}
        }
    }
    side.changed();
}
fn unknown(side: &mut Side, subject: Subject) {
    if let Some(id) = side.find(subject.clone()) {
        side.objects.get_mut(id).expect("named object exists").value = Value::Unknown;
        side.changed();
    }
}
fn named(side: &Side, subject: Subject) -> bool {
    if let Some(address) = side.address.clone()
        && subject == (address.subject())
    {
        return true;
    }
    for held in &side.parts {
        if held.subject == Some(subject.clone()) {
            return true;
        }
    }
    false
}
fn prune(side: &mut Side, out: &mut Queue<Request>) {
    let mut gone = List::with_capacity(side.names.capacity());
    for (subject, _) in &side.names {
        let object = side
            .objects
            .get(*side.names.get(subject).expect("registered subject exists"))
            .expect("named object exists");
        let pending = match object.card {
            Card::Deciding { .. } => true,
            Card::Open | Card::Refused { .. } | Card::Unknown | Card::Done { .. } => false,
        };
        if !named(side, subject.clone()) && !pending {
            assert!(gone.push(subject.clone()).is_ok(), "pruned keys fit names");
        }
    }
    for subject in gone.into_boxed() {
        let id = side.names.remove(&subject).expect("pruned subject exists");
        if let Some(object) = side.objects.get_mut(id) {
            if let Some(watch) = object.watch.take() {
                close(side, watch, out);
            }
            side.objects.retire(id);
        }
    }
}
fn close(side: &mut Side, id: Id<Watching>, out: &mut Queue<Request>) {
    if let Some(watch) = side.watches.get_mut(id)
        && !watch.closing
    {
        watch.closing = true;
        out.push(Request::Out(Out::Close { watch: id.token() }));
    }
}
fn retire_watch(side: &mut Side, id: Id<Watching>) {
    let subject = side.watches.get(id).expect("retired watch exists").subject.clone();
    if let Some(object) = side.find(subject) {
        let object = side.objects.get_mut(object).expect("named object exists");
        if object.watch == Some(id) {
            object.watch = None;
        }
    }
    let old = side.watch_ids.remove(&id);
    assert!(old.is_some(), "retired watch was indexed");
    side.watches.retire(id);
}
fn hide(side: &mut Side, out: &mut Queue<Request>) {
    let mut ids = List::with_capacity(side.watch_ids.capacity());
    for (id, ()) in &side.watch_ids {
        assert!(ids.push(*id).is_ok(), "watch ids fit");
    }
    for id in ids.into_boxed() {
        close(side, id, out);
    }
    for (_, id) in &side.names {
        let object = side.objects.get_mut(*id).expect("named object exists");
        object.value = Value::Loading;
    }
}
fn sync(side: &mut Side, out: &mut Queue<Request>) {
    let mut subjects = List::with_capacity(side.names.capacity());
    for (subject, _) in &side.names {
        assert!(subjects.push(subject.clone()).is_ok(), "subjects fit registry");
    }
    for subject in subjects.into_boxed() {
        acquire(side, subject, out);
    }
    if side.link != Link::Live {
        hide(side, out);
    }
}
fn ended(side: &mut Side, id: Id<Watching>, end: WatchEnd, out: &mut Queue<Request>) {
    let Some(watch) = side.watches.get(id) else { return };
    let subject = watch.subject.clone();
    let closing = watch.closing;
    retire_watch(side, id);
    if !closing {
        match end {
            WatchEnd::NotAllowed | WatchEnd::SubjectEnded => {
                unknown(side, subject.clone());
                out.push(Request::Out(Out::Fact { fact: Fact::WatchTerminal { end } }));
            }
            WatchEnd::Unknown => unknown(side, subject.clone()),
            WatchEnd::SignedOut => out.push(Request::Out(Out::Fact { fact: Fact::SignedOut })),
            WatchEnd::Closed => {}
        }
    }
    acquire(side, subject, out);
    side.changed();
}

fn move_end(side: &mut Side, id: Id<Object>, revision: Revision, out: &mut Queue<Request>) {
    let Some(party) = side.party else { return };
    let Some(object) = side.object(id) else { return };
    match object.card {
        Card::Deciding { .. } | Card::Unknown => return,
        Card::Open | Card::Refused { .. } | Card::Done { .. } => {}
    }
    let document = match &object.value {
        Value::Known(document) => document.as_ref(),
        Value::Loading | Value::Unknown => return,
    };
    let Document::Value(values) = document;
    let project = match values.subject.scope {
        Scope::Project(project) => project,
        Scope::Deployment => return,
    };
    let (environment, current, until) = match &values.value {
        values::Value::Environment { environment, end_revision, end, fact, .. } => {
            let until = match end {
                Some(end) => end.until,
                None => fact.until,
            };
            (environment.clone(), Revision { value: *end_revision }, until)
        }
        values::Value::Project { .. }
        | values::Value::Panel { .. }
        | values::Value::Result { .. }
        | values::Value::Effect { .. }
        | values::Value::Outbox { .. }
        | values::Value::Hold { .. }
        | values::Value::Deployment { .. } => return,
    };
    if current != revision {
        side.objects.get_mut(id).expect("named environment exists").card =
            Card::Refused { answer: Answer::Moved { environment, revision: current, until } };
        side.changed();
        return;
    }
    let Some(until) = object.draft else { return };
    let Ok(request) = side.pending.insert(Pending { live: true, object: id }) else {
        full(out);
        return;
    };
    side.objects.get_mut(id).expect("named environment exists").card =
        Card::Deciding { request: request.token(), progress: RequestProgress::Parked };
    out.push(Request::Send {
        request: request.token(),
        scope: party,
        ask: Ask::MoveEnd { object: id.token(), project, environment, revision, until },
    });
    side.changed();
}
fn full(out: &mut Queue<Request>) {
    out.push(Request::Out(Out::Fact { fact: Fact::Full }));
}
pub fn reclaim(side: &mut Side) {
    side.objects.reclaim();
    side.reads.reclaim();
    side.watches.reclaim();
    side.pending.reclaim();
}

/// Whether the connector's receiving limits and derived capacities are usable.
#[must_use]
pub fn valid(limits: &Limits, client: &jig_client::DomainLimits) -> bool {
    client.document_bytes != 0 && limits.objects.checked_mul(2).is_some() && client.rows.checked_add(1).is_some()
}

fn read_result(
    side: &mut Side,
    env: &Env<jig_client::DomainLimits>,
    read: Token,
    result: ReadResult,
    out: &mut Queue<Request>,
) {
    let id = Id::from_token(read);
    let Some(reading) = side.reads.get_mut(id) else {
        return;
    };
    if !reading.live {
        return;
    }
    reading.live = false;
    let subject = reading.subject.clone();
    let party = reading.party;
    side.reads.retire(id);
    let Some(object) = side.find(subject.clone()) else {
        return;
    };
    let object = side.objects.get_mut(object).expect("named object exists");
    if object.read != Some(id) {
        return;
    }
    object.read = None;
    if side.party != Some(party) {
        acquire(side, subject, out);
        return;
    }
    match result {
        ReadResult::Value(document) => received(side, env, subject, *document, out),
        ReadResult::NotAllowed => {
            unknown(side, subject.clone());
            out.push(Request::Out(Out::Fact { fact: Fact::ReadNotAllowed }));
        }
        ReadResult::Unknown => unknown(side, subject),
        ReadResult::Busy => full(out),
        ReadResult::SignedOut => out.push(Request::Out(Out::Fact { fact: Fact::SignedOut })),
    }
}

fn watched(
    side: &mut Side,
    env: &Env<jig_client::DomainLimits>,
    watch: Token,
    event: WatchEvent,
    out: &mut Queue<Request>,
) {
    let id = Id::from_token(watch);
    if side.watch_ids.get(&id).is_none() {
        return;
    }
    let Some(watching) = side.watches.get_mut(id) else {
        return;
    };
    if watching.closing {
        return;
    }
    let subject = watching.subject.clone();
    match event {
        WatchEvent::Delivered(document) => received(side, env, subject, *document, out),
    }
}

fn answered(side: &mut Side, request: Token, answer: Answer) {
    let id = Id::from_token(request);
    if let Some(pending) = side.pending.get_mut(id) {
        if !pending.live {
            return;
        }
        let object = pending.object;
        pending.live = false;
        side.pending.retire(id);
        if let Some(object) = side.objects.get_mut(object) {
            object.card = match &answer {
                Answer::MovedEnd { .. } => Card::Done { answer: answer.clone() },
                Answer::Moved { .. } | Answer::Gone | Answer::Refused { .. } | Answer::EntranceRefused(_) => {
                    Card::Refused { answer: answer.clone() }
                }
            };
            side.changed();
        }
    }
}

fn progress_changed(side: &mut Side, request: Token, progress: RequestProgress, out: &mut Queue<Request>) {
    let id = Id::from_token(request);
    if let Some(pending) = side.pending.get_mut(id) {
        if !pending.live {
            return;
        }
        let object = pending.object;
        if let Some(object) = side.objects.get_mut(object) {
            object.card = match progress {
                RequestProgress::Unknown => Card::Unknown,
                RequestProgress::InFlight | RequestProgress::Retrying | RequestProgress::Parked => {
                    Card::Deciding { request, progress }
                }
            };
            side.changed();
        }
        if progress == RequestProgress::Unknown {
            side.pending.get_mut(id).expect("pending exists").live = false;
            side.pending.retire(id);
            if let Some(object) = side.objects.get_mut(object) {
                object.value = Value::Loading;
                let subject = object.subject.clone();
                acquire(side, subject, out);
            }
        }
    }
}

fn part_fits(part: &ApplicationPart, segments: u32, text: u32) -> bool {
    match part {
        ApplicationPart::Resource { path, .. } => jig_client::contract::resource_path_fits(path, segments, text),
        ApplicationPart::Connector { .. }
        | ApplicationPart::Kind { .. }
        | ApplicationPart::Panel { .. }
        | ApplicationPart::Result { .. }
        | ApplicationPart::OwnerEffect { .. }
        | ApplicationPart::Effect { .. }
        | ApplicationPart::Outbox { .. }
        | ApplicationPart::Hold { .. }
        | ApplicationPart::Project { .. }
        | ApplicationPart::Deployment { .. } => true,
    }
}

/// Read this side's party from its own client handle.
pub(crate) fn me_told(
    side: &mut Side,
    client: &jig_client::Client,
    told: &jig_client::Told,
    out: &mut Queue<Request>,
) -> bool {
    let Some(update) =
        crate::me::told(jig_ops_domain_connectors::Connector::Infrastructure, &mut side.me, client, told)
    else {
        return false;
    };
    if let Some(asked) = update.asked {
        out.push(Request::Client { asked });
    }
    match update.change {
        crate::me::Change::Link(link) => {
            side.link = link;
            if link == Link::SignedOut || link == Link::OutOfDate {
                party(side, None, out);
            }
            sync(side, out);
            side.changed();
        }
        crate::me::Change::Party(value) => party(side, value, out),
        crate::me::Change::Unchanged => {}
    }
    true
}
fn party(side: &mut Side, party: Option<u64>, out: &mut Queue<Request>) {
    if side.party != party {
        hide(side, out);
    }
    side.party = party;
    sync(side, out);
    save(side, out);
    side.changed();
}
