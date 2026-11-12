//! Machine-world state access (skein/docs/design/forge/connection.md, 10).
use super::{Component, Connection, Dialect, Event, ForgeEntry, GitHubForge, Limits, Side, Start};
use crate::seam::{Access, ConnectionAccess};
use skein_lib::{Env, Queue};

impl Access for Component {
    fn test_github_bearer(
        &mut self,
        app: u64,
        key: &skein_lib::Secret,
        wall: skein_lib::Wall,
        deadline: skein_lib::Duration,
    ) -> Option<&skein_lib::Secret> {
        self.test_github_mut().jwt.bearer(app, key, wall, deadline)
    }
    fn test_calls_ready(&mut self, access: crate::Access) {
        self.forges.get_mut(0).expect("forge").base.start = Start::Ready { cap: skein_forge::github::pin::PAGE_CAP };
        self.forges.get_mut(0).expect("forge").base.forge.access = Some(access);
        let gh = self.test_github_mut();
        gh.connected = true;
        gh.identity_checked = true;
        gh.baseline = false;
    }
    fn test_github(&self) -> &crate::installations::GitHub {
        &self.test_github_forge().state
    }
    fn test_github_mut(&mut self) -> &mut crate::installations::GitHub {
        &mut self.test_github_forge_mut().state
    }
    fn test_local(&mut self, env: &Env<Limits>, up: &mut Queue<Event>) {
        let before = up.len();
        let ForgeEntry { base, dialect } = self.forges.get_mut(0).expect("forge");
        match dialect {
            Dialect::Forgejo { .. } => unreachable!("fixture dialect"),
            Dialect::GitHub(gh) => {
                for _ in 0..64_u32 {
                    if !self.common.github_local(base, gh, env, up) || up.len() != before {
                        break;
                    }
                }
            }
        }
    }

    fn test_last_connection(&mut self) -> &mut Connection {
        let (_, id) = self.common.connections.last().expect("fixture connection");
        self.common.connection_slab.get_mut(*id).expect("indexed connection")
    }
    fn test_open_call(&mut self, env: &Env<Limits>, io: &mut Queue<skein_io::Request>) {
        let forge = &self.forges.get(0).expect("fixture forge").base.forge;
        self.common.open_call_origin(forge, None, env, io);
    }

    fn test_github_handoffs(&self) -> u32 {
        let mut count = 0_u32;
        for entry in &self.forges {
            match &entry.dialect {
                Dialect::GitHub(gh) => {
                    if gh.exchange.test_github_handoff() {
                        count = count.saturating_add(1);
                    }
                }
                Dialect::Forgejo { .. } => {}
            }
        }
        for (_, id) in &self.common.calls {
            match &self.common.waiting.get(*id).expect("indexed call").side {
                Side::GitHub(call) => {
                    if call.exchange.test_github_handoff() {
                        count = count.saturating_add(1);
                    }
                }
                Side::Unbegun | Side::Forgejo(_) => {}
            }
        }
        count
    }
}

impl Component {
    fn test_github_forge(&self) -> &GitHubForge {
        match &self.forges.get(0).expect("forge").dialect {
            Dialect::Forgejo { .. } => unreachable!("fixture dialect"),
            Dialect::GitHub(gh) => gh,
        }
    }
    fn test_github_forge_mut(&mut self) -> &mut GitHubForge {
        match &mut self.forges.get_mut(0).expect("forge").dialect {
            Dialect::Forgejo { .. } => unreachable!("fixture dialect"),
            Dialect::GitHub(gh) => gh,
        }
    }
}

impl ConnectionAccess for Connection {
    fn test_deadline(&self) -> Option<skein_lib::Time> {
        self.deadline()
    }
    fn test_until(&mut self, until: Option<skein_lib::Time>) {
        self.until(until);
    }
}
