//! A cold box accepts immediately, then takes a manual-clock minute to start.
use super::*;
use temper_client_protocol_wire::{self as wire, v1 as f};

const FORGE: u16 = 3;
pub const COLD_START: Duration = Duration::from_secs(61);

impl Frame {
    pub fn setup(json: bool) -> Self {
        let mut words = vec!["forge", "setup", "3", "--project", "3", "--key", "cold"];
        if json {
            words.push("--json");
        }
        let mut frame = Self::read(&words);
        frame.answer_new_act();
        frame
    }
    pub(super) fn setup_reply(&mut self, fd: Fd, keyed: &w::Keyed) -> bool {
        if !matches!(&keyed.owner, w::Owner::Family(_)) {
            return false;
        }
        let envelope = w::FamilyRequest::decode(&w::CEILINGS, &mut skein_lib::Reader::new(keyed.request.as_ref()))
            .expect("family envelope");
        let request = wire::decode_requesting(envelope.record.as_ref(), &f::CEILINGS).expect("forge request");
        let outcome = match request.request {
            f::Request::SetUp(_) => f::Outcome::SettingUp,
            f::Request::Adopt(_) if self.wait_task.is_some() => f::Outcome::Adopted(f::Adopted {
                landing: b"main".as_slice().into(),
                forge: FORGE,
                repository: 42,
                owner: b"work".as_slice().into(),
                name: b"hello".as_slice().into(),
                role: f::Role::Owned,
                home: true,
                kinds: f::Kinds {
                    read: true,
                    push: true,
                    open: true,
                    land: true,
                    review: true,
                    status: true,
                    comment: true,
                    issue: true,
                    branch: true,
                },
                protection: f::Protection::Unknown,
            }),
            f::Request::Adopt(_)
            | f::Request::FileFinding(_)
            | f::Request::HandFinding(_)
            | f::Request::TriageFinding(_)
            | f::Request::SetDraft(_)
            | f::Request::OpenChange(_)
            | f::Request::TierAct(_)
            | f::Request::FollowAct(_)
            | f::Request::TryAccount(_)
            | f::Request::Dismiss(_)
            | f::Request::SetAway(_)
            | f::Request::Overturn(_)
            | f::Request::Looked(_)
            | f::Request::RemoveForge(_)
            | f::Request::StartPlayground
            | f::Request::EndPlayground
            | f::Request::Install(_)
            | f::Request::Replace(_)
            | f::Request::Connect(_)
            | f::Request::Accept(_)
            | f::Request::MakeRepository(_)
            | f::Request::Reachable(_)
            | f::Request::AddDevice(_)
            | f::Request::RemoveDevice(_)
            | f::Request::Checkpoint
            | f::Request::DismissMaking(_) => return false,
        };
        let answer = f::Answer { outcome };
        let document = owned(wire::answer(&answer));
        self.reply(fd, &encode!(w::KeyedAnswer::FinalAnswer(document)));
        true
    }
    pub fn finish_setup(&mut self) {
        self.deliver_setup(true);
        self.run();
    }
    pub(super) fn deliver_setup(&mut self, made: bool) {
        let mut forges = skein_lib::List::with_capacity(1);
        let first = made.then(|| f::FirstRepository {
            playground: false,
            sample: false,
            label: b"cold".as_slice().into(),
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            state: f::FirstState::Made(f::Made { id: 42 }),
        });
        forges
            .push(f::ForgeRow {
                catching_up: None,
                starts_with: f::StartsWith::Repository,
                makings: skein_lib::List::with_capacity(0),
                host: b"git.example:3000".as_slice().into(),
                number: FORGE,
                kind: f::Kind::Forgejo,
                mode: f::Mode::Owned,
                connection: if made {
                    f::Connection::Connected(f::Connected { id: 1, login: b"temper".as_slice().into() })
                } else {
                    f::Connection::SettingUp(f::SettingUp { step: Some(f::SetUpStep::Run), fetched: None })
                },
                offer: f::Offer::NotOffered,
                first,
                installations: skein_lib::List::with_capacity(0),
                ended: None,
                app: None,
                web: None,
                repositories: skein_lib::List::with_capacity(0),
                owned: None,
                admin: false,
                read_only: false,
                ended_stop: None,
            })
            .expect("valid setup fixture");
        let document =
            owned(wire::forges(&f::Forges { forges, full: skein_lib::List::with_capacity(0), linked: false }));
        let event = w::StreamEvent::Delivery(w::Delivery { watch: self.wait_task.expect("forge watch"), document });
        self.sockets.get_mut(&self.stream.expect("stream")).expect("valid setup fixture").reply.extend(sse(&event));
    }
}
fn owned(record: Box<[u8]>) -> w::Owned {
    w::Owned { owner: w::Owner::Family(w::Family { number: wire::FORGE }), version: wire::CLIENT_VERSION, record }
}
