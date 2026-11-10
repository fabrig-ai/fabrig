//! Approval operations at each client's real link, and externally applied people
//! records (jig/docs/design/protocol/client/sign-in.md;
//! jig/docs/design/client/domain/testing.md, 5–6).
use super::{ClientId, Session, Time, Wall, core, people, root};
use jig_client::{Approval, ApprovalAct};
use jig_client_protocol_link::Signing;
use skein_lib::Secret;

impl Session {
    /// Play this client's transport as a browser carrying its cookie.
    pub fn as_browser(&mut self, client: ClientId) {
        self.component.as_browser(client);
    }

    /// Ask, read or decide through this client's real link and credential.
    pub fn approval(&mut self, client: ClientId, now: Time, wall: Wall, act: ApprovalAct) {
        self.component.clock(now, wall);
        self.component.approval(client, act);
        self.pump(now, wall).expect("joint approval exchange");
    }

    /// Take through the real link; the component alone asks whose and signs in.
    pub fn take_approval(&mut self, client: ClientId, now: Time, wall: Wall, secret: Secret) {
        self.component.clock(now, wall);
        self.component.take_approval(client, secret);
        self.pump(now, wall).expect("joint approval take");
    }

    /// Consume an original link answer once, keeping its secret opaque.
    pub fn take_approval_answer(&mut self, client: ClientId) -> Option<Approval> {
        self.component.take_approval_answer(client)
    }

    /// Consume the link's terminal for a take, without inventing a refusal.
    pub fn take_approval_signing(&mut self, client: ClientId) -> Option<Signing> {
        self.component.take_approval_signing(client)
    }

    /// The credential installed by the link, never reconstructed from core state.
    #[must_use]
    pub fn credential(&self, client: ClientId) -> Option<&Secret> {
        self.component.credential(client)
    }

    /// The shipping component's next deadline, including approvals' expiry.
    #[must_use]
    pub fn component_deadline(&self) -> Option<Time> {
        self.component.next_deadline()
    }

    /// Sign-in numbers and parties in the real store peer's applied people rows.
    #[must_use]
    pub fn sign_ins(&self) -> Vec<(u64, u64)> {
        self.driver
            .world
            .store
            .rows
            .values()
            .filter_map(|row| {
                if let root::Record::Core(core::Record::People(people::Stored::SignIn { number, person, .. })) = row {
                    Some((*number, *person))
                } else {
                    None
                }
            })
            .collect()
    }
}
