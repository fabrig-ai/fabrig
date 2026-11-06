//! Sign-in's visit script, paste and callback streams (skein/docs/design/oauth.md, 6.8).
use super::browser::url_parts;
use super::{Fact, Story, confidential, registration};
use crate::streams::{self, Rig};
use crate::world::Client;
use skein_fake_oauth as fake;
use skein_io as io;
use skein_lib::{Env, Queue, Token, bytes};
use skein_oauth_accounts as accounts;
use skein_world::stream::Wire;
use std::net::Ipv4Addr;

/// The same owner runs directly over seeded protocol streams and fake issuer.
#[must_use]
pub fn protocol(seed: u64, story: Story) -> (Vec<Fact>, Vec<String>) {
    // These owners' scripts run only in the simulated world (`run`).
    assert!(
        !matches!(
            story,
            Story::PasteTwice
                | Story::PasteWrong
                | Story::StateRefusals
                | Story::ListenerWrongState
                | Story::PasteExpired
                | Story::PasteCancelled
                | Story::PasteClose
        ),
        "the protocol tier has no script for {story:?}"
    );
    let client = super::client(story, false);
    let profile = client.limits;
    let registration = registration(story);
    let issuer_limits = fake::Limits {
        codes: if matches!(story, Story::TwoPorts | Story::OpenIdAgain | Story::Return(super::Return::After)) {
            2
        } else {
            1
        },
        carried: 3,
        ..crate::world::issuer_limits(&profile.client)
    };
    let mut issuer =
        fake::Issuer::new(streams::issuer_config(&registration, super::issuer::port_terms(story)), issuer_limits)
            .expect("issuer");
    for (access, refresh) in [(b'a', b"r1".as_slice()), (b'b', b"r2".as_slice())] {
        issuer
            .queue(streams::token_plan(
                200,
                vec![if story == Story::TwoPorts { b'a' } else { access }; 16].into_boxed_slice(),
                Some(bytes::copy_of(refresh)),
                (story == Story::RequiredScopes).then(|| bytes::copy_of(b"read")),
            ))
            .expect("plan");
    }
    let expected_posts = if matches!(
        story,
        Story::Public | Story::TwoPorts | Story::OpenIdAgain | Story::Return(super::Return::After)
    ) {
        2
    } else {
        u64::from(!matches!(story, Story::Cancel | Story::Abort | Story::Timeout))
    };
    let rig = Rig {
        issuer,
        endpoint: registration.token_endpoint,
        target: b"/token",
        profile,
        server_limits: streams::server_limits(&issuer_limits),
        rewires: true,
        callbacks: Some(Callbacks::new(seed, story)),
        room: true,
        expected_posts: Some(expected_posts),
        turns: 200_000,
    };
    let (client, trace, _) = streams::drive(seed, client, rig);
    (client.sign_in.expect("sign-in story").facts, trace)
}

/// Work that surrounds the shared issuer exchange in a sign-in turn.
pub(crate) struct Callbacks {
    seed: u64,
    story: Story,
    listeners: Vec<(Token, Token, u16)>,
    preferred_failed: bool,
    callback: Option<Token>,
    pub(crate) wire: Wire,
    redirect: Option<Box<[u8]>>,
    callback_count: u64,
    scripted: usize,
    callback_pending: bool,
    pasted: bool,
}

impl Callbacks {
    fn new(seed: u64, story: Story) -> Self {
        Self {
            seed,
            story,
            listeners: Vec::new(),
            preferred_failed: false,
            callback: None,
            wire: Wire::new(seed.wrapping_add(2)),
            redirect: None,
            callback_count: 0,
            scripted: usize::from(story == Story::Restart),
            callback_pending: false,
            pasted: false,
        }
    }
    #[expect(clippy::too_many_lines, reason = "the visit, paste and callback run in their protocol turn's order")]
    pub(crate) fn script(
        &mut self,
        client: &mut Client,
        env: &Env<accounts::Limits>,
        issuer: &mut fake::Issuer,
        issuer_out: &mut Queue<fake::Request>,
    ) {
        let story = self.story;
        if (self.scripted == 0 || story == Story::Restart || super::owner(client).facts.contains(&Fact::Granted(1)))
            && let Some(url) = super::owner(client).visits.get(self.scripted)
        {
            self.scripted += 1;
            if !matches!(story, Story::Cancel | Story::Abort | Story::Timeout) {
                issuer.step(
                    fake::Event::Authorize { url: skein_lib::Secret::new(url.clone()), now: env.now },
                    issuer_out,
                );
                let Some(fake::Request::Redirect { uri, state, code, client_id }) = issuer_out.pop() else {
                    panic!("authorized fake browser")
                };
                let mut location = uri.to_vec();
                location.extend_from_slice(b"?state=");
                location.extend_from_slice(state.expose());
                location.extend_from_slice(b"&code=");
                location.extend_from_slice(code.expose());
                if let Some(client_id) = client_id {
                    location.extend_from_slice(b"&client_id=");
                    location.extend_from_slice(&client_id);
                }
                self.redirect = Some(location.into_boxed_slice());
            }
        }
        if !self.pasted
            && matches!(story, Story::Paste | Story::PasteThenListener | Story::ListenerThenPaste)
            && super::paste::delivery_ready(
                story,
                super::owner(client).callback,
                self.callback_count > 0 && !self.callback_pending,
                &super::owner(client).facts,
            )
            && let Some(uri) = &self.redirect
        {
            super::paste::paste(client, env, uri.clone());
            self.pasted = true;
            if story == Story::Paste {
                self.redirect = None;
            }
        }
        if !self.callback_pending
            && self.callback.is_none()
            && self.callback_count
                < if matches!(
                    story,
                    Story::WrongPath
                        | Story::Return(super::Return::WrongPath | super::Return::After)
                        | Story::WrongHost
                        | Story::LongHead
                        | Story::TwoPorts
                        | Story::OpenIdAgain
                ) {
                    2
                } else {
                    1
                }
            && let Some(uri) = &self.redirect
        {
            if confidential(story) {
                client.ask(env, accounts::Request::Redirected { account: 0, uri: skein_lib::Secret::new(uri.clone()) });
                self.redirect = None;
            } else if let Some((identity, _, _)) =
                self.listeners.iter().find(|(_, _, port)| *port == url_parts(uri).0.port()).copied()
            {
                client.deliver(
                    env,
                    io::Event::Accepted {
                        owner: identity,
                        socket: Token::new(11),
                        peer: io::Addr::Ip((Ipv4Addr::LOCALHOST, 31235).into()),
                    },
                );
                let (_, authority, target) = url_parts(uri);
                let target = if self.callback_count == 0
                    && matches!(story, Story::WrongPath | Story::Return(super::Return::WrongPath))
                {
                    b"/wrong?state=fake&code=fake".as_slice()
                } else {
                    target
                };
                let authority = if self.callback_count == 0 && story == Story::WrongHost {
                    b"127.0.0.1:31234".as_slice()
                } else {
                    authority
                };
                let mut head = b"GET ".to_vec();
                head.extend_from_slice(target);
                if self.callback_count == 0 && story == Story::LongHead {
                    head.extend_from_slice(&vec![b'x'; 3000]);
                }
                head.extend_from_slice(b" HTTP/1.1\r\nHost: ");
                head.extend_from_slice(authority);
                head.extend_from_slice(b"\r\nConnection: close\r\n\r\n");
                self.wire = Wire::new(self.seed.wrapping_add(2 + self.callback_count));
                self.wire.write(&head);
                self.callback_count += 1;
                self.callback_pending = true;
                if matches!(story, Story::TwoPorts | Story::OpenIdAgain | Story::Return(super::Return::After)) {
                    self.redirect = None;
                }
            }
        }
    }
    pub(crate) fn listen(&mut self, client: &mut Client, env: &Env<accounts::Limits>, identity: Token, addr: io::Addr) {
        let story = self.story;
        let io::Addr::Ip(mut address) = addr else { panic!("IP listener") };
        if story == Story::VaryingTaken && address.port() == 31234 {
            self.preferred_failed = true;
            client.deliver(env, io::Event::Failed { owner: identity, error: io::Error::Other });
            client.deliver(env, io::Event::Closed { owner: identity });
        } else {
            if address.port() == 0 {
                address.set_port(41000 + u16::try_from(self.listeners.len()).expect("two listeners"));
            }
            let token = Token::new(20 + skein_lib::len_u64(self.listeners.len()));
            self.listeners.push((identity, token, address.port()));
            client.deliver(env, io::Event::Listening { owner: identity, listener: token, addr: io::Addr::Ip(address) });
        }
    }
    pub(crate) fn bind(&mut self, client: &mut Client, identity: Token, socket: Token) {
        assert_eq!(socket, Token::new(11));
        assert!(self.callback.replace(identity).is_none());
        super::owner_mut(client).callback = super::paste::Callback::Accepted;
    }
    pub(crate) fn close(&mut self, client: &mut Client, entity: Token, trace: &mut Vec<String>) -> Token {
        let story = self.story;
        if let Some(at) = self.listeners.iter().position(|(_, token, _)| *token == entity) {
            self.listeners.remove(at).0
        } else {
            assert_eq!(entity, Token::new(11));
            if !matches!(story, Story::Cancel | Story::Abort | Story::Timeout) {
                assert!(self.wire.received.starts_with(if self.callback_count == 1 && story == Story::LongHead {
                    b"HTTP/1.1 414"
                } else if self.callback_count == 1
                    && matches!(
                        story,
                        Story::WrongPath
                            | Story::WrongHost
                            | Story::Return(super::Return::WrongPath)
                            | Story::PasteThenListener
                    )
                {
                    b"HTTP/1.1 404"
                } else if super::return_location(story).is_some()
                    || (story == Story::Return(super::Return::After) && self.callback_count == 1)
                {
                    b"HTTP/1.1 303"
                } else {
                    b"HTTP/1.1 200"
                }));
            }
            trace.append(&mut self.wire.trace);
            self.callback_pending = false;
            super::owner_mut(client).callback = super::paste::Callback::Finished;
            self.callback.take().expect("callback owner")
        }
    }
    pub(crate) fn answer(&mut self, client: &mut Client, env: &Env<accounts::Limits>) {
        if (self.story != Story::PasteThenListener || self.pasted)
            && let Some(identity) = self.callback
            && let Some(up) = self.wire.answer()
        {
            client.deliver(env, io::Event::Stream { owner: identity, up });
        }
    }
    pub(crate) fn settled(&self) -> bool {
        self.listeners.is_empty() && self.callback.is_none()
    }
    pub(crate) fn check(&self, document_paths: &[Box<[u8]>]) {
        let story = self.story;
        assert_eq!(self.preferred_failed, story == Story::VaryingTaken);
        if super::again(story) {
            assert_eq!(
                document_paths.iter().map(AsRef::as_ref).collect::<Vec<&[u8]>>(),
                [b"/.well-known/openid-configuration".as_slice(), b"/keys"].repeat(2)
            );
        }
    }
}
