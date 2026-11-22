//! The fake engine returns a credential once, outside the durable answer.
use super::*;
pub const CREDENTIAL: [u8; 40] = [90; 40];
pub const FILE: &[u8] = b"http://127.0.0.1:4100\nZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ";
const ENCODED: &str = "WlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWg";
pub const LINK: &[u8] =
    concat!("http://127.0.0.1:4100/sign-in#service=", "WlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWg", "\n")
        .as_bytes();
pub const NON_LOOPBACK_LINK: &[u8] =
    concat!("http://192.0.2.1:4100/sign-in#service=", "WlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWlpaWg")
        .as_bytes();

impl Frame {
    pub fn assert_service_sign_in(&self) {
        let engine = std::net::SocketAddr::from(([127, 0, 0, 1], 4100));
        assert_eq!(self.first_connect, Some(engine));
        assert_eq!(self.requests.first().map(Box::as_ref), Some(b"/jig/v1/sign-in/deployment".as_slice()));
        assert!(self.link_bearers.first().and_then(Option::as_ref).is_some_and(|bytes| bytes.as_ref() == CREDENTIAL));
    }
    pub(super) fn service_reply(&mut self, fd: Fd, body: &[u8]) {
        let keyed = w::Keyed::decode(&w::CEILINGS, &mut skein_lib::Reader::new(body)).expect("keyed act");
        let ask =
            w::Ask::decode(&w::CEILINGS, &mut skein_lib::Reader::new(keyed.request.as_ref())).expect("service act");
        let w::Ask::MakeService(make) = ask else { panic!("make service") };
        assert_eq!(make.project, 7);
        assert_eq!(make.name.as_ref(), b"scout");
        assert_eq!((&make.role), &w::HeldRole::Member);
        self.made_services += 1;
        if self.hold_service_erase {
            self.hold_stores = Some(());
        }
        let service = w::MadeService {
            party: 9,
            credential: skein_lib::Secret::new(if self.service_replay { Box::new([]) } else { Box::new(CREDENTIAL) }),
        };
        let outcome = w::OutcomeMade { what: w::Made::Service(service) };
        let answer = w::Final::Done(w::Done { outcome: w::Outcome::Made(outcome) });
        let owned = w::Owned { owner: w::Owner::Jig, version: w::VERSION, record: encode!(answer) };
        self.reply(fd, &encode!(w::KeyedAnswer::FinalAnswer(owned)));
    }
    pub fn no_service_leaked(&self) {
        for bytes in [self.stdout(), self.stderr()].into_iter().chain(self.store_history.iter().map(Vec::as_slice)) {
            for secret in [CREDENTIAL.as_slice(), ENCODED.as_bytes()] {
                assert!(
                    skein_lib::bytes::find(bytes, secret).is_none(),
                    "credential must stay out of output and store"
                );
            }
        }
    }
}
