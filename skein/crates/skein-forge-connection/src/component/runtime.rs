//! Runtime forge membership (skein/docs/design/forge/connection.md, 3.1).
use super::{Component, Event, ForgeEntry, Limits, find};
use crate::settings::Forge;
use skein_forge::api;
use skein_lib::{Env, Queue};

impl Component {
    pub(super) fn add(&mut self, env: &Env<Limits>, forge: Forge) {
        let Self { forges, common } = self;
        if find(forges, forge.number).is_some() {
            return;
        }
        if forges.len() >= env.limits.forges
            || crate::limits::check_added(&env.limits, &common.shared, &common.owner, &forge).is_err()
        {
            return;
        }
        forges.push(ForgeEntry::new(forge, &env.limits)).expect("forge room checked");
    }

    pub(super) fn remove(
        &mut self,
        env: &Env<Limits>,
        number: u16,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        let Self { forges, common } = self;
        let mut position = None;
        for (at, entry) in forges.iter().enumerate() {
            if entry.base.forge.number == number {
                position = Some(at);
                break;
            }
        }
        let Some(position) = position else { return };
        common.forget_kept(number);
        // Discard this forge's queued notifications before returning its terminals.
        for _ in 0..up.len() {
            let event = up.pop().expect("queued notification");
            let source = match &event {
                Event::Installation { forge, .. }
                | Event::Fact { forge, .. }
                | Event::GitToken { forge, .. }
                | Event::Connected { forge }
                | Event::Stopped { forge, .. } => Some(*forge),
                Event::Hint(hint) => Some(hint.hint.repository.forge),
                Event::Answered(_) | Event::Listening { .. } | Event::ListenerLost | Event::Closed => None,
            };
            if source != Some(number) {
                up.push(event);
            }
        }
        // Each pass removes one of the bounded live calls belonging to this forge.
        for _ in 0..common.calls.len() {
            let mut next = None;
            for (call, id) in &common.calls {
                if common.waiting.get(*id).expect("indexed call").repository.forge == number {
                    next = Some((*call, *id));
                    break;
                }
            }
            let Some((call, id)) = next else { break };
            common.finish(call, id, Err(api::Error::Unavailable), up);
        }
        for (_, id) in &common.connections {
            let c = common.connection_slab.get_mut(*id).expect("indexed connection");
            if c.forge == number {
                c.membership = crate::connection::Membership::Removed;
                c.abort(env, io);
            }
        }
        let _removed = forges.swap_remove(u32::try_from(position).expect("forge position")).expect("configured forge");
    }
}
