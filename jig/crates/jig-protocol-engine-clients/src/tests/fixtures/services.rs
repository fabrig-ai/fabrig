use super::*;
pub const SERVICE: u64 = 7;
pub const REISSUED_SERVICE: u64 = 9;
// A party number and 32 random bytes (protocol/client/sign-in.md).
pub const SERVICE_CREDENTIAL_BYTES: usize = 40;
pub fn credential() -> Secret {
    let settings = settings();
    let mut rng = CryptoRng::new(&settings.seed, settings.stream).expect("fixture seed");
    let mut random = [0; 32];
    rng.fill(&mut random);
    let mut bytes = Writer::new(SERVICE_CREDENTIAL_BYTES);
    bytes.put(&SERVICE.to_be_bytes()).expect("number");
    bytes.put(&random).expect("random");
    Secret::new(bytes.finish())
}
pub fn credential_digest() -> Secret {
    Secret::new(Box::new(skein_lib::sha256::digest(credential().expose())))
}

pub fn service_kept() -> Kept {
    Kept { digest: credential_digest(), expires: Wall::from_nanos(u64::MAX) }
}
impl Exchange {
    pub fn make_service(&mut self) -> (Token, ReplyTo) {
        let (ask, expected) = make_service_case();
        self.ask_service(&ask, expected)
    }
    pub fn reissue_service(&mut self, party: u64) -> (Token, ReplyTo) {
        self.ask_service(&w::Ask::ReissueService(w::ReissueService { party }), people::Ask::ReissueService { party })
    }
    fn ask_service(&mut self, ask: &w::Ask, expected: people::Ask) -> (Token, ReplyTo) {
        self.token = sign_in_token(NUMBER);
        let owner = self.send(crate::Message::Request(keyed(
            w::Owner::Jig,
            jig_client_protocol_wire::VERSION,
            encode_ask(ask),
        )));
        self.authenticate();
        let Some(Request::Core(event)) = self.out.pop() else { panic!("core ask") };
        let core::Event::People(people::Event::Ask { reply_to, ask, .. }) = *event else { panic!("people ask") };
        assert_eq!(ask, expected);
        (owner, reply_to)
    }
    pub fn service_made(&mut self, to: ReplyTo) {
        self.reply(to, people::Reply::Outcome(people::Outcome::ServiceMade { person: SERVICE }));
        let Some(Request::Secrets(SecretsRequest::Get { owner })) = self.out.pop() else { panic!("service get") };
        assert_eq!(owner, SecretOwner::Service(SERVICE));
    }
    pub fn service_got(&mut self, value: Option<Kept>) {
        self.service_got_for(SERVICE, value);
    }
    pub fn service_got_for(&mut self, party: u64, value: Option<Kept>) {
        self.component.down(
            &self.env,
            Event::Secrets(SecretsAnswer::Got { owner: SecretOwner::Service(party), value }),
            &mut self.out,
        );
    }
    pub fn keep_service(&mut self) {
        assert_eq!(self.take_service_keep(SERVICE).digest, credential_digest());
    }
    pub fn take_service_keep(&mut self, party: u64) -> Kept {
        let Some(Request::Secrets(SecretsRequest::Keep { owner, value })) = self.out.pop() else {
            panic!("service keep")
        };
        assert_eq!(owner, SecretOwner::Service(party));
        assert_eq!(value.expires, Wall::from_nanos(u64::MAX));
        value
    }
    pub fn service_kept_reply(&mut self, party: u64) {
        self.component.down(
            &self.env,
            Event::Secrets(SecretsAnswer::Kept { owner: SecretOwner::Service(party) }),
            &mut self.out,
        );
    }
    pub fn service_reissued(&mut self, to: ReplyTo, party: u64) {
        self.reply(to, people::Reply::Outcome(people::Outcome::ServiceReissued { party }));
    }
    pub fn reissued_credential(&self, owner: Token, party: u64) -> Secret {
        let w::Made::Reissued(value) = self.made_service_answer(owner) else { panic!("reissued") };
        assert_eq!(value.party, party);
        value.credential
    }
    pub fn service_credential(&self, owner: Token) -> Secret {
        let w::Made::Service(value) = self.made_service_answer(owner) else { panic!("service") };
        assert_eq!(value.party, SERVICE);
        value.credential
    }
    fn made_service_answer(&self, owner: Token) -> w::Made {
        let w::Final::Done(value) = self.final_answer(owner) else { panic!("done") };
        let w::Outcome::Made(value) = value.outcome else { panic!("made") };
        value.what
    }
    pub fn sign_in_service(&mut self) -> (Token, ReplyTo) {
        let owner = self.get_service(SERVICE, credential().expose());
        self.service_got(Some(service_kept()));
        (owner, self.take_service_sign_in(SERVICE))
    }
    pub fn get_service(&mut self, party: u64, token: &[u8]) -> Token {
        self.token = token.into();
        let owner = self.send(crate::Message::SignInService(Box::new([])));
        let Some(Request::Secrets(SecretsRequest::Get { owner: who })) = self.out.pop() else {
            panic!("credential get")
        };
        assert_eq!(who, SecretOwner::Service(party));
        owner
    }
    pub fn take_service_sign_in(&mut self, party: u64) -> ReplyTo {
        let Some(Request::Core(event)) = self.out.pop() else { panic!("core sign-in") };
        let core::Event::SignIn { until: None, held: None, reply_to, identity } = *event else { panic!("sign-in") };
        assert_eq!(identity.key.provider, 2);
        assert_eq!(identity.key.subject.as_ref(), &party.to_be_bytes());
        assert!(identity.login.is_empty() && identity.name.is_empty());
        reply_to
    }
    pub fn sign_in_answer(&self, owner: Token) -> w::SignInAnswer {
        let (_, bytes) = self.response(owner);
        w::SignInAnswer::decode(&wire(), &mut skein_lib::Reader::new(bytes)).expect("sign-in answer")
    }
    pub fn signed_in_answer(&self, owner: Token) -> w::SignedIn {
        let w::SignInAnswer::SignedIn(value) = self.sign_in_answer(owner) else { panic!("signed in") };
        value
    }
}
