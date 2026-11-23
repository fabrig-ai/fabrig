//! Reads waiting for room take turns (temper/docs/design/domain/forge/keeping-up.md, 3).
use super::{Domain, Env, Limits, Request, goal_branch, outage};
use skein_lib::{Decimal, Duration, List, Map, Queue, Time, Token, bytes};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Waiter {
    Making(Token),
    Brief(Token),
    Document(Token),
    Lost(Token),
    Landing(Token),
}
impl Waiter {
    fn owner(self) -> Token {
        match self {
            Self::Making(owner)
            | Self::Brief(owner)
            | Self::Document(owner)
            | Self::Lost(owner)
            | Self::Landing(owner) => owner,
        }
    }
}

#[derive(Clone, Copy, Hash, Debug)]
struct Place {
    order: u64,
    since: Time,
    at: Time,
    delay: Duration,
}

#[derive(Hash, Debug)]
pub(crate) struct Line {
    places: Map<Waiter, Place>,
    sequence: u64,
    // A refusal yields the turn, even if a place remains in the client.
    ready: bool,
    // Only the head's synchronous ask may pass the connector's Busy gate.
    asking: Option<Waiter>,
    backoff: Duration,
    slow: Duration,
}

fn capacity(limits: &Limits) -> u32 {
    limits
        .holds
        .saturating_mul(2)
        .saturating_add(limits.brief_sections.saturating_mul(2))
        .saturating_add(limits.landings)
}

impl Line {
    pub(super) fn new(limits: &Limits) -> Self {
        Self {
            places: Map::with_capacity(capacity(limits)),
            sequence: 0,
            ready: false,
            asking: None,
            backoff: limits.client.backoff,
            slow: limits.client.slow,
        }
    }

    pub(super) fn raise(&mut self, limits: &Limits) {
        self.places.raise(capacity(limits));
    }

    pub(crate) fn leave(&mut self, waiter: Waiter) {
        self.places.remove(&waiter);
        if self.places.is_empty() {
            self.ready = false;
        }
    }

    pub(super) fn clear(&mut self) {
        self.places = Map::with_capacity(self.places.capacity());
        self.ready = false;
    }

    pub(super) fn deadline(&self) -> Option<Time> {
        let mut due = None;
        for (_, place) in &self.places {
            due = skein_lib::earliest(due, Some(place.at));
        }
        due
    }

    pub(super) fn wake(&mut self) {
        self.ready = !self.places.is_empty();
    }

    pub(super) fn waiting(&self) -> bool {
        !self.places.is_empty()
    }

    pub(super) fn take_ask(&mut self, owner: Token) -> Option<Waiter> {
        let waiter = self.asking?;
        if waiter.owner() == owner { self.asking.take() } else { None }
    }

    fn head(&self) -> Option<Waiter> {
        let mut head: Option<(u64, Waiter)> = None;
        for (waiter, place) in &self.places {
            if match head {
                Some((order, _waiter)) => place.order < order,
                None => true,
            } {
                head = Some((place.order, *waiter));
            }
        }
        let (_, waiter) = head?;
        Some(waiter)
    }

    #[cfg(test)]
    pub(crate) fn ready_head(&self) -> Option<Waiter> {
        if self.ready { self.head() } else { None }
    }
}

pub(crate) fn join(d: &mut Domain, waiter: Waiter, now: Time, out: &mut Queue<Request>) {
    let line = &mut d.read_line;
    if d.stopping || line.places.contains_key(&waiter) {
        return;
    }
    let ahead = line.places.len();
    line.sequence = line.sequence.checked_add(1).expect("read places fit a process");
    let delay = line.backoff;
    line.places
        .insert(waiter, Place { order: line.sequence, since: now, at: now.saturating_add(delay), delay })
        .expect("each waiting read has a place in its own table");
    if let Waiter::Making(owner) = waiter {
        making_words(d, owner, Words::Joins(ahead), out);
    }
}

pub(super) fn askable(d: &Domain) -> bool {
    d.read_line.ready && d.read_line.waiting() && d.client.has_room()
}

pub(crate) fn serve(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) -> bool {
    if !askable(d) || out.room() == 0 {
        return false;
    }
    let waiter = d.read_line.head().expect("ready read line has a head");
    d.read_line.ready = false;
    d.read_line.asking = Some(waiter);
    match waiter {
        Waiter::Making(owner) => goal_branch::ask(d, env, owner, out),
        Waiter::Lost(owner) => {
            if !outage::ask_line(d, env, owner, out) {
                super::ask_lost_again(d, env, owner, out);
            }
        }
        Waiter::Landing(owner) => {
            if !outage::ask_line(d, env, owner, out) {
                crate::landing::ask_again(d, env, owner, out);
            }
        }
        Waiter::Brief(owner) => {
            if let Some(event) = crate::brief::ask(d, owner) {
                super::child(d, env, event, out);
            }
        }
        Waiter::Document(owner) => {
            if let Some(event) = crate::brief_document::ask(d, owner) {
                super::child(d, env, event, out);
            }
        }
    }
    if d.read_line.asking.take().is_some() {
        // The underlying gather has ended without another call.
        d.read_line.leave(waiter);
        d.read_line.wake();
    }
    !d.read_line.places.contains_key(&waiter)
}

pub(super) fn attempted(
    d: &mut Domain,
    waiter: Waiter,
    refusal: Option<skein_forge::api::Error>,
    now: Time,
    out: &mut Queue<Request>,
) {
    if refusal == Some(skein_forge::api::Error::Busy) {
        return;
    }
    if let Some(place) = d.read_line.places.remove(&waiter)
        && refusal.is_none()
        && let Waiter::Making(owner) = waiter
    {
        making_words(d, owner, Words::Admitted(now.saturating_since(place.since)), out);
    }
    d.read_line.wake();
}

pub(super) fn fire(d: &mut Domain, now: Time, out: &mut Queue<Request>) {
    let mut due = Map::with_capacity(d.read_line.places.len());
    for (waiter, place) in &d.read_line.places {
        if place.at <= now {
            due.insert(place.order, *waiter).expect("one deadline per place");
        }
    }
    for (order, waiter) in &due {
        if let Waiter::Making(_) = waiter
            && out.room() == 0
        {
            // Leave this report due when this turn has no room for its words.
            break;
        }
        d.read_line.ready = true;
        let place = d.read_line.places.get_mut(waiter).expect("due place");
        place.delay = place.delay.saturating_add(place.delay).min(d.read_line.slow);
        place.at = now.saturating_add(place.delay);
        if let Waiter::Making(owner) = waiter {
            let age = now.saturating_since(place.since);
            let mut ahead = 0_u64;
            for (_, other) in &d.read_line.places {
                if other.order < *order {
                    ahead = ahead.saturating_add(1);
                }
            }
            making_words(d, *owner, Words::Still(age, ahead), out);
        }
    }
}

pub(super) fn prune_makings(d: &mut Domain) {
    if !d.read_line.waiting() {
        return;
    }
    let mut ended = List::with_capacity(d.read_line.places.len());
    for (waiter, _) in &d.read_line.places {
        if let Waiter::Making(owner) = waiter
            && !match d.makings.get(owner) {
                Some(pending) => goal_branch::held(d, pending),
                None => false,
            }
        {
            ended.push(*owner).expect("one read per making");
        }
    }
    for owner in ended.into_boxed() {
        d.makings.remove(&owner);
        d.read_line.leave(Waiter::Making(owner));
    }
}

enum Words {
    Joins(u32),
    Still(Duration, u64),
    Admitted(Duration),
}

fn making_words(d: &Domain, owner: Token, words: Words, out: &mut Queue<Request>) {
    let Some(making) = d.makings.get(&owner) else { return };
    let Some((repository, branch)) = goal_branch::branch_read(d, &d.landing.limits, making) else { return };
    let Some(repository) = d.repositories.get(&repository) else { return };
    let task = Decimal::of(making.task);
    let resource = bytes::concat(&[b": branch ", &branch, b" of ", &repository.owner, b"/", &repository.name]);
    let prefix = [b"task ".as_slice(), task.as_bytes(), b"'s goal branch making "];
    let prefix = bytes::concat(&prefix);
    let waiting = b"waits for the forge's read admission";
    let words = match words {
        Words::Joins(ahead) => {
            bytes::concat(&[&prefix, waiting, &resource, b", ", Decimal::of(u64::from(ahead)).as_bytes(), b" ahead"])
        }
        Words::Still(age, ahead) => bytes::concat(&[
            &prefix,
            waiting,
            b" for ",
            &crate::age_words(age),
            &resource,
            b", ",
            Decimal::of(ahead).as_bytes(),
            b" ahead",
        ]),
        Words::Admitted(age) => {
            bytes::concat(&[&prefix, b"has the forge's read admission after ", &crate::age_words(age), &resource])
        }
    };
    super::emit(out, Request::FullTableFact { words });
}
