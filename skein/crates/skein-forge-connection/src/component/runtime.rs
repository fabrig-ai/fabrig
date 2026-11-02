//! Runtime forge membership (skein/docs/design/forge-connection.md, 3.1).
use super::{Component, Event, Limits, Start, initial};
use crate::settings::{Forge, Kind};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{Env, List, Queue};

impl Component {
    pub(super) fn add(&mut self, env: &Env<Limits>, forge: Forge) {
        for configured in &self.settings.forges {
            if configured.number == forge.number {
                #[cfg(test)]
                unique(configured.number, forge.number);
                return;
            }
        }
        if self.settings.forges.len() >= env.limits.forges {
            return;
        }
        let state =
            if forge.access.is_some() { initial(forge.kind, env.limits.start_backoff) } else { Start::Unconnected };
        let kind = forge.kind;
        let before = self.settings.forges.len();
        self.settings.forges.push(forge).expect("forge room checked");
        if crate::limits::check(&env.limits, &self.shared, &self.settings).is_err() {
            self.settings.forges.truncate(before);
            return;
        }
        let github = match kind {
            Kind::Forgejo => None,
            Kind::GitHub => {
                let mut github = crate::installations::GitHub::new(env.limits.installations);
                github.said_origins = Queue::with_capacity(env.limits.log_origins);
                Some(Box::new(github))
            }
        };
        self.starts.push(state).expect("forge room checked");
        self.github.push(github).expect("forge room checked");
        self.own_tokens.push(None).expect("forge room checked");
        self.own_plans.push(None).expect("forge room checked");
    }

    pub(super) fn remove(&mut self, number: u16, up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>) {
        let mut position = None;
        for at in 0..self.settings.forges.len() {
            if self.settings.forges.get(at).expect("forge").number == number {
                position = Some(at);
                break;
            }
        }
        let Some(at) = position else { return };
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
        for _ in 0..self.calls.len() {
            let mut next = None;
            for (call, id) in &self.calls {
                if self.waiting.get(*id).expect("indexed call").repository.forge == number {
                    next = Some((*call, *id));
                    break;
                }
            }
            let Some((call, id)) = next else { break };
            self.finish(call, id, Err(api::Error::Unavailable), up);
        }
        for (_, id) in &self.connections {
            let c = self.connection_slab.get_mut(*id).expect("indexed connection");
            if c.forge == number {
                c.membership = crate::connection::Membership::Removed;
                c.abort(io);
            }
        }
        let last = self.starts.len().checked_sub(1).expect("configured forge");
        let state = core::mem::replace(self.starts.get_mut(last).expect("last start"), Start::Unconnected);
        *self.starts.get_mut(at).expect("removed start") = state;
        self.starts.truncate(last);
        let github = self.github.get_mut(last).expect("last GitHub slot").take();
        *self.github.get_mut(at).expect("removed GitHub slot") = github;
        self.github.truncate(last);
        let token = self.own_tokens.get_mut(last).expect("last token").take();
        *self.own_tokens.get_mut(at).expect("removed token") = token;
        self.own_tokens.truncate(last);
        let plan = self.own_plans.get_mut(last).expect("last plan").take();
        *self.own_plans.get_mut(at).expect("removed plan") = plan;
        self.own_plans.truncate(last);
        let forges = core::mem::replace(&mut self.settings.forges, List::with_capacity(self.starts.capacity()));
        let mut forges = forges.into_boxed();
        if at != last {
            let [removed, moved] = forges
                .get_disjoint_mut([
                    usize::try_from(at).expect("forge index"),
                    usize::try_from(last).expect("last forge index"),
                ])
                .expect("distinct configured forge positions");
            core::mem::swap(removed, moved);
        }
        for (index, forge) in forges.into_iter().enumerate() {
            if index < usize::try_from(last).expect("last forge index") {
                self.settings.forges.push(forge).expect("remaining forges fit");
            }
        }
    }
}

#[cfg(test)]
#[expect(clippy::disallowed_macros, reason = "the runtime membership contract asserts duplicate numbers only in tests")]
fn unique(configured: u16, number: u16) {
    debug_assert_ne!(configured, number, "a forge number is configured once");
}
