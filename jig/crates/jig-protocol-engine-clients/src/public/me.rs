//! Public project identifiers in me (jig/docs/design/domain/people/public-reading.md, 8).
use super::{Box, Component, Env, Limits, Queue, Request, Token, Waiting, core, routes, w};
use crate::translate;
use jig_domain_core_vocabulary::documents::{Document, Frame};
use skein_lib::Writer;
pub(super) fn frame(document: &Document) -> Option<&Frame> {
    match document {
        Document::Frame(value) => Some(value),
        Document::Chips(_)
        | Document::Run(_)
        | Document::Task(_)
        | Document::Tree(_)
        | Document::Lines(_)
        | Document::Proposal(_)
        | Document::Escalation(_)
        | Document::Question(_)
        | Document::PersonTask(_)
        | Document::Result(_)
        | Document::Project(_)
        | Document::Note(_)
        | Document::Inbox(_)
        | Document::Terminal { .. }
        | Document::Accounts(_)
        | Document::Deployment(_)
        | Document::Settings(_) => None,
    }
}
pub(super) fn read_frame(answer: &core::reads::Answer) -> Option<&Frame> {
    match answer {
        core::reads::Answer::Document(value) => frame(value),
        core::reads::Answer::TaskText(_)
        | core::reads::Answer::Setting(_)
        | core::reads::Answer::Revision(_)
        | core::reads::Answer::Revisions(_)
        | core::reads::Answer::Tasks(_)
        | core::reads::Answer::Ended(_)
        | core::reads::Answer::Page(_)
        | core::reads::Answer::None(_) => None,
    }
}
impl Component {
    pub(super) fn named_read(&mut self, env: &Env<Limits>, exchange: Token, frame: &Frame) {
        let Some(waiting) = self.pending.exchanges.remove(&exchange) else {
            return;
        };
        match waiting {
            Waiting::Read { .. } => (),
            waiting @ (Waiting::Authorize { .. }
            | Waiting::ApprovalWhose { .. }
            | Waiting::ApprovalCore { .. }
            | Waiting::Core { .. }
            | Waiting::LinkGet { .. }
            | Waiting::SignOut { .. }
            | Waiting::Watch { .. }
            | Waiting::Keep { .. }
            | Waiting::Get { .. }
            | Waiting::Made { .. }
            | Waiting::Moving { .. }
            | Waiting::ApplicationRead { .. }
            | Waiting::Ask { .. }
            | Waiting::Carry { .. }
            | Waiting::Checked { .. }) => {
                self.pending.exchanges.insert(exchange, waiting).expect("same unrelated slot");
                return;
            }
        }
        let answer = read(frame, &self.settings.models, self.public.names(), &env.limits.wire);
        match answer {
            Ok(answer) => self.respond(exchange, routes::encoded_read(&answer)),
            Err(_) => {
                if let Some(owner) = self.owner(exchange) {
                    self.connections.get_mut(&owner).expect("exchange").close();
                }
            }
        }
    }
    pub(super) fn named_delivery(
        &mut self,
        env: &Env<Limits>,
        watcher: Token,
        frame: &Frame,
        out: &mut Queue<Request>,
    ) {
        let mut owner = None;
        for (name, stream) in &self.streams {
            if stream.watches.contains_key(&watcher) {
                owner = Some(*name);
                break;
            }
        }
        let Some(owner) = owner else {
            if watcher.raw() != 0 && watcher.raw() < self.next {
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
        let value =
            delivery(frame, watch.name, watch.version, &self.settings.models, self.public.names(), &env.limits.wire);
        self.queue_packet(env, watcher, value, crate::stream::Taken::Delivery(watcher), out);
    }
}
fn read(
    frame: &Frame,
    models: &[crate::Model],
    names: Option<&crate::public::Names>,
    limits: &w::Limits,
) -> Result<w::ReadAnswer, w::Problem> {
    let value = w::ReadValue::Me(translate::owner::frame_named(frame, models, names, limits)?);
    let mut out = Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut out).expect("measured me");
    translate::reads::owned(w::Owner::Jig, jig_client_protocol_wire::VERSION, out.finish(), limits)
}
fn delivery(
    frame: &Frame,
    watch: u64,
    version: u32,
    models: &[crate::Model],
    names: Option<&crate::public::Names>,
    limits: &w::Limits,
) -> Result<w::StreamEvent, w::Problem> {
    let value = w::DeliveryValue::Me(translate::owner::frame_named(frame, models, names, limits)?);
    let document = translate::watches::owned(value, version, limits)?;
    Ok(w::StreamEvent::Delivery(w::Delivery::new(limits, w::DeliveryParts { watch, document })?))
}
