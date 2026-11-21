//! Full tables name their holders without changing admission (protocol/README.md, 6.3).
use super::Domain;
use crate::{FullKind, FullTable, Limits, Report, Request, age_words};
use alloc::boxed::Box;
use skein_forge::api::Read;
use skein_forge_client::{CallOwner, Holding};
use skein_lib::{Decimal, Env, List, Map, Queue, Time, bytes};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Reservation {
    Asked(crate::room::Waiter),
    Call(skein_lib::Token),
    Projection(u64),
}
#[derive(Hash, Debug)]
struct Full {
    clock: Report,
    wall: u64,
    words: Option<Box<[u8]>>,
}
#[derive(Hash, Debug)]
pub(crate) struct State {
    outbox: Option<Full>,
    reads: Option<Full>,
    reservations: Map<Reservation, (u64, u64)>,
    sequence: u64,
    pub(crate) dirty: bool,
}
impl State {
    pub(crate) fn new(entries: u32) -> Self {
        Self { outbox: None, reads: None, reservations: Map::with_capacity(entries), sequence: 0, dirty: false }
    }
    pub(crate) fn raise(&mut self, entries: u32) {
        self.reservations.raise(entries);
    }
    pub(crate) fn reserve(&mut self, reservation: Reservation, task: u64) {
        if self.reservations.contains_key(&reservation) {
            return;
        }
        self.sequence = self.sequence.checked_add(1).expect("reservation sequence fits a process");
        self.reservations.insert(reservation, (self.sequence, task)).expect("reservations fit outbox");
    }
    pub(crate) fn document(&self) -> Box<[FullTable]> {
        let mut full = List::with_capacity(2);
        for (table, state) in [(FullKind::Outbox, &self.outbox), (FullKind::ReadAdmission, &self.reads)] {
            if let Some(state) = state
                && let Some(words) = &state.words
            {
                full.push(FullTable { table, since: state.wall, words: words.clone() }).expect("two tables");
            }
        }
        full.into_boxed()
    }
    pub(crate) fn deadline(&self) -> Option<Time> {
        let mut next = None;
        for state in [&self.outbox, &self.reads].into_iter().flatten() {
            let due = state.clock.deadline();
            next = Some(match next {
                Some(before) => due.min(before),
                None => due,
            });
        }
        next
    }
}
fn start(state: &mut Option<Full>, now: Time, wall: u64) {
    if state.is_none() {
        *state = Some(Full { clock: Report::new(now), wall, words: None });
    }
}
/// Reservations made outside a connector event still start at the supplied clock.
pub(crate) fn filled(d: &mut Domain, env: &Env<Limits>) {
    if crate::room::free(d, &env.limits) == 0 {
        start(&mut d.full.outbox, env.now, env.wall.as_nanos());
    }
}
pub(crate) fn update(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    d.full.dirty = false;
    let mut gone = List::with_capacity(d.full.reservations.len());
    for (reservation, _) in &d.full.reservations {
        let live = match reservation {
            Reservation::Asked(waiter) => d.room.asked.contains(waiter),
            Reservation::Call(owner) => d.call_room.contains(owner),
            Reservation::Projection(entry) => d.projection_room.contains(entry),
        };
        if !live {
            gone.push(*reservation).expect("counted reservations");
        }
    }
    for reservation in gone.into_boxed() {
        d.full.reservations.remove(&reservation);
    }
    let outbox_full = crate::room::free(d, &env.limits) == 0;
    let reads_full = !d.client.has_room();
    let outbox = if outbox_full { Some(outbox_words(d, env.limits.entries)) } else { None };
    let reads = if reads_full { Some(read_words(d, env.limits.client.pending)) } else { None };
    advance(&mut d.full.outbox, env, FullKind::Outbox, outbox, out);
    advance(&mut d.full.reads, env, FullKind::ReadAdmission, reads, out);
}
fn table_name(table: FullKind) -> &'static [u8] {
    match table {
        FullKind::Outbox => b"the forge's outbox",
        FullKind::ReadAdmission => b"the forge's read admission",
    }
}
fn advance(
    state: &mut Option<Full>,
    env: &Env<Limits>,
    table: FullKind,
    words: Option<Box<[u8]>>,
    out: &mut Queue<Request>,
) {
    let Some(words) = words else {
        if let Some(ended) = state.take()
            && ended.words.is_some()
        {
            out.push(Request::FullTableFact { words: bytes::concat(&[table_name(table), b" has room again"]) });
            out.push(Request::MarkedForges);
        }
        return;
    };
    start(state, env.now, env.wall.as_nanos());
    let full = state.as_mut().expect("full table has its clock");
    let age = full.clock.due(env.now);
    if age.is_some() || full.words.is_some() {
        if full.words.as_deref() != Some(&words) {
            full.words = Some(words.clone());
            out.push(Request::MarkedForges);
        }
        if let Some(age) = age {
            out.push(Request::FullTableFact {
                words: bytes::concat(&[table_name(table), b" full for ", &age_words(age), b": ", &words]),
            });
        }
    }
}
// Eight names each capped at 88 bytes leave room for counts, suffixes and the shell's prefix.
// Control bytes cannot turn a named branch or repository into a second log line.
fn safe_name(name: &[u8]) -> Box<[u8]> {
    let mut end = name.len().min(88);
    if end < name.len() {
        end = end.saturating_sub(3);
        for _ in 0_u8..3 {
            if let Some(byte) = name.get(end)
                && byte & 0xc0 != 0x80
            {
                break;
            }
            end = end.saturating_sub(1);
        }
    }
    let mut text = bytes::copy_of(name.get(..end).expect("bounded holder name"));
    for byte in &mut text {
        if byte.is_ascii_control() {
            *byte = b' ';
        }
    }
    if end < name.len() { bytes::concat(&[&text, b"..."]) } else { text }
}
fn append(holders: &mut List<Box<[u8]>>, text: &[u8]) {
    if holders.len() < 8 {
        holders.push(safe_name(text)).expect("eight named holders");
    }
}
fn census(used: u32, limit: u32, holders: List<Box<[u8]>>, waiting: u32) -> Box<[u8]> {
    let mut words = bytes::concat(&[
        Decimal::of(u64::from(used)).as_bytes(),
        b" of ",
        Decimal::of(u64::from(limit)).as_bytes(),
        b", held by ",
    ]);
    let mut first = true;
    for holder in &holders {
        let separator = if first { b"".as_slice() } else { b", ".as_slice() };
        words = bytes::concat(&[&words, separator, holder]);
        first = false;
    }
    let more = used.saturating_sub(holders.len());
    if more > 0 {
        words = bytes::concat(&[&words, b", and ", Decimal::of(u64::from(more)).as_bytes(), b" more"]);
    }
    if waiting > 0 {
        words = bytes::concat(&[&words, b"; ", Decimal::of(u64::from(waiting)).as_bytes(), b" waiting for room"]);
    }
    words
}
fn outbox_words(d: &Domain, limit: u32) -> Box<[u8]> {
    let mut holders = List::with_capacity(8);
    for (number, row) in &d.entries {
        append(
            &mut holders,
            &bytes::concat(&[
                b"entry ",
                Decimal::of(*number).as_bytes(),
                b" (task ",
                Decimal::of(row.entry.task).as_bytes(),
                b")",
            ]),
        );
        if holders.len() == 8 {
            break;
        }
    }
    let mut reservations = Map::with_capacity(d.full.reservations.len());
    for (reservation, (order, task)) in &d.full.reservations {
        reservations.insert(*order, (*reservation, *task)).expect("ordered reservations fit");
    }
    for (_, (reservation, task)) in &reservations {
        let number = Decimal::of(*task);
        let text = match reservation {
            Reservation::Asked(_) => bytes::concat(&[b"room asked for task ", number.as_bytes()]),
            Reservation::Call(_) => bytes::concat(&[b"room for task ", number.as_bytes(), b"'s call"]),
            Reservation::Projection(_) => bytes::concat(&[b"room for goal ", number.as_bytes(), b"'s projection"]),
        };
        append(&mut holders, &text);
        if holders.len() == 8 {
            break;
        }
    }
    let mut waiting = 0_u32;
    for (_, row) in &d.changes {
        if row.room.is_some() {
            waiting = waiting.saturating_add(1);
        }
    }
    for (_, row) in &d.landing.rows {
        if row.room.is_some() {
            waiting = waiting.saturating_add(1);
        }
    }
    for (_, row) in &d.releases {
        if row.room.is_some() {
            waiting = waiting.saturating_add(1);
        }
    }
    for (_, row) in &d.makings {
        if row.outbox_wait.is_some() {
            waiting = waiting.saturating_add(1);
        }
    }
    census(d.entries.len().saturating_add(crate::room::reserved(d)), limit, holders, waiting)
}
fn read_words(d: &Domain, limit: u32) -> Box<[u8]> {
    let calls = d.client.holders();
    let used = u32::try_from(calls.len()).expect("bounded calls");
    let mut holders = List::with_capacity(8);
    for holder in calls {
        let read = match holder.call {
            Holding::Read(read) => read_name(read),
            Holding::Write => bytes::copy_of(b"a write"),
        };
        let repository = match d.repositories.get(&holder.repository) {
            Some(row) => bytes::concat(&[&row.owner, b"/", &row.name]),
            None => bytes::copy_of(b"an unadopted repository"),
        };
        let mut text = bytes::concat(&[&read, b" of ", &repository]);
        if let Some(task) = task(d, &holder.owner) {
            text = bytes::concat(&[&text, b" for task ", Decimal::of(task).as_bytes()]);
        }
        append(&mut holders, &text);
        if holders.len() == 8 {
            break;
        }
    }
    census(used, limit, holders, 0)
}
fn task(d: &Domain, owner: &CallOwner) -> Option<u64> {
    match owner {
        CallOwner::Entry(number) => Some(d.entries.get(number)?.entry.task),
        CallOwner::Resource(_) | CallOwner::Repository(_) => None,
        CallOwner::Read(owner) => {
            if let Some(step) = d.steps.get(owner) {
                return Some(step.task);
            }
            if let Some(pending) = d.landing.pending.get(owner) {
                return Some(pending.task);
            }
            if let Some(pending) = d.makings.get(owner) {
                return Some(pending.task);
            }
            if let Some(pending) = d.lost.get(owner) {
                return Some(pending.task);
            }
            if let Some(pending) = d.goal_file_reads.get(owner) {
                return Some(pending.task);
            }
            if let Some(pending) = d.closing.reads.get(owner) {
                return Some(pending.task);
            }
            if let Some((task, _)) = d.closing.selection_reads.get(owner) {
                return Some(*task);
            }
            None
        }
    }
}
fn read_name(read: Read) -> Box<[u8]> {
    match read {
        Read::Branch { branch } => bytes::concat(&[b"branch ", &branch]),
        Read::Pull { number, .. } | Read::Reviews { number, .. } | Read::PullFiles { number, .. } => {
            bytes::concat(&[b"pull ", Decimal::of(number).as_bytes()])
        }
        Read::Item { number, .. } => bytes::concat(&[b"issue ", Decimal::of(number).as_bytes()]),
        Read::JobState { attempt } | Read::Job { attempt, .. } => ci_name(attempt.head),
        Read::Checks { commit } | Read::Statuses { commit, .. } => ci_name(commit),
        Read::File { .. } | Read::Tree { .. } | Read::Artifact { .. } => bytes::copy_of(b"a file"),
        Read::Locate { .. }
        | Read::Items { .. }
        | Read::PullFor { .. }
        | Read::Remarks { .. }
        | Read::Branches { .. }
        | Read::Compare { .. }
        | Read::Protection { .. }
        | Read::Settings
        | Read::Collaborators { .. }
        | Read::Permission { .. }
        | Read::Runs { .. }
        | Read::Run { .. }
        | Read::Commits { .. } => bytes::copy_of(b"the listing"),
    }
}
#[cfg(test)]
#[path = "full/tests.rs"]
mod tests;

fn ci_name(commit: skein_forge::api::Commit) -> Box<[u8]> {
    let mut head = skein_lib::Writer::new(12);
    for byte in commit.get(..6).expect("commit prefix") {
        const HEX: &[u8] = b"0123456789abcdef";
        head.put(&[*HEX.get(usize::from(byte >> 4_u8)).expect("hex"), *HEX.get(usize::from(byte & 15)).expect("hex")])
            .expect("prefix counted");
    }
    bytes::concat(&[b"CI at ", &head.finish()])
}
