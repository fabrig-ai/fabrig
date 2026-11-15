//! A call follows a job log once to a configured, separately pooled origin (4.5).
use super::{Base, Common, Component, Dialect, GitHubCall, GitHubForge, Waiting, find};
use crate::{Event, Fact, Limits};
use skein_forge::{
    api,
    github::{
        plans::plan::{Exchange, Retention},
        redirect::Redirect,
    },
};
use skein_lib::{Env, Id, List, Queue, Token};

impl Component {
    /// Origins only grow, preserving each connection's pool index (3.1, 4.5).
    pub(super) fn log_origin(&mut self, env: &Env<Limits>, number: u16, origin: crate::settings::LogOrigin) {
        let Some(entry) = find(&mut self.forges, number) else {
            return;
        };
        match entry.dialect {
            Dialect::Forgejo { .. } => return,
            Dialect::GitHub(_) => {}
        }
        let forge = &mut entry.base.forge;
        let tls = match forge.transport {
            crate::settings::Transport::Plaintext { .. } => false,
            crate::settings::Transport::Tls { .. } => true,
        };
        if crate::limits::check_origin(
            &env.limits,
            &self.common.shared,
            &self.common.owner.user_agent,
            forge,
            tls,
            &origin,
        )
        .is_err()
        {
            return;
        }
        for held in &forge.log_origins {
            if held.origin.same(&origin.origin) {
                return;
            }
        }
        if forge.log_origins.len() >= env.limits.log_origins {
            return;
        }
        if forge.log_origins.room() == 0 {
            let held = core::mem::replace(&mut forge.log_origins, List::with_capacity(env.limits.log_origins));
            for origin in held.into_boxed() {
                forge.log_origins.push(origin).expect("held origins fit the bound");
            }
        }
        forge.log_origins.push(origin).expect("checked origin room");
    }
}

impl Common {
    #[expect(
        clippy::too_many_arguments,
        reason = "the redirect carries its split forge and retention through the call"
    )]
    pub(super) fn follow_github_log(
        &mut self,
        base: &mut Base,
        gh: &mut GitHubForge,
        holder: (Token, Id<Waiting>),
        github: &mut GitHubCall,
        redirect: Redirect,
        retention: Retention,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        let (call, id) = holder;
        let forge = &base.forge;
        let mut matched = None;
        for index in 0..forge.log_origins.len() {
            if forge.log_origins.get(index).expect("origin").origin.same(&redirect.origin) {
                matched = Some(index);
                break;
            }
        }
        if let Some(token) = self.waiting.get(id).expect("call").connection {
            let c =
                self.connection_slab.get_mut(*self.connections.get(&token).expect("connection")).expect("connection");
            c.discard_redirect(env);
        }
        self.release_github_call(id, env, io);
        let Some(index) = matched else {
            let data = &mut gh.state;
            let mut said = false;
            for origin in &data.said_origins {
                if origin.same(&redirect.origin) {
                    said = true;
                    break;
                }
            }
            if !said {
                if data.said_origins.room() == 0 {
                    drop(data.said_origins.pop());
                }
                data.said_origins.push(redirect.origin.clone());
                up.push(Event::Fact { forge: base.forge.number, fact: Fact::Origin { origin: redirect.origin } });
            }
            self.finish(call, id, Err(api::Error::Refused), up);
            return;
        };
        github.route.as_mut().expect("sent route").origin = Some(index);
        github.plan.as_mut().expect("begun plan").set_pending(Exchange {
            request: skein_forge::github::redirect::request(&redirect),
            retention: Some(retention),
            comparison: skein_forge::github::plans::plan::Comparison::Full,
        });
        self.wake = Some(env.now);
    }
}
