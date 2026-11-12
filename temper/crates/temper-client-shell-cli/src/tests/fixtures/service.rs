//! The fake engine returns a credential once, outside the durable answer.
use super::*;
pub const CREDENTIAL: [u8; 40] = [90; 40];
pub const FILE: &[u8] = b"http://127.0.0.1:4100\nZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ";
impl Frame {
    pub(super) fn service_reply(&mut self, fd: Fd, body: &[u8]) {
        let keyed = w::Keyed::decode(&w::CEILINGS, &mut skein_lib::Reader::new(body)).expect("keyed act");
        let ask = w::Ask::decode(&w::CEILINGS, &mut skein_lib::Reader::new(keyed.request())).expect("service act");
        let w::Ask::MakeService(make) = ask else { panic!("make service") };
        assert_eq!(make.project(), 7);
        assert_eq!(make.name(), b"scout");
        assert_eq!(make.role(), &w::HeldRole::Member);
        self.made_services += 1;
        if self.hold_service_erase {
            self.hold_stores = Some(());
        }
        let service = w::MadeService::new(
            &w::CEILINGS,
            w::MadeServiceParts {
                party: 9,
                credential: skein_lib::Secret::new(if self.service_replay {
                    Box::new([])
                } else {
                    Box::new(CREDENTIAL)
                }),
            },
        )
        .expect("service");
        let outcome =
            w::OutcomeMade::new(&w::CEILINGS, w::OutcomeMadeParts { what: w::Made::Service(service) }).expect("made");
        let answer = w::Final::Done(
            w::Done::new(&w::CEILINGS, w::DoneParts { outcome: w::Outcome::Made(outcome) }).expect("done"),
        );
        let owned = w::Owned::new(
            &w::CEILINGS,
            w::OwnedParts { owner: w::Owner::Jig, version: w::VERSION, record: encode!(answer) },
        )
        .expect("answer");
        self.reply(fd, &encode!(w::KeyedAnswer::FinalAnswer(owned)));
    }
    pub fn no_service_leaked(&self) {
        for bytes in [self.stdout(), self.stderr()].into_iter().chain(self.store_history.iter().map(Vec::as_slice)) {
            assert!(
                skein_lib::bytes::find(bytes, &CREDENTIAL).is_none(),
                "credential must stay out of output and store"
            );
        }
    }
}
