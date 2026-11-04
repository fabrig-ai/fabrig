//! The shared fake on the component test's thread (skein/docs/design/fake-forge.md, 10.6).
use super::super::fixtures as f;
use alloc::boxed::Box;
use skein_fake_forge_domain::{self as domain, api as a};
use skein_fake_forge_github::{
    fixture,
    inline::{self, Inline},
    installations, pages, service,
};
use skein_lib::{Env, List, ReplyTo, Secret, Time, Token, Wall, Writer};

pub(super) struct Fake {
    inline: Inline,
    env: Env<domain::Config>,
    pub(super) first: u64,
    pub(super) head: u64,
    pub(super) base: u64,
    pub(super) number: u64,
    pub(super) token: Secret,
}
impl Fake {
    pub fn new(content: &[u8], divergent: bool) -> Self {
        Self::with_files(content, divergent, 1)
    }
    pub(super) fn with_files(content: &[u8], divergent: bool, files: u32) -> Self {
        let mut config = fixture::domain_config();
        config.limits.files = files.max(4);
        config.limits.page_size = config.limits.page_size.max(files);
        config.limits.content_bytes = 8192;
        let mut domain = domain::Domain::new(&config, 5);
        let first = domain::repository(&mut domain, &config, fixture::setup(1, fixture::PATH));
        domain::grant(&mut domain, fixture::PATH, fixture::BOT, a::Permission::Write);
        let (mut service, limits) = github_service(files.max(2));
        fixture::install_one(&mut service, Wall::EPOCH);
        let (token, _) =
            service.installations_mut().mint(1, installations::Scope::Full, Wall::EPOCH).expect("fake token");
        let mut tree = List::with_capacity(files);
        for index in 0..files {
            let path = if files == 1 {
                Box::from(b"file".as_slice())
            } else {
                let index = skein_lib::Decimal::of(u64::from(index));
                let mut path = Writer::new(4_usize.checked_add(index.as_bytes().len()).expect("path length"));
                path.put(b"file").expect("prefix");
                path.put(index.as_bytes()).expect("index");
                path.finish()
            };
            tree.push(a::File { path, content: Box::from(content) }).expect("file count");
        }
        let head = domain::commit(&mut domain, &config, first, tree.into_boxed(), b"feature")
            .expect("commit room")
            .expect("parent exists");
        let mut fake = Self {
            inline: Inline::new(service, domain, config, limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config },
            first,
            head,
            base: first,
            number: 0,
            token,
        };
        fake.call(a::Op::Git(a::Git::Push { branch: Box::from(b"feature".as_slice()), commit: head, expected: None }));
        let a::Answer::Created(number) = fake.call(a::Op::Write(a::Write::OpenPull {
            title: Box::from(b"feature".as_slice()),
            body: Box::from([]),
            head: Box::from(b"feature".as_slice()),
            base: Box::from(b"main".as_slice()),
            opens_empty: false,
        })) else {
            unreachable!("created pull")
        };
        fake.number = number;
        if divergent {
            fake.base = domain::commit(
                &mut fake.inline.domain,
                &config,
                first,
                Box::new([a::File {
                    path: Box::from(b"file".as_slice()),
                    content: Box::from(b"base only".as_slice()),
                }]),
                b"base",
            )
            .expect("commit room")
            .expect("parent exists");
            fake.call(a::Op::Git(a::Git::Push {
                branch: Box::from(b"main".as_slice()),
                commit: fake.base,
                expected: None,
            }));
        }
        fake
    }
    fn call(&mut self, op: a::Op) -> a::Answer {
        let event = domain::Event::Call {
            reply_to: ReplyTo::new(Token::new(77)),
            user: fixture::BOT,
            repository: Box::from(fixture::PATH),
            op,
        };
        inline::call(&mut self.inline.domain, &self.env, event).expect("scenario call")
    }
    pub fn answer(&mut self, wire: &[u8]) -> Box<[u8]> {
        // Every operation here sends GETs with empty request bodies.
        let mut previous = 0;
        let mut head = None;
        for (at, part) in wire.windows(4).enumerate() {
            if part == b"\r\n\r\n" {
                let end = at.checked_add(4).expect("head end");
                head = wire.get(previous..end);
                previous = end;
            }
        }
        self.inline.serve(head.expect("complete request"), Wall::EPOCH)
    }
}

pub(super) fn object(value: u64) -> skein_forge::api::Commit {
    let mut commit = [0; 32];
    for (out, byte) in commit.iter_mut().zip(domain::object_bytes(value)) {
        *out = byte;
    }
    commit
}

fn github_service(page: u32) -> (service::Service, service::Limits) {
    let limits = service::Limits {
        http: skein_http::server::Limits { body: u64::from(f::BODY_ROOM), ..fixture::limits().http },
        github: crate::limits::documents(&f::limits()),
        installations: installations::Limits { installations: 1, tokens: 2, listed: 1 },
        pages: pages::Limits { states: 1, manifest_bytes: 2048 },
        page,
        body: f::BODY_ROOM,
        ..fixture::limits()
    };
    let service = service::Service::new(fixture::config(fixture::widgets(1)), &limits, 5).expect("fake config");
    (service, limits)
}
