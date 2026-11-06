//! Endpoint configuration and admission (llm-connection.md, sections 3, 4
//! and 7). The component keeps configured endpoints, physical bindings and
//! an owner lifecycle and admitted calls in arrival order. It knows no retry
//! or termination policy. `down` admits
//! calls and the owner close; `up` routes io settlement; `fire` advances one
//! binding. Closed is last, after every physical binding has settled.

#![expect(clippy::single_match, clippy::manual_let_else, reason = "explicit routing cases and checked slab insertion")]

use alloc::boxed::Box;

use skein_io::{Event as IoEvent, Request as IoRequest};
use skein_lib::{Env, Id, Lifecycle, List, Queue, Slab, Token, earliest, usize_of};
use skein_llm::{self as llm, Call, Credential, Prompt};
use skein_tls::client as tls;

use crate::boundary::{Body, Event, Refusal, Request};
use crate::call::Connection;
use crate::call::Phase::{Closing, Connecting, Draining, Handshaking, Head, Idle, Ready, Streaming};
use crate::endpoint::{Endpoint, EndpointError, Transport};
use crate::limits::{Limits, check};

/// An admitted conversation waiting for an endpoint binding; terminal releases it.
pub(crate) struct Waiting {
    call: Token,
    endpoint: u32,
    order: u64,
    request: llm::client::MeasuredRequest,
    reservation: u64,
    stage: Stage,
    deadlines: crate::deadlines::Table,
    asked: bool,
}

/// A granted start prepared for an available binding; terminal releases it.
struct Prepared {
    client: llm::client::Client,
    call: Token,
    reservation: u64,
    deadlines: crate::deadlines::Table,
    asked: bool,
}

/// Memory grants are global arrival order; connections are per endpoint.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
    /// Holds its owner's start and no pool reservation.
    Memory,
    /// Holds its reservation while awaiting an endpoint's connection.
    Connection,
}

/// The owner's bounded call pool; Close or Abort ends it with one Closed.
#[expect(missing_debug_implementations, reason = "live calls retain bearer credentials")]
pub struct Component {
    endpoints: List<Endpoint>,
    limits: Limits,
    connections: Slab<Connection>,
    slots: List<Option<Id<Connection>>>,
    cursor: u32,
    /// The owner's call pool's lifecycle, ending with Closed after physical
    /// socket settlement.
    lifecycle: Lifecycle,
    waiting: List<Option<Waiting>>,
    arrival: u64,
    reserved: u64,
    bodies: Option<Queue<Body>>,
}

impl Component {
    /// Keeps later prepared bodies (skein/docs/design/llm-connection.status.md).
    pub fn keep_bodies(&mut self) {
        if self.bodies.is_none() {
            self.bodies = Some(Queue::uncapped());
        }
    }

    /// Takes the next body in preparation order.
    pub fn pop_body(&mut self) -> Option<Body> {
        self.bodies.as_mut()?.pop()
    }

    fn take_body(&mut self, call: Token, endpoint: u32, client: &mut llm::client::Client) {
        if let Some(bytes) = client.take_kept_body() {
            let provider = self.endpoints.get(endpoint).expect("admitted endpoint").llm.provider;
            self.bodies.as_mut().expect("asked for bodies").push(Body { call, provider, bytes });
        }
    }

    /// Validates bounds and stream capacities before the loop starts.
    pub fn new(mut endpoints: List<Endpoint>, limits: &Limits) -> Result<Component, EndpointError> {
        check(limits, &endpoints)?;
        for index in 0..endpoints.len() {
            let destination = endpoints.get_mut(index).expect("configured endpoint");
            destination.limits.http.request =
                match llm::client::request_head(&destination.llm, &destination.credential, &destination.limits) {
                    Ok(head) => head,
                    Err(error) => return Err(EndpointError::Client(error)),
                };
        }
        let mut waiting = List::with_capacity(limits.calls);
        for _ in 0..limits.calls {
            match waiting.push(None) {
                Ok(()) => {}
                Err(_) => unreachable!("one record per conversation"),
            }
        }
        Ok(Component {
            endpoints,
            limits: *limits,
            connections: Slab::with_capacity(limits.connections),
            slots: List::with_capacity(limits.connections),
            cursor: 0,
            lifecycle: Lifecycle::Live,
            waiting,
            arrival: 0,
            reserved: 0,
            bodies: None,
        })
    }

    /// Check and prepare a standalone call; down measures starts and prepares on running.
    pub fn admit(
        &mut self,
        call: Token,
        endpoint: u32,
        prompt: Prompt,
        credential: Option<Credential>,
    ) -> Result<llm::client::Client, Refusal> {
        self.admit_policy(call, endpoint, prompt, credential, None)
    }

    pub(crate) fn admit_policy(
        &mut self,
        call: Token,
        endpoint: u32,
        prompt: Prompt,
        credential: Option<Credential>,
        drop_reasoning: Option<bool>,
    ) -> Result<llm::client::Client, Refusal> {
        let request = self.measure_start(call, endpoint, prompt, credential, drop_reasoning)?;
        match llm::client::Client::prepare_measured(request) {
            Ok(mut client) => {
                self.take_body(call, endpoint, &mut client);
                Ok(client)
            }
            Err(error) => Err(Refusal::Client(error)),
        }
    }

    fn measure_start(
        &self,
        call: Token,
        endpoint: u32,
        prompt: Prompt,
        credential: Option<Credential>,
        drop_reasoning: Option<bool>,
    ) -> Result<llm::client::MeasuredRequest, Refusal> {
        match self.lifecycle {
            Lifecycle::Live => {}
            Lifecycle::Closing | Lifecycle::Aborting => return Err(Refusal::Closed),
            Lifecycle::Closed => unreachable!("an owner must not request work after Closed"),
        }
        let destination = self.endpoints.get(endpoint).ok_or(Refusal::Endpoint)?;
        if self.outstanding() >= self.limits.calls {
            return Err(Refusal::Calls { bound: self.limits.calls });
        }
        if let Some(credential) = credential.as_ref() {
            match llm::client::check_credential(credential, &destination.credential) {
                Ok(()) => {}
                Err(error) => return Err(Refusal::Client(error)),
            }
        }
        let enabled = match drop_reasoning {
            Some(enabled) => enabled,
            None => destination.limits.drop_reasoning,
        };
        let input = Call { owner: call, prompt, credential, endpoint: destination.llm.clone() };
        let mut request = match llm::client::measure_request(input, &destination.limits, enabled) {
            Ok(request) => request,
            Err(error) => return Err(Refusal::Client(error)),
        };
        let reservation =
            llm::client::reservation(request.measured(), &destination.limits).expect("admitted largest reservation");
        if reservation > self.limits.memory {
            return Err(Refusal::Memory { reservation, bound: self.limits.memory });
        }
        if self.bodies.is_some() {
            request.keep_body();
        }
        Ok(request)
    }

    /// Bytes currently granted to waiting and running calls.
    #[must_use]
    pub const fn reserved(&self) -> u64 {
        self.reserved
    }

    /// Release each running terminal once, then grant waiting memory in arrival order.
    fn refresh_memory(&mut self) {
        for slot in &self.slots {
            match slot {
                Some(id) => match self.connections.get_mut(*id) {
                    Some(connection) if connection.call().is_none() => {
                        self.reserved =
                            self.reserved.checked_sub(connection.reservation).expect("running reservation held");
                        connection.reservation = 0;
                    }
                    Some(_) | None => {}
                },
                None => {}
            }
        }
        match self.lifecycle {
            Lifecycle::Aborting | Lifecycle::Closed => {
                self.refresh_waiting();
                return;
            }
            Lifecycle::Live | Lifecycle::Closing => {}
        }
        for _ in 0..self.waiting.len() {
            let mut oldest = u64::MAX;
            let mut selected = None;
            for index in 0..self.waiting.len() {
                match self.waiting.get(index).expect("waiting index") {
                    Some(waiting) => match waiting.stage {
                        Stage::Memory => {
                            if waiting.order < oldest {
                                oldest = waiting.order;
                                selected = Some(index);
                            }
                        }
                        Stage::Connection => {}
                    },
                    None => {}
                }
            }
            let Some(index) = selected else {
                break;
            };
            let waiting = self.waiting.get_mut(index).expect("waiting index").as_mut().expect("oldest memory wait");
            let remaining = self.limits.memory.checked_sub(self.reserved).expect("grants fit the pool");
            if waiting.reservation > remaining {
                break;
            }
            self.reserved = self.reserved.checked_add(waiting.reservation).expect("grant fits memory");
            waiting.stage = Stage::Connection;
        }
        self.refresh_waiting();
    }

    fn release_waiting(&mut self, waiting: &Waiting) {
        match waiting.stage {
            Stage::Memory => {}
            Stage::Connection => {
                self.reserved = self.reserved.checked_sub(waiting.reservation).expect("waiting grant held");
            }
        }
    }

    fn prepare_waiting(&mut self, waiting: Waiting) -> Prepared {
        let measured = waiting.request.measured();
        let mut prepared =
            llm::client::Client::prepare_measured(waiting.request).expect("measurement made every preparation refusal");
        assert_eq!(prepared.measured(), measured, "admitted lengths encode identically");
        self.take_body(waiting.call, waiting.endpoint, &mut prepared);
        Prepared {
            client: prepared,
            call: waiting.call,
            reservation: waiting.reservation,
            deadlines: waiting.deadlines,
            asked: waiting.asked,
        }
    }

    /// Takes an owner's call or demand.
    pub fn down(&mut self, env: &Env<Limits>, request: Request, up: &mut Queue<Event>, io: &mut Queue<IoRequest>) {
        self.down_event(env, request, up, io);
        self.refresh_memory();
    }

    fn down_event(&mut self, env: &Env<Limits>, request: Request, up: &mut Queue<Event>, io: &mut Queue<IoRequest>) {
        assert!(self.lifecycle != Lifecycle::Closed, "an owner must not send requests after Closed");
        match request {
            Request::Close => {
                match self.lifecycle {
                    Lifecycle::Live => self.lifecycle = Lifecycle::Closing,
                    Lifecycle::Closing | Lifecycle::Aborting => return,
                    Lifecycle::Closed => unreachable!("checked at the entrance"),
                }
                self.request_close(false);
                self.fire(env, up, io);
            }
            Request::Abort => {
                match self.lifecycle {
                    Lifecycle::Live | Lifecycle::Closing => {}
                    Lifecycle::Aborting => return,
                    Lifecycle::Closed => unreachable!("checked at the entrance"),
                }
                self.lifecycle = Lifecycle::Aborting;
                self.request_close(true);
                self.fire(env, up, io);
            }
            Request::Start { call, endpoint, prompt, credential, deadlines, drop_reasoning } => {
                self.start(env, call, endpoint, prompt, credential, deadlines, drop_reasoning, up, io);
            }
            Request::Next { call } => {
                for index in 0..self.waiting.len() {
                    match self.waiting.get_mut(index).expect("waiting index") {
                        Some(waiting) if waiting.call == call => {
                            waiting.asked = true;
                            return;
                        }
                        Some(_) | None => {}
                    }
                }
                for slot in &self.slots {
                    match slot {
                        Some(id) => match self.connections.get_mut(*id) {
                            Some(connection) if connection.call() == Some(call) => {
                                connection.next(env, id.token(), up, io);
                                break;
                            }
                            Some(_) | None => {}
                        },
                        None => {}
                    }
                }
            }
            Request::Cancel { call } => {
                for index in 0..self.waiting.len() {
                    match self.waiting.get(index).expect("waiting index") {
                        Some(waiting) if waiting.call == call => {
                            let waiting = self
                                .waiting
                                .get_mut(index)
                                .expect("waiting index")
                                .take()
                                .expect("cancelled waiting call");
                            self.release_waiting(&waiting);
                            up.push(Event::Cancelled { call, measures: waiting.deadlines.measures() });
                            self.refresh_waiting();
                            return;
                        }
                        Some(_) | None => {}
                    }
                }
                for slot in &self.slots {
                    match slot {
                        Some(id) => match self.connections.get_mut(*id) {
                            Some(connection) if connection.call() == Some(call) => {
                                connection.cancel(env, id.token(), up, io);
                                break;
                            }
                            Some(_) | None => {}
                        },
                        None => {}
                    }
                }
            }
        }
    }

    /// Routes an io answer by the component token echoed as its owner.
    pub fn up(&mut self, env: &Env<Limits>, event: IoEvent, up: &mut Queue<Event>, io: &mut Queue<IoRequest>) {
        self.up_event(env, event, up, io);
        self.refresh_memory();
    }

    fn up_event(&mut self, env: &Env<Limits>, event: IoEvent, up: &mut Queue<Event>, io: &mut Queue<IoRequest>) {
        let owner = match &event {
            IoEvent::Connecting { owner, .. }
            | IoEvent::Connected { owner }
            | IoEvent::Stream { owner, .. }
            | IoEvent::Failed { owner, .. }
            | IoEvent::Closed { owner }
            | IoEvent::Listening { owner, .. }
            | IoEvent::Accepted { owner, .. }
            | IoEvent::Output { owner, .. }
            | IoEvent::Spawned { owner, .. }
            | IoEvent::Exited { owner, .. } => *owner,
            IoEvent::Usage { .. } | IoEvent::Resized | IoEvent::Window { .. } | IoEvent::Shutdown { .. } => return,
        };
        let id = Id::<Connection>::from_token(owner);
        match event {
            IoEvent::Connecting { socket, .. } => match self.connections.get_mut(id) {
                Some(connection) => {
                    connection.link.connecting(socket, io);
                    connection.route(env, owner, up, io);
                }
                None => {}
            },
            IoEvent::Connected { .. } => match self.connections.get_mut(id) {
                Some(connection) => match connection.phase {
                    Connecting { .. } => connection.connected(env, owner, up, io),
                    Handshaking { .. } | Head { .. } | Streaming { .. } | Draining | Idle | Ready | Closing { .. } => {}
                },
                None => {}
            },
            IoEvent::Stream { up: stream, .. } => match self.connections.get_mut(id) {
                Some(connection) => connection.cipher_up(env, owner, stream, up, io),
                None => {}
            },
            IoEvent::Failed { .. } => match self.connections.get_mut(id) {
                Some(connection) => connection.failed(env, owner, up, io),
                None => {}
            },
            IoEvent::Closed { .. } => {
                match self.connections.get_mut(id) {
                    Some(connection) => connection.closed(env, owner, up, io),
                    None => return,
                }
                self.refresh_memory();
                for index in 0..self.slots.len() {
                    let slot = self.slots.get_mut(index).expect("the index is within the slot table");
                    match slot {
                        Some(candidate) if *candidate == id => *slot = None,
                        Some(_) | None => {}
                    }
                }
                self.connections.retire(id);
                self.settled(up);
            }
            IoEvent::Listening { .. }
            | IoEvent::Accepted { .. }
            | IoEvent::Output { .. }
            | IoEvent::Spawned { .. }
            | IoEvent::Exited { .. }
            | IoEvent::Usage { .. }
            | IoEvent::Resized
            | IoEvent::Window { .. }
            | IoEvent::Shutdown { .. } => {}
        }
    }

    /// Runs buffered child work and closes idle bindings after their keep time.
    pub fn fire(&mut self, env: &Env<Limits>, up: &mut Queue<Event>, io: &mut Queue<IoRequest>) {
        self.refresh_memory();
        self.fire_event(env, up, io);
        self.refresh_memory();
    }

    fn fire_event(&mut self, env: &Env<Limits>, up: &mut Queue<Event>, io: &mut Queue<IoRequest>) {
        for index in 0..self.waiting.len() {
            match self.waiting.get(index).expect("waiting index") {
                Some(waiting) if self.lifecycle == Lifecycle::Aborting || waiting.deadlines.due(env.now).is_some() => {
                    let waiting = self.waiting.get_mut(index).expect("waiting index").take().expect("waiting call");
                    self.release_waiting(&waiting);
                    if self.lifecycle == Lifecycle::Aborting {
                        up.push(Event::Cancelled { call: waiting.call, measures: waiting.deadlines.measures() });
                    } else {
                        let bound =
                            self.endpoints.get(waiting.endpoint).expect("admitted endpoint").limits.detail_bytes;
                        let detail = b"timed out waiting for the whole call";
                        let take = detail.len().min(usize_of(bound));
                        up.push(Event::Failed {
                            call: waiting.call,
                            measures: waiting.deadlines.measures(),
                            failure: llm::Failure::TimedOut { phase: llm::Phase::Whole },
                            evidence: llm::client::Evidence::Unsent,
                            report: llm::Report::none(),
                            detail: Box::from(detail.get(..take).expect("bounded detail")),
                        });
                    }
                    self.refresh_waiting();
                    self.settled(up);
                    return;
                }
                Some(_) | None => {}
            }
        }
        if self.serve_waiting(env, up, io) {
            return;
        }
        for _ in 0..self.slots.len() {
            let index = self.cursor;
            self.cursor = match self.cursor.checked_add(1) {
                Some(next) if next < self.slots.len() => next,
                Some(_) | None => 0,
            };
            match self.slots.get(index) {
                Some(Some(id)) => match self.connections.get_mut(*id) {
                    Some(connection) => {
                        let close_due = connection.link.due(env.now);
                        if !connection.has_work() && connection.deadlines.due(env.now).is_none() && !close_due {
                            continue;
                        }
                        connection.link.fire(env.now, io);
                        if !connection.aborting() {
                            match connection.deadlines.due(env.now) {
                                Some(due) => connection.timeout(env, due, io),
                                None => {}
                            }
                        }
                        connection.route(env, id.token(), up, io);
                        break;
                    }
                    None => {}
                },
                Some(None) | None => {}
            }
        }
        self.settled(up);
    }

    /// Buffered routing work that the owner should schedule before sleeping.
    #[must_use]
    pub fn has_work(&self) -> bool {
        if self.runnable_waiting().is_some() || (self.lifecycle == Lifecycle::Aborting && self.waiting_count() > 0) {
            return true;
        }
        for slot in &self.slots {
            match slot {
                Some(id) => match self.connections.get(*id) {
                    Some(connection) if connection.has_work() => return true,
                    Some(_) | None => {}
                },
                None => {}
            }
        }
        false
    }

    /// The first call, idle or transport close deadline for the owning loop.
    #[must_use]
    pub fn next_deadline(&self) -> Option<skein_lib::Time> {
        let mut next: Option<skein_lib::Time> = None;
        for waiting in &self.waiting {
            match waiting {
                Some(waiting) => next = earliest(next, waiting.deadlines.next()),
                None => {}
            }
        }
        for slot in &self.slots {
            match slot {
                Some(id) => match self.connections.get(*id) {
                    Some(connection) => {
                        next = earliest(next, connection.deadlines.next());
                        next = earliest(next, connection.link.next_deadline());
                    }
                    None => {}
                },
                None => {}
            }
        }
        next
    }

    fn request_close(&mut self, abort: bool) {
        for slot in &self.slots {
            match slot {
                Some(id) => match self.connections.get_mut(*id) {
                    Some(connection) => {
                        connection.request_close(abort);
                    }
                    None => {}
                },
                None => {}
            }
        }
    }

    fn settled(&mut self, up: &mut Queue<Event>) {
        match self.lifecycle {
            Lifecycle::Live | Lifecycle::Closed => return,
            Lifecycle::Closing | Lifecycle::Aborting => {}
        }
        if self.waiting_count() > 0 {
            return;
        }
        for slot in &self.slots {
            if slot.is_some() {
                return;
            }
        }
        self.lifecycle = Lifecycle::Closed;
        up.push(Event::Closed);
    }

    /// Releases settled connection slots at the owning loop's reclaim point.
    pub fn reclaim(&mut self) {
        self.connections.reclaim();
    }

    #[expect(clippy::too_many_arguments, reason = "an admitted Start and both explicit output queues")]
    fn start(
        &mut self,
        env: &Env<Limits>,
        call: Token,
        endpoint: u32,
        prompt: Prompt,
        credential: Option<Credential>,
        deadlines: crate::Deadlines,
        drop_reasoning: bool,
        up: &mut Queue<Event>,
        io: &mut Queue<IoRequest>,
    ) {
        let request = match self.measure_start(call, endpoint, prompt, credential, Some(drop_reasoning)) {
            Ok(request) => request,
            Err(why) => {
                up.push(Event::Refused { call, why });
                return;
            }
        };
        let destination = self.endpoints.get(endpoint).expect("admitted endpoint");
        let reservation =
            llm::client::reservation(request.measured(), &destination.limits).expect("measured largest call fits");
        let mut table = crate::deadlines::Table::new(deadlines, env.now);
        table.arm(crate::deadlines::Phase::Waiting, env.now, self.limits.idle_keep);
        let waiting = Waiting {
            call,
            endpoint,
            order: self.arrival,
            request,
            reservation,
            stage: Stage::Memory,
            deadlines: table,
            asked: false,
        };
        self.arrival = self.arrival.checked_add(1).expect("arrival counter fits the process lifetime");
        for index in 0..self.waiting.len() {
            if self.waiting.get(index).expect("waiting index").is_none() {
                *self.waiting.get_mut(index).expect("waiting index") = Some(waiting);
                self.refresh_memory();
                self.serve_waiting(env, up, io);
                return;
            }
        }
        unreachable!("admission reserves a waiting record");
    }

    fn waiting_count(&self) -> u32 {
        let mut count = 0_u32;
        for waiting in &self.waiting {
            if waiting.is_some() {
                count = count.checked_add(1).expect("bounded conversations");
            }
        }
        count
    }

    fn outstanding(&self) -> u32 {
        let mut count = self.waiting_count();
        for slot in &self.slots {
            match slot {
                Some(id) => match self.connections.get(*id) {
                    Some(connection) if connection.call().is_some() => {
                        count = count.checked_add(1).expect("bounded conversations");
                    }
                    Some(_) | None => {}
                },
                None => {}
            }
        }
        count
    }

    fn refresh_waiting(&mut self) {
        for slot in &self.slots {
            match slot {
                Some(id) => match self.connections.get_mut(*id) {
                    Some(connection) => {
                        connection.waiting = false;
                        for waiting in &self.waiting {
                            match waiting {
                                Some(waiting) if waiting.endpoint == connection.endpoint => match waiting.stage {
                                    Stage::Memory => {}
                                    Stage::Connection => connection.waiting = true,
                                },
                                Some(_) | None => {}
                            }
                        }
                    }
                    None => {}
                },
                None => {}
            }
        }
    }

    // Arrival order is independent of which record was vacated by cancellation.
    fn runnable_waiting(&self) -> Option<u32> {
        if self.lifecycle == Lifecycle::Aborting || self.lifecycle == Lifecycle::Closed {
            return None;
        }
        let mut selected: Option<u32> = None;
        let mut oldest = u64::MAX;
        for index in 0..self.waiting.len() {
            match self.waiting.get(index).expect("waiting index") {
                Some(waiting) => {
                    match waiting.stage {
                        Stage::Memory => continue,
                        Stage::Connection => {}
                    }

                    let mut count = 0_u32;
                    let mut reusable = false;
                    let mut evictable = false;
                    for slot in &self.slots {
                        match slot {
                            Some(id) => match self.connections.get(*id) {
                                Some(connection) => {
                                    if connection.endpoint == waiting.endpoint {
                                        count = count.checked_add(1).expect("bounded connections");
                                    }
                                    if connection.reusable() && connection.endpoint == waiting.endpoint {
                                        reusable = true;
                                    } else if connection.idle() {
                                        evictable = true;
                                    }
                                }
                                None => {}
                            },
                            None => {}
                        }
                    }
                    if waiting.order < oldest
                        && (reusable
                            || (count < self.limits.per_endpoint && (!self.connections.is_full() || evictable)))
                    {
                        selected = Some(index);
                        oldest = waiting.order;
                    }
                }
                None => {}
            }
        }
        selected
    }

    fn reusable_connection(&self, endpoint: u32) -> Option<Id<Connection>> {
        for slot in &self.slots {
            match slot {
                Some(id) => match self.connections.get(*id) {
                    Some(connection) if connection.endpoint == endpoint && connection.reusable() => return Some(*id),
                    Some(_) | None => {}
                },
                None => {}
            }
        }
        None
    }

    fn serve_waiting(&mut self, env: &Env<Limits>, up: &mut Queue<Event>, io: &mut Queue<IoRequest>) -> bool {
        let index = match self.runnable_waiting() {
            Some(index) => index,
            None => return false,
        };
        let waiting = self.waiting.get(index).expect("waiting index").as_ref().expect("waiting call");
        if waiting.deadlines.due(env.now).is_some() {
            return false; // fire answers the expired wait before preparing it.
        }
        let endpoint = waiting.endpoint;
        let reusable = self.reusable_connection(endpoint);
        let prepared = if reusable.is_some() || !self.connections.is_full() {
            let waiting = self.waiting.get_mut(index).expect("waiting index").take().expect("waiting call");
            Some(self.prepare_waiting(waiting))
        } else {
            None
        };
        let mut prepared = prepared;
        if let Some(id) = reusable {
            let waiting = prepared.take().expect("reusable client prepared");
            let connection = self.connections.get_mut(id).expect("selected reusable binding");
            match connection.llm.next_call(waiting.client) {
                Ok(()) => {}
                Err(_) => unreachable!("a reusable client accepts the prepared call"),
            }
            connection.activity = 0;
            connection.reservation = waiting.reservation;
            connection.phase = Head { call: waiting.call };
            connection.deadlines = waiting.deadlines;
            connection.sync_deadlines(env);
            if waiting.asked {
                connection.next(env, id.token(), up, io);
            }
            connection.start_call(env, id.token(), up, io);
            self.refresh_waiting();
            return true;
        }
        if self.connections.is_full() {
            for slot in &self.slots {
                match slot {
                    Some(id) => match self.connections.get_mut(*id) {
                        Some(connection) if connection.idle() && connection.endpoint != endpoint => {
                            connection.request_close(false);
                            connection.route(env, id.token(), up, io);
                            return true;
                        }
                        Some(_) | None => {}
                    },
                    None => {}
                }
            }
            return false;
        }
        let waiting = prepared.take().expect("new client prepared");
        let destination = self.endpoints.get(endpoint).expect("admitted endpoint");
        let tls = match &destination.transport {
            Transport::Tls { server_name, trust } => {
                Some(tls::Client::new(trust, server_name.clone(), &self.limits.tls))
            }
            Transport::Plaintext => None,
        };
        let mut connection = Connection::new(
            endpoint,
            waiting.call,
            tls,
            waiting.client,
            destination.limits,
            crate::Deadlines::none(),
            env.now,
        );
        connection.reservation = waiting.reservation;
        connection.deadlines = waiting.deadlines;
        connection.sync_deadlines(env);
        if self.lifecycle == Lifecycle::Closing {
            connection.request_close(false);
        }
        let id = match self.connections.insert(connection) {
            Ok(id) => id,
            Err(_) => unreachable!("checked connection capacity"),
        };
        let mut stored = false;
        for index in 0..self.slots.len() {
            let slot = self.slots.get_mut(index).expect("connection index");
            if slot.is_none() {
                *slot = Some(id);
                stored = true;
                break;
            }
        }
        if !stored {
            self.slots.push(Some(id)).expect("one slot per connection");
        }
        io.push(IoRequest::Connect { owner: id.token(), addr: skein_io::Addr::Ip(destination.address) });
        if waiting.asked {
            self.connections.get_mut(id).expect("new connection").next(env, id.token(), up, io);
        }
        self.refresh_waiting();
        true
    }
}

#[cfg(test)]
mod link_tests {
    use super::*;
    use crate::tests::{component, credential, limits, prompt};
    use skein_lib::{Time, Wall};

    #[test]
    fn a_queued_tls_close_stays_runnable_until_the_socket_close() {
        let mut component = component();
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
        let mut up = Queue::uncapped();
        let mut io = Queue::uncapped();
        component.down(
            &env,
            Request::Start {
                call: Token::new(7),
                endpoint: 0,
                prompt: prompt(),
                credential: Some(credential()),
                deadlines: crate::Deadlines::none(),
                drop_reasoning: false,
            },
            &mut up,
            &mut io,
        );
        let owner = match io.pop() {
            Some(IoRequest::Connect { owner, .. }) => owner,
            other => panic!("expected connect, got {other:?}"),
        };
        let connection = component.connections.get_mut(Id::from_token(owner)).expect("physical binding");
        connection.link.connecting(Token::new(100), &mut io);
        connection.link.close(&crate::call::link_env(&env), false, &mut io);
        // The routing budget can leave TLS's Closed queued between entrances.
        connection.phase = Closing { call: None };
        connection.sync_deadlines(&env);
        assert!(component.has_work(), "queued link work keeps a closing connection runnable");
        component.fire(&env, &mut up, &mut io);
        assert_eq!(io.pop(), Some(IoRequest::Close { entity: Token::new(100) }));
        assert!(!component.has_work(), "only io settlement remains");
    }
}

#[cfg(test)]
mod memory_tests {
    use super::*;
    use crate::tests::{component, credential, endpoint, limits, prompt};

    #[test]
    fn an_entrance_larger_than_the_pool_is_refused_before_preparing() {
        // Construction normally excludes this case; lower the private pool to
        // exercise the entrance guard independently of startup validation.
        let mut component = component();
        let destination = component.endpoints.get(0).unwrap();
        let input = prompt();
        let secret = credential();
        let measured =
            skein_llm::client::measure(&input, Some(&secret), &destination.llm, &destination.limits).unwrap();
        let reservation = skein_llm::client::reservation(measured, &destination.limits).unwrap();
        component.limits.memory = reservation - 1;
        assert_eq!(
            component.admit(Token::new(7), 0, input, Some(secret)).err(),
            Some(Refusal::Memory { reservation, bound: reservation - 1 })
        );
        assert_eq!(component.reserved(), 0);
    }

    #[test]
    fn a_younger_fitting_start_stays_behind_an_older_memory_wait() {
        let destination = endpoint();
        let mut bounds = destination.limits;
        bounds.http.request = llm::client::request_head(&destination.llm, &destination.credential, &bounds).unwrap();
        let measured = llm::client::measure(&prompt(), Some(&credential()), &destination.llm, &bounds).unwrap();
        let reservation = llm::client::reservation(measured, &bounds).unwrap();
        let mut config = limits();
        config.calls = 3;
        config.connections = 3;
        config.per_endpoint = 3;
        config.io.sockets = 3;
        config.memory = 2 * reservation;
        let mut endpoints = List::with_capacity(1);
        endpoints.push(destination).unwrap();
        let mut pool = Component::new(endpoints, &config).unwrap();
        let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: config };
        let mut up = Queue::uncapped();
        let mut io = Queue::uncapped();
        for token in [7, 8, 9] {
            let mut input = prompt();
            if token == 8 {
                input.instructions = Box::new([b'x'; 1024]);
            }
            pool.down(
                &env,
                Request::Start {
                    call: Token::new(token),
                    endpoint: 0,
                    prompt: input,
                    credential: Some(credential()),
                    deadlines: crate::Deadlines::none(),
                    drop_reasoning: false,
                },
                &mut up,
                &mut io,
            );
        }
        assert_eq!(pool.reserved(), reservation);
        assert_eq!(io.len(), 1, "only the first call prepares and connects");
        assert_eq!(pool.waiting_count(), 2);
        pool.down(&env, Request::Cancel { call: Token::new(8) }, &mut up, &mut io);
        assert_eq!(pool.reserved(), 2 * reservation, "removing the older wait grants the younger call");
        assert_eq!(pool.waiting_count(), 1);
    }
}
