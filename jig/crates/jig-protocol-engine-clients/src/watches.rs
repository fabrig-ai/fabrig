//! Watch exchanges and released stream outputs (protocol/client/wire.md, 2, and protocol/client/README.md, 5).
use super::{Box, Component, Env, Limits, Queue, Request, Token, Waiting, core, routes, w};
use crate::{
    routes::Route,
    stream::{self, Stream, Taken, Watch},
    translate::watches as translate,
};
use skein_lib::Reader;
impl Component {
    fn issued_name(&self, name: Token) -> bool {
        name.raw() != 0 && name.raw() < self.next
    }
    fn stream_owner(&self, name: &[u8; 16], sign_in: u64) -> Option<Token> {
        for (owner, stream) in &self.streams {
            if stream.name == *name && stream.sign_in == sign_in && !stream.closing() {
                return Some(*owner);
            }
        }
        None
    }
    fn watcher_owner(&self, watcher: Token) -> Option<Token> {
        for (owner, stream) in &self.streams {
            if stream.watches.contains_key(&watcher) {
                return Some(*owner);
            }
        }
        None
    }
    pub(super) fn watch_route(
        &mut self,
        env: &Env<Limits>,
        exchange: Token,
        sign_in: u64,
        route: Route,
        body: &[u8],
        out: &mut Queue<Request>,
    ) {
        let Some(owner) = self.owner(exchange) else {
            return;
        };
        let mut reader = Reader::new(body);
        match route {
            Route::Stream if body.is_empty() => {
                self.stream_open(env, owner, sign_in, out);
            }
            Route::Watch => match w::Open::decode(&env.limits.wire, &mut reader) {
                Ok(value) if reader.is_empty() => self.open_watch(env, Some(exchange), sign_in, value, out),
                Ok(_) | Err(_) => {
                    self.connections.get_mut(&owner).expect("exchange owner").respond(crate::Reply::NotARecord);
                }
            },
            Route::Unwatch => {
                let Ok(value) = w::WatchClose::decode(&env.limits.wire, &mut reader) else {
                    self.connections.get_mut(&owner).expect("exchange owner").respond(crate::Reply::NotARecord);
                    return;
                };
                if !reader.is_empty() {
                    self.connections.get_mut(&owner).expect("exchange owner").respond(crate::Reply::NotARecord);
                    return;
                }
                let Some(owner) = self.stream_owner(value.stream(), sign_in) else {
                    self.respond(exchange, routes::close_answer(false));
                    return;
                };
                let stream = self.streams.get_mut(&owner).expect("selected stream");
                if value.watch() == 0 && stream.me.is_some() {
                    self.respond(exchange, routes::close_answer(false));
                    return;
                }
                let mut found = None;
                for (watcher, watch) in &stream.watches {
                    if watch.name == value.watch() {
                        found = Some(*watcher);
                        break;
                    }
                }
                if let Some(watcher) = found {
                    let watch = stream.watches.get_mut(&watcher).expect("named watch");
                    if watch.closing.is_some() {
                        self.connections
                            .get_mut(&self.owner(exchange).expect("exchange owner"))
                            .expect("connection")
                            .respond(crate::Reply::Conflict);
                        return;
                    }
                    if watch.end.is_some() {
                        self.respond(exchange, routes::close_answer(false));
                        return;
                    }
                    watch.closing = Some(exchange);
                    if !watch.unwatch {
                        watch.unwatch = true;
                        out.push(Request::Core(Box::new(core::Event::Unwatch { watcher })));
                    }
                    self.pending
                        .exchanges
                        .insert(exchange, Waiting::Watch { sign_in, watcher, opening: false })
                        .expect("admitted close");
                } else {
                    self.respond(exchange, routes::close_answer(false));
                }
            }
            Route::Gone => match w::Gone::decode(&env.limits.wire, &mut reader) {
                Ok(value) if reader.is_empty() => {
                    if let Some(owner) = self.stream_owner(value.stream(), sign_in) {
                        self.streams.get_mut(&owner).expect("selected stream").phase = stream::Phase::Closing;
                    }
                    self.respond(exchange, Box::new([]));
                }
                Ok(_) | Err(_) => {
                    self.connections.get_mut(&owner).expect("exchange owner").respond(crate::Reply::NotARecord);
                }
            },
            Route::ApprovalAsk
            | Route::LinkAsk
            | Route::ApprovalRead
            | Route::ApprovalDecide
            | Route::ApprovalTake
            | Route::Stream
            | Route::Link
            | Route::Service
            | Route::Authorize
            | Route::SignOut
            | Route::Read
            | Route::Request => {
                self.connections.get_mut(&owner).expect("exchange owner").respond(crate::Reply::NotARecord);
            }
        }
    }
    fn stream_open(&mut self, env: &Env<Limits>, owner: Token, sign_in: u64, out: &mut Queue<Request>) {
        if self.streams.len() == self.streams.capacity() {
            self.connections
                .get_mut(&owner)
                .expect("stream owner")
                .respond(crate::Reply::StreamsFull { size: env.limits.streams });
            return;
        }
        let mut name = [0; 16];
        self.rng.fill(&mut name);
        let hello = translate::hello(name, &self.settings, &env.limits.wire);
        let Ok(hello) = hello else {
            self.connections.get_mut(&owner).expect("stream owner").respond(crate::Reply::NotARecord);
            return;
        };
        let mut stream = Stream::new(name, sign_in, env);
        let version = self.connections.get(&owner).expect("stream owner").version;
        let served = match version {
            Some(version) => self.serves(None, version),
            None => false,
        };
        if served {
            stream.hello = Some(hello);
        } else {
            stream.write(stream::packet(w::StreamEvent::Hello(hello), Taken::None));
        }
        self.streams.insert(owner, stream).expect("admitted stream");
        let connection = self.connections.get_mut(&owner).expect("stream owner");
        connection.stream = true;
        if served {
            let subject = w::Subject::Me;
            let mut record = skein_lib::Writer::new(skein_lib::length::usize_of(subject.measure()));
            subject.encode(&mut record).expect("measured subject");
            let subject = w::Owned::new(
                &env.limits.wire,
                w::OwnedParts {
                    owner: w::Owner::Jig,
                    version: version.expect("served opening"),
                    record: record.finish(),
                },
            )
            .expect("me subject");
            let open =
                w::Open::new(&env.limits.wire, w::OpenParts { stream: name, watch: 0, subject }).expect("stream watch");
            self.open_watch(env, None, sign_in, open, out);
        }
    }
    fn open_watch(
        &mut self,
        env: &Env<Limits>,
        exchange: Option<Token>,
        sign_in: u64,
        open: w::Open,
        out: &mut Queue<Request>,
    ) {
        let open = open.into_parts();
        let Some(owner) = self.stream_owner(&open.stream, sign_in) else {
            if let Some(exchange) = exchange {
                self.respond(exchange, routes::open_answer(Some(core::WatchRefusal::Unknown), &env.limits.wire));
            }
            return;
        };
        let stream = self.streams.get(&owner).expect("named stream");
        let mut duplicate = false;
        for (_, watch) in &stream.watches {
            if watch.name == open.watch {
                duplicate = true;
            }
        }
        if duplicate
            || stream.watches.len() == stream.watches.capacity()
            || self.admitted() >= env.limits.exchanges
            || self.active(sign_in) >= env.limits.exchanges_per_client
        {
            if let Some(exchange) = exchange {
                self.respond(exchange, routes::open_answer(Some(core::WatchRefusal::Busy), &env.limits.wire));
            } else {
                let stream = self.streams.get_mut(&owner).expect("stream watch");
                stream.phase = stream::Phase::Closing;
                stream.ending = crate::Ended::Busy;
            }
            return;
        }
        let subject = open.subject.into_parts();
        let family = match &subject.owner {
            w::Owner::Jig => None,
            w::Owner::Family(value) => Some(value.number()),
        };
        if !self.serves(family, subject.version) {
            let answer =
                w::OpenAnswer::NotServed(crate::served::not_served(&self.settings, subject.owner, &env.limits.wire));
            if let Some(exchange) = exchange {
                self.respond(exchange, routes::encoded_open(&answer));
            }
            return;
        }
        let watcher = self.name();
        let version = subject.version;
        let Some(opening) =
            crate::served::watch(subject, watcher, sign_in, self.settings.rows, &env.limits.wire, &self.settings)
        else {
            if let Some(exchange) = exchange {
                self.respond(exchange, routes::open_answer(Some(core::WatchRefusal::Unknown), &env.limits.wire));
            }
            return;
        };
        self.streams
            .get_mut(&owner)
            .expect("named stream")
            .watches
            .insert(
                watcher,
                Watch {
                    name: open.watch,
                    family,
                    version,
                    project: opening.project,
                    rows: opening.rows,
                    parts: opening.parts,
                    opening: exchange,
                    closing: None,
                    unwatch: false,
                    delivery_pending: false,
                    text_pending: false,
                    end: None,
                    delivery: None,
                    text: None,
                    ended: None,
                },
            )
            .expect("admitted watch");
        if let Some(exchange) = exchange {
            self.pending
                .exchanges
                .insert(exchange, Waiting::Watch { sign_in, watcher, opening: true })
                .expect("admitted exchange");
        } else {
            self.streams.get_mut(&owner).expect("stream watch").me = Some(watcher);
        }
        self.admit_watch(env, owner, watcher, opening.request, out);
    }
    fn admit_watch(
        &mut self,
        env: &Env<Limits>,
        owner: Token,
        watcher: Token,
        request: Request,
        out: &mut Queue<Request>,
    ) {
        if self.stopping() {
            self.streams.get_mut(&owner).expect("watch stream").watches.get_mut(&watcher).expect("new watch").unwatch =
                true;
            self.watch_answer(env, watcher, None);
            self.watch_ended(env, watcher, jig_domain_views::End::Stopping, out);
        } else {
            out.push(request);
        }
    }
    pub(super) fn active(&self, sign_in: u64) -> u32 {
        let mut count = 0_u32;
        for (_, waiting) in &self.pending.exchanges {
            let number = match waiting {
                Waiting::Watch { sign_in, .. }
                | Waiting::Ask { sign_in, .. }
                | Waiting::Carry { sign_in, .. }
                | Waiting::Read { sign_in, .. }
                | Waiting::Authorize { sign_in, .. }
                | Waiting::ApplicationRead { sign_in, .. } => Some(*sign_in),
                Waiting::LinkAskWhose { .. }
                | Waiting::AskedWhose { .. }
                | Waiting::ApprovalWhose { .. }
                | Waiting::ApprovalCore { .. }
                | Waiting::Core { .. }
                | Waiting::LinkGet { .. }
                | Waiting::SignOut { .. }
                | Waiting::Keep { .. }
                | Waiting::Get { .. }
                | Waiting::Moving { .. }
                | Waiting::Made { .. }
                | Waiting::Checked { .. } => None,
            };
            if number == Some(sign_in) {
                count = count.saturating_add(1);
            }
        }
        count
    }
    pub(super) fn watch_answer(&mut self, env: &Env<Limits>, watcher: Token, why: Option<core::WatchRefusal>) {
        let Some(owner) = self.watcher_owner(watcher) else {
            return;
        };
        let stream = self.streams.get_mut(&owner).expect("watch stream");
        if stream.me == Some(watcher) {
            if let Some(why) = why {
                stream.watches.remove(&watcher);
                if !stream.closing() {
                    stream.phase = stream::Phase::Closing;
                    stream.ending =
                        if why == core::WatchRefusal::SignedOut { crate::Ended::SignedOut } else { crate::Ended::Busy };
                }
            }
            return;
        }
        let watch = stream.watches.get_mut(&watcher).expect("watch binding");
        let Some(exchange) = watch.opening.take() else {
            return;
        };
        if let Some(Waiting::Watch { watcher: wanted, opening: true, .. }) = self.pending.exchanges.get(&exchange)
            && *wanted == watcher
        {
            self.pending.exchanges.remove(&exchange);
            self.respond(exchange, routes::open_answer(why, &env.limits.wire));
        }
        if why.is_some() {
            self.streams.get_mut(&owner).expect("watch stream").watches.remove(&watcher);
        }
    }
    pub(crate) fn queue_packet(
        &mut self,
        env: &Env<Limits>,
        watcher: Token,
        value: Result<w::StreamEvent, w::Problem>,
        taken: Taken,
        out: &mut Queue<Request>,
    ) {
        let Some(owner) = self.watcher_owner(watcher) else {
            return;
        };
        let stream = self.streams.get_mut(&owner).expect("watch stream");
        if stream.closing() {
            return;
        }
        let value = if stream.me == Some(watcher) {
            match value {
                Ok(w::StreamEvent::Delivery(delivery)) => match stream.hello.take() {
                    Some(hello) => match w::Opened::new(
                        &env.limits.wire,
                        w::OpenedParts { hello, me: delivery.into_parts().document },
                    ) {
                        Ok(opened) => Ok(w::StreamEvent::Opened(opened)),
                        Err(problem) => Err(problem),
                    },
                    None => Ok(w::StreamEvent::Delivery(delivery)),
                },
                value @ (Ok(
                    w::StreamEvent::Hello(_)
                    | w::StreamEvent::Opened(_)
                    | w::StreamEvent::Text(_)
                    | w::StreamEvent::Ended(_),
                )
                | Err(_)) => value,
            }
        } else {
            value
        };
        let why = match value {
            Ok(value) => {
                if stream.queue(stream::packet(value, taken), env.limits.queued_bytes) {
                    return;
                }
                crate::StreamFailure::QueueBound { queued_bytes: env.limits.queued_bytes }
            }
            Err(problem) => crate::StreamFailure::Translation(problem),
        };
        stream.phase = stream::Phase::Closing;
        out.push(Request::StreamFailed { sign_in: stream.sign_in, why });
    }
    pub(super) fn delivery(
        &mut self,
        env: &Env<Limits>,
        watcher: Token,
        document: &core::documents::Document,
        out: &mut Queue<Request>,
    ) {
        let Some(owner) = self.watcher_owner(watcher) else {
            if self.issued_name(watcher) {
                out.push(Request::Core(Box::new(core::Event::Delivered { watcher })));
            }
            return;
        };
        let stream = self.streams.get_mut(&owner).expect("watch stream");
        let watch = stream.watches.get_mut(&watcher).expect("watch binding");
        if watch.family.is_some() || watch.delivery_pending {
            return;
        }
        watch.delivery_pending = true;
        let value = translate::document_packet(
            document,
            watch.project,
            watch.rows,
            watch.name,
            watch.version,
            &self.settings.models,
            &watch.parts,
            &env.limits.wire,
        );
        self.queue_packet(env, watcher, value, Taken::Delivery(watcher), out);
    }
    pub(super) fn application_delivery(
        &mut self,
        env: &Env<Limits>,
        watcher: Token,
        family: u16,
        record: Box<[u8]>,
        out: &mut Queue<Request>,
    ) {
        let Some(owner) = self.watcher_owner(watcher) else {
            if self.issued_name(watcher) {
                out.push(Request::Core(Box::new(core::Event::Delivered { watcher })));
            }
            return;
        };
        let watch =
            self.streams.get_mut(&owner).expect("watch stream").watches.get_mut(&watcher).expect("watch binding");
        if watch.family != Some(family) || watch.delivery_pending {
            return;
        }
        watch.delivery_pending = true;
        let value = translate::application_packet(family, watch.version, record, watch.name, &env.limits.wire);
        self.queue_packet(env, watcher, value, Taken::Delivery(watcher), out);
    }
    pub(super) fn run_text(
        &mut self,
        env: &Env<Limits>,
        watcher: Token,
        writing: jig_domain_views::Writing,
        out: &mut Queue<Request>,
    ) {
        let Some(owner) = self.watcher_owner(watcher) else {
            if self.issued_name(watcher) {
                out.push(Request::Core(Box::new(core::Event::StreamDelivered { watcher })));
            }
            return;
        };
        let watch =
            self.streams.get_mut(&owner).expect("watch stream").watches.get_mut(&watcher).expect("watch binding");
        if watch.family.is_some() || watch.text_pending {
            return;
        }
        watch.text_pending = true;
        let value = translate::text_packet(&writing, watch.name, &env.limits.wire);
        self.queue_packet(env, watcher, value, Taken::Text(watcher), out);
    }
    pub(super) fn watch_ended(
        &mut self,
        env: &Env<Limits>,
        watcher: Token,
        why: jig_domain_views::End,
        out: &mut Queue<Request>,
    ) {
        let Some(owner) = self.watcher_owner(watcher) else {
            return;
        };
        let watch =
            self.streams.get_mut(&owner).expect("watch stream").watches.get_mut(&watcher).expect("watch binding");
        if watch.end.is_some() {
            return;
        }
        watch.end = Some(why);
        let stream = self.streams.get_mut(&owner).expect("watch stream");
        if stream.me == Some(watcher) {
            if !stream.closing() {
                stream.phase = stream::Phase::Closing;
                stream.ending =
                    if why == jig_domain_views::End::SignedOut { crate::Ended::SignedOut } else { crate::Ended::Busy };
            }
            return;
        }
        let watch = stream.watches.get_mut(&watcher).expect("watch binding");
        let name = watch.name;
        let close = watch.closing.take();
        if let Some(exchange) = close
            && let Some(Waiting::Watch { watcher: wanted, opening: false, .. }) = self.pending.exchanges.get(&exchange)
            && *wanted == watcher
        {
            self.pending.exchanges.remove(&exchange);
            self.respond(exchange, routes::close_answer(why == jig_domain_views::End::SignedOut));
        }
        self.queue_packet(
            env,
            watcher,
            translate::end_packet(why, name, &env.limits.wire),
            Taken::Ended { watcher, signed_out: why == jig_domain_views::End::SignedOut },
            out,
        );
    }
    pub(super) fn end_streams(&mut self, sign_in: Option<u64>) {
        for _ in 0..self.streams.capacity() {
            let mut found = None;
            for (owner, stream) in &self.streams {
                if !stream.closing() && (sign_in.is_none() || sign_in == Some(stream.sign_in)) {
                    found = Some(*owner);
                    break;
                }
            }
            let Some(owner) = found else {
                break;
            };
            let stream = self.streams.get_mut(&owner).expect("ending stream");
            stream.phase = stream::Phase::Closing;
            stream.ending = if sign_in.is_some() { crate::Ended::SignedOut } else { crate::Ended::Stopping };
        }
    }
    pub(super) fn stream_up(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) -> bool {
        let mut selected = None;
        for (owner, stream) in &self.streams {
            if stream.ready() || self.aborting() {
                selected = Some(*owner);
                break;
            }
        }
        let Some(owner) = selected else {
            return false;
        };
        let mut stream = self.streams.remove(&owner).expect("selected stream");
        if stream.closing() {
            if !stream.closed_connection {
                stream.closed_connection = true;
                if let Some(connection) = self.connections.get_mut(&owner) {
                    connection.ended = Some(stream.ending);
                }
            }
            let selected = match stream.watches.first() {
                Some((watcher, _)) => Some(*watcher),
                None => None,
            };
            if let Some(watcher) = selected {
                let watch = stream.watches.get_mut(&watcher).expect("closing watch");
                if !watch.unwatch {
                    watch.unwatch = true;
                    out.push(Request::Core(Box::new(core::Event::Unwatch { watcher })));
                } else if watch.delivery_pending {
                    watch.delivery_pending = false;
                    watch.delivery = None;
                    out.push(Request::Core(Box::new(core::Event::Delivered { watcher })));
                } else if watch.text_pending {
                    watch.text_pending = false;
                    watch.text = None;
                    out.push(Request::Core(Box::new(core::Event::StreamDelivered { watcher })));
                } else {
                    let watch = stream.watches.remove(&watcher).expect("ended watch");
                    if let Some(exchange) = watch.closing {
                        self.pending.exchanges.remove(&exchange);
                        self.respond(exchange, routes::close_answer(false));
                    }
                    if let Some(exchange) = watch.opening {
                        self.pending.exchanges.remove(&exchange);
                        self.respond(
                            exchange,
                            routes::open_answer(Some(core::WatchRefusal::Unknown), &env.limits.wire),
                        );
                    }
                }
            } else if stream.watches.is_empty() {
                return true;
            }
        } else if let Some(message) = stream.message.take() {
            out.push(Request::Transport(crate::ToTransport::Streamed { stream: owner, message }));
        } else if let Some(packet) = stream.next() {
            stream.write(packet);
        }
        self.streams.insert(owner, stream).expect("same stream slot");
        true
    }
}
