//! The forge component's boundary to the engine (protocol/README.md, 4).
mod configuration;
pub(crate) mod gone;
pub(crate) mod origins;
pub(crate) mod records;
pub mod tokens;
use crate::{limits, settings};
use alloc::boxed::Box;
pub use configuration::{ForgeRuntime, USER_AGENT, config, indexed, prepare, problems, settings, shared_limits, small};
use skein_lib::{Env, List, Map, Queue, Time, Wall};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;

/// The address of a forge whose host name doesn't resolve (protocol/setup/init.md, 3). skein's
/// forge connection and setup host refuse an in-process address, so a held forge keeps its
/// entries with this one: loopback port 0, which no listener can hold. No socket ever sees it:
/// `forge::connect` and the setup gate refuse a held forge before either component is asked.
pub const HELD_ADDRESS: core::net::SocketAddr =
    core::net::SocketAddr::V4(core::net::SocketAddrV4::new(core::net::Ipv4Addr::LOCALHOST, 0));
/// Limits shared with the domain client and io.
#[must_use]
pub fn shared(limits: &limits::Limits, io: &skein_io::Limits) -> skein_forge_connection::Shared {
    shared_limits(limits, io)
}
/// Wrap the client's terminal without bringing protocol state into the root.
#[must_use]
pub fn event(answer: skein_forge_connection::Answered) -> root::Event {
    root::Event::Forge(Box::new(forge::Event::Client(skein_forge_client::Event::Answered {
        free: answer.free,
        rate: answer.rate,
        call: answer.call,
        cost: answer.cost,
        result: answer.result,
    })))
}
/// Hand a verified delivery hint to root admission.
#[must_use]
pub fn hinted(hinted: skein_forge_connection::hooks::Hinted) -> root::Event {
    root::Event::Forge(Box::new(forge::Event::Hint { hint: hinted.hint }))
}
/// Recover an answer refused by admission or returned during the drain.
pub fn back(event: Box<forge::Event>) -> Result<skein_forge_connection::Answered, Box<forge::Event>> {
    if let forge::Event::Client(skein_forge_client::Event::Answered { call, cost, result, free, rate }) = *event {
        Ok(skein_forge_connection::Answered { call, cost, result, free, rate })
    } else {
        Err(event)
    }
}
/// Engine-owned queues and component, with its own output reservations.
pub(crate) struct Layer {
    pub(crate) component: skein_forge_connection::Component,
    pub(crate) env: Env<skein_forge_connection::Limits>,
    pub(crate) above: Queue<skein_forge_connection::Event>,
    pub(crate) io: Queue<skein_io::Request>,
    pub(crate) closed: bool,
    begun: bool,
    pub(crate) held: Map<u16, Box<[u8]>>,
    pub(crate) origins: origins::Origins,
    pub(crate) endpoints: Map<u16, skein_forge_connection::settings::Forge>,
    pub(crate) github: Option<ForgeRuntime>,
    pub(crate) log_origin_limit: u32,
    pub(crate) roots: Option<skein_tls::Config>,
    pub(crate) webhooks: bool,
    pub(crate) serving: Option<()>,
    // Some waits for this item; None remembers a completed removal for this run.
    pub(crate) erasing: Map<u16, Option<temper_protocol_records::Item>>,
    pub(crate) erase_waiting: Map<u16, ()>,
    pub(crate) resolving: Map<u16, ()>,
    pub(crate) names: Map<Box<[u8]>, core::net::IpAddr>,
    #[cfg(test)]
    pub(crate) starts: List<(u16, skein_forge_connection::Access)>,
    #[cfg(test)]
    pub(crate) accepted: List<(u16, Box<[u64]>)>,
    #[cfg(test)]
    pub(crate) returned: u32,
}
struct Endpoints {
    roots: Option<skein_tls::Config>,
    forges: Map<u16, skein_forge_connection::settings::Forge>,
}
fn endpoints(settings: &mut skein_forge_connection::settings::Settings, capacity: u32) -> Endpoints {
    let mut roots = None;
    let mut forges = Map::with_capacity(capacity);
    let initial = core::mem::replace(&mut settings.forges, List::with_capacity(capacity));
    for forge in initial.into_boxed() {
        if let skein_forge_connection::settings::Transport::Tls { trust, .. } = &forge.transport {
            roots = Some(trust.clone());
        }
        forges.insert(forge.number, forge).expect("validated endpoints");
    }
    Endpoints { roots, forges }
}
impl Layer {
    pub(crate) fn new(
        mut settings: skein_forge_connection::settings::Settings,
        limits: skein_forge_connection::Limits,
        shared: &skein_forge_connection::Shared,
        configured: &[settings::Forge],
    ) -> Result<Self, skein_forge_connection::Unusable> {
        let mut held_hosts = Map::with_capacity(limits.forges);
        let mut names = Map::uncapped();
        for endpoint in &settings.forges {
            for forge in configured {
                if endpoint.number == forge.number
                    && let jig_protocol_config::Address::Host { name, .. } = &forge.api
                    && let skein_io::Addr::Ip(address) = endpoint.address
                    && address != HELD_ADDRESS
                {
                    names.insert(name.clone(), address.ip()).expect("configured host capacity");
                }
                if endpoint.number == forge.number
                    && endpoint.address == skein_io::Addr::Ip(HELD_ADDRESS)
                    && let jig_protocol_config::Address::Host { name, .. } = &forge.api
                {
                    held_hosts.insert(forge.number, name.clone()).expect("configured held forge");
                }
            }
        }
        let Endpoints { roots, forges: endpoints } = endpoints(&mut settings, limits.forges);
        Ok(Self {
            origins: origins::Origins::new(limits.forges, limits.log_origins),
            endpoints,
            github: None,
            log_origin_limit: limits.log_origins,
            roots,
            webhooks: settings.webhooks.is_some(),
            serving: None,
            erasing: Map::with_capacity(65_536),
            erase_waiting: Map::with_capacity(limits.forges),
            resolving: Map::with_capacity(limits.forges),
            names,
            held: held_hosts,
            component: skein_forge_connection::Component::new(settings, &limits, shared)?,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            above: Queue::uncapped(),
            io: Queue::uncapped(),
            closed: false,
            begun: false,
            #[cfg(test)]
            starts: List::with_capacity(limits.forges),
            #[cfg(test)]
            accepted: List::with_capacity(limits.forges),
            #[cfg(test)]
            returned: 0,
        })
    }
    pub(crate) fn unreachable(&self, forge: u16) -> Option<Box<[u8]>> {
        self.held.get(&forge).cloned()
    }
    pub(crate) fn down(&mut self, request: skein_forge_connection::Request) {
        if let skein_forge_connection::Request::Close = &request {
            self.begun = true; // A closed component owes no initial begin, including before the mark.
        }
        if let skein_forge_connection::Request::Connect { forge, .. } = &request {
            assert!(!self.held.contains_key(forge), "temper gates unresolved connects");
            self.origins.clear(*forge);
        }
        if let skein_forge_connection::Request::Remove { forge } = &request {
            self.origins.forget(*forge);
        }
        #[cfg(test)]
        if let skein_forge_connection::Request::Returned(_)
        | skein_forge_connection::Request::Refused(_)
        | skein_forge_connection::Request::RefusedHint(_) = &request
        {
            self.returned = self.returned.checked_add(1).expect("test return count");
        }
        #[cfg(test)]
        if let skein_forge_connection::Request::Connect { forge, access } = &request {
            self.starts.push((*forge, access.clone())).expect("test starts");
        }
        #[cfg(test)]
        if let skein_forge_connection::Request::Installations { forge, accepted } = &request {
            self.accepted.push((*forge, accepted.clone())).expect("test accepted");
        }
        self.component.down(&self.env, request, &mut self.above, &mut self.io);
    }
    pub(crate) fn up(&mut self, event: skein_io::Event) {
        self.component.up(&self.env, crate::accounts::untag(event), &mut self.above, &mut self.io);
    }
    #[cfg(test)]
    pub(crate) const fn startup_begun(&self) -> bool {
        self.begun
    }
    pub(crate) fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        if !self.begun {
            self.begun = true;
            self.component.begin(&self.env, &mut self.above, &mut self.io);
        } else if let Some(deadline) = self.component.next_deadline()
            && deadline <= now
        {
            self.component.fire(&self.env, &mut self.above, &mut self.io);
        }
        self.component.reclaim();
    }
    pub(crate) fn work_pending(&self, now: Time) -> bool {
        !self.begun
            || !self.above.is_empty()
            || !self.io.is_empty()
            || match self.component.next_deadline() {
                Some(deadline) => deadline <= now,
                None => false,
            }
    }
}
pub(crate) fn tag(mut request: skein_io::Request) -> skein_io::Request {
    if let skein_io::Request::Connect { owner, .. }
    | skein_io::Request::Listen { owner, .. }
    | skein_io::Request::Bind { owner, .. } = &mut request
    {
        assert!(owner.raw() < (1_u64 << 60_u32), "forge token fits its prefix");
        *owner = skein_lib::Token::new(owner.raw() | crate::tokens::FORGE);
    }
    request
}

/// What a forge connect starts from.
pub(crate) enum Start {
    /// The forge's records, with their accepted installations.
    Restored(temper_protocol_records_forge::Start),
    /// An adoption's fresh access.
    Adopted(skein_forge_connection::Access),
}
/// Every forge connect enters here. A held forge is refused first, naming its host, and
/// nothing reaches skein's connection; otherwise the start goes down, or the reason it
/// can't is noted.
pub(crate) fn connect(engine: &mut crate::engine::Engine, number: u16, start: Result<Start, crate::NotConnected>) {
    if engine.domain.forge_ended(number) {
        return;
    }
    if let Some(host) = engine.forge.unreachable(number) {
        engine.notes.push(crate::Note::ForgeNotConnected { number, why: crate::NotConnected::Unreachable { host } });
        return;
    }
    match start {
        Ok(Start::Restored(start)) => {
            if let Some(forge) = engine.workers.forges.get_mut(number) {
                forge.user = start.access.user;
            }
            engine.forge.down(skein_forge_connection::Request::Connect { forge: number, access: start.access });
            if start.slug.is_some() {
                for installation in &start.accepted {
                    crate::setup::lifecycle::installation(
                        engine,
                        number,
                        *installation,
                        None,
                        skein_forge_connection::Change::Created,
                    );
                }
                engine
                    .forge
                    .down(skein_forge_connection::Request::Installations { forge: number, accepted: start.accepted });
            }
        }
        Ok(Start::Adopted(access)) => {
            if let Some(forge) = engine.workers.forges.get_mut(number) {
                forge.user = access.user;
            }
            engine.forge.down(skein_forge_connection::Request::Connect { forge: number, access });
        }
        Err(why) => engine.notes.push(crate::Note::ForgeNotConnected { number, why }),
    }
}
/// The existing setup refusal carries the unresolved host without changing the client contract.
pub(crate) fn unreachable_text(host: &[u8]) -> Box<[u8]> {
    let mut out = skein_lib::Writer::new(host.len().checked_add(b"unreachable: ".len()).expect("host diagnostic"));
    out.put(b"unreachable: ").expect("prefix");
    out.put(host).expect("host");
    out.finish()
}
