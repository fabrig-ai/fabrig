//! Holding and releasing runtime forges (forge-connection.md, sections 3.1 and 3.5).
use super::{Component, Event, Start};
use crate::{Limits, Unusable, limits, settings};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue};

impl Component {
    /// Checks whether one more forge can be held, using start's per-forge checks.
    /// Access is checked by `Add`, as by `Connect`, and can be refused independently.
    pub fn admits(&self, limits: &Limits, forge: &settings::Forge) -> Result<(), Unusable> {
        self.admits_number(limits, forge.number, forge)
    }

    pub(super) fn admits_number(&self, limits: &Limits, number: u16, forge: &settings::Forge) -> Result<(), Unusable> {
        if number != forge.number {
            return Err(Unusable::Number { forge: number });
        }
        if self.settings.forges.len() >= limits.forges {
            return Err(Unusable::Forges { forges: limits.forges });
        }
        limits::forge(limits, &self.shared, &self.settings, forge)
    }

    pub(super) fn add(
        &mut self,
        env: &Env<Limits>,
        mut forge: settings::Forge,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        let access = forge.access.take();
        let number = forge.number;
        let github = match forge.kind {
            settings::Kind::Forgejo => None,
            settings::Kind::GitHub => {
                let mut github = crate::installations::GitHub::new(env.limits.installations);
                github.said_origins = Queue::with_capacity(env.limits.log_origins);
                Some(Box::new(github))
            }
        };
        self.settings.forges.push(forge).expect("admitted forge");
        self.starts.push(Start::Unconnected).expect("admitted start");
        self.github.push(github).expect("admitted installations");
        self.own_tokens.push(None).expect("admitted connection");
        self.own_plans.push(None).expect("admitted plan");
        if let Some(access) = access {
            self.connect(env, number, access, up, io);
        }
    }

    pub(super) fn remove(
        &mut self,
        env: &Env<Limits>,
        number: u16,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        let mut held = false;
        for forge in &self.settings.forges {
            if forge.number == number {
                held = true;
                break;
            }
        }
        if !held {
            return;
        }
        for _ in 0..self.calls.len() {
            let mut next = None;
            for (call, id) in &self.calls {
                if self.waiting.get(*id).expect("indexed call").repository.forge == number {
                    next = Some((*call, *id));
                    break;
                }
            }
            let Some((call, id)) = next else { break };
            self.expire(call, id, up, io);
        }
        for (_, id) in &self.connections {
            let connection = self.connection_slab.get_mut(*id).expect("indexed connection");
            if connection.forge == number {
                connection.close(connection.in_flight(), io);
            }
        }
        self.hooks.remove(number);
        let capacity = env.limits.forges;
        let forges = core::mem::replace(&mut self.settings.forges, List::with_capacity(capacity));
        let starts = core::mem::replace(&mut self.starts, List::with_capacity(capacity));
        let github = core::mem::replace(&mut self.github, List::with_capacity(capacity));
        let tokens = core::mem::replace(&mut self.own_tokens, List::with_capacity(capacity));
        let plans = core::mem::replace(&mut self.own_plans, List::with_capacity(capacity));
        for ((((forge, start), github), token), plan) in forges
            .into_boxed()
            .into_iter()
            .zip(starts.into_boxed())
            .zip(github.into_boxed())
            .zip(tokens.into_boxed())
            .zip(plans.into_boxed())
        {
            if forge.number != number {
                self.settings.forges.push(forge).expect("remaining forge");
                self.starts.push(start).expect("remaining start");
                self.github.push(github).expect("remaining installations");
                self.own_tokens.push(token).expect("remaining connection");
                self.own_plans.push(plan).expect("remaining plan");
            }
        }
        // Resolution tables are caches shared by forges. Clear them so that reusing a
        // removed number cannot resolve through its previous repositories or users.
        self.forgejo_tables =
            skein_forge::forgejo::plans::tables::Tables::new(env.limits.repositories, env.limits.users);
        self.github_tables = skein_forge::github::plans::tables::Tables::new(env.limits.repositories, env.limits.users);
    }
}
