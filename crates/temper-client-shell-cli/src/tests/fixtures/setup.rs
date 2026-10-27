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
        if !matches!(keyed.owner(), w::Owner::Family(_)) {
            return false;
        }
        let envelope = w::FamilyRequest::decode(&w::CEILINGS, &mut skein_lib::Reader::new(keyed.request()))
            .expect("family envelope");
        let request = wire::decode_requesting(envelope.record(), &f::CEILINGS).expect("forge request");
        let outcome = match request.into_parts().request {
            f::Request::SetUp(_) => f::Outcome::SettingUp,
            f::Request::Adopt(_) if self.wait_task.is_some() => f::Outcome::Adopted(
                f::Adopted::new(
                    &f::CEILINGS,
                    f::AdoptedParts {
                        forge: FORGE,
                        repository: 42,
                        owner: b"work".as_slice().into(),
                        name: b"hello".as_slice().into(),
                        role: f::Role::Owned,
                        home: true,
                        kinds: f::Kinds::new(&f::CEILINGS, f::KindsParts { skein_bools: [true; 9] })
                            .expect("valid setup fixture"),
                        protection: f::Protection::Unknown,
                    },
                )
                .expect("valid setup fixture"),
            ),
            f::Request::Adopt(_)
            | f::Request::Return(_)
            | f::Request::StartDemo
            | f::Request::Install(_)
            | f::Request::Connect(_)
            | f::Request::Accept(_)
            | f::Request::Reachable(_) => return false,
        };
        let answer = f::Answer::new(&f::CEILINGS, f::AnswerParts { outcome }).expect("valid setup fixture");
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
        let first = made.then(|| {
            f::FirstRepository::new(
                &f::CEILINGS,
                f::FirstRepositoryParts {
                    playground: false,
                    label: b"cold".as_slice().into(),
                    org: b"work".as_slice().into(),
                    repository: b"hello".as_slice().into(),
                    project: f::SetUpProject::Existing(
                        f::ExistingProject::new(&f::CEILINGS, f::ExistingProjectParts { project: 3 })
                            .expect("valid setup fixture"),
                    ),
                    state: f::FirstState::Made(
                        f::Made::new(&f::CEILINGS, f::MadeParts { id: 42 }).expect("valid setup fixture"),
                    ),
                },
            )
            .expect("valid setup fixture")
        });
        forges
            .push(
                f::ForgeRow::new(
                    &f::CEILINGS,
                    f::ForgeRowParts {
                        number: FORGE,
                        kind: f::Kind::Forgejo,
                        mode: f::Mode::Owned,
                        connection: if made {
                            f::Connection::Connected(
                                f::Connected::new(
                                    &f::CEILINGS,
                                    f::ConnectedParts { id: 1, login: b"temper".as_slice().into() },
                                )
                                .expect("valid setup fixture"),
                            )
                        } else {
                            f::Connection::SettingUp(
                                f::SettingUp::new(&f::CEILINGS, f::SettingUpParts { step: Some(f::SetUpStep::Run) })
                                    .expect("valid setup fixture"),
                            )
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
                    },
                )
                .expect("valid setup fixture"),
            )
            .expect("valid setup fixture");
        let document =
            owned(wire::forges(&f::Forges::new(&f::CEILINGS, f::ForgesParts { forges }).expect("valid setup fixture")));
        let event = w::StreamEvent::Delivery(
            w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch: self.wait_task.expect("forge watch"), document })
                .expect("valid setup fixture"),
        );
        self.sockets.get_mut(&self.stream.expect("stream")).expect("valid setup fixture").reply.extend(sse(&event));
    }
}
fn owned(record: Box<[u8]>) -> w::Owned {
    w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts {
            owner: w::Owner::Family(
                w::Family::new(&w::CEILINGS, w::FamilyParts { number: wire::FORGE }).expect("valid setup fixture"),
            ),
            version: wire::CLIENT_VERSION,
            record,
        },
    )
    .expect("valid setup fixture")
}
