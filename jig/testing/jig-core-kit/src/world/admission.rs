//! Explicit admission and store pacing on the core world's loop.
use super::{ReplyTo, StoreOut, Token, World, core, fleet, people, root};
use jig_domain_link::{Login, LoginState};
impl World {
    /// Offer one input; count and drop any unadmitted input. Release is explicit.
    pub fn offer(&mut self, mut event: root::Event) {
        self.observer.inbound(&self.store, self.wall, &mut event).unwrap_or_else(|finding| self.fail(&finding));
        let env = self.env();
        if root::try_step(&mut self.domain, &env, event).is_err() {
            self.returned += 1;
        }
    }

    /// Offer one input, retaining a refusal ahead of later queued inputs.
    pub fn send_retaining(&mut self, mut event: root::Event) {
        self.observer.inbound(&self.store, self.wall, &mut event).unwrap_or_else(|finding| self.fail(&finding));
        let env = self.env();
        if let Err(event) = root::try_step(&mut self.domain, &env, event) {
            self.returned += 1;
            self.events.push_front(event);
        }
    }

    /// Release one batch without offering queued inputs or applying commits.
    pub fn collect(&mut self) {
        self.drive(1, false, false).unwrap_or_else(|finding| self.fail(&finding));
    }

    /// Apply and offer the oldest store completion without releasing outputs.
    pub fn commit_one(&mut self) {
        assert!(!self.store.pending.is_empty(), "pending commit");
        self.tick_store().unwrap_or_else(|finding| self.fail(&finding));
        let event = self.events.pop_front().expect("store terminal");
        self.send_retaining(event);
    }

    /// Write sign-ins without answering commits until admission returns a probe.
    /// The refused setup probe is dropped and does not join the story's inputs.
    pub fn fill_to_watermark(&mut self) {
        let returned = self.returned;
        for _ in 0..=self.limits.watermark {
            self.probe();
            if self.returned > returned {
                return;
            }
            self.collect();
        }
        panic!("admission returns a pressure probe within watermark + 1 writing probes");
    }

    pub fn probe(&mut self) {
        self.serial += 1;
        self.offer(sign_in(Token::new(self.serial)));
    }

    #[must_use]
    pub fn session(&self) -> u64 {
        self.sign_in.expect("durable sign-in")
    }

    #[must_use]
    pub fn assignment(&self) -> &root::Assignment {
        self.assigned.as_ref().expect("durable worker assignment")
    }
}

impl World {
    pub(super) fn tick_store(&mut self) -> Result<(), String> {
        match self.store.tick(false) {
            Ok(Some(number)) => self.events.push_front(root::Event::Committed { number, fill: self.fill }),
            Ok(None) => {}
            Err(number) => self.events.push_front(root::Event::Failed { number }),
        }
        self.observer.durable(self.wall, &self.store)?;
        Ok(())
    }
    pub(super) fn capture(&mut self, mut delivery: root::Delivery) -> Result<(), String> {
        self.observer.delivered(self.wall, &mut delivery)?;
        match delivery {
            root::Delivery::Core(core::Held::ReadLoad { load, range }) => {
                self.store.ask(StoreOut::ReadLoad { load, range });
            }
            root::Delivery::Core(core::Held::Load(load)) => self.store.ask(StoreOut::Now(*load)),
            root::Delivery::Core(core::Held::NotesLoad { owner, range }) => {
                self.store.ask(StoreOut::NotesLoad { owner, range });
            }
            root::Delivery::Core(core::Held::Sources { channel, sources }) => {
                self.events.push_back(root::Event::Core(core::Event::Fleet(fleet::Event::Logins {
                    channel,
                    logins: sources
                        .iter()
                        .map(|source| Login { name: *source, state: LoginState::Usable, email: None })
                        .collect(),
                })));
            }
            other @ (root::Delivery::ProjectMade { .. }
            | root::Delivery::AssignedJob { .. }
            | root::Delivery::Link { .. }
            | root::Delivery::CallBusy { .. }
            | root::Delivery::ApplicationDocument { .. }
            | root::Delivery::ApplicationRead { .. }
            | root::Delivery::Application { .. }
            | root::Delivery::Projection { .. }
            | root::Delivery::Message { .. }
            | root::Delivery::Start(_)
            | root::Delivery::Core(_)
            | root::Delivery::CallAnswer { .. }
            | root::Delivery::Assigned { .. }
            | root::Delivery::Fleet(_)
            | root::Delivery::System { .. }
            | root::Delivery::LostRead { .. }
            | root::Delivery::Procedure { .. }) => self.released.push(root::Request::Deliver(other)),
        }
        Ok(())
    }
}
fn sign_in(reply: Token) -> root::Event {
    root::Event::Core(core::Event::SignIn {
        held: None,
        reply_to: ReplyTo::new(reply),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
            login: b"person".as_slice().into(),
            name: b"Person".as_slice().into(),
        },
    })
}
