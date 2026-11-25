//! Direct CDP replies and events for step tests of reads on an open page.
use super::{Browser, Page, Phase, document, field_text, fire, render};
use crate::boundary::Request;
use crate::{Limits, wire::decode::Document};
use skein_lib::{Env, Queue, Time, Token, Wall};

pub(super) const PAGE: Token = Token::new(3);
pub(super) const ROOT: &[u8] = br#"{"depth":0,"pierce":false}"#;

pub(super) struct Peer {
    pub(super) browser: Browser,
    pub(super) env: Env<Limits>,
}

impl Peer {
    pub(super) fn opened(limits: Limits) -> Self {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut browser = Browser::new(Token::new(1), &env.limits);
        browser.phase = Phase::Ready;
        assert!(browser.insert_page(
            Page {
                token: PAGE,
                person: Token::new(2),
                target: Some(b"target".as_slice().into()),
                session: Some(b"session".as_slice().into()),
                initial_url: b"about:blank".as_slice().into(),
                requests: false,
                files: false,
                css: render::Css::Disabled,
                frame: Some(b"main".as_slice().into()),
                document: document::Root::Empty,
                navigating: false,
                opening: false,
                load: None,
                load_due: None,
                closed: false,
            },
            env.limits.pages
        ));
        Self { browser, env }
    }

    pub(super) fn ask(&mut self, read: Request) {
        self.browser.request(&self.env, read);
    }

    pub(super) fn command(&mut self, method: &[u8], params: &[u8]) -> u64 {
        let index = self.browser.sendable().expect("next command is sendable");
        let command = self.browser.outgoing.remove(index).expect("next command");
        let bytes = command.bytes.strip_suffix(&[0]).expect("CDP delimiter");
        let document = Document::parse(bytes, self.env.limits.command).expect("encoded command");
        let root = document.root();
        assert_eq!(root.get(b"method").expect("method").text().expect("method text").as_ref(), method);
        assert_eq!(root.get(b"params").expect("parameters").raw(), params);
        assert_eq!(field_text(root, b"sessionId").as_deref(), Some(b"session".as_slice()));
        root.get(b"id").expect("id").u64().expect("integer command id")
    }

    #[expect(clippy::disallowed_macros, reason = "the synthetic reply combines an id and JSON fixtures")]
    pub(super) fn answer(&mut self, id: u64, field: &str, json: &str) {
        self.notify(format!(r#"{{"id":{id},"{field}":{json}}}"#).as_bytes());
    }

    pub(super) fn reply(&mut self, method: &[u8], params: &[u8], field: &str, json: &str) {
        let id = self.command(method, params);
        self.answer(id, field, json);
    }

    pub(super) fn notify(&mut self, json: &[u8]) {
        self.browser.message(&self.env, json);
    }

    pub(super) fn tick(&mut self) {
        let (mut above, mut below) = (Queue::uncapped(), Queue::uncapped());
        fire(&mut self.browser, &self.env, self.env.now, &mut above, &mut below);
        assert!(above.is_empty(), "no event due while restarting the read");
    }
}
